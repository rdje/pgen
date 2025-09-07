#!/usr/bin/env rust-script

//! Test to verify external annotation parsers are actually being called with debug/trace
//! This will help us determine if the AST pipeline is correctly using the external parsers

use std::collections::HashMap;

// Import our AST pipeline
mod src {
    pub mod ast_pipeline;
    pub mod ast_pipeline { 
        pub mod high_performance_generator;
    }
}

use src::ast_pipeline::{PipelineConfig, RustASTPipeline, TokenValue, Token, TokenSequence, RawAST};

fn main() {
    println!("=== Testing External Parser Debug/Trace Output ===\n");

    // Create a config with debug and trace enabled
    let config = PipelineConfig {
        debug: true,
        trace: true,
        preserve_annotations: true,
        validate_input: true,
        validate_output: true,
        max_recursion_depth: 100,
        bootstrap_mode: false, // Force external parsers
    };

    let mut pipeline = RustASTPipeline::new(config);

    // Create a simple raw AST with semantic and return annotations
    let raw_ast: RawAST = vec![
        // Rule definition with semantic and return annotations
        vec![
            vec![TokenValue::String("rule".to_string()), TokenValue::String("test_rule".to_string())],
            vec![
                TokenValue::String("semantic_annotation".to_string()), 
                TokenValue::Array(vec!["type".to_string(), "test_construct".to_string()])
            ],
            vec![
                TokenValue::String("return_scalar".to_string()),
                TokenValue::String("{key: $1}".to_string())
            ],
            vec![TokenValue::String("identifier".to_string()), TokenValue::String("test".to_string())],
        ]
    ];

    println!("Raw AST input:");
    println!("{:#?}\n", raw_ast);

    // Transform the raw AST - this should trigger the external parsers with debug/trace
    println!("=== Starting AST Transformation ===");
    match pipeline.transform_raw_ast(&raw_ast) {
        Ok((grammar_tree, rule_order)) => {
            println!("\n=== Transformation Successful ===");
            println!("Grammar tree: {:#?}", grammar_tree);
            println!("Rule order: {:?}", rule_order);
        }
        Err(e) => {
            println!("Transformation failed: {}", e);
        }
    }
}
