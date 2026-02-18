//! Unified Semantic Annotation AST
//!
//! This module provides a single, consistent AST representation for semantic annotations
//! that is used throughout the pipeline:
//! 1. Parsed from text by the external parser or bootstrap parser
//! 2. Pretty-printed for debugging
//! 3. Used directly by the code generator to emit Rust code
//!
//! This eliminates the need for multiple parallel AST representations and parsers.

use super::Logger;
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
        let parsed = UnifiedSemanticAST::parse_bootstrap(
            "str::parse::<u32>().unwrap_or(0)",
            &logger,
        )
        .expect("bootstrap semantic parser should detect known transform marker pattern");
        assert!(matches!(parsed, UnifiedSemanticAST::TransformExpr { .. }));
    }

    #[test]
    fn bootstrap_semantic_detection_is_marker_based_not_structural() {
        let logger = crate::test_runner::NoOpLogger;
        let parsed = UnifiedSemanticAST::parse_bootstrap(
            "x>().unwrap_or(0) ... ::parse::<u8>",
            &logger,
        )
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
}
