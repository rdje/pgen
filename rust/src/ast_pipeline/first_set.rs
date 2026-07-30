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
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

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
/// - `true` / `false` were listed here as zero-width ALWAYS-SUCCEED fallbacks until
///   `LANG-CAPABILITY-AUDIT.10.4` removed them from the const for exactly that reason
///   (a rule reference to either silently matched EMPTY). They are ordinary undefined
///   references now, so this summary must not resolve them.
/// - `semantic_annotation` was never listed: while it was on the const its native
///   matcher skipped leading layout before requiring `@`, so first-byte peeking at the
///   parse position was unsound and it stayed `unresolved` (the caller's fallback).
///   `LANG-CAPABILITY-AUDIT.10.3` then removed the const entry and the matcher
///   outright, so it is an ordinary undefined reference now — same outcome here, for a
///   stronger reason.
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

/// RGX-0078.5.j.4 K4b C1 — the quantifier's MAX repetition bound (`None` =
/// unbounded). Unparseable forms return the CONSERVATIVE direction for the
/// trie consumer: `None` (extra repetitions are over-approximated as allowed,
/// which only widens the admitted set and marks more boundaries accepting —
/// never a fabricated refutation or entry claim).
fn quantifier_max_repeat(quantifier: &str) -> Option<usize> {
    let trimmed = quantifier.trim();
    match trimmed {
        "?" => Some(1),
        "*" | "+" => None,
        _ if trimmed.starts_with('{') && trimmed.ends_with('}') => {
            let inner = trimmed[1..trimmed.len() - 1].trim();
            if inner.is_empty() {
                return None;
            }
            let max_part = match inner.split_once(',') {
                None => inner.trim(),                                   // {N}
                Some((_, max)) if max.trim().is_empty() => return None, // {N,}
                Some((_, max)) => max.trim(),                           // {N,M} / {,M}
            };
            max_part.parse::<usize>().ok()
        }
        _ => None,
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

// ============================================================================
// RGX-0078.5.j.4 K4b C1 — bounded per-path PREFIX TRIE (FIRSTₖ, k ≤ 4).
//
// Generalizes the level-1 FIRST guard and the D1 global `SecondByteSummary`
// into ONE per-path carrier (design: docs/tasks/artifacts/k4b_delta/
// step1_c1_design.md). Per branch, a bounded trie over the input bytes at
// offsets 0..k−1 describes every prefix a match of the branch can have; a
// guard walks it and REFUTES the branch when the walk falls off — pure
// CANNOT-match pruning, so language/AST/winner/verdicts are unchanged by
// construction. The three D1 license refusals are lifted:
//  - per-PATH admitted bytes (not global-over-first-bytes),
//  - per-PATH `accepting` (the `len1_possible` generalization: a walk that
//    reaches a node where some complete match ends must attempt the branch),
//  - the `offset1_rule_entry` REFUSAL is replaced by an EXACT furthest-position
//    EMULATION license: `entry_at_node` records where rule entries execute, and
//    a refutation arm emits one conditional max-write (§ exactness below).
//
// # Furthest-parity exactness (the C1 license)
// `furthest_position` writers are rule-entry preambles ONLY (emitter `:3903`,
// Q-GUARD `:5151`; terminals never write it — tool-verified in `-0157`/`-0158`).
// For a refuted walk that consumed bytes 0..j−1 and died at offset j, every
// grammar path merged into the walked node is GENUINELY attempted by the real
// (unpruned) branch before it fails: the tournament attempts all byte-viable
// alternatives, ordered-Or later alternatives run because earlier ones fail,
// nested structures are UNGUARDED (level-1/D1/C1 guards exist only on
// rule-top-level Or branches, and a rule boundary's own entry preamble writes
// the same offset as any same-depth internal entry), and greedy min-0
// quantified attempts either run (entering their element) or are Q-guard-elided
// WITH the exact emulation that writes the same offset. Hence the real
// attempt's deepest furthest write equals `parse_start + w` where `w` is the
// max `entry_at_node` depth along the walked path — the merged max is EXACT,
// and the emitted refutation arm reproduces it with one conditional max-write.
// Root refutation (j = 0) is UNIVERSALLY w = 0: every consuming path needs
// byte 0 ∈ FIRST, an offset-0 entry rewrites the already-written `parse_start`,
// and the only withheld-byte case (the D0 exact negative-lookahead subtraction)
// consumes via a single-byte-decided inner whose entry is at offset 0 too.
//
// # Composition safety (where exactness would be lost, the node degrades)
//  - A lookahead whose subtree contains rule references admits only its inner
//    FIRST bytes as `unresolved` children: a byte inside them may let the inner
//    attempt consume arbitrarily deep (writing furthest beyond any bound), so
//    the walk must ATTEMPT there; a byte outside them makes the inner attempt
//    fail at its char 1, whose entry behavior is the inner trie root's exact
//    `entry_at_node`. A POSITIVE lookahead's node is non-accepting (every match
//    needs the inner's first byte next) and closes the frontier (nothing may
//    graft below it — deeper refutations could not bound the passed inner's
//    writes); a NEGATIVE lookahead's node is accepting (the match can end
//    there whenever the inner fails at char 1) and composes on. A NULLABLE or
//    unresolved-FIRST inner degrades the node to `unresolved`.
//  - Ref-free lookaheads are transparent (no writers; ignoring their byte
//    constraint only over-approximates the admitted set — sound).
//  - Anything else the analysis cannot bound (undefined references, cycles,
//    underivable regex heads, cap overflows) degrades path-locally to
//    `unresolved` = the shallower guard — never unsound, only less precise.
//
// # Caps (determinism + artifact-size control; design constants)
// depth ≤ 4, per-node fanout ≤ 24 distinct bytes, ≤ 16 nodes per built trie.
// Exceeding a cap truncates that subtree to `unresolved` deterministically
// (BTreeMap byte order, fixed fold order). The D1 fallback layer (§ license)
// deliberately bypasses the fanout cap — it reproduces today's proven D1
// emission verbatim where the per-path analysis is weaker.
// ============================================================================

/// Depth cap: the walk inspects input bytes at offsets `0..PREFIX_TRIE_DEPTH_CAP`.
/// Nodes AT the cap depth are attempt-terminal (accepting or unresolved).
pub(crate) const PREFIX_TRIE_DEPTH_CAP: usize = 4;
/// Per-node fanout cap (distinct admitted bytes). Overflow ⇒ node `unresolved`.
pub(crate) const PREFIX_TRIE_FANOUT_CAP: usize = 24;
/// Whole-trie node budget per build (children created; the root is free).
pub(crate) const PREFIX_TRIE_NODE_CAP: usize = 16;

/// One bounded prefix-trie node. Offsets are RELATIVE to the trie's start until
/// the guard composition fixes the branch root at `parse_start`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct PrefixTrieNode {
    /// Admitted next bytes ALONG THIS PATH (per-path, not global).
    pub(crate) children: BTreeMap<u8, PrefixTrieNode>,
    /// Some complete match ends at this depth ⇒ the walk must attempt here.
    pub(crate) accepting: bool,
    /// Analysis incomplete along this path ⇒ the walk must attempt here.
    pub(crate) unresolved: bool,
    /// A rule/builtin entry preamble executes at exactly this depth on every
    /// merged path (exact — see the module exactness argument), so a refutation
    /// AT OR BELOW this node must emulate a furthest write at this offset.
    pub(crate) entry_at_node: bool,
    /// Any byte fact in this subtree came from a `/regex/` terminal (the same
    /// layout-trust obligation as the level-1/D1 carriers). OR-propagated to
    /// ancestors, so the root flag covers the whole trie.
    pub(crate) regex_token_derived: bool,
    /// A cap (depth / fanout / node budget) truncated this subtree somewhere.
    /// OR-propagated to ancestors (census reporting).
    pub(crate) truncated: bool,
    /// FINALIZE-ONLY: max `entry_at_node` depth on the path root..=this node
    /// (0 = only furthest-neutral offset-0 entries). The refutation arm's `w`.
    pub(crate) deepest_entry_offset: u8,
}

impl PrefixTrieNode {
    fn unresolved_leaf() -> Self {
        PrefixTrieNode {
            unresolved: true,
            ..PrefixTrieNode::default()
        }
    }

    fn epsilon() -> Self {
        PrefixTrieNode {
            accepting: true,
            ..PrefixTrieNode::default()
        }
    }

    /// The walk stops refining here (refutation can never fire at or below).
    pub(crate) fn attempt_terminal(&self) -> bool {
        self.accepting || self.unresolved
    }

    /// Degrade to an attempt-terminal because a cap fired or exactness was
    /// lost. `unresolved` is set UNCONDITIONALLY — even on an accepting node —
    /// because truncation must be STICKY under composition: an accepting node
    /// whose children were cap-dropped would otherwise become refutation-live
    /// with INCOMPLETE children the moment a later sequence graft clears its
    /// accepting flag (`keep_accepting: false`). Tool-pinned over-prune: the
    /// `digits "." digits` shape emitted a guard demanding `.` immediately
    /// after one digit — `(?(VERSION>=10.0)…)` refused — because the budget
    /// truncated the `digit+` repetition children off accepting boundary nodes
    /// and the `.`-graft then treated the remainder as exhaustive (the
    /// `prefix_trie_truncated_repetition_stays_unrefutable` regression pin).
    fn degrade_unresolved(&mut self) {
        self.truncated = true;
        self.unresolved = true;
        self.children.clear();
    }

    pub(crate) fn count_nodes(&self) -> usize {
        1 + self
            .children
            .values()
            .map(PrefixTrieNode::count_nodes)
            .sum::<usize>()
    }

    fn max_depth(&self) -> usize {
        self.children
            .values()
            .map(|c| 1 + c.max_depth())
            .max()
            .unwrap_or(0)
    }
}

/// Node budget for ONE composite accumulator (a Sequence/Or/Quantified fold or
/// an HIR composite). Each accumulator charges only the nodes cloned/merged
/// into ITSELF, so the finished trie at every level stays ≤ the cap while
/// sub-builds cannot spuriously exhaust an ancestor's budget (rule-body tries
/// stay context-free for the D0.1 cache).
struct PrefixTrieBudget {
    nodes_left: usize,
}

impl PrefixTrieBudget {
    fn fresh() -> Self {
        PrefixTrieBudget {
            nodes_left: PREFIX_TRIE_NODE_CAP,
        }
    }
}

/// Clone `src` into a fresh node, honoring the remaining depth and the node
/// budget. `None` = the budget is exhausted (the caller degrades its parent).
fn clone_bounded(
    src: &PrefixTrieNode,
    depth_left: usize,
    budget: &mut PrefixTrieBudget,
) -> Option<PrefixTrieNode> {
    if budget.nodes_left == 0 {
        return None;
    }
    budget.nodes_left -= 1;
    let mut node = PrefixTrieNode {
        children: BTreeMap::new(),
        accepting: src.accepting,
        unresolved: src.unresolved,
        entry_at_node: src.entry_at_node,
        regex_token_derived: src.regex_token_derived,
        truncated: src.truncated,
        deepest_entry_offset: 0,
    };
    if node.unresolved {
        return Some(node);
    }
    if depth_left == 0 {
        if !src.children.is_empty() {
            node.degrade_unresolved();
        }
        return Some(node);
    }
    for (byte, child) in &src.children {
        match clone_bounded(child, depth_left - 1, budget) {
            Some(cloned) => {
                node.regex_token_derived |= cloned.regex_token_derived;
                node.truncated |= cloned.truncated;
                node.children.insert(*byte, cloned);
            }
            None => {
                node.degrade_unresolved();
                return Some(node);
            }
        }
    }
    Some(node)
}

/// Merge `src` into `dst` (language union), honoring depth and budget. An
/// `unresolved` outcome absorbs children (the walk must attempt there anyway).
fn merge_node(
    dst: &mut PrefixTrieNode,
    src: &PrefixTrieNode,
    depth_left: usize,
    budget: &mut PrefixTrieBudget,
) {
    dst.accepting |= src.accepting;
    dst.unresolved |= src.unresolved;
    dst.entry_at_node |= src.entry_at_node;
    dst.regex_token_derived |= src.regex_token_derived;
    dst.truncated |= src.truncated;
    if dst.unresolved {
        dst.children.clear();
        return;
    }
    if depth_left == 0 {
        if !src.children.is_empty() {
            dst.degrade_unresolved();
        }
        return;
    }
    for (byte, src_child) in &src.children {
        if let Some(dst_child) = dst.children.get_mut(byte) {
            merge_node(dst_child, src_child, depth_left - 1, budget);
        } else {
            match clone_bounded(src_child, depth_left - 1, budget) {
                Some(cloned) => {
                    dst.regex_token_derived |= cloned.regex_token_derived;
                    dst.truncated |= cloned.truncated;
                    dst.children.insert(*byte, cloned);
                }
                None => {
                    dst.degrade_unresolved();
                    return;
                }
            }
        }
    }
    if dst.children.len() > PREFIX_TRIE_FANOUT_CAP {
        dst.degrade_unresolved();
    }
}

/// Graft `element` onto every current frontier (accepting) node of `acc`:
/// the sequence-advance step. `keep_accepting` = the frontier node remains a
/// possible match end (quantified `min == 0` / closure boundaries); otherwise
/// its accepting-ness is re-derived from the element root (nullable elements
/// keep the frontier). Fixed pre-order over a pre-captured frontier so budget
/// truncation is deterministic.
fn graft_at_frontier(
    acc: &mut PrefixTrieNode,
    element: &PrefixTrieNode,
    keep_accepting: bool,
    budget: &mut PrefixTrieBudget,
) {
    fn collect_frontier(node: &PrefixTrieNode, depth: usize, path: &mut Vec<u8>, out: &mut Vec<(Vec<u8>, usize)>) {
        if node.unresolved {
            return;
        }
        if node.accepting {
            out.push((path.clone(), depth));
        }
        for (byte, child) in &node.children {
            path.push(*byte);
            collect_frontier(child, depth + 1, path, out);
            path.pop();
        }
    }
    let mut frontier = Vec::new();
    collect_frontier(acc, 0, &mut Vec::new(), &mut frontier);
    'paths: for (path, depth) in frontier {
        let mut node = &mut *acc;
        for byte in &path {
            // A prior graft in this pass may have degraded an ancestor to
            // `unresolved` (dropping this path) — that is the sound direction.
            node = match node.children.get_mut(byte) {
                Some(next) => next,
                None => continue 'paths,
            };
        }
        if node.unresolved {
            continue;
        }
        if !keep_accepting {
            node.accepting = false;
        }
        merge_node(
            node,
            element,
            PREFIX_TRIE_DEPTH_CAP.saturating_sub(depth),
            budget,
        );
    }
}

fn has_open_frontier(node: &PrefixTrieNode) -> bool {
    if node.unresolved {
        return false;
    }
    if node.accepting {
        return true;
    }
    node.children.values().any(has_open_frontier)
}

/// The bounded prefix trie of one grammar node (offsets relative to its start).
/// The recursion mirrors [`branch_second_byte_summary`]'s skeleton with the
/// same D0.1 cache-coherence discipline for rule references.
fn node_prefix_trie(
    node: &ASTNode,
    grammar_tree: &HashMap<String, ASTNode>,
    first_set_cache: &mut HashMap<String, FirstSetSummary>,
    trie_cache: &mut HashMap<String, PrefixTrieNode>,
    visiting_rules: &mut RuleVisit<PrefixTrieNode>,
    depth: usize,
) -> PrefixTrieNode {
    if depth > MAX_FIRST_SET_DEPTH {
        return PrefixTrieNode::unresolved_leaf();
    }
    match node {
        ASTNode::Sequence { elements } => {
            let mut acc = PrefixTrieNode::epsilon();
            let mut budget = PrefixTrieBudget::fresh();
            for element in elements {
                let element_trie = node_prefix_trie(
                    element,
                    grammar_tree,
                    first_set_cache,
                    trie_cache,
                    visiting_rules,
                    depth + 1,
                );
                graft_at_frontier(&mut acc, &element_trie, false, &mut budget);
                if !has_open_frontier(&acc) {
                    // Every path is attempt-terminal or closed (e.g. a positive
                    // refs-lookahead): later elements cannot affect the walk.
                    break;
                }
            }
            acc
        }
        ASTNode::Or { alternatives } => {
            let mut acc = PrefixTrieNode::default();
            if alternatives.is_empty() {
                acc.accepting = true;
                return acc;
            }
            let mut budget = PrefixTrieBudget::fresh();
            for alternative in alternatives {
                let alt_trie = node_prefix_trie(
                    alternative,
                    grammar_tree,
                    first_set_cache,
                    trie_cache,
                    visiting_rules,
                    depth + 1,
                );
                merge_node(&mut acc, &alt_trie, PREFIX_TRIE_DEPTH_CAP, &mut budget);
            }
            acc
        }
        ASTNode::Quantified {
            element,
            quantifier,
        } => {
            let inner = node_prefix_trie(
                element,
                grammar_tree,
                first_set_cache,
                trie_cache,
                visiting_rules,
                depth + 1,
            );
            let min = quantifier_min_repeat(quantifier).min(PREFIX_TRIE_DEPTH_CAP + 1);
            let extra = quantifier_max_repeat(quantifier)
                .map(|max| max.saturating_sub(quantifier_min_repeat(quantifier)))
                .unwrap_or(PREFIX_TRIE_DEPTH_CAP + 1)
                .min(PREFIX_TRIE_DEPTH_CAP + 1);
            let mut acc = PrefixTrieNode::epsilon();
            let mut budget = PrefixTrieBudget::fresh();
            // The mandatory repetitions advance the frontier (a boundary short
            // of `min` is not a match end).
            for _ in 0..min {
                graft_at_frontier(&mut acc, &inner, false, &mut budget);
                if !has_open_frontier(&acc) {
                    return acc;
                }
            }
            // Optional repetitions: every boundary from `min` on is a possible
            // match end AND — while the max allows — may continue with another
            // repetition. Greedy attempt entries at those boundaries merge in
            // exactly (the real parse tries the element there before falling
            // through, or the Q-guard elides it WITH the equivalent emulation
            // write). The rep count honors the quantifier's MAX: fabricating a
            // repetition the max forbids would claim entry writes the real
            // attempt can never make (a furthest-parity break, not just an
            // over-approximation — pinned by the min0-parity unit test).
            for _ in 0..extra {
                let before = acc.clone();
                graft_at_frontier(&mut acc, &inner, true, &mut budget);
                if acc == before {
                    break;
                }
            }
            acc
        }
        ASTNode::Lookahead {
            element: inner,
            positive,
        } => {
            if !contains_rule_reference_shallow(inner) {
                // Ref-free: no furthest writers inside; ignoring the byte
                // constraint over-approximates the admitted set — transparent.
                return PrefixTrieNode::epsilon();
            }
            // Refs inside: the inner attempt runs at the frontier. Bytes in
            // the inner FIRST set may let it consume (and write) unboundedly
            // deep ⇒ they must be admitted `unresolved`; other bytes fail the
            // inner at its char 1, whose entry behavior is the inner trie
            // root's exact `entry_at_node`. See the module composition notes.
            let mut fs_visiting = RuleVisit::default();
            let inner_first = branch_first_set(
                inner,
                grammar_tree,
                first_set_cache,
                &mut fs_visiting,
                depth + 1,
            );
            if inner_first.unresolved || inner_first.nullable {
                return PrefixTrieNode::unresolved_leaf();
            }
            let Some(inner_bytes) = summary_all_first_bytes(&inner_first) else {
                return PrefixTrieNode::unresolved_leaf();
            };
            if inner_bytes.is_empty() || inner_bytes.len() > PREFIX_TRIE_FANOUT_CAP {
                return PrefixTrieNode::unresolved_leaf();
            }
            let inner_trie = node_prefix_trie(
                inner,
                grammar_tree,
                first_set_cache,
                trie_cache,
                visiting_rules,
                depth + 1,
            );
            if inner_trie.unresolved {
                // The char-1 entry behavior is not exactly known.
                return PrefixTrieNode::unresolved_leaf();
            }
            let mut node = PrefixTrieNode {
                // NEGATIVE: the match can end here (inner fails at char 1).
                // POSITIVE: every match needs the inner's first byte next, so
                // the node is non-accepting and — being childless-frontier —
                // nothing ever grafts below it (frontier closed).
                accepting: !positive,
                entry_at_node: inner_trie.entry_at_node,
                regex_token_derived: inner_first.regex_token_derived
                    | inner_trie.regex_token_derived,
                truncated: inner_trie.truncated,
                ..PrefixTrieNode::default()
            };
            for byte in inner_bytes {
                node.children.insert(byte, PrefixTrieNode::unresolved_leaf());
            }
            node
        }
        ASTNode::Atom { value } => match value {
            ASTValue::Node(inner) => node_prefix_trie(
                inner,
                grammar_tree,
                first_set_cache,
                trie_cache,
                visiting_rules,
                depth + 1,
            ),
            ASTValue::Token(parts) => {
                if parts.len() < 2 {
                    return PrefixTrieNode::unresolved_leaf();
                }
                let TokenValue::String(token_type) = &parts[0];
                let TokenValue::String(token_value) = &parts[1];
                match token_type.as_str() {
                    "quoted_string" => literal_trie(token_value.as_bytes()),
                    "rule_reference" => rule_prefix_trie(
                        token_value,
                        grammar_tree,
                        first_set_cache,
                        trie_cache,
                        visiting_rules,
                    ),
                    "regex" => match regex_syntax::Parser::new()
                        .parse(token_value)
                        .ok()
                        .and_then(|hir| regex_hir_trie(&hir))
                    {
                        Some(mut trie) => {
                            trie.regex_token_derived = true;
                            trie
                        }
                        None => {
                            let mut leaf = PrefixTrieNode::unresolved_leaf();
                            leaf.regex_token_derived = true;
                            leaf
                        }
                    },
                    _ => PrefixTrieNode::unresolved_leaf(),
                }
            }
        },
    }
}

/// Linear chain for a quoted terminal (depth-capped, so at most 5 nodes).
fn literal_trie(bytes: &[u8]) -> PrefixTrieNode {
    let mut node = PrefixTrieNode::default();
    if bytes.is_empty() {
        node.accepting = true;
        return node;
    }
    if bytes.len() > PREFIX_TRIE_DEPTH_CAP {
        // The tail beyond the walk horizon is unrepresentable: build the
        // in-horizon chain ending in an `unresolved` node at the cap depth.
        let mut tail = PrefixTrieNode::unresolved_leaf();
        tail.truncated = true;
        for byte in bytes[..PREFIX_TRIE_DEPTH_CAP].iter().rev() {
            let mut parent = PrefixTrieNode {
                truncated: true,
                ..PrefixTrieNode::default()
            };
            parent.children.insert(*byte, tail);
            tail = parent;
        }
        return tail;
    }
    let mut tail = PrefixTrieNode {
        accepting: true,
        ..PrefixTrieNode::default()
    };
    for byte in bytes.iter().rev() {
        let mut parent = PrefixTrieNode::default();
        parent.children.insert(*byte, tail);
        tail = parent;
    }
    tail
}

/// A rule reference's trie: the body trie (computed under a FRESH budget so the
/// cached value is context-free) with `entry_at_node` set at the root — the
/// rule's own entry preamble executes at the reference offset even when the
/// next byte refutes every continuation. D0.1 cache-coherence discipline is
/// identical to [`rule_first_set`] / [`rule_second_byte_summary`].
fn rule_prefix_trie(
    rule_name: &str,
    grammar_tree: &HashMap<String, ASTNode>,
    first_set_cache: &mut HashMap<String, FirstSetSummary>,
    trie_cache: &mut HashMap<String, PrefixTrieNode>,
    visiting_rules: &mut RuleVisit<PrefixTrieNode>,
) -> PrefixTrieNode {
    if let Some(cached) = trie_cache.get(rule_name) {
        return cached.clone();
    }
    if let Some(transient) = visiting_rules.transient.get(rule_name) {
        return transient.clone();
    }
    if !visiting_rules.visiting.insert(rule_name.to_string()) {
        visiting_rules.tainted.insert(rule_name.to_string());
        let mut leaf = PrefixTrieNode::unresolved_leaf();
        leaf.entry_at_node = true;
        return leaf;
    }
    if visiting_rules.visiting.len() > MAX_RULE_CHAIN_DEPTH {
        visiting_rules.cap_fired = true;
        visiting_rules.visiting.remove(rule_name);
        let mut leaf = PrefixTrieNode::unresolved_leaf();
        leaf.entry_at_node = true;
        return leaf;
    }

    let mut result = if let Some(rule_ast) = grammar_tree.get(rule_name) {
        node_prefix_trie(
            rule_ast,
            grammar_tree,
            first_set_cache,
            trie_cache,
            visiting_rules,
            0,
        )
    } else {
        native_builtin_prefix_trie(rule_name)
            .unwrap_or_else(PrefixTrieNode::unresolved_leaf)
    };
    // The reference's own entry preamble (rule method / native builtin /
    // inlined frame — all carry it) executes at the reference offset.
    result.entry_at_node = true;

    visiting_rules.visiting.remove(rule_name);
    visiting_rules.tainted.remove(rule_name);
    if visiting_rules.tainted.is_empty() && !visiting_rules.cap_fired {
        trie_cache.insert(rule_name.to_string(), result.clone());
    }
    visiting_rules
        .transient
        .insert(rule_name.to_string(), result.clone());
    result
}

/// The native builtins' tries (mirrors `native_builtin_first_set` /
/// `native_builtin_second_byte_summary`). Both char builtins exceed the fanout
/// cap (128 / 243 admitted bytes) ⇒ `unresolved` — the shallower guard applies
/// (their byte sets still reach level 1 through the S1 carrier).
fn native_builtin_prefix_trie(rule_name: &str) -> Option<PrefixTrieNode> {
    match rule_name {
        "builtin_ascii_char" | "builtin_any_char" => {
            let mut leaf = PrefixTrieNode::unresolved_leaf();
            leaf.truncated = true;
            Some(leaf)
        }
        _ => None,
    }
}

/// Depth-≤4 prefix trie of a regex token's HIR (the FIRSTₖ sibling of
/// [`regex_hir_prefix`] / [`regex_hir_second`]). `None` = underivable. No
/// `entry_at_node` anywhere: a token matches inside its rule's frame and never
/// writes `furthest_position`. Zero-width assertions are transparent
/// (over-approximating the admitted set — sound, same as the level-1 walk).
fn regex_hir_trie(hir: &regex_syntax::hir::Hir) -> Option<PrefixTrieNode> {
    use regex_syntax::hir::{Class, HirKind};
    match hir.kind() {
        HirKind::Empty | HirKind::Look(_) => Some(PrefixTrieNode::epsilon()),
        HirKind::Literal(literal) => Some(literal_trie(&literal.0)),
        HirKind::Class(class) => {
            let mut bytes: BTreeSet<u8> = BTreeSet::new();
            match class {
                Class::Unicode(ranges) => {
                    for range in ranges.ranges() {
                        extend_lead_bytes_for_scalar_range(
                            &mut bytes,
                            range.start() as u32,
                            range.end() as u32,
                        );
                    }
                }
                Class::Bytes(ranges) => {
                    for range in ranges.ranges() {
                        for byte in range.start()..=range.end() {
                            bytes.insert(byte);
                        }
                    }
                }
            }
            if bytes.is_empty() || bytes.len() > PREFIX_TRIE_FANOUT_CAP {
                let mut leaf = PrefixTrieNode::unresolved_leaf();
                leaf.truncated = !bytes.is_empty();
                return Some(leaf);
            }
            let mut node = PrefixTrieNode::default();
            for byte in bytes {
                let child = if byte < 0x80 {
                    // A one-byte scalar completes the class match here.
                    PrefixTrieNode {
                        accepting: true,
                        ..PrefixTrieNode::default()
                    }
                } else {
                    // Multi-byte scalar: the continuation bytes are beyond
                    // this carrier's precision — attempt.
                    PrefixTrieNode::unresolved_leaf()
                };
                node.children.insert(byte, child);
            }
            Some(node)
        }
        HirKind::Repetition(repetition) => {
            let inner = regex_hir_trie(&repetition.sub)?;
            let min = (repetition.min as usize).min(PREFIX_TRIE_DEPTH_CAP + 1);
            let extra = repetition
                .max
                .map(|max| (max as usize).saturating_sub(repetition.min as usize))
                .unwrap_or(PREFIX_TRIE_DEPTH_CAP + 1)
                .min(PREFIX_TRIE_DEPTH_CAP + 1);
            let mut acc = PrefixTrieNode::epsilon();
            let mut budget = PrefixTrieBudget::fresh();
            for _ in 0..min {
                graft_at_frontier(&mut acc, &inner, false, &mut budget);
                if !has_open_frontier(&acc) {
                    return Some(acc);
                }
            }
            for _ in 0..extra {
                let before = acc.clone();
                graft_at_frontier(&mut acc, &inner, true, &mut budget);
                if acc == before {
                    break;
                }
            }
            Some(acc)
        }
        HirKind::Capture(capture) => regex_hir_trie(&capture.sub),
        HirKind::Concat(parts) => {
            let mut acc = PrefixTrieNode::epsilon();
            let mut budget = PrefixTrieBudget::fresh();
            for part in parts {
                let part_trie = regex_hir_trie(part)?;
                graft_at_frontier(&mut acc, &part_trie, false, &mut budget);
                if !has_open_frontier(&acc) {
                    break;
                }
            }
            Some(acc)
        }
        HirKind::Alternation(alternatives) => {
            let mut acc = PrefixTrieNode::default();
            let mut budget = PrefixTrieBudget::fresh();
            for alternative in alternatives {
                let alt_trie = regex_hir_trie(alternative)?;
                merge_node(&mut acc, &alt_trie, PREFIX_TRIE_DEPTH_CAP, &mut budget);
            }
            Some(acc)
        }
    }
}

/// The licensed, walk-ready C1 guard for one branch (the SHARED result both the
/// FIRSTₖ census lane and codegen's guard emission consume — the standing
/// census↔emission no-drift discipline).
#[derive(Debug, Clone)]
pub(crate) struct PrefixTrieGuard {
    /// Finalized root: children keyed by the LEVEL-1 byte set (so a depth-1
    /// walk is behaviorally identical to today's level-1 guard), subtrees from
    /// the per-path analysis, attempt-terminal collapse applied, and
    /// `deepest_entry_offset` fixed per node. Root refutation is always w = 0.
    pub(crate) root: PrefixTrieNode,
    /// Usable walk depth (1 = level-1-degenerate).
    pub(crate) max_depth: usize,
    /// Any cap truncated the per-path analysis somewhere.
    pub(crate) truncated: bool,
    /// The D1 global FIRST₂ layer refined at least one unresolved depth-1 leaf.
    pub(crate) d1_fallback_used: bool,
}

impl PrefixTrieGuard {
    /// The guard refines nothing beyond level 1 (every root child is an
    /// attempt-terminal leaf — unresolved OR accepting, both admit-and-stop) —
    /// the emitter keeps today's exact level-1 expression.
    pub(crate) fn is_level1_degenerate(&self) -> bool {
        self.root
            .children
            .values()
            .all(|c| c.attempt_terminal() && c.children.is_empty())
    }

    /// Distinct nonzero emulation offsets over all refutation arms (census).
    pub(crate) fn emulation_offsets(&self) -> Vec<u8> {
        fn walk(node: &PrefixTrieNode, out: &mut BTreeSet<u8>) {
            if node.attempt_terminal() {
                return;
            }
            // Refutation fires AT this node (its `_` arm) with its own w.
            if node.deepest_entry_offset > 0 {
                out.insert(node.deepest_entry_offset);
            }
            for child in node.children.values() {
                walk(child, out);
            }
        }
        let mut out = BTreeSet::new();
        walk(&self.root, &mut out);
        out.into_iter().collect()
    }
}

/// FINALIZE: attempt-terminal collapse + per-node `deepest_entry_offset`
/// (max `entry_at_node` depth on the path; depth-0 entries rewrite the
/// already-recorded `parse_start`, so the root contributes 0).
fn finalize_guard_node(node: &mut PrefixTrieNode, depth: usize, inherited: u8) {
    let own = if depth > 0 && node.entry_at_node {
        depth as u8
    } else {
        0
    };
    node.deepest_entry_offset = inherited.max(own);
    if node.attempt_terminal() {
        node.children.clear();
        return;
    }
    for child in node.children.values_mut() {
        finalize_guard_node(child, depth + 1, node.deepest_entry_offset);
    }
}

/// RGX-0078.5.j.4 K4b C1 — the branch's licensed FIRSTₖ prune guard (walk-ready
/// trie), or the NAMED level-1 reason the branch must not be byte-guarded at
/// all. Level-1 admission gates are exactly [`branch_dispatch_first_bytes`]'s
/// (so an `Err` here means the branch keeps today's unguarded emission), the
/// per-path refinement degrades path-locally to `unresolved` (never an `Err`),
/// and the D1 global FIRST₂ layer refines unresolved depth-1 leaves wherever
/// [`branch_prefix2_guard_bytes`] licenses the branch — today's D1 guard is the
/// degenerate rectangle case of the returned trie.
pub(crate) fn branch_prefix_trie_guard(
    branch: &ASTNode,
    grammar_tree: &HashMap<String, ASTNode>,
    first_set_cache: &mut HashMap<String, FirstSetSummary>,
    second_byte_cache: &mut HashMap<String, SecondByteSummary>,
    trie_cache: &mut HashMap<String, PrefixTrieNode>,
    trust_regex_token_bytes: bool,
) -> Result<PrefixTrieGuard, String> {
    // Level 1: identical admission to today's guard (S1 = the emitted byte set).
    let s1 = branch_dispatch_first_bytes(
        branch,
        grammar_tree,
        first_set_cache,
        trust_regex_token_bytes,
    )?;

    // Per-path refinement (S2). A regex-derived fact without layout trust
    // degrades S2 only — level 1 already passed its own trust gate.
    let mut visiting = RuleVisit::default();
    let mut s2 = node_prefix_trie(
        branch,
        grammar_tree,
        first_set_cache,
        trie_cache,
        &mut visiting,
        0,
    );
    if s2.regex_token_derived && !trust_regex_token_bytes {
        s2 = PrefixTrieNode::unresolved_leaf();
    }

    let mut root = PrefixTrieNode::default();
    for byte in &s1 {
        let child = if s2.unresolved {
            PrefixTrieNode::unresolved_leaf()
        } else {
            s2.children
                .get(byte)
                .cloned()
                .unwrap_or_else(PrefixTrieNode::unresolved_leaf)
        };
        root.truncated |= child.truncated;
        root.children.insert(*byte, child);
    }
    root.truncated |= s2.truncated;

    // D1 fallback layer: where the per-path analysis is level-1-weak (an
    // unresolved depth-1 leaf) but the global FIRST₂ license holds, reproduce
    // today's D1 refinement (w = 0 by its `!offset1_rule_entry` license; no
    // fanout cap — this is the proven existing emission).
    let mut d1_fallback_used = false;
    if let Ok(second_bytes) = branch_prefix2_guard_bytes(
        branch,
        grammar_tree,
        first_set_cache,
        second_byte_cache,
        trust_regex_token_bytes,
    ) {
        let mut rectangle = PrefixTrieNode::default();
        for byte in &second_bytes {
            rectangle
                .children
                .insert(*byte, PrefixTrieNode::unresolved_leaf());
        }
        for child in root.children.values_mut() {
            if child.unresolved && child.children.is_empty() {
                *child = rectangle.clone();
                d1_fallback_used = true;
            }
        }
    }

    finalize_guard_node(&mut root, 0, 0);
    let max_depth = root.max_depth();
    let truncated = root.truncated;
    Ok(PrefixTrieGuard {
        root,
        max_depth,
        truncated,
        d1_fallback_used,
    })
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

        // LANG-CAPABILITY-AUDIT.10.4: `true`/`false` were zero-width always-succeed
        // natives; they are gone, so they must now read as ordinary UNRESOLVED
        // references (an undefined rule the linter reports) rather than as nullable.
        for removed in ["true", "false"] {
            let summary = summarize(&rule_ref(removed), &tree);
            assert!(
                summary.unresolved,
                "{removed} is no longer a native builtin — it must be unresolved"
            );
            assert!(!summary.nullable, "{removed} must not read as nullable");
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

    // ==================== RGX-0078.5.j.4 K4b C1 — prefix-trie guard ====================

    fn trie_guard(node: &ASTNode, tree: &HashMap<String, ASTNode>) -> Result<PrefixTrieGuard, String> {
        let mut fs_cache = HashMap::new();
        let mut sb_cache = HashMap::new();
        let mut trie_cache = HashMap::new();
        branch_prefix_trie_guard(node, tree, &mut fs_cache, &mut sb_cache, &mut trie_cache, true)
    }

    /// C1 — the `\`-escape family (`backreference = "\" nonzero_digit …` shape):
    /// the D1 `offset1_rule_entry` refusal is replaced by the EXACT emulation
    /// license — refutation at depth 1 carries w = 1 (the offset-1 rule entry),
    /// and the per-path children are exactly the referenced rule's FIRST bytes.
    #[test]
    fn prefix_trie_backslash_escape_family_gets_exact_emulation() {
        let mut tree = HashMap::new();
        tree.insert(
            "nonzero_digit".to_string(),
            ASTNode::Or {
                alternatives: (b'1'..=b'9')
                    .map(|b| quoted_atom(&(b as char).to_string()))
                    .collect(),
            },
        );
        let branch = ASTNode::Sequence {
            elements: vec![quoted_atom("\\"), rule_ref("nonzero_digit")],
        };
        let guard = trie_guard(&branch, &tree).expect("licensed");
        assert!(guard.max_depth >= 2, "depth {} — the escape family must reach byte 2", guard.max_depth);
        let backslash = guard.root.children.get(&b'\\').expect("root admits backslash");
        assert!(!backslash.attempt_terminal(), "depth-1 refutation must be live");
        assert_eq!(
            backslash.children.keys().copied().collect::<Vec<u8>>(),
            (b'1'..=b'9').collect::<Vec<u8>>(),
            "per-path byte-2 facts are the referenced rule's FIRST bytes"
        );
        assert_eq!(
            backslash.deepest_entry_offset, 1,
            "the offset-1 rule entry is emulated exactly"
        );
        assert_eq!(guard.emulation_offsets(), vec![1]);
        // The refuted D1 mirror: the OLD license still refuses this shape.
        let mut fs = HashMap::new();
        let mut sb = HashMap::new();
        let err = branch_prefix2_guard_bytes(&branch, &tree, &mut fs, &mut sb, true).unwrap_err();
        assert!(err.contains("furthest-position parity"), "{err}");
    }

    /// C1 — the anchor family: 1-byte forms are accepting at depth 1 (per-path
    /// `len1` — they cap only their OWN path), the `\`-forms discriminate at
    /// byte 2, and no refutation needs emulation (w = 0 everywhere).
    #[test]
    fn prefix_trie_anchor_family_per_path_accepting() {
        let tree = HashMap::new();
        let branch = ASTNode::Or {
            alternatives: vec![
                quoted_atom("^"),
                quoted_atom("$"),
                quoted_atom("\\A"),
                quoted_atom("\\z"),
                quoted_atom("\\Z"),
            ],
        };
        let guard = trie_guard(&branch, &tree).expect("licensed");
        let caret = guard.root.children.get(&b'^').expect("admits ^");
        assert!(caret.accepting && caret.children.is_empty());
        let backslash = guard.root.children.get(&b'\\').expect("admits backslash");
        assert!(!backslash.attempt_terminal());
        assert_eq!(
            backslash.children.keys().copied().collect::<Vec<u8>>(),
            vec![b'A', b'Z', b'z']
        );
        assert!(backslash.children.values().all(|n| n.accepting));
        assert_eq!(guard.emulation_offsets(), Vec::<u8>::new(), "anchor is w=0 everywhere");
        // The global D1 carrier refuses this exact shape (`len1_possible`).
        let mut fs = HashMap::new();
        let mut sb = HashMap::new();
        let err = branch_prefix2_guard_bytes(&branch, &tree, &mut fs, &mut sb, true).unwrap_err();
        assert!(err.contains("1-byte match"), "{err}");
    }

    /// C1 — the `(?`-family fork: a lookbehind-style literal opener refutes at
    /// depth 3 with w = 0 (the discriminating byte precedes any rule entry),
    /// while the named-group branch's wide capture-name FIRST set exceeds the
    /// fanout cap and degrades path-locally to `unresolved` (never refuted at
    /// that depth — sound, just less precise).
    #[test]
    fn prefix_trie_group_fork_literal_vs_fanout_truncation() {
        let mut tree = HashMap::new();
        tree.insert(
            "capture_name_head".to_string(),
            ASTNode::Or {
                alternatives: (b'a'..=b'z')
                    .chain(b'A'..=b'Z')
                    .map(|b| quoted_atom(&(b as char).to_string()))
                    .collect(),
            },
        );
        tree.insert("pattern_stub".to_string(), quoted_atom("ab"));
        // lookbehind-style: 4-byte literal opener then a rule body.
        let lookbehind = ASTNode::Sequence {
            elements: vec![quoted_atom("(?<="), rule_ref("pattern_stub"), quoted_atom(")")],
        };
        let guard = trie_guard(&lookbehind, &tree).expect("licensed");
        let n1 = guard.root.children.get(&b'(').expect("(");
        let n2 = n1.children.get(&b'?').expect("?");
        let n3 = n2.children.get(&b'<').expect("<");
        assert!(!n3.attempt_terminal(), "the j=3 fork must be refutable");
        assert_eq!(n3.children.keys().copied().collect::<Vec<u8>>(), vec![b'=']);
        assert_eq!(n3.deepest_entry_offset, 0, "no entry precedes the fork byte");
        let n4 = n3.children.get(&b'=').expect("=");
        assert!(n4.attempt_terminal(), "the body beyond the walk horizon must attempt");
        // named-group-style: 3-byte opener then a wide-FIRST name rule.
        let named = ASTNode::Sequence {
            elements: vec![quoted_atom("(?<"), rule_ref("capture_name_head"), quoted_atom(">")],
        };
        let guard = trie_guard(&named, &tree).expect("licensed");
        let n3 = guard.root.children[&b'('].children[&b'?'].children
            .get(&b'<')
            .expect("<");
        assert!(
            n3.unresolved && n3.truncated,
            "the >24-byte name head must degrade path-locally: {n3:?}"
        );
    }

    /// C1 — a POSITIVE lookahead with rule references closes the frontier: the
    /// guard keeps exactly today's level-1 behavior (depth-1 degenerate), never
    /// refuting where the passed inner's furthest writes would be unbounded.
    #[test]
    fn prefix_trie_positive_refs_lookahead_stays_level1() {
        let mut tree = HashMap::new();
        tree.insert("payload".to_string(), quoted_atom("ab"));
        let branch = ASTNode::Sequence {
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
        let guard = trie_guard(&branch, &tree).expect("licensed");
        assert!(guard.is_level1_degenerate(), "{:?}", guard.root);
        assert_eq!(guard.emulation_offsets(), Vec::<u8>::new());
    }

    /// C1 — a NEGATIVE lookahead with rule references composes on: bytes in the
    /// inner FIRST set are admitted `unresolved` (the inner may consume deep),
    /// other bytes walk the continuation exactly.
    #[test]
    fn prefix_trie_negative_refs_lookahead_composes() {
        let mut tree = HashMap::new();
        tree.insert("payload".to_string(), quoted_atom("ab"));
        let branch = ASTNode::Sequence {
            elements: vec![
                ASTNode::Lookahead {
                    element: Box::new(ASTNode::Sequence {
                        elements: vec![quoted_atom("x"), rule_ref("payload")],
                    }),
                    positive: false,
                },
                quoted_atom("yz"),
            ],
        };
        let guard = trie_guard(&branch, &tree).expect("licensed");
        let x = guard.root.children.get(&b'x').expect("inner FIRST admitted");
        assert!(x.unresolved && x.children.is_empty());
        let y = guard.root.children.get(&b'y').expect("continuation admitted");
        assert_eq!(y.children.keys().copied().collect::<Vec<u8>>(), vec![b'z']);
        assert!(y.children[&b'z'].accepting);
        assert_eq!(guard.emulation_offsets(), Vec::<u8>::new());
    }

    /// C1 — greedy min-0 quantified attempt parity: a refutation at the
    /// quantified site's offset emulates the elided/failed attempt's rule entry
    /// exactly (the Q-GUARD equivalence, generalized per-path).
    #[test]
    fn prefix_trie_quantified_min0_entry_parity() {
        let mut tree = HashMap::new();
        tree.insert("opt_rule".to_string(), quoted_atom("q"));
        let branch = ASTNode::Sequence {
            elements: vec![
                quoted_atom("a"),
                ASTNode::Quantified {
                    element: Box::new(rule_ref("opt_rule")),
                    quantifier: "?".to_string(),
                },
                quoted_atom("bc"),
            ],
        };
        let guard = trie_guard(&branch, &tree).expect("licensed");
        let a = guard.root.children.get(&b'a').expect("a");
        assert!(!a.attempt_terminal());
        assert_eq!(
            a.children.keys().copied().collect::<Vec<u8>>(),
            vec![b'b', b'q'],
            "both the optional element and the continuation are admitted"
        );
        assert_eq!(
            a.deepest_entry_offset, 1,
            "the greedy opt_rule attempt at offset 1 is emulated"
        );
        assert_eq!(guard.emulation_offsets(), vec![1]);
    }

    /// C1 — the D1 global FIRST₂ layer refines an unresolved depth-1 leaf where
    /// the per-path analysis is fanout-capped but the old license holds — the
    /// existing D1 guard is the degenerate rectangle case of the new carrier.
    #[test]
    fn prefix_trie_d1_fallback_fills_rectangle() {
        let tree = HashMap::new();
        // A >24-wide byte-2 class: the per-path child degrades, the global
        // second-byte carrier licenses.
        let branch = regex_atom(r"\\[a-zA-Z0-9]");
        let guard = trie_guard(&branch, &tree).expect("licensed");
        assert!(guard.d1_fallback_used, "{:?}", guard.root);
        assert_eq!(guard.max_depth, 2);
        let backslash = guard.root.children.get(&b'\\').expect("backslash");
        assert!(!backslash.attempt_terminal());
        assert_eq!(backslash.children.len(), 26 + 26 + 10);
        assert_eq!(backslash.deepest_entry_offset, 0);
    }

    /// C1 — level-1 refusals pass through verbatim (the guard's admission gates
    /// are exactly `branch_dispatch_first_bytes`').
    #[test]
    fn prefix_trie_level1_refusals_pass_through() {
        let tree = HashMap::new();
        let nullable = ASTNode::Quantified {
            element: Box::new(quoted_atom("ab")),
            quantifier: "?".to_string(),
        };
        let err = trie_guard(&nullable, &tree).unwrap_err();
        assert!(err.contains("nullable"), "{err}");
        let err = trie_guard(&rule_ref("no_such_rule"), &tree).unwrap_err();
        assert!(err.contains("unresolved"), "{err}");
        // Regex-derived bytes without layout trust refuse at level 1.
        let mut fs_cache = HashMap::new();
        let mut sb_cache = HashMap::new();
        let mut trie_cache = HashMap::new();
        let err = branch_prefix_trie_guard(
            &regex_atom("[a-z]"),
            &tree,
            &mut fs_cache,
            &mut sb_cache,
            &mut trie_cache,
            false,
        )
        .unwrap_err();
        assert!(err.contains("layout-skipping"), "{err}");
    }

    /// C1 — a single-byte-terminal branch is depth-1 degenerate through the
    /// accepting (not unresolved) leaf shape: exactly today's level-1 guard.
    #[test]
    fn prefix_trie_single_byte_terminal_is_level1_degenerate() {
        let tree = HashMap::new();
        let guard = trie_guard(&quoted_atom("^"), &tree).expect("licensed");
        assert!(guard.is_level1_degenerate());
        assert_eq!(guard.max_depth, 1);
    }

    /// C1 — the `digits "." digits` over-prune regression (the `-0160` lib
    /// battery catch, `version_conditional` `(?(VERSION>=10.0)cat|dog)`):
    /// budget truncation on the ACCEPTING `digit+` boundary nodes must stay
    /// sticky (`unresolved`), or the later `.`-graft clears `accepting` and
    /// leaves refutation-live nodes with INCOMPLETE children — the emitted
    /// guard then demands `.` immediately after one digit. Every root child
    /// must remain attempt-terminal or admit BOTH the repetition digits and
    /// the dot.
    #[test]
    fn prefix_trie_truncated_repetition_stays_unrefutable() {
        let mut tree = HashMap::new();
        tree.insert(
            "digit_stub".to_string(),
            ASTNode::Or {
                alternatives: (b'0'..=b'9')
                    .map(|b| quoted_atom(&(b as char).to_string()))
                    .collect(),
            },
        );
        tree.insert(
            "digits_stub".to_string(),
            ASTNode::Quantified {
                element: Box::new(rule_ref("digit_stub")),
                quantifier: "+".to_string(),
            },
        );
        let branch = ASTNode::Sequence {
            elements: vec![
                rule_ref("digits_stub"),
                quoted_atom("."),
                rule_ref("digits_stub"),
            ],
        };
        let guard = trie_guard(&branch, &tree).expect("licensed");
        for (byte, child) in &guard.root.children {
            let admits_more_digits =
                (b'0'..=b'9').all(|d| child.children.contains_key(&d));
            assert!(
                child.attempt_terminal() || (admits_more_digits && child.children.contains_key(&b'.')),
                "root child {byte:#04x} became refutation-live with incomplete children: {child:?}"
            );
        }
    }

    /// C1 — D0.1 cache coherence: the per-rule trie cache admits only
    /// context-free values (the same discipline as the FIRST/second caches);
    /// a query order that computes `inner` first must not poison `entry`.
    #[test]
    fn prefix_trie_cache_stores_only_context_free_values() {
        let mut tree = HashMap::new();
        tree.insert("entry".to_string(), rule_ref("inner"));
        tree.insert(
            "inner".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    ASTNode::Sequence {
                        elements: vec![quoted_atom("a"), rule_ref("entry")],
                    },
                    quoted_atom("b"),
                ],
            },
        );
        let fresh = trie_guard(&rule_ref("entry"), &tree).expect("licensed");
        let mut fs_cache = HashMap::new();
        let mut sb_cache = HashMap::new();
        let mut trie_cache = HashMap::new();
        let _ = branch_prefix_trie_guard(
            &rule_ref("inner"),
            &tree,
            &mut fs_cache,
            &mut sb_cache,
            &mut trie_cache,
            true,
        );
        let after = branch_prefix_trie_guard(
            &rule_ref("entry"),
            &tree,
            &mut fs_cache,
            &mut sb_cache,
            &mut trie_cache,
            true,
        )
        .expect("licensed");
        assert_eq!(after.root, fresh.root, "cache order must not change the guard");
    }
}
