//! Test Synchronization CLI Tool
//! Command-line interface for managing test automation

use std::env;
use std::process;
use pgen::test_automation::{TestAutomation, TestStats};
use pgen::test_registry::{TestCase, TestExpectation};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }
    
    // Determine root path (current directory if not specified)
    let root_path = env::current_dir()
        .expect("Failed to get current directory")
        .to_str()
        .expect("Path is not valid UTF-8")
        .to_string();
    
    let automation = TestAutomation::new(&root_path);
    
    match args[1].as_str() {
        "sync" | "full-sync" => {
            if let Err(e) = automation.full_sync() {
                eprintln!("❌ Full synchronization failed: {}", e);
                process::exit(1);
            }
        }
        "quick-sync" => {
            if let Err(e) = automation.quick_sync() {
                eprintln!("❌ Quick synchronization failed: {}", e);
                process::exit(1);
            }
        }
        "add" => {
            if args.len() < 5 {
                eprintln!("❌ Usage: {} add <parser_type> <input> <description> [category]", args[0]);
                process::exit(1);
            }
            
            let parser_type = &args[2];
            let input = &args[3];
            let description = &args[4];
            let category = args.get(5).map(|s| s.as_str()).unwrap_or("manual");
            
            let test_case = TestCase {
                input: input.clone(),
                description: description.clone(),
                parser_type: parser_type.clone(),
                category: category.to_string(),
                expected_result: TestExpectation::Success,
            };
            
            if let Err(e) = automation.add_test(test_case) {
                eprintln!("❌ Failed to add test case: {}", e);
                process::exit(1);
            }
        }
        "remove" => {
            if args.len() < 4 {
                eprintln!("❌ Usage: {} remove <parser_type> <input>", args[0]);
                process::exit(1);
            }
            
            let parser_type = &args[2];
            let input = &args[3];
            
            match automation.remove_test(parser_type, input) {
                Ok(true) => println!("✅ Test case removed successfully"),
                Ok(false) => {
                    eprintln!("⚠️  Test case not found");
                    process::exit(1);
                }
                Err(e) => {
                    eprintln!("❌ Failed to remove test case: {}", e);
                    process::exit(1);
                }
            }
        }
        "check" => {
            match automation.check_sync_needed() {
                Ok(true) => {
                    println!("🔄 Synchronization needed");
                    process::exit(2); // Special exit code for "sync needed"
                }
                Ok(false) => {
                    println!("✅ Tests are synchronized");
                }
                Err(e) => {
                    eprintln!("❌ Failed to check sync status: {}", e);
                    process::exit(1);
                }
            }
        }
        "stats" => {
            match automation.get_stats() {
                Ok(stats) => print_stats(&stats),
                Err(e) => {
                    eprintln!("❌ Failed to get statistics: {}", e);
                    process::exit(1);
                }
            }
        }
        "help" | "--help" | "-h" => {
            print_usage();
        }
        _ => {
            eprintln!("❌ Unknown command: {}", args[1]);
            print_usage();
            process::exit(1);
        }
    }
}

fn print_usage() {
    println!("Test Synchronization CLI Tool");
    println!("Automatically synchronizes test cases across Makefiles and Rust tests\n");
    
    println!("USAGE:");
    println!("    sync_tests <COMMAND> [OPTIONS]\n");
    
    println!("COMMANDS:");
    println!("    sync, full-sync        Discover all tests and regenerate all targets");
    println!("    quick-sync             Regenerate targets from existing registry");
    println!("    add <type> <input> <desc> [cat]  Add a new test case");
    println!("    remove <type> <input>  Remove a test case");
    println!("    check                  Check if synchronization is needed");
    println!("    stats                  Show test statistics");
    println!("    help                   Show this help message\n");
    
    println!("EXAMPLES:");
    println!("    sync_tests sync                    # Full synchronization");
    println!("    sync_tests quick-sync              # Quick regeneration");
    println!("    sync_tests add return '$3' 'Third scalar' scalar");
    println!("    sync_tests remove return '$3'");
    println!("    sync_tests check");
    println!("    sync_tests stats\n");
    
    println!("EXIT CODES:");
    println!("    0    Success");
    println!("    1    Error");
    println!("    2    Synchronization needed (check command only)");
}

fn print_stats(stats: &TestStats) {
    println!("📊 Test Registry Statistics");
    println!("═══════════════════════════");
    
    if !stats.registry_exists {
        println!("⚠️  No test registry found - run 'sync_tests sync' first");
        return;
    }
    
    println!("Return parser tests:   {}", stats.return_tests);
    println!("Semantic parser tests: {}", stats.semantic_tests);
    println!("Regex parser tests:    {}", stats.regex_tests);
    println!("─────────────────────");
    println!("Total test cases:      {}", stats.total_tests);
    println!("Last updated:          {}", stats.last_updated);
    
    println!("\n🎯 Available Make targets:");
    println!("  make test-all          # Run all tests");
    println!("  make test-return       # Run return parser tests");
    println!("  make test-semantic     # Run semantic parser tests");
    println!("  make test-quick        # Run quick test suite");
    println!("  make test-stats        # Show test statistics");
    
    println!("\n🦀 Rust test integration:");
    println!("  cargo test             # Run all individual tests");
    println!("  cargo test test_all_return_parser");
    println!("  cargo test test_all_semantic_parser");
}