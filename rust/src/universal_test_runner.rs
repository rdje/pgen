// Universal Test Runner for pgen
// ================================
// ONE test runner to rule them all!
// Works with ANY parser, driven entirely by JSON test definitions

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Deserialize, Serialize};
use serde_json;
use anyhow::{Result, Context};
use std::collections::HashMap;
use std::time::Instant;

// Universal test definition that works for ANY parser
#[derive(Debug, Deserialize, Serialize)]
pub struct TestSuite {
    pub suite_name: String,
    pub description: String,
    pub parser_type: String,  // "return", "semantic", "regex", or any future parser
    pub parser_config: Option<ParserConfig>,
    pub tests: Vec<TestCase>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ParserConfig {
    pub parser_path: Option<String>,      // Path to generated parser if not standard
    pub entry_rule: Option<String>,       // Entry rule for parser
    pub bootstrap_mode: Option<bool>,     // Use bootstrap parser
    pub debug_mode: Option<bool>,         // Enable debug by default
    pub custom_options: Option<HashMap<String, serde_json::Value>>, // Future extensibility
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TestCase {
    pub name: String,
    pub description: String,
    pub input: String,
    pub expected: Expected,
    pub skip: Option<bool>,
    pub timeout_ms: Option<u64>,
    pub tags: Option<Vec<String>>,  // For selective test running
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Expected {
    Success {
        output: Option<serde_json::Value>,  // Expected output/AST
        contains: Option<Vec<String>>,      // Output should contain these strings
        not_contains: Option<Vec<String>>,  // Output should NOT contain these
    },
    Error {
        error: String,                      // Expected error message/pattern
    },
    Any,                                    // Just check it doesn't crash
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub suite: String,
    pub test: String,
    pub passed: bool,
    pub duration_ms: f64,
    pub message: String,
    pub output: Option<String>,
}

pub struct UniversalTestRunner {
    test_data_root: PathBuf,
    pgen_binary: PathBuf,
    results: Vec<TestResult>,
    verbose: bool,
    filter_tags: Option<Vec<String>>,
    filter_parser: Option<String>,
}

impl UniversalTestRunner {
    pub fn new() -> Self {
        let test_data_root = PathBuf::from("test_data");
        let pgen_binary = PathBuf::from("target/debug/pgen");
        
        Self {
            test_data_root,
            pgen_binary,
            results: Vec::new(),
            verbose: false,
            filter_tags: None,
            filter_parser: None,
        }
    }
    
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
    
    pub fn with_tag_filter(mut self, tags: Vec<String>) -> Self {
        self.filter_tags = Some(tags);
        self
    }
    
    pub fn with_parser_filter(mut self, parser: String) -> Self {
        self.filter_parser = Some(parser);
        self
    }
    
    /// Discover and load all test suites from JSON files
    pub fn discover_test_suites(&self) -> Result<Vec<TestSuite>> {
        let mut suites = Vec::new();
        
        if !self.test_data_root.exists() {
            fs::create_dir_all(&self.test_data_root)?;
        }
        
        self.discover_recursive(&self.test_data_root, &mut suites)?;
        
        // Apply parser filter if set
        if let Some(ref parser) = self.filter_parser {
            suites.retain(|s| &s.parser_type == parser);
        }
        
        Ok(suites)
    }
    
    fn discover_recursive(&self, dir: &Path, suites: &mut Vec<TestSuite>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                self.discover_recursive(&path, suites)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
                // Skip special files
                if path.file_name().and_then(|s| s.to_str()).map_or(false, |name| {
                    name.starts_with("_") || name == "config.json"
                }) {
                    continue;
                }
                
                match self.load_test_suite(&path) {
                    Ok(suite) => suites.push(suite),
                    Err(e) => {
                        eprintln!("Warning: Failed to load {:?}: {}", path, e);
                    }
                }
            }
        }
        Ok(())
    }
    
    fn load_test_suite(&self, path: &Path) -> Result<TestSuite> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {:?}", path))?;
        
        let suite: TestSuite = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON from {:?}", path))?;
        
        Ok(suite)
    }
    
    /// Run all discovered test suites
    pub fn run_all_tests(&mut self) -> Result<TestReport> {
        let suites = self.discover_test_suites()?;
        
        println!("🔍 Discovered {} test suites", suites.len());
        
        for suite in suites {
            self.run_test_suite(&suite)?;
        }
        
        Ok(self.generate_report())
    }
    
    /// Run a specific test suite
    pub fn run_test_suite(&mut self, suite: &TestSuite) -> Result<()> {
        println!("\n📋 Running: {}", suite.suite_name);
        println!("   Parser: {}", suite.parser_type);
        println!("   Tests: {}", suite.tests.len());
        
        for test in &suite.tests {
            // Skip if marked
            if test.skip.unwrap_or(false) {
                print!("⊘");
                continue;
            }
            
            // Apply tag filter
            if let Some(ref filter_tags) = self.filter_tags {
                if let Some(ref test_tags) = test.tags {
                    let has_matching_tag = test_tags.iter()
                        .any(|t| filter_tags.contains(t));
                    if !has_matching_tag {
                        continue;
                    }
                }
            }
            
            let result = self.run_single_test(suite, test);
            
            let test_result = TestResult {
                suite: suite.suite_name.clone(),
                test: test.name.clone(),
                passed: result.is_ok(),
                duration_ms: 0.0, // Will be set by run_single_test
                message: result.as_ref().err().map(|e| e.to_string()).unwrap_or_default(),
                output: result.ok(),
            };
            
            // Visual feedback
            if test_result.passed {
                print!("✓");
            } else {
                print!("✗");
                if self.verbose {
                    println!("\n   ❌ {} failed: {}", test.name, test_result.message);
                }
            }
            
            self.results.push(test_result);
        }
        
        println!(); // Newline after dots
        Ok(())
    }
    
    /// Run a single test using the pgen CLI or direct API
    fn run_single_test(&self, suite: &TestSuite, test: &TestCase) -> Result<String> {
        let start = Instant::now();
        
        // Option 1: Use the pgen CLI tool (works for any parser)
        let output = self.run_via_cli(suite, test)?;
        
        // Option 2: Could also use direct API if available
        // let output = self.run_via_api(suite, test)?;
        
        let _duration = start.elapsed();
        
        // Validate output against expectations
        self.validate_output(&output, &test.expected)?;
        
        Ok(output)
    }
    
    /// Run test via pgen CLI
    fn run_via_cli(&self, suite: &TestSuite, test: &TestCase) -> Result<String> {
        let mut cmd = Command::new(&self.pgen_binary);
        
        cmd.arg("--parser").arg(&suite.parser_type);
        cmd.arg("--input").arg(&test.input);
        
        // Add debug flag if configured
        if suite.parser_config.as_ref()
            .and_then(|c| c.debug_mode)
            .unwrap_or(false) {
            cmd.arg("--debug");
        }
        
        // Set timeout if specified
        if let Some(timeout_ms) = test.timeout_ms {
            // Would need to implement timeout logic here
            let _ = timeout_ms; // Placeholder
        }
        
        let output = cmd.output()
            .with_context(|| format!("Failed to run pgen for test '{}'", test.name))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            // Check if this was expected to fail
            if let Expected::Error { error } = &test.expected {
                if stderr.contains(error) {
                    return Ok(stderr.to_string());
                }
            }
            
            return Err(anyhow::anyhow!("Parser failed: {}", stderr));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    /// Validate output against expectations
    fn validate_output(&self, output: &str, expected: &Expected) -> Result<()> {
        match expected {
            Expected::Success { output: expected_output, contains, not_contains } => {
                // Check exact output match if specified
                if let Some(expected) = expected_output {
                    let output_json: serde_json::Value = serde_json::from_str(output)
                        .or_else(|_| {
                            // If not JSON, wrap as string
                            Ok(serde_json::json!(output))
                        })?;
                    
                    if output_json != *expected {
                        return Err(anyhow::anyhow!(
                            "Output mismatch.\nExpected: {}\nGot: {}",
                            serde_json::to_string_pretty(expected)?,
                            serde_json::to_string_pretty(&output_json)?
                        ));
                    }
                }
                
                // Check contains
                if let Some(patterns) = contains {
                    for pattern in patterns {
                        if !output.contains(pattern) {
                            return Err(anyhow::anyhow!(
                                "Output does not contain expected pattern: '{}'",
                                pattern
                            ));
                        }
                    }
                }
                
                // Check not_contains
                if let Some(patterns) = not_contains {
                    for pattern in patterns {
                        if output.contains(pattern) {
                            return Err(anyhow::anyhow!(
                                "Output contains unexpected pattern: '{}'",
                                pattern
                            ));
                        }
                    }
                }
            }
            Expected::Error { error } => {
                if !output.contains(error) {
                    return Err(anyhow::anyhow!(
                        "Expected error '{}' not found in output",
                        error
                    ));
                }
            }
            Expected::Any => {
                // Just checking it didn't crash - already handled by run_via_cli
            }
        }
        
        Ok(())
    }
    
    /// Generate test report
    fn generate_report(&self) -> TestReport {
        let total = self.results.len();
        let passed = self.results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        
        let mut by_suite: HashMap<String, (usize, usize)> = HashMap::new();
        for result in &self.results {
            let entry = by_suite.entry(result.suite.clone()).or_insert((0, 0));
            if result.passed {
                entry.0 += 1;
            } else {
                entry.1 += 1;
            }
        }
        
        TestReport {
            total_tests: total,
            passed,
            failed,
            by_suite,
            failed_tests: self.results.iter()
                .filter(|r| !r.passed)
                .cloned()
                .collect(),
        }
    }
}

#[derive(Debug)]
pub struct TestReport {
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub by_suite: HashMap<String, (usize, usize)>, // (passed, failed)
    pub failed_tests: Vec<TestResult>,
}

impl TestReport {
    pub fn print_summary(&self) {
        println!("\n{}", "═".repeat(60));
        println!("📊 Test Results Summary");
        println!("{}", "═".repeat(60));
        
        // Overall stats
        println!("Total:  {} tests", self.total_tests);
        println!("✅ Passed: {} ({:.1}%)", 
            self.passed, 
            (self.passed as f64 / self.total_tests as f64) * 100.0
        );
        println!("❌ Failed: {} ({:.1}%)", 
            self.failed,
            (self.failed as f64 / self.total_tests as f64) * 100.0
        );
        
        // By suite breakdown
        if !self.by_suite.is_empty() {
            println!("\nBy Suite:");
            for (suite, (passed, failed)) in &self.by_suite {
                let total = passed + failed;
                println!("  {}: {}/{} passed", suite, passed, total);
            }
        }
        
        // Failed tests details
        if !self.failed_tests.is_empty() {
            println!("\n🔴 Failed Tests:");
            for test in &self.failed_tests {
                println!("  • {}/{}: {}", test.suite, test.test, test.message);
            }
        } else {
            println!("\n✨ All tests passed!");
        }
        
        println!("{}", "═".repeat(60));
    }
}

// Convenience functions for running tests
pub fn run_all_tests(verbose: bool) -> Result<TestReport> {
    let mut runner = UniversalTestRunner::new().with_verbose(verbose);
    runner.run_all_tests()
}

pub fn run_parser_tests(parser: &str, verbose: bool) -> Result<TestReport> {
    let mut runner = UniversalTestRunner::new()
        .with_verbose(verbose)
        .with_parser_filter(parser.to_string());
    runner.run_all_tests()
}

pub fn run_tagged_tests(tags: Vec<String>, verbose: bool) -> Result<TestReport> {
    let mut runner = UniversalTestRunner::new()
        .with_verbose(verbose)
        .with_tag_filter(tags);
    runner.run_all_tests()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_universal_runner() {
        let mut runner = UniversalTestRunner::new();
        
        // This will automatically discover and run all tests in test_data/
        match runner.run_all_tests() {
            Ok(report) => {
                report.print_summary();
                if report.failed > 0 {
                    panic!("{} tests failed", report.failed);
                }
            }
            Err(e) => {
                eprintln!("Test runner error: {}", e);
                panic!("Test runner failed");
            }
        }
    }
}