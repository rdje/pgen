#!/usr/bin/env perl
use strict;
use warnings;

# Debug the string array parsing failure
require './json_v5_parser.pm';

print "Debugging string array parsing...\n\n";

my $test_input = '["hello", "world"]';
print "Input: '$test_input'\n";

# Test the main parse
my $input_copy = $test_input;
my $result = yapg::GeneratedParser::parse(\$input_copy);
print "Main parse result: " . (defined $result ? "SUCCESS" : "FAILED") . "\n";

# Test parse_array directly
print "\nTesting parse_array directly:\n";
my $input2 = $test_input;
my $array_result = yapg::GeneratedParser::parse_array(\$input2);
print "parse_array result: " . (defined $array_result ? "SUCCESS" : "FAILED") . "\n";
print "Position after: " . (pos($input2) // 0) . " (should be " . length($test_input) . ")\n";

# Test components step by step
print "\nTesting array components:\n";
print "1. Testing '[' matching:\n";
my $input3 = $test_input;
pos($input3) = 0;
if ($input3 =~ /\G\[/gc) {
    print "   ✅ '[' matches at pos " . (pos($input3) - 1) . "\n";
} else {
    print "   ❌ '[' does not match\n";
}

print "2. Testing elements parsing from position " . pos($input3) . ":\n";
my $elements_result = yapg::GeneratedParser::parse_elements(\$input3);
print "   parse_elements result: " . (defined $elements_result ? "SUCCESS" : "FAILED") . "\n";
print "   Position after elements: " . (pos($input3) // 0) . "\n";

print "3. Testing ']' matching:\n";
if ($input3 =~ /\G\]/gc) {
    print "   ✅ ']' matches\n";
} else {
    print "   ❌ ']' does not match\n";
}
