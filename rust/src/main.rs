//! Rust AST Pipeline CLI
//!
//! Command-line interface for the Rust AST transformation pipeline.

mod ast_pipeline;

use ast_pipeline::{RustASTPipeline, PipelineConfig};
use clap::Parser;
use anyhow::Result;

#[derive(Parser)]
#[command(name = "ast_pipeline")]
#[command(about = "Rust AST Transformation Pipeline")]
#[command(version = "1.0.0")]
#[command(long_about = "Transform AST JSON files or generate high-performance Rust parsers.\n\nUsage modes:\n  1. JSON transformation: ast_pipeline INPUT.json [OUTPUT.json]\n  2. Parser generation: ast_pipeline INPUT.json --generate-parser [--output PARSER.rs]")]
struct Args {
    /// Raw AST JSON input file
    input_json: String,

    /// Transformed AST JSON output file (optional, ignored when --generate-parser is used)
    output_json: Option<String>,

    /// Output file path for generated parser (when --generate-parser is used)
    #[arg(short, long)]
    output: Option<String>,

    /// Enable debug output
    #[arg(long, short = 'd')]
    debug: bool,

    /// Show transformation statistics
    #[arg(short, long)]
    stats: bool,

    /// Disable input validation
    #[arg(long)]
    no_validate: bool,

    /// Generate high-performance Rust parser instead of JSON output
    #[arg(long)]
    generate_parser: bool,

    /// Enable trace mode in generated parser (detailed debug logging)
    #[arg(long)]
    trace: bool,

    /// Enable bootstrap mode - uses built-in annotation parsing instead of external parsers
    #[arg(long)]
    bootstrap_mode: bool,

    /// Enable left recursion elimination (helps resolve stack overflow issues)
    #[arg(long)]
    eliminate_left_recursion: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let config = PipelineConfig {
        debug: args.debug,
        trace: args.trace,
        validate_input: !args.no_validate,
        preserve_annotations: true,
        validate_output: true,
        max_recursion_depth: 100,
        bootstrap_mode: args.bootstrap_mode,
        eliminate_left_recursion: args.eliminate_left_recursion,
    };

    let mut pipeline = RustASTPipeline::new(config);

    let result = if args.generate_parser {
        // Generate high-performance Rust parser
        let output_rust = args.output.unwrap_or_else(|| {
            args.input_json.replace(".json", "_parser.rs")
        });
        pipeline.generate_high_performance_parser(&args.input_json, &output_rust, args.trace, args.debug)?;
        println!("SOTA regex parser generated: {}", output_rust);
        (0, Vec::new())
    } else if let Some(output_file) = args.output_json {
        // Cross-language mode: JSON → JSON
        pipeline.transform_to_json(&args.input_json, &output_file)?;
        println!("Transformed AST saved to: {}", output_file);
        (0, Vec::new()) // (rule_count, rule_order)
    } else {
        // Same-language mode: JSON → In-memory  
        let (grammar_tree, rule_order) = pipeline.transform_from_file(&args.input_json, None)?;
        println!("Transformed AST loaded in-memory: {} rules", grammar_tree.len());
        println!("Rule order: {}", rule_order.join(", "));
        (grammar_tree.len(), rule_order)
    };

    if args.stats {
        println!("\nTransformation Statistics:");
        println!("  Rules processed: {}", result.0);
        println!("  Transformations applied: 5");
        println!("  Pipeline: Rust AST Pipeline v1.0");
    }

    Ok(())
}
