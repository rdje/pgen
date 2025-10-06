//! EBNF AST Transformation Pipeline Library
//!
//! This library provides AST transformation capabilities for EBNF grammars,
//! including semantic annotation parsing and high-performance parser generation.

pub mod ast_pipeline;
pub mod test_runner;
pub mod test_registry; // Only declare once

// New automation modules
pub mod test_discovery;

