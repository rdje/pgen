#!/usr/bin/env perl
use strict;
use warnings;
use Data::Dumper;

require './json_fixed_truthiness_parser.pm';

print "Testing simplest possible case...\n\n";

# Test just opening brace  
my $just_brace = '{';
print "Testing: '$just_brace'\n";

my $input_copy = $just_brace;
my $result = yapg::GeneratedParser::parse(\$input_copy);
print "Result: " . (defined $result ? "SUCCESS" : "FAILED") . "\n";
print "Position: " . (pos($input_copy) // 0) . "/" . length($just_brace) . "\n\n";

# Test parse_json directly
print "Testing parse_json on '$just_brace':\n";
$input_copy = $just_brace;
my $json_result = yapg::GeneratedParser::parse_json(\$input_copy);
print "Result: " . (defined $json_result ? "SUCCESS" : "FAILED") . "\n";
print "Position: " . (pos($input_copy) // 0) . "\n\n";

# Test the sequence components of parse_json: ws value ws
print "Testing parse_json components on '$just_brace':\n";
$input_copy = $just_brace;
pos($input_copy) = 0;

print "1. parse_ws:\n";
my $ws1 = yapg::GeneratedParser::parse_ws(\$input_copy);
print "   Result: " . Dumper($ws1) . "   Position: " . pos($input_copy) . "\n";

print "2. parse_value:\n";  
my $value = yapg::GeneratedParser::parse_value(\$input_copy);
print "   Result: " . Dumper($value) . "   Position: " . pos($input_copy) . "\n";

print "The issue: parse_value should succeed on '{' and consume the full object\n";
print "But '{' alone is not a complete object - it needs '}' to close it!\n";

