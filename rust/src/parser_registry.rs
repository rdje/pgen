//! Registry for generated parser adapters used by parseability and round-trip checks.
//!
//! This centralizes grammar-name dispatch so new generated grammars are added in one place.

use crate::ast_pipeline::{ParseNode, UnifiedSemanticAST, runtime_logger, runtime_logger_box};
#[cfg(feature = "ebnf_dual_run")]
use crate::ebnf_generated_parser::EbnfParser;
#[cfg(has_generated_json_parser)]
use crate::generated_parsers::json::JsonParser;
#[cfg(has_generated_regex_parser)]
use crate::generated_parsers::regex::RegexParser;
#[cfg(has_generated_rtl_const_expr_parser)]
use crate::generated_parsers::rtl_const_expr::RtlConstExprParser;
#[cfg(has_generated_rtl_frontend_parser)]
use crate::generated_parsers::rtl_frontend::RtlFrontendParser;
#[cfg(has_generated_systemverilog_parser)]
use crate::generated_parsers::systemverilog::SystemverilogParser;
#[cfg(has_generated_systemverilog_preprocessor_parser)]
use crate::generated_parsers::systemverilog_preprocessor::SystemverilogPreprocessorParser;
#[cfg(has_generated_vhdl_parser)]
use crate::generated_parsers::vhdl::VhdlParser;
use crate::generated_parsers::{
    return_annotation::Return_annotationParser, semantic_annotation::Semantic_annotationParser,
};
use crate::regex_compile_validation::validate_regex_compile_contract;
use serde_json::Value as JsonValue;
#[cfg(has_generated_regex_parser)]
const GENERATED_REGEX_WORKER_STACK_BYTES: usize = 8 * 1024 * 1024;

type ParseSampleFn = fn(&str) -> bool;

fn normalize_generated_grammar_profile<'a>(
    grammar_name: &str,
    grammar_profile: Option<&'a str>,
) -> Option<&'a str> {
    let profile = grammar_profile?.trim();
    if profile.is_empty() {
        return None;
    }
    match grammar_name {
        "systemverilog" => match profile.to_ascii_lowercase().as_str() {
            "2017" | "ieee1800-2017" | "ieee_1800_2017" => Some("sv_2017"),
            "2023" | "ieee1800-2023" | "ieee_1800_2023" => Some("sv_2023"),
            _ => grammar_profile,
        },
        _ => grammar_profile,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GeneratedParserRegistryEntry {
    pub grammar_name: &'static str,
    parse_sample: ParseSampleFn,
}

impl GeneratedParserRegistryEntry {
    fn parse(&self, sample: &str) -> bool {
        (self.parse_sample)(sample)
    }
}

fn parse_with_return_annotation(sample: &str) -> bool {
    let mut parser =
        Return_annotationParser::new(sample, runtime_logger_box("generated.return_annotation"));
    parser.parse_full_return_annotation().is_ok()
}

fn parse_with_return_annotation_detail(sample: &str) -> Result<(), String> {
    let mut parser =
        Return_annotationParser::new(sample, runtime_logger_box("generated.return_annotation"));
    parser
        .parse_full_return_annotation()
        .map(|_| ())
        .map_err(|err| err.to_string())
}

fn parse_with_return_annotation_ast_json(sample: &str) -> Result<JsonValue, String> {
    let mut parser =
        Return_annotationParser::new(sample, runtime_logger_box("generated.return_annotation"));
    let parsed = parser
        .parse_full_return_annotation()
        .map_err(|err| err.to_string())?;
    parse_node_to_json(&parsed)
}

fn parse_with_semantic_annotation(sample: &str) -> bool {
    let mut parser =
        Semantic_annotationParser::new(sample, runtime_logger_box("generated.semantic_annotation"));
    parser.parse_full_semantic_annotation().is_ok()
}

fn parse_with_semantic_annotation_detail(sample: &str) -> Result<(), String> {
    let mut parser =
        Semantic_annotationParser::new(sample, runtime_logger_box("generated.semantic_annotation"));
    parser
        .parse_full_semantic_annotation()
        .map(|_| ())
        .map_err(|err| err.to_string())
}

fn parse_with_semantic_annotation_ast_json(sample: &str) -> Result<JsonValue, String> {
    let mut parser =
        Semantic_annotationParser::new(sample, runtime_logger_box("generated.semantic_annotation"));
    let parsed = parser
        .parse_full_semantic_annotation()
        .map_err(|err| err.to_string())?;
    parse_node_to_json(&parsed)
}

fn parse_with_builtin_return_annotation(sample: &str) -> bool {
    // Built-in return grammar is a strict subset of return_annotation grammar.
    parse_with_return_annotation(sample)
}

fn parse_with_builtin_return_annotation_detail(sample: &str) -> Result<(), String> {
    parse_with_return_annotation_detail(sample)
}

fn parse_with_builtin_return_annotation_ast_json(sample: &str) -> Result<JsonValue, String> {
    // Built-in return grammar is a strict subset of return_annotation grammar.
    parse_with_return_annotation_ast_json(sample)
}

fn parse_with_builtin_semantic_annotation(sample: &str) -> bool {
    // Built-in semantic parser behavior is intentionally permissive and marker-based.
    // Parseability for builtin_semantic_annotation must follow this bootstrap contract,
    // not the stricter full semantic_annotation grammar.
    let logger = runtime_logger("bootstrap.semantic_annotation");
    UnifiedSemanticAST::parse_bootstrap(sample, &logger).is_ok()
}

fn parse_with_builtin_semantic_annotation_detail(sample: &str) -> Result<(), String> {
    let logger = runtime_logger("bootstrap.semantic_annotation");
    UnifiedSemanticAST::parse_bootstrap(sample, &logger)
        .map(|_| ())
        .map_err(|err| err.to_string())
}

fn parse_with_builtin_semantic_annotation_ast_json(sample: &str) -> Result<JsonValue, String> {
    let logger = runtime_logger("bootstrap.semantic_annotation");
    let parsed =
        UnifiedSemanticAST::parse_bootstrap(sample, &logger).map_err(|err| err.to_string())?;
    serde_json::to_value(parsed)
        .map_err(|err| format!("failed to serialize bootstrap semantic AST: {}", err))
}

#[cfg(feature = "ebnf_dual_run")]
fn parse_with_ebnf(sample: &str) -> bool {
    let mut parser = EbnfParser::new(sample, runtime_logger_box("generated.ebnf"));
    parser.parse_full_grammar_file().is_ok()
}

#[cfg(feature = "ebnf_dual_run")]
fn parse_with_ebnf_detail(sample: &str) -> Result<(), String> {
    let mut parser = EbnfParser::new(sample, runtime_logger_box("generated.ebnf"));
    parser
        .parse_full_grammar_file()
        .map(|_| ())
        .map_err(|err| err.to_string())
}

#[cfg(feature = "ebnf_dual_run")]
fn parse_with_ebnf_ast_json(sample: &str) -> Result<JsonValue, String> {
    let mut parser = EbnfParser::new(sample, runtime_logger_box("generated.ebnf"));
    let parsed = parser
        .parse_full_grammar_file()
        .map_err(|err| err.to_string())?;
    parse_node_to_json(&parsed)
}

#[cfg(has_generated_json_parser)]
fn parse_with_json(sample: &str) -> bool {
    let mut parser = JsonParser::new(sample, runtime_logger_box("generated.json"));
    parser.parse_full_json().is_ok()
}

#[cfg(has_generated_json_parser)]
fn parse_with_json_detail(sample: &str) -> Result<(), String> {
    let mut parser = JsonParser::new(sample, runtime_logger_box("generated.json"));
    parser
        .parse_full_json()
        .map(|_| ())
        .map_err(|err| err.to_string())
}

#[cfg(has_generated_json_parser)]
fn parse_with_json_ast_json(sample: &str) -> Result<JsonValue, String> {
    let mut parser = JsonParser::new(sample, runtime_logger_box("generated.json"));
    let parsed = parser.parse_full_json().map_err(|err| err.to_string())?;
    parse_node_to_json(&parsed)
}

#[cfg(has_generated_regex_parser)]
fn parse_with_regex(sample: &str) -> bool {
    parse_with_regex_detail(sample).is_ok()
}

#[cfg(has_generated_regex_parser)]
fn run_generated_regex_on_dedicated_stack<T, F>(sample: &str, f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce(String) -> Result<T, String> + Send + 'static,
{
    let owned_sample = sample.to_string();
    let handle = std::thread::Builder::new()
        .name("pgen-generated-regex".to_string())
        .stack_size(GENERATED_REGEX_WORKER_STACK_BYTES)
        .spawn(move || f(owned_sample))
        .map_err(|err| format!("failed to spawn generated.regex worker thread: {}", err))?;
    handle
        .join()
        .map_err(|_| "generated.regex worker thread panicked".to_string())?
}

#[cfg(has_generated_regex_parser)]
fn parse_with_regex_detail(sample: &str) -> Result<(), String> {
    run_generated_regex_on_dedicated_stack(sample, |owned_sample| {
        let mut parser = RegexParser::new(&owned_sample, runtime_logger_box("generated.regex"));
        parser.parse_full_regex().map_err(|err| err.to_string())?;
        validate_regex_compile_contract(&owned_sample).map_err(|err| err.message)
    })
}

#[cfg(has_generated_regex_parser)]
fn parse_with_regex_ast_json(sample: &str) -> Result<JsonValue, String> {
    run_generated_regex_on_dedicated_stack(sample, |owned_sample| {
        let mut parser = RegexParser::new(&owned_sample, runtime_logger_box("generated.regex"));
        let parsed = parser.parse_full_regex().map_err(|err| err.to_string())?;
        validate_regex_compile_contract(&owned_sample).map_err(|err| err.message)?;
        parse_node_to_json(&parsed)
    })
}

#[cfg(has_generated_rtl_const_expr_parser)]
fn parse_with_rtl_const_expr(sample: &str) -> bool {
    let mut parser =
        RtlConstExprParser::new(sample, runtime_logger_box("generated.rtl_const_expr"));
    parser.parse_full_rtl_const_expr().is_ok()
}

#[cfg(has_generated_rtl_const_expr_parser)]
fn parse_with_rtl_const_expr_detail(sample: &str) -> Result<(), String> {
    let mut parser =
        RtlConstExprParser::new(sample, runtime_logger_box("generated.rtl_const_expr"));
    parser
        .parse_full_rtl_const_expr()
        .map(|_| ())
        .map_err(|err| err.to_string())
}

#[cfg(has_generated_rtl_const_expr_parser)]
fn parse_with_rtl_const_expr_ast_json(sample: &str) -> Result<JsonValue, String> {
    let mut parser =
        RtlConstExprParser::new(sample, runtime_logger_box("generated.rtl_const_expr"));
    let parsed = parser
        .parse_full_rtl_const_expr()
        .map_err(|err| err.to_string())?;
    parse_node_to_json(&parsed)
}

#[cfg(has_generated_rtl_frontend_parser)]
fn parse_with_rtl_frontend(sample: &str) -> bool {
    let mut parser = RtlFrontendParser::new(sample, runtime_logger_box("generated.rtl_frontend"));
    parser.parse_full_rtl_frontend_file().is_ok()
}

#[cfg(has_generated_rtl_frontend_parser)]
fn parse_with_rtl_frontend_detail(sample: &str) -> Result<(), String> {
    let mut parser = RtlFrontendParser::new(sample, runtime_logger_box("generated.rtl_frontend"));
    parser
        .parse_full_rtl_frontend_file()
        .map(|_| ())
        .map_err(|err| err.to_string())
}

#[cfg(has_generated_rtl_frontend_parser)]
fn parse_with_rtl_frontend_ast_json(sample: &str) -> Result<JsonValue, String> {
    let mut parser = RtlFrontendParser::new(sample, runtime_logger_box("generated.rtl_frontend"));
    let parsed = parser
        .parse_full_rtl_frontend_file()
        .map_err(|err| err.to_string())?;
    parse_node_to_json(&parsed)
}

#[cfg(has_generated_systemverilog_parser)]
fn parse_with_systemverilog(sample: &str) -> bool {
    parse_with_systemverilog_profile(sample, None)
}

#[cfg(has_generated_systemverilog_parser)]
fn parse_with_systemverilog_profile(sample: &str, grammar_profile: Option<&str>) -> bool {
    let mut parser =
        SystemverilogParser::new(sample, runtime_logger_box("generated.systemverilog"));
    parser.set_grammar_profile(normalize_generated_grammar_profile(
        "systemverilog",
        grammar_profile,
    ));
    parser.parse_full_systemverilog_file().is_ok()
}

#[cfg(has_generated_systemverilog_parser)]
fn parse_with_systemverilog_detail_profile(
    sample: &str,
    grammar_profile: Option<&str>,
) -> Result<(), String> {
    let mut parser =
        SystemverilogParser::new(sample, runtime_logger_box("generated.systemverilog"));
    parser.set_grammar_profile(normalize_generated_grammar_profile(
        "systemverilog",
        grammar_profile,
    ));
    parser
        .parse_full_systemverilog_file()
        .map(|_| ())
        .map_err(|err| err.to_string())
}

#[cfg(has_generated_systemverilog_parser)]
fn parse_with_systemverilog_ast_json(sample: &str) -> Result<JsonValue, String> {
    parse_with_systemverilog_ast_json_profile(sample, None)
}

#[cfg(has_generated_systemverilog_parser)]
fn parse_with_systemverilog_ast_json_profile(
    sample: &str,
    grammar_profile: Option<&str>,
) -> Result<JsonValue, String> {
    let mut parser =
        SystemverilogParser::new(sample, runtime_logger_box("generated.systemverilog"));
    parser.set_grammar_profile(normalize_generated_grammar_profile(
        "systemverilog",
        grammar_profile,
    ));
    let parsed = parser
        .parse_full_systemverilog_file()
        .map_err(|err| err.to_string())?;
    parse_node_to_json(&parsed)
}

#[cfg(has_generated_systemverilog_preprocessor_parser)]
fn parse_with_systemverilog_preprocessor(sample: &str) -> bool {
    let mut parser = SystemverilogPreprocessorParser::new(
        sample,
        runtime_logger_box("generated.systemverilog_preprocessor"),
    );
    parser.parse_full_systemverilog_preprocessor_file().is_ok()
}

#[cfg(has_generated_systemverilog_preprocessor_parser)]
fn parse_with_systemverilog_preprocessor_detail(sample: &str) -> Result<(), String> {
    let mut parser = SystemverilogPreprocessorParser::new(
        sample,
        runtime_logger_box("generated.systemverilog_preprocessor"),
    );
    parser
        .parse_full_systemverilog_preprocessor_file()
        .map(|_| ())
        .map_err(|err| err.to_string())
}

#[cfg(has_generated_systemverilog_preprocessor_parser)]
fn parse_with_systemverilog_preprocessor_ast_json(sample: &str) -> Result<JsonValue, String> {
    let mut parser = SystemverilogPreprocessorParser::new(
        sample,
        runtime_logger_box("generated.systemverilog_preprocessor"),
    );
    let parsed = parser
        .parse_full_systemverilog_preprocessor_file()
        .map_err(|err| err.to_string())?;
    parse_node_to_json(&parsed)
}

#[cfg(has_generated_vhdl_parser)]
fn parse_with_vhdl(sample: &str) -> bool {
    let mut parser = VhdlParser::new(sample, runtime_logger_box("generated.vhdl"));
    parser.parse_full_vhdl_file().is_ok()
}

#[cfg(has_generated_vhdl_parser)]
fn parse_with_vhdl_detail(sample: &str) -> Result<(), String> {
    let mut parser = VhdlParser::new(sample, runtime_logger_box("generated.vhdl"));
    parser
        .parse_full_vhdl_file()
        .map(|_| ())
        .map_err(|err| err.to_string())
}

#[cfg(has_generated_vhdl_parser)]
fn parse_with_vhdl_ast_json(sample: &str) -> Result<JsonValue, String> {
    let mut parser = VhdlParser::new(sample, runtime_logger_box("generated.vhdl"));
    let parsed = parser
        .parse_full_vhdl_file()
        .map_err(|err| err.to_string())?;
    parse_node_to_json(&parsed)
}

fn parse_node_to_json(node: &ParseNode<'_>) -> Result<JsonValue, String> {
    serde_json::to_value(node).map_err(|err| format!("failed to serialize parse tree: {}", err))
}

static GENERATED_PARSER_REGISTRY: &[GeneratedParserRegistryEntry] = &[
    GeneratedParserRegistryEntry {
        grammar_name: "return_annotation",
        parse_sample: parse_with_return_annotation,
    },
    GeneratedParserRegistryEntry {
        grammar_name: "semantic_annotation",
        parse_sample: parse_with_semantic_annotation,
    },
    GeneratedParserRegistryEntry {
        grammar_name: "builtin_return_annotation",
        parse_sample: parse_with_builtin_return_annotation,
    },
    GeneratedParserRegistryEntry {
        grammar_name: "builtin_semantic_annotation",
        parse_sample: parse_with_builtin_semantic_annotation,
    },
    #[cfg(feature = "ebnf_dual_run")]
    GeneratedParserRegistryEntry {
        grammar_name: "ebnf",
        parse_sample: parse_with_ebnf,
    },
    #[cfg(has_generated_json_parser)]
    GeneratedParserRegistryEntry {
        grammar_name: "json",
        parse_sample: parse_with_json,
    },
    #[cfg(has_generated_regex_parser)]
    GeneratedParserRegistryEntry {
        grammar_name: "regex",
        parse_sample: parse_with_regex,
    },
    #[cfg(has_generated_rtl_const_expr_parser)]
    GeneratedParserRegistryEntry {
        grammar_name: "rtl_const_expr",
        parse_sample: parse_with_rtl_const_expr,
    },
    #[cfg(has_generated_rtl_frontend_parser)]
    GeneratedParserRegistryEntry {
        grammar_name: "rtl_frontend",
        parse_sample: parse_with_rtl_frontend,
    },
    #[cfg(has_generated_systemverilog_parser)]
    GeneratedParserRegistryEntry {
        grammar_name: "systemverilog",
        parse_sample: parse_with_systemverilog,
    },
    #[cfg(has_generated_systemverilog_preprocessor_parser)]
    GeneratedParserRegistryEntry {
        grammar_name: "systemverilog_preprocessor",
        parse_sample: parse_with_systemverilog_preprocessor,
    },
    #[cfg(has_generated_vhdl_parser)]
    GeneratedParserRegistryEntry {
        grammar_name: "vhdl",
        parse_sample: parse_with_vhdl,
    },
    // Add future grammars here once their generated parser artifacts compile cleanly.
    // Examples: json, regex, systemverilog, vhdl.
];

fn find_entry(grammar_name: &str) -> Option<&'static GeneratedParserRegistryEntry> {
    GENERATED_PARSER_REGISTRY
        .iter()
        .find(|entry| entry.grammar_name == grammar_name)
}

pub fn supports_grammar(grammar_name: &str) -> bool {
    find_entry(grammar_name).is_some()
}

pub fn parse_sample(grammar_name: &str, sample: &str) -> Option<bool> {
    parse_sample_with_profile(grammar_name, sample, None)
}

pub fn parse_sample_with_profile(
    grammar_name: &str,
    sample: &str,
    grammar_profile: Option<&str>,
) -> Option<bool> {
    match grammar_name {
        #[cfg(has_generated_systemverilog_parser)]
        "systemverilog" => Some(parse_with_systemverilog_profile(sample, grammar_profile)),
        _ => find_entry(grammar_name).map(|entry| entry.parse(sample)),
    }
}

pub fn parse_sample_detail(grammar_name: &str, sample: &str) -> Option<Result<(), String>> {
    parse_sample_detail_with_profile(grammar_name, sample, None)
}

pub fn parse_sample_detail_with_profile(
    grammar_name: &str,
    sample: &str,
    grammar_profile: Option<&str>,
) -> Option<Result<(), String>> {
    match grammar_name {
        "return_annotation" => Some(parse_with_return_annotation_detail(sample)),
        "semantic_annotation" => Some(parse_with_semantic_annotation_detail(sample)),
        "builtin_return_annotation" => Some(parse_with_builtin_return_annotation_detail(sample)),
        "builtin_semantic_annotation" => {
            Some(parse_with_builtin_semantic_annotation_detail(sample))
        }
        #[cfg(feature = "ebnf_dual_run")]
        "ebnf" => Some(parse_with_ebnf_detail(sample)),
        #[cfg(has_generated_json_parser)]
        "json" => Some(parse_with_json_detail(sample)),
        #[cfg(has_generated_regex_parser)]
        "regex" => Some(parse_with_regex_detail(sample)),
        #[cfg(has_generated_rtl_const_expr_parser)]
        "rtl_const_expr" => Some(parse_with_rtl_const_expr_detail(sample)),
        #[cfg(has_generated_rtl_frontend_parser)]
        "rtl_frontend" => Some(parse_with_rtl_frontend_detail(sample)),
        #[cfg(has_generated_systemverilog_parser)]
        "systemverilog" => Some(parse_with_systemverilog_detail_profile(
            sample,
            grammar_profile,
        )),
        #[cfg(has_generated_systemverilog_preprocessor_parser)]
        "systemverilog_preprocessor" => Some(parse_with_systemverilog_preprocessor_detail(sample)),
        #[cfg(has_generated_vhdl_parser)]
        "vhdl" => Some(parse_with_vhdl_detail(sample)),
        _ => None,
    }
}

pub fn parse_sample_ast_json(
    grammar_name: &str,
    sample: &str,
) -> Option<Result<JsonValue, String>> {
    parse_sample_ast_json_with_profile(grammar_name, sample, None)
}

pub fn parse_sample_ast_json_with_profile(
    grammar_name: &str,
    sample: &str,
    grammar_profile: Option<&str>,
) -> Option<Result<JsonValue, String>> {
    match grammar_name {
        "return_annotation" => Some(parse_with_return_annotation_ast_json(sample)),
        "semantic_annotation" => Some(parse_with_semantic_annotation_ast_json(sample)),
        "builtin_return_annotation" => Some(parse_with_builtin_return_annotation_ast_json(sample)),
        "builtin_semantic_annotation" => {
            Some(parse_with_builtin_semantic_annotation_ast_json(sample))
        }
        #[cfg(feature = "ebnf_dual_run")]
        "ebnf" => Some(parse_with_ebnf_ast_json(sample)),
        #[cfg(has_generated_json_parser)]
        "json" => Some(parse_with_json_ast_json(sample)),
        #[cfg(has_generated_regex_parser)]
        "regex" => Some(parse_with_regex_ast_json(sample)),
        #[cfg(has_generated_rtl_const_expr_parser)]
        "rtl_const_expr" => Some(parse_with_rtl_const_expr_ast_json(sample)),
        #[cfg(has_generated_rtl_frontend_parser)]
        "rtl_frontend" => Some(parse_with_rtl_frontend_ast_json(sample)),
        #[cfg(has_generated_systemverilog_parser)]
        "systemverilog" => Some(parse_with_systemverilog_ast_json_profile(
            sample,
            grammar_profile,
        )),
        #[cfg(has_generated_systemverilog_preprocessor_parser)]
        "systemverilog_preprocessor" => {
            Some(parse_with_systemverilog_preprocessor_ast_json(sample))
        }
        #[cfg(has_generated_vhdl_parser)]
        "vhdl" => Some(parse_with_vhdl_ast_json(sample)),
        _ => None,
    }
}

pub fn registered_grammars() -> Vec<&'static str> {
    GENERATED_PARSER_REGISTRY
        .iter()
        .map(|entry| entry.grammar_name)
        .collect()
}

#[cfg(test)]
mod tests {
    #[cfg(has_generated_rtl_frontend_parser)]
    use serde::Deserialize;
    use std::fs;
    use std::path::PathBuf;

    use super::{parse_sample, parse_sample_ast_json, registered_grammars, supports_grammar};

    #[cfg(has_generated_rtl_frontend_parser)]
    #[derive(Debug, Deserialize)]
    struct RtlFrontendGeneratedContract {
        contract_version: String,
        grammar_name: String,
        purpose: String,
        provenance: String,
        samples: Vec<RtlFrontendGeneratedSample>,
    }

    #[cfg(has_generated_rtl_frontend_parser)]
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
        expected_rule_texts: std::collections::BTreeMap<String, Vec<String>>,
        sample: String,
    }

    #[cfg(has_generated_rtl_frontend_parser)]
    fn collect_rule_names(node: &serde_json::Value, names: &mut Vec<String>) {
        match node {
            serde_json::Value::Array(values) => {
                for value in values {
                    collect_rule_names(value, names);
                }
            }
            serde_json::Value::Object(map) => {
                if let Some(serde_json::Value::String(rule_name)) = map.get("rule_name") {
                    names.push(rule_name.clone());
                }
                for value in map.values() {
                    collect_rule_names(value, names);
                }
            }
            _ => {}
        }
    }

    #[cfg(has_generated_rtl_frontend_parser)]
    fn rtl_frontend_ast_contains_rule(ast_json: &serde_json::Value, rule_name: &str) -> bool {
        let mut names = Vec::new();
        collect_rule_names(ast_json, &mut names);
        names.iter().any(|candidate| candidate == rule_name)
    }

    #[cfg(has_generated_rtl_frontend_parser)]
    fn collect_rule_spans(
        node: &serde_json::Value,
        rule_name: &str,
        spans: &mut Vec<(usize, usize)>,
    ) {
        match node {
            serde_json::Value::Array(values) => {
                for value in values {
                    collect_rule_spans(value, rule_name, spans);
                }
            }
            serde_json::Value::Object(map) => {
                if let Some(serde_json::Value::String(candidate)) = map.get("rule_name") {
                    if candidate == rule_name {
                        if let Some(serde_json::Value::Object(span)) = map.get("span") {
                            if let (
                                Some(serde_json::Value::Number(start)),
                                Some(serde_json::Value::Number(end)),
                            ) = (span.get("start"), span.get("end"))
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

    #[cfg(has_generated_rtl_frontend_parser)]
    fn rtl_frontend_ast_rule_texts(
        sample: &str,
        ast_json: &serde_json::Value,
        rule_name: &str,
    ) -> Vec<String> {
        let mut spans = Vec::new();
        collect_rule_spans(ast_json, rule_name, &mut spans);
        spans
            .into_iter()
            .map(|(start, end)| {
                sample
                    .get(start..end)
                    .unwrap_or_else(|| panic!("invalid span {}..{} for rule '{}'", start, end, rule_name))
                    .trim()
                    .to_string()
            })
            .collect()
    }

    #[cfg(has_generated_rtl_frontend_parser)]
    fn rtl_frontend_generated_contract() -> RtlFrontendGeneratedContract {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test_data/grammar_quality/rtl_frontend_generated_parity_contract_v0.json");
        let raw = fs::read_to_string(&path).expect("read rtl_frontend generated contract");
        serde_json::from_str(&raw).expect("parse rtl_frontend generated contract")
    }

    #[test]
    fn registry_exposes_expected_annotation_grammars() {
        let grammars = registered_grammars();
        assert!(grammars.contains(&"return_annotation"));
        assert!(grammars.contains(&"semantic_annotation"));
        assert!(grammars.contains(&"builtin_return_annotation"));
        assert!(grammars.contains(&"builtin_semantic_annotation"));
    }

    #[cfg(feature = "ebnf_dual_run")]
    #[test]
    fn registry_exposes_ebnf_when_dual_run_enabled() {
        let grammars = registered_grammars();
        assert!(grammars.contains(&"ebnf"));
    }

    #[cfg(has_generated_json_parser)]
    #[test]
    fn registry_exposes_json_when_generated_parser_present() {
        let grammars = registered_grammars();
        assert!(grammars.contains(&"json"));
    }

    #[cfg(has_generated_regex_parser)]
    #[test]
    fn registry_exposes_regex_when_generated_parser_present() {
        let grammars = registered_grammars();
        assert!(grammars.contains(&"regex"));
    }

    #[cfg(has_generated_rtl_const_expr_parser)]
    #[test]
    fn registry_exposes_rtl_const_expr_when_generated_parser_present() {
        let grammars = registered_grammars();
        assert!(grammars.contains(&"rtl_const_expr"));
    }

    #[cfg(has_generated_rtl_frontend_parser)]
    #[test]
    fn registry_exposes_rtl_frontend_when_generated_parser_present() {
        let grammars = registered_grammars();
        assert!(grammars.contains(&"rtl_frontend"));
    }

    #[cfg(has_generated_systemverilog_parser)]
    #[test]
    fn registry_exposes_systemverilog_when_generated_parser_present() {
        let grammars = registered_grammars();
        assert!(grammars.contains(&"systemverilog"));
    }

    #[cfg(has_generated_systemverilog_preprocessor_parser)]
    #[test]
    fn registry_exposes_systemverilog_preprocessor_when_generated_parser_present() {
        let grammars = registered_grammars();
        assert!(grammars.contains(&"systemverilog_preprocessor"));
    }

    #[cfg(has_generated_vhdl_parser)]
    #[test]
    fn registry_exposes_vhdl_when_generated_parser_present() {
        let grammars = registered_grammars();
        assert!(grammars.contains(&"vhdl"));
    }

    #[test]
    fn unknown_grammar_is_not_supported() {
        assert!(!supports_grammar("unknown"));
        assert!(parse_sample("unknown", "anything").is_none());
        assert!(parse_sample_ast_json("unknown", "anything").is_none());
    }

    #[test]
    fn builtin_semantic_parseability_adapter_accepts_marker_and_raw_inputs() {
        assert_eq!(
            parse_sample("builtin_semantic_annotation", "@priority: [9, 1]"),
            Some(true)
        );
        let ast_json = parse_sample_ast_json("builtin_semantic_annotation", "@priority: [9, 1]")
            .expect("ast adapter");
        assert!(ast_json.is_ok());
        assert_eq!(
            parse_sample(
                "builtin_semantic_annotation",
                "str::parse::<u32>().unwrap_or(0)"
            ),
            Some(true)
        );
    }

    #[test]
    fn return_annotation_examples_from_grammar_are_parseable() {
        let samples = [
            "->",
            "-> $1",
            "-> \"literal\"",
            "-> 42",
            "-> true",
            "-> [$1, $2]",
            "-> [$1, $2*]",
            "-> []",
            "-> {type: \"node\"}",
            "-> {key: $1, val: $2}",
            "-> {}",
            "-> $2::2",
            "-> $2::first",
            "-> $2::last",
            "-> $2::2*",
            "-> [$1, $2::1*]",
            "-> $1.value",
            "-> $1[0]",
            "-> (($1)).field[($2::first)]",
        ];

        for sample in samples {
            assert_eq!(
                parse_sample("return_annotation", sample),
                Some(true),
                "return_annotation grammar should accept example '{}'",
                sample
            );
            let ast_json = parse_sample_ast_json("return_annotation", sample)
                .expect("return_annotation ast adapter should exist");
            assert!(
                ast_json.is_ok(),
                "return_annotation AST JSON adapter should serialize '{}'",
                sample
            );
        }
    }

    #[cfg(feature = "ebnf_dual_run")]
    #[test]
    fn ebnf_parseability_adapter_accepts_valid_rule_and_rejects_garbage() {
        assert_eq!(
            parse_sample("ebnf", r#"rule_name := /([a-zA-Z_][a-zA-Z0-9_]*)/"#),
            Some(true)
        );
        assert_eq!(parse_sample("ebnf", ":::not-ebnf:::"), Some(false));
    }

    #[cfg(feature = "ebnf_dual_run")]
    #[test]
    fn ebnf_parseability_adapter_accepts_inline_lookahead_in_sequence() {
        let sample = r#"ports := direction item ( "," !direction item )*
direction := "input" | "output"
item := identifier
identifier := /([a-zA-Z_][a-zA-Z0-9_]*)/"#;
        assert_eq!(parse_sample("ebnf", sample), Some(true));
        let ast_json = parse_sample_ast_json("ebnf", sample).expect("ebnf ast adapter should exist");
        assert!(
            ast_json.is_ok(),
            "ebnf AST JSON adapter should serialize inline lookahead sample"
        );
    }

    #[cfg(has_generated_json_parser)]
    #[test]
    fn json_parseability_adapter_accepts_valid_json_and_rejects_garbage() {
        assert_eq!(parse_sample("json", r#"{"k":[1,true,null]}"#), Some(true));
        assert_eq!(parse_sample("json", "{]"), Some(false));
    }

    #[cfg(has_generated_regex_parser)]
    #[test]
    fn regex_parseability_adapter_accepts_valid_regex_and_rejects_garbage() {
        assert_eq!(parse_sample("regex", ""), Some(true));
        assert_eq!(parse_sample("regex", "\""), Some(true));
        assert_eq!(parse_sample("regex", " *"), Some(true));
        assert_eq!(parse_sample("regex", "\t*"), Some(true));
        assert_eq!(parse_sample("regex", "(foo|bar)+"), Some(true));
        assert_eq!(parse_sample("regex", "(a|)\\1*b"), Some(true));
        assert_eq!(parse_sample("regex", "()2(3)"), Some(true));
        assert_eq!(parse_sample("regex", "(?#)"), Some(true));
        assert_eq!(parse_sample("regex", "a(?)b"), Some(true));
        assert_eq!(parse_sample("regex", "(?s)a.b"), Some(true));
        assert_eq!(parse_sample("regex", "a(?-i)b"), Some(true));
        assert_eq!(parse_sample("regex", "(?^)AB"), Some(true));
        assert_eq!(parse_sample("regex", "(?^-i)AB"), Some(true));
        assert_eq!(parse_sample("regex", "(?^x:C D)"), Some(true));
        assert_eq!(parse_sample("regex", "(?:(?-i)a)b"), Some(true));
        assert_eq!(
            parse_sample(
                "regex",
                "(?x)   ^    a   (?# begins with a)  b\\sc (?# then b c) $ (?# then end)"
            ),
            Some(true)
        );
        assert_eq!(
            parse_sample("regex", "^(?(?=abc)\\w{3}:|\\d\\d)"),
            Some(true)
        );
        assert_eq!(parse_sample("regex", "(?(DEFINE)(a))"), Some(true));
        assert_eq!(parse_sample("regex", "x{,2}(x|b)"), Some(true));
        assert_eq!(parse_sample("regex", "([ab]{,}c|xy)"), Some(true));
        assert_eq!(parse_sample("regex", "a{1,2,3}b"), Some(true));
        assert_eq!(parse_sample("regex", "a{65536"), Some(true));
        assert_eq!(parse_sample("regex", "X{"), Some(true));
        assert_eq!(parse_sample("regex", "X{A"), Some(true));
        assert_eq!(parse_sample("regex", "X{}"), Some(true));
        assert_eq!(parse_sample("regex", "X{1234"), Some(true));
        assert_eq!(parse_sample("regex", "X{12ABC}"), Some(true));
        assert_eq!(parse_sample("regex", "X{1,"), Some(true));
        assert_eq!(parse_sample("regex", "X{,9"), Some(true));
        assert_eq!(parse_sample("regex", "X{,9]"), Some(true));
        assert_eq!(parse_sample("regex", "a{(?#XYZ),2}"), Some(true));
        assert_eq!(parse_sample("regex", r"^\ca\cA\c[;\c:"), Some(true));
        assert_eq!(parse_sample("regex", "([[:]+)"), Some(true));
        assert_eq!(parse_sample("regex", "([[=]+)"), Some(true));
        assert_eq!(parse_sample("regex", "([[.]+)"), Some(true));
        assert_eq!(parse_sample("regex", "[[,abc,]+]"), Some(true));
        assert_eq!(parse_sample("regex", "[[:abcd:xyz]]"), Some(true));
        assert_eq!(parse_sample("regex", r"[abc[:x\]pqr]"), Some(true));
        assert_eq!(parse_sample("regex", "[[:space:]]+"), Some(true));
        assert_eq!(parse_sample("regex", "[[:blank:]]+"), Some(true));
        assert_eq!(parse_sample("regex", "^[:a[:digit:]]+"), Some(true));
        assert_eq!(parse_sample("regex", "^[:a[:digit:]:b]+"), Some(true));
        assert_eq!(parse_sample("regex", r"abc\Q(*+|\Eabc"), Some(true));
        assert_eq!(parse_sample("regex", "^\\p{sc=Latin}"), Some(true));
        assert_eq!(parse_sample("regex", "^\\p{L&}X"), Some(true));
        assert_eq!(parse_sample("regex", "^[[:^alnum:]]"), Some(true));
        assert_eq!(parse_sample("regex", "a]"), Some(true));
        assert_eq!(parse_sample("regex", "(?|a|b)"), Some(true));
        assert_eq!(parse_sample("regex", "(?P<name>a)"), Some(true));
        assert_eq!(parse_sample("regex", "(?P=name)"), Some(true));
        assert_eq!(parse_sample("regex", "^(?P<A>a)?(?(A)a|b)"), Some(true));
        assert_eq!(parse_sample("regex", "^(?(+1)X|Y)(.)"), Some(true));
        assert_eq!(parse_sample("regex", "(?<A>tom|bon)-\\k{A}"), Some(true));
        assert_eq!(parse_sample("regex", "(?&name)"), Some(true));
        assert_eq!(parse_sample("regex", "(?R)"), Some(true));
        assert_eq!(parse_sample("regex", "\\g{1}"), Some(true));
        assert_eq!(parse_sample("regex", "(?C1)"), Some(true));
        assert_eq!(parse_sample("regex", "(*UTF)abc"), Some(true));
        assert_eq!(parse_sample("regex", "(*MARK:A)(*SKIP:B)(C|X)"), Some(true));
        assert_eq!(parse_sample("regex", "(?[\\p{L} - \\p{Lu}])"), Some(true));
        assert_eq!(parse_sample("regex", "^[]cde]"), Some(true));
        assert_eq!(parse_sample("regex", "^[^]cde]"), Some(true));
        assert_eq!(parse_sample("regex", r"\d"), Some(true));
        assert_eq!(parse_sample("regex", r"\bword\b"), Some(true));
        assert_eq!(parse_sample("regex", r"\\"), Some(true));
        assert_eq!(parse_sample("regex", r"^\+?[1-9]\d{1,14}$"), Some(true));
        assert_eq!(
            parse_sample("regex", r"^https?://[^\s/$.?#].[^\s]*$"),
            Some(true)
        );
        assert_eq!(parse_sample("regex", r"ab\idef"), Some(false));
        assert_eq!(parse_sample("regex", r"x{5,4}"), Some(false));
        assert_eq!(parse_sample("regex", r"z{65536}"), Some(false));
        assert_eq!(parse_sample("regex", r"[\B]"), Some(false));
        assert_eq!(parse_sample("regex", r"[z-a]"), Some(false));
        assert_eq!(parse_sample("regex", r"^*"), Some(false));
        assert_eq!(parse_sample("regex", r"(?<=a+)b"), Some(false));
        assert_eq!(parse_sample("regex", "("), Some(false));
    }

    #[cfg(has_generated_regex_parser)]
    #[test]
    fn regex_parseability_adapter_accepts_embedded_code_block_structural_forms() {
        assert_eq!(parse_sample("regex", "(?{payload})"), Some(true));
        assert_eq!(parse_sample("regex", "(?{lua:return x + 1})"), Some(true));
        assert_eq!(
            parse_sample("regex", "(?{javascript:return x + 1;})"),
            Some(true)
        );
        assert_eq!(
            parse_sample("regex", "(?{{ nested { braces } }})"),
            Some(true)
        );
        assert_eq!(
            parse_sample("regex", "(?{\"} close brace inside double quotes\"})"),
            Some(true)
        );
        assert_eq!(
            parse_sample("regex", "(?{'} close brace inside single quotes'})"),
            Some(true)
        );
        assert_eq!(parse_sample("regex", "(?{{ unterminated })"), Some(false));
        assert_eq!(parse_sample("regex", "(?{\"unterminated})"), Some(false));
    }

    #[cfg(has_generated_regex_parser)]
    #[test]
    fn regex_parseability_adapter_accepts_unicode_literals_and_deep_nested_groups() {
        let deep_nested = format!("{}a{}", "(".repeat(50), ")".repeat(50));

        assert_eq!(parse_sample("regex", "🎉"), Some(true));
        assert_eq!(parse_sample("regex", "café"), Some(true));
        assert_eq!(parse_sample("regex", &deep_nested), Some(true));
    }

    #[cfg(has_generated_regex_parser)]
    #[test]
    fn regex_ast_json_adapter_handles_unicode_literals_and_deep_nested_groups() {
        let deep_nested = format!("{}a{}", "(".repeat(50), ")".repeat(50));

        let unicode_ast = parse_sample_ast_json("regex", "🎉").expect("regex ast adapter");
        assert!(unicode_ast.is_ok(), "regex AST JSON adapter should serialize emoji literal");

        let mixed_ast = parse_sample_ast_json("regex", "café").expect("regex ast adapter");
        assert!(
            mixed_ast.is_ok(),
            "regex AST JSON adapter should serialize mixed ASCII/Unicode literal runs"
        );

        let deep_ast = parse_sample_ast_json("regex", &deep_nested).expect("regex ast adapter");
        assert!(
            deep_ast.is_ok(),
            "regex AST JSON adapter should serialize 50-level nested capturing groups"
        );
    }

    #[test]
    fn tracked_grammars_expose_parseable_standalone_return_annotations() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("repo root")
            .to_path_buf();
        let grammar_dir = repo_root.join("grammars");
        let excluded = ["return_annotation.ebnf", "semantic_annotation.ebnf"];
        let mut missing = Vec::new();
        let mut invalid = Vec::new();

        for entry in fs::read_dir(&grammar_dir).expect("read grammars directory") {
            let entry = entry.expect("grammar entry");
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("ebnf") {
                continue;
            }

            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .expect("grammar file name");
            if excluded.contains(&file_name) {
                continue;
            }

            let contents = fs::read_to_string(&path).expect("read grammar file");
            let mut found_standalone_annotation = false;

            for (line_number, line) in contents.lines().enumerate() {
                let trimmed = line.trim_start();
                if let Some(payload) = trimmed.strip_prefix("->") {
                    found_standalone_annotation = true;
                    let payload = payload.trim();
                    if payload.is_empty() {
                        continue;
                    }
                    if parse_sample("return_annotation", payload) != Some(true) {
                        invalid.push(format!("{}:{} -> {}", file_name, line_number + 1, payload));
                    }
                }
            }

            if !found_standalone_annotation {
                missing.push(file_name.to_string());
            }
        }

        assert!(
            missing.is_empty(),
            "grammars missing standalone return annotations: {:?}",
            missing
        );
        assert!(
            invalid.is_empty(),
            "standalone return annotations that do not parse with return_annotation grammar: {:?}",
            invalid
        );
    }

    #[cfg(has_generated_rtl_const_expr_parser)]
    #[test]
    fn rtl_const_expr_parseability_adapter_accepts_valid_expression_and_rejects_garbage() {
        assert_eq!(
            parse_sample("rtl_const_expr", "SEL ? cfg_pkg::A + 1 : cfg.width << 2"),
            Some(true)
        );
        assert_eq!(parse_sample("rtl_const_expr", "A ? : B"), Some(false));
        let ast_json = parse_sample_ast_json("rtl_const_expr", "WIDTH + 4")
            .expect("rtl_const_expr adapter should exist");
        assert!(ast_json.is_ok());
    }

    #[cfg(has_generated_rtl_frontend_parser)]
    #[test]
    fn rtl_frontend_parseability_adapter_accepts_valid_module_and_rejects_garbage() {
        assert_eq!(
            parse_sample(
                "rtl_frontend",
                "module m(input logic clk); assign clk = clk; endmodule"
            ),
            Some(true)
        );
        assert_eq!(parse_sample("rtl_frontend", "module m("), Some(false));
        let ast_json = parse_sample_ast_json(
            "rtl_frontend",
            "module m(input logic clk); endmodule",
        )
        .expect("rtl_frontend adapter should exist");
        assert!(ast_json.is_ok());
    }

    #[cfg(has_generated_rtl_frontend_parser)]
    #[test]
    fn rtl_frontend_generated_contract_metadata_is_stable() {
        let contract = rtl_frontend_generated_contract();
        assert_eq!(contract.contract_version, "0.1.0");
        assert_eq!(contract.grammar_name, "rtl_frontend");
        assert!(
            contract
                .purpose
                .contains("Curated generated rtl_frontend syntax contract"),
            "unexpected contract purpose: {}",
            contract.purpose
        );
        assert!(
            contract
                .provenance
                .contains("local handwritten rtl_frontend::parse_design replay"),
            "unexpected contract provenance: {}",
            contract.provenance
        );
        assert!(
            !contract.samples.is_empty(),
            "rtl_frontend generated contract must contain samples"
        );
    }

    #[cfg(has_generated_rtl_frontend_parser)]
    #[test]
    fn rtl_frontend_generated_contract_samples_hold() {
        let contract = rtl_frontend_generated_contract();

        for sample in contract.samples {
            assert_eq!(
                parse_sample("rtl_frontend", &sample.sample),
                Some(sample.expected_parse_ok),
                "generated rtl_frontend parseability drifted for curated sample '{}'",
                sample.label
            );

            if sample.require_ast_json {
                let ast_json = parse_sample_ast_json("rtl_frontend", &sample.sample)
                    .expect("rtl_frontend ast adapter should exist");
                let ast_json = ast_json.expect("rtl_frontend AST JSON adapter should serialize curated sample");
                for rule_name in &sample.required_rule_names {
                    assert!(
                        rtl_frontend_ast_contains_rule(&ast_json, rule_name),
                        "generated rtl_frontend AST JSON for sample '{}' is missing required rule '{}'",
                        sample.label,
                        rule_name
                    );
                }
                for rule_name in &sample.forbidden_rule_names {
                    assert!(
                        !rtl_frontend_ast_contains_rule(&ast_json, rule_name),
                        "generated rtl_frontend AST JSON for sample '{}' unexpectedly contains forbidden rule '{}'",
                        sample.label,
                        rule_name
                    );
                }
                for (rule_name, expected_texts) in &sample.expected_rule_texts {
                    assert_eq!(
                        rtl_frontend_ast_rule_texts(&sample.sample, &ast_json, rule_name),
                        *expected_texts,
                        "generated rtl_frontend AST JSON for sample '{}' preserved unexpected texts for rule '{}'",
                        sample.label,
                        rule_name
                    );
                }
            }
        }
    }
}
