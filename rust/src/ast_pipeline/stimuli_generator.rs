use super::{
    ASTNode, ASTValue, Annotations, SemanticAnnotation, SemanticAssociativity,
    SemanticBranchPolicy, SemanticTokenClass, SemanticValueConstraints, TokenValue, TraceLevel,
    TraceVerbosity, UnifiedSemanticAST, extract_semantic_directive, global_trace_verbosity,
    normalize_semantic_scalar, parse_canonical_transform_expression, parse_semantic_bool,
    parse_semantic_branch_priorities, parse_semantic_charset, parse_semantic_constraint_expression,
    parse_semantic_coverage_target_weight, parse_semantic_deterministic_group,
    parse_semantic_group_label, parse_semantic_implication, parse_semantic_len_bounds,
    parse_semantic_numeric_bounds, parse_semantic_pattern, parse_semantic_reference_list,
    parse_semantic_string_list, parse_semantic_token_class, stimuli_hint_for_target_type,
};
use anyhow::{Context, Result, anyhow};
use rand::distributions::{Distribution, WeightedIndex};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use regex_syntax::hir::{Class, Hir, HirKind, Literal, Repetition};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::panic::Location;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStimuliMode {
    Baseline,
    RecoveryBiased,
    NearSyncNegative,
}

impl Default for RecoveryStimuliMode {
    fn default() -> Self {
        Self::Baseline
    }
}

#[derive(Debug, Clone)]
pub struct StimuliConfig {
    pub seed: Option<u64>,
    pub max_depth: usize,
    pub max_repeat: usize,
    pub max_rule_visits: usize,
    pub recovery_mode: RecoveryStimuliMode,
    pub enforce_word_boundary_spacing: bool,
    pub trace_verbosity: TraceVerbosity,
}

impl Default for StimuliConfig {
    fn default() -> Self {
        Self {
            seed: None,
            max_depth: 24,
            max_repeat: 4,
            max_rule_visits: 8,
            recovery_mode: RecoveryStimuliMode::Baseline,
            enforce_word_boundary_spacing: false,
            trace_verbosity: global_trace_verbosity(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BranchCoverageGroup {
    pub rule_name: String,
    pub node_path: String,
    pub total_branches: usize,
    pub selected_counts: Vec<u64>,
    pub success_counts: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StimuliCoverageMetrics {
    pub grammar_name: String,
    pub total_rules: usize,
    pub total_branch_groups: usize,
    pub total_branches: usize,
    pub sample_attempts: u64,
    pub sample_successes: u64,
    pub sample_errors: u64,
    pub rule_success_hits: HashMap<String, u64>,
    pub branch_groups: HashMap<String, BranchCoverageGroup>,
}

#[derive(Debug, Clone)]
struct StimuliCoverageSuccessSnapshot {
    rule_success_hits: HashMap<String, u64>,
    branch_success_counts: HashMap<String, Vec<u64>>,
}

impl StimuliCoverageMetrics {
    fn new(
        grammar_name: String,
        total_rules: usize,
        rule_success_hits: HashMap<String, u64>,
        branch_groups: HashMap<String, BranchCoverageGroup>,
    ) -> Self {
        let mut metrics = Self {
            grammar_name,
            total_rules,
            total_branch_groups: 0,
            total_branches: 0,
            sample_attempts: 0,
            sample_successes: 0,
            sample_errors: 0,
            rule_success_hits,
            branch_groups,
        };
        metrics.recompute_totals();
        metrics
    }

    fn recompute_totals(&mut self) {
        self.total_branch_groups = self.branch_groups.len();
        self.total_branches = self
            .branch_groups
            .values()
            .map(|group| group.total_branches)
            .sum();
    }

    pub fn merge_from(&mut self, other: &Self) -> Result<()> {
        if self.grammar_name != other.grammar_name {
            return Err(anyhow!(
                "Cannot merge coverage for different grammars: '{}' vs '{}'",
                self.grammar_name,
                other.grammar_name
            ));
        }

        self.sample_attempts += other.sample_attempts;
        self.sample_successes += other.sample_successes;
        self.sample_errors += other.sample_errors;

        for (rule_name, hits) in &other.rule_success_hits {
            *self.rule_success_hits.entry(rule_name.clone()).or_insert(0) += hits;
        }

        for (group_key, other_group) in &other.branch_groups {
            let group = self
                .branch_groups
                .entry(group_key.clone())
                .or_insert_with(|| BranchCoverageGroup {
                    rule_name: other_group.rule_name.clone(),
                    node_path: other_group.node_path.clone(),
                    total_branches: other_group.total_branches,
                    selected_counts: vec![0; other_group.total_branches],
                    success_counts: vec![0; other_group.total_branches],
                });

            if group.total_branches < other_group.total_branches {
                group.selected_counts.resize(other_group.total_branches, 0);
                group.success_counts.resize(other_group.total_branches, 0);
                group.total_branches = other_group.total_branches;
            }

            for (idx, count) in other_group.selected_counts.iter().enumerate() {
                if idx >= group.selected_counts.len() {
                    group.selected_counts.push(0);
                }
                group.selected_counts[idx] += count;
            }
            for (idx, count) in other_group.success_counts.iter().enumerate() {
                if idx >= group.success_counts.len() {
                    group.success_counts.push(0);
                }
                group.success_counts[idx] += count;
            }
        }

        if self.total_rules < other.total_rules {
            self.total_rules = other.total_rules;
        }
        self.recompute_totals();
        Ok(())
    }

    fn record_sample_attempt(&mut self, succeeded: bool) {
        self.sample_attempts += 1;
        if succeeded {
            self.sample_successes += 1;
        } else {
            self.sample_errors += 1;
        }
    }

    fn record_rule_success(&mut self, rule_name: &str) {
        *self
            .rule_success_hits
            .entry(rule_name.to_string())
            .or_insert(0) += 1;
    }

    fn ensure_group_entry(
        &mut self,
        group_key: &str,
        rule_name: &str,
        node_path: &str,
        total_branches: usize,
    ) {
        self.branch_groups
            .entry(group_key.to_string())
            .or_insert_with(|| BranchCoverageGroup {
                rule_name: rule_name.to_string(),
                node_path: node_path.to_string(),
                total_branches,
                selected_counts: vec![0; total_branches],
                success_counts: vec![0; total_branches],
            });
    }

    fn record_branch_selected(
        &mut self,
        group_key: &str,
        rule_name: &str,
        node_path: &str,
        total_branches: usize,
        branch_idx: usize,
    ) {
        self.ensure_group_entry(group_key, rule_name, node_path, total_branches);
        if let Some(group) = self.branch_groups.get_mut(group_key) {
            if group.selected_counts.len() <= branch_idx {
                group.selected_counts.resize(branch_idx + 1, 0);
            }
            if group.success_counts.len() <= branch_idx {
                group.success_counts.resize(branch_idx + 1, 0);
            }
            if group.total_branches < total_branches {
                group.total_branches = total_branches;
            }
            group.selected_counts[branch_idx] += 1;
        }
    }

    fn record_branch_success(
        &mut self,
        group_key: &str,
        rule_name: &str,
        node_path: &str,
        total_branches: usize,
        branch_idx: usize,
    ) {
        self.ensure_group_entry(group_key, rule_name, node_path, total_branches);
        if let Some(group) = self.branch_groups.get_mut(group_key) {
            if group.success_counts.len() <= branch_idx {
                group.success_counts.resize(branch_idx + 1, 0);
            }
            if group.selected_counts.len() <= branch_idx {
                group.selected_counts.resize(branch_idx + 1, 0);
            }
            if group.total_branches < total_branches {
                group.total_branches = total_branches;
            }
            group.success_counts[branch_idx] += 1;
        }
    }

    fn snapshot_success_state(&self) -> StimuliCoverageSuccessSnapshot {
        StimuliCoverageSuccessSnapshot {
            rule_success_hits: self.rule_success_hits.clone(),
            branch_success_counts: self
                .branch_groups
                .iter()
                .map(|(group_key, group)| (group_key.clone(), group.success_counts.clone()))
                .collect(),
        }
    }

    fn restore_success_state(&mut self, snapshot: &StimuliCoverageSuccessSnapshot) {
        self.rule_success_hits = snapshot.rule_success_hits.clone();
        for (group_key, group) in &mut self.branch_groups {
            if let Some(success_counts) = snapshot.branch_success_counts.get(group_key) {
                group.success_counts = success_counts.clone();
            } else {
                group.success_counts = vec![0; group.total_branches];
            }
            if group.success_counts.len() < group.total_branches {
                group.success_counts.resize(group.total_branches, 0);
            }
        }
    }

    pub fn covered_rules(&self) -> usize {
        self.rule_success_hits
            .values()
            .filter(|hits| **hits > 0)
            .count()
    }

    pub fn covered_branches(&self) -> usize {
        self.branch_groups
            .values()
            .map(|group| {
                group
                    .success_counts
                    .iter()
                    .filter(|hits| **hits > 0)
                    .count()
            })
            .sum()
    }

    pub fn rule_coverage_percent(&self) -> f64 {
        if self.total_rules == 0 {
            0.0
        } else {
            (self.covered_rules() as f64 * 100.0) / self.total_rules as f64
        }
    }

    pub fn branch_coverage_percent(&self) -> f64 {
        if self.total_branches == 0 {
            0.0
        } else {
            (self.covered_branches() as f64 * 100.0) / self.total_branches as f64
        }
    }

    pub fn summary_line(&self) -> String {
        format!(
            "Stimuli coverage: rules {}/{} ({:.2}%), branches {}/{} ({:.2}%), sample_successes={}/{}",
            self.covered_rules(),
            self.total_rules,
            self.rule_coverage_percent(),
            self.covered_branches(),
            self.total_branches,
            self.branch_coverage_percent(),
            self.sample_successes,
            self.sample_attempts
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StimuliCoverageTargetType {
    Rule,
    Branch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCoverageDebt {
    pub rule_name: String,
    pub reachable: bool,
    pub success_hits: u64,
    pub required_successes: u64,
    pub deficit: u64,
    pub priority_score: u64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchCoverageDebt {
    pub branch_id: String,
    pub group_key: String,
    pub rule_name: String,
    pub node_path: String,
    pub branch_index: usize,
    pub reachable: bool,
    pub selected_hits: u64,
    pub success_hits: u64,
    pub required_successes: u64,
    pub deficit: u64,
    pub priority_score: u64,
    pub reason: String,
    pub rule_references: Vec<String>,
    pub uncovered_rule_references: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StimuliCoverageTarget {
    pub id: String,
    pub target_type: StimuliCoverageTargetType,
    pub rule_name: String,
    pub node_path: Option<String>,
    pub branch_index: Option<usize>,
    pub reachable: bool,
    pub required_successes: u64,
    pub current_successes: u64,
    pub deficit: u64,
    pub priority_score: u64,
    pub reason: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageDebtSummary {
    pub required_successes_per_target: u64,
    pub sample_attempts: u64,
    pub sample_successes: u64,
    pub sample_errors: u64,
    pub total_rules: usize,
    pub reachable_rules: usize,
    pub unreachable_rules: usize,
    pub covered_rules: usize,
    pub covered_reachable_rules: usize,
    pub reachable_rules_at_threshold: usize,
    pub total_branches: usize,
    pub reachable_branches: usize,
    pub unreachable_branches: usize,
    pub covered_branches: usize,
    pub covered_reachable_branches: usize,
    pub reachable_branches_at_threshold: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StimuliCoverageGapReport {
    pub grammar_name: String,
    pub entry_rule: String,
    pub summary: CoverageDebtSummary,
    pub reachable_rule_debt: Vec<RuleCoverageDebt>,
    pub unreachable_rule_debt: Vec<RuleCoverageDebt>,
    pub reachable_branch_debt: Vec<BranchCoverageDebt>,
    pub unreachable_branch_debt: Vec<BranchCoverageDebt>,
    pub targets: Vec<StimuliCoverageTarget>,
}

impl StimuliCoverageGapReport {
    pub fn to_pretty_text(&self) -> String {
        let mut out = String::new();
        out.push_str("=== Stimuli Coverage Gap Report ===\n");
        out.push_str(&format!("Grammar: {}\n", self.grammar_name));
        out.push_str(&format!("Entry rule: {}\n", self.entry_rule));
        out.push_str(&format!(
            "Threshold: {} successful hits per target\n\n",
            self.summary.required_successes_per_target
        ));

        out.push_str("Summary\n");
        out.push_str(&format!(
            "- Samples: attempts={} successes={} errors={}\n",
            self.summary.sample_attempts, self.summary.sample_successes, self.summary.sample_errors
        ));
        out.push_str(&format!(
            "- Rules: covered {}/{} | reachable {} (at-threshold {}) | unreachable {}\n",
            self.summary.covered_rules,
            self.summary.total_rules,
            self.summary.reachable_rules,
            self.summary.reachable_rules_at_threshold,
            self.summary.unreachable_rules
        ));
        out.push_str(&format!(
            "- Branches: covered {}/{} | reachable {} (at-threshold {}) | unreachable {}\n",
            self.summary.covered_branches,
            self.summary.total_branches,
            self.summary.reachable_branches,
            self.summary.reachable_branches_at_threshold,
            self.summary.unreachable_branches
        ));
        out.push_str(&format!("- Actionable targets: {}\n\n", self.targets.len()));

        out.push_str("Reachable Rule Debt\n");
        if self.reachable_rule_debt.is_empty() {
            out.push_str("- none\n");
        } else {
            for debt in &self.reachable_rule_debt {
                out.push_str(&format!(
                    "- {} | hits={} required={} deficit={} priority={} reason={}\n",
                    debt.rule_name,
                    debt.success_hits,
                    debt.required_successes,
                    debt.deficit,
                    debt.priority_score,
                    debt.reason
                ));
            }
        }
        out.push('\n');

        out.push_str("Reachable Branch Debt\n");
        if self.reachable_branch_debt.is_empty() {
            out.push_str("- none\n");
        } else {
            for debt in &self.reachable_branch_debt {
                out.push_str(&format!(
                    "- {} | selected={} success={} required={} deficit={} priority={} reason={} refs=[{}] uncovered_refs=[{}]\n",
                    debt.branch_id,
                    debt.selected_hits,
                    debt.success_hits,
                    debt.required_successes,
                    debt.deficit,
                    debt.priority_score,
                    debt.reason,
                    debt.rule_references.join(","),
                    debt.uncovered_rule_references.join(",")
                ));
            }
        }
        out.push('\n');

        out.push_str("Unreachable Rule Debt\n");
        if self.unreachable_rule_debt.is_empty() {
            out.push_str("- none\n");
        } else {
            for debt in &self.unreachable_rule_debt {
                out.push_str(&format!(
                    "- {} | hits={} required={} deficit={} reason={}\n",
                    debt.rule_name,
                    debt.success_hits,
                    debt.required_successes,
                    debt.deficit,
                    debt.reason
                ));
            }
        }
        out.push('\n');

        out.push_str("Unreachable Branch Debt\n");
        if self.unreachable_branch_debt.is_empty() {
            out.push_str("- none\n");
        } else {
            for debt in &self.unreachable_branch_debt {
                out.push_str(&format!(
                    "- {} | selected={} success={} required={} deficit={} reason={}\n",
                    debt.branch_id,
                    debt.selected_hits,
                    debt.success_hits,
                    debt.required_successes,
                    debt.deficit,
                    debt.reason
                ));
            }
        }
        out.push('\n');

        out.push_str("Target Plan\n");
        if self.targets.is_empty() {
            out.push_str("- none\n");
        } else {
            for target in &self.targets {
                let location = if let (Some(node_path), Some(branch_index)) =
                    (&target.node_path, target.branch_index)
                {
                    format!("{}::{}#{}", target.rule_name, node_path, branch_index)
                } else {
                    target.rule_name.clone()
                };
                out.push_str(&format!(
                    "- {} | type={:?} location={} current={} required={} deficit={} priority={} reason={} depends_on=[{}]\n",
                    target.id,
                    target.target_type,
                    location,
                    target.current_successes,
                    target.required_successes,
                    target.deficit,
                    target.priority_score,
                    target.reason,
                    target.depends_on.join(",")
                ));
            }
        }

        out
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetCoverageStatus {
    pub id: String,
    pub target_type: StimuliCoverageTargetType,
    pub rule_name: String,
    pub node_path: Option<String>,
    pub branch_index: Option<usize>,
    pub current_successes: u64,
    pub required_successes: u64,
    pub remaining_successes: u64,
    pub priority_score: u64,
    pub reason: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetDriveSummary {
    pub entry_rule: String,
    pub attempts: usize,
    pub generation_successes: usize,
    pub generation_errors: usize,
    pub total_targets: usize,
    pub applied_targets: usize,
    pub resolved_targets: usize,
    pub unresolved_targets: Vec<TargetCoverageStatus>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TargetDriveValidationSummary {
    pub validated_outputs: usize,
    pub accepted_outputs: usize,
    pub rejected_outputs: usize,
    pub alternate_entry_attempts: usize,
    pub alternate_entry_accepted_outputs: usize,
    pub alternate_entry_rejected_outputs: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct DependencyProbeCandidate {
    rule_name: String,
    blocked_targets: usize,
    blocked_remaining_successes: u64,
    max_target_priority: u64,
    dependency_rule_deficit: u64,
    dependency_rule_successes: u64,
}

impl TargetDriveSummary {
    pub fn summary_line(&self) -> String {
        format!(
            "Target-driven generation: resolved {}/{} targets in {} attempts (generation_successes={}, generation_errors={})",
            self.resolved_targets,
            self.total_targets,
            self.attempts,
            self.generation_successes,
            self.generation_errors
        )
    }
}

#[derive(Debug, Clone, Default)]
struct ActiveTargetPlan {
    rule_thresholds: HashMap<String, u64>,
    branch_thresholds: HashMap<String, HashMap<usize, u64>>,
}

#[derive(Debug, Clone, Default)]
struct StimuliRelationalConstraintPolicy {
    constraint_expression: Option<String>,
    requires_references: Vec<String>,
    implication: Option<(String, String)>,
}

#[derive(Debug, Clone, Copy, Default)]
struct StimuliCoverageSteeringPolicy {
    coverage_target_weight: u64,
    critical_path: bool,
}

#[derive(Debug, Clone, Copy, Default)]
struct StimuliNegativeCasePolicy {
    invalid_case: bool,
    negative: bool,
}

#[derive(Debug, Clone, Default)]
struct StimuliDeterminismPartitionPolicy {
    enabled: bool,
    group_label: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct StimuliTokenSteeringPolicy {
    token_class: Option<SemanticTokenClass>,
    charset_pattern: Option<String>,
    explicit_pattern: Option<String>,
}

pub struct StimuliGenerator<'a> {
    grammar_name: String,
    grammar_tree: &'a HashMap<String, ASTNode>,
    rule_order: &'a [String],
    annotations: Option<&'a Annotations>,
    config: StimuliConfig,
    rng: StdRng,
    coverage: StimuliCoverageMetrics,
    target_plan: ActiveTargetPlan,
    deterministic_partition_counters: HashMap<String, u64>,
}

impl<'a> StimuliGenerator<'a> {
    pub fn new(
        grammar_name: String,
        grammar_tree: &'a HashMap<String, ASTNode>,
        rule_order: &'a [String],
        annotations: Option<&'a Annotations>,
        config: StimuliConfig,
    ) -> Self {
        let rng = if let Some(seed) = config.seed {
            StdRng::seed_from_u64(seed)
        } else {
            StdRng::from_entropy()
        };

        let mut rule_success_hits = HashMap::new();
        for rule_name in rule_order {
            rule_success_hits.entry(rule_name.clone()).or_insert(0);
        }
        for rule_name in grammar_tree.keys() {
            rule_success_hits.entry(rule_name.clone()).or_insert(0);
        }

        let mut branch_groups = HashMap::new();
        for rule_name in rule_order {
            if let Some(rule_node) = grammar_tree.get(rule_name) {
                Self::collect_branch_groups(rule_name, rule_node, "root", &mut branch_groups);
            }
        }
        for (rule_name, rule_node) in grammar_tree {
            if !rule_order.iter().any(|r| r == rule_name) {
                Self::collect_branch_groups(rule_name, rule_node, "root", &mut branch_groups);
            }
        }

        let coverage = StimuliCoverageMetrics::new(
            grammar_name.clone(),
            rule_success_hits.len(),
            rule_success_hits,
            branch_groups,
        );

        Self {
            grammar_name,
            grammar_tree,
            rule_order,
            annotations,
            config,
            rng,
            coverage,
            target_plan: ActiveTargetPlan::default(),
            deterministic_partition_counters: HashMap::new(),
        }
    }

    #[track_caller]
    fn trace(&self, level: TraceLevel, args: fmt::Arguments<'_>) {
        if !self.config.trace_verbosity.allows(level) {
            return;
        }
        let caller = Location::caller();
        crate::ast_pipeline::trace_log(
            level,
            caller.file(),
            caller.line(),
            module_path!(),
            format_args!("🎲 [stimuli:{}] {}", self.grammar_name, args),
        );
    }

    pub fn coverage_metrics(&self) -> &StimuliCoverageMetrics {
        &self.coverage
    }

    pub fn merge_coverage_metrics(&mut self, other: &StimuliCoverageMetrics) -> Result<()> {
        self.coverage.merge_from(other)
    }

    pub fn generate_gap_report(
        &self,
        entry_rule: Option<&str>,
        required_successes: u64,
    ) -> Result<StimuliCoverageGapReport> {
        let threshold = required_successes.max(1);
        let resolved_entry = self.resolve_entry_rule(entry_rule)?;
        let reachable_rules = self.compute_reachable_rules(&resolved_entry);

        let mut all_rule_names: HashSet<String> =
            self.coverage.rule_success_hits.keys().cloned().collect();
        for rule_name in self.grammar_tree.keys() {
            all_rule_names.insert(rule_name.clone());
        }
        for rule_name in self.rule_order {
            all_rule_names.insert(rule_name.clone());
        }
        let mut all_rules: Vec<String> = all_rule_names.into_iter().collect();
        all_rules.sort();

        let mut reachable_rule_debt = Vec::new();
        let mut unreachable_rule_debt = Vec::new();
        let mut reachable_branch_debt = Vec::new();
        let mut unreachable_branch_debt = Vec::new();
        let mut targets = Vec::new();

        let total_rules = all_rules.len();
        let reachable_rules_count = all_rules
            .iter()
            .filter(|rule_name| reachable_rules.contains(rule_name.as_str()))
            .count();
        let unreachable_rules_count = total_rules.saturating_sub(reachable_rules_count);
        let covered_rules = all_rules
            .iter()
            .filter(|rule_name| {
                self.coverage
                    .rule_success_hits
                    .get(*rule_name)
                    .copied()
                    .unwrap_or(0)
                    > 0
            })
            .count();
        let covered_reachable_rules = all_rules
            .iter()
            .filter(|rule_name| {
                reachable_rules.contains(rule_name.as_str())
                    && self
                        .coverage
                        .rule_success_hits
                        .get(*rule_name)
                        .copied()
                        .unwrap_or(0)
                        > 0
            })
            .count();
        let reachable_rules_at_threshold = all_rules
            .iter()
            .filter(|rule_name| {
                reachable_rules.contains(rule_name.as_str())
                    && self
                        .coverage
                        .rule_success_hits
                        .get(*rule_name)
                        .copied()
                        .unwrap_or(0)
                        >= threshold
            })
            .count();

        for rule_name in &all_rules {
            let success_hits = self
                .coverage
                .rule_success_hits
                .get(rule_name)
                .copied()
                .unwrap_or(0);
            let deficit = threshold.saturating_sub(success_hits);
            if deficit == 0 {
                continue;
            }

            let reachable = reachable_rules.contains(rule_name.as_str());
            let reason = if reachable {
                if success_hits == 0 {
                    "never_hit"
                } else {
                    "below_threshold"
                }
            } else {
                "unreachable_from_entry"
            };
            let mut priority_score = 0u64;
            if reachable {
                priority_score = 800u64
                    .saturating_add(deficit.saturating_mul(100))
                    .saturating_add(if success_hits == 0 { 160 } else { 0 });
                priority_score =
                    priority_score.saturating_add(self.semantic_coverage_priority_bonus(rule_name));
            }

            let debt = RuleCoverageDebt {
                rule_name: rule_name.clone(),
                reachable,
                success_hits,
                required_successes: threshold,
                deficit,
                priority_score,
                reason: reason.to_string(),
            };

            if reachable {
                targets.push(StimuliCoverageTarget {
                    id: Self::rule_target_id(rule_name),
                    target_type: StimuliCoverageTargetType::Rule,
                    rule_name: rule_name.clone(),
                    node_path: None,
                    branch_index: None,
                    reachable: true,
                    required_successes: threshold,
                    current_successes: success_hits,
                    deficit,
                    priority_score,
                    reason: reason.to_string(),
                    depends_on: Vec::new(),
                });
                reachable_rule_debt.push(debt);
            } else {
                unreachable_rule_debt.push(debt);
            }
        }

        let mut group_keys: Vec<String> = self.coverage.branch_groups.keys().cloned().collect();
        group_keys.sort();

        let mut total_branches = 0usize;
        let mut reachable_branches = 0usize;
        let mut covered_branches = 0usize;
        let mut covered_reachable_branches = 0usize;
        let mut reachable_branches_at_threshold = 0usize;

        for group_key in group_keys {
            let Some(group) = self.coverage.branch_groups.get(&group_key) else {
                continue;
            };
            let reachable = reachable_rules.contains(group.rule_name.as_str());
            total_branches = total_branches.saturating_add(group.total_branches);
            if reachable {
                reachable_branches = reachable_branches.saturating_add(group.total_branches);
            }
            let branch_nodes =
                self.or_alternatives_for_group_path(&group.rule_name, &group.node_path);

            for branch_idx in 0..group.total_branches {
                let selected_hits = group.selected_counts.get(branch_idx).copied().unwrap_or(0);
                let success_hits = group.success_counts.get(branch_idx).copied().unwrap_or(0);
                let deficit = threshold.saturating_sub(success_hits);

                if success_hits > 0 {
                    covered_branches = covered_branches.saturating_add(1);
                    if reachable {
                        covered_reachable_branches = covered_reachable_branches.saturating_add(1);
                    }
                }
                if reachable && success_hits >= threshold {
                    reachable_branches_at_threshold =
                        reachable_branches_at_threshold.saturating_add(1);
                }
                if deficit == 0 {
                    continue;
                }

                let mut rule_refs = Vec::new();
                let mut uncovered_rule_refs = Vec::new();
                if let Some(alternatives) = branch_nodes {
                    if let Some(branch_node) = alternatives.get(branch_idx) {
                        let mut refs = HashSet::new();
                        self.collect_rule_references(branch_node, &mut refs);
                        rule_refs = refs.into_iter().collect();
                        rule_refs.sort();
                        uncovered_rule_refs = rule_refs
                            .iter()
                            .filter(|rule_name| {
                                self.coverage
                                    .rule_success_hits
                                    .get(rule_name.as_str())
                                    .copied()
                                    .unwrap_or(0)
                                    < threshold
                            })
                            .cloned()
                            .collect();
                    }
                }

                let reason = if reachable {
                    if selected_hits == 0 {
                        "never_selected"
                    } else if success_hits == 0 {
                        "selected_but_failed"
                    } else {
                        "below_threshold"
                    }
                } else {
                    "unreachable_from_entry"
                };

                let mut priority_score = 0u64;
                if reachable {
                    priority_score = 1000u64
                        .saturating_add(deficit.saturating_mul(120))
                        .saturating_add(if selected_hits == 0 { 200 } else { 0 })
                        .saturating_add(if success_hits == 0 { 120 } else { 0 })
                        .saturating_add(
                            u64::try_from(uncovered_rule_refs.len())
                                .unwrap_or(0)
                                .saturating_mul(24),
                        );
                    priority_score = priority_score.saturating_add(
                        self.semantic_coverage_priority_bonus(group.rule_name.as_str()),
                    );
                    let semantic_ref_bonus = rule_refs
                        .iter()
                        .map(|rule_name| self.semantic_coverage_priority_bonus(rule_name))
                        .fold(0u64, u64::saturating_add)
                        .min(2048);
                    priority_score = priority_score.saturating_add(semantic_ref_bonus);
                }

                let branch_id =
                    Self::branch_target_id(&group.rule_name, &group.node_path, branch_idx);
                let debt = BranchCoverageDebt {
                    branch_id: branch_id.clone(),
                    group_key: group_key.clone(),
                    rule_name: group.rule_name.clone(),
                    node_path: group.node_path.clone(),
                    branch_index: branch_idx,
                    reachable,
                    selected_hits,
                    success_hits,
                    required_successes: threshold,
                    deficit,
                    priority_score,
                    reason: reason.to_string(),
                    rule_references: rule_refs.clone(),
                    uncovered_rule_references: uncovered_rule_refs.clone(),
                };

                if reachable {
                    targets.push(StimuliCoverageTarget {
                        id: branch_id,
                        target_type: StimuliCoverageTargetType::Branch,
                        rule_name: group.rule_name.clone(),
                        node_path: Some(group.node_path.clone()),
                        branch_index: Some(branch_idx),
                        reachable: true,
                        required_successes: threshold,
                        current_successes: success_hits,
                        deficit,
                        priority_score,
                        reason: reason.to_string(),
                        depends_on: uncovered_rule_refs,
                    });
                    reachable_branch_debt.push(debt);
                } else {
                    unreachable_branch_debt.push(debt);
                }
            }
        }

        let unreachable_branches = total_branches.saturating_sub(reachable_branches);
        reachable_rule_debt.sort_by(|a, b| {
            b.priority_score
                .cmp(&a.priority_score)
                .then_with(|| a.rule_name.cmp(&b.rule_name))
        });
        unreachable_rule_debt.sort_by(|a, b| a.rule_name.cmp(&b.rule_name));
        reachable_branch_debt.sort_by(|a, b| {
            b.priority_score
                .cmp(&a.priority_score)
                .then_with(|| a.branch_id.cmp(&b.branch_id))
        });
        unreachable_branch_debt.sort_by(|a, b| a.branch_id.cmp(&b.branch_id));
        targets.sort_by(|a, b| {
            b.priority_score
                .cmp(&a.priority_score)
                .then_with(|| a.id.cmp(&b.id))
        });

        Ok(StimuliCoverageGapReport {
            grammar_name: self.grammar_name.clone(),
            entry_rule: resolved_entry,
            summary: CoverageDebtSummary {
                required_successes_per_target: threshold,
                sample_attempts: self.coverage.sample_attempts,
                sample_successes: self.coverage.sample_successes,
                sample_errors: self.coverage.sample_errors,
                total_rules,
                reachable_rules: reachable_rules_count,
                unreachable_rules: unreachable_rules_count,
                covered_rules,
                covered_reachable_rules,
                reachable_rules_at_threshold,
                total_branches,
                reachable_branches,
                unreachable_branches,
                covered_branches,
                covered_reachable_branches,
                reachable_branches_at_threshold,
            },
            reachable_rule_debt,
            unreachable_rule_debt,
            reachable_branch_debt,
            unreachable_branch_debt,
            targets,
        })
    }

    pub fn apply_targets(&mut self, targets: &[StimuliCoverageTarget]) -> usize {
        self.clear_targets();
        let mut applied = 0usize;

        for target in targets.iter().filter(|target| target.reachable) {
            let threshold = target.required_successes.max(1);
            match target.target_type {
                StimuliCoverageTargetType::Rule => {
                    if self.grammar_tree.contains_key(target.rule_name.as_str()) {
                        self.target_plan
                            .rule_thresholds
                            .entry(target.rule_name.clone())
                            .and_modify(|existing| *existing = (*existing).max(threshold))
                            .or_insert(threshold);
                        applied = applied.saturating_add(1);
                    }
                }
                StimuliCoverageTargetType::Branch => {
                    let Some(node_path) = target.node_path.as_ref() else {
                        continue;
                    };
                    let Some(branch_index) = target.branch_index else {
                        continue;
                    };
                    let group_key = Self::branch_group_key(target.rule_name.as_str(), node_path);
                    if let Some(group) = self.coverage.branch_groups.get(&group_key) {
                        if branch_index < group.total_branches {
                            self.target_plan
                                .branch_thresholds
                                .entry(group_key)
                                .or_default()
                                .entry(branch_index)
                                .and_modify(|existing| *existing = (*existing).max(threshold))
                                .or_insert(threshold);
                            applied = applied.saturating_add(1);
                        }
                    }
                }
            }
        }

        applied
    }

    pub fn clear_targets(&mut self) {
        self.target_plan = ActiveTargetPlan::default();
    }

    pub fn evaluate_target_statuses(
        &self,
        targets: &[StimuliCoverageTarget],
    ) -> Vec<TargetCoverageStatus> {
        let mut statuses = Vec::new();
        for target in targets.iter().filter(|target| target.reachable) {
            let current_successes = self.current_target_successes(target);
            let required_successes = target.required_successes.max(1);
            let remaining_successes = required_successes.saturating_sub(current_successes);
            if remaining_successes == 0 {
                continue;
            }
            statuses.push(TargetCoverageStatus {
                id: target.id.clone(),
                target_type: target.target_type.clone(),
                rule_name: target.rule_name.clone(),
                node_path: target.node_path.clone(),
                branch_index: target.branch_index,
                current_successes,
                required_successes,
                remaining_successes,
                priority_score: target.priority_score,
                reason: target.reason.clone(),
                depends_on: target.depends_on.clone(),
            });
        }

        statuses.sort_by(|a, b| {
            b.priority_score
                .cmp(&a.priority_score)
                .then_with(|| a.id.cmp(&b.id))
        });
        statuses
    }

    pub fn generate_until_targets(
        &mut self,
        entry_rule: Option<&str>,
        targets: &[StimuliCoverageTarget],
        max_attempts: usize,
    ) -> Result<(Vec<String>, TargetDriveSummary)> {
        let resolved_entry = self.resolve_entry_rule(entry_rule)?;
        let applicable_targets: Vec<StimuliCoverageTarget> = targets
            .iter()
            .filter(|target| target.reachable)
            .cloned()
            .collect();
        let applied_targets = self.apply_targets(&applicable_targets);

        let mut outputs = Vec::new();
        let mut attempts = 0usize;
        let mut generation_successes = 0usize;
        let mut generation_errors = 0usize;
        let mut best_remaining = applicable_targets.len();
        let mut stagnant_iterations = 0usize;

        while attempts < max_attempts {
            let pending = self.evaluate_target_statuses(&applicable_targets);
            if pending.is_empty() {
                break;
            }

            if pending.len() < best_remaining {
                best_remaining = pending.len();
                stagnant_iterations = 0;
            } else {
                stagnant_iterations = stagnant_iterations.saturating_add(1);
            }

            let probe_threshold = self.target_probe_threshold(&pending);
            let generation_entry = if stagnant_iterations >= probe_threshold {
                self.select_target_probe_rule(&pending, &resolved_entry)
                    .unwrap_or_else(|| resolved_entry.clone())
            } else {
                resolved_entry.clone()
            };

            attempts = attempts.saturating_add(1);
            match self.generate_from_entry(&generation_entry) {
                Ok(sample) => {
                    generation_successes = generation_successes.saturating_add(1);
                    if generation_entry == resolved_entry {
                        outputs.push(sample);
                    }
                }
                Err(_) => {
                    generation_errors = generation_errors.saturating_add(1);
                }
            }
        }

        let unresolved_targets = self.evaluate_target_statuses(&applicable_targets);
        let total_targets = applicable_targets.len();
        let resolved_targets = total_targets.saturating_sub(unresolved_targets.len());
        self.clear_targets();

        Ok((
            outputs,
            TargetDriveSummary {
                entry_rule: resolved_entry,
                attempts,
                generation_successes,
                generation_errors,
                total_targets,
                applied_targets,
                resolved_targets,
                unresolved_targets,
            },
        ))
    }

    pub fn generate_until_targets_with_filter<F>(
        &mut self,
        entry_rule: Option<&str>,
        targets: &[StimuliCoverageTarget],
        max_attempts: usize,
        mut output_filter: F,
    ) -> Result<(
        Vec<String>,
        TargetDriveSummary,
        TargetDriveValidationSummary,
    )>
    where
        F: FnMut(&str) -> Result<bool>,
    {
        let resolved_entry = self.resolve_entry_rule(entry_rule)?;
        let applicable_targets: Vec<StimuliCoverageTarget> = targets
            .iter()
            .filter(|target| target.reachable)
            .cloned()
            .collect();
        let applied_targets = self.apply_targets(&applicable_targets);

        let mut outputs = Vec::new();
        let mut attempts = 0usize;
        let mut generation_successes = 0usize;
        let mut generation_errors = 0usize;
        let mut best_remaining = applicable_targets.len();
        let mut stagnant_iterations = 0usize;
        let mut validation_summary = TargetDriveValidationSummary::default();

        while attempts < max_attempts {
            let pending = self.evaluate_target_statuses(&applicable_targets);
            if pending.is_empty() {
                break;
            }

            if pending.len() < best_remaining {
                best_remaining = pending.len();
                stagnant_iterations = 0;
            } else {
                stagnant_iterations = stagnant_iterations.saturating_add(1);
            }

            let probe_threshold =
                self.target_probe_threshold_for_validation(&pending, &validation_summary);
            let generation_entry = if stagnant_iterations >= probe_threshold {
                self.select_target_probe_rule_for_validation(
                    &pending,
                    &resolved_entry,
                    &validation_summary,
                )
                .unwrap_or_else(|| resolved_entry.clone())
            } else {
                resolved_entry.clone()
            };

            attempts = attempts.saturating_add(1);
            let success_snapshot = self.coverage.snapshot_success_state();
            match self.generate_from_entry(&generation_entry) {
                Ok(sample) => {
                    generation_successes = generation_successes.saturating_add(1);
                    if generation_entry != resolved_entry {
                        validation_summary.alternate_entry_attempts = validation_summary
                            .alternate_entry_attempts
                            .saturating_add(1);
                    }
                    let accepted = output_filter(&sample)?;
                    if generation_entry == resolved_entry {
                        validation_summary.validated_outputs =
                            validation_summary.validated_outputs.saturating_add(1);
                    }
                    if !accepted {
                        self.coverage.restore_success_state(&success_snapshot);
                        if generation_entry == resolved_entry {
                            validation_summary.rejected_outputs =
                                validation_summary.rejected_outputs.saturating_add(1);
                        } else {
                            validation_summary.alternate_entry_rejected_outputs =
                                validation_summary
                                    .alternate_entry_rejected_outputs
                                    .saturating_add(1);
                        }
                        continue;
                    }

                    if generation_entry == resolved_entry {
                        validation_summary.accepted_outputs =
                            validation_summary.accepted_outputs.saturating_add(1);
                        outputs.push(sample);
                    } else {
                        validation_summary.alternate_entry_accepted_outputs = validation_summary
                            .alternate_entry_accepted_outputs
                            .saturating_add(1);
                    }
                }
                Err(_) => {
                    generation_errors = generation_errors.saturating_add(1);
                }
            }
        }

        let unresolved_targets = self.evaluate_target_statuses(&applicable_targets);
        let total_targets = applicable_targets.len();
        let resolved_targets = total_targets.saturating_sub(unresolved_targets.len());
        self.clear_targets();

        Ok((
            outputs,
            TargetDriveSummary {
                entry_rule: resolved_entry,
                attempts,
                generation_successes,
                generation_errors,
                total_targets,
                applied_targets,
                resolved_targets,
                unresolved_targets,
            },
            validation_summary,
        ))
    }

    fn select_target_probe_rule(
        &self,
        pending: &[TargetCoverageStatus],
        resolved_entry: &str,
    ) -> Option<String> {
        self.best_dependency_probe_candidate(pending, resolved_entry)
            .map(|candidate| candidate.rule_name)
            .or_else(|| {
                pending.iter().find_map(|status| {
                    if matches!(status.target_type, StimuliCoverageTargetType::Branch)
                        && status.rule_name != resolved_entry
                        && self.grammar_tree.contains_key(status.rule_name.as_str())
                    {
                        Some(status.rule_name.clone())
                    } else {
                        None
                    }
                })
            })
            .or_else(|| {
                pending.iter().find_map(|status| {
                    if status.rule_name != resolved_entry
                        && self.grammar_tree.contains_key(status.rule_name.as_str())
                    {
                        Some(status.rule_name.clone())
                    } else {
                        None
                    }
                })
            })
    }

    fn select_target_probe_rule_for_validation(
        &self,
        pending: &[TargetCoverageStatus],
        resolved_entry: &str,
        validation_summary: &TargetDriveValidationSummary,
    ) -> Option<String> {
        if let Some(candidate) = self.best_dependency_probe_candidate(pending, resolved_entry) {
            if !Self::validation_prefers_primary_entry(validation_summary)
                || Self::validation_dependency_probe_is_worthy(&candidate)
            {
                return Some(candidate.rule_name);
            }
        }

        if Self::validation_prefers_primary_entry(validation_summary) {
            return None;
        }

        pending
            .iter()
            .find_map(|status| {
                if matches!(status.target_type, StimuliCoverageTargetType::Branch)
                    && status.rule_name != resolved_entry
                    && self.grammar_tree.contains_key(status.rule_name.as_str())
                {
                    Some(status.rule_name.clone())
                } else {
                    None
                }
            })
            .or_else(|| {
                pending.iter().find_map(|status| {
                    if status.rule_name != resolved_entry
                        && self.grammar_tree.contains_key(status.rule_name.as_str())
                    {
                        Some(status.rule_name.clone())
                    } else {
                        None
                    }
                })
            })
    }

    fn best_dependency_probe_candidate(
        &self,
        pending: &[TargetCoverageStatus],
        resolved_entry: &str,
    ) -> Option<DependencyProbeCandidate> {
        let mut candidates: HashMap<String, DependencyProbeCandidate> = HashMap::new();
        for status in pending {
            for rule_name in &status.depends_on {
                if rule_name == resolved_entry
                    || !self.grammar_tree.contains_key(rule_name.as_str())
                {
                    continue;
                }
                let dependency_rule_deficit = self.rule_target_deficit(rule_name.as_str());
                if dependency_rule_deficit == 0 {
                    continue;
                }
                let dependency_rule_successes = self
                    .coverage
                    .rule_success_hits
                    .get(rule_name)
                    .copied()
                    .unwrap_or(0);
                let entry = candidates.entry(rule_name.clone()).or_insert_with(|| {
                    DependencyProbeCandidate {
                        rule_name: rule_name.clone(),
                        dependency_rule_deficit,
                        dependency_rule_successes,
                        ..DependencyProbeCandidate::default()
                    }
                });
                entry.blocked_targets = entry.blocked_targets.saturating_add(1);
                entry.blocked_remaining_successes = entry
                    .blocked_remaining_successes
                    .saturating_add(status.remaining_successes);
                entry.max_target_priority = entry.max_target_priority.max(status.priority_score);
                entry.dependency_rule_deficit =
                    entry.dependency_rule_deficit.max(dependency_rule_deficit);
                entry.dependency_rule_successes = entry
                    .dependency_rule_successes
                    .min(dependency_rule_successes);
            }
        }

        candidates.into_values().max_by(|left, right| {
            left.dependency_rule_deficit
                .cmp(&right.dependency_rule_deficit)
                .then_with(|| {
                    left.blocked_remaining_successes
                        .cmp(&right.blocked_remaining_successes)
                })
                .then_with(|| left.blocked_targets.cmp(&right.blocked_targets))
                .then_with(|| {
                    right
                        .dependency_rule_successes
                        .cmp(&left.dependency_rule_successes)
                })
                .then_with(|| left.max_target_priority.cmp(&right.max_target_priority))
                .then_with(|| right.rule_name.cmp(&left.rule_name))
        })
    }

    fn target_probe_threshold(&self, pending: &[TargetCoverageStatus]) -> usize {
        let mut threshold = 32usize;
        for status in pending {
            let Some(node_path) = status.node_path.as_ref() else {
                continue;
            };
            let Some(branch_index) = status.branch_index else {
                continue;
            };
            let group_key = Self::branch_group_key(status.rule_name.as_str(), node_path);
            let Some(group) = self.coverage.branch_groups.get(&group_key) else {
                continue;
            };
            let selected_hits = group
                .selected_counts
                .get(branch_index)
                .copied()
                .unwrap_or(0);
            let success_hits = group.success_counts.get(branch_index).copied().unwrap_or(0);
            let throttle = Self::target_branch_failure_throttle(selected_hits, success_hits);
            let has_probeable_dependency = status.depends_on.iter().any(|rule_name| {
                self.rule_target_deficit(rule_name.as_str()) > 0
                    && self.grammar_tree.contains_key(rule_name.as_str())
            });
            if throttle >= 8 {
                if has_probeable_dependency {
                    return 8;
                }
                threshold = threshold.min(16);
                continue;
            }
            if throttle > 1 {
                if has_probeable_dependency {
                    threshold = threshold.min(16);
                } else {
                    threshold = threshold.min(24);
                }
            }
        }

        threshold
    }

    fn validation_prefers_primary_entry(validation_summary: &TargetDriveValidationSummary) -> bool {
        let alternate_attempts = validation_summary.alternate_entry_attempts;
        if alternate_attempts < 8 {
            return false;
        }

        let primary_attempts = validation_summary.validated_outputs;
        let alternate_dominates = alternate_attempts >= primary_attempts.saturating_mul(2).max(8);
        if !alternate_dominates {
            return false;
        }

        Self::target_branch_failure_throttle(
            u64::try_from(alternate_attempts).unwrap_or(u64::MAX),
            u64::try_from(validation_summary.alternate_entry_accepted_outputs).unwrap_or(u64::MAX),
        ) > 1
    }

    fn validation_dependency_probe_is_worthy(candidate: &DependencyProbeCandidate) -> bool {
        candidate.dependency_rule_successes == 0
            || candidate.dependency_rule_deficit >= 2
            || candidate.blocked_targets >= 2
            || candidate.blocked_remaining_successes >= 2
    }

    fn target_probe_threshold_for_validation(
        &self,
        pending: &[TargetCoverageStatus],
        validation_summary: &TargetDriveValidationSummary,
    ) -> usize {
        let base = self.target_probe_threshold(pending);
        if !Self::validation_prefers_primary_entry(validation_summary) {
            return base;
        }

        let mut threshold = base.saturating_add(8);
        if validation_summary.rejected_outputs > validation_summary.accepted_outputs {
            threshold = threshold.saturating_add(8);
        }
        threshold.min(64)
    }

    #[cfg(test)]
    fn select_target_probe_rule_legacy(
        &self,
        pending: &[TargetCoverageStatus],
        resolved_entry: &str,
    ) -> Option<String> {
        pending
            .iter()
            .find_map(|status| {
                if matches!(status.target_type, StimuliCoverageTargetType::Branch)
                    && status.rule_name != resolved_entry
                    && self.grammar_tree.contains_key(status.rule_name.as_str())
                {
                    Some(status.rule_name.clone())
                } else {
                    None
                }
            })
            .or_else(|| {
                pending.iter().find_map(|status| {
                    if status.rule_name != resolved_entry
                        && self.grammar_tree.contains_key(status.rule_name.as_str())
                    {
                        Some(status.rule_name.clone())
                    } else {
                        None
                    }
                })
            })
    }

    fn rule_target_id(rule_name: &str) -> String {
        format!("rule::{}", rule_name)
    }

    fn branch_group_key(rule_name: &str, node_path: &str) -> String {
        format!("{}::{}", rule_name, node_path)
    }

    fn branch_target_id(rule_name: &str, node_path: &str, branch_index: usize) -> String {
        format!("branch::{}::{}#{}", rule_name, node_path, branch_index)
    }

    fn branch_success_hits(&self, group_key: &str, branch_index: usize) -> u64 {
        self.coverage
            .branch_groups
            .get(group_key)
            .and_then(|group| group.success_counts.get(branch_index).copied())
            .unwrap_or(0)
    }

    fn current_target_successes(&self, target: &StimuliCoverageTarget) -> u64 {
        match target.target_type {
            StimuliCoverageTargetType::Rule => self
                .coverage
                .rule_success_hits
                .get(target.rule_name.as_str())
                .copied()
                .unwrap_or(0),
            StimuliCoverageTargetType::Branch => {
                let Some(node_path) = target.node_path.as_ref() else {
                    return 0;
                };
                let Some(branch_index) = target.branch_index else {
                    return 0;
                };
                let group_key = Self::branch_group_key(target.rule_name.as_str(), node_path);
                self.branch_success_hits(&group_key, branch_index)
            }
        }
    }

    fn rule_target_deficit(&self, rule_name: &str) -> u64 {
        let Some(required) = self.target_plan.rule_thresholds.get(rule_name).copied() else {
            return 0;
        };
        let current = self
            .coverage
            .rule_success_hits
            .get(rule_name)
            .copied()
            .unwrap_or(0);
        required.saturating_sub(current)
    }

    fn branch_target_deficit(&self, group_key: &str, branch_index: usize) -> u64 {
        let Some(required) = self
            .target_plan
            .branch_thresholds
            .get(group_key)
            .and_then(|targets| targets.get(&branch_index))
            .copied()
        else {
            return 0;
        };
        required.saturating_sub(self.branch_success_hits(group_key, branch_index))
    }

    fn compute_reachable_rules(&self, entry_rule: &str) -> HashSet<String> {
        let mut reachable = HashSet::new();
        let mut pending = vec![entry_rule.to_string()];

        while let Some(rule_name) = pending.pop() {
            if !reachable.insert(rule_name.clone()) {
                continue;
            }
            let Some(rule_node) = self.grammar_tree.get(rule_name.as_str()) else {
                continue;
            };
            let mut refs = HashSet::new();
            self.collect_rule_references(rule_node, &mut refs);
            for referenced_rule in refs {
                if self.grammar_tree.contains_key(referenced_rule.as_str())
                    && !reachable.contains(referenced_rule.as_str())
                {
                    pending.push(referenced_rule);
                }
            }
        }

        reachable
    }

    fn collect_rule_references(&self, node: &ASTNode, out: &mut HashSet<String>) {
        match node {
            ASTNode::Or { alternatives } => {
                for alternative in alternatives {
                    self.collect_rule_references(alternative, out);
                }
            }
            ASTNode::Sequence { elements } => {
                for element in elements {
                    self.collect_rule_references(element, out);
                }
            }
            ASTNode::Quantified { element, .. } => {
                self.collect_rule_references(element, out);
            }
            ASTNode::Atom { value } => match value {
                ASTValue::Node(node) => self.collect_rule_references(node, out),
                ASTValue::Token(parts) => {
                    if let Some((token_type, token_value)) = Self::extract_token_pair(parts) {
                        if token_type == "rule_reference" {
                            out.insert(token_value.to_string());
                        }
                    }
                }
            },
        }
    }

    fn node_at_path<'b>(&self, node: &'b ASTNode, node_path: &str) -> Option<&'b ASTNode> {
        let mut current = node;
        for segment in node_path.split('/') {
            if segment.is_empty() || segment == "root" {
                continue;
            }
            if segment == "q" {
                let ASTNode::Quantified { element, .. } = current else {
                    return None;
                };
                current = element.as_ref();
                continue;
            }
            if segment == "a" {
                let ASTNode::Atom { value } = current else {
                    return None;
                };
                let ASTValue::Node(node) = value else {
                    return None;
                };
                current = node.as_ref();
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
            return None;
        }
        Some(current)
    }

    fn or_alternatives_for_group_path(
        &self,
        rule_name: &str,
        node_path: &str,
    ) -> Option<&Vec<ASTNode>> {
        let rule_node = self.grammar_tree.get(rule_name)?;
        let group_node = self.node_at_path(rule_node, node_path)?;
        let ASTNode::Or { alternatives } = group_node else {
            return None;
        };
        Some(alternatives)
    }

    fn collect_branch_groups(
        rule_name: &str,
        node: &ASTNode,
        node_path: &str,
        groups: &mut HashMap<String, BranchCoverageGroup>,
    ) {
        match node {
            ASTNode::Or { alternatives } => {
                let group_key = format!("{}::{}", rule_name, node_path);
                groups
                    .entry(group_key)
                    .or_insert_with(|| BranchCoverageGroup {
                        rule_name: rule_name.to_string(),
                        node_path: node_path.to_string(),
                        total_branches: alternatives.len(),
                        selected_counts: vec![0; alternatives.len()],
                        success_counts: vec![0; alternatives.len()],
                    });

                for (idx, alternative) in alternatives.iter().enumerate() {
                    let alt_path = format!("{}/o{}", node_path, idx);
                    Self::collect_branch_groups(rule_name, alternative, &alt_path, groups);
                }
            }
            ASTNode::Sequence { elements } => {
                for (idx, element) in elements.iter().enumerate() {
                    let element_path = format!("{}/s{}", node_path, idx);
                    Self::collect_branch_groups(rule_name, element, &element_path, groups);
                }
            }
            ASTNode::Quantified { element, .. } => {
                let quantified_path = format!("{}/q", node_path);
                Self::collect_branch_groups(rule_name, element, &quantified_path, groups);
            }
            ASTNode::Atom { value } => {
                if let ASTValue::Node(node) = value {
                    let atom_path = format!("{}/a", node_path);
                    Self::collect_branch_groups(rule_name, node, &atom_path, groups);
                }
            }
        }
    }

    pub fn generate_many(&mut self, count: usize, entry_rule: Option<&str>) -> Result<Vec<String>> {
        let resolved_entry = self.resolve_entry_rule(entry_rule)?;
        self.trace(
            TraceLevel::Low,
            format_args!(
                "Starting batch generation: count={} entry_rule='{}' mode={:?} max_depth={} max_repeat={} max_rule_visits={}",
                count,
                resolved_entry,
                self.config.recovery_mode,
                self.config.max_depth,
                self.config.max_repeat,
                self.config.max_rule_visits
            ),
        );
        let mut outputs = Vec::with_capacity(count);
        for idx in 0..count {
            self.trace(
                TraceLevel::Medium,
                format_args!("Generating sample {}/{}", idx + 1, count),
            );
            outputs.push(self.generate_from_entry(&resolved_entry)?);
        }
        self.trace(
            TraceLevel::Low,
            format_args!(
                "Completed batch generation: produced {} sample(s) for entry='{}'",
                outputs.len(),
                resolved_entry
            ),
        );
        Ok(outputs)
    }

    pub fn generate_from_entry(&mut self, entry_rule: &str) -> Result<String> {
        self.trace(
            TraceLevel::High,
            format_args!(
                "➡️ Enter generate_from_entry(entry='{}', recovery_mode={:?})",
                entry_rule, self.config.recovery_mode
            ),
        );
        self.activate_deterministic_partition_for_entry(entry_rule);
        let mut call_stack = Vec::new();
        let result = match self.config.recovery_mode {
            RecoveryStimuliMode::Baseline => self.generate_rule(entry_rule, 0, &mut call_stack),
            RecoveryStimuliMode::RecoveryBiased => {
                self.generate_recovery_biased_entry(entry_rule, &mut call_stack)
            }
            RecoveryStimuliMode::NearSyncNegative => {
                self.generate_near_sync_negative_entry(entry_rule, &mut call_stack)
            }
        }
        .map(|sample| self.apply_negative_case_policy(entry_rule, sample));
        self.coverage.record_sample_attempt(result.is_ok());
        match &result {
            Ok(sample) => self.trace(
                TraceLevel::High,
                format_args!(
                    "✅ Exit generate_from_entry(entry='{}'): len={} preview='{}'",
                    entry_rule,
                    sample.len(),
                    sample.chars().take(64).collect::<String>()
                ),
            ),
            Err(err) => self.trace(
                TraceLevel::Low,
                format_args!(
                    "❌ Exit generate_from_entry(entry='{}') with error: {}",
                    entry_rule, err
                ),
            ),
        }
        result
    }

    fn generate_recovery_biased_entry(
        &mut self,
        entry_rule: &str,
        call_stack: &mut Vec<String>,
    ) -> Result<String> {
        let marker = self.recovery_stimulus_fallback(entry_rule);
        let base_result = self.generate_rule(entry_rule, 0, call_stack);
        match base_result {
            Ok(base_sample) => {
                let Some(marker) = marker else {
                    return Ok(base_sample);
                };
                let wrapped = match self.rng.gen_range(0..3) {
                    0 => format!("{}{}", base_sample, marker),
                    1 => format!("{}{}", marker, base_sample),
                    _ => format!("{}{}{}", marker, base_sample, marker),
                };
                Ok(wrapped)
            }
            Err(base_error) => {
                if let Some(marker) = marker {
                    Ok(marker)
                } else {
                    Err(base_error)
                }
            }
        }
    }

    fn generate_near_sync_negative_entry(
        &mut self,
        entry_rule: &str,
        call_stack: &mut Vec<String>,
    ) -> Result<String> {
        let Some(marker) = self.recovery_stimulus_fallback(entry_rule) else {
            return self.generate_rule(entry_rule, 0, call_stack);
        };

        let noise = Self::near_sync_negative_prefix(entry_rule);
        let base_sample = self
            .generate_rule(entry_rule, 0, call_stack)
            .unwrap_or_default();
        if base_sample.is_empty() {
            Ok(format!("{}{}", noise, marker))
        } else {
            Ok(format!("{}{}{}", base_sample, noise, marker))
        }
    }

    fn activate_deterministic_partition_for_entry(&mut self, entry_rule: &str) {
        let Some(base_seed) = self.config.seed else {
            return;
        };

        let policy = self.rule_determinism_partition_policy(entry_rule);
        if !policy.enabled {
            return;
        }

        let group_key = policy
            .group_label
            .unwrap_or_else(|| format!("rule.{}", entry_rule));
        let ordinal = {
            let counter = self
                .deterministic_partition_counters
                .entry(group_key.clone())
                .or_insert(0);
            let current = *counter;
            *counter = counter.saturating_add(1);
            current
        };
        let partition_seed = Self::deterministic_partition_seed(base_seed, &group_key, ordinal);
        self.rng = StdRng::seed_from_u64(partition_seed);
    }

    fn resolve_entry_rule(&self, entry_rule: Option<&str>) -> Result<String> {
        if let Some(rule) = entry_rule {
            if self.grammar_tree.contains_key(rule) {
                return Ok(rule.to_string());
            }
            return Err(anyhow!(
                "Entry rule '{}' not found in grammar '{}'",
                rule,
                self.grammar_name
            ));
        }

        self.rule_order.first().cloned().ok_or_else(|| {
            anyhow!(
                "No entry rule available for grammar '{}'",
                self.grammar_name
            )
        })
    }

    fn generate_rule(
        &mut self,
        rule_name: &str,
        depth: usize,
        call_stack: &mut Vec<String>,
    ) -> Result<String> {
        self.trace(
            TraceLevel::Debug,
            format_args!(
                "↳ enter generate_rule(rule='{}', depth={}, active_stack={})",
                rule_name,
                depth,
                call_stack.join(" > ")
            ),
        );
        if depth > self.config.max_depth {
            return Err(anyhow!(
                "Stimuli generation depth exceeded max_depth={} while expanding rule '{}'",
                self.config.max_depth,
                rule_name
            ));
        }

        let active_rule_visits = call_stack
            .iter()
            .filter(|r| r.as_str() == rule_name)
            .count();
        if active_rule_visits >= self.config.max_rule_visits {
            return Err(anyhow!(
                "Stimuli generation exceeded max_rule_visits={} for rule '{}'",
                self.config.max_rule_visits,
                rule_name
            ));
        }

        let rule_node = self.grammar_tree.get(rule_name).with_context(|| {
            format!(
                "Missing rule '{}' in grammar '{}'",
                rule_name, self.grammar_name
            )
        })?;

        call_stack.push(rule_name.to_string());
        let result = self.generate_node(rule_node, rule_name, depth + 1, call_stack, "root");
        call_stack.pop();
        if result.is_ok() {
            self.coverage.record_rule_success(rule_name);
        }
        match &result {
            Ok(sample) => self.trace(
                TraceLevel::Debug,
                format_args!(
                    "↰ exit generate_rule(rule='{}'): success len={}",
                    rule_name,
                    sample.len()
                ),
            ),
            Err(err) => self.trace(
                TraceLevel::High,
                format_args!("↰ exit generate_rule(rule='{}'): error={}", rule_name, err),
            ),
        }
        result
    }

    fn generate_node(
        &mut self,
        node: &ASTNode,
        current_rule: &str,
        depth: usize,
        call_stack: &mut Vec<String>,
        node_path: &str,
    ) -> Result<String> {
        match node {
            ASTNode::Or { alternatives } => {
                self.generate_or(alternatives, current_rule, depth, call_stack, node_path)
            }
            ASTNode::Sequence { elements } => {
                self.generate_sequence(elements, current_rule, depth, call_stack, node_path)
            }
            ASTNode::Atom { value } => {
                self.generate_atom(value, current_rule, depth, call_stack, node_path)
            }
            ASTNode::Quantified {
                element,
                quantifier,
            } => self.generate_quantified(
                element,
                quantifier,
                current_rule,
                depth,
                call_stack,
                node_path,
            ),
        }
    }

    fn generate_or(
        &mut self,
        alternatives: &[ASTNode],
        current_rule: &str,
        depth: usize,
        call_stack: &mut Vec<String>,
        node_path: &str,
    ) -> Result<String> {
        self.trace(
            TraceLevel::High,
            format_args!(
                "OR decision point: rule='{}' path='{}' depth={} branches={}",
                current_rule,
                node_path,
                depth,
                alternatives.len()
            ),
        );
        if alternatives.is_empty() {
            return Ok(String::new());
        }

        let prepared: Vec<(Option<u32>, ASTNode)> = alternatives
            .iter()
            .map(|node| self.strip_probability_prefix(node))
            .collect();

        let mut candidate_indices: Vec<usize> = (0..prepared.len()).collect();

        if depth >= self.config.max_depth.saturating_sub(1) {
            let min_ref_count = candidate_indices
                .iter()
                .map(|idx| self.count_rule_references(&prepared[*idx].1, current_rule))
                .min()
                .unwrap_or(0);

            candidate_indices.retain(|idx| {
                self.count_rule_references(&prepared[*idx].1, current_rule) == min_ref_count
            });
        }

        if candidate_indices.is_empty() {
            return Err(anyhow!(
                "No candidate branches available for rule '{}' during stimuli generation",
                current_rule
            ));
        }

        let (branch_policy, associativity, branch_priorities) =
            self.rule_branch_controls(current_rule, prepared.len());
        let attempt_order: Vec<usize> = match branch_policy {
            SemanticBranchPolicy::Ordered => (0..candidate_indices.len()).collect(),
            SemanticBranchPolicy::PriorityFirst => {
                let mut ordered: Vec<usize> = (0..candidate_indices.len()).collect();
                ordered.sort_by(|left, right| {
                    let left_global = candidate_indices[*left];
                    let right_global = candidate_indices[*right];
                    let left_priority = branch_priorities.get(left_global).copied().unwrap_or(0);
                    let right_priority = branch_priorities.get(right_global).copied().unwrap_or(0);
                    right_priority
                        .cmp(&left_priority)
                        .then_with(|| match associativity {
                            SemanticAssociativity::Right => right_global.cmp(&left_global),
                            _ => left_global.cmp(&right_global),
                        })
                });
                ordered
            }
            SemanticBranchPolicy::LongestMatch => {
                let probabilities: Vec<Option<u32>> = candidate_indices
                    .iter()
                    .map(|idx| prepared[*idx].0)
                    .collect();
                let base_weights = self.build_weights(&probabilities)?;
                let guided_weights: Vec<u64> = candidate_indices
                    .iter()
                    .enumerate()
                    .map(|(local_idx, global_idx)| {
                        let multiplier = self.coverage_guidance_multiplier(
                            current_rule,
                            node_path,
                            *global_idx,
                            &prepared[*global_idx].1,
                        );
                        let recursion_penalty = self.recursion_pressure_penalty(
                            &prepared[*global_idx].1,
                            call_stack,
                            depth,
                        );
                        let adjusted_multiplier = (multiplier / recursion_penalty).max(1);
                        let semantic_multiplier = self.semantic_branch_multiplier(
                            associativity,
                            &branch_priorities,
                            *global_idx,
                            prepared.len(),
                        );
                        u64::from(base_weights[local_idx])
                            .saturating_mul(adjusted_multiplier)
                            .saturating_mul(semantic_multiplier)
                    })
                    .collect();

                let dist = WeightedIndex::new(&guided_weights).with_context(|| {
                    format!(
                        "Invalid branch weights for rule '{}': {:?}",
                        current_rule, guided_weights
                    )
                })?;
                let selected_local = dist.sample(&mut self.rng);
                let mut ordered: Vec<usize> = (0..candidate_indices.len()).collect();
                ordered.swap(0, selected_local);
                if ordered.len() > 2 {
                    ordered[1..].shuffle(&mut self.rng);
                }
                ordered
            }
        };
        self.trace(
            TraceLevel::Debug,
            format_args!(
                "OR policy for rule='{}': policy={:?} associativity={:?} candidate_indices={:?} attempt_order(local)={:?}",
                current_rule,
                branch_policy,
                associativity,
                candidate_indices,
                attempt_order
            ),
        );

        let mut last_error: Option<anyhow::Error> = None;
        for local_idx in attempt_order {
            let selected_global = candidate_indices[local_idx];
            let selected_node = prepared[selected_global].1.clone();
            let group_key = format!("{}::{}", current_rule, node_path);
            self.coverage.record_branch_selected(
                &group_key,
                current_rule,
                node_path,
                alternatives.len(),
                selected_global,
            );
            self.trace(
                TraceLevel::High,
                format_args!(
                    "Trying OR branch: rule='{}' path='{}' local_branch={} global_branch={}",
                    current_rule, node_path, local_idx, selected_global
                ),
            );
            let alt_path = format!("{}/o{}", node_path, selected_global);
            match self.generate_node(&selected_node, current_rule, depth, call_stack, &alt_path) {
                Ok(output) => {
                    self.coverage.record_branch_success(
                        &group_key,
                        current_rule,
                        node_path,
                        alternatives.len(),
                        selected_global,
                    );
                    self.trace(
                        TraceLevel::High,
                        format_args!(
                            "Selected OR branch: rule='{}' path='{}' branch={} output_len={}",
                            current_rule,
                            node_path,
                            selected_global,
                            output.len()
                        ),
                    );
                    return Ok(output);
                }
                Err(err) => {
                    self.trace(
                        TraceLevel::Debug,
                        format_args!(
                            "OR branch failed: rule='{}' path='{}' branch={} reason={}",
                            current_rule, node_path, selected_global, err
                        ),
                    );
                    last_error = Some(err);
                }
            }
        }

        if let Some(recovery_sample) = self.recovery_stimulus_fallback(current_rule) {
            self.trace(
                TraceLevel::Low,
                format_args!(
                    "OR fallback recovery used: rule='{}' path='{}' fallback_len={}",
                    current_rule,
                    node_path,
                    recovery_sample.len()
                ),
            );
            return Ok(recovery_sample);
        }

        Err(last_error.unwrap_or_else(|| {
            anyhow!(
                "Failed to generate any OR alternative for rule '{}'",
                current_rule
            )
        }))
    }

    fn generate_sequence(
        &mut self,
        elements: &[ASTNode],
        current_rule: &str,
        depth: usize,
        call_stack: &mut Vec<String>,
        node_path: &str,
    ) -> Result<String> {
        let relational_policy = if node_path == "root" {
            self.rule_relational_constraints(current_rule)
        } else {
            StimuliRelationalConstraintPolicy::default()
        };

        if relational_policy.constraint_expression.is_none() {
            let mut output = String::new();
            for (idx, element) in elements.iter().enumerate() {
                let element_path = format!("{}/s{}", node_path, idx);
                let generated =
                    self.generate_node(element, current_rule, depth, call_stack, &element_path)?;
                self.append_generated_segment(&mut output, &generated);
            }
            return Ok(output);
        }

        let attempt_budget = self.relational_attempt_budget();
        let mut last_error: Option<anyhow::Error> = None;
        let mut last_violation: Option<String> = None;
        let mut violation_counts: HashMap<String, usize> = HashMap::new();
        let mut relational_failures = 0usize;
        let mut generation_failures = 0usize;

        for _ in 0..attempt_budget {
            let mut output = String::new();
            let mut captures = Vec::with_capacity(elements.len());
            let mut named_captures = HashMap::new();

            let mut generation_failed = false;
            for (idx, element) in elements.iter().enumerate() {
                let element_path = format!("{}/s{}", node_path, idx);
                let capture_name = Self::sequence_element_capture_name(element);
                match self.generate_node(element, current_rule, depth, call_stack, &element_path) {
                    Ok(generated) => {
                        if let Some(name) = capture_name {
                            named_captures.insert(name, generated.clone());
                        }
                        self.append_generated_segment(&mut output, &generated);
                        captures.push(generated);
                    }
                    Err(err) => {
                        generation_failed = true;
                        generation_failures = generation_failures.saturating_add(1);
                        last_error = Some(err);
                        break;
                    }
                }
            }

            if generation_failed {
                continue;
            }

            match self.validate_relational_sample(
                current_rule,
                &relational_policy,
                &captures,
                &named_captures,
            ) {
                Ok(()) => return Ok(output),
                Err(err) => {
                    relational_failures = relational_failures.saturating_add(1);
                    let reason = err.to_string();
                    *violation_counts.entry(reason.clone()).or_insert(0) += 1;
                    last_violation = Some(reason);
                }
            }
        }

        if !violation_counts.is_empty() {
            let mut ranked_violations: Vec<(String, usize)> =
                violation_counts.into_iter().collect();
            ranked_violations
                .sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));

            let top_violations = ranked_violations
                .iter()
                .take(3)
                .map(|(reason, count)| format!("{}x {}", count, reason))
                .collect::<Vec<String>>()
                .join(" | ");
            let likely_unsatisfiable = ranked_violations
                .first()
                .map(|(_, count)| *count == relational_failures)
                .unwrap_or(false);

            return Err(anyhow!(
                "Failed to generate relationally valid sequence for rule '{}' within {} attempt(s): relational_failures={} generation_failures={} top_violations=[{}] likely_unsatisfiable={}",
                current_rule,
                attempt_budget,
                relational_failures,
                generation_failures,
                top_violations,
                likely_unsatisfiable
            ));
        }

        if let Some(err) = last_error {
            return Err(err);
        }

        Err(anyhow!(
            "Failed to generate relationally valid sequence for rule '{}' within {} attempt(s): {}",
            current_rule,
            attempt_budget,
            last_violation.unwrap_or_else(|| "unknown relational contract violation".to_string())
        ))
    }

    fn generate_atom(
        &mut self,
        value: &ASTValue,
        current_rule: &str,
        depth: usize,
        call_stack: &mut Vec<String>,
        node_path: &str,
    ) -> Result<String> {
        match value {
            ASTValue::Node(node) => {
                let atom_path = format!("{}/a", node_path);
                self.generate_node(node, current_rule, depth, call_stack, &atom_path)
            }
            ASTValue::Token(parts) => {
                let Some((token_type, token_value)) = Self::extract_token_pair(parts) else {
                    return Ok(String::new());
                };
                self.trace(
                    TraceLevel::Debug,
                    format_args!(
                        "Atom dispatch: rule='{}' depth={} path='{}' token_type='{}' token='{}'",
                        current_rule, depth, node_path, token_type, token_value
                    ),
                );

                match token_type {
                    "quoted_string" => Ok(token_value.to_string()),
                    "rule_reference" => self.generate_rule(token_value, depth + 1, call_stack),
                    "regex" => {
                        let effective_pattern =
                            self.effective_regex_pattern(current_rule, token_value);
                        Ok(self.generate_regex_sample(&effective_pattern, current_rule))
                    }
                    "probability" => Ok(String::new()),
                    "number" | "include_dir" | "include_file" | "rule" => {
                        Ok(token_value.to_string())
                    }
                    _ => Ok(token_value.to_string()),
                }
            }
        }
    }

    fn generate_quantified(
        &mut self,
        element: &ASTNode,
        quantifier: &str,
        current_rule: &str,
        depth: usize,
        call_stack: &mut Vec<String>,
        node_path: &str,
    ) -> Result<String> {
        let (min_repeat, max_repeat) = self.parse_quantifier_bounds(quantifier)?;
        let bounded_max = max_repeat.min(self.config.max_repeat.max(min_repeat));
        let repeat_candidates: Vec<usize> = if min_repeat == bounded_max {
            vec![min_repeat]
        } else if depth >= self.config.max_depth.saturating_sub(1) {
            vec![min_repeat]
        } else {
            let preferred = self.rng.gen_range(min_repeat..=bounded_max);
            let mut candidates = Vec::with_capacity(bounded_max.saturating_sub(min_repeat) + 1);
            candidates.push(preferred);
            for repeat in min_repeat..=bounded_max {
                if repeat != preferred {
                    candidates.push(repeat);
                }
            }
            candidates
        };
        self.trace(
            TraceLevel::High,
            format_args!(
                "Quantifier decision: rule='{}' path='{}' quantifier='{}' min={} max={} candidates={:?}",
                current_rule,
                node_path,
                quantifier,
                min_repeat,
                bounded_max,
                repeat_candidates
            ),
        );

        let quantified_path = format!("{}/q", node_path);
        let mut last_error: Option<anyhow::Error> = None;

        for repeats in repeat_candidates {
            let mut output = String::new();
            let mut failed = false;
            for _ in 0..repeats {
                match self.generate_node(
                    element,
                    current_rule,
                    depth + 1,
                    call_stack,
                    &quantified_path,
                ) {
                    Ok(generated) => self.append_generated_segment(&mut output, &generated),
                    Err(err) => {
                        failed = true;
                        last_error = Some(err);
                        break;
                    }
                }
            }
            if !failed {
                self.trace(
                    TraceLevel::Debug,
                    format_args!(
                        "Quantifier success: rule='{}' path='{}' repeats={} output_len={}",
                        current_rule,
                        node_path,
                        repeats,
                        output.len()
                    ),
                );
                return Ok(output);
            }
        }

        Err(last_error.unwrap_or_else(|| {
            anyhow!(
                "Failed to generate quantified element for rule '{}' with quantifier '{}'",
                current_rule,
                quantifier
            )
        }))
    }

    fn parse_quantifier_bounds(&self, quantifier: &str) -> Result<(usize, usize)> {
        match quantifier.trim() {
            "?" => Ok((0, 1)),
            "*" => Ok((0, self.config.max_repeat)),
            "+" => Ok((1, self.config.max_repeat.max(1))),
            other => {
                if let Ok(exact) = other.parse::<usize>() {
                    return Ok((exact, exact));
                }

                if other.contains(',') {
                    let parts: Vec<&str> = other.split(',').collect();
                    if parts.len() != 2 {
                        return Err(anyhow!("Unsupported quantifier format '{}'", other));
                    }

                    let min = if parts[0].trim().is_empty() {
                        0
                    } else {
                        parts[0].trim().parse::<usize>().with_context(|| {
                            format!("Invalid quantifier lower bound '{}'", parts[0])
                        })?
                    };
                    let max = if parts[1].trim().is_empty() {
                        self.config.max_repeat.max(min)
                    } else {
                        parts[1].trim().parse::<usize>().with_context(|| {
                            format!("Invalid quantifier upper bound '{}'", parts[1])
                        })?
                    };

                    if min > max {
                        return Err(anyhow!(
                            "Invalid quantifier bounds '{}': min {} > max {}",
                            other,
                            min,
                            max
                        ));
                    }
                    return Ok((min, max));
                }

                Err(anyhow!("Unknown quantifier '{}'", other))
            }
        }
    }

    fn strip_probability_prefix(&self, node: &ASTNode) -> (Option<u32>, ASTNode) {
        match node {
            ASTNode::Sequence { elements } => {
                let mut index = 0usize;
                let mut probability = None;

                while index < elements.len() {
                    if let Some(weight) = self.extract_probability_from_node(&elements[index]) {
                        probability = Some(weight);
                        index += 1;
                    } else {
                        break;
                    }
                }

                if index == 0 {
                    return (None, node.clone());
                }

                let remainder = elements[index..].to_vec();
                let stripped = match remainder.len() {
                    0 => ASTNode::Sequence { elements: vec![] },
                    1 => remainder[0].clone(),
                    _ => ASTNode::Sequence {
                        elements: remainder,
                    },
                };
                (probability, stripped)
            }
            _ => {
                if let Some(weight) = self.extract_probability_from_node(node) {
                    (Some(weight), ASTNode::Sequence { elements: vec![] })
                } else {
                    (None, node.clone())
                }
            }
        }
    }

    fn extract_probability_from_node(&self, node: &ASTNode) -> Option<u32> {
        let ASTNode::Atom { value } = node else {
            return None;
        };

        let ASTValue::Token(parts) = value else {
            return None;
        };

        let (token_type, token_value) = Self::extract_token_pair(parts)?;
        if token_type != "probability" {
            return None;
        }

        token_value.parse::<u32>().ok()
    }

    fn build_weights(&self, probabilities: &[Option<u32>]) -> Result<Vec<u32>> {
        if probabilities.is_empty() {
            return Ok(Vec::new());
        }

        let explicit_count = probabilities.iter().filter(|p| p.is_some()).count();
        if explicit_count == 0 {
            return Ok(vec![1; probabilities.len()]);
        }

        let explicit_sum: u32 = probabilities.iter().flatten().copied().sum();

        if explicit_count == probabilities.len() {
            if explicit_sum != 100 {
                return Err(anyhow!(
                    "Explicit branch probabilities must sum to 100, found {}",
                    explicit_sum
                ));
            }
            let weights: Vec<u32> = probabilities.iter().map(|p| p.unwrap_or(0)).collect();
            if weights.iter().all(|w| *w == 0) {
                return Err(anyhow!("All explicit branch probabilities are zero"));
            }
            return Ok(weights);
        }

        if explicit_sum >= 100 {
            return Err(anyhow!(
                "Explicit probabilities consume {}%, leaving no weight for unannotated branches",
                explicit_sum
            ));
        }

        let missing = probabilities.len() - explicit_count;
        let remaining = 100 - explicit_sum;
        let base = remaining / missing as u32;
        let remainder = remaining % missing as u32;

        let mut missing_seen = 0usize;
        let mut weights = Vec::with_capacity(probabilities.len());
        for probability in probabilities {
            if let Some(value) = probability {
                weights.push(*value);
            } else {
                let extra = if missing_seen < remainder as usize {
                    1
                } else {
                    0
                };
                weights.push(base + extra);
                missing_seen += 1;
            }
        }

        if weights.iter().all(|w| *w == 0) {
            return Err(anyhow!("Computed branch weights are all zero"));
        }

        Ok(weights)
    }

    fn count_rule_references(&self, node: &ASTNode, current_rule: &str) -> usize {
        match node {
            ASTNode::Or { alternatives } => alternatives
                .iter()
                .map(|alt| self.count_rule_references(alt, current_rule))
                .min()
                .unwrap_or(0),
            ASTNode::Sequence { elements } => elements
                .iter()
                .map(|el| self.count_rule_references(el, current_rule))
                .sum(),
            ASTNode::Atom { value } => {
                let ASTValue::Token(parts) = value else {
                    return 0;
                };
                let Some((token_type, token_value)) = Self::extract_token_pair(parts) else {
                    return 0;
                };
                if token_type == "rule_reference" {
                    if token_value == current_rule { 2 } else { 1 }
                } else {
                    0
                }
            }
            ASTNode::Quantified { element, .. } => {
                self.count_rule_references(element, current_rule)
            }
        }
    }

    fn recursion_pressure_penalty(
        &self,
        branch_node: &ASTNode,
        call_stack: &[String],
        depth: usize,
    ) -> u64 {
        let mut refs = HashSet::new();
        self.collect_rule_references(branch_node, &mut refs);
        if refs.is_empty() {
            return 1;
        }

        let mut max_active = 0usize;
        let mut total_active = 0usize;
        for rule_name in refs {
            let active = call_stack
                .iter()
                .filter(|active_rule| active_rule.as_str() == rule_name.as_str())
                .count();
            max_active = max_active.max(active);
            total_active = total_active.saturating_add(active);
        }

        if max_active == 0 {
            return 1;
        }

        let mut penalty = 1u64
            .saturating_add(u64::try_from(max_active.min(8)).unwrap_or(1))
            .saturating_add(u64::try_from(total_active.min(8)).unwrap_or(1));

        let remaining_depth = self.config.max_depth.saturating_sub(depth);
        if remaining_depth <= 8 {
            penalty = penalty.saturating_mul(4);
        }
        if remaining_depth <= 4 {
            penalty = penalty.saturating_mul(6);
        }
        if remaining_depth <= 2 {
            penalty = penalty.saturating_mul(8);
        }

        penalty.max(1)
    }

    fn coverage_guidance_multiplier(
        &self,
        current_rule: &str,
        node_path: &str,
        branch_idx: usize,
        branch_node: &ASTNode,
    ) -> u64 {
        let group_key = format!("{}::{}", current_rule, node_path);
        let (success_hits, selected_hits) =
            if let Some(group) = self.coverage.branch_groups.get(&group_key) {
                (
                    group.success_counts.get(branch_idx).copied().unwrap_or(0),
                    group.selected_counts.get(branch_idx).copied().unwrap_or(0),
                )
            } else {
                (0, 0)
            };
        let branch_target_deficit = self.branch_target_deficit(&group_key, branch_idx);

        let mut multiplier = 1u64;
        if success_hits == 0 {
            multiplier = multiplier.saturating_mul(24);
        } else if success_hits <= 2 {
            multiplier = multiplier.saturating_mul(8);
        } else if success_hits <= 8 {
            multiplier = multiplier.saturating_mul(3);
        }

        if selected_hits == 0 {
            multiplier = multiplier.saturating_mul(2);
        }

        let uncovered_rule_refs = self.count_uncovered_rule_references(branch_node);
        if uncovered_rule_refs > 0 {
            multiplier = multiplier
                .saturating_mul(1 + u64::try_from(uncovered_rule_refs.min(4)).unwrap_or(1));
        }

        multiplier = multiplier
            .saturating_mul(self.semantic_coverage_guidance_multiplier(current_rule, branch_node));

        multiplier = multiplier.saturating_mul(self.target_guidance_multiplier(
            current_rule,
            node_path,
            branch_idx,
            branch_node,
        ));

        // Prevent target-driven mode from over-selecting branches that repeatedly fail parser-backed validation.
        if branch_target_deficit > 0 && selected_hits > 0 {
            let throttle = Self::target_branch_failure_throttle(selected_hits, success_hits);
            multiplier = (multiplier / throttle).max(1);
        }

        multiplier
    }

    fn target_branch_failure_throttle(selected_hits: u64, success_hits: u64) -> u64 {
        if selected_hits < 8 {
            return 1;
        }

        if success_hits.saturating_mul(3) > selected_hits {
            return 1;
        }

        let raw_throttle = if success_hits == 0 {
            selected_hits.saturating_mul(2)
        } else {
            selected_hits
                .saturating_add(success_hits.saturating_sub(1))
                .saturating_div(success_hits)
        };

        raw_throttle.clamp(1, 256)
    }

    fn count_uncovered_rule_references(&self, node: &ASTNode) -> usize {
        let mut names = HashSet::new();
        self.collect_rule_references(node, &mut names);
        names.retain(|rule_name| {
            self.coverage
                .rule_success_hits
                .get(rule_name)
                .copied()
                .unwrap_or(0)
                == 0
        });
        names.len()
    }
    fn target_guidance_multiplier(
        &self,
        current_rule: &str,
        node_path: &str,
        branch_idx: usize,
        branch_node: &ASTNode,
    ) -> u64 {
        let group_key = Self::branch_group_key(current_rule, node_path);
        let branch_deficit = self.branch_target_deficit(&group_key, branch_idx);
        let mut multiplier = 1u64;

        if branch_deficit > 0 {
            multiplier =
                multiplier.saturating_mul(16u64.saturating_mul(branch_deficit.min(8)).max(16));
        }

        let current_rule_deficit = self.rule_target_deficit(current_rule);
        if current_rule_deficit > 0 {
            multiplier =
                multiplier.saturating_mul(3u64.saturating_mul(current_rule_deficit.min(8)).max(3));
        }

        let mut refs = HashSet::new();
        self.collect_rule_references(branch_node, &mut refs);
        let targeted_refs = refs
            .iter()
            .filter(|rule_name| self.rule_target_deficit(rule_name.as_str()) > 0)
            .count();
        if targeted_refs > 0 {
            multiplier = multiplier.saturating_mul(
                1 + u64::try_from(targeted_refs.min(8))
                    .unwrap_or(1)
                    .saturating_mul(4),
            );
        }

        multiplier.max(1)
    }

    fn generate_regex_sample(&mut self, pattern: &str, current_rule: &str) -> String {
        self.trace(
            TraceLevel::High,
            format_args!(
                "Regex sample generation: rule='{}' pattern='{}'",
                current_rule, pattern
            ),
        );
        let trimmed = pattern.trim();
        if trimmed.is_empty() {
            return String::new();
        }
        let constraints = self.rule_value_constraints(current_rule);
        if let Some(semantic_hint) = self.semantic_hint_for_rule(current_rule) {
            if self.value_satisfies_constraints(&semantic_hint, &constraints) {
                self.trace(
                    TraceLevel::Debug,
                    format_args!(
                        "Regex generation chose semantic hint override for rule='{}': '{}'",
                        current_rule, semantic_hint
                    ),
                );
                return self.apply_word_boundary_spacing(trimmed, semantic_hint);
            }
            self.trace(
                TraceLevel::Debug,
                format_args!(
                    "Regex generation rejected semantic hint override for rule='{}' because it violates explicit semantic constraints: '{}'",
                    current_rule, semantic_hint
                ),
            );
        }

        if !constraints.enum_values.is_empty() {
            let valid_enum_values: Vec<&String> = constraints
                .enum_values
                .iter()
                .filter(|value| {
                    Self::regex_matches_entire(trimmed, value)
                        && self.value_satisfies_constraints(value, &constraints)
                })
                .collect();
            if !valid_enum_values.is_empty() {
                let idx = if valid_enum_values.len() == 1 {
                    0
                } else {
                    self.rng.gen_range(0..valid_enum_values.len())
                };
                self.trace(
                    TraceLevel::Debug,
                    format_args!(
                        "Regex generation chose enum candidate for rule='{}': '{}'",
                        current_rule, valid_enum_values[idx]
                    ),
                );
                return self
                    .apply_word_boundary_spacing(trimmed, valid_enum_values[idx].to_string());
            }
        }

        if let Some(candidate) = self.constraint_driven_candidate(trimmed, &constraints) {
            self.trace(
                TraceLevel::Debug,
                format_args!(
                    "Regex generation chose constraint-driven candidate for rule='{}': '{}'",
                    current_rule, candidate
                ),
            );
            return self.apply_word_boundary_spacing(trimmed, candidate);
        }

        let parsed_hir = match regex_syntax::parse(trimmed) {
            Ok(hir) => Some(hir),
            Err(err) => {
                self.trace(
                    TraceLevel::Low,
                    format_args!(
                        "Regex generation could not parse pattern for rule='{}': '{}' ({})",
                        current_rule, trimmed, err
                    ),
                );
                None
            }
        };

        for _ in 0..64 {
            let Some(hir) = parsed_hir.as_ref() else {
                break;
            };
            let candidate = self.generate_from_regex_hir(hir);
            if self.regex_candidate_satisfies_contract(trimmed, &candidate, &constraints) {
                self.trace(
                    TraceLevel::Debug,
                    format_args!(
                        "Regex generation accepted contract-satisfying candidate for rule='{}': '{}'",
                        current_rule, candidate
                    ),
                );
                return self.apply_word_boundary_spacing(trimmed, candidate);
            }
        }

        if let Some(fallback) = constraints.enum_values.first() {
            if self.regex_candidate_satisfies_contract(trimmed, fallback, &constraints) {
                self.trace(
                    TraceLevel::Low,
                    format_args!(
                        "Regex generation using enum fallback for rule='{}': '{}'",
                        current_rule, fallback
                    ),
                );
                return self.apply_word_boundary_spacing(trimmed, fallback.clone());
            }
        }

        self.trace(
            TraceLevel::Low,
            format_args!(
                "Regex generation failed to synthesize contract-satisfying sample for rule='{}' pattern='{}'",
                current_rule, trimmed
            ),
        );
        String::new()
    }

    fn regex_matches_entire(pattern: &str, candidate: &str) -> bool {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(matched) = re.find(candidate) {
                return matched.start() == 0 && matched.end() == candidate.len();
            }
        }
        false
    }

    fn regex_candidate_satisfies_contract(
        &self,
        pattern: &str,
        candidate: &str,
        constraints: &SemanticValueConstraints,
    ) -> bool {
        Self::regex_matches_entire(pattern, candidate)
            && self.value_satisfies_constraints(candidate, constraints)
    }

    fn apply_word_boundary_spacing(&self, pattern: &str, candidate: String) -> String {
        if !self.config.enforce_word_boundary_spacing {
            return candidate;
        }
        if !Self::pattern_has_terminal_word_boundary(pattern) {
            return candidate;
        }
        let Some(last_char) = candidate.chars().last() else {
            return candidate;
        };
        if !Self::is_word_char(last_char) {
            return candidate;
        }
        let mut spaced = candidate;
        spaced.push(' ');
        spaced
    }

    fn pattern_has_terminal_word_boundary(pattern: &str) -> bool {
        let mut trimmed = pattern.trim_end();
        while let Some(stripped) = trimmed.strip_suffix('$') {
            trimmed = stripped.trim_end();
        }
        trimmed.ends_with("\\b")
    }

    fn is_word_char(ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ch == '_'
    }

    fn is_lexical_word_char(ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'
    }

    fn starts_with_lexical_word_char(value: &str) -> bool {
        value
            .chars()
            .next()
            .map(Self::is_lexical_word_char)
            .unwrap_or(false)
    }

    fn ends_with_lexical_word_char(value: &str) -> bool {
        value
            .chars()
            .next_back()
            .map(Self::is_lexical_word_char)
            .unwrap_or(false)
    }

    fn append_generated_segment(&self, output: &mut String, segment: &str) {
        if self.config.enforce_word_boundary_spacing
            && !output.is_empty()
            && !segment.is_empty()
            && Self::ends_with_lexical_word_char(output.as_str())
            && Self::starts_with_lexical_word_char(segment)
        {
            output.push(' ');
        }
        output.push_str(segment);
    }

    fn constraint_driven_candidate(
        &mut self,
        pattern: &str,
        constraints: &SemanticValueConstraints,
    ) -> Option<String> {
        if let (Some(min), Some(max)) = (constraints.min_numeric, constraints.max_numeric) {
            let lower = min.ceil().max(i64::MIN as f64);
            let upper = max.floor().min(i64::MAX as f64);
            if lower <= upper {
                let start = lower as i64;
                let end = upper as i64;
                let sampled = if start == end {
                    start
                } else {
                    self.rng.gen_range(start..=end)
                };
                let candidate = sampled.to_string();
                if Self::regex_matches_entire(pattern, &candidate)
                    && self.value_satisfies_constraints(&candidate, constraints)
                {
                    return Some(candidate);
                }
            }
        }

        if let (Some(min_len), Some(max_len)) = (constraints.min_len, constraints.max_len) {
            let upper = max_len.min(min_len.saturating_add(32));
            let len = if min_len == upper {
                min_len
            } else {
                self.rng.gen_range(min_len..=upper)
            };
            let candidate = "a".repeat(len);
            if Self::regex_matches_entire(pattern, &candidate)
                && self.value_satisfies_constraints(&candidate, constraints)
            {
                return Some(candidate);
            }
        }

        None
    }

    fn value_satisfies_constraints(
        &self,
        value: &str,
        constraints: &SemanticValueConstraints,
    ) -> bool {
        if constraints.is_empty() {
            return true;
        }

        if !constraints.enum_values.is_empty()
            && !constraints
                .enum_values
                .iter()
                .any(|allowed| allowed == value)
        {
            return false;
        }

        let value_len = value.chars().count();
        if let Some(min_len) = constraints.min_len {
            if value_len < min_len {
                return false;
            }
        }
        if let Some(max_len) = constraints.max_len {
            if value_len > max_len {
                return false;
            }
        }

        if constraints.min_numeric.is_some() || constraints.max_numeric.is_some() {
            let Ok(numeric_value) = value.parse::<f64>() else {
                return false;
            };
            if let Some(min_numeric) = constraints.min_numeric {
                if numeric_value < min_numeric {
                    return false;
                }
            }
            if let Some(max_numeric) = constraints.max_numeric {
                if numeric_value > max_numeric {
                    return false;
                }
            }
        }

        if let Some(pattern) = &constraints.regex_pattern {
            if !Self::regex_matches_entire(pattern, value) {
                return false;
            }
        }

        true
    }

    fn generate_from_regex_hir(&mut self, hir: &Hir) -> String {
        match hir.kind() {
            HirKind::Empty => String::new(),
            HirKind::Literal(Literal(bytes)) => String::from_utf8_lossy(bytes).into_owned(),
            HirKind::Class(class) => self.generate_from_regex_class(class),
            HirKind::Look(_) => String::new(),
            HirKind::Repetition(rep) => self.generate_from_regex_repetition(rep),
            HirKind::Capture(capture) => self.generate_from_regex_hir(&capture.sub),
            HirKind::Concat(parts) => {
                let mut out = String::new();
                for part in parts {
                    out.push_str(&self.generate_from_regex_hir(part));
                }
                out
            }
            HirKind::Alternation(parts) => {
                if parts.is_empty() {
                    return String::new();
                }
                let idx = if parts.len() == 1 {
                    0
                } else {
                    self.rng.gen_range(0..parts.len())
                };
                self.generate_from_regex_hir(&parts[idx])
            }
        }
    }

    fn generate_from_regex_repetition(&mut self, rep: &Repetition) -> String {
        let min = usize::try_from(rep.min).unwrap_or(0);
        let max = rep
            .max
            .and_then(|m| usize::try_from(m).ok())
            .unwrap_or(self.config.max_repeat.max(min));
        let bounded_max = max.min(self.config.max_repeat.max(min));

        let count = if min == bounded_max {
            min
        } else {
            self.rng.gen_range(min..=bounded_max)
        };

        let mut out = String::new();
        for _ in 0..count {
            let unit = self.generate_from_regex_hir(&rep.sub);
            out.push_str(&unit);
        }
        out
    }

    fn generate_from_regex_class(&mut self, class: &Class) -> String {
        match class {
            Class::Unicode(unicode_class) => {
                let mut printable = Vec::new();
                for range in unicode_class.ranges() {
                    let start = (range.start() as u32).max(0x20);
                    let end = (range.end() as u32).min(0x7e);
                    for codepoint in start..=end {
                        if let Some(ch) = char::from_u32(codepoint) {
                            printable.push(ch);
                        }
                    }
                }
                if !printable.is_empty() {
                    let idx = self.rng.gen_range(0..printable.len());
                    return printable[idx].to_string();
                }

                if let Some(first_range) = unicode_class.ranges().first() {
                    let start = first_range.start() as u32;
                    let end = first_range.end() as u32;
                    let sampled = if start <= end {
                        self.rng.gen_range(start..=end)
                    } else {
                        start
                    };
                    if let Some(ch) = char::from_u32(sampled) {
                        return ch.to_string();
                    }
                }

                "a".to_string()
            }
            Class::Bytes(bytes_class) => {
                let mut printable = Vec::new();
                for range in bytes_class.ranges() {
                    let start = range.start().max(0x20);
                    let end = range.end().min(0x7e);
                    if start <= end {
                        for b in start..=end {
                            printable.push(b);
                        }
                    }
                }
                if !printable.is_empty() {
                    let idx = self.rng.gen_range(0..printable.len());
                    return char::from(printable[idx]).to_string();
                }

                if let Some(first_range) = bytes_class.ranges().first() {
                    let start = first_range.start();
                    let end = first_range.end();
                    let sampled = if start <= end {
                        self.rng.gen_range(start..=end)
                    } else {
                        start
                    };
                    return char::from(sampled).to_string();
                }

                "a".to_string()
            }
        }
    }

    fn semantic_hint_for_rule(&self, rule_name: &str) -> Option<String> {
        let annotations = self.annotations?;
        let semantic_annotations = annotations.semantic_annotations.get(rule_name)?;

        for semantic_annotation in semantic_annotations {
            let directive_name = self.semantic_directive_name(semantic_annotation);
            match semantic_annotation.ast() {
                UnifiedSemanticAST::TransformExpr { expression } => {
                    if directive_name.as_deref() != Some("transform") {
                        continue;
                    }
                    if let Some(transform) = parse_canonical_transform_expression(expression) {
                        if let Some(hint) = stimuli_hint_for_target_type(&transform.target_type) {
                            return Some(hint.to_string());
                        }
                    }
                }
                UnifiedSemanticAST::Raw { content } => {
                    if matches!(
                        directive_name.as_deref(),
                        Some(name)
                            if !matches!(name, "sample" | "literal" | "example" | "stimulus")
                    ) {
                        continue;
                    }

                    let trimmed = content.trim();
                    if trimmed.len() >= 2
                        && ((trimmed.starts_with('"') && trimmed.ends_with('"'))
                            || (trimmed.starts_with('\'') && trimmed.ends_with('\'')))
                    {
                        return Some(trimmed[1..trimmed.len() - 1].to_string());
                    }
                }
            }
        }

        None
    }

    fn rule_branch_controls(
        &self,
        rule_name: &str,
        branch_count: usize,
    ) -> (SemanticBranchPolicy, SemanticAssociativity, Vec<i64>) {
        let mut branch_policy = SemanticBranchPolicy::LongestMatch;
        let mut associativity = SemanticAssociativity::Left;
        let default_priorities = vec![0i64; branch_count];
        let Some(annotations) = self.annotations else {
            return (branch_policy, associativity, default_priorities);
        };
        let Some(entries) = annotations.semantic_annotations.get(rule_name) else {
            return (branch_policy, associativity, default_priorities);
        };

        let mut precedence_priorities: Option<Vec<i64>> = None;
        let mut explicit_priorities: Option<Vec<i64>> = None;

        for annotation in entries {
            let Some((name, payload)) = self.semantic_directive_parts(annotation) else {
                continue;
            };
            match name.as_str() {
                "associativity" => {
                    if let Some(parsed) = SemanticAssociativity::parse(&payload) {
                        associativity = parsed;
                    }
                }
                "branch_policy" => {
                    if let Some(parsed) = SemanticBranchPolicy::parse(&payload) {
                        branch_policy = parsed;
                    }
                }
                "precedence" => {
                    let Some(parsed) = parse_semantic_branch_priorities(&payload, branch_count)
                    else {
                        continue;
                    };
                    precedence_priorities = Some(parsed);
                }
                "priority" => {
                    let Some(parsed) = parse_semantic_branch_priorities(&payload, branch_count)
                    else {
                        continue;
                    };
                    explicit_priorities = Some(parsed);
                }
                _ => {}
            }
        }

        let priorities = explicit_priorities
            .or(precedence_priorities)
            .unwrap_or(default_priorities);
        (branch_policy, associativity, priorities)
    }

    fn rule_value_constraints(&self, rule_name: &str) -> SemanticValueConstraints {
        let mut constraints = SemanticValueConstraints::default();
        let Some(annotations) = self.annotations else {
            return constraints;
        };
        let Some(entries) = annotations.semantic_annotations.get(rule_name) else {
            return constraints;
        };

        for annotation in entries {
            let Some((name, payload)) = self.semantic_directive_parts(annotation) else {
                continue;
            };
            match name.as_str() {
                "enum" => {
                    if let Some(values) = parse_semantic_string_list(&payload) {
                        constraints.enum_values = values;
                    }
                }
                "regex" => {
                    let pattern = normalize_semantic_scalar(&payload);
                    if !pattern.is_empty() {
                        constraints.regex_pattern = Some(pattern);
                    }
                }
                "range" => {
                    if let Some((min, max)) = parse_semantic_numeric_bounds(&payload) {
                        constraints.min_numeric = Some(min);
                        constraints.max_numeric = Some(max);
                    }
                }
                "len" => {
                    if let Some((min_len, max_len)) = parse_semantic_len_bounds(&payload) {
                        constraints.min_len = Some(min_len);
                        constraints.max_len = Some(max_len);
                    }
                }
                _ => {}
            }
        }

        constraints
    }

    fn rule_token_steering_policy(&self, rule_name: &str) -> StimuliTokenSteeringPolicy {
        let Some(annotations) = self.annotations else {
            return StimuliTokenSteeringPolicy::default();
        };
        let Some(entries) = annotations.semantic_annotations.get(rule_name) else {
            return StimuliTokenSteeringPolicy::default();
        };

        let mut policy = StimuliTokenSteeringPolicy::default();
        for annotation in entries {
            let Some((name, payload)) = self.semantic_directive_parts(annotation) else {
                continue;
            };
            match name.as_str() {
                "token_class" => {
                    if let Some(parsed) = parse_semantic_token_class(&payload) {
                        policy.token_class = Some(parsed);
                    }
                }
                "charset" => {
                    if let Some(pattern) = parse_semantic_charset(&payload) {
                        policy.charset_pattern = Some(pattern);
                    }
                }
                "pattern" => {
                    if let Some(pattern) = parse_semantic_pattern(&payload) {
                        policy.explicit_pattern = Some(pattern);
                    }
                }
                _ => {}
            }
        }

        policy
    }

    fn effective_regex_pattern(&self, rule_name: &str, grammar_pattern: &str) -> String {
        let policy = self.rule_token_steering_policy(rule_name);
        if let Some(pattern) = policy.explicit_pattern {
            return pattern;
        }
        if let Some(pattern) = policy.charset_pattern {
            return pattern;
        }
        if let Some(token_class) = policy.token_class {
            return token_class.regex_pattern().to_string();
        }
        grammar_pattern.to_string()
    }

    fn rule_coverage_steering_policy(&self, rule_name: &str) -> StimuliCoverageSteeringPolicy {
        let mut policy = StimuliCoverageSteeringPolicy::default();
        let Some(annotations) = self.annotations else {
            return policy;
        };
        let Some(entries) = annotations.semantic_annotations.get(rule_name) else {
            return policy;
        };

        for annotation in entries {
            let Some((name, payload)) = self.semantic_directive_parts(annotation) else {
                continue;
            };
            match name.as_str() {
                "coverage_target" => {
                    if let Some(weight) = parse_semantic_coverage_target_weight(&payload) {
                        policy.coverage_target_weight = weight;
                    }
                }
                "critical_path" => {
                    if let Some(enabled) = parse_semantic_bool(&payload) {
                        policy.critical_path = enabled;
                    }
                }
                _ => {}
            }
        }

        policy
    }

    fn rule_negative_case_policy(&self, rule_name: &str) -> StimuliNegativeCasePolicy {
        let mut policy = StimuliNegativeCasePolicy::default();
        let Some(annotations) = self.annotations else {
            return policy;
        };
        let Some(entries) = annotations.semantic_annotations.get(rule_name) else {
            return policy;
        };

        for annotation in entries {
            let Some((name, payload)) = self.semantic_directive_parts(annotation) else {
                continue;
            };
            match name.as_str() {
                "invalid_case" => {
                    if let Some(enabled) = parse_semantic_bool(&payload) {
                        policy.invalid_case = enabled;
                    }
                }
                "negative" => {
                    if let Some(enabled) = parse_semantic_bool(&payload) {
                        policy.negative = enabled;
                    }
                }
                _ => {}
            }
        }

        if !policy.invalid_case {
            policy.negative = false;
        }
        policy
    }

    fn rule_determinism_partition_policy(
        &self,
        rule_name: &str,
    ) -> StimuliDeterminismPartitionPolicy {
        let mut policy = StimuliDeterminismPartitionPolicy::default();
        let Some(annotations) = self.annotations else {
            return policy;
        };
        let Some(entries) = annotations.semantic_annotations.get(rule_name) else {
            return policy;
        };

        for annotation in entries {
            let Some((name, payload)) = self.semantic_directive_parts(annotation) else {
                continue;
            };
            match name.as_str() {
                "seed_group" => {
                    if let Some(label) = parse_semantic_group_label(&payload) {
                        policy.group_label = Some(label);
                    }
                }
                "deterministic_group" => {
                    if let Some(hint) = parse_semantic_deterministic_group(&payload) {
                        policy.enabled = hint.enabled;
                        if let Some(label) = hint.group {
                            policy.group_label = Some(label);
                        }
                    }
                }
                _ => {}
            }
        }

        if !policy.enabled {
            policy.group_label = None;
        }
        policy
    }

    fn apply_negative_case_policy(&self, rule_name: &str, sample: String) -> String {
        let policy = self.rule_negative_case_policy(rule_name);
        if !policy.invalid_case {
            return sample;
        }

        if policy.negative {
            return format!("{}{}", sample, Self::negative_case_suffix(rule_name));
        }

        let mut chars: Vec<char> = sample.chars().collect();
        if chars.len() > 1 {
            chars.pop();
            chars.into_iter().collect()
        } else {
            format!("{}{}", sample, Self::negative_case_suffix(rule_name))
        }
    }

    fn semantic_coverage_priority_bonus(&self, rule_name: &str) -> u64 {
        let policy = self.rule_coverage_steering_policy(rule_name);
        let mut bonus = 0u64;
        if policy.coverage_target_weight > 0 {
            bonus = bonus.saturating_add(
                200u64
                    .saturating_mul(policy.coverage_target_weight.min(16))
                    .max(200),
            );
        }
        if policy.critical_path {
            bonus = bonus.saturating_add(640);
        }
        bonus
    }

    fn semantic_coverage_guidance_multiplier(
        &self,
        current_rule: &str,
        branch_node: &ASTNode,
    ) -> u64 {
        let current_policy = self.rule_coverage_steering_policy(current_rule);
        let mut multiplier = 1u64;

        if current_policy.coverage_target_weight > 0 {
            multiplier =
                multiplier.saturating_mul(2 + current_policy.coverage_target_weight.min(16));
        }
        if current_policy.critical_path {
            multiplier = multiplier.saturating_mul(4);
        }

        let mut refs = HashSet::new();
        self.collect_rule_references(branch_node, &mut refs);
        let mut targeted_refs = 0u64;
        let mut critical_refs = 0u64;
        for rule_name in refs {
            let policy = self.rule_coverage_steering_policy(rule_name.as_str());
            if policy.coverage_target_weight > 0 {
                targeted_refs = targeted_refs.saturating_add(1);
            }
            if policy.critical_path {
                critical_refs = critical_refs.saturating_add(1);
            }
        }
        if targeted_refs > 0 {
            multiplier = multiplier.saturating_mul(1 + targeted_refs.min(8).saturating_mul(2));
        }
        if critical_refs > 0 {
            multiplier = multiplier.saturating_mul(1 + critical_refs.min(8).saturating_mul(3));
        }

        multiplier.max(1)
    }

    fn deterministic_partition_seed(base_seed: u64, group_key: &str, ordinal: u64) -> u64 {
        let mut state = base_seed ^ 0x9E37_79B9_7F4A_7C15;
        for byte in group_key.as_bytes() {
            state ^= *byte as u64;
            state = state.wrapping_mul(1_099_511_628_211);
        }
        state ^= ordinal.wrapping_mul(0xD6E8_FEB8_6659_FD93);
        state ^= state >> 33;
        state = state.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
        state ^= state >> 33;
        state = state.wrapping_mul(0xC4CE_B9FE_1A85_EC53);
        state ^= state >> 33;
        if state == 0 { 1 } else { state }
    }

    fn relational_attempt_budget(&self) -> usize {
        self.config.max_repeat.max(4).saturating_mul(8).max(8)
    }

    fn sequence_element_capture_name(element: &ASTNode) -> Option<String> {
        let ASTNode::Atom { value } = element else {
            return None;
        };
        let ASTValue::Token(parts) = value else {
            return None;
        };
        let (token_type, token_value) = Self::extract_token_pair(parts)?;
        if token_type != "rule_reference" {
            return None;
        }
        let trimmed = token_value.trim();
        if trimmed.is_empty() {
            return None;
        }
        Some(trimmed.to_string())
    }

    fn rule_relational_constraints(&self, rule_name: &str) -> StimuliRelationalConstraintPolicy {
        let mut policy = StimuliRelationalConstraintPolicy::default();
        let Some(annotations) = self.annotations else {
            return policy;
        };
        let Some(entries) = annotations.semantic_annotations.get(rule_name) else {
            return policy;
        };

        for annotation in entries {
            let Some((name, payload)) = self.semantic_directive_parts(annotation) else {
                continue;
            };
            match name.as_str() {
                "constraint" => {
                    if let Some(parsed) = parse_semantic_constraint_expression(&payload) {
                        policy.constraint_expression = Some(parsed);
                    }
                }
                "requires" => {
                    if let Some(parsed) = parse_semantic_reference_list(&payload) {
                        policy.requires_references = parsed;
                    }
                }
                "implies" => {
                    if let Some(parsed) = parse_semantic_implication(&payload) {
                        policy.implication = Some(parsed);
                    }
                }
                _ => {}
            }
        }

        if policy.constraint_expression.is_none() {
            policy.requires_references.clear();
            policy.implication = None;
        }

        policy
    }

    fn validate_relational_sample(
        &self,
        rule_name: &str,
        policy: &StimuliRelationalConstraintPolicy,
        captures: &[String],
        named_captures: &HashMap<String, String>,
    ) -> Result<()> {
        let Some(constraint_expression) = policy.constraint_expression.as_deref() else {
            return Ok(());
        };

        self.enforce_relational_requires_for_sample(
            rule_name,
            captures,
            named_captures,
            &policy.requires_references,
        )?;

        if !self.evaluate_relational_expression_for_sample(
            captures,
            named_captures,
            constraint_expression,
        )? {
            return Err(anyhow!(
                "Semantic relational constraint failed for rule '{}': {}",
                rule_name,
                constraint_expression
            ));
        }

        if let Some((antecedent, consequent)) = &policy.implication {
            if self.evaluate_relational_expression_for_sample(
                captures,
                named_captures,
                antecedent,
            )? && !self.evaluate_relational_expression_for_sample(
                captures,
                named_captures,
                consequent,
            )? {
                return Err(anyhow!(
                    "Semantic implication failed for rule '{}': {} => {}",
                    rule_name,
                    antecedent,
                    consequent
                ));
            }
        }

        Ok(())
    }

    fn enforce_relational_requires_for_sample(
        &self,
        rule_name: &str,
        captures: &[String],
        named_captures: &HashMap<String, String>,
        required_references: &[String],
    ) -> Result<()> {
        for reference in required_references {
            let normalized = reference.trim();
            if normalized.is_empty() {
                continue;
            }
            let Some(value) =
                self.resolve_semantic_reference_in_sample(captures, named_captures, normalized)
            else {
                return Err(anyhow!(
                    "Semantic @requires contract failed for rule '{}': unresolved reference '{}'",
                    rule_name,
                    normalized
                ));
            };
            if value.trim().is_empty() {
                return Err(anyhow!(
                    "Semantic @requires contract failed for rule '{}': empty reference '{}'",
                    rule_name,
                    normalized
                ));
            }
        }
        Ok(())
    }

    fn evaluate_relational_expression_for_sample(
        &self,
        captures: &[String],
        named_captures: &HashMap<String, String>,
        expression: &str,
    ) -> Result<bool> {
        let normalized = expression.trim();
        if normalized.is_empty() {
            return Err(anyhow!("Semantic relational expression cannot be empty"));
        }
        self.evaluate_relational_expression_inner_for_sample(captures, named_captures, normalized)
    }

    fn evaluate_relational_expression_inner_for_sample(
        &self,
        captures: &[String],
        named_captures: &HashMap<String, String>,
        expression: &str,
    ) -> Result<bool> {
        let mut normalized = expression.trim();
        while Self::semantic_encloses_full_parens(normalized) {
            normalized = normalized[1..normalized.len() - 1].trim();
        }

        let disjuncts = Self::split_semantic_top_level(normalized, "||");
        if disjuncts.len() > 1 {
            for term in disjuncts {
                if term.is_empty() {
                    continue;
                }
                if self.evaluate_relational_expression_inner_for_sample(
                    captures,
                    named_captures,
                    term,
                )? {
                    return Ok(true);
                }
            }
            return Ok(false);
        }

        let conjuncts = Self::split_semantic_top_level(normalized, "&&");
        if conjuncts.len() > 1 {
            for term in conjuncts {
                if term.is_empty() {
                    continue;
                }
                if !self.evaluate_relational_expression_inner_for_sample(
                    captures,
                    named_captures,
                    term,
                )? {
                    return Ok(false);
                }
            }
            return Ok(true);
        }

        if let Some(rest) = normalized.strip_prefix('!') {
            return Ok(!self.evaluate_relational_expression_inner_for_sample(
                captures,
                named_captures,
                rest,
            )?);
        }

        for operator in ["==", "!=", ">=", "<=", ">", "<"] {
            if let Some((left, right)) = Self::split_semantic_top_level_once(normalized, operator) {
                return self.evaluate_relational_comparison_for_sample(
                    captures,
                    named_captures,
                    left,
                    operator,
                    right,
                );
            }
        }

        if let Some(unquoted) = Self::semantic_unquote(normalized) {
            return Ok(Self::semantic_truthy(unquoted));
        }

        let lowered = normalized.to_ascii_lowercase();
        if lowered == "true" {
            return Ok(true);
        }
        if lowered == "false" {
            return Ok(false);
        }

        if let Ok(number) = normalized.parse::<f64>() {
            return Ok(number != 0.0);
        }

        if Self::semantic_reference_syntax(normalized) {
            let value = self
                .resolve_semantic_reference_in_sample(captures, named_captures, normalized)
                .ok_or_else(|| {
                    anyhow!(
                        "Semantic relational expression references unresolved capture '{}'",
                        normalized
                    )
                })?;
            return Ok(Self::semantic_truthy(&value));
        }

        Ok(Self::semantic_truthy(normalized))
    }

    fn evaluate_relational_comparison_for_sample(
        &self,
        captures: &[String],
        named_captures: &HashMap<String, String>,
        left: &str,
        operator: &str,
        right: &str,
    ) -> Result<bool> {
        let lhs = self.resolve_relational_operand_for_sample(captures, named_captures, left)?;
        let rhs = self.resolve_relational_operand_for_sample(captures, named_captures, right)?;
        let lhs_numeric = lhs.parse::<f64>().ok();
        let rhs_numeric = rhs.parse::<f64>().ok();

        match operator {
            "==" => {
                if let (Some(a), Some(b)) = (lhs_numeric, rhs_numeric) {
                    Ok((a - b).abs() <= f64::EPSILON)
                } else {
                    Ok(lhs == rhs)
                }
            }
            "!=" => {
                if let (Some(a), Some(b)) = (lhs_numeric, rhs_numeric) {
                    Ok((a - b).abs() > f64::EPSILON)
                } else {
                    Ok(lhs != rhs)
                }
            }
            ">" => {
                if let (Some(a), Some(b)) = (lhs_numeric, rhs_numeric) {
                    Ok(a > b)
                } else {
                    Ok(lhs > rhs)
                }
            }
            ">=" => {
                if let (Some(a), Some(b)) = (lhs_numeric, rhs_numeric) {
                    Ok(a >= b)
                } else {
                    Ok(lhs >= rhs)
                }
            }
            "<" => {
                if let (Some(a), Some(b)) = (lhs_numeric, rhs_numeric) {
                    Ok(a < b)
                } else {
                    Ok(lhs < rhs)
                }
            }
            "<=" => {
                if let (Some(a), Some(b)) = (lhs_numeric, rhs_numeric) {
                    Ok(a <= b)
                } else {
                    Ok(lhs <= rhs)
                }
            }
            _ => Err(anyhow!(
                "Unsupported semantic comparison operator '{}'",
                operator
            )),
        }
    }

    fn resolve_relational_operand_for_sample(
        &self,
        captures: &[String],
        named_captures: &HashMap<String, String>,
        operand: &str,
    ) -> Result<String> {
        let normalized = operand.trim();
        if normalized.is_empty() {
            return Err(anyhow!("Semantic relational operand cannot be empty"));
        }

        if let Some(unquoted) = Self::semantic_unquote(normalized) {
            return Ok(unquoted.to_string());
        }

        let lowered = normalized.to_ascii_lowercase();
        if lowered == "true" || lowered == "false" {
            return Ok(lowered);
        }

        if normalized.parse::<f64>().is_ok() {
            return Ok(normalized.to_string());
        }

        if Self::semantic_reference_syntax(normalized) {
            return self
                .resolve_semantic_reference_in_sample(captures, named_captures, normalized)
                .ok_or_else(|| {
                    anyhow!(
                        "Semantic relational operand references unresolved capture '{}'",
                        normalized
                    )
                });
        }

        Ok(normalized.to_string())
    }

    fn resolve_semantic_reference_in_sample(
        &self,
        captures: &[String],
        named_captures: &HashMap<String, String>,
        reference: &str,
    ) -> Option<String> {
        let normalized = reference.trim();
        if normalized.is_empty() {
            return None;
        }

        let (core_reference, wants_len) = if let Some(stripped) = normalized.strip_suffix(".len") {
            (stripped.trim(), true)
        } else {
            (normalized, false)
        };

        let resolved = if core_reference.starts_with('$') {
            self.resolve_positional_reference_in_sample(captures, core_reference)
        } else {
            self.resolve_named_reference_in_sample(named_captures, core_reference)
        }?;

        if wants_len {
            Some(resolved.chars().count().to_string())
        } else {
            Some(resolved)
        }
    }

    fn resolve_positional_reference_in_sample(
        &self,
        captures: &[String],
        reference: &str,
    ) -> Option<String> {
        let (index, path_segments) = Self::parse_positional_reference_segments(reference)?;
        let base_value = captures.get(index.saturating_sub(1))?;
        if path_segments.is_empty() {
            return Some(base_value.clone());
        }
        self.resolve_capture_path_value(base_value, &path_segments)
    }

    fn resolve_named_reference_in_sample(
        &self,
        named_captures: &HashMap<String, String>,
        reference: &str,
    ) -> Option<String> {
        let normalized = reference.trim();
        if normalized.is_empty() {
            return None;
        }

        if let Some(value) = named_captures.get(normalized) {
            return Some(value.clone());
        }

        let mut segments = normalized
            .split('.')
            .map(str::trim)
            .filter(|s| !s.is_empty());
        let first = segments.next()?;
        if !Self::semantic_identifier(first) {
            return None;
        }
        let mut path_segments = Vec::new();
        for segment in segments {
            if !Self::semantic_identifier(segment) {
                return None;
            }
            path_segments.push(segment);
        }
        let base_value = named_captures.get(first)?;
        if path_segments.is_empty() {
            return Some(base_value.clone());
        }
        self.resolve_capture_path_value(base_value, &path_segments)
    }

    fn resolve_capture_path_value(&self, raw: &str, path_segments: &[&str]) -> Option<String> {
        if path_segments.is_empty() {
            return Some(raw.to_string());
        }

        let mut current = Self::parse_capture_value_as_json(raw)?;
        for segment in path_segments {
            current = match current {
                JsonValue::Object(ref object) => object.get(*segment)?.clone(),
                JsonValue::Array(ref array) => {
                    let index = segment.parse::<usize>().ok()?;
                    array.get(index)?.clone()
                }
                _ => return None,
            };
        }

        Self::json_value_to_scalar_string(&current)
    }

    fn parse_capture_value_as_json(raw: &str) -> Option<JsonValue> {
        let normalized = raw.trim();
        if normalized.is_empty() {
            return None;
        }

        if let Ok(parsed) = serde_json::from_str::<JsonValue>(normalized) {
            return Some(parsed);
        }

        if let Some(unquoted) = Self::semantic_unquote(normalized) {
            let unquoted_trimmed = unquoted.trim();
            if let Ok(parsed) = serde_json::from_str::<JsonValue>(unquoted_trimmed) {
                return Some(parsed);
            }
            if let Some(parsed) = Self::parse_nonstructured_capture_object(unquoted_trimmed, 0) {
                return Some(parsed);
            }
        }

        Self::parse_nonstructured_capture_object(normalized, 0)
    }

    fn parse_nonstructured_capture_object(raw: &str, depth: usize) -> Option<JsonValue> {
        if depth > 4 {
            return None;
        }

        let normalized = raw.trim();
        if normalized.is_empty() {
            return None;
        }

        let candidate = Self::strip_balanced_outer_wrappers(normalized);
        if candidate.is_empty() || (!candidate.contains('=') && !candidate.contains(':')) {
            return None;
        }

        let mut object = serde_json::Map::new();
        let mut parsed_any = false;

        for pair in Self::split_nonstructured_capture_pairs(candidate) {
            let pair = pair.trim();
            if pair.is_empty() {
                continue;
            }

            let Some((raw_key, raw_value)) = Self::split_nonstructured_capture_key_value(pair)
            else {
                continue;
            };
            let Some(path_segments) = Self::parse_nonstructured_key_path(raw_key) else {
                continue;
            };

            let parsed_value = Self::parse_nonstructured_capture_scalar(raw_value, depth + 1);
            if Self::insert_nonstructured_capture_path(&mut object, &path_segments, parsed_value) {
                parsed_any = true;
            }
        }

        if parsed_any {
            Some(JsonValue::Object(object))
        } else {
            None
        }
    }

    fn parse_nonstructured_capture_scalar(raw: &str, depth: usize) -> JsonValue {
        let normalized = raw.trim();
        if normalized.is_empty() {
            return JsonValue::String(String::new());
        }

        if let Ok(parsed) = serde_json::from_str::<JsonValue>(normalized) {
            return parsed;
        }

        if let Some(unquoted) = Self::semantic_unquote(normalized) {
            let unquoted_trimmed = unquoted.trim();
            if let Ok(parsed) = serde_json::from_str::<JsonValue>(unquoted_trimmed) {
                return parsed;
            }
            if let Some(parsed) = Self::parse_nonstructured_capture_object(unquoted_trimmed, depth)
            {
                return parsed;
            }
            return JsonValue::String(unquoted.to_string());
        }

        match normalized.to_ascii_lowercase().as_str() {
            "true" => return JsonValue::Bool(true),
            "false" => return JsonValue::Bool(false),
            "null" => return JsonValue::Null,
            _ => {}
        }

        if let Ok(number) = normalized.parse::<i64>() {
            return JsonValue::Number(serde_json::Number::from(number));
        }
        if let Ok(number) = normalized.parse::<u64>() {
            return JsonValue::Number(serde_json::Number::from(number));
        }
        if let Ok(number) = normalized.parse::<f64>() {
            if let Some(parsed_number) = serde_json::Number::from_f64(number) {
                return JsonValue::Number(parsed_number);
            }
        }

        if let Some(parsed) = Self::parse_nonstructured_capture_object(normalized, depth) {
            return parsed;
        }

        JsonValue::String(normalized.to_string())
    }

    fn parse_nonstructured_key_path(raw: &str) -> Option<Vec<String>> {
        let normalized = raw.trim();
        if normalized.is_empty() {
            return None;
        }

        let normalized = if let Some(unquoted) = Self::semantic_unquote(normalized) {
            unquoted.trim()
        } else {
            normalized
        };
        if normalized.is_empty() {
            return None;
        }

        let mut segments = Vec::new();
        for segment in normalized.split('.') {
            let segment = segment.trim();
            if segment.is_empty() || !Self::semantic_identifier(segment) {
                return None;
            }
            segments.push(segment.to_string());
        }
        if segments.is_empty() {
            return None;
        }
        Some(segments)
    }

    fn insert_nonstructured_capture_path(
        object: &mut serde_json::Map<String, JsonValue>,
        segments: &[String],
        value: JsonValue,
    ) -> bool {
        if segments.is_empty() {
            return false;
        }
        if segments.len() == 1 {
            object.insert(segments[0].clone(), value);
            return true;
        }

        let entry = object
            .entry(segments[0].clone())
            .or_insert_with(|| JsonValue::Object(serde_json::Map::new()));
        if !matches!(entry, JsonValue::Object(_)) {
            *entry = JsonValue::Object(serde_json::Map::new());
        }
        let JsonValue::Object(next) = entry else {
            return false;
        };

        Self::insert_nonstructured_capture_path(next, &segments[1..], value)
    }

    fn strip_balanced_outer_wrappers(raw: &str) -> &str {
        let mut candidate = raw.trim();
        loop {
            if candidate.len() < 2 {
                break;
            }
            let bytes = candidate.as_bytes();
            let Some(last) = bytes.last().copied() else {
                break;
            };
            let first = bytes[0];
            let wraps = matches!((first, last), (b'{', b'}') | (b'(', b')') | (b'[', b']'));
            if !wraps || !Self::capture_encloses_full_wrapper(candidate) {
                break;
            }
            candidate = candidate[1..candidate.len() - 1].trim();
        }
        candidate
    }

    fn capture_encloses_full_wrapper(raw: &str) -> bool {
        let normalized = raw.trim();
        if normalized.len() < 2 {
            return false;
        }
        let bytes = normalized.as_bytes();
        let Some(last) = bytes.last().copied() else {
            return false;
        };
        let first = bytes[0];
        if !Self::matching_brace(first, last) {
            return false;
        }

        let mut stack: Vec<u8> = Vec::new();
        let mut quote: Option<u8> = None;
        for (idx, current) in bytes.iter().copied().enumerate() {
            if let Some(active_quote) = quote {
                if current == active_quote && (idx == 0 || bytes[idx - 1] != b'\\') {
                    quote = None;
                }
                continue;
            }

            match current {
                b'"' | b'\'' => {
                    quote = Some(current);
                }
                b'(' | b'[' | b'{' => stack.push(current),
                b')' | b']' | b'}' => {
                    let Some(open) = stack.pop() else {
                        return false;
                    };
                    if !Self::matching_brace(open, current) {
                        return false;
                    }
                    if stack.is_empty() && idx + 1 < bytes.len() {
                        return false;
                    }
                }
                _ => {}
            }
        }

        quote.is_none() && stack.is_empty()
    }

    fn split_nonstructured_capture_pairs<'b>(input: &'b str) -> Vec<&'b str> {
        let bytes = input.as_bytes();
        if bytes.is_empty() {
            return Vec::new();
        }

        let mut parts = Vec::new();
        let mut start = 0usize;
        let mut idx = 0usize;
        let mut quote: Option<u8> = None;
        let mut stack: Vec<u8> = Vec::new();

        while idx < bytes.len() {
            let current = bytes[idx];
            if let Some(active_quote) = quote {
                if current == active_quote && (idx == 0 || bytes[idx - 1] != b'\\') {
                    quote = None;
                }
                idx += 1;
                continue;
            }

            match current {
                b'"' | b'\'' => {
                    quote = Some(current);
                    idx += 1;
                    continue;
                }
                b'(' | b'[' | b'{' => {
                    stack.push(current);
                    idx += 1;
                    continue;
                }
                b')' | b']' | b'}' => {
                    if let Some(open) = stack.last().copied() {
                        if Self::matching_brace(open, current) {
                            stack.pop();
                        }
                    }
                    idx += 1;
                    continue;
                }
                b',' | b';' | b'\n' | b'\r' if stack.is_empty() => {
                    parts.push(input[start..idx].trim());
                    idx += 1;
                    start = idx;
                    continue;
                }
                _ => {}
            }

            idx += 1;
        }

        parts.push(input[start..].trim());
        parts
    }

    fn split_nonstructured_capture_key_value(pair: &str) -> Option<(&str, &str)> {
        let normalized = pair.trim();
        if normalized.is_empty() {
            return None;
        }

        let bytes = normalized.as_bytes();
        let mut quote: Option<u8> = None;
        let mut stack: Vec<u8> = Vec::new();

        for (idx, current) in bytes.iter().copied().enumerate() {
            if let Some(active_quote) = quote {
                if current == active_quote && (idx == 0 || bytes[idx - 1] != b'\\') {
                    quote = None;
                }
                continue;
            }

            match current {
                b'"' | b'\'' => {
                    quote = Some(current);
                    continue;
                }
                b'(' | b'[' | b'{' => {
                    stack.push(current);
                    continue;
                }
                b')' | b']' | b'}' => {
                    if let Some(open) = stack.last().copied() {
                        if Self::matching_brace(open, current) {
                            stack.pop();
                        }
                    }
                    continue;
                }
                b'=' | b':' if stack.is_empty() => {
                    let key = normalized[..idx].trim();
                    let value = normalized[idx + 1..].trim();
                    if key.is_empty() || value.is_empty() {
                        return None;
                    }
                    return Some((key, value));
                }
                _ => {}
            }
        }

        None
    }

    fn matching_brace(open: u8, close: u8) -> bool {
        matches!((open, close), (b'(', b')') | (b'[', b']') | (b'{', b'}'))
    }

    fn json_value_to_scalar_string(value: &JsonValue) -> Option<String> {
        match value {
            JsonValue::Null => Some("null".to_string()),
            JsonValue::Bool(inner) => Some(inner.to_string()),
            JsonValue::Number(inner) => Some(inner.to_string()),
            JsonValue::String(inner) => Some(inner.clone()),
            JsonValue::Array(_) | JsonValue::Object(_) => serde_json::to_string(value).ok(),
        }
    }

    fn parse_positional_reference_segments(reference: &str) -> Option<(usize, Vec<&str>)> {
        let normalized = reference.trim();
        if !normalized.starts_with('$') {
            return None;
        }

        let bytes = normalized.as_bytes();
        let mut index_end = 1usize;
        while index_end < bytes.len() && bytes[index_end].is_ascii_digit() {
            index_end += 1;
        }
        if index_end == 1 {
            return None;
        }

        let index = normalized[1..index_end].parse::<usize>().ok()?;
        if index == 0 {
            return None;
        }

        let mut segments = Vec::new();
        let suffix = normalized[index_end..].trim();
        if suffix.is_empty() {
            return Some((index, segments));
        }
        if !suffix.starts_with('.') {
            return None;
        }

        for segment in suffix[1..].split('.') {
            let normalized_segment = segment.trim();
            if normalized_segment.is_empty() || !Self::semantic_identifier(normalized_segment) {
                return None;
            }
            segments.push(normalized_segment);
        }

        Some((index, segments))
    }

    fn semantic_reference_syntax(reference: &str) -> bool {
        let normalized = reference.trim();
        if normalized.is_empty() {
            return false;
        }

        if normalized.starts_with('$') {
            return Self::parse_positional_reference_segments(normalized).is_some();
        }

        let mut segments = normalized.split('.');
        let Some(first) = segments.next() else {
            return false;
        };
        if !Self::semantic_identifier(first) {
            return false;
        }
        segments.all(Self::semantic_identifier)
    }

    fn semantic_identifier(segment: &str) -> bool {
        let bytes = segment.as_bytes();
        let Some(first) = bytes.first() else {
            return false;
        };
        if !(*first == b'_' || (*first as char).is_ascii_alphabetic()) {
            return false;
        }
        bytes[1..]
            .iter()
            .all(|b| *b == b'_' || (*b as char).is_ascii_alphanumeric())
    }

    fn split_semantic_top_level<'b>(expression: &'b str, separator: &str) -> Vec<&'b str> {
        if separator.is_empty() {
            return vec![expression.trim()];
        }

        let bytes = expression.as_bytes();
        let separator_bytes = separator.as_bytes();
        if separator_bytes.is_empty() || bytes.len() < separator_bytes.len() {
            return vec![expression.trim()];
        }

        let mut parts = Vec::new();
        let mut start = 0usize;
        let mut idx = 0usize;
        let mut depth = 0usize;
        let mut quote: Option<u8> = None;

        while idx < bytes.len() {
            let current = bytes[idx];
            if let Some(active_quote) = quote {
                if current == active_quote && (idx == 0 || bytes[idx - 1] != b'\\') {
                    quote = None;
                }
                idx += 1;
                continue;
            }

            match current {
                b'"' | b'\'' => {
                    quote = Some(current);
                    idx += 1;
                    continue;
                }
                b'(' => {
                    depth += 1;
                    idx += 1;
                    continue;
                }
                b')' => {
                    depth = depth.saturating_sub(1);
                    idx += 1;
                    continue;
                }
                _ => {}
            }

            if depth == 0
                && idx + separator_bytes.len() <= bytes.len()
                && &bytes[idx..idx + separator_bytes.len()] == separator_bytes
            {
                parts.push(expression[start..idx].trim());
                idx += separator_bytes.len();
                start = idx;
                continue;
            }

            idx += 1;
        }

        parts.push(expression[start..].trim());
        parts
    }

    fn split_semantic_top_level_once<'b>(
        expression: &'b str,
        separator: &str,
    ) -> Option<(&'b str, &'b str)> {
        let pieces = Self::split_semantic_top_level(expression, separator);
        if pieces.len() != 2 {
            return None;
        }
        Some((pieces[0], pieces[1]))
    }

    fn semantic_encloses_full_parens(expression: &str) -> bool {
        let normalized = expression.trim();
        if normalized.len() < 2 || !normalized.starts_with('(') || !normalized.ends_with(')') {
            return false;
        }

        let bytes = normalized.as_bytes();
        let mut depth = 0usize;
        let mut quote: Option<u8> = None;

        for (idx, current) in bytes.iter().enumerate() {
            let current = *current;
            if let Some(active_quote) = quote {
                if current == active_quote && (idx == 0 || bytes[idx - 1] != b'\\') {
                    quote = None;
                }
                continue;
            }

            match current {
                b'"' | b'\'' => {
                    quote = Some(current);
                }
                b'(' => depth += 1,
                b')' => {
                    if depth == 0 {
                        return false;
                    }
                    depth -= 1;
                    if depth == 0 && idx + 1 < bytes.len() {
                        return false;
                    }
                }
                _ => {}
            }
        }

        depth == 0 && quote.is_none()
    }

    fn semantic_unquote(value: &str) -> Option<&str> {
        let normalized = value.trim();
        if normalized.len() >= 2
            && ((normalized.starts_with('"') && normalized.ends_with('"'))
                || (normalized.starts_with('\'') && normalized.ends_with('\'')))
        {
            return Some(&normalized[1..normalized.len() - 1]);
        }
        None
    }

    fn semantic_truthy(value: &str) -> bool {
        let normalized = value.trim();
        if normalized.is_empty() {
            return false;
        }
        let lowered = normalized
            .trim_matches('"')
            .trim_matches('\'')
            .trim()
            .to_ascii_lowercase();
        !matches!(
            lowered.as_str(),
            "" | "false" | "0" | "no" | "off" | "none" | "null"
        )
    }

    fn rule_recovery_controls(&self, rule_name: &str) -> (bool, Vec<String>, Vec<String>) {
        let Some(annotations) = self.annotations else {
            return (false, Vec::new(), Vec::new());
        };
        let Some(entries) = annotations.semantic_annotations.get(rule_name) else {
            return (false, Vec::new(), Vec::new());
        };

        let mut recover_enabled = false;
        let mut sync_tokens = Vec::new();
        let mut panic_until_tokens = Vec::new();
        for annotation in entries {
            let Some((name, payload)) = self.semantic_directive_parts(annotation) else {
                continue;
            };
            match name.as_str() {
                "recover" => {
                    if let Some(parsed) = parse_semantic_bool(&payload) {
                        recover_enabled = parsed;
                    }
                }
                "sync" => {
                    if let Some(parsed) = parse_semantic_string_list(&payload) {
                        sync_tokens = parsed;
                    }
                }
                "panic_until" => {
                    if let Some(parsed) = parse_semantic_string_list(&payload) {
                        panic_until_tokens = parsed;
                    }
                }
                _ => {}
            }
        }

        (recover_enabled, sync_tokens, panic_until_tokens)
    }

    fn recovery_stimulus_fallback(&self, rule_name: &str) -> Option<String> {
        let (recover_enabled, sync_tokens, panic_until_tokens) =
            self.rule_recovery_controls(rule_name);
        if !recover_enabled {
            return None;
        }

        if let Some(marker) = panic_until_tokens
            .into_iter()
            .find(|token| !token.trim().is_empty())
        {
            return Some(marker);
        }

        sync_tokens
            .into_iter()
            .find(|token| !token.trim().is_empty())
    }

    fn near_sync_negative_prefix(rule_name: &str) -> String {
        let mut sanitized = String::with_capacity(rule_name.len());
        for ch in rule_name.chars() {
            if ch.is_ascii_alphanumeric() {
                sanitized.push(ch);
            } else {
                sanitized.push('_');
            }
        }
        if sanitized.is_empty() {
            sanitized.push_str("rule");
        }
        format!("__pgen_near_sync_{}__", sanitized)
    }

    fn negative_case_suffix(rule_name: &str) -> String {
        let mut sanitized = String::with_capacity(rule_name.len());
        for ch in rule_name.chars() {
            if ch.is_ascii_alphanumeric() {
                sanitized.push(ch);
            } else {
                sanitized.push('_');
            }
        }
        if sanitized.is_empty() {
            sanitized.push_str("rule");
        }
        format!("__pgen_negative_case_{}__", sanitized)
    }

    fn semantic_branch_multiplier(
        &self,
        associativity: SemanticAssociativity,
        priorities: &[i64],
        branch_index: usize,
        branch_count: usize,
    ) -> u64 {
        let priority_component = if priorities.is_empty() {
            1
        } else {
            let min = priorities.iter().copied().min().unwrap_or(0);
            let value = priorities.get(branch_index).copied().unwrap_or(min);
            value.saturating_sub(min).saturating_add(1) as u64
        };

        let associativity_component = match associativity {
            SemanticAssociativity::Left => branch_count.saturating_sub(branch_index) as u64,
            SemanticAssociativity::Right => branch_index.saturating_add(1) as u64,
            SemanticAssociativity::NonAssoc => 1,
        };

        priority_component
            .max(1)
            .saturating_mul(associativity_component.max(1))
    }

    fn semantic_directive_name(&self, annotation: &SemanticAnnotation) -> Option<String> {
        self.semantic_directive_parts(annotation)
            .map(|(name, _)| name)
    }

    fn semantic_directive_parts(
        &self,
        annotation: &SemanticAnnotation,
    ) -> Option<(String, String)> {
        if let Some(name) = annotation.name() {
            let normalized = name.trim().to_ascii_lowercase();
            if !normalized.is_empty() {
                let payload = match annotation.ast() {
                    UnifiedSemanticAST::TransformExpr { expression } => expression.clone(),
                    UnifiedSemanticAST::Raw { content } => content.clone(),
                };
                return Some((normalized, payload.trim().to_string()));
            }
        }

        match annotation.ast() {
            UnifiedSemanticAST::TransformExpr { expression } => {
                if let Some(parts) = extract_semantic_directive(expression) {
                    return Some(parts);
                }
                Some(("transform".to_string(), expression.clone()))
            }
            UnifiedSemanticAST::Raw { content } => extract_semantic_directive(content),
        }
    }

    fn extract_token_pair(parts: &[TokenValue]) -> Option<(&str, &str)> {
        if parts.len() < 2 {
            return None;
        }
        let TokenValue::String(token_type) = &parts[0] else {
            return None;
        };
        let TokenValue::String(token_value) = &parts[1] else {
            return None;
        };
        Some((token_type.as_str(), token_value.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    use serde_json::Value as JsonValue;

    fn token(token_type: &str, token_value: &str) -> ASTNode {
        ASTNode::Atom {
            value: ASTValue::Token(vec![
                TokenValue::String(token_type.to_string()),
                TokenValue::String(token_value.to_string()),
            ]),
        }
    }

    fn simple_generator<'a>(
        grammar_tree: &'a HashMap<String, ASTNode>,
        rule_order: &'a [String],
        seed: u64,
    ) -> StimuliGenerator<'a> {
        StimuliGenerator::new(
            "test".to_string(),
            grammar_tree,
            rule_order,
            None,
            StimuliConfig {
                seed: Some(seed),
                max_depth: 8,
                max_repeat: 4,
                max_rule_visits: 4,
                recovery_mode: RecoveryStimuliMode::Baseline,
                enforce_word_boundary_spacing: false,
                trace_verbosity: TraceVerbosity::None,
            },
        )
    }

    fn annotated_generator<'a>(
        grammar_tree: &'a HashMap<String, ASTNode>,
        rule_order: &'a [String],
        annotations: &'a Annotations,
        seed: u64,
    ) -> StimuliGenerator<'a> {
        StimuliGenerator::new(
            "test".to_string(),
            grammar_tree,
            rule_order,
            Some(annotations),
            StimuliConfig {
                seed: Some(seed),
                max_depth: 8,
                max_repeat: 4,
                max_rule_visits: 4,
                recovery_mode: RecoveryStimuliMode::Baseline,
                enforce_word_boundary_spacing: false,
                trace_verbosity: TraceVerbosity::None,
            },
        )
    }

    fn annotated_generator_with_mode<'a>(
        grammar_tree: &'a HashMap<String, ASTNode>,
        rule_order: &'a [String],
        annotations: &'a Annotations,
        seed: u64,
        recovery_mode: RecoveryStimuliMode,
    ) -> StimuliGenerator<'a> {
        StimuliGenerator::new(
            "test".to_string(),
            grammar_tree,
            rule_order,
            Some(annotations),
            StimuliConfig {
                seed: Some(seed),
                max_depth: 8,
                max_repeat: 4,
                max_rule_visits: 4,
                recovery_mode,
                enforce_word_boundary_spacing: false,
                trace_verbosity: TraceVerbosity::None,
            },
        )
    }

    #[test]
    fn weighted_probabilities_are_deterministic_with_seed() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    ASTNode::Sequence {
                        elements: vec![token("probability", "70"), token("quoted_string", "A")],
                    },
                    ASTNode::Sequence {
                        elements: vec![token("probability", "30"), token("quoted_string", "B")],
                    },
                ],
            },
        );
        let rule_order = vec!["start".to_string()];

        let mut gen_a = simple_generator(&grammar_tree, &rule_order, 123);
        let mut gen_b = simple_generator(&grammar_tree, &rule_order, 123);

        let a = gen_a
            .generate_many(64, None)
            .expect("generation should pass");
        let b = gen_b
            .generate_many(64, None)
            .expect("generation should pass");
        assert_eq!(a, b, "same seed should produce identical stimuli sequence");

        let count_a = a.iter().filter(|v| v.as_str() == "A").count();
        let count_b = a.iter().filter(|v| v.as_str() == "B").count();
        assert!(count_a > count_b, "70/30 weighting should bias toward A");
    }

    #[test]
    fn missing_probabilities_fallback_to_equal_weights() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![token("quoted_string", "L"), token("quoted_string", "R")],
            },
        );
        let rule_order = vec!["start".to_string()];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 7);
        let values = generator
            .generate_many(40, None)
            .expect("equal-weight generation should pass");

        assert!(
            values.iter().any(|v| v == "L"),
            "expected at least one left branch"
        );
        assert!(
            values.iter().any(|v| v == "R"),
            "expected at least one right branch"
        );
    }

    #[test]
    fn semantic_priority_directive_biases_branch_selection() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![token("quoted_string", "L"), token("quoted_string", "R")],
            },
        );
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![SemanticAnnotation::Named {
                name: "priority".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: "[1, 12]".to_string(),
                },
            }],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 17);
        let values = generator
            .generate_many(80, None)
            .expect("priority-biased generation should pass");

        let left_count = values.iter().filter(|v| v.as_str() == "L").count();
        let right_count = values.iter().filter(|v| v.as_str() == "R").count();
        assert!(
            right_count > left_count,
            "priority directive should bias toward higher-priority branch"
        );
    }

    #[test]
    fn semantic_priority_overrides_precedence_regardless_of_order() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![token("quoted_string", "L"), token("quoted_string", "R")],
            },
        );
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "priority".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[1, 12]".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "precedence".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[12, 1]".to_string(),
                    },
                },
            ],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 29);
        let values = generator
            .generate_many(80, None)
            .expect("priority-vs-precedence generation should pass");

        let left_count = values.iter().filter(|v| v.as_str() == "L").count();
        let right_count = values.iter().filter(|v| v.as_str() == "R").count();
        assert!(
            right_count > left_count,
            "priority should deterministically override precedence for branch steering"
        );
    }

    #[test]
    fn semantic_branch_policy_ordered_prefers_first_successful_branch() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![token("quoted_string", "L"), token("quoted_string", "R")],
            },
        );
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "branch_policy".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "ordered".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "priority".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[1, 99]".to_string(),
                    },
                },
            ],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 2110);
        let values = generator
            .generate_many(24, None)
            .expect("ordered branch policy generation should succeed");

        assert!(
            values.iter().all(|value| value == "L"),
            "ordered policy should keep first successful branch, got {:?}",
            values
        );
    }

    #[test]
    fn semantic_branch_policy_priority_first_prefers_high_priority_branch() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![token("quoted_string", "L"), token("quoted_string", "R")],
            },
        );
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "branch_policy".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "priority_first".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "priority".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[1, 99]".to_string(),
                    },
                },
            ],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 2111);
        let values = generator
            .generate_many(24, None)
            .expect("priority-first branch policy generation should succeed");

        assert!(
            values.iter().all(|value| value == "R"),
            "priority_first policy should prioritize highest-priority branch, got {:?}",
            values
        );
    }

    #[test]
    fn semantic_associativity_right_biases_ties_to_later_branches() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![token("quoted_string", "L"), token("quoted_string", "R")],
            },
        );
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![SemanticAnnotation::Named {
                name: "associativity".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: "right".to_string(),
                },
            }],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 23);
        let values = generator
            .generate_many(80, None)
            .expect("associativity-biased generation should pass");

        let left_count = values.iter().filter(|v| v.as_str() == "L").count();
        let right_count = values.iter().filter(|v| v.as_str() == "R").count();
        assert!(
            right_count > left_count,
            "right associativity should bias ties toward later branches"
        );
    }

    #[test]
    fn explicit_probabilities_must_sum_to_100() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    ASTNode::Sequence {
                        elements: vec![token("probability", "60"), token("quoted_string", "X")],
                    },
                    ASTNode::Sequence {
                        elements: vec![token("probability", "30"), token("quoted_string", "Y")],
                    },
                ],
            },
        );
        let rule_order = vec!["start".to_string()];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 11);
        let err = generator
            .generate_many(1, None)
            .expect_err("invalid explicit probability sum should fail");
        let err_msg = format!("{}", err);
        assert!(
            err_msg.contains("sum to 100"),
            "unexpected error for invalid probabilities: {}",
            err_msg
        );
    }

    #[test]
    fn recursion_guard_prefers_terminating_branch_at_depth_limit() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("rule_reference", "start"),
                    token("quoted_string", "x"),
                ],
            },
        );
        let rule_order = vec!["start".to_string()];

        let mut generator = StimuliGenerator::new(
            "recursive".to_string(),
            &grammar_tree,
            &rule_order,
            None,
            StimuliConfig {
                seed: Some(1),
                max_depth: 2,
                max_repeat: 2,
                max_rule_visits: 2,
                recovery_mode: RecoveryStimuliMode::Baseline,
                enforce_word_boundary_spacing: false,
                trace_verbosity: TraceVerbosity::None,
            },
        );

        let value = generator
            .generate_many(1, None)
            .expect("depth-limited generation should terminate");
        assert_eq!(value[0], "x");
    }

    #[test]
    fn regex_negated_class_avoids_control_character_samples() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("regex", "[^\"\\\\]+"));
        let rule_order = vec!["start".to_string()];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 99);
        let value = generator
            .generate_many(1, None)
            .expect("negated-class regex generation should succeed");

        assert!(!value[0].is_empty(), "regex sample should not be empty");
        assert!(
            !value[0].chars().any(|c| c == '\0'),
            "regex sample should not include NUL characters: {:?}",
            value[0]
        );
        assert!(
            value[0].chars().all(|c| !c.is_ascii_control()),
            "regex sample should avoid ASCII control characters: {:?}",
            value[0]
        );
    }

    #[test]
    fn regex_whitespace_class_prefers_space() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("regex", "\\s+"));
        let rule_order = vec!["start".to_string()];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 101);
        let value = generator
            .generate_many(1, None)
            .expect("whitespace regex generation should succeed");

        assert!(
            !value[0].is_empty(),
            "whitespace sample should not be empty"
        );
        assert!(
            value[0].chars().all(|c| c == ' '),
            "whitespace sample should prefer printable spaces over control chars: {:?}",
            value[0]
        );
    }

    #[test]
    fn regex_anchor_pattern_generates_full_match() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("regex", "^\\d{2}$"));
        let rule_order = vec!["start".to_string()];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 2026);
        let value = generator
            .generate_many(1, None)
            .expect("anchored regex generation should succeed");

        let re = Regex::new(r"^\d{2}$").expect("valid regex");
        assert!(
            re.is_match(&value[0]),
            "generated sample must satisfy anchored regex: {:?}",
            value[0]
        );
        assert_eq!(value[0].chars().count(), 2);
    }

    #[test]
    fn regex_word_boundary_pattern_generates_matchable_sample() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("regex", "\\bword\\b"));
        let rule_order = vec!["start".to_string()];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 2027);
        let value = generator
            .generate_many(1, None)
            .expect("word-boundary regex generation should succeed");

        assert!(
            StimuliGenerator::regex_matches_entire(r"\bword\b", &value[0]),
            "generated sample must fully satisfy word-boundary regex: {:?}",
            value[0]
        );
    }

    #[test]
    fn word_boundary_spacing_policy_appends_separator_for_terminal_boundary() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("regex", "input\\b"));
        let rule_order = vec!["start".to_string()];

        let mut generator = StimuliGenerator::new(
            "word_boundary_spacing".to_string(),
            &grammar_tree,
            &rule_order,
            None,
            StimuliConfig {
                seed: Some(2028),
                max_depth: 4,
                max_repeat: 2,
                max_rule_visits: 4,
                recovery_mode: RecoveryStimuliMode::Baseline,
                enforce_word_boundary_spacing: true,
                trace_verbosity: TraceVerbosity::None,
            },
        );

        let value = generator
            .generate_many(1, None)
            .expect("word-boundary spacing generation should succeed");
        assert!(
            StimuliGenerator::regex_matches_entire(r"input\b", value[0].trim_end()),
            "trimmed word-boundary sample must satisfy regex contract: {:?}",
            value[0]
        );
        assert!(
            value[0].starts_with("input"),
            "word-boundary sample should preserve regex literal prefix: {:?}",
            value[0]
        );
        assert!(
            value[0].ends_with(' '),
            "word-boundary spacing should append delimiter space: {:?}",
            value[0]
        );
    }

    #[test]
    fn word_spacing_policy_separates_adjacent_word_segments_in_sequences() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    token("regex", "module"),
                    token("regex", "automatic"),
                    token("regex", "[A-Za-z]+"),
                ],
            },
        );
        let rule_order = vec!["start".to_string()];

        let mut generator = StimuliGenerator::new(
            "word_spacing_sequence".to_string(),
            &grammar_tree,
            &rule_order,
            None,
            StimuliConfig {
                seed: Some(2029),
                max_depth: 4,
                max_repeat: 2,
                max_rule_visits: 4,
                recovery_mode: RecoveryStimuliMode::Baseline,
                enforce_word_boundary_spacing: true,
                trace_verbosity: TraceVerbosity::None,
            },
        );

        let value = generator
            .generate_many(1, None)
            .expect("word-segment spacing generation should succeed");
        assert!(
            value[0].starts_with("module automatic "),
            "word spacing should separate adjacent lexical segments: {:?}",
            value[0]
        );
        let parts: Vec<&str> = value[0].split_whitespace().collect();
        assert!(
            parts.len() >= 3,
            "word spacing sequence should keep lexical segments distinct: {:?}",
            value[0]
        );
        assert!(
            StimuliGenerator::regex_matches_entire("module", parts[0]),
            "first segment must satisfy originating regex: {:?}",
            value[0]
        );
        assert!(
            StimuliGenerator::regex_matches_entire("automatic", parts[1]),
            "second segment must satisfy originating regex: {:?}",
            value[0]
        );
        assert!(
            StimuliGenerator::regex_matches_entire("[A-Za-z]+", parts[2]),
            "third segment must satisfy originating regex: {:?}",
            value[0]
        );
    }

    #[test]
    fn regex_escape_classes_generate_printable_match() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("regex", "^\\d\\w\\s\\D\\W\\S$"));
        let rule_order = vec!["start".to_string()];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 2028);
        let value = generator
            .generate_many(1, None)
            .expect("escape-class regex generation should succeed");

        let re = Regex::new(r"^\d\w\s\D\W\S$").expect("valid regex");
        assert!(
            re.is_match(&value[0]),
            "generated sample must satisfy escape-class regex: {:?}",
            value[0]
        );
        assert!(
            value[0].chars().all(|c| !c.is_ascii_control()),
            "generated escape-class sample should avoid control chars: {:?}",
            value[0]
        );
    }

    #[test]
    fn regex_bounded_repetition_respects_length_bounds() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("regex", "^[A-Z]{2,4}$"));
        let rule_order = vec!["start".to_string()];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 2029);
        let value = generator
            .generate_many(1, None)
            .expect("bounded-repetition regex generation should succeed");

        let re = Regex::new(r"^[A-Z]{2,4}$").expect("valid regex");
        assert!(
            re.is_match(&value[0]),
            "generated sample must satisfy bounded repetition regex: {:?}",
            value[0]
        );
        let len = value[0].chars().count();
        assert!(
            (2..=4).contains(&len),
            "generated sample length should be within [2,4], got {} from {:?}",
            len,
            value[0]
        );
    }

    #[test]
    fn or_generation_retries_alternatives_after_selected_branch_error() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    ASTNode::Sequence {
                        elements: vec![
                            token("probability", "100"),
                            token("rule_reference", "missing_rule"),
                        ],
                    },
                    ASTNode::Sequence {
                        elements: vec![token("probability", "0"), token("quoted_string", "ok")],
                    },
                ],
            },
        );
        let rule_order = vec!["start".to_string()];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 3030);
        let value = generator
            .generate_many(1, None)
            .expect("generator should retry alternate OR branch and succeed");

        assert_eq!(value[0], "ok");
    }

    #[test]
    fn coverage_metrics_track_rule_and_branch_hits() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![token("quoted_string", "L"), token("quoted_string", "R")],
            },
        );
        let rule_order = vec!["start".to_string()];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 4040);
        let values = generator
            .generate_many(64, None)
            .expect("coverage tracking generation should succeed");
        assert_eq!(values.len(), 64);

        let coverage = generator.coverage_metrics();
        assert_eq!(coverage.sample_attempts, 64);
        assert_eq!(coverage.sample_successes, 64);
        assert_eq!(coverage.sample_errors, 0);
        assert_eq!(coverage.total_rules, 1);
        assert_eq!(coverage.total_branches, 2);
        assert_eq!(coverage.covered_rules(), 1);
        assert!(coverage.covered_branches() >= 1);
    }

    #[test]
    fn coverage_metrics_merge_accumulates_counts() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![token("quoted_string", "L"), token("quoted_string", "R")],
            },
        );
        let rule_order = vec!["start".to_string()];

        let mut generator_a = simple_generator(&grammar_tree, &rule_order, 5050);
        let mut generator_b = simple_generator(&grammar_tree, &rule_order, 6060);

        let _ = generator_a
            .generate_many(30, None)
            .expect("first coverage run should succeed");
        let _ = generator_b
            .generate_many(40, None)
            .expect("second coverage run should succeed");

        let mut merged = generator_a.coverage_metrics().clone();
        merged
            .merge_from(generator_b.coverage_metrics())
            .expect("coverage merge should succeed");

        assert_eq!(merged.sample_attempts, 70);
        assert_eq!(merged.sample_successes, 70);
        assert_eq!(merged.sample_errors, 0);
        assert_eq!(merged.total_rules, 1);
        assert_eq!(merged.total_branches, 2);
    }

    #[test]
    fn gap_report_separates_reachable_and_unreachable_debt() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("rule_reference", "reachable"));
        grammar_tree.insert("reachable".to_string(), token("quoted_string", "R"));
        grammar_tree.insert(
            "unreachable".to_string(),
            ASTNode::Or {
                alternatives: vec![token("quoted_string", "U1"), token("quoted_string", "U2")],
            },
        );
        let rule_order = vec![
            "start".to_string(),
            "reachable".to_string(),
            "unreachable".to_string(),
        ];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 777);
        let _ = generator
            .generate_many(8, Some("start"))
            .expect("reachable-only generation should succeed");

        let report = generator
            .generate_gap_report(Some("start"), 1)
            .expect("gap report generation should succeed");

        assert!(
            report
                .unreachable_rule_debt
                .iter()
                .any(|debt| debt.rule_name == "unreachable")
        );
        assert!(
            report
                .unreachable_branch_debt
                .iter()
                .any(|debt| debt.rule_name == "unreachable")
        );
        assert!(
            report
                .targets
                .iter()
                .all(|target| target.rule_name != "unreachable")
        );
    }

    #[test]
    fn target_driven_generation_resolves_branch_targets() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![token("quoted_string", "L"), token("quoted_string", "R")],
            },
        );
        let rule_order = vec!["start".to_string()];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 888);
        let report = generator
            .generate_gap_report(Some("start"), 1)
            .expect("gap report generation should succeed");
        assert!(!report.targets.is_empty(), "expected actionable targets");

        let (_samples, summary) = generator
            .generate_until_targets(Some("start"), &report.targets, 200)
            .expect("target-driven generation should succeed");

        assert_eq!(
            summary.resolved_targets, summary.total_targets,
            "all reachable targets should resolve within attempt budget"
        );
        assert!(
            summary.unresolved_targets.is_empty(),
            "no unresolved targets expected"
        );
    }

    #[test]
    fn target_driven_generation_filter_rejects_branch_without_paying_target_debt() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![token("quoted_string", "L"), token("quoted_string", "R")],
            },
        );
        let rule_order = vec!["start".to_string()];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 889);
        let report = generator
            .generate_gap_report(Some("start"), 1)
            .expect("gap report generation should succeed");
        assert!(!report.targets.is_empty(), "expected actionable targets");

        let (samples, summary, validation) = generator
            .generate_until_targets_with_filter(Some("start"), &report.targets, 200, |sample| {
                Ok(sample == "L")
            })
            .expect("target-driven generation with filter should succeed");

        assert_eq!(validation.accepted_outputs, samples.len());
        assert!(samples.iter().all(|sample| sample == "L"));
        assert!(
            summary.unresolved_targets.iter().any(|status| matches!(
                status.target_type,
                StimuliCoverageTargetType::Branch
            ) && status.branch_index == Some(1)),
            "rejected branch must remain unresolved"
        );

        let group = generator
            .coverage_metrics()
            .branch_groups
            .get("start::root")
            .expect("branch group should exist");
        assert_eq!(group.success_counts.get(1).copied().unwrap_or(0), 0);
        assert!(
            group.selected_counts.get(1).copied().unwrap_or(0) > 0,
            "rejected branch should still accumulate selection history for throttling"
        );
    }

    #[test]
    fn target_branch_failure_throttle_penalizes_persistently_low_success_ratio() {
        assert_eq!(StimuliGenerator::target_branch_failure_throttle(4, 0), 1);
        assert!(
            StimuliGenerator::target_branch_failure_throttle(64, 0)
                > StimuliGenerator::target_branch_failure_throttle(64, 16),
            "zero-success branches should be throttled harder than branches with real success history"
        );
        assert!(
            StimuliGenerator::target_branch_failure_throttle(64, 1)
                > StimuliGenerator::target_branch_failure_throttle(64, 16),
            "very low success ratios should be throttled harder than healthier branches"
        );
        assert_eq!(StimuliGenerator::target_branch_failure_throttle(64, 40), 1);
    }

    #[test]
    fn coverage_guidance_multiplier_deemphasizes_low_yield_target_branch() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![token("quoted_string", "L"), token("quoted_string", "R")],
            },
        );
        let rule_order = vec!["start".to_string()];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 890);
        let report = generator
            .generate_gap_report(Some("start"), 1)
            .expect("gap report generation should succeed");
        let applied = generator.apply_targets(&report.targets);
        assert!(
            applied >= 2,
            "expected branch targets to apply alongside any entry-rule debt"
        );

        let group = generator
            .coverage
            .branch_groups
            .get_mut("start::root")
            .expect("branch group should exist");
        group.selected_counts = vec![64, 64];
        group.success_counts = vec![1, 16];

        let thresholds = generator
            .target_plan
            .branch_thresholds
            .get_mut("start::root")
            .expect("branch target plan should exist");
        thresholds.insert(0, 2);
        thresholds.insert(1, 17);

        let alternatives = match grammar_tree.get("start").expect("rule should exist") {
            ASTNode::Or { alternatives } => alternatives,
            other => panic!("expected OR node, got {:?}", other),
        };

        let low_yield_multiplier =
            generator.coverage_guidance_multiplier("start", "root", 0, &alternatives[0]);
        let healthy_multiplier =
            generator.coverage_guidance_multiplier("start", "root", 1, &alternatives[1]);

        assert!(
            low_yield_multiplier < healthy_multiplier,
            "low-yield target branch should be deemphasized once selection history shows poor parseability (low_yield={}, healthy={})",
            low_yield_multiplier,
            healthy_multiplier
        );
    }

    #[test]
    fn target_probe_threshold_escalates_under_low_yield_branch_pressure() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![token("quoted_string", "L"), token("quoted_string", "R")],
            },
        );
        grammar_tree.insert("helper".to_string(), token("quoted_string", "H"));
        let rule_order = vec!["start".to_string(), "helper".to_string()];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 891);
        generator
            .target_plan
            .rule_thresholds
            .insert("helper".to_string(), 1);
        let group = generator
            .coverage
            .branch_groups
            .get_mut("start::root")
            .expect("branch group should exist");
        group.selected_counts = vec![0, 64];
        group.success_counts = vec![0, 1];

        let pending = vec![TargetCoverageStatus {
            id: "branch::start::root::1".to_string(),
            target_type: StimuliCoverageTargetType::Branch,
            rule_name: "start".to_string(),
            node_path: Some("root".to_string()),
            branch_index: Some(1),
            current_successes: 1,
            required_successes: 2,
            remaining_successes: 1,
            priority_score: 100,
            reason: "selected_but_failed".to_string(),
            depends_on: vec!["helper".to_string()],
        }];
        assert_eq!(generator.target_probe_threshold(&pending), 8);
    }

    #[test]
    fn target_probe_threshold_for_validation_backs_off_under_alternate_churn() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![token("quoted_string", "L"), token("quoted_string", "R")],
            },
        );
        grammar_tree.insert("helper".to_string(), token("quoted_string", "H"));
        let rule_order = vec!["start".to_string(), "helper".to_string()];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 895);
        generator
            .target_plan
            .rule_thresholds
            .insert("helper".to_string(), 1);
        let group = generator
            .coverage
            .branch_groups
            .get_mut("start::root")
            .expect("branch group should exist");
        group.selected_counts = vec![0, 64];
        group.success_counts = vec![0, 1];

        let pending = vec![TargetCoverageStatus {
            id: "branch::start::root::1".to_string(),
            target_type: StimuliCoverageTargetType::Branch,
            rule_name: "start".to_string(),
            node_path: Some("root".to_string()),
            branch_index: Some(1),
            current_successes: 1,
            required_successes: 2,
            remaining_successes: 1,
            priority_score: 100,
            reason: "selected_but_failed".to_string(),
            depends_on: vec!["helper".to_string()],
        }];
        let validation = TargetDriveValidationSummary {
            validated_outputs: 4,
            accepted_outputs: 1,
            rejected_outputs: 3,
            alternate_entry_attempts: 16,
            alternate_entry_accepted_outputs: 1,
            alternate_entry_rejected_outputs: 15,
        };

        assert_eq!(generator.target_probe_threshold(&pending), 8);
        assert_eq!(
            generator.target_probe_threshold_for_validation(&pending, &validation),
            24,
            "validation-aware replay should back off helper probing when low-yield alternate attempts dominate and primary entry rejects are present"
        );
    }

    #[test]
    fn target_probe_prefers_unresolved_dependency_rule_over_legacy_branch_fallback() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("rule_reference", "helper"),
                    token("quoted_string", "X"),
                ],
            },
        );
        grammar_tree.insert("helper".to_string(), token("quoted_string", "H"));
        let rule_order = vec!["start".to_string(), "helper".to_string()];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 892);
        let report = generator
            .generate_gap_report(Some("start"), 1)
            .expect("gap report generation should succeed");
        let _ = generator.apply_targets(&report.targets);

        let pending = vec![TargetCoverageStatus {
            id: "branch::start::root::1".to_string(),
            target_type: StimuliCoverageTargetType::Branch,
            rule_name: "start".to_string(),
            node_path: Some("root".to_string()),
            branch_index: Some(1),
            current_successes: 0,
            required_successes: 1,
            remaining_successes: 1,
            priority_score: 100,
            reason: "selected_but_failed".to_string(),
            depends_on: vec!["helper".to_string()],
        }];

        assert_eq!(
            generator.select_target_probe_rule_legacy(&pending, "start"),
            None,
            "legacy probing never escaped the entry rule in this case"
        );
        assert_eq!(
            generator.select_target_probe_rule(&pending, "start"),
            Some("helper".to_string()),
            "new probing should prioritize unresolved dependency rules"
        );
    }

    #[test]
    fn target_probe_validation_returns_to_primary_entry_under_low_yield_alternate_churn() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![token("quoted_string", "S"), token("quoted_string", "T")],
            },
        );
        grammar_tree.insert(
            "helper".to_string(),
            ASTNode::Or {
                alternatives: vec![token("quoted_string", "H"), token("quoted_string", "I")],
            },
        );
        let rule_order = vec!["start".to_string(), "helper".to_string()];

        let generator = simple_generator(&grammar_tree, &rule_order, 893);
        let pending = vec![TargetCoverageStatus {
            id: "branch::helper::root::1".to_string(),
            target_type: StimuliCoverageTargetType::Branch,
            rule_name: "helper".to_string(),
            node_path: Some("root".to_string()),
            branch_index: Some(1),
            current_successes: 0,
            required_successes: 1,
            remaining_successes: 1,
            priority_score: 100,
            reason: "selected_but_failed".to_string(),
            depends_on: Vec::new(),
        }];
        let validation = TargetDriveValidationSummary {
            validated_outputs: 4,
            accepted_outputs: 4,
            rejected_outputs: 0,
            alternate_entry_attempts: 16,
            alternate_entry_accepted_outputs: 1,
            alternate_entry_rejected_outputs: 15,
        };

        assert_eq!(
            generator.select_target_probe_rule(&pending, "start"),
            Some("helper".to_string()),
            "base probing still falls back to non-entry rules without validation context"
        );
        assert_eq!(
            generator.select_target_probe_rule_for_validation(&pending, "start", &validation),
            None,
            "validation-aware probing should return to primary entry when low-yield alternate attempts dominate"
        );
    }

    #[test]
    fn target_probe_validation_keeps_dependency_probe_under_alternate_churn() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("rule_reference", "helper"),
                    token("quoted_string", "S"),
                ],
            },
        );
        grammar_tree.insert(
            "helper".to_string(),
            ASTNode::Or {
                alternatives: vec![token("quoted_string", "H"), token("quoted_string", "I")],
            },
        );
        let rule_order = vec!["start".to_string(), "helper".to_string()];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 894);
        generator
            .target_plan
            .rule_thresholds
            .insert("helper".to_string(), 1);
        let pending = vec![TargetCoverageStatus {
            id: "branch::start::root::1".to_string(),
            target_type: StimuliCoverageTargetType::Branch,
            rule_name: "start".to_string(),
            node_path: Some("root".to_string()),
            branch_index: Some(1),
            current_successes: 0,
            required_successes: 1,
            remaining_successes: 1,
            priority_score: 100,
            reason: "selected_but_failed".to_string(),
            depends_on: vec!["helper".to_string()],
        }];
        let validation = TargetDriveValidationSummary {
            validated_outputs: 4,
            accepted_outputs: 4,
            rejected_outputs: 0,
            alternate_entry_attempts: 16,
            alternate_entry_accepted_outputs: 1,
            alternate_entry_rejected_outputs: 15,
        };

        assert_eq!(
            generator.select_target_probe_rule_for_validation(&pending, "start", &validation),
            Some("helper".to_string()),
            "validation-aware probing must preserve explicit dependency probes even when alternate churn is high"
        );
    }

    #[test]
    fn target_probe_prefers_more_impactful_dependency_candidate() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("rule_reference", "helper_a"),
                    token("rule_reference", "helper_b"),
                    token("quoted_string", "S"),
                ],
            },
        );
        grammar_tree.insert("helper_a".to_string(), token("quoted_string", "A"));
        grammar_tree.insert("helper_b".to_string(), token("quoted_string", "B"));
        let rule_order = vec![
            "start".to_string(),
            "helper_a".to_string(),
            "helper_b".to_string(),
        ];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 896);
        generator
            .target_plan
            .rule_thresholds
            .insert("helper_a".to_string(), 1);
        generator
            .target_plan
            .rule_thresholds
            .insert("helper_b".to_string(), 3);

        let pending = vec![
            TargetCoverageStatus {
                id: "branch::start::root::0".to_string(),
                target_type: StimuliCoverageTargetType::Branch,
                rule_name: "start".to_string(),
                node_path: Some("root".to_string()),
                branch_index: Some(0),
                current_successes: 0,
                required_successes: 1,
                remaining_successes: 1,
                priority_score: 100,
                reason: "selected_but_failed".to_string(),
                depends_on: vec!["helper_a".to_string()],
            },
            TargetCoverageStatus {
                id: "branch::start::root::1".to_string(),
                target_type: StimuliCoverageTargetType::Branch,
                rule_name: "start".to_string(),
                node_path: Some("root".to_string()),
                branch_index: Some(1),
                current_successes: 0,
                required_successes: 2,
                remaining_successes: 2,
                priority_score: 90,
                reason: "selected_but_failed".to_string(),
                depends_on: vec!["helper_b".to_string()],
            },
        ];

        assert_eq!(
            generator.select_target_probe_rule(&pending, "start"),
            Some("helper_b".to_string()),
            "dependency probing should prioritize the candidate with larger unresolved leverage, not just the first listed dependency"
        );
    }

    #[test]
    fn target_probe_validation_suppresses_marginal_dependency_under_alternate_churn() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("rule_reference", "helper"),
                    token("quoted_string", "S"),
                ],
            },
        );
        grammar_tree.insert(
            "helper".to_string(),
            ASTNode::Or {
                alternatives: vec![token("quoted_string", "H"), token("quoted_string", "I")],
            },
        );
        let rule_order = vec!["start".to_string(), "helper".to_string()];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 897);
        generator
            .target_plan
            .rule_thresholds
            .insert("helper".to_string(), 2);
        generator
            .coverage
            .rule_success_hits
            .insert("helper".to_string(), 1);
        let pending = vec![TargetCoverageStatus {
            id: "branch::start::root::1".to_string(),
            target_type: StimuliCoverageTargetType::Branch,
            rule_name: "start".to_string(),
            node_path: Some("root".to_string()),
            branch_index: Some(1),
            current_successes: 0,
            required_successes: 1,
            remaining_successes: 1,
            priority_score: 100,
            reason: "selected_but_failed".to_string(),
            depends_on: vec!["helper".to_string()],
        }];
        let validation = TargetDriveValidationSummary {
            validated_outputs: 4,
            accepted_outputs: 1,
            rejected_outputs: 3,
            alternate_entry_attempts: 16,
            alternate_entry_accepted_outputs: 1,
            alternate_entry_rejected_outputs: 15,
        };

        assert_eq!(
            generator.select_target_probe_rule(&pending, "start"),
            Some("helper".to_string()),
            "base probing should still recognize the unresolved dependency candidate"
        );
        assert_eq!(
            generator.select_target_probe_rule_for_validation(&pending, "start", &validation),
            None,
            "validation-aware probing should suppress marginal dependency probes once alternate churn dominates and the dependency is no longer zero-hit or multi-target critical"
        );
    }

    #[test]
    fn semantic_usage_stimuli_coverage_target_biases_targeted_rule_branches() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("rule_reference", "left"),
                    token("rule_reference", "right"),
                ],
            },
        );
        grammar_tree.insert("left".to_string(), token("quoted_string", "L"));
        grammar_tree.insert("right".to_string(), token("quoted_string", "R"));
        let rule_order = vec!["start".to_string(), "left".to_string(), "right".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "left".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "coverage_target".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "6".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "critical_path".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "true".to_string(),
                    },
                },
            ],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9411);
        let values = generator
            .generate_many(128, Some("start"))
            .expect("coverage-target-guided generation should succeed");

        let left_count = values.iter().filter(|value| value.as_str() == "L").count();
        let right_count = values.iter().filter(|value| value.as_str() == "R").count();
        assert!(
            left_count > right_count.saturating_mul(2),
            "coverage_target/critical_path hints should bias branch sampling toward targeted rule, got left={} right={} values={:?}",
            left_count,
            right_count,
            values
        );
    }

    #[test]
    fn semantic_usage_stimuli_coverage_target_boosts_gap_report_branch_priority() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("rule_reference", "left"),
                    token("rule_reference", "right"),
                ],
            },
        );
        grammar_tree.insert("left".to_string(), token("quoted_string", "L"));
        grammar_tree.insert("right".to_string(), token("quoted_string", "R"));
        let rule_order = vec!["start".to_string(), "left".to_string(), "right".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "left".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "coverage_target".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "3".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "critical_path".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "true".to_string(),
                    },
                },
            ],
        );

        let generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9412);
        let report = generator
            .generate_gap_report(Some("start"), 1)
            .expect("coverage gap report generation should succeed");
        let left_branch = report
            .reachable_branch_debt
            .iter()
            .find(|debt| debt.branch_id == "branch::start::root#0")
            .expect("left branch debt should be present");
        let right_branch = report
            .reachable_branch_debt
            .iter()
            .find(|debt| debt.branch_id == "branch::start::root#1")
            .expect("right branch debt should be present");

        assert!(
            left_branch.priority_score > right_branch.priority_score,
            "coverage_target/critical_path hints should raise targeted branch priority in gap report, got left={} right={}",
            left_branch.priority_score,
            right_branch.priority_score
        );
    }

    #[test]
    fn semantic_usage_stimuli_transformexpr_hint_overrides_regex_sampling() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("regex", "^[A-Z]{6}$"));
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![
                UnifiedSemanticAST::TransformExpr {
                    expression: "str::parse::<i64>().unwrap_or(0)".to_string(),
                }
                .into(),
            ],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9090);
        let values = generator
            .generate_many(1, Some("start"))
            .expect("semantic-driven generation should succeed");
        assert_eq!(
            values[0], "1",
            "semantic parse::<i*> hint should override regex sampling"
        );
    }

    #[test]
    fn semantic_usage_stimuli_transformexpr_hints_cover_float_and_bool() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "float_rule".to_string(),
            token("regex", "^[0-9]+(\\.[0-9]+)?$"),
        );
        grammar_tree.insert("bool_rule".to_string(), token("regex", "^(true|false)$"));
        let rule_order = vec!["float_rule".to_string(), "bool_rule".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "float_rule".to_string(),
            vec![
                UnifiedSemanticAST::TransformExpr {
                    expression: "str::parse::<f64>().unwrap_or(0.0)".to_string(),
                }
                .into(),
            ],
        );
        annotations.semantic_annotations.insert(
            "bool_rule".to_string(),
            vec![
                UnifiedSemanticAST::TransformExpr {
                    expression: "str::parse::<bool>().unwrap_or(false)".to_string(),
                }
                .into(),
            ],
        );

        let mut float_generator =
            annotated_generator(&grammar_tree, &rule_order, &annotations, 9091);
        let float_values = float_generator
            .generate_many(1, Some("float_rule"))
            .expect("float semantic-driven generation should succeed");
        assert_eq!(float_values[0], "1.0");

        let mut bool_generator =
            annotated_generator(&grammar_tree, &rule_order, &annotations, 9092);
        let bool_values = bool_generator
            .generate_many(1, Some("bool_rule"))
            .expect("bool semantic-driven generation should succeed");
        assert_eq!(bool_values[0], "true");
    }

    #[test]
    fn semantic_usage_stimuli_transformexpr_supports_path_target_type() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("regex", "^[A-Z]{5}$"));
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![
                UnifiedSemanticAST::TransformExpr {
                    expression: "str::parse::<std::primitive::u32>().unwrap_or(0)".to_string(),
                }
                .into(),
            ],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9094);
        let values = generator
            .generate_many(1, Some("start"))
            .expect("path-type semantic-driven generation should succeed");
        assert_eq!(values[0], "1");
    }

    #[test]
    fn semantic_usage_stimuli_noncanonical_transform_does_not_override_regex() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("regex", "^[A-Z]{6}$"));
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![
                UnifiedSemanticAST::TransformExpr {
                    expression: "str::parse::<i64>().unwrap_or_default()".to_string(),
                }
                .into(),
            ],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9095);
        let values = generator
            .generate_many(1, Some("start"))
            .expect("non-canonical semantic transform should still allow generation");

        let sample = &values[0];
        let regex = Regex::new(r"^[A-Z]{6}$").expect("valid regex");
        assert!(
            regex.is_match(sample),
            "non-canonical transform should not override regex sampling, got {:?}",
            sample
        );
    }

    #[test]
    fn semantic_usage_stimuli_raw_quoted_content_returns_literal_hint() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("regex", "^[A-Z]{4}$"));
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![
                UnifiedSemanticAST::Raw {
                    content: "\"literal-token\"".to_string(),
                }
                .into(),
            ],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9093);
        let values = generator
            .generate_many(1, Some("start"))
            .expect("raw-semantic-driven generation should succeed");
        assert_eq!(values[0], "literal-token");
    }

    #[test]
    fn semantic_usage_stimuli_raw_hint_requires_literalish_directive_when_named() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("regex", "^[A-Z]{4}$"));
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![SemanticAnnotation::Named {
                name: "type".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: "\"literal-token\"".to_string(),
                },
            }],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9193);
        let values = generator
            .generate_many(1, Some("start"))
            .expect("generation should succeed");

        let sample = &values[0];
        let regex = Regex::new(r"^[A-Z]{4}$").expect("valid regex");
        assert!(
            regex.is_match(sample),
            "non-literal directive should not override regex sampling, got {:?}",
            sample
        );
    }

    #[test]
    fn semantic_usage_stimuli_token_class_overrides_regex_sampling_pattern() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("regex", "^[0-9]{3}$"));
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![SemanticAnnotation::Named {
                name: "token_class".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: "identifier".to_string(),
                },
            }],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9205);
        let values = generator
            .generate_many(20, Some("start"))
            .expect("token_class steering generation should succeed");
        let ident_re = Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*$").expect("valid regex");
        assert!(
            values.iter().all(|value| ident_re.is_match(value)),
            "token_class steering should enforce identifier family samples, got {:?}",
            values
        );
    }

    #[test]
    fn semantic_usage_stimuli_charset_overrides_token_class_pattern() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("regex", "^[a-z]+$"));
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "token_class".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "identifier".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "charset".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[A-F0-9]".to_string(),
                    },
                },
            ],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9206);
        let values = generator
            .generate_many(20, Some("start"))
            .expect("charset steering generation should succeed");
        let charset_re = Regex::new(r"^[A-F0-9]+$").expect("valid regex");
        assert!(
            values.iter().all(|value| charset_re.is_match(value)),
            "charset steering should override token_class pattern, got {:?}",
            values
        );
    }

    #[test]
    fn semantic_usage_stimuli_pattern_overrides_charset_and_token_class() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("regex", "^[a-z]+$"));
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "token_class".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "identifier".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "charset".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[A-F0-9]".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "pattern".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "^[Q]{2}$".to_string(),
                    },
                },
            ],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9207);
        let values = generator
            .generate_many(20, Some("start"))
            .expect("pattern steering generation should succeed");
        let pattern_re = Regex::new(r"^[Q]{2}$").expect("valid regex");
        assert!(
            values.iter().all(|value| pattern_re.is_match(value)),
            "pattern steering should have highest precedence over charset/token_class, got {:?}",
            values
        );
    }

    #[test]
    fn semantic_usage_stimuli_enum_constraints_filter_regex_sampling() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("regex", "^[A-Z]{2}$"));
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![SemanticAnnotation::Named {
                name: "enum".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: "[\"AA\", \"BB\"]".to_string(),
                },
            }],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9201);
        let values = generator
            .generate_many(24, Some("start"))
            .expect("enum-constrained generation should succeed");
        assert!(
            values.iter().all(|value| value == "AA" || value == "BB"),
            "enum-constrained values must stay inside allowed set, got {:?}",
            values
        );
    }

    #[test]
    fn semantic_usage_stimuli_range_constraints_generate_in_domain_values() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("regex", "^[0-9]{2}$"));
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![SemanticAnnotation::Named {
                name: "range".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: "10..12".to_string(),
                },
            }],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9202);
        let values = generator
            .generate_many(24, Some("start"))
            .expect("range-constrained generation should succeed");

        for value in values {
            let parsed = value
                .parse::<i64>()
                .expect("range-constrained output must be numeric");
            assert!(
                (10..=12).contains(&parsed),
                "numeric sample must satisfy @range bounds, got {}",
                parsed
            );
        }
    }

    #[test]
    fn semantic_usage_stimuli_len_constraints_generate_matching_lengths() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("regex", "^[a-z]+$"));
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![SemanticAnnotation::Named {
                name: "len".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: "[3, 3]".to_string(),
                },
            }],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9203);
        let values = generator
            .generate_many(20, Some("start"))
            .expect("len-constrained generation should succeed");

        for value in values {
            assert_eq!(
                value.chars().count(),
                3,
                "len-constrained sample must have exact configured length"
            );
            assert!(
                value.chars().all(|ch| ch.is_ascii_lowercase()),
                "len-constrained sample should still satisfy base regex, got {:?}",
                value
            );
        }
    }

    #[test]
    fn semantic_usage_stimuli_regex_and_enum_constraints_compose() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("regex", "^[A-Z]{2}$"));
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "regex".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "^A[A-Z]$".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "enum".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[\"AA\", \"AB\", \"BC\"]".to_string(),
                    },
                },
            ],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9204);
        let values = generator
            .generate_many(24, Some("start"))
            .expect("composed regex+enum constraints should succeed");
        assert!(
            values.iter().all(|value| value == "AA" || value == "AB"),
            "composed constraints should filter enum candidates to regex-valid subset, got {:?}",
            values
        );
    }

    #[test]
    fn semantic_usage_stimuli_recovery_fallback_prefers_panic_until_marker() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("rule_reference", "missing_left"),
                    token("rule_reference", "missing_right"),
                ],
            },
        );
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "recover".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "true".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "sync".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[\";\", \"end\"]".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "panic_until".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[\"}\"]".to_string(),
                    },
                },
            ],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9301);
        let values = generator
            .generate_many(4, Some("start"))
            .expect("recovery fallback should provide stimuli when OR branches fail");

        assert!(
            values.iter().all(|value| value == "}"),
            "recovery fallback should prefer panic_until marker token, got {:?}",
            values
        );
    }

    #[test]
    fn semantic_usage_stimuli_recovery_fallback_requires_recover_enabled() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("rule_reference", "missing_left"),
                    token("rule_reference", "missing_right"),
                ],
            },
        );
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![SemanticAnnotation::Named {
                name: "sync".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: "[\";\"]".to_string(),
                },
            }],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9302);
        let result = generator.generate_many(1, Some("start"));
        assert!(
            result.is_err(),
            "recovery fallback must stay inactive when @recover is not enabled"
        );
    }

    #[test]
    fn semantic_usage_stimuli_recovery_biased_mode_wraps_output_with_recovery_markers() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("quoted_string", "ok"));
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "recover".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "true".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "panic_until".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[\"}\"]".to_string(),
                    },
                },
            ],
        );

        let mut generator = annotated_generator_with_mode(
            &grammar_tree,
            &rule_order,
            &annotations,
            9311,
            RecoveryStimuliMode::RecoveryBiased,
        );
        let values = generator
            .generate_many(6, Some("start"))
            .expect("recovery-biased mode should generate wrapped samples");

        assert!(
            values.iter().all(|value| value.contains('}')),
            "recovery-biased mode should inject recovery markers, got {:?}",
            values
        );
        assert!(
            values.iter().all(|value| value.contains("ok")),
            "recovery-biased mode should preserve base sample content, got {:?}",
            values
        );
    }

    #[test]
    fn semantic_usage_stimuli_near_sync_negative_mode_emits_noise_plus_marker() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("quoted_string", "ok"));
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "recover".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "true".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "sync".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[\";\"]".to_string(),
                    },
                },
            ],
        );

        let mut generator = annotated_generator_with_mode(
            &grammar_tree,
            &rule_order,
            &annotations,
            9312,
            RecoveryStimuliMode::NearSyncNegative,
        );
        let values = generator
            .generate_many(4, Some("start"))
            .expect("near-sync-negative mode should generate marker-adjacent negative samples");

        assert!(
            values
                .iter()
                .all(|value| value.contains("__pgen_near_sync_start__")),
            "near-sync-negative mode should inject deterministic near-sync noise, got {:?}",
            values
        );
        assert!(
            values.iter().all(|value| value.ends_with(';')),
            "near-sync-negative mode should terminate with sync marker token, got {:?}",
            values
        );
    }

    #[test]
    fn semantic_usage_stimuli_near_sync_negative_mode_requires_recover_contract() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("quoted_string", "ok"));
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![SemanticAnnotation::Named {
                name: "sync".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: "[\";\"]".to_string(),
                },
            }],
        );

        let mut generator = annotated_generator_with_mode(
            &grammar_tree,
            &rule_order,
            &annotations,
            9313,
            RecoveryStimuliMode::NearSyncNegative,
        );
        let values = generator.generate_many(3, Some("start")).expect(
            "near-sync-negative mode should fall back to baseline generation without recover",
        );

        assert!(
            values.iter().all(|value| value == "ok"),
            "near-sync-negative mode must stay inactive without @recover: true, got {:?}",
            values
        );
    }

    #[test]
    fn semantic_usage_stimuli_invalid_case_mutates_entry_output() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("quoted_string", "ok"));
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![SemanticAnnotation::Named {
                name: "invalid_case".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: "true".to_string(),
                },
            }],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9321);
        let values = generator
            .generate_many(4, Some("start"))
            .expect("invalid_case steering should still generate deterministic samples");

        assert!(
            values.iter().all(|value| value != "ok"),
            "invalid_case steering should mutate baseline valid sample, got {:?}",
            values
        );
    }

    #[test]
    fn semantic_usage_stimuli_negative_requires_invalid_case_contract() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("quoted_string", "ok"));
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![SemanticAnnotation::Named {
                name: "negative".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: "true".to_string(),
                },
            }],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9322);
        let values = generator
            .generate_many(4, Some("start"))
            .expect("negative-only contract should stay parseable");

        assert!(
            values.iter().all(|value| value == "ok"),
            "negative steering must stay inactive unless invalid_case is enabled, got {:?}",
            values
        );
    }

    #[test]
    fn semantic_usage_stimuli_invalid_case_plus_negative_appends_marker() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("quoted_string", "ok"));
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "invalid_case".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "true".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "negative".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "true".to_string(),
                    },
                },
            ],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9323);
        let values = generator
            .generate_many(4, Some("start"))
            .expect("invalid_case+negative steering should generate near-invalid samples");

        assert!(
            values
                .iter()
                .all(|value| value.contains("__pgen_negative_case_start__")),
            "invalid_case+negative steering should append deterministic negative-case marker, got {:?}",
            values
        );
    }

    #[test]
    fn semantic_usage_stimuli_seed_group_stays_inactive_without_deterministic_group() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("quoted_string", "A"),
                    token("quoted_string", "B"),
                    token("quoted_string", "C"),
                ],
            },
        );
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![SemanticAnnotation::Named {
                name: "seed_group".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: "\"stable.alpha\"".to_string(),
                },
            }],
        );

        let mut baseline = simple_generator(&grammar_tree, &rule_order, 9911);
        let mut hinted = annotated_generator(&grammar_tree, &rule_order, &annotations, 9911);

        let baseline_values = baseline
            .generate_many(12, Some("start"))
            .expect("baseline generation should succeed");
        let hinted_values = hinted
            .generate_many(12, Some("start"))
            .expect("hinted generation should succeed");

        assert_eq!(
            baseline_values, hinted_values,
            "seed_group alone should not change sequence unless deterministic_group is enabled"
        );
    }

    #[test]
    fn semantic_usage_stimuli_deterministic_group_string_payload_enables_partition() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("quoted_string", "ok"));
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![SemanticAnnotation::Named {
                name: "deterministic_group".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: "\"stable.alpha\"".to_string(),
                },
            }],
        );

        let generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9912);
        let policy = generator.rule_determinism_partition_policy("start");
        assert!(policy.enabled);
        assert_eq!(policy.group_label.as_deref(), Some("stable.alpha"));
    }

    #[test]
    fn semantic_usage_stimuli_deterministic_partitions_are_order_independent() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "lhs".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("quoted_string", "L0"),
                    token("quoted_string", "L1"),
                    token("quoted_string", "L2"),
                    token("quoted_string", "L3"),
                ],
            },
        );
        grammar_tree.insert(
            "rhs".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("quoted_string", "R0"),
                    token("quoted_string", "R1"),
                    token("quoted_string", "R2"),
                    token("quoted_string", "R3"),
                ],
            },
        );
        let rule_order = vec!["lhs".to_string(), "rhs".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "lhs".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "seed_group".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "\"stable.lhs\"".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "deterministic_group".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "true".to_string(),
                    },
                },
            ],
        );
        annotations.semantic_annotations.insert(
            "rhs".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "seed_group".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "\"stable.rhs\"".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "deterministic_group".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "true".to_string(),
                    },
                },
            ],
        );

        let mut a_then_b = annotated_generator(&grammar_tree, &rule_order, &annotations, 9913);
        let lhs_seq_a = vec![
            a_then_b
                .generate_from_entry("lhs")
                .expect("lhs generation should succeed"),
            a_then_b
                .generate_from_entry("lhs")
                .expect("lhs generation should succeed"),
        ];
        let rhs_seq_a = vec![
            a_then_b
                .generate_from_entry("rhs")
                .expect("rhs generation should succeed"),
            a_then_b
                .generate_from_entry("rhs")
                .expect("rhs generation should succeed"),
        ];

        let mut b_then_a = annotated_generator(&grammar_tree, &rule_order, &annotations, 9913);
        let rhs_seq_b = vec![
            b_then_a
                .generate_from_entry("rhs")
                .expect("rhs generation should succeed"),
            b_then_a
                .generate_from_entry("rhs")
                .expect("rhs generation should succeed"),
        ];
        let lhs_seq_b = vec![
            b_then_a
                .generate_from_entry("lhs")
                .expect("lhs generation should succeed"),
            b_then_a
                .generate_from_entry("lhs")
                .expect("lhs generation should succeed"),
        ];

        assert_eq!(
            lhs_seq_a, lhs_seq_b,
            "lhs partition sequence should not depend on interleaving with rhs"
        );
        assert_eq!(
            rhs_seq_a, rhs_seq_b,
            "rhs partition sequence should not depend on interleaving with lhs"
        );
    }

    #[test]
    fn semantic_usage_stimuli_relational_constraint_filters_cross_capture_values() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("ident".to_string(), token("regex", "^[A-Z]{2}$"));
        grammar_tree.insert(
            "pair".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    token("rule_reference", "ident"),
                    token("quoted_string", ":"),
                    token("rule_reference", "ident"),
                ],
            },
        );
        let rule_order = vec!["pair".to_string(), "ident".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "ident".to_string(),
            vec![SemanticAnnotation::Named {
                name: "enum".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: "[\"AA\", \"BB\"]".to_string(),
                },
            }],
        );
        annotations.semantic_annotations.insert(
            "pair".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "constraint".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "\"$1 != $3\"".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "requires".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[\"$1\", \"$3\"]".to_string(),
                    },
                },
            ],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9401);
        let values = generator
            .generate_many(24, Some("pair"))
            .expect("relationally constrained stimuli generation should succeed");

        for value in values {
            let parts: Vec<&str> = value.split(':').collect();
            assert_eq!(
                parts.len(),
                2,
                "pair stimuli should preserve expected shape, got {:?}",
                value
            );
            assert_ne!(
                parts[0], parts[1],
                "@constraint should enforce distinct captures, got {:?}",
                value
            );
            assert!(
                matches!(parts[0], "AA" | "BB") && matches!(parts[1], "AA" | "BB"),
                "pair captures should stay inside enum-constrained domain, got {:?}",
                value
            );
        }
    }

    #[test]
    fn semantic_usage_stimuli_relational_implies_enforced_during_generation() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "rhs".to_string(),
            ASTNode::Or {
                alternatives: vec![token("quoted_string", "B"), token("quoted_string", "C")],
            },
        );
        grammar_tree.insert(
            "pair".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    token("quoted_string", "A"),
                    token("quoted_string", ":"),
                    token("rule_reference", "rhs"),
                ],
            },
        );
        let rule_order = vec!["pair".to_string(), "rhs".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "pair".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "constraint".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "true".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "requires".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[\"$1\", \"$3\"]".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "implies".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "\"$1 == 'A' => $3 == 'B'\"".to_string(),
                    },
                },
            ],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9402);
        let values = generator
            .generate_many(24, Some("pair"))
            .expect("relational implication constrained generation should succeed");

        assert!(
            values.iter().all(|value| value == "A:B"),
            "@implies contract should suppress consequent-violating samples, got {:?}",
            values
        );
    }

    #[test]
    fn semantic_usage_stimuli_relational_supports_nested_named_paths() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("ident".to_string(), token("regex", "^[A-Z]{2}$"));
        grammar_tree.insert(
            "lhs".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    token("quoted_string", "{\"id\":\""),
                    token("rule_reference", "ident"),
                    token("quoted_string", "\"}"),
                ],
            },
        );
        grammar_tree.insert(
            "rhs".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    token("quoted_string", "{\"id\":\""),
                    token("rule_reference", "ident"),
                    token("quoted_string", "\"}"),
                ],
            },
        );
        grammar_tree.insert(
            "pair".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    token("rule_reference", "lhs"),
                    token("quoted_string", "|"),
                    token("rule_reference", "rhs"),
                ],
            },
        );
        let rule_order = vec![
            "pair".to_string(),
            "lhs".to_string(),
            "rhs".to_string(),
            "ident".to_string(),
        ];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "ident".to_string(),
            vec![SemanticAnnotation::Named {
                name: "enum".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: "[\"AA\", \"BB\"]".to_string(),
                },
            }],
        );
        annotations.semantic_annotations.insert(
            "pair".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "constraint".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "\"lhs.id != rhs.id\"".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "requires".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[\"lhs.id\", \"rhs.id\"]".to_string(),
                    },
                },
            ],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9404);
        let values = generator
            .generate_many(24, Some("pair"))
            .expect("nested named-path relational constraints should be satisfiable");

        for sample in values {
            let parts: Vec<&str> = sample.split('|').collect();
            assert_eq!(
                parts.len(),
                2,
                "pair sample must preserve expected split shape, got {:?}",
                sample
            );
            let lhs: JsonValue = serde_json::from_str(parts[0])
                .expect("lhs capture should remain a parseable JSON object");
            let rhs: JsonValue = serde_json::from_str(parts[1])
                .expect("rhs capture should remain a parseable JSON object");
            let lhs_id = lhs
                .get("id")
                .and_then(JsonValue::as_str)
                .expect("lhs.id should be present");
            let rhs_id = rhs
                .get("id")
                .and_then(JsonValue::as_str)
                .expect("rhs.id should be present");
            assert_ne!(
                lhs_id, rhs_id,
                "nested named-path constraint lhs.id != rhs.id should hold, got {:?}",
                sample
            );
        }
    }

    #[test]
    fn semantic_usage_stimuli_relational_supports_positional_nested_paths() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("ident".to_string(), token("regex", "^[A-Z]{2}$"));
        grammar_tree.insert(
            "lhs".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    token("quoted_string", "{\"id\":\""),
                    token("rule_reference", "ident"),
                    token("quoted_string", "\"}"),
                ],
            },
        );
        grammar_tree.insert(
            "rhs".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    token("quoted_string", "{\"id\":\""),
                    token("rule_reference", "ident"),
                    token("quoted_string", "\"}"),
                ],
            },
        );
        grammar_tree.insert(
            "pair".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    token("rule_reference", "lhs"),
                    token("quoted_string", "|"),
                    token("rule_reference", "rhs"),
                ],
            },
        );
        let rule_order = vec![
            "pair".to_string(),
            "lhs".to_string(),
            "rhs".to_string(),
            "ident".to_string(),
        ];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "ident".to_string(),
            vec![SemanticAnnotation::Named {
                name: "enum".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: "[\"AA\", \"BB\"]".to_string(),
                },
            }],
        );
        annotations.semantic_annotations.insert(
            "pair".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "constraint".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "\"$1.id == 'AA' && $3.id.len == 2\"".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "requires".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[\"$1.id\", \"$3.id.len\"]".to_string(),
                    },
                },
            ],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9405);
        let values = generator
            .generate_many(24, Some("pair"))
            .expect("positional nested-path relational constraints should be satisfiable");

        for sample in values {
            let parts: Vec<&str> = sample.split('|').collect();
            assert_eq!(
                parts.len(),
                2,
                "pair sample must preserve expected split shape, got {:?}",
                sample
            );
            let lhs: JsonValue = serde_json::from_str(parts[0])
                .expect("lhs capture should remain a parseable JSON object");
            let lhs_id = lhs
                .get("id")
                .and_then(JsonValue::as_str)
                .expect("lhs.id should be present");
            assert_eq!(
                lhs_id, "AA",
                "positional nested-path constraint $1.id == 'AA' should hold, got {:?}",
                sample
            );
        }
    }

    #[test]
    fn semantic_usage_stimuli_relational_supports_nonstructured_named_paths() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("ident".to_string(), token("regex", "^[A-Z]{2}$"));
        grammar_tree.insert(
            "lhs".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    token("quoted_string", "{id="),
                    token("rule_reference", "ident"),
                    token("quoted_string", ",meta.kind=lhs}"),
                ],
            },
        );
        grammar_tree.insert(
            "rhs".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    token("quoted_string", "{id="),
                    token("rule_reference", "ident"),
                    token("quoted_string", ",meta.kind=rhs}"),
                ],
            },
        );
        grammar_tree.insert(
            "pair".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    token("rule_reference", "lhs"),
                    token("quoted_string", "|"),
                    token("rule_reference", "rhs"),
                ],
            },
        );
        let rule_order = vec![
            "pair".to_string(),
            "lhs".to_string(),
            "rhs".to_string(),
            "ident".to_string(),
        ];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "ident".to_string(),
            vec![SemanticAnnotation::Named {
                name: "enum".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: "[\"AA\", \"BB\"]".to_string(),
                },
            }],
        );
        annotations.semantic_annotations.insert(
            "pair".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "constraint".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "\"lhs.id != rhs.id && lhs.meta.kind == 'lhs' && rhs.meta.kind == 'rhs'\""
                            .to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "requires".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[\"lhs.id\", \"rhs.id\", \"lhs.meta.kind\", \"rhs.meta.kind\"]"
                            .to_string(),
                    },
                },
            ],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9406);
        let values = generator
            .generate_many(24, Some("pair"))
            .expect("non-structured named-path relational constraints should be satisfiable");

        for sample in values {
            let parts: Vec<&str> = sample.split('|').collect();
            assert_eq!(
                parts.len(),
                2,
                "pair sample must preserve expected split shape, got {:?}",
                sample
            );
            let lhs = StimuliGenerator::parse_capture_value_as_json(parts[0])
                .expect("lhs capture should parse with non-structured object heuristics");
            let rhs = StimuliGenerator::parse_capture_value_as_json(parts[1])
                .expect("rhs capture should parse with non-structured object heuristics");
            let lhs_id = lhs
                .get("id")
                .and_then(JsonValue::as_str)
                .expect("lhs.id should be present");
            let rhs_id = rhs
                .get("id")
                .and_then(JsonValue::as_str)
                .expect("rhs.id should be present");
            let lhs_kind = lhs
                .get("meta")
                .and_then(|meta| meta.get("kind"))
                .and_then(JsonValue::as_str)
                .expect("lhs.meta.kind should be present");
            let rhs_kind = rhs
                .get("meta")
                .and_then(|meta| meta.get("kind"))
                .and_then(JsonValue::as_str)
                .expect("rhs.meta.kind should be present");
            assert_ne!(
                lhs_id, rhs_id,
                "non-structured named-path constraint lhs.id != rhs.id should hold, got {:?}",
                sample
            );
            assert_eq!(
                lhs_kind, "lhs",
                "lhs.meta.kind contract should hold, got {:?}",
                sample
            );
            assert_eq!(
                rhs_kind, "rhs",
                "rhs.meta.kind contract should hold, got {:?}",
                sample
            );
        }
    }

    #[test]
    fn semantic_usage_stimuli_relational_supports_nonstructured_positional_paths() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("ident".to_string(), token("regex", "^[A-Z]{2}$"));
        grammar_tree.insert(
            "lhs".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    token("quoted_string", "(meta.id:"),
                    token("rule_reference", "ident"),
                    token("quoted_string", ",meta.kind:lhs)"),
                ],
            },
        );
        grammar_tree.insert(
            "rhs".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    token("quoted_string", "(meta.id:"),
                    token("rule_reference", "ident"),
                    token("quoted_string", ",meta.kind:rhs)"),
                ],
            },
        );
        grammar_tree.insert(
            "pair".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    token("rule_reference", "lhs"),
                    token("quoted_string", "|"),
                    token("rule_reference", "rhs"),
                ],
            },
        );
        let rule_order = vec![
            "pair".to_string(),
            "lhs".to_string(),
            "rhs".to_string(),
            "ident".to_string(),
        ];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "ident".to_string(),
            vec![SemanticAnnotation::Named {
                name: "enum".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: "[\"AA\", \"BB\"]".to_string(),
                },
            }],
        );
        annotations.semantic_annotations.insert(
            "pair".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "constraint".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "\"$1.meta.id != $3.meta.id && $1.meta.kind == 'lhs' && $3.meta.kind == 'rhs'\""
                            .to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "requires".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[\"$1.meta.id\", \"$3.meta.id\", \"$1.meta.kind\", \"$3.meta.kind\"]"
                            .to_string(),
                    },
                },
            ],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9407);
        let values = generator
            .generate_many(24, Some("pair"))
            .expect("non-structured positional nested-path constraints should be satisfiable");

        for sample in values {
            let parts: Vec<&str> = sample.split('|').collect();
            assert_eq!(
                parts.len(),
                2,
                "pair sample must preserve expected split shape, got {:?}",
                sample
            );
            let lhs = StimuliGenerator::parse_capture_value_as_json(parts[0])
                .expect("lhs capture should parse with non-structured object heuristics");
            let rhs = StimuliGenerator::parse_capture_value_as_json(parts[1])
                .expect("rhs capture should parse with non-structured object heuristics");
            let lhs_id = lhs
                .get("meta")
                .and_then(|meta| meta.get("id"))
                .and_then(JsonValue::as_str)
                .expect("lhs.meta.id should be present");
            let rhs_id = rhs
                .get("meta")
                .and_then(|meta| meta.get("id"))
                .and_then(JsonValue::as_str)
                .expect("rhs.meta.id should be present");
            assert_ne!(
                lhs_id, rhs_id,
                "non-structured positional path constraint should keep ids distinct, got {:?}",
                sample
            );
        }
    }

    #[test]
    fn semantic_usage_stimuli_relational_hints_without_constraint_remain_inactive() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("quoted_string", "ok"));
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![SemanticAnnotation::Named {
                name: "requires".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: "[\"$9\"]".to_string(),
                },
            }],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9403);
        let values = generator
            .generate_many(8, Some("start"))
            .expect("@requires without @constraint should stay inactive for stimuli generation");
        assert!(
            values.iter().all(|value| value == "ok"),
            "inactive relational hints should not alter sample generation, got {:?}",
            values
        );
    }

    #[test]
    fn semantic_usage_stimuli_relational_unsat_reports_ranked_violation_summary() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("ident".to_string(), token("quoted_string", "AA"));
        grammar_tree.insert(
            "pair".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    token("rule_reference", "ident"),
                    token("quoted_string", ":"),
                    token("rule_reference", "ident"),
                ],
            },
        );
        let rule_order = vec!["pair".to_string(), "ident".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "pair".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "constraint".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "\"$1 != $3\"".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "requires".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[\"$1\", \"$3\"]".to_string(),
                    },
                },
            ],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9410);
        let expected_budget = generator.relational_attempt_budget();
        let err = generator
            .generate_many(1, Some("pair"))
            .expect_err("unsatisfiable relational contracts should surface structured diagnostics");
        let message = err.to_string();

        assert!(
            message.contains(&format!("relational_failures={}", expected_budget)),
            "error should report relational failure count, got {:?}",
            message
        );
        assert!(
            message.contains("generation_failures=0"),
            "error should report generation failures separately, got {:?}",
            message
        );
        assert!(
            message.contains("top_violations=["),
            "error should report ranked violation reasons, got {:?}",
            message
        );
        assert!(
            message.contains("Semantic relational constraint failed for rule 'pair': $1 != $3"),
            "error should keep root cause in ranked summary, got {:?}",
            message
        );
        assert!(
            message.contains("likely_unsatisfiable=true"),
            "error should flag consistently failing contracts as likely unsatisfiable, got {:?}",
            message
        );
    }
}
