#!/usr/bin/env perl
use strict;
use warnings;

# Debug the recursive elements parsing
require './json_v5_parser.pm';

print "Debugging recursive elements parsing...\n\n";

# Test the input: "hello", "world" (without brackets)
my $elements_input = '"hello", "world"';
print "Testing elements on: '$elements_input'\n";

my $input_copy = $elements_input;
pos($input_copy) = 0;
print "Starting position: " . pos($input_copy) . "\n";

# Manual step through what parse_elements should do:
# First alternative: parse_value && comma && parse_elements

print "\n1. Testing first parse_value:\n";
my $value1_result = yapg::GeneratedParser::parse_value(\$input_copy);
print "   Result: " . (defined $value1_result ? "SUCCESS" : "FAILED") . "\n";
print "   Position after: " . pos($input_copy) . "\n";
if (defined $value1_result) {
    print "   Should be at comma: '" . substr($elements_input, pos($input_copy), 1) . "'\n";
}

print "\n2. Testing comma matching:\n";
if ($input_copy =~ /\G,/gc) {
    print "   ✅ Comma matches\n";
    print "   Position after comma: " . pos($input_copy) . "\n";
} else {
    print "   ❌ Comma does not match\n";
}

print "\n3. Testing space after comma:\n";
print "   Character at current position: '" . substr($elements_input, pos($input_copy), 1) . "'\n";
if ($input_copy =~ /\G /gc) {
    print "   Space consumed, now at: " . pos($input_copy) . "\n";
}

print "\n4. Testing recursive parse_elements:\n";
my $elements_recursive = yapg::GeneratedParser::parse_elements(\$input_copy);
print "   Recursive result: " . (defined $elements_recursive ? "SUCCESS" : "FAILED") . "\n";
print "   Final position: " . pos($input_copy) . "\n";

# The issue might be whitespace!
print "\nWhitespace issue analysis:\n";
print "After comma at pos 8: '" . substr($elements_input, 8, 3) . "'\n";
print "Our grammar doesn't handle whitespace between tokens!\n";

