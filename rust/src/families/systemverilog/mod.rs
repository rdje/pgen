//! The SystemVerilog family's COMPONENT (`ENGINE-UNIVERSAL-SERVICES.112` slice 2, `.113`). This
//! module may name its own family and nothing else; the engine reaches it only through `HOOKS`.
//!
//! `.113`: the family's preprocessor ([`preprocessor`]) and its IEEE 1800-2017 §22.14 keyword
//! tables ([`keyword_versions`]) live here, beside the family, and are published to the engine
//! as ONE neutral [`PreprocessingStage`] (`HOOKS.preprocessing`): the engine hands it a
//! [`StageRequest`] (the unit, the include dirs, the defines, the parse profile, this stage's own
//! options by key) and gets the engine's [`StageOutput`] back. The option KEYS and the deprecated
//! pre-`.113` CLI spellings are declared below — the engine spells none of them.
//!
//! `SV-EXH-PROOF.3.3.4.b.6.2.37.2` — the standard-library preload: before a parse begins, the
//! facts of `<repo_root>/parser_libs/sv_<profile>_std/package/std.facts.json` (checked in by
//! `.37.1`) are pushed into the parser's semantic runtime state, so the LRM §G.2 `std::`
//! predefined classes (process / semaphore / mailbox) are visible to the `has_fact(type_name, X)`
//! checks for the duration of the parse without user code declaring them. A missing bundle is a
//! no-op (single-file behaviour); a read error surfaces as `Err`.

use super::{FamilyHooks, ParseTarget};
use crate::preprocessing_stage::{
    LegacyFlag, LegacyTarget, PreprocessingStage, StageOptionSpec, StageOutput, StageRequest,
};
use anyhow::Context;

pub mod keyword_versions;
pub mod preprocessor;

use preprocessor::{
    ConditionalExprPolicy, ConditionalSymbolPolicy, IncludePathPolicy, MacroRedefinitionPolicy,
    PlainStringSubstitutionPolicy, PredefinedMacro, SvPreprocessorConfig, parse_strict_warning_codes,
    preprocess_systemverilog_file, preprocess_systemverilog_unit,
};

/// The stage's options (`--stage-opt KEY=VALUE`); the universal context — `--include-dir`,
/// `--define`, `--unit-file`, `--profile` — is the engine's request, not an option.
const STAGE_OPTIONS: &[StageOptionSpec] = &[
    StageOptionSpec { key: "include_max_depth", values: "N", default: "64", doc: "the deepest `include nesting accepted before the stage refuses" },
    StageOptionSpec { key: "include_path_policy", values: "allow_absolute|relative_only", default: "allow_absolute", doc: "whether an absolute `include path is honoured" },
    StageOptionSpec { key: "macro_redefine_policy", values: "allow|warn|error", default: "allow", doc: "what a `define of an already-defined macro does" },
    StageOptionSpec { key: "conditional_symbol_policy", values: "assume_false_silent|assume_false_warn|error", default: "assume_false_silent", doc: "what an `ifdef of a never-defined symbol does" },
    StageOptionSpec { key: "conditional_expr_policy", values: "identifier_only|identifier_or_defined", default: "identifier_or_defined", doc: "what an `elsif condition may be" },
    StageOptionSpec { key: "plain_string_substitution", values: "standard|paste_delimited|all", default: "standard", doc: "what a formal's name inside a plain \"…\" string in macro text means (SVPP-EXPANSION.5 (k))" },
    StageOptionSpec { key: "strict_warning_codes", values: "all|none|CODE[,CODE…]", default: "none (or PGEN_SVPP_STRICT_WARNING_CODES)", doc: "warning codes promoted to errors" },
];

/// The pre-`.113` spellings, accepted one release: the family's own prefix on the universal
/// flags, and its policy flags as stage options. `--sv-disallow-macro-redefine` was an override
/// of the policy, so it maps to a FIXED value the engine appends last.
const LEGACY_FLAGS: &[LegacyFlag] = &[
    LegacyFlag { flag: "--sv-include-dir", takes_value: true, to: LegacyTarget::Flag("--include-dir") },
    LegacyFlag { flag: "--sv-define", takes_value: true, to: LegacyTarget::Flag("--define") },
    LegacyFlag { flag: "--sv-unit-file", takes_value: true, to: LegacyTarget::Flag("--unit-file") },
    LegacyFlag { flag: "--sv-keyword-profile", takes_value: true, to: LegacyTarget::Flag("--profile") },
    LegacyFlag { flag: "--sv-source-map-json", takes_value: true, to: LegacyTarget::Flag("--source-map-json") },
    LegacyFlag { flag: "--sv-event-log-json", takes_value: true, to: LegacyTarget::Flag("--event-log-json") },
    LegacyFlag { flag: "--sv-diagnostics-json", takes_value: true, to: LegacyTarget::Flag("--diagnostics-json") },
    LegacyFlag { flag: "--sv-include-max-depth", takes_value: true, to: LegacyTarget::StageOpt { key: "include_max_depth", fixed_value: None } },
    LegacyFlag { flag: "--sv-include-path-policy", takes_value: true, to: LegacyTarget::StageOpt { key: "include_path_policy", fixed_value: None } },
    LegacyFlag { flag: "--sv-macro-redefine-policy", takes_value: true, to: LegacyTarget::StageOpt { key: "macro_redefine_policy", fixed_value: None } },
    LegacyFlag { flag: "--sv-conditional-symbol-policy", takes_value: true, to: LegacyTarget::StageOpt { key: "conditional_symbol_policy", fixed_value: None } },
    LegacyFlag { flag: "--sv-conditional-expr-policy", takes_value: true, to: LegacyTarget::StageOpt { key: "conditional_expr_policy", fixed_value: None } },
    LegacyFlag { flag: "--sv-plain-string-substitution", takes_value: true, to: LegacyTarget::StageOpt { key: "plain_string_substitution", fixed_value: None } },
    LegacyFlag { flag: "--sv-strict-warning-codes", takes_value: true, to: LegacyTarget::StageOpt { key: "strict_warning_codes", fixed_value: None } },
    LegacyFlag { flag: "--sv-disallow-macro-redefine", takes_value: false, to: LegacyTarget::StageOpt { key: "macro_redefine_policy", fixed_value: Some("error") } },
];

/// The stage: the engine's request → this family's configuration → its outputs, one per file of
/// the unit. A one-file unit takes the one-file entry (`preprocess_systemverilog_file`), a longer
/// one the compilation-unit entry (`SVPP-EXPANSION.2` (b)) — exactly what `ast_pipeline
/// --preprocess-systemverilog` did before `.113`.
fn run_stage(request: &StageRequest) -> anyhow::Result<Vec<StageOutput>> {
    let defines = request
        .defines
        .iter()
        .map(|raw| PredefinedMacro::parse(raw).with_context(|| format!("invalid --define '{}'", raw)))
        .collect::<anyhow::Result<Vec<_>>>()?;
    let mut config = SvPreprocessorConfig {
        include_dirs: request.include_dirs.clone(),
        defines,
        parse_profile: request.parse_profile.clone(),
        ..SvPreprocessorConfig::default()
    };
    let mut strict_warning_codes: Option<String> = None;
    for (key, value) in &request.options {
        match key.as_str() {
            "include_max_depth" => {
                config.max_include_depth = value
                    .parse::<usize>()
                    .map_err(|_| anyhow::anyhow!("invalid include_max_depth '{}'; expected an integer", value))?;
            }
            "include_path_policy" => {
                config.include_path_policy = IncludePathPolicy::parse(value).ok_or_else(|| {
                    anyhow::anyhow!("invalid include_path_policy '{}'; expected allow_absolute|relative_only", value)
                })?;
            }
            "macro_redefine_policy" => {
                config.macro_redefinition_policy = MacroRedefinitionPolicy::parse(value).ok_or_else(|| {
                    anyhow::anyhow!("invalid macro_redefine_policy '{}'; expected allow|warn|error", value)
                })?;
            }
            "conditional_symbol_policy" => {
                config.conditional_symbol_policy = ConditionalSymbolPolicy::parse(value).ok_or_else(|| {
                    anyhow::anyhow!("invalid conditional_symbol_policy '{}'; expected assume_false_silent|assume_false_warn|error", value)
                })?;
            }
            "conditional_expr_policy" => {
                config.conditional_expr_policy = ConditionalExprPolicy::parse(value).ok_or_else(|| {
                    anyhow::anyhow!("invalid conditional_expr_policy '{}'; expected identifier_only|identifier_or_defined", value)
                })?;
            }
            "plain_string_substitution" => {
                config.plain_string_substitution = PlainStringSubstitutionPolicy::parse(value).ok_or_else(|| {
                    anyhow::anyhow!("invalid plain_string_substitution '{}'; expected standard|paste_delimited|all", value)
                })?;
            }
            "strict_warning_codes" => strict_warning_codes = Some(value.clone()),
            other => anyhow::bail!("unknown stage option '{}'", other),
        }
    }
    let strict_raw = strict_warning_codes
        .or_else(|| std::env::var("PGEN_SVPP_STRICT_WARNING_CODES").ok())
        .unwrap_or_else(|| "none".to_string());
    config.strict_warning_codes = parse_strict_warning_codes(&strict_raw);
    let outputs = match request.unit.as_slice() {
        [] => anyhow::bail!("the compilation unit is empty: nothing to preprocess"),
        [single] => vec![preprocess_systemverilog_file(single, &config)?],
        unit => preprocess_systemverilog_unit(unit, &config)?,
    };
    Ok(outputs.into_iter().map(StageOutput::from).collect())
}

/// This family's preprocessing stage, as the engine sees it.
pub static PREPROCESSING: PreprocessingStage = PreprocessingStage {
    options: STAGE_OPTIONS,
    legacy: LEGACY_FLAGS,
    run: run_stage,
};

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

/// This family's hooks: the stdlib preload before every parse, and the preprocessing stage.
pub const HOOKS: FamilyHooks = FamilyHooks {
    pre_parse: preload_stdlib,
    preprocessing: Some(&PREPROCESSING),
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
