# LinkedSpec Test Framework

A simple test framework for validating LinkedSpec functionality.

## Structure

```
tests/
├── specs/
│   ├── valid/          # Spec files that should work
│   │   ├── basic.spec
│   │   └── order_independent.spec
│   └── invalid/        # Spec files that should fail
│       ├── invalid_regex.spec
│       ├── duplicate_rules.spec
│       ├── empty_file.spec
│       └── malformed_start.spec
├── input/              # Test input files
│   └── simple.txt
├── run_tests.pl        # Test runner
└── README.md           # This file
```

## Running Tests

```bash
cd tests
perl run_tests.pl
```

## Test Categories

### Valid Specs (should pass)
- **basic.spec**: Basic valid spec with all required components
- **order_independent.spec**: Demonstrates rule order independence

### Invalid Specs (should fail)
- **invalid_regex.spec**: Contains invalid regex pattern
- **duplicate_rules.spec**: Contains duplicate rule definitions
- **empty_file.spec**: Empty spec file
- **malformed_start.spec**: Doesn't start with rule definition

## Adding New Tests

### To add a valid test:
1. Create a new `.spec` file in `specs/valid/`
2. Ensure it follows DSL syntax rules
3. Run tests to verify it passes

### To add an invalid test:
1. Create a new `.spec` file in `specs/invalid/`
2. Introduce a specific error or issue
3. Run tests to verify it fails as expected

## Test Output

The test runner provides:
- ✅ **Pass indicators** for successful tests
- ❌ **Fail indicators** for failed tests
- **Detailed logs** in `test_results.log`
- **Summary statistics** at the end

## Test Results

Tests are considered:
- **PASS** if valid specs succeed and invalid specs fail
- **FAIL** if valid specs fail or invalid specs succeed

## Logging

All test details are logged to `test_results.log` including:
- Spec file content
- Parser output
- Exit codes
- Expected vs actual results 