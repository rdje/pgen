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
    /// Declared-annotation inventory: every return annotation declared in the
    /// grammar source, normalized. The runner extracts the same inventory from
    /// the grammar's frontend JSON (or another source) at gate time and fails
    /// on count or normalized-text mismatch. Optional during the rollout — a
    /// missing inventory is tolerated for grammars whose frontend JSON is not
    /// yet tracked, but every grammar should eventually carry one.
    #[serde(default)]
    pub declared_annotation_inventory: Option<DeclaredAnnotationInventory>,
}

/// Tracked snapshot of every return annotation declared in the grammar.
/// `pipeline_inventory_artifact` names the path to the inventory artifact
/// the AST pipeline emits as a side-effect of `--generate-parser`. The gate
/// reads that artifact directly: single source of truth, no re-derivation.
/// The `annotations` list must match the artifact in count and
/// normalized-text terms exactly. `optional_grammar_json_crosscheck`, when
/// present, also runs the legacy raw_ast walker against the named JSON and
/// confirms the two extractors agree — a safety net against pipeline
/// implementation drift.
#[derive(Debug, Deserialize)]
pub struct DeclaredAnnotationInventory {
    pub pipeline_inventory_artifact: String,
    pub extracted_at: String,
    pub annotations: Vec<DeclaredAnnotation>,
    #[serde(default)]
    pub optional_grammar_json_crosscheck: Option<String>,
}

/// One declared annotation. `rule` and `branch_index` follow the
/// `extract_rule_annotations` semantics from the AST pipeline (group_depth
/// is honored, so `|` operators inside parentheses do not increment the
/// branch counter). `annotation_type` is one of `return_scalar`,
/// `return_array`, `return_object`. `normalized_text` is the annotation's
/// payload after `normalize_annotation_text` is applied.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DeclaredAnnotation {
    pub rule: String,
    pub branch_index: usize,
    pub annotation_type: String,
    pub normalized_text: String,
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

/// Normalize a return-annotation payload string for stable comparison. Trim
/// outer whitespace; collapse runs of whitespace inside the payload to a
/// single space; preserve characters inside string literals (quoted with
/// `"` or `'`) verbatim. The result is deterministic — two annotations that
/// differ only by inconsequential whitespace normalize to the same string,
/// while any meaningful edit (key rename, value change, structural change)
/// produces a different normalized form.
pub fn normalize_annotation_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_ws = false;
    let mut in_str = false;
    let mut quote: Option<char> = None;
    for ch in s.trim().chars() {
        if in_str {
            out.push(ch);
            if Some(ch) == quote {
                in_str = false;
                quote = None;
            }
            prev_ws = false;
        } else if ch == '"' || ch == '\'' {
            in_str = true;
            quote = Some(ch);
            out.push(ch);
            prev_ws = false;
        } else if ch.is_whitespace() {
            if !prev_ws {
                out.push(' ');
                prev_ws = true;
            }
        } else {
            out.push(ch);
            prev_ws = false;
        }
    }
    out.trim_end().to_string()
}

/// Extract the declared-annotation inventory from a grammar's frontend JSON
/// file (e.g. `generated/regex.json`). The walk mirrors the behavior of
/// `RustASTPipeline::extract_rule_annotations` in
/// [rust/src/ast_pipeline/mod.rs](rust/src/ast_pipeline/mod.rs), including
/// `group_depth` tracking so `|` operators inside parentheses do NOT
/// increment the branch counter. Annotations are returned in source order.
pub fn extract_declared_annotations_from_json<P: AsRef<Path>>(
    json_path: P,
) -> std::io::Result<Vec<DeclaredAnnotation>> {
    let raw = std::fs::read_to_string(json_path.as_ref())?;
    let value: serde_json::Value = serde_json::from_str(&raw).map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("frontend JSON deserialise failed: {}", err),
        )
    })?;
    let raw_ast = value
        .get("raw_ast")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "frontend JSON missing `raw_ast` array",
            )
        })?;

    let mut annotations = Vec::new();
    for rule_arr in raw_ast {
        let arr = match rule_arr.as_array() {
            Some(a) if !a.is_empty() => a,
            _ => continue,
        };
        let rule_name = match arr.first().and_then(|v| v.as_array()) {
            Some(first)
                if first.first().and_then(|v| v.as_str()) == Some("rule")
                    && first.get(1).and_then(|v| v.as_str()).is_some() =>
            {
                first.get(1).and_then(|v| v.as_str()).unwrap().to_string()
            }
            _ => continue,
        };

        // Mirror the outer-only branch counting used by
        // `crate::ast_pipeline::extract_rule_annotations` after the
        // 2026-05-14 remap fix: only `|` at group_depth == 0 creates a
        // new top-level branch (which is what the AST after
        // step2_group_by_or carries). Inner-group `|`s are accumulated
        // into the surrounding outer branch via the `last_closed_group_range`
        // broadcast — but since we now collapse to outer indices, the
        // broadcast resolves to a single outer slot.
        let mut group_depth: usize = 0;
        let mut outer_branch_index: usize = 0;

        for item in &arr[1..] {
            let item_arr = match item.as_array() {
                Some(a) if !a.is_empty() => a,
                _ => continue,
            };
            let tag = match item_arr.first().and_then(|v| v.as_str()) {
                Some(t) => t,
                None => continue,
            };

            match tag {
                "group_open" => {
                    group_depth = group_depth.saturating_add(1);
                }
                "group_close" => {
                    group_depth = group_depth.saturating_sub(1);
                }
                "operator" => {
                    if item_arr.get(1).and_then(|v| v.as_str()) == Some("|")
                        && group_depth == 0
                    {
                        outer_branch_index = outer_branch_index.saturating_add(1);
                    }
                }
                "return_scalar" | "return_array" | "return_object" => {
                    let text = item_arr
                        .get(1)
                        .and_then(|v| v.as_str())
                        .unwrap_or_default();
                    annotations.push(DeclaredAnnotation {
                        rule: rule_name.clone(),
                        branch_index: outer_branch_index,
                        annotation_type: tag.to_string(),
                        normalized_text: normalize_annotation_text(text),
                    });
                }
                _ => {}
            }
        }
    }
    // Match the sort order used by `EmittedReturnAnnotationInventory::from_annotations`
    // so the cross-extractor's output is byte-comparable with the pipeline's artifact.
    annotations.sort_by(|a, b| {
        a.rule
            .cmp(&b.rule)
            .then_with(|| a.branch_index.cmp(&b.branch_index))
    });
    Ok(annotations)
}

/// Read the pipeline-emitted inventory artifact at `path` and produce the
/// flat `DeclaredAnnotation` list the contract gate compares against. The
/// artifact format is produced by
/// [`crate::ast_pipeline::EmittedReturnAnnotationInventory`] during
/// `ast_pipeline --generate-parser`; this reader is the consumer side of
/// that contract.
pub fn read_pipeline_inventory_artifact<P: AsRef<Path>>(
    path: P,
) -> std::io::Result<Vec<DeclaredAnnotation>> {
    let raw = std::fs::read_to_string(path.as_ref())?;
    let parsed: crate::ast_pipeline::EmittedReturnAnnotationInventory =
        serde_json::from_str(&raw).map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("pipeline inventory artifact deserialise failed: {}", err),
            )
        })?;
    Ok(parsed
        .annotations
        .into_iter()
        .map(|entry| DeclaredAnnotation {
            rule: entry.rule,
            branch_index: entry.branch_index,
            annotation_type: entry.annotation_type,
            normalized_text: entry.normalized_text,
        })
        .collect())
}

/// Compare a manifest's tracked declared-annotation inventory against the
/// live extraction. Returns a list of human-readable mismatch lines suitable
/// for placement on `ContractReport.regression_lock_failures`. An empty list
/// means the manifest and the live source agree on every declared annotation.
pub fn diff_declared_annotation_inventory(
    manifest_inventory: &[DeclaredAnnotation],
    live_inventory: &[DeclaredAnnotation],
) -> Vec<String> {
    let mut mismatches = Vec::new();

    if manifest_inventory.len() != live_inventory.len() {
        mismatches.push(format!(
            "declared annotation count mismatch: manifest tracks {}, grammar declares {}",
            manifest_inventory.len(),
            live_inventory.len()
        ));
    }

    let pair_count = manifest_inventory.len().min(live_inventory.len());
    for idx in 0..pair_count {
        let m = &manifest_inventory[idx];
        let l = &live_inventory[idx];
        if m != l {
            mismatches.push(format!(
                "declared annotation [{}] mismatch:\n  manifest: rule={:?} branch={} type={} text={:?}\n  grammar:  rule={:?} branch={} type={} text={:?}",
                idx,
                m.rule, m.branch_index, m.annotation_type, m.normalized_text,
                l.rule, l.branch_index, l.annotation_type, l.normalized_text,
            ));
        }
    }

    if live_inventory.len() > manifest_inventory.len() {
        for (idx, ann) in live_inventory.iter().enumerate().skip(manifest_inventory.len()) {
            mismatches.push(format!(
                "declared annotation [{}] present in grammar but missing from manifest: rule={:?} branch={} type={} text={:?}",
                idx, ann.rule, ann.branch_index, ann.annotation_type, ann.normalized_text
            ));
        }
    } else if manifest_inventory.len() > live_inventory.len() {
        for (idx, ann) in manifest_inventory.iter().enumerate().skip(live_inventory.len()) {
            mismatches.push(format!(
                "declared annotation [{}] tracked in manifest but missing from grammar: rule={:?} branch={} type={} text={:?}",
                idx, ann.rule, ann.branch_index, ann.annotation_type, ann.normalized_text
            ));
        }
    }

    mismatches
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
///
/// When the manifest carries a `declared_annotation_inventory`, the runner
/// also extracts the live inventory from the named frontend JSON and adds
/// any count or normalized-text discrepancy to
/// `ContractReport.regression_lock_failures`. This catches the case where a
/// new return annotation is added to the grammar without an explicit
/// manifest update — the gate fails until the manifest matches the grammar
/// again.
pub fn run_manifest<F>(
    manifest: &AstShapeContractManifest,
    mut parse_sample: F,
) -> ContractReport
where
    F: for<'input> FnMut(&'input str) -> Result<ParseNode<'input>, String>,
{
    let mut report = ContractReport::default();

    if let Some(inventory) = &manifest.declared_annotation_inventory {
        let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap_or_else(|| std::path::Path::new(""))
            .to_path_buf();

        let artifact_path = repo_root.join(&inventory.pipeline_inventory_artifact);
        match read_pipeline_inventory_artifact(&artifact_path) {
            Ok(live) => {
                for diff in diff_declared_annotation_inventory(&inventory.annotations, &live) {
                    report.regression_lock_failures.push(format!(
                        "declared-annotation inventory check failed for grammar {} (pipeline artifact {}): {}",
                        manifest.grammar,
                        artifact_path.display(),
                        diff
                    ));
                }
            }
            Err(err) => {
                report.regression_lock_failures.push(format!(
                    "declared-annotation inventory: failed to read pipeline artifact {}: {} (regenerate the parser to refresh the artifact)",
                    artifact_path.display(),
                    err
                ));
            }
        }

        if let Some(crosscheck_json) = &inventory.optional_grammar_json_crosscheck {
            let json_path = repo_root.join(crosscheck_json);
            match extract_declared_annotations_from_json(&json_path) {
                Ok(crosscheck) => {
                    for diff in diff_declared_annotation_inventory(&inventory.annotations, &crosscheck) {
                        report.regression_lock_failures.push(format!(
                            "declared-annotation crosscheck failed for grammar {} (frontend JSON {}): {} (this means the pipeline's inventory-emit path and its raw_ast walk disagree — investigate)",
                            manifest.grammar,
                            json_path.display(),
                            diff
                        ));
                    }
                }
                Err(err) => {
                    report.regression_lock_failures.push(format!(
                        "declared-annotation crosscheck: failed to read frontend JSON {}: {}",
                        json_path.display(),
                        err
                    ));
                }
            }
        }
    }

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
