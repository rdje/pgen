#!/usr/bin/env perl
use strict;
use warnings;

# Debug what parse_elements is doing
require './json_v4_parser.pm';

my $test_input = '["hello", "world"]';
print "Input: '$test_input'\n";
print "Character positions:\n";
for my $i (0..length($test_input)-1) {
    print "  $i: '" . substr($test_input, $i, 1) . "'\n";
}

print "\nTesting parse_elements from position 1:\n";
my $input_copy = $test_input;
pos($input_copy) = 1;  # After '['
print "Starting position: " . pos($input_copy) . "\n";
print "Remaining input: '" . substr($input_copy, pos($input_copy)) . "'\n";

my $elements_result = yapg::GeneratedParser::parse_elements(\$input_copy);
print "Elements result: " . (defined $elements_result ? "SUCCESS" : "FAILED") . "\n";
print "Final position: " . pos($input_copy) . "\n";
print "Character at final position: '" . substr($input_copy, pos($input_copy), 1) . "'\n";
print "Remaining after elements: '" . substr($input_copy, pos($input_copy)) . "'\n";

# Test what elements should parse
print "\nExpected: elements should parse: \"hello\", \"world\"\n";
print "Actual position 8 is: '" . substr($test_input, 8, 1) . "'\n";
print "This suggests elements stopped after: '" . substr($test_input, 1, 7) . "'\n";

# Test individual string parsing
print "\nTesting individual string parsing:\n";
my $str1 = '"hello"';
my $str1_result = yapg::GeneratedParser::parse_string(\$str1);
print "parse_string('$str1'): " . (defined $str1_result ? "SUCCESS" : "FAILED") . "\n";

