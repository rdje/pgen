//! Per-parser-family AST-shape contract runner.
//!
//! Verifies the runtime AST that a generated parser actually produces against a
//! tracked manifest. The systemic gap this closes: drift between the grammar's
//! declared return annotations and what the running generated parser emits used
//! to be invisible until somebody read the generated source by hand. The
//! regex-grammar codegen drop (object-literal annotations declared in
//! `grammars/regex.ebnf` for the `regex` and `piece` rules but never reaching
//! `generated/regex_parser.rs`) is the prototype example.
//!
//! Each manifest documents, per sample input:
//! - the grammar rule being exercised,
//! - the return annotation declared in the EBNF source,
//! - the AST content kind we expect once the generated parser correctly applies
//!   that annotation (`expected_content_kind`),
//! - the AST content kind the tracked generated parser emits today
//!   (`current_content_kind`),
//! - and a `drift_status` label naming the open work when the two disagree.
//!
//! The runner enforces:
//! 1. for every sample, the running generated parser's emitted content kind
//!    must equal `current_content_kind` exactly. This is the regression-lock.
//!    If a parser is regenerated without updating the manifest in the same
//!    commit, this assertion fails and the inconsistency becomes visible at
//!    gate time instead of being discovered later.
//! 2. for samples whose `current_content_kind == expected_content_kind`,
//!    `drift_status` must be `"aligned"` and the additional structural
//!    assertions (object keys, string-valued fields) must hold.
//! 3. for samples with drift, the runner emits a structured drift summary so
//!    every per-family gate run reports how many samples are still drifting
//!    and which lanes track the closure work.
//!
//! When per-family regeneration lands in a follow-up commit, the workflow is
//! to update each affected sample's `current_content_kind` to match the
//! regenerated parser, set `drift_status` to `"aligned"`, and verify the
//! structural assertions pass. The act of editing the manifest in the same
//! commit as the regeneration is the explicit acknowledgement that AST shape
//! has changed.

use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

use crate::ast_pipeline::{ParseContent, ParseNode};

const ALIGNED_STATUS: &str = "aligned";

/// Top-level manifest schema. One file per grammar.
#[derive(Debug, Deserialize)]
pub struct AstShapeContractManifest {
    pub version: u32,
    pub grammar: String,
    pub purpose: String,
    pub doctrine: String,
    pub samples: Vec<AstShapeContractSample>,
}

/// One assertion per sample input.
#[derive(Debug, Deserialize)]
pub struct AstShapeContractSample {
    pub name: String,
    pub input: String,
    pub rule_under_test: String,
    pub declared_annotation: String,
    pub expected_content_kind: ContentKind,
    #[serde(default)]
    pub expected_json_object_keys_present: Vec<String>,
    #[serde(default)]
    pub expected_json_object_string_values: BTreeMap<String, String>,
    pub current_content_kind: ContentKind,
    pub drift_status: String,
    pub drift_tracked_in: String,
}

/// Stable shape labels for the runtime carrier produced by a generated parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentKind {
    Terminal,
    TransformedTerminal,
    Json,
    JsonObject,
    JsonArray,
    JsonString,
    JsonNumber,
    JsonBool,
    JsonNull,
    Sequence,
    Alternative,
    Quantified,
}

impl ContentKind {
    fn classify(content: &ParseContent<'_>) -> Self {
        match content {
            ParseContent::Terminal(_) => ContentKind::Terminal,
            ParseContent::TransformedTerminal(_) => ContentKind::TransformedTerminal,
            ParseContent::Json(value) => match value {
                serde_json::Value::Object(_) => ContentKind::JsonObject,
                serde_json::Value::Array(_) => ContentKind::JsonArray,
                serde_json::Value::String(_) => ContentKind::JsonString,
                serde_json::Value::Number(_) => ContentKind::JsonNumber,
                serde_json::Value::Bool(_) => ContentKind::JsonBool,
                serde_json::Value::Null => ContentKind::JsonNull,
            },
            ParseContent::Sequence(_) => ContentKind::Sequence,
            ParseContent::Alternative(_) => ContentKind::Alternative,
            ParseContent::Quantified(_, _) => ContentKind::Quantified,
        }
    }
}

/// One sample's assertion outcome.
#[derive(Debug)]
pub struct SampleOutcome {
    pub name: String,
    pub rule_under_test: String,
    pub observed_content_kind: ContentKind,
    pub manifest_current_content_kind: ContentKind,
    pub manifest_expected_content_kind: ContentKind,
    pub drift_status: String,
    pub structural_assertions_passed: bool,
    pub structural_assertion_details: Vec<String>,
}

/// Aggregate report for a manifest run.
#[derive(Debug, Default)]
pub struct ContractReport {
    pub samples: Vec<SampleOutcome>,
    pub regression_lock_failures: Vec<String>,
    pub aligned_samples_with_failed_assertions: Vec<String>,
    pub drift_count_by_status: BTreeMap<String, usize>,
}

impl ContractReport {
    pub fn drift_total(&self) -> usize {
        self.drift_count_by_status
            .iter()
            .filter(|(status, _)| status.as_str() != ALIGNED_STATUS)
            .map(|(_, count)| *count)
            .sum()
    }

    pub fn aligned_total(&self) -> usize {
        self.drift_count_by_status
            .get(ALIGNED_STATUS)
            .copied()
            .unwrap_or(0)
    }

    pub fn passed(&self) -> bool {
        self.regression_lock_failures.is_empty()
            && self.aligned_samples_with_failed_assertions.is_empty()
    }

    pub fn summary_line(&self) -> String {
        format!(
            "samples={} aligned={} drift={} regression_lock_failures={} aligned_assertion_failures={}",
            self.samples.len(),
            self.aligned_total(),
            self.drift_total(),
            self.regression_lock_failures.len(),
            self.aligned_samples_with_failed_assertions.len(),
        )
    }
}

/// Load a manifest from a tracked path. Path is relative to the repo's
/// `rust/` directory so callers can use the conventional
/// `test_data/ast_shape_contract/<grammar>_v<n>.json` form.
pub fn load_manifest<P: AsRef<Path>>(path: P) -> std::io::Result<AstShapeContractManifest> {
    let raw = std::fs::read_to_string(path)?;
    serde_json::from_str(&raw).map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("ast-shape-contract manifest deserialise failed: {}", err),
        )
    })
}

/// Run a manifest through a caller-supplied parser callback. The callback
/// returns the top-level `ParseNode` for a sample input. The runner
/// classifies the resulting content, asserts against the manifest, and
/// produces a structured report. The runner does NOT panic; callers decide
/// whether a non-passing report is a hard error.
pub fn run_manifest<F>(
    manifest: &AstShapeContractManifest,
    mut parse_sample: F,
) -> ContractReport
where
    F: for<'input> FnMut(&'input str) -> Result<ParseNode<'input>, String>,
{
    let mut report = ContractReport::default();

    for sample in &manifest.samples {
        let parsed = match parse_sample(&sample.input) {
            Ok(node) => node,
            Err(err) => {
                let detail = format!(
                    "sample '{}' parse failed: {} (input={:?})",
                    sample.name, err, sample.input
                );
                report.regression_lock_failures.push(detail);
                continue;
            }
        };

        let observed = ContentKind::classify(&parsed.content);

        if observed != sample.current_content_kind {
            report.regression_lock_failures.push(format!(
                "sample '{}': observed content_kind {:?} != manifest current_content_kind {:?}; either the parser was regenerated without updating the manifest in the same commit, or a code change altered runtime shape unexpectedly",
                sample.name, observed, sample.current_content_kind
            ));
        }

        let mut structural_passed = true;
        let mut details = Vec::new();

        let aligned = sample.current_content_kind == sample.expected_content_kind;
        if aligned {
            if sample.drift_status != ALIGNED_STATUS {
                report.aligned_samples_with_failed_assertions.push(format!(
                    "sample '{}': current_content_kind matches expected_content_kind but drift_status is {:?} (must be \"aligned\")",
                    sample.name, sample.drift_status
                ));
                structural_passed = false;
            }

            if matches!(sample.expected_content_kind, ContentKind::JsonObject) {
                if let ParseContent::Json(serde_json::Value::Object(map)) = &parsed.content {
                    for key in &sample.expected_json_object_keys_present {
                        if !map.contains_key(key) {
                            details.push(format!("missing required key '{}'", key));
                            structural_passed = false;
                        }
                    }
                    for (key, expected_value) in &sample.expected_json_object_string_values {
                        match map.get(key) {
                            Some(serde_json::Value::String(actual)) if actual == expected_value => {}
                            Some(serde_json::Value::String(actual)) => {
                                details.push(format!(
                                    "key '{}' string value mismatch: expected {:?}, got {:?}",
                                    key, expected_value, actual
                                ));
                                structural_passed = false;
                            }
                            Some(other) => {
                                details.push(format!(
                                    "key '{}' expected JSON string {:?}, got {}",
                                    key, expected_value, other
                                ));
                                structural_passed = false;
                            }
                            None => {
                                details.push(format!(
                                    "key '{}' missing (expected JSON string {:?})",
                                    key, expected_value
                                ));
                                structural_passed = false;
                            }
                        }
                    }
                } else {
                    details.push(format!(
                        "expected_content_kind=json_object but observed content was not Json(Object); observed={:?}",
                        observed
                    ));
                    structural_passed = false;
                }
            }

            if !structural_passed {
                report.aligned_samples_with_failed_assertions.push(format!(
                    "sample '{}' aligned but structural assertions failed: {}",
                    sample.name,
                    details.join("; ")
                ));
            }
        }

        *report
            .drift_count_by_status
            .entry(sample.drift_status.clone())
            .or_default() += 1;

        report.samples.push(SampleOutcome {
            name: sample.name.clone(),
            rule_under_test: sample.rule_under_test.clone(),
            observed_content_kind: observed,
            manifest_current_content_kind: sample.current_content_kind,
            manifest_expected_content_kind: sample.expected_content_kind,
            drift_status: sample.drift_status.clone(),
            structural_assertions_passed: structural_passed,
            structural_assertion_details: details,
        });
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_report(family: &str, report: &ContractReport) {
        eprintln!(
            "[ast_shape_contract][{}] {} samples_per_status={:?}",
            family,
            report.summary_line(),
            report.drift_count_by_status,
        );
        for outcome in &report.samples {
            eprintln!(
                "  - {} (rule={}) observed={:?} manifest_current={:?} manifest_expected={:?} drift_status={} structural_ok={}{}",
                outcome.name,
                outcome.rule_under_test,
                outcome.observed_content_kind,
                outcome.manifest_current_content_kind,
                outcome.manifest_expected_content_kind,
                outcome.drift_status,
                outcome.structural_assertions_passed,
                if outcome.structural_assertion_details.is_empty() {
                    String::new()
                } else {
                    format!(" details={:?}", outcome.structural_assertion_details)
                }
            );
        }

        assert!(
            report.regression_lock_failures.is_empty(),
            "[{}] regression-lock failures (parser shape changed without manifest update?):\n{}",
            family,
            report.regression_lock_failures.join("\n")
        );
        assert!(
            report.aligned_samples_with_failed_assertions.is_empty(),
            "[{}] aligned samples with failed structural assertions:\n{}",
            family,
            report.aligned_samples_with_failed_assertions.join("\n")
        );
        assert!(
            report.passed(),
            "[{}] ast-shape contract did not pass; summary={}",
            family,
            report.summary_line()
        );
    }

    fn manifest_path(file: &str) -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test_data")
            .join("ast_shape_contract")
            .join(file)
    }

    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    #[test]
    fn regex_ast_shape_contract_holds_against_running_generated_parser() {
        use crate::ast_pipeline::runtime_logger_box;
        use crate::generated_parsers::regex::RegexParser;

        let path = manifest_path("regex_v1.json");
        let manifest = load_manifest(&path)
            .unwrap_or_else(|err| panic!("failed to load {}: {}", path.display(), err));

        let report = run_manifest(&manifest, |input| {
            let mut parser = RegexParser::new(input, runtime_logger_box("ast_shape_contract.regex"));
            parser.parse_full_regex().map_err(|err| err.to_string())
        });
        assert_report("regex", &report);
    }

    #[cfg(feature = "generated_parsers")]
    #[test]
    fn return_annotation_ast_shape_contract_holds_against_running_generated_parser() {
        use crate::ast_pipeline::runtime_logger_box;
        use crate::generated_parsers::return_annotation::ReturnAnnotationParser;

        let path = manifest_path("return_annotation_v1.json");
        let manifest = load_manifest(&path)
            .unwrap_or_else(|err| panic!("failed to load {}: {}", path.display(), err));

        let report = run_manifest(&manifest, |input| {
            let mut parser = ReturnAnnotationParser::new(
                input,
                runtime_logger_box("ast_shape_contract.return_annotation"),
            );
            parser
                .parse_full_return_annotation()
                .map_err(|err| err.to_string())
        });
        assert_report("return_annotation", &report);
    }

    #[cfg(feature = "generated_parsers")]
    #[test]
    fn semantic_annotation_ast_shape_contract_holds_against_running_generated_parser() {
        use crate::ast_pipeline::runtime_logger_box;
        use crate::generated_parsers::semantic_annotation::SemanticAnnotationParser;

        let path = manifest_path("semantic_annotation_v1.json");
        let manifest = load_manifest(&path)
            .unwrap_or_else(|err| panic!("failed to load {}: {}", path.display(), err));

        let report = run_manifest(&manifest, |input| {
            let mut parser = SemanticAnnotationParser::new(
                input,
                runtime_logger_box("ast_shape_contract.semantic_annotation"),
            );
            parser
                .parse_full_semantic_annotation()
                .map_err(|err| err.to_string())
        });
        assert_report("semantic_annotation", &report);
    }

    #[cfg(all(feature = "generated_parsers", has_generated_rtl_const_expr_parser))]
    #[test]
    fn rtl_const_expr_ast_shape_contract_holds_against_running_generated_parser() {
        use crate::ast_pipeline::runtime_logger_box;
        use crate::generated_parsers::rtl_const_expr::RtlConstExprParser;

        let path = manifest_path("rtl_const_expr_v1.json");
        let manifest = load_manifest(&path)
            .unwrap_or_else(|err| panic!("failed to load {}: {}", path.display(), err));

        let report = run_manifest(&manifest, |input| {
            let mut parser = RtlConstExprParser::new(
                input,
                runtime_logger_box("ast_shape_contract.rtl_const_expr"),
            );
            parser
                .parse_full_rtl_const_expr()
                .map_err(|err| err.to_string())
        });
        assert_report("rtl_const_expr", &report);
    }

    #[cfg(all(feature = "generated_parsers", has_generated_rtl_frontend_parser))]
    #[test]
    fn rtl_frontend_ast_shape_contract_holds_against_running_generated_parser() {
        use crate::ast_pipeline::runtime_logger_box;
        use crate::generated_parsers::rtl_frontend::RtlFrontendParser;

        let path = manifest_path("rtl_frontend_v1.json");
        let manifest = load_manifest(&path)
            .unwrap_or_else(|err| panic!("failed to load {}: {}", path.display(), err));

        let report = run_manifest(&manifest, |input| {
            let mut parser = RtlFrontendParser::new(
                input,
                runtime_logger_box("ast_shape_contract.rtl_frontend"),
            );
            parser
                .parse_full_rtl_frontend_file()
                .map_err(|err| err.to_string())
        });
        assert_report("rtl_frontend", &report);
    }

    /// SystemVerilog AST-shape contract. The generated SV parser is NOT in the
    /// default `cargo test --features generated_parsers` build; it's produced
    /// on-demand by `sv_stimuli_quality_gate` (and similar) into
    /// `rust/target/<gate>/work/systemverilog_parser.rs`. This cfg-gated test
    /// activates whenever the parser is present (gate run or
    /// `PGEN_SYSTEMVERILOG_PARSER_PATH` override) and stays compiled-out
    /// otherwise.
    #[cfg(all(feature = "generated_parsers", has_generated_systemverilog_parser))]
    #[test]
    fn systemverilog_ast_shape_contract_holds_against_running_generated_parser() {
        use crate::ast_pipeline::runtime_logger_box;
        use crate::generated_parsers::systemverilog::SystemverilogParser;

        let path = manifest_path("systemverilog_v1.json");
        let manifest = load_manifest(&path)
            .unwrap_or_else(|err| panic!("failed to load {}: {}", path.display(), err));

        let report = run_manifest(&manifest, |input| {
            let mut parser = SystemverilogParser::new(
                input,
                runtime_logger_box("ast_shape_contract.systemverilog"),
            );
            parser
                .parse_full_systemverilog_file()
                .map_err(|err| err.to_string())
        });
        assert_report("systemverilog", &report);
    }

    #[cfg(all(
        feature = "generated_parsers",
        has_generated_systemverilog_preprocessor_parser
    ))]
    #[test]
    fn systemverilog_preprocessor_ast_shape_contract_holds_against_running_generated_parser() {
        use crate::ast_pipeline::runtime_logger_box;
        use crate::generated_parsers::systemverilog_preprocessor::SystemverilogPreprocessorParser;

        let path = manifest_path("systemverilog_preprocessor_v1.json");
        let manifest = load_manifest(&path)
            .unwrap_or_else(|err| panic!("failed to load {}: {}", path.display(), err));

        let report = run_manifest(&manifest, |input| {
            let mut parser = SystemverilogPreprocessorParser::new(
                input,
                runtime_logger_box("ast_shape_contract.systemverilog_preprocessor"),
            );
            parser
                .parse_full_systemverilog_preprocessor_file()
                .map_err(|err| err.to_string())
        });
        assert_report("systemverilog_preprocessor", &report);
    }

    #[cfg(all(feature = "generated_parsers", has_generated_vhdl_parser))]
    #[test]
    fn vhdl_ast_shape_contract_holds_against_running_generated_parser() {
        use crate::ast_pipeline::runtime_logger_box;
        use crate::generated_parsers::vhdl::VhdlParser;

        let path = manifest_path("vhdl_v1.json");
        let manifest = load_manifest(&path)
            .unwrap_or_else(|err| panic!("failed to load {}: {}", path.display(), err));

        let report = run_manifest(&manifest, |input| {
            let mut parser = VhdlParser::new(input, runtime_logger_box("ast_shape_contract.vhdl"));
            parser.parse_full_vhdl_file().map_err(|err| err.to_string())
        });
        assert_report("vhdl", &report);
    }
}
