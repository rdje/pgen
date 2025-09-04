//! Return Annotation Parser Tests
//! 
//! Tests the return annotation parser using the merged_ultimate_return_annotation.ebnf grammar
//! Focuses on testing parser capabilities with real test data

use super::{test_utils, TestResults};

// TODO: Import the generated return annotation parser when available
// mod return_annotation_parser {
//     include!("../../generated/return_annotation_parser.rs");
// }
// use return_annotation_parser::Merged_ultimate_return_annotationParser;

#[cfg(test)]
mod return_annotation_tests {
    use super::*;

    /// Test simple return annotations from test data
    #[test]
    fn test_simple_return_annotations() {
        let test_cases = test_utils::load_test_cases("return_annotations");
        let mut results = TestResults::new();
        
        for (filename, content) in test_cases {
            println!("Testing file: {}", filename);
            
            for line in content.lines() {
                let line = line.trim();
                
                // Skip comments and empty lines
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                
                println!("  Testing: {}", line);
                
                // TODO: Replace with actual parser when available
                let parse_result = mock_parse_return_annotation(line);
                
                if parse_result {
                    results.add_pass();
                    println!("    ✓ PASS");
                } else {
                    results.add_fail(format!("Failed to parse: {}", line));
                    println!("    ✗ FAIL");
                }
            }
        }
        
        println!("\nReturn Annotation Parser Test Results:");
        println!("  Passed: {}/{} ({:.1}%)", 
                results.passed, results.total(), results.success_rate() * 100.0);
        
        if results.failed > 0 {
            println!("  Failed cases:");
            for error in &results.errors {
                println!("    - {}", error);
            }
        }
        
        // For now, we expect the mock to work on basic cases
        assert!(results.success_rate() > 0.5, "More than 50% of tests should pass");
    }

    /// Test specific return annotation patterns
    #[test]
    fn test_specific_return_patterns() {
        let test_cases = vec![
            ("-> $1", true),
            ("-> \"literal\"", true),
            ("-> 42", true),
            ("-> [$1]", true),
            ("-> [$1, $2]", true),
            ("-> {key: $1}", true),
            ("-> {name: $1, value: $2}", true),
            ("-> $1.value", true),
            ("-> $2[0]", true),
            ("-> [$1*]", true),
            ("-> [$2+]", true),
            // Invalid cases
            ("$1", false),        // Missing ->
            ("-> ", false),       // Missing expression
            ("-> [", false),      // Malformed
            ("-> {key:", false),  // Incomplete
        ];
        
        for (input, should_pass) in test_cases {
            let result = mock_parse_return_annotation(input);
            assert_eq!(result, should_pass, "Test case: '{}'", input);
        }
    }
}

/// Mock parser for testing until real parser is available
fn mock_parse_return_annotation(input: &str) -> bool {
    let trimmed = input.trim();
    
    // Must start with ->
    if !trimmed.starts_with("->") {
        return false;
    }
    
    // Must have content after ->
    let content = trimmed.strip_prefix("->").unwrap().trim();
    if content.is_empty() {
        return false;
    }
    
    // Basic validation - check for balanced brackets
    let mut brace_count = 0;
    let mut bracket_count = 0;
    
    for ch in content.chars() {
        match ch {
            '{' => brace_count += 1,
            '}' => brace_count -= 1,
            '[' => bracket_count += 1,
            ']' => bracket_count -= 1,
            _ => {}
        }
        
        // Early exit on negative counts (unbalanced)
        if brace_count < 0 || bracket_count < 0 {
            return false;
        }
    }
    
    // Must end balanced
    brace_count == 0 && bracket_count == 0
}

/// Utility functions for return annotation testing
pub mod return_annotation_test_utils {
    /// Test a batch of return annotation inputs using mock parser
    pub fn test_return_annotation_batch(inputs: &[(&str, bool)]) {
        for (input, should_succeed) in inputs {
            let result = super::mock_parse_return_annotation(input);
            assert_eq!(
                result, *should_succeed,
                "Return annotation '{}' - expected: {}, got: {}",
                input, should_succeed, result
            );
        }
    }

}
