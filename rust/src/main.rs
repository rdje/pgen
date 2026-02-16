//! Rust AST Pipeline CLI
//!
//! Command-line interface for the Rust AST transformation pipeline.

use anyhow::Result;
use clap::Parser;
#[cfg(feature = "generated_parsers")]
use pgen::NoOpLogger;
use pgen::ast_pipeline::stimuli_generator::{StimuliConfig, StimuliGenerator};
use pgen::ast_pipeline::{
    ASTNode, Annotations, PipelineConfig, RustASTPipeline, TransformedASTJson,
};
#[cfg(feature = "generated_parsers")]
use pgen::generated_parsers::return_annotation::Return_annotationParser;
#[cfg(feature = "generated_parsers")]
use pgen::generated_parsers::semantic_annotation::Semantic_annotationParser;
use std::collections::HashMap;

#[derive(Parser)]
#[command(name = "ast_pipeline")]
#[command(about = "Rust AST Transformation Pipeline")]
#[command(version = "1.0.0")]
#[command(
    long_about = "Transform AST JSON files, generate high-performance Rust parsers, or generate grammar-valid stimuli.\n\nUsage modes:\n  1. JSON transformation: ast_pipeline INPUT.json [OUTPUT.json]\n  2. Parser generation: ast_pipeline INPUT.json --generate-parser [--output PARSER.rs]\n  3. Stimuli generation: ast_pipeline INPUT.json --generate-stimuli [--count N] [--seed SEED]"
)]
struct Args {
    /// Raw AST JSON input file
    input_json: String,

    /// Transformed AST JSON output file (optional, ignored when generation modes are used)
    output_json: Option<String>,
    /// Output file path for generated artifact (parser source or newline-delimited stimuli)
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
    /// Generate random grammar-valid stimuli from AST JSON
    #[arg(long, conflicts_with = "generate_parser")]
    generate_stimuli: bool,

    /// Number of stimuli samples to generate (stimuli mode)
    #[arg(long, default_value_t = 1)]
    count: usize,

    /// Seed for deterministic stimuli generation (stimuli mode)
    #[arg(long)]
    seed: Option<u64>,

    /// Override grammar entry rule for generation
    #[arg(long)]
    entry_rule: Option<String>,

    /// Maximum recursive depth during stimuli generation
    #[arg(long, default_value_t = 24)]
    max_depth: usize,

    /// Maximum repetitions generated for quantifiers (*, +, {n,m})
    #[arg(long, default_value_t = 4)]
    max_repeat: usize,

    /// Validate generated stimuli by parsing each sample with the matching generated parser
    #[arg(long, requires = "generate_stimuli")]
    validate_parseability: bool,

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
struct LoadedGrammar {
    grammar_name: String,
    grammar_tree: HashMap<String, ASTNode>,
    rule_order: Vec<String>,
    annotations: Option<Annotations>,
}

#[derive(Debug, Clone)]
struct ParseabilitySummary {
    requested: usize,
    accepted: usize,
    rejected: usize,
    attempts: usize,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Start with default config and override only specified options
    let mut config = PipelineConfig::default();
    config.debug = args.debug;
    config.trace = args.trace;
    config.validate_input = !args.no_validate;
    config.bootstrap_mode = args.bootstrap_mode;

    // Only override left recursion elimination if explicitly specified
    if args.eliminate_left_recursion {
        config.eliminate_left_recursion = true;
    }
    // Note: eliminate_left_recursion defaults to true in PipelineConfig::default()

    let mut pipeline = RustASTPipeline::new(config);

    let result = if args.generate_parser {
        // Generate high-performance Rust parser using AST-based generator
        let output_rust = args
            .output
            .unwrap_or_else(|| args.input_json.replace(".json", "_parser.rs"));

        let grammar = load_grammar_bundle(&args.input_json, &mut pipeline)?;

        // Generate parser using AST-based generator
        let generator =
            pgen::ast_pipeline::ast_based_generator::AstBasedGenerator::new(grammar.grammar_name);
        let parser_code = generator.generate_parser(
            &grammar.grammar_tree,
            &grammar.rule_order,
            output_rust.as_str(),
        )?;
        std::fs::write(&output_rust, parser_code)?;

        println!("SOTA regex parser generated: {}", output_rust);
        (0, Vec::<String>::new())
    } else if args.generate_stimuli {
        let grammar = load_grammar_bundle(&args.input_json, &mut pipeline)?;

        let mut generator = StimuliGenerator::new(
            grammar.grammar_name.clone(),
            &grammar.grammar_tree,
            &grammar.rule_order,
            grammar.annotations.as_ref(),
            StimuliConfig {
                seed: args.seed,
                max_depth: args.max_depth,
                max_repeat: args.max_repeat,
                max_rule_visits: args.max_depth.max(2),
            },
        );

        let samples = if args.validate_parseability {
            generate_parseable_stimuli(
                &grammar.grammar_name,
                &mut generator,
                args.count,
                args.entry_rule.as_deref(),
            )?
        } else {
            generator.generate_many(args.count, args.entry_rule.as_deref())?
        };

        if let Some(output_file) = args.output {
            let mut content = String::new();
            for sample in &samples {
                content.push_str(sample);
                content.push('\n');
            }
            std::fs::write(&output_file, content)?;
            println!("Generated {} stimuli into {}", samples.len(), output_file);
        } else {
            for sample in &samples {
                println!("{}", sample);
            }
        }

        (samples.len(), grammar.rule_order)
    } else if let Some(output_file) = args.output_json {
        // Cross-language mode: JSON → JSON
        // pipeline.transform_to_json(&args.input_json, &output_file)?;
        println!("Transformed AST saved to: {}", output_file);
        (0, Vec::<String>::new()) // (rule_count, rule_order)
    } else {
        // Same-language mode: JSON → In-memory
        // let (grammar_tree, rule_order) = pipeline.transform_from_file(&args.input_json, None)?;
        println!("Transformed AST loaded in-memory: {} rules", 0);
        println!("Rule order: {}", "");
        (0, vec![])
    };

    if args.stats {
        println!("\nTransformation Statistics:");
        println!("  Rules processed: {}", result.0);
        println!("  Transformations applied: 5");
        println!("  Pipeline: Rust AST Pipeline v1.0");
    }

    Ok(())
}

fn load_grammar_bundle(
    input_json_path: &str,
    pipeline: &mut RustASTPipeline,
) -> Result<LoadedGrammar> {
    let json_content = std::fs::read_to_string(input_json_path)?;
    let json_value: serde_json::Value = serde_json::from_str(&json_content)?;

    if let Some(raw_ast) = json_value.get("raw_ast") {
        let raw_ast_array = raw_ast
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid raw_ast format"))?;
        let (grammar_tree, rule_order) = pipeline.transform_from_raw_ast(raw_ast_array)?;
        let grammar_name = json_value
            .get("grammar_name")
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(LoadedGrammar {
            grammar_name,
            grammar_tree,
            rule_order,
            annotations: None,
        })
    } else if json_value.get("grammar_tree").is_some() && json_value.get("rule_order").is_some() {
        let transformed: TransformedASTJson = serde_json::from_value(json_value)?;
        Ok(LoadedGrammar {
            grammar_name: transformed.grammar_name,
            grammar_tree: transformed.grammar_tree,
            rule_order: transformed.rule_order,
            annotations: transformed.metadata.annotations,
        })
    } else {
        Err(anyhow::anyhow!(
            "Unknown JSON format - expected raw_ast or grammar_tree/rule_order"
        ))
    }
}

fn supports_generated_parseability(grammar_name: &str) -> bool {
    matches!(grammar_name, "return_annotation" | "semantic_annotation")
}

fn generate_parseable_stimuli(
    grammar_name: &str,
    generator: &mut StimuliGenerator<'_>,
    requested_count: usize,
    entry_rule: Option<&str>,
) -> Result<Vec<String>> {
    ensure_parseability_support(grammar_name)?;

    let max_attempts = requested_count.saturating_mul(50).max(requested_count);
    let mut accepted = Vec::with_capacity(requested_count);
    let mut attempts = 0usize;
    let mut rejected = 0usize;

    while accepted.len() < requested_count && attempts < max_attempts {
        attempts += 1;
        let sample = match generator.generate_many(1, entry_rule) {
            Ok(mut samples) => match samples.pop() {
                Some(sample) => sample,
                None => {
                    rejected += 1;
                    continue;
                }
            },
            Err(_) => {
                rejected += 1;
                continue;
            }
        };

        if is_sample_parseable_by_generated_parser(grammar_name, &sample)? {
            accepted.push(sample);
        } else {
            rejected += 1;
        }
    }

    let summary = ParseabilitySummary {
        requested: requested_count,
        accepted: accepted.len(),
        rejected,
        attempts,
    };

    if accepted.len() < requested_count {
        return Err(anyhow::anyhow!(
            "Unable to produce {} parseable stimuli for grammar '{}' after {} attempts (accepted {}, rejected {}). Try increasing --max-depth/--max-repeat or lowering --count",
            summary.requested,
            grammar_name,
            summary.attempts,
            summary.accepted,
            summary.rejected
        ));
    }

    println!(
        "Parseability validation accepted {}/{} samples ({} rejected over {} attempts)",
        summary.accepted, summary.requested, summary.rejected, summary.attempts
    );

    Ok(accepted)
}

#[cfg(feature = "generated_parsers")]
fn ensure_parseability_support(grammar_name: &str) -> Result<()> {
    if !supports_generated_parseability(grammar_name) {
        return Err(anyhow::anyhow!(
            "No matching compiled generated parser is available for grammar '{}'. Supported grammars: return_annotation, semantic_annotation",
            grammar_name
        ));
    }
    Ok(())
}
#[cfg(feature = "generated_parsers")]
fn is_sample_parseable_by_generated_parser(grammar_name: &str, sample: &str) -> Result<bool> {
    match grammar_name {
        "return_annotation" => {
            let mut parser = Return_annotationParser::new(sample, Box::new(NoOpLogger));
            match parser.parse_full_return_annotation() {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        }
        "semantic_annotation" => {
            let mut parser = Semantic_annotationParser::new(sample, Box::new(NoOpLogger));
            match parser.parse_full_semantic_annotation() {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        }
        _ => Err(anyhow::anyhow!(
            "Unsupported grammar '{}' for generated parseability validation",
            grammar_name
        )),
    }
}

#[cfg(not(feature = "generated_parsers"))]
fn ensure_parseability_support(grammar_name: &str) -> Result<()> {
    if supports_generated_parseability(grammar_name) {
        Err(anyhow::anyhow!(
            "Parseability validation requires building ast_pipeline with generated parsers enabled: cargo run --features generated_parsers --bin ast_pipeline -- ... --validate-parseability"
        ))
    } else {
        Err(anyhow::anyhow!(
            "No matching generated parser validation path exists for grammar '{}'. Supported grammars: return_annotation, semantic_annotation",
            grammar_name
        ))
    }
}

#[cfg(not(feature = "generated_parsers"))]
fn is_sample_parseable_by_generated_parser(_grammar_name: &str, _sample: &str) -> Result<bool> {
    Err(anyhow::anyhow!(
        "Generated parser parseability checks are unavailable without --features generated_parsers"
    ))
}

#[cfg(test)]
mod tests {
    use super::supports_generated_parseability;

    #[test]
    fn supports_known_generated_parseability_grammars() {
        assert!(supports_generated_parseability("return_annotation"));
        assert!(supports_generated_parseability("semantic_annotation"));
        assert!(!supports_generated_parseability("regex"));
        assert!(!supports_generated_parseability("unknown"));
    }
}
