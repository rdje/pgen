//! Rust AST Pipeline CLI
//!
//! Command-line interface for the Rust AST transformation pipeline.

use anyhow::{Context, Result};
use clap::Parser;
use pgen::ast_pipeline::stimuli_generator::{
    PlannableProbeVerdict, RecoveryStimuliMode, StimuliConfig, StimuliConstraintProfile,
    StimuliCoverageGapReport, StimuliCoverageMetrics, StimuliGenerator, StimuliMutationMode,
    StimuliNegativeProfile, TargetDriveFilterContext, TargetDriveValidationSummary,
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
const STIMULI_CORPUS_BUNDLE_VERSION: u32 = 1;
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

    /// PARSE-SOTA.9.1 (adoption A2): run the static grammar well-formedness LINT over the
    /// loaded grammar (left-recursion info, non-terminating errors, ordered-choice
    /// shadowing warnings) and print a report, then exit. Opt-in (lints are not emitted on
    /// every generate); the non-terminating REJECT remains always-on at load.
    #[arg(long)]
    lint_grammar: bool,

    /// SV-CORPUS-GRAD.7 (parser-agnostic corpus rule-coverage instrument): serialize the
    /// loaded grammar's FULL rule inventory with, per rule, the declared `@profiles` set
    /// (absent = universal) and the DERIVED per-profile satisfiability (the same transitive
    /// `derive_rule_profiles` computation the profile-orphan lint gates on) as deterministic
    /// JSON to this path, then exit. The external-corpus coverage reporter diffs its
    /// per-profile fired-rule unions against this inventory: an uncovered rule is either a
    /// corpus GAP (satisfiable under the profile, never fired) or N/A-for-profile (not
    /// satisfiable under it — its home profiles listed). Read-only; no codegen change.
    #[arg(long)]
    dump_rule_profiles: Option<String>,

    /// RGX-0078.5.h.1 (STEP-0 fusibility census): opt-in read-only FUSIBILITY-CENSUS report for
    /// the DERIVED-SCANNER rung. Classifies every rule against the increment-1 capability gate
    /// (regular + effect-free + text-folding + policy-encodable + layout-contiguous) over the
    /// gen-AST, prints per-tier counts, the maximal fusible roots (future `scan_*` sites), and a
    /// disqualification histogram, then exits. `PGEN_FUSIBILITY_DUMP_ALL=1` prints every per-rule
    /// verdict. Read-only; no codegen change.
    #[arg(long)]
    report_fusibility_census: bool,

    /// RGX-0078.5.h.1: optional machine-readable JSON output for the fusibility census.
    #[arg(long, value_name = "FILE", requires = "report_fusibility_census")]
    fusibility_census_json: Option<String>,

    /// RGX-0078.5.h.1: comma-separated rule-entry-count JSON files (written by
    /// `parseability_probe --parse <g> <input> --dump-rule-entry-counts-json FILE`) to join with
    /// the census — prints the measured FUSIBILITY-ENTRY-SHARE (the share of bench rule entries
    /// a derived scanner would eliminate, the `.5.h.2+` gating ceiling).
    #[arg(long, value_name = "FILES", requires = "report_fusibility_census")]
    fusibility_entry_counts: Option<String>,

    /// RGX-0078.5.h.1b: comma-separated rule-OUTCOME-count JSON files (written by
    /// `parseability_probe --parse <g> <input> --dump-rule-outcome-counts-json FILE`) to join
    /// with the census — prints the measured OUTCOME-SHARE (raw/committed/discarded
    /// decomposition; the discarded-on-encodable kill surface = the increment-(ii)
    /// merged-choice ceiling) and fills the per-choice-site sole-attribution table.
    #[arg(long, value_name = "FILES", requires = "report_fusibility_census")]
    fusibility_outcome_counts: Option<String>,

    /// STIMULI-SIGNOFF.2.3 (adoption D): opt-in k-path coverage REPORT at depth N. Generates
    /// `--count` samples from `--entry-rule` (or the first rule) and prints covered/universe
    /// k-paths (Havrikov-Zeller). Read-only; does not change generation. Use small N (2-3).
    #[arg(long, value_name = "K")]
    report_k_path_coverage: Option<usize>,

    /// STIMULI-SIGNOFF.4.2: opt-in DIRECTED generation loop (FdLoop, arXiv 2508.01472): per
    /// round, learn a per-choice-point branch distribution from the round's best sample and
    /// steer the next round's generation with it. Value names the GOAL; supported: `k_path`
    /// (goal G1 — per-sample fitness = NEW k-paths covered), `corpus_mimicry` (goal G3,
    /// STIMULI-SIGNOFF.4.3 — learn the distribution from a REAL corpus via the gen-AST
    /// interpreter and generate distributionally-similar inputs; per-sample fitness = L1
    /// proximity to the corpus distribution; needs `--mimicry-corpus-file` and/or
    /// `--mimicry-corpus-lines`), and `duality_break` (goal G2, STIMULI-SIGNOFF.4.4 — hunt
    /// generator-emitted-but-parser-REJECTED samples against the real generated parser;
    /// fitness rewards novel rejection signatures; each unique break is shrunk to a minimal
    /// signature-preserving reproducer; needs a registered generated parser). Prints a
    /// `DIRECTED-GENERATION:` headline including a same-seed, same-budget diverse-pass
    /// baseline comparison. Opt-in only — no other generation surface changes.
    #[arg(long, value_name = "GOAL")]
    directed_generation_goal: Option<String>,

    /// STIMULI-SIGNOFF.4.2: rounds for the directed loop (FdLoop generations).
    #[arg(long, default_value_t = 10, requires = "directed_generation_goal")]
    directed_rounds: usize,

    /// STIMULI-SIGNOFF.4.2: samples generated (and scored) per directed round.
    #[arg(long, default_value_t = 5, requires = "directed_generation_goal")]
    directed_samples_per_round: usize,

    /// STIMULI-SIGNOFF.4.2: the k for the k_path goal's coverage metric (use small k, 2-3).
    #[arg(long, default_value_t = 2, requires = "directed_generation_goal")]
    directed_k: usize,

    /// STIMULI-SIGNOFF.4.2: optional path for the directed loop's JSON report.
    #[arg(long, value_name = "FILE", requires = "directed_generation_goal")]
    directed_report_json: Option<String>,

    /// STIMULI-SIGNOFF.4.3 (goal `corpus_mimicry`): a corpus file whose WHOLE content is ONE
    /// corpus input (repeatable; the SV-corpus-file shape). Inputs are attributed through the
    /// gen-AST interpreter; inputs the grammar rejects contribute nothing and are reported.
    #[arg(long, value_name = "FILE", requires = "directed_generation_goal")]
    mimicry_corpus_file: Vec<String>,

    /// STIMULI-SIGNOFF.4.3 (goal `corpus_mimicry`): a corpus list file where each non-empty
    /// LINE is one input (the one-pattern-per-line shape, e.g. extracted regex corpora).
    #[arg(long, value_name = "FILE", requires = "directed_generation_goal")]
    mimicry_corpus_lines: Option<String>,

    /// GRAMMAR-WELLFORMED.G.4: opt-in CERTIFICATE-COVERAGE report (the linter⟷generator duality
    /// capstone). For every rule, is it covered by a verified unreachability PROOF or a verified
    /// reachability WITNESS (a clean diverse `--count` sample that parses through the real parser and
    /// exercises it)? Prints proof/witness/UNKNOWN; `UNKNOWN`=0 with no failures = the objective
    /// "trustworthy on this grammar" number. Read-only. Requires `--features generated_parsers` (needs
    /// the grammar's parser to verify witnesses); parser-agnostic via the registry.
    #[arg(long)]
    report_certificate_coverage: bool,

    /// GRAMMAR-WELLFORMED.H.12.8.1.1: opt-in MULTI-CONFIG certificate-coverage union. Repeatable;
    /// each value names an additional `<entry>[:<profile>]` config (e.g. `systemverilog_file:sv_2023`,
    /// `sv_multi_entry_root:sv_2017`) whose VERIFIED covered rule sets (proof ∪ witness) union into the
    /// canonical `--report-certificate-coverage` accounting. A rule is CERTIFIED iff positively covered
    /// (proof OR witness) in SOME config, UNKNOWN iff covered in NONE — the union is over
    /// positively-covered sets, never over "not-UNKNOWN-in-some-config". With this flag present an
    /// extra `CERTIFICATE-COVERAGE-UNION:` line is printed; the canonical `CERTIFICATE-COVERAGE:` line
    /// is unchanged. Empty (the default) ⇒ byte-identical single-config behavior for every grammar.
    /// Requires `--report-certificate-coverage` (+ `--features generated_parsers`).
    #[arg(long = "cert-union-config", requires = "report_certificate_coverage")]
    cert_union_config: Vec<String>,

    /// Generate high-performance Rust parser instead of JSON output
    #[arg(long)]
    generate_parser: bool,
    /// (Phase 2 M1, off by default) When generating a parser, also emit a typed
    /// entry-point skeleton — `parse_full_<entry>_typed` returning
    /// `ParseResult<serde_json::Value>` alongside the existing
    /// `parse_full_<entry>` returning `ParseResult<ParseNode>`. The skeleton body
    /// is a passthrough wrapper (parse + `serde_json::to_value`); it does NOT
    /// inline anything per-rule.
    ///
    /// This flag does NOT control "annotation support" — `@predicate`,
    /// `@emit_fact`, `@semantic_value`, and `-> {...}` return annotations all
    /// fire whether this flag is set or not (annotation support is always-on).
    /// The flag was previously named `--inline-annotations`; the rename in
    /// slice 4 reflects what the flag actually does. It controls ONLY the
    /// pipeline-internal M1 skeleton emit. (PARSER-NEUTRALITY.1: the former
    /// `--enable-parser-hooks` companion flag and the per-grammar hook
    /// mechanism are REMOVED by director ruling.)
    #[arg(long)]
    emit_typed_entry_skeleton: bool,
    /// Override the path where the pipeline emits its return-annotation inventory
    /// artifact alongside parser generation. The default is
    /// `<output-dir>/<grammar>_return_annotations.json` next to the parser output;
    /// the artifact captures the exact `(rule, branch_index, annotation_type, raw_text,
    /// normalized_text)` tuples the pipeline passes to `Return_annotationParser` for
    /// parsing. The AST-shape contract gate reads this artifact directly to assert that
    /// the manifest's tracked declared-annotation list still matches the grammar.
    #[arg(long)]
    emit_return_annotations_json: Option<PathBuf>,
    /// Disable the auto-emission of the return-annotation inventory artifact during
    /// `--generate-parser`. Default is to auto-emit; pass this flag for environments
    /// where the artifact is not desired (for example focused codegen experiments where
    /// the tracked artifact should not be touched).
    #[arg(long)]
    no_emit_return_annotations: bool,
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

    /// Stimuli near-valid negative generation profile: baseline, near_valid_local
    #[arg(
        long,
        default_value = "baseline",
        value_parser = ["baseline", "near_valid_local"]
    )]
    stimuli_negative_profile: String,

    /// Stimuli constrained-random steering profile: baseline, rare_branch_biased, deep_nesting_biased
    #[arg(
        long,
        default_value = "baseline",
        value_parser = ["baseline", "rare_branch_biased", "deep_nesting_biased"]
    )]
    stimuli_constraint_profile: String,

    /// Stimuli mutation mode: baseline, grammar_aware_local
    #[arg(
        long,
        default_value = "baseline",
        value_parser = ["baseline", "grammar_aware_local"]
    )]
    stimuli_mutation_mode: String,

    /// Extra stagnant iterations required before broad pending frontiers can outrank marginal dependency probes during target-driven replay
    #[arg(long, default_value_t = 8)]
    target_pending_frontier_extra_stagnation: usize,

    /// Per-primary target-driven generation wall-clock budget in milliseconds (`0` disables the primary target timeout)
    #[arg(long, default_value_t = 0)]
    target_generation_timeout_ms: u64,

    /// Per-helper wall-clock budget in milliseconds for alternate target-driven probe entries (`0` disables the helper timeout)
    #[arg(long, default_value_t = 1000)]
    target_helper_generation_timeout_ms: u64,

    /// DEPRECATED / redundant: lexical faithfulness (minimal token separation so generated stimuli re-lex to the intended tokens) is now ON by default. This flag is still accepted for backward compatibility and forces faithfulness on; use `--no-word-boundary-spacing` to opt out (for example negative-test generation).
    #[arg(long)]
    enforce_word_boundary_spacing: bool,

    /// Opt out of lexical faithfulness for generated stimuli (LEXICAL-ANNOTATIONS.3d): do NOT insert the minimal token separators. Use for negative-test generation that intentionally produces malformed lexical surface. Without this flag, faithful spacing is applied by default.
    #[arg(long)]
    no_word_boundary_spacing: bool,

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

    /// Write deterministic machine-readable stimuli corpus bundle JSON
    #[arg(long)]
    stimuli_corpus_json: Option<String>,

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
// `Clone` (GRAMMAR-WELLFORMED.H.12.8.1.1): the multi-config certificate-coverage union
// (`--cert-union-config`) re-filters the SAME loaded grammar bundle once per `(entry, profile)`
// config. `apply_grammar_profile_filter` consumes the bundle by value, so the caller clones the
// unfiltered bundle per union config. All fields are `Clone` (`String` / `HashMap<String, ASTNode>`
// / `Vec<String>` / `Option<Annotations>`), so the derive is structural and cheap relative to the
// per-config certification cost.
#[derive(Clone)]
struct LoadedGrammar {
    grammar_name: String,
    grammar_tree: HashMap<String, ASTNode>,
    rule_order: Vec<String>,
    annotations: Option<Annotations>,
}

#[derive(Debug, Serialize)]
struct GenerationAstDumpMetadata<'a> {
    format: &'static str,
    source_format: &'static str,
    transformed_at: &'static str,
    transformer: &'static str,
    pipeline_stage: &'static str,
    annotations: Option<&'a Annotations>,
    stats: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct GenerationAstDump<'a> {
    grammar_name: &'a str,
    rule_order: &'a [String],
    grammar_tree: &'a HashMap<String, ASTNode>,
    annotations: Option<&'a Annotations>,
    metadata: GenerationAstDumpMetadata<'a>,
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
    primary_entry_rule: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_entry_rule: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    entry_mode: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct StimuliCorpusBundleSample {
    ordinal: usize,
    sample: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parseable: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    new_rule_hits: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    new_branch_hits: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    coverage_tokens: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct StimuliCorpusGenerationConfig {
    requested_seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    effective_seed: Option<u64>,
    deterministic_replay_possible: bool,
    max_depth: usize,
    max_repeat: usize,
    max_rule_visits: usize,
    target_pending_frontier_extra_stagnation: usize,
    #[serde(default)]
    target_generation_timeout_ms: u64,
    target_helper_generation_timeout_ms: u64,
    recovery_stimuli_mode: String,
    stimuli_negative_profile: String,
    stimuli_constraint_profile: String,
    stimuli_mutation_mode: String,
    enforce_word_boundary_spacing: bool,
    validate_parseability: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    parseability_max_attempts: Option<usize>,
    coverage_guided_fuzz_rounds: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    coverage_guided_fuzz_seed_start: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StimuliCorpusBundle {
    pgen_stimuli_corpus_bundle_version: u32,
    grammar_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    grammar_profile: Option<String>,
    generation_surface: String,
    corpus_origin_mode: String,
    entry_rule: String,
    requested_sample_count: usize,
    generated_sample_count: usize,
    generation_config: StimuliCorpusGenerationConfig,
    samples: Vec<StimuliCorpusBundleSample>,
    coverage: StimuliCoverageMetrics,
    #[serde(skip_serializing_if = "Option::is_none")]
    parseability_summary: Option<ParseabilitySummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_drive_validation: Option<TargetDriveParseabilityTelemetry>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    parseability_counterexamples: Vec<ParseabilityCounterexample>,
    #[serde(skip_serializing_if = "Option::is_none")]
    coverage_guided_fuzz_replay: Option<CoverageGuidedFuzzReplayReport>,
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
    #[serde(default)]
    target_timeout_errors: usize,
    helper_timeout_errors: usize,
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
            target_timeout_errors: summary.target_timeout_errors,
            helper_timeout_errors: summary.helper_timeout_errors,
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
    minimized_bundle_samples: Vec<StimuliCorpusBundleSample>,
    merged_coverage: StimuliCoverageMetrics,
    replay_report: CoverageGuidedFuzzReplayReport,
}

#[derive(Debug, Clone)]
struct FuzzCorpusCandidate {
    sample: String,
    seed: u64,
    parseable: Option<bool>,
    new_rule_hits: Vec<String>,
    new_branch_hits: Vec<String>,
    coverage_tokens: HashSet<String>,
}

fn main() -> Result<()> {
    // `SV-CORPUS-GRAD.8c.3` — run the ENTIRE pipeline body on a dedicated
    // 256 MiB-stack thread so every driver (parse, AST dump, cert coverage,
    // stimuli replay) gives the generated parsers' 4096-frame recursion
    // ceiling enough real stack to fire (clean diagnostic) before the OS
    // guard page can SIGABRT the process. Measured: the ceiling needs ≈8 MB
    // release / ≈70 MB debug for SV, vs the 8 MB default main stack.
    // Wrapping the whole body keeps thread-local trace/dump configuration
    // and the parses on the same thread.
    pgen::dedicated_parse_stack::run_cli_main_on_dedicated_parse_stack(
        "pgen-ast-pipeline",
        pipeline_main,
    )
}

fn pipeline_main() -> Result<()> {
    // PARSE-HARNESS.10 feature-surface tripwire: report the compile-time feature surface and exit.
    // Handled BEFORE clap so the probe needs no input file and answers identically in EVERY feature
    // configuration — a binary that cannot answer it is, by definition, a pre-tripwire vintage. The
    // marker line is consumed by `pgen::parse_harness` (kept in lockstep with its parser there).
    if std::env::args().skip(1).any(|a| a == "--report-feature-surface") {
        println!(
            "AST-PIPELINE-FEATURE-SURFACE: ebnf_dual_run={} generated_parsers={}",
            cfg!(feature = "ebnf_dual_run"),
            cfg!(feature = "generated_parsers"),
        );
        return Ok(());
    }
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
            || args.stimuli_corpus_json.is_some()
            || args.gap_report_json.is_some()
            || args.gap_report_text.is_some()
            || args.gap_report_threshold != 1
            || args.recovery_stimuli_mode != "baseline"
            || args.stimuli_negative_profile != "baseline"
            || args.stimuli_constraint_profile != "baseline"
            || args.stimuli_mutation_mode != "baseline"
            || args.enforce_word_boundary_spacing
            || args.no_word_boundary_spacing;
        if has_shared_stimuli_flags {
            return Err(anyhow::anyhow!(
                "--validate-parseability/--parseability-report-json/--parseability-max-attempts/--coverage-*/--stimuli-corpus-json/--gap-report-*/--recovery-stimuli-mode/--stimuli-negative-profile/--stimuli-constraint-profile/--stimuli-mutation-mode/--enforce-word-boundary-spacing/--no-word-boundary-spacing require --generate-stimuli or --generate-stimuli-module"
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

    // PARSE-SOTA.9.1 (adoption A2): opt-in grammar well-formedness LINT.
    if args.lint_grammar {
        // UNDEFINED-REF-DIAGNOSTICS.2: keep the UNFILTERED bundle too — the undefined-reference
        // check must see the view CODEGEN compiles (always the FULL grammar; profile selection is
        // a runtime guard). The profile filter deliberately strips @profiles-gated rule
        // DEFINITIONS from the filtered view (e.g. regex's pcre2 generation default strips the
        // `relaxed` rules), which would make their intact references look dangling.
        let unfiltered_grammar = load_grammar_bundle(
            &args.input_path,
            &mut pipeline,
            args.emit_raw_ast_json.as_deref(),
        )?;
        let grammar = apply_grammar_profile_filter(
            unfiltered_grammar.clone(),
            args.grammar_profile.as_deref(),
        )?;
        return run_grammar_lint(&grammar, &unfiltered_grammar);
    }

    // SV-CORPUS-GRAD.7: machine-readable per-profile rule-inventory dump (the corpus
    // rule-coverage instrument's denominator). Runs on the UNFILTERED bundle — the full view
    // codegen compiles (profile selection is a runtime guard).
    if let Some(out_path) = args.dump_rule_profiles.clone() {
        let unfiltered_grammar = load_grammar_bundle(
            &args.input_path,
            &mut pipeline,
            args.emit_raw_ast_json.as_deref(),
        )?;
        return run_dump_rule_profiles(&unfiltered_grammar, &out_path);
    }

    // RGX-0078.5.h.1: opt-in read-only fusibility census (the derived-scanner STEP-0 gate).
    // Runs on the UNFILTERED bundle — codegen always compiles the FULL grammar (profile
    // selection is a runtime guard), and the census mirrors what codegen would fuse.
    if args.report_fusibility_census {
        let grammar = load_grammar_bundle(
            &args.input_path,
            &mut pipeline,
            args.emit_raw_ast_json.as_deref(),
        )?;
        return run_fusibility_census_report(
            &grammar,
            args.fusibility_census_json.as_deref(),
            args.fusibility_entry_counts.as_deref(),
            args.fusibility_outcome_counts.as_deref(),
        );
    }

    // STIMULI-SIGNOFF.2.3 (adoption D): opt-in k-path coverage report.
    if let Some(k) = args.report_k_path_coverage {
        let grammar = apply_grammar_profile_filter(
            load_grammar_bundle(
                &args.input_path,
                &mut pipeline,
                args.emit_raw_ast_json.as_deref(),
            )?,
            args.grammar_profile.as_deref(),
        )?;
        return run_k_path_coverage_report(
            &grammar,
            k,
            args.entry_rule.as_deref(),
            args.count,
            args.seed.unwrap_or(0),
        );
    }

    // STIMULI-SIGNOFF.4.2: opt-in FdLoop directed generation loop (goal-directed learned
    // distributions). Read-only report mode like the k-path report above.
    if let Some(goal) = args.directed_generation_goal.as_deref() {
        let grammar = apply_grammar_profile_filter(
            load_grammar_bundle(
                &args.input_path,
                &mut pipeline,
                args.emit_raw_ast_json.as_deref(),
            )?,
            args.grammar_profile.as_deref(),
        )?;
        return run_directed_generation(
            &grammar,
            DirectedGenerationRun {
                goal,
                entry: args.entry_rule.as_deref(),
                rounds: args.directed_rounds,
                samples_per_round: args.directed_samples_per_round,
                k: args.directed_k,
                seed: args.seed.unwrap_or(0),
                report_json: args.directed_report_json.as_deref(),
                mimicry_corpus_files: &args.mimicry_corpus_file,
                mimicry_corpus_lines: args.mimicry_corpus_lines.as_deref(),
                grammar_profile: args.grammar_profile.as_deref(),
            },
        );
    }

    if args.report_certificate_coverage {
        let unfiltered_grammar = load_grammar_bundle(
            &args.input_path,
            &mut pipeline,
            args.emit_raw_ast_json.as_deref(),
        )?;
        // GRAMMAR-WELLFORMED.H.12.8.1.1: keep the UNFILTERED bundle so the opt-in multi-config union
        // (`--cert-union-config`) can re-filter it per requested `(entry, profile)` config.
        // `apply_grammar_profile_filter` consumes its argument, so the canonical profile filter runs on
        // a clone of the bundle. The clone yields the SAME filtered grammar (output-identical) — only a
        // per-run allocation — so the canonical `CERTIFICATE-COVERAGE:` output stays byte-identical.
        let grammar =
            apply_grammar_profile_filter(unfiltered_grammar.clone(), args.grammar_profile.as_deref())?;
        #[cfg(feature = "generated_parsers")]
        return run_certificate_coverage_report(
            &grammar,
            &unfiltered_grammar,
            args.entry_rule.as_deref(),
            args.count,
            args.seed.unwrap_or(0),
            args.grammar_profile.as_deref(),
            args.max_depth,
            &args.cert_union_config,
        );
        #[cfg(not(feature = "generated_parsers"))]
        {
            let _ = (&grammar, &unfiltered_grammar);
            anyhow::bail!(
                "--report-certificate-coverage requires building with --features generated_parsers \
                 (the witness side parses generated samples through the grammar's real parser)"
            );
        }
    }

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

        // PARSER CODEGEN always emits the FULL grammar (all profiles) with RUNTIME profile
        // guards; profile SELECTION happens at parse time via `set_grammar_profile`. Only an
        // EXPLICIT `--grammar-profile` filters here (rare/unused). Critically, the regex
        // pcre2 GENERATION default in `apply_grammar_profile_filter` must NOT apply on this
        // path — it would strip the `@profiles:["relaxed"]` rule DEFINITIONS from the parser,
        // leaving their references unresolved (fallback stubs that always fail). The default
        // belongs to the stimuli-generation paths only (REGEX-PCRE2-FIDELITY.3.1).
        let loaded_grammar = load_grammar_bundle(
            &args.input_path,
            &mut pipeline,
            args.emit_raw_ast_json.as_deref(),
        )?;
        let grammar = match args.grammar_profile.as_deref() {
            Some(profile) => apply_grammar_profile_filter(loaded_grammar, Some(profile))?,
            None => loaded_grammar,
        };
        maybe_dump_generation_ast(
            &grammar,
            args.dump_gen_ast.as_deref(),
            args.dump_gen_ast_pretty,
            dump_gen_ast_max_bytes,
        )?;

        // QUANT-PLUS-ITER.2 (director rule): `--entry-rule` outranks a declared
        // `@entry: true`. Applied here because this is the one path that never
        // received the CLI entry at all.
        let mut grammar = grammar;
        apply_cli_entry_rule_override(&mut grammar, args.entry_rule.as_deref())?;

        // Generate parser through the direct AST integration path so typed annotation
        // validation and strict CI policies apply to normal CLI generation as well.
        // (PARSER-NEUTRALITY.1: generation takes no per-parser inputs beyond the
        // grammar itself — the former binary-boundary hook registry is removed.)
        let parser_code = pgen::ast_pipeline::ast_generator_direct::generate_parser_ast_based(
            &grammar.grammar_name,
            &grammar.grammar_tree,
            &grammar.rule_order,
            grammar.annotations.as_ref(),
            output_rust.as_str(),
            args.emit_typed_entry_skeleton,
        )?;
        std::fs::write(&output_rust, parser_code)?;

        println!("SOTA parser generated: {}", output_rust);

        // Auto-emit the return-annotation inventory artifact alongside parser
        // generation. Single source of truth for the AST-shape contract gate:
        // the same Annotations struct the codegen consumed produces this
        // artifact, eliminating any risk of divergence between what's tracked
        // and what the parser actually sees.
        if !args.no_emit_return_annotations {
            let inventory_path = args
                .emit_return_annotations_json
                .clone()
                .unwrap_or_else(|| {
                    pgen::ast_pipeline::default_return_annotation_inventory_path(
                        &grammar.grammar_name,
                        &output_rust,
                    )
                });
            let inventory =
                pgen::ast_pipeline::EmittedReturnAnnotationInventory::from_annotations(
                    &grammar.grammar_name,
                    grammar.annotations.as_ref(),
                );
            let written = inventory.write_to_file(&inventory_path)?;
            println!(
                "Return-annotation inventory: {} ({} entries)",
                written.display(),
                inventory.annotation_count
            );
        }

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
        let negative_profile = parse_stimuli_negative_profile(&args.stimuli_negative_profile)?;
        let constraint_profile =
            parse_stimuli_constraint_profile(&args.stimuli_constraint_profile)?;
        let mutation_mode = parse_stimuli_mutation_mode(&args.stimuli_mutation_mode)?;
        let stimuli_config = StimuliConfig {
            seed: Some(effective_seed),
            max_depth: args.max_depth,
            max_repeat: args.max_repeat,
            max_rule_visits: args.max_depth.max(2),
            target_pending_frontier_extra_stagnation: args.target_pending_frontier_extra_stagnation,
            target_generation_timeout_ms: args.target_generation_timeout_ms,
            target_helper_generation_timeout_ms: args.target_helper_generation_timeout_ms,
            recovery_mode,
            mutation_mode,
            constraint_profile,
            negative_profile,
            enforce_word_boundary_spacing: effective_word_boundary_spacing(
                args.enforce_word_boundary_spacing,
                args.no_word_boundary_spacing,
            ),
            trace_verbosity,
            // GRAMMAR-WELLFORMED.H.4.2: opt-in to the cert-coverage witness pass only — default
            // `--generate-stimuli` is unaffected.
            reach_uncovered_recursive_branches: false,
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
        let resolved_entry_rule = resolve_stimuli_entry_rule(
            &grammar.grammar_tree,
            &grammar.rule_order,
            args.entry_rule.as_deref(),
        )?;
        if args.validate_parseability {
            ensure_generated_parseability_entry_rule_supported(
                &grammar.rule_order,
                resolved_entry_rule.as_str(),
            )?;
        }
        let mut parseability_summary: Option<ParseabilitySummary> = None;
        let mut parseability_counterexamples = Vec::new();
        let samples = if args.validate_parseability {
            let outcome = generate_parseable_stimuli(
                &grammar.grammar_name,
                args.grammar_profile.as_deref(),
                &mut generator,
                args.count,
                Some(resolved_entry_rule.as_str()),
                args.parseability_max_attempts,
            )?;
            parseability_summary = Some(outcome.summary.clone());
            parseability_counterexamples = outcome.counterexamples.clone();
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
        if let Some(corpus_output_path) = args.stimuli_corpus_json.as_deref() {
            let corpus_bundle = build_stimuli_corpus_bundle(
                &grammar.grammar_name,
                args.grammar_profile.as_deref(),
                "generate_stimuli_module",
                if args.validate_parseability {
                    "parseability_filtered_generation"
                } else {
                    "direct_generation"
                },
                resolved_entry_rule.as_str(),
                args.count,
                args.seed,
                Some(effective_seed),
                &stimuli_config,
                args.validate_parseability,
                args.parseability_max_attempts,
                0,
                None,
                direct_bundle_samples(&samples),
                &merged_coverage,
                parseability_summary.as_ref(),
                None,
                &parseability_counterexamples,
                None,
            );
            write_stimuli_corpus_bundle(corpus_output_path, &corpus_bundle)?;
            println!("Wrote stimuli corpus bundle to {}", corpus_output_path);
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
                    target_pending_frontier_extra_stagnation: args
                        .target_pending_frontier_extra_stagnation,
                    target_generation_timeout_ms: args.target_generation_timeout_ms,
                    target_helper_generation_timeout_ms: args.target_helper_generation_timeout_ms,
                    recovery_mode,
                    mutation_mode,
                    constraint_profile,
                    negative_profile,
                    enforce_word_boundary_spacing: effective_word_boundary_spacing(
                        args.enforce_word_boundary_spacing,
                        args.no_word_boundary_spacing,
                    ),
                    trace_verbosity,
                    reach_uncovered_recursive_branches: false,
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
        let negative_profile = parse_stimuli_negative_profile(&args.stimuli_negative_profile)?;
        let constraint_profile =
            parse_stimuli_constraint_profile(&args.stimuli_constraint_profile)?;
        let mutation_mode = parse_stimuli_mutation_mode(&args.stimuli_mutation_mode)?;
        let stimuli_config = StimuliConfig {
            seed: args.seed,
            max_depth: args.max_depth,
            max_repeat: args.max_repeat,
            max_rule_visits: args.max_depth.max(2),
            target_pending_frontier_extra_stagnation: args.target_pending_frontier_extra_stagnation,
            target_generation_timeout_ms: args.target_generation_timeout_ms,
            target_helper_generation_timeout_ms: args.target_helper_generation_timeout_ms,
            recovery_mode,
            mutation_mode,
            constraint_profile,
            negative_profile,
            enforce_word_boundary_spacing: effective_word_boundary_spacing(
                args.enforce_word_boundary_spacing,
                args.no_word_boundary_spacing,
            ),
            trace_verbosity,
            // GRAMMAR-WELLFORMED.H.4.2: opt-in to the cert-coverage witness pass only — default
            // `--generate-stimuli-module` is unaffected.
            reach_uncovered_recursive_branches: false,
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
        let resolved_entry_rule = resolve_stimuli_entry_rule(
            &grammar.grammar_tree,
            &grammar.rule_order,
            args.entry_rule.as_deref(),
        )?;
        if args.validate_parseability {
            ensure_generated_parseability_entry_rule_supported(
                &grammar.rule_order,
                resolved_entry_rule.as_str(),
            )?;
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
        let mut corpus_samples: Option<Vec<StimuliCorpusBundleSample>> = None;

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
                Some(resolved_entry_rule.as_str()),
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
                                None,
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
            corpus_samples = Some(fuzz_outcome.minimized_bundle_samples.clone());
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
                    Some(resolved_entry_rule.as_str()),
                    args.parseability_max_attempts,
                )?;
                parseability_summary = Some(outcome.summary);
                parseability_counterexamples = outcome.counterexamples;
                outcome.samples
            } else {
                generator.generate_many(args.count, Some(resolved_entry_rule.as_str()))?
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
            let (mut generated_samples, target_summary) = if args.validate_parseability {
                let (samples, summary, validation) = generator.generate_until_targets_with_filter(
                    Some(resolved_entry_rule.as_str()),
                    &target_report.targets,
                    args.target_max_attempts,
                    |sample, context| {
                        let parseable = is_sample_parseable_by_generated_parser(
                            &grammar.grammar_name,
                            args.grammar_profile.as_deref(),
                            sample,
                        )?;
                        if !parseable
                            && context.is_primary_entry
                            && parseability_counterexamples.len() < MAX_PARSEABILITY_COUNTEREXAMPLES
                        {
                            parseability_counterexamples.push(build_parseability_counterexample(
                                "target_drive_output_filter",
                                &grammar.grammar_name,
                                args.grammar_profile.as_deref(),
                                sample,
                                Some(context),
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
                    Some(resolved_entry_rule.as_str()),
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
            // SV-EXH-PROOF.7.4.3: APPENDED minimal-witness pass. After the diverse +
            // target-drive passes, generate one dedicated witness per still-unresolved
            // target rooted at the target's OWN rule (fresh full depth budget — the
            // .7.4.3a depth-budget fix). Purely additive: only adds coverage into the
            // same generator, so the residual the gap report below measures can only
            // shrink (monotone). The diverse pass (separate invocation) is untouched.
            let (witness_samples, witness_summary) =
                generator.generate_target_witnesses(&target_report.targets)?;
            println!("{}", witness_summary.summary_line());
            // SV-EXH-PROOF.7.4.4.1 (TOOL-BUILD, WHY+WHERE): surface a bounded sample of
            // the "other"-class witness failures so the residual tail's cause is visible
            // on tangible proof (not guessed). Printed at default verbosity; bounded in
            // the generator so it can never flood disk.
            if !witness_summary.other_failure_samples.is_empty() {
                println!(
                    "Witness 'other'-failure samples ({} of {} shown | target_id | rule | type | node_path | branch | reason):",
                    witness_summary.other_failure_samples.len(),
                    witness_summary.other_failures
                );
                for sample in &witness_summary.other_failure_samples {
                    println!("- {sample}");
                }
            }
            // SV-EXH-PROOF.7.4.6.4 (TOOL-BUILD, WHY+WHERE): surface the target_timeout tail —
            // which rules/types dead-end + whether construction failed before search.
            if !witness_summary.timeout_failure_samples.is_empty() {
                println!(
                    "Witness 'target_timeout'-failure samples ({} of {} shown | target_id | rule | type | node_path | branch | construct_failed | reason):",
                    witness_summary.timeout_failure_samples.len(),
                    witness_summary.target_timeout_failures
                );
                for sample in &witness_summary.timeout_failure_samples {
                    println!("- {sample}");
                }
            }
            // SV-EXH-PROOF.7.4.6.4 (TOOL-BUILD): surface targets still UNRESOLVED after the
            // witness pass (incl. the non-covering class: construct Ok but witness uncovered).
            if !witness_summary.unresolved_after_samples.is_empty() {
                println!(
                    "Witness still-UNRESOLVED samples ({} shown | target_id | rule | type | node_path | branch | attempted):",
                    witness_summary.unresolved_after_samples.len()
                );
                for sample in &witness_summary.unresolved_after_samples {
                    println!("- {sample}");
                }
            }
            // NOTE: certificate-coverage (GRAMMAR-WELLFORMED.G.4) is NOT computed here. The witness
            // pass above produces reach-plan-FORCED samples (which often don't parse on replay), so it
            // is the wrong witness source. The certificate-coverage gate is its own clean, dedicated,
            // parser-agnostic mode — `--report-certificate-coverage` / `run_certificate_coverage_report`
            // — which uses CLEAN diverse samples. Keeping generation and certification separate.
            generated_samples.extend(witness_samples);
            merged_coverage = generator.coverage_metrics().clone();
            generated_samples
        } else if args.validate_parseability {
            let outcome = generate_parseable_stimuli(
                &grammar.grammar_name,
                args.grammar_profile.as_deref(),
                &mut generator,
                args.count,
                Some(resolved_entry_rule.as_str()),
                args.parseability_max_attempts,
            )?;
            parseability_summary = Some(outcome.summary);
            parseability_counterexamples = outcome.counterexamples;
            merged_coverage = generator.coverage_metrics().clone();
            outcome.samples
        } else {
            let generated_samples =
                generator.generate_many(args.count, Some(resolved_entry_rule.as_str()))?;
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

        let corpus_origin_mode = if args.coverage_guided_fuzz_rounds > 0 {
            "coverage_guided_fuzz_minimized"
        } else if args.target_report_input.is_some() {
            "target_driven_generation"
        } else if args.gap_priority_report_input.is_some() {
            "gap_priority_generation"
        } else if args.validate_parseability {
            "parseability_filtered_generation"
        } else {
            "direct_generation"
        };
        let corpus_samples = corpus_samples.unwrap_or_else(|| direct_bundle_samples(&samples));

        if let Some(report_path) = args.parseability_report_json.as_deref() {
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

        if let Some(corpus_output_path) = args.stimuli_corpus_json.as_deref() {
            let effective_seed = if args.coverage_guided_fuzz_rounds > 0 {
                args.coverage_guided_fuzz_seed_start
                    .or(args.seed)
                    .or(Some(1))
            } else {
                args.seed
            };
            let corpus_bundle = build_stimuli_corpus_bundle(
                &grammar.grammar_name,
                args.grammar_profile.as_deref(),
                "generate_stimuli",
                corpus_origin_mode,
                resolved_entry_rule.as_str(),
                args.count,
                args.seed,
                effective_seed,
                &stimuli_config,
                args.validate_parseability,
                args.parseability_max_attempts,
                args.coverage_guided_fuzz_rounds,
                args.coverage_guided_fuzz_seed_start,
                corpus_samples,
                &merged_coverage,
                parseability_summary.as_ref(),
                parseability_target_drive_validation.as_ref(),
                &parseability_counterexamples,
                replay_report.as_ref(),
            );
            write_stimuli_corpus_bundle(corpus_output_path, &corpus_bundle)?;
            println!("Wrote stimuli corpus bundle to {}", corpus_output_path);
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
        metadata: GenerationAstDumpMetadata {
            format: "transformed_ast",
            source_format: "generation_input_ast",
            transformed_at: "deterministic_generation_dump",
            transformer: "Rust AST Pipeline CLI",
            pipeline_stage: "generation_input_ast",
            annotations: grammar.annotations.as_ref(),
            stats: HashMap::new(),
        },
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

/// PARSE-SOTA.8.1 (adoption A1): grammar well-formedness guard. Reject a grammar that has
/// NON-TERMINATING rules (no finite terminal derivation — genuinely ill-formed, which
/// neither PGEN's LR-elimination nor its runtime cycle-breaking can rescue), surfaced via
/// the always-on DIAG-SEVERITY pgen_error! channel. Left recursion is deliberately NOT
/// rejected (PGEN handles it). References to rules defined elsewhere (the `include(...)`
/// system) are NOT mis-flagged — see grammar_wellformedness::detect_nonterminating_rules.
fn check_grammar_wellformed(grammar: &LoadedGrammar) -> Result<()> {
    let nonterminating = pgen::ast_pipeline::grammar_wellformedness::detect_nonterminating_rules(
        &grammar.grammar_tree,
        &grammar.rule_order,
    );
    if !nonterminating.is_empty() {
        for issue in &nonterminating {
            pgen::pgen_error!("{}", issue.message());
        }
        return Err(anyhow::anyhow!(
            "grammar '{}' is ill-formed: {} non-terminating rule(s) (see errors above)",
            grammar.grammar_name,
            nonterminating.len()
        ));
    }
    Ok(())
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
    let json_value = normalize_legacy_generation_ast_dump(json_value);
    let grammar = if let Some(raw_ast) = json_value.get("raw_ast") {
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

        LoadedGrammar {
            grammar_name,
            grammar_tree,
            rule_order,
            annotations,
        }
    } else if json_value.get("grammar_tree").is_some() && json_value.get("rule_order").is_some() {
        let transformed: TransformedASTJson = serde_json::from_value(json_value)?;
        LoadedGrammar {
            grammar_name: transformed.grammar_name,
            grammar_tree: transformed.grammar_tree,
            rule_order: transformed.rule_order,
            annotations: transformed.metadata.annotations,
        }
    } else {
        return Err(anyhow::anyhow!(
            "Unknown JSON format - expected raw_ast or grammar_tree/rule_order"
        ));
    };

    // QUANT-PLUS-ITER.2: honour a declared `@entry: true` BEFORE well-formedness, so
    // the linter's reachability roots see the same entry codegen will. Placed at this
    // chokepoint deliberately — see `apply_declared_entry_rule`.
    let mut grammar = grammar;
    apply_declared_entry_rule(&mut grammar)?;

    // PARSE-SOTA.8.1 (A1): the SINGLE grammar-load chokepoint every build path goes
    // through (--generate-parser/-stimuli, `make focus_*`, --lint-grammar). Reject a
    // grammar with NON-TERMINATING rules here so it can never be silently bypassed (the
    // earlier placement in the profile filter was skipped for non-profiled grammars).
    check_grammar_wellformed(&grammar)?;
    Ok(grammar)
}

/// `QUANT-PLUS-ITER.2`: resolve the grammar's declared entry rule (`@entry: true`,
/// attached to the rule itself) by NORMALIZING `rule_order` so that rule sits at
/// index 0.
///
/// ⭐ WHY A NORMALIZATION AND NOT A THREADED PARAMETER. `rule_order[0]` is consulted
/// as "the entry" by roughly ten places — `main.rs` (×7), `grammar_wellformedness`'s
/// reachability roots, the parse-harness interpreter, and codegen's own
/// `entry_rule = self.entry_rule.or_else(|| rule_order.first())`. Threading an
/// `Option<String>` through all of them is exactly the per-call-site fragility
/// `LANG-CAPABILITY-AUDIT.7` rejected for `include()`: it is one forgotten consumer
/// away from a silent inconsistency, and a future consumer inherits nothing. Doing it
/// ONCE here, at the single grammar-load chokepoint every build path funnels through,
/// makes every consumer correct **structurally** — none of them needs to know the
/// directive exists.
///
/// ⭐ The invariant `rule_order[0] == the entry` therefore SURVIVES INTACT. This does
/// not weaken it; it lets the author choose which rule occupies that slot instead of
/// it being an accident of file layout — which is the whole point (in EBNF a grammar
/// is a SET of productions, so rule order should carry no meaning).
///
/// A grammar that declares nothing is untouched: no reorder, byte-identical output.
fn apply_declared_entry_rule(grammar: &mut LoadedGrammar) -> anyhow::Result<()> {
    let Some(annotations) = grammar.annotations.as_ref() else {
        return Ok(()); // no annotations at all ⇒ nothing declared.
    };
    let Some(declared) = pgen::ast_pipeline::semantic_runtime::compile_entry_rule(annotations)
        .map_err(|err| anyhow::anyhow!("grammar '{}': {}", grammar.grammar_name, err))?
    else {
        return Ok(());
    };

    let Some(index) = grammar.rule_order.iter().position(|r| *r == declared) else {
        // Not reachable through the normal path — `compile_entry_rule` returns an
        // ATTACHMENT key, so the rule necessarily exists. Kept as a named error
        // rather than an `unwrap`, so a future caller that synthesizes annotations
        // gets a diagnosis instead of a panic.
        return Err(anyhow::anyhow!(
            "grammar '{}': '@entry: true' is attached to '{}', which is not in the grammar's rule set",
            grammar.grammar_name,
            declared
        ));
    };

    reorder_entry_first(&mut grammar.rule_order, index);
    Ok(())
}

/// Move `rule_order[index]` to the front, preserving the relative order of the rest.
/// A no-op when it is already first (the common case — every tracked grammar today).
fn reorder_entry_first(rule_order: &mut Vec<String>, index: usize) {
    if index == 0 || index >= rule_order.len() {
        return;
    }
    let rule = rule_order.remove(index);
    rule_order.insert(0, rule);
}

/// `QUANT-PLUS-ITER.2` — director rule (2026-07-26): *"`--entry-rule <name>` shall
/// take precedence over `@entry: true` in the EBNF file when both are mentioned on
/// the CLI."*
///
/// Every other consumer already honours that order, because it threads
/// `args.entry_rule.or_else(|| rule_order.first())`. The `--generate-parser` path did
/// NOT: `generate_parser_ast_based` receives only `&grammar.rule_order` and has never
/// been passed the CLI entry at all, so `--entry-rule` was silently ineffective there
/// **before this leaf** (measured — a pre-existing gap, not a regression introduced by
/// `@entry`). Applying the override as the same reorder keeps ONE mechanism for "which
/// rule is the entry" instead of introducing a second, divergent one.
///
/// An unknown `--entry-rule` name is a hard, named error rather than a silent
/// fallback: silently parsing from a different rule than the operator asked for is the
/// exact failure mode this whole tree exists to eliminate.
fn apply_cli_entry_rule_override(
    grammar: &mut LoadedGrammar,
    cli_entry: Option<&str>,
) -> anyhow::Result<()> {
    let Some(requested) = cli_entry else {
        return Ok(());
    };
    let Some(index) = grammar.rule_order.iter().position(|r| r == requested) else {
        return Err(anyhow::anyhow!(
            "grammar '{}': --entry-rule '{}' names a rule the grammar does not define",
            grammar.grammar_name,
            requested
        ));
    };
    reorder_entry_first(&mut grammar.rule_order, index);
    Ok(())
}

fn normalize_legacy_generation_ast_dump(mut json_value: serde_json::Value) -> serde_json::Value {
    let Some(object) = json_value.as_object_mut() else {
        return json_value;
    };
    if !object.contains_key("grammar_tree")
        || !object.contains_key("rule_order")
        || object.contains_key("metadata")
    {
        return json_value;
    }

    let annotations = object
        .get("annotations")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    object.insert(
        "metadata".to_string(),
        serde_json::json!({
            "format": "transformed_ast",
            "source_format": "generation_input_ast_legacy",
            "transformed_at": "legacy_generation_dump",
            "transformer": "Rust AST Pipeline CLI",
            "pipeline_stage": "generation_input_ast",
            "annotations": annotations,
            "stats": {}
        }),
    );
    json_value
}

// `PROFILE-ALIAS.2`: the former `normalize_grammar_profile_name` global spelling
// table (SV + VHDL aliases applied to ANY grammar) lived here — retired: request
// spellings are now declared per-grammar via `@profile_alias` and resolved
// through `compile_profile_aliases` in `apply_grammar_profile_filter`; declared
// `@profiles` list values are compared case-insensitively (the same
// `eq_ignore_ascii_case` posture as the generated `rule_profile_is_enabled`
// guard) with no alias rewriting.

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
                            .map(|value| value.trim().to_ascii_lowercase())
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
                            .map(|value| value.trim().to_ascii_lowercase())
                            .collect()
                    });
                    continue;
                }
            },
        };

        allowed_profiles = parse_semantic_string_list(payload).map(|values| {
            values
                .into_iter()
                .map(|value| value.trim().to_ascii_lowercase())
                .collect()
        });
    }

    match allowed_profiles {
        Some(allowed) if !allowed.is_empty() => allowed.iter().any(|value| value == active_profile),
        _ => true,
    }
}

/// LEXICAL-ANNOTATIONS.3d (ii-CLI) — the effective lexical-faithfulness setting for the
/// production CLI. Faithfulness is ON by default (matching `StimuliConfig::default()`); the
/// `--no-word-boundary-spacing` opt-out turns it off (e.g. negative-test generation). The legacy
/// `--enforce-word-boundary-spacing` flag is still honored as an explicit force-on (so an explicit
/// enable wins over the opt-out if both are passed). Single source of truth for the three
/// `StimuliConfig`-construction sites so the in-memory and generated-module paths stay consistent.
/// Takes the two flag values (both `Copy`) rather than `&Args` so it composes at call sites where
/// other `Args` fields have already been partially moved.
fn effective_word_boundary_spacing(enforce_flag: bool, no_spacing_flag: bool) -> bool {
    enforce_flag || !no_spacing_flag
}

fn filter_annotations_by_profile(
    annotations: Annotations,
    retained_rules: &HashSet<String>,
) -> Annotations {
    let mut branch_return_annotations = annotations.branch_return_annotations;
    branch_return_annotations.retain(|rule_name, _| retained_rules.contains(rule_name));

    let mut branch_semantic_annotations = annotations.branch_semantic_annotations;
    branch_semantic_annotations.retain(|rule_name, _| retained_rules.contains(rule_name));

    let mut branch_mid_sequence_semantic_annotations =
        annotations.branch_mid_sequence_semantic_annotations;
    branch_mid_sequence_semantic_annotations
        .retain(|rule_name, _| retained_rules.contains(rule_name));

    let mut semantic_annotations = annotations.semantic_annotations;
    semantic_annotations.retain(|rule_name, _| retained_rules.contains(rule_name));

    // LEXICAL-ANNOTATIONS.3c — profile-filter the per-rule follow-restrictions
    // consistently with the other per-rule maps: a rule pruned by the active profile
    // takes its lexical follow-restriction with it.
    let mut lexical_follow_restrictions = annotations.lexical_follow_restrictions;
    lexical_follow_restrictions.retain(|rule_name, _| retained_rules.contains(rule_name));

    let mut pre_lr_elim_branch_return_annotations = annotations
        .pre_lr_elim_branch_return_annotations;
    if let Some(snapshot) = pre_lr_elim_branch_return_annotations.as_mut() {
        snapshot.retain(|rule_name, _| retained_rules.contains(rule_name));
    }

    Annotations {
        branch_return_annotations,
        branch_semantic_annotations,
        branch_mid_sequence_semantic_annotations,
        semantic_annotations,
        lexical_follow_restrictions,
        pre_lr_elim_branch_return_annotations,
    }
}

fn apply_grammar_profile_filter(
    grammar: LoadedGrammar,
    grammar_profile: Option<&str>,
) -> Result<LoadedGrammar> {
    // REGEX-PCRE2-FIDELITY.3.1 / DEFAULT-PROFILE.2: an unspecified profile resolves to the
    // grammar's DECLARED `@default_profile` (e.g. regex → strict `pcre2`) on the GENERATION side
    // too — the generation twin of the parse-side default the generated constructor now carries.
    // Without this, the `@profiles:["relaxed"]`-gated constructs (`simple_escape_letter_relaxed`,
    // `unicode_escape`, …) would NOT be filtered out of default-mode generation, so the generator
    // could emit `\u` etc. that the (grammar-strict) default-mode parser rejects. The retired
    // `== "regex" → "pcre2"` name literal was the doctrine violation this replaces: the default
    // now comes from the grammar itself, for any grammar. A malformed/conflicting directive is a
    // hard error here, exactly as it is at codegen.
    let declared_default_profile = match grammar_profile {
        Some(_) => None,
        None => grammar
            .annotations
            .as_ref()
            .map(|annotations| {
                pgen::ast_pipeline::compile_default_profile(annotations).map_err(|err| {
                    anyhow::anyhow!(
                        "Grammar '{}': invalid @default_profile directive: {}",
                        grammar.grammar_name,
                        err
                    )
                })
            })
            .transpose()?
            .flatten(),
    };
    let grammar_profile = match (grammar_profile, declared_default_profile.as_deref()) {
        (Some(profile), _) => Some(profile),
        (None, declared) => declared,
    };
    let Some(profile) = grammar_profile else {
        return Ok(grammar);
    };
    // PROFILE-ALIAS.2: resolve the requested spelling through the grammar's OWN
    // declared `@profile_alias` map (case-insensitive; unmatched spellings pass
    // through), then compare lowercased — the generation twin of the alias
    // resolution the generated parser's `set_grammar_profile` now carries. The
    // retired global spelling table (SV + VHDL aliases applied to ANY grammar)
    // was the same doctrine-violation class as the retired default-profile name
    // gate above. A malformed/conflicting directive is a hard error here,
    // exactly as it is at codegen.
    let declared_aliases = grammar
        .annotations
        .as_ref()
        .map(|annotations| {
            pgen::ast_pipeline::compile_profile_aliases(annotations).map_err(|err| {
                anyhow::anyhow!(
                    "Grammar '{}': invalid @profile_alias directive: {}",
                    grammar.grammar_name,
                    err
                )
            })
        })
        .transpose()?
        .flatten();
    let requested_profile = profile.trim();
    let active_profile = declared_aliases
        .as_ref()
        .and_then(|aliases| aliases.get(&requested_profile.to_ascii_lowercase()))
        .map(String::as_str)
        .unwrap_or(requested_profile)
        .to_ascii_lowercase();
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

/// PARSE-SOTA.9.1 (adoption A2): run the static grammar well-formedness lint and print a
/// report. Left-recursion is informational (PGEN handles it); ordered-choice shadowing is
/// a warning (unreachable alternatives); non-terminating rules are errors (also rejected
/// at load by .8.1, so a loaded grammar shows 0). Returns Err iff non-terminating rules
/// remain (scriptable exit code).
/// STIMULI-SIGNOFF.2.3 (adoption D): k-path coverage report — generate `samples` derivations
/// from `entry` (or the first rule) and print covered/universe k-paths (Havrikov-Zeller).
/// Read-only instrumentation; generation itself is unchanged.
fn run_k_path_coverage_report(
    grammar: &LoadedGrammar,
    k: usize,
    entry: Option<&str>,
    samples: usize,
    seed: u64,
) -> Result<()> {
    let entry_rule = entry
        .map(|s| s.to_string())
        .or_else(|| grammar.rule_order.first().cloned())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "grammar '{}' has no rules to report k-path coverage for",
                grammar.grammar_name
            )
        })?;
    let config = StimuliConfig {
        seed: Some(seed),
        ..Default::default()
    };
    let mut generator = StimuliGenerator::new(
        grammar.grammar_name.clone(),
        &grammar.grammar_tree,
        &grammar.rule_order,
        grammar.annotations.as_ref(),
        config,
    );
    let samples = samples.max(1);
    let (covered, universe) = generator.k_path_coverage_report(&entry_rule, samples, k);
    let pct = if universe > 0 {
        100.0 * covered as f64 / universe as f64
    } else {
        0.0
    };
    println!(
        "k-path coverage: grammar='{}' entry='{}' k={} samples={} -> covered {}/{} k-paths ({:.1}% of the universe)",
        grammar.grammar_name, entry_rule, k, samples, covered, universe, pct
    );
    Ok(())
}

/// STIMULI-SIGNOFF.4.2: the parameters of one `--directed-generation-goal` run.
struct DirectedGenerationRun<'a> {
    goal: &'a str,
    entry: Option<&'a str>,
    rounds: usize,
    samples_per_round: usize,
    k: usize,
    seed: u64,
    report_json: Option<&'a str>,
    /// STIMULI-SIGNOFF.4.3 (goal `corpus_mimicry`): whole-file corpus inputs.
    mimicry_corpus_files: &'a [String],
    /// STIMULI-SIGNOFF.4.3 (goal `corpus_mimicry`): one-input-per-line corpus list file.
    mimicry_corpus_lines: Option<&'a str>,
    /// STIMULI-SIGNOFF.4.3: the requested dialect profile, passed through to the interpreter's
    /// corpus attribution so `@profiles` gating matches the (already profile-filtered) generator.
    grammar_profile: Option<&'a str>,
}

/// STIMULI-SIGNOFF.4.2: run the FdLoop directed generation loop for the requested goal and
/// print the headline, including the same-seed same-budget DIVERSE baseline (a fresh generator
/// generating rounds×samples_per_round samples with no learning) — the honest comparator for
/// "did the learned steering buy coverage the plain diverse pass would not have reached?".
fn run_directed_generation(grammar: &LoadedGrammar, run: DirectedGenerationRun) -> Result<()> {
    if run.goal == "corpus_mimicry" {
        return run_directed_corpus_mimicry(grammar, &run);
    }
    if run.goal == "duality_break" {
        return run_directed_duality_hunt(grammar, &run);
    }
    if run.goal != "k_path" {
        anyhow::bail!(
            "unsupported --directed-generation-goal '{}' (supported goals: k_path, corpus_mimicry, duality_break)",
            run.goal
        );
    }
    let entry_rule = run
        .entry
        .map(|s| s.to_string())
        .or_else(|| grammar.rule_order.first().cloned())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "grammar '{}' has no rules to run directed generation for",
                grammar.grammar_name
            )
        })?;
    let rounds = run.rounds.max(1);
    let samples_per_round = run.samples_per_round.max(1);
    let make_generator = || {
        StimuliGenerator::new(
            grammar.grammar_name.clone(),
            &grammar.grammar_tree,
            &grammar.rule_order,
            grammar.annotations.as_ref(),
            StimuliConfig {
                seed: Some(run.seed),
                ..Default::default()
            },
        )
    };

    let mut directed = make_generator();
    let outcome = directed.directed_k_path_generation(&entry_rule, rounds, samples_per_round, run.k);

    let mut diverse = make_generator();
    let budget = rounds.saturating_mul(samples_per_round);
    let (diverse_covered, universe) = diverse.k_path_coverage_report(&entry_rule, budget, run.k);

    let pct = |covered: usize| {
        if universe > 0 {
            100.0 * covered as f64 / universe as f64
        } else {
            0.0
        }
    };
    let delta = outcome.covered as i64 - diverse_covered as i64;
    println!(
        "DIRECTED-GENERATION: goal=k_path grammar='{}' entry='{}' k={} rounds={} samples_per_round={} seed={} -> directed covered {}/{} ({:.1}%) vs diverse baseline {}/{} ({:.1}%) [delta {:+}] learned_groups={}",
        grammar.grammar_name,
        entry_rule,
        run.k,
        rounds,
        samples_per_round,
        run.seed,
        outcome.covered,
        universe,
        pct(outcome.covered),
        diverse_covered,
        universe,
        pct(diverse_covered),
        delta,
        outcome.learned_groups
    );
    if let Some(path) = run.report_json {
        let report = serde_json::json!({
            "goal": "k_path",
            "grammar_name": grammar.grammar_name,
            "entry_rule": entry_rule,
            "k": run.k,
            "rounds": rounds,
            "samples_per_round": samples_per_round,
            "seed": run.seed,
            "universe": universe,
            "directed_covered": outcome.covered,
            "diverse_baseline_covered": diverse_covered,
            "delta": delta,
            "per_round_best_new_k_paths": outcome.per_round_best_new_k_paths,
            "learned_groups": outcome.learned_groups,
        });
        std::fs::write(path, serde_json::to_string_pretty(&report)?)
            .with_context(|| format!("failed to write directed-generation report to {path}"))?;
        println!("Wrote directed-generation report to {path}");
    }
    Ok(())
}

/// STIMULI-SIGNOFF.4.3: run the FdLoop directed loop for goal G3 (corpus mimicry). Stage 1 —
/// attribute every corpus input's derivation through the gen-AST interpreter
/// (`interpret_parse_gen_ast_with_selections`, the parser-agnostic derivation counter over the
/// SAME profile-filtered grammar tree generation uses, so group keys and branch indices match by
/// construction) and fold the ACCEPTED inputs' selection logs into the corpus distribution.
/// Stages 4–6 — `directed_corpus_mimicry_generation`. The headline compares the directed
/// population's L1 proximity to the corpus against a SAME-SEED SAME-BUDGET diverse baseline
/// scored identically (`mimicry_population_score`; selection recording is read-only, so the
/// baseline's generated output is byte-identical to a plain diverse pass).
fn run_directed_corpus_mimicry(grammar: &LoadedGrammar, run: &DirectedGenerationRun) -> Result<()> {
    let entry_rule = run
        .entry
        .map(|s| s.to_string())
        .or_else(|| grammar.rule_order.first().cloned())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "grammar '{}' has no rules to run directed generation for",
                grammar.grammar_name
            )
        })?;
    let rounds = run.rounds.max(1);
    let samples_per_round = run.samples_per_round.max(1);

    // ── Corpus ingestion (deterministic: files in the given order, then the lines file). ──
    let mut corpus_inputs: Vec<(String, String)> = Vec::new();
    for path in run.mimicry_corpus_files {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read --mimicry-corpus-file {path}"))?;
        corpus_inputs.push((path.clone(), content));
    }
    if let Some(path) = run.mimicry_corpus_lines {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read --mimicry-corpus-lines {path}"))?;
        for (idx, line) in content.lines().enumerate() {
            if !line.is_empty() {
                corpus_inputs.push((format!("{path}:{}", idx + 1), line.to_string()));
            }
        }
    }
    if corpus_inputs.is_empty() {
        anyhow::bail!(
            "goal corpus_mimicry needs a corpus: pass --mimicry-corpus-file FILE (whole file = \
             one input, repeatable) and/or --mimicry-corpus-lines FILE (one input per line)"
        );
    }

    // ── Stage 1: interpreter-attributed derivation counting over the corpus. ──
    let mut accepted_logs = Vec::new();
    let mut rejected = 0usize;
    for (label, input) in &corpus_inputs {
        let (outcome, selections) =
            pgen::parse_harness_interpreter::interpret_parse_gen_ast_with_selections(
                &grammar.grammar_name,
                run.grammar_profile,
                &grammar.grammar_tree,
                &grammar.rule_order,
                grammar.annotations.as_ref(),
                Some(entry_rule.as_str()),
                input,
            )
            .map_err(|e| anyhow::anyhow!("corpus attribution failed for {label}: {e}"))?;
        if outcome.accepted {
            accepted_logs.push(selections);
        } else {
            rejected += 1;
        }
    }
    if accepted_logs.is_empty() {
        anyhow::bail!(
            "goal corpus_mimicry: none of the {} corpus inputs is accepted by grammar '{}' \
             (entry '{}'), so there is no distribution to learn",
            corpus_inputs.len(),
            grammar.grammar_name,
            entry_rule
        );
    }
    let corpus = StimuliGenerator::learn_branch_distributions(&accepted_logs);

    let make_generator = || {
        StimuliGenerator::new(
            grammar.grammar_name.clone(),
            &grammar.grammar_tree,
            &grammar.rule_order,
            grammar.annotations.as_ref(),
            StimuliConfig {
                seed: Some(run.seed),
                ..Default::default()
            },
        )
    };

    let mut directed = make_generator();
    let outcome =
        directed.directed_corpus_mimicry_generation(&entry_rule, rounds, samples_per_round, &corpus);

    let budget = rounds.saturating_mul(samples_per_round);
    let mut diverse = make_generator();
    let (baseline_proximity, baseline_shared, baseline_samples) =
        diverse.mimicry_population_score(&entry_rule, budget, &corpus);

    let delta = outcome.population_proximity - baseline_proximity;
    println!(
        "DIRECTED-GENERATION: goal=corpus_mimicry grammar='{}' entry='{}' rounds={} samples_per_round={} seed={} corpus_inputs={} accepted={} rejected={} corpus_groups={} -> directed population proximity {:.4} (shared {}/{}, {} samples) vs diverse baseline {:.4} (shared {}/{}, {} samples) [delta {:+.4}] learned_groups={}",
        grammar.grammar_name,
        entry_rule,
        rounds,
        samples_per_round,
        run.seed,
        corpus_inputs.len(),
        accepted_logs.len(),
        rejected,
        outcome.corpus_groups,
        outcome.population_proximity,
        outcome.shared_groups,
        outcome.corpus_groups,
        outcome.generated_samples,
        baseline_proximity,
        baseline_shared,
        outcome.corpus_groups,
        baseline_samples,
        delta,
        outcome.learned_groups
    );
    if let Some(path) = run.report_json {
        let report = serde_json::json!({
            "goal": "corpus_mimicry",
            "grammar_name": grammar.grammar_name,
            "entry_rule": entry_rule,
            "rounds": rounds,
            "samples_per_round": samples_per_round,
            "seed": run.seed,
            "corpus_inputs": corpus_inputs.len(),
            "corpus_accepted": accepted_logs.len(),
            "corpus_rejected": rejected,
            "corpus_groups": outcome.corpus_groups,
            "directed_population_proximity": outcome.population_proximity,
            "directed_shared_groups": outcome.shared_groups,
            "directed_generated_samples": outcome.generated_samples,
            "diverse_baseline_proximity": baseline_proximity,
            "diverse_baseline_shared_groups": baseline_shared,
            "diverse_baseline_generated_samples": baseline_samples,
            "delta": delta,
            "per_round_best_proximity": outcome.per_round_best_proximity,
            "learned_groups": outcome.learned_groups,
        });
        std::fs::write(path, serde_json::to_string_pretty(&report)?)
            .with_context(|| format!("failed to write directed-generation report to {path}"))?;
        println!("Wrote directed-generation report to {path}");
    }
    Ok(())
}

/// STIMULI-SIGNOFF.4.4: run the FdLoop directed loop for goal G2 (duality-break hunting). The
/// rejection oracle is the REAL registered generated parser
/// (`parser_registry::parse_sample_detail_with_profile`), injected into the parser-agnostic lib
/// loop — a hit is a sample the generator emitted and the shipped parser rejects, i.e. a
/// generator⟷parser duality break. Each unique (digit-normalized) rejection signature is shrunk
/// to a minimal SIGNATURE-PRESERVING reproducer via the existing `minimize_failing_input`
/// machinery. The headline includes the same-seed same-budget diverse baseline (plain diverse
/// generation scored by the same oracle).
#[cfg(not(feature = "generated_parsers"))]
fn run_directed_duality_hunt(_grammar: &LoadedGrammar, _run: &DirectedGenerationRun) -> Result<()> {
    anyhow::bail!(
        "goal duality_break needs --features generated_parsers (the real generated parser is the rejection oracle)"
    )
}

#[cfg(feature = "generated_parsers")]
fn run_directed_duality_hunt(grammar: &LoadedGrammar, run: &DirectedGenerationRun) -> Result<()> {
    let entry_rule = run
        .entry
        .map(|s| s.to_string())
        .or_else(|| grammar.rule_order.first().cloned())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "grammar '{}' has no rules to run directed generation for",
                grammar.grammar_name
            )
        })?;
    let rounds = run.rounds.max(1);
    let samples_per_round = run.samples_per_round.max(1);
    let grammar_name = grammar.grammar_name.clone();
    let profile = run.grammar_profile.map(|s| s.to_string());

    // Support pre-check (a cheap empty-string probe): `None` = no registered generated parser.
    if parser_registry::parse_sample_detail_with_profile(&grammar_name, "", profile.as_deref())
        .is_none()
    {
        anyhow::bail!(
            "goal duality_break: grammar '{}' has no registered generated parser to hunt against. Supported grammars: {}",
            grammar_name,
            supported_generated_parseability_grammars_csv()
        );
    }
    // Captures by reference → `Copy`, so the hunt and the baseline share the same oracle.
    let oracle = |sample: &str| -> std::result::Result<(), String> {
        parser_registry::parse_sample_detail_with_profile(&grammar_name, sample, profile.as_deref())
            .unwrap_or_else(|| Err("unsupported grammar (pre-checked; unreachable)".to_string()))
    };

    let make_generator = || {
        StimuliGenerator::new(
            grammar.grammar_name.clone(),
            &grammar.grammar_tree,
            &grammar.rule_order,
            grammar.annotations.as_ref(),
            StimuliConfig {
                seed: Some(run.seed),
                ..Default::default()
            },
        )
    };

    let mut directed = make_generator();
    let outcome =
        directed.directed_duality_break_hunt(&entry_rule, rounds, samples_per_round, oracle);

    let budget = rounds.saturating_mul(samples_per_round);
    let mut diverse = make_generator();
    let (baseline_rejected, baseline_generated) =
        diverse.duality_rejection_baseline(&entry_rule, budget, oracle);

    // Shrink each unique break to a minimal reproducer that still fails with the SAME signature.
    let mut break_reports: Vec<serde_json::Value> = Vec::new();
    let mut break_lines: Vec<String> = Vec::new();
    for duality_break in &outcome.breaks {
        let shrunk = minimize_failing_input(&duality_break.first_sample, |candidate| {
            Ok(
                match parser_registry::parse_sample_detail_with_profile(
                    &grammar_name,
                    candidate,
                    profile.as_deref(),
                ) {
                    Some(Err(error)) => {
                        StimuliGenerator::normalize_rejection_signature(&error)
                            == duality_break.signature
                    }
                    _ => false,
                },
            )
        })?;
        break_lines.push(format!(
            "signature={:?} occurrences={} shrunk_reproducer={:?}",
            duality_break.signature, duality_break.occurrences, shrunk
        ));
        break_reports.push(serde_json::json!({
            "signature": duality_break.signature,
            "occurrences": duality_break.occurrences,
            "first_sample": duality_break.first_sample,
            "shrunk_reproducer": shrunk,
        }));
    }

    println!(
        "DIRECTED-GENERATION: goal=duality_break grammar='{}' entry='{}' rounds={} samples_per_round={} seed={} -> directed rejected {}/{} unique_breaks={} vs diverse baseline rejected {}/{} learned_groups={}",
        grammar.grammar_name,
        entry_rule,
        rounds,
        samples_per_round,
        run.seed,
        outcome.rejected_samples,
        outcome.generated_samples,
        outcome.breaks.len(),
        baseline_rejected,
        baseline_generated,
        outcome.learned_groups
    );
    for line in &break_lines {
        println!("  DUALITY-BREAK: {line}");
    }
    if let Some(path) = run.report_json {
        let report = serde_json::json!({
            "goal": "duality_break",
            "grammar_name": grammar.grammar_name,
            "entry_rule": entry_rule,
            "rounds": rounds,
            "samples_per_round": samples_per_round,
            "seed": run.seed,
            "directed_generated_samples": outcome.generated_samples,
            "directed_rejected_samples": outcome.rejected_samples,
            "diverse_baseline_generated_samples": baseline_generated,
            "diverse_baseline_rejected_samples": baseline_rejected,
            "unique_breaks": outcome.breaks.len(),
            "breaks": break_reports,
            "per_round_best_fitness": outcome.per_round_best_fitness,
            "learned_groups": outcome.learned_groups,
        });
        std::fs::write(path, serde_json::to_string_pretty(&report)?)
            .with_context(|| format!("failed to write directed-generation report to {path}"))?;
        println!("Wrote directed-generation report to {path}");
    }
    Ok(())
}

/// CERT-GEN-BUDGET.2: the default per-sample DETERMINISTIC step-budget (B1, expressed in the
/// `target_generation_timeout_ms` unit × `generation_steps_per_ms`) armed on the cert-coverage
/// PASS-1 diverse pass. The diverse config previously left `target_generation_timeout_ms = 0`, so
/// the B1 step-budget was never armed and the pass was TIME-UNBOUNDED — super-linear / effectively
/// non-terminating on a deeply-recursive grammar (e.g. `rtl_const_expr` at certain depths), which
/// presents as a cert hang. This default is far above any well-behaved grammar's per-sample cost
/// (verified byte-identical for the 6 fully-certified grammars + SystemVerilog at their canonical
/// depths) so it is INERT for them; only a pathological super-linear grammar reaches it, and then
/// fails fast with a deterministic `TargetTimeout` instead of hanging. Env-tunable via
/// `PGEN_CERT_DIVERSE_GENERATION_TIMEOUT_MS` (set `0` for the legacy unbounded pass deliberately).
///
/// CALIBRATION (CERT-GEN-BUDGET.2, seed 0, current binary): the heaviest LEGITIMATE per-sample
/// cost is `rtl_const_expr` at its canonical `--max-depth 32` — cumulative 8 487 959 steps over
/// 40 samples (~212k avg), and NO sample is cut at a 2 000 000-step (2000 ms) budget (the cert is
/// byte-identical), so the per-sample max there is < 2 000 000. SystemVerilog count-40 depth-24 is
/// ~93k total (~2.3k/sample). This default (4 000 000 steps) is ~2× that proven per-sample ceiling
/// and ~19× the average — ample margin so it never bites a well-behaved grammar — while it still
/// deterministically cuts the deep-recursion runaway (`rtl_const_expr`/`conditional_expr` at
/// `--max-depth` 40/48 spin unboundedly without it; the canonical entry at depth 40 crosses
/// 2 000 000 steps in ~7.3s, so 4 000 000 caps it well under ~15s instead of >200s / non-terminating).
#[cfg(feature = "generated_parsers")]
const CERT_DIVERSE_GENERATION_TIMEOUT_MS_DEFAULT: u64 = 4_000;

/// GRAMMAR-WELLFORMED.H.12.8.1.1: the result of ONE `(entry, profile)` certificate-coverage pass —
/// the VERIFIED covered rule sets (the union inputs) plus the per-pass diagnostic tallies the
/// canonical report prints. `gather_cert_covered_sets` returns this so the single-config path and the
/// opt-in multi-config union (`--cert-union-config`) share one body: the union loops configs and
/// unions `proof_covered`/`witness_covered`; the canonical run additionally prints the tallies.
#[cfg(feature = "generated_parsers")]
struct CertCoveredSets {
    proof_covered: std::collections::HashSet<String>,
    witness_covered: std::collections::HashSet<String>,
    proof_fails: Vec<String>,
    sample_parse_failures: usize,
    failures: Vec<(String, String)>,
    reach_pass_parse_failures: usize,
    plannable_no_path: Vec<String>,
    plannable_pass_parse_failures: usize,
    plannable_generation_failures: usize,
    plannable_left_unattempted: usize,
    plannable_attempted: usize,
    plannable_witnessed: usize,
    plannable_parsed_not_witnessed: usize,
    target_own_attempted: usize,
    target_own_witnessed: usize,
}

/// GRAMMAR-WELLFORMED.G.4 / H.12.8.1.1: gather the VERIFIED proof + witness covered sets for ONE
/// `(entry, profile)` config (the linter⟷generator duality capstone), PARSER-AGNOSTIC. For every rule
/// the grammar must carry either a verified unreachability PROOF (the linter proves it dead) or a
/// verified reachability WITNESS (a clean diverse `--count` sample that parses through the grammar's
/// REAL parser and exercises it). Witnesses are the DIVERSE full-file samples (the closed loop
/// guarantees they parse) — NOT the reach-plan-forced ones; the auxiliary reach passes only UNION
/// extra witnesses from probes that re-parse, so the diverse pass's `sample_parse_failures` stays
/// byte-identical per grammar. Dispatch is by `grammar.grammar_name` through the registry — this
/// function names no grammar. `emit_diagnostics` (true for the canonical run, false for union-config
/// runs) gates ONLY the two in-pass human-readable summary prints; the witness/proof computation is
/// identical either way.
#[cfg(feature = "generated_parsers")]
fn gather_cert_covered_sets(
    grammar: &LoadedGrammar,
    entry: &str,
    samples: usize,
    seed: u64,
    profile: Option<&str>,
    full_defined: &std::collections::HashSet<String>,
    entry_universe: &[String],
    gather_profile_proofs: bool,
    max_depth: usize,
    emit_diagnostics: bool,
) -> Result<CertCoveredSets> {
    use pgen::ast_pipeline::grammar_wellformedness::{
        certificate_coverage, gather_verified_profile_proof_covered_rules,
        gather_verified_proof_covered_rules,
    };
    let entry_rule = entry.to_string();

    // WITNESS side: generate CLEAN diverse full-file samples, parse each through the REAL parser, and
    // union the rules they exercise. Each such rule is verified-reachable (a concrete input parses +
    // exercises it). A sample that fails to parse is a generator bug (counted, never silently dropped).
    //
    // G.4.7 slice 2: enforce_word_boundary_spacing=true. The witnesses must be VALID source, and the
    // default-off config fused adjacent word-tokens (e.g. `endprogram`+`module` -> `endprogrammodule`),
    // making ~half the diverse samples unparseable and capping the witness count (F2). This is the
    // existing word-boundary feature (append_generated_segment), not new code — a level-1 fix.
    // GRAMMAR-WELLFORMED.H.4: honor `--max-depth` (was hardcoded to the StimuliConfig default of 24,
    // unlike every sibling report path which threads `args.max_depth`). The default stays 24 so json /
    // regex / SV cert-coverage is byte-identical, but deeply-recursive grammars (e.g. rtl_const_expr,
    // whose operator-precedence chain `conditional_expr → … → primary_expr → ( conditional_expr )` exceeds
    // 24 after one nesting level) can raise the witness-generation depth budget so the report runs.
    let samples = samples.max(1);
    let mut witness_covered: std::collections::HashSet<String> = std::collections::HashSet::new();

    // PASS 1 — the DIVERSE CERTIFICATION sample set (reach OFF). This is byte-identical to the
    // historical behaviour for every grammar, so its `sample_parse_failures` (a generator
    // over-production the real parser rejects) is the reported certification number and never
    // regresses. GRAMMAR-WELLFORMED.H.4 honors `--max-depth`; the default stays 24 so json / regex /
    // SV are byte-identical.
    let config = StimuliConfig {
        seed: Some(seed),
        enforce_word_boundary_spacing: true,
        max_depth,
        ..Default::default()
    };
    let mut generator = StimuliGenerator::new(
        grammar.grammar_name.clone(),
        &grammar.grammar_tree,
        &grammar.rule_order,
        grammar.annotations.as_ref(),
        config,
    );
    // CERT-GEN-BUDGET.2: bound the diverse pass with a GENEROUS per-sample deterministic
    // step-budget (B1) so a deeply-recursive grammar (e.g. rtl_const_expr at pathological depths)
    // fails fast instead of hanging the cert. The default is far above any well-behaved grammar's
    // per-sample cost — INERT (byte-identical cert) for the 6 fully-certified grammars + SV at the
    // canonical depths — so only a pathological super-linear grammar reaches it. Env-tunable for
    // calibration / a deliberate unbounded run (set 0).
    let diverse_budget_ms = std::env::var("PGEN_CERT_DIVERSE_GENERATION_TIMEOUT_MS")
        .ok()
        .and_then(|raw| raw.trim().parse::<u64>().ok())
        .unwrap_or(CERT_DIVERSE_GENERATION_TIMEOUT_MS_DEFAULT);
    let diverse =
        generator.generate_many_bounded(samples, Some(entry_rule.as_str()), diverse_budget_ms)?;
    pgen::pgen_trace_low!(
        "CERT-GEN-BUDGET.2 diverse pass complete: grammar='{}' samples={} max_depth={} budget_ms={} cumulative_generation_steps={}",
        grammar.grammar_name,
        samples,
        max_depth,
        diverse_budget_ms,
        generator.generation_step_count()
    );
    let mut sample_parse_failures = 0usize;
    // G.4.7: LABEL parse failures (error + sample), never silently count them — a parse failure is a
    // generator-produced sample the real parser rejects, and seeing WHY is how we tell a generator
    // defect from an expected diverse-generation miss. (error, sample) for the first few.
    let mut failures: Vec<(String, String)> = Vec::new();
    for sample in &diverse {
        if let Some((parsed, covered)) =
            pgen::parser_registry::parse_and_cover(&grammar.grammar_name, sample, profile, Some(entry_rule.as_str()))
        {
            if parsed {
                witness_covered.extend(covered);
            } else {
                sample_parse_failures += 1;
                let err = match pgen::parser_registry::parse_error(
                    &grammar.grammar_name,
                    sample,
                    profile,
                ) {
                    Some(Err(e)) => e,
                    Some(Ok(())) => "(re-parse unexpectedly succeeded)".to_string(),
                    None => "(no detail-capable parser registered)".to_string(),
                };
                failures.push((err, sample.clone()));
            }
        }
    }

    // PROOF side: the rules a verified unreachability certificate proves dead (re-checked, not trusted).
    let (mut proof_covered, mut proof_fails) =
        gather_verified_proof_covered_rules(&grammar.grammar_tree, &grammar.rule_order);

    // VERILOG-2005-PROFILE.6.7: PER-PROFILE proof promotion. When a dialect profile is active, a rule
    // stranded by the pruning of every context that could reach it (P1) or dead under the
    // unproducible-store-gate fixpoint (P2) is provably never-witnessed — but the profile-agnostic
    // whole-rule proofs above cannot say so. Fold in the VERIFIED per-profile proofs (each independently
    // re-derived by `verify_profile_certificate`), quantifying over the DECLARED ENTRY UNIVERSE (so the
    // entry-relative library cohort is never falsely branded dead). Gated on `profile.is_some()`: with no
    // profile the single-entry P1 would mis-brand alternate-entry rules, so the no-profile cert stays
    // byte-identical. INERT for a fully-certified grammar (UNKNOWN=0 ⇒ nothing to promote), so the 6
    // fully-certified grammars are unaffected by construction.
    //
    // `gather_profile_proofs` is TRUE only for the CANONICAL (base-config) run — the config being
    // certified. The multi-config union's AUXILIARY per-config runs are pure WITNESS-extenders (their
    // reason for existing), so they do NOT contribute profile-proofs: that keeps the union's proof set =
    // "provably dead under the BASE profile" (so `canonical_proof == union_proof` stays true) and its
    // witness set = "witnessed under some recognized config" — the sound recognized-union model. A rule
    // proof-under-base + witness-under-an-alternate-config is covered either way (union UNKNOWN invariant).
    if let (true, Some(active_profile)) = (gather_profile_proofs, profile) {
        let entries: Vec<String> = entry_universe
            .iter()
            .filter(|e| grammar.grammar_tree.contains_key(*e))
            .cloned()
            .collect();
        let (profile_proof, profile_fails) = gather_verified_profile_proof_covered_rules(
            &grammar.grammar_tree,
            &grammar.rule_order,
            full_defined,
            &entries,
            grammar.annotations.as_ref(),
            active_profile,
        );
        proof_covered.extend(profile_proof);
        proof_fails.extend(profile_fails);
    }

    // PASS 2 — GRAMMAR-WELLFORMED.H.4.2: the CONSTRUCTIVE-REACH witness pass. Run only if the diverse
    // pass left UNKNOWN rules. It is a deliberately SEPARATE, auxiliary witness-finder for branches the
    // clean diverse pass cannot reach within budget — e.g. rtl_const_expr's
    // `primary_expr := lparen conditional_expr rparen`, which re-enters the ~14-level precedence chain,
    // so the diverse budget is exhausted reaching `primary_expr` and the branch always DepthExceeds
    // (raising `max_depth` is not an alternative — the `*`/`?:` fan-out explodes). With
    // `reach_uncovered_recursive_branches`, such a branch is tried at its shallowest reach and its
    // minimal `( 1 )` witness is constructed with a fresh budget. Because this pass is SEPARATE, the
    // pass-1 certification sample set above stays byte-identical for every grammar — a grammar that
    // does not need the reach keeps its exact reported `sample_parse_failures`. The reach pass only
    // UNIONS witnesses from samples that re-parse; its own unparseable probes are inherent to reaching
    // the hardest branches and are reported separately, never folded into the certification number.
    let mut reach_pass_parse_failures = 0usize;
    {
        let pre_report =
            certificate_coverage(&grammar.rule_order, &proof_covered, &witness_covered);
        if !pre_report.unknown.is_empty() {
            let reach_config = StimuliConfig {
                seed: Some(seed),
                enforce_word_boundary_spacing: true,
                max_depth,
                reach_uncovered_recursive_branches: true,
                ..Default::default()
            };
            let mut reach_generator = StimuliGenerator::new(
                grammar.grammar_name.clone(),
                &grammar.grammar_tree,
                &grammar.rule_order,
                grammar.annotations.as_ref(),
                reach_config,
            );
            if let Ok(reach_samples) =
                reach_generator.generate_many(samples, Some(entry_rule.as_str()))
            {
                for sample in &reach_samples {
                    if let Some((parsed, covered)) =
                        pgen::parser_registry::parse_and_cover(&grammar.grammar_name, sample, profile, Some(entry_rule.as_str()))
                    {
                        if parsed {
                            witness_covered.extend(covered);
                        } else {
                            reach_pass_parse_failures += 1;
                        }
                    }
                }
            }
        }
    }

    // PASS 3 — GRAMMAR-WELLFORMED.H.7.2: the PLANNABLE-RULE reach pass. Run only if passes 1+2 still
    // left UNKNOWN rules. The bulk of the per-grammar residual UNKNOWN (svpp `macro_default_text`,
    // most of vhdl/regex/SV/rtl_frontend) is never-witnessed NON-recursive reachable rules behind
    // un-taken optionals and/or un-selected alternation branches — NEITHER depth-exhaustion NOR
    // recursion, so the pass-2 retry is structurally inapplicable to them. For each still-UNKNOWN
    // rule, a rule-target reach plan forces every OR decision AND every quantifier on the
    // rule-reference path (the one new H.7.2 capability — construct_mode alone would minimize the
    // gating `?`/`*` to zero), then ONE minimal construct-mode generation + ONE plan-forced search
    // fallback produce a candidate witness. Like pass 2, this pass only UNIONS witnesses from
    // samples that RE-PARSE; its own probe failures are reported separately, never folded into the
    // pass-1 certification number (`sample_parse_failures` stays byte-identical per grammar).
    let mut plannable_pass_parse_failures = 0usize;
    let mut plannable_no_path: Vec<String> = Vec::new();
    let mut plannable_generation_failures = 0usize;
    let mut plannable_left_unattempted = 0usize;
    let mut plannable_attempted = 0usize;
    let mut plannable_witnessed = 0usize;
    let mut plannable_parsed_not_witnessed = 0usize;
    let mut target_own_attempted = 0usize;
    let mut target_own_witnessed = 0usize;
    {
        let pre_report =
            certificate_coverage(&grammar.rule_order, &proof_covered, &witness_covered);
        if !pre_report.unknown.is_empty() {
            // GRAMMAR-WELLFORMED.H.7.2 Q3: a deterministic global attempt cap — big-UNKNOWN
            // grammars (SV ~1100 rules) stay bounded; anything beyond the cap is reported
            // LOUDLY below, never silently truncated.
            const MAX_PLANNABLE_REACH_ATTEMPTS: usize = 4096;
            const PLANNABLE_REACH_ATTEMPT_TIMEOUT_MS: u64 = 250;
            // GRAMMAR-WELLFORMED.H.7.2 Q3: bounded per-rule retry — a probe can parse yet
            // route through other rules when a random terminal expansion is unluckily
            // shaped; the construct skeleton is deterministic, the expansions vary, so a
            // few retries converge. Deterministic for a fixed seed.
            const PLANNABLE_REACH_MAX_ATTEMPTS_PER_RULE: usize = 4;
            let mut targets: Vec<String> = pre_report.unknown.clone();
            if targets.len() > MAX_PLANNABLE_REACH_ATTEMPTS {
                plannable_left_unattempted = targets.len() - MAX_PLANNABLE_REACH_ATTEMPTS;
                targets.truncate(MAX_PLANNABLE_REACH_ATTEMPTS);
            }
            plannable_attempted = targets.len();
            let plannable_config = StimuliConfig {
                seed: Some(seed),
                enforce_word_boundary_spacing: true,
                max_depth,
                ..Default::default()
            };
            let mut plannable_generator = StimuliGenerator::new(
                grammar.grammar_name.clone(),
                &grammar.grammar_tree,
                &grammar.rule_order,
                grammar.annotations.as_ref(),
                plannable_config,
            );
            // GRAMMAR-WELLFORMED.H.7.2: the witness check — replay each probe through the
            // REAL parser; union covered rules from every sample that parses (a probe that
            // misses its own target can still legitimately witness other rules); tell the
            // driver whether the accepted parse actually entered the target rule so it can
            // retry within its bounded per-rule budget. `PGEN_CERT_COVERAGE_DEBUG_PROBES=1`
            // prints each probe for diagnosis (informational only — gated, never default).
            let debug_probes = std::env::var_os("PGEN_CERT_COVERAGE_DEBUG_PROBES").is_some();
            let grammar_name = grammar.grammar_name.clone();
            let pass_report = plannable_generator.generate_plannable_rule_witnesses(
                entry_rule.as_str(),
                &targets,
                PLANNABLE_REACH_ATTEMPT_TIMEOUT_MS,
                PLANNABLE_REACH_MAX_ATTEMPTS_PER_RULE,
                |rule, sample| {
                    let Some((parsed, covered)) =
                        pgen::parser_registry::parse_and_cover(&grammar_name, sample, profile, Some(entry_rule.as_str()))
                    else {
                        return PlannableProbeVerdict::NotParsed;
                    };
                    let witnessed = parsed && covered.contains(rule);
                    if debug_probes {
                        println!(
                            "  [plannable-probe] rule='{}' parsed={} witnessed_target={} sample={:?}",
                            rule, parsed, witnessed, sample
                        );
                    }
                    if parsed {
                        witness_covered.extend(covered);
                        if witnessed {
                            PlannableProbeVerdict::Witnessed
                        } else {
                            PlannableProbeVerdict::ParsedNotWitnessed
                        }
                    } else {
                        PlannableProbeVerdict::NotParsed
                    }
                },
            );
            plannable_no_path = pass_report.no_path.clone();
            plannable_pass_parse_failures = pass_report.probe_parse_failures;
            plannable_generation_failures = pass_report.generation_failures;
            plannable_witnessed = pass_report.witnessed;
            plannable_parsed_not_witnessed = pass_report.parsed_not_witnessed;

            // PASS 3c — GRAMMAR-WELLFORMED.H.12.5.5.3.2 (M1b): the target-own-structure reach pass.
            // Run ONLY over the rules STILL UNKNOWN after passes 1+2+3, so a grammar those passes
            // already fully certify has an empty residual and this pass is truly inert (it never
            // generates a probe — the fully-certified roster is unaffected). For each genuine residual
            // rule it forces the target rule's OWN root-`Or` branch + inner optionals (on top of the
            // base reach plan to its reference site), closing the M1b residual where "the reach forces
            // the path to the target but not the target's own distinguishing structure". Like passes
            // 2+3 it only UNIONS witnesses from probes that re-parse, so the certification
            // `sample_parse_failures` (the diverse pass) stays byte-identical. Parser-agnostic.
            let post_plannable =
                certificate_coverage(&grammar.rule_order, &proof_covered, &witness_covered);
            if !post_plannable.unknown.is_empty() {
                target_own_attempted = post_plannable.unknown.len();
                target_own_witnessed = plannable_generator
                    .generate_target_own_structure_witnesses(
                        entry_rule.as_str(),
                        &post_plannable.unknown,
                        PLANNABLE_REACH_ATTEMPT_TIMEOUT_MS,
                        PLANNABLE_REACH_MAX_ATTEMPTS_PER_RULE,
                        |rule, sample| {
                            let Some((parsed, covered)) =
                                pgen::parser_registry::parse_and_cover(&grammar_name, sample, profile, Some(entry_rule.as_str()))
                            else {
                                return PlannableProbeVerdict::NotParsed;
                            };
                            let witnessed = parsed && covered.contains(rule);
                            if debug_probes {
                                println!(
                                    "  [target-own-probe] rule='{}' parsed={} witnessed_target={} sample={:?}",
                                    rule, parsed, witnessed, sample
                                );
                            }
                            if parsed {
                                witness_covered.extend(covered);
                                if witnessed {
                                    PlannableProbeVerdict::Witnessed
                                } else {
                                    PlannableProbeVerdict::ParsedNotWitnessed
                                }
                            } else {
                                PlannableProbeVerdict::NotParsed
                            }
                        },
                    );
            }

            // PASS 3d — GRAMMAR-WELLFORMED.H.12.5.6.2.2.2 (M2a reach-honesty): the store-free reach
            // pass. Run LAST, over ONLY the rules still UNKNOWN after the diverse / plannable /
            // target-own passes, so every prior pass keeps its exact RNG stream and witness landscape
            // (no newly-UNKNOWN by construction) and this pass can only UNION new witnesses. It routes
            // each residual target via the store-gated-edge-deprioritized BFS, so a cluster reachable
            // BOTH via a non-gated carrier and a (shorter) store-gated carrier — e.g. the SV constraint
            // body via in-class `constraint_declaration` vs out-of-class `extern_constraint_declaration`
            // (whose mandatory `class_scope` needs a DECLARED class no minimal witness can provide) — is
            // re-routed through the non-gated carrier and witnesses. Truly inert when no fact-query
            // predicate exists or the residual is empty (the fully-certified roster). Like passes 2/3/3c
            // it only UNIONS witnesses from probes that re-parse, so the certification
            // `sample_parse_failures` (the diverse pass) stays byte-identical.
            let post_target_own =
                certificate_coverage(&grammar.rule_order, &proof_covered, &witness_covered);
            if !post_target_own.unknown.is_empty() {
                let store_free_report = plannable_generator.generate_plannable_store_free_witnesses(
                    entry_rule.as_str(),
                    &post_target_own.unknown,
                    PLANNABLE_REACH_ATTEMPT_TIMEOUT_MS,
                    PLANNABLE_REACH_MAX_ATTEMPTS_PER_RULE,
                    |rule, sample| {
                        let Some((parsed, covered)) =
                            pgen::parser_registry::parse_and_cover(&grammar_name, sample, profile, Some(entry_rule.as_str()))
                        else {
                            return PlannableProbeVerdict::NotParsed;
                        };
                        let witnessed = parsed && covered.contains(rule);
                        if debug_probes {
                            println!(
                                "  [store-free-probe] rule='{}' parsed={} witnessed_target={} sample={:?}",
                                rule, parsed, witnessed, sample
                            );
                        }
                        if parsed {
                            witness_covered.extend(covered);
                            if witnessed {
                                PlannableProbeVerdict::Witnessed
                            } else {
                                PlannableProbeVerdict::ParsedNotWitnessed
                            }
                        } else {
                            PlannableProbeVerdict::NotParsed
                        }
                    },
                );
                if emit_diagnostics {
                    println!(
                        "  (store-free reach pass: {} residual UNKNOWN rules targeted; {} witnessed by re-routing through a non-gated carrier)",
                        post_target_own.unknown.len(),
                        store_free_report.witnessed
                    );
                }
            }

            // PASS 3e — STORE-AWARE-GEN.4b.12 (9C-i): the carrier-diversification reach pass. Run LAST,
            // over ONLY the rules still UNKNOWN after the diverse / plannable / target-own / store-free
            // passes, so every prior pass keeps its exact RNG stream and witness landscape (no
            // newly-UNKNOWN by construction) and this pass can only UNION new witnesses. It re-routes a
            // residual target's reach plan to reach a rule on its default path through an ALTERNATIVE
            // parent carrier, keeping the tail to the target — so a different trailing context can defeat
            // a parent-ordered-choice sibling that shadows the target on the BFS-shortest carrier. The
            // motivating win: the SV class-scope `type_parameter`/`interface_class` family, reached via
            // `class_new` (`:: new` suffix) instead of a data-declaration carrier (`:: id` suffix the
            // generic `scoped_class_scope_identifier` alternative consumes). Truly inert for an empty
            // residual (the fully-certified roster). Like passes 2/3/3c/3d it only UNIONS witnesses from
            // probes that re-parse, so the certification `sample_parse_failures` stays byte-identical.
            let post_store_free =
                certificate_coverage(&grammar.rule_order, &proof_covered, &witness_covered);
            if !post_store_free.unknown.is_empty() {
                let carrier_div_witnessed = plannable_generator
                    .generate_carrier_diversified_witnesses(
                        entry_rule.as_str(),
                        &post_store_free.unknown,
                        PLANNABLE_REACH_ATTEMPT_TIMEOUT_MS,
                        PLANNABLE_REACH_MAX_ATTEMPTS_PER_RULE,
                        |rule, sample| {
                            let Some((parsed, covered)) =
                                pgen::parser_registry::parse_and_cover(&grammar_name, sample, profile, Some(entry_rule.as_str()))
                            else {
                                return PlannableProbeVerdict::NotParsed;
                            };
                            let witnessed = parsed && covered.contains(rule);
                            if debug_probes {
                                println!(
                                    "  [carrier-div-probe] rule='{}' parsed={} witnessed_target={} sample={:?}",
                                    rule, parsed, witnessed, sample
                                );
                            }
                            if parsed {
                                witness_covered.extend(covered);
                                if witnessed {
                                    PlannableProbeVerdict::Witnessed
                                } else {
                                    PlannableProbeVerdict::ParsedNotWitnessed
                                }
                            } else {
                                PlannableProbeVerdict::NotParsed
                            }
                        },
                    );
                if emit_diagnostics {
                    println!(
                        "  (carrier-diversification reach pass: {} residual UNKNOWN rules targeted; {} witnessed by re-routing through an alternative parent carrier)",
                        post_store_free.unknown.len(),
                        carrier_div_witnessed
                    );
                }
            }

            // PASS 3f — STRUCTURED-WITNESS-SYNTH.3: the structured-witness COMPOSITION pass. Run
            // LAST, over ONLY the rules still UNKNOWN after every prior pass (diverse / plannable /
            // target-own / store-free / carrier-diversification), so every prior pass keeps its
            // exact RNG stream and witness landscape and this pass can only UNION new witnesses. A
            // store-gated structured target can need THREE conditions SIMULTANEOUSLY — a
            // fact-emitting typed declaration prelude, the gated consumer's head pinned to the
            // declared name, and the target's own distinguishing structure (the SV
            // declare→chain→method-call shape) — which the earlier passes each provide only in
            // isolation. This pass composes the name-coordinated prelude (with the pass-scoped
            // dotted-emit producer admission + typed-branch forcing on the prelude sub-path), the
            // head-leaf name pin, and the target-own structure directives into ONE reach plan per
            // residual target. Truly inert when the grammar has no name-matching store gate or the
            // residual is empty (the fully-certified roster). Like passes 2/3/3c/3d/3e it only
            // UNIONS witnesses from probes that re-parse, so the certification
            // `sample_parse_failures` stays byte-identical.
            let post_carrier_div =
                certificate_coverage(&grammar.rule_order, &proof_covered, &witness_covered);
            if !post_carrier_div.unknown.is_empty() {
                let structured_witnessed = plannable_generator.generate_structured_witnesses(
                    entry_rule.as_str(),
                    &post_carrier_div.unknown,
                    PLANNABLE_REACH_ATTEMPT_TIMEOUT_MS,
                    PLANNABLE_REACH_MAX_ATTEMPTS_PER_RULE,
                    |rule, sample| {
                        let Some((parsed, covered)) =
                            pgen::parser_registry::parse_and_cover(&grammar_name, sample, profile, Some(entry_rule.as_str()))
                        else {
                            return PlannableProbeVerdict::NotParsed;
                        };
                        let witnessed = parsed && covered.contains(rule);
                        if debug_probes {
                            println!(
                                "  [structured-witness-probe] rule='{}' parsed={} witnessed_target={} sample={:?}",
                                rule, parsed, witnessed, sample
                            );
                        }
                        if parsed {
                            witness_covered.extend(covered);
                            if witnessed {
                                PlannableProbeVerdict::Witnessed
                            } else {
                                PlannableProbeVerdict::ParsedNotWitnessed
                            }
                        } else {
                            PlannableProbeVerdict::NotParsed
                        }
                    },
                );
                if emit_diagnostics {
                    println!(
                        "  (structured-witness reach pass: {} residual UNKNOWN rules targeted; {} witnessed by composing declare-then-use prelude + head-leaf pin + target-own structure)",
                        post_carrier_div.unknown.len(),
                        structured_witnessed
                    );
                }
            }
        }
    }

    Ok(CertCoveredSets {
        proof_covered,
        witness_covered,
        proof_fails,
        sample_parse_failures,
        failures,
        reach_pass_parse_failures,
        plannable_no_path,
        plannable_pass_parse_failures,
        plannable_generation_failures,
        plannable_left_unattempted,
        plannable_attempted,
        plannable_witnessed,
        plannable_parsed_not_witnessed,
        target_own_attempted,
        target_own_witnessed,
    })
}

/// GRAMMAR-WELLFORMED.G.4 / H.12.8.1.1: the certificate-coverage REPORT. Runs the canonical
/// `(entry, profile)` config (verbose — byte-identical to the historical single-config report) and,
/// when `--cert-union-config` configs are supplied, an opt-in MULTI-CONFIG union: each extra config's
/// VERIFIED covered sets (`proof ∪ witness`) union into the canonical accounting, then the canonical
/// `rule_order` is re-classified against the bigger covered sets and a `CERTIFICATE-COVERAGE-UNION:`
/// line is printed. SOUNDNESS (load-bearing): the union is over POSITIVELY-covered sets, NEVER over
/// "not-UNKNOWN-in-some-config" — a rule a profile filters OUT of its `rule_order` is neither covered
/// nor UNKNOWN there, so it contributes nothing (that is why we union `proof_covered`/`witness_covered`
/// and classify the canonical fragment set, not subtract per-config UNKNOWN sets). With no union
/// configs the canonical path is byte-identical for every grammar. PARSER-AGNOSTIC — names no grammar;
/// the supported config set is declared entirely by the caller via the CLI.
#[cfg(feature = "generated_parsers")]
fn run_certificate_coverage_report(
    grammar: &LoadedGrammar,
    unfiltered_grammar: &LoadedGrammar,
    entry: Option<&str>,
    samples: usize,
    seed: u64,
    profile: Option<&str>,
    max_depth: usize,
    union_configs: &[String],
) -> Result<()> {
    use pgen::ast_pipeline::grammar_wellformedness::certificate_coverage;
    if !pgen::parser_registry::supports_parse_and_cover(&grammar.grammar_name) {
        anyhow::bail!(
            "certificate-coverage: no generated parser is registered for grammar '{}' — cannot \
             verify reachability witnesses through a real parser (Phase H wires more grammars)",
            grammar.grammar_name
        );
    }
    let entry_rule = entry
        .map(|s| s.to_string())
        .or_else(|| grammar.rule_order.first().cloned())
        .ok_or_else(|| anyhow::anyhow!("grammar '{}' has no rules", grammar.grammar_name))?;
    let samples = samples.max(1);

    // VERILOG-2005-PROFILE.6.7: the DECLARED ENTRY UNIVERSE for per-profile proof gathering — the
    // canonical entry PLUS every `--cert-union-config` entry (deduped, in declaration order). Each
    // `gather_cert_covered_sets` call filters this to the entries present in its own active tree, so
    // the entry-relative library cohort (`library_text`, …) is reachable → never falsely proved dead
    // (the `.6.5` load-bearing requirement). `full_defined` = the PRE-filter rule names, so
    // profile-PRUNED references (unsatisfiable) are distinguished from external/include references
    // (never accused). Both are profile-INERT when no profile is active (the gathering is gated on
    // `profile.is_some()`).
    let mut entry_universe: Vec<String> = vec![entry_rule.clone()];
    for raw in union_configs {
        let cfg_entry = match raw.split_once(':') {
            Some((e, _)) => e.trim(),
            None => raw.trim(),
        };
        if !cfg_entry.is_empty() && !entry_universe.iter().any(|e| e == cfg_entry) {
            entry_universe.push(cfg_entry.to_string());
        }
    }
    let full_defined: std::collections::HashSet<String> =
        unfiltered_grammar.grammar_tree.keys().cloned().collect();

    // CANONICAL pass — verbose (`emit_diagnostics=true`), so its per-pass diagnostic lines +
    // PGEN_CERT_COVERAGE_DEBUG_PROBES output are byte-identical to the historical single-config report.
    let canonical = gather_cert_covered_sets(
        grammar,
        &entry_rule,
        samples,
        seed,
        profile,
        &full_defined,
        &entry_universe,
        true, // canonical/base config — gather per-profile proofs
        max_depth,
        true,
    )?;

    let report = certificate_coverage(
        &grammar.rule_order,
        &canonical.proof_covered,
        &canonical.witness_covered,
    );
    println!(
        "CERTIFICATE-COVERAGE: grammar='{}' entry='{}' samples={} total={} proof={} witness={} UNKNOWN={} fully_certified={} (sample_parse_failures={}, proof_reverify_failures={})",
        grammar.grammar_name,
        entry_rule,
        samples,
        report.total,
        report.covered_by_proof.len(),
        report.covered_by_witness.len(),
        report.unknown.len(),
        report.is_fully_certified(),
        canonical.sample_parse_failures,
        canonical.proof_fails.len(),
    );
    if canonical.reach_pass_parse_failures > 0 {
        // GRAMMAR-WELLFORMED.H.4.2: transparency — the auxiliary constructive-reach pass probes the
        // hardest-to-reach branches, so some of its samples are expected not to re-parse. They are
        // reported here, NOT folded into the certification `sample_parse_failures` (the diverse pass).
        println!(
            "  (constructive-reach witness pass: {} auxiliary probe samples did not re-parse — not counted as certification failures)",
            canonical.reach_pass_parse_failures
        );
    }
    if canonical.plannable_attempted > 0 {
        // GRAMMAR-WELLFORMED.H.7.2: transparency for the plannable-rule reach pass — same contract
        // as pass 2 (probe non-parses reported separately, never folded into certification).
        println!(
            "  (plannable-rule reach pass: {} UNKNOWN rules targeted; {} witnessed, {} parsed-but-routed-elsewhere, {} probe samples did not re-parse, {} generation failures — probe failures are not certification failures)",
            canonical.plannable_attempted,
            canonical.plannable_witnessed,
            canonical.plannable_parsed_not_witnessed,
            canonical.plannable_pass_parse_failures,
            canonical.plannable_generation_failures
        );
    }
    if canonical.target_own_attempted > 0 {
        // GRAMMAR-WELLFORMED.H.12.5.5.3.2: transparency for the target-own-structure (M1b) reach pass.
        // Only runs over the residual still UNKNOWN after pass 3, so a fully-certified grammar reports
        // nothing here (zero residual ⇒ pass not run).
        println!(
            "  (target-own-structure reach pass: {} residual UNKNOWN rules targeted; {} witnessed by forcing the target rule's own root-Or branch + inner optionals)",
            canonical.target_own_attempted, canonical.target_own_witnessed
        );
    }
    // GRAMMAR-WELLFORMED.H.12.4: env-gated full-dump observability. When PGEN_CERT_COVERAGE_DUMP_ALL
    // is set, the no_path and UNKNOWN lists below print IN FULL (the @10 / @25 print caps are lifted)
    // so the residual can be enumerated and adjudicated (e.g. A1 alternate-entry vs A2 profile-orphan)
    // deterministically. Unset (the default) is byte-identical to the prior capped output. The lists
    // are emitted in the report's existing deterministic (rule-order) sequence, so the dump is stable.
    let dump_all = std::env::var_os("PGEN_CERT_COVERAGE_DUMP_ALL").is_some();
    if !canonical.plannable_no_path.is_empty() {
        // GRAMMAR-WELLFORMED.H.7.2: by the attribution rule, an UNKNOWN rule with NO path in the
        // rule-reference graph is grammar/linter territory (a dead rule candidate) — flag it loudly.
        let shown = if dump_all {
            canonical.plannable_no_path.len()
        } else {
            canonical.plannable_no_path.len().min(10)
        };
        println!(
            "  WARNING plannable-rule reach pass: {} UNKNOWN rules have NO reach path from the entry (dead-rule candidates — adjudicate via the linter): {:?}",
            canonical.plannable_no_path.len(),
            &canonical.plannable_no_path[..shown]
        );
    }
    if canonical.plannable_left_unattempted > 0 {
        // GRAMMAR-WELLFORMED.H.7.2 Q3: the global cap is reported, never silent.
        println!(
            "  WARNING plannable-rule reach pass: {} UNKNOWN rules were LEFT UNATTEMPTED by the global attempt cap — rerun or raise the cap to cover them",
            canonical.plannable_left_unattempted
        );
    }
    if !report.unknown.is_empty() {
        let shown = if dump_all { report.unknown.len() } else { report.unknown.len().min(25) };
        println!(
            "  UNKNOWN rules ({} of {} shown): {:?}",
            shown,
            report.unknown.len(),
            &report.unknown[..shown]
        );
    }
    // VERILOG-2005-PROFILE.6.6: the READ-ONLY, env-gated residual classification (P1
    // profile-entry-universe unreachability + P2 unproducible-mandatory-store-gate fixpoint,
    // designed in `.6.5`). Prints ONLY under PGEN_CERT_RESIDUAL_CLASSIFICATION (default output is
    // byte-identical by construction — the analysis does not even run otherwise) and never touches
    // generation or the reach passes. Reuses the run's DECLARED ENTRY UNIVERSE (filtered to the
    // ACTIVE profile-filtered tree) + the pre-filter rule set (distinguishes profile-PRUNED
    // references — unsatisfiable — from external/include references, never accused). Since `.6.7`
    // promoted P1∪P2 into `proof`, `report.unknown` is now the GENUINE remainder, so this block is a
    // confirmation surface (it should classify the residual as all-`genuine`).
    if std::env::var_os("PGEN_CERT_RESIDUAL_CLASSIFICATION").is_some() && !report.unknown.is_empty()
    {
        use pgen::ast_pipeline::grammar_wellformedness::classify_profile_residual;
        let entries: Vec<String> = entry_universe
            .iter()
            .filter(|e| grammar.grammar_tree.contains_key(*e))
            .cloned()
            .collect();
        let classification = classify_profile_residual(
            &grammar.grammar_tree,
            &grammar.rule_order,
            &full_defined,
            &entries,
            grammar.annotations.as_ref(),
            &report.unknown,
        );
        println!(
            "  RESIDUAL-CLASSIFICATION (read-only; PGEN_CERT_RESIDUAL_CLASSIFICATION): profile='{}' entry_universe={:?} store_analysis={}",
            profile.unwrap_or("<none>"),
            classification.entries,
            if classification.degraded_inert {
                "DEGRADED-INERT (a live rule carries @import_from_library)"
            } else {
                "active"
            },
        );
        println!(
            "    profile_entry_unreachable ({}): {:?}",
            classification.profile_entry_unreachable.len(),
            classification.profile_entry_unreachable
        );
        let store: Vec<String> = classification
            .store_unproducible
            .iter()
            .map(|(rule, why)| format!("{rule} [{why}]"))
            .collect();
        println!(
            "    store_unproducible_under_profile ({}): {:?}",
            store.len(),
            store
        );
        println!(
            "    genuine ({}): {:?}",
            classification.genuine.len(),
            classification.genuine
        );
    }
    if !canonical.proof_fails.is_empty() {
        println!(
            "  WARNING proof re-verify FAILURES (linter bugs to fix): {:?}",
            canonical.proof_fails
        );
    }
    if !canonical.failures.is_empty() {
        let shown = canonical.failures.len().min(5);
        println!(
            "  SAMPLE-PARSE FAILURES ({} of {} shown — these cap the witness count; each is a \
             generated sample the real parser rejects):",
            shown,
            canonical.failures.len()
        );
        for (i, (err, sample)) in canonical.failures.iter().take(shown).enumerate() {
            let preview: String = sample.chars().take(2000).collect();
            let truncated = if sample.len() > preview.len() { " …[truncated]" } else { "" };
            println!("    [{i}] error: {err}");
            println!("    [{i}] sample ({} bytes): {preview}{truncated}", sample.len());
        }
    }

    // GRAMMAR-WELLFORMED.H.12.8.1.1: the opt-in MULTI-CONFIG union. For each `--cert-union-config`
    // value, re-filter the UNFILTERED bundle by that profile, gather its verified covered sets QUIETLY
    // (`emit_diagnostics=false` — no per-pass spam; DEBUG_PROBES still honored if set), and UNION them
    // into the canonical covered sets. Then re-classify the canonical `rule_order` against the bigger
    // sets. Empty `union_configs` ⇒ this block is skipped entirely ⇒ byte-identical canonical-only
    // output (the inertness proof for the fully-certified roster, which never passes the flag).
    if !union_configs.is_empty() {
        let mut proof_union = canonical.proof_covered.clone();
        let mut witness_union = canonical.witness_covered.clone();
        let mut applied_configs: Vec<String> = Vec::new();
        for raw in union_configs {
            let (cfg_entry, cfg_profile) = match raw.split_once(':') {
                Some((e, p)) => (e.trim(), Some(p.trim())),
                None => (raw.trim(), None),
            };
            let cfg_profile = cfg_profile.filter(|p| !p.is_empty());
            if cfg_entry.is_empty() {
                anyhow::bail!(
                    "--cert-union-config '{raw}' has an empty entry rule (expected <entry>[:<profile>])"
                );
            }
            // Re-filter the UNFILTERED bundle by this config's profile (re-uses the canonical filter,
            // so the regex pcre2-by-default rule and profile-orphan filtering apply identically).
            let cfg_grammar = apply_grammar_profile_filter(unfiltered_grammar.clone(), cfg_profile)?;
            if !cfg_grammar.rule_order.iter().any(|r| r == cfg_entry) {
                anyhow::bail!(
                    "--cert-union-config '{raw}': entry rule '{cfg_entry}' is not present in grammar \
                     '{}'{} — check the entry/profile spelling",
                    grammar.grammar_name,
                    cfg_profile
                        .map(|p| format!(" under profile '{p}'"))
                        .unwrap_or_default()
                );
            }
            let sets = gather_cert_covered_sets(
                &cfg_grammar,
                cfg_entry,
                samples,
                seed,
                cfg_profile,
                &full_defined,
                &entry_universe,
                false, // auxiliary union config — witness-extender only, no profile-proofs
                max_depth,
                false,
            )?;
            proof_union.extend(sets.proof_covered);
            witness_union.extend(sets.witness_covered);
            applied_configs.push(format!("{}:{}", cfg_entry, cfg_profile.unwrap_or("<none>")));
        }
        let union_report =
            certificate_coverage(&grammar.rule_order, &proof_union, &witness_union);
        println!(
            "CERTIFICATE-COVERAGE-UNION: grammar='{}' base_entry='{}' base_profile='{}' union_configs=[{}] total={} proof={} witness={} UNKNOWN={} fully_certified={}",
            grammar.grammar_name,
            entry_rule,
            profile.unwrap_or("<none>"),
            applied_configs.join(", "),
            union_report.total,
            union_report.covered_by_proof.len(),
            union_report.covered_by_witness.len(),
            union_report.unknown.len(),
            union_report.is_fully_certified(),
        );
        if !union_report.unknown.is_empty() {
            let shown = if dump_all {
                union_report.unknown.len()
            } else {
                union_report.unknown.len().min(25)
            };
            println!(
                "  UNION UNKNOWN rules ({} of {} shown): {:?}",
                shown,
                union_report.unknown.len(),
                &union_report.unknown[..shown]
            );
        }
    }

    Ok(())
}

/// RGX-0078.5.h.1 — the STEP-0 fusibility census report (read-only; the derived-scanner
/// capability-gate classifier over the loaded gen-AST). Prints the census (and, when
/// entry-count files are given, the measured entry share), optionally writes the full
/// machine-readable census as JSON, then exits.
fn run_fusibility_census_report(
    grammar: &LoadedGrammar,
    census_json_path: Option<&str>,
    entry_counts_spec: Option<&str>,
    outcome_counts_spec: Option<&str>,
) -> Result<()> {
    use pgen::ast_pipeline::fusibility_census::{print_fusibility_census, run_fusibility_census};

    let split_spec = |spec: Option<&str>| -> Vec<std::path::PathBuf> {
        spec.map(|spec| {
            spec.split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(std::path::PathBuf::from)
                .collect()
        })
        .unwrap_or_default()
    };
    let entry_counts_files = split_spec(entry_counts_spec);
    // RGX-0078.5.h.1b — the raw+committed outcome files for the merged-choice join.
    let outcome_counts_files = split_spec(outcome_counts_spec);

    let census = run_fusibility_census(
        &grammar.grammar_name,
        &grammar.grammar_tree,
        &grammar.rule_order,
        grammar.annotations.as_ref(),
        &entry_counts_files,
        &outcome_counts_files,
    )
    .map_err(|e| anyhow::anyhow!("fusibility census failed: {e}"))?;

    let dump_all = std::env::var("PGEN_FUSIBILITY_DUMP_ALL").is_ok_and(|v| v == "1");
    print_fusibility_census(&census, dump_all);

    if let Some(path) = census_json_path {
        let json = serde_json::to_string_pretty(&census)
            .map_err(|e| anyhow::anyhow!("fusibility census JSON serialization failed: {e}"))?;
        std::fs::write(path, json)
            .map_err(|e| anyhow::anyhow!("cannot write fusibility census JSON '{path}': {e}"))?;
        println!("  census JSON written to {path}");
    }
    Ok(())
}

/// SV-CORPUS-GRAD.7 (parser-agnostic): the corpus rule-coverage instrument's DENOMINATOR dump —
/// the grammar's full rule inventory with, per rule, the declared `@profiles` set (absent =
/// universal) and the DERIVED per-profile satisfiability (`derive_rule_profiles`, the same
/// transitive computation the profile-orphan lint gates on). Deterministic output (BTreeMap
/// ordering + sorted profile lists) so coverage reports diff cleanly across sessions.
fn run_dump_rule_profiles(unfiltered_grammar: &LoadedGrammar, out_path: &str) -> Result<()> {
    use pgen::ast_pipeline::grammar_wellformedness::{
        derive_rule_profiles, extract_profile_context,
    };
    use std::collections::BTreeMap;

    let (declared, all_profiles) = match unfiltered_grammar.annotations.as_ref() {
        Some(ann) => extract_profile_context(ann),
        None => (HashMap::new(), Vec::new()),
    };
    let satisfiable = derive_rule_profiles(
        &unfiltered_grammar.grammar_tree,
        &unfiltered_grammar.rule_order,
        &declared,
        &all_profiles,
    );
    let mut rules: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    for rule in &unfiltered_grammar.rule_order {
        if !unfiltered_grammar.grammar_tree.contains_key(rule) {
            continue;
        }
        let declared_json = declared.get(rule).map(|v| {
            let mut sorted = v.clone();
            sorted.sort();
            sorted
        });
        let mut sat = satisfiable.get(rule).cloned().unwrap_or_default();
        sat.sort();
        rules.insert(
            rule.clone(),
            serde_json::json!({
                "declared_profiles": declared_json,
                "satisfiable_under": sat,
            }),
        );
    }
    let payload = serde_json::json!({
        "grammar": unfiltered_grammar.grammar_name,
        "profiles": all_profiles,
        "rule_count": rules.len(),
        "rules": rules,
    });
    std::fs::write(out_path, serde_json::to_string_pretty(&payload)?)
        .with_context(|| format!("failed to write --dump-rule-profiles output '{out_path}'"))?;
    println!(
        "rule-profiles dump: '{}' — {} rules, {} profiles -> {}",
        unfiltered_grammar.grammar_name,
        rules.len(),
        all_profiles.len(),
        out_path
    );
    Ok(())
}

fn run_grammar_lint(grammar: &LoadedGrammar, unfiltered_grammar: &LoadedGrammar) -> Result<()> {
    use pgen::ast_pipeline::grammar_wellformedness::{
        detect_always_succeeds_alternatives, detect_left_recursion, detect_nonterminating_rules,
        detect_nullable_repetition, detect_ordered_choice_shadowing, detect_profile_orphans,
        detect_unbound_fact_kinds, detect_undefined_references, detect_unreachable_rules,
    };
    use pgen::ast_pipeline::semantic_directive_registry::parse_semantic_string_list;
    let g = &grammar.grammar_tree;
    let order = &grammar.rule_order;
    let lr = detect_left_recursion(g, order);
    let nonterm = detect_nonterminating_rules(g, order);
    // GRAMMAR-WELLFORMED.A2.3: the annotations feed the per-rule effective @branch_policy — the
    // fixed-terminal-prefix deadness verdict fires only where its first-success-commit premise
    // holds (`@branch_policy: ordered`, no branch-phase predicates).
    let shadow = detect_ordered_choice_shadowing(g, order, grammar.annotations.as_ref());
    // GRAMMAR-WELLFORMED.A2.2: the NON-VERDICT always-succeeds smell (a nullable/total earlier
    // alternative). It makes NO deadness claim (the old unsound `EarlierAlwaysMatches` shadowing
    // verdict was retired) and never gates — surfaced as a [note].
    let always_notes = detect_always_succeeds_alternatives(g, order);
    let nullrep = detect_nullable_repetition(g, order);
    // GRAMMAR-WELLFORMED.A1b: structural reachability — rules defined but unreachable from any
    // root (entry + unreferenced secondary entries). A dead rule is a well-formedness defect.
    let unreachable = detect_unreachable_rules(g, order);

    // UNDEFINED-REF-DIAGNOSTICS.2 (F6): references to rules that are neither defined nor
    // codegen-native builtins — codegen emits a never-matching stub for them, silently killing
    // every referencing production. The structural dual of unreachable_rules; hard gate (all
    // shipped grammars are clean at 0 — the session-#50 sweep). Runs on the UNFILTERED bundle:
    // codegen always emits the FULL grammar (profile selection is a runtime guard), so the
    // filtered view's deliberately-stripped @profiles rule definitions must not read as danglers.
    let undefined_refs = detect_undefined_references(
        &unfiltered_grammar.grammar_tree,
        &unfiltered_grammar.rule_order,
    );

    // GRAMMAR-WELLFORMED.F1: data-dependent binding-before-use — a @predicate consulting a
    // fact-kind that no @emit_fact establishes (the fact can never be bound). Hard gate (all
    // authored grammars are clean: SV's consulted kinds all have emitters).
    let unbound_facts = grammar
        .annotations
        .as_ref()
        .map(detect_unbound_fact_kinds)
        .unwrap_or_default();

    // ANNOTATION-COMPOSITION.2: extract each rule's @profiles set from the annotations (same
    // shape the generator filters by) + the profile universe, then sweep for profile orphans.
    let mut rule_profiles: HashMap<String, Vec<String>> = HashMap::new();
    let mut profile_universe: std::collections::BTreeSet<String> = Default::default();
    if let Some(ann) = grammar.annotations.as_ref() {
        for (rule, entries) in &ann.semantic_annotations {
            for annotation in entries {
                let is_profiles = annotation
                    .name()
                    .map(|n| n.trim().to_ascii_lowercase())
                    == Some("profiles".to_string());
                if !is_profiles {
                    continue;
                }
                if let Some(list) = parse_semantic_string_list(annotation.ast().payload_text()) {
                    let profs: Vec<String> = list
                        .into_iter()
                        .map(|v| v.trim().to_ascii_lowercase())
                        .filter(|v| !v.is_empty())
                        .collect();
                    if !profs.is_empty() {
                        for p in &profs {
                            profile_universe.insert(p.clone());
                        }
                        rule_profiles.insert(rule.clone(), profs);
                    }
                }
            }
        }
    }
    let all_profiles: Vec<String> = profile_universe.into_iter().collect();
    // A profile-specific orphan needs ≥2 profiles to exist (the "satisfiable elsewhere" test).
    let orphans = if all_profiles.len() >= 2 {
        detect_profile_orphans(g, order, &rule_profiles, &all_profiles)
    } else {
        Vec::new()
    };

    println!(
        "grammar lint: '{}' ({} rules) — left_recursive={} (informational, handled by PGEN), non_terminating={} (error), ordered_choice_shadowing={} (error), always_succeeds_alternatives={} (note), unreachable_rules={} (error), undefined_references={} (error), unbound_fact_kinds={} (error), nullable_repetition={} (warning), profile_orphans={} (error; profiles={:?})",
        grammar.grammar_name,
        g.len(),
        lr.len(),
        nonterm.len(),
        shadow.len(),
        always_notes.len(),
        unreachable.len(),
        undefined_refs.len(),
        unbound_facts.len(),
        nullrep.len(),
        orphans.len(),
        all_profiles
    );
    // QUANT-PLUS-ITER.2: name the resolved entry rule, and say whether it was
    // DECLARED (`@entry: true`) or fell out of file position. Until this landed no
    // surface at default verbosity reported a grammar's start symbol at all, so a
    // helper rule written above the intended entry silently re-rooted the grammar
    // while every counter above still read 0 (measured, `QUANT-PLUS-ITER.1`).
    let declared_entry = grammar.annotations.as_ref().and_then(|annotations| {
        pgen::ast_pipeline::semantic_runtime::compile_entry_rule(annotations)
            .ok()
            .flatten()
    });
    match (order.first(), &declared_entry) {
        (Some(entry), Some(_)) => println!(
            "  [info] entry rule '{entry}' — DECLARED via `@entry: true`"
        ),
        (Some(entry), None) => println!(
            "  [info] entry rule '{entry}' — POSITIONAL (the first rule defined; declare it with `@entry: true` to make file order irrelevant)"
        ),
        (None, _) => println!("  [error] grammar defines no rules, so it has no entry rule"),
    }
    for issue in unreachable.iter().take(40) {
        println!("  [error] {}", issue.message());
    }
    if unreachable.len() > 40 {
        println!("  [error] ... and {} more unreachable rules", unreachable.len() - 40);
    }
    for issue in undefined_refs.iter().take(40) {
        println!("  [error] {}", issue.message());
    }
    if undefined_refs.len() > 40 {
        println!(
            "  [error] ... and {} more undefined-reference findings",
            undefined_refs.len() - 40
        );
    }
    for issue in unbound_facts.iter().take(40) {
        println!("  [error] {}", issue.message());
    }
    if unbound_facts.len() > 40 {
        println!("  [error] ... and {} more unbound fact-kinds", unbound_facts.len() - 40);
    }
    for issue in lr.iter().take(10) {
        println!("  [info]  {}", issue.message());
    }
    if lr.len() > 10 {
        println!("  [info]  ... and {} more left-recursive rules", lr.len() - 10);
    }
    // GRAMMAR-WELLFORMED.A2/A2.2: every surviving shadowing reason (exact-duplicate +
    // fixed-terminal-prefix) is a SOUND, HARD-gated unreachability verdict — all authored grammars are
    // clean at 0. (The unsound `EarlierAlwaysMatches` warning was retired at A2.2; its observation is
    // now the non-verdict always-succeeds [note] printed below.)
    for issue in shadow.iter().take(40) {
        println!("  [error] {}", issue.message());
    }
    if shadow.len() > 40 {
        println!("  [error] ... and {} more shadowed (unreachable) branch findings", shadow.len() - 40);
    }
    // GRAMMAR-WELLFORMED.A2.2: the always-succeeds smell — a NON-VERDICT [note] (never gates).
    for issue in always_notes.iter().take(40) {
        println!("  [note]  {}", issue.message());
    }
    if always_notes.len() > 40 {
        println!(
            "  [note]  ... and {} more always-succeeds-alternative notes (informational — not a deadness verdict)",
            always_notes.len() - 40
        );
    }
    for issue in nullrep.iter().take(40) {
        println!("  [warn]  {}", issue.message());
    }
    if nullrep.len() > 40 {
        println!(
            "  [warn]  ... and {} more nullable-repetition findings",
            nullrep.len() - 40
        );
    }
    // ANNOTATION-COMPOSITION.6: profile orphans are now a HARD failure (the grammar was
    // remediated to 0). A @profiles orphan is a real grammar defect (present-but-unsatisfiable
    // under an edition); locking it at 0 stops regressions. Grammars with < 2 profiles never
    // produce orphans (the detector is skipped), so this only binds the SV grammar.
    for issue in orphans.iter().take(40) {
        println!("  [error] {}", issue.message());
    }
    if orphans.len() > 40 {
        println!(
            "  [error] ... and {} more profile-orphan findings",
            orphans.len() - 40
        );
    }
    for issue in &nonterm {
        println!("  [error] {}", issue.message());
    }

    if nonterm.is_empty()
        && orphans.is_empty()
        && shadow.is_empty()
        && unreachable.is_empty()
        && undefined_refs.is_empty()
        && unbound_facts.is_empty()
    {
        Ok(())
    } else {
        let mut problems = Vec::new();
        if !nonterm.is_empty() {
            problems.push(format!("{} non-terminating rule(s)", nonterm.len()));
        }
        if !orphans.is_empty() {
            problems.push(format!("{} profile-orphan rule(s)", orphans.len()));
        }
        // GRAMMAR-WELLFORMED.A1a: a shadowed ordered-choice alternative is an UNREACHABLE
        // (dead) branch — a well-formedness defect (PEG ordered-choice hygiene; the branch-level
        // analogue of an unreachable rule). Every surviving shadowing reason (exact-dup +
        // fixed-prefix) is a SOUND HARD gate (all grammars clean at 0); the retired unsound
        // always-succeeds heuristic is a non-verdict [note] above, never gated (A2.2).
        if !shadow.is_empty() {
            problems.push(format!("{} shadowed (unreachable) branch(es)", shadow.len()));
        }
        // GRAMMAR-WELLFORMED.A1b: a defined-but-unreachable rule is a dead rule (Hopcroft–Ullman
        // "no useless symbols" — the reachable half). Hard failure.
        if !unreachable.is_empty() {
            problems.push(format!("{} unreachable rule(s)", unreachable.len()));
        }
        // UNDEFINED-REF-DIAGNOSTICS.2 (F6): a referenced-but-undefined rule compiles into a
        // never-matching stub — every referencing production is dead. Hard failure.
        if !undefined_refs.is_empty() {
            problems.push(format!("{} undefined reference(s)", undefined_refs.len()));
        }
        // GRAMMAR-WELLFORMED.F1: a @predicate consulting a fact-kind nothing emits = binding-
        // before-use (the fact can never be established). Hard failure.
        if !unbound_facts.is_empty() {
            problems.push(format!("{} unbound fact-kind(s)", unbound_facts.len()));
        }
        Err(anyhow::anyhow!(
            "grammar '{}' has {}",
            grammar.grammar_name,
            problems.join(" + ")
        ))
    }
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

fn recovery_stimuli_mode_name(mode: RecoveryStimuliMode) -> &'static str {
    match mode {
        RecoveryStimuliMode::Baseline => "baseline",
        RecoveryStimuliMode::RecoveryBiased => "recovery_biased",
        RecoveryStimuliMode::NearSyncNegative => "near_sync_negative",
    }
}

fn stimuli_negative_profile_name(profile: StimuliNegativeProfile) -> &'static str {
    match profile {
        StimuliNegativeProfile::Baseline => "baseline",
        StimuliNegativeProfile::NearValidLocal => "near_valid_local",
    }
}

fn stimuli_constraint_profile_name(profile: StimuliConstraintProfile) -> &'static str {
    match profile {
        StimuliConstraintProfile::Baseline => "baseline",
        StimuliConstraintProfile::RareBranchBiased => "rare_branch_biased",
        StimuliConstraintProfile::DeepNestingBiased => "deep_nesting_biased",
    }
}

fn stimuli_mutation_mode_name(mode: StimuliMutationMode) -> &'static str {
    match mode {
        StimuliMutationMode::Baseline => "baseline",
        StimuliMutationMode::GrammarAwareLocal => "grammar_aware_local",
    }
}

fn direct_bundle_samples(samples: &[String]) -> Vec<StimuliCorpusBundleSample> {
    samples
        .iter()
        .enumerate()
        .map(|(idx, sample)| StimuliCorpusBundleSample {
            ordinal: idx + 1,
            sample: sample.clone(),
            source_seed: None,
            parseable: None,
            new_rule_hits: Vec::new(),
            new_branch_hits: Vec::new(),
            coverage_tokens: Vec::new(),
        })
        .collect()
}

fn build_stimuli_corpus_bundle(
    grammar_name: &str,
    grammar_profile: Option<&str>,
    generation_surface: &str,
    corpus_origin_mode: &str,
    entry_rule: &str,
    requested_sample_count: usize,
    requested_seed: Option<u64>,
    effective_seed: Option<u64>,
    config: &StimuliConfig,
    validate_parseability: bool,
    parseability_max_attempts: Option<usize>,
    coverage_guided_fuzz_rounds: usize,
    coverage_guided_fuzz_seed_start: Option<u64>,
    samples: Vec<StimuliCorpusBundleSample>,
    coverage: &StimuliCoverageMetrics,
    parseability_summary: Option<&ParseabilitySummary>,
    target_drive_validation: Option<&TargetDriveParseabilityTelemetry>,
    parseability_counterexamples: &[ParseabilityCounterexample],
    coverage_guided_fuzz_replay: Option<&CoverageGuidedFuzzReplayReport>,
) -> StimuliCorpusBundle {
    StimuliCorpusBundle {
        pgen_stimuli_corpus_bundle_version: STIMULI_CORPUS_BUNDLE_VERSION,
        grammar_name: grammar_name.to_string(),
        grammar_profile: grammar_profile.map(str::to_string),
        generation_surface: generation_surface.to_string(),
        corpus_origin_mode: corpus_origin_mode.to_string(),
        entry_rule: entry_rule.to_string(),
        requested_sample_count,
        generated_sample_count: samples.len(),
        generation_config: StimuliCorpusGenerationConfig {
            requested_seed,
            effective_seed,
            deterministic_replay_possible: effective_seed.is_some(),
            max_depth: config.max_depth,
            max_repeat: config.max_repeat,
            max_rule_visits: config.max_rule_visits,
            target_pending_frontier_extra_stagnation: config
                .target_pending_frontier_extra_stagnation,
            target_generation_timeout_ms: config.target_generation_timeout_ms,
            target_helper_generation_timeout_ms: config.target_helper_generation_timeout_ms,
            recovery_stimuli_mode: recovery_stimuli_mode_name(config.recovery_mode).to_string(),
            stimuli_negative_profile: stimuli_negative_profile_name(config.negative_profile)
                .to_string(),
            stimuli_constraint_profile: stimuli_constraint_profile_name(config.constraint_profile)
                .to_string(),
            stimuli_mutation_mode: stimuli_mutation_mode_name(config.mutation_mode).to_string(),
            enforce_word_boundary_spacing: config.enforce_word_boundary_spacing,
            validate_parseability,
            parseability_max_attempts,
            coverage_guided_fuzz_rounds,
            coverage_guided_fuzz_seed_start,
        },
        samples,
        coverage: coverage.clone(),
        parseability_summary: parseability_summary.cloned(),
        target_drive_validation: target_drive_validation.cloned(),
        parseability_counterexamples: parseability_counterexamples.to_vec(),
        coverage_guided_fuzz_replay: coverage_guided_fuzz_replay.cloned(),
    }
}

fn write_stimuli_corpus_bundle(path: &str, bundle: &StimuliCorpusBundle) -> Result<()> {
    ensure_parent_dir_exists(path)?;
    let json = encode_canonical_json(bundle, true)?;
    std::fs::write(path, json)?;
    Ok(())
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

fn parse_stimuli_negative_profile(value: &str) -> Result<StimuliNegativeProfile> {
    match value.trim().to_ascii_lowercase().as_str() {
        "baseline" => Ok(StimuliNegativeProfile::Baseline),
        "near_valid_local" => Ok(StimuliNegativeProfile::NearValidLocal),
        other => Err(anyhow::anyhow!(
            "Unsupported stimuli negative profile '{}'. Supported values: baseline, near_valid_local",
            other
        )),
    }
}

fn parse_stimuli_constraint_profile(value: &str) -> Result<StimuliConstraintProfile> {
    match value.trim().to_ascii_lowercase().as_str() {
        "baseline" => Ok(StimuliConstraintProfile::Baseline),
        "rare_branch_biased" => Ok(StimuliConstraintProfile::RareBranchBiased),
        "deep_nesting_biased" => Ok(StimuliConstraintProfile::DeepNestingBiased),
        other => Err(anyhow::anyhow!(
            "Unsupported stimuli constraint profile '{}'. Supported values: baseline, rare_branch_biased, deep_nesting_biased",
            other
        )),
    }
}

fn parse_stimuli_mutation_mode(value: &str) -> Result<StimuliMutationMode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "baseline" => Ok(StimuliMutationMode::Baseline),
        "grammar_aware_local" => Ok(StimuliMutationMode::GrammarAwareLocal),
        other => Err(anyhow::anyhow!(
            "Unsupported stimuli mutation mode '{}'. Supported values: baseline, grammar_aware_local",
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
    entry_context: Option<&TargetDriveFilterContext<'_>>,
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
        primary_entry_rule: entry_context.map(|context| context.primary_entry_rule.to_string()),
        generation_entry_rule: entry_context
            .map(|context| context.generation_entry_rule.to_string()),
        entry_mode: entry_context.map(|context| {
            if context.is_primary_entry {
                "primary".to_string()
            } else {
                "alternate".to_string()
            }
        }),
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
    while try_structural_shrink_pass(&mut candidate, &mut still_fails)? {}
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
                while try_structural_shrink_pass(&mut candidate, &mut still_fails)? {}
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

fn try_structural_shrink_pass<F>(candidate: &mut String, still_fails: &mut F) -> Result<bool>
where
    F: FnMut(&str) -> Result<bool>,
{
    for trial in structural_shrink_candidates(candidate) {
        if trial.chars().count() < candidate.chars().count() && still_fails(&trial)? {
            *candidate = trial;
            return Ok(true);
        }
    }
    Ok(false)
}

fn structural_shrink_candidates(input: &str) -> Vec<String> {
    let mut stack = Vec::<(char, usize)>::new();
    let mut spans = Vec::<(usize, usize, usize)>::new();

    for (byte_idx, ch) in input.char_indices() {
        match ch {
            '(' | '[' | '{' => stack.push((ch, byte_idx)),
            ')' | ']' | '}' => {
                if let Some(stack_idx) = stack
                    .iter()
                    .rposition(|(open, _)| delimiters_match(*open, ch))
                {
                    let (_, open_byte) = stack.remove(stack_idx);
                    spans.push((open_byte, byte_idx, ch.len_utf8()));
                }
            }
            _ => {}
        }
    }

    spans.sort_by_key(|(open_byte, close_byte, close_width)| close_byte + close_width - open_byte);

    let mut candidates = Vec::new();
    let mut seen = HashSet::new();
    for (open_byte, close_byte, close_width) in spans {
        let open_width = input[open_byte..]
            .chars()
            .next()
            .map(char::len_utf8)
            .unwrap_or(0);
        if open_width == 0 {
            continue;
        }

        let open_end = open_byte + open_width;
        let close_end = close_byte + close_width;
        let prefix = &input[..open_byte];
        let open = &input[open_byte..open_end];
        let close = &input[close_byte..close_end];
        let interior = &input[open_end..close_byte];
        let suffix = &input[close_end..];

        if !interior.is_empty() {
            push_unique_structural_candidate(
                input,
                &mut candidates,
                &mut seen,
                format!("{}{}{}{}", prefix, open, close, suffix),
            );
            push_unique_structural_candidate(
                input,
                &mut candidates,
                &mut seen,
                format!("{}{}{}", prefix, interior, suffix),
            );
        }
        push_unique_structural_candidate(
            input,
            &mut candidates,
            &mut seen,
            format!("{}{}", prefix, suffix),
        );
    }

    candidates
}

fn delimiters_match(open: char, close: char) -> bool {
    matches!((open, close), ('(', ')') | ('[', ']') | ('{', '}'))
}

fn push_unique_structural_candidate(
    original: &str,
    candidates: &mut Vec<String>,
    seen: &mut HashSet<String>,
    candidate: String,
) {
    if candidate.len() < original.len() && seen.insert(candidate.clone()) {
        candidates.push(candidate);
    }
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
            minimized_bundle_samples: Vec::new(),
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
                    seed,
                    parseable,
                    new_rule_hits: new_rule_hits.clone(),
                    new_branch_hits: new_branch_hits.clone(),
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
    let minimized_bundle_samples = minimized_indices
        .into_iter()
        .enumerate()
        .map(|(ordinal, idx)| {
            let candidate = &corpus_candidates[idx];
            let mut coverage_tokens = candidate
                .coverage_tokens
                .iter()
                .cloned()
                .collect::<Vec<_>>();
            coverage_tokens.sort();
            StimuliCorpusBundleSample {
                ordinal: ordinal + 1,
                sample: candidate.sample.clone(),
                source_seed: Some(candidate.seed),
                parseable: candidate.parseable,
                new_rule_hits: candidate.new_rule_hits.clone(),
                new_branch_hits: candidate.new_branch_hits.clone(),
                coverage_tokens,
            }
        })
        .collect::<Vec<_>>();
    let minimized_samples = minimized_bundle_samples
        .iter()
        .map(|sample| sample.sample.clone())
        .collect::<Vec<_>>();
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
        minimized_bundle_samples,
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

fn ensure_generated_parseability_entry_rule_supported(
    rule_order: &[String],
    entry_rule: &str,
) -> Result<()> {
    let full_entry_rule = rule_order.first().ok_or_else(|| {
        anyhow::anyhow!("No entry rule available for generated parseability validation")
    })?;
    if entry_rule != full_entry_rule {
        return Err(anyhow::anyhow!(
            "Generated parseability validation currently supports only the grammar's full entry rule ('{}'), but '{}' was requested. Use the full entry rule when passing --validate-parseability, or omit --validate-parseability for subrule coverage triage.",
            full_entry_rule,
            entry_rule
        ));
    }
    Ok(())
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
                    None,
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
                        None,
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
        StimuliCorpusBundleSample, StimuliCoverageMetrics, TargetDriveParseabilityTelemetry,
        build_stimuli_corpus_bundle, canonicalize_json_value, coverage_branch_hit_delta,
        default_parser_output_path, default_stimuli_module_output_path, direct_bundle_samples,
        ensure_generated_parseability_entry_rule_supported, extract_parse_error_position,
        generate_stimuli_module_source, is_ebnf_input_path, load_grammar_bundle_from_json_value,
        maybe_dump_generation_ast, minimize_failing_input, minimize_fuzz_corpus_cases,
        parse_error_context_excerpt, parse_error_line_column, parse_error_line_excerpt,
        parse_recovery_stimuli_mode, parse_stimuli_constraint_profile, parse_stimuli_mutation_mode,
        parse_stimuli_negative_profile, resolve_parseability_max_attempts,
        resolve_stimuli_module_seed, structural_shrink_candidates,
        supported_generated_parseability_grammars, supports_generated_parseability,
        write_parseability_report,
    };
    use pgen::ast_pipeline::stimuli_generator::{
        BranchCoverageGroup, RecoveryStimuliMode, StimuliConfig, StimuliConstraintProfile,
        StimuliMutationMode, StimuliNegativeProfile, TargetDriveValidationSummary,
    };
    use pgen::ast_pipeline::{ASTNode, ASTValue, PipelineConfig, RustASTPipeline, TokenValue};
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
    fn generated_parseability_accepts_full_entry_rule() {
        let rule_order = vec!["vhdl_file".to_string(), "actual_part".to_string()];
        ensure_generated_parseability_entry_rule_supported(&rule_order, "vhdl_file")
            .expect("full entry rule should be accepted");
    }

    #[test]
    fn generated_parseability_rejects_subrule_entry_validation() {
        let rule_order = vec!["vhdl_file".to_string(), "actual_part".to_string()];
        let err = ensure_generated_parseability_entry_rule_supported(&rule_order, "actual_part")
            .expect_err("subrule parseability validation should be rejected");
        assert!(
            err.to_string()
                .contains("supports only the grammar's full entry rule")
        );
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
                target_timeout_errors: 1,
                helper_timeout_errors: 2,
            });

        assert_eq!(telemetry.primary_entry_attempts, 4);
        assert_eq!(telemetry.primary_entry_accepted_outputs, 3);
        assert_eq!(telemetry.primary_entry_rejected_outputs, 1);
        assert!((telemetry.primary_entry_acceptance_rate_percent - 75.0).abs() < f64::EPSILON);
        assert_eq!(telemetry.alternate_entry_attempts, 5);
        assert_eq!(telemetry.alternate_entry_accepted_outputs, 2);
        assert_eq!(telemetry.alternate_entry_rejected_outputs, 3);
        assert!((telemetry.alternate_entry_acceptance_rate_percent - 40.0).abs() < f64::EPSILON);
        assert_eq!(telemetry.target_timeout_errors, 1);
        assert_eq!(telemetry.helper_timeout_errors, 2);
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
            target_timeout_errors: 4,
            helper_timeout_errors: 5,
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
        assert_eq!(
            report_value["target_drive_validation"]["target_timeout_errors"].as_u64(),
            Some(4)
        );
        assert_eq!(
            report_value["target_drive_validation"]["helper_timeout_errors"].as_u64(),
            Some(5)
        );

        std::fs::remove_file(&path).expect("temporary parseability report should be removable");
    }

    #[test]
    fn stimuli_corpus_bundle_preserves_target_drive_timeout_telemetry() {
        let config = StimuliConfig::default();
        let coverage = StimuliCoverageMetrics {
            grammar_name: "regex".to_string(),
            total_rules: 0,
            total_branch_groups: 0,
            total_branches: 0,
            sample_attempts: 0,
            sample_successes: 0,
            sample_errors: 0,
            rule_success_hits: HashMap::new(),
            branch_groups: HashMap::new(),
        };
        let validation = TargetDriveParseabilityTelemetry {
            primary_entry_attempts: 4,
            primary_entry_accepted_outputs: 3,
            primary_entry_rejected_outputs: 1,
            primary_entry_acceptance_rate_percent: 75.0,
            alternate_entry_attempts: 9,
            alternate_entry_accepted_outputs: 2,
            alternate_entry_rejected_outputs: 7,
            alternate_entry_acceptance_rate_percent: 200.0 / 9.0,
            target_timeout_errors: 2,
            helper_timeout_errors: 3,
        };

        let bundle = build_stimuli_corpus_bundle(
            "regex",
            Some("regex_default"),
            "generate_stimuli",
            "target_report",
            "regex",
            1,
            Some(1),
            Some(1),
            &config,
            true,
            Some(50),
            0,
            None,
            direct_bundle_samples(&["a".to_string()]),
            &coverage,
            Some(&ParseabilitySummary::from_filter(4, 3, 1)),
            Some(&validation),
            &[],
            None,
        );

        assert_eq!(
            bundle
                .target_drive_validation
                .as_ref()
                .expect("target-drive validation should be preserved")
                .target_timeout_errors,
            2
        );
        assert_eq!(
            bundle
                .target_drive_validation
                .as_ref()
                .expect("target-drive validation should be preserved")
                .helper_timeout_errors,
            3
        );
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
            primary_entry_rule: Some("systemverilog_preprocessor_file".to_string()),
            generation_entry_rule: Some("pp_item".to_string()),
            entry_mode: Some("alternate".to_string()),
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
            report_value["counterexamples"][0]["primary_entry_rule"].as_str(),
            Some("systemverilog_preprocessor_file")
        );
        assert_eq!(
            report_value["counterexamples"][0]["generation_entry_rule"].as_str(),
            Some("pp_item")
        );
        assert_eq!(
            report_value["counterexamples"][0]["entry_mode"].as_str(),
            Some("alternate")
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
                seed: 1,
                parseable: Some(true),
                new_rule_hits: vec!["a".to_string()],
                new_branch_hits: Vec::new(),
                coverage_tokens: c0_tokens,
            },
            FuzzCorpusCandidate {
                sample: "beta".to_string(),
                seed: 2,
                parseable: Some(true),
                new_rule_hits: vec!["b".to_string()],
                new_branch_hits: Vec::new(),
                coverage_tokens: c1_tokens,
            },
            FuzzCorpusCandidate {
                sample: "both".to_string(),
                seed: 3,
                parseable: Some(true),
                new_rule_hits: vec!["a".to_string(), "b".to_string()],
                new_branch_hits: Vec::new(),
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
                seed: 1,
                parseable: None,
                new_rule_hits: Vec::new(),
                new_branch_hits: Vec::new(),
                coverage_tokens: HashSet::new(),
            },
            FuzzCorpusCandidate {
                sample: "x".to_string(),
                seed: 2,
                parseable: None,
                new_rule_hits: Vec::new(),
                new_branch_hits: Vec::new(),
                coverage_tokens: HashSet::new(),
            },
            FuzzCorpusCandidate {
                sample: "mid".to_string(),
                seed: 3,
                parseable: None,
                new_rule_hits: Vec::new(),
                new_branch_hits: Vec::new(),
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
        assert_eq!(json["metadata"]["format"], "transformed_ast");
        assert_eq!(json["metadata"]["pipeline_stage"], "generation_input_ast");
        assert_eq!(json["metadata"]["annotations"], serde_json::Value::Null);

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
    fn generation_ast_dump_round_trips_via_loader() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "root".to_string(),
            ASTNode::Atom {
                value: ASTValue::Token(vec![TokenValue::String("kw_root".to_string())]),
            },
        );
        let grammar = LoadedGrammar {
            grammar_name: "demo_roundtrip".to_string(),
            grammar_tree,
            rule_order: vec!["root".to_string()],
            annotations: None,
        };
        let dump_path = unique_temp_path("gen_ast_roundtrip.json");
        let dump_path_str = dump_path.to_string_lossy().to_string();
        maybe_dump_generation_ast(&grammar, Some(dump_path_str.as_str()), false, None)
            .expect("dump succeeds");

        let raw = std::fs::read_to_string(&dump_path).expect("read dump");
        let value: serde_json::Value = serde_json::from_str(&raw).expect("json parse");
        let mut pipeline = RustASTPipeline::new(PipelineConfig::default());
        let loaded = load_grammar_bundle_from_json_value(value, &mut pipeline).expect("load dump");
        assert_eq!(loaded.grammar_name, "demo_roundtrip");
        assert_eq!(loaded.rule_order, vec!["root".to_string()]);
        assert!(loaded.grammar_tree.contains_key("root"));

        let _ = std::fs::remove_file(dump_path);
    }

    #[test]
    fn legacy_generation_ast_dump_shape_is_still_loadable() {
        let mut grammar_tree = HashMap::new();
        grammar_tree.insert(
            "root".to_string(),
            ASTNode::Atom {
                value: ASTValue::Token(vec![TokenValue::String("kw_root".to_string())]),
            },
        );
        let grammar = LoadedGrammar {
            grammar_name: "legacy_demo".to_string(),
            grammar_tree,
            rule_order: vec!["root".to_string()],
            annotations: None,
        };
        let dump_path = unique_temp_path("gen_ast_legacy.json");
        let dump_path_str = dump_path.to_string_lossy().to_string();
        maybe_dump_generation_ast(&grammar, Some(dump_path_str.as_str()), false, None)
            .expect("dump succeeds");

        let raw = std::fs::read_to_string(&dump_path).expect("read dump");
        let mut legacy: serde_json::Value = serde_json::from_str(&raw).expect("json parse");
        legacy
            .as_object_mut()
            .expect("generation dump object")
            .remove("metadata");
        let mut pipeline = RustASTPipeline::new(PipelineConfig::default());
        let loaded =
            load_grammar_bundle_from_json_value(legacy, &mut pipeline).expect("load legacy dump");
        assert_eq!(loaded.grammar_name, "legacy_demo");
        assert_eq!(loaded.rule_order, vec!["root".to_string()]);
        assert!(loaded.grammar_tree.contains_key("root"));

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
                failure_reasons: vec![HashMap::new(), HashMap::new()],
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
                failure_reasons: vec![HashMap::new(), HashMap::new()],
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
    fn structural_shrink_candidates_collapse_balanced_delimiters() {
        let candidates = structural_shrink_candidates("call(alpha, beta)");
        assert!(candidates.contains(&"call()".to_string()));
        assert!(candidates.contains(&"callalpha, beta".to_string()));
        assert!(candidates.contains(&"call".to_string()));
    }

    #[test]
    fn failing_input_minimizer_collapses_delimited_payloads() {
        let minimized = minimize_failing_input("call(alpha, beta)", |candidate| {
            Ok(candidate.starts_with("call(") && candidate.ends_with(')'))
        })
        .expect("minimizer should succeed");
        assert_eq!(minimized, "call()");
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
    fn stimuli_corpus_bundle_tracks_module_effective_seed_and_direct_samples() {
        let config = StimuliConfig {
            seed: None,
            max_depth: 12,
            max_repeat: 3,
            max_rule_visits: 12,
            target_generation_timeout_ms: 250,
            recovery_mode: RecoveryStimuliMode::Baseline,
            mutation_mode: StimuliMutationMode::Baseline,
            constraint_profile: StimuliConstraintProfile::Baseline,
            negative_profile: StimuliNegativeProfile::Baseline,
            enforce_word_boundary_spacing: false,
            ..StimuliConfig::default()
        };
        let coverage = StimuliCoverageMetrics {
            grammar_name: "regex".to_string(),
            total_rules: 0,
            total_branch_groups: 0,
            total_branches: 0,
            sample_attempts: 0,
            sample_successes: 0,
            sample_errors: 0,
            rule_success_hits: HashMap::new(),
            branch_groups: HashMap::new(),
        };
        let bundle = build_stimuli_corpus_bundle(
            "regex",
            None,
            "generate_stimuli_module",
            "direct_generation",
            "regex",
            2,
            None,
            Some(1),
            &config,
            false,
            None,
            0,
            None,
            direct_bundle_samples(&["a".to_string(), "b".to_string()]),
            &coverage,
            None,
            None,
            &[],
            None,
        );
        assert_eq!(bundle.generation_surface, "generate_stimuli_module");
        assert_eq!(bundle.corpus_origin_mode, "direct_generation");
        assert_eq!(bundle.generated_sample_count, 2);
        assert_eq!(bundle.generation_config.requested_seed, None);
        assert_eq!(bundle.generation_config.effective_seed, Some(1));
        assert!(bundle.generation_config.deterministic_replay_possible);
        assert_eq!(
            bundle
                .generation_config
                .target_pending_frontier_extra_stagnation,
            8
        );
        assert_eq!(bundle.generation_config.target_generation_timeout_ms, 250);
        assert_eq!(
            bundle.generation_config.target_helper_generation_timeout_ms,
            1000
        );
        assert_eq!(bundle.samples[0].ordinal, 1);
        assert_eq!(bundle.samples[0].sample, "a");
        assert_eq!(bundle.samples[1].ordinal, 2);
        assert_eq!(bundle.samples[1].sample, "b");
    }

    #[test]
    fn stimuli_corpus_bundle_preserves_fuzz_promotion_metadata() {
        let config = StimuliConfig {
            seed: Some(7),
            max_depth: 10,
            max_repeat: 2,
            max_rule_visits: 10,
            target_generation_timeout_ms: 125,
            recovery_mode: RecoveryStimuliMode::Baseline,
            mutation_mode: StimuliMutationMode::GrammarAwareLocal,
            constraint_profile: StimuliConstraintProfile::RareBranchBiased,
            negative_profile: StimuliNegativeProfile::NearValidLocal,
            enforce_word_boundary_spacing: false,
            ..StimuliConfig::default()
        };
        let coverage = StimuliCoverageMetrics {
            grammar_name: "regex".to_string(),
            total_rules: 0,
            total_branch_groups: 0,
            total_branches: 0,
            sample_attempts: 0,
            sample_successes: 0,
            sample_errors: 0,
            rule_success_hits: HashMap::new(),
            branch_groups: HashMap::new(),
        };
        let bundle = build_stimuli_corpus_bundle(
            "regex",
            Some("regex_default"),
            "generate_stimuli",
            "coverage_guided_fuzz_minimized",
            "regex",
            4,
            Some(7),
            Some(7),
            &config,
            true,
            Some(200),
            6,
            Some(7),
            vec![StimuliCorpusBundleSample {
                ordinal: 1,
                sample: "(a|b)".to_string(),
                source_seed: Some(9),
                parseable: Some(true),
                new_rule_hits: vec!["alternation".to_string()],
                new_branch_hits: vec!["branch::regex::alt#1".to_string()],
                coverage_tokens: vec![
                    "branch::regex::alt#1".to_string(),
                    "rule::alternation".to_string(),
                ],
            }],
            &coverage,
            Some(&ParseabilitySummary::from_filter(6, 4, 2)),
            None,
            &[],
            None,
        );
        assert_eq!(bundle.corpus_origin_mode, "coverage_guided_fuzz_minimized");
        assert_eq!(bundle.generation_config.coverage_guided_fuzz_rounds, 6);
        assert_eq!(
            bundle
                .generation_config
                .target_pending_frontier_extra_stagnation,
            8
        );
        assert_eq!(bundle.generation_config.target_generation_timeout_ms, 125);
        assert_eq!(
            bundle.generation_config.target_helper_generation_timeout_ms,
            1000
        );
        assert_eq!(bundle.samples[0].source_seed, Some(9));
        assert_eq!(bundle.samples[0].new_rule_hits, vec!["alternation"]);
        assert_eq!(
            bundle.samples[0].coverage_tokens,
            vec!["branch::regex::alt#1", "rule::alternation"]
        );
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

    #[test]
    fn parses_stimuli_mutation_mode_values() {
        assert!(matches!(
            parse_stimuli_mutation_mode("baseline").expect("baseline mutation mode should parse"),
            StimuliMutationMode::Baseline
        ));
        assert!(matches!(
            parse_stimuli_mutation_mode("grammar_aware_local")
                .expect("grammar-aware mutation mode should parse"),
            StimuliMutationMode::GrammarAwareLocal
        ));
    }

    #[test]
    fn rejects_unknown_stimuli_mutation_mode_values() {
        let err = parse_stimuli_mutation_mode("mutate_everything")
            .expect_err("unknown mutation mode must be rejected");
        let message = err.to_string();
        assert!(
            message.contains("Unsupported stimuli mutation mode"),
            "unexpected mutation mode parse error message: {}",
            message
        );
    }

    #[test]
    fn parses_stimuli_constraint_profile_values() {
        assert!(matches!(
            parse_stimuli_constraint_profile("baseline")
                .expect("baseline constraint profile should parse"),
            StimuliConstraintProfile::Baseline
        ));
        assert!(matches!(
            parse_stimuli_constraint_profile("rare_branch_biased")
                .expect("rare-branch-biased profile should parse"),
            StimuliConstraintProfile::RareBranchBiased
        ));
        assert!(matches!(
            parse_stimuli_constraint_profile("deep_nesting_biased")
                .expect("deep-nesting-biased profile should parse"),
            StimuliConstraintProfile::DeepNestingBiased
        ));
    }

    #[test]
    fn rejects_unknown_stimuli_constraint_profile_values() {
        let err = parse_stimuli_constraint_profile("constraint_chaos")
            .expect_err("unknown constraint profile must be rejected");
        let message = err.to_string();
        assert!(
            message.contains("Unsupported stimuli constraint profile"),
            "unexpected constraint profile parse error message: {}",
            message
        );
    }

    #[test]
    fn parses_stimuli_negative_profile_values() {
        assert!(matches!(
            parse_stimuli_negative_profile("baseline")
                .expect("baseline negative profile should parse"),
            StimuliNegativeProfile::Baseline
        ));
        assert!(matches!(
            parse_stimuli_negative_profile("near_valid_local")
                .expect("near-valid-local negative profile should parse"),
            StimuliNegativeProfile::NearValidLocal
        ));
    }

    #[test]
    fn rejects_unknown_stimuli_negative_profile_values() {
        let err = parse_stimuli_negative_profile("chaotic_negative")
            .expect_err("unknown negative profile must be rejected");
        let message = err.to_string();
        assert!(
            message.contains("Unsupported stimuli negative profile"),
            "unexpected negative profile parse error message: {}",
            message
        );
    }
}
