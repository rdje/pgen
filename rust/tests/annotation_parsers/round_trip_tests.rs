//! Round-Trip Testing for Return Annotation Parser
//! 
//! Tests that input -> parse -> serialize -> output produces identical results
//! This validates both parser correctness and AST completeness

use super::{test_utils, TestResults};
use serde_json::Value;

// TODO: Import actual parser when available
// mod return_annotation_parser {
//     include!("../../generated/return_annotation_parser.rs");
// }
// use return_annotation_parser::{Merged_ultimate_return_annotationParser, ParseNode};

#[cfg(test)]
mod round_trip_tests {
    use super::*;

    /// Test basic round-trip functionality
    #[test]
    fn test_basic_round_trip() {
        let test_cases = vec![
            "-> $1",
            "-> $2", 
            "-> $42",
            "-> \"hello\"",
            "-> \"world\"",
            "-> 42",
            "-> 123",
            "-> 0",
        ];

        for input in test_cases {
            println!("Testing round-trip: {}", input);
            
            // TODO: Replace with real parser
            let ast = mock_parse_return_annotation(input).unwrap();
            let output = serialize_return_annotation_ast(&ast);
            
            assert_eq!(
                normalize_annotation(input), 
                normalize_annotation(&output),
                "Round-trip failed for input: '{}'", 
                input
            );
        }
    }

    /// Test array round-trips
    #[test]
    fn test_array_round_trip() {
        let test_cases = vec![
            "-> []",
            "-> [$1]",
            "-> [$1, $2]", 
            "-> [$1, $2, $3]",
            "-> [\"a\", \"b\", \"c\"]",
            "-> [1, 2, 3]",
            "-> [$1*]",
            "-> [$2+]",
            "-> [$3?]",
            "-> [$1{2,5}]",
            "-> [$2{,10}]",
        ];

        for input in test_cases {
            println!("Testing array round-trip: {}", input);
            
            let ast = mock_parse_return_annotation(input).unwrap();
            let output = serialize_return_annotation_ast(&ast);
            
            assert_eq!(
                normalize_annotation(input), 
                normalize_annotation(&output),
                "Array round-trip failed for input: '{}'", 
                input
            );
        }
    }

    /// Test object round-trips  
    #[test]
    fn test_object_round_trip() {
        let test_cases = vec![
            "-> {}",
            "-> {key: $1}",
            "-> {name: $1, value: $2}",
            "-> {a: 1, b: 2, c: 3}",
            "-> {key: \"literal\", value: $1}",
            "-> {\"quoted_key\": $1}",
        ];

        for input in test_cases {
            println!("Testing object round-trip: {}", input);
            
            let ast = mock_parse_return_annotation(input).unwrap();
            let output = serialize_return_annotation_ast(&ast);
            
            assert_eq!(
                normalize_annotation(input), 
                normalize_annotation(&output),
                "Object round-trip failed for input: '{}'", 
                input
            );
        }
    }

    /// Test dot notation round-trips
    #[test] 
    fn test_dot_notation_round_trip() {
        let test_cases = vec![
            "-> $1.value",
            "-> $2.items",
            "-> $3.data.field", 
            "-> $1[0]",
            "-> $2[*]", 
            "-> $3[1..5]",
            "-> $4[:10]",
            "-> $5[2:]",
            "-> $1.items[0]",
            "-> $2.data[*].value",
        ];

        for input in test_cases {
            println!("Testing dot notation round-trip: {}", input);
            
            let ast = mock_parse_return_annotation(input).unwrap();
            let output = serialize_return_annotation_ast(&ast);
            
            assert_eq!(
                normalize_annotation(input), 
                normalize_annotation(&output),
                "Dot notation round-trip failed for input: '{}'", 
                input
            );
        }
    }

    /// Test nested structure round-trips
    #[test]
    fn test_nested_round_trip() {
        let test_cases = vec![
            "-> [[]]",
            "-> [[$1]]", 
            "-> [$1, [$2, $3]]",
            "-> {outer: {inner: $1}}",
            "-> {items: [$1, $2], meta: {count: 2}}",
            "-> [{id: $1}, {id: $2}]",
            "-> {expr: {type: \"binary\", left: $1, op: $2, right: $3}, meta: {line: $4}}",
        ];

        for input in test_cases {
            println!("Testing nested round-trip: {}", input);
            
            let ast = mock_parse_return_annotation(input).unwrap();
            let output = serialize_return_annotation_ast(&ast);
            
            assert_eq!(
                normalize_annotation(input), 
                normalize_annotation(&output),
                "Nested round-trip failed for input: '{}'", 
                input
            );
        }
    }

    /// Test round-trip with test data files
    #[test]
    fn test_file_data_round_trip() {
        let test_cases = test_utils::load_test_cases("return_annotations");
        let mut results = TestResults::new();
        
        for (filename, content) in test_cases {
            println!("Testing round-trips from file: {}", filename);
            
            for line in content.lines() {
                let line = line.trim();
                
                // Skip comments and empty lines
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                
                println!("  Testing round-trip: {}", line);
                
                match test_single_round_trip(line) {
                    Ok(_) => {
                        results.add_pass();
                        println!("    ✓ PASS");
                    }
                    Err(err) => {
                        results.add_fail(format!("{}: {}", line, err));
                        println!("    ✗ FAIL: {}", err);
                    }
                }
            }
        }
        
        println!("\nRound-Trip Test Results:");
        println!("  Passed: {}/{} ({:.1}%)", 
                results.passed, results.total(), results.success_rate() * 100.0);
        
        if results.failed > 0 {
            println!("  Failed cases:");
            for error in &results.errors {
                println!("    - {}", error);
            }
        }
        
        // Expect high success rate for round-trip tests
        assert!(results.success_rate() > 0.9, "More than 90% of round-trip tests should pass");
    }

    /// Test that malformed input fails consistently
    #[test]
    fn test_malformed_input_handling() {
        let malformed_cases = vec![
            "$1",           // Missing ->
            "-> ",          // Missing expression
            "-> [",         // Unbalanced bracket
            "-> {key:",     // Incomplete object
            "-> $",         // Invalid scalar ref
            "-> {,}",       // Invalid object syntax
        ];

        for input in malformed_cases {
            println!("Testing malformed input: {}", input);
            
            let parse_result = mock_parse_return_annotation(input);
            assert!(parse_result.is_err(), "Malformed input '{}' should fail to parse", input);
            
            // Make sure we get useful error information
            let error = parse_result.unwrap_err();
            assert!(!error.is_empty(), "Error should have a message for '{}'", input);
        }
    }
}

/// Test a single round-trip and return detailed result
fn test_single_round_trip(input: &str) -> Result<(), String> {
    // Parse input to AST
    let ast = mock_parse_return_annotation(input)
        .map_err(|e| format!("Parse failed: {}", e))?;
    
    // Serialize AST back to string
    let output = serialize_return_annotation_ast(&ast);
    
    // Normalize both for comparison
    let normalized_input = normalize_annotation(input);
    let normalized_output = normalize_annotation(&output);
    
    if normalized_input == normalized_output {
        Ok(())
    } else {
        Err(format!("Round-trip mismatch: '{}' != '{}'", normalized_input, normalized_output))
    }
}

/// Normalize annotation string for comparison
/// Removes extra whitespace, standardizes formatting
fn normalize_annotation(input: &str) -> String {
    input
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_lowercase()
}

/// Mock AST structure for testing
#[derive(Debug, Clone)]
enum MockAST {
    ReturnAnnotation(Box<MockAST>),
    ScalarRef(u32),
    QuotedString(String), 
    Number(u32),
    Array(Vec<MockAST>),
    Object(Vec<(String, MockAST)>),
    DotNotation { base: Box<MockAST>, path: Vec<String> },
    Quantified { element: Box<MockAST>, quantifier: String },
}

/// Mock parser that creates simplified AST
fn mock_parse_return_annotation(input: &str) -> Result<MockAST, String> {
    let trimmed = input.trim();
    
    if !trimmed.starts_with("->") {
        return Err("Expected '->' at start".to_string());
    }
    
    let content = trimmed.strip_prefix("->").unwrap().trim();
    if content.is_empty() {
        return Err("Expected expression after '->'".to_string());
    }
    
    let expr = parse_expression(content)?;
    Ok(MockAST::ReturnAnnotation(Box::new(expr)))
}

/// Parse expression part of return annotation
fn parse_expression(content: &str) -> Result<MockAST, String> {
    let content = content.trim();
    
    // Scalar reference: $1, $2, etc.
    if content.starts_with('$') && content[1..].chars().all(|c| c.is_ascii_digit()) {
        let index: u32 = content[1..].parse().map_err(|_| "Invalid scalar index")?;
        return Ok(MockAST::ScalarRef(index));
    }
    
    // Quoted string: "hello"
    if content.starts_with('"') && content.ends_with('"') && content.len() >= 2 {
        let value = content[1..content.len()-1].to_string();
        return Ok(MockAST::QuotedString(value));
    }
    
    // Number: 42, 123
    if content.chars().all(|c| c.is_ascii_digit()) {
        let value: u32 = content.parse().map_err(|_| "Invalid number")?;
        return Ok(MockAST::Number(value));
    }
    
    // Array: [], [$1], [$1, $2]
    if content.starts_with('[') && content.ends_with(']') {
        let inner = &content[1..content.len()-1].trim();
        if inner.is_empty() {
            return Ok(MockAST::Array(vec![]));
        }
        
        // Simple parsing - split by comma and parse each element
        let elements: Result<Vec<_>, _> = inner
            .split(',')
            .map(|s| parse_expression(s.trim()))
            .collect();
        return Ok(MockAST::Array(elements?));
    }
    
    // Object: {}, {key: $1}
    if content.starts_with('{') && content.ends_with('}') {
        let inner = &content[1..content.len()-1].trim();
        if inner.is_empty() {
            return Ok(MockAST::Object(vec![]));
        }
        
        // Simple parsing - this is a simplification for the mock
        // Real parser would handle nested structures properly
        return Ok(MockAST::Object(vec![])); // TODO: Implement object parsing
    }
    
    Err(format!("Unrecognized expression: {}", content))
}

/// Serialize AST back to annotation string
fn serialize_return_annotation_ast(ast: &MockAST) -> String {
    match ast {
        MockAST::ReturnAnnotation(expr) => {
            format!("-> {}", serialize_expression(expr))
        }
        _ => panic!("Expected ReturnAnnotation at root"),
    }
}

/// Serialize expression AST to string
fn serialize_expression(ast: &MockAST) -> String {
    match ast {
        MockAST::ScalarRef(index) => format!("${}", index),
        MockAST::QuotedString(value) => format!("\"{}\"", value),
        MockAST::Number(value) => format!("{}", value),
        MockAST::Array(elements) => {
            let serialized: Vec<String> = elements
                .iter()
                .map(serialize_expression)
                .collect();
            format!("[{}]", serialized.join(", "))
        }
        MockAST::Object(pairs) => {
            let serialized: Vec<String> = pairs
                .iter()
                .map(|(key, value)| format!("{}: {}", key, serialize_expression(value)))
                .collect();
            format!("{{{}}}", serialized.join(", "))
        }
        MockAST::DotNotation { base, path } => {
            format!("{}.{}", serialize_expression(base), path.join("."))
        }
        MockAST::Quantified { element, quantifier } => {
            format!("[{}{}]", serialize_expression(element), quantifier)
        }
        MockAST::ReturnAnnotation(_) => panic!("Unexpected nested ReturnAnnotation"),
    }
}
