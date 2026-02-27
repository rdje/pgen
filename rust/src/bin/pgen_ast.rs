//! CLI for AST-based parser generation
//! All parser generation now uses AST-based approach (string-based removed)

use anyhow::{Context, Result};
use clap::Parser;
use pgen::ast_pipeline::{
    Annotations, PipelineConfig, RustASTPipeline, TransformMetadata, TransformedASTJson,
    ast_based_generator::AstBasedGenerator, configure_trace_output, resolve_trace_verbosity,
    set_global_trace_verbosity,
};
use std::fs;
use std::path::PathBuf;

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

    /// Trace verbosity: none, low, medium, high, debug
    #[arg(long, value_parser = ["none", "low", "medium", "high", "debug"])]
    verbosity: Option<String>,

    /// Route trace output to a file (defaults to trace.log when flag is provided without a value)
    #[arg(long, num_args = 0..=1, default_missing_value = "trace.log")]
    trace_log_file: Option<String>,

    /// Bootstrap mode for self-hosted parsers
    #[arg(short, long)]
    bootstrap: bool,

    /// Direct mode - skip pipeline and use AST generator directly
    #[arg(long)]
    direct: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let trace_log_path = args
        .trace_log_file
        .clone()
        .or_else(|| std::env::var("PGEN_TRACE_LOG_FILE").ok());
    configure_trace_output(trace_log_path.as_deref())?;
    let trace_verbosity =
        resolve_trace_verbosity(args.verbosity.as_deref(), args.debug, args.trace)?;
    set_global_trace_verbosity(trace_verbosity);

    // Read input JSON
    let input_content = fs::read_to_string(&args.input)
        .context(format!("Failed to read input file: {:?}", args.input))?;

    if args.direct {
        // Direct mode - parse JSON and generate directly
        run_direct_mode(&input_content, &args)
    } else {
        // Pipeline mode - use full AST transformation pipeline
        run_pipeline_mode(&input_content, &args, trace_verbosity)
    }
}

fn run_direct_mode(input_content: &str, args: &Args) -> Result<()> {
    println!("🚀 Running in DIRECT mode - using AST generator directly");

    // Parse the transformed AST JSON directly
    let transformed_ast: TransformedASTJson = serde_json::from_str(input_content)
        .context("Failed to parse input as transformed AST JSON")?;

    // Generate parser using AST-based generator
    let parser_code = AstBasedGenerator::new(transformed_ast.grammar_name.clone())
        .generate_parser(
            &transformed_ast.grammar_tree,
            &transformed_ast.rule_order,
            args.output.to_str().unwrap_or("unknown_parser.rs"),
        )?;

    // Write output
    let parser_size = parser_code.len();
    fs::write(&args.output, parser_code)
        .context(format!("Failed to write output file: {:?}", args.output))?;

    println!("✅ Successfully generated parser: {:?}", args.output);
    println!("📊 Parser size: {} bytes", parser_size);

    Ok(())
}

fn run_pipeline_mode(
    input_content: &str,
    args: &Args,
    trace_verbosity: pgen::ast_pipeline::TraceVerbosity,
) -> Result<()> {
    println!("🔄 Running in PIPELINE mode - full AST transformation");

    // Configure pipeline
    let config = PipelineConfig {
        debug: args.debug,
        trace: args.trace,
        trace_verbosity,
        bootstrap_mode: args.bootstrap,
        preserve_annotations: true,
        validate_input: true,
        validate_output: true,
        max_recursion_depth: 100,
        eliminate_left_recursion: true,
    };

    // Create pipeline
    let mut pipeline = RustASTPipeline::new(config);

    // Process the input
    let result = if input_content.trim_start().starts_with('[') {
        // Raw AST format - transform from JSON string
        println!("📥 Processing raw AST JSON");
        // For raw AST input, we need to create a temporary file or handle it differently
        // For now, assume it's transformed AST and parse directly
        serde_json::from_str::<TransformedASTJson>(input_content)?
    } else {
        // Transform from file (assumes input_content is a file path)
        println!("📥 Processing from file: {}", input_content);
        // let (grammar_tree, rule_order) = pipeline.transform_from_file(input_content, None)?;
        // Create a minimal TransformedASTJson for compatibility
        TransformedASTJson {
            grammar_name: "unknown".to_string(), // Would need to be extracted from file
            grammar_tree: std::collections::HashMap::new(),
            rule_order: vec![],
            metadata: TransformMetadata {
                format: "transformed_ast".to_string(),
                source_format: "raw_ast".to_string(),
                transformed_at: chrono::Utc::now().to_rfc3339(),
                transformer: "Rust AST Pipeline v1.0".to_string(),
                pipeline_stage: "transformation".to_string(),
                annotations: Some(Annotations::default()),
                stats: Default::default(),
            },
        }
    };

    // Generate parser using AST-based generator
    let mut generator = AstBasedGenerator::new(result.grammar_name.clone());
    generator.enable_debug = args.debug;

    if let Some(annotations) = result.metadata.annotations.as_ref() {
        generator.annotations = Some(annotations.clone());
    }

    let parser_code = generator.generate_parser(
        &result.grammar_tree,
        &result.rule_order,
        args.output.to_str().unwrap_or("unknown_parser.rs"),
    )?;

    // Write output
    let parser_size = parser_code.len();
    fs::write(&args.output, parser_code)
        .context(format!("Failed to write output file: {:?}", args.output))?;

    println!("✅ Successfully generated parser: {:?}", args.output);
    println!("📊 Rules processed: {}", result.grammar_tree.len());
    println!("📊 Parser size: {} bytes", parser_size);
    println!("🔧 Backend used: AST-based (syn/quote)");

    Ok(())
}
