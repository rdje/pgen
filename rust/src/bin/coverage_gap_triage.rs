use anyhow::{Context, Result, bail};
use clap::Parser;
use pgen::ast_pipeline::stimuli_generator::{
    BranchCoverageDebt, BranchCoverageGroup, StimuliCoverageGapReport, StimuliCoverageMetrics,
};
use pgen::ast_pipeline::{ASTNode, ASTValue, PipelineConfig, RustASTPipeline, TokenValue};
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

#[derive(Debug, Parser)]
#[command(
    name = "coverage_gap_triage",
    about = "Join gap-report, coverage, and grammar AST artifacts into branch-level triage"
)]
struct Args {
    #[arg(long)]
    gap_report: String,

    #[arg(long)]
    coverage: String,

    #[arg(long)]
    grammar_ast: String,

    #[arg(long)]
    branch_id: Vec<String>,

    #[arg(long)]
    rule_name: Option<String>,

    #[arg(long, default_value_t = 20)]
    top: usize,

    #[arg(long)]
    json: bool,
}

#[derive(Debug, Clone, Serialize)]
struct DependencyRuleStatus {
    name: String,
    success_hits: u64,
    below_threshold: bool,
}

#[derive(Debug, Clone, Serialize)]
struct SiblingBranchStatus {
    branch_index: usize,
    selected_hits: u64,
    success_hits: u64,
    rendering: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct BranchTriageRecord {
    branch_id: String,
    rule_name: String,
    node_path: String,
    branch_index: usize,
    reachable: bool,
    selected_hits: u64,
    success_hits: u64,
    required_successes: u64,
    deficit: u64,
    priority_score: u64,
    reason: String,
    heuristic: String,
    group_total_branches: usize,
    branch_rendering: Option<String>,
    sibling_statuses: Vec<SiblingBranchStatus>,
    rule_references: Vec<DependencyRuleStatus>,
    uncovered_rule_references: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct TriageOutput {
    grammar_name: String,
    entry_rule: String,
    reachable_branch_debt_total: usize,
    records_shown: usize,
    records: Vec<BranchTriageRecord>,
}

#[derive(Debug, Clone)]
struct GrammarAstFile {
    grammar_name: String,
    grammar_tree: HashMap<String, ASTNode>,
}

fn load_gap_report(path: &str) -> Result<StimuliCoverageGapReport> {
    let content =
        std::fs::read_to_string(path).with_context(|| format!("failed reading {}", path))?;
    serde_json::from_str(&content).with_context(|| format!("failed parsing {}", path))
}

fn load_coverage(path: &str) -> Result<StimuliCoverageMetrics> {
    let content =
        std::fs::read_to_string(path).with_context(|| format!("failed reading {}", path))?;
    serde_json::from_str(&content).with_context(|| format!("failed parsing {}", path))
}

fn load_grammar_ast(path: &str) -> Result<GrammarAstFile> {
    let content =
        std::fs::read_to_string(path).with_context(|| format!("failed reading {}", path))?;
    let json: JsonValue =
        serde_json::from_str(&content).with_context(|| format!("failed parsing {}", path))?;
    let grammar_name = json
        .get("grammar_name")
        .and_then(JsonValue::as_str)
        .unwrap_or("grammar")
        .to_string();

    if let Some(grammar_tree_value) = json.get("grammar_tree") {
        let grammar_tree = serde_json::from_value::<HashMap<String, ASTNode>>(
            grammar_tree_value.clone(),
        )
        .with_context(|| format!("failed parsing grammar_tree from {}", path))?;
        return Ok(GrammarAstFile {
            grammar_name,
            grammar_tree,
        });
    }

    if let Some(raw_ast_value) = json.get("raw_ast") {
        let raw_ast = raw_ast_value
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("raw_ast must be an array in {}", path))?;
        let pipeline = RustASTPipeline::new(PipelineConfig::default());
        let (grammar_tree, _, _) = pipeline
            .transform_from_raw_ast(raw_ast)
            .with_context(|| format!("failed transforming raw_ast from {}", path))?;
        return Ok(GrammarAstFile {
            grammar_name,
            grammar_tree,
        });
    }

    bail!(
        "unsupported grammar AST format in {}; expected grammar_tree or raw_ast",
        path
    )
}

fn node_at_path<'a>(node: &'a ASTNode, node_path: &str) -> Option<&'a ASTNode> {
    let mut current = node;
    let mut segments = node_path.split('/');
    if segments.next()? != "root" {
        return None;
    }

    for segment in segments {
        if segment.is_empty() {
            continue;
        }
        if let Some(index_str) = segment.strip_prefix('s') {
            let index = index_str.parse::<usize>().ok()?;
            let ASTNode::Sequence { elements } = current else {
                return None;
            };
            current = elements.get(index)?;
            continue;
        }
        if let Some(index_str) = segment.strip_prefix('o') {
            let index = index_str.parse::<usize>().ok()?;
            let ASTNode::Or { alternatives } = current else {
                return None;
            };
            current = alternatives.get(index)?;
            continue;
        }
        if segment == "q" {
            let ASTNode::Quantified { element, .. } = current else {
                return None;
            };
            current = element;
            continue;
        }
        if segment == "l" {
            let ASTNode::Lookahead { element, .. } = current else {
                return None;
            };
            current = element;
            continue;
        }
        if segment == "a" {
            let ASTNode::Atom { value } = current else {
                return None;
            };
            let ASTValue::Node(node) = value else {
                return None;
            };
            current = node;
            continue;
        }
        return None;
    }

    Some(current)
}

fn or_alternatives_for_group_path<'a>(
    grammar_tree: &'a HashMap<String, ASTNode>,
    rule_name: &str,
    node_path: &str,
) -> Option<&'a [ASTNode]> {
    let rule_node = grammar_tree.get(rule_name)?;
    let group_node = node_at_path(rule_node, node_path)?;
    let ASTNode::Or { alternatives } = group_node else {
        return None;
    };
    Some(alternatives)
}

fn render_token_values(values: &[TokenValue]) -> String {
    let parts = values
        .iter()
        .map(|value| match value {
            TokenValue::String(text) => text.as_str(),
        })
        .collect::<Vec<_>>();

    if parts.len() == 2 {
        match parts[0] {
            "rule_reference" | "regex" | "quoted_string" | "operator" | "literal" => {
                return parts[1].to_string();
            }
            _ => {}
        }
    }

    parts.concat()
}

fn wrap_if_needed(node: &ASTNode, rendered: String) -> String {
    match node {
        ASTNode::Or { .. } | ASTNode::Sequence { .. } => format!("({})", rendered),
        _ => rendered,
    }
}

fn render_ast(node: &ASTNode) -> String {
    match node {
        ASTNode::Or { alternatives } => alternatives
            .iter()
            .map(render_ast)
            .collect::<Vec<_>>()
            .join(" | "),
        ASTNode::Sequence { elements } => elements
            .iter()
            .map(render_ast)
            .filter(|segment| !segment.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
        ASTNode::Atom { value } => match value {
            ASTValue::Token(values) => render_token_values(values),
            ASTValue::Node(node) => wrap_if_needed(node, render_ast(node)),
        },
        ASTNode::Quantified { element, quantifier } => {
            let inner = wrap_if_needed(element, render_ast(element));
            format!("{}{}", inner, quantifier)
        }
        ASTNode::Lookahead { element, positive } => {
            let inner = wrap_if_needed(element, render_ast(element));
            if *positive {
                format!("&{}", inner)
            } else {
                format!("!{}", inner)
            }
        }
    }
}

fn dependency_rule_statuses(
    debt: &BranchCoverageDebt,
    coverage: &StimuliCoverageMetrics,
    threshold: u64,
) -> Vec<DependencyRuleStatus> {
    let mut statuses = debt
        .rule_references
        .iter()
        .map(|name| {
            let success_hits = coverage.rule_success_hits.get(name).copied().unwrap_or(0);
            DependencyRuleStatus {
                name: name.clone(),
                success_hits,
                below_threshold: success_hits < threshold,
            }
        })
        .collect::<Vec<_>>();
    statuses.sort_by(|left, right| left.name.cmp(&right.name));
    statuses
}

fn heuristic_for_branch(debt: &BranchCoverageDebt, group: Option<&BranchCoverageGroup>) -> String {
    let Some(group) = group else {
        return "missing_group_metadata".to_string();
    };

    let any_success = group.success_counts.iter().any(|hits| *hits > 0);
    let sibling_success = group
        .success_counts
        .iter()
        .enumerate()
        .any(|(idx, hits)| idx != debt.branch_index && *hits > 0);

    if !debt.uncovered_rule_references.is_empty() {
        return "dependency_rule_debt_likely".to_string();
    }

    if debt.selected_hits == 0 {
        if any_success {
            return "selection_bias_likely".to_string();
        }
        return "group_never_selected_or_succeeds".to_string();
    }

    if sibling_success {
        return "branch_specific_failure_likely".to_string();
    }

    if any_success {
        return "mixed_group_failure".to_string();
    }

    if group.total_branches > 1 {
        return "shared_dependency_failure_likely".to_string();
    }

    "single_path_failure".to_string()
}

fn build_record(
    debt: &BranchCoverageDebt,
    coverage: &StimuliCoverageMetrics,
    grammar_tree: &HashMap<String, ASTNode>,
) -> BranchTriageRecord {
    let group = coverage.branch_groups.get(&debt.group_key);
    let alternatives = or_alternatives_for_group_path(grammar_tree, &debt.rule_name, &debt.node_path);

    let sibling_statuses = match (group, alternatives) {
        (Some(group), Some(alternatives)) => (0..group.total_branches)
            .map(|index| SiblingBranchStatus {
                branch_index: index,
                selected_hits: group.selected_counts.get(index).copied().unwrap_or(0),
                success_hits: group.success_counts.get(index).copied().unwrap_or(0),
                rendering: alternatives.get(index).map(render_ast),
            })
            .collect(),
        (Some(group), None) => (0..group.total_branches)
            .map(|index| SiblingBranchStatus {
                branch_index: index,
                selected_hits: group.selected_counts.get(index).copied().unwrap_or(0),
                success_hits: group.success_counts.get(index).copied().unwrap_or(0),
                rendering: None,
            })
            .collect(),
        _ => Vec::new(),
    };

    let branch_rendering = alternatives
        .and_then(|alts| alts.get(debt.branch_index))
        .map(render_ast);

    BranchTriageRecord {
        branch_id: debt.branch_id.clone(),
        rule_name: debt.rule_name.clone(),
        node_path: debt.node_path.clone(),
        branch_index: debt.branch_index,
        reachable: debt.reachable,
        selected_hits: debt.selected_hits,
        success_hits: debt.success_hits,
        required_successes: debt.required_successes,
        deficit: debt.deficit,
        priority_score: debt.priority_score,
        reason: debt.reason.clone(),
        heuristic: heuristic_for_branch(debt, group),
        group_total_branches: group.map(|value| value.total_branches).unwrap_or(0),
        branch_rendering,
        sibling_statuses,
        rule_references: dependency_rule_statuses(debt, coverage, debt.required_successes),
        uncovered_rule_references: debt.uncovered_rule_references.clone(),
    }
}

fn matches_filters(debt: &BranchCoverageDebt, args: &Args) -> bool {
    if !args.branch_id.is_empty() && !args.branch_id.iter().any(|value| value == &debt.branch_id) {
        return false;
    }
    if let Some(rule_name) = args.rule_name.as_deref() {
        if debt.rule_name != rule_name {
            return false;
        }
    }
    true
}

fn print_text(output: &TriageOutput) {
    println!("Coverage Gap Triage");
    println!("grammar: {}", output.grammar_name);
    println!("entry_rule: {}", output.entry_rule);
    println!("reachable_branch_debt_total: {}", output.reachable_branch_debt_total);
    println!("records_shown: {}", output.records_shown);
    println!();

    for (idx, record) in output.records.iter().enumerate() {
        println!("[{}] {}", idx + 1, record.branch_id);
        println!(
            "  rule={} path={} branch={} reason={} heuristic={}",
            record.rule_name,
            record.node_path,
            record.branch_index,
            record.reason,
            record.heuristic
        );
        println!(
            "  hits: selected={} success={} required={} deficit={} priority={}",
            record.selected_hits,
            record.success_hits,
            record.required_successes,
            record.deficit,
            record.priority_score
        );
        if let Some(rendering) = record.branch_rendering.as_deref() {
            println!("  branch: {}", rendering);
        }
        if !record.rule_references.is_empty() {
            println!("  refs:");
            for status in &record.rule_references {
                println!(
                    "    - {} (success_hits={}, below_threshold={})",
                    status.name, status.success_hits, status.below_threshold
                );
            }
        }
        if !record.uncovered_rule_references.is_empty() {
            println!(
                "  uncovered_refs: {}",
                record.uncovered_rule_references.join(", ")
            );
        }
        if !record.sibling_statuses.is_empty() {
            println!("  siblings:");
            for sibling in &record.sibling_statuses {
                let label = if sibling.branch_index == record.branch_index {
                    " <target>"
                } else {
                    ""
                };
                println!(
                    "    - #{} selected={} success={}{}{}",
                    sibling.branch_index,
                    sibling.selected_hits,
                    sibling.success_hits,
                    label,
                    sibling
                        .rendering
                        .as_ref()
                        .map(|value| format!(" :: {}", value))
                        .unwrap_or_default()
                );
            }
        }
        println!();
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.top == 0 {
        bail!("--top must be >= 1");
    }

    let gap = load_gap_report(&args.gap_report)?;
    let coverage = load_coverage(&args.coverage)?;
    let grammar = load_grammar_ast(&args.grammar_ast)?;

    let filtered = gap
        .reachable_branch_debt
        .iter()
        .filter(|debt| matches_filters(debt, &args))
        .take(args.top)
        .map(|debt| build_record(debt, &coverage, &grammar.grammar_tree))
        .collect::<Vec<_>>();

    let output = TriageOutput {
        grammar_name: grammar.grammar_name,
        entry_rule: gap.entry_rule,
        reachable_branch_debt_total: gap.reachable_branch_debt.len(),
        records_shown: filtered.len(),
        records: filtered,
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        print_text(&output);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token(text: &str) -> ASTNode {
        ASTNode::Atom {
            value: ASTValue::Token(vec![TokenValue::String(text.to_string())]),
        }
    }

    #[test]
    fn node_path_finds_nested_or_group() {
        let node = ASTNode::Sequence {
            elements: vec![ASTNode::Quantified {
                element: Box::new(ASTNode::Or {
                    alternatives: vec![token("a"), token("b")],
                }),
                quantifier: "*".to_string(),
            }],
        };
        let found = node_at_path(&node, "root/s0/q").expect("path should resolve");
        match found {
            ASTNode::Or { alternatives } => assert_eq!(alternatives.len(), 2),
            other => panic!("expected Or node, got {:?}", other),
        }
    }

    #[test]
    fn render_ast_renders_basic_alternatives() {
        let node = ASTNode::Or {
            alternatives: vec![
                ASTNode::Sequence {
                    elements: vec![token("x"), token("<="), token("y")],
                },
                ASTNode::Sequence {
                    elements: vec![token("x"), token(":="), token("y")],
                },
            ],
        };
        let rendered = render_ast(&node);
        assert!(rendered.contains("x <="));
        assert!(rendered.contains("x :="));
        assert!(rendered.contains(" | "));
    }

    #[test]
    fn heuristic_prefers_shared_dependency_failure_when_no_branch_succeeds() {
        let debt = BranchCoverageDebt {
            branch_id: "branch::x::root#1".to_string(),
            group_key: "x::root".to_string(),
            rule_name: "x".to_string(),
            node_path: "root".to_string(),
            branch_index: 1,
            reachable: true,
            selected_hits: 10,
            success_hits: 0,
            required_successes: 1,
            deficit: 1,
            priority_score: 1,
            reason: "selected_but_failed".to_string(),
            rule_references: vec![],
            uncovered_rule_references: vec![],
        };
        let group = BranchCoverageGroup {
            rule_name: "x".to_string(),
            node_path: "root".to_string(),
            total_branches: 2,
            selected_counts: vec![10, 10],
            success_counts: vec![0, 0],
        };
        assert_eq!(
            heuristic_for_branch(&debt, Some(&group)),
            "shared_dependency_failure_likely"
        );
    }
}
