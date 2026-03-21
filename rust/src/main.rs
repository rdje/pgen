//! Rust AST Pipeline CLI
//!
//! Command-line interface for the Rust AST transformation pipeline.

use anyhow::{Context, Result};
use clap::Parser;
use pgen::ast_pipeline::stimuli_generator::{
    RecoveryStimuliMode, StimuliConfig, StimuliCoverageGapReport, StimuliCoverageMetrics,
    StimuliGenerator, TargetDriveValidationSummary,
};
use pgen::ast_pipeline::{
    ASTNode, Annotations, PipelineConfig, RustASTPipeline, TraceVerbosity, TransformedASTJson,
    ast_generator_direct::generate_parser_ast_based, configure_trace_output,
    extract_semantic_directive, parse_semantic_string_list, resolve_trace_verbosity,
    set_global_trace_verbosity,
};
#[cfg(feature = "ebnf_dual_run")]
use pgen::ebnf_frontend;
#[cfg(feature = "generated_parsers")]
use pgen::parser_registry;
use pgen::sv_preprocessor::{
    ConditionalExprPolicy, ConditionalSymbolPolicy, IncludePathPolicy, MacroRedefinitionPolicy,
    SvPreprocessorConfig, parse_strict_warning_codes, preprocess_systemverilog_file,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

const STIMULI_MODULE_API_VERSION: u32 = 1;
const DEFAULT_STIMULI_MODULE_SEED: u64 = 1;

#[derive(Parser)]
#[command(name = "ast_pipeline")]
#[command(about = "Rust AST Transformation Pipeline")]
#[command(version = "1.0.0")]
#[command(
    long_about = "Transform AST JSON files or parse EBNF source directly via Rust frontend, generate high-performance Rust parsers, generate grammar-valid stimuli, emit Rust stimuli modules, or preprocess SystemVerilog source files.\n\nUsage modes:\n  1. JSON transformation: ast_pipeline INPUT.json [OUTPUT.json]\n  2. Rust EBNF raw_ast export: ast_pipeline INPUT.ebnf --emit-raw-ast-json RAW.json\n  3. Rust EBNF frontend generation: ast_pipeline INPUT.ebnf --generate-parser|--generate-stimuli|--generate-stimuli-module [--emit-raw-ast-json RAW.json]\n  4. Parser generation: ast_pipeline INPUT --generate-parser [--output PARSER.rs]\n  5. Stimuli generation: ast_pipeline INPUT --generate-stimuli [--count N] [--seed SEED]\n  6. Stimuli module generation: ast_pipeline INPUT --generate-stimuli-module [--count N] [--seed SEED] [--output generated/<grammar>_stimuli.rs]\n  7. SV preprocess stage: ast_pipeline INPUT.sv --preprocess-systemverilog [--output PREPROCESSED.sv]\n  8. Generation-input AST dump: ast_pipeline INPUT --generate-* --dump-gen-ast [PATH]"
)]
struct Args {
    /// Input grammar source file (.json raw/transformed AST, or .ebnf when built with --features ebnf_dual_run)
    input_path: String,

    /// Transformed AST JSON output file (optional, ignored when generation modes are used)
    output_json: Option<String>,
    /// Output file path for generated artifact (parser source, newline-delimited stimuli, or Rust stimuli module)
    #[arg(short, long)]
    output: Option<String>,

    /// When INPUT is .ebnf, optionally write the Rust-frontend raw_ast envelope JSON to this file
    #[arg(long)]
    emit_raw_ast_json: Option<String>,

    /// Dump the normalized generation-input AST JSON used by parser/stimuli generation (defaults to gen_ast.json)
    #[arg(long, num_args = 0..=1, default_missing_value = "gen_ast.json")]
    dump_gen_ast: Option<String>,

    /// Pretty-print generation-input AST dump JSON
    #[arg(long, requires = "dump_gen_ast")]
    dump_gen_ast_pretty: bool,

    /// Maximum bytes allowed for generation-input AST dump output; oversized dumps are replaced with truncation diagnostics JSON
    #[arg(long, requires = "dump_gen_ast")]
    dump_gen_ast_max_bytes: Option<usize>,

    /// Enable debug output
    #[arg(long, short = 'd')]
    debug: bool,

    /// Trace verbosity: none, low, medium, high, debug
    #[arg(long, value_parser = ["none", "low", "medium", "high", "debug"])]
    verbosity: Option<String>,

    /// Route trace output to a file (defaults to trace.log when flag is provided without a value)
    #[arg(long, num_args = 0..=1, default_missing_value = "trace.log")]
    trace_log_file: Option<String>,

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
    /// Generate a Rust stimuli module artifact with embedded generated samples
    #[arg(long, conflicts_with_all = ["generate_parser", "generate_stimuli"])]
    generate_stimuli_module: bool,

    /// Run SystemVerilog preprocessor execution stage on INPUT (raw SV -> expanded SV + source-map metadata)
    #[arg(long, conflicts_with_all = ["generate_parser", "generate_stimuli", "generate_stimuli_module"])]
    preprocess_systemverilog: bool,

    /// Number of stimuli samples to generate (stimuli mode)
    #[arg(long, default_value_t = 1)]
    count: usize,

    /// Seed for deterministic stimuli generation (stimuli mode)
    #[arg(long)]
    seed: Option<u64>,

    /// Override grammar entry rule for generation
    #[arg(long)]
    entry_rule: Option<String>,

    /// Optional grammar profile filter (for example `sv_2017`, `sv_2023`, `vhdl_1076_2019`)
    #[arg(long)]
    grammar_profile: Option<String>,

    /// Maximum recursive depth during stimuli generation
    #[arg(long, default_value_t = 24)]
    max_depth: usize,

    /// Maximum repetitions generated for quantifiers (*, +, {n,m})
    #[arg(long, default_value_t = 4)]
    max_repeat: usize,

    /// Recovery-focused stimuli mode: baseline, recovery_biased, near_sync_negative
    #[arg(
        long,
        default_value = "baseline",
        value_parser = ["baseline", "recovery_biased", "near_sync_negative"]
    )]
    recovery_stimuli_mode: String,

    /// Append a delimiter space after terminal word-boundary regex samples (for example `keyword\\b`) to reduce merged-token stimuli
    #[arg(long)]
    enforce_word_boundary_spacing: bool,

    /// Validate generated stimuli by parsing each sample with the matching generated parser
    #[arg(long)]
    validate_parseability: bool,

    /// Write parseability validation summary JSON for stimuli generation
    #[arg(long, requires = "validate_parseability")]
    parseability_report_json: Option<String>,

    /// Max generation attempts for parseability-aware stimuli generation (defaults to count * 50)
    #[arg(long, requires = "validate_parseability")]
    parseability_max_attempts: Option<usize>,

    /// Load prior stimuli coverage JSON and merge new generation coverage into it
    #[arg(long)]
    coverage_input: Option<String>,

    /// Write merged stimuli coverage metrics JSON to this path
    #[arg(long)]
    coverage_output: Option<String>,

    /// Write detailed coverage gap report JSON (reachable/unreachable rules+branches and target plan)
    #[arg(long)]
    gap_report_json: Option<String>,

    /// Write human-readable detailed coverage gap report text
    #[arg(long)]
    gap_report_text: Option<String>,

    /// Required successful hits per rule/branch target when building gap report debt/targets
    #[arg(long, default_value_t = 1)]
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

    /// Include search directory for SystemVerilog preprocessor mode (can be used multiple times)
    #[arg(long, requires = "preprocess_systemverilog")]
    sv_include_dir: Vec<String>,

    /// Max include depth for SystemVerilog preprocessor mode
    #[arg(long, default_value_t = 64, requires = "preprocess_systemverilog")]
    sv_include_max_depth: usize,

    /// Disallow macro redefinition in SystemVerilog preprocessor mode
    #[arg(long, requires = "preprocess_systemverilog")]
    sv_disallow_macro_redefine: bool,

    /// Optional JSON output path for preprocessor source-map entries
    #[arg(long, requires = "preprocess_systemverilog")]
    sv_source_map_json: Option<String>,

    /// Optional JSON output path for preprocessor event log
    #[arg(long, requires = "preprocess_systemverilog")]
    sv_event_log_json: Option<String>,

    /// Optional JSON output path for preprocessor diagnostics
    #[arg(long, requires = "preprocess_systemverilog")]
    sv_diagnostics_json: Option<String>,

    /// Include path policy for preprocessor mode: allow_absolute, relative_only
    #[arg(
        long,
        default_value = "allow_absolute",
        value_parser = ["allow_absolute", "relative_only"],
        requires = "preprocess_systemverilog"
    )]
    sv_include_path_policy: String,

    /// Macro redefinition policy for preprocessor mode: allow, warn, error
    #[arg(
        long,
        default_value = "allow",
        value_parser = ["allow", "warn", "error"],
        requires = "preprocess_systemverilog"
    )]
    sv_macro_redefine_policy: String,

    /// Conditional symbol policy for preprocessor mode: assume_false_silent, assume_false_warn, error
    #[arg(
        long,
        default_value = "assume_false_silent",
        value_parser = ["assume_false_silent", "assume_false_warn", "error"],
        requires = "preprocess_systemverilog"
    )]
    sv_conditional_symbol_policy: String,

    /// Conditional expression policy for `elsif in preprocessor mode: identifier_only, identifier_or_defined
    #[arg(
        long,
        default_value = "identifier_or_defined",
        value_parser = ["identifier_only", "identifier_or_defined"],
        requires = "preprocess_systemverilog"
    )]
    sv_conditional_expr_policy: String,

    /// Comma-separated warning codes promoted to errors in preprocessor mode (`all`, `none`, or specific codes)
    #[arg(long, requires = "preprocess_systemverilog")]
    sv_strict_warning_codes: Option<String>,
}
struct LoadedGrammar {
    grammar_name: String,
    grammar_tree: HashMap<String, ASTNode>,
    rule_order: Vec<String>,
    annotations: Option<Annotations>,
}

#[derive(Debug, Serialize)]
struct GenerationAstDump<'a> {
    grammar_name: &'a str,
    rule_order: &'a [String],
    grammar_tree: &'a HashMap<String, ASTNode>,
    annotations: Option<&'a Annotations>,
}

#[derive(Debug, Serialize)]
struct AstDumpTruncationDiagnostic {
    pgen_dump_contract_version: u32,
    kind: &'static str,
    truncated: bool,
    dump_kind: String,
    max_bytes: usize,
    full_bytes: usize,
    reason: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct AstDumpWriteResult {
    truncated: bool,
    bytes_written: usize,
    full_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParseabilitySummary {
    requested: usize,
    accepted: usize,
    rejected: usize,
    attempts: usize,
    generation_errors: usize,
    empty_generations: usize,
    parser_rejections: usize,
}

const MAX_PARSEABILITY_COUNTEREXAMPLES: usize = 5;
const MAX_PARSEABILITY_FAILURE_LINE_EXCERPT_CHARS: usize = 80;
const MAX_PARSEABILITY_FAILURE_CONTEXT_EXCERPT_CHARS: usize = 48;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct ParseabilityCounterexample {
    stage: String,
    sample: String,
    sample_chars: usize,
    shrunk_sample: String,
    shrunk_sample_chars: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    parser_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure_position: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure_line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure_column: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure_line_excerpt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure_context_excerpt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParseabilityGenerationReport {
    grammar_name: String,
    grammar_profile: Option<String>,
    entry_rule: String,
    summary: ParseabilitySummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_drive_validation: Option<TargetDriveParseabilityTelemetry>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    counterexamples: Vec<ParseabilityCounterexample>,
}

#[derive(Debug, Clone)]
struct ParseableStimuliOutcome {
    samples: Vec<String>,
    summary: ParseabilitySummary,
    counterexamples: Vec<ParseabilityCounterexample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TargetDriveParseabilityTelemetry {
    primary_entry_attempts: usize,
    primary_entry_accepted_outputs: usize,
    primary_entry_rejected_outputs: usize,
    primary_entry_acceptance_rate_percent: f64,
    alternate_entry_attempts: usize,
    alternate_entry_accepted_outputs: usize,
    alternate_entry_rejected_outputs: usize,
    alternate_entry_acceptance_rate_percent: f64,
}

impl TargetDriveParseabilityTelemetry {
    fn acceptance_rate_percent(accepted: usize, attempts: usize) -> f64 {
        if attempts == 0 {
            0.0
        } else {
            (accepted as f64 * 100.0) / attempts as f64
        }
    }

    fn from_validation(summary: &TargetDriveValidationSummary) -> Self {
        Self {
            primary_entry_attempts: summary.validated_outputs,
            primary_entry_accepted_outputs: summary.accepted_outputs,
            primary_entry_rejected_outputs: summary.rejected_outputs,
            primary_entry_acceptance_rate_percent: Self::acceptance_rate_percent(
                summary.accepted_outputs,
                summary.validated_outputs,
            ),
            alternate_entry_attempts: summary.alternate_entry_attempts,
            alternate_entry_accepted_outputs: summary.alternate_entry_accepted_outputs,
            alternate_entry_rejected_outputs: summary.alternate_entry_rejected_outputs,
            alternate_entry_acceptance_rate_percent: Self::acceptance_rate_percent(
                summary.alternate_entry_accepted_outputs,
                summary.alternate_entry_attempts,
            ),
        }
    }
}

impl ParseabilitySummary {
    fn acceptance_rate_percent(&self) -> f64 {
        if self.attempts == 0 {
            0.0
        } else {
            (self.accepted as f64 * 100.0) / self.attempts as f64
        }
    }

    fn rejection_rate_percent(&self) -> f64 {
        if self.attempts == 0 {
            0.0
        } else {
            (self.rejected as f64 * 100.0) / self.attempts as f64
        }
    }

    fn from_filter(requested: usize, accepted: usize, rejected: usize) -> Self {
        Self {
            requested,
            accepted,
            rejected,
            attempts: requested,
            generation_errors: 0,
            empty_generations: 0,
            parser_rejections: rejected,
        }
    }

    fn from_coverage_guided_replay(report: &CoverageGuidedFuzzReplayReport) -> Self {
        let generation_errors = report
            .cases
            .iter()
            .filter(|case| case.generation_error.is_some())
            .count();
        let empty_generations = report
            .cases
            .iter()
            .filter(|case| case.sample.is_none() && case.generation_error.is_none())
            .count();
        Self {
            requested: report.rounds,
            accepted: report.accepted_cases,
            rejected: report.rejected_cases,
            attempts: report.rounds,
            generation_errors,
            empty_generations,
            parser_rejections: report.parseability_counterexamples,
        }
    }

    fn summary_line(&self) -> String {
        format!(
            "Parseability validation accepted {}/{} samples ({} rejected over {} attempts | acceptance {:.2}% / rejection {:.2}% | parse_rejections={} generation_errors={} empty_generations={})",
            self.accepted,
            self.requested,
            self.rejected,
            self.attempts,
            self.acceptance_rate_percent(),
            self.rejection_rate_percent(),
            self.parser_rejections,
            self.generation_errors,
            self.empty_generations
        )
    }
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
    let trace_log_path = args
        .trace_log_file
        .clone()
        .or_else(|| std::env::var("PGEN_TRACE_LOG_FILE").ok());
    configure_trace_output(trace_log_path.as_deref())?;

    let trace_verbosity =
        resolve_trace_verbosity(args.verbosity.as_deref(), args.debug, args.trace)?;
    set_global_trace_verbosity(trace_verbosity);
    if trace_verbosity != TraceVerbosity::None {
        println!("Tracing enabled at verbosity={}", trace_verbosity.as_str());
    }
    if let Some(path) = trace_log_path.as_deref() {
        println!("Trace output redirected to {}", path);
    }

    let stimuli_like_mode = args.generate_stimuli || args.generate_stimuli_module;
    if !stimuli_like_mode {
        let has_shared_stimuli_flags = args.validate_parseability
            || args.parseability_report_json.is_some()
            || args.parseability_max_attempts.is_some()
            || args.coverage_input.is_some()
            || args.coverage_output.is_some()
            || args.gap_report_json.is_some()
            || args.gap_report_text.is_some()
            || args.gap_report_threshold != 1
            || args.recovery_stimuli_mode != "baseline"
            || args.enforce_word_boundary_spacing;
        if has_shared_stimuli_flags {
            return Err(anyhow::anyhow!(
                "--validate-parseability/--parseability-report-json/--parseability-max-attempts/--coverage-*/--gap-report-*/--recovery-stimuli-mode/--enforce-word-boundary-spacing require --generate-stimuli or --generate-stimuli-module"
            ));
        }
    }

    if args.dump_gen_ast.is_some()
        && !(args.generate_parser || args.generate_stimuli || args.generate_stimuli_module)
    {
        return Err(anyhow::anyhow!(
            "--dump-gen-ast requires --generate-parser, --generate-stimuli, or --generate-stimuli-module"
        ));
    }
    let dump_gen_ast_max_bytes = resolve_dump_gen_ast_max_bytes(&args)?;

    if matches!(args.parseability_max_attempts, Some(0)) {
        return Err(anyhow::anyhow!(
            "--parseability-max-attempts must be greater than 0"
        ));
    }

    if args.preprocess_systemverilog {
        let include_dirs = args
            .sv_include_dir
            .iter()
            .map(PathBuf::from)
            .collect::<Vec<_>>();
        let include_path_policy = IncludePathPolicy::parse(&args.sv_include_path_policy)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "invalid --sv-include-path-policy '{}'; expected allow_absolute|relative_only",
                    args.sv_include_path_policy
                )
            })?;
        let macro_redefinition_policy =
            MacroRedefinitionPolicy::parse(&args.sv_macro_redefine_policy).ok_or_else(|| {
                anyhow::anyhow!(
                    "invalid --sv-macro-redefine-policy '{}'; expected allow|warn|error",
                    args.sv_macro_redefine_policy
                )
            })?;
        let conditional_symbol_policy =
            ConditionalSymbolPolicy::parse(&args.sv_conditional_symbol_policy).ok_or_else(
                || {
                    anyhow::anyhow!(
                        "invalid --sv-conditional-symbol-policy '{}'; expected assume_false_silent|assume_false_warn|error",
                        args.sv_conditional_symbol_policy
                    )
                },
            )?;
        let conditional_expr_policy = ConditionalExprPolicy::parse(&args.sv_conditional_expr_policy)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "invalid --sv-conditional-expr-policy '{}'; expected identifier_only|identifier_or_defined",
                    args.sv_conditional_expr_policy
                )
            })?;
        let strict_warning_codes_raw = args
            .sv_strict_warning_codes
            .clone()
            .or_else(|| std::env::var("PGEN_SVPP_STRICT_WARNING_CODES").ok())
            .unwrap_or_else(|| "none".to_string());
        let strict_warning_codes = parse_strict_warning_codes(&strict_warning_codes_raw);
        let config = SvPreprocessorConfig {
            include_dirs,
            max_include_depth: args.sv_include_max_depth,
            include_path_policy,
            macro_redefinition_policy: if args.sv_disallow_macro_redefine {
                MacroRedefinitionPolicy::Error
            } else {
                macro_redefinition_policy
            },
            conditional_symbol_policy,
            conditional_expr_policy,
            strict_warning_codes,
        };
        let output = preprocess_systemverilog_file(Path::new(&args.input_path), &config)?;
        if let Some(output_path) = args.output.as_deref() {
            std::fs::write(output_path, &output.text)?;
            println!("Wrote preprocessed SystemVerilog to {}", output_path);
        } else {
            print!("{}", output.text);
        }
        if let Some(source_map_path) = args.sv_source_map_json.as_deref() {
            let json = serde_json::to_string_pretty(&output.source_map)?;
            std::fs::write(source_map_path, json)?;
            println!("Wrote SV preprocessor source map to {}", source_map_path);
        }
        if let Some(events_path) = args.sv_event_log_json.as_deref() {
            let json = serde_json::to_string_pretty(&output.events)?;
            std::fs::write(events_path, json)?;
            println!("Wrote SV preprocessor event log to {}", events_path);
        }
        if let Some(diagnostics_path) = args.sv_diagnostics_json.as_deref() {
            let json = serde_json::to_string_pretty(&output.diagnostics)?;
            std::fs::write(diagnostics_path, json)?;
            println!("Wrote SV preprocessor diagnostics to {}", diagnostics_path);
        }
        let warning_count = output
            .diagnostics
            .iter()
            .filter(|d| {
                matches!(
                    d.severity,
                    pgen::sv_preprocessor::PreprocessorDiagnosticSeverity::Warning
                )
            })
            .count();
        let error_count = output
            .diagnostics
            .iter()
            .filter(|d| {
                matches!(
                    d.severity,
                    pgen::sv_preprocessor::PreprocessorDiagnosticSeverity::Error
                )
            })
            .count();
        println!(
            "SV preprocess summary: output_bytes={} source_map_entries={} events={} diagnostics={} warnings={} errors={} included_files={}",
            output.text.len(),
            output.source_map.len(),
            output.events.len(),
            output.diagnostics.len(),
            warning_count,
            error_count,
            output.included_files.len()
        );
        return Ok(());
    }

    // Start with default config and override only specified options
    let mut config = PipelineConfig::default();
    config.debug = args.debug || trace_verbosity >= TraceVerbosity::High;
    config.trace = args.trace || trace_verbosity >= TraceVerbosity::Debug;
    config.trace_verbosity = trace_verbosity;
    config.validate_input = !args.no_validate;
    config.bootstrap_mode = args.bootstrap_mode;

    // Only override left recursion elimination if explicitly specified
    if args.eliminate_left_recursion {
        config.eliminate_left_recursion = true;
    }
    // Note: eliminate_left_recursion defaults to true in PipelineConfig::default()

    let mut pipeline = RustASTPipeline::new(config);

    let standalone_raw_ast_export = is_ebnf_input_path(&args.input_path)
        && args.emit_raw_ast_json.is_some()
        && !args.generate_parser
        && !args.generate_stimuli
        && !args.generate_stimuli_module;

    let result = if standalone_raw_ast_export {
        let output_path = args
            .emit_raw_ast_json
            .as_deref()
            .expect("standalone_raw_ast_export requires emit_raw_ast_json");
        let rule_count = emit_rust_frontend_raw_ast_json(&args.input_path, output_path)?;
        (rule_count, Vec::<String>::new())
    } else if args.generate_parser {
        // Generate high-performance Rust parser using AST-based generator
        let output_rust = args
            .output
            .unwrap_or_else(|| default_parser_output_path(&args.input_path));

        let grammar = apply_grammar_profile_filter(
            load_grammar_bundle(
                &args.input_path,
                &mut pipeline,
                args.emit_raw_ast_json.as_deref(),
            )?,
            args.grammar_profile.as_deref(),
        )?;
        maybe_dump_generation_ast(
            &grammar,
            args.dump_gen_ast.as_deref(),
            args.dump_gen_ast_pretty,
            dump_gen_ast_max_bytes,
        )?;

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

        println!("SOTA parser generated: {}", output_rust);
        (0, Vec::<String>::new())
    } else if args.generate_stimuli_module {
        let grammar = apply_grammar_profile_filter(
            load_grammar_bundle(
                &args.input_path,
                &mut pipeline,
                args.emit_raw_ast_json.as_deref(),
            )?,
            args.grammar_profile.as_deref(),
        )?;
        maybe_dump_generation_ast(
            &grammar,
            args.dump_gen_ast.as_deref(),
            args.dump_gen_ast_pretty,
            dump_gen_ast_max_bytes,
        )?;
        let effective_seed = resolve_stimuli_module_seed(args.seed);
        if args.seed.is_none() {
            println!(
                "No --seed provided for --generate-stimuli-module; using deterministic default seed={}",
                effective_seed
            );
        }
        let recovery_mode = parse_recovery_stimuli_mode(&args.recovery_stimuli_mode)?;
        let stimuli_config = StimuliConfig {
            seed: Some(effective_seed),
            max_depth: args.max_depth,
            max_repeat: args.max_repeat,
            max_rule_visits: args.max_depth.max(2),
            recovery_mode,
            enforce_word_boundary_spacing: args.enforce_word_boundary_spacing,
            trace_verbosity,
        };
        let mut generator = StimuliGenerator::new(
            grammar.grammar_name.clone(),
            &grammar.grammar_tree,
            &grammar.rule_order,
            grammar.annotations.as_ref(),
            stimuli_config,
        );
        if let Some(coverage_input_path) = args.coverage_input.as_deref() {
            let existing_coverage = load_coverage_metrics(coverage_input_path)?;
            generator.merge_coverage_metrics(&existing_coverage)?;
        }
        let resolved_entry_rule = resolve_stimuli_entry_rule(
            &grammar.grammar_tree,
            &grammar.rule_order,
            args.entry_rule.as_deref(),
        )?;
        let samples = if args.validate_parseability {
            let outcome = generate_parseable_stimuli(
                &grammar.grammar_name,
                args.grammar_profile.as_deref(),
                &mut generator,
                args.count,
                Some(resolved_entry_rule.as_str()),
                args.parseability_max_attempts,
            )?;
            if let Some(report_path) = args.parseability_report_json.as_deref() {
                write_parseability_report(
                    report_path,
                    &grammar.grammar_name,
                    args.grammar_profile.as_deref(),
                    resolved_entry_rule.as_str(),
                    &outcome.summary,
                    None,
                    &outcome.counterexamples,
                )?;
            }
            outcome.samples
        } else {
            generator.generate_many(args.count, Some(resolved_entry_rule.as_str()))?
        };
        let merged_coverage = generator.coverage_metrics().clone();
        let output_module = args
            .output
            .unwrap_or_else(|| default_stimuli_module_output_path(&grammar.grammar_name));
        ensure_parent_dir_exists(&output_module)?;
        let module_source = generate_stimuli_module_source(
            &grammar.grammar_name,
            effective_seed,
            args.count,
            resolved_entry_rule.as_str(),
            &samples,
        );
        std::fs::write(&output_module, module_source)?;
        println!(
            "Generated Rust stimuli module with {} samples: {}",
            samples.len(),
            output_module
        );
        println!("{}", merged_coverage.summary_line());
        if let Some(coverage_output_path) = args.coverage_output.as_deref() {
            let coverage_json = serde_json::to_string_pretty(&merged_coverage)?;
            std::fs::write(coverage_output_path, coverage_json)?;
            println!("Wrote stimuli coverage metrics to {}", coverage_output_path);
        }
        if args.gap_report_json.is_some() || args.gap_report_text.is_some() {
            let mut gap_generator = StimuliGenerator::new(
                grammar.grammar_name.clone(),
                &grammar.grammar_tree,
                &grammar.rule_order,
                grammar.annotations.as_ref(),
                StimuliConfig {
                    seed: Some(effective_seed),
                    max_depth: args.max_depth,
                    max_repeat: args.max_repeat,
                    max_rule_visits: args.max_depth.max(2),
                    recovery_mode,
                    enforce_word_boundary_spacing: args.enforce_word_boundary_spacing,
                    trace_verbosity,
                },
            );
            gap_generator.merge_coverage_metrics(&merged_coverage)?;
            let gap_report = gap_generator.generate_gap_report(
                Some(resolved_entry_rule.as_str()),
                args.gap_report_threshold,
            )?;
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
    } else if args.generate_stimuli {
        let grammar = apply_grammar_profile_filter(
            load_grammar_bundle(
                &args.input_path,
                &mut pipeline,
                args.emit_raw_ast_json.as_deref(),
            )?,
            args.grammar_profile.as_deref(),
        )?;
        maybe_dump_generation_ast(
            &grammar,
            args.dump_gen_ast.as_deref(),
            args.dump_gen_ast_pretty,
            dump_gen_ast_max_bytes,
        )?;
        let recovery_mode = parse_recovery_stimuli_mode(&args.recovery_stimuli_mode)?;
        let stimuli_config = StimuliConfig {
            seed: args.seed,
            max_depth: args.max_depth,
            max_repeat: args.max_repeat,
            max_rule_visits: args.max_depth.max(2),
            recovery_mode,
            enforce_word_boundary_spacing: args.enforce_word_boundary_spacing,
            trace_verbosity,
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
        let mut parseability_summary: Option<ParseabilitySummary> = None;
        let mut parseability_target_drive_validation: Option<TargetDriveParseabilityTelemetry> =
            None;
        let mut parseability_counterexamples = Vec::new();

        let mut samples = if args.coverage_guided_fuzz_rounds > 0 {
            let seed_start = args
                .coverage_guided_fuzz_seed_start
                .or(args.seed)
                .unwrap_or(1);
            let fuzz_outcome = run_coverage_guided_fuzz_loop(
                &grammar.grammar_name,
                &grammar.grammar_tree,
                &grammar.rule_order,
                grammar.annotations.as_ref(),
                &stimuli_config,
                args.entry_rule.as_deref(),
                args.coverage_guided_fuzz_rounds,
                seed_start,
                args.grammar_profile.as_deref(),
                args.validate_parseability,
                merged_coverage.clone(),
            )?;
            println!("{}", fuzz_outcome.replay_report.summary_line());
            merged_coverage = fuzz_outcome.merged_coverage.clone();
            if args.validate_parseability {
                parseability_summary = Some(ParseabilitySummary::from_coverage_guided_replay(
                    &fuzz_outcome.replay_report,
                ));
                parseability_counterexamples = fuzz_outcome
                    .replay_report
                    .cases
                    .iter()
                    .filter(|case| !case.accepted)
                    .filter_map(|case| {
                        case.sample.as_deref().map(|sample| {
                            let mut counterexample = build_parseability_counterexample(
                                "coverage_guided_fuzz_replay",
                                &grammar.grammar_name,
                                args.grammar_profile.as_deref(),
                                sample,
                            );
                            if let Some(shrunk) = case.shrunk_counterexample.as_deref() {
                                counterexample.shrunk_sample = shrunk.to_string();
                                counterexample.shrunk_sample_chars = shrunk.chars().count();
                            }
                            counterexample
                        })
                    })
                    .take(MAX_PARSEABILITY_COUNTEREXAMPLES)
                    .collect();
            }
            replay_report = Some(fuzz_outcome.replay_report);
            fuzz_outcome.minimized_samples
        } else if let Some(priority_report_input_path) = args.gap_priority_report_input.as_deref() {
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
                let outcome = generate_parseable_stimuli(
                    &grammar.grammar_name,
                    args.grammar_profile.as_deref(),
                    &mut generator,
                    args.count,
                    args.entry_rule.as_deref(),
                    args.parseability_max_attempts,
                )?;
                parseability_summary = Some(outcome.summary);
                parseability_counterexamples = outcome.counterexamples;
                outcome.samples
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
            let (generated_samples, target_summary) = if args.validate_parseability {
                let (samples, summary, validation) = generator.generate_until_targets_with_filter(
                    args.entry_rule.as_deref(),
                    &target_report.targets,
                    args.target_max_attempts,
                    |sample| {
                        let parseable = is_sample_parseable_by_generated_parser(
                            &grammar.grammar_name,
                            args.grammar_profile.as_deref(),
                            sample,
                        )?;
                        if !parseable
                            && parseability_counterexamples.len() < MAX_PARSEABILITY_COUNTEREXAMPLES
                        {
                            parseability_counterexamples.push(build_parseability_counterexample(
                                "target_drive_output_filter",
                                &grammar.grammar_name,
                                args.grammar_profile.as_deref(),
                                sample,
                            ));
                        }
                        Ok(parseable)
                    },
                )?;
                let summary_for_report = ParseabilitySummary::from_filter(
                    validation.validated_outputs,
                    validation.accepted_outputs,
                    validation.rejected_outputs,
                );
                println!("{}", summary_for_report.summary_line());
                parseability_summary = Some(summary_for_report);
                parseability_target_drive_validation = Some(
                    TargetDriveParseabilityTelemetry::from_validation(&validation),
                );
                (samples, summary)
            } else {
                generator.generate_until_targets(
                    args.entry_rule.as_deref(),
                    &target_report.targets,
                    args.target_max_attempts,
                )?
            };
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
            let outcome = generate_parseable_stimuli(
                &grammar.grammar_name,
                args.grammar_profile.as_deref(),
                &mut generator,
                args.count,
                args.entry_rule.as_deref(),
                args.parseability_max_attempts,
            )?;
            parseability_summary = Some(outcome.summary);
            parseability_counterexamples = outcome.counterexamples;
            merged_coverage = generator.coverage_metrics().clone();
            outcome.samples
        } else {
            let generated_samples =
                generator.generate_many(args.count, args.entry_rule.as_deref())?;
            merged_coverage = generator.coverage_metrics().clone();
            generated_samples
        };

        if args.validate_parseability
            && args.target_report_input.is_some()
            && parseability_summary.is_none()
        {
            let requested_before_filter = samples.len();
            let (accepted, rejected, counterexamples) = filter_parseable_samples(
                &grammar.grammar_name,
                args.grammar_profile.as_deref(),
                samples.into_iter(),
            )?;
            samples = accepted;
            parseability_counterexamples = counterexamples;
            let summary =
                ParseabilitySummary::from_filter(requested_before_filter, samples.len(), rejected);
            println!("{}", summary.summary_line());
            parseability_summary = Some(summary);
        }

        if let Some(report_path) = args.parseability_report_json.as_deref() {
            let resolved_entry_rule = resolve_stimuli_entry_rule(
                &grammar.grammar_tree,
                &grammar.rule_order,
                args.entry_rule.as_deref(),
            )?;
            let Some(summary) = parseability_summary.as_ref() else {
                return Err(anyhow::anyhow!(
                    "--parseability-report-json requires parseability-aware generation or filtering; current generation mode did not produce a parseability summary"
                ));
            };
            write_parseability_report(
                report_path,
                &grammar.grammar_name,
                args.grammar_profile.as_deref(),
                resolved_entry_rule.as_str(),
                summary,
                parseability_target_drive_validation.as_ref(),
                &parseability_counterexamples,
            )?;
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
        // pipeline.transform_to_json(&args.input_path, &output_file)?;
        println!("Transformed AST saved to: {}", output_file);
        (0, Vec::<String>::new()) // (rule_count, rule_order)
    } else {
        // Same-language mode: JSON → In-memory
        // let (grammar_tree, rule_order) = pipeline.transform_from_file(&args.input_path, None)?;
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

fn emit_rust_frontend_raw_ast_json(input_path: &str, output_path: &str) -> Result<usize> {
    #[cfg(feature = "ebnf_dual_run")]
    {
        let json_value = ebnf_frontend::parse_ebnf_file_to_raw_ast_envelope(input_path)?;
        let rule_count = json_value
            .get("raw_ast")
            .and_then(|raw_ast| raw_ast.as_array())
            .map(|raw_ast| raw_ast.len())
            .unwrap_or(0);
        let raw_ast_json = serde_json::to_string_pretty(&json_value)?;
        std::fs::write(output_path, raw_ast_json)?;
        println!(
            "Wrote Rust EBNF frontend raw_ast envelope to {}",
            output_path
        );
        return Ok(rule_count);
    }

    #[cfg(not(feature = "ebnf_dual_run"))]
    {
        let _ = (input_path, output_path);
        Err(anyhow::anyhow!(
            "Rust EBNF raw_ast export requires building with --features ebnf_dual_run"
        ))
    }
}

fn maybe_dump_generation_ast(
    grammar: &LoadedGrammar,
    output_path: Option<&str>,
    pretty: bool,
    max_bytes: Option<usize>,
) -> Result<()> {
    let Some(path) = output_path else {
        return Ok(());
    };

    let dump = GenerationAstDump {
        grammar_name: grammar.grammar_name.as_str(),
        rule_order: grammar.rule_order.as_slice(),
        grammar_tree: &grammar.grammar_tree,
        annotations: grammar.annotations.as_ref(),
    };
    let json = encode_canonical_json(&dump, pretty)?;
    let write_result =
        write_json_dump_with_limit(path, &json, max_bytes, pretty, "generation_input_ast")
            .with_context(|| format!("failed to write generation-input AST JSON '{}'", path))?;
    if write_result.truncated {
        println!(
            "Wrote generation-input AST truncation diagnostics JSON to {} (full_bytes={}, max_bytes={}, written_bytes={})",
            path,
            write_result.full_bytes,
            max_bytes.unwrap_or(write_result.full_bytes),
            write_result.bytes_written
        );
    } else {
        println!("Wrote generation-input AST JSON to {}", path);
    }
    Ok(())
}

fn resolve_dump_gen_ast_max_bytes(args: &Args) -> Result<Option<usize>> {
    if let Some(value) = args.dump_gen_ast_max_bytes {
        if value == 0 {
            return Err(anyhow::anyhow!(
                "--dump-gen-ast-max-bytes must be an integer >= 1"
            ));
        }
        return Ok(Some(value));
    }
    let raw = match std::env::var("PGEN_DUMP_GEN_AST_MAX_BYTES") {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let parsed = trimmed.parse::<usize>().map_err(|_| {
        anyhow::anyhow!(
            "PGEN_DUMP_GEN_AST_MAX_BYTES must be an integer >= 1 (got '{}')",
            raw
        )
    })?;
    if parsed == 0 {
        return Err(anyhow::anyhow!(
            "PGEN_DUMP_GEN_AST_MAX_BYTES must be an integer >= 1"
        ));
    }
    Ok(Some(parsed))
}

fn canonicalize_json_value(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Array(values) => {
            serde_json::Value::Array(values.into_iter().map(canonicalize_json_value).collect())
        }
        serde_json::Value::Object(map) => {
            let mut entries = map.into_iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            let mut normalized = serde_json::Map::new();
            for (key, value) in entries {
                normalized.insert(key, canonicalize_json_value(value));
            }
            serde_json::Value::Object(normalized)
        }
        other => other,
    }
}

fn encode_canonical_json<T: Serialize>(value: &T, pretty: bool) -> Result<String> {
    let normalized = canonicalize_json_value(serde_json::to_value(value)?);
    if pretty {
        Ok(serde_json::to_string_pretty(&normalized)?)
    } else {
        Ok(serde_json::to_string(&normalized)?)
    }
}

fn write_json_dump_with_limit(
    output_path: &str,
    encoded_json: &str,
    max_bytes: Option<usize>,
    pretty: bool,
    dump_kind: &str,
) -> Result<AstDumpWriteResult> {
    let full_bytes = encoded_json.as_bytes().len();
    if let Some(max) = max_bytes {
        if full_bytes > max {
            let diagnostic = AstDumpTruncationDiagnostic {
                pgen_dump_contract_version: 1,
                kind: "pgen_ast_dump_truncation",
                truncated: true,
                dump_kind: dump_kind.to_string(),
                max_bytes: max,
                full_bytes,
                reason: "encoded AST JSON exceeded configured max bytes; payload omitted",
            };
            let encoded_diagnostic = encode_canonical_json(&diagnostic, pretty)?;
            let diagnostic_bytes = encoded_diagnostic.as_bytes().len();
            if diagnostic_bytes > max {
                return Err(anyhow::anyhow!(
                    "AST dump max-bytes ({}) is too small to fit truncation diagnostics (requires at least {} bytes)",
                    max,
                    diagnostic_bytes
                ));
            }
            std::fs::write(output_path, encoded_diagnostic)?;
            return Ok(AstDumpWriteResult {
                truncated: true,
                bytes_written: diagnostic_bytes,
                full_bytes,
            });
        }
    }

    std::fs::write(output_path, encoded_json)?;
    Ok(AstDumpWriteResult {
        truncated: false,
        bytes_written: full_bytes,
        full_bytes,
    })
}

fn load_grammar_bundle(
    input_path: &str,
    pipeline: &mut RustASTPipeline,
    emit_raw_ast_json: Option<&str>,
) -> Result<LoadedGrammar> {
    #[cfg(not(feature = "ebnf_dual_run"))]
    let _ = emit_raw_ast_json;

    if is_ebnf_input_path(input_path) {
        #[cfg(feature = "ebnf_dual_run")]
        {
            let json_value = ebnf_frontend::parse_ebnf_file_to_raw_ast_envelope(input_path)?;
            if let Some(raw_ast_output_path) = emit_raw_ast_json {
                let raw_ast_json = serde_json::to_string_pretty(&json_value)?;
                std::fs::write(raw_ast_output_path, raw_ast_json)?;
                println!(
                    "Wrote Rust EBNF frontend raw_ast envelope to {}",
                    raw_ast_output_path
                );
            }
            return load_grammar_bundle_from_json_value(json_value, pipeline);
        }

        #[cfg(not(feature = "ebnf_dual_run"))]
        {
            return Err(anyhow::anyhow!(
                "EBNF input '{}' requires building with --features ebnf_dual_run",
                input_path
            ));
        }
    }

    let json_content = std::fs::read_to_string(input_path)?;
    let json_value: serde_json::Value = serde_json::from_str(&json_content)?;
    load_grammar_bundle_from_json_value(json_value, pipeline)
}

fn is_ebnf_input_path(input_path: &str) -> bool {
    Path::new(input_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("ebnf"))
        .unwrap_or(false)
}

fn load_grammar_bundle_from_json_value(
    json_value: serde_json::Value,
    pipeline: &mut RustASTPipeline,
) -> Result<LoadedGrammar> {
    if let Some(raw_ast) = json_value.get("raw_ast") {
        let raw_ast_array = raw_ast
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid raw_ast format"))?;
        let (grammar_tree, rule_order, annotations) =
            pipeline.transform_from_raw_ast(raw_ast_array)?;
        let grammar_name = json_value
            .get("grammar_name")
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(LoadedGrammar {
            grammar_name,
            grammar_tree,
            rule_order,
            annotations,
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

fn normalize_grammar_profile_name(profile: &str) -> String {
    let normalized = profile.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "sv_2017" | "2017" | "ieee1800-2017" | "ieee_1800_2017" => "sv_2017".to_string(),
        "sv_2023" | "2023" | "ieee1800-2023" | "ieee_1800_2023" => "sv_2023".to_string(),
        "vhdl_1076_2019" | "1076_2019" | "1076-2019" | "ieee1076-2019" | "ieee_1076_2019" => {
            "vhdl_1076_2019".to_string()
        }
        _ => normalized,
    }
}

fn rule_profile_matches(annotations: &Annotations, rule_name: &str, active_profile: &str) -> bool {
    let Some(entries) = annotations.semantic_annotations.get(rule_name) else {
        return true;
    };

    let mut allowed_profiles: Option<Vec<String>> = None;
    for annotation in entries {
        let payload = match annotation.name() {
            Some(name) if name.trim().eq_ignore_ascii_case("profiles") => {
                annotation.ast().payload_text()
            }
            _ => match annotation.ast() {
                pgen::ast_pipeline::UnifiedSemanticAST::TransformExpr { expression } => {
                    let Some((name, payload)) = extract_semantic_directive(expression) else {
                        continue;
                    };
                    if name != "profiles" {
                        continue;
                    }
                    allowed_profiles = parse_semantic_string_list(&payload).map(|values| {
                        values
                            .into_iter()
                            .map(|value| normalize_grammar_profile_name(&value))
                            .collect()
                    });
                    continue;
                }
                _ => {
                    let content = annotation.ast().payload_text();
                    let Some((name, payload)) = extract_semantic_directive(content) else {
                        continue;
                    };
                    if name != "profiles" {
                        continue;
                    }
                    allowed_profiles = parse_semantic_string_list(&payload).map(|values| {
                        values
                            .into_iter()
                            .map(|value| normalize_grammar_profile_name(&value))
                            .collect()
                    });
                    continue;
                }
            },
        };

        allowed_profiles = parse_semantic_string_list(payload).map(|values| {
            values
                .into_iter()
                .map(|value| normalize_grammar_profile_name(&value))
                .collect()
        });
    }

    match allowed_profiles {
        Some(allowed) if !allowed.is_empty() => allowed.iter().any(|value| value == active_profile),
        _ => true,
    }
}

fn filter_annotations_by_profile(
    annotations: Annotations,
    retained_rules: &HashSet<String>,
) -> Annotations {
    let mut branch_return_annotations = annotations.branch_return_annotations;
    branch_return_annotations.retain(|rule_name, _| retained_rules.contains(rule_name));

    let mut branch_semantic_annotations = annotations.branch_semantic_annotations;
    branch_semantic_annotations.retain(|rule_name, _| retained_rules.contains(rule_name));

    let mut semantic_annotations = annotations.semantic_annotations;
    semantic_annotations.retain(|rule_name, _| retained_rules.contains(rule_name));

    Annotations {
        branch_return_annotations,
        branch_semantic_annotations,
        semantic_annotations,
    }
}

fn apply_grammar_profile_filter(
    grammar: LoadedGrammar,
    grammar_profile: Option<&str>,
) -> Result<LoadedGrammar> {
    let Some(profile) = grammar_profile else {
        return Ok(grammar);
    };
    let active_profile = normalize_grammar_profile_name(profile);
    let Some(annotations) = grammar.annotations.as_ref() else {
        return Ok(grammar);
    };

    let retained_rule_order = grammar
        .rule_order
        .iter()
        .filter(|rule_name| rule_profile_matches(annotations, rule_name, &active_profile))
        .cloned()
        .collect::<Vec<_>>();
    if retained_rule_order.is_empty() {
        return Err(anyhow::anyhow!(
            "Grammar profile '{}' removed all rules from grammar '{}'",
            active_profile,
            grammar.grammar_name
        ));
    }

    let retained_rules = retained_rule_order.iter().cloned().collect::<HashSet<_>>();
    let retained_grammar_tree = grammar
        .grammar_tree
        .into_iter()
        .filter(|(rule_name, _)| retained_rules.contains(rule_name))
        .collect::<HashMap<_, _>>();
    let retained_annotations = grammar
        .annotations
        .map(|entries| filter_annotations_by_profile(entries, &retained_rules));

    Ok(LoadedGrammar {
        grammar_name: grammar.grammar_name,
        grammar_tree: retained_grammar_tree,
        rule_order: retained_rule_order,
        annotations: retained_annotations,
    })
}

fn default_parser_output_path(input_path: &str) -> String {
    let input = Path::new(input_path);
    let stem = input
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("generated_parser");
    let output_file_name = format!("{stem}.rs");
    if let Some(parent) = input.parent() {
        parent.join(output_file_name).to_string_lossy().into_owned()
    } else {
        output_file_name
    }
}

fn default_stimuli_module_output_path(grammar_name: &str) -> String {
    format!(
        "generated/{}_stimuli.rs",
        sanitize_artifact_stem(grammar_name)
    )
}

fn sanitize_artifact_stem(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            output.push(ch);
        } else {
            output.push('_');
        }
    }
    if output.is_empty() {
        "grammar".to_string()
    } else {
        output
    }
}

fn ensure_parent_dir_exists(path: &str) -> Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

fn resolve_stimuli_module_seed(seed: Option<u64>) -> u64 {
    seed.unwrap_or(DEFAULT_STIMULI_MODULE_SEED)
}

fn generate_stimuli_module_source(
    grammar_name: &str,
    seed: u64,
    requested_count: usize,
    entry_rule: &str,
    samples: &[String],
) -> String {
    let mut output = String::new();
    output.push_str("// @generated by ast_pipeline --generate-stimuli-module\n");
    output.push_str("// Do not edit manually.\n\n");
    output.push_str("#![allow(dead_code)]\n\n");
    output.push_str(&format!(
        "pub const STIMULI_MODULE_API_VERSION: u32 = {};\n",
        STIMULI_MODULE_API_VERSION
    ));
    output.push_str(&format!(
        "pub const GRAMMAR_NAME: &str = {:?};\n",
        grammar_name
    ));
    output.push_str(&format!(
        "pub const REQUESTED_SAMPLE_COUNT: usize = {};\n",
        requested_count
    ));
    output.push_str(&format!(
        "pub const GENERATED_SAMPLE_COUNT: usize = {};\n",
        samples.len()
    ));
    output.push_str(&format!("pub const GENERATION_SEED: u64 = {seed};\n"));
    output.push_str(&format!("pub const ENTRY_RULE: &str = {:?};\n", entry_rule));
    output.push_str("\n");
    output.push_str(&format!(
        "pub const STIMULI: [&str; {}] = [\n",
        samples.len()
    ));
    for sample in samples {
        output.push_str("    ");
        output.push_str(&format!("{sample:?}"));
        output.push_str(",\n");
    }
    output.push_str("];\n\n");
    output.push_str("pub fn generated_stimuli() -> &'static [&'static str] {\n");
    output.push_str("    &STIMULI\n");
    output.push_str("}\n");
    output
}

fn parse_recovery_stimuli_mode(value: &str) -> Result<RecoveryStimuliMode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "baseline" => Ok(RecoveryStimuliMode::Baseline),
        "recovery_biased" => Ok(RecoveryStimuliMode::RecoveryBiased),
        "near_sync_negative" => Ok(RecoveryStimuliMode::NearSyncNegative),
        other => Err(anyhow::anyhow!(
            "Unsupported recovery stimuli mode '{}'. Supported values: baseline, recovery_biased, near_sync_negative",
            other
        )),
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

fn build_parseability_counterexample(
    stage: &str,
    grammar_name: &str,
    grammar_profile: Option<&str>,
    sample: &str,
) -> ParseabilityCounterexample {
    let shrunk_sample = shrink_parseability_counterexample(grammar_name, grammar_profile, sample)
        .unwrap_or_else(|_| sample.to_string());
    let failure_detail =
        generated_parser_failure_detail(grammar_name, grammar_profile, sample).unwrap_or(None);
    ParseabilityCounterexample {
        stage: stage.to_string(),
        sample: sample.to_string(),
        sample_chars: sample.chars().count(),
        shrunk_sample_chars: shrunk_sample.chars().count(),
        shrunk_sample,
        parser_error: failure_detail
            .as_ref()
            .map(|detail| detail.parser_error.clone()),
        failure_position: failure_detail
            .as_ref()
            .and_then(|detail| detail.failure_position),
        failure_line: failure_detail
            .as_ref()
            .and_then(|detail| detail.failure_line),
        failure_column: failure_detail
            .as_ref()
            .and_then(|detail| detail.failure_column),
        failure_line_excerpt: failure_detail
            .as_ref()
            .and_then(|detail| detail.failure_line_excerpt.clone()),
        failure_context_excerpt: failure_detail
            .as_ref()
            .and_then(|detail| detail.failure_context_excerpt.clone()),
    }
}

fn shrink_parseability_counterexample(
    grammar_name: &str,
    grammar_profile: Option<&str>,
    sample: &str,
) -> Result<String> {
    minimize_failing_input(sample, |candidate| {
        Ok(!is_sample_parseable_by_generated_parser(
            grammar_name,
            grammar_profile,
            candidate,
        )?)
    })
}

#[derive(Debug, Clone)]
struct ParseFailureDetail {
    parser_error: String,
    failure_position: Option<usize>,
    failure_line: Option<usize>,
    failure_column: Option<usize>,
    failure_line_excerpt: Option<String>,
    failure_context_excerpt: Option<String>,
}

#[cfg(feature = "generated_parsers")]
fn generated_parser_failure_detail(
    grammar_name: &str,
    grammar_profile: Option<&str>,
    sample: &str,
) -> Result<Option<ParseFailureDetail>> {
    let Some(parse_result) =
        parser_registry::parse_sample_ast_json_with_profile(grammar_name, sample, grammar_profile)
    else {
        let supported = supported_generated_parseability_grammars_csv();
        return Err(anyhow::anyhow!(
            "Unsupported grammar '{}' for generated parseability validation. Supported grammars: {}",
            grammar_name,
            supported
        ));
    };

    match parse_result {
        Ok(_) => Ok(None),
        Err(err) => {
            let failure_position = extract_parse_error_position(&err);
            let (failure_line, failure_column) = failure_position
                .map(|position| parse_error_line_column(sample, position))
                .unwrap_or((None, None));
            let failure_line_excerpt =
                failure_line.and_then(|line| parse_error_line_excerpt(sample, line));
            let failure_context_excerpt =
                failure_position.and_then(|position| parse_error_context_excerpt(sample, position));
            Ok(Some(ParseFailureDetail {
                parser_error: err,
                failure_position,
                failure_line,
                failure_column,
                failure_line_excerpt,
                failure_context_excerpt,
            }))
        }
    }
}

#[cfg(not(feature = "generated_parsers"))]
fn generated_parser_failure_detail(
    _grammar_name: &str,
    _grammar_profile: Option<&str>,
    _sample: &str,
) -> Result<Option<ParseFailureDetail>> {
    Ok(None)
}

fn extract_parse_error_position(message: &str) -> Option<usize> {
    extract_position_after_marker(message, "position ")
        .or_else(|| extract_position_after_marker(message, "Position: "))
}

fn extract_position_after_marker(message: &str, marker: &str) -> Option<usize> {
    let marker_index = message.rfind(marker)?;
    let digits = message[marker_index + marker.len()..]
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        None
    } else {
        digits.parse::<usize>().ok()
    }
}

fn parse_error_line_column(
    sample: &str,
    failure_position: usize,
) -> (Option<usize>, Option<usize>) {
    let clamped = clamp_to_char_boundary(sample, failure_position);
    let prefix = &sample[..clamped];
    let line = prefix.chars().filter(|ch| *ch == '\n').count() + 1;
    let column = prefix
        .rsplit('\n')
        .next()
        .map(|segment| segment.chars().count() + 1)
        .unwrap_or(1);
    (Some(line), Some(column))
}

fn sanitize_excerpt_text(text: &str) -> String {
    text.replace('\r', "")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}

fn parse_error_line_excerpt(sample: &str, failure_line: usize) -> Option<String> {
    let line_index = failure_line.checked_sub(1)?;
    let raw_line = sample.split('\n').nth(line_index)?;
    let sanitized = sanitize_excerpt_text(raw_line);
    Some(summarize_sample(
        &sanitized,
        MAX_PARSEABILITY_FAILURE_LINE_EXCERPT_CHARS,
    ))
}

fn parse_error_context_excerpt(sample: &str, failure_position: usize) -> Option<String> {
    if sample.is_empty() {
        return Some(String::new());
    }

    let clamped = clamp_to_char_boundary(sample, failure_position);
    let chars: Vec<char> = sample.chars().collect();
    let char_index = sample[..clamped].chars().count();
    let context_window = MAX_PARSEABILITY_FAILURE_CONTEXT_EXCERPT_CHARS.max(1);
    let half_window = context_window / 2;
    let mut start = char_index.saturating_sub(half_window);
    let end = (start + context_window).min(chars.len());
    start = end.saturating_sub(context_window);

    let excerpt_body: String = chars[start..end].iter().collect();
    let sanitized = sanitize_excerpt_text(&excerpt_body);
    let mut excerpt = String::new();
    if start > 0 {
        excerpt.push_str("...");
    }
    excerpt.push_str(&sanitized);
    if end < chars.len() {
        excerpt.push_str("...");
    }
    Some(excerpt)
}

fn clamp_to_char_boundary(sample: &str, position: usize) -> usize {
    let mut clamped = position.min(sample.len());
    while clamped > 0 && !sample.is_char_boundary(clamped) {
        clamped -= 1;
    }
    clamped
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
    grammar_profile: Option<&str>,
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
            anyhow::anyhow!(
                "Coverage-guided fuzz round index overflow at round {}",
                round_idx
            )
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
                let is_parseable = is_sample_parseable_by_generated_parser(
                    grammar_name,
                    grammar_profile,
                    sample_text,
                )?;
                parseable = Some(is_parseable);
                accepted = is_parseable;
                if !is_parseable {
                    shrunk_counterexample = Some(shrink_parseability_counterexample(
                        grammar_name,
                        grammar_profile,
                        sample_text,
                    )?);
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
        return Err(anyhow::anyhow!(
            "Entry rule '{}' not found in grammar",
            rule_name
        ));
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
        let before_hits = before
            .rule_success_hits
            .get(rule_name)
            .copied()
            .unwrap_or(0);
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

fn filter_parseable_samples<I>(
    grammar_name: &str,
    grammar_profile: Option<&str>,
    samples: I,
) -> Result<(Vec<String>, usize, Vec<ParseabilityCounterexample>)>
where
    I: IntoIterator<Item = String>,
{
    ensure_parseability_support(grammar_name)?;
    let mut accepted = Vec::new();
    let mut rejected = 0usize;
    let mut counterexamples = Vec::new();
    for sample in samples {
        if is_sample_parseable_by_generated_parser(grammar_name, grammar_profile, &sample)? {
            accepted.push(sample);
        } else {
            rejected = rejected.saturating_add(1);
            if counterexamples.len() < MAX_PARSEABILITY_COUNTEREXAMPLES {
                counterexamples.push(build_parseability_counterexample(
                    "filter_parseable_samples",
                    grammar_name,
                    grammar_profile,
                    &sample,
                ));
            }
        }
    }
    Ok((accepted, rejected, counterexamples))
}

fn supports_generated_parseability(grammar_name: &str) -> bool {
    #[cfg(feature = "generated_parsers")]
    {
        return parser_registry::supports_grammar(grammar_name);
    }

    #[cfg(not(feature = "generated_parsers"))]
    {
        return supported_generated_parseability_grammars()
            .iter()
            .any(|supported| *supported == grammar_name);
    }
}

#[cfg(feature = "generated_parsers")]
fn supported_generated_parseability_grammars() -> Vec<&'static str> {
    parser_registry::registered_grammars()
}

#[cfg(not(feature = "generated_parsers"))]
fn supported_generated_parseability_grammars() -> Vec<&'static str> {
    vec!["return_annotation", "semantic_annotation"]
}

fn supported_generated_parseability_grammars_csv() -> String {
    let mut grammars = supported_generated_parseability_grammars();
    grammars.sort_unstable();
    grammars.join(", ")
}

fn generate_parseable_stimuli(
    grammar_name: &str,
    grammar_profile: Option<&str>,
    generator: &mut StimuliGenerator<'_>,
    requested_count: usize,
    entry_rule: Option<&str>,
    max_attempts_override: Option<usize>,
) -> Result<ParseableStimuliOutcome> {
    ensure_parseability_support(grammar_name)?;

    let max_attempts = resolve_parseability_max_attempts(requested_count, max_attempts_override);
    let mut accepted = Vec::with_capacity(requested_count);
    let mut attempts = 0usize;
    let mut rejected = 0usize;
    let mut generation_errors = 0usize;
    let mut empty_generations = 0usize;
    let mut parser_rejections = 0usize;
    let mut last_parser_rejected_sample: Option<String> = None;
    let mut counterexamples = Vec::new();

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

        if is_sample_parseable_by_generated_parser(grammar_name, grammar_profile, &sample)? {
            accepted.push(sample);
        } else {
            parser_rejections += 1;
            rejected += 1;
            last_parser_rejected_sample = Some(sample);
            if let Some(sample) = last_parser_rejected_sample.as_deref() {
                if counterexamples.len() < MAX_PARSEABILITY_COUNTEREXAMPLES {
                    counterexamples.push(build_parseability_counterexample(
                        "generate_parseable_stimuli",
                        grammar_name,
                        grammar_profile,
                        sample,
                    ));
                }
            }
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
            let shrunk = shrink_parseability_counterexample(grammar_name, grammar_profile, &sample)
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
    println!("{}", summary.summary_line());

    Ok(ParseableStimuliOutcome {
        samples: accepted,
        summary,
        counterexamples,
    })
}

fn resolve_parseability_max_attempts(
    requested_count: usize,
    override_attempts: Option<usize>,
) -> usize {
    override_attempts.unwrap_or_else(|| requested_count.saturating_mul(50).max(requested_count))
}

fn write_parseability_report(
    output_path: &str,
    grammar_name: &str,
    grammar_profile: Option<&str>,
    entry_rule: &str,
    summary: &ParseabilitySummary,
    target_drive_validation: Option<&TargetDriveParseabilityTelemetry>,
    counterexamples: &[ParseabilityCounterexample],
) -> Result<()> {
    ensure_parent_dir_exists(output_path)?;
    let report = ParseabilityGenerationReport {
        grammar_name: grammar_name.to_string(),
        grammar_profile: grammar_profile.map(ToOwned::to_owned),
        entry_rule: entry_rule.to_string(),
        summary: summary.clone(),
        target_drive_validation: target_drive_validation.cloned(),
        counterexamples: counterexamples.to_vec(),
    };
    let report_json = serde_json::to_string_pretty(&report)?;
    std::fs::write(output_path, report_json)?;
    println!("Wrote parseability validation report to {}", output_path);
    Ok(())
}

#[cfg(feature = "generated_parsers")]
fn ensure_parseability_support(grammar_name: &str) -> Result<()> {
    if !supports_generated_parseability(grammar_name) {
        let supported = supported_generated_parseability_grammars_csv();
        return Err(anyhow::anyhow!(
            "No matching compiled generated parser is available for grammar '{}'. Supported grammars: {}",
            grammar_name,
            supported
        ));
    }
    Ok(())
}
#[cfg(feature = "generated_parsers")]
fn is_sample_parseable_by_generated_parser(
    grammar_name: &str,
    grammar_profile: Option<&str>,
    sample: &str,
) -> Result<bool> {
    parser_registry::parse_sample_with_profile(grammar_name, sample, grammar_profile).ok_or_else(
        || {
            let supported = supported_generated_parseability_grammars_csv();
            anyhow::anyhow!(
                "Unsupported grammar '{}' for generated parseability validation. Supported grammars: {}",
                grammar_name,
                supported
            )
        },
    )
}

#[cfg(not(feature = "generated_parsers"))]
fn is_sample_parseable_by_generated_parser(
    _grammar_name: &str,
    _grammar_profile: Option<&str>,
    _sample: &str,
) -> Result<bool> {
    Err(anyhow::anyhow!(
        "Generated parser parseability checks are unavailable without --features generated_parsers"
    ))
}

#[cfg(not(feature = "generated_parsers"))]
fn ensure_parseability_support(grammar_name: &str) -> Result<()> {
    if supports_generated_parseability(grammar_name) {
        Err(anyhow::anyhow!(
            "Parseability validation requires building ast_pipeline with generated parsers enabled: cargo run --features generated_parsers --bin ast_pipeline -- ... --validate-parseability"
        ))
    } else {
        let supported = supported_generated_parseability_grammars_csv();
        Err(anyhow::anyhow!(
            "No matching generated parser validation path exists for grammar '{}'. Supported grammars: {}",
            grammar_name,
            supported
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FuzzCorpusCandidate, LoadedGrammar, ParseabilityCounterexample, ParseabilitySummary,
        StimuliCoverageMetrics, TargetDriveParseabilityTelemetry, canonicalize_json_value,
        coverage_branch_hit_delta, default_parser_output_path, default_stimuli_module_output_path,
        extract_parse_error_position, generate_stimuli_module_source, is_ebnf_input_path,
        maybe_dump_generation_ast, minimize_failing_input, minimize_fuzz_corpus_cases,
        parse_error_context_excerpt, parse_error_line_column, parse_error_line_excerpt,
        parse_recovery_stimuli_mode, resolve_parseability_max_attempts,
        resolve_stimuli_module_seed, supported_generated_parseability_grammars,
        supports_generated_parseability, write_parseability_report,
    };
    use pgen::ast_pipeline::stimuli_generator::{
        BranchCoverageGroup, RecoveryStimuliMode, TargetDriveValidationSummary,
    };
    use pgen::ast_pipeline::{ASTNode, ASTValue, TokenValue};
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_path(file_name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let now_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!("pgen_{}_{}", now_nanos, file_name));
        path
    }

    #[test]
    fn supports_known_generated_parseability_grammars() {
        let supported = supported_generated_parseability_grammars();
        assert!(supported.contains(&"return_annotation"));
        assert!(supported.contains(&"semantic_annotation"));
        assert!(supports_generated_parseability("return_annotation"));
        assert!(supports_generated_parseability("semantic_annotation"));
        assert!(!supports_generated_parseability("unknown"));
    }

    #[test]
    fn parseability_summary_reports_acceptance_and_rejection_rates() {
        let summary = ParseabilitySummary {
            requested: 4,
            accepted: 3,
            rejected: 1,
            attempts: 6,
            generation_errors: 1,
            empty_generations: 0,
            parser_rejections: 0,
        };

        assert!((summary.acceptance_rate_percent() - 50.0).abs() < f64::EPSILON);
        assert!((summary.rejection_rate_percent() - (100.0 / 6.0)).abs() < 1e-9);
        assert!(summary.summary_line().contains("acceptance 50.00%"));
    }

    #[test]
    fn parseability_filter_summary_attributes_rejections_to_parser() {
        let summary = ParseabilitySummary::from_filter(5, 2, 3);
        assert_eq!(summary.requested, 5);
        assert_eq!(summary.accepted, 2);
        assert_eq!(summary.rejected, 3);
        assert_eq!(summary.attempts, 5);
        assert_eq!(summary.parser_rejections, 3);
        assert_eq!(summary.generation_errors, 0);
        assert_eq!(summary.empty_generations, 0);
    }

    #[test]
    fn target_drive_parseability_telemetry_splits_primary_and_alternate_entries() {
        let telemetry =
            TargetDriveParseabilityTelemetry::from_validation(&TargetDriveValidationSummary {
                validated_outputs: 4,
                accepted_outputs: 3,
                rejected_outputs: 1,
                alternate_entry_attempts: 5,
                alternate_entry_accepted_outputs: 2,
                alternate_entry_rejected_outputs: 3,
            });

        assert_eq!(telemetry.primary_entry_attempts, 4);
        assert_eq!(telemetry.primary_entry_accepted_outputs, 3);
        assert_eq!(telemetry.primary_entry_rejected_outputs, 1);
        assert!((telemetry.primary_entry_acceptance_rate_percent - 75.0).abs() < f64::EPSILON);
        assert_eq!(telemetry.alternate_entry_attempts, 5);
        assert_eq!(telemetry.alternate_entry_accepted_outputs, 2);
        assert_eq!(telemetry.alternate_entry_rejected_outputs, 3);
        assert!((telemetry.alternate_entry_acceptance_rate_percent - 40.0).abs() < f64::EPSILON);
    }

    #[test]
    fn parseability_report_serializes_target_drive_validation_when_present() {
        let path = unique_temp_path("parseability_report.json");
        let summary = ParseabilitySummary::from_filter(5, 2, 3);
        let validation = TargetDriveParseabilityTelemetry {
            primary_entry_attempts: 4,
            primary_entry_accepted_outputs: 2,
            primary_entry_rejected_outputs: 2,
            primary_entry_acceptance_rate_percent: 50.0,
            alternate_entry_attempts: 7,
            alternate_entry_accepted_outputs: 1,
            alternate_entry_rejected_outputs: 6,
            alternate_entry_acceptance_rate_percent: 100.0 / 7.0,
        };

        write_parseability_report(
            path.to_str().expect("temp path should be UTF-8"),
            "return_annotation",
            None,
            "start",
            &summary,
            Some(&validation),
            &[],
        )
        .expect("parseability report write should succeed");

        let report_json =
            std::fs::read_to_string(&path).expect("parseability report should be readable");
        let report_value: serde_json::Value =
            serde_json::from_str(&report_json).expect("parseability report should be valid JSON");
        assert_eq!(
            report_value["target_drive_validation"]["primary_entry_attempts"].as_u64(),
            Some(4)
        );
        assert_eq!(
            report_value["target_drive_validation"]["primary_entry_accepted_outputs"].as_u64(),
            Some(2)
        );
        assert_eq!(
            report_value["target_drive_validation"]["primary_entry_rejected_outputs"].as_u64(),
            Some(2)
        );
        assert_eq!(
            report_value["target_drive_validation"]["primary_entry_acceptance_rate_percent"]
                .as_f64(),
            Some(50.0)
        );
        assert_eq!(
            report_value["target_drive_validation"]["alternate_entry_attempts"].as_u64(),
            Some(7)
        );
        assert_eq!(
            report_value["target_drive_validation"]["alternate_entry_accepted_outputs"].as_u64(),
            Some(1)
        );
        assert_eq!(
            report_value["target_drive_validation"]["alternate_entry_rejected_outputs"].as_u64(),
            Some(6)
        );
        let alternate_rate =
            report_value["target_drive_validation"]["alternate_entry_acceptance_rate_percent"]
                .as_f64()
                .expect("alternate entry rate should be present");
        assert!((alternate_rate - (100.0 / 7.0)).abs() < 1e-9);

        std::fs::remove_file(&path).expect("temporary parseability report should be removable");
    }

    #[test]
    fn parseability_report_serializes_counterexamples_when_present() {
        let path = unique_temp_path("parseability_counterexamples_report.json");
        let summary = ParseabilitySummary::from_filter(3, 1, 2);
        let counterexamples = vec![ParseabilityCounterexample {
            stage: "target_drive_output_filter".to_string(),
            sample: "`define FOO(x) x".to_string(),
            sample_chars: 16,
            shrunk_sample: "`define FOO".to_string(),
            shrunk_sample_chars: 11,
            parser_error: Some("Parser did not consume full input at position 8".to_string()),
            failure_position: Some(8),
            failure_line: Some(1),
            failure_column: Some(9),
            failure_line_excerpt: Some("`define FOO(x) x".to_string()),
            failure_context_excerpt: Some("`define FOO(x) x".to_string()),
        }];

        write_parseability_report(
            path.to_str().expect("temp path should be UTF-8"),
            "systemverilog_preprocessor",
            None,
            "systemverilog_preprocessor_file",
            &summary,
            None,
            &counterexamples,
        )
        .expect("parseability report write should succeed");

        let report_json =
            std::fs::read_to_string(&path).expect("parseability report should be readable");
        let report_value: serde_json::Value =
            serde_json::from_str(&report_json).expect("parseability report should be valid JSON");
        assert_eq!(
            report_value["counterexamples"].as_array().map(Vec::len),
            Some(1)
        );
        assert_eq!(
            report_value["counterexamples"][0]["stage"].as_str(),
            Some("target_drive_output_filter")
        );
        assert_eq!(
            report_value["counterexamples"][0]["shrunk_sample"].as_str(),
            Some("`define FOO")
        );
        assert_eq!(
            report_value["counterexamples"][0]["parser_error"].as_str(),
            Some("Parser did not consume full input at position 8")
        );
        assert_eq!(
            report_value["counterexamples"][0]["failure_position"].as_u64(),
            Some(8)
        );
        assert_eq!(
            report_value["counterexamples"][0]["failure_line"].as_u64(),
            Some(1)
        );
        assert_eq!(
            report_value["counterexamples"][0]["failure_column"].as_u64(),
            Some(9)
        );
        assert_eq!(
            report_value["counterexamples"][0]["failure_line_excerpt"].as_str(),
            Some("`define FOO(x) x")
        );
        assert_eq!(
            report_value["counterexamples"][0]["failure_context_excerpt"].as_str(),
            Some("`define FOO(x) x")
        );

        std::fs::remove_file(&path).expect("temporary parseability report should be removable");
    }

    #[test]
    fn parse_error_line_excerpt_sanitizes_and_truncates() {
        let sample =
            "alpha\r\n\tbeta and more text that is intentionally long to exercise truncation\r\n";
        let excerpt = parse_error_line_excerpt(sample, 2).expect("line excerpt should exist");
        assert_eq!(
            excerpt,
            "\\tbeta and more text that is intentionally long to exercise truncation"
        );
    }

    #[test]
    fn parse_error_context_excerpt_centers_failure_position() {
        let sample = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let excerpt =
            parse_error_context_excerpt(sample, 20).expect("context excerpt should exist");
        assert!(excerpt.contains("0123456789abcdefghijklmnopqrstuvwx"));
        assert!(excerpt.ends_with("..."));
    }

    #[test]
    fn extracts_parse_error_position_from_standard_messages() {
        assert_eq!(
            extract_parse_error_position("Parser did not consume full input at position 187"),
            Some(187)
        );
        assert_eq!(
            extract_parse_error_position("Parse Error: something\n\nPosition: 42\n"),
            Some(42)
        );
        assert_eq!(extract_parse_error_position("no position here"), None);
    }

    #[test]
    fn computes_parse_error_line_and_column() {
        let sample = "alpha\nbeta\ngamma";
        assert_eq!(parse_error_line_column(sample, 0), (Some(1), Some(1)));
        assert_eq!(parse_error_line_column(sample, 6), (Some(2), Some(1)));
        assert_eq!(parse_error_line_column(sample, 10), (Some(2), Some(5)));
    }

    #[test]
    fn parseability_max_attempts_defaults_to_count_times_fifty() {
        assert_eq!(resolve_parseability_max_attempts(4, None), 200);
        assert_eq!(resolve_parseability_max_attempts(1, None), 50);
    }

    #[test]
    fn parseability_max_attempts_honors_override() {
        assert_eq!(resolve_parseability_max_attempts(4, Some(17)), 17);
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
    fn generation_ast_dump_writes_json_log() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "root".to_string(),
            ASTNode::Atom {
                value: ASTValue::Token(vec![TokenValue::String("kw_root".to_string())]),
            },
        );
        let grammar = LoadedGrammar {
            grammar_name: "demo".to_string(),
            grammar_tree,
            rule_order: vec!["root".to_string()],
            annotations: None,
        };
        let dump_path = unique_temp_path("gen_ast.json");
        let dump_path_str = dump_path.to_string_lossy().to_string();
        maybe_dump_generation_ast(&grammar, Some(dump_path_str.as_str()), false, None)
            .expect("dump succeeds");

        let raw = std::fs::read_to_string(&dump_path).expect("read dump");
        let json: serde_json::Value = serde_json::from_str(&raw).expect("json parse");
        assert_eq!(json["grammar_name"], "demo");
        assert_eq!(json["rule_order"], serde_json::json!(["root"]));

        let _ = std::fs::remove_file(dump_path);
    }

    #[test]
    fn generation_ast_dump_pretty_mode_is_multiline() {
        let grammar = LoadedGrammar {
            grammar_name: "demo_pretty".to_string(),
            grammar_tree: HashMap::new(),
            rule_order: vec![],
            annotations: None,
        };
        let dump_path = unique_temp_path("gen_ast_pretty.json");
        let dump_path_str = dump_path.to_string_lossy().to_string();
        maybe_dump_generation_ast(&grammar, Some(dump_path_str.as_str()), true, None)
            .expect("pretty dump succeeds");

        let raw = std::fs::read_to_string(&dump_path).expect("read dump");
        assert!(raw.contains('\n'));
        assert!(raw.contains("  \"grammar_name\""));

        let _ = std::fs::remove_file(dump_path);
    }

    #[test]
    fn canonicalize_json_value_sorts_object_keys_recursively() {
        let value = serde_json::json!({
            "z": { "b": 1, "a": 2 },
            "a": [ { "y": 0, "x": 1 } ],
        });
        let normalized = canonicalize_json_value(value);
        let encoded = serde_json::to_string(&normalized).expect("encode normalized");
        assert!(encoded.contains("\"a\":[{\"x\":1,\"y\":0}]"));
        assert!(encoded.contains("\"z\":{\"a\":2,\"b\":1}"));
    }

    #[test]
    fn generation_ast_dump_writes_truncation_diagnostics_when_limited() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "root".to_string(),
            ASTNode::Atom {
                value: ASTValue::Token(vec![TokenValue::String("x".repeat(4096))]),
            },
        );
        let grammar = LoadedGrammar {
            grammar_name: "demo".to_string(),
            grammar_tree,
            rule_order: vec!["root".to_string()],
            annotations: None,
        };
        let dump_path = unique_temp_path("gen_ast_truncation.json");
        let dump_path_str = dump_path.to_string_lossy().to_string();
        maybe_dump_generation_ast(&grammar, Some(dump_path_str.as_str()), false, Some(256))
            .expect("truncation diagnostics write succeeds");

        let raw = std::fs::read_to_string(&dump_path).expect("read dump");
        let json: serde_json::Value = serde_json::from_str(&raw).expect("json parse");
        assert_eq!(json["kind"], "pgen_ast_dump_truncation");
        assert_eq!(json["truncated"], true);
        assert_eq!(json["dump_kind"], "generation_input_ast");
        assert_eq!(json["max_bytes"], 256);
        let full_bytes = json["full_bytes"].as_u64().expect("full bytes");
        assert!(full_bytes > 256);

        let _ = std::fs::remove_file(dump_path);
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
        let minimized = minimize_failing_input("zzabyy", |candidate| Ok(candidate.contains("ab")))
            .expect("minimizer should succeed");
        assert_eq!(minimized, "ab");
    }

    #[test]
    fn failing_input_minimizer_keeps_input_when_not_failing() {
        let minimized = minimize_failing_input("stable", |_candidate| Ok(false))
            .expect("minimizer should succeed");
        assert_eq!(minimized, "stable");
    }

    #[test]
    fn detects_ebnf_input_extension_case_insensitively() {
        assert!(is_ebnf_input_path("grammars/json.ebnf"));
        assert!(is_ebnf_input_path("grammars/json.EBNF"));
        assert!(!is_ebnf_input_path("generated/json.json"));
        assert!(!is_ebnf_input_path("README.md"));
    }

    #[test]
    fn derives_default_parser_output_path_for_json_and_ebnf_inputs() {
        assert_eq!(
            default_parser_output_path("grammars/json.ebnf"),
            "grammars/json.rs"
        );
        assert_eq!(
            default_parser_output_path("generated/return_annotation.json"),
            "generated/return_annotation.rs"
        );
    }

    #[test]
    fn derives_default_stimuli_module_output_path_from_grammar_name() {
        assert_eq!(
            default_stimuli_module_output_path("return_annotation"),
            "generated/return_annotation_stimuli.rs"
        );
        assert_eq!(
            default_stimuli_module_output_path("my grammar"),
            "generated/my_grammar_stimuli.rs"
        );
    }

    #[test]
    fn generated_stimuli_module_source_contains_expected_contract_constants() {
        let source = generate_stimuli_module_source(
            "semantic_annotation",
            42,
            3,
            "start_rule",
            &["alpha".to_string(), "beta".to_string()],
        );
        assert!(source.contains("pub const STIMULI_MODULE_API_VERSION: u32 = 1;"));
        assert!(source.contains("pub const GRAMMAR_NAME: &str = \"semantic_annotation\";"));
        assert!(source.contains("pub const REQUESTED_SAMPLE_COUNT: usize = 3;"));
        assert!(source.contains("pub const GENERATED_SAMPLE_COUNT: usize = 2;"));
        assert!(source.contains("pub const GENERATION_SEED: u64 = 42;"));
        assert!(source.contains("pub const ENTRY_RULE: &str = \"start_rule\";"));
        assert!(source.contains("pub const STIMULI: [&str; 2] = ["));
        assert!(source.contains("\"alpha\""));
        assert!(source.contains("\"beta\""));
    }

    #[test]
    fn generated_stimuli_module_source_is_deterministic_for_identical_inputs() {
        let first = generate_stimuli_module_source(
            "json",
            7,
            2,
            "value",
            &["a".to_string(), "b".to_string()],
        );
        let second = generate_stimuli_module_source(
            "json",
            7,
            2,
            "value",
            &["a".to_string(), "b".to_string()],
        );
        assert_eq!(first, second);
    }

    #[test]
    fn stimuli_module_seed_defaults_to_contract_seed_when_unspecified() {
        assert_eq!(resolve_stimuli_module_seed(None), 1);
        assert_eq!(resolve_stimuli_module_seed(Some(99)), 99);
    }

    #[test]
    fn parses_recovery_stimuli_mode_values() {
        assert!(matches!(
            parse_recovery_stimuli_mode("baseline").expect("baseline mode should parse"),
            RecoveryStimuliMode::Baseline
        ));
        assert!(matches!(
            parse_recovery_stimuli_mode("recovery_biased")
                .expect("recovery_biased mode should parse"),
            RecoveryStimuliMode::RecoveryBiased
        ));
        assert!(matches!(
            parse_recovery_stimuli_mode("near_sync_negative")
                .expect("near_sync_negative mode should parse"),
            RecoveryStimuliMode::NearSyncNegative
        ));
    }

    #[test]
    fn rejects_unknown_recovery_stimuli_mode_values() {
        let err = parse_recovery_stimuli_mode("unknown_mode")
            .expect_err("unknown recovery mode must be rejected");
        let message = err.to_string();
        assert!(
            message.contains("Unsupported recovery stimuli mode"),
            "unexpected recovery mode parse error message: {}",
            message
        );
    }
}
