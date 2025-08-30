#!/usr/bin/env perl
use strict;
use warnings;

require './json_whitespace_parser.pm';

print "Debugging newline case step by step...\n\n";

my $test_input = "{\n  \"key\": \"value\"\n}";
print "Input: " . repr($test_input) . "\n";
print "Length: " . length($test_input) . "\n\n";

# Test step by step
print "=== Testing components ===\n";

print "1. Testing parse_value:\n";
my $input_copy = $test_input;
my $value_result = yapg::GeneratedParser::parse_value(\$input_copy);
print "   Result: " . (defined $value_result ? "SUCCESS" : "FAILED") . "\n";
print "   Position: " . (pos($input_copy) // 0) . "\n\n";

print "2. Testing parse_object directly:\n";
my $input_copy2 = $test_input;
my $obj_result = yapg::GeneratedParser::parse_object(\$input_copy2);
print "   Result: " . (defined $obj_result ? "SUCCESS" : "FAILED") . "\n";
print "   Position: " . (pos($input_copy2) // 0) . "\n\n";

print "3. Testing main parse:\n";
my $input_copy3 = $test_input;
my $main_result = yapg::GeneratedParser::parse(\$input_copy3);
print "   Result: " . (defined $main_result ? "SUCCESS" : "FAILED") . "\n";
print "   Position: " . (pos($input_copy3) // 0) . "\n";
print "   Complete: " . ((pos($input_copy3) // 0) == length($test_input) ? "YES" : "NO") . "\n\n";

# Check what's wrong with the simple newline case
my $simple_newline = "[\n]";
print "4. Testing simple newline array '$simple_newline':\n";
my $input_copy4 = $simple_newline;
my $simple_result = yapg::GeneratedParser::parse(\$input_copy4);
print "   Result: " . (defined $simple_result ? "SUCCESS" : "FAILED") . "\n";
print "   Position: " . (pos($input_copy4) // 0) . "/" . length($simple_newline) . "\n";

sub repr {
    my $str = shift;
    $str =~ s/\n/\\n/g;
    $str =~ s/\t/\\t/g;
    return $str;
}

