//! EBNF AST Transformation Pipeline Library
//!
//! This library provides AST transformation capabilities for EBNF grammars,
//! including semantic annotation parsing and high-performance parser generation.

pub mod ast_pipeline;
// pub mod lexer; // Module file doesn't exist
pub mod test_runner;
pub mod test_registry; // Only declare once
// pub mod universal_test_runner;
// pub mod test_target_mapper; // Obsolete - test-* targets removed
// pub mod test_reproduction_demo; // Obsolete - depends on test_target_mapper

// New automation modules
pub mod test_discovery;
// pub mod makefile_generator; // Module file doesn't exist  
// pub mod individual_tests_generator; // Module file doesn't exist
// pub mod test_automation; // Depends on non-existent modules above

