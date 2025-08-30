#!/usr/bin/env perl
use strict;
use warnings;
use Data::Dumper;

# Debug the sequence failure with detailed tracing
require './json_v5_parser.pm';

print "Debugging why the sequence fails when parts work...\n\n";

my $test_input = '"hello", "world"';
print "Input: '$test_input'\n\n";

# Let's manually execute the sequence step by step
print "=== Manual sequence execution ===\n";
my $input_copy = $test_input;
pos($input_copy) = 0;

print "Step 1: parse_value\n";
my $seq_pos = pos($input_copy);
print "  Starting position: $seq_pos\n";
my $value_result = yapg::GeneratedParser::parse_value(\$input_copy);
print "  parse_value result: " . Dumper($value_result);
print "  Position after parse_value: " . pos($input_copy) . "\n";
my $value_success = defined $value_result;
print "  Value success: $value_success\n\n";

if ($value_success) {
    print "Step 2: Comma regex\n";
    print "  Current position: " . pos($input_copy) . "\n";
    print "  Character at position: '" . substr($input_copy, pos($input_copy), 1) . "'\n";
    my $comma_success = $input_copy =~ /\G\Q,\E/gc;
    print "  Comma match result: $comma_success\n";
    print "  Position after comma: " . pos($input_copy) . "\n\n";
    
    if ($comma_success) {
        print "Step 3: Recursive parse_elements\n";
        print "  Starting position: " . pos($input_copy) . "\n";
        print "  Remaining input: '" . substr($input_copy, pos($input_copy)) . "'\n";
        my $recursive_result = yapg::GeneratedParser::parse_elements(\$input_copy);
        print "  Recursive result: " . Dumper($recursive_result);
        print "  Position after recursive: " . pos($input_copy) . "\n";
        my $recursive_success = defined $recursive_result;
        print "  Recursive success: $recursive_success\n\n";
        
        print "Overall sequence success: " . ($value_success && $comma_success && $recursive_success) . "\n";
        
        if ($value_success && $comma_success && $recursive_success) {
            print "✅ Manual sequence SHOULD work!\n";
        } else {
            print "❌ Manual sequence fails at step 3 (recursive)\n";
        }
    } else {
        print "❌ Manual sequence fails at step 2 (comma)\n";
    }
} else {
    print "❌ Manual sequence fails at step 1 (value)\n";
}

print "\n=== Now test actual parse_elements function ===\n";
my $input_copy2 = $test_input;
pos($input_copy2) = 0;
my $actual_result = yapg::GeneratedParser::parse_elements(\$input_copy2);
print "Actual parse_elements result: " . Dumper($actual_result);
print "Actual final position: " . pos($input_copy2) . "\n";

