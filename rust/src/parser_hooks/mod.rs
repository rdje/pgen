//! Parser-specific hook implementations.
//!
//! # Standing rule
//!
//! The Rust AST pipeline at [`crate::ast_pipeline`] is parser-agnostic.
//! Parser-specific behavior — anything that says "for grammar X do Y" —
//! lives in this module instead. Each grammar that needs codegen
//! extensibility gets one sibling submodule (e.g.
//! [`regex`]) that implements
//! [`crate::ast_pipeline::ParserHooks`] for that grammar.
//!
//! # Registration
//!
//! Hooks are not registered automatically. The binary boundary
//! (typically [`crate`]'s `pgen_ast` binary or a downstream embedder)
//! constructs a [`crate::ast_pipeline::ParserHookRegistry`], registers
//! the handlers it wants, and hands the registry to the AST pipeline.
//!
//! When no handler is registered for a grammar, the pipeline emits the
//! same output it has always emitted — every tracked parser is
//! byte-identical to its pre-handler baseline until a handler is
//! explicitly registered for it.
//!
//! # Adding a new grammar
//!
//! 1. Add a sibling submodule: `pub mod foolang;`.
//! 2. Implement `ParserHooks` for `FoolangParserHooks` (or whatever
//!    type name fits) with `grammar_name() -> "foolang"`.
//! 3. Wire registration at the binary boundary; do NOT touch
//!    `rust/src/ast_pipeline/`.

pub mod regex;
