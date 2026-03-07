// Test runner module for pgen
// Provides infrastructure for running JSON-based tests

use std::io::Write;

pub mod normalization;
pub mod parsers;
pub mod round_trip_tests;

// Re-export the shared Logger trait
pub use crate::Logger;
pub use crate::NoOpLogger;
pub use crate::ast_pipeline::TraceVerbosity;

// File logger that writes to the test runner's log file
#[derive(Clone, Debug)]
pub struct FileLogger {
    file: std::sync::Arc<std::sync::Mutex<Option<std::fs::File>>>,
    verbosity: TraceVerbosity,
}

impl FileLogger {
    pub fn new(file: std::fs::File, verbosity: TraceVerbosity) -> Self {
        Self {
            file: std::sync::Arc::new(std::sync::Mutex::new(Some(file))),
            verbosity,
        }
    }

    fn allows_low(&self) -> bool {
        self.verbosity >= TraceVerbosity::Low
    }

    fn allows_medium(&self) -> bool {
        self.verbosity >= TraceVerbosity::Medium
    }

    fn allows_high(&self) -> bool {
        self.verbosity >= TraceVerbosity::High
    }

    fn allows_debug(&self) -> bool {
        self.verbosity >= TraceVerbosity::Debug
    }
}

impl Logger for FileLogger {
    fn log_info(&self, file: &str, line: u32, message: &str) {
        if !self.allows_high() {
            return;
        }
        if let Ok(mut guard) = self.file.lock() {
            if let Some(ref mut f) = *guard {
                let _ = writeln!(f, "[INFO] {}:{} | {}", file, line, message);
            }
        }
    }

    fn log_warning(&self, file: &str, line: u32, message: &str) {
        if !self.allows_low() {
            return;
        }
        if let Ok(mut guard) = self.file.lock() {
            if let Some(ref mut f) = *guard {
                let _ = writeln!(f, "[WARN] {}:{} | {}", file, line, message);
            }
        }
    }

    fn log_error(&self, file: &str, line: u32, message: &str) {
        if !self.allows_low() {
            return;
        }
        if let Ok(mut guard) = self.file.lock() {
            if let Some(ref mut f) = *guard {
                let _ = writeln!(f, "[ERROR] {}:{} | {}", file, line, message);
            }
        }
    }

    fn log_success(&self, file: &str, line: u32, message: &str) {
        if !self.allows_medium() {
            return;
        }
        if let Ok(mut guard) = self.file.lock() {
            if let Some(ref mut f) = *guard {
                let _ = writeln!(f, "[SUCCESS] {}:{} | {}", file, line, message);
            }
        }
    }

    fn log_debug(&self, file: &str, line: u32, message: &str) {
        if !self.allows_debug() {
            return;
        }
        if let Ok(mut guard) = self.file.lock() {
            if let Some(ref mut f) = *guard {
                let _ = writeln!(f, "[DEBUG] {}:{} | {}", file, line, message);
            }
        }
    }

    fn is_enabled(&self) -> bool {
        self.verbosity != TraceVerbosity::None
    }

    fn clone_box(&self) -> Box<dyn Logger> {
        // Now that FileLogger is Clone, we can clone it properly
        Box::new(self.clone())
    }
}

pub use crate::test_runner::parsers::{ReturnAnnotationParser, SemanticAnnotationParser};
pub use round_trip_tests::RoundTripTestRunner as UniversalTestRunner;
pub use round_trip_tests::{Report, RoundTripTestRunner, TestSuite};

/// Trait for parser integration with round-trip testing
/// Implement this trait to plug real parsers into the testing framework
pub trait Parser {
    /// Perform a round-trip transformation: parse input to AST, then unparse back to string
    /// This enables mathematical validation that parsing is reversible
    fn round_trip(&self, input: &str) -> Result<String, Box<dyn std::error::Error>>;

    /// Set the logger for this parser
    fn set_logger(&mut self, logger: Box<dyn Logger>);

    /// Get the current logger
    fn get_logger(&self) -> &dyn Logger;
}
