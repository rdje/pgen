//! Unified Semantic Annotation AST
//!
//! This module provides a single, consistent AST representation for semantic annotations
//! that is used throughout the pipeline:
//! 1. Parsed from text by the external parser or bootstrap parser
//! 2. Pretty-printed for debugging
//! 3. Used directly by the code generator to emit Rust code
//!
//! This eliminates the need for multiple parallel AST representations and parsers.

use super::{Logger, ParseContent, ParseNode};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;

/// The unified AST representation of a semantic annotation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UnifiedSemanticAST {
    /// Transform expression: str::parse::<f64>().unwrap_or(0.0)
    /// These map to transformation functions applied to matched terminals
    TransformExpr {
        expression: String, // The raw transform expression for now
    },

    /// Raw annotation that couldn't be parsed
    Raw { content: String },
}

impl UnifiedSemanticAST {
    /// Build semantic AST from a generated-parser parse tree.
    /// This is used by non-bootstrap generated parser paths.
    pub fn parse_generated_semantic_annotation<'input>(
        input: &'input str,
        parse_tree: &ParseNode<'input>,
        logger: &dyn Logger,
    ) -> Result<Self, String> {
        let root = if parse_tree.rule_name == "semantic_annotation" {
            parse_tree
        } else if let Some(node) = Self::find_first_rule_node(parse_tree, "semantic_annotation") {
            node
        } else {
            parse_tree
        };

        let name_node = Self::find_first_rule_node(root, "annotation_name")
            .ok_or_else(|| "Generated semantic parse tree missing annotation_name".to_string())?;
        let value_node = Self::find_first_rule_node(root, "annotation_value")
            .ok_or_else(|| "Generated semantic parse tree missing annotation_value".to_string())?;

        let name_text = Self::slice_span(input, &name_node.span)
            .ok_or_else(|| {
                format!(
                    "annotation_name span {}..{} out of bounds for input len {}",
                    name_node.span.start,
                    name_node.span.end,
                    input.len()
                )
            })?
            .trim()
            .to_ascii_lowercase();
        let value_text = Self::slice_span(input, &value_node.span)
            .ok_or_else(|| {
                format!(
                    "annotation_value span {}..{} out of bounds for input len {}",
                    value_node.span.start,
                    value_node.span.end,
                    input.len()
                )
            })?
            .trim()
            .to_string();

        if logger.is_enabled() {
            logger.log_debug(
                "unified_semantic_ast.rs",
                line!(),
                &format!(
                    "Generated semantic parse-tree conversion: name='{}' value='{}'",
                    name_text, value_text
                ),
            );
        }

        if name_text == "transform" {
            Ok(UnifiedSemanticAST::TransformExpr {
                expression: value_text,
            })
        } else {
            Ok(UnifiedSemanticAST::Raw {
                content: format!("@{}: {}", name_text, value_text),
            })
        }
    }

    /// Parse a semantic annotation in bootstrap mode (minimal support)
    pub fn parse_bootstrap(annotation_value: &str, logger: &dyn Logger) -> Result<Self, String> {
        let trimmed = annotation_value.trim();

        if logger.is_enabled() {
            logger.log_info(
                "unified_semantic_ast.rs",
                line!(),
                &format!(
                    "Parsing semantic annotation in bootstrap mode: '{}'",
                    trimmed
                ),
            );
        }

        // For now, we only support transform expressions as raw strings
        // TODO: Add proper AST parsing for function calls, type parameters, etc.
        if trimmed.contains("::parse::<") && trimmed.contains(">().unwrap_or(") {
            if logger.is_enabled() {
                logger.log_success(
                    "unified_semantic_ast.rs",
                    line!(),
                    "Recognized as transform expression",
                );
            }
            Ok(UnifiedSemanticAST::TransformExpr {
                expression: trimmed.to_string(),
            })
        } else {
            if logger.is_enabled() {
                logger.log_info(
                    "unified_semantic_ast.rs",
                    line!(),
                    "Unrecognized semantic annotation, storing as raw",
                );
            }
            Ok(UnifiedSemanticAST::Raw {
                content: trimmed.to_string(),
            })
        }
    }

    fn find_first_rule_node<'a, 'input>(
        node: &'a ParseNode<'input>,
        rule_name: &str,
    ) -> Option<&'a ParseNode<'input>> {
        if node.rule_name == rule_name {
            return Some(node);
        }

        match &node.content {
            ParseContent::Sequence(children) | ParseContent::Quantified(children, _) => {
                for child in children {
                    if let Some(found) = Self::find_first_rule_node(child, rule_name) {
                        return Some(found);
                    }
                }
            }
            ParseContent::Alternative(child) => {
                if let Some(found) = Self::find_first_rule_node(child, rule_name) {
                    return Some(found);
                }
            }
            ParseContent::Terminal(_) | ParseContent::TransformedTerminal(_) => {}
        }

        None
    }

    fn slice_span<'input>(
        input: &'input str,
        span: &std::ops::Range<usize>,
    ) -> Option<&'input str> {
        if span.start <= span.end && span.end <= input.len() {
            input.get(span.start..span.end)
        } else {
            None
        }
    }

    /// Pretty print the AST for debugging
    pub fn pretty_print(&self) -> String {
        match self {
            UnifiedSemanticAST::TransformExpr { expression } => {
                format!("TransformExpr({})", expression)
            }
            UnifiedSemanticAST::Raw { content } => {
                format!("Raw({})", content)
            }
        }
    }
}

impl fmt::Display for UnifiedSemanticAST {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.pretty_print())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_semantic_never_errors_and_falls_back_to_raw() {
        let logger = crate::test_runner::NoOpLogger;
        let parsed = UnifiedSemanticAST::parse_bootstrap("not a transform", &logger)
            .expect("bootstrap semantic parser should not error on unknown syntax");
        assert!(matches!(
            parsed,
            UnifiedSemanticAST::Raw { ref content } if content == "not a transform"
        ));
    }

    #[test]
    fn bootstrap_semantic_detects_transform_by_substring_markers() {
        let logger = crate::test_runner::NoOpLogger;
        let parsed =
            UnifiedSemanticAST::parse_bootstrap("str::parse::<u32>().unwrap_or(0)", &logger)
                .expect("bootstrap semantic parser should detect known transform marker pattern");
        assert!(matches!(parsed, UnifiedSemanticAST::TransformExpr { .. }));
    }

    #[test]
    fn bootstrap_semantic_detection_is_marker_based_not_structural() {
        let logger = crate::test_runner::NoOpLogger;
        let parsed =
            UnifiedSemanticAST::parse_bootstrap("x>().unwrap_or(0) ... ::parse::<u8>", &logger)
                .expect("bootstrap semantic parser only checks marker substrings");
        assert!(matches!(parsed, UnifiedSemanticAST::TransformExpr { .. }));
    }

    #[test]
    fn bootstrap_semantic_trims_outer_whitespace() {
        let logger = crate::test_runner::NoOpLogger;
        let parsed = UnifiedSemanticAST::parse_bootstrap("   value   ", &logger)
            .expect("bootstrap semantic parser should trim outer whitespace");
        assert!(matches!(
            parsed,
            UnifiedSemanticAST::Raw { ref content } if content == "value"
        ));
    }

    #[cfg(feature = "generated_parsers")]
    #[test]
    fn generated_semantic_tree_to_ast_supports_transform_and_named_raw() {
        use crate::generated_parsers::semantic_annotation::Semantic_annotationParser;

        let logger = crate::test_runner::NoOpLogger;
        let transform_input = "@transform: $1";
        let mut transform_parser =
            Semantic_annotationParser::new(transform_input, Box::new(crate::NoOpLogger));
        let transform_tree = transform_parser
            .parse_full_semantic_annotation()
            .expect("generated parser should parse transform sample");
        let transform_ast = UnifiedSemanticAST::parse_generated_semantic_annotation(
            transform_input,
            &transform_tree,
            &logger,
        )
        .expect("generated tree -> semantic ast should succeed for transform");
        assert!(matches!(
            transform_ast,
            UnifiedSemanticAST::TransformExpr { ref expression }
                if expression == "$1"
        ));

        let raw_input = "@priority: [9, 1]";
        let mut raw_parser = Semantic_annotationParser::new(raw_input, Box::new(crate::NoOpLogger));
        let raw_tree = raw_parser
            .parse_full_semantic_annotation()
            .expect("generated parser should parse non-transform sample");
        let raw_ast =
            UnifiedSemanticAST::parse_generated_semantic_annotation(raw_input, &raw_tree, &logger)
                .expect("generated tree -> semantic ast should succeed for named raw");
        assert!(matches!(
            raw_ast,
            UnifiedSemanticAST::Raw { ref content } if content == "@priority: [9, 1]"
        ));
    }
}
