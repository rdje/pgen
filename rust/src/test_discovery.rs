//! Test Discovery System
//! Automatically scans and extracts test cases from stress test files

use crate::test_registry::{TestCase, TestExpectation, TestRegistry};
use std::fs;
use std::path::Path;

pub struct TestDiscovery {
    root_path: String,
}

impl TestDiscovery {
    pub fn new(root_path: &str) -> Self {
        Self {
            root_path: root_path.to_string(),
        }
    }

    /// Discover all test cases by reading JSON test data files
    pub fn discover_all_tests(&self) -> Result<TestRegistry, Box<dyn std::error::Error>> {
        let mut registry = TestRegistry::default();

        // Clear existing tests since we're rediscovering
        registry.return_tests.clear();
        registry.semantic_tests.clear();
        registry.regex_tests.clear();

        // Read test data from JSON files
        let test_data_dir = format!("{}/test_data", self.root_path);
        if let Ok(entries) = fs::read_dir(&test_data_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if let Some(extension) = path.extension() {
                        if extension == "json" {
                            if let Some(filename_str) = path.file_name().and_then(|n| n.to_str()) {
                                println!("Reading test data from: {}", filename_str);
                                self.read_json_test_file(&mut registry, &path)?;
                            }
                        }
                    }
                }
            }
        }

        registry.last_updated = chrono::Utc::now().to_rfc3339();
        Ok(registry)
    }

    /// Read test cases from a JSON file
    fn read_json_test_file(
        &self,
        registry: &mut TestRegistry,
        file_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        let json_data: serde_json::Value = serde_json::from_str(&content)?;

        let parser_type = json_data["parser_type"].as_str().unwrap_or("unknown");

        // Read basic tests
        if let Some(basic_tests) = json_data["basic_tests"].as_array() {
            for test in basic_tests {
                if let Some(test_case) = self.json_to_test_case(test, parser_type) {
                    self.add_test_to_registry(registry, test_case);
                }
            }
        }

        // Read complex tests
        if let Some(complex_tests) = json_data["complex_tests"].as_array() {
            for test in complex_tests {
                if let Some(test_case) = self.json_to_test_case(test, parser_type) {
                    self.add_test_to_registry(registry, test_case);
                }
            }
        }

        Ok(())
    }

    /// Convert JSON test object to TestCase
    fn json_to_test_case(
        &self,
        json_test: &serde_json::Value,
        parser_type: &str,
    ) -> Option<TestCase> {
        let input = json_test["input"].as_str()?.to_string();
        let description = json_test["description"]
            .as_str()
            .unwrap_or("No description")
            .to_string();
        let category = json_test["category"]
            .as_str()
            .unwrap_or("general")
            .to_string();
        let expected = json_test["expected"].as_str().unwrap_or("success");

        let expected_result = match expected {
            "success" => TestExpectation::Success,
            "failure" => TestExpectation::Failure("Expected to fail".to_string()),
            _ => TestExpectation::Success,
        };

        Some(TestCase {
            input,
            description,
            parser_type: parser_type.to_string(),
            category,
            expected_result,
        })
    }

    /// Add test case to the appropriate registry list
    fn add_test_to_registry(&self, registry: &mut TestRegistry, test_case: TestCase) {
        match test_case.parser_type.as_str() {
            "return" => registry.return_tests.push(test_case),
            "semantic" => registry.semantic_tests.push(test_case),
            "regex" => registry.regex_tests.push(test_case),
            _ => {
                println!(
                    "Warning: Unknown parser type '{}', defaulting to semantic",
                    test_case.parser_type
                );
                registry.semantic_tests.push(test_case);
            }
        }
    }

    /// Generate a human-readable description for a test input
    #[allow(dead_code)]
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
                    format!(
                        "Semantic annotation: {}",
                        input.split(':').next().unwrap_or(input)
                    )
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
    #[allow(dead_code)]
    fn categorize_input(&self, input: &str, parser_type: &str) -> String {
        match parser_type {
            "return" => {
                if input.starts_with('$') && input.chars().skip(1).all(|c| c.is_ascii_digit()) {
                    "scalar".to_string()
                } else if input.starts_with('"')
                    || input.parse::<i32>().is_ok()
                    || input == "true"
                    || input == "false"
                {
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
                } else if input.contains("boolean")
                    || input.contains("true")
                    || input.contains("false")
                {
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
