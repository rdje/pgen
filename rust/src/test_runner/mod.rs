// Test runner module for pgen
// Provides infrastructure for running JSON-based tests

pub mod round_trip_tests;
pub mod normalization;

pub use round_trip_tests::{RoundTripTestRunner, Report, TestSuite};
pub use parsers::{ReturnAnnotationParser, SemanticAnnotationParser};

/// Trait for parser integration with round-trip testing
/// Implement this trait to plug real parsers into the testing framework
pub trait Parser {
    /// Perform a round-trip transformation: parse input to AST, then unparse back to string
    /// This enables mathematical validation that parsing is reversible
    fn round_trip(&self, input: &str) -> Result<String>;
}
