use anyhow::{Context, Result};
use clap::Parser;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

pub mod ast_pipeline {
    pub use pgen::ast_pipeline::*;
}

#[allow(dead_code)]
mod generated_ebnf {
    include!(env!("PGEN_EBNF_PARSER_PATH_RESOLVED_BIN"));
}

use pgen::ebnf_envelope_differential as differential;

use ast_pipeline::{ParseContent, ParseError};
use ast_pipeline::{
    configure_trace_output, resolve_trace_verbosity, runtime_logger_box, set_global_trace_verbosity,
};
use generated_ebnf::EbnfParser;

#[derive(Parser, Debug)]
#[command(name = "ebnf_dual_run_diff")]
#[command(
    about = "Parse EBNF text with generated/ebnf.rs and emit structured parse/full-parse diagnostics."
)]
struct Args {
    /// Input EBNF grammar file
    #[arg(long)]
    input: PathBuf,

    /// Output JSON report path
    #[arg(long)]
    output: PathBuf,

    /// LANG-CAPABILITY-AUDIT.10.6 part 2 — also write the generated meta-parser's TYPED AST
    /// (the `serde_json` serialization of the `parse_full_grammar_file()` `ParseNode`) to this
    /// path. This is arm 2 of the frontend<->meta-parser envelope differential; arm 1 is
    /// `ast_pipeline <grammar> --emit-raw-ast-json`.
    #[arg(long)]
    emit_ast_json: Option<PathBuf>,

    /// LANG-CAPABILITY-AUDIT.10.6 part 2 — run the frontend<->meta-parser RAW-AST ENVELOPE
    /// differential and write its report here. Runs BOTH arms in this one process (the
    /// hand-written frontend via `pgen::ebnf_frontend`, the generated meta-parser via the
    /// `include!`d `generated/ebnf.rs`), so the two sides can never be compared across stale
    /// artifacts. Refuses to emit a report if either ground-truth control fails.
    #[arg(long)]
    envelope_differential: Option<PathBuf>,

    /// Trace verbosity: none, low, medium, high, debug
    #[arg(long, value_parser = ["none", "low", "medium", "high", "debug"])]
    verbosity: Option<String>,

    /// Route trace output to a file (defaults to trace.log when flag is provided without a value)
    #[arg(long, num_args = 0..=1, default_missing_value = "trace.log")]
    trace_log_file: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ParseAttempt {
    ok: bool,
    root_rule: Option<String>,
    content_kind: Option<String>,
    span_start: Option<usize>,
    span_end: Option<usize>,
    error_kind: Option<String>,
    error_message: Option<String>,
    error_position: Option<usize>,
    error_context: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct EbnfDualRunReport {
    input_path: String,
    input_bytes: usize,
    parse: ParseAttempt,
    parse_full: ParseAttempt,
    unconsumed_start: Option<usize>,
    unconsumed_context: Option<String>,
}

fn parse_content_kind(content: &ParseContent<'_>) -> &'static str {
    match content {
        ParseContent::Terminal(_) => "terminal",
        ParseContent::TransformedTerminal(_) => "transformed_terminal",
        ParseContent::Sequence(_) => "sequence",
        ParseContent::Alternative(_) => "alternative",
        ParseContent::Quantified(_, _) => "quantified",
        ParseContent::Shaped(_) => "json_object",
    }
}

fn parse_error_fields(error: &ParseError) -> (String, String, Option<usize>) {
    match error {
        ParseError::UnexpectedEof { position } => (
            "UnexpectedEof".to_string(),
            "Unexpected EOF".to_string(),
            Some(*position),
        ),
        ParseError::UnexpectedToken {
            expected,
            found,
            position,
        } => (
            "UnexpectedToken".to_string(),
            format!("Expected '{}', found '{}'", expected, found),
            Some(*position),
        ),
        ParseError::InvalidSyntax { message, position } => (
            "InvalidSyntax".to_string(),
            (*message).to_string(),
            Some(*position),
        ),
        ParseError::Backtrack { position } => (
            "Backtrack".to_string(),
            "Backtrack".to_string(),
            Some(*position),
        ),
        ParseError::RecursionDepthExceeded { position, depth } => (
            "RecursionDepthExceeded".to_string(),
            format!("Recursion depth exceeded: depth={}", depth),
            Some(*position),
        ),
        ParseError::ContextualError {
            message, position, ..
        } => (
            "ContextualError".to_string(),
            message.clone(),
            Some(*position),
        ),
    }
}

fn nearest_char_boundary_backward(input: &str, mut idx: usize) -> usize {
    if idx > input.len() {
        idx = input.len();
    }
    while idx > 0 && !input.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

fn nearest_char_boundary_forward(input: &str, mut idx: usize) -> usize {
    if idx > input.len() {
        idx = input.len();
    }
    while idx < input.len() && !input.is_char_boundary(idx) {
        idx += 1;
    }
    idx
}

fn snippet_at(input: &str, position: usize, radius: usize) -> String {
    if input.is_empty() {
        return String::new();
    }
    let start = nearest_char_boundary_backward(input, position.saturating_sub(radius));
    let end = nearest_char_boundary_forward(input, (position + radius).min(input.len()));
    input[start..end]
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn attempt_parse(input: &str, full: bool) -> ParseAttempt {
    let node_arena = pgen::ast_pipeline::NodeArena::new();
    let mut parser = EbnfParser::new(
        input,
        &node_arena,
        runtime_logger_box("generated.ebnf_dual_run_diff"),
    );
    let result = if full {
        parser.parse_full_grammar_file()
    } else {
        parser.parse()
    };

    match result {
        Ok(node) => ParseAttempt {
            ok: true,
            root_rule: Some(node.rule_name.to_string()),
            content_kind: Some(parse_content_kind(&node.content).to_string()),
            span_start: Some(node.span.start as usize),
            span_end: Some(node.span.end as usize),
            error_kind: None,
            error_message: None,
            error_position: None,
            error_context: None,
        },
        Err(error) => {
            let (kind, message, position) = parse_error_fields(&error);
            ParseAttempt {
                ok: false,
                root_rule: None,
                content_kind: None,
                span_start: None,
                span_end: None,
                error_kind: Some(kind),
                error_message: Some(message),
                error_position: position,
                error_context: position.map(|pos| snippet_at(input, pos, 48)),
            }
        }
    }
}

/// LANG-CAPABILITY-AUDIT.10.6 part 2 — serialize the generated meta-parser's typed AST.
///
/// Deliberately a SECOND parse rather than a value threaded out of [`attempt_parse`]: every
/// `ParseNode` borrows the [`NodeArena`] it was allocated in, so the tree cannot outlive the
/// arena's scope. Serializing inside that scope is what keeps this a plain function instead of
/// a lifetime-carrying return. The cost is one extra parse of a grammar file, which is
/// microseconds — and this flag is opt-in.
fn emit_meta_parser_ast_json(input: &str) -> Result<serde_json::Value> {
    let node_arena = pgen::ast_pipeline::NodeArena::new();
    let mut parser = EbnfParser::new(
        input,
        &node_arena,
        runtime_logger_box("generated.ebnf_dual_run_diff"),
    );
    let node = parser.parse_full_grammar_file().map_err(|error| {
        anyhow::anyhow!("generated meta-parser rejected the input: {:?}", error)
    })?;
    serde_json::to_value(&node).context("failed to serialize the meta-parser parse tree")
}

/// LANG-CAPABILITY-AUDIT.10.6 part 2 — run the envelope differential for one grammar.
///
/// ⭐ Both arms run HERE, in one process, from the same input string: arm 1 through
/// `pgen::ebnf_frontend` (the hand-written frontend) and arm 2 through the `include!`d
/// `generated/ebnf.rs`. That is deliberate — a differential assembled from two separately
/// produced files can compare a fresh arm against a stale one and call the result a finding.
///
/// The ground-truth controls run FIRST and a failure aborts: an instrument whose own projection
/// or differ is broken must refuse, not publish.
fn run_envelope_differential(
    grammar_path: &Path,
    grammar_name: &str,
    input: &str,
) -> Result<(
    differential::DifferentialReport,
    differential::GroundTruthOutcome,
)> {
    let control_source = differential::positive_control_grammar();
    let control_arm1 = pgen::ebnf_frontend::parse_ebnf_text_to_raw_ast_envelope(
        control_source,
        "envelope_differential_positive_control",
        None,
    )
    .context("ground-truth control: the hand-written frontend rejected the control grammar")?;
    let control_arm2 = emit_meta_parser_ast_json(control_source)
        .context("ground-truth control: the generated meta-parser rejected the control grammar")?;
    let ground_truth = differential::run_ground_truth_controls(&control_arm1, &control_arm2)
        .map_err(|reason| anyhow::anyhow!("{}", reason))?;

    // ⚠️ The FILE-based entry point, deliberately: it is the one the real pipeline uses, and it
    // resolves `include(…)` directives relative to the grammar. The text-based sibling cannot,
    // so using it here would fail the one tracked grammar that has an include and would
    // otherwise compare a resolved arm against an unresolved one.
    let arm1_envelope = pgen::ebnf_frontend::parse_ebnf_file_to_raw_ast_envelope(
        &grammar_path.display().to_string(),
    )
    .context("arm 1: the hand-written frontend failed on the input grammar")?;
    let arm2_ast = emit_meta_parser_ast_json(input)?;

    let arm1 = differential::read_arm1_envelope(&arm1_envelope);
    let projection = differential::project_meta_parser_ast(&arm2_ast);
    Ok((
        differential::diff_envelopes(grammar_name, &arm1, &projection),
        ground_truth,
    ))
}

fn main() -> Result<()> {
    let args = Args::parse();
    let trace_log_path = args
        .trace_log_file
        .clone()
        .or_else(|| std::env::var("PGEN_TRACE_LOG_FILE").ok());
    configure_trace_output(trace_log_path.as_deref())?;
    let trace_verbosity = resolve_trace_verbosity(args.verbosity.as_deref(), false, false)?;
    set_global_trace_verbosity(trace_verbosity);

    let input = fs::read_to_string(&args.input)
        .with_context(|| format!("failed to read input file '{}'", args.input.display()))?;

    let parse = attempt_parse(&input, false);
    let parse_full = attempt_parse(&input, true);
    let unconsumed_start = if parse.ok && !parse_full.ok {
        parse_full
            .error_position
            .or(parse.span_end)
            .or(parse.error_position)
    } else {
        None
    };

    let report = EbnfDualRunReport {
        input_path: args.input.display().to_string(),
        input_bytes: input.len(),
        parse,
        parse_full,
        unconsumed_start,
        unconsumed_context: unconsumed_start.map(|pos| snippet_at(&input, pos, 48)),
    };

    if let Some(ast_json_path) = args.emit_ast_json.as_ref() {
        let ast_json = emit_meta_parser_ast_json(&input)?;
        if let Some(parent) = ast_json_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create AST directory '{}'", parent.display())
            })?;
        }
        fs::write(ast_json_path, serde_json::to_string_pretty(&ast_json)?).with_context(|| {
            format!(
                "failed to write AST JSON file '{}'",
                ast_json_path.display()
            )
        })?;
    }

    if let Some(differential_path) = args.envelope_differential.as_ref() {
        let grammar_name = args
            .input
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("unknown");
        let (differential_report, ground_truth) =
            run_envelope_differential(&args.input, grammar_name, &input)?;
        let payload = serde_json::json!({
            "envelope_differential": differential_report,
            "ground_truth": {
                "positive_control": "PASS",
                "negative_control": "PASS",
                "negative_control_divergences": ground_truth.negative_control_divergences,
                "positive_control_tokens_compared":
                    ground_truth.positive_control_report.tokens_compared,
            },
        });
        if let Some(parent) = differential_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "failed to create envelope-differential directory '{}'",
                    parent.display()
                )
            })?;
        }
        fs::write(differential_path, serde_json::to_string_pretty(&payload)?).with_context(
            || {
                format!(
                    "failed to write envelope-differential report '{}'",
                    differential_path.display()
                )
            },
        )?;
    }

    let json = serde_json::to_string_pretty(&report)?;
    if let Some(parent) = args.output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create report directory '{}'", parent.display()))?;
    }
    fs::write(&args.output, json)
        .with_context(|| format!("failed to write output file '{}'", args.output.display()))?;

    Ok(())
}
