use crate::test_runner::Parser;
use std::fs;
use serde_json;
/// Round-trip testing framework for mathematical parser validation
/// Provides complete input → parse → AST → unparse → output validation

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
    pub suite_name: String,
    pub parser_type: String,
    pub description: String,
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
#[derive(Clone)]
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
    verbose: bool,
    parser_filter: Option<String>,
    tag_filter: Vec<String>,
}

impl RoundTripTestRunner {
    pub fn new() -> Self {
        let test_data_dir = PathBuf::from("test_data");
        Self {
            test_data_dir,
            results: Vec::new(),
            parser: None,
            verbose: false,
            parser_filter: None,
            tag_filter: Vec::new(),
        }
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn with_parser(mut self, parser: Box<dyn Parser>) -> Self {
        self.parser = Some(parser);
        self
    }

    pub fn with_parser_filter(mut self, filter: String) -> Self {
        self.parser_filter = Some(filter);
        self
    }

    pub fn with_tag_filter(mut self, tags: Vec<String>) -> Self {
        self.tag_filter = tags;
        self
    }

    pub fn discover_test_suites(&self) -> Result<Vec<TestSuite>> {
        let mut suites = Vec::new();
        if !self.test_data_dir.exists() {
            fs::create_dir_all(&self.test_data_dir)?;
        }
        
        // Find all .json files in subdirectories
        for entry in fs::read_dir(&self.test_data_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let subdir_name = path.file_name().unwrap().to_string_lossy().into_owned();
                
                // Look for all .json files in this subdirectory
                for json_entry in fs::read_dir(&path)? {
                    let json_entry = json_entry?;
                    let json_path = json_entry.path();
                    
                    if json_path.is_file() && json_path.extension() == Some(std::ffi::OsStr::new("json")) {
                        let json_name = json_path.file_stem().unwrap().to_string_lossy();
                        let suite_name = format!("{}_{}", subdir_name, json_name);
                        
                        match fs::read_to_string(&json_path) {
                            Ok(json_str) => {
                                // Skip if not starting with [ (not an array)
                                if !json_str.trim_start().starts_with('[') {
                                    continue;
                                }
                                
                                match serde_json::from_str::<Vec<RoundTripTest>>(&json_str) {
                                    Ok(tests) => {
                                        suites.push(TestSuite { 
                                            name: suite_name.clone(), 
                                            suite_name, 
                                            parser_type: "unknown".to_string(), 
                                            description: format!("Tests from {}/{}", subdir_name, json_path.file_name().unwrap().to_string_lossy()),
                                            tests 
                                        });
                                    }
                                    Err(_) => {
                                        // Skip invalid JSON files
                                        continue;
                                    }
                                }
                            }
                            Err(_) => continue,
                        }
                    }
                }
            }
        }
        Ok(suites)
    }

    pub fn run_all_tests(&mut self) -> Result<Report> {
        self.run_all_tests_with_filters(self.parser_filter.clone(), self.tag_filter.clone())
    }

    pub fn run_all_tests_with_filters(&mut self, parser_filter: Option<String>, tag_filter: Vec<String>) -> Result<Report> {
        let suites = self.discover_test_suites()?;
        let mut report = Report::default();
        
        for suite in suites {
            for test in &suite.tests {
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
        // Log the start of this test case
        println!("\n🧪 Testing: {}", test.input);
        println!("{}", "─".repeat(60));
        
        // Use real parser if available, otherwise fall back to mock
        let unparsed = if let Some(ref parser) = self.parser {
            match parser.round_trip(&test.input) {
                Ok(result) => result,
                Err(e) => {
                    println!("❌ Parser error: {}", e);
                    println!(""); // Empty line before next test
                    return TestResult {
                        suite: suite.name.clone(),
                        test: test.name.clone(),
                        passed: false,
                        message: format!("PARSER ERROR: {}", e),
                    };
                }
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
        let msg = if passed { 
            "✅ PASSED".to_string() 
        } else { 
            format!("❌ FAILED - Expected: '{}', Got: '{}'", expected, actual) 
        };
        
        println!("Result: {}", msg);
        println!(""); // Empty line before next test
        
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
