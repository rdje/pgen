//! EBNF AST Transformation Pipeline Library
//!
//! This library provides AST transformation capabilities for EBNF grammars,
//! including semantic annotation parsing and high-performance parser generation.

pub mod ast_pipeline;
pub mod embedding_api;
#[cfg(feature = "ebnf_dual_run")]
pub mod ebnf_frontend;
pub mod test_registry;
pub mod test_runner; // Only declare once

// New automation modules
pub mod test_discovery;

// Re-export Logger trait for generated parsers
pub use ast_pipeline::Logger;
pub use ast_pipeline::NoOpLogger;

#[cfg(feature = "ebnf_dual_run")]
pub mod ebnf_generated_parser {
    include!("../../generated/ebnf.rs");
}

// Generated parsers from EBNF grammars
#[cfg(feature = "generated_parsers")]
pub mod generated_parsers {
    pub mod return_annotation {
        include!("../../generated/return_annotation_parser.rs");
        // Backward-compat alias for previously generated snake_case parser type.
        #[allow(non_camel_case_types)]
        pub type Return_annotationParser<'input> = ReturnAnnotationParser<'input>;
    }
    pub mod semantic_annotation {
        include!("../../generated/semantic_annotation_parser.rs");
    }
}

#[cfg(feature = "generated_parsers")]
pub mod parser_registry;
