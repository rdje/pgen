//! The regex family's hooks (`ENGINE-UNIVERSAL-SERVICES.112` slice 2). This module may name its
//! own family and nothing else; the engine reaches it only through `HOOKS`.
//!
//! Two facts of this family the registry's template used to carry as a hand-copied helper set:
//! (1) every regex parse runs on a dedicated-stack worker thread of [`STACK_BYTES`] — the regex
//! grammar's recursion depth on nested groups exceeds a default thread stack; (2) an ACCEPTED parse
//! is then held to the PCRE2 compile contract (`regex_compile_validation`,
//! `REGEX-PCRE2-FIDELITY`): a contract failure is an oracle disagreement reported with the
//! family's own message and no invented byte offset (`CORPUS-GRAD-ALL.2.0`).
//!
//! Slice 3 moved the rest of the family's facts here from the embedding API: the nesting ceiling
//! (`PGEN-RGX-0085`), the inline threshold below which a parse needs no dedicated stack, the
//! downstream contract identity (`CONTRACT`) and the convenience entry points (`embed`).

use super::{FamilyContract, FamilyHooks};
use crate::regex_compile_validation::validate_regex_compile_contract;

/// The dedicated worker stack a deep regex parse runs on (the value the registry and the embedding
/// API have always used).
pub const STACK_BYTES: usize = 64 * 1024 * 1024;
/// `PGEN-RGX-0085`: the parenthesis nesting depth PCRE2 and Rust-`regex` both refuse beyond; an
/// input past it is refused BEFORE a parser is built.
pub const MAX_NESTING_DEPTH: usize = 250;
/// Up to this grouping depth a parse runs inline on the caller's stack (RGX's hot path: no thread
/// per small pattern); deeper inputs take the dedicated worker.
pub const INLINE_DEPTH_THRESHOLD: usize = 4;

/// This family's downstream contract identity (`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`
/// carries the same pair; the ledger drift test holds them equal).
pub const INTEGRATION_CONTRACT_VERSION: &str = "1.1.110";
pub const PARSER_RELEASE_VERSION: &str = "1.1.107";
pub const AST_DUMP_SCHEMA_VERSION: u32 = 1;

pub fn contract() -> Option<FamilyContract> {
    Some(FamilyContract {
        integration_contract_version: INTEGRATION_CONTRACT_VERSION.to_string(),
        parser_release_version: PARSER_RELEASE_VERSION.to_string(),
        ast_dump_schema_version: AST_DUMP_SCHEMA_VERSION,
        generated_backend_required_feature: "generated_parsers".to_string(),
        generated_backend_required_artifact: "generated/regex_parser.rs".to_string(),
        generated_backend_env_override: "PGEN_REGEX_PARSER_PATH".to_string(),
        frontend_json_artifact: "generated/regex.json".to_string(),
        frontend_json_role: "PGEN-generated frontend normalization and provenance artifact; not a stable downstream runtime input or AST contract".to_string(),
    })
}

/// The byte offset at which `input` first nests deeper than [`MAX_NESTING_DEPTH`], if it does
/// (escapes and character classes open nothing, as the regex lexer reads them).
pub fn nesting_violation_offset(input: &str) -> Option<usize> {
    let bytes = input.as_bytes();
    let mut depth: usize = 0;
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => {
                i += 2;
                continue;
            }
            b'[' => {
                i += 1;
                if i < bytes.len() && bytes[i] == b'^' {
                    i += 1;
                }
                if i < bytes.len() && bytes[i] == b']' {
                    i += 1; // leading `]` is a literal class member (PCRE2)
                }
                while i < bytes.len() && bytes[i] != b']' {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
            }
            b'(' => {
                depth += 1;
                if depth > MAX_NESTING_DEPTH {
                    return Some(i);
                }
            }
            b')' => {
                depth = depth.saturating_sub(1);
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// The maximum grouping nesting of `input` (parentheses and classes alike, escapes skipped) — the
/// cheap estimate that decides inline vs dedicated stack.
pub fn estimate_max_grouping_nesting(input: &str) -> usize {
    let mut depth = 0usize;
    let mut max_depth = 0usize;
    let mut bytes = input.bytes();
    while let Some(b) = bytes.next() {
        match b {
            b'\\' => {
                let _ = bytes.next();
            }
            b'(' | b'[' => {
                depth += 1;
                if depth > max_depth {
                    max_depth = depth;
                }
            }
            b')' | b']' => {
                depth = depth.saturating_sub(1);
            }
            _ => {}
        }
    }
    max_depth
}

fn pre_check(sample: &str) -> Result<(), String> {
    match nesting_violation_offset(sample) {
        Some(offset) => Err(format!(
            "regex parenthesis nesting exceeds the maximum supported depth of {} at position {} (PGEN-RGX-0085 ceiling; PCRE2 / Rust-`regex` parity)",
            MAX_NESTING_DEPTH, offset
        )),
        None => Ok(()),
    }
}

fn dedicated_stack(sample: &str) -> Option<usize> {
    if estimate_max_grouping_nesting(sample) <= INLINE_DEPTH_THRESHOLD { None } else { Some(STACK_BYTES) }
}

fn post_parse(sample: &str) -> Result<(), String> {
    validate_regex_compile_contract(sample).map_err(|err| err.message)
}

/// This family's hooks: the dedicated stack for every parse, the PCRE2 compile contract after an
/// accepted one.
pub const HOOKS: FamilyHooks = FamilyHooks {
    pre_check,
    dedicated_stack,
    post_parse,
    contract,
    ..FamilyHooks::NONE
};

/// The convenience entry points this family has always published through the embedding API
/// (`parse_regex_default*`), moved beside the family that owns them (slice 3). Each is one line
/// over the generic, registry-derived API.
pub mod embed {
    use crate::embedding_api::{self as api, AstDumpOptions, GrammarAstDumpOutcome, GrammarParseOutcome, ParseDiagnostic, ParseLimits};
    const FAMILY: &str = "regex";
    const PROFILE: &str = "regex_default";
    pub fn parse_regex_default(input: &str) -> GrammarParseOutcome { api::parse_grammar_profile_named_typed(FAMILY, PROFILE, input, &ParseLimits::default()) }
    pub fn parse_regex_default_with_limits(input: &str, limits: &ParseLimits) -> GrammarParseOutcome { api::parse_grammar_profile_named_typed(FAMILY, PROFILE, input, limits) }
    pub fn parse_regex_default_result(input: &str) -> Result<(), ParseDiagnostic> { api::parse_grammar_profile_named_with_limits_result(FAMILY, PROFILE, input, &ParseLimits::default()) }
    pub fn parse_regex_default_with_limits_result(input: &str, limits: &ParseLimits) -> Result<(), ParseDiagnostic> { api::parse_grammar_profile_named_with_limits_result(FAMILY, PROFILE, input, limits) }
    pub fn parse_regex_default_ast_dump(input: &str, options: &AstDumpOptions) -> GrammarAstDumpOutcome { api::parse_grammar_profile_ast_dump_named_typed(FAMILY, PROFILE, input, &ParseLimits::default(), options) }
    pub fn parse_regex_default_ast_dump_with_limits(input: &str, limits: &ParseLimits, options: &AstDumpOptions) -> GrammarAstDumpOutcome { api::parse_grammar_profile_ast_dump_named_typed(FAMILY, PROFILE, input, limits, options) }
}
