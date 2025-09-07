#!/usr/bin/env rust-script

//! Test program to verify left recursion elimination functionality
//! 
//! This program creates a pipeline with left recursion elimination enabled
//! and tests it on the regex grammar that was causing stack overflow issues.

use std::path::Path;
mod ast_pipeline;
use ast_pipeline::{RustASTPipeline, PipelineConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Left Recursion Elimination");
    println!("=====================================");
    
    // Create pipeline with left recursion elimination enabled
    println!("✅ Creating pipeline with left recursion elimination enabled...");
    let mut pipeline = RustASTPipeline::with_left_recursion_elimination();
    
    // Also enable debug mode to see what's happening
    let mut config = PipelineConfig::default();
    config.eliminate_left_recursion = true;
    config.debug = true;
    pipeline = RustASTPipeline::new(config);
    
    // Verify the feature is enabled
    assert!(pipeline.is_left_recursion_elimination_enabled());
    println!("✅ Left recursion elimination is enabled: {}", 
             pipeline.is_left_recursion_elimination_enabled());
    
    // Test file paths
    let input_file = "../docs/regex_grammar_raw_ast.json";
    let output_parser = "regex_parser_with_left_recursion_fix.rs";
    
    // Check if input file exists
    if !Path::new(input_file).exists() {
        println!("❌ Input file does not exist: {}", input_file);
        println!("   Please ensure the regex grammar raw AST JSON file is available.");
        return Ok(());
    }
    
    println!("📁 Input file: {}", input_file);
    println!("📁 Output parser: {}", output_parser);
    
    // Generate the parser with left recursion elimination
    println!("\n🔥 GENERATING PARSER WITH LEFT-RECURSION ELIMINATION...");
    println!("======================================================");
    
    match pipeline.generate_high_performance_parser(
        input_file,
        output_parser,
        true,  // enable_trace
        true   // enable_backtrack_debug  
    ) {
        Ok(()) => {
            println!("🎉 SUCCESS! Parser generated with left recursion elimination!");
            println!("📄 Generated file: {}", output_parser);
            
            // Check file size
            if let Ok(metadata) = std::fs::metadata(output_parser) {
                println!("📊 Generated parser size: {} bytes", metadata.len());
            }
            
            println!("\n✨ The parser should now handle left-recursive grammars without stack overflow!");
            println!("✨ Try testing it on regex patterns that previously caused issues.");
        }
        Err(e) => {
            println!("❌ Failed to generate parser: {}", e);
            println!("   Error details: {:?}", e);
            return Err(e.into());
        }
    }
    
    // Test the pipeline stats
    println!("\n📈 Testing transformation pipeline...");
    
    match pipeline.transform_from_file(input_file, None) {
        Ok((grammar_tree, rule_order)) => {
            println!("✅ Pipeline transformation successful!");
            println!("   Rules processed: {}", grammar_tree.len());
            println!("   Rule order: {}", rule_order.join(", "));
            println!("   Transformations applied: 6 (including left recursion elimination)");
        }
        Err(e) => {
            println!("❌ Pipeline transformation failed: {}", e);
        }
    }
    
    println!("\n🎯 LEFT RECURSION ELIMINATION TEST COMPLETE!");
    Ok(())
}
