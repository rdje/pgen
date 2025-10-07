// Direct AST-based generator integration
// No adapter layer needed - string-based generator has been removed

use crate::ast_pipeline::{
    ASTNode, Annotations, TransformedASTJson, BranchAnnotation,
    ast_based_generator::AstBasedGenerator,
};
use std::collections::HashMap;
use anyhow::{Result, Context};

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
    pub fn generate_parser(
        &self,
        transformed_ast: &TransformedASTJson,
    ) -> Result<String> {
        if self.debug {
            eprintln!(
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
        )
    }
}

/// Direct function to generate parser using AST-based approach
pub fn generate_parser_ast_based(
    grammar_name: &str,
    grammar: &HashMap<String, ASTNode>,
    rule_order: &[String],
    annotations: Option<&Annotations>,
    filename: &str,
) -> Result<String> {
    let parser_name = snake_to_pascal(grammar_name);
    let mut generator = AstBasedGenerator::new(parser_name);
    
    // Transfer annotations if provided
    if let Some(annotations) = annotations {
        // The AST generator stores annotations as Option<Annotations>
        generator.annotations = Some(annotations.clone());
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
    
    generator.generate_parser(grammar, rule_order, filename)
        .context("Failed to generate parser using AST-based generator")
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