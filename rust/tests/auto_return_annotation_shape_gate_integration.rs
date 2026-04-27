//! Per-grammar integration tests for the auto-generated return-annotation
//! shape gate.
//!
//! For each grammar whose generated parser is available in this build, the
//! test:
//!   1. Reads the pipeline-emitted `<grammar>_return_annotations.json`
//!      inventory.
//!   2. Runs the grammar's `parse_full_<entry>` method on a small sample
//!      list, converting each parse output to typed JSON via
//!      `ParseContent::to_json_value()`.
//!   3. Calls `run_inventory_wide_auto_gate`, which walks the entire
//!      typed-JSON tree of each parse output, finds every JSON object
//!      whose `type:` discriminator matches a declared annotation in the
//!      inventory (entry rule and inner rules alike, however deeply
//!      nested), and verifies each one against the shape descriptor
//!      derived from that annotation's raw text.
//!   4. Asserts the report has no failures. Coverage information
//!      (which declared discriminators were seen vs. uncovered) is
//!      surfaced via `eprintln!` for visibility.
//!
//! No per-grammar shape descriptors are hand-rolled — the gate derives
//! them from the inventory's `raw_text` field. Adding a new sample is a
//! one-line `samples.push(...)`; adding a new grammar is one new test
//! function (5 lines of boilerplate around 3 grammar-specific identifiers).

#![cfg(feature = "generated_parsers")]

use pgen::ast_pipeline::{ParseContent, runtime_logger_box};
use pgen::auto_return_annotation_shape_gate::{
    InventoryWideAutoGateReport, read_inventory_artifact, run_inventory_wide_auto_gate,
};
use std::path::PathBuf;

fn repo_generated_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("rust manifest parent")
        .join("generated")
}

fn assert_inventory_wide_clean(report: &InventoryWideAutoGateReport) {
    if !report.discriminators_not_covered().is_empty() {
        eprintln!(
            "[{}] inventory-wide auto-gate: {} declared discriminators not exercised by samples: {:?}. Add a sample that triggers them to close coverage.",
            report.grammar,
            report.discriminators_not_covered().len(),
            report.discriminators_not_covered(),
        );
    }
    if !report.annotations_skipped_no_discriminator.is_empty() {
        eprintln!(
            "[{}] inventory-wide auto-gate: {} inventory entries skipped (passthrough / array / no `type:` literal): {:?}",
            report.grammar,
            report.annotations_skipped_no_discriminator.len(),
            report.annotations_skipped_no_discriminator,
        );
    }
    assert_eq!(
        report.failures,
        Vec::<String>::new(),
        "[{}] inventory-wide auto-gate failures (samples_checked={}):\n  {}",
        report.grammar,
        report.samples_checked,
        report.failures.join("\n  "),
    );
}

#[test]
fn auto_gate_regex_inventory_wide_shape() {
    use pgen::generated_parsers::regex::RegexParser;

    let inv =
        read_inventory_artifact(&repo_generated_dir().join("regex_return_annotations.json"))
            .expect("regex inventory");
    // Curate samples that exercise as many of the inventory's
    // discriminators as possible. Today the regex inventory has 4
    // entries, of which 2 are typed-Object: `regex` (entry) and `piece`.
    let samples: Vec<String> = vec![
        "a".into(),       // exercises `regex` (entry) + `piece`
        "abc".into(),     // multiple `piece` instances
        "a*".into(),      // `piece` with quantifier
        "a|b".into(),     // alternation, `piece` siblings
        "(a|b)*c".into(), // nested groups + quantifier
    ];
    let report = run_inventory_wide_auto_gate(&inv, &samples, |input| {
        let mut parser = RegexParser::new(input, runtime_logger_box("auto_gate.regex"));
        let parsed = parser
            .parse_full_regex()
            .map_err(|err| err.to_string())?;
        Ok(parsed.content.to_json_value())
    });
    assert_inventory_wide_clean(&report);
}

#[test]
fn auto_gate_return_annotation_inventory_wide_shape() {
    use pgen::generated_parsers::return_annotation::Return_annotationParser;

    let inv = read_inventory_artifact(
        &repo_generated_dir().join("return_annotation_return_annotations.json"),
    )
    .expect("return_annotation inventory");
    // 16 inventory entries; many are Object-typed (extraction_expression,
    // array_access_expression, property_access_expression, etc.). Curate
    // samples that hit each.
    let samples: Vec<String> = vec![
        "$1".into(),
        "$2::first".into(),
        "$2::2*".into(),
        "[$1, $2*]".into(),
        "{key: $1, value: $3}".into(),
        "{type: \"object\", properties: $2}".into(),
        "$1.field".into(),
        "$1[0]".into(),
        "($1)".into(),
        "-> $1".into(),
        "true".into(),
        "\"hello\"".into(),
    ];
    let report = run_inventory_wide_auto_gate(&inv, &samples, |input| {
        let mut parser = Return_annotationParser::new(
            input,
            runtime_logger_box("auto_gate.return_annotation"),
        );
        let parsed = parser
            .parse_full_return_annotation()
            .map_err(|err| err.to_string())?;
        Ok(parsed.content.to_json_value())
    });
    assert_inventory_wide_clean(&report);
}

#[test]
fn auto_gate_semantic_annotation_inventory_wide_shape() {
    use pgen::generated_parsers::semantic_annotation::Semantic_annotationParser;

    let inv = read_inventory_artifact(
        &repo_generated_dir().join("semantic_annotation_return_annotations.json"),
    )
    .expect("semantic_annotation inventory");
    let samples: Vec<String> = vec![
        "@type: integer".into(),
        "@priority: [9, 1]".into(),
        "@category: constraint".into(),
        "@kind: \"Identifier\"".into(),
        "@throws: error_type".into(),
    ];
    let report = run_inventory_wide_auto_gate(&inv, &samples, |input| {
        let mut parser = Semantic_annotationParser::new(
            input,
            runtime_logger_box("auto_gate.semantic_annotation"),
        );
        let parsed = parser
            .parse_full_semantic_annotation()
            .map_err(|err| err.to_string())?;
        Ok(parsed.content.to_json_value())
    });
    assert_inventory_wide_clean(&report);
}

#[cfg(has_generated_rtl_const_expr_parser)]
#[test]
fn auto_gate_rtl_const_expr_inventory_wide_shape() {
    use pgen::generated_parsers::rtl_const_expr::RtlConstExprParser;

    let inv = read_inventory_artifact(
        &repo_generated_dir().join("rtl_const_expr_return_annotations.json"),
    )
    .expect("rtl_const_expr inventory");
    let samples: Vec<String> = vec!["1".into(), "1+2".into(), "1*(2+3)".into(), "a&b".into()];
    let report = run_inventory_wide_auto_gate(&inv, &samples, |input| {
        let mut parser =
            RtlConstExprParser::new(input, runtime_logger_box("auto_gate.rtl_const_expr"));
        let parsed = parser
            .parse_full_rtl_const_expr()
            .map_err(|err| err.to_string())?;
        Ok(parsed.content.to_json_value())
    });
    assert_inventory_wide_clean(&report);
}

#[cfg(has_generated_rtl_frontend_parser)]
#[test]
fn auto_gate_rtl_frontend_inventory_wide_shape() {
    use pgen::generated_parsers::rtl_frontend::RtlFrontendParser;

    let inv = read_inventory_artifact(
        &repo_generated_dir().join("rtl_frontend_return_annotations.json"),
    )
    .expect("rtl_frontend inventory");
    let samples: Vec<String> = vec!["module m; endmodule\n".into()];
    let report = run_inventory_wide_auto_gate(&inv, &samples, |input| {
        let mut parser =
            RtlFrontendParser::new(input, runtime_logger_box("auto_gate.rtl_frontend"));
        let parsed = parser
            .parse_full_rtl_frontend_file()
            .map_err(|err| err.to_string())?;
        Ok(parsed.content.to_json_value())
    });
    assert_inventory_wide_clean(&report);
}

#[cfg(has_generated_json_parser)]
#[test]
fn auto_gate_json_inventory_wide_shape() {
    use pgen::generated_parsers::json::JsonParser;

    let inv =
        read_inventory_artifact(&repo_generated_dir().join("json_return_annotations.json"))
            .expect("json inventory");
    let samples: Vec<String> = vec![
        "null".into(),
        "true".into(),
        "false".into(),
        "42".into(),
        "\"hello\"".into(),
        "{\"k\":1}".into(),
        "[1,2,3]".into(),
        "{\"a\": [1, 2, {\"b\": null}]}".into(),
    ];
    let report = run_inventory_wide_auto_gate(&inv, &samples, |input| {
        let mut parser = JsonParser::new(input, runtime_logger_box("auto_gate.json"));
        let parsed = parser
            .parse_full_json()
            .map_err(|err| err.to_string())?;
        Ok(parsed.content.to_json_value())
    });
    assert_inventory_wide_clean(&report);
}

#[cfg(has_generated_vhdl_parser)]
#[test]
fn auto_gate_vhdl_inventory_wide_shape() {
    use pgen::generated_parsers::vhdl::VhdlParser;

    let inv =
        read_inventory_artifact(&repo_generated_dir().join("vhdl_return_annotations.json"))
            .expect("vhdl inventory");
    let samples: Vec<String> = vec!["entity e is end e;\n".into()];
    let report = run_inventory_wide_auto_gate(&inv, &samples, |input| {
        let mut parser = VhdlParser::new(input, runtime_logger_box("auto_gate.vhdl"));
        let parsed = parser
            .parse_full_vhdl_file()
            .map_err(|err| err.to_string())?;
        Ok(parsed.content.to_json_value())
    });
    assert_inventory_wide_clean(&report);
}

#[cfg(has_generated_systemverilog_preprocessor_parser)]
#[test]
fn auto_gate_systemverilog_preprocessor_inventory_wide_shape() {
    use pgen::generated_parsers::systemverilog_preprocessor::SystemverilogPreprocessorParser;

    let inv = read_inventory_artifact(
        &repo_generated_dir().join("systemverilog_preprocessor_return_annotations.json"),
    )
    .expect("systemverilog_preprocessor inventory");
    let samples: Vec<String> = vec!["`define FOO 1\n".into()];
    let report = run_inventory_wide_auto_gate(&inv, &samples, |input| {
        let mut parser = SystemverilogPreprocessorParser::new(
            input,
            runtime_logger_box("auto_gate.systemverilog_preprocessor"),
        );
        let parsed = parser
            .parse_full_systemverilog_preprocessor_file()
            .map_err(|err| err.to_string())?;
        Ok(parsed.content.to_json_value())
    });
    assert_inventory_wide_clean(&report);
}

#[cfg(has_generated_systemverilog_parser)]
#[test]
fn auto_gate_systemverilog_inventory_wide_shape() {
    use pgen::generated_parsers::systemverilog::SystemverilogParser;

    let inv =
        read_inventory_artifact(&repo_generated_dir().join("systemverilog_return_annotations.json"))
            .expect("systemverilog inventory");
    // SV's inventory has one declared annotation, on
    // `systemverilog_parseable_file`. The grammar's entry rule is
    // `systemverilog_file`, which wraps `systemverilog_parseable_file`
    // somewhere down its body. The inventory-wide gate descends through
    // the parse tree and finds the nested `{type:
    // "systemverilog_parseable_file", ...}` object regardless of how deep
    // the wrapping goes — that's exactly what it was designed for.
    let samples: Vec<String> = vec!["module m; endmodule\n".into()];
    let report = run_inventory_wide_auto_gate(&inv, &samples, |input| {
        let mut parser = SystemverilogParser::new(input, runtime_logger_box("auto_gate.systemverilog"));
        let parsed = parser
            .parse_full_systemverilog_file()
            .map_err(|err| err.to_string())?;
        Ok(parsed.content.to_json_value())
    });
    assert_inventory_wide_clean(&report);
}

#[cfg(feature = "ebnf_dual_run")]
#[test]
fn auto_gate_ebnf_inventory_wide_shape() {
    use pgen::ebnf_generated_parser::EbnfParser;

    let inv =
        read_inventory_artifact(&repo_generated_dir().join("ebnf_return_annotations.json"))
            .expect("ebnf inventory");
    // EBNF inventory has 122 declared annotations. The samples below
    // exercise grammar declarations + a few common rule-body shapes.
    // Coverage will surface in the report's
    // `discriminators_not_covered()` for visibility.
    let samples: Vec<String> = vec![
        "rule_a := 'x' .\n".into(),
        "rule_b := 'a' | 'b' .\n".into(),
        "rule_c := 'a' 'b' 'c' .\n".into(),
        "rule_d := 'x'+ .\n".into(),
    ];
    let report = run_inventory_wide_auto_gate(&inv, &samples, |input| {
        let mut parser = EbnfParser::new(input, runtime_logger_box("auto_gate.ebnf"));
        let parsed = parser
            .parse_full_grammar_file()
            .map_err(|err| err.to_string())?;
        Ok(parsed.content.to_json_value())
    });
    assert_inventory_wide_clean(&report);
}

// Suppress dead_code warning from importing ParseContent for documentation.
#[allow(dead_code)]
const _PARSE_CONTENT_USED_VIA_TO_JSON_VALUE: Option<ParseContent<'_>> = None;
