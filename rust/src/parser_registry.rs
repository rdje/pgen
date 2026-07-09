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
// PARSE-HARNESS.2 — the blessed scratch-register slot (arbitrary-grammar probe).
#[cfg(has_generated_scratch_parser)]
use crate::generated_parsers::scratch::ScratchParser;
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

/// GRAMMAR-WELLFORMED.G.4: a grammar's witness-verification hook — parse `sample` through its REAL
/// generated parser and return `(parsed_ok, rules_exercised)` (the rule names PRESENT in the
/// successful AST). Registered per-grammar in the table below; the pipeline dispatches generically
/// via `parse_and_cover(grammar_name, …)` and never names a grammar.
///
/// GRAMMAR-WELLFORMED.H.12.8.4.3: the third argument is the optional ENTRY rule — the start symbol the
/// sample should be parsed from. `None` ⇒ the grammar's canonical entry (byte-identical to before this
/// param existed). `Some(entry)` ⇒ verify from `parser.parse_full_from(entry)`, so an entry-relative
/// rule (rooted under an alternate LRM start symbol such as `library_text`) is verified from the SAME
/// entry the cert's `--entry-rule` / `--cert-union-config` generated it under. Parser-agnostic.
type ParseAndCoverFn =
    fn(&str, Option<&str>, Option<&str>) -> (bool, std::collections::HashSet<String>);
/// GRAMMAR-WELLFORMED.G.4.7: the per-grammar "parse and return WHY it failed" hook — a rich error
/// string (the generated SV parser augments it with `furthest_position`, the deepest byte any branch
/// reached). Lets the certificate-coverage gate LABEL its sample-parse failures instead of silently
/// counting them. Same data-driven boundary as `parse_and_cover`: the per-grammar knowledge lives in
/// the registry table, never in the pipeline.
type ParseDetailFn = fn(&str, Option<&str>) -> Result<(), String>;

#[cfg(any(has_generated_systemverilog_parser, has_generated_regex_parser))]
/// The **active** dialect profile a grammar parses under, given a requested profile — the single source
/// of truth for per-grammar profile resolution (an unspecified/empty profile resolves to the grammar's
/// DECLARED `@default_profile`, sourced from the generated parser's `DEFAULT_GRAMMAR_PROFILE` constant
/// — e.g. regex → strict `pcre2`; SV's `2017`/`2023` aliases map to `sv_2017`/`sv_2023`). This is what
/// a profile-aware second parser (the PARSE-HARNESS interpreter) must gate `@profiles` rules against so
/// it matches `parse_sample` byte-for-byte (PARSE-HARNESS.5.1). Returns an owned `String` so a
/// non-registry caller need not borrow the request.
pub fn active_grammar_profile(grammar_name: &str, grammar_profile: Option<&str>) -> Option<String> {
    normalize_generated_grammar_profile(grammar_name, grammar_profile)
        .map(|s| s.to_string())
        .or_else(|| default_generated_grammar_profile(grammar_name).map(|s| s.to_string()))
}

/// `DEFAULT-PROFILE.2`: the grammar-DECLARED default dialect profile — the profile an
/// UNSPECIFIED/empty requested profile resolves to. Sourced from the generated parser's
/// `DEFAULT_GRAMMAR_PROFILE` constant, which codegen emits from the grammar's own
/// `@default_profile:` directive — so the registry holds NO profile knowledge of its own (the
/// retired `== "regex" → "pcre2"` literal was exactly that defect class). Same data-driven
/// boundary as `parse_and_cover`: the per-grammar datum lives in this table, sourced from the
/// grammar-derived artifact.
#[cfg(any(has_generated_systemverilog_parser, has_generated_regex_parser))]
fn default_generated_grammar_profile(grammar_name: &str) -> Option<&'static str> {
    match grammar_name {
        #[cfg(has_generated_regex_parser)]
        "regex" => Some(RegexParser::DEFAULT_GRAMMAR_PROFILE),
        _ => None,
    }
}

fn normalize_generated_grammar_profile<'a>(
    grammar_name: &str,
    grammar_profile: Option<&'a str>,
) -> Option<&'a str> {
    let profile = grammar_profile?.trim();
    if profile.is_empty() {
        return None;
    }
    Some(resolve_generated_grammar_profile_alias(
        grammar_name,
        profile,
    ))
}

/// `PROFILE-ALIAS.2`: the grammar-DECLARED request-spelling alias resolver —
/// dispatches to the generated parser's `resolve_grammar_profile_alias`
/// (emitted from the grammar's own `@profile_alias:` directives, e.g. SV's
/// `2017`/`ieee1800-2017` → `sv_2017`), so the registry holds NO spelling
/// knowledge of its own (the retired `"systemverilog"` alias match arm was
/// exactly that defect class). Same data-driven boundary as
/// `default_generated_grammar_profile`: the per-grammar datum lives in this
/// table, sourced from the grammar-derived artifact. Unmatched spellings and
/// alias-free grammars pass through unchanged.
#[cfg(has_generated_systemverilog_parser)]
fn resolve_generated_grammar_profile_alias<'a>(grammar_name: &str, profile: &'a str) -> &'a str {
    match grammar_name {
        "systemverilog" => SystemverilogParser::resolve_grammar_profile_alias(profile),
        _ => profile,
    }
}

/// `PROFILE-ALIAS.2`: without the SV artifact compiled in, no registered
/// grammar declares `@profile_alias` — every spelling passes through.
#[cfg(not(has_generated_systemverilog_parser))]
fn resolve_generated_grammar_profile_alias<'a>(_grammar_name: &str, profile: &'a str) -> &'a str {
    profile
}

#[derive(Clone, Copy, Debug)]
pub struct GeneratedParserRegistryEntry {
    pub grammar_name: &'static str,
    parse_sample: ParseSampleFn,
    /// GRAMMAR-WELLFORMED.G.4: the witness-verification hook, when this grammar's parser supports it
    /// (`None` until wired — Phase H wires the rest). Kept here so the per-grammar knowledge lives in
    /// the registry table (data-driven), never in the pipeline.
    parse_and_cover: Option<ParseAndCoverFn>,
    /// GRAMMAR-WELLFORMED.G.4.7: the "why did it fail to parse" hook (rich error string). `None` until
    /// wired per grammar. Used to LABEL certificate-coverage sample-parse failures.
    parse_detail: Option<ParseDetailFn>,
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

/// GRAMMAR-WELLFORMED.H.3 — parse `sample` through the REAL json parser and return
/// `(parsed_ok, rules_exercised)` for `certificate_coverage` (the witness side). Mirrors
/// `parse_and_cover_systemverilog`: enable the transactional `coverage_stack`, parse, and return the
/// PARSER's own record of the committed rules on a SUCCESSFUL parse. json has no grammar profile and no
/// deep recursion, so neither a profile (`_grammar_profile` is unused) nor a dedicated worker stack
/// (unlike regex's RGX-0085 stack) is needed.
#[cfg(has_generated_json_parser)]
pub fn parse_and_cover_json(
    sample: &str,
    _grammar_profile: Option<&str>,
    entry: Option<&str>,
) -> (bool, std::collections::HashSet<String>) {
    let mut parser = JsonParser::new(sample, runtime_logger_box("generated.json"));
    parser.enable_coverage();
    // GRAMMAR-WELLFORMED.H.12.8.4.3: verify from the requested entry; `None` ⇒ canonical (unchanged).
    let outcome = match entry {
        Some(e) => parser.parse_full_from(e),
        None => parser.parse_full_json(),
    };
    match outcome {
        Ok(_) => (true, parser.exercised_rule_names()),
        Err(_) => (false, std::collections::HashSet::new()),
    }
}

#[cfg(has_generated_regex_parser)]
fn parse_with_regex(sample: &str) -> bool {
    parse_with_regex_detail(sample, None).is_ok()
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
fn parse_with_regex_detail(sample: &str, grammar_profile: Option<&str>) -> Result<(), String> {
    // REGEX-PCRE2-FIDELITY.2 / DEFAULT-PROFILE.2: default = strict `pcre2` (PCRE2-faithful);
    // `relaxed` opt-out. The default now comes from the ARTIFACT: `set_grammar_profile(None)`
    // restores the grammar-declared `@default_profile` (owned into the 'static worker closure).
    let profile = normalize_generated_grammar_profile("regex", grammar_profile).map(|p| p.to_string());
    run_generated_regex_on_dedicated_stack(sample, move |owned_sample| {
        let mut parser = RegexParser::new(&owned_sample, runtime_logger_box("generated.regex"));
        parser.set_grammar_profile(profile.as_deref());
        parser.parse_full_regex().map_err(|err| err.to_string())?;
        validate_regex_compile_contract(&owned_sample).map_err(|err| err.message)
    })
}

#[cfg(has_generated_regex_parser)]
fn parse_with_regex_ast_json(sample: &str, grammar_profile: Option<&str>) -> Result<JsonValue, String> {
    // REGEX-PCRE2-FIDELITY.2 / DEFAULT-PROFILE.2: default = strict `pcre2`; the default comes from
    // the artifact (`set_grammar_profile(None)` restores the declared `@default_profile`).
    let profile = normalize_generated_grammar_profile("regex", grammar_profile).map(|p| p.to_string());
    run_generated_regex_on_dedicated_stack(sample, move |owned_sample| {
        let mut parser = RegexParser::new(&owned_sample, runtime_logger_box("generated.regex"));
        parser.set_grammar_profile(profile.as_deref());
        let parsed = parser.parse_full_regex().map_err(|err| err.to_string())?;
        validate_regex_compile_contract(&owned_sample).map_err(|err| err.message)?;
        parse_node_to_json(&parsed)
    })
}

/// GRAMMAR-WELLFORMED.H.1 — parse `sample` through the REAL regex parser and return
/// `(parsed_ok, rules_exercised)` for `certificate_coverage` (the witness side). Mirrors
/// `parse_and_cover_systemverilog`: enable the transactional `coverage_stack`, parse, and return the
/// PARSER's own record of the committed rules on a SUCCESSFUL parse. Runs on the dedicated regex worker
/// stack (regex can deeply recurse — RGX-0085). PCRE2 compile validation is intentionally NOT applied
/// here — cert-coverage asks "did the GRAMMAR parse + which rules were exercised", not "is the pattern
/// PCRE2-valid". REGEX-PCRE2-FIDELITY.2: honors the grammar profile (default = strict `pcre2`; `relaxed`
/// opt-out) so the witness side parses under the same profile the generator generated for.
#[cfg(has_generated_regex_parser)]
pub fn parse_and_cover_regex(
    sample: &str,
    grammar_profile: Option<&str>,
    entry: Option<&str>,
) -> (bool, std::collections::HashSet<String>) {
    let profile = normalize_generated_grammar_profile("regex", grammar_profile).map(|p| p.to_string());
    // GRAMMAR-WELLFORMED.H.12.8.4.3: own the entry into the 'static worker closure; `None` ⇒ canonical.
    let entry_owned = entry.map(|e| e.to_string());
    run_generated_regex_on_dedicated_stack(sample, move |owned_sample| {
        let mut parser = RegexParser::new(&owned_sample, runtime_logger_box("generated.regex"));
        parser.set_grammar_profile(profile.as_deref());
        parser.enable_coverage();
        let outcome = match entry_owned.as_deref() {
            Some(e) => parser.parse_full_from(e),
            None => parser.parse_full_regex(),
        };
        Ok(match outcome {
            Ok(_) => (true, parser.exercised_rule_names()),
            Err(_) => (false, std::collections::HashSet::new()),
        })
    })
    .unwrap_or((false, std::collections::HashSet::new()))
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

/// GRAMMAR-WELLFORMED.H.4 — parse `sample` through the REAL rtl_const_expr parser and return
/// `(parsed_ok, rules_exercised)` for `certificate_coverage` (the witness side). Mirrors
/// `parse_and_cover_json`: enable the transactional `coverage_stack`, parse, and return the PARSER's own
/// record of the committed rules on a SUCCESSFUL parse. rtl_const_expr has no grammar profile and no deep
/// recursion, so neither a profile (`_grammar_profile` is unused) nor a dedicated worker stack is needed.
#[cfg(has_generated_rtl_const_expr_parser)]
pub fn parse_and_cover_rtl_const_expr(
    sample: &str,
    _grammar_profile: Option<&str>,
    entry: Option<&str>,
) -> (bool, std::collections::HashSet<String>) {
    let mut parser =
        RtlConstExprParser::new(sample, runtime_logger_box("generated.rtl_const_expr"));
    parser.enable_coverage();
    // GRAMMAR-WELLFORMED.H.12.8.4.3: verify from the requested entry; `None` ⇒ canonical (unchanged).
    let outcome = match entry {
        Some(e) => parser.parse_full_from(e),
        None => parser.parse_full_rtl_const_expr(),
    };
    match outcome {
        Ok(_) => (true, parser.exercised_rule_names()),
        Err(_) => (false, std::collections::HashSet::new()),
    }
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

/// GRAMMAR-WELLFORMED.H.6 — parse `sample` through the REAL rtl_frontend parser and return
/// `(parsed_ok, rules_exercised)` for `certificate_coverage` (the witness side). Mirrors
/// `parse_and_cover_systemverilog_preprocessor`: enable the transactional `coverage_stack`, parse, and
/// return the PARSER's own record of the committed rules on a SUCCESSFUL parse. rtl_frontend has no
/// grammar profile and its entry (`rtl_frontend_file := trivia design_item* trivia`) is a flat item list,
/// so neither a profile (`_grammar_profile` is unused) nor a dedicated worker stack is needed.
#[cfg(has_generated_rtl_frontend_parser)]
pub fn parse_and_cover_rtl_frontend(
    sample: &str,
    _grammar_profile: Option<&str>,
    entry: Option<&str>,
) -> (bool, std::collections::HashSet<String>) {
    let mut parser = RtlFrontendParser::new(sample, runtime_logger_box("generated.rtl_frontend"));
    parser.enable_coverage();
    // GRAMMAR-WELLFORMED.H.12.8.4.3: verify from the requested entry; `None` ⇒ canonical (unchanged).
    let outcome = match entry {
        Some(e) => parser.parse_full_from(e),
        None => parser.parse_full_rtl_frontend_file(),
    };
    match outcome {
        Ok(_) => (true, parser.exercised_rule_names()),
        Err(_) => (false, std::collections::HashSet::new()),
    }
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

/// GRAMMAR-WELLFORMED.G.4 (parser-AGNOSTIC dispatch): verify a witness/sample by parsing it through
/// the grammar's REAL generated parser, returning `(parsed_ok, rules_exercised)`, or `None` when no
/// generated parser is registered for `grammar_name` (the gate then skips — it cannot witness-verify
/// that grammar yet; Phase H registers more). The grammar-name → parser mapping lives HERE (the
/// registry's job, like the existing per-grammar parse closures), so the PIPELINE (e.g. `main.rs`)
/// stays parser-agnostic — it calls this with `grammar.grammar_name` and never names a grammar.
pub fn parse_and_cover(
    grammar_name: &str,
    sample: &str,
    grammar_profile: Option<&str>,
    entry: Option<&str>,
) -> Option<(bool, std::collections::HashSet<String>)> {
    let cover = find_entry(grammar_name)?.parse_and_cover?;
    Some(cover(sample, grammar_profile, entry))
}

/// Whether a generated parser with `parse_and_cover` support is registered for `grammar_name`.
pub fn supports_parse_and_cover(grammar_name: &str) -> bool {
    find_entry(grammar_name).is_some_and(|entry| entry.parse_and_cover.is_some())
}

/// GRAMMAR-WELLFORMED.G.4.7: parse `sample` and return WHY it failed (a rich error string with
/// `furthest_position`), or `Ok(())` if it parsed. `None` if no detail-capable parser is registered
/// for `grammar_name`. Lets the certificate-coverage gate LABEL its sample-parse failures. The
/// grammar-name → parser mapping lives HERE so the pipeline stays parser-agnostic.
pub fn parse_error(
    grammar_name: &str,
    sample: &str,
    grammar_profile: Option<&str>,
) -> Option<Result<(), String>> {
    let detail = find_entry(grammar_name)?.parse_detail?;
    Some(detail(sample, grammar_profile))
}

/// GRAMMAR-WELLFORMED.G.3.3 + G.4.6: parse `sample` through the REAL SystemVerilog parser and
/// return `(parsed_ok, rules_exercised)` — the `parse_and_cover` closure that
/// `grammar_wellformedness::verify_reachability_witness` needs to independently re-validate a
/// reachability witness.
///
/// Coverage = the parser's OWN transactional record of the rules in the ACCEPTED parse
/// (`enable_coverage` + `exercised_rule_names`), NOT a walk of the output AST. The earlier
/// AST-walk (`parse_node_covered_rules`) collapsed to ~one rule on annotated grammars because a
/// `-> {…}` return annotation folds a rule's whole subtree into `ParseContent::Json`, erasing the
/// children's rule identities. The transactional `coverage_stack` records rule ENTRIES and rolls
/// them back with `try_parse` on speculation failure, so the surviving set is sound (committed
/// successes only, no backtracked attempts — what a call counter would over-count) AND complete
/// (annotation folding cannot hide an entry). The PARSER testifies to what it parsed — independent
/// of the generator's own coverage claim ("verified, not trusted").
#[cfg(has_generated_systemverilog_parser)]
pub fn parse_and_cover_systemverilog(
    sample: &str,
    grammar_profile: Option<&str>,
    entry: Option<&str>,
) -> (bool, std::collections::HashSet<String>) {
    let mut parser =
        SystemverilogParser::new(sample, runtime_logger_box("generated.systemverilog"));
    let normalized_profile = normalize_generated_grammar_profile("systemverilog", grammar_profile);
    parser.set_grammar_profile(normalized_profile);
    if preload_systemverilog_stdlib(&mut parser, normalized_profile).is_err() {
        return (false, std::collections::HashSet::new());
    }
    parser.enable_coverage();
    // GRAMMAR-WELLFORMED.H.12.8.4.3: verify from the configured entry so an entry-relative rule
    // (rooted under the LRM `library_text` / parseable-fragment start symbols) is parsed back from
    // the SAME start symbol it was generated under. `None` / the canonical entry ⇒ byte-identical to
    // the hardwired `parse_full_systemverilog_file` path (the default arm of `parse_full_from`).
    let outcome = match entry {
        Some(e) => parser.parse_full_from(e),
        None => parser.parse_full_systemverilog_file(),
    };
    match outcome {
        Ok(_) => (true, parser.exercised_rule_names()),
        Err(_) => (false, std::collections::HashSet::new()),
    }
}

#[cfg(has_generated_systemverilog_parser)]
fn parse_with_systemverilog_detail_profile(
    sample: &str,
    grammar_profile: Option<&str>,
) -> Result<(), String> {
    // The registered `ParseDetailFn` (default entry). GRAMMAR-WELLFORMED.H.12.8.4.3.1 added the
    // `_entry` variant below; `None` here is byte-identical to the legacy single-entry path.
    parse_with_systemverilog_detail_profile_entry(sample, grammar_profile, None)
}

/// GRAMMAR-WELLFORMED.H.12.8.4.3.1: SV detail parse from an OPTIONAL alternate entry. `entry=None`
/// parses from `parse_full_systemverilog_file` (byte-identical to the legacy registered path);
/// `Some(rule)` parses from `parse_full_from(rule)` so `parseability_probe --entry-rule` can
/// reproduce / trace an entry-relative rule (rooted under an alternate LRM start symbol such as
/// `library_text`) in isolation. The `furthest_position` augmentation is preserved either way.
#[cfg(has_generated_systemverilog_parser)]
fn parse_with_systemverilog_detail_profile_entry(
    sample: &str,
    grammar_profile: Option<&str>,
    entry: Option<&str>,
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
    let result = match entry {
        Some(e) => parser.parse_full_from(e).map(|_| ()),
        None => parser.parse_full_systemverilog_file().map(|_| ()),
    };
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

/// SV-AST-SHAPE-FIDELITY.2.4: entry-aware AST-JSON dump for SystemVerilog. Identical to
/// `parse_with_systemverilog_ast_json_profile` except it parses from an ALTERNATE start symbol via
/// `parse_full_from(entry)` instead of the canonical `parse_full_systemverilog_file()`. Lets a rule
/// that is PEG-shadowed / unreachable from `systemverilog_file` (e.g. an expression-position receiver
/// or an entry-relative LRM start symbol) have its typed AST shape dumped in isolation, so the
/// `<invalid_sequence_access>` return-annotation corruption class can be checked per-rule.
#[cfg(has_generated_systemverilog_parser)]
fn parse_with_systemverilog_ast_json_from_entry(
    sample: &str,
    grammar_profile: Option<&str>,
    entry: &str,
) -> Result<JsonValue, String> {
    let mut parser =
        SystemverilogParser::new(sample, runtime_logger_box("generated.systemverilog"));
    parser.set_trace_rules(current_trace_rules());
    let normalized_profile = normalize_generated_grammar_profile("systemverilog", grammar_profile);
    parser.set_grammar_profile(normalized_profile);
    preload_systemverilog_stdlib(&mut parser, normalized_profile)?;
    let _dashboard = maybe_spawn_call_count_dashboard(&parser);
    let parsed = parser
        .parse_full_from(entry)
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

/// GRAMMAR-WELLFORMED.H.5.1 — `ParseDetailFn`-shaped adapter (`fn(&str, Option<&str>) -> Result<(),
/// String>`) so the certificate-coverage report can LABEL svpp's witness-parse failures with the real
/// parse error instead of `"(no detail-capable parser registered)"`. svpp has no grammar profile, so the
/// profile arg is ignored; the body delegates to `parse_with_systemverilog_preprocessor_detail`.
#[cfg(has_generated_systemverilog_preprocessor_parser)]
fn parse_with_systemverilog_preprocessor_detail_profile(
    sample: &str,
    _grammar_profile: Option<&str>,
) -> Result<(), String> {
    parse_with_systemverilog_preprocessor_detail(sample)
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

/// GRAMMAR-WELLFORMED.H.5 — parse `sample` through the REAL systemverilog_preprocessor parser and return
/// `(parsed_ok, rules_exercised)` for `certificate_coverage` (the witness side). Mirrors
/// `parse_and_cover_json`: enable the transactional `coverage_stack`, parse, and return the PARSER's own
/// record of the committed rules on a SUCCESSFUL parse. svpp has no grammar profile and its entry
/// (`systemverilog_preprocessor_file := pp_item*`) is a flat item list, so neither a profile
/// (`_grammar_profile` is unused) nor a dedicated worker stack is needed.
#[cfg(has_generated_systemverilog_preprocessor_parser)]
pub fn parse_and_cover_systemverilog_preprocessor(
    sample: &str,
    _grammar_profile: Option<&str>,
    entry: Option<&str>,
) -> (bool, std::collections::HashSet<String>) {
    let mut parser = SystemverilogPreprocessorParser::new(
        sample,
        runtime_logger_box("generated.systemverilog_preprocessor"),
    );
    parser.enable_coverage();
    // GRAMMAR-WELLFORMED.H.12.8.4.3: verify from the requested entry; `None` ⇒ canonical (unchanged).
    let outcome = match entry {
        Some(e) => parser.parse_full_from(e),
        None => parser.parse_full_systemverilog_preprocessor_file(),
    };
    match outcome {
        Ok(_) => (true, parser.exercised_rule_names()),
        Err(_) => (false, std::collections::HashSet::new()),
    }
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

/// GRAMMAR-WELLFORMED.H.2 — parse `sample` through the REAL vhdl parser and return
/// `(parsed_ok, rules_exercised)` for `certificate_coverage` (the witness side). Mirrors
/// `parse_and_cover_systemverilog_preprocessor`: enable the transactional `coverage_stack`, parse, and
/// return the PARSER's own record of the committed rules on a SUCCESSFUL parse. vhdl has no grammar
/// profile and its entry (`vhdl_file := design_unit*`) is a flat item list, so neither a profile
/// (`_grammar_profile` is unused) nor a dedicated worker stack is needed.
#[cfg(has_generated_vhdl_parser)]
pub fn parse_and_cover_vhdl(
    sample: &str,
    _grammar_profile: Option<&str>,
    entry: Option<&str>,
) -> (bool, std::collections::HashSet<String>) {
    let mut parser = VhdlParser::new(sample, runtime_logger_box("generated.vhdl"));
    parser.enable_coverage();
    // GRAMMAR-WELLFORMED.H.12.8.4.3: verify from the requested entry; `None` ⇒ canonical (unchanged).
    let outcome = match entry {
        Some(e) => parser.parse_full_from(e),
        None => parser.parse_full_vhdl_file(),
    };
    match outcome {
        Ok(_) => (true, parser.exercised_rule_names()),
        Err(_) => (false, std::collections::HashSet::new()),
    }
}

// ============================================================================
// PARSE-HARNESS.2 — the scratch-register slot (approach 3).
//
// `scratch` is a blessed, throwaway grammar slot whose body is meant to be
// overwritten freely (`grammars/scratch/scratch.ebnf` + `make focus_scratch`),
// so an arbitrary grammar becomes a first-class registered parser drivable by
// the whole `parseability_probe` toolbox. Trust: authoritative BY CONSTRUCTION —
// this is the real generated parser + real runtime, identical to how every
// shipped grammar is built and driven; the only trusted surface is this small,
// one-time wiring, covered by the integration test below.
//
// KEY: every dispatch below calls the generated parser's ENTRY-RULE-AGNOSTIC
// `parse_full()` (or `parse_full_from(entry)` when an alternate entry is asked),
// so the registration is STABLE no matter what entry rule the probe grammar uses
// — the registry never needs to know the probe grammar's entry-rule name.
// ============================================================================

#[cfg(has_generated_scratch_parser)]
fn parse_with_scratch(sample: &str) -> bool {
    let mut parser = ScratchParser::new(sample, runtime_logger_box("generated.scratch"));
    parser.set_trace_rules(current_trace_rules());
    parser.parse_full().is_ok()
}

/// PARSE-HARNESS.2 — the `ParseDetailFn` for the scratch slot: parse and, on failure, augment the
/// error with `furthest_position` (the deepest byte any branch reached — the same A2.2/A2.3-grade
/// reject diagnostic the SV detail path emits). scratch has no grammar profile, so the profile arg is
/// ignored.
#[cfg(has_generated_scratch_parser)]
fn parse_with_scratch_detail(sample: &str, _grammar_profile: Option<&str>) -> Result<(), String> {
    parse_with_scratch_detail_entry(sample, None)
}

/// PARSE-HARNESS.2 — scratch detail parse from an OPTIONAL alternate entry (`parseability_probe
/// --entry-rule`). `entry=None` parses from the grammar's canonical entry via `parse_full()`;
/// `Some(rule)` parses from `parse_full_from(rule)`. The `furthest_position` augmentation is preserved
/// either way.
#[cfg(has_generated_scratch_parser)]
fn parse_with_scratch_detail_entry(sample: &str, entry: Option<&str>) -> Result<(), String> {
    let mut parser = ScratchParser::new(sample, runtime_logger_box("generated.scratch"));
    parser.set_trace_rules(current_trace_rules());
    let result = match entry {
        Some(e) => parser.parse_full_from(e).map(|_| ()),
        None => parser.parse_full().map(|_| ()),
    };
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

/// PARSE-HARNESS.2 — parse `sample` through the scratch parser and return `(parsed_ok,
/// rules_exercised)` for `certificate_coverage` (the witness side), so `--report-certificate-coverage`
/// works on a synthetic grammar too. Mirrors `parse_and_cover_json`: enable the transactional
/// `coverage_stack`, parse, return the parser's own record of the committed rules. scratch has no
/// grammar profile.
#[cfg(has_generated_scratch_parser)]
pub fn parse_and_cover_scratch(
    sample: &str,
    _grammar_profile: Option<&str>,
    entry: Option<&str>,
) -> (bool, std::collections::HashSet<String>) {
    let mut parser = ScratchParser::new(sample, runtime_logger_box("generated.scratch"));
    parser.enable_coverage();
    let outcome = match entry {
        Some(e) => parser.parse_full_from(e),
        None => parser.parse_full(),
    };
    match outcome {
        Ok(_) => (true, parser.exercised_rule_names()),
        Err(_) => (false, std::collections::HashSet::new()),
    }
}

#[cfg(has_generated_scratch_parser)]
fn parse_with_scratch_ast_json(sample: &str) -> Result<JsonValue, String> {
    let mut parser = ScratchParser::new(sample, runtime_logger_box("generated.scratch"));
    parser.set_trace_rules(current_trace_rules());
    let parsed = parser.parse_full().map_err(|err| err.to_string())?;
    parse_node_to_json(&parsed)
}

/// PARSE-HARNESS.2 — entry-aware AST-JSON dump for scratch (the `--parse-dump-ast[-pretty]
/// --entry-rule RULE` surface). Parses from an ALTERNATE start symbol via `parse_full_from(entry)` and
/// serializes the typed AST, so a probe grammar's non-entry rule can have its AST shape inspected in
/// isolation (directly useful for the A2.3 shadowing probes).
#[cfg(has_generated_scratch_parser)]
fn parse_with_scratch_ast_json_from_entry(sample: &str, entry: &str) -> Result<JsonValue, String> {
    let mut parser = ScratchParser::new(sample, runtime_logger_box("generated.scratch"));
    parser.set_trace_rules(current_trace_rules());
    let parsed = parser.parse_full_from(entry).map_err(|err| err.to_string())?;
    parse_node_to_json(&parsed)
}

fn parse_node_to_json(node: &ParseNode<'_>) -> Result<JsonValue, String> {
    serde_json::to_value(node).map_err(|err| format!("failed to serialize parse tree: {}", err))
}

static GENERATED_PARSER_REGISTRY: &[GeneratedParserRegistryEntry] = &[
    GeneratedParserRegistryEntry {
        grammar_name: "return_annotation",
        parse_sample: parse_with_return_annotation,
        parse_and_cover: None,
        parse_detail: None,
    },
    GeneratedParserRegistryEntry {
        grammar_name: "semantic_annotation",
        parse_sample: parse_with_semantic_annotation,
        parse_and_cover: None,
        parse_detail: None,
    },
    GeneratedParserRegistryEntry {
        grammar_name: "builtin_return_annotation",
        parse_sample: parse_with_builtin_return_annotation,
        parse_and_cover: None,
        parse_detail: None,
    },
    GeneratedParserRegistryEntry {
        grammar_name: "builtin_semantic_annotation",
        parse_sample: parse_with_builtin_semantic_annotation,
        parse_and_cover: None,
        parse_detail: None,
    },
    #[cfg(all(feature = "ebnf_dual_run", has_generated_ebnf_parser))]
    GeneratedParserRegistryEntry {
        grammar_name: "ebnf",
        parse_sample: parse_with_ebnf,
        parse_and_cover: None,
        parse_detail: None,
    },
    #[cfg(has_generated_json_parser)]
    GeneratedParserRegistryEntry {
        grammar_name: "json",
        parse_sample: parse_with_json,
        parse_and_cover: Some(parse_and_cover_json),
        parse_detail: None,
    },
    #[cfg(has_generated_regex_parser)]
    GeneratedParserRegistryEntry {
        grammar_name: "regex",
        parse_sample: parse_with_regex,
        parse_and_cover: Some(parse_and_cover_regex),
        parse_detail: None,
    },
    #[cfg(has_generated_rtl_const_expr_parser)]
    GeneratedParserRegistryEntry {
        grammar_name: "rtl_const_expr",
        parse_sample: parse_with_rtl_const_expr,
        parse_and_cover: Some(parse_and_cover_rtl_const_expr),
        parse_detail: None,
    },
    #[cfg(has_generated_rtl_frontend_parser)]
    GeneratedParserRegistryEntry {
        grammar_name: "rtl_frontend",
        parse_sample: parse_with_rtl_frontend,
        parse_and_cover: Some(parse_and_cover_rtl_frontend),
        parse_detail: None,
    },
    #[cfg(has_generated_systemverilog_parser)]
    GeneratedParserRegistryEntry {
        grammar_name: "systemverilog",
        parse_sample: parse_with_systemverilog,
        parse_and_cover: Some(parse_and_cover_systemverilog),
        parse_detail: Some(parse_with_systemverilog_detail_profile),
    },
    #[cfg(has_generated_systemverilog_preprocessor_parser)]
    GeneratedParserRegistryEntry {
        grammar_name: "systemverilog_preprocessor",
        parse_sample: parse_with_systemverilog_preprocessor,
        parse_and_cover: Some(parse_and_cover_systemverilog_preprocessor),
        parse_detail: Some(parse_with_systemverilog_preprocessor_detail_profile),
    },
    #[cfg(has_generated_vhdl_parser)]
    GeneratedParserRegistryEntry {
        grammar_name: "vhdl",
        parse_sample: parse_with_vhdl,
        parse_and_cover: Some(parse_and_cover_vhdl),
        parse_detail: None,
    },
    // PARSE-HARNESS.2 — the blessed scratch-register slot. Present only when
    // `make focus_scratch` has built the artifact (additive, cfg-gated, never
    // affecting a shipped grammar). Fully toolbox-capable: parse + cert-coverage
    // witness + labelled detail (`furthest_position`).
    #[cfg(has_generated_scratch_parser)]
    GeneratedParserRegistryEntry {
        grammar_name: "scratch",
        parse_sample: parse_with_scratch,
        parse_and_cover: Some(parse_and_cover_scratch),
        parse_detail: Some(parse_with_scratch_detail),
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

/// The **post-parse semantic contract** a grammar applies on top of the raw grammar parse, if any.
///
/// A few grammars validate more than "did the EBNF grammar parse". Today the only one is `regex`:
/// after `parse_full_regex` succeeds, `parse_sample`/`parse_sample_ast_json` additionally run
/// [`validate_regex_compile_contract`] (the PCRE2-fidelity check that rejects e.g. a quantifier on an
/// anchor, `$+`, which the grammar accepts but PCRE2 rejects — see `parse_with_regex_detail`). This
/// helper exposes that post-parse layer separately from the parse so a second parser implementation
/// (the PARSE-HARNESS interpreter) can be certified against `parse_sample` at the SAME layer: the
/// interpreter reproduces the generated *grammar parse*, and the differential-equivalence gate applies
/// this contract to the interpreter's verdict, mirroring what a downstream `parse_sample` consumer sees
/// (PARSE-HARNESS.5.1). Grammars with no post-parse contract return `Ok(())` (the common case), so the
/// helper is a general, parser-agnostic primitive — a newly-contracted grammar is added here once.
pub fn post_parse_semantic_contract(grammar_name: &str, sample: &str) -> Result<(), String> {
    match grammar_name {
        #[cfg(has_generated_regex_parser)]
        "regex" => validate_regex_compile_contract(sample).map_err(|err| err.message),
        _ => Ok(()),
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
        "regex" => Some(parse_with_regex_detail(sample, grammar_profile)),
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
        #[cfg(has_generated_scratch_parser)]
        "scratch" => Some(parse_with_scratch_detail(sample, grammar_profile)),
        _ => None,
    }
}

/// GRAMMAR-WELLFORMED.H.12.8.4.3.1: entry-aware detail parse for `parseability_probe --entry-rule` —
/// parse `sample` from an ALTERNATE start symbol via the generated parser's `parse_full_from(entry)`
/// (landed in -0141), returning the rich error (SV augments `furthest_position`). Parser-agnostic
/// dispatch (the per-grammar knowledge lives here, like the other registry detail fns). Returns `None`
/// for grammars whose generated parser predates `parse_full_from` (the annotation / `ebnf`
/// meta-grammars) — the probe then reports "no detail-capable parser registered".
pub fn parse_sample_detail_from_entry(
    grammar_name: &str,
    sample: &str,
    grammar_profile: Option<&str>,
    entry: &str,
) -> Option<Result<(), String>> {
    #[cfg(not(any(
        has_generated_systemverilog_parser,
        has_generated_json_parser,
        has_generated_regex_parser,
        has_generated_rtl_const_expr_parser,
        has_generated_rtl_frontend_parser,
        has_generated_systemverilog_preprocessor_parser,
        has_generated_vhdl_parser,
        has_generated_scratch_parser
    )))]
    let _ = (sample, grammar_profile, entry);

    match grammar_name {
        #[cfg(has_generated_systemverilog_parser)]
        "systemverilog" => Some(parse_with_systemverilog_detail_profile_entry(
            sample,
            grammar_profile,
            Some(entry),
        )),
        #[cfg(has_generated_json_parser)]
        "json" => {
            let mut parser = JsonParser::new(sample, runtime_logger_box("generated.json"));
            Some(parser.parse_full_from(entry).map(|_| ()).map_err(|err| err.to_string()))
        }
        #[cfg(has_generated_regex_parser)]
        "regex" => {
            // Runs on the dedicated worker stack (regex can deeply recurse — RGX-0085).
            let profile =
                normalize_generated_grammar_profile("regex", grammar_profile).map(|p| p.to_string());
            let entry_owned = entry.to_string();
            Some(run_generated_regex_on_dedicated_stack(sample, move |owned_sample| {
                let mut parser =
                    RegexParser::new(&owned_sample, runtime_logger_box("generated.regex"));
                parser.set_grammar_profile(profile.as_deref());
                parser
                    .parse_full_from(&entry_owned)
                    .map(|_| ())
                    .map_err(|err| err.to_string())
            }))
        }
        #[cfg(has_generated_rtl_const_expr_parser)]
        "rtl_const_expr" => {
            let mut parser =
                RtlConstExprParser::new(sample, runtime_logger_box("generated.rtl_const_expr"));
            Some(parser.parse_full_from(entry).map(|_| ()).map_err(|err| err.to_string()))
        }
        #[cfg(has_generated_rtl_frontend_parser)]
        "rtl_frontend" => {
            let mut parser =
                RtlFrontendParser::new(sample, runtime_logger_box("generated.rtl_frontend"));
            Some(parser.parse_full_from(entry).map(|_| ()).map_err(|err| err.to_string()))
        }
        #[cfg(has_generated_systemverilog_preprocessor_parser)]
        "systemverilog_preprocessor" => {
            let mut parser = SystemverilogPreprocessorParser::new(
                sample,
                runtime_logger_box("generated.systemverilog_preprocessor"),
            );
            Some(parser.parse_full_from(entry).map(|_| ()).map_err(|err| err.to_string()))
        }
        #[cfg(has_generated_vhdl_parser)]
        "vhdl" => {
            let mut parser = VhdlParser::new(sample, runtime_logger_box("generated.vhdl"));
            Some(parser.parse_full_from(entry).map(|_| ()).map_err(|err| err.to_string()))
        }
        #[cfg(has_generated_scratch_parser)]
        "scratch" => Some(parse_with_scratch_detail_entry(sample, Some(entry))),
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
        "regex" => Some(parse_with_regex_ast_json(sample, grammar_profile)),
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
        #[cfg(has_generated_scratch_parser)]
        "scratch" => Some(parse_with_scratch_ast_json(sample)),
        _ => None,
    }
}

/// SV-AST-SHAPE-FIDELITY.2.4: entry-aware AST-JSON dump. Parse `sample` from an ALTERNATE start
/// symbol via the generated parser's `parse_full_from(entry)` and serialize the typed AST, so an
/// entry-relative rule that is PEG-shadowed / unreachable from the canonical entry can have its AST
/// shape dumped in isolation (the `parseability_probe --parse-dump-ast[-pretty] --entry-rule RULE`
/// surface). `None` for a grammar whose entry-aware AST-JSON variant is not (yet) wired — currently
/// `systemverilog` only; other families can be added on demand by mirroring their
/// `parse_with_<g>_ast_json` function with `parse_full_from(entry)`.
pub fn parse_sample_ast_json_from_entry(
    grammar_name: &str,
    sample: &str,
    grammar_profile: Option<&str>,
    entry: &str,
) -> Option<Result<JsonValue, String>> {
    match grammar_name {
        #[cfg(has_generated_systemverilog_parser)]
        "systemverilog" => Some(parse_with_systemverilog_ast_json_from_entry(
            sample,
            grammar_profile,
            entry,
        )),
        // PARSE-HARNESS.2 — entry-aware AST dump for the scratch slot (probe a non-entry rule in
        // isolation; directly useful for the A2.3 shadowing probes). scratch ignores the profile.
        #[cfg(has_generated_scratch_parser)]
        "scratch" => Some(parse_with_scratch_ast_json_from_entry(sample, entry)),
        _ => {
            let _ = (sample, grammar_profile, entry);
            None
        }
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

    // PARSE-HARNESS.2 — the scratch-register slot is authoritative BY CONSTRUCTION: this test proves
    // the blessed `grammars/scratch/scratch.ebnf` fixture is driven through the REAL generated parser +
    // runtime (the same register→codegen→drive pipeline every shipped grammar uses) and yields the
    // KNOWN verdict + AST. The expecteds are derived from the fixture grammar's SPEC, not mirrored from
    // the tool output: `scratch := "hello, " name "!"`, `name := "world" | "pgen"` — so "hello, world!"
    // and "hello, pgen!" fully-consume (accept), "hello, mars!" has no matching `name` alternative
    // (reject), and "hello, world" is a partial parse (reject, since `parse_full` requires full input).
    // (Asserts against the committed default fixture — restore it via `git checkout` before committing
    // if you edited the body for a probe; see grammars/scratch/README.md.)
    #[cfg(has_generated_scratch_parser)]
    #[test]
    fn scratch_slot_parses_the_blessed_fixture_to_the_known_verdict_and_ast() {
        assert!(supports_grammar("scratch"), "scratch slot must be registered");
        assert!(
            registered_grammars().contains(&"scratch"),
            "scratch must appear in registered_grammars()"
        );

        assert_eq!(parse_sample("scratch", "hello, world!"), Some(true));
        assert_eq!(parse_sample("scratch", "hello, pgen!"), Some(true));
        assert_eq!(
            parse_sample("scratch", "hello, mars!"),
            Some(false),
            "no `name` alternative matches `mars`"
        );
        assert_eq!(
            parse_sample("scratch", "hello, world"),
            Some(false),
            "partial input must be rejected (parse_full requires full consumption)"
        );

        let ast = parse_sample_ast_json("scratch", "hello, world!")
            .expect("scratch is AST-JSON capable")
            .expect("the fixture input parses");
        let ast_str = serde_json::to_string(&ast).expect("AST serializes");
        assert!(
            ast_str.contains("scratch"),
            "AST must root at the `scratch` rule: {ast_str}"
        );
        assert!(
            ast_str.contains("name"),
            "AST must contain the `name` sub-rule: {ast_str}"
        );
        assert!(
            ast_str.contains("world"),
            "AST must retain the matched `world` text: {ast_str}"
        );
    }

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

    /// REGEX-PCRE2-FIDELITY.3.13: quantified anchors reject at the GRAMMAR layer (the anchor
    /// `piece` branch carries `!quantifier`; `find_invalid_quantified_anchor` was removed from
    /// the compile contract). PCRE2 10.47 oracle (`pcre2test`, 2026-07-07): every direct
    /// quantifier on any of the 9 anchor forms is err 109 "quantifier does not follow a
    /// repeatable item" — including the escape anchors the retired validator missed (it checked
    /// only `^`/`$`) — while grouped anchors, class members, the POSIX word-boundary aliases,
    /// and non-quantifier braces stay accepted.
    #[cfg(has_generated_regex_parser)]
    #[test]
    fn regex_quantified_anchors_reject_at_the_grammar_layer_pcre2_faithfully() {
        // Direct quantifier on an anchor: REJECT (PCRE2 err 109).
        for pattern in [
            "^*", "$*", "$?", "^+", "${2}", "${,2}", "a$*", // the pre-3.13 validator's ^/$ set
            "\\A*", "\\A{2}", "\\A{2,}", "\\A{2,3}", "\\A{,2}", // counted forms, oracle-pinned
            "\\b*", "\\B?", "\\G+", "\\z*", "\\Z*", "\\K*", // the divergence set PGEN accepted pre-3.13
        ] {
            assert_eq!(
                parse_sample("regex", pattern),
                Some(false),
                "quantified anchor must reject: {pattern}"
            );
        }
        // Anchor without a following quantifier shape: ACCEPT.
        for pattern in [
            "^", "$", "\\A", "\\Z", "\\z", "\\b", "\\B", "\\G", "\\K", // bare anchors
            "a^b$c", "^abc$", // anchors in concatenation
            "(?:^)*", "(?=\\b)", // grouped anchors ARE quantifiable / assertable
            "[$]*", "[\\b]", // class members, not anchors
            "[[:<:]]*", "[[:>:]]+", // POSIX aliases compile to quantifiable sub-groups (oracle-accepted)
            "${", "\\A{a}", "\\A{2", "\\A{}", "^{a}", // not a quantifier ⇒ literal braces (oracle-accepted)
        ] {
            assert_eq!(
                parse_sample("regex", pattern),
                Some(true),
                "non-quantified anchor form must accept: {pattern}"
            );
        }
        // The tightening applies in BOTH profiles (the contract rejected `^*` in relaxed too).
        // (`parse_sample_detail_with_profile` is the profile-routing verdict API for regex;
        // `parse_sample_with_profile` threads profiles only for systemverilog.)
        assert!(
            super::parse_sample_detail_with_profile("regex", "\\b*", Some("relaxed"))
                .expect("regex registered")
                .is_err(),
            "relaxed must also reject a quantified escape-anchor"
        );
        // Relaxed still re-admits the .3.1 fidelity letters (regression guard for the
        // simple_escape positive-enumeration restructure).
        assert!(
            super::parse_sample_detail_with_profile("regex", "\\u", Some("relaxed"))
                .expect("regex registered")
                .is_ok(),
            "relaxed must still re-admit \\u"
        );
        assert_eq!(parse_sample("regex", "\\u"), Some(false));
    }

    /// REGEX-PCRE2-FIDELITY.4.1 (ledger `REGEX-0100`, release 1.1.90): a BARE `\p` / `\P`
    /// Unicode-property escape is grammar-owned — `\p` / `\P` are ALWAYS property introducers
    /// (owned by `property_escape`, tried first in `escape_unit` / `class_escape_unit`), never a
    /// bare shorthand. A bad-letter (`\pA`), underscore (`\P_`), or at-EOF (`\p`) form hard-REJECTS
    /// instead of decomposing into `\p` + literal. Encoded structurally: `p` / `P` dropped from
    /// `simple_escape_letter_strict` (the positive-set / generation-faithful half) and the
    /// `!"p{"` / `!"P{"` guards broadened to whole-letter `!"p"` / `!"P"` on `simple_escape` +
    /// both `class_simple_escape` variants. This MIGRATED `regex_compile_validation.rs::
    /// find_invalid_property_escape` into the EBNF (the grammar is the single source of truth);
    /// behavior-NEUTRAL downstream — the validator rejected these before, the grammar rejects them
    /// now. Oracle: `pcre2test` 10.47 (`regex_pcre2_compile_oracle_gate`).
    #[cfg(has_generated_regex_parser)]
    #[test]
    fn regex_bare_property_escape_pcre2_faithfully() {
        // Bare `\p` / `\P` not a valid one-letter category (or at EOF): REJECT — atom, class,
        // and mid-pattern contexts.
        for pattern in [
            "\\pA", "\\P_", "\\p", "\\P", // atom-level bad-letter / at-EOF
            "[\\pA]", "[\\P_]", "[a\\pA]", // class-level bad-letter
            "a\\pAb", "x\\p", // mid-pattern
        ] {
            assert_eq!(
                parse_sample("regex", pattern),
                Some(false),
                "bare non-category property escape must reject: {pattern}"
            );
        }
        // Valid short one-letter category, and the braced `{name}` form: ACCEPT — atom + class.
        for pattern in [
            "\\pL", "\\PN", "\\pl", "\\Pn", "\\pC", "\\pZ", // short one-letter categories
            "[\\pL\\PN]", // short categories inside a class
            "\\p{Lu}", "\\P{Han}", "[\\p{L}]", // braced property names (unchanged)
        ] {
            assert_eq!(
                parse_sample("regex", pattern),
                Some(true),
                "valid property escape must accept: {pattern}"
            );
        }
        // The rule is unconditional (the migrated validator ran in BOTH profiles): relaxed also rejects.
        assert!(
            super::parse_sample_detail_with_profile("regex", "\\pA", Some("relaxed"))
                .expect("regex registered")
                .is_err(),
            "relaxed must also reject a bare non-category property escape"
        );
    }

    /// REGEX-PCRE2-FIDELITY.4.6 (ledger `REGEX-0101`, release 1.1.91): POSIX class NAME validity is
    /// grammar-owned — a `[:name:]` token inside a class is a POSIX-class ATTEMPT (PCRE2 err 130 for
    /// an unknown name), never a literal fallback. Encoded structurally: the class-member `[` literal
    /// is guarded by an inline negative lookahead for the `[:…:]` posix-token shape
    /// (`class_member_literal` / `class_member_literal_nocaret`), so `posix_class` (valid 14 names,
    /// optional `^`) wins for a good name and an INVALID name leaves the class unclosable → reject.
    /// BEHAVIOR-NEUTRAL migration: the guard scans to the FIRST `:]` exactly like the deleted validator
    /// `regex_compile_validation.rs::find_invalid_char_class_construct`'s `is_valid_posix_class_name`
    /// (`scan_posix_class`), so the SET of rejected inputs is byte-identical (only the reject source/message
    /// moves validator→grammar); `scan_posix_class` recognition stays for range analysis (`.4.5`). Honest
    /// bound (pre-existing, out of scope): the scan crosses BOTH escaped and unescaped `]`, whereas PCRE2's
    /// posix-name boundary stops at an UNESCAPED `]` — a validator↔PCRE2 divergence left for a follow-up.
    /// Oracle: `pcre2test` 10.47 (`regex_pcre2_compile_oracle_gate`, byte-identical `2189/1858/285/46`).
    #[cfg(has_generated_regex_parser)]
    #[test]
    fn regex_posix_class_names_reject_at_the_grammar_layer_pcre2_faithfully() {
        // Unknown / malformed POSIX name (incl. negated, mid-class, empty, spaced, `]`-free extra colon):
        // REJECT — the grammar no longer falls back to literals. All err 130 under pcre2test 10.47.
        for pattern in [
            "[[:foo:]]", "[[:foo:]",   // unknown name (with / without the outer close)
            "[a[:<:]]", "[a[:>:]]",     // word-boundary alias SPELLING inside a larger class (name `<`/`>`)
            "[[::]]", "[[:^:]]",        // empty name (plain / negated)
            "[[:al pha:]]",             // space in name
            "[x[:foo:]y]", "[[:foo:]x]", // posix attempt not at the class start
            "[^[:foo:]]", "[[:^foo:]]", // negated class / negated posix name
            "[[:foo:bar:]]", "[[:al:num:]]", // extra `:` in the (still `]`-free) name
        ] {
            assert_eq!(
                parse_sample("regex", pattern),
                Some(false),
                "unknown POSIX class name must reject at the grammar layer: {pattern}"
            );
        }
        // Valid names / non-posix `[:` shapes / aliases / quoted: ACCEPT (unchanged from the validator era).
        for pattern in [
            "[[:alpha:]]", "[[:^alpha:]]", "[[:word:]]", "[[:xdigit:]]", // valid (plain / negated)
            "[[:alnum:][:digit:]]", "[^[:alpha:]]", "[x[:alpha:]y]",     // valid, multi / negated / embedded
            "[[:<:]]", "[[:>:]]", "[[:<:]]red[[:>:]]", "[[:<:]]+", "red[[:>:]]+", // word-boundary anchor aliases
            "[[:foo]",                     // no `:]` terminator before class end → `[:` is literal
            "[[:]]", "[[:]", "([[:]+)",    // `[:` with no `:]` terminator → literal
            "[a:foo:]", "[]:foo:]",        // no `[:` opener at all (first-`]`-literal in the 2nd)
            "[\\Q[:foo:]\\E]",             // `[:foo:]` quoted inside \Q...\E is literal
        ] {
            assert_eq!(
                parse_sample("regex", pattern),
                Some(true),
                "valid POSIX class / non-posix `[:` shape must accept: {pattern}"
            );
        }
        // The guard is unconditional (the migrated validator ran in BOTH profiles): relaxed also rejects.
        assert!(
            super::parse_sample_detail_with_profile("regex", "[[:foo:]]", Some("relaxed"))
                .expect("regex registered")
                .is_err(),
            "relaxed must also reject an unknown POSIX class name"
        );
    }

    /// REGEX-PCRE2-FIDELITY.3.19: a stray `\E` (an unmatched end-of-quote) is PCRE2 zero-width
    /// and — unlike an anchor (opaque, `.3.13`) — TRANSPARENT to a quantifier. A quantifier
    /// binds THROUGH the stray `\E` to the preceding repeatable atom (`a\E*` = `a*`), but is
    /// err 109 "quantifier does not follow a repeatable item" when no repeatable predecessor is
    /// reachable through elision — nothing before it, or an anchor / group-open / alternation
    /// edge blocks it. Encoded structurally: stray `\E` is a non-quantifiable `zero_width`
    /// piece (dropped from `simple_escape_letter_strict`); the `piece` ABSORPTION branch
    /// `atom zero_width+ quantifier` lets a quantifier reach through it, the `\E`s elided.
    /// Oracle: `pcre2test` 10.47 (44-cell map). Empty-`\Q\E`-quantified is spun out to `.3.23`
    /// (the `\Q`-as-`simple_escape` / unterminated-`\Q...\E` entanglement).
    #[cfg(has_generated_regex_parser)]
    #[test]
    fn regex_stray_end_quote_quantifier_binds_through_pcre2_faithfully() {
        // Quantifier with NO repeatable predecessor reachable through elision: REJECT (err 109).
        for pattern in [
            "\\E*", "\\E+", "\\E?", "\\E{2}", "\\E{2,}", "\\E{2,3}", "\\E{,2}", // bare stray \E + quantifier
            "\\E\\E*",              // two stray \E — still no repeatable predecessor
            "^\\E*", "\\A\\E*",     // an anchor precedes: non-repeatable, blocks the bind
            "a^\\E*",              // `a` precedes but the immediate predecessor after eliding \E is `^`
            "(\\E*)",              // group-open edge resets the predecessor
            "|\\E*", "a|\\E*", "(a|\\E*)", // alternation edge resets the predecessor
        ] {
            assert_eq!(
                parse_sample("regex", pattern),
                Some(false),
                "stray-\\E quantifier with no repeatable predecessor must reject: {pattern}"
            );
        }
        // Quantifier binds THROUGH the transparent stray \E to the preceding repeatable atom: ACCEPT.
        for pattern in [
            "a\\E*", "a\\E\\E*", "ab\\E*", "\\Qa\\E\\E*", // <atom> \E+ <quantifier>
            "\\Ea*", "\\E\\Ea*",                          // leading stray \E then a quantified real atom
            "()\\E*", "(a)\\E*", "(?:)\\E*", "(a|b)\\E*", // a group is a repeatable atom
            "^\\Ea*", "\\E|a*",                          // anchor / alternation-left then a quantified atom
        ] {
            assert_eq!(
                parse_sample("regex", pattern),
                Some(true),
                "stray-\\E-transparent quantifier binding must accept: {pattern}"
            );
        }
        // Bare stray \E and non-quantifier braces (not a bound quantifier): ACCEPT (unchanged).
        for pattern in ["\\E", "\\Ea", "a\\E", "\\E{a}", "\\E{", "(\\E)*", "(\\E\\E)*"] {
            assert_eq!(
                parse_sample("regex", pattern),
                Some(true),
                "stray \\E without a bound quantifier must accept: {pattern}"
            );
        }
        // REGEX-PCRE2-FIDELITY.3.23 CLOSED the empty-`\Q\E`-quantified divergence: `\Q\E*` now
        // REJECTS err-109-faithfully (empty `\Q\E` joined `zero_width`). The full family is pinned
        // by `regex_quoted_literal_model_pcre2_faithfully` below.
        assert_eq!(parse_sample("regex", "\\Q\\E*"), Some(false));
        // The tightening applies in BOTH profiles (quantifier-target validity is not a relaxed
        // concern — the `.3.13` precedent). `parse_sample_detail_with_profile` is the
        // profile-routing verdict API for regex.
        assert!(
            super::parse_sample_detail_with_profile("regex", "\\E*", Some("relaxed"))
                .expect("regex registered")
                .is_err(),
            "relaxed must also reject a quantified stray \\E"
        );
        // Relaxed still re-admits the .3.1 fidelity letters (regression guard for the
        // simple_escape strict-set edit that dropped `E`).
        assert!(
            super::parse_sample_detail_with_profile("regex", "\\u", Some("relaxed"))
                .expect("regex registered")
                .is_ok(),
            "relaxed must still re-admit \\u"
        );
    }

    /// REGEX-PCRE2-FIDELITY.3.14: verb/start-option argument SHAPES reject at the GRAMMAR layer
    /// (name-class-conditional `directive_named` branches; the matching
    /// `find_invalid_verb_construct` shape checks were removed from the compile contract).
    /// PCRE2 10.47 oracle (`pcre2test`, 2026-07-07): MARK — named or `(*:...)` shorthand —
    /// REQUIRES a non-empty `:`-payload (err 166); the other 7 verbs take an OPTIONAL
    /// `:`-payload only (`=` is err 160); the 4 LIMIT_* start options REQUIRE `=digits`
    /// (bare/empty/non-digit forms are err 160 — ledger REGEX-0089: PGEN wrongly accepted
    /// bare `(*LIMIT_HEAP)` and `=digits` on non-LIMIT options like `(*UTF=5)`); every other
    /// start option is BARE-only (err 160 otherwise).
    #[cfg(has_generated_regex_parser)]
    #[test]
    fn regex_verb_argument_shapes_reject_at_the_grammar_layer_pcre2_faithfully() {
        // Invalid argument shapes: REJECT (PCRE2 err 166 / err 160).
        for pattern in [
            "(*:)", "(*MARK)", "(*MARK:)", // MARK without its required non-empty argument
            "(*MARK=x)", "(*MARK=)", // MARK never takes `=`
            "(*PRUNE=)", "(*PRUNE=x)", "(*SKIP=)", "(*SKIP=x)", "(*THEN=x)", "(*COMMIT=x)",
            "(*ACCEPT=x)", "(*FAIL=x)", // verbs are `:`-suffix only
            "(*UTF:x)", "(*UTF=5)", "(*UTF=)", "(*CR=5)",
            "(*TURKISH_CASING=5)", // non-LIMIT start options are bare-only (REGEX-0089)
            "(*LIMIT_HEAP)", // LIMIT_* requires `=digits` (REGEX-0089)
            "(*LIMIT_HEAP=)", "(*LIMIT_HEAP=abc)", "(*LIMIT_HEAP=5x)", "(*LIMIT_HEAP:5)",
            "a(*LIMIT_HEAP=500)", "(*FAIL)(*LIMIT_HEAP=5)a", // `=`-form position (REGEX-0090)
        ] {
            assert_eq!(
                parse_sample("regex", pattern),
                Some(false),
                "invalid verb/start-option argument shape must reject: {pattern}"
            );
        }
        // Valid shapes: ACCEPT (all oracle-verified).
        for pattern in [
            "(*:x)", "(*:name)", "(*MARK:x)", // MARK with a non-empty argument
            "(*PRUNE)", "(*PRUNE:)", "(*PRUNE:x)", "(*SKIP)", "(*SKIP:)", "(*SKIP:x)",
            "(*THEN)", "(*THEN:x)", "(*COMMIT)", "(*COMMIT:x)", "(*ACCEPT)", "(*ACCEPT:x)",
            "(*F)", "(*F:x)", "(*FAIL)", "(*FAIL:x)", // verbs: optional `:`-payload, empty OK
            "(*UTF)", "(*UCP)", "(*NOTEMPTY_ATSTART)", "(*BSR_ANYCRLF)", "(*NUL)",
            "(*NO_DOTSTAR_ANCHOR)", "(*CASELESS_RESTRICT)", // bare start options
            "(*LIMIT_HEAP=500)", "(*LIMIT_MATCH=1000)", "(*LIMIT_DEPTH=10)",
            "(*LIMIT_RECURSION=10)", "(*LIMIT_HEAP=0)", // LIMIT_* with the required digits
            "(*LIMIT_HEAP=500)a", "(*LIMIT_MATCH=10)(*UCP)a", // start-option prefix then pattern
            "(*UTF)(*UCP)a", "a(*PRUNE:x)b", "(*MARK:x)(*FAIL)a", // verbs are position-free
            "(*ACCEPT)+", "(*ACCEPT:x)+", // only ACCEPT quantifies (grammar-owned since .3.20)
        ] {
            assert_eq!(
                parse_sample("regex", pattern),
                Some(true),
                "valid verb/start-option argument shape must accept: {pattern}"
            );
        }
        // The tightening applies in BOTH profiles: the relaxed catch-all's recognized-name
        // exclusion guard keeps the strict shapes authoritative under `relaxed` too.
        // (`parse_sample_detail_with_profile` is the profile-routing verdict API for regex;
        // `parse_sample_with_profile` threads profiles only for systemverilog.)
        for pattern in ["(*:)", "(*MARK)", "(*SKIP=)", "(*UTF=5)", "(*LIMIT_HEAP)", "(*F=x)"] {
            assert!(
                super::parse_sample_detail_with_profile("regex", pattern, Some("relaxed"))
                    .expect("regex registered")
                    .is_err(),
                "relaxed must also reject the invalid recognized-name shape: {pattern}"
            );
        }
        // Relaxed still re-admits UNRECOGNIZED names with any suffix shape (the catch-all
        // surface is unchanged for extended spellings, including strict-name extensions).
        for pattern in ["(*FOO)", "(*FOO=x)", "(*FOO:x)", "(*SKIPX)", "(*LIMIT_HEAPX=5)"] {
            assert!(
                super::parse_sample_detail_with_profile("regex", pattern, Some("relaxed"))
                    .expect("regex registered")
                    .is_ok(),
                "relaxed must still re-admit the unrecognized directive name: {pattern}"
            );
            assert_eq!(
                parse_sample("regex", pattern),
                Some(false),
                "the default profile must keep rejecting the unrecognized name: {pattern}"
            );
        }
    }

    /// REGEX-PCRE2-FIDELITY.3.20 (ledger REGEX-0096): only `(*ACCEPT)` may take a quantifier —
    /// every OTHER `(*...)` directive (the 6 non-ACCEPT verbs, MARK, the `(*:x)` shorthand, LIMIT,
    /// and the bare start options) rejects a following quantifier at the GRAMMAR layer (the
    /// non-quantifiable `directive_verb_nonquant !quantifier` piece branch; the two quantified-verb
    /// arms of `find_invalid_verb_construct` were removed). PCRE2 10.47 oracle (`pcre2test`,
    /// 2026-07-08): `(*ACCEPT)+` `(*ACCEPT:x)+` `(*ACCEPT){2,3}` ACCEPT; `(*PRUNE)+` `(*:x)+`
    /// `(*MARK:x)+` err 109; and — the NEW `REGEX-0096` close — `(*UTF)+` / `(*LIMIT_HEAP=5)+`
    /// (start-options quantified) also err 109, where the pre-`.3.20` validator only checked
    /// start-option POSITION and latently accepted them.
    #[cfg(has_generated_regex_parser)]
    #[test]
    fn regex_quantified_verb_rejects_at_the_grammar_layer_pcre2_faithfully() {
        // Quantified non-ACCEPT directives: REJECT (PCRE2 err 109).
        for pattern in [
            "(*PRUNE)+", "(*FAIL)*", "(*F)+", "(*SKIP)+", "(*COMMIT)+", "(*THEN)+", // verbs
            "(*PRUNE){2}", "(*PRUNE){2,3}", "(*PRUNE){,2}", // counted forms
            "(*:x)+", "(*:x){2}",   // the `(*:x)` shorthand
            "(*MARK:x)+", "(*MARK:x){2}", // MARK
            "(*UTF)+", "(*UCP)+",   // bare start options (REGEX-0096)
            "(*LIMIT_HEAP=5)+",     // LIMIT start option (REGEX-0096)
            "a(*PRUNE)+b",          // mid-pattern, real atoms around it
        ] {
            assert_eq!(
                parse_sample("regex", pattern),
                Some(false),
                "a quantified non-ACCEPT directive must reject (err 109): {pattern}"
            );
        }
        // Only `(*ACCEPT)` quantifies: ACCEPT (every quantifier form). Bare non-ACCEPT directives
        // (no quantifier) still accept — the split changed quantifiability only, not acceptance.
        for pattern in [
            "(*ACCEPT)+", "(*ACCEPT)*", "(*ACCEPT)?", "(*ACCEPT){2,3}", "(*ACCEPT){2,}",
            "(*ACCEPT:x)+", "a(*ACCEPT)+b", // ACCEPT quantified, oracle-verified
            "(*ACCEPT)", "(*PRUNE)", "(*:x)", "(*MARK:x)", "(*UTF)", "(*LIMIT_HEAP=5)", // bare
        ] {
            assert_eq!(
                parse_sample("regex", pattern),
                Some(true),
                "an ACCEPT-quantified or non-quantified directive must accept: {pattern}"
            );
        }
        // The tightening applies in BOTH profiles for KNOWN names (the relaxed catch-all's
        // recognized-name exclusion keeps them on their non-quantifiable strict shapes).
        for pattern in ["(*PRUNE)+", "(*:x)+", "(*UTF)+", "(*LIMIT_HEAP=5)+", "(*MARK:x)+"] {
            assert!(
                super::parse_sample_detail_with_profile("regex", pattern, Some("relaxed"))
                    .expect("regex registered")
                    .is_err(),
                "relaxed must also reject a quantified KNOWN directive: {pattern}"
            );
        }
        // RELAXED-SEMANTICS DECISION (REGEX-PCRE2-FIDELITY.3.20): a quantified UNKNOWN-name verb
        // stays relaxed-ACCEPTED (relaxed = a strict superset that never newly rejects — the
        // unknown-name catch-all is quantifiable), while the default profile keeps rejecting it.
        for pattern in ["(*foo)+", "(*bar)*", "(*baz){2}"] {
            assert!(
                super::parse_sample_detail_with_profile("regex", pattern, Some("relaxed"))
                    .expect("regex registered")
                    .is_ok(),
                "relaxed must keep accepting a quantified unknown-name verb: {pattern}"
            );
            assert_eq!(
                parse_sample("regex", pattern),
                Some(false),
                "the default profile must reject a quantified unknown-name verb: {pattern}"
            );
        }
    }

    /// REGEX-PCRE2-FIDELITY.3.23 (ledger REGEX-0099): the PCRE2 `\Q` QUOTING model is grammar-encoded
    /// as first-class quoting — `\Q` is no longer a bare `simple_escape` (`Q` dropped from
    /// `simple_escape_letter_strict`). Closes two divergence classes at once:
    ///   - ACCEPTS-INVALID (tightening): empty `\Q\E` is now a non-quantifiable `zero_width`, so a
    ///     quantifier on it is err 109 (`\Q\E*` `\Q\E{2}` REJECT) — the `.3.19`-deferred family.
    ///   - REJECTS-VALID (widening): unterminated `\Q…` quotes to END-OF-PATTERN as literal
    ///     (`unterminated_quoted_literal`), so a structural metachar tail (`\Q)` `\Q(` `\Q[` `\Q^`
    ///     `\Q|`) is one literal run, not live regex.
    /// Oracle `pcre2test` 10.47 (85-cell matrix in the task leaf); validated pre-rebuild by the
    /// regex-CERTIFIED interpreter (`parse_harness_interpreter::interpret_parse`, 0 divergences).
    #[cfg(has_generated_regex_parser)]
    #[test]
    fn regex_quoted_literal_model_pcre2_faithfully() {
        // Empty `\Q\E` + quantifier: REJECT (err 109 — an empty quote is zero-width, unrepeatable).
        for pattern in [
            "\\Q\\E*", "\\Q\\E+", "\\Q\\E?", "\\Q\\E{2}", "\\Q\\E{2,}", "\\Q\\E{2,3}", "\\Q\\E{,2}",
            "\\Q\\E\\Q\\E*", // two empty quotes then a quantifier — still no repeatable predecessor
        ] {
            assert_eq!(
                parse_sample("regex", pattern),
                Some(false),
                "a quantifier on an empty \\Q\\E must reject (err 109): {pattern}"
            );
        }
        // Unterminated `\Q…` (no closing `\E`): quote-to-end, so a structural metachar tail is
        // literal — ACCEPT (was REJECTS-VALID before `.3.23`).
        for pattern in [
            "\\Q", "\\Qa", "\\Qabc", "\\Qa*b", "\\Q*", "\\Q**", "\\Qa**", "\\Q)", "\\Q(", "\\Q[",
            "\\Q]", "\\Q^", "\\Q$", "\\Q|", "\\Q(?:", "\\Qa)b", "\\Q++b", "\\Q*b*c",
        ] {
            assert_eq!(
                parse_sample("regex", pattern),
                Some(true),
                "an unterminated \\Q… quote-to-end must accept (literal tail): {pattern}"
            );
        }
        // Terminated `\Q…\E` (unchanged), the quantifier-binds-last-char form, absorption, and a
        // bare empty `\Q\E` (no quantifier): all ACCEPT.
        for pattern in [
            "\\Qa\\E", "\\Qabc\\E", "\\Q)\\E", "\\Q(?:\\E", "\\Q**\\E", // terminated (metachars literal)
            "\\Qa\\E*", "\\Qab\\E{2,3}",                                 // quantifier binds last char
            "a\\Q\\E*", "a\\Q\\E\\E*",                                   // absorption: `a*`
            "\\Q\\E", "a\\Q\\E", "\\Q\\Eb", "\\Q\\E\\E",                 // bare empty quote (no quantifier)
        ] {
            assert_eq!(
                parse_sample("regex", pattern),
                Some(true),
                "a terminated / bound / absorbed / bare \\Q form must accept: {pattern}"
            );
        }
        // The empty-quantified tightening applies in BOTH profiles (quantifier-target validity is
        // not a relaxed concern — the `.3.13`/`.3.19`/`.3.20` precedent).
        for pattern in ["\\Q\\E*", "\\Q\\E{2}"] {
            assert!(
                super::parse_sample_detail_with_profile("regex", pattern, Some("relaxed"))
                    .expect("regex registered")
                    .is_err(),
                "relaxed must also reject an empty-\\Q\\E-quantified form: {pattern}"
            );
        }
        // First-class quoting pin: unterminated `\Q)` is ONE `quoted_literal` atom whose body is the
        // literal tail — not `\Q`-as-escape + a live `)` (which rejected pre-`.3.23`).
        let ast = super::parse_sample_ast_json("regex", "\\Q)")
            .expect("regex registered")
            .expect("\\Q) parses");
        let atom = &ast["content"]["Json"]["pattern"][0][0][0]["atom"];
        assert_eq!(
            atom["kind"].as_str(),
            Some("quoted_literal"),
            "\\Q) must be a quoted_literal atom, got {atom}"
        );
        assert_eq!(
            atom["body"][0].as_str(),
            Some(")"),
            "\\Q) body must be the literal `)`, got {atom}"
        );
    }

    /// REGEX-PCRE2-FIDELITY.3.15 (ledger REGEX-0091): the PCRE2 CLASS-OPEN model is
    /// grammar-encoded — class NON-EMPTINESS counts only VISIBLE members (stray `\E` and the
    /// empty `\Q\E` are PCRE2-invisible), the negation caret is recognized THROUGH invisibles
    /// (and a caret after it is an ordinary member), and a `]` seen before any visible member
    /// is a LITERAL member. `\Q` inside a class is always the quote-opener (never a shorthand
    /// escape), so unterminated in-class quotes reject at the grammar. PCRE2 10.47 oracle
    /// (`pcre2test`, 2026-07-08): the 80-pattern matrix in the task leaf; rejects are err 106.
    #[cfg(has_generated_regex_parser)]
    #[test]
    fn regex_class_member_visibility_rejects_at_the_grammar_layer_pcre2_faithfully() {
        // Invisible-only / caret-negated-empty / unterminated-quote classes: REJECT (err 106).
        for pattern in [
            "[\\E]", "[\\Q\\E]", "[\\E\\E]", "[\\E\\Q\\E]", "[\\Q\\E\\E]",
            "[\\Q\\E\\Q\\E]", // invisible-only bodies
            "[^\\E]", "[^\\Q\\E]", "[^\\E\\E]", // negated invisible-only bodies
            "[\\E^]", "[\\Q\\E^]", // the caret THROUGH invisibles is the negation (then empty)
            "[\\Q]", "[\\Qa]", "[\\Q]x]", "[a\\Q]", "[\\Qab]",
            "[\\Qa\\E", // in-class `\Q` quotes the `]` ⇒ unterminated
            "[]", "[^]", // the .3.7 pins (the first `]` is a literal member, so these are empty)
        ] {
            assert_eq!(
                parse_sample("regex", pattern),
                Some(false),
                "invisible-only / unterminated class must reject: {pattern}"
            );
        }
        // The 17 REGEX-0091 rejects-valid flips + key stays: ACCEPT (all oracle-verified).
        for pattern in [
            "[\\E]x]", "[\\Q\\E]x]", "[\\E\\E]x]", "[\\E\\Q\\E]x]", "[\\E]]",
            "[\\Q\\E]]", // invisible prefix ⇒ the `]` is a literal member
            "[^\\E]x]", "[^\\Q\\E]x]", "[^\\E]]", // ... after the negation caret too
            "[^^]", "[^^]x]", "[^\\E^]", "[^\\Q\\E^]", // a caret after the negation is a MEMBER
            "[\\E^]x]", "[\\E^\\E]x]", "[\\E^^]", "[\\E\\E^]y]", // negation through invisibles
            "[a\\E]", "[\\Ea]", "[\\Qa\\E]", "[\\E\\Qa\\E]", "[\\Q\\Ea]",
            "[a\\Q\\E]", "[\\Q\\E\\Qa\\E]", // visible member + invisibles: unchanged accepts
            "[\\Q^\\E]", "[\\^]", // a QUOTED/ESCAPED caret first is a member, not negation
            "[\\E^a]", "[\\Q\\E^a]", "[\\E^-z]", // the negated-class semantic corrections
            "[]]", "[^]]", "[]a]", "[^]a]", "[]\\E]", // the .3.7 initial-close pins
            "[\\E-x]", "[a-\\E]", "[a\\E-z]", "[a-\\Ez]", "[\\Qa\\E-z]", // range/dash + invisibles
            "\\E", "a\\E", "\\Q\\E", "\\Qab\\E", // pattern-level quote surface unchanged
        ] {
            assert_eq!(
                parse_sample("regex", pattern),
                Some(true),
                "visible-member class form must accept: {pattern}"
            );
        }
        // The negation caret recognized through invisibles is a SEMANTIC correction: `[\E^a]`
        // is a NEGATED class of `a` (PGEN previously parsed a non-negated 3-member class).
        let ast = super::parse_sample_ast_json("regex", "[\\E^a]")
            .expect("regex registered")
            .expect("accepts [\\E^a]")
            .to_string();
        assert!(
            ast.contains("\"negated\":true"),
            "[\\E^a] must parse as a NEGATED class: {ast}"
        );
        // The tightening applies in BOTH profiles (the `\Q`/`\E` guards + visibility rules are
        // profile-shared; relaxed differs only on the .3.1 escape letters).
        // (`parse_sample_detail_with_profile` is the profile-routing verdict API for regex.)
        for pattern in ["[\\E]", "[\\Q]", "[\\Q\\E^]", "[^^]x]"] {
            let verdict = super::parse_sample_detail_with_profile("regex", pattern, Some("relaxed"))
                .expect("regex registered");
            assert_eq!(
                verdict.is_ok(),
                parse_sample("regex", pattern) == Some(true),
                "relaxed must agree with the default profile on the class-open surface: {pattern}"
            );
        }
        // Relaxed still re-admits the .3.1 fidelity letters in CLASS context (regression guard
        // for the class_simple_escape_relaxed catch-all renumbering).
        for pattern in ["[\\u]", "[\\F]", "[\\l]", "[\\i]"] {
            assert!(
                super::parse_sample_detail_with_profile("regex", pattern, Some("relaxed"))
                    .expect("regex registered")
                    .is_ok(),
                "relaxed must still re-admit the class-context escape letter: {pattern}"
            );
            assert_eq!(
                parse_sample("regex", pattern),
                Some(false),
                "the default profile must keep rejecting the class-context escape letter: {pattern}"
            );
        }
    }

    /// REGEX-PCRE2-FIDELITY.3.16: the numeric callout argument is VALUE-bounded to [0, 255]
    /// at the GRAMMAR layer (`callout_number`), replacing the migrated
    /// `find_invalid_numeric_callout` validator check. All verdicts oracle-verified
    /// (`pcre2test` 10.47: in-range and leading-zero forms accept; any value > 255 is err 138).
    #[cfg(has_generated_regex_parser)]
    #[test]
    fn regex_numeric_callout_range_rejects_at_the_grammar_layer_pcre2_faithfully() {
        // Out-of-range numeric callouts: REJECT (err 138 — the VALUE exceeds 255; leading
        // zeros do not save an out-of-range value; the digit count alone decides nothing).
        for pattern in [
            "(?C256)",
            "(?C262)",
            "(?C260)",
            "(?C999)",
            "(?C1000)",
            "(?C2555)",
            "(?C0256)",
            "(?C000000000256)",
            "(?C999999999999999999999)",
            "(?(?C262)(?=y)x|z)", // the condition-callout site shares callout_arg
        ] {
            assert_eq!(
                parse_sample("regex", pattern),
                Some(false),
                "out-of-range numeric callout must reject: {pattern}"
            );
        }
        // In-range numeric callouts (VALUE ≤ 255, arbitrary leading zeros) and string
        // callout-args (not value-bounded): ACCEPT.
        for pattern in [
            "(?C)",
            "(?C0)",
            "(?C1)",
            "(?C25)",
            "(?C99)",
            "(?C199)",
            "(?C249)",
            "(?C250)",
            "(?C255)",
            "(?C0255)",
            "(?C00)",
            "(?C000000000255)",
            "(?C010)",
            "(?C255)x",
            "(?(?C255)(?=y)x|z)",
            "(?(?C0255)(?=y)x|z)",
            "(?C`ab`)",
            "(?C'cd')",
            "(?C{ef})",
        ] {
            assert_eq!(
                parse_sample("regex", pattern),
                Some(true),
                "in-range / string callout must accept: {pattern}"
            );
        }
        // The typed-int carrier is preserved: leading zeros collapse to the VALUE (the
        // `@transform` span parse on `callout_number`), exactly as the former `digits`
        // branch surfaced it.
        let ast = super::parse_sample_ast_json("regex", "(?C0255)")
            .expect("regex registered")
            .expect("accepts (?C0255)")
            .to_string();
        assert!(
            ast.contains("\"arg\":255"),
            "(?C0255) must carry the typed int value 255: {ast}"
        );
        // The bound applies in BOTH profiles (callout_number is profile-shared).
        for pattern in ["(?C256)", "(?C0256)", "(?C255)", "(?C0255)"] {
            let verdict = super::parse_sample_detail_with_profile("regex", pattern, Some("relaxed"))
                .expect("regex registered");
            assert_eq!(
                verdict.is_ok(),
                parse_sample("regex", pattern) == Some(true),
                "relaxed must agree with the default profile on the callout range: {pattern}"
            );
        }
    }

    /// REGEX-PCRE2-FIDELITY.3.21 (ledger REGEX-0097): the LIMIT `=value` is VALUE-bounded to
    /// [0, 4294967289] at the GRAMMAR layer (the structural `directive_limit_value_body` /
    /// `directive_limit_value_core` ladder on `directive_payload_digits`; no validator check ever
    /// bounded the LIMIT value — this closed a latent released accepts-invalid hole). PCRE2 10.47
    /// oracle (`pcre2test`, 2026-07-08, blank-line-separated + binary search): `(*LIMIT_HEAP=4294967289)`
    /// accepts, `(*LIMIT_HEAP=4294967290)` rejects (err 160). The bound is PCRE2's Horner overflow
    /// guard `n > UINT32_MAX/10 - 1` (max accepted = 429496728*10 + 9 = 4294967289, NOT u32 max
    /// 4294967295, which itself rejects). Uniform across all 4 LIMIT names, purely value-based
    /// (arbitrary leading zeros neither save an out-of-range value nor doom an in-range one).
    #[cfg(has_generated_regex_parser)]
    #[test]
    fn regex_limit_value_range_rejects_at_the_grammar_layer_pcre2_faithfully() {
        // Out-of-range LIMIT values: REJECT (err 160). u32 max (4294967295) itself is out of range;
        // leading zeros do not save an out-of-range value; all 4 LIMIT names share the bound.
        for pattern in [
            "(*LIMIT_HEAP=4294967290)a",           // boundary + 1
            "(*LIMIT_HEAP=4294967295)a",           // u32 max — still rejects
            "(*LIMIT_HEAP=4294967296)a",           // u32 max + 1
            "(*LIMIT_HEAP=99999999999999999999)a", // gross overflow
            "(*LIMIT_HEAP=00000000004294967290)a", // leading zeros do not save it
            "(*LIMIT_MATCH=4294967290)a",
            "(*LIMIT_DEPTH=4294967290)a",
            "(*LIMIT_RECURSION=4294967290)a",
        ] {
            assert_eq!(
                parse_sample("regex", pattern),
                Some(false),
                "out-of-range LIMIT value must reject: {pattern}"
            );
        }
        // In-range LIMIT values (VALUE <= 4294967289, arbitrary leading zeros, all-zeros = 0): ACCEPT.
        for pattern in [
            "(*LIMIT_HEAP=0)a",
            "(*LIMIT_HEAP=500)a",
            "(*LIMIT_HEAP=500)", // empty body after the start option
            "(*LIMIT_HEAP=65535)a",
            "(*LIMIT_HEAP=4294967289)a",           // boundary — last accepted value
            "(*LIMIT_HEAP=00000000004294967289)a", // leading zeros, in range
            "(*LIMIT_HEAP=00000000000000000000)a", // all zeros = value 0
            "(*LIMIT_MATCH=4294967289)a",
            "(*LIMIT_DEPTH=4294967289)a",
            "(*LIMIT_RECURSION=4294967289)a",
        ] {
            assert_eq!(
                parse_sample("regex", pattern),
                Some(true),
                "in-range LIMIT value must accept: {pattern}"
            );
        }
        // The released `{separator:"=", value:"<digits>"}` STRING carrier is byte-identical: the
        // value is the raw digit text (leading zeros preserved), captured via `$text` over the
        // wrapper `directive_limit_value_body`.
        let ast = super::parse_sample_ast_json("regex", "(*LIMIT_HEAP=00700)a")
            .expect("regex registered")
            .expect("accepts (*LIMIT_HEAP=00700)a")
            .to_string();
        assert!(
            ast.contains("\"value\":\"00700\""),
            "(*LIMIT_HEAP=00700) must carry the raw digit string \"00700\": {ast}"
        );
        // The bound applies in BOTH profiles (directive_limit_value_body is profile-shared — the
        // `.3.14`/`.3.20` known-name-shape precedent; relaxed re-admits only UNKNOWN names).
        for pattern in ["(*LIMIT_HEAP=4294967290)a", "(*LIMIT_HEAP=4294967289)a"] {
            let verdict = super::parse_sample_detail_with_profile("regex", pattern, Some("relaxed"))
                .expect("regex registered");
            assert_eq!(
                verdict.is_ok(),
                parse_sample("regex", pattern) == Some(true),
                "relaxed must agree with the default profile on the LIMIT range: {pattern}"
            );
        }
    }

    /// REGEX-PCRE2-FIDELITY.3.18 (ledger REGEX-0092/0093/0094): the PCRE2 counted-quantifier
    /// BRACE TOKENIZATION model is grammar-encoded. Oracle (`pcre2test` 10.47, 2026-07-08,
    /// blank-line-separated + hex-pattern cells): a syntactically-valid quantifier brace —
    /// digits with SPACES/TABS anywhere inside, forms `{n}` `{n,}` `{n,m}` `{,m}` — is ALWAYS
    /// a quantifier, then (1) any bound VALUE > 65535 is err 105, (2) min > max is err 104,
    /// (3) a non-repeatable position is err 109; only a NON-quantifier-shaped brace (`{}`,
    /// `{,}`, `{a}`, a \n/\f/\r/\v inside, unterminated) is a literal `{`. Encoded via the
    /// value-structural `quant_bound_number` (≤ 65535 by construction) + the
    /// `literal_open_brace` negative-lookahead guard; the min>max ORDER rule stays
    /// validator-owned (space+tab-exact since `.3.18`).
    #[cfg(has_generated_regex_parser)]
    #[test]
    fn regex_counted_quantifier_brace_model_rejects_at_the_grammar_layer_pcre2_faithfully() {
        // Value class (err 105): the bound VALUE exceeds 65535 — including the former
        // u32-overflow hole (REGEX-0092: `a{4294967296}` skipped every check) and
        // quantifier-whitespace spellings.
        for pattern in [
            "a{65536}",
            "a{4294967296}",
            "a{99999999999999999999}",
            "a{065536}",
            "a{ 65536 }",
            "a{\t65536\t}",
            "a{0,65536}",
            "a{65536,}",
            "a{,65536}",
            "a{ ,65536}",
            "a{, 65536}",
            "a{65536,65537}",
        ] {
            assert_eq!(
                parse_sample("regex", pattern),
                Some(false),
                "out-of-range quantifier bound must reject: {pattern}"
            );
        }
        // Order class (err 104): min > max — validator-owned, now tab-aware (REGEX-0094).
        for pattern in ["a{5,2}", "a{ 5 , 2 }", "a{\t5\t,\t2\t}", "(){5,2}", "^{5,2}$"] {
            assert_eq!(
                parse_sample("regex", pattern),
                Some(false),
                "out-of-order quantifier bounds must reject: {pattern}"
            );
        }
        // Position class (err 109, REGEX-0093): a valid-syntax brace at a non-repeatable
        // position can neither quantify (nothing precedes) nor fall back to literal.
        for pattern in ["{2,5}", "x|{2,5}", "a{2}{3}", "{2}", "({2,5})", "a|{0}"] {
            assert_eq!(
                parse_sample("regex", pattern),
                Some(false),
                "non-repeatable-position quantifier brace must reject: {pattern}"
            );
        }
        // Quantifier accepts: in-range values, leading zeros (the VALUE decides, not the
        // digit count), space/tab whitespace anywhere inside (all oracle-verified).
        for pattern in [
            "a{2,5}",
            "a{ 2 , 5 }",
            "a{\t2\t,\t5\t}",
            "a{0}",
            "a{65535}",
            "a{0000000000065535}",
            "a{065535}",
            "a{0,65535}",
            "a{65535,}",
            "a{,65535}",
            "a{,5}",
            "a{2, }",
            "a{ 3 }",
            "(a){2,5}",
            "a{2,5}?",
            "a{2,5}+",
        ] {
            assert_eq!(
                parse_sample("regex", pattern),
                Some(true),
                "valid counted quantifier must accept: {pattern}"
            );
        }
        // Literal-brace accepts: NON-quantifier-shaped braces stay literal — including a
        // \n inside the brace (REGEX-0094 rejects-valid: `a{\n5,2\n}` was wrongly
        // order-rejected; PCRE2 compiles it clean as literals).
        for pattern in [
            "a{}",
            "a{,}",
            "a{ }",
            "{a}",
            "{}",
            "a{1,2,3}b",
            "a{65536",
            "a{2,",
            "a{\n2\n}",
            "a{\n5,2\n}",
            "a{\n65536\n}",
            "X{12ABC}",
            "a{(?#XYZ),2}",
        ] {
            assert_eq!(
                parse_sample("regex", pattern),
                Some(true),
                "non-quantifier-shaped brace must stay literal: {pattern}"
            );
        }
        // The typed-int carrier is preserved: leading zeros collapse to the VALUE (the
        // `@transform` span parse on `quant_bound_number`), exactly as `digits` surfaced it.
        let ast = super::parse_sample_ast_json("regex", "a{065535}")
            .expect("regex registered")
            .expect("accepts a{065535}")
            .to_string();
        assert!(
            ast.contains("\"min\":65535") && ast.contains("\"max\":65535"),
            "a{{065535}} must carry the typed int bounds 65535: {ast}"
        );
        // The brace model applies in BOTH profiles (the quantifier rules and the guard are
        // profile-shared; the validator's order rule runs unconditionally).
        for pattern in ["a{65536}", "{2,5}", "a{2}{3}", "a{5,2}", "a{2,5}", "a{}"] {
            let verdict = super::parse_sample_detail_with_profile("regex", pattern, Some("relaxed"))
                .expect("regex registered");
            assert_eq!(
                verdict.is_ok(),
                parse_sample("regex", pattern) == Some(true),
                "relaxed must agree with the default profile on the brace model: {pattern}"
            );
        }
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

    /// GRAMMAR-WELLFORMED.H.10.2.2 — the memoization × coverage-record
    /// composition gap: a rule subtree first parsed inside a rolled-back
    /// speculation and then memo-HIT on the committed path must still appear
    /// in the witness record. `\Q\A\E*` routes through
    /// `piece_quoted_run_quantified`, whose `quoted_run_inner_piece*`
    /// speculation parses `quoted_literal_char` at the `\A` position and
    /// fails on the `!"\E"` lookahead (the rollback truncates the coverage
    /// stack while the memo keeps the success); the trailing
    /// `quoted_literal_char` slot then memo-hits at the same position.
    /// Pre-fix, the accepted parse's record lacked the whole subtree
    /// (`letter_no_upper_e` stayed UNKNOWN in regex cert-coverage); post-fix
    /// the cached coverage delta is replayed on the hit.
    #[cfg(has_generated_regex_parser)]
    #[test]
    fn regex_parse_and_cover_replays_coverage_on_memo_hits() {
        let (parsed, covered) = super::parse_and_cover_regex("\\Q\\A\\E*", None, None);
        assert!(parsed, "\\Q\\A\\E* must parse");
        for rule in [
            "quoted_literal_char",
            "quoted_literal_escaped_char",
            "quoted_literal_escape_tail",
            "letter_no_upper_e",
        ] {
            assert!(
                covered.contains(rule),
                "accepted parse of \\Q\\A\\E* must witness '{}' (memo-hit coverage replay)",
                rule
            );
        }

        // The class-context analogue: `class_item`'s first-tried `class_range`
        // parses `quoted_class_range_atom → quoted_class_literal_char` at the
        // `\A` position, fails on the missing `-`, and rolls back; the
        // `quoted_class_literal` alternative then memo-hits at that position.
        let (parsed, covered) = super::parse_and_cover_regex("[]\\Q\\A\\E]", None, None);
        assert!(parsed, "[]\\Q\\A\\E] must parse");
        assert!(
            covered.contains("quoted_class_literal_escaped_char"),
            "accepted parse of []\\Q\\A\\E] must witness 'quoted_class_literal_escaped_char' (memo-hit coverage replay)"
        );
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
        // Contract migrated to the typed-AST era in RTL-FE-CLOSURE.3 (0.1.0 -> 0.2.0);
        // this stale lib-side assertion (the gate's probe binary already asserts 0.2.0)
        // is realigned to reality here, in the RTL-FE-CLOSURE.10 contract-surface wave.
        assert_eq!(contract.contract_version, "0.2.0");
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
