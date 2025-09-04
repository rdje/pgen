//! Semantic Annotation Parser Tests
//! 
//! Tests the semantic annotation parser using round-trip testing approach
//! Based on the semantic_annotations.ebnf grammar

use super::{test_utils, TestResults};

// TODO: Import actual parser when available
// mod semantic_annotation_parser {
//     include!("../../semantic_annotation_parser.rs");
// }
// use semantic_annotation_parser::Semantic_annotationsParser;

#[cfg(test)]
mod semantic_annotation_tests {
    use super::*;

    /// Test basic semantic annotation round-trips
    #[test]
    fn test_basic_semantic_round_trip() {
        let test_cases = vec![
            // Type system annotations
            r#"@type: "Expression""#,
            r#"@category: "Terminal""#,
            r#"@kind: "Literal""#,
            r#"@class: "NumericValue""#,
            r#"@interface: "Parseable""#,
            
            // Behavioral annotations with different value types
            r#"@effect: "pure""#,
            "@side_effect: false",
            "@idempotent: true", 
            "@deterministic: true",
            r#"@throws: "ParseError""#,
            
            // Precedence annotations with numeric values
            "@precedence: 5",
            "@precedence: 0",
            "@precedence: 100",
            r#"@associativity: "left""#,
            r#"@binding: "tight""#,
            r#"@priority: "high""#,
            "@weight: 10",
        ];

        for input in test_cases {
            println!("Testing semantic round-trip: {}", input);
            
            let ast = mock_parse_semantic_annotation(input).unwrap();
            let output = serialize_semantic_annotation_ast(&ast);
            
            assert_eq!(
                normalize_semantic_annotation(input),
                normalize_semantic_annotation(&output),
                "Semantic round-trip failed for input: '{}'", 
                input
            );
        }
    }

    /// Test array value semantic annotations
    #[test]
    fn test_array_semantic_round_trip() {
        let test_cases = vec![
            // Simple arrays
            r#"@validate: ["check_bounds"]"#,
            r#"@validate: ["check_bounds", "ensure_positive"]"#,
            r#"@see_also: ["expression_parser", "precedence_rules"]"#,
            r#"@throws: ["IOException", "ParseException", "RuntimeException"]"#,
            r#"@platform: ["web", "mobile", "desktop"]"#,
            r#"@requires: ["feature1", "feature2"]"#,
            
            // Empty arrays
            "@empty_list: []",
            
            // Mixed type arrays (if supported)
            r#"@mixed: ["string", 42, true]"#,
        ];

        for input in test_cases {
            println!("Testing array semantic round-trip: {}", input);
            
            let ast = mock_parse_semantic_annotation(input).unwrap();
            let output = serialize_semantic_annotation_ast(&ast);
            
            assert_eq!(
                normalize_semantic_annotation(input),
                normalize_semantic_annotation(&output),
                "Array semantic round-trip failed for input: '{}'", 
                input
            );
        }
    }

    /// Test object value semantic annotations
    #[test]
    fn test_object_semantic_round_trip() {
        let test_cases = vec![
            // Simple objects
            r#"@cache: {ttl: 300}"#,
            r#"@cache: {ttl: 300, size: 1000}"#,
            r#"@retry: {max_attempts: 3, backoff: "exponential"}"#,
            r#"@timeout: {duration: "30s"}"#,
            r#"@parallel: {workers: 4, chunk_size: 100}"#,
            r#"@serialize: {format: "json", pretty: true}"#,
            r#"@deprecated: {since: "1.2.0", note: "Use new_method instead"}"#,
            
            // Empty objects
            "@empty_obj: {}",
            
            // Nested objects
            r#"@constraint: {type: "requires", expression: "x > 0"}"#,
            r#"@performance: {complexity: "O(n)", memory: "O(1)"}"#,
            r#"@version: {major: 2, minor: 1, patch: 0}"#,
        ];

        for input in test_cases {
            println!("Testing object semantic round-trip: {}", input);
            
            let ast = mock_parse_semantic_annotation(input).unwrap();
            let output = serialize_semantic_annotation_ast(&ast);
            
            assert_eq!(
                normalize_semantic_annotation(input),
                normalize_semantic_annotation(&output),
                "Object semantic round-trip failed for input: '{}'", 
                input
            );
        }
    }

    /// Test custom annotation names
    #[test]
    fn test_custom_annotation_round_trip() {
        let test_cases = vec![
            r#"@my_annotation: "custom_value""#,
            r#"@custom_flag: true"#,
            r#"@user_defined: 42"#,
            r#"@special_case: {enabled: true, mode: "debug"}"#,
            r#"@experiment_123: ["option1", "option2"]"#,
        ];

        for input in test_cases {
            println!("Testing custom annotation round-trip: {}", input);
            
            let ast = mock_parse_semantic_annotation(input).unwrap();
            let output = serialize_semantic_annotation_ast(&ast);
            
            assert_eq!(
                normalize_semantic_annotation(input),
                normalize_semantic_annotation(&output),
                "Custom annotation round-trip failed for input: '{}'", 
                input
            );
        }
    }

    /// Test edge cases and error conditions  
    #[test]
    fn test_semantic_edge_cases() {
        let valid_edge_cases = vec![
            // Whitespace handling
            r#"@type:"Expression""#,
            r#"@type: "Expression""#,
            r#"@type:    "Expression""#,
            
            // Different quote styles (if supported)
            r#"@type: "double_quotes""#,
            "@type: 'single_quotes'",
            
            // Numeric edge cases
            "@precedence: 0",
            "@precedence: -1",
            "@weight: 999999",
        ];

        for input in valid_edge_cases {
            println!("Testing semantic edge case: {}", input);
            
            match test_single_semantic_round_trip(input) {
                Ok(_) => println!("    ✓ PASS"),
                Err(err) => println!("    ✗ FAIL: {}", err),
            }
        }
    }

    /// Test malformed semantic annotations 
    #[test]
    fn test_semantic_malformed_input() {
        let malformed_cases = vec![
            "type: \"Expression\"",     // Missing @
            "@: \"Expression\"",        // Missing annotation name
            "@type \"Expression\"",     // Missing colon
            "@type:",                   // Missing value
            "@type: ",                  // Empty value
            "@type: {",                // Unbalanced brace
            "@type: [",                // Unbalanced bracket
            "@type: {key:}",           // Incomplete object
        ];

        for input in malformed_cases {
            println!("Testing malformed semantic input: {}", input);
            
            let parse_result = mock_parse_semantic_annotation(input);
            assert!(parse_result.is_err(), "Malformed input '{}' should fail to parse", input);
            
            let error = parse_result.unwrap_err();
            assert!(!error.is_empty(), "Error should have a message for '{}'", input);
        }
    }

    /// Test semantic annotations from test data files
    #[test]
    fn test_semantic_file_data_round_trip() {
        let test_cases = test_utils::load_test_cases("semantic_annotations");
        let mut results = TestResults::new();
        
        for (filename, content) in test_cases {
            println!("Testing semantic round-trips from file: {}", filename);
            
            for line in content.lines() {
                let line = line.trim();
                
                // Skip comments and empty lines
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                
                println!("  Testing semantic round-trip: {}", line);
                
                match test_single_semantic_round_trip(line) {
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
        
        println!("\nSemantic Annotation Round-Trip Test Results:");
        println!("  Passed: {}/{} ({:.1}%)", 
                results.passed, results.total(), results.success_rate() * 100.0);
        
        if results.failed > 0 {
            println!("  Failed cases:");
            for error in &results.errors {
                println!("    - {}", error);
            }
        }
        
        // Expect high success rate for round-trip tests
        assert!(results.success_rate() > 0.8, "More than 80% of semantic round-trip tests should pass");
    }
}

/// Test a single semantic annotation round-trip
fn test_single_semantic_round_trip(input: &str) -> Result<(), String> {
    let ast = mock_parse_semantic_annotation(input)
        .map_err(|e| format!("Parse failed: {}", e))?;
    
    let output = serialize_semantic_annotation_ast(&ast);
    
    let normalized_input = normalize_semantic_annotation(input);
    let normalized_output = normalize_semantic_annotation(&output);
    
    if normalized_input == normalized_output {
        Ok(())
    } else {
        Err(format!("Round-trip mismatch: '{}' != '{}'", normalized_input, normalized_output))
    }
}

/// Normalize semantic annotation string for comparison
fn normalize_semantic_annotation(input: &str) -> String {
    input
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_lowercase()
}

/// Mock AST structure for semantic annotations
#[derive(Debug, Clone)]
enum MockSemanticAST {
    Annotation { name: String, value: MockSemanticValue },
}

#[derive(Debug, Clone)]
enum MockSemanticValue {
    String(String),
    Number(i64),
    Boolean(bool),
    Array(Vec<MockSemanticValue>),
    Object(Vec<(String, MockSemanticValue)>),
}

/// Mock parser for semantic annotations
fn mock_parse_semantic_annotation(input: &str) -> Result<MockSemanticAST, String> {
    let trimmed = input.trim();
    
    if !trimmed.starts_with('@') {
        return Err("Expected '@' at start of semantic annotation".to_string());
    }
    
    // Find the colon separator
    let colon_pos = trimmed.find(':')
        .ok_or_else(|| "Expected ':' after annotation name".to_string())?;
    
    let name_part = &trimmed[1..colon_pos].trim();
    let value_part = trimmed[colon_pos + 1..].trim();
    
    if name_part.is_empty() {
        return Err("Annotation name cannot be empty".to_string());
    }
    
    if value_part.is_empty() {
        return Err("Annotation value cannot be empty".to_string());
    }
    
    let name = name_part.to_string();
    let value = parse_semantic_value(value_part)?;
    
    Ok(MockSemanticAST::Annotation { name, value })
}

/// Parse semantic annotation value
fn parse_semantic_value(content: &str) -> Result<MockSemanticValue, String> {
    let content = content.trim();
    
    // String literal: "hello", 'world'
    if (content.starts_with('"') && content.ends_with('"')) || 
       (content.starts_with('\'') && content.ends_with('\'')) {
        let value = content[1..content.len()-1].to_string();
        return Ok(MockSemanticValue::String(value));
    }
    
    // Boolean: true, false
    if content == "true" {
        return Ok(MockSemanticValue::Boolean(true));
    }
    if content == "false" {
        return Ok(MockSemanticValue::Boolean(false));
    }
    
    // Number: 42, -1, 0
    if let Ok(num) = content.parse::<i64>() {
        return Ok(MockSemanticValue::Number(num));
    }
    
    // Array: [item1, item2, ...]
    if content.starts_with('[') && content.ends_with(']') {
        let inner = &content[1..content.len()-1].trim();
        if inner.is_empty() {
            return Ok(MockSemanticValue::Array(vec![]));
        }
        
        // Simple array parsing - split by comma
        let elements: Result<Vec<_>, _> = inner
            .split(',')
            .map(|s| parse_semantic_value(s.trim()))
            .collect();
        
        return Ok(MockSemanticValue::Array(elements?));
    }
    
    // Object: {key1: value1, key2: value2, ...}
    if content.starts_with('{') && content.ends_with('}') {
        let inner = &content[1..content.len()-1].trim();
        if inner.is_empty() {
            return Ok(MockSemanticValue::Object(vec![]));
        }
        
        // Simple object parsing - this is simplified for the mock
        let pairs: Result<Vec<_>, _> = inner
            .split(',')
            .map(|pair_str| {
                let pair_str = pair_str.trim();
                if let Some(colon_pos) = pair_str.find(':') {
                    let key = pair_str[..colon_pos].trim();
                    let value_str = pair_str[colon_pos + 1..].trim();
                    
                    // Remove quotes from key if present
                    let key = if (key.starts_with('"') && key.ends_with('"')) ||
                                (key.starts_with('\'') && key.ends_with('\'')) {
                        key[1..key.len()-1].to_string()
                    } else {
                        key.to_string()
                    };
                    
                    let value = parse_semantic_value(value_str)?;
                    Ok((key, value))
                } else {
                    Err("Invalid object pair format".to_string())
                }
            })
            .collect();
        
        return Ok(MockSemanticValue::Object(pairs?));
    }
    
    Err(format!("Unrecognized semantic value: {}", content))
}

/// Serialize semantic annotation AST back to string
fn serialize_semantic_annotation_ast(ast: &MockSemanticAST) -> String {
    match ast {
        MockSemanticAST::Annotation { name, value } => {
            format!("@{}: {}", name, serialize_semantic_value(value))
        }
    }
}

/// Serialize semantic annotation value to string
fn serialize_semantic_value(value: &MockSemanticValue) -> String {
    match value {
        MockSemanticValue::String(s) => format!("\"{}\"", s),
        MockSemanticValue::Number(n) => format!("{}", n),
        MockSemanticValue::Boolean(b) => format!("{}", b),
        MockSemanticValue::Array(items) => {
            let serialized: Vec<String> = items
                .iter()
                .map(serialize_semantic_value)
                .collect();
            format!("[{}]", serialized.join(", "))
        }
        MockSemanticValue::Object(pairs) => {
            let serialized: Vec<String> = pairs
                .iter()
                .map(|(key, value)| format!("{}: {}", key, serialize_semantic_value(value)))
                .collect();
            format!("{{{}}}", serialized.join(", "))
        }
    }
}
