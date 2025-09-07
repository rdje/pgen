//! EBNF AST Transformation Pipeline Library
//!
//! This library provides AST transformation capabilities for EBNF grammars,
//! including semantic annotation parsing and high-performance parser generation.

pub mod ast_pipeline;
pub mod comprehensive_stress_test;
pub mod return_parser_stress_test;

// Re-export commonly used types for convenience
pub use ast_pipeline::{
    RustASTPipeline, 
    PipelineConfig, 
    ASTNode, 
    ASTValue, 
    TokenValue
};
