use super::{ASTNode, ASTValue, Annotations, TokenValue, UnifiedSemanticAST};
use anyhow::{Context, Result, anyhow};
use rand::distributions::{Distribution, WeightedIndex};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use regex_syntax::hir::{Class, Hir, HirKind, Literal, Repetition};
use std::collections::HashMap;

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

pub struct StimuliGenerator<'a> {
    grammar_name: String,
    grammar_tree: &'a HashMap<String, ASTNode>,
    rule_order: &'a [String],
    annotations: Option<&'a Annotations>,
    config: StimuliConfig,
    rng: StdRng,
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

        Self {
            grammar_name,
            grammar_tree,
            rule_order,
            annotations,
            config,
            rng,
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
        self.generate_rule(entry_rule, 0, &mut call_stack)
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
        let result = self.generate_node(rule_node, rule_name, depth + 1, call_stack);
        call_stack.pop();
        result
    }

    fn generate_node(
        &mut self,
        node: &ASTNode,
        current_rule: &str,
        depth: usize,
        call_stack: &mut Vec<String>,
    ) -> Result<String> {
        match node {
            ASTNode::Or { alternatives } => {
                self.generate_or(alternatives, current_rule, depth, call_stack)
            }
            ASTNode::Sequence { elements } => {
                self.generate_sequence(elements, current_rule, depth, call_stack)
            }
            ASTNode::Atom { value } => self.generate_atom(value, current_rule, depth, call_stack),
            ASTNode::Quantified {
                element,
                quantifier,
            } => self.generate_quantified(element, quantifier, current_rule, depth, call_stack),
        }
    }

    fn generate_or(
        &mut self,
        alternatives: &[ASTNode],
        current_rule: &str,
        depth: usize,
        call_stack: &mut Vec<String>,
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
        let weights = self.build_weights(&probabilities)?;

        let dist = WeightedIndex::new(&weights).with_context(|| {
            format!(
                "Invalid branch weights for rule '{}': {:?}",
                current_rule, weights
            )
        })?;
        let selected_local = dist.sample(&mut self.rng);
        let selected_global = candidate_indices[selected_local];

        let (_, selected_node) = prepared.swap_remove(selected_global);
        self.generate_node(&selected_node, current_rule, depth, call_stack)
    }

    fn generate_sequence(
        &mut self,
        elements: &[ASTNode],
        current_rule: &str,
        depth: usize,
        call_stack: &mut Vec<String>,
    ) -> Result<String> {
        let mut output = String::new();
        for element in elements {
            output.push_str(&self.generate_node(element, current_rule, depth, call_stack)?);
        }
        Ok(output)
    }

    fn generate_atom(
        &mut self,
        value: &ASTValue,
        current_rule: &str,
        depth: usize,
        call_stack: &mut Vec<String>,
    ) -> Result<String> {
        match value {
            ASTValue::Node(node) => self.generate_node(node, current_rule, depth, call_stack),
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
            output.push_str(&self.generate_node(element, current_rule, depth + 1, call_stack)?);
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

        let unit = self.generate_from_regex_hir(&rep.sub);
        let mut out = String::new();
        for _ in 0..count {
            out.push_str(&unit);
        }
        out
    }

    fn generate_from_regex_class(&mut self, class: &Class) -> String {
        match class {
            Class::Unicode(unicode_class) => unicode_class
                .ranges()
                .first()
                .map(|range| range.start().to_string())
                .unwrap_or_default(),
            Class::Bytes(bytes_class) => bytes_class
                .ranges()
                .first()
                .map(|range| {
                    let b = range.start();
                    if b.is_ascii() {
                        char::from(b).to_string()
                    } else {
                        "a".to_string()
                    }
                })
                .unwrap_or_default(),
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
}
