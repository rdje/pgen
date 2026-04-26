// Direct AST-based generator integration
// No adapter layer needed - string-based generator has been removed

use crate::ast_pipeline::{
    ASTNode, Annotations, BranchAnnotation, TransformedASTJson, UnknownSemanticDirectivePolicy,
    annotation_validator::{AnnotationSeverity, AnnotationValidator, AnnotationValidatorConfig},
    ast_based_generator::AstBasedGenerator,
};
use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};

/// Direct integration point for AST-based parser generation
pub struct AstGeneratorIntegration {
    debug: bool,
}

impl AstGeneratorIntegration {
    /// Create a new integration instance
    pub fn new() -> Self {
        Self { debug: false }
    }

    /// Enable debug output
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Generate parser from transformed AST using AST-based generator
    pub fn generate_parser(&self, transformed_ast: &TransformedASTJson) -> Result<String> {
        if self.debug {
            crate::pgen_trace_high!(
                "[ast_generator] Generating parser '{}' with {} rules",
                transformed_ast.grammar_name,
                transformed_ast.grammar_tree.len()
            );
        }

        generate_parser_ast_based(
            &transformed_ast.grammar_name,
            &transformed_ast.grammar_tree,
            &transformed_ast.rule_order,
            transformed_ast.metadata.annotations.as_ref(),
            &format!("{}_parser.rs", transformed_ast.grammar_name),
            // Phase 2 M1: this internal generator entry path keeps the legacy emit
            // shape; --inline-annotations is only routed through the CLI binary path.
            false,
        )
    }
}

/// Direct function to generate parser using AST-based approach.
///
/// `inline_annotations` is the Phase 2 M1 toggle. When false (default), the
/// generator emits the existing parser shape unchanged. When true, the generator
/// additionally emits `parse_full_<entry>_typed` returning
/// `ParseResult<serde_json::Value>`. M1's typed method is a skeleton wrapper
/// (parse + `serde_json::to_value`); M2 replaces the body with truly inline
/// shape-emit logic per the rule's return annotation.
pub fn generate_parser_ast_based(
    grammar_name: &str,
    grammar: &HashMap<String, ASTNode>,
    rule_order: &[String],
    annotations: Option<&Annotations>,
    filename: &str,
    inline_annotations: bool,
) -> Result<String> {
    let parser_name = snake_to_pascal(grammar_name);
    let mut generator = AstBasedGenerator::new(parser_name);
    generator.inline_annotations = inline_annotations;

    // Transfer annotations if provided
    if let Some(annotations) = annotations {
        let strict_validation = strict_annotation_validation_enabled();

        let validator = AnnotationValidator::new(AnnotationValidatorConfig {
            max_capture_index: None,
            strict_semantic_transforms: strict_validation,
            unknown_semantic_directive_policy: unknown_semantic_directive_policy(),
            strict_semantic_warning_codes: strict_semantic_warning_codes(strict_validation),
        });
        let validation_report = validator.validate_annotations_with_grammar(annotations, grammar);

        for diagnostic in &validation_report.diagnostics {
            crate::pgen_trace_low!(
                "[annotation-validator][{}][{}][{}][rule='{}'][annotation={}] {}",
                diagnostic.severity.as_str(),
                diagnostic.code,
                diagnostic.kind.as_str(),
                diagnostic.rule_name,
                diagnostic
                    .annotation_index
                    .map(|i| i.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                diagnostic.message
            );

            if let Some(annotation_text) = &diagnostic.annotation {
                crate::pgen_trace_low!("  -> {}", annotation_text);
            }
        }

        if strict_validation && validation_report.has_errors() {
            return Err(anyhow::anyhow!(
                "Annotation validation failed: {} error(s), {} warning(s). Strict mode is enabled by CI default or PGEN_STRICT_ANNOTATION_VALIDATION=1; set PGEN_STRICT_ANNOTATION_VALIDATION=0 to keep non-blocking diagnostics.",
                validation_report.error_count(),
                validation_report.warning_count()
            ));
        } else if !validation_report.diagnostics.is_empty() {
            let error_count = validation_report
                .diagnostics
                .iter()
                .filter(|d| d.severity == AnnotationSeverity::Error)
                .count();
            let warning_count = validation_report
                .diagnostics
                .iter()
                .filter(|d| d.severity == AnnotationSeverity::Warning)
                .count();
            crate::pgen_trace_medium!(
                "[annotation-validator] Completed with {} error(s), {} warning(s).",
                error_count,
                warning_count
            );
        }

        // The AST generator stores annotations as Option<Annotations>
        generator.annotations = Some(annotations.clone());
        generator.branch_return_annotations = annotations
            .branch_return_annotations
            .iter()
            .map(|(rule, branches)| {
                let converted_branches = branches
                    .iter()
                    .map(|opt_annotation| {
                        opt_annotation.as_ref().map(|ann| BranchAnnotation {
                            annotation_type: ann.annotation_type.clone(),
                            annotation_content: ann.annotation_content.clone(),
                            parsed_ast: ann.parsed_ast.clone(),
                        })
                    })
                    .collect();
                (rule.clone(), converted_branches)
            })
            .collect();
    }

    generator
        .generate_parser(grammar, rule_order, filename)
        .context("Failed to generate parser using AST-based generator")
}

fn strict_annotation_validation_enabled() -> bool {
    if let Some(explicit) = parse_bool_env("PGEN_STRICT_ANNOTATION_VALIDATION") {
        return explicit;
    }

    // In CI, strict annotation validation is the default unless CI is explicitly false-ish.
    parse_bool_env("CI").unwrap_or(false)
}

fn unknown_semantic_directive_policy() -> UnknownSemanticDirectivePolicy {
    std::env::var("PGEN_UNKNOWN_SEMANTIC_DIRECTIVE_POLICY")
        .ok()
        .and_then(|raw| UnknownSemanticDirectivePolicy::parse(&raw))
        .unwrap_or_default()
}

fn strict_semantic_warning_codes(strict_validation: bool) -> HashSet<String> {
    if let Ok(raw) = std::env::var("PGEN_STRICT_SEMANTIC_WARNING_CODES") {
        let mut codes = HashSet::new();
        for token in raw.split(',') {
            let normalized = token.trim().to_ascii_uppercase();
            if normalized.is_empty() || normalized == "NONE" {
                continue;
            }
            if normalized == "ALL" {
                codes.insert("*".to_string());
                continue;
            }
            codes.insert(normalized);
        }
        return codes;
    }

    if strict_validation {
        return HashSet::from([
            "W_SEM_INVALID_COVERAGE_TARGET_PAYLOAD".to_string(),
            "W_SEM_INVALID_CRITICAL_PATH_PAYLOAD".to_string(),
        ]);
    }

    HashSet::new()
}

fn parse_bool_env(var_name: &str) -> Option<bool> {
    std::env::var(var_name).ok().map(|value| {
        let normalized = value.trim().to_ascii_lowercase();
        !(normalized.is_empty()
            || normalized == "0"
            || normalized == "false"
            || normalized == "no"
            || normalized == "off")
    })
}

/// Convert snake_case to PascalCase
fn snake_to_pascal(name: &str) -> String {
    name.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}
