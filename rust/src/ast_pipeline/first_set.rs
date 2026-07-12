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
//! `unresolved: true` is the conservative escape hatch — a regex token, an unknown
//! token type, an undefined rule reference, or a cycle/depth cutoff all set it. A
//! consumer that PRUNES on this summary (skips a branch that cannot start at the next
//! input byte) MUST treat both `unresolved` and `nullable` as "never prune": the
//! `terminals` set is only guaranteed exhaustive when `!unresolved && !nullable`.

use super::{ASTNode, ASTValue, TokenValue};
use regex::Regex;
use std::collections::{HashMap, HashSet};

/// A sound over-approximation of a node's FIRST set.
#[derive(Debug, Clone, Default)]
pub(crate) struct FirstSetSummary {
    /// Quoted terminal literals (`'X..'`) that can begin a match. A SUPERSET — every
    /// terminal that could start here is present, but the set may over-include.
    pub(crate) terminals: HashSet<String>,
    /// The node can match the empty string ⇒ it is never prunable on the next char.
    pub(crate) nullable: bool,
    /// FIRST analysis was incomplete (regex token, unknown token type, undefined /
    /// cyclic / too-deep rule reference) ⇒ `terminals` is NOT exhaustive ⇒ never prune.
    pub(crate) unresolved: bool,
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
            terminals: HashSet::new(),
            nullable: false,
            unresolved: true,
        };
    }

    match node {
        ASTNode::Sequence { elements } => {
            let mut result = FirstSetSummary {
                terminals: HashSet::new(),
                nullable: true,
                unresolved: false,
            };

            if elements.is_empty() {
                return result;
            }

            for element in elements {
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
                result.unresolved |= element_first.unresolved;
                if !element_first.nullable {
                    result.nullable = false;
                    return result;
                }
            }

            result
        }
        ASTNode::Or { alternatives } => {
            let mut result = FirstSetSummary {
                terminals: HashSet::new(),
                nullable: false,
                unresolved: false,
            };

            if alternatives.is_empty() {
                result.nullable = true;
                return result;
            }

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
                result.nullable |= alternative_first.nullable;
                result.unresolved |= alternative_first.unresolved;
            }

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
                    terminals: HashSet::new(),
                    nullable: false,
                    unresolved: true,
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
                        unresolved: false,
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
                    let nullable = Regex::new(token_value)
                        .ok()
                        .and_then(|re| re.find(""))
                        .map(|m| m.start() == 0 && m.end() == 0)
                        .unwrap_or(false);
                    FirstSetSummary {
                        terminals: HashSet::new(),
                        nullable,
                        unresolved: true,
                    }
                }
                _ => FirstSetSummary {
                    terminals: HashSet::new(),
                    nullable: false,
                    unresolved: true,
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
            terminals: HashSet::new(),
            nullable: false,
            unresolved: true,
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
    } else {
        FirstSetSummary {
            terminals: HashSet::new(),
            nullable: false,
            unresolved: true,
        }
    };

    visiting_rules.remove(rule_name);
    first_set_cache.insert(rule_name.to_string(), result.clone());
    result
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
