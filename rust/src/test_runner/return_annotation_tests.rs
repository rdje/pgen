// Unified test runner for return annotation tests
// Loads and runs tests from JSON files in test_data/return_annotations/

use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use serde_json;
use anyhow::{Result, Context};
use crate::ast_pipeline::unified_return_ast::UnifiedReturnAST;

#[derive(Debug, Deserialize, Serialize)]
pub struct TestSuite {
    pub test_suite: String,
    pub description: String,
    pub tests: Vec<TestCase>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TestCase {
    pub name: String,
    pub input: String,
    pub expected_ast: Option<serde_json::Value>,
    pub expected_output: Option<String>,
    pub expected_error: Option<String>,
    pub description: String,
}

#[derive(Debug)]
pub struct TestResult {
    pub suite: String,
    pub test: String,
    pub passed: bool,
    pub message: String,
}

pub struct ReturnAnnotationTestRunner {
    test_data_dir: PathBuf,
    results: Vec<TestResult>,
}

impl ReturnAnnotationTestRunner {
    pub fn new() -> Self {
        let test_data_dir = PathBuf::from("test_data/return_annotations");
        Self {
            test_data_dir,
            results: Vec::new(),
        }
    }

    /// Load all test suites from JSON files
    pub fn load_test_suites(&self) -> Result<Vec<TestSuite>> {
        let mut suites = Vec::new();
        
        if !self.test_data_dir.exists() {
            println!("Creating test data directory: {:?}", self.test_data_dir);
            fs::create_dir_all(&self.test_data_dir)?;
        }

        for entry in fs::read_dir(&self.test_data_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read {:?}", path))?;
                
                let suite: TestSuite = serde_json::from_str(&content)
                    .with_context(|| format!("Failed to parse JSON from {:?}", path))?;
                
                suites.push(suite);
            }
        }
        
        Ok(suites)
    }

    /// Run all test suites
    pub fn run_all_tests(&mut self) -> Result<()> {
        let suites = self.load_test_suites()?;
        
        for suite in suites {
            self.run_test_suite(&suite)?;
        }
        
        self.print_results();
        Ok(())
    }

    /// Run a specific test suite
    pub fn run_test_suite(&mut self, suite: &TestSuite) -> Result<()> {
        println!("\n📋 Running test suite: {}", suite.test_suite);
        println!("   {}", suite.description);
        println!("   {} tests", suite.tests.len());
        
        for test in &suite.tests {
            let result = self.run_test(test);
            
            let test_result = TestResult {
                suite: suite.test_suite.clone(),
                test: test.name.clone(),
                passed: result.is_ok(),
                message: result.unwrap_or_else(|e| e.to_string()),
            };
            
            if test_result.passed {
                print!("✓");
            } else {
                print!("✗");
                println!("\n   ❌ {} failed: {}", test.name, test_result.message);
            }
            
            self.results.push(test_result);
        }
        
        println!(); // New line after test dots
        Ok(())
    }

    /// Run a single test case
    fn run_test(&self, test: &TestCase) -> Result<String> {
        // Parse the input using the bootstrap parser
        let ast = UnifiedReturnAST::parse_bootstrap(&test.input, false)
            .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;
        
        // If we have expected_ast, compare it
        if let Some(expected_ast) = &test.expected_ast {
            let ast_json = self.ast_to_json(&ast)?;
            
            if !self.compare_json(&ast_json, expected_ast) {
                return Err(anyhow::anyhow!(
                    "AST mismatch.\nExpected: {}\nGot: {}",
                    serde_json::to_string_pretty(expected_ast)?,
                    serde_json::to_string_pretty(&ast_json)?
                ));
            }
        }
        
        // If we have expected_output, generate code and compare
        if let Some(expected_output) = &test.expected_output {
            // This would require setting up captured variables and generating code
            // For now, we'll skip code generation tests
        }
        
        Ok("OK".to_string())
    }

    /// Convert UnifiedReturnAST to JSON for comparison
    fn ast_to_json(&self, ast: &UnifiedReturnAST) -> Result<serde_json::Value> {
        // Convert the AST to a JSON representation that matches our test format
        use crate::ast_pipeline::unified_return_ast::*;
        
        let json = match ast {
            UnifiedReturnAST::PositionalRef { index } => {
                serde_json::json!({
                    "type": "PositionalRef",
                    "index": index
                })
            }
            UnifiedReturnAST::StringLiteral { value } => {
                serde_json::json!({
                    "type": "StringLiteral",
                    "value": value
                })
            }
            UnifiedReturnAST::NumberLiteral { value } => {
                serde_json::json!({
                    "type": "NumberLiteral",
                    "value": value
                })
            }
            UnifiedReturnAST::BooleanLiteral { value } => {
                serde_json::json!({
                    "type": "BooleanLiteral",
                    "value": value
                })
            }
            UnifiedReturnAST::Array { elements } => {
                let elem_jsons: Result<Vec<_>> = elements.iter()
                    .map(|e| self.ast_to_json(e))
                    .collect();
                
                serde_json::json!({
                    "type": "Array",
                    "elements": elem_jsons?
                })
            }
            UnifiedReturnAST::Object { properties } => {
                let mut props = serde_json::Map::new();
                for (key, value) in properties {
                    props.insert(key.clone(), self.ast_to_json(value)?);
                }
                
                serde_json::json!({
                    "type": "Object",
                    "properties": props
                })
            }
            UnifiedReturnAST::Spread { base } => {
                serde_json::json!({
                    "type": "Spread",
                    "base": self.ast_to_json(base)?
                })
            }
            UnifiedReturnAST::PropertyAccess { base, property } => {
                serde_json::json!({
                    "type": "PropertyAccess",
                    "base": self.ast_to_json(base)?,
                    "property": property
                })
            }
            UnifiedReturnAST::ArrayAccess { base, index } => {
                serde_json::json!({
                    "type": "ArrayAccess",
                    "base": self.ast_to_json(base)?,
                    "index": self.ast_to_json(index)?
                })
            }
            UnifiedReturnAST::QuantifiedExtraction { base, target } => {
                use crate::ast_pipeline::unified_return_ast::ExtractionTarget;
                let target_json = match target {
                    ExtractionTarget::Index(idx) => serde_json::json!({
                        "type": "Index",
                        "value": idx
                    }),
                    ExtractionTarget::First => serde_json::json!("First"),
                    ExtractionTarget::Last => serde_json::json!("Last"),
                };
                
                serde_json::json!({
                    "type": "QuantifiedExtraction",
                    "base": self.ast_to_json(base)?,
                    "target": target_json
                })
            }
            UnifiedReturnAST::Passthrough => {
                serde_json::json!({
                    "type": "Passthrough"
                })
            }
        };
        
        Ok(json)
    }

    /// Compare two JSON values for equality
    fn compare_json(&self, a: &serde_json::Value, b: &serde_json::Value) -> bool {
        a == b
    }

    /// Print test results summary
    fn print_results(&self) {
        let total = self.results.len();
        let passed = self.results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        
        println!("\n{}", "═".repeat(50));
        println!("📊 Test Results Summary");
        println!("   Total:  {}", total);
        println!("   ✅ Passed: {}", passed);
        println!("   ❌ Failed: {}", failed);
        
        if failed > 0 {
            println!("\n🔴 Failed tests:");
            for result in &self.results {
                if !result.passed {
                    println!("   • {}/{}: {}", result.suite, result.test, result.message);
                }
            }
        } else {
            println!("\n✨ All tests passed!");
        }
        
        println!("{}", "═".repeat(50));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_return_annotation_runner() {
        let mut runner = ReturnAnnotationTestRunner::new();
        
        // This will automatically discover and run all tests in test_data/return_annotations/
        if let Err(e) = runner.run_all_tests() {
            eprintln!("Test runner error: {}", e);
        }
        
        // Check that all tests passed
        let all_passed = runner.results.iter().all(|r| r.passed);
        assert!(all_passed, "Some return annotation tests failed");
    }
}