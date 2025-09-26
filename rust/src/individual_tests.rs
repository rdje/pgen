//! Individual Test Functions
//! Allows running each stress test case individually for debugging and focused testing

use crate::ast_pipeline::return_annotation_parser::Return_annotationParser;
use crate::ast_pipeline::semantic_annotation_parser::Semantic_annotationParser;
use std::time::Instant;

#[cfg(test)]
mod individual_tests {
    use super::*;

    /// Individual test function for return annotation parser
    /// Can be called with: PGEN_TEST_INPUT="$1" cargo test test_individual_return_test -- --nocapture
    #[test]
    fn test_individual_return_test() {
        let test_input = std::env::var("PGEN_TEST_INPUT").unwrap_or_else(|_| "$1".to_string());

        println!("\n{}", "=".repeat(80));
        println!("🔍 INDIVIDUAL RETURN ANNOTATION PARSER TEST");
        println!("{}", "=".repeat(80));
        println!("📋 Test Input: '{}'", test_input);
        println!("🔧 Parser: Return Annotation Parser");
        println!("📁 Generated Parser: /Users/richarddje/Documents/github/pgen/generated/return_annotation_parser.rs");
        println!("{}", "=".repeat(80));

        let mut parser = Return_annotationParser::with_debug(&test_input);
        let start = Instant::now();

        match parser.parse() {
            Ok(ast) => {
                let parse_time = start.elapsed();
                println!("✅ PARSE SUCCESS in {:.3}ms", parse_time.as_secs_f64() * 1000.0);
                println!("📊 AST Rule: {}", ast.rule_name);
                println!("📊 AST Span: {:?}", ast.span);
                println!("📊 AST Content: {:?}", ast.content);

                // Print debug trace
                let debug_output = parser.debug_output();
                if !debug_output.is_empty() {
                    println!("\n🔍 COMPLETE DEBUG TRACE ({} steps):", debug_output.len());
                    for (step, msg) in debug_output.iter().enumerate() {
                        println!("   {:4}: {}", step + 1, msg);
                    }
                }

                println!("\n✅ RETURN PARSER: Individual test PASSED for '{}'", test_input);
            }
            Err(e) => {
                let parse_time = start.elapsed();
                println!("❌ PARSE FAILED in {:.3}ms: {}", parse_time.as_secs_f64() * 1000.0, e);

                // Print debug trace for failure analysis
                let debug_output = parser.debug_output();
                if !debug_output.is_empty() {
                    println!("\n🔍 FAILURE DEBUG TRACE ({} steps):", debug_output.len());
                    for (step, msg) in debug_output.iter().enumerate() {
                        println!("   {:4}: {}", step + 1, msg);
                    }
                }

                println!("❌ RETURN PARSER: Individual test FAILED for '{}'", test_input);
                panic!("Individual test failed for input: {}", test_input);
            }
        }
    }

    /// Individual test function for semantic annotation parser
    /// Can be called with: PGEN_TEST_INPUT="@type: \"Expression\"" cargo test test_individual_semantic_test -- --nocapture
    #[test]
    fn test_individual_semantic_test() {
        let test_input = std::env::var("PGEN_TEST_INPUT").unwrap_or_else(|_| "@type: \"Expression\"".to_string());

        println!("\n{}", "=".repeat(80));
        println!("🔍 INDIVIDUAL SEMANTIC ANNOTATION PARSER TEST");
        println!("{}", "=".repeat(80));
        println!("📋 Test Input: '{}'", test_input);
        println!("🔧 Parser: Semantic Annotation Parser");
        println!("📁 Generated Parser: /Users/richarddje/Documents/github/pgen/generated/semantic_annotation_parser.rs");
        println!("{}", "=".repeat(80));

        let mut parser = Semantic_annotationParser::with_debug(&test_input);
        let start = Instant::now();

        match parser.parse() {
            Ok(ast) => {
                let parse_time = start.elapsed();
                println!("✅ PARSE SUCCESS in {:.3}ms", parse_time.as_secs_f64() * 1000.0);
                println!("📊 AST Rule: {}", ast.rule_name);
                println!("📊 AST Span: {:?}", ast.span);
                println!("📊 AST Content: {:?}", ast.content);

                // Print debug trace
                let debug_output = parser.debug_output();
                if !debug_output.is_empty() {
                    println!("\n🔍 COMPLETE DEBUG TRACE ({} steps):", debug_output.len());
                    for (step, msg) in debug_output.iter().enumerate() {
                        println!("   {:4}: {}", step + 1, msg);
                    }
                }

                println!("\n✅ SEMANTIC PARSER: Individual test PASSED for '{}'", test_input);
            }
            Err(e) => {
                let parse_time = start.elapsed();
                println!("❌ PARSE FAILED in {:.3}ms: {}", parse_time.as_secs_f64() * 1000.0, e);

                // Print debug trace for failure analysis
                let debug_output = parser.debug_output();
                if !debug_output.is_empty() {
                    println!("\n🔍 FAILURE DEBUG TRACE ({} steps):", debug_output.len());
                    for (step, msg) in debug_output.iter().enumerate() {
                        println!("   {:4}: {}", step + 1, msg);
                    }
                }

                println!("❌ SEMANTIC PARSER: Individual test FAILED for '{}'", test_input);
                panic!("Individual test failed for input: {}", test_input);
            }
        }
    }

    /// Individual test function for regex parser
    /// Can be called with: PGEN_TEST_INPUT="hello" cargo test test_individual_regex_test -- --nocapture
    #[test]  
    fn test_individual_regex_test() {
        let test_input = std::env::var("PGEN_TEST_INPUT").unwrap_or_else(|_| "hello".to_string());

        println!("\n{}", "=".repeat(80));
        println!("🔍 INDIVIDUAL REGEX PARSER TEST");
        println!("{}", "=".repeat(80));
        println!("📋 Test Input: '{}'", test_input);
        println!("🔧 Parser: Regex Parser");
        println!("📁 Generated Parser: /Users/richarddje/Documents/github/pgen/generated/regex_parser.rs");
        println!("{}", "=".repeat(80));

        // Note: This is a placeholder - the actual regex parser instantiation will depend on the generated parser interface
        println!("⚠️  REGEX PARSER TEST: Implementation needed once regex parser interface is confirmed");
        println!("📋 Test input '{}' ready for testing", test_input);
        
        // TODO: Uncomment and modify once regex parser is available
        /*
        let mut parser = RegexParser::with_debug(test_input);
        let start = Instant::now();

        match parser.parse() {
            Ok(ast) => {
                let parse_time = start.elapsed();
                println!("✅ PARSE SUCCESS in {:.3}ms", parse_time.as_secs_f64() * 1000.0);
                println!("📊 AST Rule: {}", ast.rule_name);
                println!("📊 AST Span: {:?}", ast.span);
                println!("📊 AST Content: {:?}", ast.content);

                // Print debug trace
                let debug_output = parser.debug_output();
                if !debug_output.is_empty() {
                    println!("\n🔍 COMPLETE DEBUG TRACE ({} steps):", debug_output.len());
                    for (step, msg) in debug_output.iter().enumerate() {
                        println!("   {:4}: {}", step + 1, msg);
                    }
                }

                println!("\n✅ REGEX PARSER: Individual test PASSED for '{}'", test_input);
            }
            Err(e) => {
                let parse_time = start.elapsed();
                println!("❌ PARSE FAILED in {:.3}ms: {}", parse_time.as_secs_f64() * 1000.0, e);

                // Print debug trace for failure analysis
                let debug_output = parser.debug_output();
                if !debug_output.is_empty() {
                    println!("\n🔍 FAILURE DEBUG TRACE ({} steps):", debug_output.len());
                    for (step, msg) in debug_output.iter().enumerate() {
                        println!("   {:4}: {}", step + 1, msg);
                    }
                }

                println!("❌ REGEX PARSER: Individual test FAILED for '{}'", test_input);
                panic!("Individual test failed for input: {}", test_input);
            }
        }
        */
    }

    /// Test all available individual test cases for return parser
    #[test]
    fn test_all_return_individual_cases() {
        let test_cases = vec![
            // Basic scalar references
            "$1", "$2", "$10", "$99",
            // Literals
            "\"hello\"", "\"test string\"", "42", "123", "true", "false",
            // Simple arrays
            "[$1]", "[$2]", "[$1, $2]", "[\"item1\", \"item2\"]", "[42, 100]", "[]",
            // Simple objects
            "{key: $1}", "{name: $1}", "{value: $2}", "{name: $1, value: $2}", "{id: 42, name: \"test\"}", "{}",
            // Dot notation
            "$1.value", "$1.name", "$1.data",
            // Array indexing
            "$1[0]", "$1[1]", "$2[0]",
        ];

        println!("\n{}", "=".repeat(80));
        println!("🚀 RUNNING ALL INDIVIDUAL RETURN PARSER TEST CASES");
        println!("{}", "=".repeat(80));
        println!("📊 Total test cases: {}", test_cases.len());
        
        let mut passed = 0;
        let mut failed = 0;

        for (i, test_input) in test_cases.iter().enumerate() {
            println!("\n--- Test {}/{}: {} ---", i + 1, test_cases.len(), test_input);
            
            let mut parser = Return_annotationParser::with_debug(test_input);
            match parser.parse() {
                Ok(_) => {
                    passed += 1;
                    println!("✅ PASSED");
                }
                Err(e) => {
                    failed += 1;
                    println!("❌ FAILED: {}", e);
                }
            }
        }

        println!("\n{}", "=".repeat(80));
        println!("🎯 RETURN PARSER INDIVIDUAL TESTS SUMMARY");
        println!("✅ Passed: {}", passed);
        println!("❌ Failed: {}", failed);
        println!("📊 Success Rate: {:.1}%", passed as f64 / test_cases.len() as f64 * 100.0);
        println!("{}", "=".repeat(80));
    }

    /// Test all available individual test cases for semantic parser
    #[test]
    fn test_all_semantic_individual_cases() {
        let test_cases = vec![
            // Basic annotations
            "@type: \"Expression\"", "@category: \"Terminal\"", "@kind: \"Literal\"", 
            "@effect: \"pure\"", "@precedence: 5", "@associativity: \"left\"",
            // Boolean values
            "@side_effect: false", "@idempotent: true", "@deterministic: true",
            // Numeric values
            "@precedence: 0", "@precedence: -1", "@precedence: 100", "@weight: 10", "@weight: 999999",
            // String arrays
            "@validate: [\"check_bounds\"]", "@validate: [\"check_bounds\", \"ensure_positive\"]",
            "@see_also: [\"expression_parser\", \"precedence_rules\"]",
            // Objects
            "@cache: {ttl: 300}", "@cache: {ttl: 300, size: 1000}",
            "@retry: {max_attempts: 3, backoff: \"exponential\"}",
            // Empty containers
            "@empty_list: []", "@empty_obj: {}",
        ];

        println!("\n{}", "=".repeat(80));
        println!("🚀 RUNNING ALL INDIVIDUAL SEMANTIC PARSER TEST CASES");
        println!("{}", "=".repeat(80));
        println!("📊 Total test cases: {}", test_cases.len());
        
        let mut passed = 0;
        let mut failed = 0;

        for (i, test_input) in test_cases.iter().enumerate() {
            println!("\n--- Test {}/{}: {} ---", i + 1, test_cases.len(), test_input);
            
            let mut parser = Semantic_annotationParser::with_debug(test_input);
            match parser.parse() {
                Ok(_) => {
                    passed += 1;
                    println!("✅ PASSED");
                }
                Err(e) => {
                    failed += 1;
                    println!("❌ FAILED: {}", e);
                }
            }
        }

        println!("\n{}", "=".repeat(80));
        println!("🎯 SEMANTIC PARSER INDIVIDUAL TESTS SUMMARY");
        println!("✅ Passed: {}", passed);
        println!("❌ Failed: {}", failed);
        println!("📊 Success Rate: {:.1}%", passed as f64 / test_cases.len() as f64 * 100.0);
        println!("{}", "=".repeat(80));
    }
}