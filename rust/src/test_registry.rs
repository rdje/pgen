//! Centralized Test Registry System
//! Automatically discovers and synchronizes test cases across all files

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub input: String,
    pub description: String,
    pub parser_type: String,
    pub category: String,
    pub expected_result: TestExpectation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestExpectation {
    Success,
    Failure(String),
    ParseResult(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRegistry {
    pub return_tests: Vec<TestCase>,
    pub semantic_tests: Vec<TestCase>,
    pub regex_tests: Vec<TestCase>,
    pub version: String,
    pub last_updated: String,
}

impl TestRegistry {
    pub fn new() -> Self {
        Self {
            return_tests: Self::default_return_tests(),
            semantic_tests: Self::default_semantic_tests(),
            regex_tests: Self::default_regex_tests(),
            version: "1.0.0".to_string(),
            last_updated: chrono::Utc::now().to_rfc3339(),
        }
    }
    
    fn default_return_tests() -> Vec<TestCase> {
        vec![
            TestCase {
                input: "$1".to_string(),
                description: "Basic scalar reference".to_string(),
                parser_type: "return".to_string(),
                category: "scalar".to_string(),
                expected_result: TestExpectation::Success,
            },
            TestCase {
                input: "$2".to_string(),
                description: "Second scalar reference".to_string(),
                parser_type: "return".to_string(),
                category: "scalar".to_string(),
                expected_result: TestExpectation::Success,
            },
            TestCase {
                input: "$10".to_string(),
                description: "Double-digit scalar reference".to_string(),
                parser_type: "return".to_string(),
                category: "scalar".to_string(),
                expected_result: TestExpectation::Success,
            },
            TestCase {
                input: "\"hello\"".to_string(),
                description: "String literal".to_string(),
                parser_type: "return".to_string(),
                category: "literal".to_string(),
                expected_result: TestExpectation::Success,
            },
            TestCase {
                input: "42".to_string(),
                description: "Numeric literal".to_string(),
                parser_type: "return".to_string(),
                category: "literal".to_string(),
                expected_result: TestExpectation::Success,
            },
            TestCase {
                input: "true".to_string(),
                description: "Boolean literal".to_string(),
                parser_type: "return".to_string(),
                category: "literal".to_string(),
                expected_result: TestExpectation::Success,
            },
            TestCase {
                input: "[$1]".to_string(),
                description: "Simple array".to_string(),
                parser_type: "return".to_string(),
                category: "array".to_string(),
                expected_result: TestExpectation::Success,
            },
            TestCase {
                input: "[$1, $2]".to_string(),
                description: "Array with multiple elements".to_string(),
                parser_type: "return".to_string(),
                category: "array".to_string(),
                expected_result: TestExpectation::Success,
            },
            TestCase {
                input: "[]".to_string(),
                description: "Empty array".to_string(),
                parser_type: "return".to_string(),
                category: "array".to_string(),
                expected_result: TestExpectation::Success,
            },
            TestCase {
                input: "{key: $1}".to_string(),
                description: "Simple object".to_string(),
                parser_type: "return".to_string(),
                category: "object".to_string(),
                expected_result: TestExpectation::Success,
            },
            TestCase {
                input: "{name: $1, value: $2}".to_string(),
                description: "Object with multiple properties".to_string(),
                parser_type: "return".to_string(),
                category: "object".to_string(),
                expected_result: TestExpectation::Success,
            },
            TestCase {
                input: "{}".to_string(),
                description: "Empty object".to_string(),
                parser_type: "return".to_string(),
                category: "object".to_string(),
                expected_result: TestExpectation::Success,
            },
            TestCase {
                input: "$1.value".to_string(),
                description: "Dot notation access".to_string(),
                parser_type: "return".to_string(),
                category: "accessor".to_string(),
                expected_result: TestExpectation::Success,
            },
            TestCase {
                input: "$1[0]".to_string(),
                description: "Array index access".to_string(),
                parser_type: "return".to_string(),
                category: "accessor".to_string(),
                expected_result: TestExpectation::Success,
            },
        ]
    }
    
    fn default_semantic_tests() -> Vec<TestCase> {
        vec![
            TestCase {
                input: "@type: \"Expression\"".to_string(),
                description: "Type annotation".to_string(),
                parser_type: "semantic".to_string(),
                category: "type".to_string(),
                expected_result: TestExpectation::Success,
            },
            TestCase {
                input: "@precedence: 5".to_string(),
                description: "Precedence annotation".to_string(),
                parser_type: "semantic".to_string(),
                category: "precedence".to_string(),
                expected_result: TestExpectation::Success,
            },
            TestCase {
                input: "@side_effect: false".to_string(),
                description: "Boolean annotation".to_string(),
                parser_type: "semantic".to_string(),
                category: "boolean".to_string(),
                expected_result: TestExpectation::Success,
            },
            TestCase {
                input: "@validate: [\"check_bounds\"]".to_string(),
                description: "Array annotation".to_string(),
                parser_type: "semantic".to_string(),
                category: "array".to_string(),
                expected_result: TestExpectation::Success,
            },
            TestCase {
                input: "@cache: {ttl: 300}".to_string(),
                description: "Object annotation".to_string(),
                parser_type: "semantic".to_string(),
                category: "object".to_string(),
                expected_result: TestExpectation::Success,
            },
        ]
    }
    
    fn default_regex_tests() -> Vec<TestCase> {
        vec![
            TestCase {
                input: "hello".to_string(),
                description: "Simple literal pattern".to_string(),
                parser_type: "regex".to_string(),
                category: "literal".to_string(),
                expected_result: TestExpectation::Success,
            },
            TestCase {
                input: ".".to_string(),
                description: "Dot metacharacter".to_string(),
                parser_type: "regex".to_string(),
                category: "metachar".to_string(),
                expected_result: TestExpectation::Success,
            },
            TestCase {
                input: "^start".to_string(),
                description: "Start anchor".to_string(),
                parser_type: "regex".to_string(),
                category: "anchor".to_string(),
                expected_result: TestExpectation::Success,
            },
            TestCase {
                input: "[a-z]".to_string(),
                description: "Character class".to_string(),
                parser_type: "regex".to_string(),
                category: "charclass".to_string(),
                expected_result: TestExpectation::Success,
            },
            TestCase {
                input: "\\d".to_string(),
                description: "Digit shorthand".to_string(),
                parser_type: "regex".to_string(),
                category: "shorthand".to_string(),
                expected_result: TestExpectation::Success,
            },
            TestCase {
                input: "a*".to_string(),
                description: "Zero or more quantifier".to_string(),
                parser_type: "regex".to_string(),
                category: "quantifier".to_string(),
                expected_result: TestExpectation::Success,
            },
            TestCase {
                input: "b+".to_string(),
                description: "One or more quantifier".to_string(),
                parser_type: "regex".to_string(),
                category: "quantifier".to_string(),
                expected_result: TestExpectation::Success,
            },
            TestCase {
                input: "(abc)".to_string(),
                description: "Capturing group".to_string(),
                parser_type: "regex".to_string(),
                category: "group".to_string(),
                expected_result: TestExpectation::Success,
            },
        ]
    }
    
    /// Get all test cases for a specific parser type
    pub fn get_tests_for_parser(&self, parser_type: &str) -> Vec<&TestCase> {
        match parser_type {
            "return" => self.return_tests.iter().collect(),
            "semantic" => self.semantic_tests.iter().collect(),
            "regex" => self.regex_tests.iter().collect(),
            _ => vec![],
        }
    }
    
    /// Generate a unique make target name for a test case
    pub fn generate_make_target(&self, test: &TestCase) -> String {
        let category_part = test.category.replace("_", "-");
        let input_part = self.sanitize_for_target(&test.input);
        format!("test-{}-{}-{}", test.parser_type, category_part, input_part)
    }
    
    /// Generate a unique function name for a test case
    pub fn generate_function_name(&self, test: &TestCase) -> String {
        let category_part = test.category.replace("-", "_");
        let input_part = self.sanitize_for_function(&test.input);
        format!("test_{}_{}__{}", test.parser_type, category_part, input_part)
    }
    
    fn sanitize_for_target(&self, input: &str) -> String {
        input
            .chars()
            .map(|c| match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' => c,
                '$' => 's',
                '"' => 'q',
                '[' | ']' => 'a',
                '{' | '}' => 'o',
                ':' => 'c',
                '.' => 'd',
                ',' => 'm',
                ' ' => '_',
                _ => 'x',
            })
            .collect::<String>()
            .trim_matches('_')
            .to_lowercase()
    }
    
    fn sanitize_for_function(&self, input: &str) -> String {
        input
            .chars()
            .map(|c| match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' => c,
                '$' => 's',
                '"' => 'q',
                '[' | ']' => 'a',
                '{' | '}' => 'o',
                ':' => 'c',
                '.' => 'd',
                ',' => 'm',
                ' ' => '_',
                _ => 'x',
            })
            .collect::<String>()
            .trim_matches('_')
            .to_lowercase()
    }
    
    /// Add a new test case
    pub fn add_test(&mut self, test: TestCase) {
        match test.parser_type.as_str() {
            "return" => self.return_tests.push(test),
            "semantic" => self.semantic_tests.push(test),
            "regex" => self.regex_tests.push(test),
            _ => eprintln!("Unknown parser type: {}", test.parser_type),
        }
        self.last_updated = chrono::Utc::now().to_rfc3339();
    }
    
    /// Remove a test case by input and parser type
    pub fn remove_test(&mut self, parser_type: &str, input: &str) -> bool {
        let removed = match parser_type {
            "return" => {
                let initial_len = self.return_tests.len();
                self.return_tests.retain(|t| t.input != input);
                self.return_tests.len() < initial_len
            }
            "semantic" => {
                let initial_len = self.semantic_tests.len();
                self.semantic_tests.retain(|t| t.input != input);
                self.semantic_tests.len() < initial_len
            }
            "regex" => {
                let initial_len = self.regex_tests.len();
                self.regex_tests.retain(|t| t.input != input);
                self.regex_tests.len() < initial_len
            }
            _ => false,
        };
        
        if removed {
            self.last_updated = chrono::Utc::now().to_rfc3339();
        }
        
        removed
    }
    
    /// Get all test cases across all parser types
    pub fn get_all_tests(&self) -> Vec<&TestCase> {
        let mut all_tests = Vec::new();
        all_tests.extend(&self.return_tests);
        all_tests.extend(&self.semantic_tests);
        all_tests.extend(&self.regex_tests);
        all_tests
    }
    
    /// Save registry to JSON file
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    /// Load registry from JSON file
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let registry: TestRegistry = serde_json::from_str(&json)?;
        Ok(registry)
    }
}

impl Default for TestRegistry {
    fn default() -> Self {
        Self::new()
    }
}