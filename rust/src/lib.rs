//! EBNF AST Transformation Pipeline Library
//!
//! This library provides AST transformation capabilities for EBNF grammars,
//! including semantic annotation parsing and high-performance parser generation.

pub mod ast_pipeline;
// pub mod lexer; // Module file doesn't exist
pub mod test_runner;
pub mod test_registry; // Only declare once
pub mod universal_test_runner;
pub mod individual_tests;
pub mod test_target_mapper;
pub mod test_reproduction_demo;

// New automation modules
// pub mod test_discovery; // Module file doesn't exist
// pub mod makefile_generator; // Module file doesn't exist  
// pub mod individual_tests_generator; // Module file doesn't exist
// pub mod test_automation; // Depends on non-existent modules above

// Re-export commonly used types for convenience
pub use ast_pipeline::{
    RustASTPipeline, 
    PipelineConfig, 
    ASTNode, 
    ASTValue, 
    TokenValue
};
