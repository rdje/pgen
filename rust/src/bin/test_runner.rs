#!/usr/bin/env rust
// Universal Test Runner CLI
// Run all tests, filter by parser, or filter by tags

use chrono::Utc;
use clap::{Arg, Command};
use lazy_static::lazy_static;
#[cfg(feature = "generated_parsers")]
use pgen::NoOpLogger;
#[cfg(feature = "generated_parsers")]
use pgen::ast_pipeline::UnifiedReturnAST;
#[cfg(feature = "generated_parsers")]
use pgen::ast_pipeline::UnifiedSemanticAST;
#[cfg(feature = "generated_parsers")]
use pgen::generated_parsers::return_annotation::Return_annotationParser;
#[cfg(feature = "generated_parsers")]
use pgen::generated_parsers::semantic_annotation::Semantic_annotationParser;
#[cfg(feature = "generated_parsers")]
use pgen::test_runner::Logger;
#[cfg(feature = "generated_parsers")]
use pgen::test_runner::normalization::{Normalizer, apply_normalizer};
#[cfg(feature = "generated_parsers")]
use pgen::test_runner::parsers::unparse_return_ast;
use pgen::test_runner::parsers::{
    ReturnAnnotationParser as BootstrapReturnAnnotationParser,
    SemanticAnnotationParser as BootstrapSemanticAnnotationParser,
};
#[cfg(feature = "generated_parsers")]
use pgen::test_runner::round_trip_tests::{RoundTripTest, TestSuite};
use pgen::test_runner::{FileLogger, Parser, UniversalTestRunner};
#[cfg(feature = "generated_parsers")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "generated_parsers")]
use std::collections::{BTreeMap, BTreeSet};
use std::fs::OpenOptions;
use std::io::Write;
use std::process::exit;
use std::sync::Mutex;

#[cfg(feature = "generated_parsers")]
/// Wrapper for generated Return_annotationParser to implement Parser trait
pub struct GeneratedReturnAnnotationParser {
    logger: Box<dyn Logger>,
}

#[cfg(feature = "generated_parsers")]
impl GeneratedReturnAnnotationParser {
    pub fn new(logger: Box<dyn Logger>) -> Self {
        Self { logger }
    }
}

#[cfg(feature = "generated_parsers")]
impl Parser for GeneratedReturnAnnotationParser {
    fn round_trip(&self, input: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Create a parser instance for this specific input
        let mut parser = Return_annotationParser::new(input, self.logger.clone_box());
        let has_arrow_prefix = input.trim_start().starts_with("->");

        // Parse the input
        match parser.parse_full_return_annotation() {
            Ok(parse_node) => {
                let ast = UnifiedReturnAST::parse_generated_return_annotation(
                    input,
                    &parse_node,
                    &*self.logger,
                )
                .map_err(|e| {
                    Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
                        as Box<dyn std::error::Error>
                })?;
                let unparsed = unparse_return_ast(&ast);
                let result = if has_arrow_prefix {
                    format!("-> {}", unparsed)
                } else {
                    unparsed
                };
                Ok(result)
            }
            Err(e) => Err(Box::new(e)),
        }
    }

    fn set_logger(&mut self, logger: Box<dyn Logger>) {
        self.logger = logger;
    }

    fn get_logger(&self) -> &dyn Logger {
        &*self.logger
    }
}

#[cfg(feature = "generated_parsers")]
/// Wrapper for generated Semantic_annotationParser to implement Parser trait
pub struct GeneratedSemanticAnnotationParser {
    logger: Box<dyn Logger>,
}

#[cfg(feature = "generated_parsers")]
impl GeneratedSemanticAnnotationParser {
    pub fn new(logger: Box<dyn Logger>) -> Self {
        Self { logger }
    }
}

#[cfg(feature = "generated_parsers")]
impl Parser for GeneratedSemanticAnnotationParser {
    fn round_trip(&self, input: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Create a parser instance for this specific input
        let mut parser = Semantic_annotationParser::new(input, self.logger.clone_box());

        // Parse the input
        match parser.parse_full_semantic_annotation() {
            Ok(parse_node) => {
                let ast = UnifiedSemanticAST::parse_generated_semantic_annotation(
                    input,
                    &parse_node,
                    &*self.logger,
                )
                .map_err(|e| {
                    Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
                        as Box<dyn std::error::Error>
                })?;
                let unparsed = match ast {
                    UnifiedSemanticAST::TransformExpr { expression } => {
                        format!("@transform: {}", expression)
                    }
                    UnifiedSemanticAST::Raw { content } => content,
                };
                Ok(unparsed)
            }
            Err(e) => Err(Box::new(e)),
        }
    }

    fn set_logger(&mut self, logger: Box<dyn Logger>) {
        self.logger = logger;
    }

    fn get_logger(&self) -> &dyn Logger {
        &*self.logger
    }
}

lazy_static! {
    static ref LOG_FILE: Mutex<Option<std::fs::File>> = Mutex::new(None);
    static ref CURRENT_LOG_PATH: Mutex<Option<String>> = Mutex::new(None);
}

fn log_output(message: &str) {
    // Print to console
    println!("{}", message);

    // Write to log file if available
    if let Ok(mut file_guard) = LOG_FILE.lock() {
        if let Some(ref mut file) = *file_guard {
            let _ = writeln!(file, "{}", message);
        }
    }
}

fn log_error(message: &str) {
    // Print to stderr
    eprintln!("{}", message);

    // Write to log file if available
    if let Ok(mut file_guard) = LOG_FILE.lock() {
        if let Some(ref mut file) = *file_guard {
            let _ = writeln!(file, "{}", message);
        }
    }
}

fn get_current_log_file_path() -> Result<String, Box<dyn std::error::Error>> {
    CURRENT_LOG_PATH
        .lock()
        .unwrap()
        .as_ref()
        .cloned()
        .ok_or_else(|| "No log file path set".into())
}

fn setup_logging(log_path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let log_file_path = match log_path {
        Some(path) => path.to_string(),
        None => {
            let now = Utc::now();
            format!("test_runner_{}.log", now.format("%Y%m%d_%H%M%S"))
        }
    };

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)?;

    *LOG_FILE.lock().unwrap() = Some(file);
    *CURRENT_LOG_PATH.lock().unwrap() = Some(log_file_path);
    Ok(())
}

fn canonical_parser_type(parser_type: &str) -> String {
    match parser_type.trim().to_lowercase().as_str() {
        "return" | "return_annotation" | "return_annotations" => "return".to_string(),
        "semantic" | "semantic_annotation" | "semantic_annotations" => "semantic".to_string(),
        "regex" => "regex".to_string(),
        "all" => "all".to_string(),
        _ => parser_type.trim().to_string(),
    }
}

fn configure_parser_logger(parser: &mut dyn Parser, debug_enabled: bool) {
    if !debug_enabled {
        return;
    }
    if let Ok(log_file_path) = get_current_log_file_path() {
        if let Ok(file) = OpenOptions::new().append(true).open(&log_file_path) {
            parser.set_logger(Box::new(FileLogger::new(file)));
        }
    }
}

#[cfg(feature = "generated_parsers")]
fn suite_matches_filter(suite: &TestSuite, suite_filter: &str) -> bool {
    let name = suite.name.to_lowercase();
    let suite_name = suite.suite_name.to_lowercase();
    let description = suite.description.to_lowercase();
    name == suite_filter
        || suite_name == suite_filter
        || name.contains(suite_filter)
        || suite_name.contains(suite_filter)
        || description.contains(suite_filter)
}

#[cfg(feature = "generated_parsers")]
fn test_matches_tag_filter(test: &RoundTripTest, tag_filter: &[String]) -> bool {
    tag_filter.is_empty()
        || test
            .tags
            .iter()
            .any(|tag| tag_filter.iter().any(|f| f == tag))
}

#[cfg(feature = "generated_parsers")]
fn normalize_round_trip_output(parser_type: &str, test: &RoundTripTest, unparsed: &str) -> String {
    let mut normalizer = Normalizer::from_str(&test.normalizer);
    if matches!(normalizer, Normalizer::Text) && parser_type == "return" {
        normalizer = Normalizer::ReturnAst;
    }
    apply_normalizer(normalizer, unparsed, test.float_precision)
}

#[cfg(feature = "generated_parsers")]
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum DifferentialOutcome {
    Success { raw: String, normalized: String },
    Failure { error: String },
}

#[cfg(feature = "generated_parsers")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct DifferentialMismatchKey {
    suite: String,
    test: String,
}

#[cfg(feature = "generated_parsers")]
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum DifferentialMismatchCategory {
    BaselineSuccessCandidateFailure,
    BaselineFailureCandidateSuccess,
    NormalizedOutputMismatch,
}

#[cfg(feature = "generated_parsers")]
impl DifferentialMismatchCategory {
    fn as_str(self) -> &'static str {
        match self {
            Self::BaselineSuccessCandidateFailure => "baseline_success_candidate_failure",
            Self::BaselineFailureCandidateSuccess => "baseline_failure_candidate_success",
            Self::NormalizedOutputMismatch => "normalized_output_mismatch",
        }
    }
}

#[cfg(feature = "generated_parsers")]
#[derive(Debug, Serialize)]
struct DifferentialMismatch {
    suite: String,
    test: String,
    category: DifferentialMismatchCategory,
    input: String,
    normalizer: String,
    expected_round_trip: String,
    baseline: DifferentialOutcome,
    candidate: DifferentialOutcome,
}

#[cfg(feature = "generated_parsers")]
#[derive(Debug, Serialize)]
struct DifferentialReport {
    parser: String,
    baseline: String,
    candidate: String,
    comparable_only: bool,
    suite_filter: Option<String>,
    tag_filter: Vec<String>,
    total_cases: usize,
    skipped_non_comparable_cases: usize,
    matched_cases: usize,
    mismatched_cases: usize,
    mismatch_category_counts: BTreeMap<String, usize>,
    baseline_path: Option<String>,
    baseline_allowed_mismatches: Option<usize>,
    baseline_new_mismatches: Option<usize>,
    baseline_resolved_mismatches: Option<usize>,
    baseline_new_mismatch_cases: Vec<DifferentialMismatchKey>,
    baseline_resolved_mismatch_cases: Vec<DifferentialMismatchKey>,
    mismatches: Vec<DifferentialMismatch>,
}

#[cfg(feature = "generated_parsers")]
#[derive(Debug, Deserialize, Serialize)]
struct DifferentialBaselineFile {
    parser: String,
    allowed_mismatches: Vec<DifferentialMismatchKey>,
}

#[cfg(feature = "generated_parsers")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DifferentialExpectationClass {
    Pass,
    Fail,
    Skip,
}

#[cfg(feature = "generated_parsers")]
fn parse_differential_expectation(raw: &str) -> DifferentialExpectationClass {
    match raw.trim().to_lowercase().as_str() {
        "skip" => DifferentialExpectationClass::Skip,
        "fail" | "expected_fail" => DifferentialExpectationClass::Fail,
        _ => DifferentialExpectationClass::Pass,
    }
}

#[cfg(feature = "generated_parsers")]
fn differential_case_is_comparable(test: &RoundTripTest) -> bool {
    let baseline = parse_differential_expectation(&test.expectations.bootstrap_parser);
    let candidate = parse_differential_expectation(&test.expectations.generated_parser);

    baseline != DifferentialExpectationClass::Skip
        && candidate != DifferentialExpectationClass::Skip
        && baseline == candidate
}

#[cfg(feature = "generated_parsers")]
fn classify_outcome_mismatch(
    baseline: &DifferentialOutcome,
    candidate: &DifferentialOutcome,
) -> Option<DifferentialMismatchCategory> {
    match (baseline, candidate) {
        (
            DifferentialOutcome::Success {
                normalized: baseline,
                ..
            },
            DifferentialOutcome::Success {
                normalized: candidate,
                ..
            },
        ) => {
            if baseline == candidate {
                None
            } else {
                Some(DifferentialMismatchCategory::NormalizedOutputMismatch)
            }
        }
        (DifferentialOutcome::Failure { .. }, DifferentialOutcome::Failure { .. }) => None,
        (DifferentialOutcome::Success { .. }, DifferentialOutcome::Failure { .. }) => {
            Some(DifferentialMismatchCategory::BaselineSuccessCandidateFailure)
        }
        (DifferentialOutcome::Failure { .. }, DifferentialOutcome::Success { .. }) => {
            Some(DifferentialMismatchCategory::BaselineFailureCandidateSuccess)
        }
    }
}

#[cfg(feature = "generated_parsers")]
fn run_differential_mode(
    parser_type: &str,
    suite_filter: Option<String>,
    tag_filter: Vec<String>,
    comparable_only: bool,
    debug_enabled: bool,
    fail_fast: bool,
    baseline_json_path: Option<&str>,
    write_baseline_json_path: Option<&str>,
    regression_only: bool,
    report_json_path: Option<&str>,
) -> Result<i32, Box<dyn std::error::Error>> {
    let canonical = canonical_parser_type(parser_type);
    if canonical != "return" && canonical != "semantic" {
        return Err(format!(
            "Differential mode requires --parser return|semantic, got '{}'",
            parser_type
        )
        .into());
    }

    let (baseline_parser, candidate_parser): (Box<dyn Parser>, Box<dyn Parser>) =
        match canonical.as_str() {
            "return" => {
                let mut baseline = BootstrapReturnAnnotationParser::new();
                configure_parser_logger(&mut baseline, debug_enabled);
                let mut candidate = GeneratedReturnAnnotationParser::new(Box::new(NoOpLogger));
                configure_parser_logger(&mut candidate, debug_enabled);
                (Box::new(baseline), Box::new(candidate))
            }
            "semantic" => {
                let mut baseline = BootstrapSemanticAnnotationParser::new();
                configure_parser_logger(&mut baseline, debug_enabled);
                let mut candidate = GeneratedSemanticAnnotationParser::new(Box::new(NoOpLogger));
                configure_parser_logger(&mut candidate, debug_enabled);
                (Box::new(baseline), Box::new(candidate))
            }
            _ => unreachable!(),
        };

    let suites = UniversalTestRunner::new().discover_test_suites()?;
    let normalized_suite_filter = suite_filter.as_ref().map(|s| s.trim().to_lowercase());
    let mut report = DifferentialReport {
        parser: canonical.clone(),
        baseline: "bootstrap".to_string(),
        candidate: "generated".to_string(),
        comparable_only,
        suite_filter: suite_filter.clone(),
        tag_filter: tag_filter.clone(),
        total_cases: 0,
        skipped_non_comparable_cases: 0,
        matched_cases: 0,
        mismatched_cases: 0,
        mismatch_category_counts: BTreeMap::new(),
        baseline_path: baseline_json_path.map(|s| s.to_string()),
        baseline_allowed_mismatches: None,
        baseline_new_mismatches: None,
        baseline_resolved_mismatches: None,
        baseline_new_mismatch_cases: Vec::new(),
        baseline_resolved_mismatch_cases: Vec::new(),
        mismatches: Vec::new(),
    };

    'suite_loop: for suite in suites {
        if let Some(ref suite_filter_value) = normalized_suite_filter {
            if !suite_matches_filter(&suite, suite_filter_value) {
                continue;
            }
        }

        for test in &suite.tests {
            if test.skip {
                continue;
            }
            if canonical_parser_type(&test.parser_type) != canonical {
                continue;
            }
            if !test_matches_tag_filter(test, &tag_filter) {
                continue;
            }
            if comparable_only && !differential_case_is_comparable(test) {
                report.skipped_non_comparable_cases += 1;
                continue;
            }

            report.total_cases += 1;

            let baseline_outcome = match baseline_parser.round_trip(&test.input) {
                Ok(raw) => DifferentialOutcome::Success {
                    normalized: normalize_round_trip_output(&canonical, test, &raw),
                    raw,
                },
                Err(err) => DifferentialOutcome::Failure {
                    error: err.to_string(),
                },
            };
            let candidate_outcome = match candidate_parser.round_trip(&test.input) {
                Ok(raw) => DifferentialOutcome::Success {
                    normalized: normalize_round_trip_output(&canonical, test, &raw),
                    raw,
                },
                Err(err) => DifferentialOutcome::Failure {
                    error: err.to_string(),
                },
            };

            let mismatch_category =
                classify_outcome_mismatch(&baseline_outcome, &candidate_outcome);
            if mismatch_category.is_none() {
                report.matched_cases += 1;
                continue;
            }

            report.mismatched_cases += 1;
            let category = mismatch_category.expect("category already checked for mismatch");
            *report
                .mismatch_category_counts
                .entry(category.as_str().to_string())
                .or_insert(0) += 1;
            report.mismatches.push(DifferentialMismatch {
                suite: suite.name.clone(),
                test: test.name.clone(),
                category,
                input: test.input.clone(),
                normalizer: test.normalizer.clone(),
                expected_round_trip: test.expected_round_trip.clone(),
                baseline: baseline_outcome,
                candidate: candidate_outcome,
            });
            log_output(&format!(
                "❌ Differential mismatch: {} / {}",
                suite.name, test.name
            ));
            if fail_fast {
                break 'suite_loop;
            }
        }
    }

    if let Some(path) = baseline_json_path {
        let baseline_json = std::fs::read_to_string(path).map_err(|e| {
            format!(
                "Failed to read differential baseline JSON '{}': {}",
                path, e
            )
        })?;
        let baseline: DifferentialBaselineFile =
            serde_json::from_str(&baseline_json).map_err(|e| {
                format!(
                    "Failed to parse differential baseline JSON '{}': {}",
                    path, e
                )
            })?;
        let baseline_parser = canonical_parser_type(&baseline.parser);
        if baseline_parser != canonical {
            return Err(format!(
                "Differential baseline parser mismatch: baseline='{}' current='{}'",
                baseline_parser, canonical
            )
            .into());
        }

        let current_set: BTreeSet<DifferentialMismatchKey> = report
            .mismatches
            .iter()
            .map(|m| DifferentialMismatchKey {
                suite: m.suite.clone(),
                test: m.test.clone(),
            })
            .collect();
        let baseline_set: BTreeSet<DifferentialMismatchKey> =
            baseline.allowed_mismatches.into_iter().collect();

        report.baseline_allowed_mismatches = Some(baseline_set.len());
        report.baseline_new_mismatch_cases = current_set
            .difference(&baseline_set)
            .cloned()
            .collect::<Vec<_>>();
        report.baseline_resolved_mismatch_cases = baseline_set
            .difference(&current_set)
            .cloned()
            .collect::<Vec<_>>();
        report.baseline_new_mismatches = Some(report.baseline_new_mismatch_cases.len());
        report.baseline_resolved_mismatches = Some(report.baseline_resolved_mismatch_cases.len());
    }

    log_output("🧪 Differential Harness (generated vs bootstrap)");
    log_output(&"=".repeat(60));
    log_output(&format!("Parser: {}", report.parser));
    if let Some(ref suite_filter_value) = report.suite_filter {
        log_output(&format!("Suite filter: {}", suite_filter_value));
    }
    if !report.tag_filter.is_empty() {
        log_output(&format!("Tag filter: {}", report.tag_filter.join(",")));
    }
    if report.comparable_only {
        log_output(&format!(
            "Comparable-only filter: skipped {} non-comparable case(s)",
            report.skipped_non_comparable_cases
        ));
    }
    log_output(&format!(
        "Compared {} case(s): matched={} mismatched={}",
        report.total_cases, report.matched_cases, report.mismatched_cases
    ));
    if !report.mismatch_category_counts.is_empty() {
        log_output("Mismatch categories:");
        for (category, count) in &report.mismatch_category_counts {
            log_output(&format!("  - {}: {}", category, count));
        }
    }
    if let Some(path) = baseline_json_path {
        log_output(&format!("Baseline comparison: {}", path));
        log_output(&format!(
            "  allowed={} new={} resolved={}",
            report.baseline_allowed_mismatches.unwrap_or(0),
            report.baseline_new_mismatches.unwrap_or(0),
            report.baseline_resolved_mismatches.unwrap_or(0)
        ));
    }

    if let Some(path) = write_baseline_json_path {
        let baseline_output = DifferentialBaselineFile {
            parser: canonical.clone(),
            allowed_mismatches: report
                .mismatches
                .iter()
                .map(|m| DifferentialMismatchKey {
                    suite: m.suite.clone(),
                    test: m.test.clone(),
                })
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
        };
        std::fs::write(path, serde_json::to_string_pretty(&baseline_output)?)?;
        log_output(&format!("Wrote differential baseline JSON: {}", path));
    }

    if let Some(path) = report_json_path {
        std::fs::write(path, serde_json::to_string_pretty(&report)?)?;
        log_output(&format!("Wrote differential report JSON: {}", path));
    }

    if regression_only {
        if baseline_json_path.is_none() {
            return Err(
                "--differential-regression-only requires --differential-baseline-json".into(),
            );
        }
        if report.baseline_new_mismatches.unwrap_or(0) > 0 {
            for mismatch in report.baseline_new_mismatch_cases.iter().take(20) {
                log_output(&format!(
                    "  - NEW mismatch: {} / {}",
                    mismatch.suite, mismatch.test
                ));
            }
            return Ok(1);
        }
        return Ok(0);
    }

    if report.mismatched_cases > 0 {
        for mismatch in report.mismatches.iter().take(20) {
            log_output(&format!(
                "  - {} / {} [{}] (normalizer: {})",
                mismatch.suite,
                mismatch.test,
                mismatch.category.as_str(),
                mismatch.normalizer
            ));
        }
        return Ok(1);
    }

    Ok(0)
}

#[cfg(not(feature = "generated_parsers"))]
fn run_differential_mode(
    _parser_type: &str,
    _suite_filter: Option<String>,
    _tag_filter: Vec<String>,
    _comparable_only: bool,
    _debug_enabled: bool,
    _fail_fast: bool,
    _baseline_json_path: Option<&str>,
    _write_baseline_json_path: Option<&str>,
    _regression_only: bool,
    _report_json_path: Option<&str>,
) -> Result<i32, Box<dyn std::error::Error>> {
    Err("Differential mode requires `cargo run --features generated_parsers --bin test_runner -- --differential ...`".into())
}

fn main() {
    let matches = Command::new("test_runner")
        .about("Universal Test Runner for pgen")
        .version("1.0.0")
        .arg(
            Arg::new("parser")
                .short('p')
                .long("parser")
                .value_name("TYPE")
                .help("Filter tests by parser type")
                .value_parser([
                    "return",
                    "return_annotation",
                    "return_annotations",
                    "semantic",
                    "semantic_annotation",
                    "semantic_annotations",
                    "regex",
                    "all",
                ])
        )
        .arg(
            Arg::new("tags")
                .short('t')
                .long("tags")
                .value_name("TAGS")
                .help("Filter tests by tags (comma-separated)")
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Show detailed output")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("suite")
                .short('s')
                .long("suite")
                .value_name("NAME")
                .help("Run specific test suite by name")
        )
        .arg(
            Arg::new("list")
                .short('l')
                .long("list")
                .help("List available test suites without running")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("dashboard")
                .short('d')
                .long("dashboard")
                .help("Show comprehensive dashboard output (like stress tests)")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("log_file")
                .short('L')
                .long("log-file")
                .help("Path to log file (default: test_runner_YYYYMMDD_HHMMSS.log in current directory)")
                .value_name("PATH")
        )
        .arg(
            Arg::new("debug")
                .short('D')
                .long("debug")
                .help("Enable debug logging for parsers")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("fail_fast")
                .short('f')
                .long("fail-fast")
                .help("Stop after first failure (best for focused debugging)")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("differential")
                .long("differential")
                .help("Run differential harness: generated parser vs bootstrap baseline")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("differential_report_json")
                .long("differential-report-json")
                .value_name("PATH")
                .help("Write machine-readable differential mismatch report JSON")
                .requires("differential")
        )
        .arg(
            Arg::new("differential_comparable_only")
                .long("differential-comparable-only")
                .help("Compare only cases where bootstrap/generated expectations are both non-skip and equivalent")
                .requires("differential")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("differential_baseline_json")
                .long("differential-baseline-json")
                .value_name("PATH")
                .help("Read baseline mismatch set and compute new/resolved mismatch closure metrics")
                .requires("differential")
        )
        .arg(
            Arg::new("differential_write_baseline_json")
                .long("differential-write-baseline-json")
                .value_name("PATH")
                .help("Write current mismatch set as baseline JSON for future regression checks")
                .requires("differential")
        )
        .arg(
            Arg::new("differential_regression_only")
                .long("differential-regression-only")
                .help("Exit non-zero only for mismatches that are NEW vs baseline")
                .requires("differential_baseline_json")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    let verbose = matches.get_flag("verbose");
    let list_only = matches.get_flag("list");
    let debug_enabled = matches.get_flag("debug");
    let fail_fast = matches.get_flag("fail_fast");
    let differential_mode = matches.get_flag("differential");

    // Setup logging
    let log_file_path = matches.get_one::<String>("log_file").map(|s| s.as_str());
    if let Err(e) = setup_logging(log_file_path) {
        eprintln!("Failed to setup logging: {}", e);
        exit(1);
    }

    let selected_parser = matches
        .get_one::<String>("parser")
        .map(|parser| canonical_parser_type(parser));
    let tag_filter: Vec<String> = matches
        .get_one::<String>("tags")
        .map(|tags_str| tags_str.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    if differential_mode {
        let parser_type = selected_parser.as_deref().unwrap_or("all");
        let suite_filter = matches.get_one::<String>("suite").cloned();
        let report_path = matches
            .get_one::<String>("differential_report_json")
            .map(|s| s.as_str());
        let comparable_only = matches.get_flag("differential_comparable_only");
        let baseline_path = matches
            .get_one::<String>("differential_baseline_json")
            .map(|s| s.as_str());
        let write_baseline_path = matches
            .get_one::<String>("differential_write_baseline_json")
            .map(|s| s.as_str());
        let regression_only = matches.get_flag("differential_regression_only");
        match run_differential_mode(
            parser_type,
            suite_filter,
            tag_filter,
            comparable_only,
            debug_enabled,
            fail_fast,
            baseline_path,
            write_baseline_path,
            regression_only,
            report_path,
        ) {
            Ok(code) => exit(code),
            Err(e) => {
                log_error(&format!("Differential harness error: {}", e));
                exit(2);
            }
        }
    }

    // Create runner with options
    let mut runner = UniversalTestRunner::new().with_verbose(verbose);
    runner = runner.with_fail_fast(fail_fast);

    // Select parser based on filter if specified
    if let Some(ref parser_type) = selected_parser {
        match parser_type.as_str() {
            "return" => {
                #[cfg(feature = "generated_parsers")]
                {
                    let mut parser = GeneratedReturnAnnotationParser::new(Box::new(NoOpLogger));
                    configure_parser_logger(&mut parser, debug_enabled);
                    runner = runner.with_parser(Box::new(parser));
                }
                #[cfg(not(feature = "generated_parsers"))]
                {
                    let mut parser = BootstrapReturnAnnotationParser::new();
                    configure_parser_logger(&mut parser, debug_enabled);
                    runner = runner.with_parser(Box::new(parser));
                }
            }
            "semantic" => {
                #[cfg(feature = "generated_parsers")]
                {
                    let mut parser = GeneratedSemanticAnnotationParser::new(Box::new(NoOpLogger));
                    configure_parser_logger(&mut parser, debug_enabled);
                    runner = runner.with_parser(Box::new(parser));
                }
                #[cfg(not(feature = "generated_parsers"))]
                {
                    let mut parser = BootstrapSemanticAnnotationParser::new();
                    configure_parser_logger(&mut parser, debug_enabled);
                    runner = runner.with_parser(Box::new(parser));
                }
            }
            // For "all" or other values, use mock parser
            _ => {}
        }
    }

    // Apply filters
    if let Some(ref parser) = selected_parser {
        if parser != "all" {
            runner = runner.with_parser_filter(parser.to_string());
        }
    }

    if !tag_filter.is_empty() {
        runner = runner.with_tag_filter(tag_filter.clone());
    }
    if let Some(suite) = matches.get_one::<String>("suite") {
        runner = runner.with_suite_filter(suite.to_string());
    }

    // List mode
    if list_only {
        match runner.discover_test_suites() {
            Ok(suites) => {
                log_output("📋 Available Test Suites:");
                log_output(&"=".repeat(60));
                let suite_filter = matches
                    .get_one::<String>("suite")
                    .map(|s| s.trim().to_lowercase());
                let mut suite_count = 0usize;
                for suite in suites {
                    if let Some(ref filter) = suite_filter {
                        let name = suite.name.to_lowercase();
                        let suite_name = suite.suite_name.to_lowercase();
                        let description = suite.description.to_lowercase();
                        if !(name == *filter
                            || suite_name == *filter
                            || name.contains(filter)
                            || suite_name.contains(filter)
                            || description.contains(filter))
                        {
                            continue;
                        }
                    }
                    suite_count += 1;
                    log_output(&format!("• {} ({})", suite.suite_name, suite.parser_type));
                    log_output(&format!("  {}", suite.description));
                    log_output(&format!("  Tests: {}", suite.tests.len()));
                }
                log_output(&"=".repeat(60));
                log_output(&format!("Total: {} suites", suite_count));
            }
            Err(e) => {
                log_error(&format!("Error discovering test suites: {}", e));
                exit(1);
            }
        }
        return;
    }

    // Run tests
    log_output("🚀 Universal Test Runner");
    log_output(&"=".repeat(60));

    if let Some(ref parser) = selected_parser {
        log_output(&format!("Parser filter: {}", parser));
    }
    if let Some(tags) = matches.get_one::<String>("tags") {
        log_output(&format!("Tag filter: {}", tags));
    }
    if let Some(suite) = matches.get_one::<String>("suite") {
        log_output(&format!("Suite filter: {}", suite));
    }
    if fail_fast {
        log_output("Fail-fast: enabled");
    }

    let show_dashboard = matches.get_flag("dashboard");

    match runner.run_all_tests() {
        Ok(report) => {
            if show_dashboard {
                // Get parser name from filter or use "All Parsers"
                let parser_name = selected_parser.as_deref().unwrap_or("All Parsers");
                report.print_dashboard(parser_name);
            } else {
                report.print_summary();
            }

            if report.failed > 0 {
                exit(1);
            } else {
                exit(0);
            }
        }
        Err(e) => {
            log_error(&format!("\n❌ Test runner error: {}", e));
            exit(2);
        }
    }
}
