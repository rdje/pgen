//! EBNF AST Transformation Pipeline Library
//!
//! This library provides AST transformation capabilities for EBNF grammars,
//! including semantic annotation parsing and high-performance parser generation.

pub mod ast_pipeline;
pub mod ast_shape_contract;
pub mod auto_return_annotation_shape_gate;
#[cfg(feature = "ebnf_dual_run")]
pub mod ebnf_frontend;
pub mod embedding_api;
/// Parser-specific hook implementations. Each module here implements
/// the [`ast_pipeline::ParserHooks`] contract for one grammar's
/// codegen-time extensibility needs. Code in this module is
/// PARSER-SPECIFIC by design — the pipeline itself stays parser-
/// agnostic; per-grammar behavior lives here and registers itself
/// with the pipeline through the public hook surface. New grammars
/// only need to add a sibling module if they have a real reason to
/// extend pipeline codegen; until then the pipeline's default emit
/// path runs unchanged for them.
pub mod parser_hooks;
pub mod regex_compile_validation;
pub mod sv_preprocessor;
pub mod test_registry;
pub mod test_runner; // Only declare once

// New automation modules
pub mod test_discovery;

// Re-export Logger trait for generated parsers
pub use ast_pipeline::Logger;
pub use ast_pipeline::NoOpLogger;

#[cfg(all(feature = "ebnf_dual_run", has_generated_ebnf_parser))]
pub mod ebnf_generated_parser {
    include!(env!("PGEN_EBNF_PARSER_PATH_RESOLVED"));
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
        // Backward-compat alias for previously generated snake_case parser type.
        #[allow(non_camel_case_types)]
        pub type Semantic_annotationParser<'input> = SemanticAnnotationParser<'input>;
    }
    #[cfg(has_generated_systemverilog_parser)]
    pub mod systemverilog {
        include!(env!("PGEN_SYSTEMVERILOG_PARSER_PATH_RESOLVED"));
    }
    #[cfg(has_generated_json_parser)]
    pub mod json {
        include!(env!("PGEN_JSON_PARSER_PATH_RESOLVED"));
    }
    #[cfg(has_generated_regex_parser)]
    pub mod regex {
        include!(env!("PGEN_REGEX_PARSER_PATH_RESOLVED"));
    }
    #[cfg(has_generated_systemverilog_preprocessor_parser)]
    pub mod systemverilog_preprocessor {
        include!(env!("PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_PATH_RESOLVED"));
    }
    #[cfg(has_generated_vhdl_parser)]
    pub mod vhdl {
        include!(env!("PGEN_VHDL_PARSER_PATH_RESOLVED"));
    }
    #[cfg(has_generated_rtl_const_expr_parser)]
    pub mod rtl_const_expr {
        include!(env!("PGEN_RTL_CONST_EXPR_PARSER_PATH_RESOLVED"));
    }
    #[cfg(has_generated_rtl_frontend_parser)]
    pub mod rtl_frontend {
        include!(env!("PGEN_RTL_FRONTEND_PARSER_PATH_RESOLVED"));
    }
}

#[cfg(feature = "generated_parsers")]
pub mod parser_registry;
