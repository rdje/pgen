//! Test Automation Demo
//! Demonstrates the complete test synchronization workflow

use std::env;
use std::process;
use pgen::test_automation::TestAutomation;
use pgen::test_registry::{TestCase, TestExpectation};

fn main() {
    println!("🚀 Test Automation System Demo");
    println!("═══════════════════════════════");
    
    // Get current directory
    let root_path = env::current_dir()
        .expect("Failed to get current directory")
        .to_str()
        .expect("Path is not valid UTF-8")
        .to_string();
    
    let automation = TestAutomation::new(&root_path);
    
    println!("📁 Working directory: {}", root_path);
    
    // Step 1: Show initial statistics
    println!("\n1️⃣ Initial Test Statistics");
    println!("─────────────────────────");
    match automation.get_stats() {
        Ok(stats) => {
            if stats.registry_exists {
                println!("   Return tests: {}", stats.return_tests);
                println!("   Semantic tests: {}", stats.semantic_tests);
                println!("   Regex tests: {}", stats.regex_tests);
                println!("   Total: {}", stats.total_tests);
            } else {
                println!("   ⚠️  No existing test registry found");
            }
        }
        Err(e) => {
            println!("   ❌ Failed to get stats: {}", e);
        }
    }
    
    // Step 2: Check if sync is needed
    println!("\n2️⃣ Checking Synchronization Status");
    println!("──────────────────────────────────");
    match automation.check_sync_needed() {
        Ok(true) => println!("   🔄 Synchronization is needed"),
        Ok(false) => println!("   ✅ Tests are already synchronized"),
        Err(e) => println!("   ❌ Failed to check sync status: {}", e),
    }
    
    // Step 3: Perform full synchronization
    println!("\n3️⃣ Performing Full Synchronization");
    println!("──────────────────────────────────");
    
    match automation.full_sync() {
        Ok(_) => {
            println!("   ✅ Full synchronization completed successfully!");
        }
        Err(e) => {
            println!("   ❌ Full synchronization failed: {}", e);
            process::exit(1);
        }
    }
    
    // Step 4: Show updated statistics
    println!("\n4️⃣ Updated Test Statistics");
    println!("─────────────────────────");
    match automation.get_stats() {
        Ok(stats) => {
            println!("   Return tests: {}", stats.return_tests);
            println!("   Semantic tests: {}", stats.semantic_tests);
            println!("   Regex tests: {}", stats.regex_tests);
            println!("   Total: {}", stats.total_tests);
            println!("   Last updated: {}", stats.last_updated);
        }
        Err(e) => {
            println!("   ❌ Failed to get updated stats: {}", e);
        }
    }
    
    // Step 5: Demonstrate adding a custom test
    println!("\n5️⃣ Adding a Custom Test Case");
    println!("────────────────────────────");
    
    let custom_test = TestCase {
        input: "$custom".to_string(),
        description: "Demo custom scalar reference".to_string(),
        parser_type: "return".to_string(),
        category: "demo".to_string(),
        expected_result: TestExpectation::Success,
    };
    
    match automation.add_test(custom_test) {
        Ok(_) => {
            println!("   ✅ Custom test case added successfully!");
        }
        Err(e) => {
            println!("   ❌ Failed to add custom test: {}", e);
        }
    }
    
    // Step 6: Show final statistics
    println!("\n6️⃣ Final Test Statistics");
    println!("────────────────────────");
    match automation.get_stats() {
        Ok(stats) => {
            println!("   Return tests: {}", stats.return_tests);
            println!("   Semantic tests: {}", stats.semantic_tests);
            println!("   Regex tests: {}", stats.regex_tests);
            println!("   Total: {}", stats.total_tests);
        }
        Err(e) => {
            println!("   ❌ Failed to get final stats: {}", e);
        }
    }
    
    // Step 7: Demonstrate removing the custom test
    println!("\n7️⃣ Removing Custom Test Case");
    println!("────────────────────────────");
    
    match automation.remove_test("return", "$custom") {
        Ok(true) => {
            println!("   ✅ Custom test case removed successfully!");
        }
        Ok(false) => {
            println!("   ⚠️  Custom test case not found");
        }
        Err(e) => {
            println!("   ❌ Failed to remove custom test: {}", e);
        }
    }
    
    // Step 8: Summary and next steps
    println!("\n8️⃣ Demo Summary & Next Steps");
    println!("────────────────────────────");
    println!("   ✅ Test automation system is now active!");
    println!("   📄 Generated files:");
    println!("      • test_registry.json - Central test registry");
    println!("      • Makefile targets - Auto-generated test targets");
    println!("      • Makefile.stress - Standalone stress test Makefile");
    println!("      • src/individual_tests.rs - Individual Rust tests");
    
    println!("\n💡 Usage Examples:");
    println!("   make test-all                    # Run all stress tests");
    println!("   make test-return                 # Run return parser tests");
    println!("   make test-semantic               # Run semantic parser tests");
    println!("   cargo run --bin sync_tests sync  # Re-synchronize tests");
    println!("   cargo run --bin sync_tests stats # Show test statistics");
    println!("   cargo test                       # Run Rust unit tests");
    
    println!("\n🎯 The test environment will now automatically adjust");
    println!("   whenever you add or remove tests from the stress test files!");
}