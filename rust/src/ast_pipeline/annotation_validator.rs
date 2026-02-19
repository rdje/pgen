use super::{
    ASTNode, Annotations, BranchAnnotation, ExtractionTarget, UnifiedReturnAST, UnifiedSemanticAST,
    parse_canonical_transform_expression,
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
}

impl AnnotationKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnnotationKind::Return => "return",
            AnnotationKind::Semantic => "semantic",
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
}

#[derive(Debug, Clone, Default)]
pub struct AnnotationValidator {
    config: AnnotationValidatorConfig,
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
            for (idx, annotation_ast) in semantic_annotations.iter().enumerate() {
                self.validate_semantic_annotation(rule_name, idx + 1, annotation_ast, &mut report);
            }
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
                self.validate_return_ast(rule_name, annotation_index, index, raw_annotation, report);
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
        semantic_ast: &UnifiedSemanticAST,
        report: &mut AnnotationValidationReport,
    ) {
        match semantic_ast {
            UnifiedSemanticAST::TransformExpr { expression } => {
                self.validate_transform_expression(
                    rule_name,
                    annotation_index,
                    expression,
                    report,
                );
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
                message: "Transform expression has empty parse target type or default expression.".to_string(),
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
            "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128",
            "usize",
        ]
        .iter()
        .copied()
        .collect();
        let float_types: HashSet<&str> = ["f32", "f64"].iter().copied().collect();

        let integer_default_re = Regex::new(r"^-?[0-9](?:[0-9_]*)(?:i8|i16|i32|i64|i128|isize|u8|u16|u32|u64|u128|usize)?$")
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
            UnifiedReturnAST::ArrayAccess { base, index } => {
                self.max_positional_ref(base).max(self.max_positional_ref(index))
            }
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
        assert!(report
            .diagnostics
            .iter()
            .any(|d| d.code == "E_RET_POS_ZERO"));
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
        assert!(report
            .diagnostics
            .iter()
            .any(|d| d.code == "W_RET_UNPARSED" && d.severity == AnnotationSeverity::Warning));
    }

    #[test]
    fn semantic_validator_accepts_canonical_transform() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "number".to_string(),
            vec![UnifiedSemanticAST::TransformExpr {
                expression: "str::parse::<u32>().unwrap_or(0)".to_string(),
            }],
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
            vec![UnifiedSemanticAST::TransformExpr {
                expression: "parse(value)".to_string(),
            }],
        );

        let report = AnnotationValidator::new(AnnotationValidatorConfig {
            max_capture_index: None,
            strict_semantic_transforms: true,
        })
        .validate_annotations(&annotations);

        assert!(report.has_errors());
        assert!(report
            .diagnostics
            .iter()
            .any(|d| d.code == "W_SEM_NON_CANONICAL_TRANSFORM"
                && d.severity == AnnotationSeverity::Error));
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
        })
        .validate_annotations(&annotations);

        assert!(report.has_errors());
        assert!(report
            .diagnostics
            .iter()
            .any(|d| d.code == "E_RET_POS_OUT_OF_RANGE"));
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
        assert!(report
            .diagnostics
            .iter()
            .any(|d| d.code == "W_RET_POS_RULE_BOUND"));
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
        assert!(report
            .diagnostics
            .iter()
            .any(|d| d.code == "W_RET_BRANCH_NOT_SEQUENCE"));
    }
}
