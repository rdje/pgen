# Development Guide - PGEN Test Automation System

**Contributing, Extending, and Maintaining the Test Automation System**

---

## 🎯 Overview

This guide provides comprehensive information for developers who want to:

- **Contribute** to the test automation system
- **Extend** functionality for new use cases  
- **Maintain** and debug the existing system
- **Understand** the codebase for modifications

---

## 🛠️ Development Environment Setup

### **Prerequisites**

- **Rust** 1.70+ with Cargo
- **Make** (GNU Make or compatible)
- **Git** for version control
- **macOS/Linux** (Windows support via WSL)

### **Optional Tools**

- **fswatch** (macOS): `brew install fswatch`
- **inotify-tools** (Linux): `apt-get install inotify-tools`
- **VS Code** with Rust extension

### **Project Structure**

```
pgen/rust/
├── src/
│   ├── test_registry.rs           # Core data structures
│   ├── test_discovery.rs          # File scanning logic
│   ├── makefile_generator.rs      # Makefile generation
│   ├── individual_tests_generator.rs  # Rust test generation
│   ├── test_automation.rs         # Workflow orchestration
│   └── bin/
│       ├── sync_tests.rs          # Main CLI tool
│       └── test_automation_demo.rs # System demo
├── Makefile                       # Main build file (auto-updated)
├── Makefile.auto-sync            # Auto-sync functionality
├── test_registry.json            # Test database (generated)
└── docs/                         # Documentation
```

### **Building and Testing**

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run the sync tool
cargo run --bin sync_tests sync

# Run the demo
cargo run --bin test_automation_demo
```

---

## 🏗️ Architecture Deep Dive

### **Core Components**

The system follows a **pipeline architecture** with clear separation of concerns:

```
Input Files → Discovery → Registry → Generation → Integration
```

Each component has a **single responsibility** and **well-defined interfaces**.

### **Component Interfaces**

```rust
// Core traits for extensibility
pub trait TestDiscoverer {
    fn discover_tests(&self, file_path: &str) -> Result<Vec<TestCase>, Error>;
}

pub trait TargetGenerator {
    fn generate_targets(&self, registry: &TestRegistry) -> Result<String, Error>;
}

pub trait TestValidator {
    fn validate_test(&self, test: &TestCase) -> Result<(), ValidationError>;
}
```

### **Data Flow**

1. **Discovery Phase**: Scan source files for test patterns
2. **Registry Phase**: Update central test database
3. **Generation Phase**: Create artifacts (Makefiles, Rust tests)
4. **Integration Phase**: Update existing project files

---

## 🔧 Adding New Parser Types

### **Step 1: Update Test Registry**

Add support for the new parser in `test_registry.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRegistry {
    pub return_tests: Vec<TestCase>,
    pub semantic_tests: Vec<TestCase>,
    pub regex_tests: Vec<TestCase>,
    pub my_new_parser_tests: Vec<TestCase>,  // Add this
    pub version: String,
    pub last_updated: String,
}

impl TestRegistry {
    pub fn new() -> Self {
        Self {
            // ... existing fields
            my_new_parser_tests: Self::default_my_new_parser_tests(),
        }
    }
    
    fn default_my_new_parser_tests() -> Vec<TestCase> {
        vec![
            TestCase {
                input: "example_input".to_string(),
                description: "Example test case".to_string(),
                parser_type: "my_new_parser".to_string(),
                category: "basic".to_string(),
                expected_result: TestExpectation::Success,
            },
        ]
    }
}
```

### **Step 2: Update Discovery Logic**

Add discovery support in `test_discovery.rs`:

```rust
impl TestDiscovery {
    pub fn discover_all_tests(&self) -> Result<TestRegistry, Error> {
        let mut registry = TestRegistry::default();
        
        // Clear existing tests
        registry.my_new_parser_tests.clear();
        
        // Add scanning for your parser
        self.scan_my_new_parser_stress_test(&mut registry)?;
        
        Ok(registry)
    }
    
    /// Scan for my_new_parser test cases
    fn scan_my_new_parser_stress_test(&self, registry: &mut TestRegistry) -> Result<(), Error> {
        let file_path = format!("{}/src/my_new_parser_stress_test.rs", self.root_path);
        
        if !Path::new(&file_path).exists() {
            println!("Warning: {} not found", file_path);
            return Ok(());
        }
        
        let content = fs::read_to_string(&file_path)?;
        let inputs = self.extract_test_inputs_from_array(&content, "test_inputs")?;
        
        for input in inputs {
            let test_case = TestCase {
                input: input.clone(),
                description: self.generate_description_for_input(&input, "my_new_parser"),
                parser_type: "my_new_parser".to_string(),
                category: self.categorize_input(&input, "my_new_parser"),
                expected_result: TestExpectation::Success,
            };
            registry.my_new_parser_tests.push(test_case);
        }
        
        Ok(())
    }
}
```

### **Step 3: Update Categorization**

Add categorization logic:

```rust
fn categorize_input(&self, input: &str, parser_type: &str) -> String {
    match parser_type {
        "my_new_parser" => {
            if input.starts_with("@") {
                "annotation".to_string()
            } else if input.contains("(") && input.contains(")") {
                "function_call".to_string()
            } else {
                "basic".to_string()
            }
        }
        // ... existing cases
        _ => "unknown".to_string(),
    }
}
```

### **Step 4: Update Generators**

Add generation support in `makefile_generator.rs`:

```rust
// Add parser-specific targets
for parser_type in &["return", "semantic", "regex", "my_new_parser"] {
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
```

Update `individual_tests_generator.rs`:

```rust
match test.parser_type.as_str() {
    "my_new_parser" => {
        format!("#[test]\nfn {}() {{\n    // {}\n    let _input = \"{}\";\n    println!(\"✅ Test {}: completed\", \"{}\");\n    // TODO: Add actual my_new_parser test\n}}\n\n",
            function_name, description, escaped_input, category, category)
    }
    // ... existing cases
}
```

---

## 🎨 Customizing Generation Templates

### **Makefile Template Customization**

Modify the target generation in `makefile_generator.rs`:

```rust
fn generate_individual_target(&self, test: &TestCase, target_name: &str) -> String {
    let escaped_input = self.escape_for_shell(&test.input);
    let description = &test.description;
    
    // Customize the template here
    format!(r#"# {description} - {category}
{target_name}: target/debug/pgen
	@echo "🧪 Testing {parser_type} parser with: {input}"
	@echo "Running: cargo run -- --parser {parser_type} --input '{escaped_input}'"
	@if cargo run -- --parser {parser_type} --input '{escaped_input}' > /dev/null 2>&1; then \
		echo "✅ PASS: {target_name} - {description}"; \
	else \
		echo "❌ FAIL: {target_name} - {description}"; \
		echo "🔧 REPRODUCE: cargo run -- --parser {parser_type} --input '{escaped_input}'"; \
		exit 1; \
	fi
	@echo ""

"#, 
    description = description,
    category = test.category,
    target_name = target_name,
    parser_type = test.parser_type,
    input = test.input,
    escaped_input = escaped_input,
    )
}
```

### **Rust Test Template Customization**

Modify test generation in `individual_tests_generator.rs`:

```rust
fn generate_single_test_function(&self, test: &TestCase) -> String {
    // Custom function naming
    let function_name = self.custom_function_naming(test);
    
    // Custom test body generation
    let test_body = self.generate_custom_test_body(test);
    
    format!("#[test]\nfn {}() {{\n{}\n}}\n\n", function_name, test_body)
}

fn generate_custom_test_body(&self, test: &TestCase) -> String {
    match test.parser_type.as_str() {
        "return" => {
            format!(r#"    // {}
    let input = "{}";
    let mut parser = ReturnParser::new();
    
    match parser.parse(input) {{
        Ok(result) => {{
            println!("✅ PASS: Test completed successfully");
            assert!(!result.is_empty(), "Result should not be empty");
        }}
        Err(e) => {{
            panic!("❌ FAIL: Parsing failed: {{:?}}", e);
        }}
    }}"#, test.description, self.escape_for_rust_string(&test.input))
        }
        // ... other parsers
        _ => format!("    // TODO: Implement test for {}", test.parser_type)
    }
}
```

---

## 🔍 Adding New Discovery Patterns

### **Custom Pattern Matching**

Extend discovery logic for new file formats:

```rust
/// Extract test inputs from custom format
fn extract_custom_format(&self, content: &str) -> Result<Vec<String>, Error> {
    let mut inputs = Vec::new();
    
    // Custom regex for your format
    let pattern = Regex::new(r#"TEST_CASE\s*\(\s*"([^"]+)"\s*\)"#)?;
    
    for capture in pattern.captures_iter(content) {
        if let Some(input) = capture.get(1) {
            inputs.push(input.as_str().to_string());
        }
    }
    
    Ok(inputs)
}

/// Extract from YAML format
fn extract_from_yaml(&self, content: &str) -> Result<Vec<String>, Error> {
    let yaml_pattern = Regex::new(r#"- input:\s*"([^"]+)""#)?;
    
    yaml_pattern
        .captures_iter(content)
        .filter_map(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .collect::<Vec<_>>()
        .into()
}
```

### **Multi-File Discovery**

Support discovering from multiple file types:

```rust
impl TestDiscovery {
    pub fn discover_from_directory(&self, dir_path: &str) -> Result<TestRegistry, Error> {
        let mut registry = TestRegistry::default();
        
        // Scan different file types
        let patterns = vec![
            ("**/*_stress_test.rs", self.scan_rust_files),
            ("**/*_test_cases.yaml", self.scan_yaml_files),
            ("**/*_tests.json", self.scan_json_files),
        ];
        
        for (pattern, scanner) in patterns {
            let files = glob::glob(&format!("{}/{}", dir_path, pattern))?;
            for file_path in files {
                let file_path = file_path?;
                scanner(&mut registry, file_path.to_str().unwrap())?;
            }
        }
        
        Ok(registry)
    }
}
```

---

## 🧪 Testing and Validation

### **Unit Testing Strategy**

Create comprehensive unit tests for each component:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_discovery_basic_patterns() {
        let temp_dir = create_test_environment();
        let discovery = TestDiscovery::new(temp_dir.path().to_str().unwrap());
        
        // Create test file
        let test_content = r#"
            let test_inputs = [
                "$1",
                "$2", 
                "\"hello\"",
            ];
        "#;
        
        fs::write(temp_dir.path().join("test_stress_test.rs"), test_content).unwrap();
        
        let registry = discovery.discover_all_tests().unwrap();
        assert_eq!(registry.return_tests.len(), 3);
        assert_eq!(registry.return_tests[0].input, "$1");
    }
    
    #[test]
    fn test_target_generation() {
        let generator = MakefileGenerator::new("/tmp/Makefile", Some("test"));
        let registry = create_test_registry();
        
        // Test target generation
        let result = generator.generate_targets(&registry);
        assert!(result.is_ok());
        
        // Verify generated content
        let content = fs::read_to_string("/tmp/Makefile").unwrap();
        assert!(content.contains("test-return-scalar-s1"));
    }
    
    fn create_test_registry() -> TestRegistry {
        let mut registry = TestRegistry::default();
        registry.return_tests.push(TestCase {
            input: "$1".to_string(),
            description: "Test case".to_string(),
            parser_type: "return".to_string(),
            category: "scalar".to_string(),
            expected_result: TestExpectation::Success,
        });
        registry
    }
    
    fn create_test_environment() -> TempDir {
        let temp_dir = tempfile::TempDir::new().unwrap();
        // Set up test environment
        temp_dir
    }
}
```

### **Integration Testing**

Test the complete workflow:

```rust
#[test]
fn test_end_to_end_workflow() {
    let temp_dir = create_test_environment();
    let automation = TestAutomation::new(temp_dir.path().to_str().unwrap());
    
    // Create source files
    setup_test_files(&temp_dir);
    
    // Run full sync
    let result = automation.full_sync();
    assert!(result.is_ok());
    
    // Verify all artifacts generated
    assert!(temp_dir.path().join("test_registry.json").exists());
    assert!(temp_dir.path().join("Makefile.stress").exists());
    assert!(temp_dir.path().join("src/individual_tests.rs").exists());
}
```

### **Property-Based Testing**

Use property-based testing for robustness:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_sanitization_properties(input in "\\PC*") {
        let registry = TestRegistry::default();
        let sanitized = registry.sanitize_for_target(&input);
        
        // Property: sanitized strings should be valid make targets
        prop_assert!(sanitized.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-'));
        
        // Property: should not be empty
        prop_assert!(!sanitized.is_empty());
        
        // Property: should be deterministic
        prop_assert_eq!(sanitized, registry.sanitize_for_target(&input));
    }
}
```

---

## 🐛 Debugging and Troubleshooting

### **Debug Logging**

Add comprehensive logging throughout the system:

```rust
use log::{debug, info, warn, error};

impl TestDiscovery {
    fn scan_stress_test_file(&self, file_path: &str) -> Result<Vec<TestCase>, Error> {
        info!("Scanning file: {}", file_path);
        
        let content = fs::read_to_string(file_path)?;
        debug!("File content length: {} bytes", content.len());
        
        let inputs = self.extract_test_inputs(&content)?;
        debug!("Extracted {} test inputs", inputs.len());
        
        for (i, input) in inputs.iter().enumerate() {
            debug!("  [{}]: {}", i, input);
        }
        
        Ok(self.convert_to_test_cases(inputs)?)
    }
}
```

### **Error Context**

Provide rich error context:

```rust
use anyhow::{Context, Result};

fn process_test_file(&self, path: &str) -> Result<Vec<TestCase>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read test file: {}", path))?;
    
    let inputs = self.extract_inputs(&content)
        .with_context(|| format!("Failed to extract test inputs from: {}", path))?;
    
    if inputs.is_empty() {
        warn!("No test inputs found in file: {}", path);
    }
    
    Ok(inputs.into_iter()
        .map(|input| self.create_test_case(input))
        .collect::<Result<Vec<_>>>()?)
}
```

### **Diagnostic Tools**

Create diagnostic utilities:

```rust
pub struct SystemDiagnostics {
    root_path: String,
}

impl SystemDiagnostics {
    pub fn run_full_diagnostic(&self) -> DiagnosticReport {
        let mut report = DiagnosticReport::new();
        
        // Check file system state
        report.add_section("File System", self.check_file_system());
        
        // Check registry integrity
        report.add_section("Registry", self.check_registry());
        
        // Check generated files
        report.add_section("Generated Files", self.check_generated_files());
        
        // Performance metrics
        report.add_section("Performance", self.check_performance());
        
        report
    }
    
    fn check_registry(&self) -> Vec<DiagnosticItem> {
        let mut items = Vec::new();
        
        if let Ok(registry) = TestRegistry::load_from_file("test_registry.json") {
            items.push(DiagnosticItem::info(
                "Registry loaded successfully",
                format!("Contains {} total test cases", registry.get_all_tests().len())
            ));
            
            // Check for inconsistencies
            for test in registry.get_all_tests() {
                if test.input.is_empty() {
                    items.push(DiagnosticItem::warning(
                        "Empty test input found",
                        format!("Test: {}", test.description)
                    ));
                }
            }
        } else {
            items.push(DiagnosticItem::error(
                "Failed to load registry",
                "Run sync to regenerate"
            ));
        }
        
        items
    }
}
```

---

## 📈 Performance Optimization

### **Profiling Integration**

Add performance monitoring:

```rust
use std::time::Instant;

pub struct PerformanceMonitor {
    timings: HashMap<String, Duration>,
}

impl PerformanceMonitor {
    pub fn time_operation<F, R>(&mut self, name: &str, operation: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = operation();
        let duration = start.elapsed();
        
        self.timings.insert(name.to_string(), duration);
        
        debug!("Operation '{}' took {:?}", name, duration);
        result
    }
    
    pub fn print_summary(&self) {
        println!("Performance Summary:");
        for (name, duration) in &self.timings {
            println!("  {}: {:?}", name, duration);
        }
    }
}
```

### **Optimization Strategies**

Implement performance optimizations:

```rust
// Parallel processing
use rayon::prelude::*;

impl TestDiscovery {
    fn discover_all_tests_parallel(&self) -> Result<TestRegistry, Error> {
        let files = vec![
            "src/comprehensive_stress_test.rs",
            "src/semantic_annotation_stress_test.rs", 
            "src/regex_stress_test.rs",
        ];
        
        let results: Result<Vec<_>, _> = files
            .par_iter()
            .map(|file| self.scan_single_file(file))
            .collect();
        
        let all_tests = results?;
        Ok(self.merge_test_results(all_tests))
    }
}

// Caching
use std::collections::HashMap;

pub struct CachedRegistry {
    cache: HashMap<String, TestRegistry>,
    cache_timeout: Duration,
}

impl CachedRegistry {
    pub fn get_or_load(&mut self, path: &str) -> Result<&TestRegistry, Error> {
        if let Some(registry) = self.cache.get(path) {
            if self.is_cache_valid(path)? {
                return Ok(registry);
            }
        }
        
        let registry = TestRegistry::load_from_file(path)?;
        self.cache.insert(path.to_string(), registry);
        Ok(self.cache.get(path).unwrap())
    }
}
```

---

## 🔌 Extension Points and Plugins

### **Plugin Architecture**

Design a plugin system:

```rust
pub trait SyncPlugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    
    fn on_discovery_start(&self, context: &DiscoveryContext) -> Result<(), PluginError>;
    fn on_test_discovered(&self, test: &TestCase) -> Result<TestCase, PluginError>;
    fn on_generation_complete(&self, artifacts: &GeneratedArtifacts) -> Result<(), PluginError>;
}

pub struct PluginManager {
    plugins: Vec<Box<dyn SyncPlugin>>,
}

impl PluginManager {
    pub fn register_plugin(&mut self, plugin: Box<dyn SyncPlugin>) {
        println!("Registering plugin: {} v{}", plugin.name(), plugin.version());
        self.plugins.push(plugin);
    }
    
    pub fn trigger_discovery_start(&self, context: &DiscoveryContext) -> Result<(), PluginError> {
        for plugin in &self.plugins {
            plugin.on_discovery_start(context)?;
        }
        Ok(())
    }
}

// Example plugin implementation
pub struct ValidationPlugin;

impl SyncPlugin for ValidationPlugin {
    fn name(&self) -> &str { "ValidationPlugin" }
    fn version(&self) -> &str { "1.0.0" }
    
    fn on_test_discovered(&self, test: &TestCase) -> Result<TestCase, PluginError> {
        // Validate test case
        if test.input.is_empty() {
            return Err(PluginError::ValidationFailed("Empty test input".to_string()));
        }
        
        // Transform test case if needed
        let mut modified_test = test.clone();
        modified_test.description = format!("[VALIDATED] {}", test.description);
        
        Ok(modified_test)
    }
}
```

### **Configuration System**

Implement flexible configuration:

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SyncConfig {
    pub discovery: DiscoveryConfig,
    pub generation: GenerationConfig,
    pub integration: IntegrationConfig,
    pub plugins: Vec<PluginConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DiscoveryConfig {
    pub file_patterns: Vec<String>,
    pub ignore_patterns: Vec<String>,
    pub parallel_processing: bool,
    pub custom_extractors: HashMap<String, String>,
}

impl SyncConfig {
    pub fn load_from_file(path: &str) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)
            .map_err(|e| ConfigError::FileError(e))?;
        
        let config: SyncConfig = toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e))?;
        
        config.validate()?;
        Ok(config)
    }
    
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.discovery.file_patterns.is_empty() {
            return Err(ConfigError::ValidationError("No file patterns specified".to_string()));
        }
        
        // More validation...
        Ok(())
    }
}
```

---

## 📚 Documentation and Examples

### **Code Documentation Standards**

Follow consistent documentation patterns:

```rust
/// Discovers test cases from source files.
///
/// This component scans specified files for test input patterns and converts
/// them into structured test cases for the registry.
/// 
/// # Examples
/// 
/// ```rust
/// use pgen::test_discovery::TestDiscovery;
/// 
/// let discovery = TestDiscovery::new("/path/to/project");
/// let registry = discovery.discover_all_tests()?;
/// println!("Found {} test cases", registry.get_all_tests().len());
/// ```
/// 
/// # Supported File Formats
/// 
/// - Rust files with `let test_inputs = [...]` arrays
/// - YAML files with `test_cases` sections  
/// - JSON files with test case arrays
/// 
/// # Error Handling
/// 
/// Returns `Err` if:
/// - Source files cannot be read
/// - File content cannot be parsed
/// - Test case validation fails
pub struct TestDiscovery {
    root_path: String,
}

impl TestDiscovery {
    /// Creates a new test discovery instance.
    /// 
    /// # Arguments
    /// 
    /// * `root_path` - The root directory to search for test files
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let discovery = TestDiscovery::new("/path/to/project");
    /// ```
    pub fn new(root_path: &str) -> Self {
        Self {
            root_path: root_path.to_string(),
        }
    }
}
```

### **Integration Examples**

Provide comprehensive examples:

```rust
// examples/custom_parser_integration.rs

use pgen::test_automation::TestAutomation;
use pgen::test_registry::{TestCase, TestExpectation};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize automation system
    let automation = TestAutomation::new(".")?;
    
    // Add a custom test case
    let custom_test = TestCase {
        input: "custom_input".to_string(),
        description: "Custom parser test".to_string(),
        parser_type: "custom".to_string(),
        category: "basic".to_string(),
        expected_result: TestExpectation::Success,
    };
    
    automation.add_test(custom_test)?;
    
    // Run synchronization
    automation.full_sync()?;
    
    println!("Custom parser integration complete!");
    Ok(())
}
```

---

## 🎯 Contributing Guidelines

### **Code Style**

- **Formatting**: Use `cargo fmt` for consistent formatting
- **Linting**: Address all `cargo clippy` warnings
- **Naming**: Follow Rust naming conventions
- **Documentation**: Document all public APIs

### **Pull Request Process**

1. **Fork** the repository
2. **Create** a feature branch: `git checkout -b feature/new-parser-support`
3. **Implement** your changes with tests
4. **Update** documentation
5. **Run** the full test suite: `cargo test`
6. **Submit** a pull request with detailed description

### **Testing Requirements**

- **Unit tests** for all new functionality
- **Integration tests** for workflow changes
- **Documentation examples** must be tested
- **Performance tests** for optimization changes

### **Review Checklist**

- [ ] All tests pass
- [ ] Code follows style guidelines
- [ ] Documentation is updated
- [ ] No breaking changes to public API
- [ ] Performance impact is acceptable
- [ ] Error handling is comprehensive

---

This development guide provides the foundation for contributing to and extending the PGEN Test Automation System. Follow these patterns and guidelines to maintain code quality and system reliability.