use std::collections::HashMap;
use ebnf_pipeline::{RustASTPipeline, PipelineConfig, ASTNode};

/// Test that the regex grammar transforms successfully and contains expected productions
#[test]
fn test_regex_grammar_transformation() {
    let config = PipelineConfig::default();
    let mut pipeline = RustASTPipeline::new(config);
    
    // Path to the regex raw AST JSON
    let raw_ast_path = "../grammars/regex_raw.json";
    
    // Skip test if the raw AST doesn't exist (not checked into repo yet)
    if !std::path::Path::new(raw_ast_path).exists() {
        println!("Skipping regex grammar test - {} not found", raw_ast_path);
        return;
    }
    
    // Transform the grammar
    let result = pipeline.transform_from_file(raw_ast_path, None);
    assert!(result.is_ok(), "Failed to transform regex grammar: {:?}", result.err());
    
    let (grammar_tree, rule_order) = result.unwrap();
    
    // Verify basic properties
    assert!(!grammar_tree.is_empty(), "Grammar tree should not be empty");
    assert!(!rule_order.is_empty(), "Rule order should not be empty");
    
    // Verify entry point exists
    assert!(grammar_tree.contains_key("regex"), "Grammar should have 'regex' entry point");
    
    // Verify core regex components exist
    let expected_rules = vec![
        "pattern", "alternation", "concatenation", "piece", "atom",
        "quantifier", "quant_base", "quant_suffix", "counted_quantifier",
        "literal", "dot", "anchor", "escape", "backreference", "char_class",
        "group", "atomic_group", "lookaround",
        "inline_modifiers", "scoped_inline_modifiers", "modifier_seq", "modifier_char",
        "conditional", "yes_branch", "no_branch", "condition"
    ];
    
    for rule in expected_rules {
        assert!(grammar_tree.contains_key(rule), "Grammar should contain rule: {}", rule);
    }
}

/// Test that code block features are properly parsed
#[test]
fn test_code_block_features() {
    let config = PipelineConfig::default();
    let mut pipeline = RustASTPipeline::new(config);
    
    let raw_ast_path = "../grammars/regex_raw.json";
    if !std::path::Path::new(raw_ast_path).exists() {
        println!("Skipping code block test - {} not found", raw_ast_path);
        return;
    }
    
    let result = pipeline.transform_from_file(raw_ast_path, None);
    assert!(result.is_ok(), "Failed to transform regex grammar");
    
    let (grammar_tree, _) = result.unwrap();
    
    // Verify code block structures exist
    let code_block_rules = vec![
        "code_block",
        "code_block_plain", 
        "code_block_lang",
        "code_lang",
        "code_content",
        "code_element",
        "code_string_double",
        "code_string_single", 
        "code_balanced_braces",
        "code_escaped_char",
        "code_regular_char"
    ];
    
    for rule in code_block_rules {
        assert!(grammar_tree.contains_key(rule), "Grammar should contain code block rule: {}", rule);
    }
    
    // Verify code_block is an alternation between plain and language-tagged forms
    if let Some(code_block) = grammar_tree.get("code_block") {
        match code_block {
            ASTNode::Or { alternatives } => {
                assert_eq!(alternatives.len(), 2, "code_block should have 2 alternatives: plain and lang");
            },
            _ => panic!("code_block should be an Or node with alternatives")
        }
    }
    
    // Verify code_lang supports lua and js
    if let Some(code_lang) = grammar_tree.get("code_lang") {
        match code_lang {
            ASTNode::Or { alternatives } => {
                assert_eq!(alternatives.len(), 2, "code_lang should support lua and js");
            },
            _ => panic!("code_lang should be an Or node")
        }
    }
}

/// Test that advanced regex features are supported
#[test] 
fn test_advanced_regex_features() {
    let config = PipelineConfig::default();
    let mut pipeline = RustASTPipeline::new(config);
    
    let raw_ast_path = "../grammars/regex_raw.json";
    if !std::path::Path::new(raw_ast_path).exists() {
        println!("Skipping advanced features test - {} not found", raw_ast_path);
        return;
    }
    
    let result = pipeline.transform_from_file(raw_ast_path, None);
    assert!(result.is_ok(), "Failed to transform regex grammar");
    
    let (grammar_tree, _) = result.unwrap();
    
    // Verify lookaround constructs
    let lookaround_rules = vec![
        "lookaround",
        "lookahead_pos",    // (?= ... )
        "lookahead_neg",    // (?! ... )  
        "lookbehind_pos",   // (?<= ... )
        "lookbehind_neg"    // (?<! ... )
    ];
    
    for rule in lookaround_rules {
        assert!(grammar_tree.contains_key(rule), "Grammar should contain lookaround rule: {}", rule);
    }
    
    // Verify group types
    let group_rules = vec![
        "group",
        "capturing_group",      // ( ... )
        "noncapturing_group",   // (?: ... )
        "named_group",          // (?<name> ... ) or (?'name' ... )
        "atomic_group"          // (?> ... )
    ];
    
    for rule in group_rules {
        assert!(grammar_tree.contains_key(rule), "Grammar should contain group rule: {}", rule);
    }
    
    // Verify escape sequences
    let escape_rules = vec![
        "escape", "escape_unit", "simple_escape",
        "hex_escape", "unicode_escape", "octal_escape",
        "control_escape", "property_escape"
    ];
    
    for rule in escape_rules {
        assert!(grammar_tree.contains_key(rule), "Grammar should contain escape rule: {}", rule);
    }
    
    // Verify character classes
    let char_class_rules = vec![
        "char_class", "class_body", "class_item", "class_range",
        "class_atom", "posix_class", "posix_name"
    ];
    
    for rule in char_class_rules {
        assert!(grammar_tree.contains_key(rule), "Grammar should contain char class rule: {}", rule);
    }
}

/// Test that inline modifiers are comprehensively supported  
#[test]
fn test_inline_modifiers() {
    let config = PipelineConfig::default();
    let mut pipeline = RustASTPipeline::new(config);
    
    let raw_ast_path = "../grammars/regex_raw.json";
    if !std::path::Path::new(raw_ast_path).exists() {
        println!("Skipping inline modifiers test - {} not found", raw_ast_path);
        return;
    }
    
    let result = pipeline.transform_from_file(raw_ast_path, None);
    assert!(result.is_ok(), "Failed to transform regex grammar");
    
    let (grammar_tree, _) = result.unwrap();
    
    // Verify modifier infrastructure
    let modifier_rules = vec![
        "inline_modifiers",        // (?i-mx)
        "scoped_inline_modifiers", // (?i-mx: ... )
        "modifier_seq",            // i-mx
        "modifier_group",          // i or mx  
        "modifier_char"            // individual modifier letters
    ];
    
    for rule in modifier_rules {
        assert!(grammar_tree.contains_key(rule), "Grammar should contain modifier rule: {}", rule);
    }
    
    // Verify modifier_char includes comprehensive PCRE2 flags
    if let Some(modifier_char) = grammar_tree.get("modifier_char") {
        match modifier_char {
            ASTNode::Or { alternatives } => {
                // Should include: i, m, s, x, U, J, u, a, d, S, A, X, R, n
                assert!(alternatives.len() >= 10, "modifier_char should support many PCRE2 flags, got {}", alternatives.len());
            },
            _ => panic!("modifier_char should be an Or node with multiple alternatives")
        }
    }
}

/// Test that conditional constructs are properly supported
#[test]
fn test_conditional_constructs() {
    let config = PipelineConfig::default();
    let mut pipeline = RustASTPipeline::new(config);
    
    let raw_ast_path = "../grammars/regex_raw.json";
    if !std::path::Path::new(raw_ast_path).exists() {
        println!("Skipping conditional test - {} not found", raw_ast_path);
        return;
    }
    
    let result = pipeline.transform_from_file(raw_ast_path, None);
    assert!(result.is_ok(), "Failed to transform regex grammar");
    
    let (grammar_tree, _) = result.unwrap();
    
    // Verify conditional structure: (?(cond)yes|no)
    let conditional_rules = vec![
        "conditional",
        "yes_branch", 
        "no_branch",
        "condition"
    ];
    
    for rule in conditional_rules {
        assert!(grammar_tree.contains_key(rule), "Grammar should contain conditional rule: {}", rule);
    }
    
    // Verify condition can be lookaround, name_ref, or digits
    if let Some(condition) = grammar_tree.get("condition") {
        match condition {
            ASTNode::Or { alternatives } => {
                assert_eq!(alternatives.len(), 3, "condition should have 3 alternatives: lookaround, name_ref, digits");
            },
            _ => panic!("condition should be an Or node")
        }
    }
}

/// Test transformation statistics and metadata
#[test]
fn test_transformation_metadata() {
    let config = PipelineConfig::default();
    let mut pipeline = RustASTPipeline::new(config);
    
    let raw_ast_path = "../grammars/regex_raw.json";
    if !std::path::Path::new(raw_ast_path).exists() {
        println!("Skipping metadata test - {} not found", raw_ast_path);
        return;
    }
    
    let output_path = "../grammars/test_regex_output.json";
    let result = pipeline.transform_from_file(raw_ast_path, Some(output_path));
    assert!(result.is_ok(), "Failed to transform and save regex grammar");
    
    // Verify output file exists
    assert!(std::path::Path::new(output_path).exists(), "Output file should be created");
    
    // Read and parse output to verify structure
    let output_content = std::fs::read_to_string(output_path).expect("Failed to read output file");
    let output_json: serde_json::Value = serde_json::from_str(&output_content).expect("Failed to parse output JSON");
    
    // Verify required fields
    assert!(output_json.get("grammar_name").is_some(), "Output should have grammar_name");
    assert!(output_json.get("grammar_tree").is_some(), "Output should have grammar_tree");
    assert!(output_json.get("rule_order").is_some(), "Output should have rule_order");
    assert!(output_json.get("metadata").is_some(), "Output should have metadata");
    
    let metadata = output_json.get("metadata").unwrap();
    assert_eq!(metadata.get("format").unwrap().as_str().unwrap(), "transformed_ast");
    assert_eq!(metadata.get("source_format").unwrap().as_str().unwrap(), "raw_ast");
    assert_eq!(metadata.get("transformer").unwrap().as_str().unwrap(), "Rust AST Pipeline v1.0");
    
    let stats = metadata.get("stats").unwrap();
    assert!(stats.get("rules_processed").unwrap().as_u64().unwrap() > 50, "Should process many rules");
    assert_eq!(stats.get("transformations_applied").unwrap().as_u64().unwrap(), 5);
    
    // Clean up test file
    std::fs::remove_file(output_path).ok();
}
