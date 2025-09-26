//! Test Automation Orchestrator
//! Coordinates all test synchronization components

use std::path::Path;
use crate::test_registry::TestRegistry;
use crate::test_discovery::TestDiscovery;
use crate::makefile_generator::MakefileGenerator;
use crate::individual_tests_generator::IndividualTestsGenerator;

pub struct TestAutomation {
    root_path: String,
    registry_path: String,
}

impl TestAutomation {
    pub fn new(root_path: &str) -> Self {
        Self {
            root_path: root_path.to_string(),
            registry_path: format!("{}/test_registry.json", root_path),
        }
    }
    
    /// Full synchronization - discover, update registry, and regenerate all targets
    pub fn full_sync(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔄 Starting full test synchronization...");
        
        // Step 1: Discover all tests from source files
        let discovery = TestDiscovery::new(&self.root_path);
        let mut registry = discovery.discover_all_tests()?;
        
        // Step 2: Load existing registry if it exists and merge any manual entries
        if Path::new(&self.registry_path).exists() {
            match TestRegistry::load_from_file(&self.registry_path) {
                Ok(existing_registry) => {
                    println!("📖 Loaded existing registry, merging discoveries...");
                    self.merge_registries(&mut registry, &existing_registry);
                }
                Err(e) => {
                    println!("⚠️  Failed to load existing registry: {}, using discovered tests only", e);
                }
            }
        }
        
        // Step 3: Save updated registry
        registry.save_to_file(&self.registry_path)?;
        println!("💾 Saved updated test registry to: {}", self.registry_path);
        
        // Step 4: Generate Makefile targets
        let makefile_path = format!("{}/Makefile", self.root_path);
        let makefile_generator = MakefileGenerator::new(&makefile_path, Some("test"));
        makefile_generator.generate_targets(&registry)?;
        
        // Also generate standalone stress test Makefile
        let stress_makefile_path = format!("{}/Makefile.stress", self.root_path);
        makefile_generator.generate_standalone_makefile(&registry, &stress_makefile_path)?;
        
        // Step 5: Generate individual tests file
        let individual_tests_path = format!("{}/src/individual_tests.rs", self.root_path);
        let tests_generator = IndividualTestsGenerator::new(&individual_tests_path);
        tests_generator.generate_tests(&registry)?;
        
        // Step 6: Update existing TestTargetMapper if it exists
        self.update_target_mapper(&registry)?;
        
        println!("✅ Full test synchronization completed successfully!");
        self.print_summary(&registry);
        
        Ok(())
    }
    
    /// Quick sync - only regenerate targets from existing registry
    pub fn quick_sync(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("⚡ Starting quick test synchronization...");
        
        // Load existing registry
        let registry = TestRegistry::load_from_file(&self.registry_path)?;
        
        // Regenerate all targets
        let makefile_path = format!("{}/Makefile", self.root_path);
        let makefile_generator = MakefileGenerator::new(&makefile_path, Some("test"));
        makefile_generator.generate_targets(&registry)?;
        
        let individual_tests_path = format!("{}/src/individual_tests.rs", self.root_path);
        let tests_generator = IndividualTestsGenerator::new(&individual_tests_path);
        tests_generator.generate_tests(&registry)?;
        
        self.update_target_mapper(&registry)?;
        
        println!("✅ Quick synchronization completed!");
        Ok(())
    }
    
    /// Add a new test case and sync
    pub fn add_test(&self, test: crate::test_registry::TestCase) -> Result<(), Box<dyn std::error::Error>> {
        println!("➕ Adding new test case: {}", test.description);
        
        let mut registry = if Path::new(&self.registry_path).exists() {
            TestRegistry::load_from_file(&self.registry_path)?
        } else {
            TestRegistry::default()
        };
        
        registry.add_test(test);
        registry.save_to_file(&self.registry_path)?;
        
        // Regenerate all targets
        self.quick_sync()?;
        
        println!("✅ Test case added and synchronized!");
        Ok(())
    }
    
    /// Remove a test case and sync
    pub fn remove_test(&self, parser_type: &str, input: &str) -> Result<bool, Box<dyn std::error::Error>> {
        println!("➖ Removing test case: {} parser with input '{}'", parser_type, input);
        
        let mut registry = TestRegistry::load_from_file(&self.registry_path)?;
        let removed = registry.remove_test(parser_type, input);
        
        if removed {
            registry.save_to_file(&self.registry_path)?;
            self.quick_sync()?;
            println!("✅ Test case removed and synchronized!");
        } else {
            println!("⚠️  Test case not found");
        }
        
        Ok(removed)
    }
    
    /// Check if synchronization is needed
    pub fn check_sync_needed(&self) -> Result<bool, Box<dyn std::error::Error>> {
        if !Path::new(&self.registry_path).exists() {
            return Ok(true);
        }
        
        let registry = TestRegistry::load_from_file(&self.registry_path)?;
        let discovery = TestDiscovery::new(&self.root_path);
        let discovered_registry = discovery.discover_all_tests()?;
        
        // Compare test counts as a simple check
        let current_total = registry.return_tests.len() + registry.semantic_tests.len() + registry.regex_tests.len();
        let discovered_total = discovered_registry.return_tests.len() + discovered_registry.semantic_tests.len() + discovered_registry.regex_tests.len();
        
        Ok(current_total != discovered_total)
    }
    
    /// Get current test statistics
    pub fn get_stats(&self) -> Result<TestStats, Box<dyn std::error::Error>> {
        let registry = if Path::new(&self.registry_path).exists() {
            TestRegistry::load_from_file(&self.registry_path)?
        } else {
            TestRegistry::default()
        };
        
        Ok(TestStats {
            return_tests: registry.return_tests.len(),
            semantic_tests: registry.semantic_tests.len(),
            regex_tests: registry.regex_tests.len(),
            total_tests: registry.return_tests.len() + registry.semantic_tests.len() + registry.regex_tests.len(),
            last_updated: registry.last_updated,
            registry_exists: Path::new(&self.registry_path).exists(),
        })
    }
    
    fn merge_registries(&self, target: &mut TestRegistry, source: &TestRegistry) {
        // For now, just preserve version info from source
        // In the future, could implement more sophisticated merging
        // of manually added test cases
        target.version = source.version.clone();
    }
    
    fn update_target_mapper(&self, registry: &TestRegistry) -> Result<(), Box<dyn std::error::Error>> {
        let target_mapper_path = format!("{}/src/test_target_mapper.rs", self.root_path);
        
        if !Path::new(&target_mapper_path).exists() {
            println!("ℹ️  TestTargetMapper not found, skipping update");
            return Ok(());
        }
        
        // Read current mapper file
        let current_content = std::fs::read_to_string(&target_mapper_path)?;
        
        // Generate new mapping code
        let mut new_mappings = String::new();
        new_mappings.push_str("        // Auto-generated mappings from test registry\n");
        
        for test in registry.get_all_tests() {
            let target_name = registry.generate_make_target(test);
            let escaped_input = test.input.replace('\\', "\\\\").replace('"', "\\\"");
            
            new_mappings.push_str(&format!(
                "        mappings.insert(\"{}\".to_string(), \"{}\".to_string());\n",
                escaped_input, target_name
            ));
        }
        
        // Replace the auto-generated section
        let start_marker = "        // Auto-generated mappings from test registry";
        let end_marker = "        // End auto-generated mappings";
        
        let updated_content = if let Some(start_pos) = current_content.find(start_marker) {
            let before = &current_content[..start_pos];
            let after = if let Some(end_pos) = current_content.find(end_marker) {
                &current_content[end_pos..]
            } else {
                "\n        // End auto-generated mappings\n"
            };
            
            format!("{}{}{}", before, new_mappings, after)
        } else {
            current_content // No auto-generated section found, leave as-is
        };
        
        std::fs::write(&target_mapper_path, updated_content)?;
        println!("🔄 Updated TestTargetMapper mappings");
        
        Ok(())
    }
    
    fn print_summary(&self, registry: &TestRegistry) {
        println!("\n📊 Test Synchronization Summary:");
        println!("  Return parser tests: {}", registry.return_tests.len());
        println!("  Semantic parser tests: {}", registry.semantic_tests.len());
        println!("  Regex parser tests: {}", registry.regex_tests.len());
        println!("  Total test cases: {}", registry.return_tests.len() + registry.semantic_tests.len() + registry.regex_tests.len());
        println!("  Registry: {}", self.registry_path);
        println!("  Last updated: {}", registry.last_updated);
        
        println!("\n🎯 Generated Files:");
        println!("  Makefile targets updated");
        println!("  Makefile.stress created");  
        println!("  src/individual_tests.rs updated");
        println!("  src/test_target_mapper.rs updated");
        
        println!("\n💡 Usage:");
        println!("  make test-all           # Run all tests");
        println!("  make test-return        # Run return parser tests");
        println!("  make test-semantic      # Run semantic parser tests");
        println!("  make test-quick         # Run quick test suite");
        println!("  make test-stats         # Show test statistics");
        println!("  cargo test              # Run Rust unit tests");
    }
}

#[derive(Debug)]
pub struct TestStats {
    pub return_tests: usize,
    pub semantic_tests: usize,
    pub regex_tests: usize,
    pub total_tests: usize,
    pub last_updated: String,
    pub registry_exists: bool,
}