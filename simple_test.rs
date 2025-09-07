use std::fs;

mod generated {
    pub mod semantic_annotation_parser;
}

use generated::semantic_annotation_parser::Semantic_annotationParser;

fn main() {
    println!("🚀 Starting simple parser test");
    
    let test_input = "42";
    println!("📝 Input: '{}'", test_input);
    
    let mut parser = Semantic_annotationParser::with_debug(test_input);
    
    println!("🔍 About to parse...");
    match parser.parse() {
        Ok(result) => {
            println!("✅ Parse successful!");
            println!("📊 Result: {:?}", result);
        },
        Err(error) => {
            println!("❌ Parse failed: {:?}", error);
        }
    }
    
    println!("\n📋 Debug output:");
    for (i, line) in parser.debug_output().iter().enumerate() {
        println!("{}: {}", i, line);
    }
}
