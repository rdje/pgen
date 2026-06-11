use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use pgen::parser_registry::{parse_sample, parse_sample_ast_json};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct RtlFrontendGeneratedContract {
    contract_version: String,
    grammar_name: String,
    purpose: String,
    provenance: String,
    samples: Vec<RtlFrontendGeneratedSample>,
}

#[derive(Debug, Deserialize)]
struct RtlFrontendGeneratedSample {
    label: String,
    expected_parse_ok: bool,
    require_ast_json: bool,
    #[serde(default)]
    required_rule_names: Vec<String>,
    #[serde(default)]
    forbidden_rule_names: Vec<String>,
    #[serde(default)]
    required_typed_string_values: Vec<String>,
    sample: String,
}

/// Rule-participation testimony from the REAL parser's transactional coverage record
/// (`enable_coverage` + `exercised_rule_names`). This is the typed-era replacement for
/// walking the dumped AST for `rule_name` keys: return annotations fold annotated rules
/// into typed `ParseContent::Json` (no `rule_name` children survive), while the coverage
/// record keeps exactly the rules of the ACCEPTED parse — sound (backtracked attempts
/// are truncated) and complete (annotation folding cannot hide a rule entry).
#[cfg(has_generated_rtl_frontend_parser)]
fn cover_sample(sample: &str) -> Option<(bool, HashSet<String>)> {
    Some(pgen::parser_registry::parse_and_cover_rtl_frontend(
        sample, None,
    ))
}

#[cfg(not(has_generated_rtl_frontend_parser))]
fn cover_sample(_sample: &str) -> Option<(bool, HashSet<String>)> {
    None
}

/// Collect every string scalar in the typed AST JSON (the schema-3 carrier), with
/// multiplicity. The curated `required_typed_string_values` locks assert against this
/// multiset: identifier-level evidence (signal names, kind discriminators, operator
/// kinds) survives in the typed carrier as exact string values.
fn collect_string_values(node: &Value, values: &mut Vec<String>) {
    match node {
        Value::Array(items) => {
            for item in items {
                collect_string_values(item, values);
            }
        }
        Value::Object(map) => {
            for value in map.values() {
                collect_string_values(value, values);
            }
        }
        Value::String(text) => values.push(text.clone()),
        _ => {}
    }
}

fn missing_required_texts(actual_texts: &[String], required_texts: &[String]) -> Vec<String> {
    let mut remaining_actual = actual_texts.to_vec();
    let mut missing = Vec::new();
    for required in required_texts {
        if let Some(index) = remaining_actual
            .iter()
            .position(|actual| actual == required)
        {
            remaining_actual.remove(index);
        } else {
            missing.push(required.clone());
        }
    }
    missing
}

fn load_contract() -> Result<RtlFrontendGeneratedContract> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test_data/grammar_quality/rtl_frontend_generated_parity_contract_v0.json");
    let raw = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parsing {}", path.display()))
}

fn run() -> Result<()> {
    let contract = load_contract()?;
    if contract.contract_version != "0.2.0" {
        bail!("unexpected contract version: {}", contract.contract_version);
    }
    if contract.grammar_name != "rtl_frontend" {
        bail!("unexpected grammar name: {}", contract.grammar_name);
    }
    if !contract
        .purpose
        .contains("Curated generated rtl_frontend syntax contract")
    {
        bail!("unexpected contract purpose: {}", contract.purpose);
    }
    if !contract
        .provenance
        .contains("local handwritten rtl_frontend::parse_design replay")
    {
        bail!("unexpected contract provenance: {}", contract.provenance);
    }
    if contract.samples.is_empty() {
        bail!("rtl_frontend generated contract must contain at least one sample");
    }

    for sample in contract.samples {
        let generated_ok = parse_sample("rtl_frontend", &sample.sample).with_context(|| {
            format!(
                "generated rtl_frontend adapter missing for '{}'",
                sample.label
            )
        })?;
        if generated_ok != sample.expected_parse_ok {
            bail!(
                "generated rtl_frontend parseability drifted for sample '{}': expected {}, got {}",
                sample.label,
                sample.expected_parse_ok,
                generated_ok
            );
        }

        if !sample.required_rule_names.is_empty() || !sample.forbidden_rule_names.is_empty() {
            let (covered_ok, covered) = cover_sample(&sample.sample).with_context(|| {
                format!(
                    "generated rtl_frontend coverage adapter missing for '{}'",
                    sample.label
                )
            })?;
            if !covered_ok {
                bail!(
                    "generated rtl_frontend coverage parse rejected curated sample '{}'",
                    sample.label
                );
            }
            for rule_name in &sample.required_rule_names {
                if !covered.contains(rule_name) {
                    bail!(
                        "generated rtl_frontend accepted parse for sample '{}' did not exercise required rule '{}'",
                        sample.label,
                        rule_name
                    );
                }
            }
            for rule_name in &sample.forbidden_rule_names {
                if covered.contains(rule_name) {
                    bail!(
                        "generated rtl_frontend accepted parse for sample '{}' unexpectedly exercised forbidden rule '{}'",
                        sample.label,
                        rule_name
                    );
                }
            }
        }

        if sample.require_ast_json {
            let ast_json =
                parse_sample_ast_json("rtl_frontend", &sample.sample).with_context(|| {
                    format!(
                        "generated rtl_frontend AST adapter missing for '{}'",
                        sample.label
                    )
                })?;
            let ast_json = ast_json.map_err(|_| {
                anyhow::anyhow!(
                    "generated rtl_frontend AST JSON adapter rejected curated sample '{}'",
                    sample.label
                )
            })?;
            if !sample.required_typed_string_values.is_empty() {
                let mut actual_values = Vec::new();
                collect_string_values(&ast_json, &mut actual_values);
                let missing =
                    missing_required_texts(&actual_values, &sample.required_typed_string_values);
                if !missing.is_empty() {
                    bail!(
                        "generated rtl_frontend typed AST JSON for sample '{}' is missing required typed string values: missing {:?}",
                        sample.label,
                        missing
                    );
                }
            }
        }

        println!(
            "sample '{}' passed: generated={}",
            sample.label, generated_ok
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{collect_string_values, missing_required_texts};
    use serde_json::json;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn required_texts_accept_subset_matches() {
        let actual = strings(&["a", "b + c", "d << 1"]);
        let required = strings(&["b + c", "d << 1"]);

        assert!(missing_required_texts(&actual, &required).is_empty());
    }

    #[test]
    fn required_texts_report_missing_values() {
        let actual = strings(&["a", "b + c"]);
        let required = strings(&["b + c", "d << 1"]);

        assert_eq!(
            missing_required_texts(&actual, &required),
            strings(&["d << 1"])
        );
    }

    #[test]
    fn required_texts_preserve_multiplicity() {
        let actual = strings(&["a", "a"]);
        let required = strings(&["a", "a", "a"]);

        assert_eq!(missing_required_texts(&actual, &required), strings(&["a"]));
    }

    #[test]
    fn string_values_collect_with_multiplicity_across_nesting() {
        let ast = json!({
            "content": {
                "Json": {
                    "items": [
                        {"kind": "module", "body": {"name": "top", "ports": ["clk", "clk"]}},
                        {"kind": "semi"}
                    ]
                }
            },
            "rule_name": "rtl_frontend_file"
        });
        let mut values = Vec::new();
        collect_string_values(&ast, &mut values);
        assert_eq!(
            values.iter().filter(|value| value.as_str() == "clk").count(),
            2
        );
        assert!(values.iter().any(|value| value == "module"));
        assert!(values.iter().any(|value| value == "top"));
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err:#}");
            ExitCode::FAILURE
        }
    }
}
