//! EBNF AST Transformation Pipeline Library
//!
//! This library provides AST transformation capabilities for EBNF grammars,
//! including semantic annotation parsing and high-performance parser generation.

pub mod ast_pipeline;

// Re-export commonly used types for convenience
pub use ast_pipeline::{
    RustASTPipeline, 
    PipelineConfig, 
    ASTNode, 
    ASTValue, 
    TokenValue
};
