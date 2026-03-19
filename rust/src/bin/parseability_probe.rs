use anyhow::{Context, Result, bail};
use serde::Serialize;

#[cfg(feature = "generated_parsers")]
use pgen::parser_registry;
use pgen::ast_pipeline::{configure_trace_output, resolve_trace_verbosity, set_global_trace_verbosity};

fn usage() -> &'static str {
    "Usage:\n  parseability_probe --supports <grammar_name> [--profile PROFILE] [--trace] [--trace-log-file [FILE]]\n  parseability_probe --parse <grammar_name> <input_file> [--profile PROFILE] [--trace] [--trace-log-file [FILE]]\n  parseability_probe --parse-dump-ast <grammar_name> <input_file> [output_file] [--profile PROFILE] [--max-bytes N] [--trace] [--trace-log-file [FILE]]\n  parseability_probe --parse-dump-ast-pretty <grammar_name> <input_file> [output_file] [--profile PROFILE] [--max-bytes N] [--trace] [--trace-log-file [FILE]]\n\nDefault AST dump filename (when output_file omitted): <grammar_name>_ast.json\nOptional env fallback for dump-size bound: PGEN_PARSE_DUMP_AST_MAX_BYTES\nOptional env fallback for trace verbosity: PGEN_TRACE_VERBOSITY"
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
        remaining.push(args[idx].clone());
        idx += 1;
    }
    Ok((remaining, options))
}

fn configure_runtime_trace(options: &GlobalOptions) -> Result<()> {
    configure_trace_output(options.trace_log_file.as_deref())?;
    let verbosity = resolve_trace_verbosity(None, false, options.trace)?;
    set_global_trace_verbosity(verbosity);
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
fn command_parse(grammar_name: &str, input_file: &str, profile: Option<&str>) -> Result<()> {
    let sample = std::fs::read_to_string(input_file)
        .with_context(|| format!("failed to read input file '{}'", input_file))?;
    match parser_registry::parse_sample_detail_with_profile(grammar_name, &sample, profile) {
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
fn command_parse(grammar_name: &str, input_file: &str, profile: Option<&str>) -> Result<()> {
    let _ = (grammar_name, input_file, profile);
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
            command_parse(&remaining[0], &remaining[1], options.profile.as_deref())
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
