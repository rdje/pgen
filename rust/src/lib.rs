//! EBNF AST Transformation Pipeline Library
//!
//! This library provides AST transformation capabilities for EBNF grammars,
//! including semantic annotation parsing and high-performance parser generation.

pub mod ast_pipeline;
pub mod test_registry;
pub mod test_runner; // Only declare once

// New automation modules
pub mod test_discovery;

// Re-export Logger trait for generated parsers
pub use ast_pipeline::Logger;
pub use ast_pipeline::NoOpLogger;

// Generated parsers from EBNF grammars
#[cfg(feature = "generated_parsers")]
pub mod generated_parsers {
    pub mod return_annotation {
        include!("../../generated/return_annotation_parser.rs");
    }
    pub mod semantic_annotation {
        include!("../../generated/semantic_annotation_parser.rs");
    }
}
