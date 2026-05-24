use anyhow::{Context, Result, bail};
use serde::Serialize;

use pgen::ast_pipeline::{
    configure_trace_output, resolve_trace_verbosity, set_global_trace_verbosity,
};
#[cfg(feature = "generated_parsers")]
use pgen::parser_registry;

fn usage() -> &'static str {
    "Usage:\n  parseability_probe --supports <grammar_name> [--profile PROFILE] [--trace] [--trace-rules R1,R2,...] [--trace-log-file [FILE]] [--dump-rule-call-counts]\n  parseability_probe --parse <grammar_name> <input_file> [--profile PROFILE] [--lib-in DIR] [--lib-out DIR] [--trace] [--trace-rules R1,R2,...] [--trace-log-file [FILE]] [--dump-rule-call-counts]\n  parseability_probe --parse-dump-ast <grammar_name> <input_file> [output_file] [--profile PROFILE] [--max-bytes N] [--lib-in DIR] [--lib-out DIR] [--trace] [--trace-rules R1,R2,...] [--trace-log-file [FILE]] [--dump-rule-call-counts]\n  parseability_probe --parse-dump-ast-pretty <grammar_name> <input_file> [output_file] [--profile PROFILE] [--max-bytes N] [--lib-in DIR] [--lib-out DIR] [--trace] [--trace-rules R1,R2,...] [--trace-log-file [FILE]] [--dump-rule-call-counts]\n\nDefault AST dump filename (when output_file omitted): <grammar_name>_ast.json\nOptional env fallback for dump-size bound: PGEN_PARSE_DUMP_AST_MAX_BYTES\nOptional env fallback for trace verbosity: PGEN_TRACE_VERBOSITY\n--lib-in DIR              : (`SV-EXH-PROOF.3.3.4.a` MVP-0) directory artifacts are READ from for `@import_from_library`.\n--lib-out DIR             : (`SV-EXH-PROOF.3.3.4.a` MVP-0) directory artifacts are WRITTEN to for `@export_to_library`.\n--trace-rules             : (`SV-EXH-PROOF.3.3.4.b.6.2.17`) comma-separated rule-name list. Trace activates ONLY inside the call-tree of these rules (implies --trace). Reduces trace volume 100-1000× vs --trace for targeted investigation.\n--dump-rule-call-counts   : (`SV-EXH-PROOF.3.3.4.b.6.2.22`) live per-rule call-count dashboard. Each rule's call counter is incremented on every entry; the top-20 rules sorted by count are shown on stderr and updated every 250ms in place. Use to identify which rules dominate a stuck or slow parse; works on timeout (dashboard keeps refreshing until the process is killed). Accepts an optional integer arg to control the top-N (default 20).\n--dump-rule-call-counts-exclude R1,R2,... : (`SV-EXH-PROOF.3.3.4.b.6.2.22`) filter these rules OUT of the dashboard before computing the top-N. Use to hide always-dominant noise like `trivia` (whitespace handling) so the diagnostically interesting rules win display slots."
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
            let value = args
                .get(idx + 1)
                .ok_or_else(|| anyhow::anyhow!("--trace-rules requires a comma-separated rule-name list"))?;
            let rules: Vec<String> = value.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
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
                anyhow::anyhow!("--dump-rule-call-counts-exclude requires a comma-separated rule-name list")
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
    Ok((remaining, options))
}

fn configure_runtime_trace(options: &GlobalOptions) -> Result<()> {
    configure_trace_output(options.trace_log_file.as_deref())?;
    let verbosity = resolve_trace_verbosity(None, false, options.trace)?;
    set_global_trace_verbosity(verbosity);
    // SV-EXH-PROOF.3.3.4.b.6.2.17 — propagate rule-level trace filter to the
    // parser_registry thread-local so subsequent parser invocations on this
    // thread apply it via `set_trace_rules` after construction.
    let rules = options.trace_rules.as_ref().map(|v| v.iter().cloned().collect());
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
fn command_parse(
    grammar_name: &str,
    input_file: &str,
    profile: Option<&str>,
    library_in_dir: Option<&str>,
    library_out_dir: Option<&str>,
) -> Result<()> {
    let sample = std::fs::read_to_string(input_file)
        .with_context(|| format!("failed to read input file '{}'", input_file))?;
    let library_options = parser_registry::LibraryOptions {
        in_dir: library_in_dir.map(std::path::PathBuf::from),
        out_dir: library_out_dir.map(std::path::PathBuf::from),
    };
    let result = parser_registry::parse_sample_detail_with_options(
        grammar_name,
        &sample,
        profile,
        &library_options,
    );
    match result {
        Some(Ok(())) => {
            println!(
                "parse_full passed for grammar '{}' on '{}'",
                grammar_name, input_file
            );
            Ok(())
        }
        Some(Err(err)) => bail!(
            "parse_full rejected sample for grammar '{}' on '{}': {}",
            grammar_name,
            input_file,
            err
        ),
        None => bail!(
            "parseability adapter unavailable for grammar '{}'. Supported grammars: {}",
            grammar_name,
            supported_grammars_csv()
        ),
    }
}

#[cfg(feature = "generated_parsers")]
fn command_parse_dump_ast(
    grammar_name: &str,
    input_file: &str,
    output_file: Option<&str>,
    profile: Option<&str>,
    pretty: bool,
    max_bytes: Option<usize>,
) -> Result<()> {
    let sample = std::fs::read_to_string(input_file)
        .with_context(|| format!("failed to read input file '{}'", input_file))?;
    let parse_result =
        parser_registry::parse_sample_ast_json_with_profile(grammar_name, &sample, profile)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "parseability adapter unavailable for grammar '{}'. Supported grammars: {}",
                    grammar_name,
                    supported_grammars_csv()
                )
            })?;
    let ast_json = parse_result.map_err(|err| {
        anyhow::anyhow!(
            "parse_full rejected sample for grammar '{}' on '{}': {}",
            grammar_name,
            input_file,
            err
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
    profile: Option<&str>,
    pretty: bool,
    max_bytes: Option<usize>,
) -> Result<()> {
    let _ = (
        grammar_name,
        input_file,
        output_file,
        profile,
        pretty,
        max_bytes,
    );
    bail!("parseability_probe requires building with --features generated_parsers");
}

#[cfg(not(feature = "generated_parsers"))]
fn command_parse(
    grammar_name: &str,
    input_file: &str,
    profile: Option<&str>,
    library_in_dir: Option<&str>,
    library_out_dir: Option<&str>,
) -> Result<()> {
    let _ = (
        grammar_name,
        input_file,
        profile,
        library_in_dir,
        library_out_dir,
    );
    bail!("parseability_probe requires building with --features generated_parsers");
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("{}", usage());
        std::process::exit(2);
    }

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
            command_parse(
                &remaining[0],
                &remaining[1],
                options.profile.as_deref(),
                options.library_in_dir.as_deref(),
                options.library_out_dir.as_deref(),
            )
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
                options.profile.as_deref(),
                false,
                max_bytes,
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
                options.profile.as_deref(),
                true,
                max_bytes,
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
        GlobalOptions, canonicalize_json_value, parse_dump_command_tail, parse_positive_usize,
        strip_global_flags, write_json_dump_with_limit,
    };
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_path(file_name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
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
}
