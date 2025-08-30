#!/usr/bin/env perl

use strict;
use warnings;

# Test configuration
my $TEST_INPUT = "tests/input/simple.txt";
my $LOG_FILE = "test_results.log";

# Test counters
my $total_tests = 0;
my $passed_tests = 0;
my $failed_tests = 0;

print "=== LinkedSpec Test Suite ===\n";
print "Test input: $TEST_INPUT\n";
print "Log file: $LOG_FILE\n\n";

# Clear previous log
open(my $log_fh, '>', $LOG_FILE) or die "Cannot create log file: $!";
close($log_fh);

sub log_message {
    my ($message) = @_;
    open(my $log_fh, '>>', $LOG_FILE) or return;
    print $log_fh "$message\n";
    close($log_fh);
}

sub run_test {
    my ($spec_file, $should_pass, $test_name) = @_;
    $total_tests++;
    
    print "Testing: $test_name\n";
    log_message("=== Testing: $test_name ===");
    log_message("Spec file: $spec_file");
    log_message("Expected: " . ($should_pass ? "PASS" : "FAIL"));
    
    # Run the parser using shell script
    my $output = `bash run_single_test.sh "$spec_file" "$TEST_INPUT" 2>&1`;
    my $exit_code = $? >> 8;
    
    log_message("Exit code: $exit_code");
    log_message("Output:\n$output");
    
    my $result;
    if ($should_pass) {
        if ($exit_code == 0) {
            $result = "✅ PASS";
            $passed_tests++;
        } else {
            $result = "❌ FAIL (should have passed)";
            $failed_tests++;
        }
    } else {
        if ($exit_code != 0) {
            $result = "✅ PASS (correctly failed)";
            $passed_tests++;
        } else {
            $result = "❌ FAIL (should have failed)";
            $failed_tests++;
        }
    }
    
    print "  $result\n";
    log_message("Result: $result\n");
    
    return $result;
}

# Test valid specs
print "=== Testing Valid Specs ===\n";
run_test("specs/valid/basic.spec", 1, "Valid: basic.spec");
run_test("specs/valid/order_independent.spec", 1, "Valid: order_independent.spec");

print "\n=== Testing Invalid Specs ===\n";
run_test("specs/invalid/empty_file.spec", 0, "Invalid: empty_file.spec");
run_test("specs/invalid/malformed_start.spec", 0, "Invalid: malformed_start.spec");
run_test("specs/invalid/duplicate_rules.spec", 0, "Invalid: duplicate_rules.spec");
run_test("specs/invalid/invalid_regex.spec", 0, "Invalid: invalid_regex.spec");

# Summary
print "\n=== Test Summary ===\n";
print "Total tests: $total_tests\n";
print "Passed: $passed_tests\n";
print "Failed: $failed_tests\n";
print "Success rate: " . sprintf("%.1f", ($passed_tests / $total_tests) * 100) . "%\n";

log_message("=== Test Summary ===");
log_message("Total tests: $total_tests");
log_message("Passed: $passed_tests");
log_message("Failed: $failed_tests");
log_message("Success rate: " . sprintf("%.1f", ($passed_tests / $total_tests) * 100) . "%");

if ($failed_tests > 0) {
    print "\n❌ Some tests failed. Check $LOG_FILE for details.\n";
    exit 1;
} else {
    print "\n✅ All tests passed!\n";
    exit 0;
} 