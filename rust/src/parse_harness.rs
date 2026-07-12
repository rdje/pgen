//! PARSE-HARNESS.3 — the compile-and-run harness (approach 2), authoritative BY CONSTRUCTION.
//!
//! # What this is
//!
//! [`compile_and_parse`] takes an **arbitrary** grammar (`grammars/foo.ebnf`) and an input string,
//! runs the **real PGEN codegen**, compiles the emitted parser as a **throwaway external crate**, runs
//! it on the input, and returns the [`ParseOutcome`] — the accept/reject verdict, the
//! `furthest_position` on reject, and the **typed AST** on accept. It needs **no registry wiring and no
//! `pgen` rebuild**: the probe grammar never becomes a compiled-in parser, so this is the self-contained
//! path that approach 3 (the [`crate::parser_registry`] `scratch` slot) is not.
//!
//! # Why it is trustworthy ("authoritative by construction")
//!
//! The harness does **not** re-implement parsing. It invokes the shipped `ast_pipeline` codegen and the
//! shipped `pgen` runtime, exactly as `make focus_<grammar>` does — the *only* new (trusted) surface is
//! this plumbing (codegen invocation + throwaway-crate synthesis + I/O marshalling). That surface is
//! covered by an integration test that reproduces a *registered* grammar's known verdict + typed AST
//! through the harness (see `tests` at the bottom of this file). Because it runs the real generated
//! parser + real runtime, its verdict/AST are byte-for-byte what the shipped parser produces.
//!
//! # The load-bearing feasibility fact (spike-verified, PARSE-HARNESS.3)
//!
//! A generated parser's source hard-codes `use crate::ast_pipeline::{…}`. Inside `pgen` that resolves
//! against the whole crate (including `pub(crate)` items); an **external** crate sees only `pub` items.
//! A static audit of every `generated/*_parser.rs` shows they reach **only** `crate::ast_pipeline::*`
//! (22 distinct symbols, all `pub`) plus the externs `regex` / `rustc_hash` / `serde_json`. So a
//! throwaway crate that (a) path-depends on `pgen`, (b) adds the single shim `use pgen::ast_pipeline;`
//! at its crate root — which makes the generated file's `crate::ast_pipeline::…` paths resolve to
//! `pgen`'s — and (c) `include!`s the generated parser, **compiles and runs**. [`ParseNode`] derives
//! `serde::Serialize`, so the throwaway emits the *same* typed-AST JSON the registry's
//! `parse_node_to_json` (`serde_json::to_value(node)`) does.
//!
//! [`ParseNode`]: crate::ast_pipeline::ParseNode
//!
//! # Cost & reuse
//!
//! The dominant cost is the throwaway crate's **first** `cargo build`, which compiles the `pgen` lib as
//! a dependency (seconds→a couple of minutes cold). Subsequent probes that reuse the same
//! [`CompileAndParseOptions::workdir`] recompile only the tiny probe bin (`pgen` stays cached in the
//! isolated target dir), so a batch driver — e.g. the PARSE-HARNESS.5 differential-equivalence gate,
//! for which this harness is the CI oracle — pays the `pgen` compile once. The isolated
//! `CARGO_TARGET_DIR` also means a nested `cargo build` here never contends on an outer `cargo test`'s
//! `pgen` target lock.

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::ast_pipeline::TraceLevel;

/// The verdict + typed AST produced by running an arbitrary grammar's REAL generated parser on an input.
///
/// `accepted == true` iff the parser fully consumed the input. On accept, [`ast_json`](Self::ast_json)
/// is `Some(<typed AST>)` and [`error`](Self::error) is `None`; on reject the two swap.
/// [`furthest_position`](Self::furthest_position) is always the deepest input byte any branch reached
/// (the A2.2/A2.3-grade reject locus — see [`crate::parser_registry`]).
#[derive(Debug, Clone, PartialEq)]
pub struct ParseOutcome {
    /// `true` iff the parser consumed the entire input (a full parse).
    pub accepted: bool,
    /// The deepest input byte any branch reached (even if it later backtracked). On reject this is the
    /// real defect locus; on accept it is typically the input length.
    pub furthest_position: usize,
    /// The reject message on `!accepted`; `None` on accept.
    pub error: Option<String>,
    /// The typed AST (`serde_json::to_value(&ParseNode)`) on accept; `None` on reject. This is
    /// byte-for-byte the shape [`crate::parser_registry::parse_sample_ast_json`] returns for a
    /// registered grammar.
    pub ast_json: Option<serde_json::Value>,
}

/// Knobs for [`compile_and_parse`]. All optional; [`Default`] mirrors the standard PGEN working tree.
#[derive(Debug, Clone)]
pub struct CompileAndParseOptions {
    /// Parse from an alternate start symbol (`parse_full_from(entry)`); `None` = the grammar's canonical
    /// entry (`parse_full()`). The harness is entry-rule-agnostic, exactly like the scratch slot.
    pub entry_rule: Option<String>,
    /// The `ast_pipeline` binary used for codegen. It MUST be built with `--features ebnf_dual_run` so it
    /// can read a `.ebnf` directly. Default: `<pgen>/target/debug/ast_pipeline` (the standard tree's
    /// build carries `ebnf_dual_run`).
    pub ast_pipeline_bin: Option<PathBuf>,
    /// The `pgen` crate directory the throwaway crate path-depends on. Default: this crate's
    /// `CARGO_MANIFEST_DIR` (i.e. the `pgen` crate being compiled).
    pub pgen_manifest_dir: Option<PathBuf>,
    /// Working directory for the probe artifacts + isolated cargo target dir. Default: a fresh temp dir
    /// under the system temp root. Reuse one dir across probes to keep the `pgen` build cached.
    pub workdir: Option<PathBuf>,
    /// Keep the working directory after the call (for debugging). Default `false` → removed on success.
    /// A FAILED call always keeps the workdir so it can be inspected, regardless of this flag.
    pub keep_workdir: bool,
    /// Pass `--eliminate-left-recursion` to codegen (default `true` — mirrors the shipped `RUST_GENERATOR`
    /// recipe). Only set `false` for a deliberate no-LR-elimination experiment.
    pub eliminate_left_recursion: bool,
    /// `PROFILE-ALIAS.2`: request a dialect profile SPELLING before parsing — the probe calls
    /// `set_grammar_profile(Some(<spelling>))` on the compiled parser, so the artifact's own
    /// resolution (declared `@profile_alias` spellings, unknown-value pass-through) is what gates
    /// `@profiles` rules. `None` (default) = the artifact's constructor posture (the declared
    /// `@default_profile` if any, else unset/permissive).
    pub requested_profile: Option<String>,
}

impl Default for CompileAndParseOptions {
    fn default() -> Self {
        Self {
            entry_rule: None,
            ast_pipeline_bin: None,
            pgen_manifest_dir: None,
            workdir: None,
            keep_workdir: false,
            eliminate_left_recursion: true,
            requested_profile: None,
        }
    }
}

/// A structured failure of the compile-and-run plumbing. Every variant carries enough context to act on
/// it (the failing step + captured tool output + the fix hint where applicable). A *parse rejection* is
/// NOT an error — it is a successful [`ParseOutcome`] with `accepted == false`.
#[derive(Debug)]
pub enum HarnessError {
    /// A filesystem/process I/O failure while driving the harness.
    Io(std::io::Error),
    /// The grammar file does not exist / is not readable.
    GrammarNotFound(PathBuf),
    /// The `ast_pipeline` codegen binary was not found at the resolved path.
    ToolNotFound { path: PathBuf, hint: String },
    /// Codegen (`ast_pipeline <grammar>.ebnf --generate-parser …`) exited non-zero or produced no parser.
    Codegen { status: Option<i32>, stderr: String },
    /// The emitted parser source did not contain the expected single `pub struct <Name>Parser<'input>`.
    StructNameNotFound { parser_src: PathBuf },
    /// The throwaway crate failed to compile (captured `cargo build` stderr).
    Compile { status: Option<i32>, stderr: String },
    /// The compiled probe binary crashed / exited non-zero at run time.
    Run { status: Option<i32>, stderr: String },
    /// The probe binary's sentinel-wrapped JSON was missing or unparseable.
    OutputParse { detail: String, stdout: String },
}

impl fmt::Display for HarnessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HarnessError::Io(e) => write!(f, "parse-harness I/O error: {e}"),
            HarnessError::GrammarNotFound(p) => {
                write!(f, "parse-harness: grammar file not found: {}", p.display())
            }
            HarnessError::ToolNotFound { path, hint } => write!(
                f,
                "parse-harness: ast_pipeline codegen binary not found at {} — {hint}",
                path.display()
            ),
            HarnessError::Codegen { status, stderr } => write!(
                f,
                "parse-harness: codegen failed (exit {}). Ensure the ast_pipeline binary was built with \
                 --features ebnf_dual_run so it can read a .ebnf directly.\n{stderr}",
                fmt_status(status)
            ),
            HarnessError::StructNameNotFound { parser_src } => write!(
                f,
                "parse-harness: could not find a unique `pub struct <Name>Parser<'input>` in the emitted \
                 parser {}",
                parser_src.display()
            ),
            HarnessError::Compile { status, stderr } => write!(
                f,
                "parse-harness: throwaway crate failed to compile (exit {}).\n{stderr}",
                fmt_status(status)
            ),
            HarnessError::Run { status, stderr } => write!(
                f,
                "parse-harness: probe binary failed at run time (exit {}).\n{stderr}",
                fmt_status(status)
            ),
            HarnessError::OutputParse { detail, stdout } => write!(
                f,
                "parse-harness: could not parse the probe's outcome JSON ({detail}).\n--- stdout ---\n{stdout}"
            ),
        }
    }
}

impl std::error::Error for HarnessError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            HarnessError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for HarnessError {
    fn from(e: std::io::Error) -> Self {
        HarnessError::Io(e)
    }
}

fn fmt_status(status: &Option<i32>) -> String {
    match status {
        Some(c) => c.to_string(),
        None => "signal".to_string(),
    }
}

/// The sentinel lines that fence the probe binary's outcome JSON on stdout, so codegen/tooling noise can
/// never be confused for the result. Kept in lockstep with the emitted `main.rs` template below.
const JSON_BEGIN: &str = "<<<PGEN_PARSE_HARNESS_JSON";
const JSON_END: &str = "PGEN_PARSE_HARNESS_JSON>>>";

/// Compile an arbitrary grammar's REAL parser and run it on `input`, returning the [`ParseOutcome`].
///
/// This is approach 2 of the PARSE-HARNESS tree — authoritative **by construction** (it runs the shipped
/// codegen + runtime). See the module docs for the trust argument and cost/reuse notes.
///
/// # Errors
/// Returns [`HarnessError`] for *plumbing* failures (missing tool, codegen/compile/run failure,
/// unparseable output). A grammar that simply *rejects* the input is a successful `Ok(ParseOutcome)`
/// with `accepted == false` — not an error.
pub fn compile_and_parse(
    grammar_ebnf: &Path,
    input: &str,
    opts: &CompileAndParseOptions,
) -> Result<ParseOutcome, HarnessError> {
    crate::pgen_trace_low!(
        "parse-harness: compile_and_parse grammar={} entry={:?} input_len={}",
        grammar_ebnf.display(),
        opts.entry_rule,
        input.len()
    );

    if !grammar_ebnf.is_file() {
        return Err(HarnessError::GrammarNotFound(grammar_ebnf.to_path_buf()));
    }
    let grammar_ebnf = std::fs::canonicalize(grammar_ebnf)?;

    let manifest_dir = opts
        .pgen_manifest_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")));

    let ast_pipeline_bin = opts
        .ast_pipeline_bin
        .clone()
        .unwrap_or_else(|| manifest_dir.join("target/debug/ast_pipeline"));
    if !ast_pipeline_bin.is_file() {
        return Err(HarnessError::ToolNotFound {
            path: ast_pipeline_bin,
            hint: "build it with `cd rust && cargo build --features \"generated_parsers ebnf_dual_run\" \
                   --bin ast_pipeline`, or set CompileAndParseOptions::ast_pipeline_bin"
                .to_string(),
        });
    }

    // Working dir: caller-provided (reused → warm cache) or a fresh, unique temp dir.
    let (workdir, owns_workdir) = match &opts.workdir {
        Some(dir) => (dir.clone(), false),
        None => (fresh_temp_dir()?, true),
    };
    std::fs::create_dir_all(&workdir)?;

    let result = run_in_workdir(&workdir, &grammar_ebnf, &ast_pipeline_bin, &manifest_dir, input, opts);

    // Cleanup policy: remove a harness-owned workdir on SUCCESS unless keep_workdir; always keep it on
    // failure (for debugging) and never remove a caller-provided dir.
    match &result {
        Ok(_) if owns_workdir && !opts.keep_workdir => {
            let _ = std::fs::remove_dir_all(&workdir);
        }
        _ => {
            crate::pgen_trace_low!("parse-harness: workdir retained at {}", workdir.display());
        }
    }
    result
}

fn run_in_workdir(
    workdir: &Path,
    grammar_ebnf: &Path,
    ast_pipeline_bin: &Path,
    manifest_dir: &Path,
    input: &str,
    opts: &CompileAndParseOptions,
) -> Result<ParseOutcome, HarnessError> {
    let crate_dir = workdir.join("probe_crate");
    let src_dir = crate_dir.join("src");
    std::fs::create_dir_all(&src_dir)?;
    let parser_src = crate_dir.join("probe_parser.rs");
    let target_dir = crate_dir.join("target");

    // ── Step 1: codegen — the REAL ast_pipeline, one-step .ebnf → parser.rs (mode 3), exactly the shipped
    //    recipe minus the `--debug`/`--trace` logging flags (verified byte-identical output). The only
    //    difference from `make focus_<grammar>` is the embedded diagnostic output-path label, which is
    //    inert (never appears in the typed AST).
    let mut codegen = Command::new(ast_pipeline_bin);
    codegen
        .arg(grammar_ebnf)
        .arg("--generate-parser")
        .arg("-o")
        .arg(&parser_src);
    if opts.eliminate_left_recursion {
        codegen.arg("--eliminate-left-recursion");
    }
    crate::pgen_trace!(TraceLevel::Medium, "parse-harness: codegen -> {}", parser_src.display());
    let codegen_out = codegen.output()?;
    if !codegen_out.status.success() || !parser_src.is_file() {
        return Err(HarnessError::Codegen {
            status: codegen_out.status.code(),
            stderr: String::from_utf8_lossy(&codegen_out.stderr).into_owned(),
        });
    }

    // ── Step 2: discover the parser struct name from the emitted source (exactly one per file — verified
    //    across every shipped grammar). Robust against however codegen derives the grammar name.
    let parser_source = std::fs::read_to_string(&parser_src)?;
    let struct_name = discover_parser_struct_name(&parser_source)
        .ok_or_else(|| HarnessError::StructNameNotFound { parser_src: parser_src.clone() })?;
    crate::pgen_trace!(TraceLevel::Medium, "parse-harness: parser struct = {struct_name}");

    // ── Step 3: synthesize the throwaway external crate (Cargo.toml + main.rs) and the input file.
    let input_file = crate_dir.join("input.txt");
    std::fs::write(&input_file, input.as_bytes())?;
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        render_cargo_toml(manifest_dir),
    )?;
    std::fs::write(
        src_dir.join("main.rs"),
        render_probe_main(&parser_src, &struct_name),
    )?;

    // ── Step 4: compile the throwaway (isolated CARGO_TARGET_DIR → no lock contention with an outer build).
    crate::pgen_trace!(TraceLevel::Medium, "parse-harness: cargo build (isolated target {})", target_dir.display());
    let build_out = Command::new("cargo")
        .arg("build")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(crate_dir.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()?;
    if !build_out.status.success() {
        return Err(HarnessError::Compile {
            status: build_out.status.code(),
            stderr: String::from_utf8_lossy(&build_out.stderr).into_owned(),
        });
    }

    // ── Step 5: run the probe binary on the input file (+ optional alternate entry) and marshal its
    //    sentinel-wrapped outcome JSON back into a ParseOutcome.
    let bin = target_dir.join("debug").join(PROBE_BIN_NAME);
    let mut run = Command::new(&bin);
    run.arg(&input_file);
    // Positional args: [entry_rule] [requested_profile] — an empty entry slot is passed as ""
    // (the probe main filters empties) so a profile-only request keeps its position.
    if opts.entry_rule.is_some() || opts.requested_profile.is_some() {
        run.arg(opts.entry_rule.as_deref().unwrap_or(""));
    }
    if let Some(profile) = &opts.requested_profile {
        run.arg(profile);
    }
    crate::pgen_trace!(TraceLevel::Medium, "parse-harness: run {}", bin.display());
    let run_out = run.output()?;
    if !run_out.status.success() {
        return Err(HarnessError::Run {
            status: run_out.status.code(),
            stderr: String::from_utf8_lossy(&run_out.stderr).into_owned(),
        });
    }
    let stdout = String::from_utf8_lossy(&run_out.stdout).into_owned();
    parse_probe_stdout(&stdout)
}

/// The throwaway crate + its bin are named this so the harness knows the built binary path.
const PROBE_BIN_NAME: &str = "pgen_parse_harness_probe";

/// Find the single `pub struct <Name>Parser<'input> {` a generated parser declares. Returns `None` if
/// there is not exactly one (which would mean the codegen shape changed — a real signal, not silent).
fn discover_parser_struct_name(parser_source: &str) -> Option<String> {
    let mut found: Option<String> = None;
    for line in parser_source.lines() {
        let line = line.trim_start();
        // Match `pub struct XxxParser<'input> {` (the codegen's canonical top-level parser struct).
        if let Some(rest) = line.strip_prefix("pub struct ")
            && let Some(name_end) = rest.find("Parser<'input>")
        {
            let name = &rest[..name_end + "Parser".len()];
            if name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                if found.is_some() {
                    return None; // more than one → ambiguous; refuse rather than guess
                }
                found = Some(name.to_string());
            }
        }
    }
    found
}

fn render_cargo_toml(pgen_manifest_dir: &Path) -> String {
    // `[workspace]` detaches the throwaway from `pgen`'s workspace; default features on `pgen` are enough
    // for `ast_pipeline` + `NoOpLogger` (the generated_parsers feature is deliberately NOT enabled).
    format!(
        r#"[package]
name = "{name}"
version = "0.0.0"
edition = "2024"

[[bin]]
name = "{name}"
path = "src/main.rs"

[dependencies]
pgen = {{ path = {pgen:?} }}
rustc-hash = "2.1"
regex = "1.7"
serde_json = "1.0"

[workspace]
"#,
        name = PROBE_BIN_NAME,
        pgen = pgen_manifest_dir,
    )
}

fn render_probe_main(parser_src: &Path, struct_name: &str) -> String {
    // The shim `use pgen::ast_pipeline;` makes the generated file's `crate::ast_pipeline::…` paths resolve
    // to `pgen`'s module — the load-bearing trick that lets the parser compile OUTSIDE `pgen`.
    format!(
        r#"// GENERATED by pgen::parse_harness (PARSE-HARNESS.3) — throwaway probe, do not edit.
#[allow(unused_imports)]
use pgen::ast_pipeline;

#[allow(warnings)]
mod generated {{
    include!({parser_src:?});
}}

fn main() {{
    let args: Vec<String> = std::env::args().collect();
    let input_path = args
        .get(1)
        .expect("usage: probe <input_file> [entry_rule] [requested_profile]");
    let entry: Option<&str> = args.get(2).map(|s| s.as_str()).filter(|s| !s.is_empty());
    let profile: Option<&str> = args.get(3).map(|s| s.as_str()).filter(|s| !s.is_empty());
    let input = std::fs::read_to_string(input_path).expect("read input file");

    let node_arena = pgen::NodeArena::new();
    let mut parser = generated::{struct_name}::new(&input, &node_arena, Box::new(pgen::NoOpLogger));
    if let Some(requested) = profile {{
        parser.set_grammar_profile(Some(requested));
    }}
    let result = match entry {{
        Some(e) => parser.parse_full_from(e),
        None => parser.parse_full(),
    }};
    let furthest = parser.furthest_position();
    let outcome = match result {{
        Ok(node) => serde_json::json!({{
            "accepted": true,
            "furthest_position": furthest,
            "error": serde_json::Value::Null,
            "ast": serde_json::to_value(&node).expect("serialize typed AST"),
        }}),
        Err(err) => serde_json::json!({{
            "accepted": false,
            "furthest_position": furthest,
            "error": err.to_string(),
            "ast": serde_json::Value::Null,
        }}),
    }};

    println!("{begin}");
    println!("{{}}", serde_json::to_string(&outcome).expect("serialize outcome"));
    println!("{end}");
}}
"#,
        parser_src = parser_src,
        struct_name = struct_name,
        begin = JSON_BEGIN,
        end = JSON_END,
    )
}

fn parse_probe_stdout(stdout: &str) -> Result<ParseOutcome, HarnessError> {
    let begin = stdout.find(JSON_BEGIN).ok_or_else(|| HarnessError::OutputParse {
        detail: "missing begin sentinel".to_string(),
        stdout: stdout.to_string(),
    })?;
    let after_begin = begin + JSON_BEGIN.len();
    let end_rel = stdout[after_begin..]
        .find(JSON_END)
        .ok_or_else(|| HarnessError::OutputParse {
            detail: "missing end sentinel".to_string(),
            stdout: stdout.to_string(),
        })?;
    let json_str = stdout[after_begin..after_begin + end_rel].trim();

    let value: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| HarnessError::OutputParse {
            detail: format!("invalid JSON: {e}"),
            stdout: stdout.to_string(),
        })?;

    let accepted = value.get("accepted").and_then(|v| v.as_bool()).ok_or_else(|| {
        HarnessError::OutputParse {
            detail: "missing `accepted`".to_string(),
            stdout: stdout.to_string(),
        }
    })?;
    let furthest_position = value
        .get("furthest_position")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| HarnessError::OutputParse {
            detail: "missing `furthest_position`".to_string(),
            stdout: stdout.to_string(),
        })? as usize;
    let error = value
        .get("error")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let ast_json = value.get("ast").cloned().filter(|v| !v.is_null());

    Ok(ParseOutcome {
        accepted,
        furthest_position,
        error,
        ast_json,
    })
}

fn fresh_temp_dir() -> Result<PathBuf, HarnessError> {
    // A unique, harness-owned dir under the system temp root. Uniqueness without `Date::now`/randomness
    // (unavailable/nondeterministic here): the process id + a monotonic per-process counter.
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("pgen_parse_harness_{}_{}", std::process::id(), n));
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_parser_struct_name_picks_the_single_top_level_struct() {
        let src = "use crate::ast_pipeline::ParseNode;\npub struct JsonParser<'input> {\n    input: &'input str,\n}\n";
        assert_eq!(discover_parser_struct_name(src).as_deref(), Some("JsonParser"));
    }

    #[test]
    fn discover_parser_struct_name_refuses_when_ambiguous() {
        let src = "pub struct AParser<'input> {}\npub struct BParser<'input> {}\n";
        assert_eq!(discover_parser_struct_name(src), None);
    }

    #[test]
    fn parse_probe_stdout_reads_the_sentinel_wrapped_json() {
        let stdout = format!(
            "cargo noise line\n{JSON_BEGIN}\n{{\"accepted\":true,\"furthest_position\":13,\"error\":null,\"ast\":{{\"rule_name\":\"scratch\"}}}}\n{JSON_END}\ntrailing\n"
        );
        let outcome = parse_probe_stdout(&stdout).expect("parse");
        assert!(outcome.accepted);
        assert_eq!(outcome.furthest_position, 13);
        assert_eq!(outcome.error, None);
        assert_eq!(
            outcome.ast_json,
            Some(serde_json::json!({"rule_name": "scratch"}))
        );
    }

    #[test]
    fn parse_probe_stdout_reads_a_reject() {
        let stdout = format!(
            "{JSON_BEGIN}\n{{\"accepted\":false,\"furthest_position\":7,\"error\":\"Backtrack at position 7\",\"ast\":null}}\n{JSON_END}\n"
        );
        let outcome = parse_probe_stdout(&stdout).expect("parse");
        assert!(!outcome.accepted);
        assert_eq!(outcome.furthest_position, 7);
        assert_eq!(outcome.error.as_deref(), Some("Backtrack at position 7"));
        assert_eq!(outcome.ast_json, None);
    }

    #[test]
    fn parse_probe_stdout_errors_without_sentinels() {
        assert!(parse_probe_stdout("no sentinels here").is_err());
    }

    // ── Authoritative-by-construction integration test (PARSE-HARNESS.3 acceptance) ──────────────────
    //
    // Reproduce a REGISTERED grammar's known verdict + byte-identical typed AST through the compile-and-run
    // harness, proving the plumbing is faithful to the shipped registry path. Gated on the `json` artifact
    // being present (built via `make -C rust focus_json`) so the registry comparison side exists. Uses a
    // persistent workdir under the crate's target dir so the `pgen` dependency compiles once and stays warm.
    #[cfg(all(feature = "generated_parsers", has_generated_json_parser))]
    #[test]
    fn compile_and_run_harness_reproduces_json_registry_verdict_and_ast() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let grammar = manifest_dir.join("../grammars/json.ebnf");
        assert!(
            grammar.is_file(),
            "json grammar must exist at {}",
            grammar.display()
        );

        let opts = CompileAndParseOptions {
            // Persistent, gitignored workdir → the pgen dependency stays compiled across reruns.
            workdir: Some(manifest_dir.join("target/parse_harness_it/json")),
            keep_workdir: true,
            ..Default::default()
        };

        // ACCEPT: verdict + typed AST must match the registry byte-for-byte.
        let accept_input = r#"{"a": 1, "b": [true, null, "x"]}"#;
        let outcome = compile_and_parse(&grammar, accept_input, &opts)
            .expect("harness ACCEPT parse should succeed as plumbing");
        assert!(outcome.accepted, "harness must accept valid json: {outcome:?}");
        assert_eq!(
            crate::parser_registry::parse_sample("json", accept_input),
            Some(true),
            "registry must agree the input is accepted"
        );
        let registry_ast = crate::parser_registry::parse_sample_ast_json("json", accept_input)
            .expect("json registered")
            .expect("registry AST on accept");
        assert_eq!(
            outcome.ast_json.as_ref(),
            Some(&registry_ast),
            "compile-and-run typed AST must be byte-identical to the registry's"
        );

        // REJECT: verdict must match the registry (no AST).
        let reject_input = r#"{"a": }"#;
        let rejected = compile_and_parse(&grammar, reject_input, &opts)
            .expect("harness REJECT parse should still succeed as plumbing");
        assert!(!rejected.accepted, "harness must reject invalid json: {rejected:?}");
        assert_eq!(rejected.ast_json, None);
        assert_eq!(
            crate::parser_registry::parse_sample("json", reject_input),
            Some(false),
            "registry must agree the input is rejected"
        );
    }
}
