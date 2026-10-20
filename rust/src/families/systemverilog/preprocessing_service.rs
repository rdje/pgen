//! `SVPP-EXPANSION.9.2` — the SystemVerilog PREPROCESSING SERVICE's typed surface over the
//! existing expansion stage: the contract
//! `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_EXPANSION_CONTRACT.md` (`1.0`) names these
//! items; this module is where they live until the director's placement answer adds a second
//! path (additive — the names are the contract, the module path is not).
//!
//! What a downstream project gets here that the neutral stage socket
//! ([`crate::preprocessing_stage::stage_for`] → `execute`) does not:
//! - a PUBLISHED output type ([`PreprocessedOutput`], the family's own, with the typed event kind);
//! - a TYPED refusal ([`PreprocessRefusal`]): the kind, the code (the same `E_SVPP_…` vocabulary
//!   the diagnostics use), the file and line where the refusing site knew them, and the message
//!   byte-for-byte as the stage has always printed it — attached AT THE REFUSING SITE
//!   ([`Refusal`] in the stage), never parsed back out of the message;
//! - a FILE PROVIDER ([`FileProvider`], the engine's neutral trait): `path → bytes` for the unit's
//!   files and every `` `include ``; the default reads the disk, [`MemoryFileProvider`] serves
//!   memory, and [`preprocess_str`] is the in-memory one-file form built on it;
//! - the two stability axes as constants: [`PREPROCESSING_SERVICE_VERSION`] (this service's own
//!   release line) and [`PREPROCESSED_OUTPUT_SCHEMA_VERSION`] (the shape of the five outputs; the
//!   probe's `--emit-source-map` carrier carries it).
//!
//! One engine: [`preprocess`] runs EXACTLY the code path the SystemVerilog parser's own front end
//! runs (`parseability_probe … --preprocess`, the pipeline lanes) — the same configuration
//! derivation from the same declared option table, the same per-file / per-unit entry — so the
//! text and the verdict a consumer receives are the ones PGEN's corpus lanes measure.
//!
//! This module is a COMPONENT of the SystemVerilog family (`ENGINE-NEUTRALITY`): it names its own
//! family and the engine's neutral socket, nothing else.

use super::preprocessor::{PreprocessedOutput, Refusal, RefusalKind, RefusalSite};
use super::{PREPROCESSING, run_unit};
pub use crate::preprocessing_stage::{
    DiskFileProvider, FileProvider, MemoryFileProvider, OUTPUT_SCHEMA_VERSION, ProviderError,
    SourceLocation, SourceMapEntry,
};
use crate::preprocessing_stage::StageRequest;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// The service's own release line (the contract's first stability axis) — independent of the
/// `systemverilog_preprocessor` parser's release and of the SystemVerilog parser's. Any change a
/// consumer could observe in the five outputs for an unchanged input and context moves it.
pub const PREPROCESSING_SERVICE_VERSION: &str = "1.0.0";

/// The preprocessed-output schema (the contract's second stability axis): the shape of the five
/// outputs. `1` is the serde form of [`PreprocessedOutput`] / `StageOutput` as it has been since
/// `ENGINE-UNIVERSAL-SERVICES.113`; `2` (`.9.3`) adds the macro-expansion chain to every
/// source-map entry. Carried by the probe's `--emit-source-map` JSON carrier.
pub const PREPROCESSED_OUTPUT_SCHEMA_VERSION: u32 = OUTPUT_SCHEMA_VERSION;

/// The context — a VALUE, never an implicit filesystem (the contract's "The context" table).
#[derive(Clone)]
pub struct PreprocessContext {
    /// The edition profile the EXPANDED text will be parsed under (`sv_2017`, `sv_2023`,
    /// `verilog_2005`, or an alias the svpp contract lists); `None` is the unprofiled reading.
    pub profile: Option<String>,
    /// Predefined macros, `NAME` or `NAME=VALUE`, in order — seeded as `` `define `` before the
    /// first line (the socket's `symbols`, the CLIs' `--symbol` / `--define`).
    pub defines: Vec<String>,
    /// Include search directories, in order (`--include-dir`).
    pub include_dirs: Vec<PathBuf>,
    /// The stage's DECLARED options, `(key, value)` in order (`--stage-opt KEY=VALUE`); an
    /// undeclared key is refused by name ([`RefusalKind::InvalidContext`]).
    pub options: Vec<(String, String)>,
    /// Where the unit's files and every `` `include `` come from. The default reads the disk.
    pub file_provider: Arc<dyn FileProvider>,
}

impl Default for PreprocessContext {
    fn default() -> Self {
        Self {
            profile: None,
            defines: Vec::new(),
            include_dirs: Vec::new(),
            options: Vec::new(),
            file_provider: Arc::new(DiskFileProvider),
        }
    }
}

impl fmt::Debug for PreprocessContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreprocessContext")
            .field("profile", &self.profile)
            .field("defines", &self.defines)
            .field("include_dirs", &self.include_dirs)
            .field("options", &self.options)
            .field("file_provider", &"<dyn FileProvider>")
            .finish()
    }
}

impl PreprocessContext {
    /// A context whose files come from `provider` (an in-memory store, a virtual filesystem, a
    /// build system's cache) instead of the disk.
    pub fn with_file_provider(mut self, provider: Arc<dyn FileProvider>) -> Self {
        self.file_provider = provider;
        self
    }
}

/// The code of a refusal kind — the same `E_SVPP_…` vocabulary the diagnostics use. A kind that
/// carries a diagnostic's own code ([`RefusalKind::PromotedWarning`]: a `W_SVPP_…` promoted by
/// `strict_warning_codes`) reports that code on the refusal instead of this table's.
///
/// ⛔ A code is never reused with a different meaning; a new code is ADDITIVE (the contract's
/// versioning rules). `scripts/check_svpp_expansion_contract_currency.sh` holds the contract's
/// refusal-code list equal to this table.
pub fn refusal_code(kind: RefusalKind) -> &'static str {
    match kind {
        RefusalKind::IncludeNotFound => "E_SVPP_INCLUDE_NOT_FOUND",
        RefusalKind::IncludeNesting => "E_SVPP_INCLUDE_NESTING",
        RefusalKind::IncludeMalformed => "E_SVPP_INCLUDE_MALFORMED",
        RefusalKind::SourceUnreadable => "E_SVPP_SOURCE_UNREADABLE",
        RefusalKind::UndefinedMacro => "E_SVPP_UNDEFINED_MACRO",
        RefusalKind::UnresolvedBacktick => "E_SVPP_UNRESOLVED_BACKTICK",
        RefusalKind::MissingMacroArgument => "E_SVPP_MISSING_MACRO_ARGUMENT",
        RefusalKind::ExtraMacroArgument => "E_SVPP_EXTRA_MACRO_ARGUMENT",
        RefusalKind::RecursiveMacro => "E_SVPP_RECURSIVE_MACRO",
        RefusalKind::MacroRedefined => "E_SVPP_MACRO_REDEFINED",
        RefusalKind::DefineMalformed => "E_SVPP_DEFINE_MALFORMED",
        RefusalKind::UndefinedConditionalSymbol => "E_SVPP_UNDEFINED_CONDITIONAL_SYMBOL",
        RefusalKind::UnterminatedConditional => "E_SVPP_UNTERMINATED_CONDITIONAL",
        RefusalKind::UnterminatedProtectedEnvelope => "E_SVPP_UNTERMINATED_PROTECTED_ENVELOPE",
        RefusalKind::KeywordsRegion => "E_SVPP_KEYWORDS_REGION",
        RefusalKind::PromotedWarning => "E_SVPP_PROMOTED_WARNING",
        RefusalKind::InvalidContext => "E_SVPP_INVALID_CONTEXT",
        RefusalKind::Other => "E_SVPP_REFUSED",
    }
}

/// A refusal of the service: the expansion did not complete, the parser never sees the text.
///
/// `message` is byte-for-byte what the stage has always printed (`preprocess_failed …` on the
/// probe, the `Err` of the socket) — a consumer that matched on text keeps working; one that
/// wants structure reads `kind` / `code` / `file` / `line`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreprocessRefusal {
    pub kind: RefusalKind,
    /// [`refusal_code`] of `kind`, or the promoted warning's own `W_SVPP_…` code.
    pub code: String,
    /// The file the refusing site knew, when it knew one (an include's path as resolved, the unit
    /// file as canonicalized by the provider).
    pub file: Option<String>,
    /// The line the refusing site knew, when it knew one.
    pub line: Option<usize>,
    /// The full message, context chain included (`{:#}` of the stage's error).
    pub message: String,
}

impl PreprocessRefusal {
    /// Classify the stage's error without reading its text: the kind and code come from the
    /// [`Refusal`] at the root of the chain when a refusing site put one there, else from the
    /// outermost [`RefusalSite`] context (a site that wrapped a plain error), else `fallback`
    /// (the neutral socket's own refusal of an undeclared option key); the file and line come
    /// from the outermost [`RefusalSite`] when there is one (it knows the line of the directive),
    /// else from the root [`Refusal`].
    fn from_stage_error(err: anyhow::Error, fallback: RefusalKind) -> Self {
        let message = format!("{:#}", err);
        let site = err.downcast_ref::<RefusalSite>().cloned();
        let root = err.downcast_ref::<Refusal>().cloned();
        let kind = root
            .as_ref()
            .map(|refusal| refusal.kind)
            .or_else(|| site.as_ref().map(|site| site.kind))
            .unwrap_or(fallback);
        let code = root
            .as_ref()
            .and_then(|refusal| refusal.code.clone())
            .unwrap_or_else(|| refusal_code(kind).to_string());
        let (file, line) = match (&site, &root) {
            (Some(site), _) => (Some(site.file.clone()), site.line),
            (None, Some(refusal)) => (refusal.file.clone(), refusal.line),
            (None, None) => (None, None),
        };
        Self {
            kind,
            code,
            file,
            line,
            message,
        }
    }
}

impl From<anyhow::Error> for PreprocessRefusal {
    fn from(err: anyhow::Error) -> Self {
        Self::from_stage_error(err, RefusalKind::Other)
    }
}

impl fmt::Display for PreprocessRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for PreprocessRefusal {}

/// The stage's request for a typed call: the unit and the context's four value members.
fn request_for(unit: &[PathBuf], ctx: &PreprocessContext) -> StageRequest {
    StageRequest {
        unit: unit.to_vec(),
        include_dirs: ctx.include_dirs.clone(),
        symbols: ctx.defines.clone(),
        parse_profile: ctx.profile.clone(),
        options: ctx.options.clone(),
    }
}

/// The one code path: the declared option table checked (the socket's `execute` does the same
/// first), then the family's [`run_unit`] — the configuration derived exactly as the socket
/// derives it, the per-file or per-unit entry chosen exactly as the socket chooses it.
fn run_typed(
    request: &StageRequest,
    provider: Arc<dyn FileProvider>,
) -> Result<Vec<PreprocessedOutput>, PreprocessRefusal> {
    PREPROCESSING
        .check_option_keys(request)
        .map_err(|err| PreprocessRefusal::from_stage_error(err, RefusalKind::InvalidContext))?;
    run_unit(request, provider).map_err(PreprocessRefusal::from)
}

/// PGEN expands: the ordered compilation unit (earlier files first; their macro tables and open
/// `` `begin_keywords `` regions carry forward, IEEE 1800-2017 §22.5.1) under `ctx`, one
/// [`PreprocessedOutput`] per file, index-aligned with `unit`. `Err` is a refusal: the parser
/// never sees the text.
pub fn preprocess(
    unit: &[&Path],
    ctx: &PreprocessContext,
) -> Result<Vec<PreprocessedOutput>, PreprocessRefusal> {
    let unit: Vec<PathBuf> = unit.iter().map(|path| path.to_path_buf()).collect();
    run_typed(&request_for(&unit, ctx), ctx.file_provider.clone())
}

/// PGEN expands, in memory: `text` is the one file of the unit, named `name` in every output
/// (the source map, the events, the diagnostics, the included-file list); its `` `include ``s
/// are served by `ctx.file_provider` (a [`MemoryFileProvider`] for a fully in-memory call, the
/// disk by default), searched from `name`'s parent and then `ctx.include_dirs`.
pub fn preprocess_str(
    name: &str,
    text: &str,
    ctx: &PreprocessContext,
) -> Result<PreprocessedOutput, PreprocessRefusal> {
    let provider: Arc<dyn FileProvider> = Arc::new(
        MemoryFileProvider::new()
            .with_file(name, text.as_bytes().to_vec())
            .with_fallback(ctx.file_provider.clone()),
    );
    let unit = vec![PathBuf::from(name)];
    let mut outputs = run_typed(&request_for(&unit, ctx), provider)?;
    Ok(outputs
        .pop()
        .expect("one file in the unit produces exactly one output"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preprocessing_stage::stage_for;
    use std::fs;

    /// Under `rust/target/` (project data lives on the project volume, never `/tmp`), one
    /// directory per test, fresh each run — the precedent the stage's own tests set.
    fn scratch(name: &str) -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("svpp_unit_tests")
            .join(format!("service_{}", name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("mkdir repo scratch");
        dir
    }

    // ── the typed call is the socket's call: one engine ────────────────────────────────────

    #[test]
    fn preprocess_on_disk_equals_the_socket_byte_for_byte() {
        let dir = scratch("disk_equals_socket");
        let inc = dir.join("inc.svh");
        let top = dir.join("top.sv");
        fs::write(&inc, "`define FROM_INC 3\n").expect("write inc");
        fs::write(&top, "`include \"inc.svh\"\n`define W 8\nlogic [`W-1:0] a; logic [`FROM_INC:0] b;\n").expect("write top");
        let ctx = PreprocessContext {
            profile: Some("sv_2017".to_string()),
            defines: vec!["SYNTHESIS".to_string(), "DEPTH=4".to_string()],
            ..PreprocessContext::default()
        };
        let typed = preprocess(&[top.as_path()], &ctx).expect("typed call");
        let stage = stage_for("systemverilog").expect("declared");
        let socket = stage
            .execute(&StageRequest {
                unit: vec![top.clone()],
                include_dirs: vec![],
                symbols: ctx.defines.clone(),
                parse_profile: ctx.profile.clone(),
                options: vec![],
            })
            .expect("socket call");
        assert_eq!(typed.len(), 1);
        let typed_json = serde_json::to_string(&crate::preprocessing_stage::StageOutput::from(typed[0].clone())).unwrap();
        let socket_json = serde_json::to_string(&socket[0]).unwrap();
        assert_eq!(typed_json, socket_json, "the typed API and the socket produce the same five outputs");
        assert!(typed[0].text.contains("logic [8-1:0] a; logic [3:0] b;"));
    }

    // ── RED-first: an in-memory include is a refusal without a provider, text with one ──────

    #[test]
    fn preprocess_str_serves_an_include_from_memory_and_refuses_without_it() {
        let ctx = PreprocessContext::default().with_file_provider(Arc::new(
            MemoryFileProvider::new(),
        ));
        let refused = preprocess_str("top.sv", "`include \"inc.svh\"\nlogic a;\n", &ctx)
            .expect_err("no provider entry: the include is not found");
        assert_eq!(refused.kind, RefusalKind::IncludeNotFound);
        assert_eq!(refused.code, "E_SVPP_INCLUDE_NOT_FOUND");
        assert!(refused.message.contains("include 'inc.svh' was not found in search paths"), "{}", refused.message);

        let ctx = PreprocessContext::default().with_file_provider(Arc::new(
            MemoryFileProvider::new().with_file("inc.svh", b"`define FROM_MEMORY 1\nlogic from_inc;\n".to_vec()),
        ));
        let output = preprocess_str("top.sv", "`include \"inc.svh\"\nlogic [`FROM_MEMORY:0] a;\n", &ctx)
            .expect("the include is served from memory");
        assert_eq!(output.text, "logic from_inc;\nlogic [1:0] a;\n");
        // `included_files` is every file the expansion READ, the unit file first (the stage's
        // contract since `.3`); both are named exactly as the provider spelled them.
        assert_eq!(output.included_files, vec!["top.sv".to_string(), "inc.svh".to_string()]);
        assert!(output.source_map.iter().any(|entry| entry.source.file == "inc.svh"));
        assert!(output.source_map.iter().any(|entry| entry.source.file == "top.sv"));
        let origin = output
            .source_map
            .iter()
            .find(|entry| entry.output_start == 0)
            .map(|entry| (entry.source.file.as_str(), entry.source.line))
            .expect("the first byte is mapped");
        assert_eq!(origin, ("inc.svh", 2));
    }

    #[test]
    fn preprocess_str_includes_fall_back_to_the_context_provider() {
        let dir = scratch("str_fallback_to_disk");
        fs::write(dir.join("disk.svh"), "logic from_disk;\n").expect("write inc");
        let ctx = PreprocessContext {
            include_dirs: vec![dir.clone()],
            ..PreprocessContext::default()
        };
        let output = preprocess_str("mem.sv", "`include \"disk.svh\"\nlogic a;\n", &ctx).expect("disk fallback");
        assert_eq!(output.text, "logic from_disk;\nlogic a;\n");
        assert_eq!(output.included_files.len(), 2, "the in-memory unit file, then the disk include");
        assert_eq!(output.included_files[0], "mem.sv");
        assert!(output.included_files[1].ends_with("disk.svh"), "the disk provider canonicalizes: {}", output.included_files[1]);
    }

    // ── the typed refusal: kind, code, file, line, the message unchanged ───────────────────

    #[test]
    fn an_undefined_macro_is_a_typed_refusal_with_its_site() {
        let ctx = PreprocessContext::default().with_file_provider(Arc::new(MemoryFileProvider::new()));
        let refused = preprocess_str("u.sv", "module m;\n  initial `NOT_DEFINED;\nendmodule\n", &ctx)
            .expect_err("undefined_macro_policy=error refuses");
        assert_eq!(refused.kind, RefusalKind::UndefinedMacro);
        assert_eq!(refused.code, "E_SVPP_UNDEFINED_MACRO");
        assert_eq!(refused.file.as_deref(), Some("u.sv"));
        assert_eq!(refused.line, Some(2));
        assert!(refused.message.contains("`NOT_DEFINED is neither a defined macro nor a compiler directive at u.sv:2"), "{}", refused.message);
        assert_eq!(refused.to_string(), refused.message);
    }

    #[test]
    fn a_refusal_that_is_a_coded_diagnostic_carries_that_code() {
        let ctx = PreprocessContext {
            options: vec![("macro_redefine_policy".to_string(), "error".to_string())],
            ..PreprocessContext::default()
        }
        .with_file_provider(Arc::new(MemoryFileProvider::new()));
        let refused = preprocess_str("r.sv", "`define A 1\n`define A 2\n", &ctx).expect_err("redefinition refused");
        assert_eq!(refused.kind, RefusalKind::MacroRedefined);
        assert_eq!(refused.code, "E_SVPP_MACRO_REDEFINED");
        assert_eq!(refused.line, Some(2));

        let ctx = PreprocessContext {
            options: vec![("strict_warning_codes".to_string(), "W_SVPP_MACRO_REDEFINED".to_string()), ("macro_redefine_policy".to_string(), "warn".to_string())],
            ..PreprocessContext::default()
        }
        .with_file_provider(Arc::new(MemoryFileProvider::new()));
        let refused = preprocess_str("p.sv", "`define A 1\n`define A 2\n", &ctx).expect_err("promoted warning refused");
        assert_eq!(refused.kind, RefusalKind::PromotedWarning);
        assert_eq!(refused.code, "W_SVPP_MACRO_REDEFINED", "a promoted warning reports its own code");
    }

    #[test]
    fn an_undeclared_option_is_an_invalid_context_refusal_naming_the_key() {
        let ctx = PreprocessContext {
            options: vec![("no_such_option".to_string(), "1".to_string())],
            ..PreprocessContext::default()
        };
        let refused = preprocess_str("o.sv", "logic a;\n", &ctx).expect_err("undeclared key refused");
        assert_eq!(refused.kind, RefusalKind::InvalidContext);
        assert_eq!(refused.code, "E_SVPP_INVALID_CONTEXT");
        assert!(refused.message.contains("unknown stage option 'no_such_option'"), "{}", refused.message);

        let ctx = PreprocessContext {
            options: vec![("include_max_depth".to_string(), "deep".to_string())],
            ..PreprocessContext::default()
        };
        let refused = preprocess_str("o.sv", "logic a;\n", &ctx).expect_err("invalid value refused");
        assert_eq!(refused.kind, RefusalKind::InvalidContext);
        assert!(refused.message.contains("invalid include_max_depth 'deep'"), "{}", refused.message);

        let refused = preprocess(&[], &PreprocessContext::default()).expect_err("an empty unit is refused");
        assert_eq!(refused.kind, RefusalKind::InvalidContext);
    }

    #[test]
    fn a_missing_unit_file_is_a_source_unreadable_refusal_with_the_disk_text() {
        let dir = scratch("missing_unit_file");
        let missing = dir.join("absent.sv");
        let refused = preprocess(&[missing.as_path()], &PreprocessContext::default()).expect_err("absent file");
        assert_eq!(refused.kind, RefusalKind::SourceUnreadable);
        assert_eq!(refused.code, "E_SVPP_SOURCE_UNREADABLE");
        assert!(
            refused.message.starts_with(&format!("failed to canonicalize '{}': ", missing.display())),
            "the disk's own text is kept: {}",
            refused.message
        );
    }

    #[test]
    fn a_unit_carries_its_table_forward_and_names_the_failing_file() {
        let dir = scratch("unit_forward");
        let first = dir.join("a.sv");
        let second = dir.join("b.sv");
        fs::write(&first, "`define FROM_A 2\n").expect("write a");
        fs::write(&second, "logic [`FROM_A:0] b;\n").expect("write b");
        let outputs = preprocess(&[first.as_path(), second.as_path()], &PreprocessContext::default()).expect("unit");
        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs[1].text, "logic [2:0] b;\n");

        fs::write(&second, "`ifdef FROM_A\nlogic unterminated;\n").expect("rewrite b");
        let refused = preprocess(&[first.as_path(), second.as_path()], &PreprocessContext::default()).expect_err("unterminated");
        assert_eq!(refused.kind, RefusalKind::UnterminatedConditional);
        assert!(refused.message.starts_with("compilation unit file 2 of 2 ("), "{}", refused.message);
    }

    // ── the two axes ───────────────────────────────────────────────────────────────────────

    #[test]
    fn the_stability_axes_are_the_contracts() {
        assert_eq!(PREPROCESSING_SERVICE_VERSION, "1.0.0");
        assert_eq!(PREPROCESSED_OUTPUT_SCHEMA_VERSION, 1);
        assert_eq!(PREPROCESSING.service_version, PREPROCESSING_SERVICE_VERSION);
        // every kind has a code, and no two kinds share one
        let kinds = RefusalKind::ALL;
        let mut codes: Vec<&str> = kinds.iter().map(|kind| refusal_code(*kind)).collect();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), kinds.len(), "one code per refusal kind");
        assert!(codes.iter().all(|code| code.starts_with("E_SVPP_")));
    }
}
