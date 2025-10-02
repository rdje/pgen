// Unified generator adapter - AST-based only
// String-based generator removed due to fundamental flaws with delimiter balancing

use crate::ast_pipeline::{
    ASTNode, Annotations,
    ast_based_generator::{AstBasedGenerator, BranchAnnotation},
};
use std::collections::HashMap;
use anyhow::{Result, Context};

/// Generator using only AST-based backend
/// String-based generation has been completely removed
pub struct UnifiedGenerator {
    parser_name: String,
}

impl UnifiedGenerator {
    /// Create a new generator (AST-based only)
    pub fn new(parser_name: String) -> Self {
        Self { parser_name }
    }
    
    /// Generate parser using AST-based generator
    pub fn generate_parser(
        &self,
        grammar: &HashMap<String, ASTNode>,
        rule_order: &[String],
        annotations: Option<&Annotations>,
    ) -> Result<String> {
        let mut generator = AstBasedGenerator::new(self.parser_name.clone());
        
        // Transfer annotations if provided
        if let Some(annotations) = annotations {
            // Copy semantic annotations
            generator.semantic_annotations = annotations.semantic_annotations.clone();
            
            // Copy logging annotations
            generator.logging_annotations = annotations.logging_annotations.clone();
            
            // Convert and copy branch return annotations
            generator.branch_return_annotations = annotations.branch_return_annotations
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
        
        generator.generate_parser(grammar, rule_order)
            .context("Failed to generate parser using AST-based generator")
    }
}

/// Direct function to generate parser using AST-based approach
pub fn generate_parser_ast(
    parser_name: &str,
    grammar: &HashMap<String, ASTNode>,
    rule_order: &[String],
    annotations: Option<&Annotations>,
) -> Result<String> {
    let generator = UnifiedGenerator::new(parser_name.to_string());
    generator.generate_parser(grammar, rule_order, annotations)
}