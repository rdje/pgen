use super::{ASTNode, ASTValue, Annotations, TokenValue, UnifiedSemanticAST};
use anyhow::{Context, Result, anyhow};
use rand::distributions::{Distribution, WeightedIndex};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use regex_syntax::hir::{Class, Hir, HirKind, Literal, Repetition};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct StimuliConfig {
    pub seed: Option<u64>,
    pub max_depth: usize,
    pub max_repeat: usize,
    pub max_rule_visits: usize,
}

impl Default for StimuliConfig {
    fn default() -> Self {
        Self {
            seed: None,
            max_depth: 24,
            max_repeat: 4,
            max_rule_visits: 8,
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
        *self.rule_success_hits.entry(rule_name.to_string()).or_insert(0) += 1;
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

    pub fn covered_rules(&self) -> usize {
        self.rule_success_hits.values().filter(|hits| **hits > 0).count()
    }

    pub fn covered_branches(&self) -> usize {
        self.branch_groups
            .values()
            .map(|group| group.success_counts.iter().filter(|hits| **hits > 0).count())
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

pub struct StimuliGenerator<'a> {
    grammar_name: String,
    grammar_tree: &'a HashMap<String, ASTNode>,
    rule_order: &'a [String],
    annotations: Option<&'a Annotations>,
    config: StimuliConfig,
    rng: StdRng,
    coverage: StimuliCoverageMetrics,
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
        }
    }

    pub fn coverage_metrics(&self) -> &StimuliCoverageMetrics {
        &self.coverage
    }

    pub fn merge_coverage_metrics(&mut self, other: &StimuliCoverageMetrics) -> Result<()> {
        self.coverage.merge_from(other)
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
        let mut outputs = Vec::with_capacity(count);
        for _ in 0..count {
            outputs.push(self.generate_from_entry(&resolved_entry)?);
        }
        Ok(outputs)
    }

    pub fn generate_from_entry(&mut self, entry_rule: &str) -> Result<String> {
        let mut call_stack = Vec::new();
        let result = self.generate_rule(entry_rule, 0, &mut call_stack);
        self.coverage.record_sample_attempt(result.is_ok());
        result
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
        if alternatives.is_empty() {
            return Ok(String::new());
        }

        let mut prepared: Vec<(Option<u32>, ASTNode)> = alternatives
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
                u64::from(base_weights[local_idx]).saturating_mul(multiplier.max(1))
            })
            .collect();

        let dist = WeightedIndex::new(&guided_weights).with_context(|| {
            format!(
                "Invalid branch weights for rule '{}': {:?}",
                current_rule, guided_weights
            )
        })?;
        let selected_local = dist.sample(&mut self.rng);
        let mut attempt_order: Vec<usize> = (0..candidate_indices.len()).collect();
        attempt_order.swap(0, selected_local);
        if attempt_order.len() > 2 {
            attempt_order[1..].shuffle(&mut self.rng);
        }

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
                    return Ok(output);
                }
                Err(err) => {
                    last_error = Some(err);
                }
            }
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
        let mut output = String::new();
        for (idx, element) in elements.iter().enumerate() {
            let element_path = format!("{}/s{}", node_path, idx);
            output.push_str(&self.generate_node(
                element,
                current_rule,
                depth,
                call_stack,
                &element_path,
            )?);
        }
        Ok(output)
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

                match token_type {
                    "quoted_string" => Ok(token_value.to_string()),
                    "rule_reference" => self.generate_rule(token_value, depth + 1, call_stack),
                    "regex" => Ok(self.generate_regex_sample(token_value, current_rule)),
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
        let repeats = if depth >= self.config.max_depth.saturating_sub(1) {
            min_repeat
        } else if min_repeat == bounded_max {
            min_repeat
        } else {
            self.rng.gen_range(min_repeat..=bounded_max)
        };

        let mut output = String::new();
        for _ in 0..repeats {
            let quantified_path = format!("{}/q", node_path);
            output.push_str(&self.generate_node(
                element,
                current_rule,
                depth + 1,
                call_stack,
                &quantified_path,
            )?);
        }
        Ok(output)
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
            multiplier =
                multiplier.saturating_mul(1 + u64::try_from(uncovered_rule_refs.min(4)).unwrap_or(1));
        }

        multiplier
    }

    fn count_uncovered_rule_references(&self, node: &ASTNode) -> usize {
        let mut names = HashSet::new();
        self.collect_uncovered_rule_references(node, &mut names);
        names.len()
    }

    fn collect_uncovered_rule_references(&self, node: &ASTNode, names: &mut HashSet<String>) {
        match node {
            ASTNode::Or { alternatives } => {
                for alternative in alternatives {
                    self.collect_uncovered_rule_references(alternative, names);
                }
            }
            ASTNode::Sequence { elements } => {
                for element in elements {
                    self.collect_uncovered_rule_references(element, names);
                }
            }
            ASTNode::Quantified { element, .. } => {
                self.collect_uncovered_rule_references(element, names);
            }
            ASTNode::Atom { value } => match value {
                ASTValue::Node(node) => self.collect_uncovered_rule_references(node, names),
                ASTValue::Token(parts) => {
                    if let Some((token_type, token_value)) = Self::extract_token_pair(parts) {
                        if token_type == "rule_reference"
                            && self
                                .coverage
                                .rule_success_hits
                                .get(token_value)
                                .copied()
                                .unwrap_or(0)
                                == 0
                        {
                            names.insert(token_value.to_string());
                        }
                    }
                }
            },
        }
    }

    fn generate_regex_sample(&mut self, pattern: &str, current_rule: &str) -> String {
        if let Some(semantic_hint) = self.semantic_hint_for_rule(current_rule) {
            return semantic_hint;
        }

        let trimmed = pattern.trim();
        if trimmed.is_empty() {
            return String::new();
        }

        match regex_syntax::parse(trimmed) {
            Ok(hir) => self.generate_from_regex_hir(&hir),
            Err(_) => "x".to_string(),
        }
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
        let semantic_asts = annotations.semantic_annotations.get(rule_name)?;

        for ast in semantic_asts {
            match ast {
                UnifiedSemanticAST::TransformExpr { expression } => {
                    let lower = expression.to_lowercase();
                    if lower.contains("parse::<f") {
                        return Some("1.0".to_string());
                    }
                    if lower.contains("parse::<i")
                        || lower.contains("parse::<u")
                        || lower.contains("parse::<isize")
                        || lower.contains("parse::<usize")
                    {
                        return Some("1".to_string());
                    }
                    if lower.contains("parse::<bool") {
                        return Some("true".to_string());
                    }
                }
                UnifiedSemanticAST::Raw { content } => {
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

        assert!(!value[0].is_empty(), "whitespace sample should not be empty");
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

        let re = Regex::new(r"\bword\b").expect("valid regex");
        assert!(
            re.is_match(&value[0]),
            "generated sample must satisfy word-boundary regex: {:?}",
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
}
