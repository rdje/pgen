use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use pgen::parser_registry::{parse_sample, parse_sample_ast_json};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

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
    expected_rule_texts: BTreeMap<String, Vec<String>>,
    sample: String,
}

fn collect_rule_names(node: &Value, names: &mut Vec<String>) {
    match node {
        Value::Array(values) => {
            for value in values {
                collect_rule_names(value, names);
            }
        }
        Value::Object(map) => {
            if let Some(Value::String(rule_name)) = map.get("rule_name") {
                names.push(rule_name.clone());
            }
            for value in map.values() {
                collect_rule_names(value, names);
            }
        }
        _ => {}
    }
}

fn ast_contains_rule(ast_json: &Value, rule_name: &str) -> bool {
    let mut names = Vec::new();
    collect_rule_names(ast_json, &mut names);
    names.iter().any(|candidate| candidate == rule_name)
}

fn collect_rule_spans(node: &Value, rule_name: &str, spans: &mut Vec<(usize, usize)>) {
    match node {
        Value::Array(values) => {
            for value in values {
                collect_rule_spans(value, rule_name, spans);
            }
        }
        Value::Object(map) => {
            if let Some(Value::String(candidate)) = map.get("rule_name") {
                if candidate == rule_name {
                    if let Some(Value::Object(span)) = map.get("span") {
                        if let (Some(Value::Number(start)), Some(Value::Number(end))) =
                            (span.get("start"), span.get("end"))
                        {
                            if let (Some(start), Some(end)) = (start.as_u64(), end.as_u64()) {
                                spans.push((start as usize, end as usize));
                            }
                        }
                    }
                }
            }
            for value in map.values() {
                collect_rule_spans(value, rule_name, spans);
            }
        }
        _ => {}
    }
}

fn ast_rule_texts(sample: &str, ast_json: &Value, rule_name: &str) -> Result<Vec<String>> {
    let mut spans = Vec::new();
    collect_rule_spans(ast_json, rule_name, &mut spans);
    spans
        .into_iter()
        .map(|(start, end)| {
            sample
                .get(start..end)
                .map(|text| text.trim().to_string())
                .ok_or_else(|| {
                    anyhow::anyhow!("invalid span {}..{} for rule '{}'", start, end, rule_name)
                })
        })
        .collect()
}

fn load_contract() -> Result<RtlFrontendGeneratedContract> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test_data/grammar_quality/rtl_frontend_generated_parity_contract_v0.json");
    let raw = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parsing {}", path.display()))
}

fn run() -> Result<()> {
    let contract = load_contract()?;
    if contract.contract_version != "0.1.0" {
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
            for rule_name in &sample.required_rule_names {
                if !ast_contains_rule(&ast_json, rule_name) {
                    bail!(
                        "generated rtl_frontend AST JSON for sample '{}' is missing required rule '{}'",
                        sample.label,
                        rule_name
                    );
                }
            }
            for rule_name in &sample.forbidden_rule_names {
                if ast_contains_rule(&ast_json, rule_name) {
                    bail!(
                        "generated rtl_frontend AST JSON for sample '{}' unexpectedly contains forbidden rule '{}'",
                        sample.label,
                        rule_name
                    );
                }
            }
            for (rule_name, expected_texts) in &sample.expected_rule_texts {
                let actual_texts = ast_rule_texts(&sample.sample, &ast_json, rule_name)?;
                if &actual_texts != expected_texts {
                    bail!(
                        "generated rtl_frontend AST JSON for sample '{}' preserved unexpected texts for rule '{}': expected {:?}, got {:?}",
                        sample.label,
                        rule_name,
                        expected_texts,
                        actual_texts
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

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err:#}");
            ExitCode::FAILURE
        }
    }
}
