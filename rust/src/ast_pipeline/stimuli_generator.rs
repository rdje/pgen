use super::{
    ASTNode, ASTValue, Annotations, SemanticAnnotation, SemanticAssociativity,
    SemanticBranchPolicy, SemanticTokenClass, SemanticValueConstraints, TokenValue, TraceLevel,
    TraceVerbosity, UnifiedSemanticAST, UnifiedSemanticValue, extract_semantic_directive,
    global_trace_verbosity, normalize_semantic_scalar, parse_canonical_transform_expression,
    parse_semantic_bool, parse_semantic_branch_priorities, parse_semantic_charset,
    parse_semantic_constraint_expression, parse_semantic_coverage_target_weight,
    parse_semantic_deterministic_group, parse_semantic_group_label, parse_semantic_implication,
    parse_semantic_len_bounds, parse_semantic_numeric_bounds, parse_semantic_pattern,
    parse_semantic_reference_list, parse_semantic_string_list, parse_semantic_token_class,
    stimuli_hint_for_target_type,
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
// GRAMMAR-WELLFORMED.B1: wall-clock time (`std::time::{Duration, Instant}`) is no longer used —
// the generation budget is a DETERMINISTIC step counter, so no run-to-run timing variance.

const HELPER_TIMEOUT_ERROR_PREFIX: &str = "Stimuli generation helper timeout exceeded";
const TARGET_TIMEOUT_ERROR_PREFIX: &str = "Stimuli generation target timeout exceeded";
// SV-EXH-PROOF.7.4.5 (PGEN-SV-EXH-PROOF-0145): default per-witness budget FLOOR (ms). The
// witness pass is a deliberate, count-bounded, monotone pass, so it can afford more time
// than the (often tiny) PRIMARY target-drive budget it used to reuse — e.g. the SV gate's
// `closed_loop_target_generation_timeout_ms=5` left ~99% of the residual as `target_timeout`
// (.7.4.4 canonical run: 753, of which 746 were timeouts). The witness budget is
// `max(primary, floor)` so it NEVER shrinks below the primary; override the floor with
// `PGEN_WITNESS_TIMEOUT_FLOOR_MS` for tuning/A-B (0 disables the floor → reuse the primary).
const WITNESS_TIMEOUT_FLOOR_MS: u64 = 200;

// SV-EXH-PROOF.7.4.6.2: nullability cache — the dominant witness-generation cost is
// `node_is_nullable` recomputing a STATIC grammar property recursively on every call. This
// thread-local cache is grammar-GUARDED (rule-name keys are grammar-specific, so it is cleared
// when the active grammar changes — see `StimuliGenerator::new`). SOUND / monotone BY
// CONSTRUCTION: a rule's nullability is cached ONLY when its computation was CYCLE-FREE (no
// back-edge into the `visiting` set) — i.e. context-INDEPENDENT — so a cache hit equals the
// per-call DFS result exactly. Cyclic / context-dependent results are never cached (recomputed,
// behaviour unchanged). Generation output is therefore byte-identical; this is pure speedup.
thread_local! {
    static NULLABLE_CACHE: std::cell::RefCell<HashMap<String, bool>> =
        std::cell::RefCell::new(HashMap::new());
    static NULLABLE_CACHE_GRAMMAR: std::cell::RefCell<Option<String>> =
        std::cell::RefCell::new(None);
}
// DIAG-SEVERITY.3 (PGEN-DIAG-SEVERITY-0003): the depth-limit error raised at
// `generate_rule` (this file, ~4379). Classifying generation failures by this reason —
// instead of folding them into an anonymous `generation_errors` count — is what makes
// the SV residual's dominant cause (depth-budget exhaustion, SV-EXH-PROOF.7.4.3a)
// visible at the gate level instead of masked.
const DEPTH_EXCEEDED_ERROR_PREFIX: &str = "Stimuli generation depth exceeded max_depth=";
// DIAG-SEVERITY.3.1: the SECOND structural-budget failure (sibling of depth-exceeded),
// raised at `generate_rule` (~4438) when a rule is entered more than `max_rule_visits`
// times in one derivation. The .3 taxonomy revealed it was also anonymous; classifying
// it un-masks a second potential contributor to the SV residual.
const RULE_VISIT_LIMIT_ERROR_PREFIX: &str = "Stimuli generation exceeded max_rule_visits=";

/// DIAG-SEVERITY.3.1 (PGEN-DIAG-SEVERITY-0004): the canonical, drift-proof enumeration of
/// stimuli generation-failure reasons — the single source of truth for "classify errors
/// by reason" (director ask). `classify_generation_error` is the one place that maps an
/// error to its reason; counters/telemetry derive from it. The two STRUCTURAL-BUDGET
/// reasons (DepthExceeded, RuleVisitLimit) are the residual-relevant class
/// (SV-EXH-PROOF.7.4.3a). See the taxonomy table in docs/tasks/DIAG-SEVERITY.md.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerationErrorReason {
    DepthExceeded,
    RuleVisitLimit,
    TargetTimeout,
    HelperTimeout,
    /// Any other generation failure (incl. expected PEG backtracking, quantifier/weight/
    /// semantic config errors). Folded into the generic `generation_errors` total; split
    /// into finer reasons only if one ever proves residual-relevant.
    Other,
}

#[derive(Debug, Clone, Copy)]
struct GenerationTimeoutBudget {
    /// GRAMMAR-WELLFORMED.B1: a DETERMINISTIC budget in generation STEPS (deadline-checks),
    /// replacing the old wall-clock `Duration`. Derived from the ms budget × steps-per-ms, so
    /// the existing ms config still drives it — but the cutoff is machine-INDEPENDENT (a seeded
    /// run produces the SAME residual every time, the literal-0 prerequisite).
    step_budget: u64,
    error_prefix: &'static str,
    budget_ms: u64,
}

#[derive(Debug, Clone, Copy)]
struct ActiveGenerationDeadline {
    /// GRAMMAR-WELLFORMED.B1: the step-counter value at which generation is over budget
    /// (`start_step + step_budget`). Deterministic — no wall clock.
    step_deadline: u64,
    error_prefix: &'static str,
    budget_ms: u64,
}

/// GRAMMAR-WELLFORMED.B1: how many generation STEPS (deadline-checks, ≈ per rule/node) one
/// millisecond of the configured budget maps to. The mapping is fixed (deterministic); the value
/// only sets the budget MAGNITUDE so existing ms config (gate's witness/target budgets) keeps
/// working. Env-tunable so the magnitude can be calibrated without recompiling.
const DEFAULT_GENERATION_STEPS_PER_MS: u64 = 1000;
fn generation_steps_per_ms() -> u64 {
    std::env::var("PGEN_GENERATION_STEPS_PER_MS")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .filter(|&v| v > 0)
        .unwrap_or(DEFAULT_GENERATION_STEPS_PER_MS)
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StimuliMutationMode {
    Baseline,
    GrammarAwareLocal,
}

impl Default for StimuliMutationMode {
    fn default() -> Self {
        Self::Baseline
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StimuliConstraintProfile {
    Baseline,
    RareBranchBiased,
    DeepNestingBiased,
}

impl Default for StimuliConstraintProfile {
    fn default() -> Self {
        Self::Baseline
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StimuliNegativeProfile {
    Baseline,
    NearValidLocal,
}

impl Default for StimuliNegativeProfile {
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
    pub target_pending_frontier_extra_stagnation: usize,
    pub target_generation_timeout_ms: u64,
    pub target_helper_generation_timeout_ms: u64,
    pub recovery_mode: RecoveryStimuliMode,
    pub mutation_mode: StimuliMutationMode,
    pub constraint_profile: StimuliConstraintProfile,
    pub negative_profile: StimuliNegativeProfile,
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
            target_pending_frontier_extra_stagnation: 8,
            target_generation_timeout_ms: 0,
            target_helper_generation_timeout_ms: 1000,
            recovery_mode: RecoveryStimuliMode::Baseline,
            mutation_mode: StimuliMutationMode::Baseline,
            constraint_profile: StimuliConstraintProfile::Baseline,
            negative_profile: StimuliNegativeProfile::Baseline,
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
    #[serde(default)]
    pub failure_reasons: Vec<HashMap<String, u64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BranchFailureReasonCount {
    pub reason: String,
    pub count: u64,
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
                    failure_reasons: vec![HashMap::new(); other_group.total_branches],
                });

            if group.total_branches < other_group.total_branches {
                group.selected_counts.resize(other_group.total_branches, 0);
                group.success_counts.resize(other_group.total_branches, 0);
                group
                    .failure_reasons
                    .resize(other_group.total_branches, HashMap::new());
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
            for (idx, reasons) in other_group.failure_reasons.iter().enumerate() {
                if idx >= group.failure_reasons.len() {
                    group.failure_reasons.push(HashMap::new());
                }
                for (reason, count) in reasons {
                    *group.failure_reasons[idx]
                        .entry(reason.clone())
                        .or_insert(0) += count;
                }
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
                failure_reasons: vec![HashMap::new(); total_branches],
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
            if group.failure_reasons.len() <= branch_idx {
                group.failure_reasons.resize(branch_idx + 1, HashMap::new());
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
            if group.failure_reasons.len() <= branch_idx {
                group.failure_reasons.resize(branch_idx + 1, HashMap::new());
            }
            if group.total_branches < total_branches {
                group.total_branches = total_branches;
            }
            group.success_counts[branch_idx] += 1;
        }
    }

    fn record_branch_failure(
        &mut self,
        group_key: &str,
        rule_name: &str,
        node_path: &str,
        total_branches: usize,
        branch_idx: usize,
        reason: &str,
    ) {
        self.ensure_group_entry(group_key, rule_name, node_path, total_branches);
        if let Some(group) = self.branch_groups.get_mut(group_key) {
            if group.failure_reasons.len() <= branch_idx {
                group.failure_reasons.resize(branch_idx + 1, HashMap::new());
            }
            if group.selected_counts.len() <= branch_idx {
                group.selected_counts.resize(branch_idx + 1, 0);
            }
            if group.success_counts.len() <= branch_idx {
                group.success_counts.resize(branch_idx + 1, 0);
            }
            if group.total_branches < total_branches {
                group.total_branches = total_branches;
            }
            let normalized = Self::normalize_failure_reason(reason);
            *group.failure_reasons[branch_idx]
                .entry(normalized)
                .or_insert(0) += 1;
        }
    }

    fn normalize_failure_reason(reason: &str) -> String {
        let single_line = reason
            .lines()
            .next()
            .unwrap_or(reason)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let trimmed = single_line.trim();
        const MAX_LEN: usize = 160;
        if trimmed.len() <= MAX_LEN {
            trimmed.to_string()
        } else {
            format!("{}...", &trimmed[..MAX_LEN])
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    #[serde(default)]
    pub top_failure_reasons: Vec<BranchFailureReasonCount>,
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
    /// SV-EXH-PROOF.7.2.16 (PGEN-SV-EXH-PROOF-0131): READ-ONLY reach classification
    /// of this residual target, computed by `generate_gap_report` via
    /// `compute_reach_path` (no effect on generation). One of:
    ///   * `"reachable_by_plan"`  — a deterministic reach plan EXISTS to steer to
    ///                              this branch (so it is in principle coverable by
    ///                              the reach driver; not-yet-covered is a steering
    ///                              gap, not a structural impossibility).
    ///   * `"no_reach_path"`      — no reach plan exists to this specific branch
    ///                              (compute_reach_path returned None): the branch
    ///                              is not steerable from the entry as encoded.
    ///   * `"reachable_rule_not_generated"` — a RULE target (no node_path/branch):
    ///                              graph-reachable but never generated.
    /// `None` when classification was not run (older artifacts / non-target rows).
    /// Additive + `#[serde(default)]` → back-compatible with existing report JSON.
    #[serde(default)]
    pub reach_classification: Option<String>,
}

/// SV-EXH-PROOF.7.2.1 (PGEN-SV-EXH-PROOF-0115): one forced OR-branch decision in
/// a target-reach plan — at the OR node located at `node_path` within rule
/// `rule_name`, force alternative `branch_index`. Produced by
/// `compute_reach_path` (pure analysis); consumed by a later leaf (.7.2.2) that
/// wires the plan into the OR-decision. See
/// docs/tasks/SV-EXH-PROOF-7.2-target-reach-design.md.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReachDirective {
    pub rule_name: String,
    pub node_path: String,
    pub branch_index: usize,
}

/// SV-EXH-PROOF.7.2.1: where a rule reference occurs inside another rule's body,
/// using the same `node_path` encoding as `collect_branch_groups`. Internal
/// helper for the rule-reference-graph BFS in `compute_reach_path`.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct RuleReferenceSite {
    referenced_rule: String,
    node_path: String,
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
                let top_failure_reasons = if debt.top_failure_reasons.is_empty() {
                    String::new()
                } else {
                    format!(
                        " failure_reasons=[{}]",
                        debt.top_failure_reasons
                            .iter()
                            .map(|item| format!("{} ({})", item.reason, item.count))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };
                out.push_str(&format!(
                    "- {} | selected={} success={} required={} deficit={} priority={} reason={} refs=[{}] uncovered_refs=[{}]{}\n",
                    debt.branch_id,
                    debt.selected_hits,
                    debt.success_hits,
                    debt.required_successes,
                    debt.deficit,
                    debt.priority_score,
                    debt.reason,
                    debt.rule_references.join(","),
                    debt.uncovered_rule_references.join(","),
                    top_failure_reasons
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
    /// DIAG-SEVERITY.3 (PGEN-DIAG-SEVERITY-0003): generation attempts that failed by
    /// hitting the recursion depth limit. Surfaced as its own bucket so the SV
    /// residual's dominant cause (SV-EXH-PROOF.7.4.3a) is visible, not folded into the
    /// anonymous `generation_errors` count. `#[serde(default)]` keeps older JSON loadable.
    #[serde(default)]
    pub depth_exceeded_errors: usize,
    /// DIAG-SEVERITY.3.1: generation attempts that failed by hitting `max_rule_visits`
    /// (the second structural-budget reason; sibling of `depth_exceeded_errors`).
    #[serde(default)]
    pub rule_visit_limit_errors: usize,
    #[serde(default)]
    pub target_timeout_errors: usize,
    #[serde(default)]
    pub helper_timeout_errors: usize,
    pub total_targets: usize,
    pub applied_targets: usize,
    pub resolved_targets: usize,
    pub unresolved_targets: Vec<TargetCoverageStatus>,
    /// SV-EXH-PROOF.7.2.5 (PGEN-SV-EXH-PROOF-0119): how many times the `.7.2.4`
    /// reach-driver steering hook fired during this run. Surfaced so the gate /
    /// CLI can MEASURE whether the reach driver actually engaged (vs the baseline
    /// weighting alone retiring targets) — without it the driver's impact is
    /// unobservable. `#[serde(default)]` keeps older summary JSON loadable.
    #[serde(default)]
    pub reach_plan_activations: u64,
}

/// SV-EXH-PROOF.7.4.3 (PGEN-SV-EXH-PROOF-0141): outcome of the APPENDED minimal-witness
/// pass — the decoupled literal-0 completion (Purdom/Hennessy-&-Power set-cover) that runs
/// AFTER the diverse + target-drive passes and only ADDS coverage (monotone: residual can
/// only shrink). Failures are classified by the canonical `GenerationErrorReason`
/// (DIAG-SEVERITY) so a residual tail's cause is visible, not masked.
/// SV-EXH-PROOF.7.4.4.1: cap on the number of "other"-class witness-failure samples
/// collected into the summary. Bounded so the diagnostic never blows up memory/disk
/// (a per-target Debug trace at debug verbosity produced a 6.9 GB log). 40 is enough to
/// characterise the residual tail's cause distribution without retaining every failure.
const WITNESS_OTHER_FAILURE_SAMPLE_CAP: usize = 40;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WitnessSummary {
    pub total_targets: usize,
    pub resolved_before: usize,
    pub resolved_after: usize,
    pub witnesses_generated: usize,
    #[serde(default)]
    pub depth_exceeded_failures: usize,
    #[serde(default)]
    pub rule_visit_limit_failures: usize,
    #[serde(default)]
    pub other_failures: usize,
    #[serde(default)]
    pub no_entry_rule: usize,
    /// SV-EXH-PROOF.7.4.4.1: timeout failures, classified separately from `other_failures`
    /// so a bounded per-generation budget does not pollute the genuine-error tally.
    #[serde(default)]
    pub target_timeout_failures: usize,
    #[serde(default)]
    pub helper_timeout_failures: usize,
    /// SV-EXH-PROOF.7.4.4.1 (TOOL-BUILD): bounded sample of the "other"-class witness
    /// failures (the residual tail), each "target_id=.. rule=.. type=.. node_path=..
    /// branch=.. reason=.." (reason = full anyhow error chain). Capped at
    /// `WITNESS_OTHER_FAILURE_SAMPLE_CAP`. Lets the tail be characterised on tangible
    /// proof at default verbosity ([[feedback_why_and_where_before_solution]]).
    #[serde(default)]
    pub other_failure_samples: Vec<String>,
    /// SV-EXH-PROOF.7.4.6.4 (TOOL-BUILD, WHY+WHERE before fix): how many target witnesses
    /// had the CONSTRUCTION attempt return Err and therefore fell back to the search path.
    /// Distinguishes "construction dead-ended → slow search → timeout" from "construction
    /// succeeded but didn't cover" — the two have opposite fixes. `#[serde(default)]` keeps
    /// older JSON loadable.
    #[serde(default)]
    pub construct_attempt_failures: usize,
    /// SV-EXH-PROOF.7.4.6.4 (TOOL-BUILD): bounded sample of the TARGET_TIMEOUT-class witness
    /// failures (the residual tail at the gate's tight budget) — same shape as
    /// `other_failure_samples`. The target_timeout class was previously counted but never
    /// sampled, so the residual's WHERE (which rules/types dead-end) was invisible.
    #[serde(default)]
    pub timeout_failure_samples: Vec<String>,
    /// SV-EXH-PROOF.7.4.6.4 (TOOL-BUILD): bounded sample of targets STILL unresolved after the
    /// witness pass — surfaces the non-covering class (construct Ok, witness didn't cover) that
    /// never errors. `attempted=true` means a witness WAS generated for it yet it stayed
    /// uncovered.
    #[serde(default)]
    pub unresolved_after_samples: Vec<String>,
}

impl WitnessSummary {
    pub fn summary_line(&self) -> String {
        format!(
            "Witness pass: resolved {} -> {} of {} reachable targets (+{} via {} witnesses; failures depth_exceeded={}, rule_visit_limit={}, target_timeout={}, helper_timeout={}, other={}, no_entry={}; construct_fell_back_to_search={})",
            self.resolved_before,
            self.resolved_after,
            self.total_targets,
            self.resolved_after.saturating_sub(self.resolved_before),
            self.witnesses_generated,
            self.depth_exceeded_failures,
            self.rule_visit_limit_failures,
            self.target_timeout_failures,
            self.helper_timeout_failures,
            self.other_failures,
            self.no_entry_rule,
            self.construct_attempt_failures
        )
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TargetDriveValidationSummary {
    pub validated_outputs: usize,
    pub accepted_outputs: usize,
    pub rejected_outputs: usize,
    pub alternate_entry_attempts: usize,
    pub alternate_entry_accepted_outputs: usize,
    pub alternate_entry_rejected_outputs: usize,
    pub target_timeout_errors: usize,
    pub helper_timeout_errors: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetDriveFilterContext<'a> {
    pub primary_entry_rule: &'a str,
    pub generation_entry_rule: &'a str,
    pub is_primary_entry: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct DependencyProbeCandidate {
    rule_name: String,
    blocked_targets: usize,
    blocked_remaining_successes: u64,
    max_target_priority: u64,
    dependency_rule_deficit: u64,
    dependency_rule_successes: u64,
    literalish_hint_score: u64,
    probe_attempts: u64,
    probe_resolved_delta_total: u64,
    probe_best_resolved_delta: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct PendingProbeCandidate {
    rule_name: String,
    max_target_priority: u64,
    blocked_remaining_successes: u64,
    branch_target_count: usize,
    literalish_hint_score: u64,
    probe_attempts: u64,
    probe_resolved_delta_total: u64,
    probe_best_resolved_delta: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct TargetProbeHistory {
    attempts: u64,
    successful_generations: u64,
    resolved_delta_total: u64,
    best_resolved_delta: u64,
}

impl TargetDriveSummary {
    pub fn summary_line(&self) -> String {
        format!(
            "Target-driven generation: resolved {}/{} targets in {} attempts (generation_successes={}, generation_errors={}, depth_exceeded_errors={}, rule_visit_limit_errors={}, target_timeout_errors={}, helper_timeout_errors={}, reach_plan_activations={})",
            self.resolved_targets,
            self.total_targets,
            self.attempts,
            self.generation_successes,
            self.generation_errors,
            self.depth_exceeded_errors,
            self.rule_visit_limit_errors,
            self.target_timeout_errors,
            self.helper_timeout_errors,
            self.reach_plan_activations
        )
    }
}

#[derive(Debug, Clone, Default)]
struct ActiveTargetPlan {
    rule_thresholds: HashMap<String, u64>,
    branch_thresholds: HashMap<String, HashMap<usize, u64>>,
}

/// SV-EXH-PROOF.7.2.2 (PGEN-SV-EXH-PROOF-0116): an installed target-reach plan —
/// the `.7.2.1` forced-directive chain (`compute_reach_path`) plus a per-target
/// budget that bounds how aggressively the on-path candidate-set bypass is
/// applied. Consumed by `generate_or`:
///   * `directive_for_site(rule, node_path)` → if this OR site is on the reach
///     path, return the branch index to force (and whether the bypass applies);
///   * the bypass keeps an over-budget deep target branch in the candidate set
///     ON-PATH only (off-path OR nodes keep normal depth-floor pruning, so
///     generation stays terminating everywhere else).
/// GENERAL/parser-agnostic: directives are keyed on (rule_name, node_path), never
/// on rule semantics, per [[feedback_ast_pipeline_parser_agnostic]].
#[derive(Debug, Clone)]
struct ActiveReachPlan {
    /// (rule_name, node_path) -> forced branch index, for every OR site on the
    /// path from entry down to the target (target site included).
    directives: HashMap<(String, String), usize>,
    /// Remaining reach-budget "fuel": each on-path candidate-set bypass spends
    /// one unit; when exhausted, the bypass stops (normal pruning resumes) so a
    /// pathological/cyclic path cannot force unbounded depth. 0 disables the
    /// bypass while still allowing pure branch-forcing of in-budget sites.
    bypass_fuel: u32,
    /// SV-EXH-PROOF.7.2.3 (PGEN-SV-EXH-PROOF-0117): the plan's own target identity
    /// — (target_group_key, target_branch_index) — so the outcome can be read back
    /// honestly after generation from the coverage metrics the OR-decision already
    /// records at that site. `target_group_key` is `branch_group_key(rule,
    /// node_path)` of the final directive.
    target_group_key: String,
    target_branch_index: usize,
    /// SV-EXH-PROOF.7.2.3: target metrics captured when the plan was installed, so
    /// the outcome reflects only THIS plan's effect (deltas), not pre-existing
    /// coverage. Filled by `set_reach_plan` after `from_directives`.
    baseline_selected_hits: u64,
    baseline_success_hits: u64,
}

/// SV-EXH-PROOF.7.2.3 (PGEN-SV-EXH-PROOF-0117): honest result of attempting a
/// target-reach plan during one generation, derived from the existing coverage
/// metrics recorded at the target OR site (no new hot-path instrumentation):
///   * `Reached`            — the target branch was selected AND produced a valid
///                            sample (success_hits increased): genuine coverage.
///   * `SelectedButFailed`  — the target branch was selected but its own subtree
///                            failed (e.g. a predicate/required rule could not be
///                            satisfied in this reach context): the target is
///                            unsatisfiable HERE and must be REPORTED, not retried
///                            forever.
///   * `NotReached`         — the target branch was never selected (the plan could
///                            not steer to it this run; e.g. an upstream off-path
///                            constraint blocked the path).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ReachOutcome {
    Reached,
    SelectedButFailed,
    NotReached,
}

impl ActiveReachPlan {
    /// Build from a `.7.2.1` directive chain. Later directives for the same site
    /// win (the target's own directive is appended last in `compute_reach_path`).
    /// The LAST directive is the target itself, so its (group_key, branch) is the
    /// plan's target identity (`.7.2.3`).
    fn from_directives(chain: &[ReachDirective], bypass_fuel: u32) -> Self {
        let mut directives = HashMap::new();
        for directive in chain {
            directives.insert(
                (directive.rule_name.clone(), directive.node_path.clone()),
                directive.branch_index,
            );
        }
        let (target_group_key, target_branch_index) = chain
            .last()
            .map(|d| {
                (
                    StimuliGenerator::branch_group_key(&d.rule_name, &d.node_path),
                    d.branch_index,
                )
            })
            .unwrap_or_else(|| (String::new(), 0));
        Self {
            directives,
            bypass_fuel,
            target_group_key,
            target_branch_index,
            baseline_selected_hits: 0,
            baseline_success_hits: 0,
        }
    }

    fn forced_branch_for(&self, rule_name: &str, node_path: &str) -> Option<usize> {
        self.directives
            .get(&(rule_name.to_string(), node_path.to_string()))
            .copied()
    }
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

#[derive(Debug, Clone)]
enum GrammarMutationCandidateKind {
    Or { alternative_branches: Vec<usize> },
    Quantifier { alternative_repeats: Vec<usize> },
}

#[derive(Debug, Clone)]
struct GrammarMutationCandidate {
    site_key: String,
    kind: GrammarMutationCandidateKind,
}

#[derive(Debug, Clone, Default)]
struct StimuliDecisionTrace {
    or_choices: HashMap<String, usize>,
    quantifier_repeats: HashMap<String, usize>,
    mutation_candidates: Vec<GrammarMutationCandidate>,
}

#[derive(Debug, Clone)]
enum GrammarMutationSelection {
    Or {
        site_key: String,
        forced_branch: usize,
    },
    Quantifier {
        site_key: String,
        forced_repeats: usize,
    },
}

#[derive(Debug, Clone)]
struct ActiveGrammarMutationReplay {
    baseline_trace: StimuliDecisionTrace,
    selection: GrammarMutationSelection,
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
    /// SV-EXH-PROOF.7.2.2 (PGEN-SV-EXH-PROOF-0116): when `Some`, an active
    /// target-reach plan steers the OR-decision toward one coverage target along
    /// a precomputed forced-directive chain (see `ActiveReachPlan`). Default
    /// `None` → the OR-decision behaves exactly as before (zero behavior change
    /// on the production path until a plan is installed via `set_reach_plan`).
    reach_plan: Option<ActiveReachPlan>,
    /// SV-EXH-PROOF.7.2.4 (PGEN-SV-EXH-PROOF-0118): count of times the driver's
    /// reach-plan steering hook installed a plan during a target-driven run.
    /// Observability for the driver + lets tests assert the hook actually fired
    /// (vs the baseline weighting having already covered the target).
    reach_plan_activations: u64,
    /// SV-EXH-PROOF.7.4.4 (PGEN-SV-EXH-PROOF-0143): true ONLY while the appended
    /// minimal-witness pass (`generate_target_witnesses`) is running. When set, the
    /// shared `generate_or` core prefers the shortest-terminating alternative (Purdom
    /// 1972 shortest-derivation) so a witness on a deeply-factored rule converges
    /// before the per-witness budget instead of timing out. The diverse background
    /// pass never sets this flag → its output is byte-identical → the witness pass
    /// stays monotone-additive (`replay_target_count` can only shrink).
    witness_mode: bool,
    /// SV-EXH-PROOF.7.4.4: the `.7.4.2` Purdom min-terminal-length table, computed
    /// once at witness-pass start and consulted by `generate_or` while `witness_mode`
    /// is on. `None` outside the witness pass.
    witness_min_terminal_lengths: Option<HashMap<String, usize>>,
    /// SV-EXH-PROOF.7.4.6.3: derivation-CONSTRUCTION sub-mode (implies `witness_mode`). When
    /// true, `generate_or` commits to the single shortest (min-length) branch with NO
    /// backtracking, and `generate_quantified` emits the MINIMUM repetition count — so a
    /// witness is built as the minimal derivation tree in O(tree-size), no search/timeout.
    /// The caller (`generate_target_witnesses`) tries this first and falls back to search.
    construct_mode: bool,
    /// STIMULI-SIGNOFF.2.2 (PGEN-STIMULI-SIGNOFF-0003): k-path coverage NUMERATOR recorder.
    /// `None` = OFF (default → zero overhead, generation byte-identical → monotone). When
    /// `Some((k, set))`, every `generate_rule` entry records the last-k window of the live
    /// rule call-stack as a covered k-path. Read-only instrumentation (never changes a
    /// generation decision). Pairs with `compute_k_paths(k)` (the universe/denominator).
    k_path_recording: Option<(usize, HashSet<Vec<String>>)>,
    target_probe_history: HashMap<String, TargetProbeHistory>,
    target_drive_validation_active: bool,
    active_generation_entry_rule: Option<String>,
    deterministic_partition_counters: HashMap<String, u64>,
    mutation_trace: Option<StimuliDecisionTrace>,
    mutation_replay: Option<ActiveGrammarMutationReplay>,
    mutation_site_visit_counters: HashMap<String, u64>,
    active_generation_deadline: Option<ActiveGenerationDeadline>,
    // GRAMMAR-WELLFORMED.B1: monotonic generation-step counter (interior-mutable so the
    // &self deadline check can bump it). Drives the DETERMINISTIC step budget that replaced
    // the wall-clock deadline → a seeded run yields the same residual every time.
    generation_step_counter: std::cell::Cell<u64>,
    // GRAMMAR-WELLFORMED.G.3.2: reachability WITNESS certificates captured by the last
    // `generate_target_witnesses` pass — one `ReachabilityWitness { fragment, input }` per resolved
    // reachable target (its id + the witness input). The constructive half of the certifying linter
    // (the dual of the unreachability PROOF); read via `witness_certificates()`.
    witness_certificates: Vec<super::grammar_wellformedness::ReachabilityWitness>,
    // SV-EXH-PROOF.2.3.2 (P-a): per-`generate_regex_sample` set of
    // chars to exclude from ALL regex-class materialization, derived
    // from a permissive *leading* negated class in the rule's own
    // pattern (e.g. `[^\`\r\n]…` ⇒ {`` ` ``}). The grammar author's
    // anchored leading negation declares those chars structurally
    // hazardous for that content; the closed-loop must not emit them
    // anywhere in the run, else the parser re-lexes them as structure
    // and the round-trip fails. Empty except during a qualifying
    // `generate_regex_sample` call. Parser-agnostic, all-lanes-safe.
    regex_content_forbidden: HashSet<char>,
    // SV-EXH-PROOF.2.3.2: grammar-scoped structural-sigil set `G`
    // (union of every permissive leading-negated content class's
    // printable complement across the whole grammar). Derived once
    // from the grammar's own author-written leading negations;
    // parser/EBNF-agnostic. `None` until first computed.
    grammar_content_sigils: Option<HashSet<char>>,
    // SV-EXH-PROOF.2.3.2 (Mode B): dynamic stack of *required
    // fixed-literal structural-closer* lexemes for every
    // closer-bearing recursive/quantified construct currently OPEN
    // (its body being generated). Pushed before generating the
    // pre-closer elements of a `… item* CLOSE`-shaped sequence,
    // popped before the CLOSE element itself. A *free* terminal
    // (variable-HIR regex: class/repetition/alternation) materialized
    // anywhere in that open body must not produce a string containing
    // an active closer lexeme — else the parser consumes it
    // (first-match) as body content and the construct never closes
    // (generated sample fails its own round-trip). Fixed-literal
    // terminals are exempt (so a legitimately-nested same-construct's
    // own structural CLOSE is unaffected ⇒ nesting-safe); empty when
    // no closer-bearing construct is open (so the lexeme is still
    // freely generatable standalone ⇒ coverage-preserving). Derived
    // purely from grammar structure + terminal HIR ⇒
    // parser/EBNF-agnostic, all-lanes-safe.
    structural_closer_forbidden: Vec<String>,
    // SV-EXH-PROOF.2.3.2: always-on observability of the round-trip
    // stability guard — how many times a closer-bearing construct's
    // body scope was entered (a `… item* CLOSE` push fired), and how
    // many free-terminal candidates were discarded for colliding with
    // an active closer lexeme. Cheap counters; expose generation-path
    // ground truth (decisively separates "generator never builds the
    // construct" from "construct built, guard engaged"). Parser/EBNF
    // -agnostic.
    closer_scopes_entered: usize,
    free_terminal_closer_discards: usize,
}

impl<'a> StimuliGenerator<'a> {
    fn target_drive_progress_interval(max_attempts: usize) -> usize {
        match max_attempts {
            0..=64 => 8,
            65..=256 => 16,
            257..=1024 => 32,
            1025..=4096 => 64,
            _ => 128,
        }
    }

    fn should_trace_target_drive_progress(attempt: usize, max_attempts: usize) -> bool {
        let effective_max = max_attempts.max(1);
        attempt == 1
            || attempt == effective_max
            || attempt % Self::target_drive_progress_interval(effective_max) == 0
    }

    pub fn new(
        grammar_name: String,
        grammar_tree: &'a HashMap<String, ASTNode>,
        rule_order: &'a [String],
        annotations: Option<&'a Annotations>,
        config: StimuliConfig,
    ) -> Self {
        // SV-EXH-PROOF.7.4.6.2: the thread-local nullability cache is keyed by rule name, which
        // is grammar-specific — clear it whenever the active grammar changes so a different
        // grammar's rule of the same name can never return a stale cached value.
        NULLABLE_CACHE_GRAMMAR.with(|g| {
            let mut g = g.borrow_mut();
            if g.as_deref() != Some(grammar_name.as_str()) {
                NULLABLE_CACHE.with(|c| c.borrow_mut().clear());
                *g = Some(grammar_name.clone());
            }
        });
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
            reach_plan: None,
            reach_plan_activations: 0,
            witness_mode: false,
            witness_min_terminal_lengths: None,
            construct_mode: false,
            k_path_recording: None,
            target_probe_history: HashMap::new(),
            target_drive_validation_active: false,
            active_generation_entry_rule: None,
            deterministic_partition_counters: HashMap::new(),
            mutation_trace: None,
            mutation_replay: None,
            mutation_site_visit_counters: HashMap::new(),
            active_generation_deadline: None,
            generation_step_counter: std::cell::Cell::new(0),
            witness_certificates: Vec::new(),
            regex_content_forbidden: HashSet::new(),
            grammar_content_sigils: None,
            structural_closer_forbidden: Vec::new(),
            closer_scopes_entered: 0,
            free_terminal_closer_discards: 0,
        }
    }

    /// SV-EXH-PROOF.2.3.2 observability: how many closer-bearing
    /// construct bodies were entered (a `… item* CLOSE` push fired).
    /// `0` over many samples ⇒ the generator never builds that
    /// construct (the structural sigil must be arriving via free
    /// text the parser re-lexes).
    pub fn closer_scopes_entered(&self) -> usize {
        self.closer_scopes_entered
    }

    /// SV-EXH-PROOF.2.3.2 observability: free-terminal candidates
    /// discarded for colliding with an active closer lexeme.
    pub fn free_terminal_closer_discards(&self) -> usize {
        self.free_terminal_closer_discards
    }

    /// SV-EXH-PROOF.2.3.2 (Mode B, hint route): a literal/probe hint
    /// in `generate_rule`/`generate_or` returns its string DIRECTLY,
    /// bypassing `generate_atom`'s structural-closer consult-point.
    /// Under target-drive steering those hint routes are active (they
    /// are inert under Baseline), so a hint emitted while a
    /// closer-bearing construct is open can re-introduce the absorbed
    /// -closer round-trip break the terminal consult prevents. Same
    /// agnostic round-trip-stability rule, applied to the hint route:
    /// a hint that *contains* an active closer lexeme must be skipped
    /// (fall through to normal — guarded — generation). Empty stack
    /// ⇒ inert (coverage-preserving). Zero grammar identifiers.
    fn hint_collides_with_active_closer(&self, hint: &str) -> bool {
        !self.structural_closer_forbidden.is_empty()
            && self
                .structural_closer_forbidden
                .iter()
                .any(|c| hint.contains(c.as_str()))
    }

    /// SV-EXH-PROOF.2.3.2 (Mode B, hazard gate — decisive fix): a
    /// `… item* CLOSE` closer is a genuine round-trip hazard for free
    /// content ONLY when its lexeme begins with a **grammar-declared
    /// structural sigil** — a char some content rule's author
    /// leading-negated (P-a's `grammar_content_sigils`, e.g. `` ` ``
    /// for `` `endif ``). Such a char forces a NEW token on re-lex, so
    /// free text containing it IS mis-lexed as structure. A closer
    /// whose lexeme is ordinary punctuation (e.g. `)` closing
    /// `macro_formals`) is NOT a hazard: a `)` inside a `/*…*/`
    /// comment / string / identifier is absorbed by that token, never
    /// re-lexed as the closer — so the substring guard there is
    /// spuriously over-broad and corrupts generation. Gating on the
    /// grammar's own leading-negation set keeps this parser/EBNF
    /// -agnostic and surgical (zero grammar identifiers).
    fn closer_lexeme_is_structural_hazard(&mut self, lexeme: &str) -> bool {
        let Some(first) = lexeme.chars().next() else {
            return false;
        };
        self.grammar_content_sigils().contains(&first)
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

    fn timeout_budget_from_ms(
        timeout_ms: u64,
        error_prefix: &'static str,
    ) -> Option<GenerationTimeoutBudget> {
        if timeout_ms == 0 {
            None
        } else {
            Some(GenerationTimeoutBudget {
                step_budget: timeout_ms.saturating_mul(generation_steps_per_ms()),
                error_prefix,
                budget_ms: timeout_ms,
            })
        }
    }

    fn generation_deadline_exceeded(&self) -> bool {
        // GRAMMAR-WELLFORMED.B1: count one generation STEP per deadline-check (called at every
        // rule/node generation chokepoint via enforce_generation_deadline) and compare to the
        // step deadline. Deterministic — no wall clock; only the seeded generation drives the
        // counter, so a given (seed, grammar, budget) yields the SAME residual every run.
        let step = self.generation_step_counter.get().saturating_add(1);
        self.generation_step_counter.set(step);
        self.active_generation_deadline
            .map(|deadline| step >= deadline.step_deadline)
            .unwrap_or(false)
    }

    fn is_helper_timeout_error(error: &anyhow::Error) -> bool {
        Self::is_timeout_error_with_prefix(error, HELPER_TIMEOUT_ERROR_PREFIX)
    }

    fn is_target_timeout_error(error: &anyhow::Error) -> bool {
        Self::is_timeout_error_with_prefix(error, TARGET_TIMEOUT_ERROR_PREFIX)
    }

    /// DIAG-SEVERITY.3: a generation failure caused by hitting the recursion depth
    /// limit (`max_depth`). Distinct from timeouts; counted separately so the dominant
    /// SV-residual cause is surfaced, not masked in a generic error tally.
    fn is_depth_exceeded_error(error: &anyhow::Error) -> bool {
        Self::is_timeout_error_with_prefix(error, DEPTH_EXCEEDED_ERROR_PREFIX)
    }

    /// DIAG-SEVERITY.3.1: a generation failure caused by hitting the per-rule visit
    /// limit (`max_rule_visits`) — the second structural-budget reason.
    fn is_rule_visit_limit_error(error: &anyhow::Error) -> bool {
        Self::is_timeout_error_with_prefix(error, RULE_VISIT_LIMIT_ERROR_PREFIX)
    }

    /// DIAG-SEVERITY.3.1: THE canonical classifier — the single place that maps a
    /// generation failure to its `GenerationErrorReason`. All reason tallies derive from
    /// this, so the classification cannot drift across call sites. Priority order is
    /// irrelevant for correctness (the prefixes are mutually exclusive) but fixed for
    /// determinism. Anything unrecognised (incl. expected PEG backtracking and rare
    /// quantifier/weight/semantic config errors) is `Other`.
    fn classify_generation_error(error: &anyhow::Error) -> GenerationErrorReason {
        if Self::is_depth_exceeded_error(error) {
            GenerationErrorReason::DepthExceeded
        } else if Self::is_rule_visit_limit_error(error) {
            GenerationErrorReason::RuleVisitLimit
        } else if Self::is_target_timeout_error(error) {
            GenerationErrorReason::TargetTimeout
        } else if Self::is_helper_timeout_error(error) {
            GenerationErrorReason::HelperTimeout
        } else {
            GenerationErrorReason::Other
        }
    }

    fn is_timeout_error_with_prefix(error: &anyhow::Error, prefix: &str) -> bool {
        error
            .chain()
            .any(|cause| cause.to_string().starts_with(prefix))
    }

    fn active_target_generation_timeout(&self) -> Option<GenerationTimeoutBudget> {
        Self::timeout_budget_from_ms(
            self.config.target_generation_timeout_ms,
            TARGET_TIMEOUT_ERROR_PREFIX,
        )
    }

    fn active_helper_generation_timeout(&self) -> Option<GenerationTimeoutBudget> {
        Self::timeout_budget_from_ms(
            self.config.target_helper_generation_timeout_ms,
            HELPER_TIMEOUT_ERROR_PREFIX,
        )
    }

    /// SV-EXH-PROOF.7.4.5 (PGEN-SV-EXH-PROOF-0145): the per-witness budget. Decoupled from
    /// the PRIMARY target-drive budget (which is a pathological-attempt guard — the SV gate
    /// sets it to 5 ms): the witness pass runs ONE deliberate, count-bounded witness per
    /// uncovered target and is monotone, so a slow-but-resolvable witness deserves more time.
    /// Budget = `max(primary, floor)` (floor default `WITNESS_TIMEOUT_FLOOR_MS`, overridable
    /// via `PGEN_WITNESS_TIMEOUT_FLOOR_MS`): NEVER below the primary (so callers that already
    /// gave witnesses a generous budget — e.g. the .7.4.4 A/B's 7000 ms — are unchanged), and
    /// unbounded primary (0) stays unbounded. Classified as `TargetTimeout` (same prefix).
    fn witness_generation_timeout(&self) -> Option<GenerationTimeoutBudget> {
        let primary_ms = self.config.target_generation_timeout_ms;
        if primary_ms == 0 {
            return None; // unbounded primary → witnesses stay unbounded (prior behavior)
        }
        let floor_ms = std::env::var("PGEN_WITNESS_TIMEOUT_FLOOR_MS")
            .ok()
            .and_then(|raw| raw.trim().parse::<u64>().ok())
            .unwrap_or(WITNESS_TIMEOUT_FLOOR_MS);
        Self::timeout_budget_from_ms(primary_ms.max(floor_ms), TARGET_TIMEOUT_ERROR_PREFIX)
    }

    fn target_drive_generation_timeout(
        &self,
        helper_probe_active: bool,
    ) -> Option<GenerationTimeoutBudget> {
        if helper_probe_active {
            self.active_helper_generation_timeout()
        } else {
            self.active_target_generation_timeout()
        }
    }

    fn enforce_generation_deadline(&self, current_rule: &str, node_path: &str) -> Result<()> {
        if self.generation_deadline_exceeded() {
            let deadline = self
                .active_generation_deadline
                .expect("generation deadline state should exist when exceeded");
            return Err(anyhow!(
                "{} for rule '{}' at path '{}' (budget={}ms)",
                deadline.error_prefix,
                current_rule,
                node_path,
                deadline.budget_ms
            ));
        }
        Ok(())
    }

    fn generate_from_entry_with_optional_timeout(
        &mut self,
        entry_rule: &str,
        timeout_budget: Option<GenerationTimeoutBudget>,
    ) -> Result<String> {
        let previous_deadline = self.active_generation_deadline;
        if let Some(timeout_budget) = timeout_budget {
            self.active_generation_deadline = Some(ActiveGenerationDeadline {
                step_deadline: self
                    .generation_step_counter
                    .get()
                    .saturating_add(timeout_budget.step_budget),
                error_prefix: timeout_budget.error_prefix,
                budget_ms: timeout_budget.budget_ms,
            });
        }
        let result = self.generate_from_entry(entry_rule);
        self.active_generation_deadline = previous_deadline;
        result
    }

    pub fn merge_coverage_metrics(&mut self, other: &StimuliCoverageMetrics) -> Result<()> {
        self.coverage.merge_from(other)
    }

    fn trace_target_drive_progress(
        &self,
        resolved_entry: &str,
        generation_entry: &str,
        total_targets: usize,
        pending_targets: usize,
        attempts: usize,
        max_attempts: usize,
        generation_successes: usize,
        generation_errors: usize,
        target_timeout_errors: usize,
        helper_timeout_errors: usize,
        stagnant_iterations: usize,
        probe_threshold: usize,
        validation_summary: Option<&TargetDriveValidationSummary>,
    ) {
        let resolved_targets = total_targets.saturating_sub(pending_targets);
        if let Some(validation) = validation_summary {
            self.trace(
                TraceLevel::Low,
                format_args!(
                    "Target-drive progress: entry='{}' generation_entry='{}' attempts={}/{} resolved={}/{} pending={} stagnant={} probe_threshold={} generation_successes={} generation_errors={} target_timeout_errors={} helper_timeout_errors={} accepted_outputs={} rejected_outputs={} alternate_attempts={} alternate_accepted={} alternate_rejected={}",
                    resolved_entry,
                    generation_entry,
                    attempts,
                    max_attempts,
                    resolved_targets,
                    total_targets,
                    pending_targets,
                    stagnant_iterations,
                    probe_threshold,
                    generation_successes,
                    generation_errors,
                    target_timeout_errors,
                    helper_timeout_errors,
                    validation.accepted_outputs,
                    validation.rejected_outputs,
                    validation.alternate_entry_attempts,
                    validation.alternate_entry_accepted_outputs,
                    validation.alternate_entry_rejected_outputs,
                ),
            );
        } else {
            self.trace(
                TraceLevel::Low,
                format_args!(
                    "Target-drive progress: entry='{}' generation_entry='{}' attempts={}/{} resolved={}/{} pending={} stagnant={} probe_threshold={} generation_successes={} generation_errors={} target_timeout_errors={} helper_timeout_errors={}",
                    resolved_entry,
                    generation_entry,
                    attempts,
                    max_attempts,
                    resolved_targets,
                    total_targets,
                    pending_targets,
                    stagnant_iterations,
                    probe_threshold,
                    generation_successes,
                    generation_errors,
                    target_timeout_errors,
                    helper_timeout_errors,
                ),
            );
        }
    }

    fn trace_target_probe_activation(
        &self,
        resolved_entry: &str,
        generation_entry: &str,
        pending: &[TargetCoverageStatus],
        stagnant_iterations: usize,
        probe_threshold: usize,
        validation_summary: Option<&TargetDriveValidationSummary>,
    ) {
        let pending_rule_targets = pending
            .iter()
            .filter(|status| status.rule_name == generation_entry)
            .count();
        let blocked_dependency_targets = pending
            .iter()
            .filter(|status| {
                status
                    .depends_on
                    .iter()
                    .any(|rule_name| rule_name == generation_entry)
            })
            .count();
        let blocked_remaining_successes = pending
            .iter()
            .filter(|status| {
                status
                    .depends_on
                    .iter()
                    .any(|rule_name| rule_name == generation_entry)
            })
            .map(|status| status.remaining_successes)
            .sum::<u64>();
        let generation_entry_deficit = self.rule_target_deficit(generation_entry);
        let generation_entry_successes = self
            .coverage
            .rule_success_hits
            .get(generation_entry)
            .copied()
            .unwrap_or(0);

        if let Some(validation) = validation_summary {
            self.trace(
                TraceLevel::Low,
                format_args!(
                    "Target-drive helper probe: entry='{}' generation_entry='{}' stagnant={} probe_threshold={} pending_rule_targets={} blocked_dependency_targets={} blocked_remaining_successes={} generation_entry_deficit={} generation_entry_successes={} accepted_outputs={} rejected_outputs={} alternate_attempts={} alternate_accepted={} alternate_rejected={}",
                    resolved_entry,
                    generation_entry,
                    stagnant_iterations,
                    probe_threshold,
                    pending_rule_targets,
                    blocked_dependency_targets,
                    blocked_remaining_successes,
                    generation_entry_deficit,
                    generation_entry_successes,
                    validation.accepted_outputs,
                    validation.rejected_outputs,
                    validation.alternate_entry_attempts,
                    validation.alternate_entry_accepted_outputs,
                    validation.alternate_entry_rejected_outputs,
                ),
            );
        } else {
            self.trace(
                TraceLevel::Low,
                format_args!(
                    "Target-drive helper probe: entry='{}' generation_entry='{}' stagnant={} probe_threshold={} pending_rule_targets={} blocked_dependency_targets={} blocked_remaining_successes={} generation_entry_deficit={} generation_entry_successes={}",
                    resolved_entry,
                    generation_entry,
                    stagnant_iterations,
                    probe_threshold,
                    pending_rule_targets,
                    blocked_dependency_targets,
                    blocked_remaining_successes,
                    generation_entry_deficit,
                    generation_entry_successes,
                ),
            );
        }
    }

    fn trace_target_probe_ranking(
        &self,
        resolved_entry: &str,
        generation_entry: &str,
        pending: &[TargetCoverageStatus],
        stagnant_iterations: usize,
        probe_threshold: usize,
        validation_summary: Option<&TargetDriveValidationSummary>,
    ) {
        let dependency_candidate = self.best_dependency_probe_candidate(pending, resolved_entry);
        let pending_candidate = self.best_pending_probe_candidate(pending, resolved_entry);
        let pending_frontier_unlock_threshold =
            self.pending_frontier_unlock_threshold(probe_threshold);
        let pending_frontier_unlocked = stagnant_iterations >= pending_frontier_unlock_threshold;
        let selected_pool = if dependency_candidate
            .as_ref()
            .map(|candidate| candidate.rule_name.as_str())
            == Some(generation_entry)
        {
            "dependency"
        } else if pending_candidate
            .as_ref()
            .map(|candidate| candidate.rule_name.as_str())
            == Some(generation_entry)
        {
            "pending"
        } else {
            "unknown"
        };

        let dependency_summary = dependency_candidate
            .as_ref()
            .map(Self::format_dependency_probe_candidate)
            .unwrap_or_else(|| "none".to_string());
        let pending_summary = pending_candidate
            .as_ref()
            .map(Self::format_pending_probe_candidate)
            .unwrap_or_else(|| "none".to_string());

        if let Some(validation) = validation_summary {
            self.trace(
                TraceLevel::Low,
                format_args!(
                    "Target-drive helper ranking: entry='{}' generation_entry='{}' selected_pool={} pending_frontier_unlocked={} pending_frontier_unlock_threshold={} pending_frontier_extra_stagnation={} target_timeout_ms={} helper_timeout_ms={} dependency_top={} pending_top={} accepted_outputs={} rejected_outputs={} alternate_attempts={} alternate_accepted={} alternate_rejected={}",
                    resolved_entry,
                    generation_entry,
                    selected_pool,
                    pending_frontier_unlocked,
                    pending_frontier_unlock_threshold,
                    self.config.target_pending_frontier_extra_stagnation,
                    self.config.target_generation_timeout_ms,
                    self.config.target_helper_generation_timeout_ms,
                    dependency_summary,
                    pending_summary,
                    validation.accepted_outputs,
                    validation.rejected_outputs,
                    validation.alternate_entry_attempts,
                    validation.alternate_entry_accepted_outputs,
                    validation.alternate_entry_rejected_outputs,
                ),
            );
        } else {
            self.trace(
                TraceLevel::Low,
                format_args!(
                    "Target-drive helper ranking: entry='{}' generation_entry='{}' selected_pool={} pending_frontier_unlocked={} pending_frontier_unlock_threshold={} pending_frontier_extra_stagnation={} target_timeout_ms={} helper_timeout_ms={} dependency_top={} pending_top={}",
                    resolved_entry,
                    generation_entry,
                    selected_pool,
                    pending_frontier_unlocked,
                    pending_frontier_unlock_threshold,
                    self.config.target_pending_frontier_extra_stagnation,
                    self.config.target_generation_timeout_ms,
                    self.config.target_helper_generation_timeout_ms,
                    dependency_summary,
                    pending_summary,
                ),
            );
        }
    }

    fn trace_target_probe_result(
        &self,
        resolved_entry: &str,
        generation_entry: &str,
        pending_before: usize,
        pending_after: usize,
        generation_succeeded: bool,
        validation_summary: Option<&TargetDriveValidationSummary>,
    ) {
        let resolved_delta = pending_before.saturating_sub(pending_after);
        if let Some(validation) = validation_summary {
            self.trace(
                TraceLevel::Low,
                format_args!(
                    "Target-drive helper result: entry='{}' generation_entry='{}' pending_before={} pending_after={} resolved_delta={} generation_succeeded={} accepted_outputs={} rejected_outputs={} alternate_attempts={} alternate_accepted={} alternate_rejected={}",
                    resolved_entry,
                    generation_entry,
                    pending_before,
                    pending_after,
                    resolved_delta,
                    generation_succeeded,
                    validation.accepted_outputs,
                    validation.rejected_outputs,
                    validation.alternate_entry_attempts,
                    validation.alternate_entry_accepted_outputs,
                    validation.alternate_entry_rejected_outputs,
                ),
            );
        } else {
            self.trace(
                TraceLevel::Low,
                format_args!(
                    "Target-drive helper result: entry='{}' generation_entry='{}' pending_before={} pending_after={} resolved_delta={} generation_succeeded={}",
                    resolved_entry,
                    generation_entry,
                    pending_before,
                    pending_after,
                    resolved_delta,
                    generation_succeeded,
                ),
            );
        }
    }

    fn record_target_probe_result(
        &mut self,
        generation_entry: &str,
        pending_before: usize,
        pending_after: usize,
        generation_succeeded: bool,
    ) {
        let resolved_delta =
            u64::try_from(pending_before.saturating_sub(pending_after)).unwrap_or(u64::MAX);
        let history = self
            .target_probe_history
            .entry(generation_entry.to_string())
            .or_default();
        history.attempts = history.attempts.saturating_add(1);
        if generation_succeeded {
            history.successful_generations = history.successful_generations.saturating_add(1);
        }
        history.resolved_delta_total = history.resolved_delta_total.saturating_add(resolved_delta);
        history.best_resolved_delta = history.best_resolved_delta.max(resolved_delta);
    }

    fn format_dependency_probe_candidate(candidate: &DependencyProbeCandidate) -> String {
        format!(
            "{}(deficit={},blocked_remaining={},blocked_targets={},yield_score={},best_delta={},total_delta={},attempts={},literalish={})",
            candidate.rule_name,
            candidate.dependency_rule_deficit,
            candidate.blocked_remaining_successes,
            candidate.blocked_targets,
            Self::probe_yield_score(
                candidate.probe_resolved_delta_total,
                candidate.probe_attempts,
            ),
            candidate.probe_best_resolved_delta,
            candidate.probe_resolved_delta_total,
            candidate.probe_attempts,
            candidate.literalish_hint_score,
        )
    }

    fn format_pending_probe_candidate(candidate: &PendingProbeCandidate) -> String {
        format!(
            "{}(branch_targets={},blocked_remaining={},yield_score={},best_delta={},total_delta={},attempts={},literalish={})",
            candidate.rule_name,
            candidate.branch_target_count,
            candidate.blocked_remaining_successes,
            Self::probe_yield_score(
                candidate.probe_resolved_delta_total,
                candidate.probe_attempts,
            ),
            candidate.probe_best_resolved_delta,
            candidate.probe_resolved_delta_total,
            candidate.probe_attempts,
            candidate.literalish_hint_score,
        )
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
                    reach_classification: None, // SV-EXH-PROOF.7.2.16: filled by the post-sort classification pass
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
            let branch_nodes =
                self.or_alternatives_for_group_path(&group.rule_name, &group.node_path);

            for branch_idx in 0..group.total_branches {
                let selected_hits = group.selected_counts.get(branch_idx).copied().unwrap_or(0);
                let success_hits = group.success_counts.get(branch_idx).copied().unwrap_or(0);
                let deficit = threshold.saturating_sub(success_hits);

                if success_hits > 0 {
                    covered_branches = covered_branches.saturating_add(1);
                }
                if deficit == 0 {
                    continue;
                }

                let mut rule_refs = Vec::new();
                let mut uncovered_rule_refs = Vec::new();
                let mut missing_rule_refs = Vec::new();
                if let Some(alternatives) = branch_nodes {
                    if let Some(branch_node) = alternatives.get(branch_idx) {
                        let mut refs = HashSet::new();
                        self.collect_rule_references(branch_node, &mut refs);
                        rule_refs = refs.into_iter().collect();
                        rule_refs.sort();
                        missing_rule_refs = self.missing_rule_references(branch_node);
                        uncovered_rule_refs = rule_refs
                            .iter()
                            .filter(|rule_name| {
                                !missing_rule_refs
                                    .iter()
                                    .any(|missing| missing == *rule_name)
                                    && self
                                        .coverage
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

                let branch_reachable = reachable && missing_rule_refs.is_empty();
                if branch_reachable {
                    reachable_branches = reachable_branches.saturating_add(1);
                }
                if success_hits > 0 && branch_reachable {
                    covered_reachable_branches = covered_reachable_branches.saturating_add(1);
                }
                if branch_reachable && success_hits >= threshold {
                    reachable_branches_at_threshold =
                        reachable_branches_at_threshold.saturating_add(1);
                }

                let reason = if branch_reachable {
                    if selected_hits == 0 {
                        "never_selected"
                    } else if success_hits == 0 {
                        "selected_but_failed"
                    } else {
                        "below_threshold"
                    }
                } else if !missing_rule_refs.is_empty() {
                    "references_rule_missing_from_active_grammar"
                } else {
                    "unreachable_from_entry"
                };

                let mut priority_score = 0u64;
                if branch_reachable {
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
                let top_failure_reasons = group
                    .failure_reasons
                    .get(branch_idx)
                    .map(|reasons| {
                        let mut ranked: Vec<BranchFailureReasonCount> = reasons
                            .iter()
                            .map(|(reason, count)| BranchFailureReasonCount {
                                reason: reason.clone(),
                                count: *count,
                            })
                            .collect();
                        ranked.sort_by(|left, right| {
                            right
                                .count
                                .cmp(&left.count)
                                .then_with(|| left.reason.cmp(&right.reason))
                        });
                        ranked.truncate(3);
                        ranked
                    })
                    .unwrap_or_default();
                let debt = BranchCoverageDebt {
                    branch_id: branch_id.clone(),
                    group_key: group_key.clone(),
                    rule_name: group.rule_name.clone(),
                    node_path: group.node_path.clone(),
                    branch_index: branch_idx,
                    reachable: branch_reachable,
                    selected_hits,
                    success_hits,
                    required_successes: threshold,
                    deficit,
                    priority_score,
                    reason: reason.to_string(),
                    rule_references: rule_refs.clone(),
                    uncovered_rule_references: uncovered_rule_refs.clone(),
                    top_failure_reasons,
                };

                if branch_reachable {
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
                        reach_classification: None, // SV-EXH-PROOF.7.2.16: filled by the post-sort classification pass
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

        // SV-EXH-PROOF.7.2.16 (PGEN-SV-EXH-PROOF-0131): READ-ONLY reach
        // classification of each residual target. This is pure analysis over the
        // already-computed `targets` — it does NOT touch generation, so it cannot
        // change coverage (the .7.2.10 regression is impossible here). For a BRANCH
        // target we ask `compute_reach_path` whether a deterministic steering plan
        // exists to it (Some → reachable_by_plan, None → no_reach_path). A RULE
        // target (no node_path/branch) is, by construction here, graph-reachable
        // but never generated → reachable_rule_not_generated. Turns the bare
        // residual count into an evidence-backed partition for signoff.
        let classification_entry = resolved_entry.clone();
        for target in targets.iter_mut() {
            let classification = match (
                &target.target_type,
                target.node_path.as_deref(),
                target.branch_index,
            ) {
                (StimuliCoverageTargetType::Branch, Some(node_path), Some(branch_index)) => {
                    if self
                        .compute_reach_path(
                            &classification_entry,
                            &target.rule_name,
                            node_path,
                            branch_index,
                        )
                        .is_some()
                    {
                        "reachable_by_plan"
                    } else {
                        "no_reach_path"
                    }
                }
                (StimuliCoverageTargetType::Rule, _, _) => "reachable_rule_not_generated",
                // A branch target missing its path/index is malformed for reach
                // analysis; mark it explicitly rather than silently mislabel.
                (StimuliCoverageTargetType::Branch, _, _) => "no_reach_path",
            };
            target.reach_classification = Some(classification.to_string());
        }

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
        self.target_probe_history.clear();
    }

    /// SV-EXH-PROOF.7.2.2 (PGEN-SV-EXH-PROOF-0116): build + install a target-reach
    /// plan that steers the OR-decision toward one coverage target (the OR node
    /// at `target_node_path` in `target_rule`, alternative
    /// `target_branch_index`), reached from `entry_rule`. Returns `true` if a
    /// reach path exists and the plan was installed, `false` otherwise (target
    /// unreachable / invalid OR node / bad branch index — see
    /// `compute_reach_path`). `bypass_fuel` bounds the on-path candidate-set
    /// bypass. GENERAL/parser-agnostic. Until this is called, `reach_plan` stays
    /// `None` and the OR-decision is byte-identical to pre-`.7.2.2` behavior.
    #[allow(dead_code)]
    pub fn set_reach_plan(
        &mut self,
        entry_rule: &str,
        target_rule: &str,
        target_node_path: &str,
        target_branch_index: usize,
        bypass_fuel: u32,
    ) -> bool {
        match self.compute_reach_path(
            entry_rule,
            target_rule,
            target_node_path,
            target_branch_index,
        ) {
            Some(chain) => {
                let mut plan = ActiveReachPlan::from_directives(&chain, bypass_fuel);
                // SV-EXH-PROOF.7.2.3: snapshot the target site's existing coverage
                // so reach_target_outcome reports only THIS plan's delta.
                plan.baseline_selected_hits =
                    self.branch_selected_hits(&plan.target_group_key, plan.target_branch_index);
                plan.baseline_success_hits =
                    self.branch_success_hits(&plan.target_group_key, plan.target_branch_index);
                self.reach_plan = Some(plan);
                true
            }
            None => false,
        }
    }

    /// SV-EXH-PROOF.7.2.3 (PGEN-SV-EXH-PROOF-0117): honest outcome of the installed
    /// reach plan, derived from the coverage metrics the OR-decision already
    /// records at the target site (delta vs the install-time baseline). Returns
    /// `None` if no plan is installed. This lets the `.7.2.4` driver distinguish
    /// genuine coverage (`Reached`) from an unsatisfiable-in-context target
    /// (`SelectedButFailed` — REPORT it, do not retry forever) from a steering
    /// miss (`NotReached`). NO new hot-path instrumentation — reuses existing
    /// selected/success counters (NO-WORKAROUNDS level 1-2).
    #[allow(dead_code)]
    pub fn reach_target_outcome(&self) -> Option<ReachOutcome> {
        let plan = self.reach_plan.as_ref()?;
        let selected =
            self.branch_selected_hits(&plan.target_group_key, plan.target_branch_index);
        let success =
            self.branch_success_hits(&plan.target_group_key, plan.target_branch_index);
        if success > plan.baseline_success_hits {
            Some(ReachOutcome::Reached)
        } else if selected > plan.baseline_selected_hits {
            Some(ReachOutcome::SelectedButFailed)
        } else {
            Some(ReachOutcome::NotReached)
        }
    }

    /// SV-EXH-PROOF.7.2.2: remove any installed reach plan, restoring the
    /// unsteered OR-decision.
    #[allow(dead_code)]
    pub fn clear_reach_plan(&mut self) {
        self.reach_plan = None;
    }

    /// SV-EXH-PROOF.7.2.4: how many times the driver's reach-plan steering hook
    /// fired during the last target-driven run (observability + test discriminant).
    #[allow(dead_code)]
    pub fn reach_plan_activations(&self) -> u64 {
        self.reach_plan_activations
    }

    /// SV-EXH-PROOF.7.2.4 (PGEN-SV-EXH-PROOF-0118): the DRIVER hook. Given a
    /// pending branch-coverage target (carrying rule_name + node_path +
    /// branch_index), install a reach plan that steers the next generation from
    /// `entry_rule` down to that branch. Returns the installed forced-branch count
    /// (>0) on success, or 0 if the target is not a branch target / lacks a path /
    /// is unreachable (in which case no plan is installed and the caller falls
    /// back to the existing helper-probe mechanism). Deterministic + parser-
    /// agnostic: it only forwards (rule, node_path, branch) into compute_reach_path.
    fn try_install_reach_plan_for_status(
        &mut self,
        entry_rule: &str,
        status: &TargetCoverageStatus,
        bypass_fuel: u32,
    ) -> usize {
        if status.target_type != StimuliCoverageTargetType::Branch {
            return 0;
        }
        let (Some(node_path), Some(branch_index)) =
            (status.node_path.as_ref(), status.branch_index)
        else {
            return 0;
        };
        let node_path = node_path.clone();
        if self.set_reach_plan(
            entry_rule,
            &status.rule_name.clone(),
            &node_path,
            branch_index,
            bypass_fuel,
        ) {
            self.reach_plan
                .as_ref()
                .map(|plan| plan.directives.len())
                .unwrap_or(0)
        } else {
            0
        }
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

    /// SV-EXH-PROOF.7.4.3 (PGEN-SV-EXH-PROOF-0141): the APPENDED minimal-witness pass.
    /// For each still-unresolved reachable target, generate ONE dedicated witness rooted
    /// at the TARGET'S OWN RULE (for a branch target, force that branch within the rule),
    /// so the target's subtree gets a FRESH, full depth budget instead of the leftovers
    /// after descending from the top entry — the root-cause fix from `.7.4.3a` (deep rules
    /// generate fine standalone but exhaust the budget reached from `systemverilog_file`).
    /// PURELY ADDITIVE: it only generates extra samples and accumulates their coverage into
    /// `self.coverage`; it never touches the diverse pass (a separate invocation) nor the
    /// target-drive pass (already complete), so the combined residual can only SHRINK
    /// (monotone). Re-evaluates after each witness (greedy set-cover: a target covered "on
    /// the way" by a prior witness is skipped). The witness pass runs under a temporary
    /// depth/visit slack (×2) so even deeper rules complete; the per-witness timeout makes
    /// pathologically-recursive rules fail (counted) rather than hang. Failures are
    /// classified by reason (DIAG-SEVERITY) so any tail's cause is visible.
    /// GRAMMAR-WELLFORMED.G.3.2: the reachability WITNESS certificates captured by the most recent
    /// `generate_target_witnesses` pass — one per resolved reachable target. Each is verified by
    /// replaying its `input` through the real parser (`grammar_wellformedness::verify_reachability_witness`).
    pub fn witness_certificates(&self) -> &[super::grammar_wellformedness::ReachabilityWitness] {
        &self.witness_certificates
    }

    pub fn generate_target_witnesses(
        &mut self,
        targets: &[StimuliCoverageTarget],
    ) -> Result<(Vec<String>, WitnessSummary)> {
        let applicable: Vec<StimuliCoverageTarget> = targets
            .iter()
            .filter(|t| t.reachable)
            .cloned()
            .collect();
        let total_targets = applicable.len();
        let resolved_before =
            total_targets.saturating_sub(self.evaluate_target_statuses(&applicable).len());

        // Temporary depth/visit slack for the witness pass only (generalises the existing
        // per-branch depth-slack retry at ~4869). Restored before return; never affects the
        // diverse pass (separate invocation). The per-witness timeout bounds the cost.
        let original_max_depth = self.config.max_depth;
        let original_max_rule_visits = self.config.max_rule_visits;
        self.config.max_depth = original_max_depth.saturating_mul(2);
        self.config.max_rule_visits = original_max_rule_visits.saturating_mul(2);
        let bypass_fuel = self.config.max_depth.saturating_add(1) as u32;

        // SV-EXH-PROOF.7.4.4 (PGEN-SV-EXH-PROOF-0143): enable Purdom shortest-derivation
        // ordering for the witness pass only. `generate_or` reads `witness_mode` +
        // `witness_min_terminal_lengths` to try the shortest-terminating alternative first,
        // so witnesses on deeply-factored rules converge before the per-witness budget
        // (the `.7.4.4.1` measurement showed the tail = `target_timeout`, not errors).
        // Restored in lockstep with the depth/visit slack below; the diverse pass never
        // sets these → its output is byte-identical → the witness pass stays monotone.
        self.witness_mode = true;
        // PGEN_WITNESS_NO_PURDOM disables the Purdom ordering (table stays None → the
        // witness arm in `generate_or` is skipped → default weighting), for a same-binary
        // A/B measurement of the ordering's effect. Default (unset) = Purdom ON.
        self.witness_min_terminal_lengths = if std::env::var_os("PGEN_WITNESS_NO_PURDOM").is_some() {
            None
        } else {
            Some(self.compute_min_terminal_lengths())
        };

        let mut outputs = Vec::new();
        // GRAMMAR-WELLFORMED.G.3.2: capture a reachability WITNESS per resolved target this pass.
        self.witness_certificates.clear();
        let mut attempted: HashSet<String> = HashSet::new();
        let mut witnesses_generated = 0usize;
        let mut depth_exceeded_failures = 0usize;
        let mut rule_visit_limit_failures = 0usize;
        let mut other_failures = 0usize;
        let mut no_entry_rule = 0usize;
        // SV-EXH-PROOF.7.4.4.1: timeouts classified separately so `other` is genuine errors.
        let mut target_timeout_failures = 0usize;
        let mut helper_timeout_failures = 0usize;
        // SV-EXH-PROOF.7.4.4.1: bounded WHY+WHERE samples for the "other"-class tail.
        let mut other_failure_samples: Vec<String> = Vec::new();
        // SV-EXH-PROOF.7.4.6.4 (TOOL-BUILD): construct-vs-search attribution + timeout sampling.
        let mut construct_attempt_failures = 0usize;
        let mut timeout_failure_samples: Vec<String> = Vec::new();

        loop {
            let pending = self.evaluate_target_statuses(&applicable);
            let Some(status) = pending.iter().find(|s| !attempted.contains(&s.id)).cloned() else {
                break;
            };
            attempted.insert(status.id.clone());

            // Witness entry = the target's OWN rule → its subtree gets the full budget.
            let entry_rule = status.rule_name.clone();
            if !self.grammar_tree.contains_key(entry_rule.as_str()) {
                no_entry_rule = no_entry_rule.saturating_add(1);
                continue;
            }

            // For a branch target, force that branch within its rule.
            let mut plan_installed = false;
            if status.target_type == StimuliCoverageTargetType::Branch {
                if let (Some(node_path), Some(branch_index)) =
                    (status.node_path.as_ref(), status.branch_index)
                {
                    plan_installed = self.set_reach_plan(
                        &entry_rule,
                        &entry_rule,
                        node_path,
                        branch_index,
                        bypass_fuel,
                    );
                }
            }

            // SV-EXH-PROOF.7.4.5: the witness pass uses its OWN budget (max(primary, floor)),
            // not the tiny 5 ms primary target-drive guard it used to reuse here.
            let timeout = self.witness_generation_timeout();
            // SV-EXH-PROOF.7.4.6.3: try derivation CONSTRUCTION first — bounded (O(tree-size),
            // no search/timeout). The reach plan set above still forces a branch target's
            // branch; construct_mode min-lengths every other choice. If the committed shortest
            // path dead-ends, fall back to the .7.4.5 search-with-budget.
            self.construct_mode = true;
            let construct_result =
                self.generate_from_entry_with_optional_timeout(&entry_rule, timeout);
            self.construct_mode = false;
            let construct_result_was_err = construct_result.is_err();
            // SV-EXH-PROOF.7.4.6.4 (TOOL-BUILD): capture WHY construction dead-ended (depth /
            // visit-limit / timeout / other) — the min-terminal-length-guided path can be short
            // in terminals yet too DEEP to complete within max_depth.
            let construct_err_reason: String = match &construct_result {
                Ok(_) => "none".to_string(),
                Err(e) => format!("{:?}", Self::classify_generation_error(e)),
            };
            let result = match construct_result {
                Ok(sample) => Ok(sample),
                Err(_) => {
                    // SV-EXH-PROOF.7.4.6.4: construction dead-ended for this target → search.
                    construct_attempt_failures = construct_attempt_failures.saturating_add(1);
                    self.generate_from_entry_with_optional_timeout(&entry_rule, timeout)
                }
            };
            if plan_installed {
                self.clear_reach_plan();
            }
            match result {
                Ok(sample) => {
                    // GRAMMAR-WELLFORMED.G.3.2: record the constructive reachability witness for
                    // this target (its id + the witness input) — verified later by replaying
                    // `input` through the real parser (`verify_reachability_witness`).
                    self.witness_certificates.push(
                        super::grammar_wellformedness::ReachabilityWitness {
                            fragment: status.id.clone(),
                            input: sample.clone(),
                        },
                    );
                    outputs.push(sample);
                    witnesses_generated = witnesses_generated.saturating_add(1);
                }
                Err(error) => match Self::classify_generation_error(&error) {
                    GenerationErrorReason::DepthExceeded => {
                        depth_exceeded_failures = depth_exceeded_failures.saturating_add(1);
                    }
                    GenerationErrorReason::RuleVisitLimit => {
                        rule_visit_limit_failures = rule_visit_limit_failures.saturating_add(1);
                    }
                    // SV-EXH-PROOF.7.4.4.1: classify TIMEOUTS separately (DIAG-SEVERITY:
                    // classify by reason, never lump). The old `_ =>` arm folded
                    // TargetTimeout + HelperTimeout into `other_failures`, so a bounded
                    // per-generation budget polluted the "other" tally with slow-but-not-
                    // erroring targets — masking whether "other" means a GENUINE generation
                    // error. With these arms, `other_failures` is strictly genuine errors.
                    GenerationErrorReason::TargetTimeout => {
                        target_timeout_failures = target_timeout_failures.saturating_add(1);
                        // SV-EXH-PROOF.7.4.6.4 (TOOL-BUILD): sample the timeout tail's WHERE
                        // (which rules/types/branches dead-end) — previously uncharacterised.
                        if timeout_failure_samples.len() < WITNESS_OTHER_FAILURE_SAMPLE_CAP {
                            timeout_failure_samples.push(format!(
                                "target_id='{}' rule='{}' type={:?} node_path={:?} branch={:?} construct_failed={} construct_reason={} search_reason={:#}",
                                status.id,
                                status.rule_name,
                                status.target_type,
                                status.node_path,
                                status.branch_index,
                                construct_result_was_err,
                                construct_err_reason,
                                error
                            ));
                        }
                    }
                    GenerationErrorReason::HelperTimeout => {
                        helper_timeout_failures = helper_timeout_failures.saturating_add(1);
                    }
                    GenerationErrorReason::Other => {
                        other_failures = other_failures.saturating_add(1);
                        // SV-EXH-PROOF.7.4.4.1 (TOOL-BUILD, WHY+WHERE before any fix):
                        // the "other" witness failures previously discarded their cause,
                        // leaving the residual tail uncharacterisable — the context-gating
                        // hypothesis could be neither confirmed nor refuted from the
                        // counters alone. Collect a BOUNDED sample of (WHY = error chain)
                        // + (WHERE = target identity) into the summary so the tail is
                        // diagnosed on tangible proof, not a guess
                        // ([[feedback_why_and_where_before_solution]]) at DEFAULT verbosity.
                        // Deliberately a bounded in-summary sample, NOT a per-target Debug
                        // trace: a Debug trace here required global debug verbosity, which
                        // floods every atom dispatch and produced a 6.9 GB log in seconds —
                        // disk is a critical resource that shall not be wasted.
                        if other_failure_samples.len() < WITNESS_OTHER_FAILURE_SAMPLE_CAP {
                            other_failure_samples.push(format!(
                                "target_id='{}' rule='{}' type={:?} node_path={:?} branch={:?} reason={:#}",
                                status.id,
                                status.rule_name,
                                status.target_type,
                                status.node_path,
                                status.branch_index,
                                error
                            ));
                        }
                    }
                },
            }
        }

        self.config.max_depth = original_max_depth;
        self.config.max_rule_visits = original_max_rule_visits;
        // SV-EXH-PROOF.7.4.4: leave witness mode (restore byte-identical diverse-pass behavior).
        self.witness_mode = false;
        self.witness_min_terminal_lengths = None;
        self.construct_mode = false; // defensive (also reset per-iteration after each attempt)

        let final_unresolved = self.evaluate_target_statuses(&applicable);
        let resolved_after = total_targets.saturating_sub(final_unresolved.len());
        // SV-EXH-PROOF.7.4.6.4 (TOOL-BUILD): sample the targets STILL unresolved after the
        // witness pass — includes the "construct succeeded but the witness did not cover the
        // target" (non-covering) class, which never errors and so is invisible to the failure
        // counters. Each: target_id | rule | type | node_path | branch | attempted.
        let unresolved_after_samples: Vec<String> = final_unresolved
            .iter()
            .take(WITNESS_OTHER_FAILURE_SAMPLE_CAP)
            .map(|status| {
                format!(
                    "target_id='{}' rule='{}' type={:?} node_path={:?} branch={:?} attempted={}",
                    status.id,
                    status.rule_name,
                    status.target_type,
                    status.node_path,
                    status.branch_index,
                    attempted.contains(&status.id)
                )
            })
            .collect();

        Ok((
            outputs,
            WitnessSummary {
                total_targets,
                resolved_before,
                resolved_after,
                witnesses_generated,
                depth_exceeded_failures,
                rule_visit_limit_failures,
                other_failures,
                no_entry_rule,
                target_timeout_failures,
                helper_timeout_failures,
                other_failure_samples,
                construct_attempt_failures,
                timeout_failure_samples,
                unresolved_after_samples,
            },
        ))
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
        let total_targets = applicable_targets.len();
        self.target_probe_history.clear();
        self.reach_plan_activations = 0; // SV-EXH-PROOF.7.2.4: per-run counter

        let mut outputs = Vec::new();
        let mut attempts = 0usize;
        let mut generation_successes = 0usize;
        let mut generation_errors = 0usize;
        let mut target_timeout_errors = 0usize;
        let mut helper_timeout_errors = 0usize;
        // DIAG-SEVERITY.3/.3.1: classify structural-budget failures separately (Err arm).
        let mut depth_exceeded_errors = 0usize;
        let mut rule_visit_limit_errors = 0usize;
        let mut best_remaining = applicable_targets.len();
        let mut stagnant_iterations = 0usize;

        self.trace(
            TraceLevel::Low,
            format_args!(
                "Starting target-driven generation: entry='{}' total_targets={} max_attempts={}",
                resolved_entry, total_targets, max_attempts
            ),
        );

        while attempts < max_attempts {
            let pending = self.evaluate_target_statuses(&applicable_targets);
            if pending.is_empty() {
                break;
            }
            let pending_before = pending.len();

            if pending.len() < best_remaining {
                best_remaining = pending.len();
                stagnant_iterations = 0;
            } else {
                stagnant_iterations = stagnant_iterations.saturating_add(1);
            }

            let probe_threshold = self.target_probe_threshold(&pending);

            // SV-EXH-PROOF.7.2.7 (PGEN-SV-EXH-PROOF-0121): reach-steering PREEMPTS
            // the helper-probe-entry switch (see the matching fix in
            // generate_until_targets_with_filter). When stagnant, FIRST look for a
            // pending BRANCH target to reach-steer toward; if one exists, keep
            // generation_entry = resolved_entry so the reach plan forces the whole
            // path from the real entry. Only when there is NO Branch candidate do
            // we fall back to the existing helper-probe-rule switch. This fixes the
            // .7.2.4/.7.2.6 dead-guard flaw (the old !helper_probe_active guard
            // could never be true under stagnation; measured activations=0).
            // Reach-steering is a DEEPER FALLBACK than the helper-probe (see the
            // matching fix in _with_filter): it engages only once stagnation
            // persists past reach_threshold (> the helper-probe's probe_threshold),
            // and only when the highest-priority pending target (pending[0]) is a
            // Branch. This preserves helper-probe/alternate-entry behavior for
            // Rule targets while still firing on the real corpus.
            let reach_threshold = probe_threshold.saturating_mul(2).saturating_add(8);
            let reach_candidate: Option<TargetCoverageStatus> =
                if stagnant_iterations >= reach_threshold {
                    pending
                        .first()
                        .filter(|status| status.target_type == StimuliCoverageTargetType::Branch)
                        .cloned()
                } else {
                    None
                };

            let generation_entry = if reach_candidate.is_some() {
                resolved_entry.clone()
            } else if stagnant_iterations >= probe_threshold {
                self.select_target_probe_rule_with_stagnation(
                    &pending,
                    &resolved_entry,
                    stagnant_iterations,
                    probe_threshold,
                )
                .unwrap_or_else(|| resolved_entry.clone())
            } else {
                resolved_entry.clone()
            };

            if generation_entry != resolved_entry {
                self.trace_target_probe_activation(
                    &resolved_entry,
                    &generation_entry,
                    &pending,
                    stagnant_iterations,
                    probe_threshold,
                    None,
                );
                self.trace_target_probe_ranking(
                    &resolved_entry,
                    &generation_entry,
                    &pending,
                    stagnant_iterations,
                    probe_threshold,
                    None,
                );
            }

            let helper_probe_active = generation_entry != resolved_entry;
            let mut reach_plan_active = false;
            if let Some(branch_status) = reach_candidate {
                let bypass_fuel = self.config.max_depth.saturating_add(1) as u32;
                if self.try_install_reach_plan_for_status(
                    &resolved_entry,
                    &branch_status,
                    bypass_fuel,
                ) > 0
                {
                    reach_plan_active = true;
                    self.reach_plan_activations = self.reach_plan_activations.saturating_add(1);
                    self.trace(
                        TraceLevel::Debug,
                        format_args!(
                            "Reach-plan steering activated: entry='{}' target_rule='{}' node_path={:?} branch={:?} stagnant={}",
                            resolved_entry,
                            branch_status.rule_name,
                            branch_status.node_path,
                            branch_status.branch_index,
                            stagnant_iterations
                        ),
                    );
                }
            }

            let mut generation_succeeded = false;
            attempts = attempts.saturating_add(1);
            let generation_timeout = self.target_drive_generation_timeout(helper_probe_active);
            let generation_result = self
                .generate_from_entry_with_optional_timeout(&generation_entry, generation_timeout);
            if reach_plan_active {
                self.clear_reach_plan();
            }
            match generation_result {
                Ok(sample) => {
                    generation_successes = generation_successes.saturating_add(1);
                    generation_succeeded = true;
                    if generation_entry == resolved_entry {
                        outputs.push(sample);
                    }
                }
                Err(error) => {
                    generation_errors = generation_errors.saturating_add(1);
                    // DIAG-SEVERITY.3/.3.1: classify the failure REASON via the canonical
                    // classifier (single source of truth) instead of an anonymous count.
                    match Self::classify_generation_error(&error) {
                        GenerationErrorReason::DepthExceeded => {
                            depth_exceeded_errors = depth_exceeded_errors.saturating_add(1);
                        }
                        GenerationErrorReason::RuleVisitLimit => {
                            rule_visit_limit_errors = rule_visit_limit_errors.saturating_add(1);
                        }
                        GenerationErrorReason::TargetTimeout => {
                            target_timeout_errors = target_timeout_errors.saturating_add(1);
                        }
                        GenerationErrorReason::HelperTimeout => {
                            helper_timeout_errors = helper_timeout_errors.saturating_add(1);
                        }
                        GenerationErrorReason::Other => {}
                    }
                }
            }

            let progress_checkpoint =
                Self::should_trace_target_drive_progress(attempts, max_attempts);
            let current_pending = if helper_probe_active || progress_checkpoint {
                Some(self.evaluate_target_statuses(&applicable_targets).len())
            } else {
                None
            };

            if helper_probe_active {
                self.record_target_probe_result(
                    &generation_entry,
                    pending_before,
                    current_pending.unwrap_or(pending_before),
                    generation_succeeded,
                );
                self.trace_target_probe_result(
                    &resolved_entry,
                    &generation_entry,
                    pending_before,
                    current_pending.unwrap_or(pending_before),
                    generation_succeeded,
                    None,
                );
            }

            if progress_checkpoint {
                self.trace_target_drive_progress(
                    &resolved_entry,
                    &generation_entry,
                    total_targets,
                    current_pending.unwrap_or(pending_before),
                    attempts,
                    max_attempts,
                    generation_successes,
                    generation_errors,
                    target_timeout_errors,
                    helper_timeout_errors,
                    stagnant_iterations,
                    probe_threshold,
                    None,
                );
            }
        }

        let unresolved_targets = self.evaluate_target_statuses(&applicable_targets);
        let resolved_targets = total_targets.saturating_sub(unresolved_targets.len());
        self.clear_targets();

        self.trace(
            TraceLevel::Low,
            format_args!(
                "Completed target-driven generation: entry='{}' resolved_targets={}/{} attempts={} generation_successes={} generation_errors={} depth_exceeded_errors={} rule_visit_limit_errors={} target_timeout_errors={} helper_timeout_errors={}",
                resolved_entry,
                resolved_targets,
                total_targets,
                attempts,
                generation_successes,
                generation_errors,
                depth_exceeded_errors,
                rule_visit_limit_errors,
                target_timeout_errors,
                helper_timeout_errors
            ),
        );

        // DIAG-SEVERITY.3/.3.1: an AGGREGATE, once-per-run, severity-Warning diagnostic
        // that is NEVER gated by verbosity (the correct use of the .2 mechanism — not
        // per-attempt spam). Surfaces the STRUCTURAL-BUDGET causes (depth + rule-visit)
        // that were previously masked in an anonymous error count.
        let structural_budget_failures =
            depth_exceeded_errors.saturating_add(rule_visit_limit_errors);
        if structural_budget_failures > 0 {
            crate::pgen_warn!(
                "target-driven stimuli generation: {} of {} attempts failed on a structural budget (depth_exceeded={} of max_depth={}, rule_visit_limit={} of max_rule_visits={}) — deep-factored rules cannot reach+complete from entry '{}'; see SV-EXH-PROOF.7.4.3a. {} of {} targets remain unresolved.",
                structural_budget_failures,
                attempts,
                depth_exceeded_errors,
                self.config.max_depth,
                rule_visit_limit_errors,
                self.config.max_rule_visits,
                resolved_entry,
                unresolved_targets.len(),
                total_targets
            );
        }

        Ok((
            outputs,
            TargetDriveSummary {
                entry_rule: resolved_entry,
                attempts,
                generation_successes,
                generation_errors,
                depth_exceeded_errors,
                rule_visit_limit_errors,
                target_timeout_errors,
                helper_timeout_errors,
                total_targets,
                applied_targets,
                resolved_targets,
                unresolved_targets,
                reach_plan_activations: self.reach_plan_activations,
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
        F: FnMut(&str, &TargetDriveFilterContext<'_>) -> Result<bool>,
    {
        let previous_validation_mode = self.target_drive_validation_active;
        self.target_drive_validation_active = true;
        let result = (|| {
            let resolved_entry = self.resolve_entry_rule(entry_rule)?;
            let applicable_targets: Vec<StimuliCoverageTarget> = targets
                .iter()
                .filter(|target| target.reachable)
                .cloned()
                .collect();
            let applied_targets = self.apply_targets(&applicable_targets);
            let total_targets = applicable_targets.len();
            self.target_probe_history.clear();
            self.reach_plan_activations = 0; // SV-EXH-PROOF.7.2.4: per-run counter

            let mut outputs = Vec::new();
            let mut attempts = 0usize;
            let mut generation_successes = 0usize;
            let mut generation_errors = 0usize;
            let mut depth_exceeded_errors = 0usize; // DIAG-SEVERITY.3
            let mut rule_visit_limit_errors = 0usize; // DIAG-SEVERITY.3.1
            let mut target_timeout_errors = 0usize;
            let mut helper_timeout_errors = 0usize;
            let mut best_remaining = applicable_targets.len();
            let mut stagnant_iterations = 0usize;
            let mut validation_summary = TargetDriveValidationSummary::default();

            self.trace(
                TraceLevel::Low,
                format_args!(
                    "Starting validation-aware target-driven generation: entry='{}' total_targets={} max_attempts={}",
                    resolved_entry, total_targets, max_attempts
                ),
            );

            while attempts < max_attempts {
                let pending = self.evaluate_target_statuses(&applicable_targets);
                if pending.is_empty() {
                    break;
                }
                let pending_before = pending.len();

                if pending.len() < best_remaining {
                    best_remaining = pending.len();
                    stagnant_iterations = 0;
                } else {
                    stagnant_iterations = stagnant_iterations.saturating_add(1);
                }

                let probe_threshold =
                    self.target_probe_threshold_for_validation(&pending, &validation_summary);

                // SV-EXH-PROOF.7.2.7 (PGEN-SV-EXH-PROOF-0121): reach-steering
                // PREEMPTS the helper-probe-entry switch. The .7.2.4/.7.2.6 flaw
                // was that the reach hook required !helper_probe_active while the
                // loop switched to a helper-probe rule on the SAME stagnation
                // condition, so the hook could never fire (measured: activations=0).
                // Fix: reach-steering is a DEEPER FALLBACK than the helper-probe.
                // The helper-probe / alternate-entry mechanism keeps its existing
                // window [probe_threshold, reach_threshold); reach-steering only
                // engages once stagnation PERSISTS past reach_threshold (i.e. the
                // helper-probe has had its chance and the run is still stuck). This
                // preserves alternate-entry probing for helper/Rule targets while
                // still firing on the real corpus (where stagnation climbed to 139
                // with probe_threshold=8, far past any margin). When it engages, it
                // steers toward the HIGHEST-PRIORITY pending Branch target
                // (pending[0], priority-sorted) from the real entry.
                let reach_threshold = probe_threshold.saturating_mul(2).saturating_add(8);
                let reach_candidate: Option<TargetCoverageStatus> =
                    if stagnant_iterations >= reach_threshold {
                        pending
                            .first()
                            .filter(|status| {
                                status.target_type == StimuliCoverageTargetType::Branch
                            })
                            .cloned()
                    } else {
                        None
                    };

                let generation_entry = if reach_candidate.is_some() {
                    // Reach-steer from the real entry this attempt.
                    resolved_entry.clone()
                } else if stagnant_iterations >= probe_threshold {
                    self.select_target_probe_rule_for_validation_with_stagnation(
                        &pending,
                        &resolved_entry,
                        stagnant_iterations,
                        probe_threshold,
                        &validation_summary,
                    )
                    .unwrap_or_else(|| resolved_entry.clone())
                } else {
                    resolved_entry.clone()
                };

                if generation_entry != resolved_entry {
                    self.trace_target_probe_activation(
                        &resolved_entry,
                        &generation_entry,
                        &pending,
                        stagnant_iterations,
                        probe_threshold,
                        Some(&validation_summary),
                    );
                    self.trace_target_probe_ranking(
                        &resolved_entry,
                        &generation_entry,
                        &pending,
                        stagnant_iterations,
                        probe_threshold,
                        Some(&validation_summary),
                    );
                }

                let helper_probe_active = generation_entry != resolved_entry;
                let mut reach_plan_active = false;
                if let Some(branch_status) = reach_candidate {
                    let bypass_fuel = self.config.max_depth.saturating_add(1) as u32;
                    if self.try_install_reach_plan_for_status(
                        &resolved_entry,
                        &branch_status,
                        bypass_fuel,
                    ) > 0
                    {
                        reach_plan_active = true;
                        self.reach_plan_activations =
                            self.reach_plan_activations.saturating_add(1);
                    }
                }

                let mut generation_succeeded = false;
                attempts = attempts.saturating_add(1);
                let success_snapshot = self.coverage.snapshot_success_state();
                let generation_timeout = self.target_drive_generation_timeout(helper_probe_active);
                let generation_outcome = self.generate_from_entry_with_optional_timeout(
                    &generation_entry,
                    generation_timeout,
                );
                if reach_plan_active {
                    self.clear_reach_plan();
                }
                match generation_outcome {
                    Ok(sample) => {
                        generation_successes = generation_successes.saturating_add(1);
                        generation_succeeded = true;
                        let is_primary_entry = generation_entry == resolved_entry;
                        if !is_primary_entry {
                            validation_summary.alternate_entry_attempts = validation_summary
                                .alternate_entry_attempts
                                .saturating_add(1);
                        }
                        let filter_context = TargetDriveFilterContext {
                            primary_entry_rule: resolved_entry.as_str(),
                            generation_entry_rule: generation_entry.as_str(),
                            is_primary_entry,
                        };
                        let accepted = output_filter(&sample, &filter_context)?;
                        if is_primary_entry {
                            validation_summary.validated_outputs =
                                validation_summary.validated_outputs.saturating_add(1);
                        }
                        if !accepted {
                            if is_primary_entry {
                                // Primary-entry outputs are candidate final samples, so failed
                                // validation must not pay target debt for the canonical entry rule.
                                self.coverage.restore_success_state(&success_snapshot);
                                validation_summary.rejected_outputs =
                                    validation_summary.rejected_outputs.saturating_add(1);
                            } else {
                                // Alternate-entry probes exist only to exercise helper-rule debt.
                                // Even when those helper outputs are not valid full-entry samples,
                                // keep the local coverage progress so target driving can retire
                                // subrule debt instead of spinning on rolled-back helper probes.
                                validation_summary.alternate_entry_rejected_outputs =
                                    validation_summary
                                        .alternate_entry_rejected_outputs
                                        .saturating_add(1);
                            }
                            if helper_probe_active {
                                let pending_after =
                                    self.evaluate_target_statuses(&applicable_targets).len();
                                self.record_target_probe_result(
                                    &generation_entry,
                                    pending_before,
                                    pending_after,
                                    generation_succeeded,
                                );
                                self.trace_target_probe_result(
                                    &resolved_entry,
                                    &generation_entry,
                                    pending_before,
                                    pending_after,
                                    generation_succeeded,
                                    Some(&validation_summary),
                                );
                            }
                            continue;
                        }

                        if is_primary_entry {
                            validation_summary.accepted_outputs =
                                validation_summary.accepted_outputs.saturating_add(1);
                            outputs.push(sample);
                        } else {
                            validation_summary.alternate_entry_accepted_outputs =
                                validation_summary
                                    .alternate_entry_accepted_outputs
                                    .saturating_add(1);
                        }
                    }
                    Err(error) => {
                        generation_errors = generation_errors.saturating_add(1);
                        // DIAG-SEVERITY.3/.3.1: canonical reason classification.
                        match Self::classify_generation_error(&error) {
                            GenerationErrorReason::DepthExceeded => {
                                depth_exceeded_errors = depth_exceeded_errors.saturating_add(1);
                            }
                            GenerationErrorReason::RuleVisitLimit => {
                                rule_visit_limit_errors =
                                    rule_visit_limit_errors.saturating_add(1);
                            }
                            GenerationErrorReason::TargetTimeout => {
                                target_timeout_errors = target_timeout_errors.saturating_add(1);
                                validation_summary.target_timeout_errors =
                                    validation_summary.target_timeout_errors.saturating_add(1);
                            }
                            GenerationErrorReason::HelperTimeout => {
                                helper_timeout_errors = helper_timeout_errors.saturating_add(1);
                                validation_summary.helper_timeout_errors =
                                    validation_summary.helper_timeout_errors.saturating_add(1);
                            }
                            GenerationErrorReason::Other => {}
                        }
                    }
                }

                let progress_checkpoint =
                    Self::should_trace_target_drive_progress(attempts, max_attempts);
                let current_pending = if helper_probe_active || progress_checkpoint {
                    Some(self.evaluate_target_statuses(&applicable_targets).len())
                } else {
                    None
                };

                if helper_probe_active {
                    self.record_target_probe_result(
                        &generation_entry,
                        pending_before,
                        current_pending.unwrap_or(pending_before),
                        generation_succeeded,
                    );
                    self.trace_target_probe_result(
                        &resolved_entry,
                        &generation_entry,
                        pending_before,
                        current_pending.unwrap_or(pending_before),
                        generation_succeeded,
                        Some(&validation_summary),
                    );
                }

                if progress_checkpoint {
                    self.trace_target_drive_progress(
                        &resolved_entry,
                        &generation_entry,
                        total_targets,
                        current_pending.unwrap_or(pending_before),
                        attempts,
                        max_attempts,
                        generation_successes,
                        generation_errors,
                        target_timeout_errors,
                        helper_timeout_errors,
                        stagnant_iterations,
                        probe_threshold,
                        Some(&validation_summary),
                    );
                }
            }

            let unresolved_targets = self.evaluate_target_statuses(&applicable_targets);
            let resolved_targets = total_targets.saturating_sub(unresolved_targets.len());
            self.clear_targets();

            self.trace(
                TraceLevel::Low,
                format_args!(
                    "Completed validation-aware target-driven generation: entry='{}' resolved_targets={}/{} attempts={} generation_successes={} generation_errors={} target_timeout_errors={} helper_timeout_errors={} accepted_outputs={} rejected_outputs={} alternate_attempts={} alternate_accepted={} alternate_rejected={}",
                    resolved_entry,
                    resolved_targets,
                    total_targets,
                    attempts,
                    generation_successes,
                    generation_errors,
                    target_timeout_errors,
                    helper_timeout_errors,
                    validation_summary.accepted_outputs,
                    validation_summary.rejected_outputs,
                    validation_summary.alternate_entry_attempts,
                    validation_summary.alternate_entry_accepted_outputs,
                    validation_summary.alternate_entry_rejected_outputs,
                ),
            );

            Ok((
                outputs,
                TargetDriveSummary {
                    entry_rule: resolved_entry,
                    attempts,
                    generation_successes,
                    generation_errors,
                    depth_exceeded_errors,
                    rule_visit_limit_errors,
                    target_timeout_errors,
                    helper_timeout_errors,
                    total_targets,
                    applied_targets,
                    resolved_targets,
                    unresolved_targets,
                    reach_plan_activations: self.reach_plan_activations,
                },
                validation_summary,
            ))
        })();
        self.target_drive_validation_active = previous_validation_mode;
        result
    }

    #[cfg(test)]
    fn select_target_probe_rule(
        &self,
        pending: &[TargetCoverageStatus],
        resolved_entry: &str,
    ) -> Option<String> {
        let probe_threshold = self.target_probe_threshold(pending);
        let stagnant_iterations = self.pending_frontier_unlock_threshold(probe_threshold);
        self.select_target_probe_rule_with_stagnation(
            pending,
            resolved_entry,
            stagnant_iterations,
            probe_threshold,
        )
    }

    fn select_target_probe_rule_with_stagnation(
        &self,
        pending: &[TargetCoverageStatus],
        resolved_entry: &str,
        stagnant_iterations: usize,
        probe_threshold: usize,
    ) -> Option<String> {
        let dependency_candidate = self.best_dependency_probe_candidate(pending, resolved_entry);
        let pending_candidate = self.best_pending_probe_candidate(pending, resolved_entry);
        self.select_target_probe_candidate(
            dependency_candidate.as_ref(),
            pending_candidate.as_ref(),
            stagnant_iterations,
            probe_threshold,
        )
    }

    #[cfg(test)]
    fn select_target_probe_rule_for_validation(
        &self,
        pending: &[TargetCoverageStatus],
        resolved_entry: &str,
        validation_summary: &TargetDriveValidationSummary,
    ) -> Option<String> {
        let probe_threshold =
            self.target_probe_threshold_for_validation(pending, validation_summary);
        let stagnant_iterations = self.pending_frontier_unlock_threshold(probe_threshold);
        self.select_target_probe_rule_for_validation_with_stagnation(
            pending,
            resolved_entry,
            stagnant_iterations,
            probe_threshold,
            validation_summary,
        )
    }

    fn select_target_probe_rule_for_validation_with_stagnation(
        &self,
        pending: &[TargetCoverageStatus],
        resolved_entry: &str,
        stagnant_iterations: usize,
        probe_threshold: usize,
        validation_summary: &TargetDriveValidationSummary,
    ) -> Option<String> {
        let dependency_candidate = self.best_dependency_probe_candidate(pending, resolved_entry);
        if Self::validation_prefers_primary_entry(validation_summary) {
            return dependency_candidate
                .filter(Self::validation_dependency_probe_is_worthy)
                .map(|candidate| candidate.rule_name);
        }

        let pending_candidate = self.best_pending_probe_candidate(pending, resolved_entry);
        self.select_target_probe_candidate(
            dependency_candidate.as_ref(),
            pending_candidate.as_ref(),
            stagnant_iterations,
            probe_threshold,
        )
    }

    fn select_target_probe_candidate(
        &self,
        dependency_candidate: Option<&DependencyProbeCandidate>,
        pending_candidate: Option<&PendingProbeCandidate>,
        stagnant_iterations: usize,
        probe_threshold: usize,
    ) -> Option<String> {
        match (dependency_candidate, pending_candidate) {
            (Some(dependency), Some(pending))
                if self.pending_frontier_outranks_dependency_probe(
                    dependency,
                    pending,
                    stagnant_iterations,
                    probe_threshold,
                ) =>
            {
                Some(pending.rule_name.clone())
            }
            (Some(dependency), _) => Some(dependency.rule_name.clone()),
            (None, Some(pending)) => Some(pending.rule_name.clone()),
            (None, None) => None,
        }
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
                    let probe_history = self
                        .target_probe_history
                        .get(rule_name.as_str())
                        .cloned()
                        .unwrap_or_default();
                    DependencyProbeCandidate {
                        rule_name: rule_name.clone(),
                        dependency_rule_deficit,
                        dependency_rule_successes,
                        literalish_hint_score: self.rule_literalish_hint_probe_score(rule_name),
                        probe_attempts: probe_history.attempts,
                        probe_resolved_delta_total: probe_history.resolved_delta_total,
                        probe_best_resolved_delta: probe_history.best_resolved_delta,
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
                    Self::probe_yield_score(left.probe_resolved_delta_total, left.probe_attempts)
                        .cmp(&Self::probe_yield_score(
                            right.probe_resolved_delta_total,
                            right.probe_attempts,
                        ))
                })
                .then_with(|| {
                    left.blocked_remaining_successes
                        .cmp(&right.blocked_remaining_successes)
                })
                .then_with(|| {
                    left.probe_best_resolved_delta
                        .cmp(&right.probe_best_resolved_delta)
                })
                .then_with(|| {
                    left.probe_resolved_delta_total
                        .cmp(&right.probe_resolved_delta_total)
                })
                .then_with(|| left.blocked_targets.cmp(&right.blocked_targets))
                .then_with(|| left.literalish_hint_score.cmp(&right.literalish_hint_score))
                .then_with(|| {
                    right
                        .dependency_rule_successes
                        .cmp(&left.dependency_rule_successes)
                })
                .then_with(|| left.max_target_priority.cmp(&right.max_target_priority))
                .then_with(|| right.rule_name.cmp(&left.rule_name))
        })
    }

    fn best_pending_probe_candidate(
        &self,
        pending: &[TargetCoverageStatus],
        resolved_entry: &str,
    ) -> Option<PendingProbeCandidate> {
        let mut candidates: HashMap<String, PendingProbeCandidate> = HashMap::new();
        for status in pending {
            if status.rule_name == resolved_entry
                || !self.grammar_tree.contains_key(status.rule_name.as_str())
            {
                continue;
            }

            let entry = candidates
                .entry(status.rule_name.clone())
                .or_insert_with(|| PendingProbeCandidate {
                    probe_attempts: self
                        .target_probe_history
                        .get(status.rule_name.as_str())
                        .map(|history| history.attempts)
                        .unwrap_or(0),
                    probe_resolved_delta_total: self
                        .target_probe_history
                        .get(status.rule_name.as_str())
                        .map(|history| history.resolved_delta_total)
                        .unwrap_or(0),
                    probe_best_resolved_delta: self
                        .target_probe_history
                        .get(status.rule_name.as_str())
                        .map(|history| history.best_resolved_delta)
                        .unwrap_or(0),
                    rule_name: status.rule_name.clone(),
                    literalish_hint_score: self
                        .rule_literalish_hint_probe_score(status.rule_name.as_str()),
                    ..PendingProbeCandidate::default()
                });
            entry.max_target_priority = entry.max_target_priority.max(status.priority_score);
            entry.blocked_remaining_successes = entry
                .blocked_remaining_successes
                .saturating_add(status.remaining_successes);
            if matches!(status.target_type, StimuliCoverageTargetType::Branch) {
                entry.branch_target_count = entry.branch_target_count.saturating_add(1);
            }
        }

        candidates.into_values().max_by(|left, right| {
            left.branch_target_count
                .cmp(&right.branch_target_count)
                .then_with(|| {
                    Self::probe_yield_score(left.probe_resolved_delta_total, left.probe_attempts)
                        .cmp(&Self::probe_yield_score(
                            right.probe_resolved_delta_total,
                            right.probe_attempts,
                        ))
                })
                .then_with(|| {
                    left.probe_best_resolved_delta
                        .cmp(&right.probe_best_resolved_delta)
                })
                .then_with(|| left.literalish_hint_score.cmp(&right.literalish_hint_score))
                .then_with(|| {
                    left.blocked_remaining_successes
                        .cmp(&right.blocked_remaining_successes)
                })
                .then_with(|| {
                    left.probe_resolved_delta_total
                        .cmp(&right.probe_resolved_delta_total)
                })
                .then_with(|| left.max_target_priority.cmp(&right.max_target_priority))
                .then_with(|| right.rule_name.cmp(&left.rule_name))
        })
    }

    fn pending_frontier_outranks_dependency_probe(
        &self,
        dependency: &DependencyProbeCandidate,
        pending: &PendingProbeCandidate,
        stagnant_iterations: usize,
        probe_threshold: usize,
    ) -> bool {
        if dependency.dependency_rule_deficit > 1
            || dependency.probe_best_resolved_delta > 0
            || dependency.probe_resolved_delta_total > 0
            || !self.pending_frontier_is_unlocked(stagnant_iterations, probe_threshold)
        {
            return false;
        }

        let dependency_yield = Self::probe_yield_score(
            dependency.probe_resolved_delta_total,
            dependency.probe_attempts,
        );
        let pending_yield =
            Self::probe_yield_score(pending.probe_resolved_delta_total, pending.probe_attempts);
        if pending_yield < dependency_yield {
            return false;
        }

        let pending_branch_floor = dependency.blocked_targets.saturating_mul(2).max(8);
        let pending_remaining_floor = dependency
            .blocked_remaining_successes
            .saturating_mul(2)
            .max(8);

        pending.branch_target_count >= pending_branch_floor
            && pending.blocked_remaining_successes >= pending_remaining_floor
    }

    fn pending_frontier_unlock_threshold(&self, probe_threshold: usize) -> usize {
        probe_threshold.saturating_add(self.config.target_pending_frontier_extra_stagnation)
    }

    fn pending_frontier_is_unlocked(
        &self,
        stagnant_iterations: usize,
        probe_threshold: usize,
    ) -> bool {
        stagnant_iterations >= self.pending_frontier_unlock_threshold(probe_threshold)
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
            || candidate.probe_best_resolved_delta >= 4
            || candidate.probe_resolved_delta_total >= 4
    }

    fn probe_yield_score(resolved_delta_total: u64, attempts: u64) -> u64 {
        if attempts == 0 {
            return 0;
        }

        resolved_delta_total
            .saturating_mul(1024)
            .saturating_div(attempts)
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
            ASTNode::Lookahead { element, .. } => {
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

    /// STIMULI-SIGNOFF.2 (PGEN-STIMULI-SIGNOFF-0002, PURE analysis — no generation change):
    /// enumerate the grammar's **k-path coverage universe** (Havrikov & Zeller,
    /// "Systematically Covering Input Structure", ASE 2019). A k-path is a length-k chain of
    /// nonterminals `n1 -> n2 -> ... -> nk` where each `n_{i+1}` is directly referenced in
    /// `n_i`'s body (the rule-reference graph, via `collect_rule_references`). k=1 = every
    /// rule (≈ our rule coverage); k=2 = every reference edge (≈ our branch coverage); k>=3 =
    /// the deeper ancestor-context combinations our current rule+branch metric does NOT
    /// measure — i.e. the gap the signoff bar should close. The returned set's size is the
    /// coverage DENOMINATOR; a later slice tracks the covered subset (numerator) from
    /// generation. Determinism: built in `rule_order` with sorted successors; no RNG/time.
    /// GENERAL/parser-agnostic (keyed only on the rule-reference graph). NOTE: the k-path
    /// count grows combinatorially in k — callers should use small k (2-3) on large grammars.
    /// `dead_code`-allowed: analysis-only first slice; the coverage-tracking caller lands next.
    #[allow(dead_code)]
    fn compute_k_paths(&self, k: usize) -> Vec<Vec<String>> {
        if k == 0 {
            return Vec::new();
        }
        // Direct rule-reference adjacency: rule -> sorted successors that exist in the grammar.
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        for rule_name in self.rule_order.iter() {
            let Some(node) = self.grammar_tree.get(rule_name.as_str()) else {
                continue;
            };
            let mut refs: HashSet<String> = HashSet::new();
            self.collect_rule_references(node, &mut refs);
            let mut succ: Vec<String> = refs
                .into_iter()
                .filter(|r| self.grammar_tree.contains_key(r.as_str()))
                .collect();
            succ.sort();
            adj.insert(rule_name.clone(), succ);
        }
        // Level 1 = every defined rule (in rule_order for determinism).
        let mut paths: Vec<Vec<String>> = self
            .rule_order
            .iter()
            .filter(|r| self.grammar_tree.contains_key(r.as_str()))
            .map(|r| vec![r.clone()])
            .collect();
        // Extend each path by one referenced rule, k-1 times.
        for _ in 1..k {
            let mut next: Vec<Vec<String>> = Vec::new();
            for path in &paths {
                if let Some(succ) = path.last().and_then(|last| adj.get(last)) {
                    for s in succ {
                        let mut extended = path.clone();
                        extended.push(s.clone());
                        next.push(extended);
                    }
                }
            }
            paths = next;
            if paths.is_empty() {
                break;
            }
        }
        paths
    }

    /// STIMULI-SIGNOFF.2.2 (PGEN-STIMULI-SIGNOFF-0003): turn ON k-path coverage recording at
    /// depth `k`. Default is OFF (`k_path_recording == None`) → generation is byte-identical.
    /// Enabling makes `generate_rule` record covered k-paths (read-only; no decision change).
    #[allow(dead_code)]
    fn enable_k_path_recording(&mut self, k: usize) {
        self.k_path_recording = Some((k.max(1), HashSet::new()));
    }

    /// The set of k-paths covered so far (numerator), or `None` if recording is off.
    #[allow(dead_code)]
    fn covered_k_paths(&self) -> Option<&HashSet<Vec<String>>> {
        self.k_path_recording.as_ref().map(|(_, set)| set)
    }

    /// k-path COVERAGE = (covered ∩ universe, universe size) for the recording depth `k`.
    /// `compute_k_paths(k)` is the universe (denominator); the recorder is the numerator. The
    /// covered set is filtered to the universe (a covered window is always a real reference
    /// chain, so this only guards against hint/shortcut artifacts). `None` if recording is off.
    #[allow(dead_code)]
    fn k_path_coverage(&self) -> Option<(usize, usize)> {
        let (k, covered) = self.k_path_recording.as_ref()?;
        let universe: HashSet<Vec<String>> = self.compute_k_paths(*k).into_iter().collect();
        let hit = covered.iter().filter(|p| universe.contains(*p)).count();
        Some((hit, universe.len()))
    }

    /// STIMULI-SIGNOFF.2.3 (PGEN-STIMULI-SIGNOFF-0004): public k-path coverage REPORT — the
    /// usable surface over `.2.1` (universe / `compute_k_paths`) + `.2.2` (recorder). Generates
    /// `samples` derivations from `entry` with k-path recording ON, then returns
    /// `(covered_in_universe, universe_size)` for depth `k`. Intended for a one-shot report
    /// run (it leaves recording enabled, caller-scoped). Generation itself is unchanged —
    /// recording is read-only instrumentation.
    pub fn k_path_coverage_report(&mut self, entry: &str, samples: usize, k: usize) -> (usize, usize) {
        self.enable_k_path_recording(k);
        for _ in 0..samples {
            let _ = self.generate_from_entry(entry);
        }
        self.k_path_coverage().unwrap_or((0, 0))
    }

    /// SV-EXH-PROOF.7.2.1 (PGEN-SV-EXH-PROOF-0115, pure analysis — no generation
    /// behavior change): like `collect_rule_references`, but also records WHERE
    /// each rule reference occurs, using the same `node_path` encoding as
    /// `collect_branch_groups` (`o{i}` Or alt, `s{i}` Sequence elem, `q`
    /// Quantified, `l` Lookahead, `a` Atom→Node), so a site can be replayed by
    /// `node_at_path`. `base_path` is the caller's path prefix ("root" at the
    /// rule top). GENERAL — keyed only on ASTNode structure, never on rule
    /// names/sigils, per [[feedback_ast_pipeline_parser_agnostic]].
    ///
    /// `dead_code`-allowed: analysis-only in `.7.2.1`; the production caller that
    /// drives generation lands in `.7.2.2`. Exercised now by unit tests.
    #[allow(dead_code)]
    fn collect_rule_reference_sites(
        node: &ASTNode,
        base_path: &str,
        out: &mut Vec<RuleReferenceSite>,
    ) {
        match node {
            ASTNode::Or { alternatives } => {
                for (idx, alternative) in alternatives.iter().enumerate() {
                    let alt_path = format!("{}/o{}", base_path, idx);
                    Self::collect_rule_reference_sites(alternative, &alt_path, out);
                }
            }
            ASTNode::Sequence { elements } => {
                for (idx, element) in elements.iter().enumerate() {
                    let element_path = format!("{}/s{}", base_path, idx);
                    Self::collect_rule_reference_sites(element, &element_path, out);
                }
            }
            ASTNode::Quantified { element, .. } => {
                let quantified_path = format!("{}/q", base_path);
                Self::collect_rule_reference_sites(element, &quantified_path, out);
            }
            ASTNode::Lookahead { element, .. } => {
                let lookahead_path = format!("{}/l", base_path);
                Self::collect_rule_reference_sites(element, &lookahead_path, out);
            }
            ASTNode::Atom { value } => match value {
                ASTValue::Node(node) => {
                    let atom_path = format!("{}/a", base_path);
                    Self::collect_rule_reference_sites(node, &atom_path, out);
                }
                ASTValue::Token(parts) => {
                    if let Some((token_type, token_value)) = Self::extract_token_pair(parts) {
                        if token_type == "rule_reference" {
                            out.push(RuleReferenceSite {
                                referenced_rule: token_value.to_string(),
                                node_path: base_path.to_string(),
                            });
                        }
                    }
                }
            },
        }
    }

    /// SV-EXH-PROOF.7.4.2 (PGEN-SV-EXH-PROOF-0139, PURE analysis — NO generation
    /// change): Purdom phase-1 shortest-derivation table. For each rule, the
    /// MINIMUM number of terminal atoms (lexemes) a derivation rooted at that rule
    /// must produce — the grammar-agnostic analogue of Purdom's `SLEN` (Purdom,
    /// "A sentence generator for testing parsers", BIT 1972; see
    /// docs/tasks/SV-EXH-PROOF-7.3-literature-grounded-literal-zero-design.md). This
    /// is the foundation `.7.4.3` uses to construct ONE minimal witness per residual
    /// coverage obligation (the decoupled literal-0 plan from `.7.3`).
    ///
    /// Metric choice (documented, deterministic): we count TERMINAL ATOMS (each
    /// terminal lexeme = 1; a `rule_reference` = the referenced rule's min; a
    /// quantifier with min-count `n` contributes `n × element_min`; a `?`/`*`
    /// contributes 0; a lookahead contributes 0 since predicates consume no input),
    /// NOT characters — a structural proxy for "shortest derivation" that is exactly
    /// what we need to pick the shortest alternative/path when building a witness.
    ///
    /// Recursion is handled by a FIXPOINT (Purdom phase 1): every rule starts
    /// "unknown" (absent from the map = currently +∞) and is relaxed from its body
    /// using the current estimates until no rule's value changes. A left/right/self-
    /// recursive rule with a non-recursive alternative (`expr := expr "+" n | n`)
    /// resolves via that alternative; a rule that STAYS absent after the fixpoint can
    /// only derive via unbounded recursion with no terminating base case — it has no
    /// finite witness (and is exactly the kind of non-terminating rule Ford's PEG
    /// well-formedness check, PARSE-SOTA A1, would also flag). Determinism: no RNG,
    /// no time; iteration order is `self.rule_order`; the fixpoint result is
    /// order-independent (min is commutative). GENERAL/parser-agnostic — keyed only
    /// on the node shape + the `rule_reference` token type, no rule-name special-
    /// casing, per [[feedback_ast_pipeline_parser_agnostic]].
    ///
    /// Caller: `generate_target_witnesses` (`.7.4.4`) computes this once at witness-pass
    /// start; `generate_or` then consults it (under `witness_mode`) to try the shortest-
    /// terminating alternative first (Purdom). Also exercised by unit tests.
    fn compute_min_terminal_lengths(&self) -> HashMap<String, usize> {
        let mut min_len: HashMap<String, usize> = HashMap::new();
        loop {
            let mut changed = false;
            for rule_name in self.rule_order.iter() {
                let Some(node) = self.grammar_tree.get(rule_name.as_str()) else {
                    continue;
                };
                let Some(candidate) = Self::min_terminal_length_of_node(node, &min_len) else {
                    continue;
                };
                match min_len.get(rule_name.as_str()) {
                    Some(&existing) if existing <= candidate => {}
                    _ => {
                        min_len.insert(rule_name.clone(), candidate);
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        min_len
    }

    /// SV-EXH-PROOF.7.4.2 (PGEN-SV-EXH-PROOF-0139, PURE analysis): minimum
    /// terminal-atom count for a single node given the current per-rule estimates
    /// `min_len`. Returns `None` when the node's minimum cannot YET be determined —
    /// a referenced rule has no estimate yet, OR (for an `Or`) every alternative is
    /// currently unresolvable, OR (for a `Sequence`/`Quantified`) a needed child is
    /// unresolvable — so the fixpoint in `compute_min_terminal_lengths` retries on
    /// the next pass. See that function's doc for the metric definition.
    fn min_terminal_length_of_node(
        node: &ASTNode,
        min_len: &HashMap<String, usize>,
    ) -> Option<usize> {
        match node {
            // Ordered choice: resolvable as soon as ANY alternative is; the min over
            // the currently-resolvable alternatives (Purdom's SHORT picks this one).
            ASTNode::Or { alternatives } => alternatives
                .iter()
                .filter_map(|alt| Self::min_terminal_length_of_node(alt, min_len))
                .min(),
            // Sequence: sum of every element's minimum (all must be resolvable).
            ASTNode::Sequence { elements } => {
                let mut sum: usize = 0;
                for element in elements {
                    sum = sum.saturating_add(Self::min_terminal_length_of_node(element, min_len)?);
                }
                Some(sum)
            }
            // Quantifier: min-count repetitions of the body. `?`/`*` (min 0) can
            // match empty → contributes 0 with no dependency on the body's estimate.
            ASTNode::Quantified { element, quantifier } => {
                let (min_count, _) =
                    super::parse_quantifier_bounds(quantifier).unwrap_or((0, None));
                if min_count == 0 {
                    Some(0)
                } else {
                    let element_min = Self::min_terminal_length_of_node(element, min_len)?;
                    Some(min_count.saturating_mul(element_min))
                }
            }
            // Syntactic predicate (`&X`/`!X`): consumes no input → 0 terminals.
            ASTNode::Lookahead { .. } => Some(0),
            ASTNode::Atom { value } => match value {
                ASTValue::Node(inner) => Self::min_terminal_length_of_node(inner, min_len),
                ASTValue::Token(parts) => match Self::extract_token_pair(parts) {
                    // A rule reference costs the referenced rule's current minimum.
                    Some(("rule_reference", referenced_rule)) => {
                        min_len.get(referenced_rule).copied()
                    }
                    // Any other token type is a terminal lexeme → exactly one atom.
                    Some(_) => Some(1),
                    // Malformed/empty token payload: treat conservatively as one
                    // terminal so it never blocks the fixpoint.
                    None => Some(1),
                },
            },
        }
    }

    /// SV-EXH-PROOF.7.2.1 (PGEN-SV-EXH-PROOF-0115, pure analysis): compute the
    /// chain of forced OR-branch directives that deterministically steers
    /// generation from `entry_rule` down to a coverage target — the OR node at
    /// `target_node_path` in `target_rule`, alternative `target_branch_index`.
    ///
    /// Algorithm: breadth-first search over the rule-reference graph from
    /// `entry_rule`, tracking each rule's discoverer so the shortest rule path
    /// (fewest hops, hence fewest forced directives) is recovered. For each
    /// parent→child hop, the reference site's `node_path` is walked from the
    /// parent rule's root; every `o{i}` segment crossed contributes a
    /// `ReachDirective` (force alternative `i` at that OR node) so the child rule
    /// is actually entered. The final directive forces `target_branch_index` at
    /// the target OR node itself.
    ///
    /// Determinism: rule references within a body are visited in source order,
    /// and BFS dequeues in insertion order, so the same (grammar, entry, target)
    /// always yields the same plan (no RNG, no time). Cycle-safe: each rule is
    /// enqueued at most once (`discovered` set). Returns `None` if the target
    /// rule is unreachable from `entry_rule`, or if the target OR node / branch
    /// index does not exist. GENERAL — no rule-name special-casing, per
    /// [[feedback_ast_pipeline_parser_agnostic]].
    ///
    /// `dead_code`-allowed: analysis-only in `.7.2.1`; the production caller that
    /// drives generation lands in `.7.2.2`. Exercised now by unit tests.
    #[allow(dead_code)]
    fn compute_reach_path(
        &self,
        entry_rule: &str,
        target_rule: &str,
        target_node_path: &str,
        target_branch_index: usize,
    ) -> Option<Vec<ReachDirective>> {
        // Validate the target OR node + branch index up front.
        let alternatives = self.or_alternatives_for_group_path(target_rule, target_node_path)?;
        if target_branch_index >= alternatives.len() {
            return None;
        }

        // BFS over the rule-reference graph; record each rule's predecessor +
        // the directives that realise the discovering hop.
        use std::collections::VecDeque;
        struct Discovery {
            predecessor: Option<String>,
            hop_directives: Vec<ReachDirective>,
        }
        let mut discovered: HashMap<String, Discovery> = HashMap::new();
        discovered.insert(
            entry_rule.to_string(),
            Discovery {
                predecessor: None,
                hop_directives: Vec::new(),
            },
        );
        let mut queue: VecDeque<String> = VecDeque::new();
        queue.push_back(entry_rule.to_string());

        while let Some(rule_name) = queue.pop_front() {
            if rule_name == target_rule {
                break;
            }
            let Some(rule_node) = self.grammar_tree.get(rule_name.as_str()) else {
                continue;
            };
            let mut sites: Vec<RuleReferenceSite> = Vec::new();
            Self::collect_rule_reference_sites(rule_node, "root", &mut sites);
            for site in sites {
                if !self.grammar_tree.contains_key(site.referenced_rule.as_str()) {
                    continue;
                }
                if discovered.contains_key(site.referenced_rule.as_str()) {
                    continue;
                }
                let hop_directives =
                    Self::directives_along_path(&rule_name, &site.node_path);
                discovered.insert(
                    site.referenced_rule.clone(),
                    Discovery {
                        predecessor: Some(rule_name.clone()),
                        hop_directives,
                    },
                );
                queue.push_back(site.referenced_rule);
            }
        }

        // Target rule must have been reached.
        if !discovered.contains_key(target_rule) {
            return None;
        }

        // Walk predecessors target→entry, collecting each hop's directives, then
        // reverse to entry→target order.
        let mut chain: Vec<ReachDirective> = Vec::new();
        let mut cursor = target_rule.to_string();
        loop {
            let Some(discovery) = discovered.get(&cursor) else {
                break;
            };
            for directive in discovery.hop_directives.iter().rev() {
                chain.push(directive.clone());
            }
            match &discovery.predecessor {
                Some(pred) => cursor = pred.clone(),
                None => break,
            }
        }
        chain.reverse();

        // Navigate WITHIN the target rule: if the target OR node is nested under
        // other OR nodes in the target rule's body, those outer choices must be
        // forced too (the cross-rule hops only get us as far as entering the
        // target rule). directives_along_path emits one directive per `o{i}`
        // segment crossed on the way to the target OR node.
        chain.extend(Self::directives_along_path(target_rule, target_node_path));

        // Finally force the target branch at the target OR node itself.
        chain.push(ReachDirective {
            rule_name: target_rule.to_string(),
            node_path: target_node_path.to_string(),
            branch_index: target_branch_index,
        });

        Some(chain)
    }

    /// SV-EXH-PROOF.7.2.19 (PGEN-SV-EXH-PROOF-0134, ANALYSIS ONLY — no generation
    /// change): translate a never-hit RULE target into a BRANCH target the
    /// (branch-only) reach machinery can steer to. "Reach rule R" = "enter R at
    /// least once"; R is entered when the OR-decision that introduces R selects the
    /// alternative referencing R. So we find that OR site and return it as a branch
    /// target `(via_rule, node_path, branch_index)` such that
    /// `compute_reach_path(entry, via_rule, node_path, branch_index)` (UNCHANGED)
    /// steers generation into R.
    ///
    /// Algorithm (reuses the exact `compute_reach_path` BFS over the rule-reference
    /// graph, so it inherits its determinism + cycle-safety): BFS from `entry_rule`
    /// recording, per discovered rule, the `(predecessor_rule, reference-site
    /// node_path)` it was discovered through. To produce a branch target for
    /// `target_rule`, walk up that discovery chain to the FIRST hop whose
    /// reference-site path contains an `o{i}` (OR) segment — the deepest such `o{i}`
    /// on that path is the OR node + alternative whose selection enters the next
    /// rule on the way to `target_rule`. Return `(that hop's predecessor rule, the
    /// OR node's group path, the alternative index)`.
    ///
    /// If NO hop on the chain crosses an OR (every introduction is in a plain
    /// sequence, i.e. unconditionally taken once the parent is entered), then
    /// `target_rule` is generated whenever the entry is — there is no branch to
    /// force and a rule-reach plan is neither possible nor needed; returns `None`.
    /// Also `None` if `target_rule == entry_rule` (already the root) or is
    /// graph-unreachable. GENERAL/parser-agnostic — keyed only on the reference
    /// graph + path encoding, no rule-name special-casing.
    ///
    /// PURE analysis: returns a descriptor only; performs NO generation and mutates
    /// nothing. The driver wiring that consumes it is a SEPARATE leaf (`.7.2.20`).
    #[allow(dead_code)]
    fn compute_rule_reach_target(
        &self,
        entry_rule: &str,
        target_rule: &str,
    ) -> Option<(String, String, usize)> {
        if target_rule == entry_rule {
            return None;
        }
        if !self.grammar_tree.contains_key(target_rule) {
            return None;
        }

        use std::collections::VecDeque;
        // discovered[rule] = (predecessor_rule, reference-site node_path in predecessor)
        let mut discovered: HashMap<String, Option<(String, String)>> = HashMap::new();
        discovered.insert(entry_rule.to_string(), None);
        let mut queue: VecDeque<String> = VecDeque::new();
        queue.push_back(entry_rule.to_string());

        while let Some(rule_name) = queue.pop_front() {
            if rule_name == target_rule {
                break;
            }
            let Some(rule_node) = self.grammar_tree.get(rule_name.as_str()) else {
                continue;
            };
            let mut sites: Vec<RuleReferenceSite> = Vec::new();
            Self::collect_rule_reference_sites(rule_node, "root", &mut sites);
            for site in sites {
                if !self.grammar_tree.contains_key(site.referenced_rule.as_str()) {
                    continue;
                }
                if discovered.contains_key(site.referenced_rule.as_str()) {
                    continue;
                }
                discovered.insert(
                    site.referenced_rule.clone(),
                    Some((rule_name.clone(), site.node_path.clone())),
                );
                queue.push_back(site.referenced_rule);
            }
        }

        if !discovered.contains_key(target_rule) {
            return None; // graph-unreachable from entry
        }

        // Walk the discovery chain target → entry; the FIRST hop (closest to the
        // target) whose reference-site path crosses an OR gives the branch to force.
        let mut cursor = target_rule.to_string();
        loop {
            let Some(Some((pred_rule, site_path))) = discovered.get(&cursor) else {
                break; // reached entry (predecessor None) — no OR on the whole chain
            };
            if let Some((or_node_path, branch_index)) =
                Self::deepest_or_on_path(site_path)
            {
                return Some((pred_rule.clone(), or_node_path, branch_index));
            }
            cursor = pred_rule.clone();
        }
        None
    }

    /// SV-EXH-PROOF.7.2.19: given a reference-site `node_path` (the
    /// `collect_branch_groups` encoding), return the DEEPEST `o{i}` (OR
    /// alternative) crossed on it as `(or_node_group_path, branch_index)` — i.e.
    /// the last OR choice on the way to the site, whose `or_node_group_path` is the
    /// prefix up to (not including) that `o{i}` segment. `None` if the path crosses
    /// no OR (the reference is unconditional within its parent).
    fn deepest_or_on_path(node_path: &str) -> Option<(String, usize)> {
        let mut prefix = String::from("root");
        let mut deepest: Option<(String, usize)> = None;
        for segment in node_path.split('/') {
            if segment.is_empty() || segment == "root" {
                continue;
            }
            if let Some(index_str) = segment.strip_prefix('o') {
                if let Ok(index) = index_str.parse::<usize>() {
                    deepest = Some((prefix.clone(), index));
                }
            }
            prefix.push('/');
            prefix.push_str(segment);
        }
        deepest
    }

    /// SV-EXH-PROOF.7.2.1: walk `node_path` (the `collect_branch_groups`
    /// encoding) from a rule's root, emitting a `ReachDirective` for every
    /// `o{i}` (Or-alternative) segment crossed — i.e. the OR choices that must be
    /// forced to follow this structural path to the reference site. Non-OR
    /// segments (`s{i}`, `q`, `l`, `a`) carry no branch choice. The `node_path`
    /// prefix accumulated up to each `o{i}` is the OR node's own group path.
    ///
    /// `dead_code`-allowed: analysis-only in `.7.2.1`; the production caller that
    /// drives generation lands in `.7.2.2`. Exercised now by unit tests.
    #[allow(dead_code)]
    fn directives_along_path(rule_name: &str, node_path: &str) -> Vec<ReachDirective> {
        let mut directives = Vec::new();
        let mut prefix = String::from("root");
        for segment in node_path.split('/') {
            if segment.is_empty() || segment == "root" {
                continue;
            }
            if let Some(index_str) = segment.strip_prefix('o') {
                if let Ok(index) = index_str.parse::<usize>() {
                    directives.push(ReachDirective {
                        rule_name: rule_name.to_string(),
                        node_path: prefix.clone(),
                        branch_index: index,
                    });
                }
            }
            prefix.push('/');
            prefix.push_str(segment);
        }
        directives
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
                        failure_reasons: vec![HashMap::new(); alternatives.len()],
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
            ASTNode::Lookahead { element, .. } => {
                let lookahead_path = format!("{}/l", node_path);
                Self::collect_branch_groups(rule_name, element, &lookahead_path, groups);
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
        self.reset_mutation_runtime_state();
        let previous_active_entry = self
            .active_generation_entry_rule
            .replace(entry_rule.to_string());
        let result = self
            .generate_entry_with_configured_modes(entry_rule)
            .map(|sample| self.apply_negative_case_policy(entry_rule, sample));
        self.active_generation_entry_rule = previous_active_entry;
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

    fn generate_entry_with_configured_modes(&mut self, entry_rule: &str) -> Result<String> {
        if self.config.mutation_mode == StimuliMutationMode::GrammarAwareLocal
            && self.config.recovery_mode == RecoveryStimuliMode::Baseline
        {
            return self.generate_entry_with_local_grammar_mutation(entry_rule);
        }

        let mut call_stack = Vec::new();
        self.generate_entry_core(entry_rule, &mut call_stack)
    }

    fn generate_entry_core(
        &mut self,
        entry_rule: &str,
        call_stack: &mut Vec<String>,
    ) -> Result<String> {
        match self.config.recovery_mode {
            RecoveryStimuliMode::Baseline => self.generate_rule(entry_rule, 0, call_stack),
            RecoveryStimuliMode::RecoveryBiased => {
                self.generate_recovery_biased_entry(entry_rule, call_stack)
            }
            RecoveryStimuliMode::NearSyncNegative => {
                self.generate_near_sync_negative_entry(entry_rule, call_stack)
            }
        }
    }

    fn generate_entry_with_local_grammar_mutation(&mut self, entry_rule: &str) -> Result<String> {
        let coverage_before = self.coverage.clone();
        let deterministic_counters_after_activation = self.deterministic_partition_counters.clone();
        let rng_after_activation = self.rng.clone();

        self.reset_mutation_runtime_state();
        self.mutation_trace = Some(StimuliDecisionTrace::default());

        let mut baseline_call_stack = Vec::new();
        let baseline_result = self.generate_entry_core(entry_rule, &mut baseline_call_stack);
        let baseline_coverage_after = self.coverage.clone();
        let baseline_rng_after = self.rng.clone();
        let baseline_trace = self.mutation_trace.take().unwrap_or_default();
        self.mutation_replay = None;
        self.mutation_site_visit_counters.clear();

        let baseline_sample = match baseline_result {
            Ok(sample) => sample,
            Err(err) => {
                self.mutation_replay = None;
                return Err(err);
            }
        };

        let mut selections = self.build_grammar_mutation_selections(&baseline_trace);
        if selections.is_empty() {
            return Ok(baseline_sample);
        }

        let mut selection_rng = baseline_rng_after.clone();
        selections.shuffle(&mut selection_rng);

        for selection in selections {
            self.coverage = coverage_before.clone();
            self.deterministic_partition_counters = deterministic_counters_after_activation.clone();
            self.rng = rng_after_activation.clone();
            self.mutation_site_visit_counters.clear();
            self.mutation_replay = Some(ActiveGrammarMutationReplay {
                baseline_trace: baseline_trace.clone(),
                selection,
            });

            let mut replay_call_stack = Vec::new();
            let replay_result = self.generate_entry_core(entry_rule, &mut replay_call_stack);
            self.mutation_replay = None;
            self.mutation_site_visit_counters.clear();

            if let Ok(mutated_sample) = replay_result {
                if mutated_sample != baseline_sample {
                    return Ok(mutated_sample);
                }
            }
        }

        self.coverage = baseline_coverage_after;
        self.deterministic_partition_counters = deterministic_counters_after_activation;
        self.rng = baseline_rng_after;
        Ok(baseline_sample)
    }

    fn build_grammar_mutation_selections(
        &self,
        trace: &StimuliDecisionTrace,
    ) -> Vec<GrammarMutationSelection> {
        let mut selections = Vec::new();
        for candidate in &trace.mutation_candidates {
            match &candidate.kind {
                GrammarMutationCandidateKind::Or {
                    alternative_branches,
                    ..
                } => {
                    for branch in alternative_branches {
                        selections.push(GrammarMutationSelection::Or {
                            site_key: candidate.site_key.clone(),
                            forced_branch: *branch,
                        });
                    }
                }
                GrammarMutationCandidateKind::Quantifier {
                    alternative_repeats,
                    ..
                } => {
                    for repeats in alternative_repeats {
                        selections.push(GrammarMutationSelection::Quantifier {
                            site_key: candidate.site_key.clone(),
                            forced_repeats: *repeats,
                        });
                    }
                }
            }
        }
        selections
    }

    fn reset_mutation_runtime_state(&mut self) {
        self.mutation_trace = None;
        self.mutation_replay = None;
        self.mutation_site_visit_counters.clear();
    }

    fn next_mutation_site_key(
        &mut self,
        current_rule: &str,
        node_path: &str,
        kind: &str,
    ) -> String {
        let base_key = format!("{}::{}::{}", current_rule, node_path, kind);
        let ordinal = {
            let counter = self
                .mutation_site_visit_counters
                .entry(base_key.clone())
                .or_insert(0);
            let current = *counter;
            *counter = counter.saturating_add(1);
            current
        };
        format!("{}#{}", base_key, ordinal)
    }

    fn forced_or_branch_for_site(&self, site_key: &str) -> Option<(usize, Option<usize>)> {
        let replay = self.mutation_replay.as_ref()?;
        let baseline = replay.baseline_trace.or_choices.get(site_key).copied();
        match &replay.selection {
            GrammarMutationSelection::Or {
                site_key: selected_site_key,
                forced_branch,
            } if selected_site_key == site_key => Some((*forced_branch, baseline)),
            _ => baseline.map(|branch| (branch, baseline)),
        }
    }

    fn forced_quantifier_repeats_for_site(&self, site_key: &str) -> Option<(usize, Option<usize>)> {
        let replay = self.mutation_replay.as_ref()?;
        let baseline = replay
            .baseline_trace
            .quantifier_repeats
            .get(site_key)
            .copied();
        match &replay.selection {
            GrammarMutationSelection::Quantifier {
                site_key: selected_site_key,
                forced_repeats,
            } if selected_site_key == site_key => Some((*forced_repeats, baseline)),
            _ => baseline.map(|repeats| (repeats, baseline)),
        }
    }

    fn record_or_mutation_choice(
        &mut self,
        site_key: &str,
        selected_branch: usize,
        candidate_indices: &[usize],
    ) {
        let Some(trace) = self.mutation_trace.as_mut() else {
            return;
        };
        trace
            .or_choices
            .insert(site_key.to_string(), selected_branch);
        let alternative_branches: Vec<usize> = candidate_indices
            .iter()
            .copied()
            .filter(|branch| *branch != selected_branch)
            .collect();
        if !alternative_branches.is_empty() {
            trace.mutation_candidates.push(GrammarMutationCandidate {
                site_key: site_key.to_string(),
                kind: GrammarMutationCandidateKind::Or {
                    alternative_branches,
                },
            });
        }
    }

    fn record_quantifier_mutation_choice(
        &mut self,
        site_key: &str,
        chosen_repeats: usize,
        min_repeat: usize,
        bounded_max: usize,
    ) {
        let Some(trace) = self.mutation_trace.as_mut() else {
            return;
        };
        trace
            .quantifier_repeats
            .insert(site_key.to_string(), chosen_repeats);
        let alternative_repeats: Vec<usize> = (min_repeat..=bounded_max)
            .filter(|repeats| *repeats != chosen_repeats)
            .collect();
        if !alternative_repeats.is_empty() {
            trace.mutation_candidates.push(GrammarMutationCandidate {
                site_key: site_key.to_string(),
                kind: GrammarMutationCandidateKind::Quantifier {
                    alternative_repeats,
                },
            });
        }
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
        if rule_name == "epsilon" {
            self.trace(
                TraceLevel::Debug,
                format_args!("Builtin epsilon expansion: depth={}", depth),
            );
            return Ok(String::new());
        }
        self.enforce_generation_deadline(rule_name, "rule")?;
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

        if Self::node_supports_rule_literal_override(rule_node) {
            if let Some(sample_hint) = self
                .literalish_hint_for_rule(rule_name)
                .or_else(|| self.probe_literalish_hint_for_rule(rule_name))
                .filter(|hint| !self.hint_collides_with_active_closer(hint))
            {
                self.trace(
                    TraceLevel::Debug,
                    format_args!(
                        "Rule-level literal hint override: rule='{}' depth={} sample='{}'",
                        rule_name, depth, sample_hint
                    ),
                );
                self.coverage.record_rule_success(rule_name);
                return Ok(sample_hint);
            }
        }

        call_stack.push(rule_name.to_string());
        // STIMULI-SIGNOFF.2.2: record the covered k-path (last-k window of the live call
        // stack). OFF by default (zero overhead / byte-identical generation); read-only.
        if let Some((k, set)) = self.k_path_recording.as_mut() {
            let n = call_stack.len();
            if n >= *k {
                set.insert(call_stack[n - *k..].to_vec());
            }
        }
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
        self.enforce_generation_deadline(current_rule, node_path)?;
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
            ASTNode::Lookahead { .. } => Ok(String::new()),
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
        self.enforce_generation_deadline(current_rule, node_path)?;
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

        // SV-EXH-PROOF.7.4.6.5: Cow — borrow each alternative in the common (no probability
        // prefix) case instead of deep-cloning all of them (the profiled construction hot spot).
        let prepared: Vec<(Option<u32>, std::borrow::Cow<ASTNode>)> = alternatives
            .iter()
            .map(|node| self.strip_probability_prefix(node))
            .collect();

        let mut candidate_indices: Vec<usize> = (0..prepared.len()).collect();

        if depth >= self.config.max_depth.saturating_sub(1) {
            let min_ref_count = candidate_indices
                .iter()
                .map(|idx| self.count_rule_references(prepared[*idx].1.as_ref(), current_rule))
                .min()
                .unwrap_or(0);

            candidate_indices.retain(|idx| {
                self.count_rule_references(prepared[*idx].1.as_ref(), current_rule) == min_ref_count
            });
        }

        candidate_indices.retain(|idx| {
            let missing = self.missing_rule_references(prepared[*idx].1.as_ref());
            if missing.is_empty() {
                true
            } else if prepared[*idx].0.is_some() {
                self.trace(
                    TraceLevel::Debug,
                    format_args!(
                        "Kept OR branch with missing rule references because it carries an explicit probability (retry loop will handle): rule='{}' path='{}' branch={} missing={:?}",
                        current_rule, node_path, idx, missing
                    ),
                );
                true
            } else {
                self.trace(
                    TraceLevel::Debug,
                    format_args!(
                        "Pruned OR branch with missing rule references: rule='{}' path='{}' branch={} missing={:?}",
                        current_rule, node_path, idx, missing
                    ),
                );
                false
            }
        });

        // SV-EXH-PROOF.7.2.2 (PGEN-SV-EXH-PROOF-0116): on-path candidate-set
        // bypass. If a reach plan is active AND this OR site is on the reach path
        // AND the forced target branch was pruned out above (depth-floor /
        // missing-rule), re-admit it — but ONLY on-path and ONLY while
        // bypass_fuel remains, so off-path nodes keep normal pruning and a
        // cyclic/pathological path cannot force unbounded depth. This is the
        // phase-1 intervention the .7.1 root-cause analysis identified: a
        // never_selected target is dropped from the candidate set BEFORE the
        // phase-2 weighting, so weighting alone can never reach it.
        // Does an active reach plan force a branch here that pruning removed, and
        // is there bypass fuel left to re-admit it? Resolve to a single Option so
        // the apply step is one flat block (no nested-if pyramid).
        let reach_bypass_branch: Option<usize> = self.reach_plan.as_ref().and_then(|plan| {
            plan.forced_branch_for(current_rule, node_path)
                .filter(|forced_branch| {
                    plan.bypass_fuel > 0
                        && *forced_branch < prepared.len()
                        && !candidate_indices.contains(forced_branch)
                })
        });
        if let Some(forced_branch) = reach_bypass_branch {
            if let Some(plan) = self.reach_plan.as_mut() {
                plan.bypass_fuel = plan.bypass_fuel.saturating_sub(1);
            }
            // Re-admit the forced branch, keeping candidate_indices sorted so
            // downstream local/global index bookkeeping stays stable.
            candidate_indices.push(forced_branch);
            candidate_indices.sort_unstable();
            candidate_indices.dedup();
            self.trace(
                TraceLevel::Debug,
                format_args!(
                    "Reach-plan candidate bypass: rule='{}' path='{}' re-admitted forced branch={}",
                    current_rule, node_path, forced_branch
                ),
            );
        }

        if candidate_indices.is_empty() {
            if let Some(recovery_sample) = self.recovery_stimulus_fallback(current_rule) {
                self.trace(
                    TraceLevel::Low,
                    format_args!(
                        "OR fallback recovery used after all branches pruned: rule='{}' path='{}' fallback_len={}",
                        current_rule,
                        node_path,
                        recovery_sample.len()
                    ),
                );
                return Ok(recovery_sample);
            }
            return Err(anyhow!(
                "No candidate branches available for rule '{}' during stimuli generation",
                current_rule
            ));
        }

        let group_key = format!("{}::{}", current_rule, node_path);
        let mutation_site_key = self.next_mutation_site_key(current_rule, node_path, "or");
        let (branch_policy, associativity, branch_priorities) =
            self.rule_branch_controls(current_rule, prepared.len());
        // SV-EXH-PROOF.7.2.2 (PGEN-SV-EXH-PROOF-0116): reach-plan forcing takes
        // priority over every other attempt-ordering source. If this OR site is
        // on the active reach path and its forced branch survived candidate
        // filtering (always true once the .7.2.1 chain + the on-path bypass
        // admit it), try that branch FIRST; the remaining branches follow in
        // natural order as fallbacks (so generation still terminates if the
        // forced branch fails downstream). When no reach plan is active this
        // whole arm is skipped and behavior is identical to before.
        let reach_forced_local: Option<usize> = self
            .reach_plan
            .as_ref()
            .and_then(|plan| plan.forced_branch_for(current_rule, node_path))
            .and_then(|forced_global| {
                candidate_indices
                    .iter()
                    .position(|global_idx| *global_idx == forced_global)
            });
        let attempt_order: Vec<usize> = if let Some(forced_local) = reach_forced_local {
            let mut ordered = Vec::with_capacity(candidate_indices.len());
            ordered.push(forced_local);
            for local_idx in 0..candidate_indices.len() {
                if local_idx != forced_local {
                    ordered.push(local_idx);
                }
            }
            self.trace(
                TraceLevel::Debug,
                format_args!(
                    "Reach-plan branch forcing: rule='{}' path='{}' forced_local={} forced_global={}",
                    current_rule, node_path, forced_local, candidate_indices[forced_local]
                ),
            );
            ordered
        } else if self.witness_mode && self.witness_min_terminal_lengths.is_some() {
            // SV-EXH-PROOF.7.4.4 (PGEN-SV-EXH-PROOF-0143): Purdom shortest-derivation
            // ordering — witness pass ONLY (gated by `witness_mode`; reach-plan forcing
            // above still wins on-path, so branch targets are unaffected). Order the
            // surviving candidates by ASCENDING min-terminal-length (the `.7.4.2` table) so
            // the shortest-terminating alternative is tried first and the witness converges
            // fast instead of timing out. Pure attempt-ORDER change — every other branch
            // stays as a fallback, so correctness is unaffected; unresolvable nodes (`None`)
            // sort last; ties broken by global index for determinism.
            let mut ordered: Vec<usize> = (0..candidate_indices.len()).collect();
            if let Some(table) = self.witness_min_terminal_lengths.as_ref() {
                ordered.sort_by(|left, right| {
                    let left_min = Self::min_terminal_length_of_node(
                        prepared[candidate_indices[*left]].1.as_ref(),
                        table,
                    )
                    .unwrap_or(usize::MAX);
                    let right_min = Self::min_terminal_length_of_node(
                        prepared[candidate_indices[*right]].1.as_ref(),
                        table,
                    )
                    .unwrap_or(usize::MAX);
                    left_min
                        .cmp(&right_min)
                        .then_with(|| candidate_indices[*left].cmp(&candidate_indices[*right]))
                });
            }
            // SV-EXH-PROOF.7.4.6.3: derivation CONSTRUCTION — commit to the single shortest
            // (min-length) branch with NO backtracking. The min-length-table-guided path is a
            // complete valid minimal derivation, so dropping the fallbacks makes the witness
            // bounded (O(tree-size), no search) instead of timing out. Reach-plan forcing
            // above still wins on-path (branch targets unaffected); if the committed path
            // fails downstream the witness errors and the caller falls back to search.
            if self.construct_mode {
                ordered.truncate(1);
            }
            ordered
        } else if let Some((preferred_global, baseline_global)) =
            self.forced_or_branch_for_site(&mutation_site_key)
        {
            let mut ordered = Vec::with_capacity(candidate_indices.len());
            if let Some(local_idx) = candidate_indices
                .iter()
                .position(|global_idx| *global_idx == preferred_global)
            {
                ordered.push(local_idx);
            }
            if let Some(baseline_global) = baseline_global {
                if baseline_global != preferred_global {
                    if let Some(local_idx) = candidate_indices
                        .iter()
                        .position(|global_idx| *global_idx == baseline_global)
                    {
                        if !ordered.contains(&local_idx) {
                            ordered.push(local_idx);
                        }
                    }
                }
            }
            for local_idx in 0..candidate_indices.len() {
                if !ordered.contains(&local_idx) {
                    ordered.push(local_idx);
                }
            }
            ordered
        } else {
            match branch_policy {
                SemanticBranchPolicy::Ordered => (0..candidate_indices.len()).collect(),
                SemanticBranchPolicy::PriorityFirst => {
                    let mut ordered: Vec<usize> = (0..candidate_indices.len()).collect();
                    ordered.sort_by(|left, right| {
                        let left_global = candidate_indices[*left];
                        let right_global = candidate_indices[*right];
                        let left_target_probe =
                            self.target_priority_probe_bias(&group_key, left_global);
                        let right_target_probe =
                            self.target_priority_probe_bias(&group_key, right_global);
                        let left_priority =
                            branch_priorities.get(left_global).copied().unwrap_or(0);
                        let right_priority =
                            branch_priorities.get(right_global).copied().unwrap_or(0);
                        right_target_probe
                            .cmp(&left_target_probe)
                            .then_with(|| {
                                if left_target_probe || right_target_probe {
                                    let left_deficit =
                                        self.branch_target_deficit(&group_key, left_global);
                                    let right_deficit =
                                        self.branch_target_deficit(&group_key, right_global);
                                    let left_success =
                                        self.branch_success_hits(&group_key, left_global);
                                    let right_success =
                                        self.branch_success_hits(&group_key, right_global);
                                    let left_selected =
                                        self.branch_selected_hits(&group_key, left_global);
                                    let right_selected =
                                        self.branch_selected_hits(&group_key, right_global);

                                    right_deficit
                                        .cmp(&left_deficit)
                                        .then_with(|| left_selected.cmp(&right_selected))
                                        .then_with(|| left_success.cmp(&right_success))
                                } else {
                                    std::cmp::Ordering::Equal
                                }
                            })
                            .then_with(|| right_priority.cmp(&left_priority))
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
                                prepared[*global_idx].1.as_ref(),
                            );
                            let recursion_penalty = self.recursion_pressure_penalty(
                                prepared[*global_idx].1.as_ref(),
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
                            let constraint_multiplier = self.constraint_profile_branch_multiplier(
                                current_rule,
                                node_path,
                                *global_idx,
                                prepared[*global_idx].1.as_ref(),
                                depth,
                                call_stack,
                            );
                            u64::from(base_weights[local_idx])
                                .saturating_mul(adjusted_multiplier)
                                .saturating_mul(semantic_multiplier)
                                .saturating_mul(constraint_multiplier)
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
            let selected_node = prepared[selected_global].1.as_ref().clone();
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
                    "Trying OR branch: rule='{}' path='{}' site='{}' local_branch={} global_branch={}",
                    current_rule, node_path, mutation_site_key, local_idx, selected_global
                ),
            );
            if let Some(sample_hint) = self
                .literalish_hint_for_branch(current_rule, selected_global)
                .or_else(|| self.probe_literalish_hint_for_branch(current_rule, selected_global))
                .filter(|hint| !self.hint_collides_with_active_closer(hint))
            {
                self.record_or_mutation_choice(
                    &mutation_site_key,
                    selected_global,
                    &candidate_indices,
                );
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
                        "Selected OR branch literal hint: rule='{}' path='{}' branch={} output_len={}",
                        current_rule,
                        node_path,
                        selected_global,
                        sample_hint.len()
                    ),
                );
                return Ok(sample_hint);
            }
            let alt_path = format!("{}/o{}", node_path, selected_global);
            match self.generate_node(&selected_node, current_rule, depth, call_stack, &alt_path) {
                Ok(output) => {
                    self.record_or_mutation_choice(
                        &mutation_site_key,
                        selected_global,
                        &candidate_indices,
                    );
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
                    if let Some(depth_retry_slack) =
                        self.target_branch_depth_retry_slack(&group_key, selected_global, &err)
                    {
                        self.trace(
                            TraceLevel::Debug,
                            format_args!(
                                "Retrying targeted OR branch with temporary depth slack: rule='{}' path='{}' branch={} retry_max_depth={}",
                                current_rule,
                                node_path,
                                selected_global,
                                self.config.max_depth.saturating_add(depth_retry_slack)
                            ),
                        );
                        let original_max_depth = self.config.max_depth;
                        self.config.max_depth =
                            original_max_depth.saturating_add(depth_retry_slack);
                        let retry_result = self.generate_node(
                            &selected_node,
                            current_rule,
                            depth,
                            call_stack,
                            &alt_path,
                        );
                        self.config.max_depth = original_max_depth;

                        match retry_result {
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
                                        "Selected OR branch after depth-slack retry: rule='{}' path='{}' branch={} output_len={} retry_max_depth={}",
                                        current_rule,
                                        node_path,
                                        selected_global,
                                        output.len(),
                                        original_max_depth.saturating_add(depth_retry_slack)
                                    ),
                                );
                                return Ok(output);
                            }
                            Err(retry_err) => {
                                self.coverage.record_branch_failure(
                                    &group_key,
                                    current_rule,
                                    node_path,
                                    alternatives.len(),
                                    selected_global,
                                    &retry_err.to_string(),
                                );
                                self.trace(
                                    TraceLevel::Debug,
                                    format_args!(
                                        "OR branch failed after depth-slack retry: rule='{}' path='{}' branch={} reason={}",
                                        current_rule, node_path, selected_global, retry_err
                                    ),
                                );
                                last_error = Some(retry_err);
                                continue;
                            }
                        }
                    }
                    self.coverage.record_branch_failure(
                        &group_key,
                        current_rule,
                        node_path,
                        alternatives.len(),
                        selected_global,
                        &err.to_string(),
                    );
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

    /// SV-EXH-PROOF.2.3.2: generate one sequence element, but when it
    /// is the force-line-terminator slot (`force_nl_idx`) generate
    /// the quantified element's INNER exactly once — guaranteeing the
    /// optional newline terminator is emitted so a preceding
    /// line-greedy content terminal cannot absorb the following
    /// structural element on reparse. Otherwise behaves exactly like
    /// `generate_node` on the element. Parser/EBNF-agnostic.
    fn generate_sequence_element(
        &mut self,
        elements: &[ASTNode],
        idx: usize,
        force_nl_idx: Option<usize>,
        current_rule: &str,
        depth: usize,
        call_stack: &mut Vec<String>,
        element_path: &str,
    ) -> Result<String> {
        if Some(idx) == force_nl_idx {
            if let ASTNode::Quantified { element, .. } = &elements[idx] {
                return self.generate_node(
                    element,
                    current_rule,
                    depth,
                    call_stack,
                    element_path,
                );
            }
        }
        self.generate_node(
            &elements[idx],
            current_rule,
            depth,
            call_stack,
            element_path,
        )
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

        // SV-EXH-PROOF.2.3.2 (line-terminator completeness): if this
        // sequence is `… <line-greedy content> … <optional newline
        // terminator>`, the trailing optional newline must be
        // force-emitted (generated as exactly one inner repetition)
        // so the line-greedy content cannot absorb the following
        // structural element on reparse. Derived from regex HIR +
        // sequence shape ⇒ parser/EBNF-agnostic, all-lanes-safe.
        let force_nl_idx =
            Self::sequence_force_line_terminator_idx(self.grammar_tree, elements);

        if relational_policy.constraint_expression.is_none() {
            let mut output = String::new();
            // SV-EXH-PROOF.2.3.2 (Mode B): if this sequence is the
            // `… item* CLOSE` idiom (a quantified/optional body region
            // followed by a *required fixed-literal structural
            // closer*), the closer's lexeme must be unspellable by any
            // *free* terminal materialized while the pre-closer body
            // (incl. recursive descent) is generated — else the parser
            // consumes that text (first-match) as body content and the
            // construct never closes (sample fails its own
            // round-trip). Pushed only over `[0, closer)`, popped
            // before the closer element itself (fixed-literal closers
            // are exempt anyway ⇒ nesting-safe; empty stack when no
            // such construct is open ⇒ the lexeme stays freely
            // generatable standalone ⇒ coverage-preserving). Derived
            // from grammar structure + terminal HIR ⇒
            // parser/EBNF-agnostic, all-lanes-safe.
            let closer = match Self::sequence_closer_split(self.grammar_tree, elements) {
                Some((idx, lex)) if self.closer_lexeme_is_structural_hazard(&lex) => Some((idx, lex)),
                _ => None,
            };
            if let Some((closer_idx, closer_lexeme)) = closer {
                self.trace(
                    TraceLevel::Low,
                    format_args!(
                        "SV-EXH-PROOF.2.3.2 closer-scope ENTER (pure): rule='{}' path='{}' closer_idx={} lexeme={:?}",
                        current_rule, node_path, closer_idx, closer_lexeme
                    ),
                );
                self.structural_closer_forbidden.push(closer_lexeme);
                self.closer_scopes_entered += 1;
                let mut body_err: Option<anyhow::Error> = None;
                for idx in 0..closer_idx {
                    let element_path = format!("{}/s{}", node_path, idx);
                    match self.generate_sequence_element(
                        elements,
                        idx,
                        force_nl_idx,
                        current_rule,
                        depth,
                        call_stack,
                        &element_path,
                    ) {
                        Ok(generated) => self.append_generated_segment(&mut output, &generated),
                        Err(err) => {
                            body_err = Some(err);
                            break;
                        }
                    }
                }
                self.structural_closer_forbidden.pop();
                if let Some(err) = body_err {
                    return Err(err);
                }
                for idx in closer_idx..elements.len() {
                    let element_path = format!("{}/s{}", node_path, idx);
                    let generated = self.generate_sequence_element(
                        elements,
                        idx,
                        force_nl_idx,
                        current_rule,
                        depth,
                        call_stack,
                        &element_path,
                    )?;
                    self.append_generated_segment(&mut output, &generated);
                }
                return Ok(output);
            }
            for idx in 0..elements.len() {
                let element_path = format!("{}/s{}", node_path, idx);
                let generated = self.generate_sequence_element(
                    elements,
                    idx,
                    force_nl_idx,
                    current_rule,
                    depth,
                    call_stack,
                    &element_path,
                )?;
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
            self.enforce_generation_deadline(current_rule, node_path)?;
            let mut output = String::new();
            let mut captures = Vec::with_capacity(elements.len());
            let mut named_captures = HashMap::new();

            // SV-EXH-PROOF.2.3.2 (Mode B): identical closer-scoping as
            // the pure path, applied per relational attempt. Pushed at
            // attempt start, popped exactly when the closer element is
            // reached (or on early failure / attempt end) so the stack
            // is balanced across every exit, retry, and discard.
            let closer = match Self::sequence_closer_split(self.grammar_tree, elements) {
                Some((idx, lex)) if self.closer_lexeme_is_structural_hazard(&lex) => Some((idx, lex)),
                _ => None,
            };
            let mut closer_active = false;
            if let Some((_, closer_lexeme)) = &closer {
                self.trace(
                    TraceLevel::Low,
                    format_args!(
                        "SV-EXH-PROOF.2.3.2 closer-scope ENTER (relational): rule='{}' path='{}' lexeme={:?}",
                        current_rule, node_path, closer_lexeme
                    ),
                );
                self.structural_closer_forbidden.push(closer_lexeme.clone());
                self.closer_scopes_entered += 1;
                closer_active = true;
            }
            let mut generation_failed = false;
            for (idx, element) in elements.iter().enumerate() {
                if closer_active {
                    if let Some((closer_idx, _)) = &closer {
                        if idx == *closer_idx {
                            self.structural_closer_forbidden.pop();
                            closer_active = false;
                        }
                    }
                }
                let element_path = format!("{}/s{}", node_path, idx);
                let capture_name = Self::sequence_element_capture_name(element);
                match self.generate_sequence_element(
                    elements,
                    idx,
                    force_nl_idx,
                    current_rule,
                    depth,
                    call_stack,
                    &element_path,
                ) {
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
            if closer_active {
                self.structural_closer_forbidden.pop();
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
                        let mut sample =
                            self.generate_regex_sample(&effective_pattern, current_rule);
                        // SV-EXH-PROOF.2.3.2 (Mode B): consult/discard.
                        // A *free* terminal (variable-HIR regex —
                        // class/repetition/alternation) materialized
                        // while a closer-bearing construct is OPEN
                        // (its required fixed-literal closer is on the
                        // `structural_closer_forbidden` stack) must
                        // not yield text *containing* an active closer
                        // lexeme: the parser would consume it
                        // (first-match) as body content, the construct
                        // would never close, and the generated sample
                        // would fail its own round-trip. Fixed-literal
                        // terminals are exempt (so a legitimately-
                        // nested same-construct's own structural CLOSE
                        // is unaffected ⇒ nesting-safe). On collision
                        // re-materialize a bounded number of times
                        // (free content is stochastic), then DISCARD
                        // the whole attempt via the existing clean-
                        // discard contract (`Err` → the closed-loop
                        // drops it) — never loosen `==0`, never emit a
                        // round-trip-unstable sample. Empty stack
                        // (no open construct) ⇒ this is inert, so the
                        // lexeme stays freely generatable standalone
                        // ⇒ coverage-preserving. Zero grammar
                        // identifiers — the lexeme is derived from the
                        // grammar's own structural shape.
                        if !self.structural_closer_forbidden.is_empty()
                            && Self::regex_fixed_literal(token_value).is_none()
                        {
                            let collides = |s: &str, set: &[String]| {
                                set.iter().any(|c| s.contains(c.as_str()))
                            };
                            if collides(&sample, &self.structural_closer_forbidden) {
                                self.trace(
                                    TraceLevel::Low,
                                    format_args!(
                                        "SV-EXH-PROOF.2.3.2 consult COLLISION: rule='{}' path='{}' token={:?} sample={:?} forbidden={:?}",
                                        current_rule, node_path, token_value, sample, self.structural_closer_forbidden
                                    ),
                                );
                            }
                            let mut tries = 0;
                            while collides(&sample, &self.structural_closer_forbidden) {
                                if tries >= 24 {
                                    self.free_terminal_closer_discards += 1;
                                    return Err(anyhow!(
                                        "SV-EXH-PROOF.2.3.2: free terminal in rule '{}' (path '{}') would emit an active structural-closer lexeme inside an open closer-bearing construct; discarding attempt (round-trip stability)",
                                        current_rule,
                                        node_path
                                    ));
                                }
                                self.enforce_generation_deadline(
                                    current_rule,
                                    node_path,
                                )?;
                                sample = self.generate_regex_sample(
                                    &effective_pattern,
                                    current_rule,
                                );
                                tries += 1;
                            }
                        }
                        // SV-EXH-PROOF.2.3.2: a generation deadline that
                        // fires DURING regex materialization makes
                        // generate_regex_sample / generate_from_regex_hir
                        // return a SILENTLY-TRUNCATED partial — e.g. a
                        // literal `` ` `` prefix (bt_identifier/kw_*/
                        // macro_token_paste/…) emitted, then the
                        // following class/keyword cut to "" — yielding a
                        // structurally-invalid dangling sigil the parser
                        // correctly rejects. Honor the SAME deadline-
                        // DISCARD contract every other generate_node
                        // element already uses
                        // (`enforce_generation_deadline(...)?`): a
                        // timed-out attempt is discarded (Err → the
                        // closed-loop drops the whole attempt), never
                        // emitted as a truncated partial. Parser/EBNF-
                        // agnostic — deadline-handling consistency only,
                        // zero grammar identifiers.
                        self.enforce_generation_deadline(current_rule, node_path)?;
                        Ok(sample)
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
        self.enforce_generation_deadline(current_rule, node_path)?;
        let (min_repeat, max_repeat) = self.parse_quantifier_bounds(quantifier)?;
        let bounded_max = max_repeat.min(self.config.max_repeat.max(min_repeat));
        let mutation_site_key = self.next_mutation_site_key(current_rule, node_path, "quantifier");
        let repeat_candidates: Vec<usize> = if self.construct_mode {
            // SV-EXH-PROOF.7.4.6.3: minimal derivation → emit exactly the minimum repetitions.
            vec![min_repeat]
        } else if let Some((preferred_repeats, baseline_repeats)) =
            self.forced_quantifier_repeats_for_site(&mutation_site_key)
        {
            let mut candidates = Vec::new();
            if preferred_repeats >= min_repeat && preferred_repeats <= bounded_max {
                candidates.push(preferred_repeats);
            }
            if let Some(baseline_repeats) = baseline_repeats {
                if baseline_repeats >= min_repeat
                    && baseline_repeats <= bounded_max
                    && !candidates.contains(&baseline_repeats)
                {
                    candidates.push(baseline_repeats);
                }
            }
            for repeat in min_repeat..=bounded_max {
                if !candidates.contains(&repeat) {
                    candidates.push(repeat);
                }
            }
            candidates
        } else if min_repeat == bounded_max {
            vec![min_repeat]
        } else if depth >= self.config.max_depth.saturating_sub(1) {
            vec![min_repeat]
        } else {
            let preferred = self.select_preferred_quantifier_repeat(min_repeat, bounded_max)?;
            let mut candidates = Vec::with_capacity(bounded_max.saturating_sub(min_repeat) + 1);
            candidates.push(preferred);
            let remainder_order = self.quantifier_remainder_order(min_repeat, bounded_max);
            for repeat in remainder_order {
                if repeat != preferred {
                    candidates.push(repeat);
                }
            }
            candidates
        };
        self.trace(
            TraceLevel::High,
            format_args!(
                "Quantifier decision: rule='{}' path='{}' site='{}' quantifier='{}' min={} max={} candidates={:?}",
                current_rule,
                node_path,
                mutation_site_key,
                quantifier,
                min_repeat,
                bounded_max,
                repeat_candidates
            ),
        );

        let quantified_path = format!("{}/q", node_path);
        let mut last_error: Option<anyhow::Error> = None;

        for repeats in repeat_candidates {
            self.enforce_generation_deadline(current_rule, node_path)?;
            let mut output = String::new();
            let mut failed = false;
            for _ in 0..repeats {
                self.enforce_generation_deadline(current_rule, &quantified_path)?;
                match self.generate_node(
                    element,
                    current_rule,
                    depth + 1,
                    call_stack,
                    &quantified_path,
                ) {
                    Ok(generated) => {
                        if self.should_insert_quantified_separator(
                            current_rule,
                            element,
                            output.as_str(),
                            &generated,
                        ) {
                            output.push('\n');
                        }
                        self.append_generated_segment(&mut output, &generated)
                    }
                    Err(err) => {
                        failed = true;
                        last_error = Some(err);
                        break;
                    }
                }
            }
            if !failed {
                self.record_quantifier_mutation_choice(
                    &mutation_site_key,
                    repeats,
                    min_repeat,
                    bounded_max,
                );
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

    /// SV-EXH-PROOF.7.4.6.5: returns `Cow::Borrowed(node)` in the common (no probability
    /// prefix) case — the prior version cloned EVERY alternative's subtree on every
    /// `generate_or` call (profiled as the dominant construction cost: deep `property_expr`
    /// recursion clones the full branch set per OR visit → O(depth x grammar-subtree) churn).
    /// Output is identical (the stripped node is the same); only the wasted clone is removed.
    /// Parser-agnostic (generic ASTNode), monotone (byte-identical generation).
    fn strip_probability_prefix<'n>(
        &self,
        node: &'n ASTNode,
    ) -> (Option<u32>, std::borrow::Cow<'n, ASTNode>) {
        use std::borrow::Cow;
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
                    return (None, Cow::Borrowed(node));
                }

                let mut remainder = elements[index..].to_vec();
                let stripped = match remainder.len() {
                    0 => ASTNode::Sequence { elements: vec![] },
                    1 => remainder.pop().unwrap(),
                    _ => ASTNode::Sequence {
                        elements: remainder,
                    },
                };
                (probability, Cow::Owned(stripped))
            }
            _ => {
                if let Some(weight) = self.extract_probability_from_node(node) {
                    (
                        Some(weight),
                        Cow::Owned(ASTNode::Sequence { elements: vec![] }),
                    )
                } else {
                    (None, Cow::Borrowed(node))
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

    fn select_preferred_quantifier_repeat(
        &mut self,
        min_repeat: usize,
        bounded_max: usize,
    ) -> Result<usize> {
        if min_repeat == bounded_max {
            return Ok(min_repeat);
        }

        match self.config.constraint_profile {
            StimuliConstraintProfile::Baseline | StimuliConstraintProfile::RareBranchBiased => {
                Ok(self.rng.gen_range(min_repeat..=bounded_max))
            }
            StimuliConstraintProfile::DeepNestingBiased => {
                let weights: Vec<u64> = (min_repeat..=bounded_max)
                    .map(|repeat| {
                        let bias = repeat.saturating_sub(min_repeat).saturating_add(1);
                        u64::try_from(bias.saturating_mul(bias)).unwrap_or(1)
                    })
                    .collect();
                let dist = WeightedIndex::new(&weights).with_context(|| {
                    format!(
                        "Invalid quantifier weights for range {}..={}: {:?}",
                        min_repeat, bounded_max, weights
                    )
                })?;
                Ok(min_repeat + dist.sample(&mut self.rng))
            }
        }
    }

    fn quantifier_remainder_order(&self, min_repeat: usize, bounded_max: usize) -> Vec<usize> {
        let mut repeats: Vec<usize> = (min_repeat..=bounded_max).collect();
        if self.config.constraint_profile == StimuliConstraintProfile::DeepNestingBiased {
            repeats.sort_by(|left, right| right.cmp(left));
        }
        repeats
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
            ASTNode::Lookahead { element, .. } => self.count_rule_references(element, current_rule),
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

    fn target_branch_depth_retry_slack(
        &self,
        group_key: &str,
        branch_idx: usize,
        err: &anyhow::Error,
    ) -> Option<usize> {
        if self.target_drive_validation_active {
            return None;
        }
        if !Self::is_depth_exhaustion_error(err) {
            return None;
        }
        if self.branch_target_deficit(group_key, branch_idx) == 0 {
            return None;
        }
        if self.branch_success_hits(group_key, branch_idx) > 0 {
            return None;
        }
        Some(4)
    }

    fn target_priority_probe_bias(&self, group_key: &str, branch_idx: usize) -> bool {
        self.branch_target_deficit(group_key, branch_idx) > 0
            && self.branch_success_hits(group_key, branch_idx) == 0
            && self.branch_selected_hits(group_key, branch_idx) == 0
    }

    fn branch_selected_hits(&self, group_key: &str, branch_idx: usize) -> u64 {
        self.coverage
            .branch_groups
            .get(group_key)
            .and_then(|group| group.selected_counts.get(branch_idx))
            .copied()
            .unwrap_or(0)
    }

    fn is_depth_exhaustion_error(err: &anyhow::Error) -> bool {
        err.to_string()
            .starts_with("Stimuli generation depth exceeded max_depth=")
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
        let mut rule_refs = HashSet::new();
        self.collect_rule_references(branch_node, &mut rule_refs);
        let targeted_dependency_refs: Vec<&str> = rule_refs
            .iter()
            .map(|rule_name| rule_name.as_str())
            .filter(|rule_name| self.rule_target_deficit(rule_name) > 0)
            .collect();
        let blocked_on_zero_success_target_dependencies = !targeted_dependency_refs.is_empty()
            && targeted_dependency_refs.iter().all(|rule_name| {
                self.coverage
                    .rule_success_hits
                    .get(*rule_name)
                    .copied()
                    .unwrap_or(0)
                    == 0
            });

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

        // Prevent target-driven mode from over-selecting branches that repeatedly fail parser-backed
        // validation, but do not punish a branch solely because its still-targeted dependencies have
        // never succeeded yet.
        if branch_target_deficit > 0 && selected_hits > 0 {
            let throttle = Self::target_branch_failure_throttle(selected_hits, success_hits);
            if !blocked_on_zero_success_target_dependencies {
                multiplier = (multiplier / throttle).max(1);
            }
        }

        multiplier
    }

    fn constraint_profile_branch_multiplier(
        &self,
        current_rule: &str,
        node_path: &str,
        branch_idx: usize,
        branch_node: &ASTNode,
        depth: usize,
        call_stack: &[String],
    ) -> u64 {
        match self.config.constraint_profile {
            StimuliConstraintProfile::Baseline => 1,
            StimuliConstraintProfile::RareBranchBiased => {
                let group_key = format!("{}::{}", current_rule, node_path);
                let success_hits = self.branch_success_hits(&group_key, branch_idx);
                let selected_hits = self.branch_selected_hits(&group_key, branch_idx);
                let target_deficit = self.branch_target_deficit(&group_key, branch_idx);
                let uncovered_rule_refs = self.count_uncovered_rule_references(branch_node);

                let mut multiplier = 1u64;
                if success_hits == 0 {
                    multiplier = multiplier.saturating_mul(12);
                } else if success_hits <= 2 {
                    multiplier = multiplier.saturating_mul(4);
                } else if success_hits <= 8 {
                    multiplier = multiplier.saturating_mul(2);
                }

                if selected_hits == 0 {
                    multiplier = multiplier.saturating_mul(3);
                } else if selected_hits <= 2 {
                    multiplier = multiplier.saturating_mul(2);
                }

                if target_deficit > 0 {
                    multiplier = multiplier
                        .saturating_mul(1 + u64::try_from(target_deficit.min(6)).unwrap_or(1));
                }

                if uncovered_rule_refs > 0 {
                    multiplier = multiplier
                        .saturating_mul(1 + u64::try_from(uncovered_rule_refs.min(4)).unwrap_or(1));
                }

                multiplier.max(1)
            }
            StimuliConstraintProfile::DeepNestingBiased => {
                let mut refs = HashSet::new();
                self.collect_rule_references(branch_node, &mut refs);
                if refs.is_empty() {
                    return 1;
                }

                let remaining_depth = self.config.max_depth.saturating_sub(depth);
                let is_recursive = refs.iter().any(|rule_name| {
                    call_stack
                        .iter()
                        .any(|active_rule| active_rule == rule_name.as_str())
                });

                let mut multiplier = 2u64;
                if remaining_depth > 2 {
                    multiplier = multiplier.saturating_mul(2);
                }
                if remaining_depth > 4 {
                    multiplier = multiplier.saturating_mul(2);
                }
                if is_recursive && remaining_depth > 2 {
                    multiplier = multiplier.saturating_mul(3);
                }

                multiplier.max(1)
            }
        }
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

    fn missing_rule_references(&self, node: &ASTNode) -> Vec<String> {
        let mut names = HashSet::new();
        self.collect_rule_references(node, &mut names);
        let mut missing: Vec<String> = names
            .into_iter()
            .filter(|rule_name| !self.grammar_tree.contains_key(rule_name))
            .collect();
        missing.sort();
        missing
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
        if self.generation_deadline_exceeded() {
            return String::new();
        }
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

        // SV-EXH-PROOF.2.3.2: if this pattern's leading atom is a
        // PERMISSIVE free-text content class, exclude — from ALL its
        // class materialization — both this rule's own author-declared
        // content-hazard set AND the grammar-scoped structural-sigil
        // set `G` (a sigil any content rule's author negated at
        // content-start is structural for the whole grammar; covers a
        // permissive content rule like `directive_tail` that negates
        // nothing of its own). Restrictive/positive leading classes
        // (`[a-z]`) ⇒ `None` ⇒ untouched (all-lanes-safe). Derived
        // purely from the grammar's own leading negations ⇒
        // parser/EBNF-agnostic.
        let regex_content_forbidden = match parsed_hir
            .as_ref()
            .and_then(|hir| Self::leading_permissive_negation_chars(hir))
        {
            Some(mut own) => {
                own.extend(self.grammar_content_sigils());
                own
            }
            None => HashSet::new(),
        };
        for _ in 0..64 {
            if self.generation_deadline_exceeded() {
                break;
            }
            let Some(hir) = parsed_hir.as_ref() else {
                break;
            };
            self.regex_content_forbidden = regex_content_forbidden.clone();
            let candidate = self.generate_from_regex_hir(hir);
            self.regex_content_forbidden.clear();
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
        // SV-EXH-PROOF.7.4.6.1: cache compiled terminal regexes. `Regex::new(P)` is
        // deterministic + costly, and the generator recompiles the same terminal patterns
        // (identifiers, numbers, ...) constantly during witness/diverse generation (profiled:
        // ~15% of witness self-time). PURE monotone speedup — same match result, no recompile.
        // Thread-local so it needs no signature/state change and is safe across the generator's
        // single-threaded use; no re-entrancy (`find` calls nothing back into here).
        thread_local! {
            static REGEX_COMPILE_CACHE: std::cell::RefCell<HashMap<String, Option<regex::Regex>>> =
                std::cell::RefCell::new(HashMap::new());
        }
        REGEX_COMPILE_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            let compiled = cache
                .entry(pattern.to_string())
                .or_insert_with(|| regex::Regex::new(pattern).ok());
            if let Some(re) = compiled.as_ref() {
                if let Some(matched) = re.find(candidate) {
                    return matched.start() == 0 && matched.end() == candidate.len();
                }
            }
            false
        })
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

    fn should_insert_quantified_separator(
        &self,
        current_rule: &str,
        element: &ASTNode,
        output: &str,
        segment: &str,
    ) -> bool {
        if self.grammar_name != "systemverilog_preprocessor"
            || output.is_empty()
            || segment.is_empty()
            || output.ends_with('\n')
            || segment.starts_with('\n')
            || segment.starts_with('\r')
        {
            return false;
        }

        matches!(
            current_rule,
            "systemverilog_preprocessor_file"
                | "pp_if_branch"
                | "pp_elsif_branch"
                | "pp_else_branch"
        ) && Self::node_is_rule_reference(element, "pp_item")
    }

    fn node_is_rule_reference(node: &ASTNode, expected_rule: &str) -> bool {
        let ASTNode::Atom { value } = node else {
            return false;
        };
        let ASTValue::Token(parts) = value else {
            return false;
        };
        let Some((token_type, token_value)) = Self::extract_token_pair(parts) else {
            return false;
        };
        token_type == "rule_reference" && token_value == expected_rule
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
        if self.generation_deadline_exceeded() {
            return String::new();
        }
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
                // LEXICAL-ANNOTATIONS.3 — Obligation A (intra-token faithfulness): prefer
                // alternation branches that are NOT pure zero-width assertions. A bare anchor
                // branch (`$`, `^`, `\b`) realizes to "" and ASSERTS an input boundary; chosen
                // inline (where the assertion is false) it yields a token valid ONLY at that
                // boundary — e.g. the line-comment regex `//[^\n]*(\n|$)` picking the `$` branch
                // emits a comment with no terminating newline, which then swallows whatever
                // follows on the line. Preferring a concrete sibling (here `\n`) makes every
                // emitted surface a complete, valid token. A genuine empty branch (no assertion)
                // is NOT anchor-only and stays pickable, so optional-alternative variety is kept.
                let preferred: Vec<&Hir> = parts
                    .iter()
                    .filter(|p| !Self::regex_branch_is_anchor_only(p))
                    .collect();
                let chosen: &Hir = if !preferred.is_empty() {
                    let i = if preferred.len() == 1 { 0 } else { self.rng.gen_range(0..preferred.len()) };
                    preferred[i]
                } else {
                    let i = if parts.len() == 1 { 0 } else { self.rng.gen_range(0..parts.len()) };
                    &parts[i]
                };
                self.generate_from_regex_hir(chosen)
            }
        }
    }

    fn generate_from_regex_repetition(&mut self, rep: &Repetition) -> String {
        if self.generation_deadline_exceeded() {
            return String::new();
        }
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
            if self.generation_deadline_exceeded() {
                break;
            }
            let unit = self.generate_from_regex_hir(&rep.sub);
            out.push_str(&unit);
        }
        out
    }

    /// LEXICAL-ANNOTATIONS.3 — Obligation A helper. A regex branch is "anchor-only" if it
    /// produces NO characters AND contains a look-around assertion (`$`, `^`, `\b`, …).
    /// Realizing it inline emits nothing but asserts an input boundary that may not hold, so the
    /// alternation picker avoids it when a concrete sibling exists. A genuine empty branch (no
    /// assertion) is NOT anchor-only and stays pickable.
    fn regex_branch_is_anchor_only(hir: &Hir) -> bool {
        !Self::regex_hir_can_produce_nonempty(hir) && Self::regex_hir_contains_look(hir)
    }

    /// Whether a regex HIR can yield a non-empty string on some realization.
    fn regex_hir_can_produce_nonempty(hir: &Hir) -> bool {
        match hir.kind() {
            HirKind::Empty | HirKind::Look(_) => false,
            HirKind::Literal(Literal(bytes)) => !bytes.is_empty(),
            HirKind::Class(_) => true,
            HirKind::Repetition(rep) => {
                rep.max != Some(0) && Self::regex_hir_can_produce_nonempty(&rep.sub)
            }
            HirKind::Capture(capture) => Self::regex_hir_can_produce_nonempty(&capture.sub),
            HirKind::Concat(parts) => parts.iter().any(Self::regex_hir_can_produce_nonempty),
            HirKind::Alternation(parts) => parts.iter().any(Self::regex_hir_can_produce_nonempty),
        }
    }

    /// Whether a regex HIR contains any look-around assertion.
    fn regex_hir_contains_look(hir: &Hir) -> bool {
        match hir.kind() {
            HirKind::Look(_) => true,
            HirKind::Empty | HirKind::Literal(_) | HirKind::Class(_) => false,
            HirKind::Repetition(rep) => Self::regex_hir_contains_look(&rep.sub),
            HirKind::Capture(capture) => Self::regex_hir_contains_look(&capture.sub),
            HirKind::Concat(parts) | HirKind::Alternation(parts) => {
                parts.iter().any(Self::regex_hir_contains_look)
            }
        }
    }

    /// SV-EXH-PROOF.2.3.2 (P-a): derive the structural-hazard set from
    /// a *permissive leading negated* class in the rule's own regex.
    /// A grammar author who anchors `[^X…]` at content-start is
    /// declaring X structurally significant for that content type; the
    /// closed-loop must therefore not emit X *anywhere* in the run
    /// (not just position 1), else the parser re-lexes it as structure
    /// and the generated sample fails its own round-trip.
    ///
    /// Returns the small printable-ASCII complement of the first
    /// emission atom **only when** that atom is a class that matches
    /// most of printable ASCII while excluding just a few chars (a
    /// negation of a small structural set) — distinguishing it from a
    /// restrictive *positive* leading class like `[a-z]` (whose huge
    /// complement is NOT the author's intent). Otherwise empty ⇒
    /// no-op. Derived from the grammar, parser-agnostic,
    /// all-lanes-safe.
    /// `Some(printable_complement)` iff the first emission atom is a
    /// **permissive** class — one that matches the vast majority of
    /// printable ASCII (a free-text content class, e.g. `[^\r\n]+`,
    /// `[^\`\r\n]…`). The returned set is the printable chars that
    /// class excludes (its own author-declared content-hazard set —
    /// *may be empty*, e.g. `directive_tail`'s `[^\r\n]`). `None` iff
    /// the leading atom is a *restrictive positive* class (`[a-z]`),
    /// not a class, or ambiguous — those are NOT free-text content
    /// and must be left untouched (all-lanes-safe).
    ///
    /// SV-EXH-PROOF.2.3.2: a permissive-leading content rule must not
    /// emit, *anywhere* in its run, the grammar's structural sigils
    /// (this rule's own complement ∪ the grammar-scoped set `G`) —
    /// else the parser re-lexes the free-text as structure and the
    /// closed-loop round-trip fails. Returning `Some(∅)` (vs `None`)
    /// is load-bearing: it marks the rule as free-text content so the
    /// caller still applies `G` even when the rule negates nothing of
    /// its own (the `directive_tail` case).
    fn leading_permissive_negation_chars(hir: &Hir) -> Option<HashSet<char>> {
        fn first_atom(h: &Hir) -> Option<&Hir> {
            match h.kind() {
                HirKind::Empty | HirKind::Look(_) => None,
                HirKind::Concat(parts) => parts.iter().find_map(first_atom),
                HirKind::Capture(c) => first_atom(&c.sub),
                HirKind::Repetition(r) => first_atom(&r.sub),
                // Ambiguous leading (branch-dependent) — be
                // conservative: not classified as content.
                HirKind::Alternation(_) => None,
                _ => Some(h),
            }
        }
        let atom = first_atom(hir)?;
        let HirKind::Class(class) = atom.kind() else {
            return None;
        };
        let matches = |c: char| -> bool {
            match class {
                Class::Unicode(u) => u
                    .ranges()
                    .iter()
                    .any(|r| r.start() <= c && c <= r.end()),
                Class::Bytes(b) => {
                    (c as u32) <= 0xff && {
                        let cb = c as u8;
                        b.ranges().iter().any(|r| r.start() <= cb && cb <= r.end())
                    }
                }
            }
        };
        // Printable ASCII universe (mirrors the [0x20,0x7e]
        // materialization clamp the precedent already uses).
        let complement: HashSet<char> = (0x20u8..=0x7e)
            .map(char::from)
            .filter(|c| !matches(*c))
            .collect();
        let matched = (0x7eusize - 0x20 + 1) - complement.len();
        // Permissive free-text content class: matches the vast
        // majority of printable (excludes only a handful — line
        // terminators ± a structural sigil). Restrictive positive
        // classes (`[a-z]`, ~26 matched) are NOT content ⇒ `None`.
        if matched >= 80 && complement.len() <= 8 {
            Some(complement)
        } else {
            None
        }
    }

    /// Parse-then-classify wrapper used to derive the grammar-scoped
    /// structural-sigil set `G` from every rule's regex pattern.
    fn leading_permissive_negation_chars_of_pattern(pattern: &str) -> Option<HashSet<char>> {
        regex_syntax::parse(pattern.trim())
            .ok()
            .and_then(|hir| Self::leading_permissive_negation_chars(&hir))
    }

    /// Recursively collect every `regex`-token pattern string in a
    /// grammar AST node (parser/EBNF-agnostic structural walk).
    fn collect_regex_patterns(node: &ASTNode, out: &mut Vec<String>) {
        match node {
            ASTNode::Or { alternatives } => {
                for a in alternatives {
                    Self::collect_regex_patterns(a, out);
                }
            }
            ASTNode::Sequence { elements } => {
                for e in elements {
                    Self::collect_regex_patterns(e, out);
                }
            }
            ASTNode::Quantified { element, .. } | ASTNode::Lookahead { element, .. } => {
                Self::collect_regex_patterns(element, out);
            }
            ASTNode::Atom { value } => match value {
                ASTValue::Node(n) => Self::collect_regex_patterns(n, out),
                ASTValue::Token(parts) => {
                    if let Some(("regex", pat)) = Self::extract_token_pair(parts) {
                        out.push(pat.to_string());
                    }
                }
            },
        }
    }

    /// SV-EXH-PROOF.2.3.2 — the grammar-scoped structural-sigil set
    /// `G`: the union of the printable complements of EVERY permissive
    /// leading-negated content class across the whole grammar. A char
    /// any content rule's author negated at content-start is a
    /// structural sigil for *that grammar*; no permissive content
    /// rule may emit it (even one — like `directive_tail` — that
    /// negates nothing of its own). Derived purely from the grammar's
    /// own author-written leading negations ⇒ parser/EBNF-agnostic,
    /// all-lanes-safe. Computed once, cached.
    fn grammar_content_sigils(&mut self) -> HashSet<char> {
        if let Some(cached) = &self.grammar_content_sigils {
            return cached.clone();
        }
        let mut patterns = Vec::new();
        for node in self.grammar_tree.values() {
            Self::collect_regex_patterns(node, &mut patterns);
        }
        let mut sigils: HashSet<char> = HashSet::new();
        for pat in &patterns {
            if let Some(complement) = Self::leading_permissive_negation_chars_of_pattern(pat) {
                sigils.extend(complement);
            }
        }
        self.grammar_content_sigils = Some(sigils.clone());
        sigils
    }

    // ── SV-EXH-PROOF.2.3.2 (Mode B): parser/EBNF-agnostic structural
    // helpers. Pure functions of the grammar AST + regex HIR; they
    // contain ZERO grammar/parser/EBNF identifiers (no rule names, no
    // sigils) — every value is *derived* from the grammar the author
    // wrote. ────────────────────────────────────────────────────────

    /// `Some(literal)` iff the regex pattern denotes exactly ONE
    /// fixed string (modulo zero-width anchors / look-around).
    /// `None` for any *variable* terminal (class / repetition /
    /// alternation). This is the free-vs-fixed-literal terminal
    /// discriminator: structural keyword terminals are fixed-literal
    /// (exempt from the closer check ⇒ nesting-safe); content /
    /// identifier terminals are free (subject to it).
    fn regex_fixed_literal(pattern: &str) -> Option<String> {
        regex_syntax::parse(pattern.trim())
            .ok()
            .and_then(|hir| Self::hir_fixed_literal(&hir))
    }

    fn hir_fixed_literal(hir: &Hir) -> Option<String> {
        match hir.kind() {
            HirKind::Empty | HirKind::Look(_) => Some(String::new()),
            HirKind::Literal(Literal(bytes)) => {
                Some(String::from_utf8_lossy(bytes).into_owned())
            }
            HirKind::Capture(capture) => Self::hir_fixed_literal(&capture.sub),
            HirKind::Concat(parts) => {
                let mut out = String::new();
                for part in parts {
                    out.push_str(&Self::hir_fixed_literal(part)?);
                }
                Some(out)
            }
            HirKind::Repetition(rep) => {
                if rep.min == 1 && rep.max == Some(1) {
                    Self::hir_fixed_literal(&rep.sub)
                } else {
                    None
                }
            }
            HirKind::Alternation(parts) => {
                if parts.len() == 1 {
                    Self::hir_fixed_literal(&parts[0])
                } else {
                    None
                }
            }
            HirKind::Class(_) => None,
        }
    }

    fn hir_matches_empty(hir: &Hir) -> bool {
        match hir.kind() {
            HirKind::Empty | HirKind::Look(_) => true,
            HirKind::Literal(Literal(bytes)) => bytes.is_empty(),
            HirKind::Class(_) => false,
            HirKind::Capture(capture) => Self::hir_matches_empty(&capture.sub),
            HirKind::Repetition(rep) => {
                rep.min == 0 || Self::hir_matches_empty(&rep.sub)
            }
            HirKind::Concat(parts) => {
                parts.iter().all(|p| Self::hir_matches_empty(p))
            }
            HirKind::Alternation(parts) => {
                parts.iter().any(|p| Self::hir_matches_empty(p))
            }
        }
    }

    /// Generic nullability (does the node's language include ε?) over
    /// the grammar AST, with a rule-reference cycle/depth guard (an
    /// unresolved cycle is treated as NON-nullable — conservatively
    /// keeping such an element *required*). Fully agnostic; used to
    /// skip nullable parts (e.g. optional leading trivia) when
    /// resolving a construct's mandatory closer lexeme.
    /// SV-EXH-PROOF.7.4.6.2: `hit_cycle` is set to `true` if this result was influenced by a
    /// back-edge into `visiting` (a recursion cycle) OR the depth cutoff — i.e. it is
    /// context-DEPENDENT and must NOT be cached. A rule-reference result is cached in
    /// `NULLABLE_CACHE` only when its sub-computation was cycle-free (context-independent), so a
    /// cache hit equals the per-call DFS exactly (monotone — generation output unchanged).
    fn node_is_nullable(
        tree: &HashMap<String, ASTNode>,
        node: &ASTNode,
        depth: usize,
        visiting: &mut HashSet<String>,
        hit_cycle: &mut bool,
    ) -> bool {
        if depth > 32 {
            *hit_cycle = true; // depth cutoff → path-dependent result; never cache it
            return false;
        }
        match node {
            ASTNode::Or { alternatives } => {
                // any(): nullable if any alternative is. Accumulate hit_cycle over explored alts.
                for a in alternatives {
                    let mut c = false;
                    if Self::node_is_nullable(tree, a, depth + 1, visiting, &mut c) {
                        *hit_cycle |= c;
                        return true;
                    }
                    *hit_cycle |= c;
                }
                false
            }
            ASTNode::Sequence { elements } => {
                // all(): nullable iff every element is. Accumulate hit_cycle over explored elems.
                for e in elements {
                    let mut c = false;
                    let n = Self::node_is_nullable(tree, e, depth + 1, visiting, &mut c);
                    *hit_cycle |= c;
                    if !n {
                        return false;
                    }
                }
                true
            }
            ASTNode::Lookahead { .. } => true,
            ASTNode::Quantified {
                element,
                quantifier,
            } => {
                let q = quantifier.trim();
                if q.starts_with('?') || q.starts_with('*') || q.starts_with("{0") {
                    true
                } else {
                    Self::node_is_nullable(tree, element, depth + 1, visiting, hit_cycle)
                }
            }
            ASTNode::Atom { value } => match value {
                ASTValue::Node(n) => {
                    Self::node_is_nullable(tree, n, depth + 1, visiting, hit_cycle)
                }
                ASTValue::Token(parts) => match Self::extract_token_pair(parts) {
                    Some(("rule_reference", name)) => {
                        if visiting.contains(name) {
                            *hit_cycle = true; // back-edge → context-dependent
                            return false;
                        }
                        // Cache hit: stored entries are cycle-free (context-independent), so this
                        // equals what a fresh DFS would compute — return it without recursing.
                        if let Some(cached) = NULLABLE_CACHE.with(|c| c.borrow().get(name).copied())
                        {
                            return cached;
                        }
                        visiting.insert(name.to_string());
                        let mut sub_cycle = false;
                        let r = tree
                            .get(name)
                            .map(|n| {
                                Self::node_is_nullable(tree, n, depth + 1, visiting, &mut sub_cycle)
                            })
                            .unwrap_or(false);
                        visiting.remove(name);
                        if sub_cycle {
                            *hit_cycle = true; // this rule's result depended on a cycle → don't cache
                        } else {
                            NULLABLE_CACHE.with(|c| {
                                c.borrow_mut().insert(name.to_string(), r);
                            });
                        }
                        r
                    }
                    Some(("quoted_string", s)) => s.is_empty(),
                    Some(("regex", pat)) => regex_syntax::parse(pat.trim())
                        .ok()
                        .map(|hir| Self::hir_matches_empty(&hir))
                        .unwrap_or(false),
                    _ => false,
                },
            },
        }
    }

    /// The *mandatory* fixed-literal lexeme a node always contributes
    /// (nullable sub-parts — e.g. optional leading trivia — skipped),
    /// or `None` if the node has any required *variable* sub-part
    /// (⇒ not a deterministic fixed-literal structural closer).
    /// Resolves rule references with a cycle/depth guard. Agnostic.
    fn terminal_literal_of_node(
        tree: &HashMap<String, ASTNode>,
        node: &ASTNode,
        depth: usize,
        visiting: &mut HashSet<String>,
    ) -> Option<String> {
        if depth > 32 {
            return None;
        }
        match node {
            ASTNode::Atom { value } => match value {
                ASTValue::Node(n) => {
                    Self::terminal_literal_of_node(tree, n, depth + 1, visiting)
                }
                ASTValue::Token(parts) => match Self::extract_token_pair(parts) {
                    Some(("quoted_string", s)) => Some(s.to_string()),
                    Some(("regex", pat)) => Self::regex_fixed_literal(pat),
                    Some(("rule_reference", name)) => {
                        if !visiting.insert(name.to_string()) {
                            return None;
                        }
                        let r = tree.get(name).and_then(|n| {
                            Self::terminal_literal_of_node(tree, n, depth + 1, visiting)
                        });
                        visiting.remove(name);
                        r
                    }
                    _ => None,
                },
            },
            ASTNode::Sequence { elements } => {
                let mut out = String::new();
                for e in elements {
                    if let Some(lit) =
                        Self::terminal_literal_of_node(tree, e, depth + 1, visiting)
                    {
                        out.push_str(&lit);
                    } else if Self::node_is_nullable(tree, e, 0, &mut HashSet::new(), &mut false) {
                        // nullable (e.g. optional leading trivia) ⇒ skip
                    } else {
                        return None;
                    }
                }
                if out.is_empty() {
                    None
                } else {
                    Some(out)
                }
            }
            // An alternation / quantified / lookahead element is not a
            // single deterministic fixed lexeme.
            _ => None,
        }
    }

    /// Detect the `… <quantified/optional body> … CLOSE` idiom:
    /// returns `(closer_index, closer_lexeme)` iff (a) the LAST
    /// element is *required* (not `?`/`*`-quantified) and resolves to
    /// a non-empty fixed-literal lexeme (the structural closer), and
    /// (b) some earlier element is `*`/`+`/`?`-quantified (the
    /// open-ended body that — possibly via recursion — can host free
    /// content). Purely structural ⇒ parser/EBNF-agnostic; the
    /// Quantified gate keeps the blast radius to exactly the
    /// closer-bearing-construct idiom.
    fn sequence_closer_split(
        tree: &HashMap<String, ASTNode>,
        elements: &[ASTNode],
    ) -> Option<(usize, String)> {
        if elements.len() < 2 {
            return None;
        }
        let last = elements.len() - 1;
        if matches!(
            &elements[last],
            ASTNode::Quantified { quantifier, .. }
                if { let q = quantifier.trim(); q.starts_with('?') || q.starts_with('*') }
        ) {
            return None;
        }
        let has_quantified_body = elements[..last].iter().any(|e| {
            matches!(
                e,
                ASTNode::Quantified { quantifier, .. }
                    if {
                        let q = quantifier.trim();
                        q.starts_with('*') || q.starts_with('+') || q.starts_with('?')
                    }
            )
        });
        if !has_quantified_body {
            return None;
        }
        let lexeme = Self::terminal_literal_of_node(
            tree,
            &elements[last],
            0,
            &mut HashSet::new(),
        )?;
        if lexeme.is_empty() {
            None
        } else {
            Some((last, lexeme))
        }
    }

    // ── SV-EXH-PROOF.2.3.2 (line-terminator completeness): a
    // line-oriented construct whose content terminal greedily
    // consumes to end-of-line, followed by an *optional* newline
    // terminator, must EMIT that newline when generated — else, on
    // reparse, the line-greedy content absorbs whatever structural
    // element follows (e.g. a `pp_define` macro body swallowing a
    // following `` `endif ``, leaving a real `` `ifdef `` unclosed =
    // genuinely-invalid output the parser correctly rejects =
    // generator over-generation). Pure functions of regex HIR + the
    // grammar AST; zero grammar/parser/EBNF identifiers; all-lanes
    // -safe (a trailing newline is universally benign, and the
    // detector fires ONLY on the line-greedy-content → optional
    // -newline-terminator shape, so grammars without it are
    // untouched). ───────────────────────────────────────────────────

    fn hir_is_newline_only(hir: &Hir) -> bool {
        let only_nl = |c: char| c == '\r' || c == '\n';
        match hir.kind() {
            HirKind::Empty | HirKind::Look(_) => true,
            HirKind::Literal(Literal(bytes)) => {
                String::from_utf8_lossy(bytes).chars().all(only_nl)
            }
            HirKind::Class(Class::Unicode(u)) => u
                .ranges()
                .iter()
                .all(|r| only_nl(r.start()) && only_nl(r.end()) && r.start() == r.end()),
            HirKind::Class(Class::Bytes(b)) => b.ranges().iter().all(|r| {
                (r.start() == b'\r' || r.start() == b'\n')
                    && (r.end() == b'\r' || r.end() == b'\n')
            }),
            HirKind::Repetition(rep) => Self::hir_is_newline_only(&rep.sub),
            HirKind::Capture(c) => Self::hir_is_newline_only(&c.sub),
            HirKind::Concat(parts) | HirKind::Alternation(parts) => {
                parts.iter().all(Self::hir_is_newline_only)
            }
        }
    }

    /// `true` iff the pattern's language is non-empty and consists
    /// only of newline characters (e.g. `\r?\n`, `\n`).
    fn regex_is_newline_only(pattern: &str) -> bool {
        regex_syntax::parse(pattern.trim())
            .ok()
            .map(|hir| Self::hir_is_newline_only(&hir) && !Self::hir_matches_empty(&hir))
            .unwrap_or(false)
    }

    fn hir_can_match_nonnewline(hir: &Hir) -> bool {
        match hir.kind() {
            HirKind::Empty | HirKind::Look(_) => false,
            HirKind::Literal(Literal(bytes)) => String::from_utf8_lossy(bytes)
                .chars()
                .any(|c| c != '\r' && c != '\n'),
            HirKind::Class(Class::Unicode(u)) => u.ranges().iter().any(|r| {
                // any code point in [start,end] other than CR/LF
                !(r.start() == r.end()
                    && (r.start() == '\r' || r.start() == '\n'))
            }),
            HirKind::Class(Class::Bytes(b)) => b
                .ranges()
                .iter()
                .any(|r| !(r.start() >= b'\r' && r.end() <= b'\n')),
            HirKind::Repetition(rep) => {
                rep.max != Some(0) && Self::hir_can_match_nonnewline(&rep.sub)
            }
            HirKind::Capture(c) => Self::hir_can_match_nonnewline(&c.sub),
            HirKind::Concat(parts) | HirKind::Alternation(parts) => {
                parts.iter().any(Self::hir_can_match_nonnewline)
            }
        }
    }

    fn hir_has_unbounded_nonnewline_repetition(hir: &Hir) -> bool {
        match hir.kind() {
            HirKind::Repetition(rep) => {
                (rep.max.is_none() && Self::hir_can_match_nonnewline(&rep.sub))
                    || Self::hir_has_unbounded_nonnewline_repetition(&rep.sub)
            }
            HirKind::Capture(c) => {
                Self::hir_has_unbounded_nonnewline_repetition(&c.sub)
            }
            HirKind::Concat(parts) | HirKind::Alternation(parts) => parts
                .iter()
                .any(Self::hir_has_unbounded_nonnewline_repetition),
            _ => false,
        }
    }

    /// `true` iff the pattern can greedily consume an unbounded run
    /// of non-newline characters (a line-to-EOL content terminal,
    /// e.g. `[^\r\n]*`, `[^\`(),?:\r\n]+`, the comment-aware macro
    /// body regex).
    fn regex_is_line_greedy(pattern: &str) -> bool {
        regex_syntax::parse(pattern.trim())
            .ok()
            .map(|hir| Self::hir_has_unbounded_nonnewline_repetition(&hir))
            .unwrap_or(false)
    }

    fn node_is_newline_terminator(
        tree: &HashMap<String, ASTNode>,
        node: &ASTNode,
        depth: usize,
        visiting: &mut HashSet<String>,
    ) -> bool {
        if depth > 32 {
            return false;
        }
        match node {
            ASTNode::Atom { value } => match value {
                ASTValue::Node(n) => {
                    Self::node_is_newline_terminator(tree, n, depth + 1, visiting)
                }
                ASTValue::Token(parts) => match Self::extract_token_pair(parts) {
                    Some(("regex", pat)) => Self::regex_is_newline_only(pat),
                    Some(("rule_reference", name)) => {
                        if !visiting.insert(name.to_string()) {
                            return false;
                        }
                        let r = tree
                            .get(name)
                            .map(|n| {
                                Self::node_is_newline_terminator(
                                    tree,
                                    n,
                                    depth + 1,
                                    visiting,
                                )
                            })
                            .unwrap_or(false);
                        visiting.remove(name);
                        r
                    }
                    _ => false,
                },
            },
            ASTNode::Sequence { elements } => {
                let mut found = false;
                for e in elements {
                    if Self::node_is_newline_terminator(tree, e, depth + 1, visiting) {
                        found = true;
                    } else if Self::node_is_nullable(tree, e, 0, &mut HashSet::new(), &mut false) {
                        // nullable (e.g. optional leading trivia) ⇒ skip
                    } else {
                        return false;
                    }
                }
                found
            }
            _ => false,
        }
    }

    fn node_contains_line_greedy(
        tree: &HashMap<String, ASTNode>,
        node: &ASTNode,
        depth: usize,
        visiting: &mut HashSet<String>,
    ) -> bool {
        if depth > 64 {
            return false;
        }
        match node {
            ASTNode::Or { alternatives } => alternatives
                .iter()
                .any(|a| Self::node_contains_line_greedy(tree, a, depth + 1, visiting)),
            ASTNode::Sequence { elements } => elements
                .iter()
                .any(|e| Self::node_contains_line_greedy(tree, e, depth + 1, visiting)),
            ASTNode::Quantified { element, .. }
            | ASTNode::Lookahead { element, .. } => {
                Self::node_contains_line_greedy(tree, element, depth + 1, visiting)
            }
            ASTNode::Atom { value } => match value {
                ASTValue::Node(n) => {
                    Self::node_contains_line_greedy(tree, n, depth + 1, visiting)
                }
                ASTValue::Token(parts) => match Self::extract_token_pair(parts) {
                    Some(("regex", pat)) => Self::regex_is_line_greedy(pat),
                    Some(("rule_reference", name)) => {
                        if !visiting.insert(name.to_string()) {
                            return false;
                        }
                        let r = tree
                            .get(name)
                            .map(|n| {
                                Self::node_contains_line_greedy(
                                    tree,
                                    n,
                                    depth + 1,
                                    visiting,
                                )
                            })
                            .unwrap_or(false);
                        visiting.remove(name);
                        r
                    }
                    _ => false,
                },
            },
        }
    }

    /// Returns the index of a trailing element that is an *optional*
    /// (`?`/`*`) newline terminator, when an EARLIER element in the
    /// same sequence (transitively) contains a line-greedy content
    /// terminal. That trailing newline must then be force-emitted
    /// (generated as exactly one inner repetition) so the line-greedy
    /// content cannot absorb the following structural element on
    /// reparse. Purely structural ⇒ parser/EBNF-agnostic,
    /// all-lanes-safe.
    fn sequence_force_line_terminator_idx(
        tree: &HashMap<String, ASTNode>,
        elements: &[ASTNode],
    ) -> Option<usize> {
        if elements.len() < 2 {
            return None;
        }
        let last = elements.len() - 1;
        let inner = match &elements[last] {
            ASTNode::Quantified {
                element,
                quantifier,
            } if {
                let q = quantifier.trim();
                q.starts_with('?') || q.starts_with('*')
            } =>
            {
                element.as_ref()
            }
            _ => return None,
        };
        if !Self::node_is_newline_terminator(tree, inner, 0, &mut HashSet::new()) {
            return None;
        }
        let preceded_by_line_greedy = elements[..last].iter().any(|e| {
            Self::node_contains_line_greedy(tree, e, 0, &mut HashSet::new())
        });
        if preceded_by_line_greedy {
            Some(last)
        } else {
            None
        }
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
                // SV-EXH-PROOF.2.3.2 (P-a): never materialize a char
                // the rule's own leading negated class declared
                // structurally hazardous (else it re-lexes as
                // structure and the closed-loop round-trip fails).
                if !self.regex_content_forbidden.is_empty() {
                    printable.retain(|ch| !self.regex_content_forbidden.contains(ch));
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
                        if !self.regex_content_forbidden.contains(&ch) {
                            return ch.to_string();
                        }
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
                // SV-EXH-PROOF.2.3.2 (P-a): see the Unicode arm.
                if !self.regex_content_forbidden.is_empty() {
                    printable
                        .retain(|b| !self.regex_content_forbidden.contains(&char::from(*b)));
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
                    let ch = char::from(sampled);
                    if !self.regex_content_forbidden.contains(&ch) {
                        return ch.to_string();
                    }
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
                UnifiedSemanticAST::Structured { canonical, value } => {
                    if matches!(
                        directive_name.as_deref(),
                        Some(name)
                            if !matches!(name, "sample" | "literal" | "example" | "stimulus")
                    ) {
                        continue;
                    }

                    match value {
                        UnifiedSemanticValue::String(text) => return Some(text.clone()),
                        _ => {
                            let trimmed = canonical.trim();
                            if trimmed.len() >= 2
                                && ((trimmed.starts_with('"') && trimmed.ends_with('"'))
                                    || (trimmed.starts_with('\'') && trimmed.ends_with('\'')))
                            {
                                return Some(trimmed[1..trimmed.len() - 1].to_string());
                            }
                        }
                    }
                }
            }
        }

        Self::literalish_hint_from_annotations(semantic_annotations)
    }

    fn literalish_override_is_active_for_rule(&self, rule_name: &str) -> bool {
        self.active_generation_entry_rule.as_deref() == Some(rule_name)
    }

    fn literalish_hint_for_rule(&self, rule_name: &str) -> Option<String> {
        let annotations = self.annotations?;
        let semantic_annotations = annotations.semantic_annotations.get(rule_name)?;
        Self::literalish_hint_from_annotations(semantic_annotations)
    }

    fn literalish_hint_for_branch(&self, rule_name: &str, branch_index: usize) -> Option<String> {
        let annotations = self.annotations?;
        let branch_annotations = annotations.branch_semantic_annotations.get(rule_name)?;
        let semantic_annotations = branch_annotations.get(branch_index)?;
        Self::literalish_hint_from_annotations(semantic_annotations)
    }

    fn probe_literalish_hint_for_rule(&self, rule_name: &str) -> Option<String> {
        if !self.literalish_override_is_active_for_rule(rule_name) {
            return None;
        }
        let annotations = self.annotations?;
        let semantic_annotations = annotations.semantic_annotations.get(rule_name)?;
        Self::probe_literalish_hint_from_annotations(semantic_annotations)
    }

    fn probe_literalish_hint_for_branch(
        &self,
        rule_name: &str,
        branch_index: usize,
    ) -> Option<String> {
        if !self.literalish_override_is_active_for_rule(rule_name) {
            return None;
        }
        let annotations = self.annotations?;
        let branch_annotations = annotations.branch_semantic_annotations.get(rule_name)?;
        let semantic_annotations = branch_annotations.get(branch_index)?;
        Self::probe_literalish_hint_from_annotations(semantic_annotations)
    }

    fn literalish_hint_from_annotations(
        semantic_annotations: &[SemanticAnnotation],
    ) -> Option<String> {
        for semantic_annotation in semantic_annotations {
            let directive_name = semantic_annotation.name();
            match semantic_annotation.ast() {
                UnifiedSemanticAST::TransformExpr { .. } => continue,
                UnifiedSemanticAST::Raw { content } => {
                    if matches!(
                        directive_name,
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
                UnifiedSemanticAST::Structured { canonical, value } => {
                    if matches!(
                        directive_name,
                        Some(name)
                            if !matches!(name, "sample" | "literal" | "example" | "stimulus")
                    ) {
                        continue;
                    }

                    match value {
                        UnifiedSemanticValue::String(text) => return Some(text.clone()),
                        _ => {
                            let trimmed = canonical.trim();
                            if trimmed.len() >= 2
                                && ((trimmed.starts_with('"') && trimmed.ends_with('"'))
                                    || (trimmed.starts_with('\'') && trimmed.ends_with('\'')))
                            {
                                return Some(trimmed[1..trimmed.len() - 1].to_string());
                            }
                        }
                    }
                }
            }
        }

        None
    }

    fn probe_literalish_hint_from_annotations(
        semantic_annotations: &[SemanticAnnotation],
    ) -> Option<String> {
        for semantic_annotation in semantic_annotations {
            if !matches!(semantic_annotation.name(), Some("probe_sample")) {
                continue;
            }

            match semantic_annotation.ast() {
                UnifiedSemanticAST::TransformExpr { .. } => continue,
                UnifiedSemanticAST::Raw { content } => {
                    let trimmed = content.trim();
                    if trimmed.len() >= 2
                        && ((trimmed.starts_with('"') && trimmed.ends_with('"'))
                            || (trimmed.starts_with('\'') && trimmed.ends_with('\'')))
                    {
                        return Some(trimmed[1..trimmed.len() - 1].to_string());
                    }
                }
                UnifiedSemanticAST::Structured { canonical, value } => match value {
                    UnifiedSemanticValue::String(text) => return Some(text.clone()),
                    _ => {
                        let trimmed = canonical.trim();
                        if trimmed.len() >= 2
                            && ((trimmed.starts_with('"') && trimmed.ends_with('"'))
                                || (trimmed.starts_with('\'') && trimmed.ends_with('\'')))
                        {
                            return Some(trimmed[1..trimmed.len() - 1].to_string());
                        }
                    }
                },
            }
        }

        None
    }

    fn annotations_have_literalish_hint(semantic_annotations: &[SemanticAnnotation]) -> bool {
        Self::literalish_hint_from_annotations(semantic_annotations).is_some()
    }

    fn annotations_have_probe_literalish_hint(semantic_annotations: &[SemanticAnnotation]) -> bool {
        Self::probe_literalish_hint_from_annotations(semantic_annotations).is_some()
    }

    fn rule_literalish_hint_probe_score(&self, rule_name: &str) -> u64 {
        let Some(annotations) = self.annotations else {
            return 0;
        };

        let mut score = 0u64;
        if let Some(semantic_annotations) = annotations.semantic_annotations.get(rule_name) {
            if Self::annotations_have_literalish_hint(semantic_annotations)
                || Self::annotations_have_probe_literalish_hint(semantic_annotations)
            {
                score = score.saturating_add(4);
            }
        }
        if let Some(branch_annotations) = annotations.branch_semantic_annotations.get(rule_name) {
            let branch_hint_count = branch_annotations
                .iter()
                .filter(|semantic_annotations| {
                    Self::annotations_have_literalish_hint(semantic_annotations)
                        || Self::annotations_have_probe_literalish_hint(semantic_annotations)
                })
                .count();
            score = score
                .saturating_add(u64::try_from(branch_hint_count).unwrap_or(u64::MAX))
                .min(16);
        }
        score
    }

    fn node_supports_rule_literal_override(node: &ASTNode) -> bool {
        match node {
            // Rule-level samples and probe samples should also steer wrapper rules whose root is
            // an OR node; branch accounting is still handled by the ordinary expansion path when
            // no explicit literal override is active.
            ASTNode::Atom { value } => !matches!(
                value,
                ASTValue::Token(parts)
                    if matches!(
                        Self::extract_token_pair(parts),
                        Some((token_type, _)) if token_type == "regex"
                    )
            ),
            _ => true,
        }
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
        let should_apply_near_valid = policy.invalid_case
            || self.config.negative_profile == StimuliNegativeProfile::NearValidLocal;
        if !should_apply_near_valid {
            return sample;
        }

        let mutated = self.apply_near_valid_negative_profile(rule_name, &sample);
        if policy.invalid_case && policy.negative {
            format!("{}{}", mutated, Self::negative_case_suffix(rule_name))
        } else {
            mutated
        }
    }

    fn apply_near_valid_negative_profile(&self, rule_name: &str, sample: &str) -> String {
        let mut candidates = self.near_valid_negative_candidates(sample);
        if candidates.is_empty() {
            return format!("{}{}", sample, Self::negative_case_suffix(rule_name));
        }
        candidates.sort();
        candidates.dedup();
        let selected =
            self.deterministic_negative_candidate_index(rule_name, sample, candidates.len());
        candidates.swap_remove(selected)
    }

    fn near_valid_negative_candidates(&self, sample: &str) -> Vec<String> {
        let chars: Vec<char> = sample.chars().collect();
        if chars.is_empty() {
            return Vec::new();
        }

        let mut candidates = Vec::new();

        if let Some((idx, ch)) = chars
            .iter()
            .enumerate()
            .rev()
            .find(|(_, ch)| Self::is_closing_delimiter(**ch))
        {
            let mut removed = chars.clone();
            removed.remove(idx);
            let removed_candidate: String = removed.into_iter().collect();
            if removed_candidate != sample {
                candidates.push(removed_candidate);
            }

            if let Some(replacement) = Self::mismatched_closing_delimiter(*ch) {
                let mut swapped = chars.clone();
                swapped[idx] = replacement;
                let swapped_candidate: String = swapped.into_iter().collect();
                if swapped_candidate != sample {
                    candidates.push(swapped_candidate);
                }
            }
        }

        if let Some((idx, ch)) = chars
            .iter()
            .enumerate()
            .find(|(_, ch)| Self::is_separator_candidate(**ch))
        {
            let mut duplicated = chars.clone();
            duplicated.insert(idx, *ch);
            let duplicated_candidate: String = duplicated.into_iter().collect();
            if duplicated_candidate != sample {
                candidates.push(duplicated_candidate);
            }

            if !sample.ends_with(*ch) {
                let mut appended = sample.to_string();
                appended.push(*ch);
                if appended != sample {
                    candidates.push(appended);
                }
            }
        }

        if let Some((idx, _)) = chars
            .iter()
            .enumerate()
            .find(|(_, ch)| !ch.is_whitespace() && !Self::is_delimiter_candidate(**ch))
        {
            let mut removed = chars.clone();
            removed.remove(idx);
            let removed_candidate: String = removed.into_iter().collect();
            if removed_candidate != sample && !removed_candidate.is_empty() {
                candidates.push(removed_candidate);
            }
        }

        if chars.len() > 1 {
            let mid_idx = chars.len() / 2;
            let mut removed = chars.clone();
            removed.remove(mid_idx);
            let removed_candidate: String = removed.into_iter().collect();
            if removed_candidate != sample && !removed_candidate.is_empty() {
                candidates.push(removed_candidate);
            }
        }

        candidates
    }

    fn deterministic_negative_candidate_index(
        &self,
        rule_name: &str,
        sample: &str,
        candidate_count: usize,
    ) -> usize {
        if candidate_count <= 1 {
            return 0;
        }

        let base_seed = self.config.seed.unwrap_or(0);
        let mut state = base_seed ^ 0xA24B_AED4_963E_E407;
        for byte in rule_name.as_bytes() {
            state ^= *byte as u64;
            state = state.wrapping_mul(1_099_511_628_211);
        }
        for byte in sample.as_bytes() {
            state ^= (*byte as u64).wrapping_add(0x9E37_79B9);
            state = state.wrapping_mul(1_099_511_628_211);
        }
        (state as usize) % candidate_count
    }

    fn is_closing_delimiter(ch: char) -> bool {
        matches!(ch, ')' | ']' | '}' | '>')
    }

    fn is_separator_candidate(ch: char) -> bool {
        matches!(ch, ',' | ';' | ':')
    }

    fn is_delimiter_candidate(ch: char) -> bool {
        matches!(
            ch,
            '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>' | ',' | ';' | ':'
        )
    }

    fn mismatched_closing_delimiter(ch: char) -> Option<char> {
        match ch {
            ')' => Some(']'),
            ']' => Some('}'),
            '}' => Some(')'),
            '>' => Some(')'),
            _ => None,
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
                    _ => annotation.ast().payload_text().to_string(),
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
            _ => extract_semantic_directive(annotation.ast().payload_text()),
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
        simple_generator_with_profiles(
            grammar_tree,
            rule_order,
            seed,
            StimuliMutationMode::Baseline,
            StimuliConstraintProfile::Baseline,
        )
    }

    fn simple_generator_with_mutation_mode<'a>(
        grammar_tree: &'a HashMap<String, ASTNode>,
        rule_order: &'a [String],
        seed: u64,
        mutation_mode: StimuliMutationMode,
    ) -> StimuliGenerator<'a> {
        simple_generator_with_profiles(
            grammar_tree,
            rule_order,
            seed,
            mutation_mode,
            StimuliConstraintProfile::Baseline,
        )
    }

    fn simple_generator_with_profiles<'a>(
        grammar_tree: &'a HashMap<String, ASTNode>,
        rule_order: &'a [String],
        seed: u64,
        mutation_mode: StimuliMutationMode,
        constraint_profile: StimuliConstraintProfile,
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
                target_pending_frontier_extra_stagnation: 8,
                target_generation_timeout_ms: 0,
                target_helper_generation_timeout_ms: 1000,
                recovery_mode: RecoveryStimuliMode::Baseline,
                mutation_mode,
                constraint_profile,
                negative_profile: StimuliNegativeProfile::Baseline,
                enforce_word_boundary_spacing: false,
                trace_verbosity: TraceVerbosity::None,
            },
        )
    }

    /// LEXICAL-ANNOTATIONS.3 — Obligation A (intra-token faithfulness). The line-comment regex
    /// `//[^\n]*(\n|$)` must NEVER be realized via the bare `$` branch when generated inline: every
    /// generated comment must carry its terminating newline, otherwise it swallows whatever follows
    /// on the line (the G.4.7-slice-3 defect). With the old uniform alternation pick ~half of
    /// realizations dropped the newline; the anchor-preference makes it 100%.
    #[test]
    fn obligation_a_line_comment_regex_always_terminates_with_newline() {
        let grammar_tree: HashMap<String, ASTNode> = HashMap::new();
        let rule_order: Vec<String> = Vec::new();
        let pattern = r"//[^\n]*(\n|$)";
        for seed in 0..64u64 {
            let mut g = simple_generator_with_profiles(
                &grammar_tree,
                &rule_order,
                seed,
                StimuliMutationMode::Baseline,
                StimuliConstraintProfile::Baseline,
            );
            let s = g.generate_regex_sample(pattern, "line_comment");
            assert!(s.starts_with("//"), "seed {seed}: expected a // comment, got {s:?}");
            assert!(
                s.ends_with('\n'),
                "seed {seed}: Obligation A — comment must terminate with a newline, got {s:?}"
            );
        }
    }

    fn simple_generator_with_pending_frontier_extra_stagnation<'a>(
        grammar_tree: &'a HashMap<String, ASTNode>,
        rule_order: &'a [String],
        seed: u64,
        target_pending_frontier_extra_stagnation: usize,
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
                target_pending_frontier_extra_stagnation,
                target_generation_timeout_ms: 0,
                target_helper_generation_timeout_ms: 1000,
                recovery_mode: RecoveryStimuliMode::Baseline,
                mutation_mode: StimuliMutationMode::Baseline,
                constraint_profile: StimuliConstraintProfile::Baseline,
                negative_profile: StimuliNegativeProfile::Baseline,
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
                target_pending_frontier_extra_stagnation: 8,
                target_generation_timeout_ms: 0,
                target_helper_generation_timeout_ms: 1000,
                recovery_mode: RecoveryStimuliMode::Baseline,
                mutation_mode: StimuliMutationMode::Baseline,
                constraint_profile: StimuliConstraintProfile::Baseline,
                negative_profile: StimuliNegativeProfile::Baseline,
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
                target_pending_frontier_extra_stagnation: 8,
                target_generation_timeout_ms: 0,
                target_helper_generation_timeout_ms: 1000,
                recovery_mode,
                mutation_mode: StimuliMutationMode::Baseline,
                constraint_profile: StimuliConstraintProfile::Baseline,
                negative_profile: StimuliNegativeProfile::Baseline,
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
    fn rare_branch_profile_boosts_uncovered_branch_multiplier() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("quoted_string", "common"),
                    token("quoted_string", "rare"),
                ],
            },
        );
        let rule_order = vec!["start".to_string()];

        let mut generator = simple_generator_with_profiles(
            &grammar_tree,
            &rule_order,
            4410,
            StimuliMutationMode::Baseline,
            StimuliConstraintProfile::RareBranchBiased,
        );
        for _ in 0..8 {
            generator
                .coverage
                .record_branch_selected("start::root", "start", "root", 2, 0);
            generator
                .coverage
                .record_branch_success("start::root", "start", "root", 2, 0);
        }

        let common_multiplier = generator.constraint_profile_branch_multiplier(
            "start",
            "root",
            0,
            &token("quoted_string", "common"),
            0,
            &[],
        );
        let rare_multiplier = generator.constraint_profile_branch_multiplier(
            "start",
            "root",
            1,
            &token("quoted_string", "rare"),
            0,
            &[],
        );

        assert!(
            rare_multiplier > common_multiplier,
            "rare-branch profile should boost under-hit branch multiplier (rare={}, common={})",
            rare_multiplier,
            common_multiplier
        );
    }

    #[test]
    fn deep_nesting_profile_biases_preferred_quantifier_repeat_upward() {
        let grammar_tree = HashMap::new();
        let rule_order: Vec<String> = Vec::new();

        let mut baseline = simple_generator_with_profiles(
            &grammar_tree,
            &rule_order,
            4411,
            StimuliMutationMode::Baseline,
            StimuliConstraintProfile::Baseline,
        );
        let mut deep = simple_generator_with_profiles(
            &grammar_tree,
            &rule_order,
            4411,
            StimuliMutationMode::Baseline,
            StimuliConstraintProfile::DeepNestingBiased,
        );

        let baseline_total: usize = (0..64)
            .map(|_| {
                baseline
                    .select_preferred_quantifier_repeat(0, 4)
                    .expect("baseline repeat choice should succeed")
            })
            .sum();
        let deep_total: usize = (0..64)
            .map(|_| {
                deep.select_preferred_quantifier_repeat(0, 4)
                    .expect("deep-nesting repeat choice should succeed")
            })
            .sum();

        assert!(
            deep_total > baseline_total,
            "deep-nesting profile should bias preferred repeats upward (deep_total={}, baseline_total={})",
            deep_total,
            baseline_total
        );
    }

    #[test]
    fn near_valid_negative_profile_prefers_structural_delimiter_corruption() {
        let grammar_tree = HashMap::new();
        let rule_order: Vec<String> = Vec::new();

        let generator = simple_generator_with_profiles(
            &grammar_tree,
            &rule_order,
            4412,
            StimuliMutationMode::Baseline,
            StimuliConstraintProfile::Baseline,
        );
        let mutated = generator.apply_near_valid_negative_profile("start", "call(arg)");

        assert_ne!(mutated, "call(arg)");
        assert!(
            mutated == "call(arg" || mutated == "call(arg]",
            "near-valid negative profile should prefer closing-delimiter corruption, got {:?}",
            mutated
        );
    }

    #[test]
    fn near_valid_negative_profile_can_append_or_duplicate_separator() {
        let grammar_tree = HashMap::new();
        let rule_order: Vec<String> = Vec::new();

        let generator = simple_generator_with_profiles(
            &grammar_tree,
            &rule_order,
            4413,
            StimuliMutationMode::Baseline,
            StimuliConstraintProfile::Baseline,
        );
        let mutated = generator.apply_near_valid_negative_profile("start", "a,b,c");

        assert_ne!(mutated, "a,b,c");
        assert!(
            mutated == "a,,b,c" || mutated == "a,b,c,",
            "near-valid negative profile should create separator-local corruption, got {:?}",
            mutated
        );
    }

    #[test]
    fn grammar_aware_mutation_replays_with_alternate_or_branch() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    token("quoted_string", "pre"),
                    ASTNode::Or {
                        alternatives: vec![
                            token("quoted_string", "left"),
                            token("quoted_string", "right"),
                        ],
                    },
                    token("quoted_string", "post"),
                ],
            },
        );
        let rule_order = vec!["start".to_string()];

        let mut baseline_generator = simple_generator(&grammar_tree, &rule_order, 4401);
        let baseline = baseline_generator
            .generate_many(1, Some("start"))
            .expect("baseline generation should succeed");

        let mut mutation_generator = simple_generator_with_mutation_mode(
            &grammar_tree,
            &rule_order,
            4401,
            StimuliMutationMode::GrammarAwareLocal,
        );
        let mutated = mutation_generator
            .generate_many(1, Some("start"))
            .expect("grammar-aware mutation should succeed");

        assert_ne!(
            baseline[0], mutated[0],
            "mutation mode should perturb the local OR choice"
        );
        assert!(
            mutated[0] == "preleftpost" || mutated[0] == "prerightpost",
            "mutated OR sample should stay grammar-valid, got {:?}",
            mutated[0]
        );
    }

    #[test]
    fn built_in_epsilon_rule_reference_generates_empty_string() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    token("quoted_string", "{"),
                    token("rule_reference", "epsilon"),
                    token("quoted_string", "}"),
                ],
            },
        );
        let rule_order = vec!["start".to_string()];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 4402);
        let generated = generator
            .generate_many(1, Some("start"))
            .expect("epsilon-backed generation should succeed");

        assert_eq!(generated, vec!["{}".to_string()]);
    }

    #[test]
    fn grammar_aware_mutation_replays_with_alternate_quantifier_count() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    token("quoted_string", "["),
                    ASTNode::Quantified {
                        element: Box::new(token("quoted_string", "x")),
                        quantifier: "1,3".to_string(),
                    },
                    token("quoted_string", "]"),
                ],
            },
        );
        let rule_order = vec!["start".to_string()];

        let mut baseline_generator = simple_generator(&grammar_tree, &rule_order, 4402);
        let baseline = baseline_generator
            .generate_many(1, Some("start"))
            .expect("baseline generation should succeed");

        let mut mutation_generator = simple_generator_with_mutation_mode(
            &grammar_tree,
            &rule_order,
            4402,
            StimuliMutationMode::GrammarAwareLocal,
        );
        let mutated = mutation_generator
            .generate_many(1, Some("start"))
            .expect("grammar-aware mutation should succeed");

        let regex = Regex::new(r"^\[x{1,3}\]$").expect("valid regex");
        assert_ne!(
            baseline[0], mutated[0],
            "mutation mode should perturb the quantifier repeat count"
        );
        assert!(
            regex.is_match(&mutated[0]),
            "mutated quantifier sample should stay grammar-valid, got {:?}",
            mutated[0]
        );
    }

    #[test]
    fn grammar_aware_mutation_falls_back_when_no_local_site_exists() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("quoted_string", "ok"));
        let rule_order = vec!["start".to_string()];

        let mut mutation_generator = simple_generator_with_mutation_mode(
            &grammar_tree,
            &rule_order,
            4403,
            StimuliMutationMode::GrammarAwareLocal,
        );
        let values = mutation_generator
            .generate_many(2, Some("start"))
            .expect("mutation fallback generation should succeed");

        assert!(
            values.iter().all(|value| value == "ok"),
            "no-site mutation fallback should preserve the baseline sample, got {:?}",
            values
        );
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
                target_pending_frontier_extra_stagnation: 8,
                target_generation_timeout_ms: 0,
                target_helper_generation_timeout_ms: 1000,
                recovery_mode: RecoveryStimuliMode::Baseline,
                mutation_mode: StimuliMutationMode::Baseline,
                constraint_profile: StimuliConstraintProfile::Baseline,
                negative_profile: StimuliNegativeProfile::Baseline,
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
    fn regex_leading_negation_excludes_structural_sigil_from_whole_content() {
        // SV-EXH-PROOF.2.3.2 (P-a): a permissive *leading* negated
        // class (`[^`\r\n]…` — the `non_directive_text` shape) must
        // keep its negated sigil out of the WHOLE generated run, not
        // just position 1; otherwise the closed-loop emits `` ` ``
        // mid-text and the parser re-lexes it as a directive, breaking
        // the sample's own round-trip.
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("regex", "[^`\\r\\n][^\\r\\n]*"));
        let rule_order = vec!["start".to_string()];
        for seed in [7u64, 42, 99, 123, 2024, 31337] {
            let mut generator = simple_generator(&grammar_tree, &rule_order, seed);
            let value = generator
                .generate_many(8, None)
                .expect("permissive leading-negation regex generation should succeed");
            for v in &value {
                assert!(
                    !v.contains('`'),
                    "leading [^`…] must exclude the backtick sigil from the ENTIRE content \
                     (seed {seed}): {v:?}"
                );
            }
        }

        // All-lanes-safe: a *restrictive positive* leading class
        // (`[a-z][a-z0-9]*`) is NOT a structural negation — P-a must
        // NOT over-restrict it (digits from the tail must still
        // appear across samples).
        let mut positive_tree = HashMap::new();
        positive_tree.insert("start".to_string(), token("regex", "[a-z][a-z0-9]*"));
        let positive_order = vec!["start".to_string()];
        let mut saw_digit = false;
        for seed in 0u64..40 {
            let mut generator = simple_generator(&positive_tree, &positive_order, seed);
            if let Ok(vals) = generator.generate_many(4, None) {
                if vals
                    .iter()
                    .any(|v| v.chars().any(|c| c.is_ascii_digit()))
                {
                    saw_digit = true;
                    break;
                }
            }
        }
        assert!(
            saw_digit,
            "P-a must not strip digits from a restrictive positive leading class \
             [a-z][a-z0-9]* (all-lanes-safe)"
        );

        // GRAMMAR-SCOPED `G`: a permissive content rule that negates
        // NOTHING of its own (`[^\r\n]+` — the `directive_tail` shape)
        // must STILL exclude a structural sigil that a *sibling*
        // content rule's author declared (`[^`\r\n]…`). This is the
        // generalization that covers `directive_tail`.
        let mut g_tree = HashMap::new();
        g_tree.insert("start".to_string(), token("regex", "[^\\r\\n]+"));
        g_tree.insert(
            "sibling_declares_sigil".to_string(),
            token("regex", "[^`\\r\\n][^\\r\\n]*"),
        );
        let g_order = vec!["start".to_string()];
        for seed in [1u64, 8, 64, 512, 4096, 65535] {
            let mut generator = simple_generator(&g_tree, &g_order, seed);
            let value = generator
                .generate_many(8, None)
                .expect("directive_tail-shape generation should succeed");
            for v in &value {
                assert!(
                    !v.contains('`'),
                    "grammar-scoped G: a permissive content rule ([^\\r\\n]+) must exclude a \
                     sigil declared by a sibling content rule ([^`…]) (seed {seed}): {v:?}"
                );
            }
        }
    }

    #[cfg(feature = "ebnf_dual_run")]
    #[test]
    fn real_sv_preprocessor_in_process_closer_scope_observation() {
        // SV-EXH-PROOF.2.3.2 — DECISIVE H2a-vs-H2b observation
        // (verify, do not reason — 5 mis-fires). Generate the real
        // `systemverilog_preprocessor_file` in-process and read the
        // ground-truth counters:
        //   closer_scopes_entered == 0  ⇒ H2a: the generator NEVER
        //     builds a `pp_conditional`; the failing samples'
        //     `` `ifdef ``/`` `endif `` are free-text the parser
        //     re-lexes (the original -0009 mode) — Mode-B targets a
        //     path the generator does not take.
        //   closer_scopes_entered  > 0  ⇒ the construct IS built; if
        //     the ce0 signature still appears the gap is H2b
        //     (consult bypass: cross-terminal concat / steering /
        //     literal-hint path).
        use crate::ast_pipeline::{PipelineConfig, RustASTPipeline};
        use crate::ebnf_frontend::parse_ebnf_file_to_raw_ast_envelope;

        let grammar_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../grammars/systemverilog_preprocessor.ebnf"
        );
        let envelope = parse_ebnf_file_to_raw_ast_envelope(grammar_path)
            .expect("parse systemverilog_preprocessor.ebnf");
        let raw_ast: Vec<JsonValue> = envelope
            .get("raw_ast")
            .and_then(|v| v.as_array())
            .expect("envelope.raw_ast array")
            .clone();
        let (grammar_tree, rule_order, _ann) = RustASTPipeline::new(PipelineConfig::default())
            .transform_from_raw_ast(&raw_ast)
            .expect("transform_from_raw_ast");

        let entry = "systemverilog_preprocessor_file";
        assert!(
            grammar_tree.contains_key(entry),
            "grammar must define the file entry rule"
        );

        // CORRECTED signature (the gate `shrunk_sample` is a bare
        // dangling `` ` ``, NOT an absorbed closer): every valid
        // backtick token in this grammar is `\`<ident>` /  `\`\`` /
        // `\`"`. A `\`` NOT followed by [A-Za-z_`"] (incl. trailing
        // or whitespace-followed) is a structurally-invalid dangling
        // sigil the parser correctly rejects = generator
        // over-generation. No parser needed.
        let kw_re = regex::Regex::new(r#"`(?:[^A-Za-z_`"]|$)"#).unwrap();

        let mut total = 0usize;
        let mut dangling_bt = 0usize;
        let mut first_example: Option<String> = None;
        let mut last_scopes = 0usize;
        for seed in [3u64, 17, 71, 256, 1024, 9001, 65537, 424242] {
            let mut generator = simple_generator(&grammar_tree, &rule_order, seed);
            for _ in 0..1200 {
                if let Ok(sample) = generator.generate_from_entry(entry) {
                    total += 1;
                    if kw_re.is_match(&sample) {
                        dangling_bt += 1;
                        if first_example.is_none() {
                            first_example = Some(sample);
                        }
                    }
                }
            }
            last_scopes = generator.closer_scopes_entered();
        }

        eprintln!(
            "SV-EXH-PROOF.2.3.2 dangling-backtick observation: samples={} \
             dangling_bt={} closer_scopes_entered(last)={} first={:?}",
            total, dangling_bt, last_scopes, first_example
        );

        assert!(total > 0, "in-process generation must produce samples");
        // Decisive: does the PRIMARY Baseline path itself emit a bare
        // dangling backtick (the actual gate shrunk-sample shape)? If
        // YES → fast in-process repro of the real defect; if NO → the
        // defect is target-drive-steering-specific.
        assert_eq!(
            dangling_bt, 0,
            "PRIMARY path emits a structurally-invalid dangling backtick \
             ({dangling_bt}/{total}) — fast in-process reproduction of the \
             gate's `shrunk_sample` defect; first example: {first_example:?}"
        );
    }

    #[cfg(feature = "ebnf_dual_run")]
    #[test]
    fn real_sv_preprocessor_grammar_closer_split_fires_for_pp_conditional() {
        // SV-EXH-PROOF.2.3.2 — DECISIVE H1 check (verify, do not
        // assume): load the REAL systemverilog_preprocessor.ebnf and
        // confirm `sequence_closer_split` actually fires for
        // `pp_conditional` and resolves the closer through
        // `pp_endif := kw_endif directive_tail? newline?` →
        // `kw_endif := inline_trivia /\`endif\b/` to the fixed
        // literal. If this fails, the detector is a no-op on the real
        // AST shape (root cause = detection); if it passes, the
        // detector fires and the remaining gap is elsewhere
        // (generator never builds the construct / consult bypass).
        use crate::ast_pipeline::{PipelineConfig, RustASTPipeline};
        use crate::ebnf_frontend::parse_ebnf_file_to_raw_ast_envelope;

        let grammar_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../grammars/systemverilog_preprocessor.ebnf"
        );
        let envelope = parse_ebnf_file_to_raw_ast_envelope(grammar_path)
            .expect("parse systemverilog_preprocessor.ebnf");
        let raw_ast: Vec<JsonValue> = envelope
            .get("raw_ast")
            .and_then(|v| v.as_array())
            .expect("envelope.raw_ast array")
            .clone();
        let (grammar_tree, _rule_order, _ann) = RustASTPipeline::new(PipelineConfig::default())
            .transform_from_raw_ast(&raw_ast)
            .expect("transform_from_raw_ast");

        let pp_conditional = grammar_tree
            .get("pp_conditional")
            .expect("grammar must define pp_conditional");

        // The generator reaches a rule body via generate_node, which
        // dispatches Or→(pick alt)→generate_sequence. Mirror that:
        // collect every top-level Sequence the generator could land
        // in (the node itself, or each Or alternative / unwrapped
        // single-child), then assert the detector fires on one.
        fn candidate_sequences(node: &ASTNode) -> Vec<&[ASTNode]> {
            match node {
                ASTNode::Sequence { elements } => vec![elements.as_slice()],
                ASTNode::Or { alternatives } => alternatives
                    .iter()
                    .flat_map(candidate_sequences)
                    .collect(),
                ASTNode::Atom {
                    value: ASTValue::Node(inner),
                } => candidate_sequences(inner),
                _ => Vec::new(),
            }
        }

        let detections: Vec<(usize, String)> = candidate_sequences(pp_conditional)
            .into_iter()
            .filter_map(|els| {
                StimuliGenerator::sequence_closer_split(&grammar_tree, els)
            })
            .collect();

        assert!(
            !detections.is_empty(),
            "H1: sequence_closer_split MUST fire for the real pp_conditional \
             (`pp_if_branch pp_elsif_branch* pp_else_branch? pp_endif`); \
             node shape = {:?}",
            pp_conditional
        );
        assert!(
            detections.iter().any(|(_, lex)| lex.contains("`endif")),
            "the resolved closer lexeme must be the fixed literal `endif` \
             (through pp_endif := kw_endif directive_tail? newline?); got {:?}",
            detections
        );
    }

    #[test]
    fn structural_closer_unspellable_by_free_terminal_while_construct_open() {
        // SV-EXH-PROOF.2.3.2 (Mode B), parser/EBNF-agnostic proof on a
        // SYNTHETIC grammar (no SV/preprocessor identifiers — the fix
        // is a general property of grammar structure):
        //
        //   wrap     := open  body*  close      (the `… item* CLOSE` idiom)
        //   open     := /OPEN/                  (fixed-literal opener)
        //   close    := /#END/                  (REQUIRED fixed-literal
        //                                        closer; STARTS with the
        //                                        structural sigil `#`)
        //   body     := free_tok | wrap         (recursive ⇒ nesting)
        //   free_tok := /#END|[a-z]+/           (FREE alternation — one
        //                                        alt IS the closer)
        //   content  := /[^#\r\n][^\r\n]*/      (a content rule whose
        //                                        author leading-negates
        //                                        `#` ⇒ declares `#` a
        //                                        grammar structural
        //                                        sigil, exactly like SV
        //                                        `non_directive_text`
        //                                        leading-negating `` ` ``)
        //
        // The hazard gate engages the closer-scope ONLY because the
        // closer `#END` begins with a grammar-declared structural
        // sigil (`#`, from `content`'s leading negation) — mirroring
        // the real `` `endif `` / `` ` `` relationship. Without the
        // fix the closed-loop freely picks the `#END` alternative
        // inside the body and the parser re-lexes it as the structural
        // closer; with it, the ONLY `#END` occurrences are genuine
        // structural closes ⇒ #opens == #closes in every sample
        // (balanced, round-trip-stable), nesting included.
        let close_kw = "#END";
        let open_kw = "OPEN";
        let wrap_seq = ASTNode::Sequence {
            elements: vec![
                token("rule_reference", "open"),
                ASTNode::Quantified {
                    element: Box::new(token("rule_reference", "body")),
                    quantifier: "*".to_string(),
                },
                token("rule_reference", "close"),
            ],
        };
        let body_or = ASTNode::Or {
            alternatives: vec![
                token("rule_reference", "free_tok"),
                token("rule_reference", "wrap"),
            ],
        };
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("wrap".to_string(), wrap_seq);
        grammar_tree.insert("open".to_string(), token("regex", open_kw));
        grammar_tree.insert("close".to_string(), token("regex", close_kw));
        grammar_tree.insert("body".to_string(), body_or);
        grammar_tree.insert("free_tok".to_string(), token("regex", "#END|[a-z]+"));
        // Declares `#` a grammar structural sigil (leading negation),
        // gating the hazard check on — parser/EBNF-agnostically.
        grammar_tree.insert(
            "content".to_string(),
            token("regex", r"[^#\r\n][^\r\n]*"),
        );
        let rule_order = vec![
            "wrap".to_string(),
            "open".to_string(),
            "close".to_string(),
            "body".to_string(),
            "free_tok".to_string(),
            "content".to_string(),
        ];

        // The closer-shape detector must fire on `wrap` and resolve
        // the closer lexeme purely structurally (rule-ref → regex).
        let detected = StimuliGenerator::sequence_closer_split(
            &grammar_tree,
            match grammar_tree.get("wrap").unwrap() {
                ASTNode::Sequence { elements } => elements,
                _ => unreachable!(),
            },
        );
        assert_eq!(
            detected,
            Some((2usize, close_kw.to_string())),
            "sequence_closer_split must detect `open body* close` and resolve \
             the fixed-literal closer through the rule reference"
        );
        // A fixed-literal terminal is exempt (nesting-safe); a free
        // terminal is subject to the check.
        assert!(
            StimuliGenerator::regex_fixed_literal(close_kw).is_some(),
            "the structural closer terminal must classify as fixed-literal (exempt)"
        );
        assert!(
            StimuliGenerator::regex_fixed_literal("#END|[a-z]+").is_none(),
            "the body's alternation terminal must classify as a FREE terminal"
        );

        let mut saw_nesting = false;
        for seed in [3u64, 17, 71, 256, 1024, 9001, 65537] {
            let mut generator = simple_generator(&grammar_tree, &rule_order, seed);
            let samples = generator
                .generate_many(12, None)
                .expect("closer-bearing recursive grammar generation should succeed");
            for s in &samples {
                let opens = s.matches(open_kw).count();
                let closes = s.matches(close_kw).count();
                assert!(
                    opens >= 1 && opens == closes,
                    "every emitted `wrap` must be structurally balanced — the \
                     closer lexeme must never be spelled by a free terminal while \
                     a construct is open (seed {seed}): opens={opens} closes={closes} {s:?}"
                );
                if opens >= 2 {
                    saw_nesting = true;
                }
            }
        }
        assert!(
            saw_nesting,
            "the recursive grammar must exercise NESTED constructs (whose own \
             fixed-literal structural closers stay generatable ⇒ nesting-safe)"
        );

        // Coverage-preserving: with NO closer-bearing construct open
        // (entry = the free terminal itself, empty forbidden stack),
        // the very same lexeme is still freely generatable — the fix
        // is strictly contextual, it does not shrink the language.
        let free_order = vec!["free_tok".to_string()];
        let mut saw_kw_standalone = false;
        for seed in 0u64..60 {
            let mut generator = simple_generator(&grammar_tree, &free_order, seed);
            if let Ok(vals) = generator.generate_many(6, None) {
                if vals.iter().any(|v| v == close_kw) {
                    saw_kw_standalone = true;
                    break;
                }
            }
        }
        assert!(
            saw_kw_standalone,
            "coverage-preserving: a free terminal must STILL be able to emit the \
             closer lexeme when no closer-bearing construct is open"
        );
    }

    #[test]
    fn line_greedy_content_forces_optional_newline_terminator() {
        // SV-EXH-PROOF.2.3.2 (line-terminator completeness),
        // parser/EBNF-agnostic proof on a SYNTHETIC grammar (no
        // SV/preprocessor identifiers — a general grammar-structure
        // property):
        //
        //   file      := directive+
        //   directive := kw content nl?     (line-greedy content +
        //                                    OPTIONAL newline terminator)
        //   kw        := /D/
        //   content   := /[^D\r\n]+/        (line-greedy: unbounded
        //                                    run of non-newline chars)
        //   nl        := /\r?\n/            (newline-only terminator)
        //
        // Without the fix the generator may skip `nl?`, so on reparse
        // `content` (line-greedy) absorbs the following `directive`
        // — exactly the `pp_define` macro-body-swallows-`` `endif ``
        // shape. The fix detects the `line-greedy … optional-newline
        // -terminator` sequence and force-emits the newline, so every
        // directive is newline-terminated (#newlines == #directives,
        // since `content` cannot produce `D` or `\n`).
        let directive_seq = ASTNode::Sequence {
            elements: vec![
                token("rule_reference", "kw"),
                token("rule_reference", "content"),
                ASTNode::Quantified {
                    element: Box::new(token("rule_reference", "nl")),
                    quantifier: "?".to_string(),
                },
            ],
        };
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "file".to_string(),
            ASTNode::Quantified {
                element: Box::new(token("rule_reference", "directive")),
                quantifier: "+".to_string(),
            },
        );
        grammar_tree.insert("directive".to_string(), directive_seq);
        grammar_tree.insert("kw".to_string(), token("regex", "D"));
        grammar_tree.insert("content".to_string(), token("regex", r"[^D\r\n]+"));
        grammar_tree.insert("nl".to_string(), token("regex", r"\r?\n"));
        let rule_order = vec![
            "file".to_string(),
            "directive".to_string(),
            "kw".to_string(),
            "content".to_string(),
            "nl".to_string(),
        ];

        // HIR classifiers (necessary, not sufficient — purely
        // structural, agnostic):
        assert!(StimuliGenerator::regex_is_newline_only(r"\r?\n"));
        assert!(StimuliGenerator::regex_is_newline_only(r"\n"));
        assert!(!StimuliGenerator::regex_is_newline_only(r"[^D\r\n]+"));
        assert!(!StimuliGenerator::regex_is_newline_only("D"));
        assert!(StimuliGenerator::regex_is_line_greedy(r"[^D\r\n]+"));
        assert!(StimuliGenerator::regex_is_line_greedy(r"[^\r\n]*"));
        assert!(!StimuliGenerator::regex_is_line_greedy(r"\r?\n"));
        assert!(!StimuliGenerator::regex_is_line_greedy("D"));

        // The detector must fire on `directive` (last elem = `nl?`
        // newline terminator; an earlier elem `content` is
        // line-greedy) and target the trailing index.
        let directive_elems = match grammar_tree.get("directive").unwrap() {
            ASTNode::Sequence { elements } => elements.as_slice(),
            _ => unreachable!(),
        };
        assert_eq!(
            StimuliGenerator::sequence_force_line_terminator_idx(
                &grammar_tree,
                directive_elems
            ),
            Some(2),
            "must detect `<line-greedy content> <optional newline terminator>` \
             and force the trailing newline"
        );
        // All-lanes-safe: a sequence WITHOUT a preceding line-greedy
        // element must NOT be forced (no spurious newline injection).
        let no_greedy = vec![
            token("rule_reference", "kw"),
            ASTNode::Quantified {
                element: Box::new(token("rule_reference", "nl")),
                quantifier: "?".to_string(),
            },
        ];
        assert_eq!(
            StimuliGenerator::sequence_force_line_terminator_idx(&grammar_tree, &no_greedy),
            None,
            "all-lanes-safe: no line-greedy predecessor ⇒ the optional \
             newline must stay optional (no forced injection)"
        );

        // Generation: every directive is newline-terminated ⇒ the
        // line-greedy `content` can never absorb the next directive.
        for seed in [2u64, 19, 88, 313, 2718, 99991] {
            let mut generator = simple_generator(&grammar_tree, &rule_order, seed);
            let samples = generator
                .generate_many(16, None)
                .expect("line-oriented grammar generation should succeed");
            for s in &samples {
                let directives = s.matches('D').count();
                let newlines = s.matches('\n').count();
                assert!(
                    directives >= 1 && newlines == directives,
                    "every directive must be newline-terminated (seed {seed}): \
                     directives={directives} newlines={newlines} {s:?}"
                );
            }
        }
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
                target_pending_frontier_extra_stagnation: 8,
                target_generation_timeout_ms: 0,
                target_helper_generation_timeout_ms: 1000,
                recovery_mode: RecoveryStimuliMode::Baseline,
                mutation_mode: StimuliMutationMode::Baseline,
                constraint_profile: StimuliConstraintProfile::Baseline,
                negative_profile: StimuliNegativeProfile::Baseline,
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
                target_pending_frontier_extra_stagnation: 8,
                target_generation_timeout_ms: 0,
                target_helper_generation_timeout_ms: 1000,
                recovery_mode: RecoveryStimuliMode::Baseline,
                mutation_mode: StimuliMutationMode::Baseline,
                constraint_profile: StimuliConstraintProfile::Baseline,
                negative_profile: StimuliNegativeProfile::Baseline,
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
    fn gap_report_marks_branches_with_missing_rule_references_unreachable() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("rule_reference", "wrapper"));
        grammar_tree.insert(
            "wrapper".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("rule_reference", "available"),
                    token("rule_reference", "missing_profile_branch"),
                ],
            },
        );
        grammar_tree.insert("available".to_string(), token("quoted_string", "ok"));
        let rule_order = vec![
            "start".to_string(),
            "wrapper".to_string(),
            "available".to_string(),
        ];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 778);
        let _ = generator
            .generate_many(8, Some("start"))
            .expect("generation should succeed via available branch");

        let report = generator
            .generate_gap_report(Some("start"), 1)
            .expect("gap report generation should succeed");

        assert!(
            report
                .reachable_branch_debt
                .iter()
                .all(|debt| debt.branch_id != "branch::wrapper::root#1"),
            "missing-rule branch should not remain actionable reachable debt"
        );
        let branch = report
            .unreachable_branch_debt
            .iter()
            .find(|debt| debt.branch_id == "branch::wrapper::root#1")
            .expect("missing-rule branch should be classified as unreachable");
        assert_eq!(branch.reason, "references_rule_missing_from_active_grammar");
        assert_eq!(
            branch.rule_references,
            vec!["missing_profile_branch".to_string()]
        );
        assert!(branch.uncovered_rule_references.is_empty());
        assert!(
            report
                .targets
                .iter()
                .all(|target| target.id != "branch::wrapper::root#1"),
            "missing-rule branch should not become a target-drive item"
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
    fn target_driven_generation_can_probe_unseen_low_priority_branch_once() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![token("quoted_string", " "), token("quoted_string", "--x\n")],
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
                        content: "[32, 1]".to_string(),
                    },
                },
            ],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 890);
        let report = generator
            .generate_gap_report(Some("start"), 1)
            .expect("gap report generation should succeed");
        assert!(
            report
                .targets
                .iter()
                .any(|target| target.id == "branch::start::root#1"),
            "expected lower-priority branch target to be present"
        );

        let (_samples, summary) = generator
            .generate_until_targets(Some("start"), &report.targets, 200)
            .expect("target-driven generation should succeed");

        assert!(
            summary
                .unresolved_targets
                .iter()
                .all(|status| status.id != "branch::start::root#1"),
            "target driving should retire the lower-priority branch target"
        );
    }

    #[test]
    fn preprocessor_item_repetition_inserts_newline_separator() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "systemverilog_preprocessor_file".to_string(),
            ASTNode::Quantified {
                element: Box::new(token("rule_reference", "pp_item")),
                quantifier: "2".to_string(),
            },
        );
        grammar_tree.insert(
            "pp_item".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("quoted_string", "`celldefine"),
                    token("quoted_string", "`endif"),
                ],
            },
        );
        let rule_order = vec![
            "systemverilog_preprocessor_file".to_string(),
            "pp_item".to_string(),
        ];

        let mut generator = StimuliGenerator::new(
            "systemverilog_preprocessor".to_string(),
            &grammar_tree,
            &rule_order,
            None,
            StimuliConfig {
                seed: Some(991),
                max_depth: 8,
                max_repeat: 4,
                max_rule_visits: 4,
                target_pending_frontier_extra_stagnation: 8,
                target_generation_timeout_ms: 0,
                target_helper_generation_timeout_ms: 1000,
                recovery_mode: RecoveryStimuliMode::Baseline,
                mutation_mode: StimuliMutationMode::Baseline,
                constraint_profile: StimuliConstraintProfile::Baseline,
                negative_profile: StimuliNegativeProfile::Baseline,
                enforce_word_boundary_spacing: false,
                trace_verbosity: TraceVerbosity::None,
            },
        );

        let sample = generator
            .generate_many(1, None)
            .expect("preprocessor repetition generation should succeed");
        assert!(
            sample[0].contains('\n'),
            "repeated preprocessor items should be newline-separated: {:?}",
            sample[0]
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
            .generate_until_targets_with_filter(Some("start"), &report.targets, 200, |sample, _| {
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
    fn target_driven_generation_filter_keeps_alternate_probe_helper_coverage() {
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

        let mut generator = simple_generator(&grammar_tree, &rule_order, 990);
        let report = generator
            .generate_gap_report(Some("start"), 1)
            .expect("gap report generation should succeed");
        let helper_targets: Vec<_> = report
            .targets
            .into_iter()
            .filter(|target| target.rule_name == "helper")
            .collect();
        assert!(
            !helper_targets.is_empty(),
            "expected helper targets to drive alternate-entry probing"
        );

        let mut saw_primary_entry = false;
        let mut saw_alternate_entry = false;
        let (samples, summary, validation) = generator
            .generate_until_targets_with_filter(
                Some("start"),
                &helper_targets,
                400,
                |sample, context| {
                    if context.is_primary_entry {
                        saw_primary_entry = true;
                        assert_eq!(context.primary_entry_rule, "start");
                        assert_eq!(context.generation_entry_rule, "start");
                    } else {
                        saw_alternate_entry = true;
                        assert_eq!(context.primary_entry_rule, "start");
                        assert_eq!(context.generation_entry_rule, "helper");
                    }
                    Ok(sample == "S")
                },
            )
            .expect("target-driven generation with helper-only filter should succeed");

        assert!(
            samples.iter().all(|sample| sample == "S"),
            "only primary-entry outputs should pass the strict filter"
        );
        assert!(
            saw_primary_entry,
            "helper-only validation should still evaluate some primary-entry outputs"
        );
        assert!(
            saw_alternate_entry,
            "helper-only validation should expose alternate-entry probe context"
        );
        assert!(
            validation.alternate_entry_attempts > 0,
            "helper-only targets should force alternate-entry probing"
        );
        assert_eq!(
            validation.alternate_entry_accepted_outputs, 0,
            "helper probe outputs should never pass the primary-entry filter in this setup"
        );
        assert!(
            validation.alternate_entry_rejected_outputs > 0,
            "helper probe outputs should be rejected by the primary-entry filter in this setup"
        );
        assert_eq!(
            summary.resolved_targets, summary.total_targets,
            "alternate-entry helper probes should still retire helper-only target debt"
        );
        assert!(
            summary.unresolved_targets.is_empty(),
            "helper-only targets should resolve even when helper outputs fail the primary-entry filter"
        );

        let helper_group = generator
            .coverage_metrics()
            .branch_groups
            .get("helper::root")
            .expect("helper branch group should exist");
        assert!(
            helper_group.success_counts.iter().all(|count| *count > 0),
            "helper branch successes should be retained after rejected alternate-entry probes"
        );
    }

    #[test]
    fn target_driven_generation_retries_target_branch_with_depth_slack() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("rule_reference", "long_branch"),
                    token("quoted_string", "S"),
                ],
            },
        );
        grammar_tree.insert("long_branch".to_string(), token("rule_reference", "helper"));
        grammar_tree.insert("helper".to_string(), token("rule_reference", "leaf"));
        grammar_tree.insert("leaf".to_string(), token("quoted_string", "L"));
        let rule_order = vec![
            "start".to_string(),
            "long_branch".to_string(),
            "helper".to_string(),
            "leaf".to_string(),
        ];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 2501);
        generator.config.max_depth = 3;

        let report = generator
            .generate_gap_report(Some("start"), 1)
            .expect("gap report generation should succeed");

        let (_samples, summary) = generator
            .generate_until_targets(Some("start"), &report.targets, 400)
            .expect("target-driven generation should succeed");

        assert!(
            summary
                .unresolved_targets
                .iter()
                .all(|status| status.id != "branch::start::root#0"),
            "depth-slack retry should retire the previously depth-blocked start branch target"
        );

        let group = generator
            .coverage_metrics()
            .branch_groups
            .get("start::root")
            .expect("branch group should exist");
        assert!(
            group.success_counts.first().copied().unwrap_or(0) > 0,
            "targeted start branch should record at least one success after depth-slack retry"
        );
    }

    #[test]
    fn gap_report_surfaces_top_branch_failure_reasons() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![token("quoted_string", "L"), token("quoted_string", "R")],
            },
        );
        let rule_order = vec!["start".to_string()];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 1991);
        generator
            .coverage
            .record_branch_selected("start::root", "start", "root", 2, 0);
        generator.coverage.record_branch_failure(
            "start::root",
            "start",
            "root",
            2,
            0,
            "max depth exceeded while generating expression\ncontext that should be trimmed",
        );
        generator
            .coverage
            .record_branch_selected("start::root", "start", "root", 2, 0);
        generator.coverage.record_branch_failure(
            "start::root",
            "start",
            "root",
            2,
            0,
            "max depth exceeded while generating expression",
        );
        generator
            .coverage
            .record_branch_selected("start::root", "start", "root", 2, 0);
        generator.coverage.record_branch_failure(
            "start::root",
            "start",
            "root",
            2,
            0,
            "visit limit exceeded",
        );

        let report = generator
            .generate_gap_report(Some("start"), 1)
            .expect("gap report generation should succeed");
        let branch = report
            .reachable_branch_debt
            .iter()
            .find(|debt| debt.branch_id == "branch::start::root#0")
            .expect("branch debt should be present");

        assert_eq!(branch.reason, "selected_but_failed");
        assert_eq!(
            branch.top_failure_reasons,
            vec![
                BranchFailureReasonCount {
                    reason: "max depth exceeded while generating expression".to_string(),
                    count: 2,
                },
                BranchFailureReasonCount {
                    reason: "visit limit exceeded".to_string(),
                    count: 1,
                },
            ]
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
    fn coverage_guidance_multiplier_preserves_dependency_blocked_target_branch() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("rule_reference", "helper"),
                    token("quoted_string", "R"),
                ],
            },
        );
        grammar_tree.insert("helper".to_string(), token("quoted_string", "H"));
        let rule_order = vec!["start".to_string(), "helper".to_string()];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 8911);
        generator
            .target_plan
            .branch_thresholds
            .entry("start::root".to_string())
            .or_default()
            .insert(0, 1);
        generator
            .target_plan
            .rule_thresholds
            .insert("helper".to_string(), 1);

        let group = generator
            .coverage
            .branch_groups
            .get_mut("start::root")
            .expect("branch group should exist");
        group.selected_counts = vec![64, 64];
        group.success_counts = vec![0, 16];

        let alternatives = match grammar_tree.get("start").expect("rule should exist") {
            ASTNode::Or { alternatives } => alternatives,
            other => panic!("expected OR node, got {:?}", other),
        };

        let blocked_multiplier =
            generator.coverage_guidance_multiplier("start", "root", 0, &alternatives[0]);

        generator
            .coverage
            .rule_success_hits
            .insert("helper".to_string(), 1);
        let throttled_multiplier =
            generator.coverage_guidance_multiplier("start", "root", 0, &alternatives[0]);

        assert!(
            blocked_multiplier > throttled_multiplier,
            "dependency-blocked target branches should not be throttled as harshly before their targeted dependency has any success history (blocked={}, throttled={})",
            blocked_multiplier,
            throttled_multiplier
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
            target_timeout_errors: 0,
            helper_timeout_errors: 0,
        };

        assert_eq!(generator.target_probe_threshold(&pending), 8);
        assert_eq!(
            generator.target_probe_threshold_for_validation(&pending, &validation),
            24,
            "validation-aware replay should back off helper probing when low-yield alternate attempts dominate and primary entry rejects are present"
        );
    }

    #[test]
    fn target_drive_progress_interval_scales_with_attempt_budget() {
        assert_eq!(StimuliGenerator::target_drive_progress_interval(32), 8);
        assert_eq!(StimuliGenerator::target_drive_progress_interval(200), 16);
        assert_eq!(StimuliGenerator::target_drive_progress_interval(800), 32);
        assert_eq!(StimuliGenerator::target_drive_progress_interval(3000), 64);
        assert_eq!(StimuliGenerator::target_drive_progress_interval(5000), 128);
    }

    #[test]
    fn target_drive_progress_traces_first_last_and_periodic_attempts() {
        assert!(StimuliGenerator::should_trace_target_drive_progress(
            1, 5000
        ));
        assert!(StimuliGenerator::should_trace_target_drive_progress(
            128, 5000
        ));
        assert!(StimuliGenerator::should_trace_target_drive_progress(
            5000, 5000
        ));
        assert!(!StimuliGenerator::should_trace_target_drive_progress(
            127, 5000
        ));
        assert!(!StimuliGenerator::should_trace_target_drive_progress(
            129, 5000
        ));
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
            target_timeout_errors: 0,
            helper_timeout_errors: 0,
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
            target_timeout_errors: 0,
            helper_timeout_errors: 0,
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
    fn target_probe_prefers_observed_high_yield_dependency_candidate() {
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

        let mut generator = simple_generator(&grammar_tree, &rule_order, 910);
        generator
            .target_plan
            .rule_thresholds
            .insert("helper_a".to_string(), 2);
        generator
            .target_plan
            .rule_thresholds
            .insert("helper_b".to_string(), 2);
        generator.target_probe_history.insert(
            "helper_b".to_string(),
            TargetProbeHistory {
                attempts: 1,
                successful_generations: 1,
                resolved_delta_total: 6,
                best_resolved_delta: 6,
            },
        );

        let pending = vec![
            TargetCoverageStatus {
                id: "branch::start::root::0".to_string(),
                target_type: StimuliCoverageTargetType::Branch,
                rule_name: "start".to_string(),
                node_path: Some("root".to_string()),
                branch_index: Some(0),
                current_successes: 0,
                required_successes: 1,
                remaining_successes: 3,
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
                required_successes: 1,
                remaining_successes: 3,
                priority_score: 100,
                reason: "selected_but_failed".to_string(),
                depends_on: vec!["helper_b".to_string()],
            },
        ];

        assert_eq!(
            generator.select_target_probe_rule(&pending, "start"),
            Some("helper_b".to_string()),
            "once two dependency candidates have equal static leverage, observed replay payoff should break the tie toward the helper that has already retired more target debt"
        );
    }

    #[test]
    fn target_probe_prefers_dependency_candidate_with_literalish_hint_when_leverage_ties() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("rule_reference", "helper_a"),
                    token("rule_reference", "helper_b"),
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

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "helper_b".to_string(),
            vec![SemanticAnnotation::Named {
                name: "sample".to_string(),
                ast: UnifiedSemanticAST::Structured {
                    canonical: "\"seed-b\"".to_string(),
                    value: UnifiedSemanticValue::String("seed-b".to_string()),
                },
            }],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 898);
        generator
            .target_plan
            .rule_thresholds
            .insert("helper_a".to_string(), 1);
        generator
            .target_plan
            .rule_thresholds
            .insert("helper_b".to_string(), 1);

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
                required_successes: 1,
                remaining_successes: 1,
                priority_score: 100,
                reason: "selected_but_failed".to_string(),
                depends_on: vec!["helper_b".to_string()],
            },
        ];

        assert_eq!(
            generator.select_target_probe_rule(&pending, "start"),
            Some("helper_b".to_string()),
            "dependency probing should prefer the equally-impactful helper that carries a literalish sample hint"
        );
    }

    #[test]
    fn target_probe_fallback_prefers_pending_rule_with_literalish_hint() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("rule_reference", "helper_a"),
                    token("rule_reference", "helper_b"),
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

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "helper_b".to_string(),
            vec![SemanticAnnotation::Named {
                name: "sample".to_string(),
                ast: UnifiedSemanticAST::Structured {
                    canonical: "\"seed-b\"".to_string(),
                    value: UnifiedSemanticValue::String("seed-b".to_string()),
                },
            }],
        );

        let generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 899);
        let pending = vec![
            TargetCoverageStatus {
                id: "branch::helper_a::root::0".to_string(),
                target_type: StimuliCoverageTargetType::Branch,
                rule_name: "helper_a".to_string(),
                node_path: Some("root".to_string()),
                branch_index: Some(0),
                current_successes: 0,
                required_successes: 1,
                remaining_successes: 1,
                priority_score: 100,
                reason: "never_selected".to_string(),
                depends_on: Vec::new(),
            },
            TargetCoverageStatus {
                id: "branch::helper_b::root::0".to_string(),
                target_type: StimuliCoverageTargetType::Branch,
                rule_name: "helper_b".to_string(),
                node_path: Some("root".to_string()),
                branch_index: Some(0),
                current_successes: 0,
                required_successes: 1,
                remaining_successes: 1,
                priority_score: 100,
                reason: "never_selected".to_string(),
                depends_on: Vec::new(),
            },
        ];

        assert_eq!(
            generator.select_target_probe_rule(&pending, "start"),
            Some("helper_b".to_string()),
            "fallback probing should prefer a pending non-entry rule that has a literalish hint"
        );
    }

    #[test]
    fn target_probe_prefers_broad_pending_frontier_over_marginal_dependency() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("rule_reference", "dependency_helper"),
                    token("rule_reference", "pending_helper"),
                ],
            },
        );
        grammar_tree.insert("dependency_helper".to_string(), token("quoted_string", "D"));
        grammar_tree.insert("pending_helper".to_string(), token("quoted_string", "P"));
        let rule_order = vec![
            "start".to_string(),
            "dependency_helper".to_string(),
            "pending_helper".to_string(),
        ];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 912);
        generator
            .target_plan
            .rule_thresholds
            .insert("dependency_helper".to_string(), 1);

        let mut pending = vec![TargetCoverageStatus {
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
            depends_on: vec!["dependency_helper".to_string()],
        }];
        for branch_index in 0..8 {
            pending.push(TargetCoverageStatus {
                id: format!("branch::pending_helper::root::{}", branch_index),
                target_type: StimuliCoverageTargetType::Branch,
                rule_name: "pending_helper".to_string(),
                node_path: Some("root".to_string()),
                branch_index: Some(branch_index),
                current_successes: 0,
                required_successes: 1,
                remaining_successes: 1,
                priority_score: 80,
                reason: "never_selected".to_string(),
                depends_on: Vec::new(),
            });
        }

        assert_eq!(
            generator.select_target_probe_rule(&pending, "start"),
            Some("pending_helper".to_string()),
            "a wide untouched pending frontier should outrank a fresh one-shot dependency candidate"
        );
    }

    #[test]
    fn target_probe_validation_prefers_broad_pending_frontier_when_not_primary_bound() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("rule_reference", "dependency_helper"),
                    token("rule_reference", "pending_helper"),
                ],
            },
        );
        grammar_tree.insert("dependency_helper".to_string(), token("quoted_string", "D"));
        grammar_tree.insert("pending_helper".to_string(), token("quoted_string", "P"));
        let rule_order = vec![
            "start".to_string(),
            "dependency_helper".to_string(),
            "pending_helper".to_string(),
        ];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 913);
        generator
            .target_plan
            .rule_thresholds
            .insert("dependency_helper".to_string(), 1);

        let mut pending = vec![TargetCoverageStatus {
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
            depends_on: vec!["dependency_helper".to_string()],
        }];
        for branch_index in 0..8 {
            pending.push(TargetCoverageStatus {
                id: format!("branch::pending_helper::root::{}", branch_index),
                target_type: StimuliCoverageTargetType::Branch,
                rule_name: "pending_helper".to_string(),
                node_path: Some("root".to_string()),
                branch_index: Some(branch_index),
                current_successes: 0,
                required_successes: 1,
                remaining_successes: 1,
                priority_score: 80,
                reason: "never_selected".to_string(),
                depends_on: Vec::new(),
            });
        }

        let validation = TargetDriveValidationSummary {
            validated_outputs: 2,
            accepted_outputs: 2,
            rejected_outputs: 0,
            alternate_entry_attempts: 2,
            alternate_entry_accepted_outputs: 1,
            alternate_entry_rejected_outputs: 1,
            target_timeout_errors: 0,
            helper_timeout_errors: 0,
        };

        assert_eq!(
            generator.select_target_probe_rule_for_validation(&pending, "start", &validation),
            Some("pending_helper".to_string()),
            "validation-aware probing should use the same broad-frontier preference when it is not currently backing off to the primary entry"
        );
    }

    #[test]
    fn target_probe_staging_keeps_dependency_before_pending_frontier_unlock() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("rule_reference", "dependency_helper"),
                    token("rule_reference", "pending_helper"),
                ],
            },
        );
        grammar_tree.insert("dependency_helper".to_string(), token("quoted_string", "D"));
        grammar_tree.insert("pending_helper".to_string(), token("quoted_string", "P"));
        let rule_order = vec![
            "start".to_string(),
            "dependency_helper".to_string(),
            "pending_helper".to_string(),
        ];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 914);
        generator
            .target_plan
            .rule_thresholds
            .insert("dependency_helper".to_string(), 1);

        let mut pending = vec![TargetCoverageStatus {
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
            depends_on: vec!["dependency_helper".to_string()],
        }];
        for branch_index in 0..8 {
            pending.push(TargetCoverageStatus {
                id: format!("branch::pending_helper::root::{}", branch_index),
                target_type: StimuliCoverageTargetType::Branch,
                rule_name: "pending_helper".to_string(),
                node_path: Some("root".to_string()),
                branch_index: Some(branch_index),
                current_successes: 0,
                required_successes: 1,
                remaining_successes: 1,
                priority_score: 80,
                reason: "never_selected".to_string(),
                depends_on: Vec::new(),
            });
        }

        assert_eq!(
            generator.select_target_probe_rule_with_stagnation(&pending, "start", 8, 8),
            Some("dependency_helper".to_string()),
            "broad pending frontiers should stay locked until replay has stalled beyond the ordinary helper threshold"
        );
    }

    #[test]
    fn target_probe_staging_unlocks_pending_frontier_after_extra_stagnation() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("rule_reference", "dependency_helper"),
                    token("rule_reference", "pending_helper"),
                ],
            },
        );
        grammar_tree.insert("dependency_helper".to_string(), token("quoted_string", "D"));
        grammar_tree.insert("pending_helper".to_string(), token("quoted_string", "P"));
        let rule_order = vec![
            "start".to_string(),
            "dependency_helper".to_string(),
            "pending_helper".to_string(),
        ];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 915);
        generator
            .target_plan
            .rule_thresholds
            .insert("dependency_helper".to_string(), 1);

        let mut pending = vec![TargetCoverageStatus {
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
            depends_on: vec!["dependency_helper".to_string()],
        }];
        for branch_index in 0..8 {
            pending.push(TargetCoverageStatus {
                id: format!("branch::pending_helper::root::{}", branch_index),
                target_type: StimuliCoverageTargetType::Branch,
                rule_name: "pending_helper".to_string(),
                node_path: Some("root".to_string()),
                branch_index: Some(branch_index),
                current_successes: 0,
                required_successes: 1,
                remaining_successes: 1,
                priority_score: 80,
                reason: "never_selected".to_string(),
                depends_on: Vec::new(),
            });
        }

        assert_eq!(
            generator.select_target_probe_rule_with_stagnation(&pending, "start", 16, 8),
            Some("pending_helper".to_string()),
            "once replay has stayed stagnant past the extra unlock window, the broad pending frontier should become eligible"
        );
    }

    #[test]
    fn target_probe_configurable_pending_frontier_unlock_can_choose_heavy_lane_earlier() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("rule_reference", "dependency_helper"),
                    token("rule_reference", "pending_helper"),
                ],
            },
        );
        grammar_tree.insert("dependency_helper".to_string(), token("quoted_string", "D"));
        grammar_tree.insert("pending_helper".to_string(), token("quoted_string", "P"));
        let rule_order = vec![
            "start".to_string(),
            "dependency_helper".to_string(),
            "pending_helper".to_string(),
        ];

        let mut generator = simple_generator_with_pending_frontier_extra_stagnation(
            &grammar_tree,
            &rule_order,
            916,
            0,
        );
        generator
            .target_plan
            .rule_thresholds
            .insert("dependency_helper".to_string(), 1);

        let mut pending = vec![TargetCoverageStatus {
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
            depends_on: vec!["dependency_helper".to_string()],
        }];
        for branch_index in 0..8 {
            pending.push(TargetCoverageStatus {
                id: format!("branch::pending_helper::root::{}", branch_index),
                target_type: StimuliCoverageTargetType::Branch,
                rule_name: "pending_helper".to_string(),
                node_path: Some("root".to_string()),
                branch_index: Some(branch_index),
                current_successes: 0,
                required_successes: 1,
                remaining_successes: 1,
                priority_score: 80,
                reason: "never_selected".to_string(),
                depends_on: Vec::new(),
            });
        }

        assert_eq!(
            generator.select_target_probe_rule_with_stagnation(&pending, "start", 8, 8),
            Some("pending_helper".to_string()),
            "setting pending-frontier extra stagnation to zero should make the heavy pending-frontier lane eligible as soon as helper probing unlocks"
        );
    }

    #[test]
    fn helper_generation_timeout_aborts_entry_and_restores_generator_state() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("quoted_string", "ok"));
        let rule_order = vec!["start".to_string()];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 917);
        let err = generator
            .generate_from_entry_with_optional_timeout(
                "start",
                Some(GenerationTimeoutBudget {
                    step_budget: 0,
                    error_prefix: HELPER_TIMEOUT_ERROR_PREFIX,
                    budget_ms: 0,
                }),
            )
            .expect_err("expired helper timeout should abort generation");
        assert!(
            err.to_string()
                .contains("Stimuli generation helper timeout exceeded"),
            "timeout error should explain the helper budget, got: {err}"
        );
        assert!(
            generator.active_generation_deadline.is_none(),
            "helper timeout wrapper should restore the generator deadline state"
        );

        let sample = generator
            .generate_from_entry("start")
            .expect("subsequent direct generation should still succeed");
        assert_eq!(sample, "ok");
    }

    #[test]
    fn target_generation_timeout_aborts_entry_and_restores_generator_state() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert("start".to_string(), token("quoted_string", "ok"));
        let rule_order = vec!["start".to_string()];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 918);
        let err = generator
            .generate_from_entry_with_optional_timeout(
                "start",
                Some(GenerationTimeoutBudget {
                    step_budget: 0,
                    error_prefix: TARGET_TIMEOUT_ERROR_PREFIX,
                    budget_ms: 0,
                }),
            )
            .expect_err("expired primary target timeout should abort generation");
        assert!(
            err.to_string()
                .contains("Stimuli generation target timeout exceeded"),
            "timeout error should explain the target budget, got: {err}"
        );
        assert!(
            generator.active_generation_deadline.is_none(),
            "target timeout wrapper should restore the generator deadline state"
        );

        let sample = generator
            .generate_from_entry("start")
            .expect("subsequent direct generation should still succeed");
        assert_eq!(sample, "ok");
    }

    #[test]
    fn witness_pass_resolves_uncovered_targets_monotonically() {
        // SV-EXH-PROOF.7.4.3: a fresh generator has all reachable rules uncovered; the
        // appended witness pass generates each from its own rule and must RESOLVE them
        // (monotone gain), without the diverse pass having run.
        let grammar_tree = synthetic_reach_grammar();
        let rule_order: Vec<String> = grammar_tree.keys().cloned().collect();
        let mut generator = simple_generator(&grammar_tree, &rule_order, 7);

        let report = generator
            .generate_gap_report(Some("start"), 1)
            .expect("gap report");
        assert!(
            !report.targets.is_empty(),
            "a fresh generator must list uncovered reachable targets"
        );
        let before = generator.evaluate_target_statuses(&report.targets).len();

        let (witnesses, summary) = generator
            .generate_target_witnesses(&report.targets)
            .expect("witness pass");

        assert!(
            summary.witnesses_generated > 0,
            "the witness pass must generate witnesses, got {:?}",
            summary
        );
        assert!(
            summary.resolved_after > summary.resolved_before,
            "witnesses must RESOLVE targets (monotone gain): {:?}",
            summary
        );
        let after = generator.evaluate_target_statuses(&report.targets).len();
        assert!(
            after < before,
            "residual must shrink after the witness pass ({before} -> {after})"
        );
        assert!(!witnesses.is_empty());
    }

    #[test]
    fn classify_generation_error_maps_each_reason() {
        // DIAG-SEVERITY.3.1: the canonical classifier is the single source of truth for
        // the error-by-reason taxonomy. Each structural-budget / timeout prefix must map
        // to its reason; anything else is Other (incl. expected PEG backtracking).
        let depth = anyhow::anyhow!(
            "Stimuli generation depth exceeded max_depth=24 while expanding rule 'x'"
        );
        let visits = anyhow::anyhow!("Stimuli generation exceeded max_rule_visits=8 for rule 'y'");
        let target = anyhow::anyhow!("Stimuli generation target timeout exceeded after 1000ms");
        let helper = anyhow::anyhow!("Stimuli generation helper timeout exceeded after 1000ms");
        let other = anyhow::anyhow!("All explicit branch probabilities are zero");

        assert_eq!(
            StimuliGenerator::classify_generation_error(&depth),
            GenerationErrorReason::DepthExceeded
        );
        assert_eq!(
            StimuliGenerator::classify_generation_error(&visits),
            GenerationErrorReason::RuleVisitLimit
        );
        assert_eq!(
            StimuliGenerator::classify_generation_error(&target),
            GenerationErrorReason::TargetTimeout
        );
        assert_eq!(
            StimuliGenerator::classify_generation_error(&helper),
            GenerationErrorReason::HelperTimeout
        );
        assert_eq!(
            StimuliGenerator::classify_generation_error(&other),
            GenerationErrorReason::Other
        );
    }

    #[test]
    fn target_drive_summary_reports_helper_timeout_errors() {
        let summary = TargetDriveSummary {
            entry_rule: "start".to_string(),
            attempts: 12,
            generation_successes: 5,
            generation_errors: 3,
            depth_exceeded_errors: 2,
            rule_visit_limit_errors: 1,
            target_timeout_errors: 1,
            helper_timeout_errors: 2,
            total_targets: 9,
            applied_targets: 9,
            resolved_targets: 7,
            unresolved_targets: Vec::new(),
            reach_plan_activations: 4,
        };

        assert!(
            summary.summary_line().contains("helper_timeout_errors=2"),
            "target-drive summaries should expose helper timeout counts for auditability"
        );
        assert!(
            summary.summary_line().contains("depth_exceeded_errors=2"),
            "DIAG-SEVERITY.3: target-drive summaries must expose depth-exceeded counts (the masked SV-residual cause)"
        );
        assert!(
            summary.summary_line().contains("rule_visit_limit_errors=1"),
            "DIAG-SEVERITY.3.1: target-drive summaries must expose rule-visit-limit counts (the 2nd masked structural-budget cause)"
        );
        assert!(
            summary.summary_line().contains("target_timeout_errors=1"),
            "target-drive summaries should expose primary target timeout counts for auditability"
        );
        assert!(
            summary.summary_line().contains("reach_plan_activations=4"),
            "target-drive summaries should expose the .7.2.4 reach-driver activation count (.7.2.5 measurability)"
        );
    }

    #[test]
    fn target_probe_validation_keeps_observed_high_yield_dependency_under_alternate_churn() {
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

        let mut generator = simple_generator(&grammar_tree, &rule_order, 911);
        generator
            .target_plan
            .rule_thresholds
            .insert("helper".to_string(), 2);
        generator
            .coverage
            .rule_success_hits
            .insert("helper".to_string(), 1);
        generator.target_probe_history.insert(
            "helper".to_string(),
            TargetProbeHistory {
                attempts: 1,
                successful_generations: 1,
                resolved_delta_total: 5,
                best_resolved_delta: 5,
            },
        );

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
            target_timeout_errors: 0,
            helper_timeout_errors: 0,
        };

        assert_eq!(
            generator.select_target_probe_rule_for_validation(&pending, "start", &validation),
            Some("helper".to_string()),
            "validation-aware probing should keep a marginal dependency helper once that helper has already shown strong observed payoff in the same replay run"
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
            target_timeout_errors: 0,
            helper_timeout_errors: 0,
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
    fn semantic_usage_stimuli_literalish_directives_accept_structured_and_legacy_alias_hints() {
        let cases = [
            (
                None,
                UnifiedSemanticAST::Structured {
                    canonical: "\"structured-bare\"".to_string(),
                    value: UnifiedSemanticValue::String("structured-bare".to_string()),
                },
                "structured-bare",
            ),
            (
                Some("sample"),
                UnifiedSemanticAST::Structured {
                    canonical: "\"sample-seed\"".to_string(),
                    value: UnifiedSemanticValue::String("sample-seed".to_string()),
                },
                "sample-seed",
            ),
            (
                Some("literal"),
                UnifiedSemanticAST::Structured {
                    canonical: "\"literal-seed\"".to_string(),
                    value: UnifiedSemanticValue::String("literal-seed".to_string()),
                },
                "literal-seed",
            ),
            (
                Some("example"),
                UnifiedSemanticAST::Raw {
                    content: "'example-seed'".to_string(),
                },
                "example-seed",
            ),
            (
                Some("stimulus"),
                UnifiedSemanticAST::Structured {
                    canonical: "\"legacy-stimulus-seed\"".to_string(),
                    value: UnifiedSemanticValue::String("legacy-stimulus-seed".to_string()),
                },
                "legacy-stimulus-seed",
            ),
        ];

        for (directive_name, ast, expected) in cases {
            let mut grammar_tree = HashMap::new();
            grammar_tree.insert("start".to_string(), token("regex", "^[A-Z]{4}$"));
            let rule_order = vec!["start".to_string()];

            let mut annotations = Annotations::default();
            let annotation = if let Some(name) = directive_name {
                SemanticAnnotation::Named {
                    name: name.to_string(),
                    ast,
                }
            } else {
                ast.into()
            };
            annotations
                .semantic_annotations
                .insert("start".to_string(), vec![annotation]);

            let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9293);
            let values = generator
                .generate_many(1, Some("start"))
                .expect("literalish semantic hint generation should succeed");
            assert_eq!(
                values[0], expected,
                "directive {:?} should surface literal hint {:?}, got {:?}",
                directive_name, expected, values[0]
            );
        }
    }

    #[test]
    fn semantic_usage_stimuli_literalish_directives_override_non_regex_rules() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    token("quoted_string", "alpha"),
                    token("quoted_string", "beta"),
                ],
            },
        );
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![SemanticAnnotation::Named {
                name: "sample".to_string(),
                ast: UnifiedSemanticAST::Structured {
                    canonical: "\"seeded-branch\"".to_string(),
                    value: UnifiedSemanticValue::String("seeded-branch".to_string()),
                },
            }],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9294);
        let values = generator
            .generate_many(1, Some("start"))
            .expect("non-regex literalish hint generation should succeed");
        assert_eq!(values[0], "seeded-branch");
    }

    #[test]
    fn semantic_usage_stimuli_literalish_directives_override_or_branch_generation() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("quoted_string", "fallback"),
                    token("quoted_string", "other"),
                ],
            },
        );
        let rule_order = vec!["start".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "start".to_string(),
            vec![SemanticAnnotation::Named {
                name: "branch_policy".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: "ordered".to_string(),
                },
            }],
        );
        annotations.branch_semantic_annotations.insert(
            "start".to_string(),
            vec![
                vec![SemanticAnnotation::Named {
                    name: "sample".to_string(),
                    ast: UnifiedSemanticAST::Structured {
                        canonical: "\"branch-seed\"".to_string(),
                        value: UnifiedSemanticValue::String("branch-seed".to_string()),
                    },
                }],
                vec![],
            ],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9295);
        let values = generator
            .generate_many(1, Some("start"))
            .expect("branch literalish hint generation should succeed");
        assert_eq!(values[0], "branch-seed");

        let group = generator
            .coverage_metrics()
            .branch_groups
            .get("start::root")
            .expect("branch coverage group should exist");
        assert!(
            group.success_counts.first().copied().unwrap_or(0) > 0,
            "branch literal hint should register branch success, got {:?}",
            group.success_counts
        );
    }

    #[test]
    fn stimuli_generation_prunes_or_branches_that_reference_missing_rules() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Sequence {
                elements: vec![token("rule_reference", "wrapper")],
            },
        );
        grammar_tree.insert(
            "wrapper".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("rule_reference", "available"),
                    token("rule_reference", "missing_profile_branch"),
                ],
            },
        );
        grammar_tree.insert("available".to_string(), token("quoted_string", "ok"));
        let rule_order = vec![
            "start".to_string(),
            "wrapper".to_string(),
            "available".to_string(),
        ];

        let mut generator = simple_generator(&grammar_tree, &rule_order, 92955);
        let values = generator
            .generate_many(1, Some("start"))
            .expect("generation should prune missing-rule branches and succeed");
        assert_eq!(values[0], "ok");

        let group = generator
            .coverage_metrics()
            .branch_groups
            .get("wrapper::root")
            .expect("branch coverage group should exist");
        assert_eq!(
            group.selected_counts.get(1).copied().unwrap_or(0),
            0,
            "branch referencing a missing rule should be pruned instead of selected"
        );
        assert!(
            group.success_counts.first().copied().unwrap_or(0) > 0,
            "reachable branch should still succeed after pruning missing-rule alternatives"
        );
    }

    #[test]
    fn semantic_usage_stimuli_literalish_directives_only_override_active_entry_rule() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Sequence {
                elements: vec![token("rule_reference", "helper")],
            },
        );
        grammar_tree.insert(
            "helper".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    token("quoted_string", "alpha"),
                    token("quoted_string", "beta"),
                ],
            },
        );
        let rule_order = vec!["start".to_string(), "helper".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "helper".to_string(),
            vec![SemanticAnnotation::Named {
                name: "probe_sample".to_string(),
                ast: UnifiedSemanticAST::Structured {
                    canonical: "\"seeded-helper\"".to_string(),
                    value: UnifiedSemanticValue::String("seeded-helper".to_string()),
                },
            }],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9296);
        let nested_values = generator
            .generate_many(1, Some("start"))
            .expect("nested generation should succeed");
        assert_eq!(
            nested_values[0], "alphabeta",
            "literalish hints should not short-circuit nested non-entry rules during ordinary entry generation"
        );

        let helper_values = generator
            .generate_many(1, Some("helper"))
            .expect("direct helper generation should succeed");
        assert_eq!(
            helper_values[0], "seeded-helper",
            "literalish hints should still override when the hinted rule is the active generation entry"
        );
    }

    #[test]
    fn semantic_usage_stimuli_rule_level_sample_overrides_or_entry_rule() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "helper".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("quoted_string", "alpha"),
                    token("quoted_string", "beta"),
                ],
            },
        );
        let rule_order = vec!["helper".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "helper".to_string(),
            vec![SemanticAnnotation::Named {
                name: "sample".to_string(),
                ast: UnifiedSemanticAST::Structured {
                    canonical: "\"seeded-helper\"".to_string(),
                    value: UnifiedSemanticValue::String("seeded-helper".to_string()),
                },
            }],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 92961);
        let helper_values = generator
            .generate_many(1, Some("helper"))
            .expect("direct helper generation should succeed");
        assert_eq!(
            helper_values[0], "seeded-helper",
            "rule-level samples should override OR-root entry rules too"
        );
    }

    #[test]
    fn semantic_usage_stimuli_rule_level_probe_sample_overrides_or_only_for_active_entry() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Sequence {
                elements: vec![token("rule_reference", "helper")],
            },
        );
        grammar_tree.insert(
            "helper".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("quoted_string", "alpha"),
                    token("quoted_string", "beta"),
                ],
            },
        );
        let rule_order = vec!["start".to_string(), "helper".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "helper".to_string(),
            vec![SemanticAnnotation::Named {
                name: "probe_sample".to_string(),
                ast: UnifiedSemanticAST::Structured {
                    canonical: "\"seeded-helper\"".to_string(),
                    value: UnifiedSemanticValue::String("seeded-helper".to_string()),
                },
            }],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 92962);
        let nested_values = generator
            .generate_many(12, Some("start"))
            .expect("nested generation should succeed");
        assert!(
            nested_values
                .iter()
                .all(|value| value == "alpha" || value == "beta"),
            "probe samples should not short-circuit non-entry OR rules during ordinary generation: {:?}",
            nested_values
        );

        let helper_values = generator
            .generate_many(1, Some("helper"))
            .expect("direct helper generation should succeed");
        assert_eq!(
            helper_values[0], "seeded-helper",
            "probe samples should override OR-root rules when they are the active generation entry"
        );
    }

    #[test]
    fn semantic_usage_stimuli_nested_regular_literalish_hints_remain_active() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Sequence {
                elements: vec![token("rule_reference", "helper")],
            },
        );
        grammar_tree.insert(
            "helper".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    token("quoted_string", "alpha"),
                    token("quoted_string", "beta"),
                ],
            },
        );
        let rule_order = vec!["start".to_string(), "helper".to_string()];

        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "helper".to_string(),
            vec![SemanticAnnotation::Named {
                name: "sample".to_string(),
                ast: UnifiedSemanticAST::Structured {
                    canonical: "\"seeded-helper\"".to_string(),
                    value: UnifiedSemanticValue::String("seeded-helper".to_string()),
                },
            }],
        );

        let mut generator = annotated_generator(&grammar_tree, &rule_order, &annotations, 9297);
        let values = generator
            .generate_many(1, Some("start"))
            .expect("nested generation should succeed");
        assert_eq!(
            values[0], "seeded-helper",
            "ordinary sample hints should continue to short-circuit nested rules"
        );
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

    // ---- SV-EXH-PROOF.7.2.1: target-reach path computation (pure analysis) ----

    fn rule_ref(rule: &str) -> ASTNode {
        token("rule_reference", rule)
    }

    /// Synthetic grammar (NO SystemVerilog dependency):
    ///   start := mid_a | mid_b
    ///   mid_a := leaf_x leaf_y          (a decoy path, no nested OR target)
    ///   mid_b := ( "kw" deep ) | "lit"  (nested: deep reached via outer o0 then s1)
    ///   deep  := "p" | "q" | "r"        (the TARGET OR node lives here)
    ///   leaf_x, leaf_y, ... terminals
    fn synthetic_reach_grammar() -> HashMap<String, ASTNode> {
        let mut g = HashMap::new();
        g.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![rule_ref("mid_a"), rule_ref("mid_b")],
            },
        );
        g.insert(
            "mid_a".to_string(),
            ASTNode::Sequence {
                elements: vec![rule_ref("leaf_x"), rule_ref("leaf_y")],
            },
        );
        g.insert(
            "mid_b".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    ASTNode::Sequence {
                        elements: vec![token("keyword", "kw"), rule_ref("deep")],
                    },
                    token("string", "lit"),
                ],
            },
        );
        g.insert(
            "deep".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("string", "p"),
                    token("string", "q"),
                    token("string", "r"),
                ],
            },
        );
        g.insert("leaf_x".to_string(), token("string", "x"));
        g.insert("leaf_y".to_string(), token("string", "y"));
        g
    }

    #[test]
    fn reach_path_reaches_nested_or_target_with_forced_chain() {
        let grammar_tree = synthetic_reach_grammar();
        let rule_order: Vec<String> = grammar_tree.keys().cloned().collect();
        let generator = simple_generator(&grammar_tree, &rule_order, 1);

        // Target: branch index 2 ("r") of the OR node at deep::root.
        let plan = generator
            .compute_reach_path("start", "deep", "root", 2)
            .expect("deep must be reachable from start");

        // Expected forced chain (entry -> target order):
        //  - start::root   force o1 (choose mid_b over mid_a)
        //  - mid_b::root   force o0 (choose the "kw deep" sequence over "lit")
        //  - deep::root    force branch 2 (the target itself)
        let expected = vec![
            ReachDirective { rule_name: "start".into(), node_path: "root".into(), branch_index: 1 },
            ReachDirective { rule_name: "mid_b".into(), node_path: "root".into(), branch_index: 0 },
            ReachDirective { rule_name: "deep".into(), node_path: "root".into(), branch_index: 2 },
        ];
        assert_eq!(plan, expected, "reach plan must force the full parent-OR chain + the target branch");
    }

    #[test]
    fn reach_path_for_entry_local_target_is_just_the_target() {
        let grammar_tree = synthetic_reach_grammar();
        let rule_order: Vec<String> = grammar_tree.keys().cloned().collect();
        let generator = simple_generator(&grammar_tree, &rule_order, 1);

        // Target lives in the entry rule itself (start::root, branch 0).
        let plan = generator
            .compute_reach_path("start", "start", "root", 0)
            .expect("entry-local target must resolve");
        assert_eq!(
            plan,
            vec![ReachDirective { rule_name: "start".into(), node_path: "root".into(), branch_index: 0 }],
            "an entry-local target needs only the single target directive"
        );
    }

    #[test]
    fn reach_path_returns_none_for_unreachable_or_bad_index() {
        let grammar_tree = synthetic_reach_grammar();
        let rule_order: Vec<String> = grammar_tree.keys().cloned().collect();
        let generator = simple_generator(&grammar_tree, &rule_order, 1);

        // Branch index out of range for deep (only 0..=2 exist).
        assert!(generator.compute_reach_path("start", "deep", "root", 3).is_none());
        // Unknown target rule.
        assert!(generator.compute_reach_path("start", "nonexistent", "root", 0).is_none());
        // Node path that is not an OR node (mid_a::root is a Sequence).
        assert!(generator.compute_reach_path("start", "mid_a", "root", 0).is_none());
    }

    #[test]
    fn reach_path_is_cycle_safe_and_deterministic() {
        // Self-referential / mutually recursive grammar must terminate.
        //   expr := term ( "+" expr ) | term      (expr references itself)
        //   term := "n"
        let mut g = HashMap::new();
        g.insert(
            "expr".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    ASTNode::Sequence {
                        elements: vec![rule_ref("term"), token("op", "+"), rule_ref("expr")],
                    },
                    rule_ref("term"),
                ],
            },
        );
        g.insert("term".to_string(), token("string", "n"));
        let rule_order: Vec<String> = g.keys().cloned().collect();
        let generator = simple_generator(&g, &rule_order, 1);

        // `term` is a Token (no OR node), so it is not a valid OR target — must
        // be None rather than looping forever on the expr self-reference.
        assert!(
            generator.compute_reach_path("expr", "term", "root", 0).is_none(),
            "a non-OR target must return None, and the self-recursive expr must not hang"
        );

        // Targeting expr's own OR node terminates (BFS enqueues expr once) and is
        // deterministic across repeated calls (no RNG / time dependence).
        let a = generator.compute_reach_path("expr", "expr", "root", 0);
        let b = generator.compute_reach_path("expr", "expr", "root", 0);
        assert_eq!(a, b, "same inputs must yield identical plans (no RNG/time dependence)");
        assert_eq!(
            a,
            Some(vec![ReachDirective {
                rule_name: "expr".into(),
                node_path: "root".into(),
                branch_index: 0,
            }]),
            "entry-local OR target needs only the single target directive"
        );
    }

    // ---- SV-EXH-PROOF.7.2.2: reach-plan steers the OR-decision ----

    #[test]
    fn reach_plan_forces_target_branch_selection() {
        // start := mid_a | mid_b ; mid_b := ("kw" deep) | "lit" ; deep := "p"|"q"|"r"
        // Target: deep::root branch 2 ("r"). Without a plan, the weighted choice
        // may never take start->mid_b->(kw deep)->r. With a plan installed it MUST.
        let grammar_tree = synthetic_reach_grammar();
        let rule_order: Vec<String> = grammar_tree.keys().cloned().collect();
        let mut generator = simple_generator(&grammar_tree, &rule_order, 7);

        let installed = generator.set_reach_plan("start", "deep", "root", 2, 16);
        assert!(installed, "reach plan for deep::root#2 must install");

        let out = generator
            .generate_from_entry("start")
            .expect("generation under a reach plan must succeed");

        // The target branch's selected count is now >= 1 (0 -> >=1).
        assert!(
            generator.branch_selected_hits("deep::root", 2) >= 1,
            "reach plan must drive the target branch to be selected; got selected_hits=0, sample={:?}",
            out
        );
        // And the forced parent choices were taken: start chose mid_b (o1), so the
        // decoy mid_a branch (o0) was NOT selected this run.
        assert_eq!(
            generator.branch_selected_hits("start::root", 0),
            0,
            "reach plan must steer start away from the decoy mid_a branch"
        );
        assert!(
            generator.branch_selected_hits("start::root", 1) >= 1,
            "reach plan must steer start into mid_b"
        );
        // Output is the forced path "kw" + "r".
        assert_eq!(out, "kwr", "forced reach path should emit kw + r, got {:?}", out);
    }

    #[test]
    fn reach_plan_none_leaves_or_decision_unchanged() {
        // With NO reach plan installed, generation is identical to baseline: the
        // same seed produces the same output with and without the (absent) plan
        // machinery. This guards the "zero behavior change on the production
        // path" invariant.
        let grammar_tree = synthetic_reach_grammar();
        let rule_order: Vec<String> = grammar_tree.keys().cloned().collect();

        let mut a = simple_generator(&grammar_tree, &rule_order, 12345);
        let mut b = simple_generator(&grammar_tree, &rule_order, 12345);
        // b briefly installs then clears a plan — clearing must restore baseline.
        assert!(b.set_reach_plan("start", "deep", "root", 1, 8));
        b.clear_reach_plan();

        let sa = a.generate_from_entry("start").expect("a generates");
        let sb = b.generate_from_entry("start").expect("b generates");
        assert_eq!(
            sa, sb,
            "clearing a reach plan must restore byte-identical baseline generation"
        );
    }

    #[test]
    fn reach_plan_install_fails_for_invalid_target() {
        let grammar_tree = synthetic_reach_grammar();
        let rule_order: Vec<String> = grammar_tree.keys().cloned().collect();
        let mut generator = simple_generator(&grammar_tree, &rule_order, 1);
        // Out-of-range branch index, unknown rule, and non-OR node all fail to
        // install (and leave reach_plan = None).
        assert!(!generator.set_reach_plan("start", "deep", "root", 99, 8));
        assert!(!generator.set_reach_plan("start", "nope", "root", 0, 8));
        assert!(!generator.set_reach_plan("start", "mid_a", "root", 0, 8));
        // A subsequent valid generation is unaffected (plan stayed None).
        assert!(generator.generate_from_entry("start").is_ok());
    }

    // ---- SV-EXH-PROOF.7.2.3: honest reach-outcome reporting ----

    #[test]
    fn reach_outcome_reached_for_satisfiable_target() {
        let grammar_tree = synthetic_reach_grammar();
        let rule_order: Vec<String> = grammar_tree.keys().cloned().collect();
        let mut generator = simple_generator(&grammar_tree, &rule_order, 7);
        assert!(generator.set_reach_plan("start", "deep", "root", 2, 16));
        let _ = generator.generate_from_entry("start").expect("generates");
        assert_eq!(
            generator.reach_target_outcome(),
            Some(ReachOutcome::Reached),
            "a satisfiable target that produced a valid sample must report Reached"
        );
    }

    #[test]
    fn reach_outcome_selected_but_failed_for_unsatisfiable_target() {
        // deep's branch 2 references a MISSING rule, so when the plan forces that
        // branch it is SELECTED but its subtree generation fails -> the plan must
        // honestly report SelectedButFailed (driver should report, not retry).
        let mut grammar_tree = synthetic_reach_grammar();
        grammar_tree.insert(
            "deep".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    token("string", "p"),
                    token("string", "q"),
                    rule_ref("does_not_exist"), // branch 2: unsatisfiable
                ],
            },
        );
        let rule_order: Vec<String> = grammar_tree.keys().cloned().collect();
        let mut generator = simple_generator(&grammar_tree, &rule_order, 7);
        assert!(generator.set_reach_plan("start", "deep", "root", 2, 16));
        // Generation overall may still succeed via fallback branches; the point is
        // the TARGET branch (2) was selected but could not be satisfied.
        let _ = generator.generate_from_entry("start");
        assert_eq!(
            generator.reach_target_outcome(),
            Some(ReachOutcome::SelectedButFailed),
            "a target branch whose subtree cannot be satisfied must report SelectedButFailed, not Reached"
        );
    }

    #[test]
    fn reach_outcome_none_without_plan_and_not_reached_is_distinct() {
        let grammar_tree = synthetic_reach_grammar();
        let rule_order: Vec<String> = grammar_tree.keys().cloned().collect();
        let mut generator = simple_generator(&grammar_tree, &rule_order, 7);
        // No plan installed -> None.
        assert_eq!(generator.reach_target_outcome(), None);
        // Install a plan but DON'T generate: the target was never selected, so the
        // honest outcome is NotReached (distinct from Reached/SelectedButFailed).
        assert!(generator.set_reach_plan("start", "deep", "root", 1, 16));
        assert_eq!(
            generator.reach_target_outcome(),
            Some(ReachOutcome::NotReached),
            "a plan installed but not yet exercised must report NotReached"
        );
    }

    // ---- SV-EXH-PROOF.7.2.4: the driver resolves a nested branch target ----

    #[test]
    fn target_driver_resolves_nested_branch_via_reach_steering() {
        // A nested branch target (deep::root#2 = "r") reachable only through the
        // start->mid_b->(kw deep) chain. The driver (generate_until_targets) must
        // retire it within the attempt budget — the .7.2.4 reach hook steers to it
        // when the baseline weighting stagnates.
        let grammar_tree = synthetic_reach_grammar();
        let rule_order: Vec<String> = grammar_tree.keys().cloned().collect();
        let mut generator = simple_generator(&grammar_tree, &rule_order, 4242);

        let report = generator
            .generate_gap_report(Some("start"), 1)
            .expect("gap report should generate");
        // The nested deep branch targets must be present as actionable items.
        assert!(
            report
                .targets
                .iter()
                .any(|t| t.rule_name == "deep" && t.branch_index == Some(2)),
            "expected deep::root#2 to be an actionable branch target; got {:?}",
            report.targets.iter().map(|t| &t.id).collect::<Vec<_>>()
        );

        let (_samples, summary) = generator
            .generate_until_targets(Some("start"), &report.targets, 400)
            .expect("target-driven generation should succeed");

        // Every reachable target retired (the reach hook closes the nested ones
        // the plain weighting would otherwise leave never_selected).
        assert!(
            summary.unresolved_targets.is_empty(),
            "driver must retire all reachable targets incl. nested deep branches; unresolved={:?}",
            summary
                .unresolved_targets
                .iter()
                .map(|s| &s.id)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            summary.resolved_targets, summary.total_targets,
            "resolved should equal total"
        );
    }

    #[test]
    fn target_driver_hook_installs_plan_for_a_branch_status() {
        // Direct unit test of the .7.2.4 hook helper: given a pending BRANCH
        // status (rule + node_path + branch_index), it installs a reach plan that
        // steers to that branch; given a RULE status it declines (returns 0). This
        // is the deterministic discriminant for the hook itself — manufacturing a
        // 32-iteration stagnation end-to-end is fragile (the baseline ×24 boost
        // resolves small grammars in a few attempts), so the hook's CORRECTNESS is
        // proven here and its end-to-end no-regression by
        // target_driver_resolves_nested_branch_via_reach_steering.
        let grammar_tree = synthetic_reach_grammar();
        let rule_order: Vec<String> = grammar_tree.keys().cloned().collect();
        let mut generator = simple_generator(&grammar_tree, &rule_order, 9);

        // A BRANCH status for deep::root#2 → plan installs (forced-directive
        // count > 0) and steers generation to "kwr".
        let branch_status = TargetCoverageStatus {
            id: "branch::deep::root#2".to_string(),
            target_type: StimuliCoverageTargetType::Branch,
            rule_name: "deep".to_string(),
            node_path: Some("root".to_string()),
            branch_index: Some(2),
            current_successes: 0,
            required_successes: 1,
            remaining_successes: 1,
            priority_score: 1,
            reason: "never_selected".to_string(),
            depends_on: Vec::new(),
        };
        let installed = generator.try_install_reach_plan_for_status("start", &branch_status, 16);
        assert!(installed > 0, "hook must install a plan for a branch status");
        let out = generator.generate_from_entry("start").expect("generates");
        assert_eq!(out, "kwr", "installed plan must steer to the target branch");
        generator.clear_reach_plan();

        // A RULE status → hook declines (no node_path/branch to force).
        let rule_status = TargetCoverageStatus {
            id: "rule::deep".to_string(),
            target_type: StimuliCoverageTargetType::Rule,
            rule_name: "deep".to_string(),
            node_path: None,
            branch_index: None,
            current_successes: 0,
            required_successes: 1,
            remaining_successes: 1,
            priority_score: 1,
            reason: "never_selected".to_string(),
            depends_on: Vec::new(),
        };
        assert_eq!(
            generator.try_install_reach_plan_for_status("start", &rule_status, 16),
            0,
            "hook must decline a non-branch (rule) status"
        );
    }

    // ---- SV-EXH-PROOF.7.2.7: the reach hook FIRES under deep stagnation ----

    #[test]
    fn reach_hook_fires_under_deep_stagnation() {
        // Regression guard for the .7.2.4/.7.2.6 dead-guard flaw (measured
        // reach_plan_activations=0). Build a target the baseline weighting keeps
        // FAILING (selected-but-fails), which drives probe_threshold down to 8 and
        // lets stagnation climb past reach_threshold (=2*8+8=24); the .7.2.7
        // deeper-fallback hook must then engage (activations>0).
        //   start := mid | "x"
        //   mid   := "p" | bad        (target = mid::root#1, but `bad` is
        //   bad   := does_not_exist    unsatisfiable → branch selected-but-fails)
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "start".to_string(),
            ASTNode::Or {
                alternatives: vec![rule_ref("mid"), token("string", "x")],
            },
        );
        grammar_tree.insert(
            "mid".to_string(),
            ASTNode::Or {
                alternatives: vec![token("string", "p"), rule_ref("bad")],
            },
        );
        grammar_tree.insert("bad".to_string(), rule_ref("does_not_exist"));
        let rule_order: Vec<String> = grammar_tree.keys().cloned().collect();
        let mut generator = simple_generator(&grammar_tree, &rule_order, 7);

        // Hand-built target on the hard branch mid::root#1, requiring a success it
        // can never get (bad → missing rule), so the run stays pending + stagnates.
        let targets = vec![StimuliCoverageTarget {
            id: "branch::mid::root#1".to_string(),
            target_type: StimuliCoverageTargetType::Branch,
            rule_name: "mid".to_string(),
            node_path: Some("root".to_string()),
            branch_index: Some(1),
            reachable: true,
            required_successes: 1,
            current_successes: 0,
            deficit: 1,
            priority_score: 100,
            reason: "never_selected".to_string(),
            depends_on: Vec::new(),
            reach_classification: None,
        }];

        let (_samples, summary) = generator
            .generate_until_targets(Some("start"), &targets, 400)
            .expect("driver should run");

        // The hook engaged at least once under the sustained stagnation. (The
        // target itself stays unsatisfiable — that is .7.2.3's SelectedButFailed
        // territory — but .7.2.7's contract here is simply that the hook FIRES,
        // unlike the .7.2.4 dead guard which gave activations=0.)
        assert!(
            generator.reach_plan_activations() >= 1,
            "reach hook must fire under deep stagnation (deeper-fallback .7.2.7); activations={} attempts={}",
            generator.reach_plan_activations(),
            summary.attempts
        );
    }

    // ---- SV-EXH-PROOF.7.2.16: read-only reach classification in the gap report ----

    #[test]
    fn gap_report_classifies_residual_targets_read_only() {
        // synthetic_reach_grammar: start := mid_a | mid_b ; mid_b := ("kw" deep) | "lit";
        // deep := "p"|"q"|"r". Every branch target here IS reachable by a plan, so
        // its classification must be "reachable_by_plan"; every RULE target must be
        // "reachable_rule_not_generated". (A genuinely-unreachable branch can't arise
        // in this fully-connected grammar; the no_reach_path arm is covered by the
        // compute_reach_path None-returning unit tests in .7.2.1.)
        let grammar_tree = synthetic_reach_grammar();
        let rule_order: Vec<String> = grammar_tree.keys().cloned().collect();
        let mut generator = simple_generator(&grammar_tree, &rule_order, 1);

        let report = generator
            .generate_gap_report(Some("start"), 1)
            .expect("gap report should generate");
        assert!(!report.targets.is_empty(), "expected residual targets to classify");

        // FACT 1: every target is classified (no None left after the pass).
        assert!(
            report.targets.iter().all(|t| t.reach_classification.is_some()),
            "every residual target must carry a reach_classification"
        );

        // FACT 2: each class is the right one for its target type, and branch
        // targets in this fully-connected grammar are reachable_by_plan.
        for t in &report.targets {
            match t.target_type {
                StimuliCoverageTargetType::Rule => assert_eq!(
                    t.reach_classification.as_deref(),
                    Some("reachable_rule_not_generated"),
                    "rule target {} misclassified: {:?}",
                    t.id,
                    t.reach_classification
                ),
                StimuliCoverageTargetType::Branch => assert_eq!(
                    t.reach_classification.as_deref(),
                    Some("reachable_by_plan"),
                    "branch target {} (reachable in this grammar) misclassified: {:?}",
                    t.id,
                    t.reach_classification
                ),
            }
        }
    }

    #[test]
    fn gap_report_classification_does_not_change_generation() {
        // PROOF the classification pass is READ-ONLY: the gap report's target SET
        // (ids + reasons + counts) is identical whether or not we read the new
        // field, and re-generating the report twice is stable. (Generation code is
        // untouched; this guards against the pass accidentally mutating coverage.)
        let grammar_tree = synthetic_reach_grammar();
        let rule_order: Vec<String> = grammar_tree.keys().cloned().collect();
        let mut g1 = simple_generator(&grammar_tree, &rule_order, 5);
        let r1 = g1.generate_gap_report(Some("start"), 1).expect("r1");
        let mut g2 = simple_generator(&grammar_tree, &rule_order, 5);
        let r2 = g2.generate_gap_report(Some("start"), 1).expect("r2");
        let ids1: Vec<&str> = r1.targets.iter().map(|t| t.id.as_str()).collect();
        let ids2: Vec<&str> = r2.targets.iter().map(|t| t.id.as_str()).collect();
        assert_eq!(ids1, ids2, "classification must not change the target set/order");
        assert_eq!(
            r1.summary.total_branches, r2.summary.total_branches,
            "classification must not change coverage summary"
        );
    }

    // ---- SV-EXH-PROOF.7.2.19: rule -> branch reach translation (analysis only) ----

    #[test]
    fn compute_rule_reach_target_finds_or_branch_introducing_a_rule() {
        // synthetic_reach_grammar: start := mid_a | mid_b ; mid_b := ("kw" deep) | "lit"
        // deep := "p"|"q"|"r". `deep` is reachable ONLY via start->o1 (mid_b) then
        // mid_b->o0 ("kw" deep). The DEEPEST OR introducing `deep` is mid_b::root#0.
        let grammar_tree = synthetic_reach_grammar();
        let rule_order: Vec<String> = grammar_tree.keys().cloned().collect();
        let generator = simple_generator(&grammar_tree, &rule_order, 1);

        let tgt = generator
            .compute_rule_reach_target("start", "deep")
            .expect("deep is reachable behind ORs, so a rule-reach branch target must exist");
        // `deep` is introduced inside mid_b at root/s0 (the "kw deep" sequence) which
        // sits under mid_b::root#0. The deepest OR on that site path is mid_b::root#0.
        assert_eq!(
            tgt,
            ("mid_b".to_string(), "root".to_string(), 0),
            "rule-reach target for `deep` must be the OR branch mid_b::root#0 that introduces it; got {:?}",
            tgt
        );

        // And the translated branch target must yield a valid reach plan via the
        // UNCHANGED compute_reach_path (entry -> that branch).
        let (via_rule, node_path, branch_index) = tgt;
        let plan = generator
            .compute_reach_path("start", &via_rule, &node_path, branch_index)
            .expect("the translated branch target must be reachable by a plan");
        assert!(!plan.is_empty(), "reach plan to the introducing branch must be non-empty");
    }

    #[test]
    fn compute_rule_reach_target_none_for_unconditional_or_trivial_cases() {
        // A rule referenced UNCONDITIONALLY (no OR on the path) needs no branch
        // forcing: start := pre body ; body := "x". `body` is always generated when
        // start is, so there is no branch target -> None.
        let mut g = HashMap::new();
        g.insert(
            "start".to_string(),
            ASTNode::Sequence {
                elements: vec![rule_ref("pre"), rule_ref("body")],
            },
        );
        g.insert("pre".to_string(), token("string", "P"));
        g.insert("body".to_string(), token("string", "x"));
        let rule_order: Vec<String> = g.keys().cloned().collect();
        let generator = simple_generator(&g, &rule_order, 1);

        assert_eq!(
            generator.compute_rule_reach_target("start", "body"),
            None,
            "an unconditionally-referenced rule has no OR to force -> None"
        );
        // entry itself, and an unknown rule, are also None.
        assert_eq!(generator.compute_rule_reach_target("start", "start"), None);
        assert_eq!(generator.compute_rule_reach_target("start", "does_not_exist"), None);
    }

    // ---- SV-EXH-PROOF.7.4.2: Purdom phase-1 shortest-derivation table ----

    #[test]
    fn min_terminal_lengths_on_synthetic_grammar() {
        // synthetic_reach_grammar:
        //   start := mid_a | mid_b           -> min(2, 1) = 1
        //   mid_a := leaf_x leaf_y           -> 1 + 1     = 2
        //   mid_b := ("kw" deep) | "lit"     -> min(1+1, 1) = 1
        //   deep  := "p" | "q" | "r"         -> min(1,1,1) = 1
        //   leaf_x := "x"                    -> 1
        //   leaf_y := "y"                    -> 1
        let grammar_tree = synthetic_reach_grammar();
        let rule_order: Vec<String> = grammar_tree.keys().cloned().collect();
        let generator = simple_generator(&grammar_tree, &rule_order, 1);

        let table = generator.compute_min_terminal_lengths();
        assert_eq!(table.get("leaf_x"), Some(&1));
        assert_eq!(table.get("leaf_y"), Some(&1));
        assert_eq!(table.get("deep"), Some(&1), "deep is min over 3 single-token alts");
        assert_eq!(table.get("mid_a"), Some(&2), "mid_a is a 2-element sequence");
        assert_eq!(
            table.get("mid_b"),
            Some(&1),
            "mid_b's shortest alt is the bare \"lit\" terminal (1), not the kw+deep seq (2)"
        );
        assert_eq!(
            table.get("start"),
            Some(&1),
            "start's shortest derivation goes via mid_b (1), not mid_a (2)"
        );
    }

    #[test]
    fn k_path_universe_on_synthetic_grammar() {
        // STIMULI-SIGNOFF.2: k-path coverage universe (Havrikov & Zeller, ASE 2019).
        // synthetic_reach_grammar rule-reference graph (6 rules):
        //   start  -> {mid_a, mid_b}
        //   mid_a  -> {leaf_x, leaf_y}
        //   mid_b  -> {deep}
        //   deep / leaf_x / leaf_y -> {}  (terminals only)
        let grammar_tree = synthetic_reach_grammar();
        let rule_order: Vec<String> = grammar_tree.keys().cloned().collect();
        let g = simple_generator(&grammar_tree, &rule_order, 1);

        assert_eq!(g.compute_k_paths(0).len(), 0, "k=0 is empty");
        assert_eq!(g.compute_k_paths(1).len(), 6, "k=1 = every rule (6)");
        // k=2 edges: start->mid_a, start->mid_b, mid_a->leaf_x, mid_a->leaf_y, mid_b->deep
        assert_eq!(g.compute_k_paths(2).len(), 5, "k=2 = reference edges (5)");
        // k=3 chains: [start,mid_a,leaf_x], [start,mid_a,leaf_y], [start,mid_b,deep]
        let k3 = g.compute_k_paths(3);
        assert_eq!(k3.len(), 3, "k=3 = depth-3 chains (3); got {:?}", k3);
        assert!(
            k3.contains(&vec![
                "start".to_string(),
                "mid_b".to_string(),
                "deep".to_string()
            ]),
            "the deep-context k-path start->mid_b->deep must be in the k=3 universe (the kind \
             of context combination rule+branch coverage does NOT measure); got {:?}",
            k3
        );
        assert_eq!(
            g.compute_k_paths(4).len(),
            0,
            "no length-4 chain exists (deepest is start->mid_b->deep, length 3)"
        );
        // Deterministic given a fixed rule_order (two calls identical within the run).
        assert_eq!(
            g.compute_k_paths(3),
            g.compute_k_paths(3),
            "k-path enumeration must be deterministic"
        );
    }

    #[test]
    fn k_path_numerator_records_covered_paths_within_universe() {
        // STIMULI-SIGNOFF.2.2: covered k-paths (numerator) recorded from generation, all
        // within the compute_k_paths universe (denominator). Default = OFF (monotone).
        let grammar_tree = synthetic_reach_grammar();
        let rule_order: Vec<String> = grammar_tree.keys().cloned().collect();
        let mut g = simple_generator(&grammar_tree, &rule_order, 10);

        // Default: recording OFF (byte-identical generation; the monotone safety property).
        assert!(
            g.covered_k_paths().is_none(),
            "k-path recording must be OFF by default"
        );
        assert!(g.k_path_coverage().is_none());

        // Enable k=2 recording, generate from the entry several times.
        g.enable_k_path_recording(2);
        for _ in 0..20 {
            let _ = g.generate_from_entry("start");
        }

        let universe: HashSet<Vec<String>> = g.compute_k_paths(2).into_iter().collect();
        let covered = g.covered_k_paths().expect("recording is on").clone();
        assert!(
            !covered.is_empty(),
            "generation from `start` must cover at least one k-path"
        );
        for p in &covered {
            assert!(
                universe.contains(p),
                "every covered k-path must be in the universe; {:?} was not",
                p
            );
        }
        let (hit, total) = g.k_path_coverage().expect("recording is on");
        assert_eq!(total, 5, "k=2 universe is 5 reference edges");
        assert!((1..=5).contains(&hit), "covered-in-universe count in [1,5]; got {hit}");
        // `start` always expands to mid_a or mid_b, so a start-> edge is always covered.
        assert!(
            covered.contains(&vec!["start".to_string(), "mid_a".to_string()])
                || covered.contains(&vec!["start".to_string(), "mid_b".to_string()]),
            "a `start`-> reference edge must be covered; got {covered:?}"
        );
    }

    #[test]
    fn min_terminal_lengths_fixpoint_resolves_recursion_drops_nonterminating() {
        // expr := expr "+" term | term ; term := "n"   (well-formed, has a base case)
        //   term -> 1 ; expr resolves via the non-recursive alt `term` -> 1.
        // bad  := bad "z"                               (NO base case -> never resolves)
        let mut g = HashMap::new();
        g.insert(
            "expr".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    ASTNode::Sequence {
                        elements: vec![rule_ref("expr"), token("string", "+"), rule_ref("term")],
                    },
                    rule_ref("term"),
                ],
            },
        );
        g.insert("term".to_string(), token("string", "n"));
        g.insert(
            "bad".to_string(),
            ASTNode::Sequence {
                elements: vec![rule_ref("bad"), token("string", "z")],
            },
        );
        let rule_order: Vec<String> = g.keys().cloned().collect();
        let generator = simple_generator(&g, &rule_order, 1);

        let table = generator.compute_min_terminal_lengths();
        assert_eq!(table.get("term"), Some(&1));
        assert_eq!(
            table.get("expr"),
            Some(&1),
            "left-recursive expr must resolve via its non-recursive `term` alternative"
        );
        assert_eq!(
            table.get("bad"),
            None,
            "a rule with no terminating base case has no finite min length -> absent"
        );
    }

    #[test]
    fn min_terminal_lengths_quantifier_and_lookahead_semantics() {
        // opt := "a"?   -> 0 (may match empty)
        // star := "a"*  -> 0
        // plus := "a"+  -> 1 (min-count 1 x 1)
        // exact := "a"{3} -> 3
        // look := &"a" "b" -> 0 (lookahead) + 1 (terminal) = 1
        let q = |quant: &str| ASTNode::Quantified {
            element: Box::new(token("string", "a")),
            quantifier: quant.to_string(),
        };
        let mut g = HashMap::new();
        g.insert("opt".to_string(), q("?"));
        g.insert("star".to_string(), q("*"));
        g.insert("plus".to_string(), q("+"));
        g.insert("exact".to_string(), q("{3}"));
        g.insert(
            "look".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    ASTNode::Lookahead { element: Box::new(token("string", "a")), positive: true },
                    token("string", "b"),
                ],
            },
        );
        let rule_order: Vec<String> = g.keys().cloned().collect();
        let generator = simple_generator(&g, &rule_order, 1);

        let table = generator.compute_min_terminal_lengths();
        assert_eq!(table.get("opt"), Some(&0), "`?` can match empty -> 0");
        assert_eq!(table.get("star"), Some(&0), "`*` can match empty -> 0");
        assert_eq!(table.get("plus"), Some(&1), "`+` is min-count 1 x body(1) = 1");
        assert_eq!(table.get("exact"), Some(&3), "`{{3}}` is min-count 3 x body(1) = 3");
        assert_eq!(
            table.get("look"),
            Some(&1),
            "a positive lookahead consumes no input (0) + the following terminal (1)"
        );
    }
}
