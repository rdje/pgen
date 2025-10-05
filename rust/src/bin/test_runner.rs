#!/usr/bin/env rust
// Universal Test Runner CLI
// Run all tests, filter by parser, or filter by tags

use clap::{Command, Arg};
use pgen::test_runner::{RoundTripTestRunner, TestSuite, Report};
use std::process::exit;

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
        .get_matches();

    let verbose = matches.get_flag("verbose");
    let list_only = matches.get_flag("list");

    // Create runner with options
    let mut runner = UniversalTestRunner::new().with_verbose(verbose);

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
                println!("📋 Available Test Suites:");
                println!("{}", "=".repeat(60));
                let suite_count = suites.len();
                for suite in suites {
                    println!("• {} ({})", suite.suite_name, suite.parser_type);
                    println!("  {}", suite.description);
                    println!("  Tests: {}", suite.tests.len());
                }
                println!("{}", "=".repeat(60));
                println!("Total: {} suites", suite_count);
            }
            Err(e) => {
                eprintln!("Error discovering test suites: {}", e);
                exit(1);
            }
        }
        return;
    }

    // Run tests
    println!("🚀 Universal Test Runner");
    println!("{}", "=".repeat(60));
    
    if let Some(parser) = matches.get_one::<String>("parser") {
        println!("Parser filter: {}", parser);
    }
    if let Some(tags) = matches.get_one::<String>("tags") {
        println!("Tag filter: {}", tags);
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
            eprintln!("\n❌ Test runner error: {}", e);
            exit(2);
        }
    }
}