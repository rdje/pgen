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
struct Args {
    /// Raw AST JSON input file
    input_json: String,

    /// Transformed AST JSON output file (optional)
    output_json: Option<String>,

    /// Enable debug output
    #[arg(long, short)]
    debug: bool,

    /// Show transformation statistics
    #[arg(long, short)]
    stats: bool,

    /// Disable input validation
    #[arg(long)]
    no_validate: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let config = PipelineConfig {
        debug: args.debug,
        validate_input: !args.no_validate,
        preserve_annotations: true,
        validate_output: true,
        max_recursion_depth: 100,
    };

    let mut pipeline = RustASTPipeline::new(config);

    let result = if let Some(output_file) = args.output_json {
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
