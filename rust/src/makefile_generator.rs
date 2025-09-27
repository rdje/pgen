//! Makefile Target Generator
//! Automatically generates Makefile targets from test registry

use std::fs;
use std::path::Path;
use crate::test_registry::TestRegistry;

pub struct MakefileGenerator {
    makefile_path: String,
    target_prefix: String,
}

impl MakefileGenerator {
    pub fn new(makefile_path: &str, target_prefix: Option<&str>) -> Self {
        Self {
            makefile_path: makefile_path.to_string(),
            target_prefix: target_prefix.unwrap_or("test").to_string(),
        }
    }
    
    /// Generate all Makefile targets from the test registry
    pub fn generate_targets(&self, registry: &TestRegistry) -> Result<(), Box<dyn std::error::Error>> {
        let mut makefile_content = String::new();
        
        // Add header
        makefile_content.push_str(&self.generate_header());
        
        // Generate individual test targets
        let all_tests = registry.get_all_tests();
        let mut target_names = Vec::new();
        
        for test in &all_tests {
            let target_name = registry.generate_make_target(test);
            target_names.push(target_name.clone());
            makefile_content.push_str(&self.generate_individual_target(test, &target_name));
        }
        
        // Generate aggregate targets
        makefile_content.push_str(&self.generate_aggregate_targets(registry, &target_names));
        
        // Generate convenience targets
        makefile_content.push_str(&self.generate_convenience_targets(registry));
        
        // Write the generated content to file
        self.write_makefile_targets(&makefile_content)?;
        
        println!("Generated {} individual test targets", target_names.len());
        Ok(())
    }
    
    fn generate_header(&self) -> String {
        format!(r#"# Auto-generated Makefile targets for stress tests
# Generated at: {}
# DO NOT EDIT MANUALLY - This section is automatically regenerated

"#, chrono::Utc::now().to_rfc3339())
    }
    
    fn generate_individual_target(&self, test: &crate::test_registry::TestCase, target_name: &str) -> String {
        let escaped_input = self.escape_for_shell(&test.input);
        let description = &test.description;
        
        format!(r#"# {} - {}
{}: target/debug/pgen
	@echo "🧪 Testing {} parser with: {}"
	@echo "Running: cargo run -- --parser {} --input '{}'"
	@if cargo run -- --parser {} --input '{}' > /dev/null 2>&1; then \
		echo "✅ PASS: {} - {}"; \
	else \
		echo "❌ FAIL: {} - {}"; \
		echo "🔧 REPRODUCE with make: make {}"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser {} --input '{}'"; \
		exit 1; \
	fi
	@echo ""

"#, 
        description, 
        test.category,
        target_name,
        test.parser_type, 
        escaped_input,
        test.parser_type, 
        escaped_input,
        test.parser_type, 
        escaped_input,
        target_name, 
        description,
        target_name, 
        description,
        target_name,
        test.parser_type, 
        escaped_input
        )
    }
    
    fn generate_aggregate_targets(&self, registry: &TestRegistry, target_names: &[String]) -> String {
        let mut content = String::new();
        
        // All tests target
        content.push_str(&format!(
            "# Run all stress tests\n{}-all: {}\n\t@echo \"🎯 All stress tests completed successfully!\"\n\n",
            self.target_prefix,
            target_names.join(" ")
        ));
        
        // Parser-specific targets
        for parser_type in &["return", "semantic", "regex"] {
            let parser_targets: Vec<String> = target_names
                .iter()
                .filter(|name| name.contains(&format!("{}-", parser_type)))
                .cloned()
                .collect();
            
            if !parser_targets.is_empty() {
                content.push_str(&format!(
                    "# Run all {} parser tests\n{}-{}: {}\n\t@echo \"🎯 All {} parser tests completed!\"\n\n",
                    parser_type,
                    self.target_prefix,
                    parser_type,
                    parser_targets.join(" "),
                    parser_type
                ));
            }
        }
        
        // Category-specific targets
        let categories = self.get_unique_categories(registry);
        for category in categories {
            let category_targets: Vec<String> = target_names
                .iter()
                .filter(|name| name.contains(&format!("{}-", category)))
                .cloned()
                .collect();
            
            if !category_targets.is_empty() {
                content.push_str(&format!(
                    "# Run all {} category tests\n{}-{}: {}\n\t@echo \"🎯 All {} tests completed!\"\n\n",
                    category,
                    self.target_prefix,
                    category,
                    category_targets.join(" "),
                    category
                ));
            }
        }
        
        content
    }
    
    fn generate_convenience_targets(&self, registry: &TestRegistry) -> String {
        let mut content = String::new();
        
        content.push_str(&format!(r#"# Convenience targets
{}-quick: {}-return {}-semantic
	@echo "🚀 Quick test suite completed!"

{}-comprehensive: {}-all
	@echo "🔍 Comprehensive test suite completed!"

{}-debug: target/debug/pgen
	@echo "🐛 Running debug mode tests..."
	@cargo run -- --debug --parser return --input '$$1'
	@cargo run -- --debug --parser semantic --input '@type: "test"'

{}-stats:
	@echo "📊 Test Statistics:"
	@echo "  Return tests: {}"
	@echo "  Semantic tests: {}"
	@echo "  Regex tests: {}"
	@echo "  Total tests: {}"

.PHONY: {}-all {}-return {}-semantic {}-regex {}-quick {}-comprehensive {}-debug {}-stats

"#,
        self.target_prefix, self.target_prefix, self.target_prefix,
        self.target_prefix, self.target_prefix,
        self.target_prefix,
        self.target_prefix,
        registry.return_tests.len(),
        registry.semantic_tests.len(),
        registry.regex_tests.len(),
        registry.return_tests.len() + registry.semantic_tests.len() + registry.regex_tests.len(),
        self.target_prefix, self.target_prefix, self.target_prefix, self.target_prefix,
        self.target_prefix, self.target_prefix, self.target_prefix, self.target_prefix
        ));
        
        content
    }
    
    fn get_unique_categories(&self, registry: &TestRegistry) -> Vec<String> {
        let mut categories = std::collections::HashSet::new();
        
        for test in registry.get_all_tests() {
            categories.insert(test.category.clone());
        }
        
        let mut sorted_categories: Vec<String> = categories.into_iter().collect();
        sorted_categories.sort();
        sorted_categories
    }
    
    fn escape_for_shell(&self, input: &str) -> String {
        input
            .replace('\\', "\\\\")
            .replace('\'', "\\'")
            .replace('"', "\\\"")
            .replace('$', "\\$")
            .replace('`', "\\`")
    }
    
    fn write_makefile_targets(&self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Check if Makefile exists
        if Path::new(&self.makefile_path).exists() {
            // Read existing Makefile
            let existing_content = fs::read_to_string(&self.makefile_path)?;
            
            // Find the auto-generated section markers
            let start_marker = "# Auto-generated Makefile targets for stress tests";
            let end_marker = "# End of auto-generated targets";
            
            let updated_content = if let Some(start_pos) = existing_content.find(start_marker) {
                // Replace existing auto-generated section
                let before = &existing_content[..start_pos];
                let after = if let Some(end_pos) = existing_content.find(end_marker) {
                    &existing_content[end_pos + end_marker.len()..]
                } else {
                    // No end marker, append to end
                    "\n"
                };
                
                format!("{}{}\n# End of auto-generated targets\n{}", before, content, after)
            } else {
                // Append to existing Makefile
                format!("{}\n\n{}\n# End of auto-generated targets\n", existing_content, content)
            };
            
            fs::write(&self.makefile_path, updated_content)?;
        } else {
            // Create new Makefile
            let full_content = format!("{}\n# End of auto-generated targets\n", content);
            fs::write(&self.makefile_path, full_content)?;
        }
        
        println!("Updated Makefile targets in: {}", self.makefile_path);
        Ok(())
    }
    
    /// Generate a standalone Makefile with just the test targets
    pub fn generate_standalone_makefile(&self, registry: &TestRegistry, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut content = String::new();
        
        // Add standard Makefile header
        content.push_str(&format!(r#"# Stress Test Makefile
# Auto-generated from test registry
# Generated at: {}

# Default target
all: {}-all

# Build target
target/debug/pgen:
	cargo build

"#, chrono::Utc::now().to_rfc3339(), self.target_prefix));
        
        // Generate all targets
        let all_tests = registry.get_all_tests();
        let mut target_names = Vec::new();
        
        for test in &all_tests {
            let target_name = registry.generate_make_target(test);
            target_names.push(target_name.clone());
            content.push_str(&self.generate_individual_target(test, &target_name));
        }
        
        // Add aggregate and convenience targets
        content.push_str(&self.generate_aggregate_targets(registry, &target_names));
        content.push_str(&self.generate_convenience_targets(registry));
        
        fs::write(output_path, content)?;
        println!("Generated standalone Makefile: {}", output_path);
        
        Ok(())
    }
}