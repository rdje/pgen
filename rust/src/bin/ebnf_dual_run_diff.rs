use anyhow::{Context, Result};
use clap::Parser;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

pub mod ast_pipeline {
    pub use pgen::ast_pipeline::*;
}

#[allow(dead_code)]
mod generated_ebnf {
    include!(env!("PGEN_EBNF_PARSER_PATH_RESOLVED_BIN"));
}

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
    let mut parser = EbnfParser::new(input, runtime_logger_box("generated.ebnf_dual_run_diff"));
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
            span_start: Some(node.span.start),
            span_end: Some(node.span.end),
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

    let json = serde_json::to_string_pretty(&report)?;
    if let Some(parent) = args.output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create report directory '{}'", parent.display()))?;
    }
    fs::write(&args.output, json)
        .with_context(|| format!("failed to write output file '{}'", args.output.display()))?;

    Ok(())
}
