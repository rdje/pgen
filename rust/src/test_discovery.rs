//! Test Discovery System
//! Automatically scans and extracts test cases from stress test files

use std::path::Path;
use std::fs;
use regex::Regex;
use crate::test_registry::{TestCase, TestExpectation, TestRegistry};

pub struct TestDiscovery {
    root_path: String,
}

impl TestDiscovery {
    pub fn new(root_path: &str) -> Self {
        Self {
            root_path: root_path.to_string(),
        }
    }
    
    /// Discover all test cases by scanning stress test files
    pub fn discover_all_tests(&self) -> Result<TestRegistry, Box<dyn std::error::Error>> {
        let mut registry = TestRegistry::default();
        
        // Clear existing tests since we're rediscovering
        registry.return_tests.clear();
        registry.semantic_tests.clear();
        registry.regex_tests.clear();
        
        // Scan comprehensive_stress_test.rs for return tests
        self.scan_comprehensive_stress_test(&mut registry)?;
        
        // Scan semantic_annotation_stress_test.rs for semantic tests
        self.scan_semantic_stress_test(&mut registry)?;
        
        // Scan regex_stress_test.rs for regex tests (if exists)
        self.scan_regex_stress_test(&mut registry)?;
        
        registry.last_updated = chrono::Utc::now().to_rfc3339();
        Ok(registry)
    }
    
    /// Scan comprehensive_stress_test.rs for return parser test cases
    fn scan_comprehensive_stress_test(&self, registry: &mut TestRegistry) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = format!("{}/src/comprehensive_stress_test.rs", self.root_path);
        
        if !Path::new(&file_path).exists() {
            println!("Warning: {} not found", file_path);
            return Ok(());
        }
        
        let content = fs::read_to_string(&file_path)?;
        
        // Extract test inputs from the test_inputs array
        let inputs = self.extract_test_inputs_from_array(&content, "test_inputs")?;
        
        for input in inputs {
            let test_case = TestCase {
                input: input.clone(),
                description: self.generate_description_for_input(&input, "return"),
                parser_type: "return".to_string(),
                category: self.categorize_input(&input, "return"),
                expected_result: TestExpectation::Success,
            };
            registry.return_tests.push(test_case);
        }
        
        println!("Discovered {} return test cases from {}", registry.return_tests.len(), file_path);
        Ok(())
    }
    
    /// Scan semantic_annotation_stress_test.rs for semantic parser test cases
    fn scan_semantic_stress_test(&self, registry: &mut TestRegistry) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = format!("{}/src/semantic_annotation_stress_test.rs", self.root_path);
        
        if !Path::new(&file_path).exists() {
            println!("Warning: {} not found", file_path);
            return Ok(());
        }
        
        let content = fs::read_to_string(&file_path)?;
        
        // Extract test inputs from the test_inputs array
        let inputs = self.extract_test_inputs_from_array(&content, "test_inputs")?;
        
        for input in inputs {
            let test_case = TestCase {
                input: input.clone(),
                description: self.generate_description_for_input(&input, "semantic"),
                parser_type: "semantic".to_string(),
                category: self.categorize_input(&input, "semantic"),
                expected_result: TestExpectation::Success,
            };
            registry.semantic_tests.push(test_case);
        }
        
        println!("Discovered {} semantic test cases from {}", registry.semantic_tests.len(), file_path);
        Ok(())
    }
    
    /// Scan regex_stress_test.rs for regex parser test cases
    fn scan_regex_stress_test(&self, registry: &mut TestRegistry) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = format!("{}/src/regex_stress_test.rs", self.root_path);
        
        if !Path::new(&file_path).exists() {
            println!("Warning: {} not found - skipping regex test discovery", file_path);
            return Ok(());
        }
        
        let content = fs::read_to_string(&file_path)?;
        
        // Extract test inputs from the test_inputs array
        let inputs = self.extract_test_inputs_from_array(&content, "test_inputs")?;
        
        for input in inputs {
            let test_case = TestCase {
                input: input.clone(),
                description: self.generate_description_for_input(&input, "regex"),
                parser_type: "regex".to_string(),
                category: self.categorize_input(&input, "regex"),
                expected_result: TestExpectation::Success,
            };
            registry.regex_tests.push(test_case);
        }
        
        println!("Discovered {} regex test cases from {}", registry.regex_tests.len(), file_path);
        Ok(())
    }
    
    /// Extract test inputs from a Rust array declaration
    fn extract_test_inputs_from_array(&self, content: &str, array_name: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // Find the array declaration
        let array_pattern = format!(r"let\s+{}\s*=\s*\[(.*?)\];", array_name);
        let array_regex = Regex::new(&array_pattern)?;
        
        if let Some(captures) = array_regex.captures(content) {
            let array_content = captures.get(1).unwrap().as_str();
            
            // Extract string literals from the array
            let string_regex = Regex::new(r#""([^"\\]*(\\.[^"\\]*)*)""#)?;
            let inputs: Vec<String> = string_regex
                .captures_iter(array_content)
                .map(|cap| cap.get(1).unwrap().as_str().to_string())
                .collect();
            
            Ok(inputs)
        } else {
            // Try alternative pattern - maybe it's split across multiple lines
            self.extract_multiline_array(content, array_name)
        }
    }
    
    /// Extract test inputs from multiline array declarations
    fn extract_multiline_array(&self, content: &str, array_name: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let lines: Vec<&str> = content.lines().collect();
        let mut in_array = false;
        let mut brace_count = 0;
        let mut inputs = Vec::new();
        
        for line in lines {
            // Look for array start
            if line.contains(&format!("let {}", array_name)) && line.contains("[") {
                in_array = true;
                brace_count = 1;
                
                // Check if there are strings on the same line
                let string_regex = Regex::new(r#""([^"\\]*(\\.[^"\\]*)*)""#)?;
                for cap in string_regex.captures_iter(line) {
                    inputs.push(cap.get(1).unwrap().as_str().to_string());
                }
                continue;
            }
            
            if in_array {
                // Count braces to track array boundaries
                brace_count += line.matches('[').count();
                brace_count -= line.matches(']').count();
                
                // Extract strings from this line
                let string_regex = Regex::new(r#""([^"\\]*(\\.[^"\\]*)*)""#)?;
                for cap in string_regex.captures_iter(line) {
                    inputs.push(cap.get(1).unwrap().as_str().to_string());
                }
                
                // Check if array is complete
                if brace_count == 0 {
                    break;
                }
            }
        }
        
        Ok(inputs)
    }
    
    /// Generate a human-readable description for a test input
    fn generate_description_for_input(&self, input: &str, parser_type: &str) -> String {
        match parser_type {
            "return" => {
                if input.starts_with('$') && input.chars().skip(1).all(|c| c.is_ascii_digit()) {
                    format!("Scalar reference: {}", input)
                } else if input.starts_with('"') && input.ends_with('"') {
                    "String literal".to_string()
                } else if input.parse::<i32>().is_ok() {
                    "Numeric literal".to_string()
                } else if input == "true" || input == "false" {
                    "Boolean literal".to_string()
                } else if input.starts_with('[') && input.ends_with(']') {
                    "Array expression".to_string()
                } else if input.starts_with('{') && input.ends_with('}') {
                    "Object expression".to_string()
                } else if input.contains('.') && !input.starts_with('.') {
                    "Property access".to_string()
                } else if input.contains('[') && input.contains(']') && !input.starts_with('[') {
                    "Index access".to_string()
                } else {
                    format!("Complex expression: {}", input)
                }
            }
            "semantic" => {
                if input.starts_with("@type") {
                    "Type annotation".to_string()
                } else if input.starts_with("@precedence") {
                    "Precedence annotation".to_string()
                } else if input.contains("side_effect") {
                    "Side effect annotation".to_string()
                } else if input.contains("validate") {
                    "Validation annotation".to_string()
                } else if input.contains("cache") {
                    "Cache annotation".to_string()
                } else {
                    format!("Semantic annotation: {}", input.split(':').next().unwrap_or(input))
                }
            }
            "regex" => {
                if input.chars().all(|c| c.is_alphanumeric()) {
                    "Literal pattern".to_string()
                } else if input.contains('[') && input.contains(']') {
                    "Character class".to_string()
                } else if input.contains('*') || input.contains('+') || input.contains('?') {
                    "Quantifier pattern".to_string()
                } else if input.contains('^') || input.contains('$') {
                    "Anchor pattern".to_string()
                } else if input.contains('(') && input.contains(')') {
                    "Group pattern".to_string()
                } else if input.contains('\\') {
                    "Escape sequence".to_string()
                } else {
                    format!("Regex pattern: {}", input)
                }
            }
            _ => format!("Test input: {}", input),
        }
    }
    
    /// Categorize a test input into a logical category
    fn categorize_input(&self, input: &str, parser_type: &str) -> String {
        match parser_type {
            "return" => {
                if input.starts_with('$') && input.chars().skip(1).all(|c| c.is_ascii_digit()) {
                    "scalar".to_string()
                } else if input.starts_with('"') || input.parse::<i32>().is_ok() || input == "true" || input == "false" {
                    "literal".to_string()
                } else if input.starts_with('[') && input.ends_with(']') {
                    "array".to_string()
                } else if input.starts_with('{') && input.ends_with('}') {
                    "object".to_string()
                } else if input.contains('.') || input.contains('[') {
                    "accessor".to_string()
                } else {
                    "complex".to_string()
                }
            }
            "semantic" => {
                if input.starts_with("@type") {
                    "type".to_string()
                } else if input.starts_with("@precedence") {
                    "precedence".to_string()
                } else if input.contains("boolean") || input.contains("true") || input.contains("false") {
                    "boolean".to_string()
                } else if input.contains('[') && input.contains(']') {
                    "array".to_string()
                } else if input.contains('{') && input.contains('}') {
                    "object".to_string()
                } else {
                    "annotation".to_string()
                }
            }
            "regex" => {
                if input.chars().all(|c| c.is_alphanumeric()) {
                    "literal".to_string()
                } else if input.contains('[') && input.contains(']') {
                    "charclass".to_string()
                } else if input.contains('*') || input.contains('+') || input.contains('?') {
                    "quantifier".to_string()
                } else if input.contains('^') || input.contains('$') {
                    "anchor".to_string()
                } else if input.contains('(') && input.contains(')') {
                    "group".to_string()
                } else if input.contains('\\') {
                    "escape".to_string()
                } else {
                    "metachar".to_string()
                }
            }
            _ => "unknown".to_string(),
        }
    }
}