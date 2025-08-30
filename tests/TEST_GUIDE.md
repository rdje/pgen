# LinkedSpec Test Framework Guide

## Table of Contents
- [Overview](#overview)
- [Directory Structure](#directory-structure)
- [Quick Start](#quick-start)
- [Running Tests](#running-tests)
- [Test Modes and Configuration](#test-modes-and-configuration)
- [Test Categories](#test-categories)
- [Debugging and Verbosity](#debugging-and-verbosity)
- [Understanding Results](#understanding-results)
- [Troubleshooting](#troubleshooting)
- [Examples](#examples)

## Overview

The LinkedSpec test framework provides comprehensive testing for the parser generator system. It includes both valid and invalid test cases to ensure robust error handling and correct functionality.

**Key Features:**
- Automated test suite with pass/fail reporting
- Multiple test modes (parse-only, generate-only, full pipeline)
- Configurable verbosity levels for debugging
- Clean log-based result analysis
- Individual test execution capabilities

## Directory Structure

```
afx/cursor/
├── fx/perl/LinkedSpec.pm          # Core parser generator
├── fx/perl/LinkedRE.pm            # Regex orchestration
├── run_parser.pl                   # Main parser runner
└── tests/                          # Test infrastructure
    ├── run_tests.pl               # Main test runner
    ├── run_single_test.sh         # Single test runner
    ├── TEST_GUIDE.md              # This documentation
    ├── generate_test_input.pl     # Test input generator
    ├── GENERATE_TEST_INPUT.md     # Generator documentation
    ├── input/                     # Test input files
    │   ├── simple.txt             # Lispish test data
    │   └── simple_ab.txt          # Simple pattern test data
    └── specs/                     # Test specification files
        ├── valid/                 # Valid .spec files
        │   ├── basic.spec         # Basic Lispish parser
        │   └── order_independent.spec  # Rule order independence
        └── invalid/               # Invalid .spec files
            ├── empty_file.spec    # Empty spec file
            ├── malformed_start.spec    # Invalid start
            ├── simple_duplicate.spec   # Duplicate rules
            ├── duplicate_rules.spec    # More duplicate rules
            └── invalid_regex.spec      # Invalid regex patterns
```

## Quick Start

```bash
# 1. Navigate to test directory
cd /Users/richarddje/Downloads/AFX/fsm/afx/cursor/tests

# 2. Run all tests
perl run_tests.pl

# 3. Check results
cat test_results.log
```

## Running Tests

### 0. Generate Test Input (Optional)

**Command:**
```bash
cd /Users/richarddje/Downloads/AFX/fsm/afx/cursor/tests
perl generate_test_input.pl specs/valid/basic.spec
```

**What it does:**
- Analyzes `.spec` files to understand grammar structure
- Generates pseudo-random test input based on grammar rules
- Creates hierarchical nested structures for comprehensive testing
- Supports multiple grammar types and generation strategies

**For detailed documentation:** See `GENERATE_TEST_INPUT.md`

### 1. Run All Tests (Recommended)

**Command:**
```bash
cd /Users/richarddje/Downloads/AFX/fsm/afx/cursor/tests
perl run_tests.pl
```

**What it does:**
- Executes all tests in both `valid/` and `invalid/` directories
- Shows real-time progress for each test
- Provides summary statistics
- Logs detailed results to `test_results.log`

**Example Output:**
```
=== LinkedSpec Test Suite ===
Test input: tests/input/simple.txt
Log file: test_results.log

=== Testing Valid Specs ===
Testing: Valid: basic.spec
  ✅ PASS
Testing: Valid: order_independent.spec
  ✅ PASS

=== Testing Invalid Specs ===
Testing: Invalid: empty_file.spec
  ✅ PASS
Testing: Invalid: malformed_start.spec
  ✅ PASS
Testing: Invalid: simple_duplicate.spec
  ❌ FAIL
Testing: Invalid: duplicate_rules.spec
  ❌ FAIL
Testing: Invalid: invalid_regex.spec
  ✅ PASS

=== Test Summary ===
Total tests: 7
Passed: 5
Failed: 2
Success rate: 71.4%
```

### 2. Run a Single Test

**Command:**
```bash
cd /Users/richarddje/Downloads/AFX/fsm/afx/cursor/tests
./run_single_test.sh specs/valid/basic.spec tests/input/simple.txt
```

**What it does:**
- Runs one specific test with detailed output
- Useful for debugging individual test issues
- Shows complete execution flow

**Example Output:**
```
=== LinkedSpec Parser Runner ===
Log file: run_parser_1754586351.log
Spec file: specs/valid/basic.spec
Input file: tests/input/simple.txt
Dump mode: OFF
Verbosity level: 0

=== Test Configuration ===
Test mode: full_pipeline
Expected outcome: pass

=== Specification File Content ===
# TEST_MODE: full_pipeline
# EXPECT: pass
Lispish::
 -> parenthesis     {return call(parenthesis)}
...

✅ TEST PASSED: Expected success and parser generation succeeded
Parser generated successfully!

=== Input File Content ===
(test_data
 (simple_value 123)
...

=== Parsing Input ===
Parse successful!
=== Parse Result ===
$VAR1 = [
          'test_data',
          [
            [
              'simple_value',
              [
                '123'
              ]
            ],
...
          ]
        ];
✅ TEST PASSED: Expected success and got success
```

### 3. Generate and Test Input

**Command:**
```bash
cd /Users/richarddje/Downloads/AFX/fsm/afx/cursor/tests
perl generate_test_input.pl -o test_input.txt specs/valid/basic.spec
perl run_parser.pl specs/valid/basic.spec test_input.txt
```

**What it does:**
- Generates test input from spec file
- Immediately tests the generated input with the parser
- Validates that generated input is parseable
- Useful for testing grammar-driven input generation

### 4. Run Parser Directly (Manual Testing)

**Command:**
```bash
cd /Users/richarddje/Downloads/AFX/fsm/afx/cursor
perl run_parser.pl path/to/spec.spec path/to/input.txt
```

**What it does:**
- Bypasses test framework for direct parser execution
- Useful for manual testing and debugging
- Shows raw parsing output

**Example:**
```bash
perl run_parser.pl tests/specs/valid/basic.spec tests/input/simple.txt
```

## Test Modes and Configuration

### Test Mode Declaration

Each `.spec` file must include test configuration comments at the top:

```perl
# TEST_MODE: full_pipeline    # Execution mode
# EXPECT: pass                # Expected outcome
```

### Available Test Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| `full_pipeline` | Parse spec → Generate parser → Parse input | Complete functionality test |
| `parse_only` | Parse spec → Generate parser → Stop | Test spec parsing only |
| `generate_only` | Parse spec → Stop | Test spec parsing without generation |

### Expected Outcomes

| Outcome | Description | When to Use |
|---------|-------------|-------------|
| `pass` | Test should succeed | Valid specs, working functionality |
| `fail` | Test should fail | Invalid specs, error conditions |

### Example Test Configurations

**Valid Test (basic.spec):**
```perl
# TEST_MODE: full_pipeline
# EXPECT: pass
Lispish::
 -> parenthesis     {return call(parenthesis)}
...
```

**Invalid Test (empty_file.spec):**
```perl
# TEST_MODE: parse_only
# EXPECT: fail
 
```

## Test Categories

### Valid Tests (Should Pass)

| Test File | Description | Purpose |
|-----------|-------------|---------|
| `basic.spec` | Basic Lispish parser | Tests core parsing functionality |
| `order_independent.spec` | Rule order independence | Tests that rule definition order doesn't matter |

### Invalid Tests (Should Fail)

| Test File | Description | Expected Failure |
|-----------|-------------|------------------|
| `empty_file.spec` | Empty spec file | Validation error - no rules defined |
| `malformed_start.spec` | Invalid start | Validation error - doesn't start with rule |
| `simple_duplicate.spec` | Duplicate rule definitions | Should detect duplicate rules |
| `duplicate_rules.spec` | Another duplicate test | Should detect duplicate rules |
| `invalid_regex.spec` | Invalid regex pattern | Validation error - malformed regex |

## Debugging and Verbosity

### Verbosity Levels

Set verbosity before running tests:

```bash
export DUMP_VERBOSITY=200  # Set verbosity level
perl run_tests.pl
```

| Level | Description | Use Case |
|-------|-------------|----------|
| `0` | No dumps (default) | Normal testing |
| `100` | LOW - Basic info | General debugging |
| `200` | MEDIUM - Detailed dumps | Shows `$retv` structure |
| `300` | HIGH - Very detailed | Deep debugging |

### Debugging Commands

```bash
# Enable medium verbosity for debugging
export DUMP_VERBOSITY=200
perl run_tests.pl

# Check detailed logs
cat test_results.log

# Check individual test logs
ls run_parser_*.log
cat run_parser_1754586351.log

# Run single test with verbosity
export DUMP_VERBOSITY=200
./run_single_test.sh specs/invalid/duplicate_rules.spec tests/input/simple.txt
```

### What Verbosity Shows

**DUMP_MEDIUM (200) includes:**
- `$retv` structure (raw parser output)
- Generated spec structure
- Validation details
- Error context

**DUMP_HIGH (300) includes:**
- All medium information plus
- Detailed execution flow
- Internal data structures
- Step-by-step processing

## Understanding Results

### PASS Indicators

Look for these messages in logs:
- `✅ TEST PASSED: Expected success and parser generation succeeded`
- `✅ TEST PASSED: Expected failure and parser generation failed`
- `Parser generation completed successfully`
- `Parse successful`
- `Validation failed as expected` (for invalid tests)

### FAIL Indicators

Look for these messages in logs:
- `❌ TEST FAILED: Expected failure but parser generation succeeded`
- `❌ TEST FAILED: Expected success but parser generation failed`
- `CRITICAL ERROR`
- `SPEC PARSING FAILED`
- Unexpected validation failures

### Exit Codes

**Important:** All tests exit with code 0 (as per framework requirements)
- Test success/failure is determined by log analysis, not exit codes
- This ensures clean integration with CI/CD systems

### Log File Locations

| File | Purpose | Location |
|------|---------|----------|
| `test_results.log` | Main test results | `tests/test_results.log` |
| `run_parser_*.log` | Individual test logs | `tests/run_parser_*.log` |

## Troubleshooting

### Common Issues and Solutions

#### 1. "command not found" errors

**Problem:** Scripts not found or not executable

**Solution:**
```bash
# Check current directory
pwd
# Should be: /Users/richarddje/Downloads/AFX/fsm/afx/cursor/tests

# Make script executable
chmod +x run_single_test.sh

# Verify script exists
ls -la run_single_test.sh
```

#### 2. Path issues

**Problem:** Tests can't find spec or input files

**Solution:**
```bash
# Always run from tests directory
cd /Users/richarddje/Downloads/AFX/fsm/afx/cursor/tests

# Use relative paths as shown in examples
./run_single_test.sh specs/valid/basic.spec tests/input/simple.txt
```

#### 3. Permission issues

**Problem:** Can't write log files

**Solution:**
```bash
# Check directory permissions
ls -la

# Fix permissions if needed
chmod 755 .
chmod 644 *.log 2>/dev/null || true
```

#### 4. Test failures

**Problem:** Tests failing unexpectedly

**Solution:**
```bash
# Enable verbosity for debugging
export DUMP_VERBOSITY=200
perl run_tests.pl

# Check detailed logs
cat test_results.log

# Run individual failing test
./run_single_test.sh specs/invalid/duplicate_rules.spec tests/input/simple.txt
```

### Debugging Checklist

- [ ] Are you in the correct directory? (`tests/`)
- [ ] Are all scripts executable? (`chmod +x run_single_test.sh`)
- [ ] Is verbosity enabled for debugging? (`export DUMP_VERBOSITY=200`)
- [ ] Are log files writable?
- [ ] Are spec files properly formatted with test mode declarations?

## Examples

### Example 1: Generate and Test Input

```bash
# Navigate to test directory
cd /Users/richarddje/Downloads/AFX/fsm/afx/cursor/tests

# Generate test input with hierarchical structures
perl generate_test_input.pl -n 5 -d 8 specs/valid/basic.spec

# Generate and save to file
perl generate_test_input.pl -o generated_input.txt specs/valid/basic.spec

# Test the generated input
perl run_parser.pl specs/valid/basic.spec generated_input.txt
```

### Example 2: Debug a Failing Test

```bash
# Navigate to test directory
cd /Users/richarddje/Downloads/AFX/fsm/afx/cursor/tests

# Enable debugging verbosity
export DUMP_VERBOSITY=200

# Run the failing test
./run_single_test.sh specs/invalid/duplicate_rules.spec tests/input/simple.txt

# Check the log
cat run_parser_*.log
```

### Example 3: Test a New Spec File

```bash
# Create a new test spec
cat > specs/valid/my_test.spec << 'EOF'
# TEST_MODE: full_pipeline
# EXPECT: pass
MyRule::
 -> my_pattern     {return call(my_pattern)}

my_pattern: /test/    I {
 return {type=>'TEST', content=>$IMATCH}
}
EOF

# Create test input
echo "test" > input/my_test.txt

# Run the test
./run_single_test.sh specs/valid/my_test.spec tests/input/my_test.txt
```

### Example 4: Run All Tests with Verbosity

```bash
# Set up environment
cd /Users/richarddje/Downloads/AFX/fsm/afx/cursor/tests
export DUMP_VERBOSITY=200

# Run all tests
perl run_tests.pl

# Check results
echo "=== Test Summary ==="
grep -E "(PASS|FAIL)" test_results.log | tail -10
```

### Example 5: Continuous Testing

```bash
# Run tests and check for failures
cd /Users/richarddje/Downloads/AFX/fsm/afx/cursor/tests
perl run_tests.pl

# Exit with error if any tests failed
if grep -q "Failed: [1-9]" test_results.log; then
    echo "❌ Some tests failed!"
    exit 1
else
    echo "✅ All tests passed!"
fi
```

## Integration with Development Workflow

### Before Committing Code

```bash
# Run full test suite
cd /Users/richarddje/Downloads/AFX/fsm/afx/cursor/tests
perl run_tests.pl

# Ensure all tests pass
if grep -q "Failed: [1-9]" test_results.log; then
    echo "❌ Tests failing - fix before commit!"
    exit 1
fi
```

### During Development

```bash
# Quick test of changes
./run_single_test.sh specs/valid/basic.spec tests/input/simple.txt

# Debug with verbosity
export DUMP_VERBOSITY=200
./run_single_test.sh specs/valid/basic.spec tests/input/simple.txt
```

---

**Note:** This test framework is designed to ensure the LinkedSpec parser generator works correctly and maintains backward compatibility. Always run tests before making significant changes to the core framework.
