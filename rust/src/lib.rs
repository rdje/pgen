//! EBNF AST Transformation Pipeline Library
//!
//! This library provides AST transformation capabilities for EBNF grammars,
//! including semantic annotation parsing and high-performance parser generation.

pub mod ast_pipeline;
pub mod ast_shape_contract;
pub mod auto_return_annotation_shape_gate;
/// `SV-CORPUS-GRAD.8c.3` — dedicated 256 MiB-stack execution for generated-parser entry
/// boundaries, so the engine's 4096-frame recursion ceiling provably fires before the OS guard
/// page in BOTH build modes (the PGEN-RGX-0085 "ceiling must bound the REAL stack" law,
/// generalized parser-agnostically). See the module docs for the measured constants.
pub mod dedicated_parse_stack;
#[cfg(feature = "ebnf_dual_run")]
pub mod ebnf_frontend;
/// LANG-CAPABILITY-AUDIT.10.6 part 2 — the frontend⟷meta-parser raw-AST envelope differential:
/// project the parser GENERATED from `grammars/ebnf.ebnf` into the hand-written frontend's
/// `raw_ast` token vocabulary and diff them, so the frontend-REPLACEMENT gap is measured rather
/// than assumed from a parse verdict. Carries its own positive and negative controls. See the
/// module docs.
#[cfg(feature = "ebnf_dual_run")]
pub mod ebnf_envelope_differential;
pub mod embedding_api;
/// PARSE-HARNESS.3 — the compile-and-run harness (approach 2): run the REAL codegen + runtime on an
/// ARBITRARY grammar via a throwaway external crate, authoritative by construction. See the module docs.
pub mod parse_harness;
/// PARSE-HARNESS.4 — the grammar-AST interpreter (approach 1): an in-process, no-codegen dynamic
/// dispatcher over the normalized gen-AST that reuses the shipped semantic/AST runtime, authoritative
/// by VERIFICATION (a differential-equivalence oracle vs the generated parser). See the module docs.
pub mod parse_harness_interpreter;
/// PARSE-HARNESS.5 — the differential-equivalence gate driver: run the interpreter (`.4`) and the
/// shipped generated parser over one deterministic stimuli corpus and assert byte-identical
/// verdict + typed AST, per registered grammar. Report-first (never panics); the certified/deferred
/// split is the honest scope of the interpreter's trust. Requires BOTH `ebnf_dual_run` (the `.ebnf`
/// loader + stimuli corpus) and `generated_parsers` (the `parser_registry` oracle). See the module docs.
#[cfg(all(feature = "ebnf_dual_run", feature = "generated_parsers"))]
pub mod parse_harness_equivalence;
/// PARSE-HARNESS.6.1 — the structural combinator isolating suite: a table of small synthetic grammars,
/// one per structural combinator the `.4` interpreter dispatches, each differentially verified
/// byte-identical against the compile-and-run oracle (`.3`). Upgrades the interpreter's trust from "the
/// shipped grammars' constructs" to "any grammar built from PGEN's structural constructs". Requires
/// `ebnf_dual_run` (the interpreter's `.ebnf` loader; the oracle shells the codegen binary). See the
/// module docs.
#[cfg(feature = "ebnf_dual_run")]
pub mod parse_harness_combinator_suite;
/// PARSE-HARNESS.6.2 — the semantic-directive orchestration isolating suite: the `.6.1` sibling for the
/// store-gated-outcome surface (`@predicate` pre/branch/post gates, `@emit_fact` + the query vocabulary,
/// the scope tree, C3-B rollback, `$reference` resolution, branch-start inline actions, library no-op
/// parity, and memoization × store composition), each construct differentially verified byte-identical
/// against the compile-and-run oracle (`.3`) on small isolating grammars. See the module docs.
#[cfg(feature = "ebnf_dual_run")]
pub mod parse_harness_semantic_suite;
pub mod regex_compile_validation;
pub mod sv_preprocessor;
pub mod test_registry;
pub mod test_runner; // Only declare once

// New automation modules
pub mod test_discovery;

// Re-export Logger trait for generated parsers
pub use ast_pipeline::Logger;
pub use ast_pipeline::NoOpLogger;
// RGX-0078.5.d.4.i — the per-parse node arena (candidate B). Re-exported at the
// crate root so boundary sites and the compile-and-run harness template can
// create one (`pgen::NodeArena::new()`) and pass `&arena` into a generated
// parser's `new(input, arena, logger)`.
pub use ast_pipeline::NodeArena;

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
    // PARSE-HARNESS.2 — the blessed scratch-register slot (arbitrary-grammar probe).
    // Present only when `make focus_scratch` has built the artifact.
    #[cfg(has_generated_scratch_parser)]
    pub mod scratch {
        include!(env!("PGEN_SCRATCH_PARSER_PATH_RESOLVED"));
    }
}

#[cfg(feature = "generated_parsers")]
pub mod parser_registry;
