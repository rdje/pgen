//! EBNF AST Transformation Pipeline Library
//!
//! This library provides AST transformation capabilities for EBNF grammars,
//! including semantic annotation parsing and high-performance parser generation.

pub mod ast_pipeline;
pub mod return_parser_stress_test;
pub mod semantic_annotation_stress_test;
pub mod regex_stress_test;
pub mod individual_tests;
pub mod test_target_mapper;
pub mod test_reproduction_demo;

// New automation modules
pub mod test_registry;
pub mod test_discovery;
pub mod makefile_generator;
pub mod individual_tests_generator;
pub mod test_automation;

// Re-export commonly used types for convenience
pub use ast_pipeline::{
    RustASTPipeline, 
    PipelineConfig, 
    ASTNode, 
    ASTValue, 
    TokenValue
};
