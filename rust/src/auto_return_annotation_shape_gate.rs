//! Auto-generated return-annotation shape gate.
//!
//! For each grammar `foolang.ebnf`, the AST pipeline emits two artifacts:
//!   1. `generated/foolang_parser.rs` — the parser source.
//!   2. `generated/foolang_return_annotations.json` — every declared return
//!      annotation in raw text form.
//!
//! This module provides the third leg: a runtime gate that takes the
//! inventory + the running generated parser + a small per-grammar samples
//! list and verifies the parser's typed AST output matches the declared
//! shape of each annotation. No per-grammar manifest is needed — the
//! shape descriptors are derived directly from the inventory's `raw_text`
//! field by reusing `UnifiedReturnAST::parse_bootstrap` to parse the
//! annotation expression.
//!
//! Today the gate verifies the **entry rule**'s annotation shape against
//! each provided sample. Per-rule sample mapping (so every annotation in
//! the inventory gets exercised, not just the entry rule's) is a future
//! extension that needs either an auto-generated samples corpus or a
//! curated `<grammar>_auto_gate_samples.json` file.

use crate::ast_pipeline::{
    EmittedReturnAnnotationEntry, EmittedReturnAnnotationInventory, UnifiedReturnAST,
    runtime_logger,
};
use std::collections::BTreeSet;

/// Structural shape derived from a return annotation's raw text.
/// Drives `verify_typed_value` to confirm the parser's runtime output
/// matches the grammar-author-declared shape.
#[derive(Debug, Clone)]
pub struct AnnotationShapeDescriptor {
    pub rule: String,
    pub branch_index: usize,
    pub raw_text: String,
    pub kind: ShapeKind,
}

#[derive(Debug, Clone)]
pub enum ShapeKind {
    /// `{type: "X", k: $...}` — typed object literal. Verifier checks the
    /// runtime value is a JSON object with the declared keys present, and
    /// for any string-valued literal in the annotation (e.g. `type: "X"`),
    /// asserts the runtime key carries that exact string.
    Object {
        required_keys: BTreeSet<String>,
        required_string_values: Vec<(String, String)>,
    },
    /// `[...]` — array literal. Verifier checks the runtime value is a JSON
    /// array. Per-element shape inference is out of scope today.
    Array,
    /// `"literal"` — string literal at the top level of the annotation.
    /// Verifier asserts the runtime value is a JSON string equal to the
    /// declared literal.
    StringLiteral(String),
    /// `$N`, `$N::target*`, `$1.field`, etc. — passthrough or extraction
    /// shapes that depend on the captured rule's own structure. Verifier
    /// can't enforce shape without resolving the captured rule's typed
    /// output, so it skips these without failing.
    Passthrough,
    /// Unrecognized annotation form (parse failed or unsupported variant).
    /// Verifier skips with a structured note.
    Unknown,
}

impl AnnotationShapeDescriptor {
    /// Derive the shape descriptor from one inventory entry by parsing its
    /// `raw_text` via the bootstrap return-annotation parser. Errors during
    /// bootstrap parse are folded into `ShapeKind::Unknown` so a malformed
    /// or new-syntax annotation doesn't break the gate — it just skips.
    pub fn from_inventory_entry(entry: &EmittedReturnAnnotationEntry) -> Self {
        let logger = runtime_logger("auto_return_annotation_gate");
        let kind = match UnifiedReturnAST::parse_bootstrap(&entry.raw_text, &logger) {
            Ok(ast) => derive_shape_kind_from_ast(&ast),
            Err(_) => ShapeKind::Unknown,
        };
        AnnotationShapeDescriptor {
            rule: entry.rule.clone(),
            branch_index: entry.branch_index,
            raw_text: entry.raw_text.clone(),
            kind,
        }
    }
}

fn derive_shape_kind_from_ast(ast: &UnifiedReturnAST) -> ShapeKind {
    match ast {
        UnifiedReturnAST::Object { properties } => {
            let required_keys: BTreeSet<String> = properties.keys().cloned().collect();
            let mut required_string_values = Vec::new();
            for (key, value) in properties {
                if let UnifiedReturnAST::StringLiteral { value: s } = value.as_ref() {
                    required_string_values.push((key.clone(), s.clone()));
                }
            }
            ShapeKind::Object {
                required_keys,
                required_string_values,
            }
        }
        UnifiedReturnAST::Array { .. } => ShapeKind::Array,
        UnifiedReturnAST::StringLiteral { value } => ShapeKind::StringLiteral(value.clone()),
        UnifiedReturnAST::PositionalRef { .. }
        | UnifiedReturnAST::QuantifiedExtraction { .. }
        | UnifiedReturnAST::PropertyAccess { .. }
        | UnifiedReturnAST::ArrayAccess { .. }
        | UnifiedReturnAST::Spread { .. }
        | UnifiedReturnAST::FlattenSpread { .. }
        | UnifiedReturnAST::Passthrough => ShapeKind::Passthrough,
        UnifiedReturnAST::NumberLiteral { .. }
        | UnifiedReturnAST::BooleanLiteral { .. }
        | UnifiedReturnAST::NullLiteral
        | UnifiedReturnAST::Identifier { .. } => ShapeKind::Unknown,
    }
}

/// Verify a runtime JSON value matches a shape descriptor. Returns
/// `Ok(())` on match, `Err(reason)` on mismatch with a precise diagnostic.
pub fn verify_typed_value(
    descriptor: &AnnotationShapeDescriptor,
    value: &serde_json::Value,
) -> Result<(), String> {
    match &descriptor.kind {
        ShapeKind::Object {
            required_keys,
            required_string_values,
        } => {
            let obj = value.as_object().ok_or_else(|| {
                format!(
                    "rule {}/{}: declared shape is Object {{...}} but parsed value is not a JSON object: {}",
                    descriptor.rule, descriptor.branch_index, value
                )
            })?;
            for key in required_keys {
                if !obj.contains_key(key) {
                    return Err(format!(
                        "rule {}/{}: declared key '{}' missing from parsed value: {}",
                        descriptor.rule, descriptor.branch_index, key, value
                    ));
                }
            }
            for (key, expected_str) in required_string_values {
                let actual = obj.get(key).and_then(|v| v.as_str()).ok_or_else(|| {
                    format!(
                        "rule {}/{}: declared string-valued key '{}' is not a JSON string in parsed value: {}",
                        descriptor.rule, descriptor.branch_index, key, value
                    )
                })?;
                if actual != expected_str {
                    return Err(format!(
                        "rule {}/{}: declared key '{}' = \"{}\", parsed value carries \"{}\"",
                        descriptor.rule, descriptor.branch_index, key, expected_str, actual
                    ));
                }
            }
            Ok(())
        }
        ShapeKind::Array => {
            if !value.is_array() {
                return Err(format!(
                    "rule {}/{}: declared shape is Array but parsed value is not an array: {}",
                    descriptor.rule, descriptor.branch_index, value
                ));
            }
            Ok(())
        }
        ShapeKind::StringLiteral(expected) => {
            let actual = value.as_str().ok_or_else(|| {
                format!(
                    "rule {}/{}: declared shape is string \"{}\" but parsed value is not a JSON string: {}",
                    descriptor.rule, descriptor.branch_index, expected, value
                )
            })?;
            if actual != expected {
                return Err(format!(
                    "rule {}/{}: declared string \"{}\" but parsed value \"{}\"",
                    descriptor.rule, descriptor.branch_index, expected, actual
                ));
            }
            Ok(())
        }
        ShapeKind::Passthrough | ShapeKind::Unknown => {
            // No structural constraint to enforce. Passthrough annotations
            // (`$N`, `$N::target*`, etc.) depend on the captured rule's
            // own shape; verifying that here would require recursing into
            // the captured rule's annotation, which the gate doesn't do
            // today. Unknown shapes (parse failed) are also skipped — the
            // gate's job is to catch shape regressions, not flag novel
            // annotation syntax.
            Ok(())
        }
    }
}

/// Per-run report from the auto-gate.
#[derive(Debug, Clone, Default)]
pub struct AutoGateReport {
    pub grammar: String,
    pub entry_rule: String,
    pub samples_checked: usize,
    pub failures: Vec<String>,
    /// True when the entry rule's annotation is `Passthrough`/`Unknown`
    /// and the gate had nothing to enforce. Not a failure — informational.
    pub entry_skipped_no_enforceable_shape: bool,
    /// True when the entry rule has no declared annotation in the
    /// inventory at all. Not a failure — informational.
    pub entry_has_no_annotation: bool,
}

/// Run the auto-gate against a grammar's entry rule. For each sample
/// input, parses it via `parser_invoke`, then verifies the resulting
/// typed JSON matches the entry rule's declared annotation shape.
///
/// `parser_invoke` closure shape: `(input) -> Result<typed_json_value, parse_error_message>`.
/// The closure is responsible for invoking the right `parse_full_<entry>`
/// method on the right parser type and returning the runtime
/// `ParseContent::to_json_value()` of the result.
pub fn run_entry_rule_auto_gate(
    inventory: &EmittedReturnAnnotationInventory,
    entry_rule: &str,
    samples: &[String],
    mut parser_invoke: impl FnMut(&str) -> Result<serde_json::Value, String>,
) -> AutoGateReport {
    let mut report = AutoGateReport {
        grammar: inventory.grammar.clone(),
        entry_rule: entry_rule.to_string(),
        ..Default::default()
    };

    let entry_entry = inventory
        .annotations
        .iter()
        .find(|e| e.rule == entry_rule);
    let Some(entry_entry) = entry_entry else {
        report.entry_has_no_annotation = true;
        return report;
    };

    let descriptor = AnnotationShapeDescriptor::from_inventory_entry(entry_entry);
    if matches!(
        descriptor.kind,
        ShapeKind::Passthrough | ShapeKind::Unknown
    ) {
        report.entry_skipped_no_enforceable_shape = true;
        return report;
    }

    for sample in samples {
        report.samples_checked += 1;
        match parser_invoke(sample) {
            Ok(value) => {
                if let Err(reason) = verify_typed_value(&descriptor, &value) {
                    report
                        .failures
                        .push(format!("input {:?}: {}", sample, reason));
                }
            }
            Err(parse_error) => {
                report
                    .failures
                    .push(format!("input {:?}: parse failed: {}", sample, parse_error));
            }
        }
    }

    report
}

/// Convenience helper: read an inventory artifact from disk.
pub fn read_inventory_artifact(
    path: &std::path::Path,
) -> Result<EmittedReturnAnnotationInventory, String> {
    let raw = std::fs::read_to_string(path)
        .map_err(|err| format!("read inventory {}: {}", path.display(), err))?;
    serde_json::from_str(&raw)
        .map_err(|err| format!("parse inventory {}: {}", path.display(), err))
}

/// Per-run report from the inventory-wide auto-gate.
///
/// The inventory-wide gate walks the parse tree and verifies every typed
/// object whose `type` discriminator matches a declared annotation in the
/// inventory. This catches inner-rule annotations whose typed AST lives
/// nested inside the entry rule's output (e.g. `regex`'s `piece` rule
/// produces `{type: "piece", atom: ..., quantifier: ...}` deep inside
/// the entry rule's `pattern` field).
#[derive(Debug, Clone, Default)]
pub struct InventoryWideAutoGateReport {
    pub grammar: String,
    pub samples_checked: usize,
    pub failures: Vec<String>,
    /// Discriminators that have at least one declared `Object` annotation
    /// in the inventory. These are the only annotations the inventory-wide
    /// gate can locate post-parse (it relies on the `type: "X"` literal
    /// to pin a runtime object back to its declaring rule).
    pub discriminators_declared: BTreeSet<String>,
    /// Discriminators encountered (and verified) during sample runs.
    pub discriminators_seen: BTreeSet<String>,
    /// Inventory entries whose annotation is `Passthrough`/`Array`/
    /// `StringLiteral`/`Unknown`. These can't be tied to a discriminator,
    /// so the gate does not enforce them today; they're tracked here as
    /// `<rule>/<branch_index>` so the test surface stays honest about
    /// what's covered vs. skipped.
    pub annotations_skipped_no_discriminator: BTreeSet<String>,
}

impl InventoryWideAutoGateReport {
    /// Discriminators that are declared in the inventory but never appeared
    /// in any sample's parse output. These rules are under-exercised by
    /// the test's samples — add a sample that triggers them to close the
    /// coverage gap.
    pub fn discriminators_not_covered(&self) -> BTreeSet<String> {
        self.discriminators_declared
            .difference(&self.discriminators_seen)
            .cloned()
            .collect()
    }
}

/// Inventory-wide gate. For each declared annotation in the inventory
/// whose shape is an `Object` with a string-literal `type:` key, build a
/// shape descriptor. For each sample input, parse it, then walk the
/// resulting JSON tree and:
///   - For every JSON object encountered with a `type: "X"` field where
///     `X` is a declared discriminator: verify the object matches the
///     descriptor. Failures get recorded with the sample input + reason.
///   - Track which discriminators were seen across all samples; the
///     report exposes uncovered ones so the test surface stays honest.
///
/// `parser_invoke` returns the runtime typed-JSON value
/// (`ParseContent::to_json_value()` of the parse result).
pub fn run_inventory_wide_auto_gate(
    inventory: &EmittedReturnAnnotationInventory,
    samples: &[String],
    mut parser_invoke: impl FnMut(&str) -> Result<serde_json::Value, String>,
) -> InventoryWideAutoGateReport {
    let mut report = InventoryWideAutoGateReport {
        grammar: inventory.grammar.clone(),
        ..Default::default()
    };

    // Build the discriminator → descriptor map. Only Object annotations
    // with a string-literal `type:` key are addressable post-parse —
    // they're how the gate ties a runtime JSON object back to the rule
    // that emitted it.
    let mut discriminator_to_descriptor: std::collections::HashMap<
        String,
        AnnotationShapeDescriptor,
    > = std::collections::HashMap::new();
    for entry in &inventory.annotations {
        let descriptor = AnnotationShapeDescriptor::from_inventory_entry(entry);
        match &descriptor.kind {
            ShapeKind::Object {
                required_string_values,
                ..
            } => {
                let type_literal = required_string_values
                    .iter()
                    .find(|(k, _)| k == "type")
                    .map(|(_, v)| v.clone());
                match type_literal {
                    Some(t) => {
                        report.discriminators_declared.insert(t.clone());
                        discriminator_to_descriptor.insert(t, descriptor);
                    }
                    None => {
                        // Object literal without a `type:` key — can't be
                        // located in the runtime JSON without a
                        // discriminator. Skip but log.
                        report.annotations_skipped_no_discriminator.insert(format!(
                            "{}/{}",
                            descriptor.rule, descriptor.branch_index
                        ));
                    }
                }
            }
            _ => {
                report.annotations_skipped_no_discriminator.insert(format!(
                    "{}/{}",
                    descriptor.rule, descriptor.branch_index
                ));
            }
        }
    }

    for sample in samples {
        report.samples_checked += 1;
        match parser_invoke(sample) {
            Ok(value) => {
                walk_and_verify_against_discriminator_map(
                    &value,
                    &discriminator_to_descriptor,
                    sample,
                    &mut report.failures,
                    &mut report.discriminators_seen,
                );
            }
            Err(parse_error) => {
                report
                    .failures
                    .push(format!("input {:?}: parse failed: {}", sample, parse_error));
            }
        }
    }

    report
}

fn walk_and_verify_against_discriminator_map(
    value: &serde_json::Value,
    discriminator_to_descriptor: &std::collections::HashMap<String, AnnotationShapeDescriptor>,
    sample_input: &str,
    failures: &mut Vec<String>,
    seen: &mut BTreeSet<String>,
) {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(t) = map.get("type").and_then(|v| v.as_str()) {
                if let Some(descriptor) = discriminator_to_descriptor.get(t) {
                    seen.insert(t.to_string());
                    if let Err(reason) = verify_typed_value(descriptor, value) {
                        failures.push(format!("input {:?}: {}", sample_input, reason));
                    }
                }
            }
            for nested in map.values() {
                walk_and_verify_against_discriminator_map(
                    nested,
                    discriminator_to_descriptor,
                    sample_input,
                    failures,
                    seen,
                );
            }
        }
        serde_json::Value::Array(items) => {
            for nested in items {
                walk_and_verify_against_discriminator_map(
                    nested,
                    discriminator_to_descriptor,
                    sample_input,
                    failures,
                    seen,
                );
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_entry(raw_text: &str) -> EmittedReturnAnnotationEntry {
        EmittedReturnAnnotationEntry {
            rule: "test_rule".to_string(),
            branch_index: 0,
            annotation_type: "return_object".to_string(),
            raw_text: raw_text.to_string(),
            normalized_text: raw_text.to_string(),
        }
    }

    #[test]
    fn descriptor_object_with_typed_discriminator_extracts_required_keys_and_string_values() {
        let entry = make_entry(r#"{type: "regex", pattern: $1}"#);
        let desc = AnnotationShapeDescriptor::from_inventory_entry(&entry);
        match desc.kind {
            ShapeKind::Object {
                required_keys,
                required_string_values,
            } => {
                assert!(required_keys.contains("type"));
                assert!(required_keys.contains("pattern"));
                assert_eq!(
                    required_string_values,
                    vec![("type".to_string(), "regex".to_string())]
                );
            }
            other => panic!("expected Object kind, got {:?}", other),
        }
    }

    #[test]
    fn verify_typed_object_passes_when_required_keys_and_type_match() {
        let entry = make_entry(r#"{type: "regex", pattern: $1}"#);
        let desc = AnnotationShapeDescriptor::from_inventory_entry(&entry);
        let value = json!({
            "type": "regex",
            "pattern": [{"x": 1}],
        });
        assert!(verify_typed_value(&desc, &value).is_ok());
    }

    #[test]
    fn verify_typed_object_fails_when_type_discriminator_disagrees() {
        let entry = make_entry(r#"{type: "regex", pattern: $1}"#);
        let desc = AnnotationShapeDescriptor::from_inventory_entry(&entry);
        let value = json!({"type": "wrong", "pattern": []});
        let err = verify_typed_value(&desc, &value).unwrap_err();
        assert!(
            err.contains("declared key 'type' = \"regex\""),
            "expected discriminator-mismatch diagnostic, got: {}",
            err
        );
    }

    #[test]
    fn verify_typed_object_fails_when_required_key_missing() {
        let entry = make_entry(r#"{type: "regex", pattern: $1}"#);
        let desc = AnnotationShapeDescriptor::from_inventory_entry(&entry);
        let value = json!({"type": "regex"});
        let err = verify_typed_value(&desc, &value).unwrap_err();
        assert!(
            err.contains("declared key 'pattern' missing"),
            "expected missing-key diagnostic, got: {}",
            err
        );
    }

    #[test]
    fn verify_typed_object_fails_when_value_is_not_an_object() {
        let entry = make_entry(r#"{type: "regex", pattern: $1}"#);
        let desc = AnnotationShapeDescriptor::from_inventory_entry(&entry);
        let value = json!([1, 2, 3]);
        let err = verify_typed_value(&desc, &value).unwrap_err();
        assert!(
            err.contains("not a JSON object"),
            "expected wrong-kind diagnostic, got: {}",
            err
        );
    }

    #[test]
    fn descriptor_array_kind() {
        let entry = make_entry(r#"[$1, $2::2*]"#);
        let desc = AnnotationShapeDescriptor::from_inventory_entry(&entry);
        assert!(matches!(desc.kind, ShapeKind::Array));
    }

    #[test]
    fn verify_array_passes_for_array_value_and_fails_otherwise() {
        let entry = make_entry(r#"[$1]"#);
        let desc = AnnotationShapeDescriptor::from_inventory_entry(&entry);
        assert!(verify_typed_value(&desc, &json!(["x"])).is_ok());
        let err = verify_typed_value(&desc, &json!({"k": "v"})).unwrap_err();
        assert!(err.contains("not an array"));
    }

    #[test]
    fn descriptor_passthrough_skips_verification() {
        let entry = make_entry(r#"$1"#);
        let desc = AnnotationShapeDescriptor::from_inventory_entry(&entry);
        assert!(matches!(desc.kind, ShapeKind::Passthrough));
        // Anything matches a passthrough — gate skips, no failure.
        assert!(verify_typed_value(&desc, &json!(null)).is_ok());
        assert!(verify_typed_value(&desc, &json!({"x": 1})).is_ok());
    }

    #[test]
    fn run_entry_rule_auto_gate_reports_no_failures_when_shape_matches() {
        let inv = EmittedReturnAnnotationInventory {
            version: 1,
            grammar: "demo".to_string(),
            annotation_count: 1,
            annotations: vec![EmittedReturnAnnotationEntry {
                rule: "root".to_string(),
                branch_index: 0,
                annotation_type: "return_object".to_string(),
                raw_text: r#"{type: "demo", payload: $1}"#.to_string(),
                normalized_text: r#"{type: "demo", payload: $1}"#.to_string(),
            }],
        };
        let report = run_entry_rule_auto_gate(&inv, "root", &["sample".to_string()], |_| {
            Ok(json!({"type": "demo", "payload": "hi"}))
        });
        assert_eq!(report.samples_checked, 1);
        assert!(report.failures.is_empty());
    }

    #[test]
    fn run_entry_rule_auto_gate_reports_failure_when_type_disagrees() {
        let inv = EmittedReturnAnnotationInventory {
            version: 1,
            grammar: "demo".to_string(),
            annotation_count: 1,
            annotations: vec![EmittedReturnAnnotationEntry {
                rule: "root".to_string(),
                branch_index: 0,
                annotation_type: "return_object".to_string(),
                raw_text: r#"{type: "demo", payload: $1}"#.to_string(),
                normalized_text: r#"{type: "demo", payload: $1}"#.to_string(),
            }],
        };
        let report = run_entry_rule_auto_gate(&inv, "root", &["sample".to_string()], |_| {
            Ok(json!({"type": "wrong", "payload": "hi"}))
        });
        assert_eq!(report.failures.len(), 1);
    }

    #[test]
    fn run_entry_rule_auto_gate_skips_passthrough_annotation() {
        let inv = EmittedReturnAnnotationInventory {
            version: 1,
            grammar: "demo".to_string(),
            annotation_count: 1,
            annotations: vec![EmittedReturnAnnotationEntry {
                rule: "root".to_string(),
                branch_index: 0,
                annotation_type: "return_scalar".to_string(),
                raw_text: "$1".to_string(),
                normalized_text: "$1".to_string(),
            }],
        };
        let report = run_entry_rule_auto_gate(&inv, "root", &["sample".to_string()], |_| {
            Ok(json!({"x": 1}))
        });
        assert!(report.entry_skipped_no_enforceable_shape);
        assert_eq!(report.samples_checked, 0);
    }

    #[test]
    fn inventory_wide_gate_verifies_nested_inner_rule_discriminator() {
        // Two declared annotations: `outer` (entry) and `inner` (nested).
        // The simulated parser output puts the inner rule's typed object
        // deep inside the outer's `payload` array. The gate must walk
        // there to verify the inner shape.
        let inv = EmittedReturnAnnotationInventory {
            version: 1,
            grammar: "demo".to_string(),
            annotation_count: 2,
            annotations: vec![
                EmittedReturnAnnotationEntry {
                    rule: "outer".to_string(),
                    branch_index: 0,
                    annotation_type: "return_object".to_string(),
                    raw_text: r#"{type: "outer", payload: $1}"#.to_string(),
                    normalized_text: r#"{type: "outer", payload: $1}"#.to_string(),
                },
                EmittedReturnAnnotationEntry {
                    rule: "inner".to_string(),
                    branch_index: 0,
                    annotation_type: "return_object".to_string(),
                    raw_text: r#"{type: "inner", value: $2}"#.to_string(),
                    normalized_text: r#"{type: "inner", value: $2}"#.to_string(),
                },
            ],
        };
        let report = run_inventory_wide_auto_gate(&inv, &["sample".to_string()], |_| {
            Ok(json!({
                "type": "outer",
                "payload": [
                    {"type": "inner", "value": 42},
                    {"type": "inner", "value": 7},
                ],
            }))
        });
        assert!(report.failures.is_empty(), "expected clean run; got {:?}", report.failures);
        assert!(report.discriminators_seen.contains("outer"));
        assert!(report.discriminators_seen.contains("inner"));
        assert!(report.discriminators_not_covered().is_empty());
    }

    #[test]
    fn inventory_wide_gate_fails_when_nested_inner_rule_shape_disagrees() {
        let inv = EmittedReturnAnnotationInventory {
            version: 1,
            grammar: "demo".to_string(),
            annotation_count: 2,
            annotations: vec![
                EmittedReturnAnnotationEntry {
                    rule: "outer".to_string(),
                    branch_index: 0,
                    annotation_type: "return_object".to_string(),
                    raw_text: r#"{type: "outer", payload: $1}"#.to_string(),
                    normalized_text: r#"{type: "outer", payload: $1}"#.to_string(),
                },
                EmittedReturnAnnotationEntry {
                    rule: "inner".to_string(),
                    branch_index: 0,
                    annotation_type: "return_object".to_string(),
                    raw_text: r#"{type: "inner", value: $2}"#.to_string(),
                    normalized_text: r#"{type: "inner", value: $2}"#.to_string(),
                },
            ],
        };
        // Simulated parser produces an inner object missing the `value` key.
        let report = run_inventory_wide_auto_gate(&inv, &["sample".to_string()], |_| {
            Ok(json!({
                "type": "outer",
                "payload": [{"type": "inner"}],
            }))
        });
        assert_eq!(
            report.failures.len(),
            1,
            "expected one nested-shape failure; got {:?}",
            report.failures
        );
        assert!(report.failures[0].contains("declared key 'value' missing"));
    }

    #[test]
    fn inventory_wide_gate_reports_uncovered_discriminators() {
        let inv = EmittedReturnAnnotationInventory {
            version: 1,
            grammar: "demo".to_string(),
            annotation_count: 2,
            annotations: vec![
                EmittedReturnAnnotationEntry {
                    rule: "outer".to_string(),
                    branch_index: 0,
                    annotation_type: "return_object".to_string(),
                    raw_text: r#"{type: "outer", payload: $1}"#.to_string(),
                    normalized_text: r#"{type: "outer", payload: $1}"#.to_string(),
                },
                EmittedReturnAnnotationEntry {
                    rule: "rare_branch".to_string(),
                    branch_index: 0,
                    annotation_type: "return_object".to_string(),
                    raw_text: r#"{type: "rare", value: $1}"#.to_string(),
                    normalized_text: r#"{type: "rare", value: $1}"#.to_string(),
                },
            ],
        };
        // No `rare` discriminator in the parsed output — coverage gap.
        let report = run_inventory_wide_auto_gate(&inv, &["sample".to_string()], |_| {
            Ok(json!({"type": "outer", "payload": []}))
        });
        assert!(report.failures.is_empty());
        assert!(report.discriminators_seen.contains("outer"));
        assert!(!report.discriminators_seen.contains("rare"));
        assert_eq!(
            report.discriminators_not_covered(),
            ["rare"].iter().map(|s| s.to_string()).collect()
        );
    }

    #[test]
    fn inventory_wide_gate_records_passthrough_and_array_skips() {
        let inv = EmittedReturnAnnotationInventory {
            version: 1,
            grammar: "demo".to_string(),
            annotation_count: 3,
            annotations: vec![
                EmittedReturnAnnotationEntry {
                    rule: "outer".to_string(),
                    branch_index: 0,
                    annotation_type: "return_object".to_string(),
                    raw_text: r#"{type: "outer", payload: $1}"#.to_string(),
                    normalized_text: r#"{type: "outer", payload: $1}"#.to_string(),
                },
                EmittedReturnAnnotationEntry {
                    rule: "passthru".to_string(),
                    branch_index: 0,
                    annotation_type: "return_scalar".to_string(),
                    raw_text: "$1".to_string(),
                    normalized_text: "$1".to_string(),
                },
                EmittedReturnAnnotationEntry {
                    rule: "list_rule".to_string(),
                    branch_index: 0,
                    annotation_type: "return_array".to_string(),
                    raw_text: "[$1, $2]".to_string(),
                    normalized_text: "[$1, $2]".to_string(),
                },
            ],
        };
        let report = run_inventory_wide_auto_gate(&inv, &["sample".to_string()], |_| {
            Ok(json!({"type": "outer", "payload": []}))
        });
        assert!(report.failures.is_empty());
        assert!(report.annotations_skipped_no_discriminator.contains("passthru/0"));
        assert!(report.annotations_skipped_no_discriminator.contains("list_rule/0"));
    }
}
