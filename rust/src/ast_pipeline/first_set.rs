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

/// Intra-BODY structural recursion bound (a body is a finite tree, so this is
/// belt-and-braces against pathological nesting, not a termination requirement).
const MAX_FIRST_SET_DEPTH: usize = 24;

/// D0.1 CACHE-COHERENCE — rule-CHAIN bound. Depth RESETS at rule boundaries so a
/// rule's computed (and cached) summary never depends on how deep the QUERY that
/// first reached it happened to start — the depth-context dual of the
/// visiting-context poisoning (tool-proven: a level-2 fold's inherited depth made
/// `quant_bound_core` hit the intra-body cutoff mid-chain and cache `unresolved`,
/// flipping a site verdict that resolves fine from a fresh query). Termination
/// across rules is owned by `visiting_rules` (cycle guard); this cap is the
/// absolute stack bound for pathological reference chains. Real grammars sit far
/// below it, so it never fires in practice; if it ever does, the affected value is
/// conservative (`unresolved`) and NOT cached.
const MAX_RULE_CHAIN_DEPTH: usize = 64;

/// D0.1 CACHE-COHERENCE — the traversal state of ONE top-level FIRST / second-byte
/// query.
///
/// - `visiting` — the in-progress rule stack (the cycle guard).
/// - `tainted` — markers for in-progress rules whose cycle guard FIRED: a value
///   computed while any marker is live absorbed an ancestor-context `unresolved`
///   and must not enter the PERSISTENT cache (it would make later fresh queries
///   order-dependent — the poisoning class this struct exists to kill).
/// - `transient` — the per-QUERY memo for exactly those context-dependent values:
///   sound within the query (conservative direction only — extra `unresolved`,
///   never a smaller resolved set) and discarded with it. This is what keeps deep
///   cyclic grammars (SV: 1466 rules) LINEAR — without it, cycle-scoped subtrees
///   recompute exponentially (tool-proven: the SV regen hung for 85 minutes under
///   outermost-only caching with no transient memo).
/// - `cap_fired` — the rule-chain cap fired somewhere in this query (chain-length
///   context ⇒ nothing later in the query may enter the persistent cache).
#[derive(Debug, Default)]
pub(crate) struct RuleVisit<T> {
    visiting: HashSet<String>,
    tainted: HashSet<String>,
    transient: HashMap<String, T>,
    cap_fired: bool,
}

/// FIRST set of an arbitrary grammar node (branch body / rule body / sub-expression).
pub(crate) fn branch_first_set(
    node: &ASTNode,
    grammar_tree: &HashMap<String, ASTNode>,
    first_set_cache: &mut HashMap<String, FirstSetSummary>,
    visiting_rules: &mut RuleVisit<FirstSetSummary>,
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
    visiting_rules: &mut RuleVisit<FirstSetSummary>,
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
    visiting_rules: &mut RuleVisit<FirstSetSummary>,
    // D0.1 — the query's depth deliberately does NOT reach the body (depth resets
    // per rule so cached summaries are depth-context-free); the chain cap on the
    // visiting stack is the recursion bound.
    _depth: usize,
) -> FirstSetSummary {
    if let Some(cached) = first_set_cache.get(rule_name) {
        return cached.clone();
    }
    if let Some(transient) = visiting_rules.transient.get(rule_name) {
        return transient.clone();
    }

    if !visiting_rules.visiting.insert(rule_name.to_string()) {
        // Cycle guard: the hit taints every value computed while this rule is
        // still in progress (they absorb an ancestor-context `unresolved`).
        visiting_rules.tainted.insert(rule_name.to_string());
        return FirstSetSummary {
            unresolved: true,
            ..FirstSetSummary::default()
        };
    }

    // D0.1 CACHE-COHERENCE — the rule-chain cap (the absolute recursion bound now
    // that depth resets per body). Chain length is query context, so nothing
    // computed after a cap hit may enter the persistent cache.
    if visiting_rules.visiting.len() > MAX_RULE_CHAIN_DEPTH {
        visiting_rules.cap_fired = true;
        visiting_rules.visiting.remove(rule_name);
        return FirstSetSummary {
            unresolved: true,
            ..FirstSetSummary::default()
        };
    }

    let result = if let Some(rule_ast) = grammar_tree.get(rule_name) {
        // D0.1 — depth 0: a rule's summary is computed from its OWN body root,
        // never from the query's inherited depth (depth-context-free caching).
        branch_first_set(rule_ast, grammar_tree, first_set_cache, visiting_rules, 0)
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

    visiting_rules.visiting.remove(rule_name);
    // A guard hit ON this rule is part of its own true conservative value, not
    // context-dependence for anyone else — clear the marker now that it left the
    // in-progress stack.
    visiting_rules.tainted.remove(rule_name);
    // D0.1 CACHE-COHERENCE — the PERSISTENT cache admits only CONTEXT-FREE values
    // (no live taint marker, no cap hit); everything else goes to the per-query
    // transient memo, which keeps the traversal linear without letting an
    // ancestor-context value leak into later, fresh queries (tool-proven: the
    // census's degeneracy count flipped 60→56 purely from a changed traversal
    // order, drifting from codegen's own emission decisions).
    if visiting_rules.tainted.is_empty() && !visiting_rules.cap_fired {
        first_set_cache.insert(rule_name.to_string(), result.clone());
    }
    visiting_rules
        .transient
        .insert(rule_name.to_string(), result.clone());
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
    let mut visiting_rules = RuleVisit::default();
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

/// RGX-0078.5.i.7 D1 STEP-0 — a node's SECOND-byte facts (global over all admitting
/// first bytes — exact for the singleton-FIRST families D1 targets, a sound
/// over-approximation everywhere else).
///
/// # Soundness contract (mirrors the FIRST carrier)
/// A consumer may exclude a branch at a two-byte prefix `(b1, b2)` ONLY when the
/// branch's level-1 summary admits `b1`, this summary is `!unresolved`, and
/// `b2 ∉ second_bytes` **and** `!len1_possible` **and** `!nullable`. `len1_possible`
/// means some match consumes EXACTLY ONE byte, so byte 2 is unconstrained — the
/// branch must be admitted in every second-byte arm (including at end-of-input).
/// `regex_token_derived` carries the same layout-trust obligation as the FIRST
/// carrier (anchored `match_regex` peeking is sound only without a leading layout
/// skip).
#[derive(Debug, Clone, Default)]
pub(crate) struct SecondByteSummary {
    /// Sound over-approximation of every possible SECOND byte of a non-empty match.
    pub(crate) second_bytes: BTreeSet<u8>,
    /// Some match may consume exactly one byte ⇒ byte-2 wildcard.
    pub(crate) len1_possible: bool,
    /// The node can match the empty string (mirrors `FirstSetSummary::nullable`).
    pub(crate) nullable: bool,
    /// Analysis incomplete ⇒ `second_bytes` is NOT exhaustive ⇒ never exclude on it.
    pub(crate) unresolved: bool,
    /// Any fact came from a `/regex/` terminal (layout-trust obligation).
    pub(crate) regex_token_derived: bool,
    /// RGX-0078.5.i.7 D1 — a byte-2-REFUTED attempt of this node may still ENTER a
    /// rule method (or native-builtin / inlined frame — all carry the
    /// furthest-position preamble) at offset ≥ 1. `furthest_position` is written at
    /// rule entry ONLY, so pruning such a branch on byte 2 is NOT
    /// furthest-position-neutral: the counterfactual attempt would have recorded
    /// `parse_start + 1` on rejected inputs. A byte-2 exclusion is licensed ONLY
    /// when this is `false`. Over-approximating (`true` when uncertain) is the
    /// sound, no-exclusion direction.
    pub(crate) offset1_rule_entry: bool,
}

impl SecondByteSummary {
    fn unresolved() -> Self {
        SecondByteSummary {
            unresolved: true,
            offset1_rule_entry: true,
            ..SecondByteSummary::default()
        }
    }
}

/// RGX-0078.5.i.7 D1 — does this node's SUBTREE (not following rule references)
/// contain any rule reference? Used at the sequence fold's offset-1 frontier: an
/// element ATTEMPTED at offset 1 may enter any rule referenced anywhere in its
/// subtree before failing (alternatives/optionals are tried even when they fail),
/// and every such entry writes `furthest_position`. Over-approximating (refs behind
/// consuming units are unreachable in a byte-2-refuted attempt, but still count
/// here) is the sound, no-exclusion direction. Unknown token kinds count as
/// references (conservative).
fn contains_rule_reference_shallow(node: &ASTNode) -> bool {
    match node {
        ASTNode::Sequence { elements } => elements.iter().any(contains_rule_reference_shallow),
        ASTNode::Or { alternatives } => {
            alternatives.iter().any(contains_rule_reference_shallow)
        }
        ASTNode::Quantified { element, .. } => contains_rule_reference_shallow(element),
        ASTNode::Lookahead { element, .. } => contains_rule_reference_shallow(element),
        ASTNode::Atom { value } => match value {
            ASTValue::Node(inner) => contains_rule_reference_shallow(inner),
            ASTValue::Token(parts) => {
                let Some(TokenValue::String(token_type)) = parts.first() else {
                    return true;
                };
                !matches!(token_type.as_str(), "quoted_string" | "regex")
            }
        },
    }
}

/// RGX-0078.5.i.7 Q-GUARD — the furthest-emulation FRONTIER class of a min-0
/// quantified site's ELEMENT: does a byte-1-REFUTED attempt of the element enter a
/// rule (writing `furthest_position` at the attempt position)?
///
/// - `BareRef` — the element (unwrapping group shells and single-element sequences)
///   is exactly a rule/builtin reference: a refuted attempt ALWAYS executes that
///   entry's preamble at the attempt position ⇒ the emulation
///   `if p > furthest { furthest = p }` is EXACT. The memo-hit case is exact by
///   MONOTONICITY: a cached failure at p implies an earlier REAL entry already
///   bumped furthest ≥ p, so the counterfactual hit (no preamble) and the emulation
///   both no-op.
/// - `NoRefs` — the element subtree contains NO rule reference (pure quoted/regex
///   probing): only rule methods / native builtins / inlined frames write
///   `furthest_position`, so a refuted attempt writes nothing ⇒ NO emulation is
///   EXACT.
/// - `Mixed` — anything else: whether a refuted attempt enters a rule depends on
///   inner alternative/optional structure; consumers must NOT guard (either
///   emulation guess can diverge — over-advance or lost diagnostics).
///
/// SHARED between the census (`QuantSiteCensus::frontier`) and codegen's Q-guard
/// emission — the same no-drift discipline as `branch_dispatch_first_bytes`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QuantFrontier {
    BareRef,
    NoRefs,
    Mixed,
}

impl QuantFrontier {
    /// Stable snake_case name (the census's serialized form).
    pub(crate) fn name(self) -> &'static str {
        match self {
            QuantFrontier::BareRef => "bare_ref",
            QuantFrontier::NoRefs => "no_refs",
            QuantFrontier::Mixed => "mixed",
        }
    }
}

pub(crate) fn quantified_element_frontier(element: &ASTNode) -> QuantFrontier {
    fn unwrap_shells(node: &ASTNode) -> &ASTNode {
        match node {
            ASTNode::Atom {
                value: ASTValue::Node(inner),
            } => unwrap_shells(inner),
            ASTNode::Sequence { elements } if elements.len() == 1 => unwrap_shells(&elements[0]),
            _ => node,
        }
    }
    if let ASTNode::Atom {
        value: ASTValue::Token(parts),
    } = unwrap_shells(element)
    {
        if let Some(TokenValue::String(token_type)) = parts.first() {
            if token_type == "rule_reference" {
                return QuantFrontier::BareRef;
            }
        }
    }
    if !contains_rule_reference_shallow(element) {
        return QuantFrontier::NoRefs;
    }
    QuantFrontier::Mixed
}

/// The union of a FIRST summary's admissible first BYTES (quoted-terminal first
/// bytes ∪ the D0 `first_bytes`), or `None` when any terminal's first byte is
/// unextractable — the composition helper the second-byte fold uses for "an element
/// beginning at offset 1 contributes its FIRST bytes as sequence second bytes".
fn summary_all_first_bytes(summary: &FirstSetSummary) -> Option<BTreeSet<u8>> {
    let mut bytes = summary.first_bytes.clone();
    for terminal in &summary.terminals {
        bytes.insert(terminal_first_byte(terminal)?);
    }
    Some(bytes)
}

/// SECOND-byte facts of an arbitrary grammar node. Composes with
/// [`branch_first_set`] (level 1) through the shared caches.
pub(crate) fn branch_second_byte_summary(
    node: &ASTNode,
    grammar_tree: &HashMap<String, ASTNode>,
    first_set_cache: &mut HashMap<String, FirstSetSummary>,
    second_byte_cache: &mut HashMap<String, SecondByteSummary>,
    visiting_rules: &mut RuleVisit<SecondByteSummary>,
    depth: usize,
) -> SecondByteSummary {
    if depth > MAX_FIRST_SET_DEPTH {
        return SecondByteSummary::unresolved();
    }

    match node {
        ASTNode::Sequence { elements } => {
            let mut result = SecondByteSummary::default();
            // `at0` / `at1` — a path can reach the CURRENT element having consumed
            // exactly 0 / exactly 1 byte(s). Elements past both frontiers cannot
            // affect the first two bytes.
            let mut at0 = true;
            let mut at1 = false;
            for element in elements {
                if !at0 && !at1 {
                    break;
                }
                let mut fs_visiting = RuleVisit::default();
                let element_first = branch_first_set(
                    element,
                    grammar_tree,
                    first_set_cache,
                    &mut fs_visiting,
                    depth + 1,
                );
                let element_second = branch_second_byte_summary(
                    element,
                    grammar_tree,
                    first_set_cache,
                    second_byte_cache,
                    visiting_rules,
                    depth + 1,
                );
                if at0 {
                    // Matches of the element starting at offset 0 contribute their
                    // own second bytes.
                    result
                        .second_bytes
                        .extend(element_second.second_bytes.iter().copied());
                    result.unresolved |= element_second.unresolved;
                    result.regex_token_derived |= element_second.regex_token_derived;
                    // D1 — the element's own offset-≥1 rule entries are at
                    // sequence offset ≥1 too.
                    result.offset1_rule_entry |= element_second.offset1_rule_entry;
                }
                if at1 {
                    // The element begins at offset 1 ⇒ its FIRST bytes are the
                    // sequence's second bytes.
                    match summary_all_first_bytes(&element_first) {
                        Some(bytes) => result.second_bytes.extend(bytes),
                        None => result.unresolved = true,
                    }
                    result.unresolved |= element_first.unresolved;
                    result.regex_token_derived |= element_first.regex_token_derived;
                    // D1 — an element ATTEMPTED at offset 1 may enter any rule its
                    // subtree references before failing (furthest write at +1).
                    result.offset1_rule_entry |= contains_rule_reference_shallow(element);
                }
                let next_at0 = at0 && element_first.nullable;
                let next_at1 =
                    (at1 && element_first.nullable) || (at0 && element_second.len1_possible);
                at0 = next_at0;
                at1 = next_at1;
            }
            result.nullable = at0;
            result.len1_possible = at1;
            result
        }
        ASTNode::Or { alternatives } => {
            let mut result = SecondByteSummary::default();
            if alternatives.is_empty() {
                result.nullable = true;
                return result;
            }
            for alternative in alternatives {
                let alt = branch_second_byte_summary(
                    alternative,
                    grammar_tree,
                    first_set_cache,
                    second_byte_cache,
                    visiting_rules,
                    depth + 1,
                );
                result.second_bytes.extend(alt.second_bytes.iter().copied());
                result.len1_possible |= alt.len1_possible;
                result.nullable |= alt.nullable;
                result.unresolved |= alt.unresolved;
                result.regex_token_derived |= alt.regex_token_derived;
                result.offset1_rule_entry |= alt.offset1_rule_entry;
            }
            result
        }
        ASTNode::Atom { value } => match value {
            ASTValue::Node(inner) => branch_second_byte_summary(
                inner,
                grammar_tree,
                first_set_cache,
                second_byte_cache,
                visiting_rules,
                depth + 1,
            ),
            ASTValue::Token(parts) => {
                if parts.len() < 2 {
                    return SecondByteSummary::unresolved();
                }
                let TokenValue::String(token_type) = &parts[0];
                let TokenValue::String(token_value) = &parts[1];
                match token_type.as_str() {
                    "quoted_string" => {
                        let bytes = token_value.as_bytes();
                        SecondByteSummary {
                            second_bytes: bytes.get(1).copied().into_iter().collect(),
                            len1_possible: bytes.len() == 1,
                            nullable: bytes.is_empty(),
                            ..SecondByteSummary::default()
                        }
                    }
                    "rule_reference" => rule_second_byte_summary(
                        token_value,
                        grammar_tree,
                        first_set_cache,
                        second_byte_cache,
                        visiting_rules,
                        depth + 1,
                    ),
                    "regex" => match regex_syntax::Parser::new()
                        .parse(token_value)
                        .ok()
                        .and_then(|hir| regex_hir_second(&hir))
                    {
                        Some(mut summary) => {
                            summary.regex_token_derived = true;
                            summary
                        }
                        None => SecondByteSummary::unresolved(),
                    },
                    _ => SecondByteSummary::unresolved(),
                }
            }
        },
        ASTNode::Quantified {
            element,
            quantifier,
        } => {
            let mut result = branch_second_byte_summary(
                element,
                grammar_tree,
                first_set_cache,
                second_byte_cache,
                visiting_rules,
                depth + 1,
            );
            let min_repeat = quantifier_min_repeat(quantifier);
            if min_repeat == 0 {
                result.nullable = true;
            }
            // A second repetition can begin at offset 1 when the first consumed
            // exactly one byte — its FIRST bytes join the second-byte set.
            if result.len1_possible && quantifier_max_allows_second_repeat(quantifier) {
                let mut fs_visiting = RuleVisit::default();
                let element_first = branch_first_set(
                    element,
                    grammar_tree,
                    first_set_cache,
                    &mut fs_visiting,
                    depth + 1,
                );
                match summary_all_first_bytes(&element_first) {
                    Some(bytes) => result.second_bytes.extend(bytes),
                    None => result.unresolved = true,
                }
                result.unresolved |= element_first.unresolved;
                result.regex_token_derived |= element_first.regex_token_derived;
                // D1 — the second repetition is ATTEMPTED at offset 1.
                result.offset1_rule_entry |= contains_rule_reference_shallow(element);
            }
            // `len1_possible` stays as the element's (over-approximating it is the
            // conservative, no-exclusion direction).
            result
        }
        ASTNode::Lookahead { element, .. } => {
            // Zero-width: contributes nothing at either offset; conservative
            // unresolvedness propagation mirrors the level-1 arm.
            let element_second = branch_second_byte_summary(
                element,
                grammar_tree,
                first_set_cache,
                second_byte_cache,
                visiting_rules,
                depth + 1,
            );
            SecondByteSummary {
                nullable: true,
                unresolved: element_second.unresolved,
                regex_token_derived: element_second.regex_token_derived,
                // D1 — the lookahead's inner attempt runs at offset 0; its own
                // offset-≥1 rule entries still write furthest (monotone, never
                // restored by the lookahead's position rollback).
                offset1_rule_entry: element_second.offset1_rule_entry,
                ..SecondByteSummary::default()
            }
        }
    }
}

fn rule_second_byte_summary(
    rule_name: &str,
    grammar_tree: &HashMap<String, ASTNode>,
    first_set_cache: &mut HashMap<String, FirstSetSummary>,
    second_byte_cache: &mut HashMap<String, SecondByteSummary>,
    visiting_rules: &mut RuleVisit<SecondByteSummary>,
    // D0.1 — see `rule_first_set`: depth resets per rule body.
    _depth: usize,
) -> SecondByteSummary {
    if let Some(cached) = second_byte_cache.get(rule_name) {
        return cached.clone();
    }
    if let Some(transient) = visiting_rules.transient.get(rule_name) {
        return transient.clone();
    }
    if !visiting_rules.visiting.insert(rule_name.to_string()) {
        visiting_rules.tainted.insert(rule_name.to_string());
        return SecondByteSummary::unresolved();
    }
    // D0.1 CACHE-COHERENCE — chain cap + per-body depth reset (see `rule_first_set`).
    if visiting_rules.visiting.len() > MAX_RULE_CHAIN_DEPTH {
        visiting_rules.cap_fired = true;
        visiting_rules.visiting.remove(rule_name);
        return SecondByteSummary::unresolved();
    }

    let result = if let Some(rule_ast) = grammar_tree.get(rule_name) {
        branch_second_byte_summary(
            rule_ast,
            grammar_tree,
            first_set_cache,
            second_byte_cache,
            visiting_rules,
            0,
        )
    } else {
        native_builtin_second_byte_summary(rule_name)
            .unwrap_or_else(SecondByteSummary::unresolved)
    };
    visiting_rules.visiting.remove(rule_name);
    visiting_rules.tainted.remove(rule_name);
    // D0.1 CACHE-COHERENCE — persistent cache admits context-free values only;
    // the per-query transient memo carries the rest (see `rule_first_set`).
    if visiting_rules.tainted.is_empty() && !visiting_rules.cap_fired {
        second_byte_cache.insert(rule_name.to_string(), result.clone());
    }
    visiting_rules
        .transient
        .insert(rule_name.to_string(), result.clone());
    result
}

/// The native builtins' SECOND-byte facts (mirrors `native_builtin_first_set`):
/// both char builtins can consume exactly one byte (ASCII) ⇒ byte-2 wildcard;
/// `builtin_any_char` additionally spans multi-byte scalars whose second byte is a
/// UTF-8 continuation byte.
fn native_builtin_second_byte_summary(rule_name: &str) -> Option<SecondByteSummary> {
    match rule_name {
        "builtin_ascii_char" => Some(SecondByteSummary {
            len1_possible: true,
            ..SecondByteSummary::default()
        }),
        "builtin_any_char" => Some(SecondByteSummary {
            second_bytes: (0x80u8..=0xBF).collect(),
            len1_possible: true,
            ..SecondByteSummary::default()
        }),
        "true" | "false" => Some(SecondByteSummary {
            nullable: true,
            ..SecondByteSummary::default()
        }),
        _ => None,
    }
}

/// RGX-0078.5.i.7 D1 — the branch's admissible SECOND bytes for a FIRST₂ prune
/// guard (sorted), or the NAMED reason the branch must not be byte-2-guarded.
///
/// This is the SHARED licensing predicate behind BOTH consumers — the FIRST₂ census
/// (`fusibility_census.rs`) and codegen's prune-guard emission
/// (`first_set_prune_guard_for_branch`) — the same no-drift discipline as
/// [`branch_dispatch_first_bytes`] at level 1. A branch is byte-2-guardable iff its
/// second-byte summary is resolved + non-nullable + `!len1_possible` (a 1-byte match
/// leaves byte 2 unconstrained) + non-empty + trust-gated for regex-token-derived
/// facts, AND `!offset1_rule_entry` — the furthest-position-parity license: a
/// refuted attempt must not have entered any rule at offset ≥ 1, or pruning it
/// changes rejected-parse `furthest_position` (a contract surface). `Err` = "guard
/// on byte 1 only" — the branch simply keeps today's level-1 behavior.
pub(crate) fn branch_prefix2_guard_bytes(
    branch: &ASTNode,
    grammar_tree: &HashMap<String, ASTNode>,
    first_set_cache: &mut HashMap<String, FirstSetSummary>,
    second_byte_cache: &mut HashMap<String, SecondByteSummary>,
    trust_regex_token_bytes: bool,
) -> Result<Vec<u8>, String> {
    let mut visiting_rules = RuleVisit::default();
    let summary = branch_second_byte_summary(
        branch,
        grammar_tree,
        first_set_cache,
        second_byte_cache,
        &mut visiting_rules,
        0,
    );
    if summary.unresolved {
        return Err("unresolved SECOND-byte facts (regex token / cycle / depth cutoff)".to_string());
    }
    if summary.nullable {
        return Err("nullable (can match empty)".to_string());
    }
    if summary.len1_possible {
        return Err("a 1-byte match is possible (byte-2 wildcard)".to_string());
    }
    if summary.regex_token_derived && !trust_regex_token_bytes {
        return Err(
            "regex-token-derived second bytes under layout-skipping regex tokens".to_string(),
        );
    }
    if summary.offset1_rule_entry {
        return Err(
            "a refuted attempt may enter a rule at offset ≥1 (furthest-position parity)"
                .to_string(),
        );
    }
    if summary.second_bytes.is_empty() {
        return Err("empty SECOND-byte set".to_string());
    }
    Ok(summary.second_bytes.iter().copied().collect())
}

/// SECOND-byte facts of a regex pattern's HIR (the level-2 sibling of
/// [`regex_hir_prefix`]). `None` = underivable (caller falls back to unresolved).
fn regex_hir_second(hir: &regex_syntax::hir::Hir) -> Option<SecondByteSummary> {
    use regex_syntax::hir::{Class, HirKind};
    match hir.kind() {
        HirKind::Empty => Some(SecondByteSummary {
            nullable: true,
            ..SecondByteSummary::default()
        }),
        HirKind::Literal(literal) => Some(SecondByteSummary {
            second_bytes: literal.0.get(1).copied().into_iter().collect(),
            len1_possible: literal.0.len() == 1,
            nullable: literal.0.is_empty(),
            ..SecondByteSummary::default()
        }),
        HirKind::Class(class) => {
            let mut summary = SecondByteSummary::default();
            match class {
                Class::Unicode(ranges) => {
                    for range in ranges.ranges() {
                        if (range.start() as u32) <= 0x7F {
                            summary.len1_possible = true;
                        }
                        if (range.end() as u32) > 0x7F {
                            // Multi-byte scalar ⇒ its second byte is a UTF-8
                            // continuation byte (sound over-approximation).
                            summary.second_bytes.extend(0x80u8..=0xBF);
                        }
                    }
                }
                Class::Bytes(_) => {
                    summary.len1_possible = true;
                }
            }
            Some(summary)
        }
        HirKind::Look(_) => Some(SecondByteSummary {
            nullable: true,
            ..SecondByteSummary::default()
        }),
        HirKind::Repetition(repetition) => {
            let mut result = regex_hir_second(&repetition.sub)?;
            if repetition.min == 0 {
                result.nullable = true;
            }
            if result.len1_possible && repetition.max.map_or(true, |max| max >= 2) {
                let prefix = regex_hir_prefix(&repetition.sub)?;
                result.second_bytes.extend(prefix.bytes.iter().copied());
            }
            Some(result)
        }
        HirKind::Capture(capture) => regex_hir_second(&capture.sub),
        HirKind::Concat(parts) => {
            let mut result = SecondByteSummary::default();
            let mut at0 = true;
            let mut at1 = false;
            for part in parts {
                if !at0 && !at1 {
                    break;
                }
                let part_second = regex_hir_second(part)?;
                let part_prefix = regex_hir_prefix(part)?;
                if at0 {
                    result
                        .second_bytes
                        .extend(part_second.second_bytes.iter().copied());
                }
                if at1 {
                    result.second_bytes.extend(part_prefix.bytes.iter().copied());
                }
                let next_at0 = at0 && part_prefix.nullable;
                let next_at1 = (at1 && part_prefix.nullable) || (at0 && part_second.len1_possible);
                at0 = next_at0;
                at1 = next_at1;
            }
            result.nullable = at0;
            result.len1_possible = at1;
            Some(result)
        }
        HirKind::Alternation(alternatives) => {
            let mut result = SecondByteSummary::default();
            for alternative in alternatives {
                let alt = regex_hir_second(alternative)?;
                result.second_bytes.extend(alt.second_bytes.iter().copied());
                result.len1_possible |= alt.len1_possible;
                result.nullable |= alt.nullable;
            }
            Some(result)
        }
    }
}

/// Does the quantifier's MAX bound allow a second repetition? (`?`/`{1}`/`{0,1}`
/// forbid it; `*`/`+`/`{n,}`/`{n,m≥2}` allow it.)
fn quantifier_max_allows_second_repeat(quantifier: &str) -> bool {
    let trimmed = quantifier.trim();
    match trimmed {
        "?" => false,
        "*" | "+" => true,
        _ if trimmed.starts_with('{') && trimmed.ends_with('}') => {
            let inner = trimmed[1..trimmed.len() - 1].trim();
            if inner.is_empty() {
                return true;
            }
            let max_part = match inner.split_once(',') {
                None => inner.trim(),                  // {N}
                Some((_, max)) if max.trim().is_empty() => return true, // {N,}
                Some((_, max)) => max.trim(),          // {N,M} / {,M}
            };
            max_part.parse::<usize>().map_or(true, |max| max >= 2)
        }
        _ => true,
    }
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
        let mut visiting = RuleVisit::default();
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

    fn summarize2(node: &ASTNode, tree: &HashMap<String, ASTNode>) -> SecondByteSummary {
        let mut first_cache = HashMap::new();
        let mut second_cache = HashMap::new();
        let mut visiting = RuleVisit::default();
        branch_second_byte_summary(
            node,
            tree,
            &mut first_cache,
            &mut second_cache,
            &mut visiting,
            0,
        )
    }

    #[test]
    fn second_bytes_of_escape_alternation_regex() {
        // The zero_width shape: 2-byte literals sharing the `\` first byte — the
        // second byte is the D1 discriminator.
        let summary = summarize2(&regex_atom(r"\\b|\\B|\\A"), &HashMap::new());
        assert!(!summary.unresolved);
        assert!(!summary.len1_possible);
        assert!(!summary.nullable);
        assert!(summary.regex_token_derived);
        assert_eq!(
            summary.second_bytes,
            BTreeSet::from([b'A', b'B', b'b'])
        );
    }

    #[test]
    fn second_bytes_of_group_open_sequence() {
        // The `(`-cluster shape: quoted '(' then '?' — byte 2 discriminates.
        let seq = ASTNode::Sequence {
            elements: vec![quoted_atom("("), quoted_atom("?")],
        };
        let summary = summarize2(&seq, &HashMap::new());
        assert!(!summary.unresolved);
        assert!(!summary.len1_possible);
        assert_eq!(summary.second_bytes, BTreeSet::from([b'?']));
    }

    #[test]
    fn single_byte_terminal_is_len1_wildcard() {
        let summary = summarize2(&quoted_atom("x"), &HashMap::new());
        assert!(summary.len1_possible, "byte-2 must be unconstrained");
        assert!(summary.second_bytes.is_empty());
        // A multi-byte terminal pins its second byte instead.
        let summary = summarize2(&quoted_atom("ab"), &HashMap::new());
        assert!(!summary.len1_possible);
        assert_eq!(summary.second_bytes, BTreeSet::from([b'b']));
    }

    #[test]
    fn builtin_second_bytes_mirror_native_matchers() {
        let any = summarize2(&rule_ref("builtin_any_char"), &HashMap::new());
        assert!(any.len1_possible);
        assert_eq!(any.second_bytes, (0x80u8..=0xBF).collect::<BTreeSet<u8>>());
        let ascii = summarize2(&rule_ref("builtin_ascii_char"), &HashMap::new());
        assert!(ascii.len1_possible && ascii.second_bytes.is_empty());
        let undefined = summarize2(&rule_ref("no_such_rule"), &HashMap::new());
        assert!(undefined.unresolved);
    }

    #[test]
    fn quantified_second_repetition_contributes_first_bytes() {
        // 'a'+ — a second repetition can begin at offset 1.
        let quant = ASTNode::Quantified {
            element: Box::new(quoted_atom("a")),
            quantifier: "+".to_string(),
        };
        let summary = summarize2(&quant, &HashMap::new());
        assert!(summary.len1_possible);
        assert_eq!(summary.second_bytes, BTreeSet::from([b'a']));
        // 'ab'? — max 1 repetition: only the literal's own second byte, nullable.
        let quant = ASTNode::Quantified {
            element: Box::new(quoted_atom("ab")),
            quantifier: "?".to_string(),
        };
        let summary = summarize2(&quant, &HashMap::new());
        assert!(summary.nullable);
        assert!(!summary.len1_possible);
        assert_eq!(summary.second_bytes, BTreeSet::from([b'b']));
    }

    #[test]
    fn regex_bounded_repeat_second_bytes() {
        // `[0-9]{1,2}` — a single-byte class with a possible second repetition.
        let summary = summarize2(&regex_atom("[0-9]{1,2}"), &HashMap::new());
        assert!(!summary.unresolved);
        assert!(summary.len1_possible);
        assert_eq!(
            summary.second_bytes,
            (b'0'..=b'9').collect::<BTreeSet<u8>>()
        );
    }

    #[test]
    fn second_byte_cycle_is_unresolved() {
        let mut tree = HashMap::new();
        tree.insert("looper".to_string(), rule_ref("looper"));
        let summary = summarize2(&rule_ref("looper"), &tree);
        assert!(summary.unresolved);
    }

    /// D1 — the furthest-position-parity flag: a rule reference ATTEMPTED at
    /// offset 1 (the `hex_escape = "x" payload_rule` shape) writes
    /// `furthest_position` even when the attempt fails, so the branch must not be
    /// byte-2-pruned. Pure-terminal 2-byte prefixes refute without any offset-≥1
    /// rule entry and stay licensed.
    #[test]
    fn offset1_rule_entry_flags_ref_at_offset_one() {
        let mut tree = HashMap::new();
        tree.insert("payload".to_string(), quoted_atom("ab"));
        // "x" payload — the payload rule is entered at offset 1.
        let seq = ASTNode::Sequence {
            elements: vec![quoted_atom("x"), rule_ref("payload")],
        };
        let summary = summarize2(&seq, &tree);
        assert!(!summary.unresolved);
        assert!(summary.offset1_rule_entry);
        // "\Q" "\E" — both units are 2-byte quoted terminals: the offset-1
        // frontier never crosses a rule reference.
        let seq = ASTNode::Sequence {
            elements: vec![quoted_atom("\\Q"), quoted_atom("\\E")],
        };
        let summary = summarize2(&seq, &tree);
        assert!(!summary.offset1_rule_entry);
        assert_eq!(summary.second_bytes, BTreeSet::from([b'Q']));
    }

    /// D1 — the flag resolves THROUGH rule references (a branch that is a bare
    /// ref to an offending rule inherits its body's verdict), and an at-offset-0
    /// lookahead's inner offset-≥1 entries propagate (furthest is monotone —
    /// the lookahead's position rollback never restores it).
    #[test]
    fn offset1_rule_entry_propagates_through_rules_and_lookaheads() {
        let mut tree = HashMap::new();
        tree.insert("payload".to_string(), quoted_atom("ab"));
        tree.insert(
            "offender".to_string(),
            ASTNode::Sequence {
                elements: vec![quoted_atom("x"), rule_ref("payload")],
            },
        );
        tree.insert("clean".to_string(), quoted_atom("\\E"));
        let summary = summarize2(&rule_ref("offender"), &tree);
        assert!(summary.offset1_rule_entry);
        let summary = summarize2(&rule_ref("clean"), &tree);
        assert!(!summary.offset1_rule_entry);
        // &("x" payload) "yz" — the lookahead's inner attempt enters `payload`
        // at offset 1 before the sequence's own terminals run.
        let seq = ASTNode::Sequence {
            elements: vec![
                ASTNode::Lookahead {
                    element: Box::new(ASTNode::Sequence {
                        elements: vec![quoted_atom("x"), rule_ref("payload")],
                    }),
                    positive: true,
                },
                quoted_atom("yz"),
            ],
        };
        let summary = summarize2(&seq, &tree);
        assert!(summary.offset1_rule_entry);
    }

    /// D1 — a second repetition beginning at offset 1 attempts its element there:
    /// a rule-reference element flags, a terminal element does not.
    #[test]
    fn offset1_rule_entry_from_quantified_second_repetition() {
        let mut tree = HashMap::new();
        tree.insert("one".to_string(), quoted_atom("a"));
        let quant = ASTNode::Quantified {
            element: Box::new(rule_ref("one")),
            quantifier: "+".to_string(),
        };
        let summary = summarize2(&quant, &tree);
        assert!(summary.offset1_rule_entry);
        let quant = ASTNode::Quantified {
            element: Box::new(quoted_atom("a")),
            quantifier: "+".to_string(),
        };
        let summary = summarize2(&quant, &tree);
        assert!(!summary.offset1_rule_entry);
    }

    /// D1 — the SHARED emission-licensing predicate: every refusal is NAMED, and
    /// the licensed shape returns its sorted second-byte guard set.
    #[test]
    fn branch_prefix2_guard_bytes_license_matrix() {
        let mut tree = HashMap::new();
        tree.insert("payload".to_string(), quoted_atom("ab"));
        let mut fs_cache = HashMap::new();
        let mut sb_cache = HashMap::new();
        // Licensed: 2-byte quoted prefix.
        let seq = ASTNode::Sequence {
            elements: vec![quoted_atom("\\Q"), quoted_atom("\\E")],
        };
        assert_eq!(
            branch_prefix2_guard_bytes(&seq, &tree, &mut fs_cache, &mut sb_cache, true),
            Ok(vec![b'Q'])
        );
        // Licensed: regex-token escape alternation (under regex-token trust).
        let node = regex_atom(r"\\b|\\B");
        assert_eq!(
            branch_prefix2_guard_bytes(&node, &tree, &mut fs_cache, &mut sb_cache, true),
            Ok(vec![b'B', b'b'])
        );
        // Refused: the same summary without regex-token layout trust.
        let err = branch_prefix2_guard_bytes(&node, &tree, &mut fs_cache, &mut sb_cache, false)
            .unwrap_err();
        assert!(err.contains("layout-skipping"), "{err}");
        // Refused: a 1-byte match leaves byte 2 unconstrained.
        let err =
            branch_prefix2_guard_bytes(&quoted_atom("x"), &tree, &mut fs_cache, &mut sb_cache, true)
                .unwrap_err();
        assert!(err.contains("1-byte match"), "{err}");
        // Refused: nullable.
        let opt = ASTNode::Quantified {
            element: Box::new(quoted_atom("ab")),
            quantifier: "?".to_string(),
        };
        let err = branch_prefix2_guard_bytes(&opt, &tree, &mut fs_cache, &mut sb_cache, true)
            .unwrap_err();
        assert!(err.contains("nullable"), "{err}");
        // Refused: furthest-position parity (rule entry at offset 1).
        let seq = ASTNode::Sequence {
            elements: vec![quoted_atom("x"), rule_ref("payload")],
        };
        let err = branch_prefix2_guard_bytes(&seq, &tree, &mut fs_cache, &mut sb_cache, true)
            .unwrap_err();
        assert!(err.contains("furthest-position parity"), "{err}");
        // Refused: unresolved (undefined reference).
        let err = branch_prefix2_guard_bytes(
            &rule_ref("no_such_rule"),
            &tree,
            &mut fs_cache,
            &mut sb_cache,
            true,
        )
        .unwrap_err();
        assert!(err.contains("unresolved"), "{err}");
    }

    /// D0.1 CACHE-COHERENCE — a summary computed mid-cycle (an ancestor still on
    /// the visiting stack) must never be cached: the pre-fix code returned the
    /// poisoned ancestor-context value on later fresh queries, making verdicts
    /// call-ORDER-dependent (tool-proven on the real grammar: the census's
    /// degeneracy count flipped 60→56 from a changed traversal order alone).
    #[test]
    fn rule_cache_stores_only_context_free_summaries() {
        let mut tree = HashMap::new();
        tree.insert("entry".to_string(), rule_ref("inner"));
        tree.insert(
            "inner".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    ASTNode::Sequence {
                        elements: vec![
                            ASTNode::Quantified {
                                element: Box::new(quoted_atom("a")),
                                quantifier: "?".to_string(),
                            },
                            rule_ref("entry"),
                        ],
                    },
                    quoted_atom("b"),
                ],
            },
        );
        // Fresh-cache truth for `entry`.
        let fresh = summarize(&rule_ref("entry"), &tree);
        assert!(fresh.terminals.contains("'a'") && fresh.terminals.contains("'b'"));
        // A shared cache that first computes `inner` (whose recursion finishes
        // `entry` while `inner` is still in progress), then queries `entry` —
        // the answer must equal the fresh-cache truth. Pre-fix, the poisoned
        // mid-cycle `entry` value (EMPTY terminals) was cached and returned.
        let mut cache = HashMap::new();
        let mut visiting = RuleVisit::default();
        let _ = branch_first_set(&rule_ref("inner"), &tree, &mut cache, &mut visiting, 0);
        let mut visiting = RuleVisit::default();
        let after = branch_first_set(&rule_ref("entry"), &tree, &mut cache, &mut visiting, 0);
        assert_eq!(after.terminals, fresh.terminals);
        assert_eq!(after.unresolved, fresh.unresolved);
        assert_eq!(after.nullable, fresh.nullable);
    }

    /// RGX-0078.5.i.7 Q-GUARD — the furthest-emulation frontier classification:
    /// a bare rule reference (through group shells / single-element sequences) is
    /// `BareRef`; a pure quoted/regex subtree is `NoRefs`; a mix (a refuted
    /// attempt MAY or may not enter a rule) is `Mixed`; an unknown token kind is
    /// conservatively a reference (never `NoRefs`).
    #[test]
    fn quantified_element_frontier_classifies_emulation_exactness() {
        assert_eq!(
            quantified_element_frontier(&rule_ref("class_zero_width")),
            QuantFrontier::BareRef
        );
        // Group shell + single-element sequence unwrap to the bare reference.
        let shelled = ASTNode::Atom {
            value: ASTValue::Node(Box::new(ASTNode::Sequence {
                elements: vec![rule_ref("r")],
            })),
        };
        assert_eq!(quantified_element_frontier(&shelled), QuantFrontier::BareRef);
        assert_eq!(
            quantified_element_frontier(&quoted_atom("a")),
            QuantFrontier::NoRefs
        );
        assert_eq!(
            quantified_element_frontier(&ASTNode::Sequence {
                elements: vec![quoted_atom("a"), regex_atom("[0-9]")],
            }),
            QuantFrontier::NoRefs
        );
        assert_eq!(
            quantified_element_frontier(&ASTNode::Sequence {
                elements: vec![quoted_atom("a"), rule_ref("r")],
            }),
            QuantFrontier::Mixed
        );
        assert_eq!(
            quantified_element_frontier(&ASTNode::Or {
                alternatives: vec![quoted_atom("a"), rule_ref("r")],
            }),
            QuantFrontier::Mixed
        );
        let unknown_token = ASTNode::Atom {
            value: ASTValue::Token(vec![
                TokenValue::String("mystery_kind".to_string()),
                TokenValue::String("payload".to_string()),
            ]),
        };
        assert_eq!(
            quantified_element_frontier(&unknown_token),
            QuantFrontier::Mixed
        );
    }
}
