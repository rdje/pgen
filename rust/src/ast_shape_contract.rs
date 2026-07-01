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

        // Mirror the branch bookkeeping of
        // `crate::ast_pipeline::extract_rule_annotations` exactly (the
        // 2026-05-14 outer remap + the BRANCH-BROADCAST-FIX.2 whole-body-group
        // refinement): inner branches are counted for EVERY `|` so a trailing
        // annotation after a `group_close` can broadcast across the just-closed
        // group's branch range; the inner indices are then remapped to runtime
        // branch indices — `|` at depth 0 (`branch_to_outer`) for ordinary
        // rules, `|` at depth <= 1 (`branch_to_body`) when the rule body is
        // exactly one whole top-level group (step2_group_by_or unwraps that
        // group's alternatives into the rule's own runtime branches). The last
        // annotation per mapped slot wins, matching the pipeline's "multiple
        // return annotations in branch — keeping last" semantics.
        let mut group_depth: usize = 0;
        let mut branch_idx: usize = 0;
        let mut outer_branch_idx: usize = 0;
        let mut body_branch_idx: usize = 0;
        let mut branch_to_outer: Vec<usize> = vec![0];
        let mut branch_to_body: Vec<usize> = vec![0];
        let mut group_open_branch_stack: Vec<usize> = Vec::new();
        let mut last_closed_group_range: Option<(usize, usize)> = None;
        // Inner-indexed annotation slots: (annotation_type, raw_text).
        let mut slots: Vec<Option<(String, String)>> = vec![None];
        // The annotation-free syntax token list, mirrored so the shared
        // whole-body-group discriminator sees what the pipeline sees.
        let mut syntax_tokens: Vec<serde_json::Value> = Vec::new();

        for item in &arr[1..] {
            let item_arr = match item.as_array() {
                Some(a) if !a.is_empty() => a,
                _ => {
                    // Pipeline pushes non-array items into syntax_elements
                    // and clears the pending broadcast range.
                    syntax_tokens.push(item.clone());
                    last_closed_group_range = None;
                    continue;
                }
            };
            let tag = match item_arr.first().and_then(|v| v.as_str()) {
                Some(t) => t,
                None => {
                    syntax_tokens.push(item.clone());
                    last_closed_group_range = None;
                    continue;
                }
            };

            match tag {
                "group_open" => {
                    group_open_branch_stack.push(branch_idx);
                    group_depth = group_depth.saturating_add(1);
                    syntax_tokens.push(item.clone());
                    last_closed_group_range = None;
                }
                "group_close" => {
                    group_depth = group_depth.saturating_sub(1);
                    last_closed_group_range = group_open_branch_stack
                        .pop()
                        .map(|open_branch_idx| (open_branch_idx, branch_idx));
                    syntax_tokens.push(item.clone());
                }
                "operator" => {
                    let is_pipe = item_arr.get(1).and_then(|v| v.as_str()) == Some("|");
                    if is_pipe {
                        branch_idx = branch_idx.saturating_add(1);
                        if group_depth == 0 {
                            outer_branch_idx = outer_branch_idx.saturating_add(1);
                        }
                        if group_depth <= 1 {
                            body_branch_idx = body_branch_idx.saturating_add(1);
                        }
                        if branch_to_outer.len() <= branch_idx {
                            branch_to_outer.push(outer_branch_idx);
                        }
                        if branch_to_body.len() <= branch_idx {
                            branch_to_body.push(body_branch_idx);
                        }
                        if slots.len() <= branch_idx {
                            slots.push(None);
                        }
                        last_closed_group_range = None;
                    }
                    syntax_tokens.push(item.clone());
                    if !is_pipe {
                        last_closed_group_range = None;
                    }
                }
                "return_scalar" | "return_array" | "return_object" => {
                    let text = item_arr
                        .get(1)
                        .and_then(|v| v.as_str())
                        .unwrap_or_default();
                    let (range_start, range_end) = match last_closed_group_range {
                        Some((s, e)) => (s, e),
                        None => (branch_idx, branch_idx),
                    };
                    if slots.len() <= range_end {
                        slots.resize(range_end + 1, None);
                    }
                    for slot in slots.iter_mut().take(range_end + 1).skip(range_start) {
                        *slot = Some((tag.to_string(), text.to_string()));
                    }
                    last_closed_group_range = None;
                }
                // Annotation tokens are not syntax: they neither enter the
                // discriminator's token list nor clear the pending broadcast
                // range (mirrors the pipeline's arms exactly).
                "semantic_annotation"
                | "semantic_annotation_inline"
                | "semantic_annotation_mid_sequence"
                | "lexical_annotation" => {}
                _ => {
                    syntax_tokens.push(item.clone());
                    last_closed_group_range = None;
                }
            }
        }

        let whole_body_group =
            crate::ast_pipeline::syntax_is_single_whole_body_group(&syntax_tokens);
        let (branch_map, mapped_count) = if whole_body_group {
            (&branch_to_body, body_branch_idx + 1)
        } else {
            (&branch_to_outer, outer_branch_idx + 1)
        };
        let mut mapped_slots: Vec<Option<(String, String)>> = vec![None; mapped_count];
        for (i, slot) in slots.into_iter().enumerate() {
            if let Some(entry) = slot {
                let mapped_idx = branch_map.get(i).copied().unwrap_or(0);
                if mapped_idx < mapped_slots.len() {
                    mapped_slots[mapped_idx] = Some(entry);
                }
            }
        }
        for (branch_index, slot) in mapped_slots.into_iter().enumerate() {
            if let Some((annotation_type, text)) = slot {
                annotations.push(DeclaredAnnotation {
                    rule: rule_name.clone(),
                    branch_index,
                    annotation_type,
                    normalized_text: normalize_annotation_text(&text),
                });
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
/// receives a sample's input AND its `rule_under_test`, and returns the
/// top-level `ParseNode` for that sample. Passing the rule lets a grammar's
/// callback parse the input as a *non-root* entry rule (e.g. a nested rule
/// whose own return annotation is the carrier under test), not only the
/// whole-file root — this is how the SystemVerilog UDP truth-table entries
/// (`combinational_entry` / `sequential_entry`) are shape-locked. Callbacks
/// that only ever parse the root may ignore the rule argument. The runner
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
    F: for<'input> FnMut(&'input str, &str) -> Result<ParseNode<'input>, String>,
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
        let parsed = match parse_sample(&sample.input, &sample.rule_under_test) {
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

    // BRANCH-BROADCAST-FIX.2 — the cross-extractor mirrors the pipeline's
    // whole-body-group broadcast: a trailing annotation on `( A | B )` yields
    // one inventory row PER runtime branch, while the documented
    // disambiguations (`(A|B) | C -> ann` → last branch only; quantified
    // groups → rule-level slot 0) keep their pre-fix rows.
    #[test]
    fn declared_annotation_crosscheck_broadcasts_whole_body_group_rows() {
        let json = serde_json::json!({
            "raw_ast": [
                [
                    ["rule", "string_literal"],
                    ["group_open", "("],
                    ["quoted_string", "\""],
                    ["rule_reference", "dq_body"],
                    ["quoted_string", "\""],
                    ["operator", "|"],
                    ["quoted_string", "'"],
                    ["rule_reference", "sq_body"],
                    ["quoted_string", "'"],
                    ["group_close", ")"],
                    ["return_object", "{type: \"string\", value: $2}"]
                ],
                [
                    ["rule", "mixed"],
                    ["group_open", "("],
                    ["rule_reference", "a"],
                    ["operator", "|"],
                    ["rule_reference", "b"],
                    ["group_close", ")"],
                    ["operator", "|"],
                    ["rule_reference", "c"],
                    ["return_scalar", "$1"]
                ],
                [
                    ["rule", "quantified"],
                    ["group_open", "("],
                    ["rule_reference", "a"],
                    ["operator", "|"],
                    ["rule_reference", "b"],
                    ["group_close", ")"],
                    ["operator", "*"],
                    ["return_scalar", "$1"]
                ]
            ]
        });
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("crosscheck_probe.json");
        std::fs::write(&path, serde_json::to_string_pretty(&json).unwrap()).expect("write probe");

        let rows = extract_declared_annotations_from_json(&path).expect("extraction");

        let by_rule = |rule: &str| -> Vec<(usize, String)> {
            rows.iter()
                .filter(|r| r.rule == rule)
                .map(|r| (r.branch_index, r.annotation_type.clone()))
                .collect()
        };

        // Whole-body group: one row per runtime branch (the broadcast).
        assert_eq!(
            by_rule("string_literal"),
            vec![
                (0usize, "return_object".to_string()),
                (1usize, "return_object".to_string())
            ],
            "whole-body group trailing annotation must broadcast to both branches"
        );
        // `(A|B) | C -> ann`: ann on the LAST top-level branch only.
        assert_eq!(
            by_rule("mixed"),
            vec![(1usize, "return_scalar".to_string())],
            "mixed-shape annotation binds the last top-level branch only"
        );
        // `( A | B )* -> ann`: quantified group is a sub-part — rule-level slot 0.
        assert_eq!(
            by_rule("quantified"),
            vec![(0usize, "return_scalar".to_string())],
            "quantified-group rule keeps the single rule-level row"
        );
    }

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

        let report = run_manifest(&manifest, |input, _rule| {
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

        let report = run_manifest(&manifest, |input, _rule| {
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

        let report = run_manifest(&manifest, |input, _rule| {
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

        let report = run_manifest(&manifest, |input, _rule| {
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

        let report = run_manifest(&manifest, |input, _rule| {
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

        let report = run_manifest(&manifest, |input, rule| {
            let mut parser = SystemverilogParser::new(
                input,
                runtime_logger_box("ast_shape_contract.systemverilog"),
            );
            // Non-root `rule_under_test` support (GRAMMAR-WELLFORMED.H.14.3.1):
            // a sample may lock the shape of a *nested* rule by naming it as
            // `rule_under_test`, in which case we parse the input AS that entry
            // rule. The carrier comes from that rule's own return annotation, so
            // a standalone parse exhibits the same typed↔raw shape it has in
            // context. This shape-locks the UDP truth-table entries whose typed
            // `{inputs,…}` carriers silently regressed to raw `Sequence` from
            // SV-Slice-66 until H.14.3 (ledger SV-0009) — so the class cannot
            // recur unnoticed. Unrecognized rules fall back to the whole-file root.
            match rule {
                "combinational_entry" => parser
                    .parse_combinational_entry()
                    .map_err(|err| err.to_string()),
                "sequential_entry" => parser
                    .parse_sequential_entry()
                    .map_err(|err| err.to_string()),
                // SV-AST-SHAPE-FIDELITY.1: lock the ANSI-port shape at its own
                // entry rules so the `ansi_port_declaration` inline-alternation-`$1`
                // corruption (typed ports emitted `<invalid_sequence_access>` for
                // the `header` sub-shape) cannot recur unnoticed. `ansi_port_header`
                // is the corruption-site lock — its clean `{direction, port_type}`
                // root keys differ from the corrupted `{kind,header,default,dims,name}`.
                "ansi_port_declaration" => parser
                    .parse_ansi_port_declaration()
                    .map_err(|err| err.to_string()),
                "ansi_port_header" => parser
                    .parse_ansi_port_header()
                    .map_err(|err| err.to_string()),
                // SV-AST-SHAPE-FIDELITY.2.1: lock the forward-typedef keyword shape
                // at its own entry rules so the `type_declaration_{sv_2017,sv_2023}`
                // br6 inline-alternation-`$2` corruption (`typedef enum e_t;` emitted
                // `<invalid_sequence_access>` in the `keyword` sub-shape) cannot recur
                // unnoticed. `forward_type_keyword` is the corruption-site + compile-time
                // revert guard — its existence is the named-lift; removing it (reverting
                // to the inline alternation) fails `parse_forward_type_keyword()` to compile.
                "forward_type_keyword" => parser
                    .parse_forward_type_keyword()
                    .map_err(|err| err.to_string()),
                "type_declaration_sv_2017" => parser
                    .parse_type_declaration_sv_2017()
                    .map_err(|err| err.to_string()),
                // SV-AST-SHAPE-FIDELITY.2.2: lock the base-class-type shape at its
                // own entry rules so the `base_class_type` inline-alternation-`$1`
                // corruption (`class D extends pkg::B;` emitted
                // `<invalid_sequence_access>` for the `params`/`scope_chain`
                // sub-shapes) cannot recur unnoticed. `base_class_type_head` is the
                // corruption-site + compile-time revert guard — removing it (reverting
                // to the inline alternation) fails `parse_base_class_type_head()` to
                // compile. Mirrors the already-correct sibling `class_type_head`.
                "base_class_type_head" => parser
                    .parse_base_class_type_head()
                    .map_err(|err| err.to_string()),
                "base_class_type" => parser
                    .parse_base_class_type()
                    .map_err(|err| err.to_string()),
                // SV-AST-SHAPE-FIDELITY.2.3: lock the scoped-type-reference cluster
                // (candidates #1/#5/#6). `module m; C::D::E v; endmodule` emitted 4
                // `<invalid_sequence_access>` sentinels from TWO inline-alternation-`$N`
                // corruptions: `class_scope_type` (head:$1 over a 4-way alt →
                // `params`/`scope_chain` corrupt) and `scoped_data_type_identifier` /
                // `scoped_block_type_identifier` (scope:$1 over the shared alt
                // ( class_scope | non_typedef_package_scope ) → `type`/`dims` corrupt).
                // `class_scope_type_head` + `scoped_type_scope_prefix` are the
                // corruption-site + compile-time revert guards — removing either
                // named-lift (reverting to the inline alternation) fails its
                // `parse_*()` to compile. Mirrors the `base_class_type_head` idiom.
                "class_scope_type_head" => parser
                    .parse_class_scope_type_head()
                    .map_err(|err| err.to_string()),
                "class_scope_type" => parser
                    .parse_class_scope_type()
                    .map_err(|err| err.to_string()),
                "scoped_type_scope_prefix" => parser
                    .parse_scoped_type_scope_prefix()
                    .map_err(|err| err.to_string()),
                "scoped_data_type_identifier" => parser
                    .parse_scoped_data_type_identifier()
                    .map_err(|err| err.to_string()),
                _ => parser
                    .parse_full_systemverilog_file()
                    .map_err(|err| err.to_string()),
            }
        });
        assert_report("systemverilog", &report);
    }

    /// `SV-EXH-PROOF.3.3.4.b.6.1.1`: the `@fact_kind:` declarations in
    /// `systemverilog.ebnf` must reach the generated parser's
    /// `CompiledSemanticRuntimeAnnotations` registry. Before `.b.6.1.1` the
    /// codegen never serialised `fact_kinds` — the registry was populated only
    /// on the compile-time path — so a generated parser always had an empty
    /// registry. This test pins the end-to-end producer-pass wiring.
    #[cfg(all(feature = "generated_parsers", has_generated_systemverilog_parser))]
    #[test]
    fn systemverilog_fact_kind_registry_is_populated_in_generated_parser() {
        use crate::ast_pipeline::runtime_logger_box;
        use crate::generated_parsers::systemverilog::SystemverilogParser;

        let parser = SystemverilogParser::new(
            "",
            runtime_logger_box("ast_shape_contract.systemverilog.fact_kinds"),
        );
        let annotations = parser.semantic_runtime_annotations();
        assert_eq!(
            annotations.fact_kinds_len(),
            4,
            "expected the 4 @fact_kind: declarations (type_name, variable_binding, \
             type_binding, wildcard_import_open)",
        );
        let type_name = annotations
            .fact_kind("type_name")
            .expect("type_name fact-kind must be declared");
        assert!(
            type_name.exportable,
            "type_name must stay exportable so package_declaration @export_to_library keeps \
             exporting it — the veer cross-file no-regression guard",
        );
        assert!(
            annotations.fact_kind("variable_binding").is_some(),
            "variable_binding fact-kind must be declared",
        );
        assert!(
            annotations.fact_kind("type_binding").is_some(),
            "type_binding fact-kind must be declared",
        );
        // SV-PARSE-STRICT.2: the wildcard-import marker that gates sound net-type
        // acceptance. Declared NON-exportable (a scope-local parse-phase marker),
        // so the exportable-kind set below stays exactly {type_name}.
        let wildcard_import_open = annotations
            .fact_kind("wildcard_import_open")
            .expect("wildcard_import_open fact-kind must be declared");
        assert!(
            !wildcard_import_open.exportable,
            "wildcard_import_open is a scope-local marker; it must NOT be exportable",
        );
        // Veer no-regression proof: with a declared schema, exportable_fact_kinds()
        // must still resolve to exactly {type_name} — byte-identical to the
        // pre-.b.6.1 MVP-0 default the package_declaration export relied on.
        let exportable = annotations.exportable_fact_kinds();
        assert_eq!(
            exportable.len(),
            1,
            "exactly one exportable kind expected; got {:?}",
            exportable,
        );
        assert!(
            exportable.contains("type_name"),
            "type_name must be the (only) exportable kind",
        );
    }

    /// `SV-EXH-PROOF.3.3.4.b.6.1.2`: the minimal producer — `@emit_fact` on
    /// `variable_decl_assignment` must emit one `variable_binding` fact per
    /// declared variable. The decl-site element rule executes once per
    /// variable, so `int alpha, beta;` produces two facts with no fan-out
    /// machinery. This pins the producer end-to-end.
    #[cfg(all(feature = "generated_parsers", has_generated_systemverilog_parser))]
    #[test]
    fn systemverilog_variable_decl_emits_variable_binding_facts() {
        use crate::ast_pipeline::runtime_logger_box;
        use crate::generated_parsers::systemverilog::SystemverilogParser;

        let mut parser = SystemverilogParser::new(
            "module m; int alpha, beta; endmodule",
            runtime_logger_box("ast_shape_contract.systemverilog.variable_binding"),
        );
        parser
            .parse_full_systemverilog_file()
            .expect("module with a two-variable data declaration must parse");

        let names: Vec<String> = parser
            .semantic_runtime_state()
            .facts()
            .iter()
            .filter(|fact| fact.kind == "variable_binding")
            .filter_map(|fact| fact.name.as_text().map(|text| text.to_string()))
            .collect();
        // Natural per-element fan-out: one fact per variable.
        assert!(
            names.iter().any(|n| n == "alpha") && names.iter().any(|n| n == "beta"),
            "expected variable_binding facts for both `alpha` and `beta`; got {:?}",
            names,
        );
    }

    /// `SV-EXH-PROOF.3.3.4.b.6.2`: the context-gated consumer. A 3-level
    /// method chain `a.b.c(x)` failed to parse before `.b.6.2` (the
    /// `.b.4`-diagnosed `call_primary` no-chain path). The new
    /// `context_member_method_call` branch parses it when the chain head is a
    /// known declared variable (`has_fact(variable_binding, $head)`). This
    /// pins the producer→consumer loop end-to-end.
    #[cfg(all(feature = "generated_parsers", has_generated_systemverilog_parser))]
    #[test]
    fn systemverilog_context_gated_method_chain_parses_with_known_variable_head() {
        use crate::ast_pipeline::runtime_logger_box;
        use crate::generated_parsers::systemverilog::SystemverilogParser;

        // `a` is a declared variable → a `variable_binding` fact is emitted
        // before the `if`, so the context-gated branch fires and the 3-level
        // chain `a.b.c(x)` parses (it did NOT before `.b.6.2`).
        let source = "module m; int a; initial if (a.b.c(x)) ; endmodule";
        let mut parser =
            SystemverilogParser::new(source, runtime_logger_box("ast_shape_contract.sv.ctx_chain"));
        assert!(
            parser.parse_full_systemverilog_file().is_ok(),
            "a 3-level method chain on a known-variable head must parse",
        );
    }

    /// `SV-EXH-PROOF.3.3.4.b.6.2`: the context-gated branch handles the
    /// 3-level method chain in its negated form and in the exact uvm shape
    /// (`if(!seed_map.seed_table.exists(type_id))` inside a function, with a
    /// class-typed receiver declared locally). All three failed before
    /// `.b.6.2`.
    #[cfg(all(feature = "generated_parsers", has_generated_systemverilog_parser))]
    #[test]
    fn systemverilog_context_gated_method_chain_handles_negated_and_uvm_shape() {
        use crate::ast_pipeline::runtime_logger_box;
        use crate::generated_parsers::systemverilog::SystemverilogParser;

        let cases: &[(&str, &str)] = &[
            ("negated 3-level", "module m; int a; initial if (!a.b.c(x)) ; endmodule"),
            (
                "uvm-shaped function",
                // `uvm_seed_map` is declared via a real-UVM-style `typedef
                // class` forward declaration so the gated
                // `provisional_unscoped_block_class_type` rule (added in
                // SV-EXH-PROOF.3.3.4.b.6.2.35.1) sees a `type_name` fact for
                // it. The test's purpose remains 3-level chain parsing in the
                // exact uvm shape; the typedef just makes the type lookup
                // realistic (UVM heavily uses typedef-class forward decls).
                "module m; typedef class uvm_seed_map; \
                 function void f(); uvm_seed_map seed_map; \
                 if(!seed_map.seed_table.exists(type_id)) begin end endfunction endmodule",
            ),
        ];
        for (label, src) in cases {
            let mut parser =
                SystemverilogParser::new(src, runtime_logger_box("ast_shape_contract.sv.ctx_chain"));
            assert!(
                parser.parse_full_systemverilog_file().is_ok(),
                "context-gated method chain must parse: {}",
                label,
            );
        }
    }

    /// `SV-EXH-PROOF.3.3.4.b.6.2.7`: diagnostic dump-facts test for C3.
    /// Parses the typedef-of-TYPE-parameter form and prints all emitted facts
    /// to confirm whether a `type_name{TYPE, declaration_family:typedef}` fact
    /// is leaked by the typedef parse path. Non-asserting — output is observed
    /// via `--nocapture` to characterise the fact-store state.
    #[cfg(all(feature = "generated_parsers", has_generated_systemverilog_parser))]
    #[test]
    fn systemverilog_b627_diag_typedef_type_parameter_fact_dump() {
        use crate::ast_pipeline::runtime_logger_box;
        use crate::generated_parsers::systemverilog::SystemverilogParser;

        // Three controlled inputs to isolate WHERE the spurious typedef fact
        // on the RHS comes from.
        let cases: &[(&str, &str)] = &[
            ("A: bare-typedef, no class    ", "package p; typedef int t; endpackage"),
            ("B: typedef refs type_param   ", "package p; class C #(type TYPE=int); typedef TYPE T; endclass endpackage"),
            ("C: typedef refs ordinary id  ", "package p; class C; typedef bit U; typedef U V; endclass endpackage"),
        ];
        for (label, src) in cases {
            let mut parser =
                SystemverilogParser::new(src, runtime_logger_box("ast_shape_contract.sv.b627"));
            let parsed = parser.parse_full_systemverilog_file().is_ok();
            let facts = parser.semantic_runtime_state().facts();
            println!("\nb627-diag [{}] parse_ok={} fact_count={}", label, parsed, facts.len());
            let mut by_name: std::collections::BTreeMap<String, Vec<String>> = Default::default();
            for fact in facts {
                let name_text = fact.name.as_text().map(|s| s.to_string())
                    .unwrap_or_else(|| format!("{:?}", fact.name));
                let fam = fact
                    .attributes
                    .iter()
                    .find(|p| p.key == "declaration_family")
                    .map(|p| format!("{:?}", p.value))
                    .unwrap_or_else(|| "<none>".to_string());
                by_name.entry(name_text).or_default().push(fam);
            }
            for (name, fams) in &by_name {
                let mut tally: std::collections::BTreeMap<String, usize> = Default::default();
                for f in fams {
                    *tally.entry(f.clone()).or_insert(0) += 1;
                }
                let summary: Vec<String> =
                    tally.iter().map(|(f, c)| format!("{}×{}", f, c)).collect();
                println!("  {} → {}", name, summary.join(", "));
            }
        }
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

        let report = run_manifest(&manifest, |input, _rule| {
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

        let report = run_manifest(&manifest, |input, _rule| {
            let mut parser = VhdlParser::new(input, runtime_logger_box("ast_shape_contract.vhdl"));
            parser.parse_full_vhdl_file().map_err(|err| err.to_string())
        });
        assert_report("vhdl", &report);
    }

    // GRAMMAR-WELLFORMED.H.11.5 regression lock (PGEN-GRAMMAR-WELLFORMED-0075):
    // the layout skipper's hard-coded comment arms are now suppressed at EMIT
    // time for any introducer a grammar token can start with. In SV, `#` is a
    // real token (delays, parameter lists) and `//`//`/*` comments are
    // grammar-owned by `trivia`; pre-fix, while the parser speculatively
    // attempted `line_comment` after `interface i`, the skipper's `#` arm
    // (whose H.11.3 dynamic guard only protects the ACTIVE token) swallowed
    // `#  (  ) ;timeunit 09 ns//>Mg` to end-of-line as a "comment", the bogus
    // trivia span was memoized, and the real ANSI-header `#` died on the
    // poisoned memo — rejecting grammar-valid input on every shipped SV
    // release.
    #[cfg(all(feature = "generated_parsers", has_generated_systemverilog_parser))]
    #[test]
    fn systemverilog_hash_token_is_not_stolen_by_comment_arms_during_speculation() {
        use crate::ast_pipeline::runtime_logger_box;
        use crate::generated_parsers::systemverilog::SystemverilogParser;

        let samples = [
            // The minimal repro family (bisected from the canonical seed-0
            // cert-coverage failing sample): an ANSI `#( )` header plus a
            // line comment followed by another comment before the
            // timeunit-ratio `/`.
            "interface i #  (  ) ;timeunit 09 ns//>Mg\n//QF.\n/633 s;endinterface",
            // Controls: single-comment / no-`#()` / comment-free variants
            // must keep parsing.
            "interface i #  (  ) ;timeunit 09 ns//>Mg\n/633 s;endinterface",
            "interface i ;timeunit 09 ns//>Mg\n//QF.\n/633 s;endinterface",
            "interface i #  (  ) ;timeunit 09 ns/633 s;endinterface",
        ];
        for sample in samples {
            let mut parser = SystemverilogParser::new(
                sample,
                runtime_logger_box("ast_shape_contract.sv_hash_not_stolen"),
            );
            parser.set_grammar_profile(Some("sv_2017"));
            assert!(
                parser.parse_full_systemverilog_file().is_ok(),
                "SV parser rejected grammar-valid `#`/comment sample: {sample}"
            );
        }
    }

    /// GRAMMAR-WELLFORMED.H.12.5.5.3.3.4.2.2.1 regression lock (ledger `SV-0003`,
    /// release 1.0.141): a module-scope class-handle declaration `C a;` (C a
    /// declared class) must parse as a `data_declaration` (emitting a
    /// `variable_binding` fact for `a`), NOT as a `net_declaration`. Before the
    /// fix, `checked_nettype_identifier`'s gate was the under-specified
    /// `has_fact(type_name, $body)` — a class is also a `type_name`, so `C a;`
    /// was consumed by the `net_declaration` user-nettype branch, emitting no
    /// `variable_binding` and so blocking class-handle member-method chains
    /// (the store-gated `context_member_method_call` needs the head bound). The
    /// gate is now `fact_attribute_equals(type_name, $body, declaration_family,
    /// nettype)`, so only a declared nettype matches and `C a;` routes to
    /// `data_declaration`. This pins: (1) `C a;` binds `a`; (2) the module-scope
    /// class-handle indexed member-method chain parses; (3) a real user nettype
    /// `nettype logic NT; NT a;` still parses (the gate was not over-tightened).
    #[cfg(all(feature = "generated_parsers", has_generated_systemverilog_parser))]
    #[test]
    fn systemverilog_class_handle_decl_is_data_declaration_not_net() {
        use crate::ast_pipeline::runtime_logger_box;
        use crate::generated_parsers::systemverilog::SystemverilogParser;

        // (1) `C a;` at module scope must parse AND emit a `variable_binding`
        // fact for `a` — the signature of the `data_declaration`/`variable_decl`
        // path. A `net_declaration` (the pre-fix mis-route) emits no such fact.
        let mut parser = SystemverilogParser::new(
            "class C; endclass module m; C a; endmodule",
            runtime_logger_box("ast_shape_contract.sv_class_handle_decl"),
        );
        parser.set_grammar_profile(Some("sv_2017"));
        parser
            .parse_full_systemverilog_file()
            .expect("a module-scope class-handle declaration `C a;` must parse");
        let bound_a = parser
            .semantic_runtime_state()
            .facts()
            .iter()
            .filter(|fact| fact.kind == "variable_binding")
            .filter_map(|fact| fact.name.as_text().map(|t| t.to_string()))
            .any(|n| n == "a");
        assert!(
            bound_a,
            "`C a;` (C a declared class) must bind `a` as a variable (data_declaration), \
             not be consumed as a net_declaration",
        );

        // (2) the module-scope class-handle indexed member-method chain must
        // parse (it was REJECTED pre-fix because `a` was never bound).
        // (3) a real declared user nettype must still parse (gate not over-tightened).
        let must_parse = [
            "class C; endclass module m; C a; int x; initial x = a.b[0].c(); endmodule",
            "nettype logic NT;\nmodule m; NT a; endmodule",
            "module m; wire a; endmodule",
        ];
        for sample in must_parse {
            let mut parser = SystemverilogParser::new(
                sample,
                runtime_logger_box("ast_shape_contract.sv_class_handle_decl"),
            );
            parser.set_grammar_profile(Some("sv_2017"));
            assert!(
                parser.parse_full_systemverilog_file().is_ok(),
                "SV parser rejected grammar-valid sample: {sample}",
            );
        }
    }

    /// GRAMMAR-WELLFORMED.H.12.5.5.3.3.3 regression lock (ledger `SV-0004`,
    /// release 1.0.142): the `boolean_abbrev` sequence-repetition family carries
    /// its IEEE 1800 §A.8.1 literal `[ ]` brackets — `consecutive_repetition`
    /// (`[* N]` / `[*]` / `[+]`), `goto_repetition` (`[-> N]`), and
    /// `non_consecutive_repetition` (`[= N]`). Extraction had dropped the `[ ]`,
    /// so the bracket-less forms collided with the `*`/`->`/`=` operators: the
    /// bracketed LRM forms were REJECTED and `goto_repetition` was
    /// operator-shadowed (never witnessed → cert `UNKNOWN`). This pins: (1) every
    /// bracketed sequence-repetition form parses; (2) a bare `a *3` still parses
    /// (as a multiplication expression — the fix drops the bare sequence-repetition
    /// spelling, not multiply, so nothing valid regresses).
    #[cfg(all(feature = "generated_parsers", has_generated_systemverilog_parser))]
    #[test]
    fn systemverilog_sequence_repetition_requires_lrm_brackets() {
        use crate::ast_pipeline::runtime_logger_box;
        use crate::generated_parsers::systemverilog::SystemverilogParser;

        let must_parse = [
            // (1) the LRM-bracketed sequence-repetition forms (§A.8.1) — REJECTED
            // at all releases <= 1.0.141 because the `[ ]` had been dropped.
            "module m; sequence s; a [*3]; endsequence endmodule", // consecutive_repetition [* N]
            "module m; sequence s; a [*]; endsequence endmodule",  // consecutive_repetition [*]
            "module m; sequence s; a [+]; endsequence endmodule",  // consecutive_repetition [+]
            "module m; sequence s; a [->2]; endsequence endmodule", // goto_repetition [-> N]
            "module m; sequence s; a [=2]; endsequence endmodule", // non_consecutive_repetition [= N]
            // (2) no-regression: a bare `a *3` is a multiplication expression and
            // must still parse (the fix removes only the bare repetition spelling).
            "module m; sequence s; a *3; endsequence endmodule",
        ];
        for sample in must_parse {
            let mut parser = SystemverilogParser::new(
                sample,
                runtime_logger_box("ast_shape_contract.sv_sequence_repetition_brackets"),
            );
            parser.set_grammar_profile(Some("sv_2017"));
            assert!(
                parser.parse_full_systemverilog_file().is_ok(),
                "SV parser rejected grammar-valid sequence-repetition sample: {sample}",
            );
        }
    }

    // GRAMMAR-WELLFORMED.H.11.3 regression lock: the generated layout skipper
    // hard-codes `#`-to-end-of-line comment skipping (an EBNF meta-grammar
    // convention), which used to swallow the VHDL based-literal `#` delimiter
    // before the `hash := trivia /#/` regex could match it — so every based
    // literal was rejected. The guard added to `consume_layout_for_regex`
    // lets a token whose own pattern matches at a comment introducer win over
    // the comment convention (mirroring the introducer guard
    // `consume_layout_for_terminal` already applies to string terminals).
    // H.11.5 then made the suppression STATIC for claimed introducers — the
    // vhdl parser no longer emits a `#` arm at all; these samples still lock
    // the user-visible property (based literals parse).
    #[cfg(all(feature = "generated_parsers", has_generated_vhdl_parser))]
    #[test]
    fn vhdl_based_literal_hash_token_is_not_swallowed_as_comment() {
        use crate::ast_pipeline::runtime_logger_box;
        use crate::generated_parsers::vhdl::VhdlParser;

        let samples = [
            "architecture a of e is signal i:t:=2#1010#;begin end;",
            "architecture a of e is signal i:t:=16#F_f#;begin end;",
            // The cert-coverage reach-probe shell that exposed the defect.
            "ARChitECtuRe BDfsk oF ZJ iS SignAL I:y:=2#1010#;BeGIn eNd;",
            // Control: a `#`-free declaration must keep parsing.
            "architecture a of e is signal i:t:=3374;begin end;",
        ];
        for sample in samples {
            let mut parser = VhdlParser::new(
                sample,
                runtime_logger_box("ast_shape_contract.vhdl_based_literal"),
            );
            assert!(
                parser.parse_full_vhdl_file().is_ok(),
                "vhdl parser rejected valid based-literal sample: {sample}"
            );
        }
    }
}
