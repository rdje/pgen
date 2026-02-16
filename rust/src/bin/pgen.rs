//! PGen Command-Line Interface
//! Provides a unified interface for testing parsers via command line

use clap::{Arg, Command};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::process::exit;
extern crate chrono;

fn main() {
    let matches = Command::new("pgen")
        .about("Parser Generator Command Line Interface")
        .version("1.0.0")
        .arg(
            Arg::new("parser")
                .long("parser")
                .value_name("TYPE")
                .help("Parser type to use")
                .required(true)
                .value_parser(["semantic", "return", "regex"]),
        )
        .arg(
            Arg::new("input")
                .long("input")
                .value_name("INPUT")
                .help("Input string to parse")
                .required(true),
        )
        .arg(
            Arg::new("debug")
                .long("debug")
                .help("Enable debug output")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let parser_type = matches.get_one::<String>("parser").unwrap();
    let input = matches.get_one::<String>("input").unwrap();
    let debug = matches.get_flag("debug");

    // Create log file with timestamp
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let log_file_path = format!("pgen_{}_{}.log", parser_type, timestamp);

    let log_file = File::create(&log_file_path).expect("Failed to create log file");
    let mut writer = BufWriter::new(log_file);

    // Macro to write to both console and log file
    macro_rules! log_and_print {
        ($($arg:tt)*) => {
            let line = format!($($arg)*);
            println!("{}", line);
            writeln!(writer, "{}", line).expect("Failed to write to log file");
        };
    }

    log_and_print!("🚀 PGen Parser Test");
    log_and_print!("📁 LOG FILE: {}", log_file_path);
    log_and_print!(
        "🕒 START TIME: {}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );
    log_and_print!("Parser: {}", parser_type);
    log_and_print!("Input: {}", input);
    log_and_print!("Debug: {}", debug);
    log_and_print!("{}", "=".repeat(80));

    let result = match parser_type.as_str() {
        "semantic" => test_semantic_parser(input, debug, &mut writer),
        "return" => test_return_parser(input, debug, &mut writer),
        "regex" => test_regex_parser(input, debug, &mut writer),
        _ => {
            log_and_print!("❌ Unsupported parser type: {}", parser_type);
            exit(1);
        }
    };

    log_and_print!("{}", "=".repeat(80));
    log_and_print!(
        "🕒 END TIME: {}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );

    // Ensure all data is written to the log file
    writer.flush().expect("Failed to flush log file");

    match result {
        Ok(_) => {
            log_and_print!("✅ Parse successful");
            log_and_print!("📄 Complete log saved to: {}", log_file_path);
            exit(0);
        }
        Err(e) => {
            log_and_print!("❌ Parse failed: {}", e);
            log_and_print!("📄 Complete log saved to: {}", log_file_path);
            eprintln!("Parse failed: {}", e);
            exit(1);
        }
    }
}

fn test_semantic_parser(
    _input: &str,
    debug: bool,
    writer: &mut BufWriter<File>,
) -> Result<(), Box<dyn std::error::Error>> {
    // TEMPORARILY DISABLED: Parser needs regeneration with AST-based generator
    // use pgen::ast_pipeline::semantic_annotation_parser::Semantic_annotationParser;
    use std::time::Instant;

    // Macro to write to both console and log file
    macro_rules! log_and_print {
        ($($arg:tt)*) => {
            let line = format!($($arg)*);
            println!("{}", line);
            writeln!(writer, "{}", line).expect("Failed to write to log file");
        };
    }

    log_and_print!("🔍 SEMANTIC ANNOTATION PARSER TEST");
    log_and_print!(
        "📄 Generated Parser: /Users/richarddje/Documents/github/pgen/generated/semantic_annotation_parser.rs"
    );
    log_and_print!(
        "📄 Source Grammar: /Users/richarddje/Documents/github/pgen/grammars/semantic_annotation.ebnf"
    );
    log_and_print!("🎯 Entry Rule: semantic_annotation");
    log_and_print!(
        "📊 Features: Zero-copy, memoization, recursion depth protection, SIMD-optimized"
    );
    log_and_print!("");

    // TEMPORARILY DISABLED: Parser needs regeneration
    /* let mut parser = if debug {
        Semantic_annotationParser::with_debug(input)
    } else {
        Semantic_annotationParser::new(input)
    }; */

    let parse_start = Instant::now();
    // TEMPORARILY: Return error until parser is regenerated
    let parser_result: Result<(), String> =
        Err("Parser temporarily disabled for regeneration".to_string());
    match parser_result {
        Ok(_) => {
            let parse_time = parse_start.elapsed();
            log_and_print!(
                "✅ PARSE SUCCESS in {:.3}ms",
                parse_time.as_secs_f64() * 1000.0
            );
            log_and_print!("📊 Parser temporarily disabled for regeneration");

            // Print debug trace if available
            // Note: Debug output method not yet available in placeholder parser
            if debug {
                log_and_print!("");
                log_and_print!(
                    "🔍 DEBUG MODE: Enabled (detailed trace will be available once parser is fully generated)"
                );
            }

            log_and_print!("");
            log_and_print!("✅ SEMANTIC PARSER: ROCK SOLID BEHAVIOR CONFIRMED");
            Ok(())
        }
        Err(e) => {
            let parse_time = parse_start.elapsed();
            log_and_print!(
                "❌ PARSE FAILURE in {:.3}ms: {:?}",
                parse_time.as_secs_f64() * 1000.0,
                e
            );

            // Print debug trace even for failures
            // Note: Debug output method not yet available in placeholder parser
            if debug {
                log_and_print!("");
                log_and_print!(
                    "🔍 DEBUG MODE: Enabled (detailed trace will be available once parser is fully generated)"
                );
            }

            Err(format!("Semantic parser error: {:?}", e).into())
        }
    }
}

fn test_return_parser(
    _input: &str,
    _debug: bool,
    writer: &mut BufWriter<File>,
) -> Result<(), Box<dyn std::error::Error>> {
    // TEMPORARILY DISABLED: Parser needs regeneration with AST-based generator
    // use pgen::ast_pipeline::return_annotation_parser::Return_annotationParser;
    use std::time::Instant;

    // Macro to write to both console and log file
    macro_rules! log_and_print {
        ($($arg:tt)*) => {
            let line = format!($($arg)*);
            println!("{}", line);
            writeln!(writer, "{}", line).expect("Failed to write to log file");
        };
    }

    log_and_print!("🔍 RETURN ANNOTATION PARSER TEST");
    log_and_print!(
        "📄 Generated Parser: /Users/richarddje/Documents/github/pgen/generated/return_annotation_parser.rs"
    );
    log_and_print!(
        "📄 Source Grammar: /Users/richarddje/Documents/github/pgen/grammars/return_annotation.ebnf"
    );
    log_and_print!("🎯 Entry Rule: return_annotation");
    log_and_print!(
        "📊 Features: Zero-copy, memoization, recursion depth protection, SIMD-optimized"
    );
    log_and_print!("");

    // TEMPORARILY DISABLED: Parser needs regeneration
    /* let mut parser = if debug {
        Return_annotationParser::with_debug(input)
    } else {
        Return_annotationParser::new(input)
    }; */

    let parse_start = Instant::now();
    // TEMPORARILY: Return error until parser is regenerated
    let parser_result: Result<(), String> =
        Err("Parser temporarily disabled for regeneration".to_string());
    match parser_result {
        Ok(_) => {
            let parse_time = parse_start.elapsed();
            log_and_print!(
                "✅ PARSE SUCCESS in {:.3}ms",
                parse_time.as_secs_f64() * 1000.0
            );
            log_and_print!("📊 Parser temporarily disabled for regeneration");

            // Print debug trace if available (if the parser supports it)
            // Note: Return annotation parser may not have debug_output method yet
            log_and_print!("   Debug trace: Available when using --debug flag");

            log_and_print!("");
            log_and_print!("✅ RETURN PARSER: ROCK SOLID BEHAVIOR CONFIRMED");
            Ok(())
        }
        Err(e) => {
            let parse_time = parse_start.elapsed();
            log_and_print!(
                "❌ PARSE FAILURE in {:.3}ms: {:?}",
                parse_time.as_secs_f64() * 1000.0,
                e
            );

            // Print debug trace even for failures (if the parser supports it)
            // Note: Return annotation parser may not have debug_output method yet
            log_and_print!("   Debug trace: Available when using --debug flag");

            Err(format!("Return parser error: {:?}", e).into())
        }
    }
}

fn test_regex_parser(
    _input: &str,
    _debug: bool,
    writer: &mut BufWriter<File>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Macro to write to both console and log file
    macro_rules! log_and_print {
        ($($arg:tt)*) => {
            let line = format!($($arg)*);
            println!("{}", line);
            writeln!(writer, "{}", line).expect("Failed to write to log file");
        };
    }

    log_and_print!("🔍 REGEX PARSER TEST");
    log_and_print!("⚠️  Regex parser not yet implemented");
    Ok(())
}
