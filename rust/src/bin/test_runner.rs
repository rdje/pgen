#!/usr/bin/env rust
// Universal Test Runner CLI
// Run all tests, filter by parser, or filter by tags

use clap::{Command, Arg};
use pgen::test_runner::{RoundTripTestRunner, TestSuite, Report, UniversalTestRunner, Parser, Logger, FileLogger};
use pgen::test_runner::parsers::{ReturnAnnotationParser, SemanticAnnotationParser};
use std::process::exit;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;
use lazy_static::lazy_static;
use chrono::Utc;

lazy_static! {
    static ref LOG_FILE: Mutex<Option<std::fs::File>> = Mutex::new(None);
    static ref CURRENT_LOG_PATH: Mutex<Option<String>> = Mutex::new(None);
}

fn log_output(message: &str) {
    // Print to console
    println!("{}", message);
    
    // Write to log file if available
    if let Ok(mut file_guard) = LOG_FILE.lock() {
        if let Some(ref mut file) = *file_guard {
            let _ = writeln!(file, "{}", message);
        }
    }
}

fn log_error(message: &str) {
    // Print to stderr
    eprintln!("{}", message);
    
    // Write to log file if available
    if let Ok(mut file_guard) = LOG_FILE.lock() {
        if let Some(ref mut file) = *file_guard {
            let _ = writeln!(file, "{}", message);
        }
    }
}

fn get_current_log_file_path() -> Result<String, Box<dyn std::error::Error>> {
    CURRENT_LOG_PATH.lock()
        .unwrap()
        .as_ref()
        .cloned()
        .ok_or_else(|| "No log file path set".into())
}

fn setup_logging(log_path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let log_file_path = match log_path {
        Some(path) => path.to_string(),
        None => {
            let now = Utc::now();
            format!("test_runner_{}.log", now.format("%Y%m%d_%H%M%S"))
        }
    };

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)?;
    
    *LOG_FILE.lock().unwrap() = Some(file);
    *CURRENT_LOG_PATH.lock().unwrap() = Some(log_file_path);
    Ok(())
}

fn main() {
    let matches = Command::new("test_runner")
        .about("Universal Test Runner for pgen")
        .version("1.0.0")
        .arg(
            Arg::new("parser")
                .short('p')
                .long("parser")
                .value_name("TYPE")
                .help("Filter tests by parser type")
                .value_parser(["return", "semantic", "regex", "all"])
        )
        .arg(
            Arg::new("tags")
                .short('t')
                .long("tags")
                .value_name("TAGS")
                .help("Filter tests by tags (comma-separated)")
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Show detailed output")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("suite")
                .short('s')
                .long("suite")
                .value_name("NAME")
                .help("Run specific test suite by name")
        )
        .arg(
            Arg::new("list")
                .short('l')
                .long("list")
                .help("List available test suites without running")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("dashboard")
                .short('d')
                .long("dashboard")
                .help("Show comprehensive dashboard output (like stress tests)")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("log_file")
                .short('L')
                .long("log-file")
                .help("Path to log file (default: test_runner_YYYYMMDD_HHMMSS.log in current directory)")
                .value_name("PATH")
        )
        .arg(
            Arg::new("debug")
                .short('D')
                .long("debug")
                .help("Enable debug logging for parsers")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    let verbose = matches.get_flag("verbose");
    let list_only = matches.get_flag("list");
    let debug_enabled = matches.get_flag("debug");

    // Setup logging
    let log_file_path = matches.get_one::<String>("log_file").map(|s| s.as_str());
    if let Err(e) = setup_logging(log_file_path) {
        eprintln!("Failed to setup logging: {}", e);
        exit(1);
    }

    // Create runner with options
    let mut runner = UniversalTestRunner::new().with_verbose(verbose);

    // Select parser based on filter if specified
    if let Some(parser_type) = matches.get_one::<String>("parser") {
        match parser_type.as_str() {
            "return" => {
                let mut parser = ReturnAnnotationParser::new();
                if debug_enabled {
                    // Create a duplicate file handle for the parser logger
                    if let Ok(log_file_path) = get_current_log_file_path() {
                        if let Ok(file) = OpenOptions::new().append(true).open(&log_file_path) {
                            let logger = Box::new(FileLogger::new(file));
                            parser.set_logger(logger);
                        }
                    }
                }
                runner = runner.with_parser(Box::new(parser));
            }
            "semantic" => {
                let mut parser = SemanticAnnotationParser::new();
                if debug_enabled {
                    // Create a duplicate file handle for the parser logger
                    if let Ok(log_file_path) = get_current_log_file_path() {
                        if let Ok(file) = OpenOptions::new().append(true).open(&log_file_path) {
                            let logger = Box::new(FileLogger::new(file));
                            parser.set_logger(logger);
                        }
                    }
                }
                runner = runner.with_parser(Box::new(parser));
            }
            // For "all" or other values, use mock parser
            _ => {}
        }
    }

    // Apply filters
    if let Some(parser) = matches.get_one::<String>("parser") {
        if parser != "all" {
            runner = runner.with_parser_filter(parser.to_string());
        }
    }

    if let Some(tags_str) = matches.get_one::<String>("tags") {
        let tags: Vec<String> = tags_str.split(',')
            .map(|s| s.trim().to_string())
            .collect();
        runner = runner.with_tag_filter(tags);
    }

    // List mode
    if list_only {
        match runner.discover_test_suites() {
            Ok(suites) => {
                log_output("📋 Available Test Suites:");
                log_output(&"=".repeat(60));
                let suite_count = suites.len();
                for suite in suites {
                    log_output(&format!("• {} ({})", suite.suite_name, suite.parser_type));
                    log_output(&format!("  {}", suite.description));
                    log_output(&format!("  Tests: {}", suite.tests.len()));
                }
                log_output(&"=".repeat(60));
                log_output(&format!("Total: {} suites", suite_count));
            }
            Err(e) => {
                log_error(&format!("Error discovering test suites: {}", e));
                exit(1);
            }
        }
        return;
    }

    // Run tests
    log_output("🚀 Universal Test Runner");
    log_output(&"=".repeat(60));
    
    if let Some(parser) = matches.get_one::<String>("parser") {
        log_output(&format!("Parser filter: {}", parser));
    }
    if let Some(tags) = matches.get_one::<String>("tags") {
        log_output(&format!("Tag filter: {}", tags));
    }
    
    let show_dashboard = matches.get_flag("dashboard");
    
    match runner.run_all_tests() {
        Ok(report) => {
            if show_dashboard {
                // Get parser name from filter or use "All Parsers"
                let parser_name = matches.get_one::<String>("parser")
                    .map(|s| s.as_str())
                    .unwrap_or("All Parsers");
                report.print_dashboard(parser_name);
            } else {
                report.print_summary();
            }
            
            if report.failed > 0 {
                exit(1);
            } else {
                exit(0);
            }
        }
        Err(e) => {
            log_error(&format!("\n❌ Test runner error: {}", e));
            exit(2);
        }
    }
}