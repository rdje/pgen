//! The SystemVerilog family's COMPONENT (`ENGINE-UNIVERSAL-SERVICES.112` slice 2, `.113`). This
//! module may name its own family and nothing else; the engine reaches it only through `HOOKS`.
//!
//! `.113`: the family's preprocessor ([`preprocessor`]) and its IEEE 1800-2017 §22.14 keyword
//! tables ([`keyword_versions`]) live here, beside the family, and are published to the engine
//! as ONE neutral [`PreprocessingStage`] (`HOOKS.preprocessing`): the engine hands it a
//! [`StageRequest`] (the unit, the include dirs, the symbols, the parse profile, this stage's own
//! options by key) and gets the engine's [`StageOutput`] back. The option KEYS and the deprecated
//! pre-`.113` CLI spellings are declared below — the engine spells none of them.
//!
//! `SV-EXH-PROOF.3.3.4.b.6.2.37.2` — the standard-library preload: before a parse begins, the
//! facts of the profile's bundle (`parser_libs/sv_<profile>_std/package/std.facts.json`, checked in
//! by `.37.1`) are pushed into the parser's semantic runtime state, so the LRM §G.2 `std::`
//! predefined classes (process / semaphore / mailbox) are visible to the `has_fact(type_name, X)`
//! checks for the duration of the parse without user code declaring them.
//!
//! ⛔ `ENGINE-UNIVERSAL-SERVICES.73` — **the bundle is EMBEDDED in the binary, never resolved from a
//! path at run time.** The previous form joined `env!("CARGO_MANIFEST_DIR")` with `../parser_libs`
//! at run time and returned `Ok(())` when the directory was not there — so a binary built at path
//! A and run after the tree moved to path B parsed with an EMPTY stdlib, silently, and REJECTED
//! LRM-legal input: `mailbox m;` at module scope, and the shipped-corpus rows
//! `sv-tests/tests/chapter-15/15.4--mailbox-{blocking,non-blocking}.sv` (both `must_accept`), went
//! `parse_full passed` → `Parser did not consume full input` with the bundle hidden, on the same
//! binary. Now the two tracked bundles are baked in with `include_str!` (the compile-time path is
//! used exactly once, at build time, which is the harmless class `.73`'s census identified), an
//! operator may point [`STDLIB_OVERRIDE_ENV`] at a directory holding `<bundle_dir>/package/std.facts.json`
//! instead, and a bundle missing THERE is a refusal that names the knob, the bundle and the path —
//! there is no silent `Ok(())` path left.

use super::{FamilyHooks, ParseTarget};
use crate::ast_pipeline::SemanticFactRecord;
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
/// `--symbol`, `--unit-file`, `--profile` — is the engine's request, not an option.
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
    LegacyFlag { flag: "--sv-define", takes_value: true, to: LegacyTarget::Flag("--symbol") },
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
        .symbols
        .iter()
        .map(|raw| PredefinedMacro::parse(raw).with_context(|| format!("invalid --symbol '{}'", raw)))
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
/// other (or unrequested) profile rides the `sv_2017` bundle, the corpus baseline. The opt-in
/// tolerance profile (`SV-CORPUS-GRAD.43`) is `sv_2023` plus a closed list of spellings, so it rides
/// `sv_2023`'s bundle.
fn stdlib_bundle_dir(normalized_profile: Option<&str>) -> &'static str {
    match normalized_profile {
        Some("sv_2023") | Some("sv_2023_tolerant") => "sv_2023_std",
        _ => "sv_2017_std",
    }
}

/// `ENGINE-UNIVERSAL-SERVICES.73` — the two tracked bundles, baked into the binary at build time.
/// `include_str!` re-runs the build when the tracked file moves, and the path is consumed HERE, at
/// compile time, and nowhere at run time.
const STDLIB_SV_2017_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../parser_libs/sv_2017_std/package/std.facts.json"
));
const STDLIB_SV_2023_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../parser_libs/sv_2023_std/package/std.facts.json"
));

/// The one operator override for the standard-library bundle: a directory holding
/// `<bundle_dir>/package/std.facts.json` for the profile being parsed (the tracked `parser_libs/`
/// is one such directory). Unset, the bundle embedded in this binary is used. Set to a directory
/// that does not hold the bundle, the parse is REFUSED by name — the operator asked for something
/// this binary cannot honour, and a silent fallback is the defect `.73` removed.
pub const STDLIB_OVERRIDE_ENV: &str = "PGEN_PARSER_LIBS_DIR";

fn embedded_stdlib_json(normalized_profile: Option<&str>) -> &'static str {
    match stdlib_bundle_dir(normalized_profile) {
        "sv_2023_std" => STDLIB_SV_2023_JSON,
        _ => STDLIB_SV_2017_JSON,
    }
}

/// The stdlib fact records for a profile: from `override_dir` when one is named (a bundle missing
/// THERE is an error), else from the bundle embedded in this binary.
pub(crate) fn resolve_stdlib_records(
    normalized_profile: Option<&str>,
    override_dir: Option<&std::path::Path>,
) -> Result<Vec<SemanticFactRecord>, String> {
    use crate::ast_pipeline::library::{read_artifact, read_artifact_text};
    let bundle = stdlib_bundle_dir(normalized_profile);
    if let Some(root) = override_dir {
        let dir = root.join(bundle);
        return read_artifact(&dir, "package", "std").map_err(|e| {
            format!(
                "SV stdlib preload REFUSED: {STDLIB_OVERRIDE_ENV}={} names no readable bundle \
                 for the {bundle} profile bundle (expected {}/package/std.facts.json): {:?}. \
                 Unset {STDLIB_OVERRIDE_ENV} to use the bundle embedded in this binary.",
                root.display(),
                dir.display(),
                e
            )
        });
    }
    read_artifact_text(embedded_stdlib_json(normalized_profile), "package", "std").map_err(|e| {
        format!(
            "SV stdlib preload failed: the {bundle} bundle EMBEDDED in this binary is unreadable \
             ({:?}) — the tracked parser_libs source was malformed when this binary was built",
            e
        )
    })
}

fn preload_stdlib(parser: &mut dyn ParseTarget, normalized_profile: Option<&str>) -> Result<(), String> {
    let override_dir = std::env::var_os(STDLIB_OVERRIDE_ENV).map(std::path::PathBuf::from);
    let records = resolve_stdlib_records(normalized_profile, override_dir.as_deref())?;
    let state = parser.semantic_runtime_state_mut();
    for record in records {
        state.push_fact_record(record);
    }
    Ok(())
}

#[cfg(test)]
mod stdlib_tests {
    use super::{STDLIB_OVERRIDE_ENV, resolve_stdlib_records, stdlib_bundle_dir};
    use crate::ast_pipeline::library::read_artifact;
    use std::path::{Path, PathBuf};

    fn tracked_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("parser_libs")
    }

    /// `.73` — the embedded bundle IS the tracked source, record for record, for every profile
    /// spelling the family maps to a bundle; a divergence here is a binary shipping a different
    /// stdlib than the tree it was built from.
    #[test]
    fn embedded_bundle_equals_the_tracked_source_for_every_profile() {
        for profile in [None, Some("sv_2017"), Some("sv_2023"), Some("verilog_2005"), Some("sv_2023_tolerant")] {
            let embedded = resolve_stdlib_records(profile, None).expect("the embedded bundle parses");
            let from_tree = read_artifact(&tracked_root().join(stdlib_bundle_dir(profile)), "package", "std")
                .expect("the tracked bundle reads");
            assert_eq!(embedded, from_tree, "profile {profile:?}: embedded bundle != tracked source");
            assert!(!embedded.is_empty(), "profile {profile:?}: the stdlib bundle carries no facts");
        }
    }

    /// `SV-CORPUS-GRAD.43` — the opt-in tolerance profile is `sv_2023` plus a closed list of spellings, so it
    /// preloads `sv_2023`'s standard library, record for record. Falling through to the default arm would hand
    /// it the 2017 bundle: a profile that accepts 1800-2023 text with 1800-2017's `std` facts.
    #[test]
    fn the_tolerant_profile_rides_the_sv_2023_bundle() {
        assert_eq!(stdlib_bundle_dir(Some("sv_2023_tolerant")), stdlib_bundle_dir(Some("sv_2023")));
        assert_eq!(
            resolve_stdlib_records(Some("sv_2023_tolerant"), None).expect("the tolerant profile's bundle parses"),
            resolve_stdlib_records(Some("sv_2023"), None).expect("the sv_2023 bundle parses"),
        );
        assert_ne!(stdlib_bundle_dir(Some("sv_2023_tolerant")), stdlib_bundle_dir(Some("sv_2017")));
    }

    /// `.73` — a named override is honoured, and a bundle missing there is refused BY NAME rather
    /// than degrading to an empty stdlib (the defect this leaf removed).
    #[test]
    fn an_override_directory_is_honoured_and_a_bundle_missing_there_is_refused_by_name() {
        let from_override = resolve_stdlib_records(Some("sv_2017"), Some(&tracked_root()))
            .expect("the tracked parser_libs/ is a valid override root");
        assert_eq!(from_override, resolve_stdlib_records(Some("sv_2017"), None).unwrap());

        let empty = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/eus73_empty_override");
        std::fs::create_dir_all(&empty).unwrap();
        let err = resolve_stdlib_records(Some("sv_2017"), Some(&empty))
            .expect_err("a named override without the bundle must refuse, never fall back");
        for needle in [STDLIB_OVERRIDE_ENV, "sv_2017_std", "REFUSED", "std.facts.json"] {
            assert!(err.contains(needle), "the refusal must name `{needle}`: {err}");
        }
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
