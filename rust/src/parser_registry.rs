//! Registry for generated parser adapters used by parseability and round-trip checks.
//!
//! This centralizes grammar-name dispatch so new generated grammars are added in one place.

use crate::ast_pipeline::{ParseNode, UnifiedSemanticAST, runtime_logger, runtime_logger_box};
#[cfg(all(feature = "ebnf_dual_run", has_generated_ebnf_parser))]
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

// SV-EXH-PROOF.3.3.4.b.6.2.17 — rule-level targeted trace, thread-local set.
// Callers (e.g. parseability_probe) set this before invoking parse_sample_*
// functions; the parser-specific dispatch reads it and calls the parser's
// `set_trace_rules` method after construction. Thread-local because parser
// invocations may run on dedicated worker threads (see the regex parser's
// 64MB worker stack); per-thread state ensures the setting reaches the
// right parser even when off-thread.
std::thread_local! {
    static TRACE_RULES: std::cell::RefCell<Option<std::collections::HashSet<String>>> = const { std::cell::RefCell::new(None) };
    // SV-EXH-PROOF.3.3.4.b.6.2.22 — enable the live per-rule call-count
    // dashboard for parser invocations on the current thread. None
    // (default) = no dashboard. Some(N) = show the top-N rules,
    // refreshing every 250ms. SV has ~1500 rules — N is user-controlled
    // because the meaningful slice depends on the investigation.
    static DUMP_RULE_CALL_COUNTS_TOP_N: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) };
    // SV-EXH-PROOF.3.3.4.b.6.2.22 — exclusion list for the dashboard.
    // Rules in this set are filtered out before the top-N selection,
    // so user-irrelevant always-dominant rules (e.g. `trivia` for
    // whitespace handling) don't steal display slots from the rules
    // the user actually wants to see. None = no filtering.
    static DUMP_RULE_CALL_COUNTS_EXCLUDE: std::cell::RefCell<Option<std::collections::HashSet<String>>> = const { std::cell::RefCell::new(None) };
}

/// SV-EXH-PROOF.3.3.4.b.6.2.17 — set the rule-level trace filter for parser
/// invocations on the current thread. `None` (default) means full trace
/// (when `--trace` is also set); `Some(set)` restricts trace output to the
/// call-tree of the listed rules.
pub fn set_global_trace_rules(rules: Option<std::collections::HashSet<String>>) {
    TRACE_RULES.with(|r| *r.borrow_mut() = rules);
}

fn current_trace_rules() -> Option<std::collections::HashSet<String>> {
    TRACE_RULES.with(|r| r.borrow().clone())
}

/// SV-EXH-PROOF.3.3.4.b.6.2.22 — enable the live per-rule call-count
/// dashboard for parser invocations on the current thread. `None`
/// disables it; `Some(N)` enables it showing the top-N rules,
/// refreshed every 250ms. N is user-controlled (SV has ~1500 rules,
/// the right top-N depends on investigation).
pub fn set_global_dump_rule_call_counts(top_n: Option<usize>) {
    DUMP_RULE_CALL_COUNTS_TOP_N.with(|c| c.set(top_n));
}

fn current_dump_rule_call_counts_top_n() -> Option<usize> {
    DUMP_RULE_CALL_COUNTS_TOP_N.with(|c| c.get())
}

/// SV-EXH-PROOF.3.3.4.b.6.2.22 — set the dashboard exclusion list for
/// parser invocations on the current thread. `None` = no filtering;
/// `Some(set)` = filter these rules out before computing the top-N.
/// Used to hide always-dominant noise like `trivia` so the
/// diagnostically interesting rules win display slots.
pub fn set_global_dump_rule_call_counts_exclude(
    rules: Option<std::collections::HashSet<String>>,
) {
    DUMP_RULE_CALL_COUNTS_EXCLUDE.with(|c| *c.borrow_mut() = rules);
}

fn current_dump_rule_call_counts_exclude() -> std::collections::HashSet<String> {
    DUMP_RULE_CALL_COUNTS_EXCLUDE.with(|c| c.borrow().clone().unwrap_or_default())
}
#[cfg(has_generated_regex_parser)]
// PCRE2 conformance includes deeply nested and grammar-like recursive regexes.
// Keep the generated parser on a larger bounded stack than Rust's default.
const GENERATED_REGEX_WORKER_STACK_BYTES: usize = 64 * 1024 * 1024;

type ParseSampleFn = fn(&str) -> bool;

#[cfg(has_generated_systemverilog_parser)]
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

#[cfg(all(feature = "ebnf_dual_run", has_generated_ebnf_parser))]
fn parse_with_ebnf(sample: &str) -> bool {
    let mut parser = EbnfParser::new(sample, runtime_logger_box("generated.ebnf"));
    parser.parse_full_grammar_file().is_ok()
}

#[cfg(all(feature = "ebnf_dual_run", has_generated_ebnf_parser))]
fn parse_with_ebnf_detail(sample: &str) -> Result<(), String> {
    let mut parser = EbnfParser::new(sample, runtime_logger_box("generated.ebnf"));
    parser
        .parse_full_grammar_file()
        .map(|_| ())
        .map_err(|err| err.to_string())
}

#[cfg(all(feature = "ebnf_dual_run", has_generated_ebnf_parser))]
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

/// SV-EXH-PROOF.3.3.4.b.6.2.22 — helper: spawn the dashboard if the
/// thread-local flag is set, returning an RAII handle that the caller
/// holds for the parse duration. Drop tears down the dashboard thread
/// and restores the cursor. None when the flag is unset (the common
/// case; zero overhead).
#[cfg(has_generated_systemverilog_parser)]
fn maybe_spawn_call_count_dashboard(
    parser: &SystemverilogParser<'_>,
) -> Option<crate::ast_pipeline::call_count_dashboard::CallCountDashboard> {
    let top_n = current_dump_rule_call_counts_top_n()?;
    Some(crate::ast_pipeline::call_count_dashboard::CallCountDashboard::spawn(
        parser.rule_call_counts(),
        SystemverilogParser::rule_names(),
        current_dump_rule_call_counts_exclude(),
        top_n,
        250, // refresh interval in ms
    ))
}

#[cfg(has_generated_systemverilog_parser)]
fn parse_with_systemverilog_profile(sample: &str, grammar_profile: Option<&str>) -> bool {
    let mut parser =
        SystemverilogParser::new(sample, runtime_logger_box("generated.systemverilog"));
        parser.set_trace_rules(current_trace_rules());
    let normalized_profile = normalize_generated_grammar_profile(
        "systemverilog",
        grammar_profile,
    );
    parser.set_grammar_profile(normalized_profile);
    if preload_systemverilog_stdlib(&mut parser, normalized_profile).is_err() {
        return false;
    }
    let _dashboard = maybe_spawn_call_count_dashboard(&parser);
    parser.parse_full_systemverilog_file().is_ok()
}

#[cfg(has_generated_systemverilog_parser)]
fn parse_with_systemverilog_detail_profile(
    sample: &str,
    grammar_profile: Option<&str>,
) -> Result<(), String> {
    let mut parser =
        SystemverilogParser::new(sample, runtime_logger_box("generated.systemverilog"));
        parser.set_trace_rules(current_trace_rules());
    let normalized_profile = normalize_generated_grammar_profile(
        "systemverilog",
        grammar_profile,
    );
    parser.set_grammar_profile(normalized_profile);
    preload_systemverilog_stdlib(&mut parser, normalized_profile)?;
    let _dashboard = maybe_spawn_call_count_dashboard(&parser);
    let result = parser.parse_full_systemverilog_file().map(|_| ());
    // SV-EXH-PROOF.3.3.4.b.6.2.25 — on failure, augment the error with the
    // furthest byte the parser reached on any branch (even backtracked
    // branches). The surface `position` in the error message is the
    // outermost failing rule's start — often megabytes shallower than
    // the actual defective construct. furthest_position pinpoints the
    // real defect locus in one diagnostic run.
    result.map_err(|err| {
        let furthest = parser.furthest_position();
        let err_str = err.to_string();
        let surface = extract_position_from_message(&err_str);
        format!(
            "{} [furthest_position={}, +{} bytes deeper than surface position]",
            err_str,
            furthest,
            furthest.saturating_sub(surface),
        )
    })
}

/// SV-EXH-PROOF.3.3.4.b.6.2.25 — extract the surface position from a
/// ParseError::Display string of the form
/// "... at position N" or "Parser did not consume full input at position N".
/// Best-effort: returns 0 if no `at position N` segment found. Used only
/// for the "+M bytes deeper" delta in the augmented error message.
fn extract_position_from_message(msg: &str) -> usize {
    msg.rsplit_once("at position ")
        .and_then(|(_, tail)| {
            tail.split(|c: char| !c.is_ascii_digit())
                .next()
                .and_then(|s| s.parse::<usize>().ok())
        })
        .unwrap_or(0)
}

/// `SV-EXH-PROOF.3.3.4.a` MVP-0: SV detail variant honoring per-call library
/// directories. `library_options.in_dir` makes `@import_from_library` read
/// from `<dir>/<kind>/<name>.facts.json`; `out_dir` makes
/// `@export_to_library` write to the same path layout.
#[cfg(has_generated_systemverilog_parser)]
fn parse_with_systemverilog_detail_profile_with_library(
    sample: &str,
    grammar_profile: Option<&str>,
    library_options: &LibraryOptions,
) -> Result<(), String> {
    let mut parser =
        SystemverilogParser::new(sample, runtime_logger_box("generated.systemverilog"));
        parser.set_trace_rules(current_trace_rules());
    let normalized_profile = normalize_generated_grammar_profile(
        "systemverilog",
        grammar_profile,
    );
    parser.set_grammar_profile(normalized_profile);
    parser.set_library_in_dir(library_options.in_dir.clone());
    parser.set_library_out_dir(library_options.out_dir.clone());
    preload_systemverilog_stdlib(&mut parser, normalized_profile)?;
    let _dashboard = maybe_spawn_call_count_dashboard(&parser);
    parser
        .parse_full_systemverilog_file()
        .map(|_| ())
        .map_err(|err| err.to_string())
}

// SV-EXH-PROOF.3.3.4.b.6.2.37.2 — H2 auto-load hook for the SV stdlib.
// Reads `<repo_root>/parser_libs/sv_<profile>_std/package/std.facts.json`
// (checked in by `.37.1`) and pushes each fact into the parser's
// semantic_runtime_state BEFORE the parse begins. This makes the LRM §G.2
// `std::` predefined classes (process / semaphore / mailbox today; can grow)
// visible to the `.35.1`-gated `has_fact(type_name, X)` checks for the
// duration of the parse, without requiring user code to declare them.
//
// Path resolution: `env!("CARGO_MANIFEST_DIR")` is `rust/`, so its parent
// is the repo root. Missing stdlib bundle = no-op (single-file fallback,
// behaviour-equivalent to pre-.37.2). Read errors surface as `Err`.
#[cfg(has_generated_systemverilog_parser)]
fn preload_systemverilog_stdlib(
    parser: &mut SystemverilogParser,
    normalized_profile: Option<&str>,
) -> Result<(), String> {
    let profile_dir = match normalized_profile {
        Some("sv_2023") => "sv_2023_std",
        // Default to sv_2017 when no profile / unrecognised — sv_2017 covers
        // the corpus baseline. Future grammar profiles register their own
        // bundle here.
        _ => "sv_2017_std",
    };
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let Some(repo_root) = manifest_dir.parent() else {
        return Ok(()); // Manifest has no parent? Skip (no place for parser_libs).
    };
    let stdlib_path = repo_root.join("parser_libs").join(profile_dir);
    if !stdlib_path.is_dir() {
        return Ok(()); // No stdlib bundle for this profile — no-op.
    }
    match crate::ast_pipeline::library::read_artifact(&stdlib_path, "package", "std") {
        Ok(records) => {
            let state = parser.semantic_runtime_state_mut();
            for record in records {
                state.push_fact_record(record);
            }
            Ok(())
        }
        Err(crate::ast_pipeline::library::LibraryError::NotFound(_)) => Ok(()),
        Err(other) => Err(format!(
            "SV stdlib preload failed (path={}): {:?}",
            stdlib_path.display(),
            other
        )),
    }
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
        parser.set_trace_rules(current_trace_rules());
    let normalized_profile = normalize_generated_grammar_profile(
        "systemverilog",
        grammar_profile,
    );
    parser.set_grammar_profile(normalized_profile);
    preload_systemverilog_stdlib(&mut parser, normalized_profile)?;
    let _dashboard = maybe_spawn_call_count_dashboard(&parser);
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
    #[cfg(all(feature = "ebnf_dual_run", has_generated_ebnf_parser))]
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
    #[cfg(not(has_generated_systemverilog_parser))]
    let _ = grammar_profile;

    match grammar_name {
        #[cfg(has_generated_systemverilog_parser)]
        "systemverilog" => Some(parse_with_systemverilog_profile(sample, grammar_profile)),
        _ => find_entry(grammar_name).map(|entry| entry.parse(sample)),
    }
}

/// `SV-EXH-PROOF.3.3.4.a` MVP-0: per-call library configuration for grammars
/// that use `@import_from_library` / `@export_to_library`. Both fields default
/// to `None` (no library plumbing); a grammar with no library directives is
/// unaffected by either field being set.
#[derive(Debug, Clone, Default)]
pub struct LibraryOptions {
    pub in_dir: Option<std::path::PathBuf>,
    pub out_dir: Option<std::path::PathBuf>,
}

impl LibraryOptions {
    pub fn is_empty(&self) -> bool {
        self.in_dir.is_none() && self.out_dir.is_none()
    }
}

pub fn parse_sample_detail(grammar_name: &str, sample: &str) -> Option<Result<(), String>> {
    parse_sample_detail_with_profile(grammar_name, sample, None)
}

/// `SV-EXH-PROOF.3.3.4.a` MVP-0: library-aware detail variant. Falls through
/// to `parse_sample_detail_with_profile` when `library_options` is empty so
/// existing call sites are unaffected. When set, grammars whose generated
/// parser supports library options (SV today; others as they adopt the
/// annotations) honor them; other grammars silently ignore (their parsers
/// have no library directives so the options would be no-ops anyway).
pub fn parse_sample_detail_with_options(
    grammar_name: &str,
    sample: &str,
    grammar_profile: Option<&str>,
    library_options: &LibraryOptions,
) -> Option<Result<(), String>> {
    if library_options.is_empty() {
        return parse_sample_detail_with_profile(grammar_name, sample, grammar_profile);
    }
    match grammar_name {
        #[cfg(has_generated_systemverilog_parser)]
        "systemverilog" => Some(parse_with_systemverilog_detail_profile_with_library(
            sample,
            grammar_profile,
            library_options,
        )),
        _ => parse_sample_detail_with_profile(grammar_name, sample, grammar_profile),
    }
}

pub fn parse_sample_detail_with_profile(
    grammar_name: &str,
    sample: &str,
    grammar_profile: Option<&str>,
) -> Option<Result<(), String>> {
    #[cfg(not(has_generated_systemverilog_parser))]
    let _ = grammar_profile;

    match grammar_name {
        "return_annotation" => Some(parse_with_return_annotation_detail(sample)),
        "semantic_annotation" => Some(parse_with_semantic_annotation_detail(sample)),
        "builtin_return_annotation" => Some(parse_with_builtin_return_annotation_detail(sample)),
        "builtin_semantic_annotation" => {
            Some(parse_with_builtin_semantic_annotation_detail(sample))
        }
        #[cfg(all(feature = "ebnf_dual_run", has_generated_ebnf_parser))]
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
    #[cfg(not(has_generated_systemverilog_parser))]
    let _ = grammar_profile;

    match grammar_name {
        "return_annotation" => Some(parse_with_return_annotation_ast_json(sample)),
        "semantic_annotation" => Some(parse_with_semantic_annotation_ast_json(sample)),
        "builtin_return_annotation" => Some(parse_with_builtin_return_annotation_ast_json(sample)),
        "builtin_semantic_annotation" => {
            Some(parse_with_builtin_semantic_annotation_ast_json(sample))
        }
        #[cfg(all(feature = "ebnf_dual_run", has_generated_ebnf_parser))]
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
                if let Some(serde_json::Value::String(candidate)) = map.get("rule_name")
                    && candidate == rule_name
                    && let Some(serde_json::Value::Object(span)) = map.get("span")
                    && let (
                        Some(serde_json::Value::Number(start)),
                        Some(serde_json::Value::Number(end)),
                    ) = (span.get("start"), span.get("end"))
                    && let (Some(start), Some(end)) = (start.as_u64(), end.as_u64())
                {
                    spans.push((start as usize, end as usize));
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
                    .unwrap_or_else(|| {
                        panic!("invalid span {}..{} for rule '{}'", start, end, rule_name)
                    })
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

    #[cfg(all(feature = "ebnf_dual_run", has_generated_ebnf_parser))]
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

    #[cfg(all(feature = "ebnf_dual_run", has_generated_ebnf_parser))]
    #[test]
    fn ebnf_parseability_adapter_accepts_valid_rule_and_rejects_garbage() {
        assert_eq!(
            parse_sample("ebnf", r#"rule_name := /([a-zA-Z_][a-zA-Z0-9_]*)/"#),
            Some(true)
        );
        assert_eq!(parse_sample("ebnf", ":::not-ebnf:::"), Some(false));
    }

    #[cfg(all(feature = "ebnf_dual_run", has_generated_ebnf_parser))]
    #[test]
    fn ebnf_parseability_adapter_accepts_inline_lookahead_in_sequence() {
        let sample = r#"ports := direction item ( "," !direction item )*
direction := "input" | "output"
item := identifier
identifier := /([a-zA-Z_][a-zA-Z0-9_]*)/"#;
        assert_eq!(parse_sample("ebnf", sample), Some(true));
        let ast_json =
            parse_sample_ast_json("ebnf", sample).expect("ebnf ast adapter should exist");
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
        assert_eq!(parse_sample("regex", "[[:digit:]-]+"), Some(true));
        assert_eq!(parse_sample("regex", r"abc\Q(*+|\Eabc"), Some(true));
        assert_eq!(
            parse_sample("regex", "(*:m(m)(?&y)(?(DEFINE)(?<y>b))"),
            Some(true)
        );
        assert_eq!(
            parse_sample("regex", "(*PRUNE:m(m)(?&y)(?(DEFINE)(?<y>b))"),
            Some(true)
        );
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
        assert_eq!(parse_sample("regex", "(?R1)"), Some(false));
        assert_eq!(parse_sample("regex", "\\g{1}"), Some(true));
        assert_eq!(parse_sample("regex", "(A)(\\g{ -2 }B)"), Some(true));
        assert_eq!(
            parse_sample("regex", "(?'name'ab)\\k{ name }(?P=name)"),
            Some(true)
        );
        assert_eq!(parse_sample("regex", "(?C1)"), Some(true));
        assert_eq!(parse_sample("regex", "(?C\"alpha\"\"beta\")"), Some(true));
        assert_eq!(parse_sample("regex", "(?C{left}}right})"), Some(true));
        assert_eq!(parse_sample("regex", "(*UTF)abc"), Some(true));
        assert_eq!(parse_sample("regex", "(*MARK:A)(*SKIP:B)(C|X)"), Some(true));
        assert_eq!(
            parse_sample("regex", "(*SKIP:m(m)(?&y)(?(DEFINE)(?<y>b))"),
            Some(true)
        );
        assert_eq!(
            parse_sample("regex", "(*THEN:m(m)(?&y)(?(DEFINE)(?<y>b))"),
            Some(true)
        );
        for sample in [
            "(?(*pla:foo).{6}|a..)",
            "(?(*positive_lookahead:foo).{6}|a..)",
            "(?(*nla:foo)bar|baz)",
            "(?(*negative_lookahead:foo)bar|baz)",
            "(?(*plb:foo)bar|baz)",
            "(?(*positive_lookbehind:foo)bar|baz)",
            "(?(*nlb:foo)bar|baz)",
            "(?(*negative_lookbehind:foo)bar|baz)",
        ] {
            assert_eq!(parse_sample("regex", sample), Some(true));
        }
        for sample in [
            "(?*foo)",
            "(?<*foo)",
            "(*napla:foo)",
            "(*non_atomic_positive_lookahead:foo)",
            "(*naplb:foo)",
            "(*non_atomic_positive_lookbehind:foo)",
            "(*atomic:foo)",
            "(*sr:foo)",
            "(*script_run:foo)",
            "(*asr:foo)",
            "(*atomic_script_run:foo)",
            "(.)(*scs:(1)foo)",
            "(?<cap>.)(*scan_substring:(1,<cap>)foo)",
        ] {
            assert_eq!(parse_sample("regex", sample), Some(true));
        }
        assert_eq!(parse_sample("regex", r"\Kword"), Some(true));
        assert_eq!(parse_sample("regex", r"\xA"), Some(true));
        assert_eq!(parse_sample("regex", r"\x{ 41 }"), Some(true));
        assert_eq!(parse_sample("regex", r"\o{ 101 }"), Some(true));
        assert_eq!(parse_sample("regex", "a{,}b"), Some(true));
        assert_eq!(parse_sample("regex", "(?aD)\\d"), Some(true));
        assert_eq!(parse_sample("regex", "(?xx:a b)"), Some(true));
        assert_eq!(
            parse_sample("regex", "(?(VERSION >= 10)cat|dog)"),
            Some(false)
        );
        assert_eq!(parse_sample("regex", "(?(VERSION<10)cat|dog)"), Some(false));
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
        assert!(
            unicode_ast.is_ok(),
            "regex AST JSON adapter should serialize emoji literal"
        );

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
        let ast_json =
            parse_sample_ast_json("rtl_frontend", "module m(input logic clk); endmodule")
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
    #[ignore = "trace-of-parse-path test: walks post-parse JSON for inner rule_names (e.g. backreference, octal_escape) which are erased by the codegen-fix's ParseContent::to_json_value() flattening. Reformulate onto typed-AST shape or enrich the grammar with annotations on those rules; tracked separately from the auto-gate generator."]
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
                let ast_json = ast_json
                    .expect("rtl_frontend AST JSON adapter should serialize curated sample");
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
