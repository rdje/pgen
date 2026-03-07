#!/bin/bash

# Simplified Bootstrap Parser Test Suite
# Tests both semantic and return annotation bootstrap parsers

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PGEN_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
ARTIFACT_ROOT="$PGEN_ROOT/rust/target/bootstrap_tests"
cd "$PGEN_ROOT"

echo "🧪 Bootstrap Parser Comprehensive Test Suite"
echo "=============================================="
echo ""

# Counters for results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
EXPECTED_FAILURES=0
UNEXPECTED_RESULTS=0

# Function to run a single test
run_test() {
    local test_file="$1"
    local test_type="$2"  # "success" or "failure"
    local parser_type="$3"  # "return_annotation" or "semantic_annotation"
    
    local test_name=$(basename "$test_file" .ebnf)
    local full_test_name="${parser_type}/${test_type}/${test_name}"
    
    echo "  Testing: $full_test_name"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    # Keep bootstrap test scratch artifacts out of generated/
    local artifact_dir="$ARTIFACT_ROOT/${parser_type}/${test_type}"
    mkdir -p "$artifact_dir"
    local json_file="${artifact_dir}/${test_name}.json"
    local parser_file="${artifact_dir}/${test_name}_parser.rs"
    local log_file="${artifact_dir}/${test_name}.log"
    
    # Step 1: EBNF to JSON conversion
    if ! tools/ebnf_to_json.pl "$test_file" -o "$json_file" > "$log_file" 2>&1; then
        echo "    ❌ FAILED: EBNF to JSON conversion failed"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
    
    # Step 2: JSON to AST pipeline (this is where bootstrap parsers are tested)
    if ! rust/target/debug/ast_pipeline --generate-parser --debug --trace "$json_file" -o "$parser_file" >> "$log_file" 2>&1; then
        echo "    ❌ FAILED: AST pipeline failed"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
    
    # Step 3: Analyze the output for expected patterns
    local log_content=$(cat "$log_file")
    
    if [[ "$test_type" == "success" ]]; then
        # For success tests, we expect successful parsing without bootstrap warnings
        if [[ "$parser_type" == "return_annotation" ]]; then
            if echo "$log_content" | grep -q "WARNING.*Bootstrap mode supports FLAT structures only"; then
                echo "    ❌ UNEXPECTED: Success test triggered bootstrap fallback warning"
                UNEXPECTED_RESULTS=$((UNEXPECTED_RESULTS + 1))
                return 1
            elif echo "$log_content" | grep -q "Parsed return annotation"; then
                echo "    ✅ PASSED: Return annotation parsed successfully"
                PASSED_TESTS=$((PASSED_TESTS + 1))
                return 0
            else
                echo "    ❌ UNCLEAR: No clear success indicators found"
                UNEXPECTED_RESULTS=$((UNEXPECTED_RESULTS + 1))
                return 1
            fi
        else
            # Semantic annotation success tests
            if echo "$log_content" | grep -q "WARNING.*Semantic annotation pattern not recognized"; then
                echo "    ❌ UNEXPECTED: Success test failed to parse"
                UNEXPECTED_RESULTS=$((UNEXPECTED_RESULTS + 1))
                return 1
            else
                echo "    ✅ PASSED: Semantic annotation parsed successfully"  
                PASSED_TESTS=$((PASSED_TESTS + 1))
                return 0
            fi
        fi
    else
        # For failure tests, we expect bootstrap warnings or fallback to raw strings
        if [[ "$parser_type" == "return_annotation" ]]; then
            if echo "$log_content" | grep -q "WARNING.*Return annotation pattern not recognized in bootstrap mode"; then
                echo "    ✅ EXPECTED: Properly rejected with bootstrap warning"
                EXPECTED_FAILURES=$((EXPECTED_FAILURES + 1))
                return 0
            else
                echo "    ❌ UNEXPECTED: Should have been rejected by bootstrap parser"
                UNEXPECTED_RESULTS=$((UNEXPECTED_RESULTS + 1))
                return 1
            fi
        else
            # Semantic annotation failure tests
            if echo "$log_content" | grep -q "WARNING.*Semantic annotation pattern not recognized"; then
                echo "    ✅ EXPECTED: Properly fell back to raw string"
                EXPECTED_FAILURES=$((EXPECTED_FAILURES + 1))
                return 0
            else
                echo "    ❌ UNEXPECTED: Should have fallen back to raw string"
                UNEXPECTED_RESULTS=$((UNEXPECTED_RESULTS + 1))
                return 1
            fi
        fi
    fi
}

# Function to run all tests in a directory
run_test_category() {
    local category_dir="$1"
    local test_type="$2"
    local parser_type="$3"
    
    echo ""
    echo "📋 Testing $parser_type $test_type cases:"
    echo "----------------------------------------"
    
    local test_count=0
    for test_file in "$category_dir"/*.ebnf; do
        if [[ -f "$test_file" ]]; then
            run_test "$test_file" "$test_type" "$parser_type"
            test_count=$((test_count + 1))
        fi
    done
    
    if [[ $test_count -eq 0 ]]; then
        echo "  ⚠️  No test files found in $category_dir"
    fi
}

# Main test execution
echo "🚀 Starting comprehensive bootstrap parser testing..."
echo ""

# Test return annotation bootstrap parser
run_test_category "$SCRIPT_DIR/return_annotation/success" "success" "return_annotation"
run_test_category "$SCRIPT_DIR/return_annotation/failure" "failure" "return_annotation"

# Test semantic annotation bootstrap parser  
run_test_category "$SCRIPT_DIR/semantic_annotation/success" "success" "semantic_annotation"
run_test_category "$SCRIPT_DIR/semantic_annotation/failure" "failure" "semantic_annotation"

# Final summary
echo ""
echo "📊 Test Results Summary"
echo "======================="
echo "Total tests:           $TOTAL_TESTS"
echo "Passed:               $PASSED_TESTS"
echo "Expected failures:    $EXPECTED_FAILURES"
echo "Unexpected results:   $UNEXPECTED_RESULTS"
echo ""

if [[ $UNEXPECTED_RESULTS -eq 0 ]]; then
    echo "🎉 ALL TESTS BEHAVED AS EXPECTED!"
    echo "   Bootstrap parsers are working correctly."
    echo ""
    echo "✅ Ready to proceed with external parser creation:"
    echo "   - semantic_annotation_parser.rs ← grammars/semantic_annotation.ebnf" 
    echo "   - return_annotation_parser.rs ← grammars/return_annotation.ebnf"
else
    echo "⚠️  $UNEXPECTED_RESULTS tests had unexpected results."
    echo "   Review the bootstrap parser implementations before proceeding."
fi

echo ""
echo "📁 Test artifacts saved in: generated/"
echo "   - Log files: *.log"
echo "   - JSON files: *.json" 
echo "   - Generated parsers: *_parser.rs"

exit $UNEXPECTED_RESULTS
