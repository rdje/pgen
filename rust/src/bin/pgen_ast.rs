//! CLI for AST-based parser generation
//! All parser generation now uses AST-based approach (string-based removed)

use clap::Parser;
use pgen::ast_pipeline::{
    RustASTPipeline, PipelineConfig, TransformedASTJson,
    ast_generator_direct::{AstGeneratorIntegration, generate_parser_ast_based},
};
use std::fs;
use std::path::PathBuf;
use anyhow::{Result, Context};

#[derive(Parser, Debug)]
#[command(author, version, about = "AST-based EBNF parser generator", long_about = None)]
struct Args {
    /// Input JSON file from ebnf_to_json.pl
    #[arg(short, long)]
    input: PathBuf,

    /// Output Rust parser file
    #[arg(short, long)]
    output: PathBuf,

    /// Debug mode - show detailed processing information
    #[arg(short, long)]
    debug: bool,

    /// Trace mode - show even more detailed information
    #[arg(short, long)]
    trace: bool,

    /// Bootstrap mode for self-hosted parsers
    #[arg(short, long)]
    bootstrap: bool,

    /// Direct mode - skip pipeline and use AST generator directly
    #[arg(long)]
    direct: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Read input JSON
    let input_content = fs::read_to_string(&args.input)
        .context(format!("Failed to read input file: {:?}", args.input))?;

    if args.direct {
        // Direct mode - parse JSON and generate directly
        run_direct_mode(&input_content, &args)
    } else {
        // Pipeline mode - use full AST transformation pipeline
        run_pipeline_mode(&input_content, &args)
    }
}

fn run_direct_mode(input_content: &str, args: &Args) -> Result<()> {
    println!("🚀 Running in DIRECT mode - using AST generator directly");
    
    // Parse the transformed AST JSON directly
    let transformed_ast: TransformedASTJson = serde_json::from_str(input_content)
        .context("Failed to parse input as transformed AST JSON")?;
    
    // Generate parser using AST-based generator
    println!("✨ Generating parser with AST-based generator");
    let parser_code = generate_parser_ast_based(
        &transformed_ast.grammar_name,
        &transformed_ast.grammar_tree,
        &transformed_ast.rule_order,
        transformed_ast.metadata.annotations.as_ref(),
    )?;
    
    // Write output
    fs::write(&args.output, parser_code)
        .context(format!("Failed to write output file: {:?}", args.output))?;
    
    println!("✅ Successfully generated parser: {:?}", args.output);
    println!("📊 Parser size: {} bytes", parser_code.len());
    
    Ok(())
}

fn run_pipeline_mode(input_content: &str, args: &Args) -> Result<()> {
    println!("🔄 Running in PIPELINE mode - full AST transformation");
    
    // Configure pipeline
    let config = PipelineConfig {
        debug: args.debug,
        trace: args.trace,
        bootstrap_mode: args.bootstrap,
        preserve_annotations: true,
        validate_input: true,
        validate_output: true,
        max_recursion_depth: 100,
        eliminate_left_recursion: true,
    };
    
    // Create pipeline
    let mut pipeline = RustASTPipeline::with_config(config);
    
    // Process the input
    let result = if input_content.trim_start().starts_with('[') {
        // Raw AST format
        println!("📥 Processing raw AST JSON");
        pipeline.transform_from_json(input_content)?
    } else {
        // Already transformed
        println!("📥 Input appears to be already transformed");
        serde_json::from_str(input_content)?
    };
    
    // Generate parser using AST-based generator
    let integration = AstGeneratorIntegration::new()
        .with_debug(args.debug);
    
    let parser_code = integration.generate_parser(&result)?;
    
    // Write output
    fs::write(&args.output, parser_code)
        .context(format!("Failed to write output file: {:?}", args.output))?;
    
    println!("✅ Successfully generated parser: {:?}", args.output);
    println!("📊 Rules processed: {}", result.grammar_tree.len());
    println!("📊 Parser size: {} bytes", parser_code.len());
    println!("🔧 Backend used: AST-based (syn/quote)");
    
    Ok(())
}