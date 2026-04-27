//! Per-grammar integration tests for the auto-generated return-annotation
//! shape gate.
//!
//! For each grammar whose generated parser is available in this build, the
//! test:
//!   1. Reads the pipeline-emitted `<grammar>_return_annotations.json`
//!      inventory.
//!   2. Picks the entry rule's declared annotation.
//!   3. Runs the grammar's `parse_full_<entry>` method on a small sample
//!      list, converting each parse output to typed JSON via
//!      `ParseContent::to_json_value()`.
//!   4. Asserts each sample's typed JSON matches the entry rule's declared
//!      shape via `auto_return_annotation_shape_gate::run_entry_rule_auto_gate`.
//!
//! No per-grammar shape descriptors are hand-rolled — the gate derives
//! them from the inventory's `raw_text` field. Adding a new sample is a
//! one-line `samples.push(...)`; adding a new grammar is one new test
//! that wires up the entry rule + parse_full method + a couple of inputs.

#![cfg(feature = "generated_parsers")]

use pgen::ast_pipeline::{ParseContent, runtime_logger_box};
use pgen::auto_return_annotation_shape_gate::{
    AutoGateReport, read_inventory_artifact, run_entry_rule_auto_gate,
};
use std::path::PathBuf;

fn repo_generated_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("rust manifest parent")
        .join("generated")
}

fn assert_gate_report_clean(report: &AutoGateReport) {
    assert!(
        !report.entry_has_no_annotation,
        "[{}] entry rule '{}' has no declared annotation in the inventory; \
         either the inventory artifact is stale (regenerate) or the test's \
         entry_rule name disagrees with the grammar",
        report.grammar, report.entry_rule
    );
    assert!(
        !report.failures.is_empty() || !report.entry_skipped_no_enforceable_shape,
        "[{}] entry rule '{}' has only Passthrough/Unknown annotation — \
         nothing to enforce. If this is intentional, drop the test; \
         otherwise pick a grammar entry rule whose annotation is an Object \
         or Array literal.",
        report.grammar, report.entry_rule
    );
    assert_eq!(
        report.failures,
        Vec::<String>::new(),
        "[{}] entry rule '{}' shape gate failures (samples_checked={}):\n  {}",
        report.grammar,
        report.entry_rule,
        report.samples_checked,
        report.failures.join("\n  "),
    );
}

#[test]
fn auto_gate_regex_entry_rule_shape() {
    use pgen::generated_parsers::regex::RegexParser;

    let inv =
        read_inventory_artifact(&repo_generated_dir().join("regex_return_annotations.json"))
            .expect("regex inventory");
    let samples: Vec<String> = vec!["a".into(), "abc".into(), "a|b".into(), "(a|b)*c".into()];
    let report = run_entry_rule_auto_gate(&inv, "regex", &samples, |input| {
        let mut parser = RegexParser::new(input, runtime_logger_box("auto_gate.regex"));
        let parsed = parser
            .parse_full_regex()
            .map_err(|err| err.to_string())?;
        Ok(parsed.content.to_json_value())
    });
    assert_gate_report_clean(&report);
}

#[test]
fn auto_gate_return_annotation_no_entry_annotation_skipped() {
    use pgen::generated_parsers::return_annotation::Return_annotationParser;

    let inv = read_inventory_artifact(
        &repo_generated_dir().join("return_annotation_return_annotations.json"),
    )
    .expect("return_annotation inventory");
    // `return_annotation` grammar's entry rule itself has no listed
    // annotation (its `-> $2` is per-branch and rule-level passthrough
    // semantics flow through). The gate should report
    // `entry_has_no_annotation = true` cleanly. This test exercises that
    // path so future additions of an entry annotation surface as a
    // behavior change.
    let samples: Vec<String> = vec!["$1".into()];
    let report = run_entry_rule_auto_gate(&inv, "return_annotation", &samples, |input| {
        let mut parser =
            Return_annotationParser::new(input, runtime_logger_box("auto_gate.return_annotation"));
        let parsed = parser
            .parse_full_return_annotation()
            .map_err(|err| err.to_string())?;
        Ok(parsed.content.to_json_value())
    });
    assert!(
        report.entry_has_no_annotation,
        "expected return_annotation entry rule to have no declared \
         annotation today; got report={:?}",
        report
    );
}

#[test]
fn auto_gate_semantic_annotation_entry_rule_shape() {
    use pgen::generated_parsers::semantic_annotation::Semantic_annotationParser;

    let inv = read_inventory_artifact(
        &repo_generated_dir().join("semantic_annotation_return_annotations.json"),
    )
    .expect("semantic_annotation inventory");
    let samples: Vec<String> = vec![
        "@type: integer".into(),
        "@priority: [9, 1]".into(),
        "@category: constraint".into(),
    ];
    let report = run_entry_rule_auto_gate(&inv, "semantic_annotation", &samples, |input| {
        let mut parser =
            Semantic_annotationParser::new(input, runtime_logger_box("auto_gate.semantic_annotation"));
        let parsed = parser
            .parse_full_semantic_annotation()
            .map_err(|err| err.to_string())?;
        Ok(parsed.content.to_json_value())
    });
    assert_gate_report_clean(&report);
}

#[cfg(has_generated_rtl_const_expr_parser)]
#[test]
fn auto_gate_rtl_const_expr_entry_rule_shape() {
    use pgen::generated_parsers::rtl_const_expr::RtlConstExprParser;

    let inv = read_inventory_artifact(
        &repo_generated_dir().join("rtl_const_expr_return_annotations.json"),
    )
    .expect("rtl_const_expr inventory");
    let samples: Vec<String> = vec!["1".into(), "1+2".into()];
    let report = run_entry_rule_auto_gate(&inv, "rtl_const_expr", &samples, |input| {
        let mut parser =
            RtlConstExprParser::new(input, runtime_logger_box("auto_gate.rtl_const_expr"));
        let parsed = parser
            .parse_full_rtl_const_expr()
            .map_err(|err| err.to_string())?;
        Ok(parsed.content.to_json_value())
    });
    assert_gate_report_clean(&report);
}

#[cfg(has_generated_rtl_frontend_parser)]
#[test]
fn auto_gate_rtl_frontend_entry_rule_shape() {
    use pgen::generated_parsers::rtl_frontend::RtlFrontendParser;

    let inv = read_inventory_artifact(
        &repo_generated_dir().join("rtl_frontend_return_annotations.json"),
    )
    .expect("rtl_frontend inventory");
    let samples: Vec<String> = vec!["module m; endmodule\n".into()];
    let report = run_entry_rule_auto_gate(&inv, "rtl_frontend_file", &samples, |input| {
        let mut parser =
            RtlFrontendParser::new(input, runtime_logger_box("auto_gate.rtl_frontend"));
        let parsed = parser
            .parse_full_rtl_frontend_file()
            .map_err(|err| err.to_string())?;
        Ok(parsed.content.to_json_value())
    });
    assert_gate_report_clean(&report);
}

#[cfg(has_generated_json_parser)]
#[test]
fn auto_gate_json_entry_rule_shape() {
    use pgen::generated_parsers::json::JsonParser;

    let inv =
        read_inventory_artifact(&repo_generated_dir().join("json_return_annotations.json"))
            .expect("json inventory");
    let samples: Vec<String> = vec!["null".into(), "true".into(), "{\"k\":1}".into(), "[1,2,3]".into()];
    let report = run_entry_rule_auto_gate(&inv, "json", &samples, |input| {
        let mut parser = JsonParser::new(input, runtime_logger_box("auto_gate.json"));
        let parsed = parser
            .parse_full_json()
            .map_err(|err| err.to_string())?;
        Ok(parsed.content.to_json_value())
    });
    assert_gate_report_clean(&report);
}

#[cfg(has_generated_vhdl_parser)]
#[test]
fn auto_gate_vhdl_entry_rule_shape() {
    use pgen::generated_parsers::vhdl::VhdlParser;

    let inv =
        read_inventory_artifact(&repo_generated_dir().join("vhdl_return_annotations.json"))
            .expect("vhdl inventory");
    let samples: Vec<String> = vec!["entity e is end e;\n".into()];
    let report = run_entry_rule_auto_gate(&inv, "vhdl_file", &samples, |input| {
        let mut parser = VhdlParser::new(input, runtime_logger_box("auto_gate.vhdl"));
        let parsed = parser
            .parse_full_vhdl_file()
            .map_err(|err| err.to_string())?;
        Ok(parsed.content.to_json_value())
    });
    assert_gate_report_clean(&report);
}

#[cfg(has_generated_systemverilog_preprocessor_parser)]
#[test]
fn auto_gate_systemverilog_preprocessor_entry_rule_shape() {
    use pgen::generated_parsers::systemverilog_preprocessor::SystemverilogPreprocessorParser;

    let inv = read_inventory_artifact(
        &repo_generated_dir().join("systemverilog_preprocessor_return_annotations.json"),
    )
    .expect("systemverilog_preprocessor inventory");
    let samples: Vec<String> = vec!["`define FOO 1\n".into()];
    let report = run_entry_rule_auto_gate(
        &inv,
        "systemverilog_preprocessor_file",
        &samples,
        |input| {
            let mut parser = SystemverilogPreprocessorParser::new(
                input,
                runtime_logger_box("auto_gate.systemverilog_preprocessor"),
            );
            let parsed = parser
                .parse_full_systemverilog_preprocessor_file()
                .map_err(|err| err.to_string())?;
            Ok(parsed.content.to_json_value())
        },
    );
    assert_gate_report_clean(&report);
}

// `systemverilog` and `ebnf` entry rules are deferred:
//   * systemverilog: the inventory's only annotation is on
//     `systemverilog_parseable_file`, but `parse_full_systemverilog_file`
//     wraps that rule via `systemverilog_file` (which has no annotation),
//     so the parser's top-level content is not directly the
//     `parseable_file` typed object. Wiring this needs either a
//     `parse_full_systemverilog_parseable_file` entry or a typed-Json
//     descent step in the gate. Tracked as a follow-up.
//   * ebnf: regenerated parser is large (4MB); compiling for tests adds
//     wall time. Wiring this is mechanical once the build cost is
//     accepted; tracked as a follow-up.

// Suppress dead_code warning from importing ParseContent for documentation.
#[allow(dead_code)]
const _PARSE_CONTENT_USED_VIA_TO_JSON_VALUE: Option<ParseContent<'_>> = None;
