use super::{
    ASTNode, ASTValue, Annotations, BranchAnnotation, ExtractionTarget, SemanticAnnotation,
    SemanticAssociativity, UnifiedReturnAST, UnifiedSemanticAST, UnknownSemanticDirectivePolicy,
    extract_semantic_directive, extract_semantic_directive_name, normalize_semantic_scalar,
    parse_canonical_transform_expression, parse_semantic_len_bounds, parse_semantic_numeric_bounds,
    parse_semantic_numeric_list, parse_semantic_string_list, semantic_directive_spec,
};
use regex::Regex;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnnotationSeverity {
    Error,
    Warning,
}

impl AnnotationSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnnotationSeverity::Error => "error",
            AnnotationSeverity::Warning => "warning",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnnotationKind {
    Return,
    Semantic,
    Grammar,
}

impl AnnotationKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnnotationKind::Return => "return",
            AnnotationKind::Semantic => "semantic",
            AnnotationKind::Grammar => "grammar",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotationDiagnostic {
    pub code: &'static str,
    pub severity: AnnotationSeverity,
    pub kind: AnnotationKind,
    pub rule_name: String,
    pub annotation_index: Option<usize>,
    pub message: String,
    pub annotation: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct AnnotationValidationReport {
    pub diagnostics: Vec<AnnotationDiagnostic>,
}

impl AnnotationValidationReport {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == AnnotationSeverity::Error)
    }

    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == AnnotationSeverity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == AnnotationSeverity::Warning)
            .count()
    }
}

#[derive(Debug, Clone, Default)]
pub struct AnnotationValidatorConfig {
    pub max_capture_index: Option<usize>,
    pub strict_semantic_transforms: bool,
    pub unknown_semantic_directive_policy: UnknownSemanticDirectivePolicy,
}

#[derive(Debug, Clone, Default)]
pub struct AnnotationValidator {
    config: AnnotationValidatorConfig,
}

#[derive(Debug, Clone, Default)]
struct FirstSetSummary {
    terminals: HashSet<String>,
    nullable: bool,
    unresolved: bool,
}

impl AnnotationValidator {
    pub fn new(config: AnnotationValidatorConfig) -> Self {
        Self { config }
    }

    pub fn validate_annotations(&self, annotations: &Annotations) -> AnnotationValidationReport {
        let mut report = AnnotationValidationReport::default();

        for (rule_name, branch_annotations) in &annotations.branch_return_annotations {
            for (idx, annotation) in branch_annotations.iter().enumerate() {
                if let Some(annotation) = annotation {
                    self.validate_return_annotation(rule_name, idx + 1, annotation, &mut report);
                }
            }
        }

        for (rule_name, semantic_annotations) in &annotations.semantic_annotations {
            for (idx, semantic_annotation) in semantic_annotations.iter().enumerate() {
                self.validate_semantic_annotation(
                    rule_name,
                    idx + 1,
                    semantic_annotation,
                    &mut report,
                );
            }
            self.validate_semantic_directive_conflicts(
                rule_name,
                semantic_annotations,
                &mut report,
            );
        }

        report
    }

    pub fn validate_annotations_with_grammar(
        &self,
        annotations: &Annotations,
        grammar_tree: &HashMap<String, ASTNode>,
    ) -> AnnotationValidationReport {
        let mut report = self.validate_annotations(annotations);

        for (rule_name, branch_annotations) in &annotations.branch_return_annotations {
            let Some(rule_ast) = grammar_tree.get(rule_name) else {
                report.diagnostics.push(AnnotationDiagnostic {
                    code: "W_RET_RULE_NOT_FOUND",
                    severity: AnnotationSeverity::Warning,
                    kind: AnnotationKind::Return,
                    rule_name: rule_name.clone(),
                    annotation_index: None,
                    message: "Return annotation references rule that is missing from grammar tree."
                        .to_string(),
                    annotation: None,
                });
                continue;
            };

            let branches = self.top_level_branches(rule_ast);
            for (idx, branch_annotation) in branch_annotations.iter().enumerate() {
                let Some(annotation) = branch_annotation else {
                    continue;
                };
                let Some(parsed_ast) = annotation.parsed_ast.as_ref() else {
                    continue;
                };

                let max_positional_ref = self.max_positional_ref(parsed_ast);
                if max_positional_ref == 0 {
                    continue;
                }

                let Some(branch_ast) = branches.get(idx) else {
                    report.diagnostics.push(AnnotationDiagnostic {
                        code: "W_RET_BRANCH_INDEX_OOB",
                        severity: AnnotationSeverity::Warning,
                        kind: AnnotationKind::Return,
                        rule_name: rule_name.clone(),
                        annotation_index: Some(idx + 1),
                        message: format!(
                            "Return annotation targets branch {} but rule has only {} branch(es).",
                            idx + 1,
                            branches.len()
                        ),
                        annotation: Some(annotation.annotation_content.clone()),
                    });
                    continue;
                };

                let bound = self.positional_capture_bound(branch_ast);
                if bound == 0 {
                    report.diagnostics.push(AnnotationDiagnostic {
                        code: "W_RET_BRANCH_NOT_SEQUENCE",
                        severity: AnnotationSeverity::Warning,
                        kind: AnnotationKind::Return,
                        rule_name: rule_name.clone(),
                        annotation_index: Some(idx + 1),
                        message: format!(
                            "Return annotation uses positional references up to ${}, but branch {} does not expose sequence captures.",
                            max_positional_ref,
                            idx + 1
                        ),
                        annotation: Some(annotation.annotation_content.clone()),
                    });
                    continue;
                }

                if max_positional_ref > bound {
                    report.diagnostics.push(AnnotationDiagnostic {
                        code: "W_RET_POS_RULE_BOUND",
                        severity: AnnotationSeverity::Warning,
                        kind: AnnotationKind::Return,
                        rule_name: rule_name.clone(),
                        annotation_index: Some(idx + 1),
                        message: format!(
                            "Return annotation references ${}, but branch {} has only {} top-level capture slot(s).",
                            max_positional_ref,
                            idx + 1,
                            bound
                        ),
                        annotation: Some(annotation.annotation_content.clone()),
                    });
                }
            }
        }

        self.validate_grammar_ambiguity(grammar_tree, &mut report);

        report
    }

    fn validate_return_annotation(
        &self,
        rule_name: &str,
        annotation_index: usize,
        annotation: &BranchAnnotation,
        report: &mut AnnotationValidationReport,
    ) {
        let raw = Some(annotation.annotation_content.clone());

        let Some(ast) = annotation.parsed_ast.as_ref() else {
            report.diagnostics.push(AnnotationDiagnostic {
                code: "W_RET_UNPARSED",
                severity: AnnotationSeverity::Warning,
                kind: AnnotationKind::Return,
                rule_name: rule_name.to_string(),
                annotation_index: Some(annotation_index),
                message: "Return annotation was not parsed into typed AST; generation will fall back to raw behavior.".to_string(),
                annotation: raw,
            });
            return;
        };

        self.validate_return_ast(rule_name, annotation_index, ast, raw.as_deref(), report);
    }

    fn validate_return_ast(
        &self,
        rule_name: &str,
        annotation_index: usize,
        ast: &UnifiedReturnAST,
        raw_annotation: Option<&str>,
        report: &mut AnnotationValidationReport,
    ) {
        match ast {
            UnifiedReturnAST::PositionalRef { index } => {
                if *index == 0 {
                    report.diagnostics.push(AnnotationDiagnostic {
                        code: "E_RET_POS_ZERO",
                        severity: AnnotationSeverity::Error,
                        kind: AnnotationKind::Return,
                        rule_name: rule_name.to_string(),
                        annotation_index: Some(annotation_index),
                        message: "Positional reference '$0' is invalid for typed return validation; positions are 1-based.".to_string(),
                        annotation: raw_annotation.map(|s| s.to_string()),
                    });
                }

                if let Some(max_capture_index) = self.config.max_capture_index {
                    if *index > max_capture_index {
                        report.diagnostics.push(AnnotationDiagnostic {
                            code: "E_RET_POS_OUT_OF_RANGE",
                            severity: AnnotationSeverity::Error,
                            kind: AnnotationKind::Return,
                            rule_name: rule_name.to_string(),
                            annotation_index: Some(annotation_index),
                            message: format!(
                                "Positional reference '${}' exceeds configured capture bound {}.",
                                index, max_capture_index
                            ),
                            annotation: raw_annotation.map(|s| s.to_string()),
                        });
                    }
                }
            }
            UnifiedReturnAST::PropertyAccess { base, property } => {
                if property.trim().is_empty() {
                    report.diagnostics.push(AnnotationDiagnostic {
                        code: "E_RET_EMPTY_PROPERTY",
                        severity: AnnotationSeverity::Error,
                        kind: AnnotationKind::Return,
                        rule_name: rule_name.to_string(),
                        annotation_index: Some(annotation_index),
                        message: "Property access uses an empty property name.".to_string(),
                        annotation: raw_annotation.map(|s| s.to_string()),
                    });
                }
                self.validate_return_ast(rule_name, annotation_index, base, raw_annotation, report);
            }
            UnifiedReturnAST::ArrayAccess { base, index } => {
                self.validate_return_ast(rule_name, annotation_index, base, raw_annotation, report);
                self.validate_return_ast(
                    rule_name,
                    annotation_index,
                    index,
                    raw_annotation,
                    report,
                );
            }
            UnifiedReturnAST::QuantifiedExtraction { base, target } => {
                self.validate_return_ast(rule_name, annotation_index, base, raw_annotation, report);
                if let ExtractionTarget::Index(idx) = target {
                    if *idx > 10_000 {
                        report.diagnostics.push(AnnotationDiagnostic {
                            code: "W_RET_LARGE_EXTRACTION_INDEX",
                            severity: AnnotationSeverity::Warning,
                            kind: AnnotationKind::Return,
                            rule_name: rule_name.to_string(),
                            annotation_index: Some(annotation_index),
                            message: format!(
                                "Extraction index '{}' is unusually large; verify this is intentional.",
                                idx
                            ),
                            annotation: raw_annotation.map(|s| s.to_string()),
                        });
                    }
                }
            }
            UnifiedReturnAST::Object { properties } => {
                for (key, value) in properties {
                    if key.trim().is_empty() {
                        report.diagnostics.push(AnnotationDiagnostic {
                            code: "E_RET_EMPTY_OBJECT_KEY",
                            severity: AnnotationSeverity::Error,
                            kind: AnnotationKind::Return,
                            rule_name: rule_name.to_string(),
                            annotation_index: Some(annotation_index),
                            message: "Object return annotation contains an empty key.".to_string(),
                            annotation: raw_annotation.map(|s| s.to_string()),
                        });
                    }
                    self.validate_return_ast(
                        rule_name,
                        annotation_index,
                        value,
                        raw_annotation,
                        report,
                    );
                }
            }
            UnifiedReturnAST::Array { elements } => {
                for element in elements {
                    self.validate_return_ast(
                        rule_name,
                        annotation_index,
                        element,
                        raw_annotation,
                        report,
                    );
                }
            }
            UnifiedReturnAST::Spread { base } => {
                if matches!(base.as_ref(), UnifiedReturnAST::Passthrough) {
                    report.diagnostics.push(AnnotationDiagnostic {
                        code: "W_RET_SPREAD_PASSTHROUGH",
                        severity: AnnotationSeverity::Warning,
                        kind: AnnotationKind::Return,
                        rule_name: rule_name.to_string(),
                        annotation_index: Some(annotation_index),
                        message: "Spread over passthrough value is suspicious and may not produce intended AST shape.".to_string(),
                        annotation: raw_annotation.map(|s| s.to_string()),
                    });
                }
                self.validate_return_ast(rule_name, annotation_index, base, raw_annotation, report);
            }
            UnifiedReturnAST::StringLiteral { .. }
            | UnifiedReturnAST::NumberLiteral { .. }
            | UnifiedReturnAST::BooleanLiteral { .. }
            | UnifiedReturnAST::Passthrough => {}
        }
    }

    fn validate_semantic_annotation(
        &self,
        rule_name: &str,
        annotation_index: usize,
        semantic_annotation: &SemanticAnnotation,
        report: &mut AnnotationValidationReport,
    ) {
        if let Some(directive_name) = self.semantic_directive_name(semantic_annotation) {
            if semantic_directive_spec(&directive_name).is_none() {
                if let Some(severity) = self.unknown_semantic_directive_severity() {
                    report.diagnostics.push(AnnotationDiagnostic {
                        code: "W_SEM_UNKNOWN_DIRECTIVE",
                        severity,
                        kind: AnnotationKind::Semantic,
                        rule_name: rule_name.to_string(),
                        annotation_index: Some(annotation_index),
                        message: format!(
                            "Unknown semantic directive '{}' is not registered in the typed directive registry.",
                            directive_name
                        ),
                        annotation: Some(self.semantic_annotation_raw_text(semantic_annotation)),
                    });
                }
            }
        }

        self.validate_semantic_directive_payload(
            rule_name,
            annotation_index,
            semantic_annotation,
            report,
        );

        let semantic_ast = semantic_annotation.ast();
        match semantic_ast {
            UnifiedSemanticAST::TransformExpr { expression } => {
                self.validate_transform_expression(rule_name, annotation_index, expression, report);
            }
            UnifiedSemanticAST::Raw { content } => {
                if content.contains("::parse::<") || content.contains(">().unwrap_or(") {
                    report.diagnostics.push(AnnotationDiagnostic {
                        code: "W_SEM_MARKER_IN_RAW",
                        severity: AnnotationSeverity::Warning,
                        kind: AnnotationKind::Semantic,
                        rule_name: rule_name.to_string(),
                        annotation_index: Some(annotation_index),
                        message: "Raw semantic annotation contains transform markers but was not classified as TransformExpr.".to_string(),
                        annotation: Some(content.clone()),
                    });
                }
            }
        }
    }

    fn validate_semantic_directive_payload(
        &self,
        rule_name: &str,
        annotation_index: usize,
        semantic_annotation: &SemanticAnnotation,
        report: &mut AnnotationValidationReport,
    ) {
        let Some((directive_name, payload)) = self.semantic_directive_parts(semantic_annotation)
        else {
            return;
        };

        let raw_annotation = self.semantic_annotation_raw_text(semantic_annotation);
        let payload_trimmed = payload.trim();

        match directive_name.as_str() {
            "associativity" => {
                if SemanticAssociativity::parse(payload_trimmed).is_none() {
                    report.diagnostics.push(AnnotationDiagnostic {
                        code: "W_SEM_INVALID_ASSOCIATIVITY_PAYLOAD",
                        severity: AnnotationSeverity::Warning,
                        kind: AnnotationKind::Semantic,
                        rule_name: rule_name.to_string(),
                        annotation_index: Some(annotation_index),
                        message:
                            "Directive '@associativity' expects one of: left, right, nonassoc."
                                .to_string(),
                        annotation: Some(raw_annotation),
                    });
                }
            }
            "priority" | "precedence" => {
                if parse_semantic_numeric_list(payload_trimmed).is_none() {
                    report.diagnostics.push(AnnotationDiagnostic {
                        code: "W_SEM_INVALID_PRIORITY_PAYLOAD",
                        severity: AnnotationSeverity::Warning,
                        kind: AnnotationKind::Semantic,
                        rule_name: rule_name.to_string(),
                        annotation_index: Some(annotation_index),
                        message: "Directive '@priority/@precedence' expects an integer payload such as '5' or '[1, 9, 2]'.".to_string(),
                        annotation: Some(raw_annotation),
                    });
                }
            }
            "enum" => {
                if parse_semantic_string_list(payload_trimmed).is_none() {
                    report.diagnostics.push(AnnotationDiagnostic {
                        code: "W_SEM_INVALID_ENUM_PAYLOAD",
                        severity: AnnotationSeverity::Warning,
                        kind: AnnotationKind::Semantic,
                        rule_name: rule_name.to_string(),
                        annotation_index: Some(annotation_index),
                        message: "Directive '@enum' expects one or more scalar values, for example '[\"A\", \"B\"]'.".to_string(),
                        annotation: Some(raw_annotation),
                    });
                }
            }
            "range" => {
                if parse_semantic_numeric_bounds(payload_trimmed).is_none() {
                    report.diagnostics.push(AnnotationDiagnostic {
                        code: "W_SEM_INVALID_RANGE_PAYLOAD",
                        severity: AnnotationSeverity::Warning,
                        kind: AnnotationKind::Semantic,
                        rule_name: rule_name.to_string(),
                        annotation_index: Some(annotation_index),
                        message: "Directive '@range' expects numeric bounds such as '0..10' or '[0, 10]'.".to_string(),
                        annotation: Some(raw_annotation),
                    });
                }
            }
            "len" => {
                if parse_semantic_len_bounds(payload_trimmed).is_none() {
                    report.diagnostics.push(AnnotationDiagnostic {
                        code: "W_SEM_INVALID_LEN_PAYLOAD",
                        severity: AnnotationSeverity::Warning,
                        kind: AnnotationKind::Semantic,
                        rule_name: rule_name.to_string(),
                        annotation_index: Some(annotation_index),
                        message: "Directive '@len' expects non-negative integer bounds such as '2..8' or '[2, 8]'.".to_string(),
                        annotation: Some(raw_annotation),
                    });
                }
            }
            "regex" => {
                let pattern = normalize_semantic_scalar(payload_trimmed);
                if pattern.is_empty() {
                    report.diagnostics.push(AnnotationDiagnostic {
                        code: "W_SEM_INVALID_REGEX_PAYLOAD",
                        severity: AnnotationSeverity::Warning,
                        kind: AnnotationKind::Semantic,
                        rule_name: rule_name.to_string(),
                        annotation_index: Some(annotation_index),
                        message:
                            "Directive '@regex' expects a non-empty regular expression pattern."
                                .to_string(),
                        annotation: Some(raw_annotation),
                    });
                    return;
                }

                if let Err(err) = Regex::new(pattern.as_str()) {
                    report.diagnostics.push(AnnotationDiagnostic {
                        code: "W_SEM_INVALID_REGEX_PAYLOAD",
                        severity: AnnotationSeverity::Warning,
                        kind: AnnotationKind::Semantic,
                        rule_name: rule_name.to_string(),
                        annotation_index: Some(annotation_index),
                        message: format!(
                            "Directive '@regex' contains an invalid regular expression: {}.",
                            err
                        ),
                        annotation: Some(raw_annotation),
                    });
                }
            }
            _ => {}
        }
    }

    fn validate_semantic_directive_conflicts(
        &self,
        rule_name: &str,
        semantic_annotations: &[SemanticAnnotation],
        report: &mut AnnotationValidationReport,
    ) {
        let mut directive_occurrences: HashMap<String, Vec<(usize, String)>> = HashMap::new();

        for (idx, semantic_annotation) in semantic_annotations.iter().enumerate() {
            let Some((directive_name, payload)) =
                self.semantic_directive_parts(semantic_annotation)
            else {
                continue;
            };

            if semantic_directive_spec(&directive_name).is_none() {
                continue;
            }

            directive_occurrences
                .entry(directive_name)
                .or_default()
                .push((idx + 1, payload));
        }

        if let (Some(priority_entries), Some(precedence_entries)) = (
            directive_occurrences.get("priority"),
            directive_occurrences.get("precedence"),
        ) {
            let annotation_index = priority_entries
                .last()
                .map(|(idx, _)| *idx)
                .max(precedence_entries.last().map(|(idx, _)| *idx));
            let priority_payload = priority_entries
                .last()
                .map(|(_, payload)| payload.as_str())
                .unwrap_or("");
            let precedence_payload = precedence_entries
                .last()
                .map(|(_, payload)| payload.as_str())
                .unwrap_or("");

            report.diagnostics.push(AnnotationDiagnostic {
                code: "W_SEM_PRIORITY_PRECEDENCE_CONFLICT",
                severity: AnnotationSeverity::Warning,
                kind: AnnotationKind::Semantic,
                rule_name: rule_name.to_string(),
                annotation_index,
                message: "Both '@priority' and '@precedence' are present; deterministic conflict policy applies with '@priority' taking precedence.".to_string(),
                annotation: Some(format!(
                    "@priority: {}; @precedence: {}",
                    priority_payload, precedence_payload
                )),
            });
        }

        self.validate_unsatisfiable_value_domain_intersection(
            rule_name,
            &directive_occurrences,
            report,
        );

        for (directive_name, entries) in &directive_occurrences {
            if entries.len() <= 1 {
                continue;
            }

            let (last_index, last_payload) = entries
                .last()
                .map(|(idx, payload)| (*idx, payload.as_str()))
                .unwrap_or((1, ""));
            report.diagnostics.push(AnnotationDiagnostic {
                code: "W_SEM_DIRECTIVE_OVERRIDDEN",
                severity: AnnotationSeverity::Warning,
                kind: AnnotationKind::Semantic,
                rule_name: rule_name.to_string(),
                annotation_index: Some(last_index),
                message: format!(
                    "Directive '@{}' appears {} times for rule '{}'; deterministic conflict policy uses the last occurrence.",
                    directive_name,
                    entries.len(),
                    rule_name
                ),
                annotation: Some(format!("@{}: {}", directive_name.as_str(), last_payload)),
            });
        }
    }

    fn validate_unsatisfiable_value_domain_intersection(
        &self,
        rule_name: &str,
        directive_occurrences: &HashMap<String, Vec<(usize, String)>>,
        report: &mut AnnotationValidationReport,
    ) {
        let Some((enum_idx, enum_payload)) =
            Self::latest_directive_payload(directive_occurrences, "enum")
        else {
            return;
        };
        let Some(enum_values) = parse_semantic_string_list(enum_payload) else {
            return;
        };
        if enum_values.is_empty() {
            return;
        }

        let len_bounds = Self::latest_directive_payload(directive_occurrences, "len")
            .and_then(|(_, payload)| parse_semantic_len_bounds(payload));
        let numeric_bounds = Self::latest_directive_payload(directive_occurrences, "range")
            .and_then(|(_, payload)| parse_semantic_numeric_bounds(payload));
        let regex_payload = Self::latest_directive_payload(directive_occurrences, "regex")
            .map(|(_, payload)| normalize_semantic_scalar(payload))
            .filter(|payload| !payload.is_empty());
        let semantic_regex = match regex_payload.as_deref() {
            Some(pattern) => match Regex::new(pattern) {
                Ok(compiled) => Some(compiled),
                Err(_) => return,
            },
            None => None,
        };

        if len_bounds.is_none() && numeric_bounds.is_none() && semantic_regex.is_none() {
            return;
        }

        let any_candidate_satisfies = enum_values.iter().any(|value| {
            if let Some((min_len, max_len)) = len_bounds {
                let len = value.chars().count();
                if len < min_len || len > max_len {
                    return false;
                }
            }

            if let Some((min_numeric, max_numeric)) = numeric_bounds {
                let Ok(parsed_numeric) = value.parse::<f64>() else {
                    return false;
                };
                if parsed_numeric < min_numeric || parsed_numeric > max_numeric {
                    return false;
                }
            }

            if let Some(regex) = &semantic_regex {
                if !Self::regex_matches_entire(regex, value) {
                    return false;
                }
            }

            true
        });

        if any_candidate_satisfies {
            return;
        }

        let mut annotation_index = enum_idx;
        let mut details = vec![format!("@enum: {}", enum_payload)];

        if let Some((idx, payload)) = Self::latest_directive_payload(directive_occurrences, "len") {
            annotation_index = annotation_index.max(idx);
            details.push(format!("@len: {}", payload));
        }
        if let Some((idx, payload)) = Self::latest_directive_payload(directive_occurrences, "range")
        {
            annotation_index = annotation_index.max(idx);
            details.push(format!("@range: {}", payload));
        }
        if let Some((idx, payload)) = Self::latest_directive_payload(directive_occurrences, "regex")
        {
            annotation_index = annotation_index.max(idx);
            details.push(format!("@regex: {}", payload));
        }

        report.diagnostics.push(AnnotationDiagnostic {
            code: "W_SEM_UNSATISFIABLE_VALUE_DOMAIN",
            severity: AnnotationSeverity::Warning,
            kind: AnnotationKind::Semantic,
            rule_name: rule_name.to_string(),
            annotation_index: Some(annotation_index),
            message: "Value-domain directives are unsatisfiable: no '@enum' value satisfies all active constraints.".to_string(),
            annotation: Some(details.join("; ")),
        });
    }

    fn latest_directive_payload<'a>(
        directive_occurrences: &'a HashMap<String, Vec<(usize, String)>>,
        directive_name: &str,
    ) -> Option<(usize, &'a str)> {
        directive_occurrences
            .get(directive_name)?
            .last()
            .map(|(idx, payload)| (*idx, payload.as_str()))
    }

    fn regex_matches_entire(regex: &Regex, candidate: &str) -> bool {
        regex
            .find(candidate)
            .map(|matched| matched.start() == 0 && matched.end() == candidate.len())
            .unwrap_or(false)
    }

    fn semantic_annotation_raw_text(&self, semantic_annotation: &SemanticAnnotation) -> String {
        match semantic_annotation {
            SemanticAnnotation::Legacy(UnifiedSemanticAST::TransformExpr { expression }) => {
                expression.clone()
            }
            SemanticAnnotation::Legacy(UnifiedSemanticAST::Raw { content }) => content.clone(),
            SemanticAnnotation::Named { name, ast } => match ast {
                UnifiedSemanticAST::TransformExpr { expression } => {
                    format!("@{}: {}", name, expression)
                }
                UnifiedSemanticAST::Raw { content } => format!("@{}: {}", name, content),
            },
        }
    }

    fn semantic_directive_name(&self, semantic_annotation: &SemanticAnnotation) -> Option<String> {
        if let Some(name) = semantic_annotation.name() {
            let normalized = name.trim().to_ascii_lowercase();
            if !normalized.is_empty() {
                return Some(normalized);
            }
        }

        match semantic_annotation.ast() {
            UnifiedSemanticAST::TransformExpr { expression } => {
                if let Some(name) = extract_semantic_directive_name(expression) {
                    return Some(name);
                }
                Some("transform".to_string())
            }
            UnifiedSemanticAST::Raw { content } => extract_semantic_directive_name(content),
        }
    }

    fn semantic_directive_parts(
        &self,
        semantic_annotation: &SemanticAnnotation,
    ) -> Option<(String, String)> {
        if let Some(name) = semantic_annotation.name() {
            let normalized = name.trim().to_ascii_lowercase();
            if !normalized.is_empty() {
                let payload = match semantic_annotation.ast() {
                    UnifiedSemanticAST::TransformExpr { expression } => expression.clone(),
                    UnifiedSemanticAST::Raw { content } => content.clone(),
                };
                return Some((normalized, payload.trim().to_string()));
            }
        }

        match semantic_annotation.ast() {
            UnifiedSemanticAST::TransformExpr { expression } => {
                if let Some(parts) = extract_semantic_directive(expression) {
                    return Some(parts);
                }
                Some(("transform".to_string(), expression.trim().to_string()))
            }
            UnifiedSemanticAST::Raw { content } => extract_semantic_directive(content),
        }
    }

    fn unknown_semantic_directive_severity(&self) -> Option<AnnotationSeverity> {
        match self.config.unknown_semantic_directive_policy {
            UnknownSemanticDirectivePolicy::Ignore => None,
            UnknownSemanticDirectivePolicy::Warn => Some(AnnotationSeverity::Warning),
            UnknownSemanticDirectivePolicy::Strict => Some(AnnotationSeverity::Error),
        }
    }

    fn validate_transform_expression(
        &self,
        rule_name: &str,
        annotation_index: usize,
        expression: &str,
        report: &mut AnnotationValidationReport,
    ) {
        let Some(transform) = parse_canonical_transform_expression(expression) else {
            report.diagnostics.push(AnnotationDiagnostic {
                code: "W_SEM_NON_CANONICAL_TRANSFORM",
                severity: self.semantic_check_severity(),
                kind: AnnotationKind::Semantic,
                rule_name: rule_name.to_string(),
                annotation_index: Some(annotation_index),
                message: "Transform expression does not match canonical 'str::parse::<T>().unwrap_or(default)' form.".to_string(),
                annotation: Some(expression.to_string()),
            });
            return;
        };

        let target_type = transform.target_type.as_str();
        let default_expr = transform.default_expr.as_str();

        if target_type.is_empty() || default_expr.is_empty() {
            report.diagnostics.push(AnnotationDiagnostic {
                code: "E_SEM_EMPTY_COMPONENT",
                severity: AnnotationSeverity::Error,
                kind: AnnotationKind::Semantic,
                rule_name: rule_name.to_string(),
                annotation_index: Some(annotation_index),
                message: "Transform expression has empty parse target type or default expression."
                    .to_string(),
                annotation: Some(expression.to_string()),
            });
            return;
        }

        self.validate_transform_type_default(
            rule_name,
            annotation_index,
            target_type,
            default_expr,
            expression,
            report,
        );
    }

    fn validate_transform_type_default(
        &self,
        rule_name: &str,
        annotation_index: usize,
        target_type: &str,
        default_expr: &str,
        raw_expression: &str,
        report: &mut AnnotationValidationReport,
    ) {
        let integer_types: HashSet<&str> = [
            "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
        ]
        .iter()
        .copied()
        .collect();
        let float_types: HashSet<&str> = ["f32", "f64"].iter().copied().collect();

        let integer_default_re = Regex::new(
            r"^-?[0-9](?:[0-9_]*)(?:i8|i16|i32|i64|i128|isize|u8|u16|u32|u64|u128|usize)?$",
        )
        .expect("int default regex must compile");
        let float_default_re = Regex::new(
            r"^-?(?:(?:[0-9](?:[0-9_]*)?(?:\.[0-9_]*)?)|(?:\.[0-9_]+))(?:[eE][+-]?[0-9_]+)?(?:f32|f64)?$",
        )
        .expect("float default regex must compile");

        let type_known = integer_types.contains(target_type)
            || float_types.contains(target_type)
            || matches!(target_type, "bool" | "String" | "str");

        if !type_known {
            report.diagnostics.push(AnnotationDiagnostic {
                code: "W_SEM_UNKNOWN_TARGET_TYPE",
                severity: self.semantic_check_severity(),
                kind: AnnotationKind::Semantic,
                rule_name: rule_name.to_string(),
                annotation_index: Some(annotation_index),
                message: format!(
                    "Unknown transform target type '{}' in semantic annotation.",
                    target_type
                ),
                annotation: Some(raw_expression.to_string()),
            });
            return;
        }

        let default_matches = if integer_types.contains(target_type) {
            integer_default_re.is_match(default_expr)
        } else if float_types.contains(target_type) {
            float_default_re.is_match(default_expr)
        } else if target_type == "bool" {
            matches!(default_expr, "true" | "false")
        } else {
            // String/str are flexible (String::new(), "", etc.)
            true
        };

        if !default_matches {
            report.diagnostics.push(AnnotationDiagnostic {
                code: "W_SEM_DEFAULT_TYPE_MISMATCH",
                severity: self.semantic_check_severity(),
                kind: AnnotationKind::Semantic,
                rule_name: rule_name.to_string(),
                annotation_index: Some(annotation_index),
                message: format!(
                    "Default expression '{}' does not look compatible with transform target type '{}'.",
                    default_expr, target_type
                ),
                annotation: Some(raw_expression.to_string()),
            });
        }
    }

    fn semantic_check_severity(&self) -> AnnotationSeverity {
        if self.config.strict_semantic_transforms {
            AnnotationSeverity::Error
        } else {
            AnnotationSeverity::Warning
        }
    }

    fn top_level_branches<'a>(&self, rule_ast: &'a ASTNode) -> Vec<&'a ASTNode> {
        match rule_ast {
            ASTNode::Or { alternatives } => alternatives.iter().collect(),
            _ => vec![rule_ast],
        }
    }

    fn validate_grammar_ambiguity(
        &self,
        grammar_tree: &HashMap<String, ASTNode>,
        report: &mut AnnotationValidationReport,
    ) {
        let mut rule_names: Vec<&String> = grammar_tree.keys().collect();
        rule_names.sort_unstable();

        for rule_name in rule_names {
            let Some(rule_ast) = grammar_tree.get(rule_name) else {
                continue;
            };

            let branches = self.top_level_branches(rule_ast);
            if branches.len() < 2 {
                continue;
            }

            let mut prefix_signatures = HashSet::new();
            let mut fingerprint_to_branches: HashMap<String, Vec<usize>> = HashMap::new();
            for (idx, branch) in branches.iter().enumerate() {
                let Some(fingerprint) = self.branch_leading_terminal_fingerprint(branch) else {
                    continue;
                };
                fingerprint_to_branches
                    .entry(fingerprint)
                    .or_default()
                    .push(idx + 1);
            }

            let mut grouped: Vec<(String, Vec<usize>)> =
                fingerprint_to_branches.into_iter().collect();
            grouped.sort_by(|left, right| left.0.cmp(&right.0));

            for (fingerprint, mut branch_indices) in grouped {
                if branch_indices.len() < 2 {
                    continue;
                }
                branch_indices.sort_unstable();
                let branch_list = branch_indices
                    .iter()
                    .map(|idx| idx.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                let signature = format!("{}|{}", fingerprint, branch_list);
                prefix_signatures.insert(signature);

                report.diagnostics.push(AnnotationDiagnostic {
                    code: "W_GRAM_AMBIGUOUS_PREFIX",
                    severity: AnnotationSeverity::Warning,
                    kind: AnnotationKind::Grammar,
                    rule_name: rule_name.to_string(),
                    annotation_index: None,
                    message: format!(
                        "Rule '{}' has alternative branches [{}] sharing leading terminal {}; parse selection may depend on branch order.",
                        rule_name, branch_list, fingerprint
                    ),
                    annotation: None,
                });
            }

            let mut first_set_cache: HashMap<String, FirstSetSummary> = HashMap::new();
            let mut branch_first_sets = Vec::with_capacity(branches.len());
            for (idx, branch) in branches.iter().enumerate() {
                let mut visiting_rules = HashSet::new();
                let summary = self.branch_first_set(
                    branch,
                    grammar_tree,
                    &mut first_set_cache,
                    &mut visiting_rules,
                    0,
                );
                branch_first_sets.push((idx + 1, summary));
            }

            let mut first_to_branches: HashMap<String, Vec<usize>> = HashMap::new();
            for (branch_index, summary) in &branch_first_sets {
                for terminal in &summary.terminals {
                    first_to_branches
                        .entry(terminal.clone())
                        .or_default()
                        .push(*branch_index);
                }
            }

            let mut first_overlaps: Vec<(String, Vec<usize>)> = first_to_branches.into_iter().collect();
            first_overlaps.sort_by(|left, right| left.0.cmp(&right.0));

            for (terminal, mut branch_indices) in first_overlaps {
                branch_indices.sort_unstable();
                branch_indices.dedup();
                if branch_indices.len() < 2 {
                    continue;
                }

                let branch_list = branch_indices
                    .iter()
                    .map(|idx| idx.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                let signature = format!("{}|{}", terminal, branch_list);
                if prefix_signatures.contains(&signature) {
                    continue;
                }

                report.diagnostics.push(AnnotationDiagnostic {
                    code: "W_GRAM_FIRST_SET_OVERLAP",
                    severity: AnnotationSeverity::Warning,
                    kind: AnnotationKind::Grammar,
                    rule_name: rule_name.to_string(),
                    annotation_index: None,
                    message: format!(
                        "Rule '{}' has alternative branches [{}] with overlapping FIRST terminal {}; parse selection may depend on branch order.",
                        rule_name, branch_list, terminal
                    ),
                    annotation: None,
                });
            }

            for (branch_index, summary) in &branch_first_sets {
                if summary.nullable && *branch_index < branches.len() {
                    let unresolved_note = if summary.unresolved {
                        " (nullable from partial FIRST analysis)"
                    } else {
                        ""
                    };
                    report.diagnostics.push(AnnotationDiagnostic {
                        code: "W_GRAM_NULLABLE_BRANCH_SHADOW",
                        severity: AnnotationSeverity::Warning,
                        kind: AnnotationKind::Grammar,
                        rule_name: rule_name.to_string(),
                        annotation_index: None,
                        message: format!(
                            "Rule '{}' has nullable alternative branch {} before later branches; ordered choice may shadow subsequent alternatives{}.",
                            rule_name, branch_index, unresolved_note
                        ),
                        annotation: None,
                    });
                }
            }
        }
    }

    fn branch_leading_terminal_fingerprint(&self, node: &ASTNode) -> Option<String> {
        match node {
            ASTNode::Sequence { elements } => elements
                .first()
                .and_then(|first| self.branch_leading_terminal_fingerprint(first)),
            ASTNode::Atom { value } => self.atom_terminal_fingerprint(value),
            ASTNode::Quantified {
                element,
                quantifier,
            } if quantifier == "+" => self.branch_leading_terminal_fingerprint(element),
            ASTNode::Or { alternatives } => {
                let mut shared: Option<String> = None;
                for alternative in alternatives {
                    let candidate = self.branch_leading_terminal_fingerprint(alternative)?;
                    match shared.as_ref() {
                        None => shared = Some(candidate),
                        Some(existing) if existing == &candidate => {}
                        Some(_) => return None,
                    }
                }
                shared
            }
            _ => None,
        }
    }

    fn atom_terminal_fingerprint(&self, value: &ASTValue) -> Option<String> {
        let ASTValue::Token(parts) = value else {
            return None;
        };
        if parts.len() < 2 {
            return None;
        }

        let token_type = match &parts[0] {
            super::TokenValue::String(token_type) => token_type.as_str(),
        };
        let token_value = match &parts[1] {
            super::TokenValue::String(token_value) => token_value.as_str(),
        };

        if token_type == "quoted_string" {
            Some(format!("'{}'", token_value))
        } else {
            None
        }
    }

    fn branch_first_set(
        &self,
        node: &ASTNode,
        grammar_tree: &HashMap<String, ASTNode>,
        first_set_cache: &mut HashMap<String, FirstSetSummary>,
        visiting_rules: &mut HashSet<String>,
        depth: usize,
    ) -> FirstSetSummary {
        const MAX_FIRST_SET_DEPTH: usize = 24;
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
                    let element_first = self.branch_first_set(
                        element,
                        grammar_tree,
                        first_set_cache,
                        visiting_rules,
                        depth + 1,
                    );
                    result.terminals.extend(element_first.terminals.iter().cloned());
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
                    let alternative_first = self.branch_first_set(
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
            ASTNode::Atom { value } => self.atom_first_set(
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
                let mut element_first = self.branch_first_set(
                    element,
                    grammar_tree,
                    first_set_cache,
                    visiting_rules,
                    depth + 1,
                );
                let min_repeat = self.quantifier_min_repeat(quantifier);
                if min_repeat == 0 {
                    element_first.nullable = true;
                }
                element_first
            }
        }
    }

    fn atom_first_set(
        &self,
        value: &ASTValue,
        grammar_tree: &HashMap<String, ASTNode>,
        first_set_cache: &mut HashMap<String, FirstSetSummary>,
        visiting_rules: &mut HashSet<String>,
        depth: usize,
    ) -> FirstSetSummary {
        match value {
            ASTValue::Node(node) => self.branch_first_set(
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
                    super::TokenValue::String(token_type) => token_type.as_str(),
                };
                let token_value = match &parts[1] {
                    super::TokenValue::String(token_value) => token_value.as_str(),
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
                    "rule_reference" => self.rule_first_set(
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
        &self,
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
            self.branch_first_set(
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

    fn quantifier_min_repeat(&self, quantifier: &str) -> usize {
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

    fn positional_capture_bound(&self, branch_ast: &ASTNode) -> usize {
        match branch_ast {
            ASTNode::Sequence { elements } => elements.len(),
            _ => 0,
        }
    }

    fn max_positional_ref(&self, ast: &UnifiedReturnAST) -> usize {
        match ast {
            UnifiedReturnAST::PositionalRef { index } => *index,
            UnifiedReturnAST::Spread { base }
            | UnifiedReturnAST::PropertyAccess { base, .. }
            | UnifiedReturnAST::QuantifiedExtraction { base, .. } => self.max_positional_ref(base),
            UnifiedReturnAST::ArrayAccess { base, index } => self
                .max_positional_ref(base)
                .max(self.max_positional_ref(index)),
            UnifiedReturnAST::Object { properties } => properties
                .values()
                .map(|value| self.max_positional_ref(value))
                .max()
                .unwrap_or(0),
            UnifiedReturnAST::Array { elements } => elements
                .iter()
                .map(|value| self.max_positional_ref(value))
                .max()
                .unwrap_or(0),
            UnifiedReturnAST::StringLiteral { .. }
            | UnifiedReturnAST::NumberLiteral { .. }
            | UnifiedReturnAST::BooleanLiteral { .. }
            | UnifiedReturnAST::Passthrough => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast_pipeline::{ASTValue, TokenValue};

    #[test]
    fn return_validator_flags_zero_positional_reference() {
        let mut annotations = Annotations::default();
        annotations.branch_return_annotations.insert(
            "rule".to_string(),
            vec![Some(BranchAnnotation {
                annotation_type: "return_scalar".to_string(),
                annotation_content: "$0".to_string(),
                parsed_ast: Some(UnifiedReturnAST::PositionalRef { index: 0 }),
            })],
        );

        let report = AnnotationValidator::default().validate_annotations(&annotations);
        assert!(report.has_errors());
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.code == "E_RET_POS_ZERO")
        );
    }

    #[test]
    fn return_validator_warns_on_unparsed_annotation() {
        let mut annotations = Annotations::default();
        annotations.branch_return_annotations.insert(
            "rule".to_string(),
            vec![Some(BranchAnnotation {
                annotation_type: "return_object".to_string(),
                annotation_content: "{complex: syntax}".to_string(),
                parsed_ast: None,
            })],
        );

        let report = AnnotationValidator::default().validate_annotations(&annotations);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.code == "W_RET_UNPARSED" && d.severity == AnnotationSeverity::Warning)
        );
    }

    #[test]
    fn semantic_validator_accepts_canonical_transform() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "number".to_string(),
            vec![
                UnifiedSemanticAST::TransformExpr {
                    expression: "str::parse::<u32>().unwrap_or(0)".to_string(),
                }
                .into(),
            ],
        );

        let report = AnnotationValidator::default().validate_annotations(&annotations);
        assert!(!report.has_errors());
        assert_eq!(report.warning_count(), 0);
    }

    #[test]
    fn semantic_validator_strict_mode_promotes_noncanonical_to_error() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "number".to_string(),
            vec![
                UnifiedSemanticAST::TransformExpr {
                    expression: "parse(value)".to_string(),
                }
                .into(),
            ],
        );

        let report = AnnotationValidator::new(AnnotationValidatorConfig {
            max_capture_index: None,
            strict_semantic_transforms: true,
            unknown_semantic_directive_policy: UnknownSemanticDirectivePolicy::Warn,
        })
        .validate_annotations(&annotations);

        assert!(report.has_errors());
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.code == "W_SEM_NON_CANONICAL_TRANSFORM"
                    && d.severity == AnnotationSeverity::Error)
        );
    }

    #[test]
    fn semantic_validator_warns_on_unknown_directive_in_warn_mode() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "rule".to_string(),
            vec![SemanticAnnotation::Named {
                name: "custom_directive".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: "\"value\"".to_string(),
                },
            }],
        );

        let report = AnnotationValidator::new(AnnotationValidatorConfig {
            max_capture_index: None,
            strict_semantic_transforms: false,
            unknown_semantic_directive_policy: UnknownSemanticDirectivePolicy::Warn,
        })
        .validate_annotations(&annotations);

        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.code == "W_SEM_UNKNOWN_DIRECTIVE"
                    && d.severity == AnnotationSeverity::Warning)
        );
    }

    #[test]
    fn semantic_validator_errors_on_unknown_directive_in_strict_mode() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "rule".to_string(),
            vec![SemanticAnnotation::Named {
                name: "custom_directive".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: "\"value\"".to_string(),
                },
            }],
        );

        let report = AnnotationValidator::new(AnnotationValidatorConfig {
            max_capture_index: None,
            strict_semantic_transforms: false,
            unknown_semantic_directive_policy: UnknownSemanticDirectivePolicy::Strict,
        })
        .validate_annotations(&annotations);

        assert!(report.has_errors());
        assert!(report.diagnostics.iter().any(
            |d| d.code == "W_SEM_UNKNOWN_DIRECTIVE" && d.severity == AnnotationSeverity::Error
        ));
    }

    #[test]
    fn return_validator_honors_capture_index_bounds() {
        let mut annotations = Annotations::default();
        annotations.branch_return_annotations.insert(
            "rule".to_string(),
            vec![Some(BranchAnnotation {
                annotation_type: "return_scalar".to_string(),
                annotation_content: "$4".to_string(),
                parsed_ast: Some(UnifiedReturnAST::PositionalRef { index: 4 }),
            })],
        );

        let report = AnnotationValidator::new(AnnotationValidatorConfig {
            max_capture_index: Some(3),
            strict_semantic_transforms: false,
            unknown_semantic_directive_policy: UnknownSemanticDirectivePolicy::Warn,
        })
        .validate_annotations(&annotations);

        assert!(report.has_errors());
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.code == "E_RET_POS_OUT_OF_RANGE")
        );
    }

    #[test]
    fn grammar_aware_validation_warns_when_positional_ref_exceeds_branch_bound() {
        let mut annotations = Annotations::default();
        annotations.branch_return_annotations.insert(
            "rule".to_string(),
            vec![Some(BranchAnnotation {
                annotation_type: "return_scalar".to_string(),
                annotation_content: "$3".to_string(),
                parsed_ast: Some(UnifiedReturnAST::PositionalRef { index: 3 }),
            })],
        );

        let grammar_tree = HashMap::from([(
            "rule".to_string(),
            ASTNode::Sequence {
                elements: vec![
                    ASTNode::Atom {
                        value: ASTValue::Token(vec![
                            TokenValue::String("quoted_string".to_string()),
                            TokenValue::String("a".to_string()),
                        ]),
                    },
                    ASTNode::Atom {
                        value: ASTValue::Token(vec![
                            TokenValue::String("quoted_string".to_string()),
                            TokenValue::String("b".to_string()),
                        ]),
                    },
                ],
            },
        )]);

        let report = AnnotationValidator::default()
            .validate_annotations_with_grammar(&annotations, &grammar_tree);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.code == "W_RET_POS_RULE_BOUND")
        );
    }

    #[test]
    fn grammar_aware_validation_warns_on_non_sequence_branch_positional_ref() {
        let mut annotations = Annotations::default();
        annotations.branch_return_annotations.insert(
            "rule".to_string(),
            vec![Some(BranchAnnotation {
                annotation_type: "return_scalar".to_string(),
                annotation_content: "$1".to_string(),
                parsed_ast: Some(UnifiedReturnAST::PositionalRef { index: 1 }),
            })],
        );

        let grammar_tree = HashMap::from([(
            "rule".to_string(),
            ASTNode::Atom {
                value: ASTValue::Token(vec![
                    TokenValue::String("quoted_string".to_string()),
                    TokenValue::String("x".to_string()),
                ]),
            },
        )]);

        let report = AnnotationValidator::default()
            .validate_annotations_with_grammar(&annotations, &grammar_tree);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.code == "W_RET_BRANCH_NOT_SEQUENCE")
        );
    }

    #[test]
    fn semantic_validator_warns_on_invalid_associativity_payload() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "expr".to_string(),
            vec![SemanticAnnotation::Named {
                name: "associativity".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: "\"diagonal\"".to_string(),
                },
            }],
        );

        let report = AnnotationValidator::default().validate_annotations(&annotations);
        assert!(report.diagnostics.iter().any(|d| {
            d.code == "W_SEM_INVALID_ASSOCIATIVITY_PAYLOAD"
                && d.severity == AnnotationSeverity::Warning
        }));
    }

    #[test]
    fn semantic_validator_warns_on_invalid_priority_payload() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "expr".to_string(),
            vec![SemanticAnnotation::Named {
                name: "priority".to_string(),
                ast: UnifiedSemanticAST::Raw {
                    content: "{level: 5}".to_string(),
                },
            }],
        );

        let report = AnnotationValidator::default().validate_annotations(&annotations);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.code == "W_SEM_INVALID_PRIORITY_PAYLOAD")
        );
    }

    #[test]
    fn semantic_validator_warns_on_invalid_value_domain_payloads() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "token".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "range".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "\"low..high\"".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "len".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[-1, 2]".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "enum".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[]".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "regex".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "\"[A-Z+\"".to_string(),
                    },
                },
            ],
        );

        let report = AnnotationValidator::default().validate_annotations(&annotations);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.code == "W_SEM_INVALID_RANGE_PAYLOAD")
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.code == "W_SEM_INVALID_LEN_PAYLOAD")
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.code == "W_SEM_INVALID_ENUM_PAYLOAD")
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.code == "W_SEM_INVALID_REGEX_PAYLOAD")
        );
    }

    #[test]
    fn semantic_validator_warns_when_priority_and_precedence_both_present() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "expr".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "precedence".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[9, 1]".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "priority".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[1, 9]".to_string(),
                    },
                },
            ],
        );

        let report = AnnotationValidator::default().validate_annotations(&annotations);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.code == "W_SEM_PRIORITY_PRECEDENCE_CONFLICT")
        );
    }

    #[test]
    fn semantic_validator_warns_on_duplicate_directive_override_contract() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "expr".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "associativity".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "left".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "associativity".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "right".to_string(),
                    },
                },
            ],
        );

        let report = AnnotationValidator::default().validate_annotations(&annotations);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.code == "W_SEM_DIRECTIVE_OVERRIDDEN")
        );
    }

    #[test]
    fn semantic_validator_warns_on_unsatisfiable_enum_regex_intersection() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "ident".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "enum".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[\"AA\", \"BB\"]".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "regex".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "^C[A-Z]$".to_string(),
                    },
                },
            ],
        );

        let report = AnnotationValidator::default().validate_annotations(&annotations);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.code == "W_SEM_UNSATISFIABLE_VALUE_DOMAIN")
        );
    }

    #[test]
    fn semantic_validator_warns_on_unsatisfiable_enum_range_intersection() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "number".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "enum".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[\"1\", \"2\", \"3\"]".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "range".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "10..20".to_string(),
                    },
                },
            ],
        );

        let report = AnnotationValidator::default().validate_annotations(&annotations);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.code == "W_SEM_UNSATISFIABLE_VALUE_DOMAIN")
        );
    }

    #[test]
    fn semantic_validator_does_not_warn_when_enum_intersection_is_satisfiable() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "ident".to_string(),
            vec![
                SemanticAnnotation::Named {
                    name: "enum".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[\"AA\", \"AB\", \"BC\"]".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "regex".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "^A[A-Z]$".to_string(),
                    },
                },
                SemanticAnnotation::Named {
                    name: "len".to_string(),
                    ast: UnifiedSemanticAST::Raw {
                        content: "[2, 2]".to_string(),
                    },
                },
            ],
        );

        let report = AnnotationValidator::default().validate_annotations(&annotations);
        assert!(
            !report
                .diagnostics
                .iter()
                .any(|d| d.code == "W_SEM_UNSATISFIABLE_VALUE_DOMAIN")
        );
    }

    #[test]
    fn grammar_aware_validation_warns_on_ambiguous_literal_prefix() {
        let annotations = Annotations::default();
        let grammar_tree = HashMap::from([(
            "statement".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    ASTNode::Sequence {
                        elements: vec![
                            ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("quoted_string".to_string()),
                                    TokenValue::String("if".to_string()),
                                ]),
                            },
                            ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("rule_reference".to_string()),
                                    TokenValue::String("expr".to_string()),
                                ]),
                            },
                        ],
                    },
                    ASTNode::Sequence {
                        elements: vec![
                            ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("quoted_string".to_string()),
                                    TokenValue::String("if".to_string()),
                                ]),
                            },
                            ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("rule_reference".to_string()),
                                    TokenValue::String("stmt".to_string()),
                                ]),
                            },
                        ],
                    },
                ],
            },
        )]);

        let report = AnnotationValidator::default()
            .validate_annotations_with_grammar(&annotations, &grammar_tree);

        assert!(report.diagnostics.iter().any(|d| {
            d.code == "W_GRAM_AMBIGUOUS_PREFIX"
                && d.kind == AnnotationKind::Grammar
                && d.rule_name == "statement"
        }));
    }

    #[test]
    fn grammar_aware_validation_does_not_warn_on_distinct_literal_prefixes() {
        let annotations = Annotations::default();
        let grammar_tree = HashMap::from([(
            "statement".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    ASTNode::Sequence {
                        elements: vec![
                            ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("quoted_string".to_string()),
                                    TokenValue::String("if".to_string()),
                                ]),
                            },
                            ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("rule_reference".to_string()),
                                    TokenValue::String("expr".to_string()),
                                ]),
                            },
                        ],
                    },
                    ASTNode::Sequence {
                        elements: vec![
                            ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("quoted_string".to_string()),
                                    TokenValue::String("while".to_string()),
                                ]),
                            },
                            ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("rule_reference".to_string()),
                                    TokenValue::String("expr".to_string()),
                                ]),
                            },
                        ],
                    },
                ],
            },
        )]);

        let report = AnnotationValidator::default()
            .validate_annotations_with_grammar(&annotations, &grammar_tree);

        assert!(
            !report
                .diagnostics
                .iter()
                .any(|d| d.code == "W_GRAM_AMBIGUOUS_PREFIX")
        );
    }

    #[test]
    fn grammar_aware_validation_warns_on_first_set_overlap_from_nullable_prefix() {
        let annotations = Annotations::default();
        let grammar_tree = HashMap::from([
            (
                "prefix".to_string(),
                ASTNode::Quantified {
                    element: Box::new(ASTNode::Atom {
                        value: ASTValue::Token(vec![
                            TokenValue::String("quoted_string".to_string()),
                            TokenValue::String("a".to_string()),
                        ]),
                    }),
                    quantifier: "?".to_string(),
                },
            ),
            (
                "statement".to_string(),
                ASTNode::Or {
                    alternatives: vec![
                        ASTNode::Sequence {
                            elements: vec![
                                ASTNode::Atom {
                                    value: ASTValue::Token(vec![
                                        TokenValue::String("rule_reference".to_string()),
                                        TokenValue::String("prefix".to_string()),
                                    ]),
                                },
                                ASTNode::Atom {
                                    value: ASTValue::Token(vec![
                                        TokenValue::String("quoted_string".to_string()),
                                        TokenValue::String("if".to_string()),
                                    ]),
                                },
                            ],
                        },
                        ASTNode::Sequence {
                            elements: vec![
                                ASTNode::Atom {
                                    value: ASTValue::Token(vec![
                                        TokenValue::String("quoted_string".to_string()),
                                        TokenValue::String("if".to_string()),
                                    ]),
                                },
                                ASTNode::Atom {
                                    value: ASTValue::Token(vec![
                                        TokenValue::String("rule_reference".to_string()),
                                        TokenValue::String("expr".to_string()),
                                    ]),
                                },
                            ],
                        },
                    ],
                },
            ),
        ]);

        let report = AnnotationValidator::default()
            .validate_annotations_with_grammar(&annotations, &grammar_tree);

        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.code == "W_GRAM_FIRST_SET_OVERLAP"
                    && d.kind == AnnotationKind::Grammar
                    && d.rule_name == "statement")
        );
    }

    #[test]
    fn grammar_aware_validation_warns_on_nullable_branch_shadow() {
        let annotations = Annotations::default();
        let grammar_tree = HashMap::from([(
            "statement".to_string(),
            ASTNode::Or {
                alternatives: vec![
                    ASTNode::Quantified {
                        element: Box::new(ASTNode::Atom {
                            value: ASTValue::Token(vec![
                                TokenValue::String("quoted_string".to_string()),
                                TokenValue::String("if".to_string()),
                            ]),
                        }),
                        quantifier: "?".to_string(),
                    },
                    ASTNode::Sequence {
                        elements: vec![
                            ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("quoted_string".to_string()),
                                    TokenValue::String("while".to_string()),
                                ]),
                            },
                            ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("rule_reference".to_string()),
                                    TokenValue::String("expr".to_string()),
                                ]),
                            },
                        ],
                    },
                ],
            },
        )]);

        let report = AnnotationValidator::default()
            .validate_annotations_with_grammar(&annotations, &grammar_tree);

        assert!(report.diagnostics.iter().any(|d| {
            d.code == "W_GRAM_NULLABLE_BRANCH_SHADOW"
                && d.kind == AnnotationKind::Grammar
                && d.rule_name == "statement"
        }));
    }
}
