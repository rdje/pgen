use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use pgen::parser_registry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct CorpusCase {
    id: String,
    pattern: String,
    source: CorpusSource,
    expected: CorpusExpected,
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CorpusSource {
    file: String,
    case_ref: Option<String>,
    line_start: Option<u64>,
    line_end: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct CorpusExpected {
    parse: String,
}

#[derive(Debug, Serialize)]
struct ObservationRow<'a> {
    id: &'a str,
    pattern_preview: String,
    source_file: &'a str,
    case_ref: Option<&'a str>,
    line_start: Option<u64>,
    line_end: Option<u64>,
    expected_parse: &'a str,
    actual_parse: &'a str,
    parser_error: Option<&'a str>,
    expectation_mismatch: bool,
    tags: &'a [String],
}

#[derive(Debug, Serialize)]
struct Summary {
    version: u32,
    grammar_name: &'static str,
    input_jsonl: String,
    cases_declared: u64,
    cases_executed: u64,
    expected_parse_ok_total: u64,
    expected_parse_fail_total: u64,
    expected_parse_unknown_total: u64,
    parse_pass_total: u64,
    parse_fail_total: u64,
    parse_expectation_match_total: u64,
    parse_expectation_mismatch_total: u64,
    false_accept_total: u64,
    false_reject_total: u64,
    acceptance_rate_percent: String,
    max_pattern_bytes: u64,
    total_pattern_bytes: u64,
    primary_failure_case: String,
    primary_failure_source_file: String,
    primary_failure_case_ref: Option<String>,
    primary_failure_line_start: Option<u64>,
    primary_failure_line_end: Option<u64>,
    primary_failure_parser_error: String,
    primary_mismatch_case: String,
    primary_mismatch_source_file: String,
    primary_mismatch_case_ref: Option<String>,
    primary_mismatch_line_start: Option<u64>,
    primary_mismatch_line_end: Option<u64>,
    primary_mismatch_expected_parse: String,
    primary_mismatch_actual_parse: String,
    primary_mismatch_parser_error: Option<String>,
    unique_parser_error_count: u64,
    parser_error_counts: BTreeMap<String, u64>,
}

fn usage() -> &'static str {
    "Usage: regex_corpus_probe <input_jsonl> <summary_json> <failures_jsonl>"
}

fn preview_pattern(pattern: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for ch in pattern.chars().take(max_chars) {
        match ch {
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{0085}' | '\u{2028}' | '\u{2029}' => {
                out.push_str(&format!("\\u{{{:04X}}}", ch as u32));
            }
            other if other.is_control() => {
                out.push_str(&format!("\\u{{{:04X}}}", other as u32));
            }
            other => out.push(other),
        }
    }
    out
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        bail!("{}", usage());
    }

    let input_jsonl = PathBuf::from(&args[1]);
    let summary_json = PathBuf::from(&args[2]);
    let failures_jsonl = PathBuf::from(&args[3]);

    let input_file = File::open(&input_jsonl)
        .with_context(|| format!("failed to open input JSONL '{}'", input_jsonl.display()))?;
    let reader = BufReader::new(input_file);

    let mut failure_file = File::create(&failures_jsonl).with_context(|| {
        format!(
            "failed to create failures JSONL '{}'",
            failures_jsonl.display()
        )
    })?;

    let mut cases_declared = 0u64;
    let mut cases_executed = 0u64;
    let mut expected_parse_ok_total = 0u64;
    let mut expected_parse_fail_total = 0u64;
    let mut expected_parse_unknown_total = 0u64;
    let mut parse_pass_total = 0u64;
    let mut parse_fail_total = 0u64;
    let mut parse_expectation_match_total = 0u64;
    let mut parse_expectation_mismatch_total = 0u64;
    let mut false_accept_total = 0u64;
    let mut false_reject_total = 0u64;
    let mut total_pattern_bytes = 0u64;
    let mut max_pattern_bytes = 0u64;
    let mut parser_error_counts: BTreeMap<String, u64> = BTreeMap::new();

    let mut primary_failure_case = "<none>".to_string();
    let mut primary_failure_source_file = "<none>".to_string();
    let mut primary_failure_case_ref: Option<String> = None;
    let mut primary_failure_line_start: Option<u64> = None;
    let mut primary_failure_line_end: Option<u64> = None;
    let mut primary_failure_parser_error = "<none>".to_string();
    let mut primary_mismatch_case = "<none>".to_string();
    let mut primary_mismatch_source_file = "<none>".to_string();
    let mut primary_mismatch_case_ref: Option<String> = None;
    let mut primary_mismatch_line_start: Option<u64> = None;
    let mut primary_mismatch_line_end: Option<u64> = None;
    let mut primary_mismatch_expected_parse = "<none>".to_string();
    let mut primary_mismatch_actual_parse = "<none>".to_string();
    let mut primary_mismatch_parser_error: Option<String> = None;

    for (index, line) in reader.lines().enumerate() {
        let line = line.with_context(|| {
            format!(
                "failed to read line {} from '{}'",
                index + 1,
                input_jsonl.display()
            )
        })?;
        if line.trim().is_empty() {
            continue;
        }
        let case: CorpusCase = serde_json::from_str(&line).with_context(|| {
            format!(
                "failed to decode JSONL case at line {} from '{}'",
                index + 1,
                input_jsonl.display()
            )
        })?;

        cases_declared += 1;
        cases_executed += 1;
        match case.expected.parse.as_str() {
            "ok" => expected_parse_ok_total += 1,
            "fail" => expected_parse_fail_total += 1,
            _ => expected_parse_unknown_total += 1,
        }

        let pattern_bytes = case.pattern.len() as u64;
        total_pattern_bytes += pattern_bytes;
        if pattern_bytes > max_pattern_bytes {
            max_pattern_bytes = pattern_bytes;
        }

        match parser_registry::parse_sample_detail("regex", &case.pattern) {
            Some(Ok(())) => {
                parse_pass_total += 1;

                let expectation_mismatch = case.expected.parse == "fail";
                if expectation_mismatch {
                    parse_expectation_mismatch_total += 1;
                    false_accept_total += 1;
                    if primary_mismatch_case == "<none>" {
                        primary_mismatch_case = case.id.clone();
                        primary_mismatch_source_file = case.source.file.clone();
                        primary_mismatch_case_ref = case.source.case_ref.clone();
                        primary_mismatch_line_start = case.source.line_start;
                        primary_mismatch_line_end = case.source.line_end;
                        primary_mismatch_expected_parse = case.expected.parse.clone();
                        primary_mismatch_actual_parse = "ok".to_string();
                        primary_mismatch_parser_error = None;
                    }
                } else if case.expected.parse != "unknown" {
                    parse_expectation_match_total += 1;
                }

                if expectation_mismatch {
                    let row = ObservationRow {
                        id: &case.id,
                        pattern_preview: preview_pattern(&case.pattern, 160),
                        source_file: &case.source.file,
                        case_ref: case.source.case_ref.as_deref(),
                        line_start: case.source.line_start,
                        line_end: case.source.line_end,
                        expected_parse: &case.expected.parse,
                        actual_parse: "ok",
                        parser_error: None,
                        expectation_mismatch,
                        tags: &case.tags,
                    };
                    serde_json::to_writer(&mut failure_file, &row)?;
                    failure_file.write_all(b"\n")?;
                }
            }
            Some(Err(err)) => {
                parse_fail_total += 1;
                *parser_error_counts.entry(err.clone()).or_insert(0) += 1;

                if primary_failure_case == "<none>" {
                    primary_failure_case = case.id.clone();
                    primary_failure_source_file = case.source.file.clone();
                    primary_failure_case_ref = case.source.case_ref.clone();
                    primary_failure_line_start = case.source.line_start;
                    primary_failure_line_end = case.source.line_end;
                    primary_failure_parser_error = err.clone();
                }

                let expectation_mismatch = case.expected.parse == "ok";
                if expectation_mismatch {
                    parse_expectation_mismatch_total += 1;
                    false_reject_total += 1;
                    if primary_mismatch_case == "<none>" {
                        primary_mismatch_case = case.id.clone();
                        primary_mismatch_source_file = case.source.file.clone();
                        primary_mismatch_case_ref = case.source.case_ref.clone();
                        primary_mismatch_line_start = case.source.line_start;
                        primary_mismatch_line_end = case.source.line_end;
                        primary_mismatch_expected_parse = case.expected.parse.clone();
                        primary_mismatch_actual_parse = "fail".to_string();
                        primary_mismatch_parser_error = Some(err.clone());
                    }
                } else if case.expected.parse != "unknown" {
                    parse_expectation_match_total += 1;
                }

                let row = ObservationRow {
                    id: &case.id,
                    pattern_preview: preview_pattern(&case.pattern, 160),
                    source_file: &case.source.file,
                    case_ref: case.source.case_ref.as_deref(),
                    line_start: case.source.line_start,
                    line_end: case.source.line_end,
                    expected_parse: &case.expected.parse,
                    actual_parse: "fail",
                    parser_error: Some(&err),
                    expectation_mismatch,
                    tags: &case.tags,
                };
                serde_json::to_writer(&mut failure_file, &row)?;
                failure_file.write_all(b"\n")?;
            }
            None => bail!("generated parser registry does not expose grammar 'regex'"),
        }
    }

    if cases_executed == 0 {
        bail!(
            "regex corpus probe executed zero cases from '{}'",
            input_jsonl.display()
        );
    }

    let acceptance_rate_percent = format!(
        "{:.2}",
        (parse_pass_total as f64 * 100.0) / cases_executed as f64
    );

    let summary = Summary {
        version: 1,
        grammar_name: "regex",
        input_jsonl: input_jsonl.display().to_string(),
        cases_declared,
        cases_executed,
        expected_parse_ok_total,
        expected_parse_fail_total,
        expected_parse_unknown_total,
        parse_pass_total,
        parse_fail_total,
        parse_expectation_match_total,
        parse_expectation_mismatch_total,
        false_accept_total,
        false_reject_total,
        acceptance_rate_percent,
        max_pattern_bytes,
        total_pattern_bytes,
        primary_failure_case,
        primary_failure_source_file,
        primary_failure_case_ref,
        primary_failure_line_start,
        primary_failure_line_end,
        primary_failure_parser_error,
        primary_mismatch_case,
        primary_mismatch_source_file,
        primary_mismatch_case_ref,
        primary_mismatch_line_start,
        primary_mismatch_line_end,
        primary_mismatch_expected_parse,
        primary_mismatch_actual_parse,
        primary_mismatch_parser_error,
        unique_parser_error_count: parser_error_counts.len() as u64,
        parser_error_counts,
    };

    let summary_file = File::create(&summary_json)
        .with_context(|| format!("failed to create summary JSON '{}'", summary_json.display()))?;
    serde_json::to_writer_pretty(summary_file, &summary)?;
    Ok(())
}
