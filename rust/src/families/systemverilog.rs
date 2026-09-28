//! The SystemVerilog family's hooks (`ENGINE-UNIVERSAL-SERVICES.112` slice 2). This module may
//! name its own family and nothing else; the engine reaches it only through `HOOKS`.
//!
//! `SV-EXH-PROOF.3.3.4.b.6.2.37.2` — the standard-library preload: before a parse begins, the
//! facts of `<repo_root>/parser_libs/sv_<profile>_std/package/std.facts.json` (checked in by
//! `.37.1`) are pushed into the parser's semantic runtime state, so the LRM §G.2 `std::`
//! predefined classes (process / semaphore / mailbox) are visible to the `has_fact(type_name, X)`
//! checks for the duration of the parse without user code declaring them. A missing bundle is a
//! no-op (single-file behaviour); a read error surfaces as `Err`.

use super::{FamilyHooks, ParseTarget};

/// The per-profile bundle directory — a fact of THIS family: `sv_2023` has its own bundle, every
/// other (or unrequested) profile rides the `sv_2017` bundle, the corpus baseline.
fn stdlib_bundle_dir(normalized_profile: Option<&str>) -> &'static str {
    match normalized_profile {
        Some("sv_2023") => "sv_2023_std",
        _ => "sv_2017_std",
    }
}

fn preload_stdlib(parser: &mut dyn ParseTarget, normalized_profile: Option<&str>) -> Result<(), String> {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let Some(repo_root) = manifest_dir.parent() else {
        return Ok(());
    };
    let stdlib_path = repo_root
        .join("parser_libs")
        .join(stdlib_bundle_dir(normalized_profile));
    if !stdlib_path.is_dir() {
        return Ok(());
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

/// This family's hooks: the stdlib preload before every parse; nothing else.
pub const HOOKS: FamilyHooks = FamilyHooks {
    pre_parse: preload_stdlib,
    ..FamilyHooks::NONE
};

/// The convenience entry points this family has always published through the embedding API
/// (`parse_systemverilog_2017*` / `_2023*`), moved beside the family that owns them (slice 3).
/// Each is one line over the generic, registry-derived API.
pub mod embed {
    use crate::embedding_api::{self as api, AstDumpOptions, GrammarAstDumpOutcome, GrammarParseOutcome, ParseDiagnostic, ParseLimits};
    const FAMILY: &str = "systemverilog";
    macro_rules! shims {
        ($profile:literal, $p:ident, $pl:ident, $pr:ident, $plr:ident, $pa:ident, $pal:ident) => {
            pub fn $p(input: &str) -> GrammarParseOutcome { api::parse_grammar_profile_named_typed(FAMILY, $profile, input, &ParseLimits::default()) }
            pub fn $pl(input: &str, limits: &ParseLimits) -> GrammarParseOutcome { api::parse_grammar_profile_named_typed(FAMILY, $profile, input, limits) }
            pub fn $pr(input: &str) -> Result<(), ParseDiagnostic> { api::parse_grammar_profile_named_with_limits_result(FAMILY, $profile, input, &ParseLimits::default()) }
            pub fn $plr(input: &str, limits: &ParseLimits) -> Result<(), ParseDiagnostic> { api::parse_grammar_profile_named_with_limits_result(FAMILY, $profile, input, limits) }
            pub fn $pa(input: &str, options: &AstDumpOptions) -> GrammarAstDumpOutcome { api::parse_grammar_profile_ast_dump_named_typed(FAMILY, $profile, input, &ParseLimits::default(), options) }
            pub fn $pal(input: &str, limits: &ParseLimits, options: &AstDumpOptions) -> GrammarAstDumpOutcome { api::parse_grammar_profile_ast_dump_named_typed(FAMILY, $profile, input, limits, options) }
        };
    }
    shims!("sv_2017", parse_systemverilog_2017, parse_systemverilog_2017_with_limits, parse_systemverilog_2017_result, parse_systemverilog_2017_with_limits_result, parse_systemverilog_2017_ast_dump, parse_systemverilog_2017_ast_dump_with_limits);
    shims!("sv_2023", parse_systemverilog_2023, parse_systemverilog_2023_with_limits, parse_systemverilog_2023_result, parse_systemverilog_2023_with_limits_result, parse_systemverilog_2023_ast_dump, parse_systemverilog_2023_ast_dump_with_limits);
}
