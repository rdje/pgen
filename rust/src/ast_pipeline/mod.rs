// Shared Logger trait that both binaries can access
pub trait Logger {
    fn is_enabled(&self) -> bool;
    fn log_info(&self, file: &str, line: u32, message: &str);
    fn log_debug(&self, file: &str, line: u32, message: &str);
    fn log_success(&self, file: &str, line: u32, message: &str);
    fn log_warning(&self, file: &str, line: u32, message: &str);
    fn log_error(&self, file: &str, line: u32, message: &str);
}

// No-op logger implementation
pub struct NoOpLogger;

impl Logger for NoOpLogger {
    fn is_enabled(&self) -> bool { false }
    fn log_info(&self, _file: &str, _line: u32, _message: &str) {}
    fn log_debug(&self, _file: &str, _line: u32, _message: &str) {}
    fn log_success(&self, _file: &str, _line: u32, _message: &str) {}
    fn log_warning(&self, _file: &str, _line: u32, _message: &str) {}
    fn log_error(&self, _file: &str, _line: u32, _message: &str) {}
}

use serde;
use std::collections::HashMap;
use anyhow::Result;
#[derive(Debug, Clone, serde::Deserialize)]
pub enum ASTValue {
    Token(Vec<TokenValue>),
    Node(Box<ASTNode>),
}

#[derive(Debug, Clone, serde::Deserialize)]
pub enum TokenValue {
    String(String),
}

#[derive(Debug, Clone, serde::Deserialize)]
pub enum ASTNode {
    Or { alternatives: Vec<ASTNode> },
    Sequence { elements: Vec<ASTNode> },
    Atom { value: ASTValue },
    Quantified { element: Box<ASTNode>, quantifier: String },
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct BranchAnnotation {
    pub annotation_type: String,
    pub annotation_content: String,
    pub parsed_ast: Option<UnifiedReturnAST>,
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct Annotations {
    #[serde(default)]
    pub branch_return_annotations: std::collections::HashMap<String, Vec<Option<BranchAnnotation>>>,
    #[serde(default)]
    pub semantic_annotations: std::collections::HashMap<String, Vec<UnifiedSemanticAST>>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TransformMetadata {
    pub format: String,
    pub source_format: String,
    pub transformed_at: String,
    pub transformer: String,
    pub pipeline_stage: String,
    pub annotations: Option<Annotations>,
    #[serde(default)]
    pub stats: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TransformedASTJson {
    pub grammar_name: String,
    pub grammar_tree: std::collections::HashMap<String, ASTNode>,
    pub rule_order: Vec<String>,
    pub metadata: TransformMetadata,
}

// Type aliases for compatibility
pub type ParseNode<'input> = ASTNode;

pub struct PipelineConfig {
    pub debug: bool,
    pub trace: bool,
    pub bootstrap_mode: bool,
    pub preserve_annotations: bool,
    pub validate_input: bool,
    pub validate_output: bool,
    pub max_recursion_depth: usize,
    pub eliminate_left_recursion: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        PipelineConfig {
            debug: false,
            trace: false,
            bootstrap_mode: false,
            preserve_annotations: true,
            validate_input: true,
            validate_output: true,
            max_recursion_depth: 100,
            eliminate_left_recursion: true,
        }
    }
}

pub struct RustASTPipeline;

impl RustASTPipeline {
    pub fn new(_config: PipelineConfig) -> Self {
        RustASTPipeline
    }

    /// Transform raw AST JSON into processed AST format
    pub fn transform_from_raw_ast(&self, raw_ast_data: &[serde_json::Value]) -> Result<(HashMap<String, ASTNode>, Vec<String>)> {
        let mut grammar_tree = HashMap::new();
        let mut rule_order = Vec::new();

        for rule_data in raw_ast_data {
            if let Some(rule_array) = rule_data.as_array() {
                if rule_array.is_empty() { continue; }

                // First element should be ["rule", "rule_name"]
                if let Some(first_elem) = rule_array.first() {
                    if let Some(rule_name) = self.extract_rule_name(first_elem) {
                        rule_order.push(rule_name.clone());

                        // Parse the rule content (everything after the rule declaration)
                        let rule_content = &rule_array[1..];
                        let ast_node = self.parse_rule_content(rule_content)?;

                        grammar_tree.insert(rule_name, ast_node);
                    }
                }
            }
        }

        Ok((grammar_tree, rule_order))
    }

    fn extract_rule_name(&self, rule_decl: &serde_json::Value) -> Option<String> {
        if let Some(arr) = rule_decl.as_array() {
            if arr.len() >= 2 {
                if let (Some(type_str), Some(name_str)) = (arr[0].as_str(), arr[1].as_str()) {
                    if type_str == "rule" {
                        return Some(name_str.to_string());
                    }
                }
            }
        }
        None
    }

    fn parse_rule_content(&self, content: &[serde_json::Value]) -> Result<ASTNode> {
        if content.is_empty() {
            return Ok(ASTNode::Sequence { elements: vec![] });
        }

        // For now, treat as a sequence - this is a simplified implementation
        // In a full implementation, this would handle alternatives (|), quantifiers (*, +, ?), etc.
        let mut elements = Vec::new();

        for item in content {
            if let Some(ast_node) = self.parse_single_element(item)? {
                elements.push(ast_node);
            }
        }

        if elements.len() == 1 {
            Ok(elements.into_iter().next().unwrap())
        } else {
            Ok(ASTNode::Sequence { elements })
        }
    }

    fn parse_single_element(&self, element: &serde_json::Value) -> Result<Option<ASTNode>> {
        if let Some(arr) = element.as_array() {
            if arr.len() >= 2 {
                if let (Some(elem_type), Some(elem_value)) = (arr[0].as_str(), arr[1].as_str()) {
                    match elem_type {
                        "rule_reference" => {
                            Ok(Some(ASTNode::Atom {
                                value: ASTValue::Node(Box::new(ASTNode::Atom {
                                    value: ASTValue::Token(vec![
                                        TokenValue::String("rule_reference".to_string()),
                                        TokenValue::String(elem_value.to_string()),
                                    ])
                                }))
                            }))
                        }
                        "quoted_string" => {
                            Ok(Some(ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("quoted_string".to_string()),
                                    TokenValue::String(elem_value.to_string()),
                                ])
                            }))
                        }
                        "operator" => {
                            // Handle quantifiers
                            match elem_value {
                                "?" => Ok(Some(ASTNode::Quantified {
                                    element: Box::new(ASTNode::Sequence { elements: vec![] }), // Placeholder
                                    quantifier: "?".to_string(),
                                })),
                                "*" => Ok(Some(ASTNode::Quantified {
                                    element: Box::new(ASTNode::Sequence { elements: vec![] }), // Placeholder
                                    quantifier: "*".to_string(),
                                })),
                                "+" => Ok(Some(ASTNode::Quantified {
                                    element: Box::new(ASTNode::Sequence { elements: vec![] }), // Placeholder
                                    quantifier: "+".to_string(),
                                })),
                                _ => Ok(None) // Skip unknown operators
                            }
                        }
                        "return_scalar" | "return_array" | "return_object" => {
                            // Skip return annotations for now
                            Ok(None)
                        }
                        _ => Ok(None) // Skip unknown element types
                    }
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

pub mod ast_based_generator;
pub mod ast_code_generator;
pub mod ast_generator_direct;
pub mod ast_return_transform;
pub mod grouped_quantifier_parser;
pub mod mutual_recursion_handler;
pub mod return_annotation_handler;
pub mod unified_return_ast;
pub mod unified_semantic_ast;

// Re-export key types
pub use unified_return_ast::{UnifiedReturnAST, ExtractionTarget};
pub use unified_semantic_ast::UnifiedSemanticAST;
