use anyhow::{Context, Result, bail};
use serde::Serialize;

use pgen::ast_pipeline::{
    configure_trace_output, resolve_trace_verbosity, set_global_trace_verbosity,
};
#[cfg(feature = "generated_parsers")]
use pgen::parser_registry;
use pgen::preprocessing_stage::{self, StageDiagnosticSeverity, StageOutput, StageRequest};

fn usage() -> &'static str {
    "Usage:\n  parseability_probe --parser-fingerprint\n  parseability_probe --stack-reservation\n  parseability_probe --supports <grammar_name> [--profile PROFILE] [--trace] [--trace-rules R1,R2,...] [--trace-log-file [FILE]] [--dump-rule-call-counts]\n  parseability_probe --parse <grammar_name> <input_file> [--profile PROFILE] [--entry-rule RULE] [--lib-in DIR] [--lib-out DIR] [--trace] [--trace-rules R1,R2,...] [--trace-log-file [FILE]] [--dump-rule-call-counts]\n  parseability_probe --parse-dump-ast <grammar_name> <input_file> [output_file] [--profile PROFILE] [--max-bytes N] [--entry-rule RULE] [--lib-in DIR] [--lib-out DIR] [--trace] [--trace-rules R1,R2,...] [--trace-log-file [FILE]] [--dump-rule-call-counts]\n  parseability_probe --parse-dump-ast-pretty <grammar_name> <input_file> [output_file] [--profile PROFILE] [--max-bytes N] [--entry-rule RULE] [--lib-in DIR] [--lib-out DIR] [--trace] [--trace-rules R1,R2,...] [--trace-log-file [FILE]] [--dump-rule-call-counts]\n\nDefault AST dump filename (when output_file omitted): <grammar_name>_ast.json\nOptional env fallback for dump-size bound: PGEN_PARSE_DUMP_AST_MAX_BYTES\nOptional env fallback for trace verbosity: PGEN_TRACE_VERBOSITY\nOptional opt-in stack-probe report (`ENGINE-UNIVERSAL-SERVICES.69`(d)): PGEN_REPORT_STACK_PROBE=1 prints, on stderr, the depth probe's own reading at its last refusal — cause, live depth, stack bytes actually spent, the reservation and the spendable budget. Opt-in because `stimuli/run_external_corpus.sh` captures this binary's stderr as a per-file corpus column (`SV-CORPUS-GRAD.12a`). Use it to tell a genuinely too-deep input (`cause=BudgetExhausted`) from a parse that never learned its own stack size (`cause=ExtentUnknown`); a parse can CONTACT the ceiling and still ACCEPT, at a large cost.\n--entry-rule RULE         : (`GRAMMAR-WELLFORMED.H.12.8.4.3.1`; dump support `SV-AST-SHAPE-FIDELITY.2.4`) parse `--parse` input — or dump the AST of `--parse-dump-ast[-pretty]` input — from an ALTERNATE start symbol (e.g. `library_text`, or a PEG-shadowed expression-position rule) via the generated parser's `parse_full_from`, instead of the grammar's canonical entry. Lets an entry-relative or shadowed rule (unreachable from the default entry) be reproduced/traced/AST-dumped in isolation. AST-dump `--entry-rule` is served by every registered generated parser (`.112` slice 2's one template). Default = the canonical entry (byte-identical to omitting the flag).\n--lib-in DIR              : (`SV-EXH-PROOF.3.3.4.a` MVP-0) directory artifacts are READ from for `@import_from_library`.\n--lib-out DIR             : (`SV-EXH-PROOF.3.3.4.a` MVP-0) directory artifacts are WRITTEN to for `@export_to_library`.\n--preprocess              : (`SVPP-EXPANSION.3`, `ENGINE-UNIVERSAL-SERVICES.113`) run the grammar's family PREPROCESSING STAGE on <input_file> IN-PROCESS (a family declares one or the flag is refused, naming the families that do) and parse the EXPANDED text — the preprocess→parse pipeline in one command. A rejection is reported with `[origin=FILE:LINE:COL]`, the position mapped back through the preprocessor's source map (an included file names itself; a macro expansion names its use site). If the preprocessor refuses the input the parser never runs and the exit code is 3 (`preprocess_failed …` on stderr), so a corpus lane can tell it from a parse rejection (1).\n--include-dir DIR         : (with --preprocess, repeatable, ordered) an include search directory — the same flag `ast_pipeline --preprocess <family>` takes.\n--symbol NAME[=VALUE]     : (with --preprocess, repeatable, ordered; alias --define) a symbol PREDEFINED before the first line as if it were defined at the top of the unit; NAME alone = empty text (`SVPP-EXPANSION.2`).\n--unit-file F             : (with --preprocess, repeatable, ordered) a file preprocessed BEFORE <input_file> in the same COMPILATION UNIT (its definitions carry over); its own text is discarded.\n--stage-opt KEY=VALUE     : (with --preprocess, repeatable) a stage option the family DECLARES (`ast_pipeline --preprocess <family> --list-stage-options` prints them); an undeclared key is refused. The pre-.113 `--<family-prefix>-…` spellings are accepted through release 1.0.247 and rewritten to these, one stderr line each.\nPGEN_REPORT_PREPROCESS=1  : print one `preprocess: …` summary line on stderr under --preprocess (opt-in, like PGEN_REPORT_STACK_PROBE, because corpus runners capture this binary's stderr).\n--trace-rules             : (`SV-EXH-PROOF.3.3.4.b.6.2.17`) comma-separated rule-name list. Trace activates ONLY inside the call-tree of these rules (implies --trace). Reduces trace volume 100-1000× vs --trace for targeted investigation.\n--dump-rule-call-counts   : (`SV-EXH-PROOF.3.3.4.b.6.2.22`) live per-rule call-count dashboard. Each rule's call counter is incremented on every entry; the top-20 rules sorted by count are shown on stderr and updated every 250ms in place. Use to identify which rules dominate a stuck or slow parse; works on timeout (dashboard keeps refreshing until the process is killed). Accepts an optional integer arg to control the top-N (default 20).\n--dump-rule-call-counts-exclude R1,R2,... : (`SV-EXH-PROOF.3.3.4.b.6.2.22`) filter these rules OUT of the dashboard before computing the top-N. Use to hide always-dominant noise like `trivia` (whitespace handling) so the diagnostically interesting rules win display slots.\n--parser-fingerprint      : (`ENGINE-UNIVERSAL-SERVICES.24` (ii′)) print, as JSON on stdout, the sha256 of every generated parser THIS BINARY was compiled against — computed by `build.rs`, which already `rerun-if-changed`s each resolved parser. Answers the question a source-only identity table cannot: which parser does the executable that produces the numbers actually embed? Consumed by `stimuli/sv/corpus_parse_cost.py`, which REFUSES to measure when the probe's SystemVerilog digest differs from the parser on disk.\n--dump-ast-with-coverage  : (`ENGINE-UNIVERSAL-SERVICES.22` (b)) `--parse-dump-ast[-pretty]` only. Dump the AST of a parse run with TRANSACTIONAL COVERAGE enabled — the exact configuration the TOOLBOX 3.4/3.5 dumps measure under — so it can be compared against a bare parse's AST. Without it the dump path always runs `bare_parse = true` (the fused `cascade_*` graph), and passing `--dump-rule-outcome-counts-json` beside it does NOT change that: the coverage call lives in the `--parse` detail macro, so both arms come out bare and agree for the wrong reason. SystemVerilog only, and it REFUSES on any other grammar or alongside `--entry-rule` rather than silently dumping bare.\n--stack-reservation       : (`ENGINE-UNIVERSAL-SERVICES.69`(a)) print, as `key=value` lines on stdout, the parse-stack reservation THIS PROCESS resolved on THIS HOST and where it came from — `operator_override` (`PGEN_PARSE_STACK_BYTES`), `host_memory_derived` (physical memory / the share divisor, clamped to the floor and the reference), or `reference_fallback` (the host's memory could not be read). Since `.69`(a) the reservation is a derived host quantity rather than a literal, and the only other surface that reports it is the `.69`(d) stack-probe line, which appears ONLY on a refusal — i.e. only after provoking a parse deep enough to contact the ceiling."
}

/// `SV-CORPUS-GRAD.12c.1` — read the input as USER SOURCE TEXT.
///
/// `std::fs::read_to_string` refused the FILE (not a construct) for the 13 ISO-8859-1 files of the
/// tracked SV corpus, so those rows never reached the parser at all. The shared decoder reads
/// them; anything other than plain UTF-8 is announced on **stderr**, which the corpus runner
/// captures, so a shifted byte offset is never a silent surprise.
#[cfg(feature = "generated_parsers")]
fn read_source(input_file: &str) -> Result<String> {
    let decoded = pgen::source_text::read_source_file(input_file)
        .with_context(|| format!("failed to read input file '{}'", input_file))?;
    if let Some(notice) = decoded.non_utf8_notice(input_file) {
        eprintln!("{notice}");
    }
    Ok(decoded.text)
}

fn default_ast_dump_file(grammar_name: &str) -> String {
    let mut stem = String::with_capacity(grammar_name.len());
    for ch in grammar_name.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            stem.push(ch);
        } else {
            stem.push('_');
        }
    }
    let stem = if stem.is_empty() {
        "grammar".to_string()
    } else {
        stem
    };
    format!("{}_ast.json", stem)
}

#[derive(Debug, Serialize)]
struct AstDumpTruncationDiagnostic {
    pgen_dump_contract_version: u32,
    kind: &'static str,
    truncated: bool,
    dump_kind: &'static str,
    max_bytes: usize,
    full_bytes: usize,
    reason: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct AstDumpWriteResult {
    truncated: bool,
    bytes_written: usize,
    full_bytes: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct GlobalOptions {
    profile: Option<String>,
    /// GRAMMAR-WELLFORMED.H.12.8.4.3.1 — optional ALTERNATE start symbol for `--parse`.
    /// `None` (default) parses from the grammar's canonical entry (byte-identical to before).
    /// `Some(rule)` parses from `parse_full_from(rule)` so an entry-relative rule (rooted under an
    /// alternate LRM start symbol such as `library_text`) can be reproduced/traced in isolation.
    entry_rule: Option<String>,
    trace: bool,
    trace_log_file: Option<String>,
    /// `SV-EXH-PROOF.3.3.4.a` MVP-0: source directory for
    /// `@import_from_library` artifact reads. `None` (the default) keeps
    /// single-file behaviour byte-identical to today.
    library_in_dir: Option<String>,
    /// `SV-EXH-PROOF.3.3.4.a` MVP-0: target directory for
    /// `@export_to_library` artifact writes. `None` (the default) keeps
    /// single-file behaviour byte-identical to today.
    library_out_dir: Option<String>,
    /// `SV-EXH-PROOF.3.3.4.b.6.2.17` — rule-level targeted trace.
    /// Comma-separated list of rule names; trace activates only inside
    /// the call-tree of these rules. `None` (default) = parser-level
    /// full trace (the prior `--trace` behavior).
    trace_rules: Option<Vec<String>>,
    /// `SV-EXH-PROOF.3.3.4.b.6.2.22` — live per-rule call-count
    /// dashboard. `None` = disabled (default). `Some(N)` enables the
    /// dashboard showing the top-N rules by call count, refreshing
    /// every 250ms. N is user-controlled because SV has ~1500 rules
    /// and the right top-N varies by investigation (top-10 for quick
    /// triage, top-50 for fine-grained pattern hunting).
    dump_rule_call_counts: Option<usize>,
    /// `SV-EXH-PROOF.3.3.4.b.6.2.22` — exclusion list for the dashboard.
    /// Filter OUT these rules before computing the top-N so user-
    /// irrelevant always-dominant rules (e.g. `trivia` for whitespace
    /// handling) don't steal display slots. Empty (default) = no
    /// filtering.
    dump_rule_call_counts_exclude: Option<Vec<String>>,
    /// `RGX-0078.5.h.1` — machine-readable per-rule ENTRY-COUNT dump: after a
    /// `--parse`, write the parser's monotone rule-entry counters (successful
    /// AND backtracked entries — the fusibility census's dynamic input) as
    /// JSON to this path. `None` = disabled (default). The machine-readable
    /// dual of the live `--dump-rule-call-counts` dashboard, which is
    /// stderr-only and refresh-based (useless for a sub-millisecond parse).
    dump_rule_entry_counts_json: Option<String>,
    /// `RGX-0078.5.h.1b` — machine-readable per-rule OUTCOME-COUNT dump: after a
    /// `--parse` run with the transactional coverage stack enabled, write BOTH the
    /// raw entry counters AND the committed (surviving) counts as JSON to this
    /// path. `committed` is an OCCURRENCE count (C3-B survivors, memo replays with
    /// multiplicity), so `raw − committed` is NOT the failed-speculation count and can
    /// go negative (SV-CORPUS-GRAD.13e.12) — the choice-site census, its consumer,
    /// subtracts with `saturating_sub` and reports `committed_overshoot`. `None` =
    /// disabled (default; coverage stays off, behavior byte-identical).
    dump_rule_outcome_counts_json: Option<String>,
    /// `ENGINE-UNIVERSAL-SERVICES.22` (b) — dump the AST of a parse with TRANSACTIONAL COVERAGE
    /// enabled, i.e. the exact configuration TOOLBOX 3.4/3.5 measures under. Without it,
    /// `--parse-dump-ast` always runs `bare_parse = true` (the fused `cascade_*` graph), so an A/B
    /// against a dump-attached parse cannot be built at all — the AST-dump path never calls
    /// `enable_coverage`, and passing `--dump-rule-outcome-counts-json` beside it does NOT change
    /// that (measured: no counts file is written and both arms come out bare). `--parse-dump-ast`
    /// only; SystemVerilog only, and it says so rather than silently ignoring the flag.
    dump_ast_with_coverage: bool,
    // `SVPP-EXPANSION.3` / `.113` — the preprocess→parse chain and its compilation context: the
    // engine's neutral request (include dirs, defines, the unit's earlier files) plus the stage
    // options the family declares (`--stage-opt KEY=VALUE`).
    preprocess: bool,
    include_dirs: Vec<String>,
    symbols: Vec<String>,
    unit_files: Vec<String>,
    stage_opts: Vec<(String, String)>,
}

fn parse_positive_usize(value: &str, label: &str) -> Result<usize> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| anyhow::anyhow!("{} must be an integer >= 1 (got '{}')", label, value))?;
    if parsed == 0 {
        bail!("{} must be an integer >= 1", label);
    }
    Ok(parsed)
}

fn resolve_dump_max_bytes(cli_value: Option<usize>) -> Result<Option<usize>> {
    if let Some(value) = cli_value {
        if value == 0 {
            bail!("--max-bytes must be an integer >= 1");
        }
        return Ok(Some(value));
    }
    let raw = match std::env::var("PGEN_PARSE_DUMP_AST_MAX_BYTES") {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let parsed = parse_positive_usize(trimmed, "PGEN_PARSE_DUMP_AST_MAX_BYTES")?;
    Ok(Some(parsed))
}

fn parse_dump_command_tail(args: &[String]) -> Result<(Option<String>, Option<usize>)> {
    let mut output_file: Option<String> = None;
    let mut max_bytes: Option<usize> = None;
    let mut idx = 0usize;
    while idx < args.len() {
        let token = args[idx].as_str();
        if token == "--max-bytes" {
            if max_bytes.is_some() {
                bail!("--max-bytes cannot be specified multiple times");
            }
            let value = args
                .get(idx + 1)
                .ok_or_else(|| anyhow::anyhow!("--max-bytes requires a value (integer >= 1)"))?;
            max_bytes = Some(parse_positive_usize(value, "--max-bytes")?);
            idx += 2;
            continue;
        }
        if output_file.is_some() {
            bail!(
                "unexpected extra positional argument '{}'; expected at most one output_file",
                token
            );
        }
        output_file = Some(args[idx].clone());
        idx += 1;
    }
    Ok((output_file, max_bytes))
}

fn strip_global_flags(args: &[String]) -> Result<(Vec<String>, GlobalOptions)> {
    let mut remaining = Vec::new();
    let mut options = GlobalOptions::default();
    let mut idx = 0usize;
    while idx < args.len() {
        if args[idx] == "--profile" {
            if options.profile.is_some() {
                bail!("--profile cannot be specified multiple times");
            }
            let value = args
                .get(idx + 1)
                .ok_or_else(|| anyhow::anyhow!("--profile requires a value"))?;
            options.profile = Some(value.clone());
            idx += 2;
            continue;
        }
        if args[idx] == "--entry-rule" {
            // GRAMMAR-WELLFORMED.H.12.8.4.3.1: parse from an alternate start symbol.
            if options.entry_rule.is_some() {
                bail!("--entry-rule cannot be specified multiple times");
            }
            let value = args
                .get(idx + 1)
                .ok_or_else(|| anyhow::anyhow!("--entry-rule requires a value"))?;
            options.entry_rule = Some(value.clone());
            idx += 2;
            continue;
        }
        if args[idx] == "--trace" {
            options.trace = true;
            idx += 1;
            continue;
        }
        // SV-EXH-PROOF.3.3.4.b.6.2.17 — rule-level targeted trace.
        if args[idx] == "--trace-rules" {
            if options.trace_rules.is_some() {
                bail!("--trace-rules cannot be specified multiple times");
            }
            let value = args.get(idx + 1).ok_or_else(|| {
                anyhow::anyhow!("--trace-rules requires a comma-separated rule-name list")
            })?;
            let rules: Vec<String> = value
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if rules.is_empty() {
                bail!("--trace-rules list cannot be empty");
            }
            // Implies --trace (rule-level trace is meaningless without the logger).
            options.trace = true;
            options.trace_rules = Some(rules);
            idx += 2;
            continue;
        }
        if args[idx] == "--trace-log-file" {
            if options.trace_log_file.is_some() {
                bail!("--trace-log-file cannot be specified multiple times");
            }
            if let Some(value) = args.get(idx + 1) {
                if !value.starts_with("--") {
                    options.trace_log_file = Some(value.clone());
                    idx += 2;
                    continue;
                }
            }
            options.trace_log_file = Some("trace.log".to_string());
            idx += 1;
            continue;
        }
        // `SV-EXH-PROOF.3.3.4.a` MVP-0: parser-agnostic library directories.
        if args[idx] == "--lib-in" {
            if options.library_in_dir.is_some() {
                bail!("--lib-in cannot be specified multiple times");
            }
            let value = args
                .get(idx + 1)
                .ok_or_else(|| anyhow::anyhow!("--lib-in requires a directory path"))?;
            if value.starts_with("--") {
                bail!("--lib-in requires a directory path (got flag '{}')", value);
            }
            options.library_in_dir = Some(value.clone());
            idx += 2;
            continue;
        }
        if args[idx] == "--lib-out" {
            if options.library_out_dir.is_some() {
                bail!("--lib-out cannot be specified multiple times");
            }
            let value = args
                .get(idx + 1)
                .ok_or_else(|| anyhow::anyhow!("--lib-out requires a directory path"))?;
            if value.starts_with("--") {
                bail!("--lib-out requires a directory path (got flag '{}')", value);
            }
            options.library_out_dir = Some(value.clone());
            idx += 2;
            continue;
        }
        // `SVPP-EXPANSION.3` / `.113` — the preprocess→parse chain. `--preprocess` runs the parsed
        // grammar's family preprocessing stage on the input IN-PROCESS and parses the EXPANDED
        // text; the context flags are `ast_pipeline --preprocess <family>`'s own, repeatable and
        // ordered, and they mean nothing without `--preprocess` (checked once every flag is read).
        if args[idx] == "--preprocess" {
            if options.preprocess {
                bail!("--preprocess cannot be specified multiple times");
            }
            options.preprocess = true;
            idx += 1;
            continue;
        }
        if args[idx] == "--include-dir"
            || args[idx] == "--symbol"
            || args[idx] == "--define"
            || args[idx] == "--unit-file"
            || args[idx] == "--stage-opt"
        {
            let flag = args[idx].clone();
            let value = args
                .get(idx + 1)
                .ok_or_else(|| anyhow::anyhow!("{} requires a value", flag))?;
            if value.starts_with("--") || value.trim().is_empty() {
                bail!("{} requires a value (got '{}')", flag, value);
            }
            match flag.as_str() {
                "--include-dir" => options.include_dirs.push(value.clone()),
                // `--define` is the alias of `--symbol` (director, 2026-09-29): same list, same order.
                "--symbol" | "--define" => options.symbols.push(value.clone()),
                "--stage-opt" => options.stage_opts.push(preprocessing_stage::parse_stage_opt(value)?),
                _ => options.unit_files.push(value.clone()),
            }
            idx += 2;
            continue;
        }
        // SV-EXH-PROOF.3.3.4.b.6.2.22 — live per-rule call-count dashboard.
        // Bare form `--dump-rule-call-counts` defaults to top-20. With a
        // following numeric arg `--dump-rule-call-counts 50`, that becomes
        // top-N. SV has ~1500 rules — N matters because the right slice
        // varies by investigation (top-10 for triage, top-50 for fine
        // pattern hunting).
        // SV-EXH-PROOF.3.3.4.b.6.2.22 — dashboard exclusion list. Must come
        // before --dump-rule-call-counts in the prefix-match below, otherwise
        // --dump-rule-call-counts-exclude would match the shorter prefix.
        if args[idx] == "--dump-rule-call-counts-exclude" {
            if options.dump_rule_call_counts_exclude.is_some() {
                bail!("--dump-rule-call-counts-exclude cannot be specified multiple times");
            }
            let value = args.get(idx + 1).ok_or_else(|| {
                anyhow::anyhow!(
                    "--dump-rule-call-counts-exclude requires a comma-separated rule-name list"
                )
            })?;
            let rules: Vec<String> = value
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if rules.is_empty() {
                bail!("--dump-rule-call-counts-exclude list cannot be empty");
            }
            options.dump_rule_call_counts_exclude = Some(rules);
            idx += 2;
            continue;
        }
        // RGX-0078.5.h.1 — the JSON entry-count dump. Must also come before
        // --dump-rule-call-counts in the prefix chain (longer spelling first).
        if args[idx] == "--dump-rule-entry-counts-json" {
            if options.dump_rule_entry_counts_json.is_some() {
                bail!("--dump-rule-entry-counts-json cannot be specified multiple times");
            }
            let value = args.get(idx + 1).ok_or_else(|| {
                anyhow::anyhow!("--dump-rule-entry-counts-json requires an output file path")
            })?;
            if value.starts_with("--") || value.trim().is_empty() {
                bail!("--dump-rule-entry-counts-json requires an output file path");
            }
            options.dump_rule_entry_counts_json = Some(value.clone());
            idx += 2;
            continue;
        }
        // ENGINE-UNIVERSAL-SERVICES.22 (b) — AST dump of a COVERAGE-ENABLED parse.
        if args[idx] == "--dump-ast-with-coverage" {
            if options.dump_ast_with_coverage {
                bail!("--dump-ast-with-coverage cannot be specified multiple times");
            }
            options.dump_ast_with_coverage = true;
            idx += 1;
            continue;
        }
        // RGX-0078.5.h.1b — the JSON outcome-count dump (raw + committed entries).
        if args[idx] == "--dump-rule-outcome-counts-json" {
            if options.dump_rule_outcome_counts_json.is_some() {
                bail!("--dump-rule-outcome-counts-json cannot be specified multiple times");
            }
            let value = args.get(idx + 1).ok_or_else(|| {
                anyhow::anyhow!("--dump-rule-outcome-counts-json requires an output file path")
            })?;
            if value.starts_with("--") || value.trim().is_empty() {
                bail!("--dump-rule-outcome-counts-json requires an output file path");
            }
            options.dump_rule_outcome_counts_json = Some(value.clone());
            idx += 2;
            continue;
        }
        if args[idx] == "--dump-rule-call-counts" {
            if options.dump_rule_call_counts.is_some() {
                bail!("--dump-rule-call-counts cannot be specified multiple times");
            }
            let top_n: usize = match args.get(idx + 1) {
                Some(value) if !value.starts_with("--") => {
                    match value.parse::<usize>() {
                        Ok(n) if n >= 1 => {
                            // Consumed the numeric arg.
                            options.dump_rule_call_counts = Some(n);
                            idx += 2;
                            continue;
                        }
                        Ok(_) => bail!("--dump-rule-call-counts N must be >= 1"),
                        Err(_) => {
                            // Non-numeric next-token: treat as a separate
                            // arg and use the default.
                            20
                        }
                    }
                }
                _ => 20,
            };
            options.dump_rule_call_counts = Some(top_n);
            idx += 1;
            continue;
        }
        remaining.push(args[idx].clone());
        idx += 1;
    }
    if !options.preprocess
        && (!options.include_dirs.is_empty()
            || !options.symbols.is_empty()
            || !options.unit_files.is_empty()
            || !options.stage_opts.is_empty())
    {
        bail!(
            "--include-dir / --symbol / --unit-file / --stage-opt describe the preprocessing stage's \
             compilation context and need --preprocess"
        );
    }
    Ok((remaining, options))
}

/// `SVPP-EXPANSION.3` — what a parse command reads: the file's decoded text, or — under
/// `--preprocess` — the EXPANDED text the family's preprocessing stage produced from it, kept
/// whole (source map included) so a rejection can be mapped back to the text the operator wrote.
enum ProbeInput {
    Raw(String),
    Preprocessed(StageOutput),
}

impl ProbeInput {
    fn text(&self) -> &str {
        match self {
            ProbeInput::Raw(text) => text,
            ProbeInput::Preprocessed(output) => &output.text,
        }
    }

    fn preprocessed(&self) -> Option<&StageOutput> {
        match self {
            ProbeInput::Raw(_) => None,
            ProbeInput::Preprocessed(output) => Some(output),
        }
    }
}

/// `SVPP-EXPANSION.3` / `.113` — read the input as the command will parse it. Under
/// `--preprocess` the grammar's family stage (`preprocessing_stage::stage_for`, refused with the
/// families that declare one when there is none) runs the compilation unit `--unit-file`s… then
/// `<input_file>` under ONE definition table (`SVPP-EXPANSION.2`); only the last file's expansion
/// is parsed. ⛔ A stage refusal is NOT a parse rejection: the parser never ran, so the process
/// exits **3** (documented) with a `preprocess_failed …` line, never 1 — a corpus lane keys on
/// exactly that difference.
#[cfg(feature = "generated_parsers")]
fn load_probe_input(
    grammar_name: &str,
    input_file: &str,
    options: &GlobalOptions,
) -> Result<ProbeInput> {
    if !options.preprocess {
        return Ok(ProbeInput::Raw(read_source(input_file)?));
    }
    let Some(stage) = preprocessing_stage::stage_for(grammar_name) else {
        let with = preprocessing_stage::families_with_stage();
        bail!(
            "--preprocess runs a family's preprocessing stage before the parser; grammar '{}' declares \
             none (families that do: {})",
            grammar_name,
            if with.is_empty() { "none in this build".to_string() } else { with.join(", ") }
        );
    };
    let mut unit: Vec<std::path::PathBuf> = options
        .unit_files
        .iter()
        .map(std::path::PathBuf::from)
        .collect();
    unit.push(std::path::PathBuf::from(input_file));
    let request = StageRequest {
        unit,
        include_dirs: options
            .include_dirs
            .iter()
            .map(std::path::PathBuf::from)
            .collect(),
        symbols: options.symbols.clone(),
        // `SVPP-EXPANSION.8`: the profile this command will PARSE under is what the stage is told,
        // so a dialect-profiled family can decide what its keyword regions must escape.
        parse_profile: options.profile.clone(),
        options: options.stage_opts.clone(),
    };
    let mut outputs = match stage.execute(&request) {
        Ok(outputs) => outputs,
        Err(err) => {
            eprintln!(
                "preprocess_failed for grammar '{}' on '{}': {:#}",
                grammar_name, input_file, err
            );
            std::process::exit(3);
        }
    };
    let output = outputs.pop().ok_or_else(|| {
        anyhow::anyhow!(
            "the compilation unit produced no output for '{}'",
            input_file
        )
    })?;
    if std::env::var("PGEN_REPORT_PREPROCESS").is_ok() {
        report_preprocess_summary(input_file, options, &output);
    }
    Ok(ProbeInput::Preprocessed(output))
}

/// `SVPP-EXPANSION.3` — the opt-in `preprocess:` line (`PGEN_REPORT_PREPROCESS=1`). Opt-in for the
/// same reason as `stack_probe:`: `stimuli/run_external_corpus.sh` captures this binary's stderr
/// as a per-file corpus column, and an unconditional line would edit a tracked measurement.
#[cfg(feature = "generated_parsers")]
fn report_preprocess_summary(
    input_file: &str,
    options: &GlobalOptions,
    output: &StageOutput,
) {
    let warnings = output
        .diagnostics
        .iter()
        .filter(|d| matches!(d.severity, StageDiagnosticSeverity::Warning))
        .count();
    eprintln!(
        "preprocess: input='{}' unit_predecessors={} include_dirs={} symbols={} output_bytes={} source_map_entries={} included_files={} events={} warnings={}",
        input_file,
        options.unit_files.len(),
        options.include_dirs.len(),
        options.symbols.len(),
        output.text.len(),
        output.source_map.len(),
        output.included_files.len(),
        output.events.len(),
        warnings
    );
}

/// `SVPP-EXPANSION.3` — the byte offset a rejection names, read off the registry's message:
/// `furthest_position=N` (the deepest byte any branch reached — the one that locates the defect,
/// TOOLBOX §3.2) when present, else the surface `at position N`.
fn rejection_position(message: &str) -> Option<usize> {
    fn leading_digits(tail: &str) -> Option<usize> {
        tail.split(|c: char| !c.is_ascii_digit())
            .next()
            .and_then(|digits| digits.parse::<usize>().ok())
    }
    if let Some((_, tail)) = message.rsplit_once("furthest_position=") {
        return leading_digits(tail);
    }
    message
        .rsplit_once("at position ")
        .and_then(|(_, tail)| leading_digits(tail))
}

/// `SVPP-EXPANSION.3` — map a byte offset in the EXPANDED text back to where that text came from.
/// The preprocessor appends one source-map chunk per emitted line, so the chunk holding `position`
/// names the FILE and LINE the bytes were written from: an `include`d line names the included file,
/// and a macro EXPANSION names the line of the macro's USE — the expansion has no line of its own.
/// The column is the chunk's first column plus the CHARACTERS (not bytes) between the chunk's start
/// and `position`. A position at or past the end of the text lands on the last chunk's end.
fn origin_of_position(output: &StageOutput, position: usize) -> Option<String> {
    let map = &output.source_map;
    if map.is_empty() {
        return None;
    }
    let index = match map.binary_search_by(|entry| entry.output_start.cmp(&position)) {
        Ok(index) => index,
        Err(0) => return None,
        Err(index) => index - 1,
    };
    let entry = &map[index];
    let end = entry.output_end.min(output.text.len());
    let clamped = position.min(end).max(entry.output_start);
    let chars = output
        .text
        .get(entry.output_start..clamped)
        .map(|chunk| chunk.chars().count())
        .unwrap_or(0);
    Some(format!(
        "{}:{}:{}",
        display_origin_file(&entry.source.file),
        entry.source.line,
        entry.source.column_start + chars
    ))
}

/// The source map records canonical (absolute) paths; print them relative to the working directory
/// when they sit under it, so a corpus row reads like the row's own path.
fn display_origin_file(file: &str) -> String {
    if let Ok(cwd) = std::env::current_dir() {
        let cwd = cwd.to_string_lossy().to_string();
        if let Some(rest) = file.strip_prefix(&cwd) {
            let rest = rest.trim_start_matches('/');
            if !rest.is_empty() {
                return rest.to_string();
            }
        }
    }
    file.to_string()
}

/// `SVPP-EXPANSION.3` — append `[origin=FILE:LINE:COL]` to a rejection of PREPROCESSED input.
/// Raw input is returned untouched (byte-identical messages for every existing caller); a
/// preprocessed rejection whose position cannot be mapped says `origin=unknown` rather than
/// nothing, so the absence is visible.
fn augment_rejection_with_origin(message: String, input: &ProbeInput) -> String {
    let Some(output) = input.preprocessed() else {
        return message;
    };
    match rejection_position(&message).and_then(|position| origin_of_position(output, position)) {
        Some(origin) => format!("{} [origin={}]", message, origin),
        None => format!("{} [origin=unknown]", message),
    }
}

fn configure_runtime_trace(options: &GlobalOptions) -> Result<()> {
    configure_trace_output(options.trace_log_file.as_deref())?;
    let verbosity = resolve_trace_verbosity(None, false, options.trace)?;
    set_global_trace_verbosity(verbosity);
    // SV-EXH-PROOF.3.3.4.b.6.2.17 — propagate rule-level trace filter to the
    // parser_registry thread-local so subsequent parser invocations on this
    // thread apply it via `set_trace_rules` after construction.
    let rules = options
        .trace_rules
        .as_ref()
        .map(|v| v.iter().cloned().collect());
    pgen::parser_registry::set_global_trace_rules(rules);
    // SV-EXH-PROOF.3.3.4.b.6.2.22 — propagate the per-rule call-count
    // dashboard top-N selector. None = no dashboard; Some(N) = show
    // top-N. The registry's parse-with-systemverilog-* helpers spawn
    // the dashboard right after parser construction and tear it down
    // when the parser is dropped at parse return.
    pgen::parser_registry::set_global_dump_rule_call_counts(options.dump_rule_call_counts);
    // SV-EXH-PROOF.3.3.4.b.6.2.22 — propagate the dashboard exclusion list.
    let exclude = options
        .dump_rule_call_counts_exclude
        .as_ref()
        .map(|v| v.iter().cloned().collect());
    pgen::parser_registry::set_global_dump_rule_call_counts_exclude(exclude);
    // RGX-0078.5.h.1 — propagate the machine-readable entry-count dump path.
    // The registry's per-grammar detail-parse functions snapshot the counters
    // after the parse and write the JSON (see `set_global_dump_rule_entry_counts_json`).
    pgen::parser_registry::set_global_dump_rule_entry_counts_json(
        options
            .dump_rule_entry_counts_json
            .as_ref()
            .map(std::path::PathBuf::from),
    );
    // RGX-0078.5.h.1b — propagate the outcome-count dump path (raw + committed);
    // when set, the detail-parse functions enable transactional coverage before
    // the parse (see `set_global_dump_rule_outcome_counts_json`).
    pgen::parser_registry::set_global_dump_rule_outcome_counts_json(
        options
            .dump_rule_outcome_counts_json
            .as_ref()
            .map(std::path::PathBuf::from),
    );
    Ok(())
}

fn canonicalize_json_value(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Array(values) => {
            serde_json::Value::Array(values.into_iter().map(canonicalize_json_value).collect())
        }
        serde_json::Value::Object(map) => {
            let mut entries = map.into_iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            let mut normalized = serde_json::Map::new();
            for (key, value) in entries {
                normalized.insert(key, canonicalize_json_value(value));
            }
            serde_json::Value::Object(normalized)
        }
        other => other,
    }
}

fn encode_canonical_json<T: Serialize>(value: &T, pretty: bool) -> Result<String> {
    let normalized = canonicalize_json_value(serde_json::to_value(value)?);
    if pretty {
        Ok(serde_json::to_string_pretty(&normalized)?)
    } else {
        Ok(serde_json::to_string(&normalized)?)
    }
}

fn write_json_dump_with_limit(
    output_path: &str,
    encoded_json: &str,
    max_bytes: Option<usize>,
    pretty: bool,
    dump_kind: &'static str,
) -> Result<AstDumpWriteResult> {
    let full_bytes = encoded_json.len();
    if let Some(max) = max_bytes {
        if full_bytes > max {
            let diagnostic = AstDumpTruncationDiagnostic {
                pgen_dump_contract_version: 1,
                kind: "pgen_ast_dump_truncation",
                truncated: true,
                dump_kind,
                max_bytes: max,
                full_bytes,
                reason: "encoded parser AST JSON exceeded configured max bytes; payload omitted",
            };
            let encoded_diagnostic = encode_canonical_json(&diagnostic, pretty)?;
            let diagnostic_bytes = encoded_diagnostic.len();
            if diagnostic_bytes > max {
                bail!(
                    "AST dump max-bytes ({}) is too small to fit truncation diagnostics (requires at least {} bytes)",
                    max,
                    diagnostic_bytes
                );
            }
            std::fs::write(output_path, encoded_diagnostic)?;
            return Ok(AstDumpWriteResult {
                truncated: true,
                bytes_written: diagnostic_bytes,
                full_bytes,
            });
        }
    }

    std::fs::write(output_path, encoded_json)?;
    Ok(AstDumpWriteResult {
        truncated: false,
        bytes_written: full_bytes,
        full_bytes,
    })
}

#[cfg(feature = "generated_parsers")]
fn supported_grammars_csv() -> String {
    let mut grammars = parser_registry::registered_grammars();
    grammars.sort_unstable();
    grammars.join(", ")
}

#[cfg(not(feature = "generated_parsers"))]
fn supported_grammars_csv() -> String {
    String::new()
}

#[cfg(feature = "generated_parsers")]
fn command_supports(grammar_name: &str, profile: Option<&str>) -> Result<()> {
    let _ = profile;
    if parser_registry::supports_grammar(grammar_name) {
        println!(
            "generated parseability adapter available for grammar '{}'",
            grammar_name
        );
        return Ok(());
    }
    bail!(
        "parseability adapter unavailable for grammar '{}'. Supported grammars: {}",
        grammar_name,
        supported_grammars_csv()
    );
}

#[cfg(not(feature = "generated_parsers"))]
fn command_supports(grammar_name: &str, profile: Option<&str>) -> Result<()> {
    let _ = (grammar_name, profile);
    bail!("parseability_probe requires building with --features generated_parsers");
}

#[cfg(feature = "generated_parsers")]
fn command_parse(grammar_name: &str, input_file: &str, options: &GlobalOptions) -> Result<()> {
    let profile = options.profile.as_deref();
    let entry_rule = options.entry_rule.as_deref();
    let input = load_probe_input(grammar_name, input_file, options)?;
    let sample = input.text().to_string();
    let library_options = parser_registry::LibraryOptions {
        in_dir: options
            .library_in_dir
            .as_deref()
            .map(std::path::PathBuf::from),
        out_dir: options
            .library_out_dir
            .as_deref()
            .map(std::path::PathBuf::from),
    };
    // GRAMMAR-WELLFORMED.H.12.8.4.3.1: an explicit `--entry-rule` parses from that alternate start
    // symbol via the entry-aware detail path; `None` keeps the byte-identical default-entry path
    // (which also honors `--lib-in`/`--lib-out` for `@import_from_library`/`@export_to_library`).
    // `ENGINE-UNIVERSAL-SERVICES.69`(d) — make the reading a PER-PARSE one. The
    // record is a per-thread latch, and this binary runs its whole body on one
    // dedicated parse thread, so without this a second `--parse` in the same
    // process would report the first one's refusal.
    pgen::dedicated_parse_stack::clear_last_stack_probe_refusal();
    let result = match entry_rule {
        Some(entry) => {
            parser_registry::parse_sample_detail_from_entry(grammar_name, &sample, profile, entry)
        }
        None => parser_registry::parse_sample_detail_with_options(
            grammar_name,
            &sample,
            profile,
            &library_options,
        ),
    };
    report_stack_probe_refusal();
    match result {
        Some(Ok(())) => {
            println!(
                "parse_full passed for grammar '{}' on '{}'{}",
                grammar_name,
                input_file,
                if input.preprocessed().is_some() {
                    " [preprocessed]"
                } else {
                    ""
                }
            );
            Ok(())
        }
        Some(Err(err)) => bail!(
            "parse_full rejected sample for grammar '{}' on '{}': {}",
            grammar_name,
            input_file,
            augment_rejection_with_origin(err, &input)
        ),
        None => bail!(
            "parseability adapter unavailable for grammar '{}'. Supported grammars: {}",
            grammar_name,
            supported_grammars_csv()
        ),
    }
}

/// `ENGINE-UNIVERSAL-SERVICES.69`(d) — print the depth probe's reading at its
/// last refusal, when `PGEN_REPORT_STACK_PROBE` is set.
///
/// ⛔ **OPT-IN, and deliberately so.** `stimuli/run_external_corpus.sh` CAPTURES
/// this binary's stderr and derives per-file corpus columns from it
/// (`SV-CORPUS-GRAD.12a`), so an unconditional line here would edit a tracked
/// measurement surface for every ceiling-contacting file in the corpus. Same
/// convention as `PGEN_REPORT_MEMO_STATS`.
///
/// ⭐ **WHY IT IS WORTH A SURFACE AT ALL.** A parse can CONTACT the ceiling and
/// still ACCEPT, and until this line the only evidence of that was the wall
/// clock. Measured on the shipped SV parser (`--profile sv_2017`, release), the
/// same 3 000-deep nested-paren input accepts under BOTH reservations and the
/// cost is not close: **9.61 s at the 256 MiB default and 162.11 s at the
/// 96 MiB floor — one verdict, 16.9× the cost.** The 96 MiB arm crosses from
/// 6.37 s (2 400 deep) to 23.55 s (2 600 deep), a 3.7× step for a 1.08× depth
/// step. Nothing in the parser said why.
fn report_stack_probe_refusal() {
    if std::env::var("PGEN_REPORT_STACK_PROBE").is_err() {
        return;
    }
    let Some(observation) = pgen::dedicated_parse_stack::last_stack_probe_refusal() else {
        eprintln!("stack_probe: no_refusal (the recursion ceiling was never contacted)");
        return;
    };
    // A not-measured field prints as `not_measured`, never as `0`: `0` is a
    // legal measured spend and reading one as the other is the whole failure
    // mode `StackProbeObservation`'s doc comment warns about.
    let byte_field = |measured: bool, value: usize| -> String {
        if measured {
            value.to_string()
        } else {
            "not_measured".to_string()
        }
    };
    let measured = observation.spend_is_measured();
    let pct = if observation.budget_bytes > 0 {
        format!(
            "{:.3}",
            (observation.used_bytes as f64 / observation.budget_bytes as f64) * 100.0
        )
    } else {
        "not_measured".to_string()
    };
    eprintln!(
        "stack_probe: cause={:?} depth={} used_bytes={} reservation_bytes={} budget_bytes={} bytes_per_frame={} used_pct_of_budget={}",
        observation.cause,
        observation.depth,
        byte_field(measured, observation.used_bytes),
        byte_field(
            observation.reservation_bytes > 0,
            observation.reservation_bytes
        ),
        byte_field(observation.budget_bytes > 0, observation.budget_bytes),
        byte_field(observation.bytes_per_frame > 0, observation.bytes_per_frame),
        pct,
    );
}

#[cfg(feature = "generated_parsers")]
fn command_parse_dump_ast(
    grammar_name: &str,
    input_file: &str,
    output_file: Option<&str>,
    pretty: bool,
    max_bytes: Option<usize>,
    options: &GlobalOptions,
) -> Result<()> {
    let profile = options.profile.as_deref();
    let entry_rule = options.entry_rule.as_deref();
    let with_coverage = options.dump_ast_with_coverage;
    let input = load_probe_input(grammar_name, input_file, options)?;
    let sample = input.text().to_string();
    // SV-AST-SHAPE-FIDELITY.2.4: an explicit `--entry-rule` dumps the AST parsed from that ALTERNATE
    // start symbol via the generated parser's `parse_full_from`, so a PEG-shadowed / entry-relative
    // rule can be inspected in isolation. `None` keeps the byte-identical canonical-entry dump path.
    // ⛔ REFUSE the combination rather than honouring one flag and dropping the other: the
    // entry-aware dump path has no coverage-enabled variant, so accepting both would hand back a
    // BARE dump under a flag that says coverage — the silent-agreement failure again.
    if with_coverage && entry_rule.is_some() {
        bail!(
            "--dump-ast-with-coverage cannot be combined with --entry-rule: the entry-aware AST \
             dump has no coverage-enabled variant, and running it bare under this flag would \
             produce an A/B whose arms are secretly identical"
        );
    }
    let parse_result = match entry_rule {
        Some(entry) => {
            parser_registry::parse_sample_ast_json_from_entry(grammar_name, &sample, profile, entry)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "entry-aware AST dump (`--entry-rule`) needs a registered generated parser for grammar '{}' (supported: {})",
                        grammar_name,
                        supported_grammars_csv()
                    )
                })?
        }
        // ENGINE-UNIVERSAL-SERVICES.22 (b): dump the AST of a COVERAGE-ENABLED parse, i.e. the
        // exact configuration TOOLBOX 3.4/3.5 measures under, so the derivation can be compared
        // against a bare parse's. ⛔ It REFUSES on an unwired grammar rather than silently
        // producing a bare dump — an A/B whose arms are secretly identical agrees for the wrong
        // reason, and that is precisely the trap this flag exists to avoid.
        // ⛔ Dispatched through the registry (mirroring `parse_sample_ast_json_from_entry`) rather
        // than cfg-branching here: `#[cfg]` on a block in TAIL-EXPRESSION position is an
        // expression attribute, which is unstable — the registry keeps every `has_generated_*`
        // decision in one place, which is why that file is shaped the way it is.
        None if with_coverage => {
            parser_registry::parse_sample_ast_json_with_coverage(grammar_name, &sample, profile)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "--dump-ast-with-coverage needs a registered generated parser for grammar \
                         '{}' (supported: {}); regenerate with \
                         `make -C rust SHELL=/bin/bash regenerate_generated_parsers`",
                        grammar_name,
                        supported_grammars_csv()
                    )
                })?
        }
        None => {
            parser_registry::parse_sample_ast_json_with_profile(grammar_name, &sample, profile)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "parseability adapter unavailable for grammar '{}'. Supported grammars: {}",
                        grammar_name,
                        supported_grammars_csv()
                    )
                })?
        }
    };
    let ast_json = parse_result.map_err(|err| {
        anyhow::anyhow!(
            "parse_full rejected sample for grammar '{}' on '{}': {}",
            grammar_name,
            input_file,
            augment_rejection_with_origin(err, &input)
        )
    })?;

    let resolved_output_path = output_file
        .map(|value| value.to_string())
        .unwrap_or_else(|| default_ast_dump_file(grammar_name));
    let encoded = encode_canonical_json(&ast_json, pretty)?;
    let write_result = write_json_dump_with_limit(
        &resolved_output_path,
        &encoded,
        max_bytes,
        pretty,
        "parser_return_ast",
    )
    .with_context(|| {
        format!(
            "failed to write parser AST log '{}'",
            resolved_output_path.as_str()
        )
    })?;
    if write_result.truncated {
        println!(
            "parse_full passed for grammar '{}' on '{}' (AST truncation diagnostics: {}, full_bytes={}, max_bytes={}, written_bytes={})",
            grammar_name,
            input_file,
            resolved_output_path.as_str(),
            write_result.full_bytes,
            max_bytes.unwrap_or(write_result.full_bytes),
            write_result.bytes_written
        );
    } else {
        println!(
            "parse_full passed for grammar '{}' on '{}' (AST dump: {})",
            grammar_name,
            input_file,
            resolved_output_path.as_str()
        );
    }
    Ok(())
}

#[cfg(not(feature = "generated_parsers"))]
fn command_parse_dump_ast(
    grammar_name: &str,
    input_file: &str,
    output_file: Option<&str>,
    pretty: bool,
    max_bytes: Option<usize>,
    options: &GlobalOptions,
) -> Result<()> {
    let _ = (
        grammar_name,
        input_file,
        output_file,
        pretty,
        max_bytes,
        options,
    );
    bail!("parseability_probe requires building with --features generated_parsers");
}

#[cfg(not(feature = "generated_parsers"))]
fn command_parse(grammar_name: &str, input_file: &str, options: &GlobalOptions) -> Result<()> {
    let _ = (grammar_name, input_file, options);
    bail!("parseability_probe requires building with --features generated_parsers");
}

/// `ENGINE-UNIVERSAL-SERVICES.24` (ii′) — the build-time parser fingerprint, one row per generated
/// parser this binary was compiled against.
///
/// `build.rs` hashes each generated parser it resolves and publishes the digest as
/// `PGEN_<FAMILY>_PARSER_SHA256`; this mode reads them back. It answers the one question the
/// `PARSE-COST-RATCHET` identity table could not: **which parser does the executable that produces
/// the numbers actually embed?** The table pins the grammar, the parser file, the instrument and
/// the sampled inputs — all SOURCES — while the probe binary itself is an untracked build artifact
/// nothing hashed.
///
/// ⛔ `option_env!` and not `env!`: a build with no generated parser on disk must still produce a
/// working binary that says the fingerprint is UNAVAILABLE. Compiling it away instead would make a
/// cold clone unbuildable, and defaulting it to a placeholder would make "not measured" look
/// exactly like "wrong parser".
// `.113`: the table is the registry's (`parser_registry::parser_fingerprints()`): every row that
// is its own generated artifact, with the digest build.rs computed — nothing here names a family.

/// The dump contract version for `--parser-fingerprint`. Bump it when the SHAPE changes, so a
/// consumer that outlives this schema refuses instead of positionally mis-reading it.
const PARSER_FINGERPRINT_VERSION: u32 = 1;

#[derive(Debug, Serialize)]
struct ParserFingerprintReport {
    pgen_parser_fingerprint_version: u32,
    /// family -> sha256 of the generated parser this binary compiled against.
    parsers: std::collections::BTreeMap<String, String>,
    /// registered artifact rows build.rs could not hash, so this binary embeds no digest for them.
    /// A family with NO generated parser on disk is not registered at all and appears in neither
    /// list (`.113`: the table is derived from the registry, which only knows what was built).
    absent: Vec<String>,
}

#[cfg(feature = "generated_parsers")]
fn command_parser_fingerprint() -> Result<()> {
    let mut parsers = std::collections::BTreeMap::new();
    let mut absent = Vec::new();
    for (family, digest) in parser_registry::parser_fingerprints() {
        match digest {
            Some(d) => {
                parsers.insert(family.to_string(), d.to_string());
            }
            None => absent.push(family.to_string()),
        }
    }
    let report = ParserFingerprintReport {
        pgen_parser_fingerprint_version: PARSER_FINGERPRINT_VERSION,
        parsers,
        absent,
    };
    println!("{}", serde_json::to_string(&report)?);
    Ok(())
}

#[cfg(not(feature = "generated_parsers"))]
fn command_parser_fingerprint() -> Result<()> {
    bail!("parseability_probe requires building with --features generated_parsers");
}

fn main() -> Result<()> {
    // `SV-CORPUS-GRAD.8c.3` — run the ENTIRE probe body on a dedicated
    // 256 MiB-stack thread so the generated parsers' 4096-frame recursion
    // ceiling provably fires (clean parse-failure diagnostic, exit 1) before
    // the OS guard page can SIGABRT the process (rc 134). Measured: the
    // ceiling needs ≈8 MB of real stack in release and ≈70 MB in debug for
    // SV, vs the 8 MB default main stack. Wrapping the whole body (not just
    // the parse call) keeps the thread-local trace/dump configuration and
    // the parse on the same thread.
    pgen::dedicated_parse_stack::run_cli_main_on_dedicated_parse_stack(
        "pgen-parseability-probe",
        probe_main,
    )
}

/// `ENGINE-UNIVERSAL-SERVICES.69`(a) — report the parse-stack reservation this process
/// resolved, AND WHERE IT CAME FROM.
///
/// ⛔ **WHY THIS EXISTS: A DERIVED DEFAULT NOBODY CAN OBSERVE IS NOT OPERABLE.** Before
/// `.69`(a) the reservation was a literal, so reading the source told an operator what their
/// host would use. It is now a function of that host's physical memory, and the only surface
/// that reported it — the `.69`(d) stack-probe line — appears ONLY on a refusal, i.e. only
/// after provoking a parse deep enough to contact the ceiling. That is minutes of work to
/// answer a question the process knows at startup, and it is the same shape as the gap
/// `.69`(d) itself closed one level down: the quantity existed and nothing emitted it.
///
/// Deterministic and load-independent — every field is an integer the process already holds.
fn command_stack_reservation() -> Result<()> {
    use pgen::dedicated_parse_stack as dps;
    let (reservation, source) = dps::resolved_parse_stack_reservation();
    let (source_name, host_memory) = match source {
        dps::ParseStackReservationSource::OperatorOverride => ("operator_override", None),
        dps::ParseStackReservationSource::HostMemoryDerived { host_memory_bytes } => {
            ("host_memory_derived", Some(host_memory_bytes))
        }
        dps::ParseStackReservationSource::ReferenceFallback => ("reference_fallback", None),
    };
    println!("reservation_bytes={reservation}");
    println!("reservation_mib={}", reservation / (1024 * 1024));
    println!("source={source_name}");
    match host_memory {
        Some(bytes) => {
            println!("host_memory_bytes={bytes}");
            println!("host_memory_mib={}", bytes / (1024 * 1024));
            // The UNCLAMPED share, so an operator can see whether the floor is what decided
            // their number rather than inferring it from the constants. ⭐ Since `.87`(d)
            // there is no upper clamp, so on any host past the floor this field EQUALS
            // `reservation_bytes` — and that equality is itself the readable statement that
            // the host is getting its full share. Kept (rather than dropped as redundant)
            // because it is the only field that shows the floor binding on a small host.
            println!(
                "unclamped_share_bytes={}",
                bytes / dps::PARSE_STACK_HOST_MEMORY_SHARE_DIVISOR
            );
        }
        None => {
            // ⛔ Absent, never `0` — the `.69`(d) rule that a zero byte field means NOT
            // MEASURED rather than measured-as-zero holds on this surface too.
            println!("host_memory_bytes=not_measured");
        }
    }
    println!(
        "share_divisor={}",
        dps::PARSE_STACK_HOST_MEMORY_SHARE_DIVISOR
    );
    println!("floor_bytes={}", dps::MIN_DEDICATED_PARSE_STACK_BYTES);
    println!("reference_bytes={}", dps::DEDICATED_PARSE_STACK_BYTES);
    Ok(())
}

fn probe_main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    // `ENGINE-UNIVERSAL-SERVICES.24` (ii′) — handled BEFORE the arity check, because it is the one
    // mode that takes no operand: it reports what this binary was built from, not what it parses.
    if args.len() >= 2 && args[1] == "--parser-fingerprint" {
        if args.len() != 2 {
            eprintln!("{}", usage());
            eprintln!();
            eprintln!(
                "--parser-fingerprint takes no arguments (it reports this binary's own build)"
            );
            std::process::exit(2);
        }
        return command_parser_fingerprint();
    }
    // `ENGINE-UNIVERSAL-SERVICES.69`(a) — same reason as the fingerprint above: it reports a
    // property of THIS PROCESS ON THIS HOST, not of an input, so it takes no operand.
    if args.len() >= 2 && args[1] == "--stack-reservation" {
        if args.len() != 2 {
            eprintln!("{}", usage());
            eprintln!();
            eprintln!(
                "--stack-reservation takes no arguments (it reports this host's own reservation)"
            );
            std::process::exit(2);
        }
        return command_stack_reservation();
    }
    if args.len() < 3 {
        eprintln!("{}", usage());
        std::process::exit(2);
    }

    // `.113`(b): a family's deprecated pre-.113 flag spellings are rewritten to the neutral flags
    // before parsing, ONE stderr line per old spelling used; the family is the parsed grammar.
    let args = {
        let family = args.get(2).cloned().unwrap_or_default();
        let (rewritten, notes) = preprocessing_stage::rewrite_legacy_args(&args, &family);
        for note in &notes {
            eprintln!("{note}");
        }
        rewritten
    };
    let (remaining, options) = strip_global_flags(&args[2..])?;
    configure_runtime_trace(&options)?;

    match args[1].as_str() {
        "--supports" => {
            if remaining.len() != 1 {
                eprintln!("{}", usage());
                std::process::exit(2);
            }
            command_supports(&remaining[0], options.profile.as_deref())
        }
        "--parse" => {
            if remaining.len() != 2 {
                eprintln!("{}", usage());
                std::process::exit(2);
            }
            command_parse(&remaining[0], &remaining[1], &options)
        }
        "--parse-dump-ast" => {
            if args.len() < 4 {
                eprintln!("{}", usage());
                std::process::exit(2);
            }
            if remaining.len() < 2 {
                eprintln!("{}", usage());
                std::process::exit(2);
            }
            let (output_file, cli_max_bytes) = match parse_dump_command_tail(&remaining[2..]) {
                Ok(values) => values,
                Err(err) => {
                    eprintln!("{}", usage());
                    eprintln!();
                    eprintln!("{}", err);
                    std::process::exit(2);
                }
            };
            let max_bytes = resolve_dump_max_bytes(cli_max_bytes)?;
            command_parse_dump_ast(
                &remaining[0],
                &remaining[1],
                output_file.as_deref(),
                false,
                max_bytes,
                &options,
            )
        }
        "--parse-dump-ast-pretty" => {
            if args.len() < 4 {
                eprintln!("{}", usage());
                std::process::exit(2);
            }
            if remaining.len() < 2 {
                eprintln!("{}", usage());
                std::process::exit(2);
            }
            let (output_file, cli_max_bytes) = match parse_dump_command_tail(&remaining[2..]) {
                Ok(values) => values,
                Err(err) => {
                    eprintln!("{}", usage());
                    eprintln!();
                    eprintln!("{}", err);
                    std::process::exit(2);
                }
            };
            let max_bytes = resolve_dump_max_bytes(cli_max_bytes)?;
            command_parse_dump_ast(
                &remaining[0],
                &remaining[1],
                output_file.as_deref(),
                true,
                max_bytes,
                &options,
            )
        }
        _ => {
            eprintln!("{}", usage());
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        GlobalOptions, ProbeInput, augment_rejection_with_origin, canonicalize_json_value,
        origin_of_position, parse_dump_command_tail, parse_positive_usize, rejection_position,
        strip_global_flags, write_json_dump_with_limit,
    };
        use pgen::preprocessing_stage::{SourceLocation, SourceMapEntry, StageOutput};
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_path(file_name: &str) -> PathBuf {
        // `DATA-LOCALITY.2`: the repository volume, derived at run time — never `$TMPDIR`.
        let mut path = pgen::repo_scratch::repo_scratch_dir("parseability_probe_test")
            .expect("repo scratch dir");
        let now_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "pgen_parseability_probe_{}_{}",
            now_nanos, file_name
        ));
        path
    }

    #[test]
    fn parse_dump_tail_accepts_output_and_max_bytes() {
        let args = vec![
            "out.json".to_string(),
            "--max-bytes".to_string(),
            "512".to_string(),
        ];
        let (output, max_bytes) = parse_dump_command_tail(&args).expect("tail parsing succeeds");
        assert_eq!(output.as_deref(), Some("out.json"));
        assert_eq!(max_bytes, Some(512));
    }

    #[test]
    fn parse_dump_tail_accepts_flag_only() {
        let args = vec!["--max-bytes".to_string(), "2048".to_string()];
        let (output, max_bytes) = parse_dump_command_tail(&args).expect("tail parsing succeeds");
        assert_eq!(output, None);
        assert_eq!(max_bytes, Some(2048));
    }

    #[test]
    fn parse_dump_tail_rejects_duplicate_max_bytes() {
        let args = vec![
            "--max-bytes".to_string(),
            "10".to_string(),
            "--max-bytes".to_string(),
            "20".to_string(),
        ];
        let err = parse_dump_command_tail(&args).expect_err("duplicate max-bytes must fail");
        assert!(
            err.to_string()
                .contains("cannot be specified multiple times")
        );
    }

    #[test]
    fn parse_positive_usize_rejects_zero() {
        let err = parse_positive_usize("0", "label").expect_err("zero must fail");
        assert!(err.to_string().contains(">= 1"));
    }

    #[test]
    fn strip_global_flags_extracts_profile_and_trace_flags() {
        let args = vec![
            "systemverilog".to_string(),
            "sample.sv".to_string(),
            "--profile".to_string(),
            "2017".to_string(),
            "--trace".to_string(),
            "--trace-log-file".to_string(),
            "trace.out".to_string(),
        ];
        let (remaining, options) =
            strip_global_flags(&args).expect("global flags should parse successfully");
        assert_eq!(
            remaining,
            vec!["systemverilog".to_string(), "sample.sv".to_string()]
        );
        assert_eq!(
            options,
            GlobalOptions {
                profile: Some("2017".to_string()),
                trace: true,
                trace_log_file: Some("trace.out".to_string()),
                ..GlobalOptions::default()
            }
        );
    }

    #[test]
    fn strip_global_flags_defaults_trace_log_file_name() {
        let args = vec!["--trace-log-file".to_string()];
        let (remaining, options) =
            strip_global_flags(&args).expect("trace-log-file should parse successfully");
        assert!(remaining.is_empty());
        assert_eq!(
            options,
            GlobalOptions {
                profile: None,
                trace: false,
                trace_log_file: Some("trace.log".to_string()),
                ..GlobalOptions::default()
            }
        );
    }

    #[test]
    fn canonicalize_json_value_sorts_keys_recursively() {
        let value = serde_json::json!({
            "z": { "b": 1, "a": 2 },
            "a": [ { "y": 0, "x": 1 } ],
        });
        let normalized = canonicalize_json_value(value);
        let encoded = serde_json::to_string(&normalized).expect("encode normalized");
        assert!(encoded.contains("\"a\":[{\"x\":1,\"y\":0}]"));
        assert!(encoded.contains("\"z\":{\"a\":2,\"b\":1}"));
    }

    #[test]
    fn write_json_dump_with_limit_emits_truncation_envelope() {
        let path = unique_temp_path("ast_dump.json");
        let large_payload = format!("{{\"payload\":\"{}\"}}", "x".repeat(4096));
        let result = write_json_dump_with_limit(
            path.to_str().expect("path"),
            large_payload.as_str(),
            Some(256),
            false,
            "parser_return_ast",
        )
        .expect("bounded write succeeds");
        assert!(result.truncated);
        assert!(result.full_bytes > 256);
        let raw = std::fs::read_to_string(&path).expect("read output");
        let json: serde_json::Value = serde_json::from_str(&raw).expect("json parse");
        assert_eq!(json["kind"], "pgen_ast_dump_truncation");
        assert_eq!(json["dump_kind"], "parser_return_ast");
        assert_eq!(json["max_bytes"], 256);
        let _ = std::fs::remove_file(path);
    }

    // ── SVPP-EXPANSION.3 — the preprocess→parse chain's flags and its origin mapping ──────────

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn preprocess_flags_are_parsed_in_order_and_need_preprocess() {
        let (remaining, options) = strip_global_flags(&args(&[
            "systemverilog",
            "top.sv",
            "--preprocess",
            "--include-dir",
            "a",
            "--symbol",
            "UVM",
            "--include-dir",
            "b",
            "--define",
            "W=8",
            "--unit-file",
            "first.svh",
            "--stage-opt",
            "plain_string_substitution=all",
        ]))
        .expect("flags");
        assert_eq!(remaining, args(&["systemverilog", "top.sv"]));
        assert!(options.preprocess);
        assert_eq!(options.include_dirs, args(&["a", "b"]));
        assert_eq!(options.symbols, args(&["UVM", "W=8"]), "--symbol and its alias --define feed one ordered list");
        assert_eq!(options.unit_files, args(&["first.svh"]));
        assert_eq!(
            options.stage_opts,
            vec![("plain_string_substitution".to_string(), "all".to_string())]
        );

        let refused = strip_global_flags(&args(&["systemverilog", "top.sv", "--symbol", "UVM"]));
        assert!(
            refused.is_err(),
            "a context flag without --preprocess is refused"
        );
        let refused = strip_global_flags(&args(&[
            "systemverilog",
            "top.sv",
            "--preprocess",
            "--preprocess",
        ]));
        assert!(refused.is_err(), "--preprocess twice is refused");
        let refused = strip_global_flags(&args(&[
            "systemverilog",
            "top.sv",
            "--preprocess",
            "--symbol",
        ]));
        assert!(refused.is_err(), "a context flag needs a value");
    }

    /// `.113`(b): the pre-.113 spellings a family declares are rewritten to the neutral flags
    /// (one deprecation note per distinct old spelling), and a family that declares none — or
    /// no family at all — leaves the line untouched.
    #[test]
    fn legacy_family_spellings_are_rewritten_before_the_flags_are_read() {
        let line = args(&[
            "probe",
            "--parse",
            "systemverilog",
            "top.sv",
            "--preprocess",
            "--sv-include-dir",
            "a",
            "--sv-define",
            "UVM",
            "--sv-unit-file",
            "first.svh",
            "--sv-plain-string-substitution",
            "all",
            "--sv-define",
            "W=8",
        ]);
        let (rewritten, notes) = pgen::preprocessing_stage::rewrite_legacy_args(&line, "systemverilog");
        assert_eq!(
            rewritten,
            args(&[
                "probe",
                "--parse",
                "systemverilog",
                "top.sv",
                "--preprocess",
                "--include-dir",
                "a",
                "--symbol",
                "UVM",
                "--unit-file",
                "first.svh",
                "--stage-opt",
                "plain_string_substitution=all",
                "--symbol",
                "W=8",
            ])
        );
        assert_eq!(notes.len(), 4, "one note per DISTINCT old spelling: {notes:?}");
        let (remaining, options) = strip_global_flags(&rewritten[2..]).expect("flags");
        assert_eq!(remaining, args(&["systemverilog", "top.sv"]));
        assert_eq!(options.symbols, args(&["UVM", "W=8"]));
        let (untouched, notes) = pgen::preprocessing_stage::rewrite_legacy_args(&line, "json");
        assert_eq!(untouched, line);
        assert!(notes.is_empty());
    }

    fn synthetic_output() -> StageOutput {
        // two chunks: file a line 1 = bytes 0..10, file b line 7 = bytes 10..21 (an `include`d line)
        let chunk = |output_start, output_end, file: &str, line| SourceMapEntry {
            output_start,
            output_end,
            source: SourceLocation {
                file: file.to_string(),
                line,
                column_start: 1,
                column_end: 10,
            },
        };
        StageOutput {
            text: "0123456789abcdéfghi\n".to_string(), // the second chunk holds a two-byte `é`
            source_map: vec![chunk(0, 10, "/x/a.sv", 1), chunk(10, 21, "/x/b.svh", 7)],
            events: Vec::new(),
            diagnostics: Vec::new(),
            included_files: Vec::new(),
        }
    }

    #[test]
    fn origin_maps_a_position_to_its_chunks_file_line_and_character_column() {
        let output = synthetic_output();
        assert_eq!(
            origin_of_position(&output, 0).as_deref(),
            Some("/x/a.sv:1:1")
        );
        assert_eq!(
            origin_of_position(&output, 9).as_deref(),
            Some("/x/a.sv:1:10")
        );
        assert_eq!(
            origin_of_position(&output, 10).as_deref(),
            Some("/x/b.svh:7:1")
        );
        assert_eq!(
            origin_of_position(&output, 13).as_deref(),
            Some("/x/b.svh:7:4")
        );
        // byte 16 sits AFTER the two-byte `é` (bytes 14-15): 5 characters in, so column 6 — chars, not bytes
        assert_eq!(
            origin_of_position(&output, 16).as_deref(),
            Some("/x/b.svh:7:6")
        );
        // at / past the end of the text: the last chunk's end
        assert_eq!(
            origin_of_position(&output, 21).as_deref(),
            Some("/x/b.svh:7:11")
        );
        assert_eq!(
            origin_of_position(&output, 999).as_deref(),
            Some("/x/b.svh:7:11")
        );
        let empty = StageOutput {
            text: String::new(),
            source_map: Vec::new(),
            events: Vec::new(),
            diagnostics: Vec::new(),
            included_files: Vec::new(),
        };
        assert_eq!(origin_of_position(&empty, 0), None);
    }

    #[test]
    fn rejection_position_prefers_furthest_over_surface() {
        assert_eq!(
            rejection_position(
                "Parser did not consume full input at position 171 [furthest_position=8829, +8658 bytes deeper than surface position]"
            ),
            Some(8829)
        );
        assert_eq!(rejection_position("failed at position 12"), Some(12));
        assert_eq!(rejection_position("no position here"), None);
    }

    #[test]
    fn a_raw_rejection_is_untouched_and_a_preprocessed_one_carries_its_origin() {
        let message =
            "x at position 3 [furthest_position=13, +10 bytes deeper than surface position]"
                .to_string();
        let raw = ProbeInput::Raw("irrelevant".to_string());
        assert_eq!(
            augment_rejection_with_origin(message.clone(), &raw),
            message
        );
        let pre = ProbeInput::Preprocessed(synthetic_output());
        assert_eq!(
            augment_rejection_with_origin(message.clone(), &pre),
            format!("{} [origin=/x/b.svh:7:4]", message)
        );
        let unmapped = ProbeInput::Preprocessed(StageOutput {
            text: String::new(),
            source_map: Vec::new(),
            events: Vec::new(),
            diagnostics: Vec::new(),
            included_files: Vec::new(),
        });
        assert_eq!(
            augment_rejection_with_origin("no position".to_string(), &unmapped),
            "no position [origin=unknown]"
        );
    }
}
