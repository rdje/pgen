use anyhow::{Context, Result, bail};

#[cfg(feature = "generated_parsers")]
use pgen::parser_registry;

fn usage() -> &'static str {
    "Usage:\n  parseability_probe --supports <grammar_name>\n  parseability_probe --parse <grammar_name> <input_file>\n  parseability_probe --parse-dump-ast <grammar_name> <input_file> [output_file]\n  parseability_probe --parse-dump-ast-pretty <grammar_name> <input_file> [output_file]\n\nDefault AST dump filename (when output_file omitted): <grammar_name>_ast.json"
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
fn command_supports(grammar_name: &str) -> Result<()> {
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
fn command_supports(grammar_name: &str) -> Result<()> {
    let _ = grammar_name;
    bail!("parseability_probe requires building with --features generated_parsers");
}

#[cfg(feature = "generated_parsers")]
fn command_parse(grammar_name: &str, input_file: &str) -> Result<()> {
    let sample = std::fs::read_to_string(input_file)
        .with_context(|| format!("failed to read input file '{}'", input_file))?;
    match parser_registry::parse_sample(grammar_name, &sample) {
        Some(true) => {
            println!(
                "parse_full passed for grammar '{}' on '{}'",
                grammar_name, input_file
            );
            Ok(())
        }
        Some(false) => bail!(
            "parse_full rejected sample for grammar '{}' on '{}'",
            grammar_name,
            input_file
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
    pretty: bool,
) -> Result<()> {
    let sample = std::fs::read_to_string(input_file)
        .with_context(|| format!("failed to read input file '{}'", input_file))?;
    let parse_result =
        parser_registry::parse_sample_ast_json(grammar_name, &sample).ok_or_else(|| {
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
    let encoded = if pretty {
        serde_json::to_string_pretty(&ast_json)?
    } else {
        serde_json::to_string(&ast_json)?
    };
    std::fs::write(&resolved_output_path, encoded).with_context(|| {
        format!(
            "failed to write parser AST log '{}'",
            resolved_output_path.as_str()
        )
    })?;
    println!(
        "parse_full passed for grammar '{}' on '{}' (AST dump: {})",
        grammar_name,
        input_file,
        resolved_output_path.as_str()
    );
    Ok(())
}

#[cfg(not(feature = "generated_parsers"))]
fn command_parse_dump_ast(
    grammar_name: &str,
    input_file: &str,
    output_file: Option<&str>,
    pretty: bool,
) -> Result<()> {
    let _ = (grammar_name, input_file, output_file, pretty);
    bail!("parseability_probe requires building with --features generated_parsers");
}

#[cfg(not(feature = "generated_parsers"))]
fn command_parse(grammar_name: &str, input_file: &str) -> Result<()> {
    let _ = (grammar_name, input_file);
    bail!("parseability_probe requires building with --features generated_parsers");
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("{}", usage());
        std::process::exit(2);
    }

    match args[1].as_str() {
        "--supports" => {
            if args.len() != 3 {
                eprintln!("{}", usage());
                std::process::exit(2);
            }
            command_supports(&args[2])
        }
        "--parse" => {
            if args.len() != 4 {
                eprintln!("{}", usage());
                std::process::exit(2);
            }
            command_parse(&args[2], &args[3])
        }
        "--parse-dump-ast" => {
            if args.len() != 4 && args.len() != 5 {
                eprintln!("{}", usage());
                std::process::exit(2);
            }
            let output_file = if args.len() == 5 {
                Some(args[4].as_str())
            } else {
                None
            };
            command_parse_dump_ast(&args[2], &args[3], output_file, false)
        }
        "--parse-dump-ast-pretty" => {
            if args.len() != 4 && args.len() != 5 {
                eprintln!("{}", usage());
                std::process::exit(2);
            }
            let output_file = if args.len() == 5 {
                Some(args[4].as_str())
            } else {
                None
            };
            command_parse_dump_ast(&args[2], &args[3], output_file, true)
        }
        _ => {
            eprintln!("{}", usage());
            std::process::exit(2);
        }
    }
}
