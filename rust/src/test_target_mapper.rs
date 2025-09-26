//! Test Target Mapper
//! Maps test inputs to their corresponding Makefile targets for easy reproduction

use std::collections::HashMap;

/// Maps test inputs to their corresponding Makefile targets
pub struct TestTargetMapper {
    return_targets: HashMap<String, String>,
    semantic_targets: HashMap<String, String>,
    regex_targets: HashMap<String, String>,
}

impl TestTargetMapper {
    pub fn new() -> Self {
        let mut mapper = Self {
            return_targets: HashMap::new(),
            semantic_targets: HashMap::new(),
            regex_targets: HashMap::new(),
        };
        
        mapper.initialize_return_targets();
        mapper.initialize_semantic_targets();
        mapper.initialize_regex_targets();
        
        mapper
    }
    
    fn initialize_return_targets(&mut self) {
        // Basic scalar references
        self.return_targets.insert("$1".to_string(), "test-return-scalar-1".to_string());
        self.return_targets.insert("$2".to_string(), "test-return-scalar-2".to_string());
        self.return_targets.insert("$10".to_string(), "test-return-scalar-10".to_string());
        
        // Literals
        self.return_targets.insert("\"hello\"".to_string(), "test-return-literal-hello".to_string());
        self.return_targets.insert("42".to_string(), "test-return-literal-42".to_string());
        self.return_targets.insert("true".to_string(), "test-return-literal-true".to_string());
        
        // Arrays
        self.return_targets.insert("[$1]".to_string(), "test-return-array-simple".to_string());
        self.return_targets.insert("[$1, $2]".to_string(), "test-return-array-dual".to_string());
        self.return_targets.insert("[]".to_string(), "test-return-array-empty".to_string());
        
        // Objects
        self.return_targets.insert("{key: $1}".to_string(), "test-return-object-simple".to_string());
        self.return_targets.insert("{name: $1, value: $2}".to_string(), "test-return-object-dual".to_string());
        self.return_targets.insert("{}".to_string(), "test-return-object-empty".to_string());
        
        // Dot notation
        self.return_targets.insert("$1.value".to_string(), "test-return-dot-value".to_string());
        
        // Array access
        self.return_targets.insert("$1[0]".to_string(), "test-return-array-access-0".to_string());
    }
    
    fn initialize_semantic_targets(&mut self) {
        // Basic annotations
        self.semantic_targets.insert("@type: \"Expression\"".to_string(), "test-semantic-type".to_string());
        self.semantic_targets.insert("@precedence: 5".to_string(), "test-semantic-precedence".to_string());
        self.semantic_targets.insert("@side_effect: false".to_string(), "test-semantic-boolean".to_string());
        
        // Arrays
        self.semantic_targets.insert("@validate: [\"check_bounds\"]".to_string(), "test-semantic-array-validate".to_string());
        
        // Objects
        self.semantic_targets.insert("@cache: {ttl: 300}".to_string(), "test-semantic-object-cache".to_string());
    }
    
    fn initialize_regex_targets(&mut self) {
        // Basic patterns
        self.regex_targets.insert("hello".to_string(), "test-regex-hello".to_string());
        self.regex_targets.insert(".".to_string(), "test-regex-dot".to_string());
        self.regex_targets.insert("^start".to_string(), "test-regex-anchor-start".to_string());
        
        // Character classes
        self.regex_targets.insert("[a-z]".to_string(), "test-regex-char-class-az".to_string());
        self.regex_targets.insert("\\d".to_string(), "test-regex-digit".to_string());
        
        // Quantifiers
        self.regex_targets.insert("a*".to_string(), "test-regex-star".to_string());
        self.regex_targets.insert("b+".to_string(), "test-regex-plus".to_string());
        
        // Groups
        self.regex_targets.insert("(abc)".to_string(), "test-regex-group".to_string());
    }
    
    /// Get Makefile target for return annotation test input
    pub fn get_return_target(&self, input: &str) -> Option<&String> {
        self.return_targets.get(input)
    }
    
    /// Get Makefile target for semantic annotation test input
    pub fn get_semantic_target(&self, input: &str) -> Option<&String> {
        self.semantic_targets.get(input)
    }
    
    /// Get Makefile target for regex test input
    pub fn get_regex_target(&self, input: &str) -> Option<&String> {
        self.regex_targets.get(input)
    }
    
    /// Get reproduction command for any test
    pub fn get_reproduction_command(&self, parser_type: &str, input: &str) -> String {
        let target = match parser_type {
            "return" => self.get_return_target(input),
            "semantic" => self.get_semantic_target(input),
            "regex" => self.get_regex_target(input),
            _ => None,
        };
        
        match target {
            Some(make_target) => format!("make {}", make_target),
            None => format!("# No specific target for '{}' parser input '{}'", parser_type, input),
        }
    }
    
    /// Generate all supported test cases for return parser
    pub fn get_return_test_cases(&self) -> Vec<String> {
        self.return_targets.keys().cloned().collect()
    }
    
    /// Generate all supported test cases for semantic parser
    pub fn get_semantic_test_cases(&self) -> Vec<String> {
        self.semantic_targets.keys().cloned().collect()
    }
    
    /// Generate all supported test cases for regex parser
    pub fn get_regex_test_cases(&self) -> Vec<String> {
        self.regex_targets.keys().cloned().collect()
    }
}

impl Default for TestTargetMapper {
    fn default() -> Self {
        Self::new()
    }
}