use anyhow::{Context, Result, bail};

#[cfg(feature = "generated_parsers")]
use pgen::parser_registry;

fn usage() -> &'static str {
    "Usage:\n  generated_parse_probe --supports <grammar_name>\n  generated_parse_probe --parse <grammar_name> <input_file>"
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
    bail!("generated_parse_probe requires building with --features generated_parsers");
}

#[cfg(feature = "generated_parsers")]
fn command_parse(grammar_name: &str, input_file: &str) -> Result<()> {
    // SV-CORPUS-GRAD.12c.1 — USER SOURCE TEXT goes through the shared decoder, so a Latin-1 or
    // UTF-16 file is parsed rather than refused before the parser ever sees it.
    let decoded = pgen::source_text::read_source_file(input_file)
        .with_context(|| format!("failed to read input file '{}'", input_file))?;
    if let Some(notice) = decoded.non_utf8_notice(input_file) {
        eprintln!("{notice}");
    }
    let sample = decoded.text;
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

#[cfg(not(feature = "generated_parsers"))]
fn command_parse(grammar_name: &str, input_file: &str) -> Result<()> {
    let _ = (grammar_name, input_file);
    bail!("generated_parse_probe requires building with --features generated_parsers");
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
        _ => {
            eprintln!("{}", usage());
            std::process::exit(2);
        }
    }
}
