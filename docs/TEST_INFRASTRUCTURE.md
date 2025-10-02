# pgen Universal Test Infrastructure

## Overview

pgen uses a **SINGLE universal test runner** that works with ALL parsers through JSON test definitions:

- **One Runner to Rule Them All** - ONE test runner handles ALL parsers (current and future)
- **JSON-Driven** - All test definitions are in JSON files, no test code needed
- **Persistent** - Tests are stored in version control and never thrown away
- **Organized** - Tests are grouped by parser/feature in subdirectories
- **Discoverable** - Test runner automatically finds and runs all tests
- **Extensible** - Add new parsers or tests without changing any code
- **Filterable** - Run tests by parser type, tags, or specific suites

## Directory Structure

```
rust/
├── test_data/
│   ├── return_annotations/       # Return annotation parser tests
│   │   ├── basic_positional.json
│   │   ├── extraction_operators.json
│   │   ├── arrays_and_spreading.json
│   │   └── objects.json
│   │
│   ├── semantic_annotations/     # Semantic annotation parser tests
│   │   └── (future tests)
│   │
│   └── stress_tests/            # Large-scale stress tests
│       ├── regex_stress_test.json
│       └── semantic_stress_test.json
```

## Universal JSON Test Format

Each JSON test file defines a test suite that works with ANY parser:

```json
{
  "suite_name": "Suite Name",
  "description": "What this suite tests",
  "parser_type": "return",  // or "semantic", "regex", etc.
  "parser_config": {        // Optional parser configuration
    "debug_mode": false,
    "bootstrap_mode": false,
    "custom_options": {}
  },
  "tests": [
    {
      "name": "test_name",
      "description": "What this test validates",
      "input": "input string to parse",
      "expected": {
        // Option 1: Check for specific content
        "contains": ["pattern1", "pattern2"],
        "not_contains": ["error", "invalid"],
        
        // Option 2: Expect exact output (JSON)
        "output": {"type": "result", "value": 42},
        
        // Option 3: Expect an error
        "error": "Expected error message"
      },
      "skip": false,           // Optional: skip this test
      "timeout_ms": 5000,      // Optional: timeout in milliseconds
      "tags": ["regression", "critical"]  // Optional: for filtering
    }
  ]
}
```

### Test Suite Fields

- **suite_name** (required): Name of the test suite
- **description** (required): Description of what the suite tests
- **parser_type** (required): Which parser to use ("return", "semantic", "regex", etc.)
- **parser_config** (optional): Parser-specific configuration
  - **debug_mode**: Enable debug output
  - **bootstrap_mode**: Use bootstrap parser
  - **custom_options**: Future extensibility
- **tests** (required): Array of test cases

### Test Case Fields

- **name** (required): Unique test name
- **description** (required): What this test validates
- **input** (required): Input string to parse
- **expected** (required): Expected result (see below)
- **skip** (optional): Skip this test
- **timeout_ms** (optional): Test timeout in milliseconds
- **tags** (optional): Tags for filtering tests

### Expected Result Types

The `expected` field supports three formats:

1. **Success with validation**:
```json
{
  "output": {/* exact JSON output */},
  "contains": ["patterns to find"],
  "not_contains": ["patterns to avoid"]
}
```

2. **Error expectation**:
```json
{
  "error": "Expected error message or pattern"
}
```

3. **Any result** (just check it doesn't crash):
```json
{}
```

## Running Tests

### Using the Universal Test Runner CLI

```bash
# Build the test runner
cargo build --bin test_runner

# Run all tests
./target/debug/test_runner

# Run tests for a specific parser
./target/debug/test_runner --parser return
./target/debug/test_runner --parser semantic
./target/debug/test_runner --parser regex

# Run tests with specific tags
./target/debug/test_runner --tags regression,critical

# List available test suites without running
./target/debug/test_runner --list

# Verbose output
./target/debug/test_runner --verbose

# Combine filters
./target/debug/test_runner --parser return --tags extraction --verbose
```

### Using the pgen CLI (for single tests)

```bash
# Test a single input with a specific parser
./target/debug/pgen --parser return --input "$1"
./target/debug/pgen --parser semantic --input "$1.foo.bar"
./target/debug/pgen --parser return --input "[$1, $2::2*]" --debug
```

### In Rust Code

```rust
use pgen::universal_test_runner::{UniversalTestRunner, run_all_tests, run_parser_tests};

// Run all tests
let report = run_all_tests(false)?;
report.print_summary();

// Run tests for specific parser
let report = run_parser_tests("return", true)?;  // verbose = true

// Custom runner with filters
let mut runner = UniversalTestRunner::new()
    .with_verbose(true)
    .with_parser_filter("semantic".to_string())
    .with_tag_filter(vec!["regression".to_string()]);
    
let report = runner.run_all_tests()?;
```

### As a Cargo Test

```bash
# Run the universal test runner as a cargo test
cargo test test_universal_runner -- --nocapture
```

## Adding New Tests

### 1. Create a JSON file in the appropriate subdirectory

```json
// test_data/return_annotations/extraction_tests.json
{
  "suite_name": "Return Annotations - Extraction Tests",
  "description": "Tests for the new extraction operator",
  "parser_type": "return",
  "tests": [
    {
      "name": "basic_extraction",
      "description": "Test basic extraction operator",
      "input": "$2::2",
      "expected": {
        "contains": ["QuantifiedExtraction", "Index(2)"]
      }
    },
    {
      "name": "extraction_with_spread",
      "description": "Test extraction with spread",
      "input": "[$1, $2::2*]",
      "expected": {
        "contains": ["Array", "Spread", "QuantifiedExtraction"]
      },
      "tags": ["extraction", "spread"]
    }
  ]
}
```

### 2. Tests are automatically discovered

The universal test runner will automatically find and run your new tests the next time tests are executed.

## Adding Support for New Parsers

### 1. Implement the parser in pgen

Create your parser that can be called via the pgen CLI:

```rust
// In src/bin/pgen.rs
fn test_my_new_parser(input: &str, debug: bool, writer: &mut BufWriter<File>) 
    -> Result<(), Box<dyn std::error::Error>> 
{
    // Your parser implementation
}
```

### 2. Create test data directory

```bash
mkdir -p test_data/my_new_parser/
```

### 3. Add test suites

```json
// test_data/my_new_parser/basic_tests.json
{
  "suite_name": "My New Parser - Basic Tests",
  "description": "Basic tests for my new parser",
  "parser_type": "my_new_parser",  // Must match the name in pgen CLI
  "tests": [
    {
      "name": "test1",
      "description": "First test",
      "input": "test input",
      "expected": {
        "contains": ["expected", "output"]
      }
    }
  ]
}
```

### 4. Run tests

Your new parser tests will automatically work with the universal test runner:

```bash
./target/debug/test_runner --parser my_new_parser
```

## Best Practices

1. **Group related tests** - Use separate JSON files for different features
2. **Use descriptive names** - Test names should indicate what they're testing
3. **Include descriptions** - Every test should have a clear description
4. **Test edge cases** - Include tests for error conditions and edge cases
5. **Keep tests focused** - Each test should validate one specific behavior

## Benefits of Universal Test Infrastructure

- ✅ **ONE test runner for ALL parsers** - No need to create new runners for each parser
- ✅ **Zero code duplication** - Test logic is in ONE place, tests are pure JSON data
- ✅ **Parser agnostic** - Works with any parser (current or future)
- ✅ **Easy to add tests** - Just add JSON files, no code changes needed
- ✅ **Easy to add parsers** - New parsers automatically work with existing infrastructure
- ✅ **Version controlled** - All tests are tracked in git
- ✅ **Regression testing** - Old tests ensure new changes don't break existing features
- ✅ **Filterable execution** - Run by parser, tags, or specific suites
- ✅ **Consistent format** - Same JSON format works for all parsers
- ✅ **Language agnostic** - JSON tests could be run by implementations in other languages
- ✅ **Extensible** - Add new features to test format without changing existing tests
- ✅ **Self-documenting** - JSON tests clearly show what's being tested

## Future Enhancements

- [ ] Test categories (unit, integration, stress)
- [ ] Test tags for selective running
- [ ] Performance benchmarks in test data
- [ ] Expected parse time limits
- [ ] Test dependencies and ordering
- [ ] Parallel test execution
- [ ] HTML test report generation