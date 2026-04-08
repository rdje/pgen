use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use pgen::parser_registry::{parse_sample, parse_sample_ast_json};
use serde::Deserialize;

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
    sample: String,
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
        let generated_ok = parse_sample("rtl_frontend", &sample.sample)
            .with_context(|| format!("generated rtl_frontend adapter missing for '{}'", sample.label))?;
        if generated_ok != sample.expected_parse_ok {
            bail!(
                "generated rtl_frontend parseability drifted for sample '{}': expected {}, got {}",
                sample.label,
                sample.expected_parse_ok,
                generated_ok
            );
        }

        if sample.require_ast_json {
            let ast_json = parse_sample_ast_json("rtl_frontend", &sample.sample)
                .with_context(|| format!("generated rtl_frontend AST adapter missing for '{}'", sample.label))?;
            if ast_json.is_err() {
                bail!(
                    "generated rtl_frontend AST JSON adapter rejected curated sample '{}'",
                    sample.label
                );
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
