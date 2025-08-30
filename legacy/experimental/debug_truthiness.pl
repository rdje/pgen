#!/usr/bin/env perl
use strict;
use warnings;
use Data::Dumper;

require './json_fixed_truthiness_parser.pm';

print "Debugging parse_json directly...\n\n";

my $simple_input = '{}';
print "Testing simple: '$simple_input'\n";

# Call parse_json directly
my $input_copy = $simple_input;
my $result = yapg::GeneratedParser::parse_json(\$input_copy);
print "parse_json result: " . Dumper($result);
print "Position: " . (pos($input_copy) // 0) . "/" . length($simple_input) . "\n\n";

# Test each component of parse_json sequence: ws value ws
print "Testing parse_json components manually:\n";
$input_copy = $simple_input;
pos($input_copy) = 0;

print "1. parse_ws at start:\n";
my $ws1 = yapg::GeneratedParser::parse_ws(\$input_copy);
print "   Result: " . Dumper($ws1);
print "   Position: " . pos($input_copy) . "\n";

print "2. parse_value:\n";
my $value = yapg::GeneratedParser::parse_value(\$input_copy);
print "   Result: " . Dumper($value);
print "   Position: " . pos($input_copy) . "\n";

print "3. parse_ws at end:\n";
my $ws2 = yapg::GeneratedParser::parse_ws(\$input_copy);
print "   Result: " . Dumper($ws2);
print "   Position: " . pos($input_copy) . "\n";

print "\nAll components work: " . (defined($ws1) && defined($value) && defined($ws2) ? "YES" : "NO") . "\n";
