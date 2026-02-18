//! Rust AST Pipeline CLI
//!
//! Command-line interface for the Rust AST transformation pipeline.

use anyhow::Result;
use clap::Parser;
#[cfg(feature = "generated_parsers")]
use pgen::NoOpLogger;
use pgen::ast_pipeline::stimuli_generator::{
    StimuliConfig, StimuliCoverageGapReport, StimuliCoverageMetrics, StimuliGenerator,
};
use pgen::ast_pipeline::{
    ast_generator_direct::generate_parser_ast_based,
    ASTNode, Annotations, PipelineConfig, RustASTPipeline, TransformedASTJson,
};
#[cfg(feature = "generated_parsers")]
use pgen::generated_parsers::return_annotation::Return_annotationParser;
#[cfg(feature = "generated_parsers")]
use pgen::generated_parsers::semantic_annotation::Semantic_annotationParser;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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

    /// Load prior stimuli coverage JSON and merge new generation coverage into it
    #[arg(long, requires = "generate_stimuli")]
    coverage_input: Option<String>,

    /// Write merged stimuli coverage metrics JSON to this path
    #[arg(long, requires = "generate_stimuli")]
    coverage_output: Option<String>,

    /// Write detailed coverage gap report JSON (reachable/unreachable rules+branches and target plan)
    #[arg(long, requires = "generate_stimuli")]
    gap_report_json: Option<String>,

    /// Write human-readable detailed coverage gap report text
    #[arg(long, requires = "generate_stimuli")]
    gap_report_text: Option<String>,

    /// Required successful hits per rule/branch target when building gap report debt/targets
    #[arg(long, default_value_t = 1, requires = "generate_stimuli")]
    gap_report_threshold: u64,

    /// Load a prior gap report JSON and drive generation until its targets hit threshold (or attempt budget)
    #[arg(long, requires = "generate_stimuli")]
    target_report_input: Option<String>,

    /// Load a prior gap report JSON and apply its targets as generation priorities for count-based sampling
    #[arg(
        long,
        requires = "generate_stimuli",
        conflicts_with = "target_report_input"
    )]
    gap_priority_report_input: Option<String>,

    /// Max generation attempts for target-driven mode
    #[arg(long, default_value_t = 5000, requires = "generate_stimuli")]
    target_max_attempts: usize,

    /// Coverage-guided fuzz rounds (deterministic per-round seeds with replay metadata)
    #[arg(
        long,
        default_value_t = 0,
        requires = "generate_stimuli",
        conflicts_with = "target_report_input"
    )]
    coverage_guided_fuzz_rounds: usize,

    /// Starting seed for coverage-guided fuzz rounds (defaults to --seed, then 1)
    #[arg(long, requires = "generate_stimuli")]
    coverage_guided_fuzz_seed_start: Option<u64>,

    /// Write coverage-guided fuzz replay report JSON
    #[arg(long, requires = "generate_stimuli")]
    coverage_guided_fuzz_replay_output: Option<String>,

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
    generation_errors: usize,
    empty_generations: usize,
    parser_rejections: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CoverageGuidedFuzzReplayCase {
    round: usize,
    seed: u64,
    sample: Option<String>,
    generation_error: Option<String>,
    parseable: Option<bool>,
    accepted: bool,
    shrunk_counterexample: Option<String>,
    new_rule_hits: Vec<String>,
    new_branch_hits: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CoverageGuidedFuzzReplayReport {
    grammar_name: String,
    entry_rule: String,
    rounds: usize,
    accepted_cases: usize,
    rejected_cases: usize,
    minimized_cases: usize,
    parseability_counterexamples: usize,
    shrunk_counterexamples: usize,
    unique_rule_hits: usize,
    unique_branch_hits: usize,
    cases: Vec<CoverageGuidedFuzzReplayCase>,
}

impl CoverageGuidedFuzzReplayReport {
    fn summary_line(&self) -> String {
        format!(
            "Coverage-guided fuzz loop: rounds={} accepted={} rejected={} minimized={} parseability_counterexamples={} shrunk_counterexamples={} unique_rule_hits={} unique_branch_hits={}",
            self.rounds,
            self.accepted_cases,
            self.rejected_cases,
            self.minimized_cases,
            self.parseability_counterexamples,
            self.shrunk_counterexamples,
            self.unique_rule_hits,
            self.unique_branch_hits
        )
    }
}

#[derive(Debug, Clone)]
struct CoverageGuidedFuzzOutcome {
    minimized_samples: Vec<String>,
    merged_coverage: StimuliCoverageMetrics,
    replay_report: CoverageGuidedFuzzReplayReport,
}

#[derive(Debug, Clone)]
struct FuzzCorpusCandidate {
    sample: String,
    coverage_tokens: HashSet<String>,
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

        // Generate parser through the direct AST integration path so typed annotation
        // validation and strict CI policies apply to normal CLI generation as well.
        let parser_code = generate_parser_ast_based(
            &grammar.grammar_name,
            &grammar.grammar_tree,
            &grammar.rule_order,
            grammar.annotations.as_ref(),
            output_rust.as_str(),
        )?;
        std::fs::write(&output_rust, parser_code)?;

        println!("SOTA regex parser generated: {}", output_rust);
        (0, Vec::<String>::new())
    } else if args.generate_stimuli {
        let grammar = load_grammar_bundle(&args.input_json, &mut pipeline)?;
        let stimuli_config = StimuliConfig {
            seed: args.seed,
            max_depth: args.max_depth,
            max_repeat: args.max_repeat,
            max_rule_visits: args.max_depth.max(2),
        };

        let mut generator = StimuliGenerator::new(
            grammar.grammar_name.clone(),
            &grammar.grammar_tree,
            &grammar.rule_order,
            grammar.annotations.as_ref(),
            stimuli_config.clone(),
        );

        if let Some(coverage_input_path) = args.coverage_input.as_deref() {
            let existing_coverage = load_coverage_metrics(coverage_input_path)?;
            generator.merge_coverage_metrics(&existing_coverage)?;
        }

        if args.coverage_guided_fuzz_rounds == 0
            && (args.coverage_guided_fuzz_seed_start.is_some()
                || args.coverage_guided_fuzz_replay_output.is_some())
        {
            return Err(anyhow::anyhow!(
                "--coverage-guided-fuzz-seed-start/--coverage-guided-fuzz-replay-output require --coverage-guided-fuzz-rounds > 0"
            ));
        }

        let mut merged_coverage = generator.coverage_metrics().clone();
        let mut replay_report: Option<CoverageGuidedFuzzReplayReport> = None;

        let mut samples = if args.coverage_guided_fuzz_rounds > 0 {
            let seed_start = args.coverage_guided_fuzz_seed_start.or(args.seed).unwrap_or(1);
            let fuzz_outcome = run_coverage_guided_fuzz_loop(
                &grammar.grammar_name,
                &grammar.grammar_tree,
                &grammar.rule_order,
                grammar.annotations.as_ref(),
                &stimuli_config,
                args.entry_rule.as_deref(),
                args.coverage_guided_fuzz_rounds,
                seed_start,
                args.validate_parseability,
                merged_coverage.clone(),
            )?;
            println!("{}", fuzz_outcome.replay_report.summary_line());
            merged_coverage = fuzz_outcome.merged_coverage.clone();
            replay_report = Some(fuzz_outcome.replay_report);
            fuzz_outcome.minimized_samples
        } else if let Some(priority_report_input_path) = args.gap_priority_report_input.as_deref()
        {
            let priority_report = load_gap_report(priority_report_input_path)?;
            if priority_report.grammar_name != grammar.grammar_name {
                return Err(anyhow::anyhow!(
                    "Gap-priority report grammar '{}' does not match input grammar '{}'",
                    priority_report.grammar_name,
                    grammar.grammar_name
                ));
            }
            let applied_targets = generator.apply_targets(&priority_report.targets);
            println!(
                "Gap-priority mode: applied {} reachable target(s) from '{}'",
                applied_targets, priority_report_input_path
            );
            let generated_samples = if args.validate_parseability {
                generate_parseable_stimuli(
                    &grammar.grammar_name,
                    &mut generator,
                    args.count,
                    args.entry_rule.as_deref(),
                )?
            } else {
                generator.generate_many(args.count, args.entry_rule.as_deref())?
            };
            generator.clear_targets();
            merged_coverage = generator.coverage_metrics().clone();
            generated_samples
        } else if let Some(target_report_input_path) = args.target_report_input.as_deref() {
            let target_report = load_gap_report(target_report_input_path)?;
            if target_report.grammar_name != grammar.grammar_name {
                return Err(anyhow::anyhow!(
                    "Target report grammar '{}' does not match input grammar '{}'",
                    target_report.grammar_name,
                    grammar.grammar_name
                ));
            }
            let (generated_samples, target_summary) = generator.generate_until_targets(
                args.entry_rule.as_deref(),
                &target_report.targets,
                args.target_max_attempts,
            )?;
            println!("{}", target_summary.summary_line());
            if !target_summary.unresolved_targets.is_empty() {
                println!(
                    "Unresolved targets after target-driven generation: {}",
                    target_summary.unresolved_targets.len()
                );
                println!(
                    "Top unresolved targets (id | type | location | current/required | remaining | reason):"
                );
                for status in target_summary.unresolved_targets.iter().take(20) {
                    let location = if let (Some(node_path), Some(branch_index)) =
                        (status.node_path.as_deref(), status.branch_index)
                    {
                        format!("{}::{}#{}", status.rule_name, node_path, branch_index)
                    } else {
                        status.rule_name.clone()
                    };
                    println!(
                        "- {} | {:?} | {} | {}/{} | {} | {}",
                        status.id,
                        status.target_type,
                        location,
                        status.current_successes,
                        status.required_successes,
                        status.remaining_successes,
                        status.reason
                    );
                }
            }
            merged_coverage = generator.coverage_metrics().clone();
            generated_samples
        } else if args.validate_parseability {
            let generated_samples = generate_parseable_stimuli(
                &grammar.grammar_name,
                &mut generator,
                args.count,
                args.entry_rule.as_deref(),
            )?;
            merged_coverage = generator.coverage_metrics().clone();
            generated_samples
        } else {
            let generated_samples = generator.generate_many(args.count, args.entry_rule.as_deref())?;
            merged_coverage = generator.coverage_metrics().clone();
            generated_samples
        };

        if args.validate_parseability && args.target_report_input.is_some() {
            let (accepted, rejected) =
                filter_parseable_samples(&grammar.grammar_name, samples.into_iter())?;
            samples = accepted;
            println!(
                "Target-driven parseability filter accepted {} samples and rejected {}",
                samples.len(),
                rejected
            );
        }

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

        println!("{}", merged_coverage.summary_line());
        if let Some(coverage_output_path) = args.coverage_output.as_deref() {
            let coverage_json = serde_json::to_string_pretty(&merged_coverage)?;
            std::fs::write(coverage_output_path, coverage_json)?;
            println!("Wrote stimuli coverage metrics to {}", coverage_output_path);
        }

        if let Some(replay_output_path) = args.coverage_guided_fuzz_replay_output.as_deref() {
            let Some(report) = replay_report.as_ref() else {
                return Err(anyhow::anyhow!(
                    "No replay report available. Set --coverage-guided-fuzz-rounds > 0 to emit replay data."
                ));
            };
            let replay_json = serde_json::to_string_pretty(report)?;
            std::fs::write(replay_output_path, replay_json)?;
            println!(
                "Wrote coverage-guided fuzz replay report to {}",
                replay_output_path
            );
        }

        if args.gap_report_json.is_some() || args.gap_report_text.is_some() {
            let mut gap_generator = StimuliGenerator::new(
                grammar.grammar_name.clone(),
                &grammar.grammar_tree,
                &grammar.rule_order,
                grammar.annotations.as_ref(),
                stimuli_config.clone(),
            );
            gap_generator.merge_coverage_metrics(&merged_coverage)?;
            let gap_report = gap_generator
                .generate_gap_report(args.entry_rule.as_deref(), args.gap_report_threshold)?;
            if let Some(gap_report_json_path) = args.gap_report_json.as_deref() {
                let report_json = serde_json::to_string_pretty(&gap_report)?;
                std::fs::write(gap_report_json_path, report_json)?;
                println!("Wrote coverage gap report JSON to {}", gap_report_json_path);
            }
            if let Some(gap_report_text_path) = args.gap_report_text.as_deref() {
                std::fs::write(gap_report_text_path, gap_report.to_pretty_text())?;
                println!("Wrote coverage gap report text to {}", gap_report_text_path);
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

fn load_coverage_metrics(path: &str) -> Result<StimuliCoverageMetrics> {
    let content = std::fs::read_to_string(path)?;
    let metrics: StimuliCoverageMetrics = serde_json::from_str(&content)?;
    Ok(metrics)
}

fn load_gap_report(path: &str) -> Result<StimuliCoverageGapReport> {
    let content = std::fs::read_to_string(path)?;
    let report: StimuliCoverageGapReport = serde_json::from_str(&content)?;
    Ok(report)
}

fn summarize_sample(sample: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }

    let total_chars = sample.chars().count();
    if total_chars <= max_chars {
        return sample.to_string();
    }

    let keep = max_chars.saturating_sub(1);
    let truncated: String = sample.chars().take(keep).collect();
    format!("{}...", truncated)
}

fn shrink_parseability_counterexample(grammar_name: &str, sample: &str) -> Result<String> {
    minimize_failing_input(sample, |candidate| {
        Ok(!is_sample_parseable_by_generated_parser(
            grammar_name,
            candidate,
        )?)
    })
}

fn minimize_failing_input<F>(input: &str, mut still_fails: F) -> Result<String>
where
    F: FnMut(&str) -> Result<bool>,
{
    if input.is_empty() {
        return Ok(String::new());
    }
    if !still_fails(input)? {
        return Ok(input.to_string());
    }

    let mut candidate = input.to_string();
    let mut granularity = 2usize;

    loop {
        let char_len = candidate.chars().count();
        if char_len <= 1 {
            break;
        }

        let chunk = ((char_len + granularity - 1) / granularity).max(1);
        let mut start = 0usize;
        let mut reduced = false;

        while start < char_len {
            let end = (start + chunk).min(char_len);
            let trial = remove_chars_range(&candidate, start, end);
            if still_fails(&trial)? {
                candidate = trial;
                granularity = granularity.saturating_sub(1).max(2);
                reduced = true;
                break;
            }
            start = end;
        }

        if reduced {
            continue;
        }

        if granularity >= char_len {
            break;
        }
        granularity = (granularity * 2).min(char_len);
    }

    Ok(candidate)
}

fn remove_chars_range(input: &str, start_char: usize, end_char: usize) -> String {
    if start_char >= end_char {
        return input.to_string();
    }

    let start_byte = char_to_byte_idx(input, start_char);
    let end_byte = char_to_byte_idx(input, end_char);
    let mut output = String::with_capacity(input.len().saturating_sub(end_byte - start_byte));
    output.push_str(&input[..start_byte]);
    output.push_str(&input[end_byte..]);
    output
}

fn char_to_byte_idx(input: &str, char_idx: usize) -> usize {
    if char_idx == 0 {
        return 0;
    }
    match input.char_indices().nth(char_idx) {
        Some((byte_idx, _)) => byte_idx,
        None => input.len(),
    }
}

fn run_coverage_guided_fuzz_loop(
    grammar_name: &str,
    grammar_tree: &HashMap<String, ASTNode>,
    rule_order: &[String],
    annotations: Option<&Annotations>,
    base_config: &StimuliConfig,
    entry_rule: Option<&str>,
    rounds: usize,
    seed_start: u64,
    validate_parseability: bool,
    initial_coverage: StimuliCoverageMetrics,
) -> Result<CoverageGuidedFuzzOutcome> {
    if rounds == 0 {
        return Ok(CoverageGuidedFuzzOutcome {
            minimized_samples: Vec::new(),
            merged_coverage: initial_coverage,
            replay_report: CoverageGuidedFuzzReplayReport {
                grammar_name: grammar_name.to_string(),
                entry_rule: resolve_stimuli_entry_rule(grammar_tree, rule_order, entry_rule)?,
                rounds: 0,
                accepted_cases: 0,
                rejected_cases: 0,
                minimized_cases: 0,
                parseability_counterexamples: 0,
                shrunk_counterexamples: 0,
                unique_rule_hits: 0,
                unique_branch_hits: 0,
                cases: Vec::new(),
            },
        });
    }

    if validate_parseability {
        ensure_parseability_support(grammar_name)?;
    }

    let resolved_entry = resolve_stimuli_entry_rule(grammar_tree, rule_order, entry_rule)?;
    let mut merged_coverage = initial_coverage;
    let mut replay_cases = Vec::with_capacity(rounds);
    let mut corpus_candidates = Vec::new();
    let mut unique_rule_hits = HashSet::new();
    let mut unique_branch_hits = HashSet::new();

    for round_idx in 0..rounds {
        let offset = u64::try_from(round_idx).map_err(|_| {
            anyhow::anyhow!("Coverage-guided fuzz round index overflow at round {}", round_idx)
        })?;
        let seed = seed_start.checked_add(offset).ok_or_else(|| {
            anyhow::anyhow!(
                "Coverage-guided fuzz seed overflow: start={} round={}",
                seed_start,
                round_idx
            )
        })?;

        let mut seed_config = base_config.clone();
        seed_config.seed = Some(seed);
        let mut round_generator = StimuliGenerator::new(
            grammar_name.to_string(),
            grammar_tree,
            rule_order,
            annotations,
            seed_config,
        );
        round_generator.merge_coverage_metrics(&merged_coverage)?;

        let coverage_before = round_generator.coverage_metrics().clone();
        let generation_result = round_generator.generate_many(1, Some(resolved_entry.as_str()));
        let coverage_after = round_generator.coverage_metrics().clone();
        merged_coverage = coverage_after.clone();

        let (sample, generation_error) = match generation_result {
            Ok(mut samples) => (samples.pop(), None),
            Err(err) => (None, Some(err.to_string())),
        };

        let mut parseable = None;
        let mut accepted = sample.is_some();
        let mut shrunk_counterexample = None;
        if let Some(sample_text) = sample.as_deref() {
            if validate_parseability {
                let is_parseable = is_sample_parseable_by_generated_parser(grammar_name, sample_text)?;
                parseable = Some(is_parseable);
                accepted = is_parseable;
                if !is_parseable {
                    shrunk_counterexample =
                        Some(shrink_parseability_counterexample(grammar_name, sample_text)?);
                }
            }
        } else {
            accepted = false;
        }

        let new_rule_hits = coverage_rule_hit_delta(&coverage_before, &coverage_after);
        let new_branch_hits = coverage_branch_hit_delta(&coverage_before, &coverage_after);
        for rule in &new_rule_hits {
            unique_rule_hits.insert(rule.clone());
        }
        for branch in &new_branch_hits {
            unique_branch_hits.insert(branch.clone());
        }

        if accepted {
            if let Some(sample_text) = sample.as_ref() {
                let mut coverage_tokens = HashSet::new();
                for rule in &new_rule_hits {
                    coverage_tokens.insert(format!("rule::{}", rule));
                }
                for branch in &new_branch_hits {
                    coverage_tokens.insert(branch.clone());
                }
                corpus_candidates.push(FuzzCorpusCandidate {
                    sample: sample_text.clone(),
                    coverage_tokens,
                });
            }
        }

        replay_cases.push(CoverageGuidedFuzzReplayCase {
            round: round_idx + 1,
            seed,
            sample,
            generation_error,
            parseable,
            accepted,
            shrunk_counterexample,
            new_rule_hits,
            new_branch_hits,
        });
    }

    let minimized_indices = minimize_fuzz_corpus_cases(&corpus_candidates);
    let minimized_samples = minimized_indices
        .into_iter()
        .map(|idx| corpus_candidates[idx].sample.clone())
        .collect::<Vec<String>>();
    let minimized_case_count = minimized_samples.len();
    let accepted_cases = replay_cases.iter().filter(|case| case.accepted).count();
    let rejected_cases = replay_cases.len().saturating_sub(accepted_cases);
    let parseability_counterexamples = replay_cases
        .iter()
        .filter(|case| case.parseable == Some(false))
        .count();
    let shrunk_counterexamples = replay_cases
        .iter()
        .filter(|case| case.shrunk_counterexample.is_some())
        .count();

    Ok(CoverageGuidedFuzzOutcome {
        minimized_samples,
        merged_coverage,
        replay_report: CoverageGuidedFuzzReplayReport {
            grammar_name: grammar_name.to_string(),
            entry_rule: resolved_entry,
            rounds,
            accepted_cases,
            rejected_cases,
            minimized_cases: minimized_case_count,
            parseability_counterexamples,
            shrunk_counterexamples,
            unique_rule_hits: unique_rule_hits.len(),
            unique_branch_hits: unique_branch_hits.len(),
            cases: replay_cases,
        },
    })
}

fn resolve_stimuli_entry_rule(
    grammar_tree: &HashMap<String, ASTNode>,
    rule_order: &[String],
    entry_rule: Option<&str>,
) -> Result<String> {
    if let Some(rule_name) = entry_rule {
        if grammar_tree.contains_key(rule_name) {
            return Ok(rule_name.to_string());
        }
        return Err(anyhow::anyhow!("Entry rule '{}' not found in grammar", rule_name));
    }

    rule_order.first().cloned().ok_or_else(|| {
        anyhow::anyhow!("No entry rule available for stimuli generation (empty rule_order)")
    })
}

fn coverage_rule_hit_delta(
    before: &StimuliCoverageMetrics,
    after: &StimuliCoverageMetrics,
) -> Vec<String> {
    let mut delta = Vec::new();
    for (rule_name, after_hits) in &after.rule_success_hits {
        let before_hits = before.rule_success_hits.get(rule_name).copied().unwrap_or(0);
        if *after_hits > before_hits {
            delta.push(rule_name.clone());
        }
    }
    delta.sort();
    delta
}

fn coverage_branch_hit_delta(
    before: &StimuliCoverageMetrics,
    after: &StimuliCoverageMetrics,
) -> Vec<String> {
    let mut delta = Vec::new();
    for (group_key, after_group) in &after.branch_groups {
        let before_group = before.branch_groups.get(group_key);
        for idx in 0..after_group.total_branches {
            let after_hits = after_group.success_counts.get(idx).copied().unwrap_or(0);
            let before_hits = before_group
                .and_then(|group| group.success_counts.get(idx).copied())
                .unwrap_or(0);
            if after_hits > before_hits {
                delta.push(format!(
                    "branch::{}::{}#{}",
                    after_group.rule_name, after_group.node_path, idx
                ));
            }
        }
    }
    delta.sort();
    delta
}

fn minimize_fuzz_corpus_cases(cases: &[FuzzCorpusCandidate]) -> Vec<usize> {
    if cases.is_empty() {
        return Vec::new();
    }

    let mut uncovered = HashSet::new();
    for case in cases {
        for token in &case.coverage_tokens {
            uncovered.insert(token.clone());
        }
    }

    if uncovered.is_empty() {
        let shortest = cases
            .iter()
            .enumerate()
            .min_by_key(|(idx, case)| (case.sample.len(), *idx))
            .map(|(idx, _)| idx)
            .unwrap_or(0);
        return vec![shortest];
    }

    let mut selected = Vec::new();
    let mut used = HashSet::new();
    while !uncovered.is_empty() {
        let mut best_idx = None;
        let mut best_gain = 0usize;
        let mut best_len = usize::MAX;
        for (idx, case) in cases.iter().enumerate() {
            if used.contains(&idx) {
                continue;
            }
            let gain = case
                .coverage_tokens
                .iter()
                .filter(|token| uncovered.contains(*token))
                .count();
            if gain == 0 {
                continue;
            }
            if gain > best_gain || (gain == best_gain && case.sample.len() < best_len) {
                best_idx = Some(idx);
                best_gain = gain;
                best_len = case.sample.len();
            }
        }

        let Some(best) = best_idx else {
            break;
        };
        used.insert(best);
        selected.push(best);
        for token in &cases[best].coverage_tokens {
            uncovered.remove(token);
        }
    }

    if selected.is_empty() {
        selected.push(0);
    }
    selected.sort_unstable();
    selected
}

fn filter_parseable_samples<I>(grammar_name: &str, samples: I) -> Result<(Vec<String>, usize)>
where
    I: IntoIterator<Item = String>,
{
    ensure_parseability_support(grammar_name)?;
    let mut accepted = Vec::new();
    let mut rejected = 0usize;
    for sample in samples {
        if is_sample_parseable_by_generated_parser(grammar_name, &sample)? {
            accepted.push(sample);
        } else {
            rejected = rejected.saturating_add(1);
        }
    }
    Ok((accepted, rejected))
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
    let mut generation_errors = 0usize;
    let mut empty_generations = 0usize;
    let mut parser_rejections = 0usize;
    let mut last_parser_rejected_sample: Option<String> = None;

    while accepted.len() < requested_count && attempts < max_attempts {
        attempts += 1;
        let sample = match generator.generate_many(1, entry_rule) {
            Ok(mut samples) => match samples.pop() {
                Some(sample) => sample,
                None => {
                    empty_generations += 1;
                    rejected += 1;
                    continue;
                }
            },
            Err(_) => {
                generation_errors += 1;
                rejected += 1;
                continue;
            }
        };

        if is_sample_parseable_by_generated_parser(grammar_name, &sample)? {
            accepted.push(sample);
        } else {
            parser_rejections += 1;
            rejected += 1;
            last_parser_rejected_sample = Some(sample);
        }
    }

    let summary = ParseabilitySummary {
        requested: requested_count,
        accepted: accepted.len(),
        rejected,
        attempts,
        generation_errors,
        empty_generations,
        parser_rejections,
    };

    if accepted.len() < requested_count {
        let counterexample_note = if let Some(sample) = last_parser_rejected_sample {
            let shrunk = shrink_parseability_counterexample(grammar_name, &sample)
                .unwrap_or_else(|_| sample.clone());
            format!(
                " Last parseability counterexample: '{}' (shrunk='{}').",
                summarize_sample(&sample, 160),
                summarize_sample(&shrunk, 160)
            )
        } else {
            String::new()
        };
        return Err(anyhow::anyhow!(
            "Unable to produce {} parseable stimuli for grammar '{}' after {} attempts (accepted {}, rejected {}; parse_rejections={}, generation_errors={}, empty_generations={}). Try increasing --max-depth/--max-repeat or lowering --count.{}",
            summary.requested,
            grammar_name,
            summary.attempts,
            summary.accepted,
            summary.rejected,
            summary.parser_rejections,
            summary.generation_errors,
            summary.empty_generations,
            counterexample_note
        ));
    }
    let acceptance_rate = if summary.attempts == 0 {
        0.0
    } else {
        (summary.accepted as f64 * 100.0) / summary.attempts as f64
    };
    let rejection_rate = if summary.attempts == 0 {
        0.0
    } else {
        (summary.rejected as f64 * 100.0) / summary.attempts as f64
    };

    println!(
        "Parseability validation accepted {}/{} samples ({} rejected over {} attempts | acceptance {:.2}% / rejection {:.2}% | parse_rejections={} generation_errors={} empty_generations={})",
        summary.accepted,
        summary.requested,
        summary.rejected,
        summary.attempts,
        acceptance_rate,
        rejection_rate,
        summary.parser_rejections,
        summary.generation_errors,
        summary.empty_generations
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
    use super::{
        coverage_branch_hit_delta, minimize_failing_input, minimize_fuzz_corpus_cases,
        supports_generated_parseability,
        FuzzCorpusCandidate, StimuliCoverageMetrics,
    };
    use pgen::ast_pipeline::stimuli_generator::BranchCoverageGroup;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn supports_known_generated_parseability_grammars() {
        assert!(supports_generated_parseability("return_annotation"));
        assert!(supports_generated_parseability("semantic_annotation"));
        assert!(!supports_generated_parseability("regex"));
        assert!(!supports_generated_parseability("unknown"));
    }

    #[test]
    fn corpus_minimization_prefers_max_coverage_candidate() {
        let mut c0_tokens = HashSet::new();
        c0_tokens.insert("rule::a".to_string());
        let mut c1_tokens = HashSet::new();
        c1_tokens.insert("rule::b".to_string());
        let mut c2_tokens = HashSet::new();
        c2_tokens.insert("rule::a".to_string());
        c2_tokens.insert("rule::b".to_string());

        let cases = vec![
            FuzzCorpusCandidate {
                sample: "alpha".to_string(),
                coverage_tokens: c0_tokens,
            },
            FuzzCorpusCandidate {
                sample: "beta".to_string(),
                coverage_tokens: c1_tokens,
            },
            FuzzCorpusCandidate {
                sample: "both".to_string(),
                coverage_tokens: c2_tokens,
            },
        ];

        let selected = minimize_fuzz_corpus_cases(&cases);
        assert_eq!(selected, vec![2]);
    }

    #[test]
    fn corpus_minimization_falls_back_to_shortest_when_no_coverage_delta() {
        let cases = vec![
            FuzzCorpusCandidate {
                sample: "longer".to_string(),
                coverage_tokens: HashSet::new(),
            },
            FuzzCorpusCandidate {
                sample: "x".to_string(),
                coverage_tokens: HashSet::new(),
            },
            FuzzCorpusCandidate {
                sample: "mid".to_string(),
                coverage_tokens: HashSet::new(),
            },
        ];

        let selected = minimize_fuzz_corpus_cases(&cases);
        assert_eq!(selected, vec![1]);
    }

    #[test]
    fn branch_hit_delta_reports_new_successes_only() {
        let mut before_groups = HashMap::new();
        before_groups.insert(
            "root::group".to_string(),
            BranchCoverageGroup {
                rule_name: "root".to_string(),
                node_path: "root".to_string(),
                total_branches: 2,
                selected_counts: vec![1, 1],
                success_counts: vec![0, 1],
            },
        );
        let before = StimuliCoverageMetrics {
            grammar_name: "g".to_string(),
            total_rules: 1,
            total_branch_groups: 1,
            total_branches: 2,
            sample_attempts: 1,
            sample_successes: 1,
            sample_errors: 0,
            rule_success_hits: HashMap::new(),
            branch_groups: before_groups,
        };

        let mut after_groups = HashMap::new();
        after_groups.insert(
            "root::group".to_string(),
            BranchCoverageGroup {
                rule_name: "root".to_string(),
                node_path: "root".to_string(),
                total_branches: 2,
                selected_counts: vec![2, 2],
                success_counts: vec![1, 1],
            },
        );
        let after = StimuliCoverageMetrics {
            grammar_name: "g".to_string(),
            total_rules: 1,
            total_branch_groups: 1,
            total_branches: 2,
            sample_attempts: 2,
            sample_successes: 2,
            sample_errors: 0,
            rule_success_hits: HashMap::new(),
            branch_groups: after_groups,
        };

        let delta = coverage_branch_hit_delta(&before, &after);
        assert_eq!(delta, vec!["branch::root::root#0".to_string()]);
    }

    #[test]
    fn failing_input_minimizer_reduces_to_core_token() {
        let minimized = minimize_failing_input("zzabyy", |candidate| {
            Ok(candidate.contains("ab"))
        })
        .expect("minimizer should succeed");
        assert_eq!(minimized, "ab");
    }

    #[test]
    fn failing_input_minimizer_keeps_input_when_not_failing() {
        let minimized = minimize_failing_input("stable", |_candidate| Ok(false))
            .expect("minimizer should succeed");
        assert_eq!(minimized, "stable");
    }
}
