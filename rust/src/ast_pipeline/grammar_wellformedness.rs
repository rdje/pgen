//! PARSE-SOTA.8 / adoption A1 (PGEN-PARSE-SOTA-0005): static grammar well-formedness.
//!
//! Ford's PEG well-formedness condition (Bryan Ford, "Parsing Expression Grammars", POPL
//! 2004, §3.5–3.6; see docs/book/src/academic-foundations.md): a **left-recursive** rule —
//! one reachable from itself through only nullable left-edge positions, so it can recurse
//! WITHOUT consuming input — loops forever / stack-overflows at runtime. PGEN grammars
//! deliberately avoid left recursion via the iterative `next (OP next)*` idiom, so this is
//! a guard: detect a left-recursive rule from the compiled grammar IR up front and let the
//! generator REJECT it (via the DIAG-SEVERITY always-on `pgen_error!` channel) instead of
//! hanging. PARSER-AGNOSTIC; PURE analysis (no I/O, no generation, mutates nothing).
//!
//! Soundness for a *rejection* check = NO false positives: we are conservative about
//! nullability (terminals are treated as never-nullable) so we never over-report a
//! well-formed grammar as left-recursive.

use super::{ASTNode, ASTValue, TokenValue, parse_quantifier_bounds};
use std::collections::{HashMap, HashSet};

/// A detected grammar well-formedness issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WellformednessIssue {
    /// `rule` is directly or indirectly left-recursive. `cycle` records one left-recursion
    /// path (starting and ending at `rule`) for the diagnostic message.
    LeftRecursive { rule: String, cycle: Vec<String> },
}

impl WellformednessIssue {
    pub fn message(&self) -> String {
        match self {
            WellformednessIssue::LeftRecursive { rule, cycle } => format!(
                "grammar well-formedness: rule '{}' is left-recursive (cycle: {}) — it can recurse without consuming input and will not terminate; rewrite using the iterative `next (OP next)*` idiom",
                rule,
                cycle.join(" -> ")
            ),
        }
    }
}

/// Extract the referenced rule name from a token atom that is a `rule_reference`, else None.
fn referenced_rule(parts: &[TokenValue]) -> Option<&str> {
    if parts.len() < 2 {
        return None;
    }
    let TokenValue::String(token_type) = &parts[0] else {
        return None;
    };
    let TokenValue::String(token_value) = &parts[1] else {
        return None;
    };
    if token_type == "rule_reference" {
        Some(token_value.as_str())
    } else {
        None
    }
}

/// Can `node` match the empty string (succeed without consuming input)? Uses the current
/// per-rule `nullable` estimates (fixpoint). Conservative: a terminal token is treated as
/// never-nullable (consumes input), so we never over-report nullability.
fn node_nullable(node: &ASTNode, nullable: &HashMap<String, bool>) -> bool {
    match node {
        ASTNode::Or { alternatives } => {
            alternatives.iter().any(|a| node_nullable(a, nullable))
        }
        ASTNode::Sequence { elements } => {
            elements.iter().all(|e| node_nullable(e, nullable))
        }
        ASTNode::Quantified { element, quantifier } => {
            let (min, _) = parse_quantifier_bounds(quantifier).unwrap_or((0, None));
            min == 0 || node_nullable(element, nullable)
        }
        // A syntactic predicate consumes no input.
        ASTNode::Lookahead { .. } => true,
        ASTNode::Atom { value } => match value {
            ASTValue::Node(inner) => node_nullable(inner, nullable),
            ASTValue::Token(parts) => match referenced_rule(parts) {
                Some(rule) => nullable.get(rule).copied().unwrap_or(false),
                None => false, // a terminal lexeme consumes input
            },
        },
    }
}

/// Fixpoint over the grammar: which rules are nullable.
fn compute_nullable(grammar: &HashMap<String, ASTNode>, rule_order: &[String]) -> HashMap<String, bool> {
    let mut nullable: HashMap<String, bool> = HashMap::new();
    loop {
        let mut changed = false;
        for rule in rule_order {
            let Some(body) = grammar.get(rule) else { continue };
            let value = node_nullable(body, &nullable);
            if nullable.get(rule).copied().unwrap_or(false) != value {
                nullable.insert(rule.clone(), value);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    nullable
}

/// Rules referenced at the LEFT EDGE of `node` — i.e. reachable before any input is
/// necessarily consumed. A `Sequence` extends past a leading element only while that
/// element is nullable; a quantifier/lookahead exposes its element at the left edge.
fn leftmost_refs(node: &ASTNode, nullable: &HashMap<String, bool>, out: &mut HashSet<String>) {
    match node {
        ASTNode::Or { alternatives } => {
            for a in alternatives {
                leftmost_refs(a, nullable, out);
            }
        }
        ASTNode::Sequence { elements } => {
            for e in elements {
                leftmost_refs(e, nullable, out);
                if !node_nullable(e, nullable) {
                    break; // this element must consume → later elements are not left-edge
                }
            }
        }
        ASTNode::Quantified { element, .. } => leftmost_refs(element, nullable, out),
        ASTNode::Lookahead { element, .. } => leftmost_refs(element, nullable, out),
        ASTNode::Atom { value } => match value {
            ASTValue::Node(inner) => leftmost_refs(inner, nullable, out),
            ASTValue::Token(parts) => {
                if let Some(rule) = referenced_rule(parts) {
                    out.insert(rule.to_string());
                }
            }
        },
    }
}

/// Detect every left-recursive rule in the grammar (direct or indirect). PURE analysis.
/// Returns one `LeftRecursive` issue per offending rule (deterministic order via
/// `rule_order`), each with a concrete recursion cycle for the diagnostic.
pub fn detect_left_recursion(
    grammar: &HashMap<String, ASTNode>,
    rule_order: &[String],
) -> Vec<WellformednessIssue> {
    let nullable = compute_nullable(grammar, rule_order);

    // Left-edge call graph: rule -> rules it can reach at the left edge.
    let mut left_edge: HashMap<String, HashSet<String>> = HashMap::new();
    for rule in rule_order {
        let Some(body) = grammar.get(rule) else { continue };
        let mut refs = HashSet::new();
        leftmost_refs(body, &nullable, &mut refs);
        // Keep only references to defined rules.
        refs.retain(|r| grammar.contains_key(r));
        left_edge.insert(rule.clone(), refs);
    }

    let mut issues = Vec::new();
    for start in rule_order {
        if !grammar.contains_key(start.as_str()) {
            continue;
        }
        // DFS from `start`; report if we return to `start` along left edges.
        if let Some(cycle) = find_left_cycle(start, &left_edge) {
            issues.push(WellformednessIssue::LeftRecursive {
                rule: start.clone(),
                cycle,
            });
        }
    }
    issues
}

/// DFS that returns a cycle path `start -> … -> start` if `start` is left-recursive.
fn find_left_cycle(start: &str, left_edge: &HashMap<String, HashSet<String>>) -> Option<Vec<String>> {
    let mut stack: Vec<String> = vec![start.to_string()];
    let mut visited: HashSet<String> = HashSet::new();
    if dfs(start, start, left_edge, &mut stack, &mut visited) {
        Some(stack)
    } else {
        None
    }
}

fn dfs(
    start: &str,
    current: &str,
    left_edge: &HashMap<String, HashSet<String>>,
    stack: &mut Vec<String>,
    visited: &mut HashSet<String>,
) -> bool {
    let Some(next_set) = left_edge.get(current) else {
        return false;
    };
    // Deterministic neighbour order.
    let mut neighbours: Vec<&String> = next_set.iter().collect();
    neighbours.sort();
    for next in neighbours {
        if next == start {
            stack.push(start.to_string());
            return true; // closed the cycle back to start
        }
        if visited.insert(next.clone()) {
            stack.push(next.clone());
            if dfs(start, next, left_edge, stack, visited) {
                return true;
            }
            stack.pop();
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule_ref(name: &str) -> ASTNode {
        ASTNode::Atom {
            value: ASTValue::Token(vec![
                TokenValue::String("rule_reference".into()),
                TokenValue::String(name.into()),
            ]),
        }
    }
    fn token(kind: &str, value: &str) -> ASTNode {
        ASTNode::Atom {
            value: ASTValue::Token(vec![
                TokenValue::String(kind.into()),
                TokenValue::String(value.into()),
            ]),
        }
    }
    fn seq(elems: Vec<ASTNode>) -> ASTNode {
        ASTNode::Sequence { elements: elems }
    }
    fn or(alts: Vec<ASTNode>) -> ASTNode {
        ASTNode::Or { alternatives: alts }
    }

    #[test]
    fn detects_direct_left_recursion() {
        // expr := expr "+" term | term ; term := "n"
        let mut g = HashMap::new();
        g.insert("expr".into(), or(vec![seq(vec![rule_ref("expr"), token("op", "+"), rule_ref("term")]), rule_ref("term")]));
        g.insert("term".into(), token("string", "n"));
        let order: Vec<String> = vec!["expr".into(), "term".into()];
        let issues = detect_left_recursion(&g, &order);
        assert!(
            issues.iter().any(|i| matches!(i, WellformednessIssue::LeftRecursive { rule, .. } if rule == "expr")),
            "direct left recursion in expr must be detected: {issues:?}"
        );
        assert!(
            !issues.iter().any(|i| matches!(i, WellformednessIssue::LeftRecursive { rule, .. } if rule == "term")),
            "term is not left-recursive"
        );
    }

    #[test]
    fn detects_indirect_left_recursion() {
        // a := b ; b := a "z" | "w"
        let mut g = HashMap::new();
        g.insert("a".into(), rule_ref("b"));
        g.insert("b".into(), or(vec![seq(vec![rule_ref("a"), token("string", "z")]), token("string", "w")]));
        let order: Vec<String> = vec!["a".into(), "b".into()];
        let issues = detect_left_recursion(&g, &order);
        assert!(!issues.is_empty(), "indirect a->b->a left recursion must be detected: {issues:?}");
    }

    #[test]
    fn detects_left_recursion_through_nullable_prefix() {
        // a := opt a | "x" ; opt := "p"?   (opt is nullable, so `a` sits at the left edge)
        let mut g = HashMap::new();
        g.insert("a".into(), or(vec![seq(vec![rule_ref("opt"), rule_ref("a")]), token("string", "x")]));
        g.insert("opt".into(), ASTNode::Quantified { element: Box::new(token("string", "p")), quantifier: "?".into() });
        let order: Vec<String> = vec!["a".into(), "opt".into()];
        let issues = detect_left_recursion(&g, &order);
        assert!(
            issues.iter().any(|i| matches!(i, WellformednessIssue::LeftRecursive { rule, .. } if rule == "a")),
            "left recursion through a nullable prefix must be detected: {issues:?}"
        );
    }

    #[test]
    fn no_false_positive_on_well_formed_grammar() {
        // The iterative idiom PGEN uses: expr := term (op term)* ; term := "n" ; op := "+"
        let mut g = HashMap::new();
        g.insert(
            "expr".into(),
            seq(vec![
                rule_ref("term"),
                ASTNode::Quantified {
                    element: Box::new(seq(vec![rule_ref("op"), rule_ref("term")])),
                    quantifier: "*".into(),
                },
            ]),
        );
        g.insert("term".into(), token("string", "n"));
        g.insert("op".into(), token("op", "+"));
        let order: Vec<String> = vec!["expr".into(), "term".into(), "op".into()];
        let issues = detect_left_recursion(&g, &order);
        assert!(issues.is_empty(), "the iterative idiom must NOT be flagged: {issues:?}");
    }

    #[test]
    fn non_left_recursive_chain_is_clean() {
        // a := "k" b ; b := "m" a   (each consumes before recursing → not left-recursive)
        let mut g = HashMap::new();
        g.insert("a".into(), seq(vec![token("string", "k"), rule_ref("b")]));
        g.insert("b".into(), seq(vec![token("string", "m"), rule_ref("a")]));
        let order: Vec<String> = vec!["a".into(), "b".into()];
        let issues = detect_left_recursion(&g, &order);
        assert!(issues.is_empty(), "a consuming prefix breaks left recursion: {issues:?}");
    }
}
