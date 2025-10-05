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
