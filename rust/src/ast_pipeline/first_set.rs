//! RGX-0078.5.c.2 — shared FIRST-set analysis over the normalized gen-AST.
//!
//! Computes a **sound OVER-APPROXIMATION** of the terminal literals that can BEGIN a
//! match for a grammar node/branch/rule, plus `nullable` and `unresolved` flags.
//!
//! This logic was extracted VERBATIM from `annotation_validator` (where it backs the
//! `W_GRAM_FIRST_SET_OVERLAP` / `W_GRAM_NULLABLE_BRANCH_SHADOW` lints). The four
//! functions are PURE — they never read validator config — so the move is
//! behavior-preserving, and now BOTH consumers compute FIRST sets from the SAME
//! algorithm on the SAME normalized (post-LR-elimination) `grammar_tree`:
//!   1. the annotation validator's ambiguity/nullable lints, and
//!   2. codegen's first-set predictive-dispatch prune guard in
//!      `AstBasedGenerator::generate_or_logic` (RGX-0078.5.c.2).
//!
//! # Soundness contract
//! `unresolved: true` is the conservative escape hatch — an underivable regex token,
//! an unknown token type, an undefined non-builtin rule reference, or a cycle/depth
//! cutoff all set it. A consumer that PRUNES on this summary (skips a branch that
//! cannot start at the next input byte) MUST treat both `unresolved` and `nullable`
//! as "never prune": the byte set `terminals`-first-bytes ∪ `first_bytes` is only
//! guaranteed exhaustive when `!unresolved && !nullable`. RGX-0078.5.i.7 D0 adds two
//! more consumer obligations:
//! - `regex_token_derived: true` bytes are sound ONLY when the grammar's regex tokens
//!   are whitespace-SENSITIVE (`match_regex` is anchored — PARSE-TERMINATION.7.1 —
//!   so the match's first byte IS the byte at the parse position, but a leading
//!   layout skip would break that identity). A consumer without that guarantee must
//!   treat the whole summary as unresolved (never fall back to the non-regex bytes
//!   alone — that would under-approximate).
//! - `first_bytes` must be UNIONED with the `terminals` first bytes; neither carrier
//!   alone is the FIRST set.

use super::{ASTNode, ASTValue, TokenValue};
use regex::Regex;
use std::collections::{BTreeSet, HashMap, HashSet};

/// A sound over-approximation of a node's FIRST set.
#[derive(Debug, Clone, Default)]
pub(crate) struct FirstSetSummary {
    /// Quoted terminal literals (`'X..'`) that can begin a match. A SUPERSET — every
    /// terminal that could start here is present, but the set may over-include.
    pub(crate) terminals: HashSet<String>,
    /// RGX-0078.5.i.7 D0 — first BYTES contributed by NON-quoted origins: regex-token
    /// terminals with a derivable prefix (via the pattern's HIR) and the native
    /// builtin matchers (`builtin_any_char` / `builtin_ascii_char`). Disjoint carrier
    /// from `terminals` (which the validator's string-keyed lints consume unchanged);
    /// byte-level consumers use the union of both.
    pub(crate) first_bytes: BTreeSet<u8>,
    /// Any of `first_bytes` came from a `/regex/` terminal. Such bytes are sound for
    /// prune/dispatch ONLY when regex tokens do not skip leading layout (see the
    /// module soundness contract).
    pub(crate) regex_token_derived: bool,
    /// The node can match the empty string ⇒ it is never prunable on the next char.
    pub(crate) nullable: bool,
    /// FIRST analysis was incomplete ⇒ the byte set is NOT exhaustive ⇒ never prune.
    pub(crate) unresolved: bool,
    /// Match success at any position is EXACTLY `next byte ∈ (terminals' first bytes
    /// ∪ first_bytes)` — the set over-includes nothing and membership implies a
    /// definite match. This is the license for exact negative-lookahead subtraction
    /// (`!X rest` ⇒ FIRST(rest) − bytes(X)); an over-approximated set must NEVER be
    /// subtracted. True only for single-byte-decided shapes: `builtin_ascii_char` /
    /// `builtin_any_char`, a single-byte quoted terminal, a single ASCII-only regex
    /// class or single-byte regex literal, and unions / `min==1` repetitions thereof.
    pub(crate) byte_decided: bool,
}

const MAX_FIRST_SET_DEPTH: usize = 24;

/// FIRST set of an arbitrary grammar node (branch body / rule body / sub-expression).
pub(crate) fn branch_first_set(
    node: &ASTNode,
    grammar_tree: &HashMap<String, ASTNode>,
    first_set_cache: &mut HashMap<String, FirstSetSummary>,
    visiting_rules: &mut HashSet<String>,
    depth: usize,
) -> FirstSetSummary {
    if depth > MAX_FIRST_SET_DEPTH {
        return FirstSetSummary {
            unresolved: true,
            ..FirstSetSummary::default()
        };
    }

    match node {
        ASTNode::Sequence { elements } => {
            // Single-element pass-through: exactly the loop's result, but preserves
            // `byte_decided` / `first_bytes` for wrapper shapes (D0).
            if elements.len() == 1 {
                return branch_first_set(
                    &elements[0],
                    grammar_tree,
                    first_set_cache,
                    visiting_rules,
                    depth + 1,
                );
            }

            let mut result = FirstSetSummary {
                nullable: true,
                ..FirstSetSummary::default()
            };

            if elements.is_empty() {
                return result;
            }

            // D0 — exact negative-lookahead subtraction. Every FIRST contribution of
            // a sequence occurs AT the sequence's start position, and `!X` with X
            // byte-decided fails there exactly on X's byte set, so every byte X
            // definitely matches is EXCLUDED from the later contributions. Only an
            // EXACT set may be subtracted (see `byte_decided`); anything weaker falls
            // through to the generic over-approximating union below.
            let mut excluded: BTreeSet<u8> = BTreeSet::new();

            for element in elements {
                if let ASTNode::Lookahead {
                    element: inner,
                    positive: false,
                } = element
                {
                    let inner_first = branch_first_set(
                        inner,
                        grammar_tree,
                        first_set_cache,
                        visiting_rules,
                        depth + 1,
                    );
                    if inner_first.byte_decided
                        && !inner_first.unresolved
                        && !inner_first.nullable
                    {
                        for terminal in &inner_first.terminals {
                            if let Some(byte) = terminal_first_byte(terminal) {
                                excluded.insert(byte);
                            }
                        }
                        excluded.extend(inner_first.first_bytes.iter().copied());
                        // The lookahead's own terminals still union in (they are the
                        // pre-D0 over-approximation the validator lints consume);
                        // only its junk `first_bytes` are withheld. Zero-width:
                        // consumes nothing, so the sequence stays nullable so far.
                        result.terminals.extend(inner_first.terminals.iter().cloned());
                        continue;
                    }
                    // No exact license: fall through to the generic union, matching
                    // the pre-D0 behavior for this element.
                }

                let element_first = branch_first_set(
                    element,
                    grammar_tree,
                    first_set_cache,
                    visiting_rules,
                    depth + 1,
                );
                result
                    .terminals
                    .extend(element_first.terminals.iter().cloned());
                result.first_bytes.extend(
                    element_first
                        .first_bytes
                        .iter()
                        .copied()
                        .filter(|byte| !excluded.contains(byte)),
                );
                result.regex_token_derived |= element_first.regex_token_derived;
                result.unresolved |= element_first.unresolved;
                if !element_first.nullable {
                    result.nullable = false;
                    return result;
                }
            }

            result
        }
        ASTNode::Or { alternatives } => {
            let mut result = FirstSetSummary::default();

            if alternatives.is_empty() {
                result.nullable = true;
                return result;
            }

            let mut all_byte_decided = true;
            for alternative in alternatives {
                let alternative_first = branch_first_set(
                    alternative,
                    grammar_tree,
                    first_set_cache,
                    visiting_rules,
                    depth + 1,
                );
                result
                    .terminals
                    .extend(alternative_first.terminals.iter().cloned());
                result
                    .first_bytes
                    .extend(alternative_first.first_bytes.iter().copied());
                result.regex_token_derived |= alternative_first.regex_token_derived;
                result.nullable |= alternative_first.nullable;
                result.unresolved |= alternative_first.unresolved;
                all_byte_decided &= alternative_first.byte_decided;
            }

            // A union of exact byte-decided sets is itself exact: membership names
            // some alternative that definitely matches, and non-membership fails all.
            result.byte_decided = all_byte_decided && !result.nullable && !result.unresolved;
            result
        }
        ASTNode::Atom { value } => atom_first_set(
            value,
            grammar_tree,
            first_set_cache,
            visiting_rules,
            depth + 1,
        ),
        ASTNode::Quantified {
            element,
            quantifier,
        } => {
            let mut element_first = branch_first_set(
                element,
                grammar_tree,
                first_set_cache,
                visiting_rules,
                depth + 1,
            );
            let min_repeat = quantifier_min_repeat(quantifier);
            if min_repeat == 0 {
                element_first.nullable = true;
            }
            // `X{1,…}` succeeds iff the FIRST repetition succeeds, so byte-decidedness
            // carries through exactly at min==1; any other bound loses the license.
            element_first.byte_decided =
                min_repeat == 1 && element_first.byte_decided && !element_first.nullable;
            element_first
        }
        ASTNode::Lookahead { element, .. } => {
            let mut element_first = branch_first_set(
                element,
                grammar_tree,
                first_set_cache,
                visiting_rules,
                depth + 1,
            );
            element_first.nullable = true;
            element_first.byte_decided = false;
            element_first
        }
    }
}

fn atom_first_set(
    value: &ASTValue,
    grammar_tree: &HashMap<String, ASTNode>,
    first_set_cache: &mut HashMap<String, FirstSetSummary>,
    visiting_rules: &mut HashSet<String>,
    depth: usize,
) -> FirstSetSummary {
    match value {
        ASTValue::Node(node) => branch_first_set(
            node,
            grammar_tree,
            first_set_cache,
            visiting_rules,
            depth + 1,
        ),
        ASTValue::Token(parts) => {
            if parts.len() < 2 {
                return FirstSetSummary {
                    unresolved: true,
                    ..FirstSetSummary::default()
                };
            }

            let token_type = match &parts[0] {
                TokenValue::String(token_type) => token_type.as_str(),
            };
            let token_value = match &parts[1] {
                TokenValue::String(token_value) => token_value.as_str(),
            };

            match token_type {
                "quoted_string" => {
                    let mut terminals = HashSet::new();
                    if !token_value.is_empty() {
                        terminals.insert(format!("'{}'", token_value));
                    }
                    FirstSetSummary {
                        terminals,
                        nullable: token_value.is_empty(),
                        // A single-BYTE terminal is matched or refuted by the next
                        // byte alone — the exact-set license (D0).
                        byte_decided: token_value.len() == 1,
                        ..FirstSetSummary::default()
                    }
                }
                "rule_reference" => rule_first_set(
                    token_value,
                    grammar_tree,
                    first_set_cache,
                    visiting_rules,
                    depth + 1,
                ),
                "regex" => {
                    // The exact whole-pattern emptiness probe (pre-D0 behavior kept
                    // as belt-and-braces alongside the structural nullability below,
                    // which additionally over-approximates position-dependent
                    // zero-width matches like a leading `\b` — `find("")` alone
                    // UNDER-approximates those, which was harmless only while the
                    // summary stayed `unresolved`).
                    let matches_empty = Regex::new(token_value)
                        .ok()
                        .and_then(|re| re.find(""))
                        .map(|m| m.start() == 0 && m.end() == 0)
                        .unwrap_or(false);
                    match regex_token_prefix(token_value) {
                        Some(prefix) => {
                            let nullable = prefix.nullable || matches_empty;
                            FirstSetSummary {
                                terminals: HashSet::new(),
                                first_bytes: prefix.bytes,
                                regex_token_derived: true,
                                nullable,
                                unresolved: false,
                                byte_decided: prefix.byte_decided && !nullable,
                            }
                        }
                        None => FirstSetSummary {
                            nullable: matches_empty,
                            unresolved: true,
                            ..FirstSetSummary::default()
                        },
                    }
                }
                _ => FirstSetSummary {
                    unresolved: true,
                    ..FirstSetSummary::default()
                },
            }
        }
    }
}

fn rule_first_set(
    rule_name: &str,
    grammar_tree: &HashMap<String, ASTNode>,
    first_set_cache: &mut HashMap<String, FirstSetSummary>,
    visiting_rules: &mut HashSet<String>,
    depth: usize,
) -> FirstSetSummary {
    if let Some(cached) = first_set_cache.get(rule_name) {
        return cached.clone();
    }

    if !visiting_rules.insert(rule_name.to_string()) {
        return FirstSetSummary {
            unresolved: true,
            ..FirstSetSummary::default()
        };
    }

    let result = if let Some(rule_ast) = grammar_tree.get(rule_name) {
        branch_first_set(
            rule_ast,
            grammar_tree,
            first_set_cache,
            visiting_rules,
            depth + 1,
        )
    } else if let Some(builtin) = native_builtin_first_set(rule_name) {
        // D0 — a reference undefined in the grammar tree resolves to codegen's
        // native builtin matcher (`generate_unresolved_reference_method`); the
        // grammar-definition-wins precedence above mirrors codegen's
        // `known_rules` filter exactly.
        builtin
    } else {
        FirstSetSummary {
            unresolved: true,
            ..FirstSetSummary::default()
        }
    };

    visiting_rules.remove(rule_name);
    first_set_cache.insert(rule_name.to_string(), result.clone());
    result
}

/// RGX-0078.5.i.7 D0 — the FIRST summary of codegen's NATIVE builtin matchers
/// (`ast_based_generator::generate_unresolved_reference_method`, locked to
/// `NATIVE_UNRESOLVED_REFERENCE_BUILTINS`). Each entry mirrors the emitted matcher's
/// exact semantics:
/// - `builtin_ascii_char` consumes one char iff `ch.is_ascii()` — success is decided
///   by the next byte alone (`0x00–0x7F`), never skips layout ⇒ byte-decided.
/// - `builtin_any_char` consumes one Unicode scalar — at a char boundary of a valid
///   `&str` the next byte is always a UTF-8 lead byte (`0x00–0x7F` / `0xC2–0xF4`) and
///   the match succeeds, never skips layout ⇒ byte-decided.
/// - `true` / `false` are zero-width ALWAYS-SUCCEED fallbacks ⇒ nullable, resolved.
/// - `semantic_annotation` is NOT listed: its native matcher skips leading layout
///   before requiring `@`, so first-byte peeking at the parse position is unsound —
///   it stays `unresolved` (the caller's fallback).
fn native_builtin_first_set(rule_name: &str) -> Option<FirstSetSummary> {
    match rule_name {
        "builtin_ascii_char" => Some(FirstSetSummary {
            first_bytes: (0x00u8..=0x7F).collect(),
            byte_decided: true,
            ..FirstSetSummary::default()
        }),
        "builtin_any_char" => Some(FirstSetSummary {
            first_bytes: (0x00u8..=0x7F).chain(0xC2u8..=0xF4).collect(),
            byte_decided: true,
            ..FirstSetSummary::default()
        }),
        "true" | "false" => Some(FirstSetSummary {
            nullable: true,
            ..FirstSetSummary::default()
        }),
        _ => None,
    }
}

/// RGX-0078.5.i.7 D0 — a regex token's derived prefix facts.
struct RegexTokenPrefix {
    /// Sound over-approximation of the bytes a match can begin with.
    bytes: BTreeSet<u8>,
    /// Structural nullability (over-approximates: zero-width `Look`s count as
    /// possibly-empty, so a position-dependent `\b` never masks emptiness).
    nullable: bool,
    /// The exact-set license (see `FirstSetSummary::byte_decided`).
    byte_decided: bool,
}

/// Derive a regex token's first-byte prefix from its HIR — the SAME dialect the
/// emitted `match_regex` compiles with `regex::Regex::new` (and the runtime match is
/// ANCHORED, `\A(?:…)` — PARSE-TERMINATION.7.1 — so the match's first byte IS the
/// byte at the parse position). Returns `None` when the pattern does not parse
/// (the caller falls back to `unresolved`).
fn regex_token_prefix(pattern: &str) -> Option<RegexTokenPrefix> {
    let hir = regex_syntax::Parser::new().parse(pattern).ok()?;
    regex_hir_prefix(&hir)
}

fn regex_hir_prefix(hir: &regex_syntax::hir::Hir) -> Option<RegexTokenPrefix> {
    use regex_syntax::hir::{Class, HirKind};
    match hir.kind() {
        HirKind::Empty => Some(RegexTokenPrefix {
            bytes: BTreeSet::new(),
            nullable: true,
            byte_decided: false,
        }),
        HirKind::Literal(literal) => match literal.0.first() {
            None => Some(RegexTokenPrefix {
                bytes: BTreeSet::new(),
                nullable: true,
                byte_decided: false,
            }),
            Some(&first) => Some(RegexTokenPrefix {
                bytes: BTreeSet::from([first]),
                nullable: false,
                // A one-BYTE literal is matched or refuted by the next byte alone.
                byte_decided: literal.0.len() == 1,
            }),
        },
        HirKind::Class(class) => {
            let mut bytes = BTreeSet::new();
            let byte_decided = match class {
                Class::Unicode(ranges) => {
                    let mut ascii_only = true;
                    for range in ranges.ranges() {
                        let (start, end) = (range.start() as u32, range.end() as u32);
                        if end > 0x7F {
                            // A multi-byte char's membership needs its continuation
                            // bytes — the lead byte alone no longer DECIDES success.
                            ascii_only = false;
                        }
                        extend_lead_bytes_for_scalar_range(&mut bytes, start, end);
                    }
                    ascii_only
                }
                Class::Bytes(ranges) => {
                    for range in ranges.ranges() {
                        for byte in range.start()..=range.end() {
                            bytes.insert(byte);
                        }
                    }
                    // A byte class consumes exactly one byte: in-set ⟺ match.
                    true
                }
            };
            let byte_decided = byte_decided && !bytes.is_empty();
            Some(RegexTokenPrefix {
                bytes,
                nullable: false,
                byte_decided,
            })
        }
        // Zero-width assertion: prefix-transparent, possibly-empty (position-
        // dependent — over-approximated as nullable, the conservative direction).
        HirKind::Look(_) => Some(RegexTokenPrefix {
            bytes: BTreeSet::new(),
            nullable: true,
            byte_decided: false,
        }),
        HirKind::Repetition(repetition) => {
            let sub = regex_hir_prefix(&repetition.sub)?;
            let nullable = repetition.min == 0 || sub.nullable;
            Some(RegexTokenPrefix {
                bytes: sub.bytes,
                nullable,
                // `X{1,…}` succeeds iff the first repetition succeeds.
                byte_decided: repetition.min == 1 && sub.byte_decided && !nullable,
            })
        }
        HirKind::Capture(capture) => regex_hir_prefix(&capture.sub),
        HirKind::Concat(parts) => {
            if parts.len() == 1 {
                return regex_hir_prefix(&parts[0]);
            }
            let mut bytes = BTreeSet::new();
            let mut nullable = true;
            for part in parts {
                let part_prefix = regex_hir_prefix(part)?;
                bytes.extend(part_prefix.bytes.iter().copied());
                if !part_prefix.nullable {
                    nullable = false;
                    break;
                }
            }
            Some(RegexTokenPrefix {
                bytes,
                nullable,
                byte_decided: false,
            })
        }
        HirKind::Alternation(alternatives) => {
            let mut bytes = BTreeSet::new();
            let mut nullable = false;
            let mut all_byte_decided = true;
            for alternative in alternatives {
                let alt_prefix = regex_hir_prefix(alternative)?;
                bytes.extend(alt_prefix.bytes.iter().copied());
                nullable |= alt_prefix.nullable;
                all_byte_decided &= alt_prefix.byte_decided;
            }
            Some(RegexTokenPrefix {
                bytes,
                nullable,
                byte_decided: all_byte_decided && !nullable && !alternatives.is_empty(),
            })
        }
    }
}

/// The UTF-8 LEAD bytes of every scalar in `[start, end]` (both inclusive, valid
/// Unicode scalar values). Within each encoded-length band the lead byte is monotonic
/// in the code point, so each band contributes one contiguous lead-byte range.
fn extend_lead_bytes_for_scalar_range(out: &mut BTreeSet<u8>, start: u32, end: u32) {
    const BANDS: [(u32, u32); 4] = [
        (0x0000, 0x007F),
        (0x0080, 0x07FF),
        (0x0800, 0xFFFF),
        (0x1_0000, 0x10_FFFF),
    ];
    for (band_lo, band_hi) in BANDS {
        let lo = start.max(band_lo);
        let hi = end.min(band_hi);
        if lo > hi {
            continue;
        }
        for byte in utf8_lead_byte(lo)..=utf8_lead_byte(hi) {
            out.insert(byte);
        }
    }
}

fn utf8_lead_byte(scalar: u32) -> u8 {
    match scalar {
        0..=0x7F => scalar as u8,
        0x80..=0x7FF => 0xC0 | (scalar >> 6) as u8,
        0x800..=0xFFFF => 0xE0 | (scalar >> 12) as u8,
        _ => 0xF0 | (scalar >> 18) as u8,
    }
}

fn quantifier_min_repeat(quantifier: &str) -> usize {
    let trimmed = quantifier.trim();
    match trimmed {
        "?" | "*" => 0,
        "+" => 1,
        _ if trimmed.starts_with('{') && trimmed.ends_with('}') => {
            let inner = trimmed[1..trimmed.len() - 1].trim();
            if inner.is_empty() || inner.starts_with(',') {
                return 0;
            }
            let min_part = inner.split(',').next().unwrap_or(inner).trim();
            min_part.parse::<usize>().unwrap_or(1)
        }
        _ => 1,
    }
}

/// RGX-0078.5.i.3 (P2) — the branch's admissible DISPATCH first bytes (sorted), or the
/// NAMED reason the branch is not first-byte-decided.
///
/// This is the SHARED eligibility predicate behind BOTH consumers — the degeneracy
/// census (`fusibility_census.rs`) and codegen's degenerate-dispatch gate
/// (`generate_or_logic`) — so the census verdict and the emitted dispatch can never
/// drift: a branch is first-byte-decided iff its FIRST summary is resolved +
/// non-nullable + non-empty and EVERY FIRST terminal yields an extractable first byte
/// (exactly the `.5.c.2` prune-guard eligibility), unioned with the D0
/// regex-token/builtin `first_bytes`. `Err` = "always try this branch" — a site
/// containing such a branch can never dispatch degenerately.
///
/// `trust_regex_token_bytes` — the D0 layout gate: pass the grammar's
/// `layout_sensitivity().regex_tokens` (codegen) / compiled `layout.regex_tokens`
/// (census). A summary carrying regex-token-derived bytes under a layout-SKIPPING
/// regex-token policy is treated as unresolved outright (never reduced to its
/// non-regex bytes — that would under-approximate the FIRST set).
pub(crate) fn branch_dispatch_first_bytes(
    branch: &ASTNode,
    grammar_tree: &HashMap<String, ASTNode>,
    first_set_cache: &mut HashMap<String, FirstSetSummary>,
    trust_regex_token_bytes: bool,
) -> Result<Vec<u8>, String> {
    let mut visiting_rules = HashSet::new();
    let summary = branch_first_set(branch, grammar_tree, first_set_cache, &mut visiting_rules, 0);
    if summary.nullable {
        return Err("nullable (can match empty)".to_string());
    }
    if summary.unresolved {
        return Err("unresolved FIRST set (regex token / cycle / depth cutoff)".to_string());
    }
    if summary.regex_token_derived && !trust_regex_token_bytes {
        return Err(
            "regex-token-derived first bytes under layout-skipping regex tokens".to_string(),
        );
    }
    if summary.terminals.is_empty() && summary.first_bytes.is_empty() {
        return Err("empty FIRST terminal set".to_string());
    }
    let mut bytes: std::collections::BTreeSet<u8> = summary.first_bytes.clone();
    for terminal in &summary.terminals {
        match terminal_first_byte(terminal) {
            Some(byte) => {
                bytes.insert(byte);
            }
            None => {
                return Err(format!("unextractable first byte for terminal {terminal}"));
            }
        }
    }
    Ok(bytes.into_iter().collect())
}

/// RGX-0078.5.c.2 — the first BYTE of a quoted terminal literal.
///
/// Terminals are stored as `format!("'{}'", value)`, so `'X..'` → the first byte of
/// `X` (index 1, since index 0 is the opening `'`). This is exactly the byte the
/// generated `match_string` compares first at the branch's start position, so a
/// non-nullable branch whose every FIRST terminal starts with a byte different from
/// the next input byte would fail at char 1.
///
/// Returns `None` when the string is not a well-formed quoted terminal (no leading
/// `'`), in which case a prune consumer MUST fall back to "always try" (never prune).
/// Empty-value terminals (`''`) never appear in a `FirstSetSummary` (an empty
/// `quoted_string` is recorded as `nullable`, not as a terminal), so a real terminal
/// always has a value byte at index 1 — including the single-quote terminal `'''`
/// (value `'`), whose first byte is correctly `0x27`.
pub(crate) fn terminal_first_byte(quoted_terminal: &str) -> Option<u8> {
    quoted_terminal
        .strip_prefix('\'')
        .and_then(|rest| rest.as_bytes().first().copied())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn regex_atom(pattern: &str) -> ASTNode {
        ASTNode::Atom {
            value: ASTValue::Token(vec![
                TokenValue::String("regex".to_string()),
                TokenValue::String(pattern.to_string()),
            ]),
        }
    }

    fn quoted_atom(value: &str) -> ASTNode {
        ASTNode::Atom {
            value: ASTValue::Token(vec![
                TokenValue::String("quoted_string".to_string()),
                TokenValue::String(value.to_string()),
            ]),
        }
    }

    fn rule_ref(name: &str) -> ASTNode {
        ASTNode::Atom {
            value: ASTValue::Token(vec![
                TokenValue::String("rule_reference".to_string()),
                TokenValue::String(name.to_string()),
            ]),
        }
    }

    fn summarize(node: &ASTNode, tree: &HashMap<String, ASTNode>) -> FirstSetSummary {
        let mut cache = HashMap::new();
        let mut visiting = HashSet::new();
        branch_first_set(node, tree, &mut cache, &mut visiting, 0)
    }

    #[test]
    fn ascii_class_regex_resolves_byte_decided() {
        let summary = summarize(&regex_atom("[a-z]"), &HashMap::new());
        assert!(!summary.unresolved);
        assert!(!summary.nullable);
        assert!(summary.regex_token_derived);
        assert!(summary.byte_decided);
        assert_eq!(
            summary.first_bytes,
            (b'a'..=b'z').collect::<BTreeSet<u8>>()
        );
    }

    #[test]
    fn negated_class_regex_excludes_metachars_and_includes_lead_bytes() {
        let summary = summarize(&regex_atom(r"[^\\^$.|?*+()\[{]"), &HashMap::new());
        assert!(!summary.unresolved);
        assert!(!summary.nullable);
        // The negated-class complement contains multi-byte scalars, whose lead
        // byte alone cannot DECIDE membership.
        assert!(!summary.byte_decided);
        for meta in [b'\\', b'^', b'$', b'.', b'|', b'?', b'*', b'+', b'(', b')', b'[', b'{'] {
            assert!(!summary.first_bytes.contains(&meta), "meta {meta:#04x} leaked");
        }
        assert!(summary.first_bytes.contains(&b'a'));
        assert!(summary.first_bytes.contains(&0xC2));
        assert!(summary.first_bytes.contains(&0xF4));
        // Continuation bytes are never lead bytes.
        assert!(!summary.first_bytes.contains(&0x80));
        assert!(!summary.first_bytes.contains(&0xC1));
    }

    #[test]
    fn multibyte_band_class_maps_to_lead_byte_range() {
        let summary = summarize(&regex_atom("[\u{80}-\u{7FF}]"), &HashMap::new());
        assert_eq!(
            summary.first_bytes,
            (0xC2u8..=0xDF).collect::<BTreeSet<u8>>()
        );
        let summary = summarize(&regex_atom("\u{E9}"), &HashMap::new());
        assert_eq!(summary.first_bytes, BTreeSet::from([0xC3u8]));
        // A 2-byte literal is not decided by its first byte.
        assert!(!summary.byte_decided);
    }

    #[test]
    fn multichar_literal_contributes_first_byte_only_not_decided() {
        let summary = summarize(&regex_atom("abc"), &HashMap::new());
        assert_eq!(summary.first_bytes, BTreeSet::from([b'a']));
        assert!(!summary.byte_decided);
        assert!(!summary.nullable);
    }

    #[test]
    fn nullable_regex_is_flagged_nullable() {
        let summary = summarize(&regex_atom("(?:abc)?"), &HashMap::new());
        assert!(!summary.unresolved);
        assert!(summary.nullable);
    }

    #[test]
    fn leading_word_boundary_is_conservatively_nullable() {
        // `find("")` says `\b` cannot match empty (no boundary in ""), but at a
        // mid-input boundary it matches zero-width — the structural walk must keep
        // it nullable so no consumer ever prunes on the empty byte set.
        let summary = summarize(&regex_atom(r"\b"), &HashMap::new());
        assert!(!summary.unresolved);
        assert!(summary.nullable);
        // A boundary-then-literal pattern is anchored on the literal's first byte.
        let summary = summarize(&regex_atom(r"\bmodule\b"), &HashMap::new());
        assert!(!summary.nullable);
        assert_eq!(summary.first_bytes, BTreeSet::from([b'm']));
    }

    #[test]
    fn builtin_references_resolve_like_codegen_native_matchers() {
        let tree = HashMap::new();
        let ascii = summarize(&rule_ref("builtin_ascii_char"), &tree);
        assert!(!ascii.unresolved && !ascii.nullable && ascii.byte_decided);
        assert_eq!(ascii.first_bytes, (0x00u8..=0x7F).collect::<BTreeSet<u8>>());

        let any = summarize(&rule_ref("builtin_any_char"), &tree);
        assert!(!any.unresolved && !any.nullable && any.byte_decided);
        assert!(any.first_bytes.contains(&0x00) && any.first_bytes.contains(&0xF4));
        assert!(!any.first_bytes.contains(&0x80) && !any.first_bytes.contains(&0xC1));

        for zero_width in ["true", "false"] {
            let summary = summarize(&rule_ref(zero_width), &tree);
            assert!(!summary.unresolved, "{zero_width} should resolve");
            assert!(summary.nullable, "{zero_width} is zero-width always-succeed");
        }

        // The semantic_annotation native matcher skips layout — must stay unresolved.
        let sem = summarize(&rule_ref("semantic_annotation"), &tree);
        assert!(sem.unresolved);

        // A grammar DEFINITION wins over the native builtin (codegen precedence).
        let mut tree = HashMap::new();
        tree.insert("builtin_ascii_char".to_string(), quoted_atom("x"));
        let shadowed = summarize(&rule_ref("builtin_ascii_char"), &tree);
        assert!(shadowed.first_bytes.is_empty());
        assert_eq!(shadowed.terminals, HashSet::from(["'x'".to_string()]));
    }

    #[test]
    fn negative_lookahead_subtracts_exact_byte_decided_sets() {
        // The scout's central case: unicode_char := !builtin_ascii_char builtin_any_char
        let unicode_char = ASTNode::Sequence {
            elements: vec![
                ASTNode::Lookahead {
                    element: Box::new(rule_ref("builtin_ascii_char")),
                    positive: false,
                },
                rule_ref("builtin_any_char"),
            ],
        };
        let summary = summarize(&unicode_char, &HashMap::new());
        assert!(!summary.unresolved);
        assert!(!summary.nullable);
        assert_eq!(
            summary.first_bytes,
            (0xC2u8..=0xF4).collect::<BTreeSet<u8>>(),
            "ASCII bytes must be subtracted, leaving exactly the non-ASCII lead bytes"
        );
    }

    #[test]
    fn negative_lookahead_without_exact_license_stays_over_approximated() {
        // `!'ab' <any>` — a two-byte terminal is NOT byte-decided, so no subtraction:
        // the generic union keeps the pre-D0 over-approximation.
        let seq = ASTNode::Sequence {
            elements: vec![
                ASTNode::Lookahead {
                    element: Box::new(quoted_atom("ab")),
                    positive: false,
                },
                rule_ref("builtin_any_char"),
            ],
        };
        let summary = summarize(&seq, &HashMap::new());
        assert!(summary.first_bytes.contains(&b'a'));
        assert!(summary.terminals.contains("'ab'"));
    }

    #[test]
    fn dispatch_trust_gate_rejects_untrusted_regex_bytes() {
        let branch = regex_atom("[a-z]");
        let tree = HashMap::new();
        let mut cache = HashMap::new();
        let err = branch_dispatch_first_bytes(&branch, &tree, &mut cache, false)
            .expect_err("regex-derived bytes must be refused without layout trust");
        assert!(err.contains("layout-skipping"), "unexpected reason: {err}");
        let bytes = branch_dispatch_first_bytes(&branch, &tree, &mut cache, true)
            .expect("trusted regex-derived bytes dispatch");
        assert_eq!(bytes, (b'a'..=b'z').collect::<Vec<u8>>());
    }

    #[test]
    fn dispatch_unions_quoted_terminal_and_regex_bytes() {
        let branch = ASTNode::Or {
            alternatives: vec![quoted_atom("x"), regex_atom("[ab]")],
        };
        let mut cache = HashMap::new();
        let bytes = branch_dispatch_first_bytes(&branch, &HashMap::new(), &mut cache, true)
            .expect("union dispatch");
        assert_eq!(bytes, vec![b'a', b'b', b'x']);
    }

    #[test]
    fn underivable_or_invalid_regex_stays_unresolved() {
        // An invalid pattern must fall back to the pre-D0 unresolved shape.
        let summary = summarize(&regex_atom("(unclosed"), &HashMap::new());
        assert!(summary.unresolved);
        assert!(summary.first_bytes.is_empty());
    }
}
