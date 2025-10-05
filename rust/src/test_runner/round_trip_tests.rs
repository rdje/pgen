use crate::test_runner::Parser;
use std::fs;
use serde_json;
//! Round-trip testing framework for mathematical parser validation
//! Provides complete input → parse → AST → unparse → output validation

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use anyhow::Result;

#[derive(Debug, Deserialize, Serialize)]
pub struct RoundTripTest {
    pub name: String,
    pub description: String,
    pub input: String,
    pub expected_round_trip: String,
    #[serde(default)]
    pub parser_type: String,
    #[serde(default)]
    pub normalizer: String,
    #[serde(default)]
    pub float_precision: Option<usize>,
    #[serde(default)]
    pub skip: bool,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub struct TestSuite {
    pub name: String,
    pub tests: Vec<RoundTripTest>,
}

#[derive(Debug, Default)]
pub struct Report {
    pub passed: usize,
    pub failed: usize,
    pub total: usize,
    pub results: Vec<TestResult>,
}

impl Report {
    pub fn add_result(&mut self, result: TestResult) {
        self.results.push(result.clone());
        self.total += 1;
        if result.passed { self.passed += 1; } else { self.failed += 1; }
    }

    pub fn print_summary(&self) {
        println!("Summary: {}/{} passed", self.passed, self.total);
        if self.failed > 0 {
            println!("Failures:");
            for r in &self.results { if !r.passed { println!("  - {}: {}", r.suite, r.test); } }
        }
    }

    pub fn print_dashboard(&self, parser_name: &str) {
        println!("Dashboard for {}:", parser_name);
        println!("Passed: {}, Failed: {}, Rate: {:.1}%", self.passed, self.failed, (self.passed as f64 / self.total as f64) * 100.0);
        println!("| {:<20} | {:<15} | {:<6} | {}", "Suite", "Test", "Passed", "Message");
        println!("{}", "-".repeat(70));
        for r in &self.results {
            println!("| {:<20} | {:<15} | {:<6} | {}", r.suite, r.test, r.passed, r.message);
        }
    }
}

#[derive(Debug)]
pub struct TestResult {
    pub suite: String,
    pub test: String,
    pub passed: bool,
    pub message: String,
}

pub struct RoundTripTestRunner {
    test_data_dir: PathBuf,
    pub results: Vec<TestResult>,
    parser: Option<Box<dyn Parser>>,
}

impl RoundTripTestRunner {
    pub fn new() -> Self {
        let test_data_dir = PathBuf::from("test_data/return_annotations");
        Self {
            test_data_dir,
            results: Vec::new(),
            parser: None,
        }
    }

    /// Configure the runner with a specific parser implementation
    /// This allows testing real parsers instead of mock implementations
    pub fn with_parser(mut self, parser: Box<dyn Parser>) -> Self {
        self.parser = Some(parser);
        self
    }

    pub fn discover_test_suites(&self) -> Result<Vec<TestSuite>> {
        let mut suites = Vec::new();
        if !self.test_data_dir.exists() {
            fs::create_dir_all(&self.test_data_dir)?;
        }
        for entry in fs::read_dir(&self.test_data_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let suite_name = path.file_name().unwrap().to_string_lossy().into_owned();
                let mut tests = Vec::new();
                // Assume one suite.json per dir
                let json_path = path.join("suite.json");
                if json_path.exists() {
                    let json_str = fs::read_to_string(&json_path)?;
                    let suite_tests: Vec<RoundTripTest> = serde_json::from_str(&json_str)?;
                    tests = suite_tests;
                }
                suites.push(TestSuite { name: suite_name, tests });
            }
        }
        Ok(suites)
    }

    pub fn run_all_tests(&mut self, parser_filter: Option<String>, tag_filter: Vec<String>) -> Result<Report> {
        let suites = self.discover_test_suites()?;
        let mut report = Report::default();
        
        for suite in suites {
            for test in suite.tests {
                if test.skip { continue; }
                
                // Filter by parser
                if let Some(ref parser) = parser_filter {
                    if test.parser_type != *parser { continue; }
                }
                
                // Filter by tags
                if !tag_filter.is_empty() {
                    let has_matching_tag = test.tags.iter().any(|tag| tag_filter.contains(tag));
                    if !has_matching_tag { continue; }
                }
                
                // Run test with timeout
                let start_time = Instant::now();
                let test_result = self.run_single_test_with_timeout(&suite, &test);
                let duration = start_time.elapsed();
                
                // Check for timeout
                let timeout_ms = test.timeout_ms.unwrap_or(5000);
                if duration > Duration::from_millis(timeout_ms) {
                    report.add_result(TestResult {
                        suite: suite.name.clone(),
                        test: test.name.clone(),
                        passed: false,
                        message: format!("TIMEOUT: {}ms > {}ms", duration.as_millis(), timeout_ms),
                    });
                    continue;
                }
                
                report.add_result(test_result);
            }
        }
        
        self.results = report.results.clone();
        Ok(report)
    }

    fn run_single_test_with_timeout(&self, suite: &TestSuite, test: &RoundTripTest) -> TestResult {
        // Use real parser if available, otherwise fall back to mock
        let unparsed = if let Some(ref parser) = self.parser {
            match parser.round_trip(&test.input) {
                Ok(result) => result,
                Err(e) => return TestResult {
                    suite: suite.name.clone(),
                    test: test.name.clone(),
                    passed: false,
                    message: format!("PARSER ERROR: {}", e),
                },
            }
        } else {
            // Mock round-trip for testing the framework
            let parsed = self.mock_parse(&test.input);
            self.mock_unparse(&parsed)
        };
        
        // Apply normalization
        let normalizer = crate::test_runner::normalization::Normalizer::from_str(&test.normalizer);
        let actual = crate::test_runner::normalization::apply_normalizer(normalizer, &unparsed, test.float_precision);
        let expected = crate::test_runner::normalization::apply_normalizer(normalizer, &test.expected_round_trip, test.float_precision);
        
        let passed = actual == expected;
        let msg = if passed { "OK".to_string() } else { format!("Expected: '{}', Got: '{}'", expected, actual) };
        
        TestResult {
            suite: suite.name.clone(),
            test: test.name.clone(),
            passed,
            message: msg,
        }
    }

    // Mock implementations for testing the framework
    fn mock_parse(&self, input: &str) -> String {
        // For now, just return the input as a mock AST representation
        format!("parsed: {}", input)
    }
    
    fn mock_unparse(&self, ast: &str) -> String {
        // For now, just return the AST as a string
        ast.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_round_trip_runner() {
        let mut runner = RoundTripTestRunner::new();
        let report = runner.run_all_tests(None, vec![]).unwrap();
        assert!(report.failed == 0);
    }
}
