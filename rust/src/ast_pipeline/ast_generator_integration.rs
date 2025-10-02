// Integration module for AST-based generator
// Direct integration with the pipeline - no adapter needed

use crate::ast_pipeline::{
    ASTNode, Annotations, TransformedASTJson,
    ast_based_generator::{AstBasedGenerator, BranchAnnotation},
};
use std::collections::HashMap;
use anyhow::{Result, Context};

/// Configuration for the AST generator
#[derive(Debug, Clone)]
pub struct AstGeneratorConfig {
    /// Enable debug output during generation
    pub debug: bool,
}

impl Default for AstGeneratorConfig {
    fn default() -> Self {
        Self {
            debug: false,
        }
    }
}

/// Main integration point for AST-based parser generation
pub struct AstGeneratorIntegration {
    config: AstGeneratorConfig,
}

impl AstGeneratorIntegration {
    /// Create a new integration instance with default config
    pub fn new() -> Self {
        Self {
            config: AstGeneratorConfig::default(),
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(config: AstGeneratorConfig) -> Self {
        Self { config }
    }
    
    /// Generate parser from transformed AST using AST-based generator
    pub fn generate_parser(
        &self,
        transformed_ast: &TransformedASTJson,
        annotations: Option<&Annotations>,
    ) -> Result<String> {
        let parser_name = self.snake_to_pascal(&transformed_ast.grammar_name);
        
        if self.config.debug {
            eprintln!(
                "[ast_generator_integration] Generating parser '{}' with AST-based generator",
                parser_name
            );
        }
        
        // Direct use of AST-based generator
        generate_parser_ast_based(
            &transformed_ast.grammar_name,
            &transformed_ast.grammar_tree,
            &transformed_ast.rule_order,
            annotations,
        ).context("Failed to generate parser")
    }
    
    /// Select the appropriate backend based on grammar complexity
    fn select_backend(&self, grammar: &HashMap<String, ASTNode>) -> GeneratorBackend {
        if self.config.force_ast_generator {
            return GeneratorBackend::AstBased;
        }
        
        // Calculate complexity metrics
        let rule_count = grammar.len();
        let max_depth = self.calculate_max_depth(grammar);
        let avg_alternatives = self.calculate_avg_alternatives(grammar);
        
        // Decision logic
        if rule_count >= self.config.complexity_threshold {
            GeneratorBackend::AstBased
        } else if max_depth > 3 {
            GeneratorBackend::AstBased
        } else if avg_alternatives > 5.0 {
            GeneratorBackend::AstBased
        } else {
            GeneratorBackend::StringBased
        }
    }
    
    /// Calculate maximum nesting depth in the grammar
    fn calculate_max_depth(&self, grammar: &HashMap<String, ASTNode>) -> usize {
        grammar.values()
            .map(|node| self.node_depth(node))
            .max()
            .unwrap_or(0)
    }
    
    /// Calculate depth of a single AST node
    fn node_depth(&self, node: &ASTNode) -> usize {
        match node {
            ASTNode::Atom { .. } => 1,
            ASTNode::Sequence { elements } => {
                1 + elements.iter().map(|e| self.node_depth(e)).max().unwrap_or(0)
            }
            ASTNode::Or { alternatives } => {
                1 + alternatives.iter().map(|a| self.node_depth(a)).max().unwrap_or(0)
            }
            ASTNode::Quantified { element, .. } => 1 + self.node_depth(element),
        }
    }
    
    /// Calculate average number of alternatives per Or node
    fn calculate_avg_alternatives(&self, grammar: &HashMap<String, ASTNode>) -> f64 {
        let mut total_alternatives = 0;
        let mut or_node_count = 0;
        
        for node in grammar.values() {
            self.count_alternatives(node, &mut total_alternatives, &mut or_node_count);
        }
        
        if or_node_count > 0 {
            total_alternatives as f64 / or_node_count as f64
        } else {
            0.0
        }
    }
    
    /// Count alternatives in Or nodes
    fn count_alternatives(
        &self,
        node: &ASTNode,
        total: &mut usize,
        count: &mut usize,
    ) {
        match node {
            ASTNode::Or { alternatives } => {
                *total += alternatives.len();
                *count += 1;
                for alt in alternatives {
                    self.count_alternatives(alt, total, count);
                }
            }
            ASTNode::Sequence { elements } => {
                for elem in elements {
                    self.count_alternatives(elem, total, count);
                }
            }
            ASTNode::Quantified { element, .. } => {
                self.count_alternatives(element, total, count);
            }
            ASTNode::Atom { .. } => {}
        }
    }
    
    /// Convert snake_case to PascalCase
    fn snake_to_pascal(&self, name: &str) -> String {
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
}

/// Direct AST-based generator replacement for high_performance_generator
/// This can be used to directly replace the existing generator
pub fn generate_parser_ast_based(
    grammar_name: &str,
    grammar: &HashMap<String, ASTNode>,
    rule_order: &[String],
    annotations: Option<&Annotations>,
) -> Result<String> {
    let parser_name = grammar_name
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect();
    
    let mut generator = AstBasedGenerator::new(parser_name);
    
    // Transfer annotations if provided
    if let Some(annotations) = annotations {
        generator.semantic_annotations = annotations.semantic_annotations.clone();
        generator.logging_annotations = annotations.logging_annotations.clone();
        generator.branch_return_annotations = annotations.branch_return_annotations
            .iter()
            .map(|(rule, branches)| {
                let converted_branches = branches
                    .iter()
                    .map(|opt_annotation| {
                        opt_annotation.as_ref().map(|ann| {
                            ast_based_generator::BranchAnnotation {
                                annotation_type: ann.annotation_type.clone(),
                                annotation_content: ann.annotation_content.clone(),
                                parsed_ast: ann.parsed_ast.clone(),
                            }
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

/// Builder pattern for configuring AST generator integration
pub struct AstGeneratorIntegrationBuilder {
    config: AstGeneratorConfig,
}

impl AstGeneratorIntegrationBuilder {
    pub fn new() -> Self {
        Self {
            config: AstGeneratorConfig::default(),
        }
    }
    
    pub fn force_ast_generator(mut self, force: bool) -> Self {
        self.config.force_ast_generator = force;
        self
    }
    
    pub fn enable_fallback(mut self, enable: bool) -> Self {
        self.config.enable_fallback = enable;
        self
    }
    
    pub fn complexity_threshold(mut self, threshold: usize) -> Self {
        self.config.complexity_threshold = threshold;
        self
    }
    
    pub fn debug_selection(mut self, debug: bool) -> Self {
        self.config.debug_selection = debug;
        self
    }
    
    pub fn build(self) -> AstGeneratorIntegration {
        AstGeneratorIntegration::with_config(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast_pipeline::ASTValue;
    
    #[test]
    fn test_backend_selection() {
        let integration = AstGeneratorIntegration::new();
        
        // Simple grammar - should use string-based
        let mut simple_grammar = HashMap::new();
        simple_grammar.insert(
            "rule1".to_string(),
            ASTNode::Atom {
                value: ASTValue::Token(vec!["test".to_string()]),
            },
        );
        
        assert_eq!(
            integration.select_backend(&simple_grammar),
            GeneratorBackend::StringBased
        );
        
        // Complex grammar - should use AST-based
        let mut complex_grammar = HashMap::new();
        for i in 0..15 {
            complex_grammar.insert(
                format!("rule{}", i),
                ASTNode::Atom {
                    value: ASTValue::Token(vec!["test".to_string()]),
                },
            );
        }
        
        assert_eq!(
            integration.select_backend(&complex_grammar),
            GeneratorBackend::AstBased
        );
    }
    
    #[test]
    fn test_forced_ast_generator() {
        let config = AstGeneratorConfig {
            force_ast_generator: true,
            ..Default::default()
        };
        
        let integration = AstGeneratorIntegration::with_config(config);
        
        // Even simple grammar should use AST-based when forced
        let mut simple_grammar = HashMap::new();
        simple_grammar.insert(
            "rule1".to_string(),
            ASTNode::Atom {
                value: ASTValue::Token(vec!["test".to_string()]),
            },
        );
        
        assert_eq!(
            integration.select_backend(&simple_grammar),
            GeneratorBackend::AstBased
        );
    }
}