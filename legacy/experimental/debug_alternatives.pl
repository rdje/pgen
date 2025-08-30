#!/usr/bin/env perl
use strict;
use warnings;

# Debug each alternative in parse_value for '0'
require './json_v2_parser.pm';

print "Testing each parse_value alternative for '0'...\n\n";

my $test_input = '0';

print "1. parse_object('0'): ";
my $input1 = $test_input;
my $result1 = yapg::GeneratedParser::parse_object(\$input1);
print defined $result1 ? "✅ SUCCESS (pos=" . pos($input1) . ")" : "❌ FAILED (pos=" . (pos($input1) // 0) . ")";
print "\n";

print "2. parse_array('0'): ";
my $input2 = $test_input;
my $result2 = yapg::GeneratedParser::parse_array(\$input2);
print defined $result2 ? "✅ SUCCESS (pos=" . pos($input2) . ")" : "❌ FAILED (pos=" . (pos($input2) // 0) . ")";
print "\n";

print "3. parse_string('0'): ";
my $input3 = $test_input;
my $result3 = yapg::GeneratedParser::parse_string(\$input3);
print defined $result3 ? "✅ SUCCESS (pos=" . pos($input3) . ")" : "❌ FAILED (pos=" . (pos($input3) // 0) . ")";
print "\n";

print "4. parse_number('0'): ";
my $input4 = $test_input;
my $result4 = yapg::GeneratedParser::parse_number(\$input4);
print defined $result4 ? "✅ SUCCESS (pos=" . pos($input4) . ")" : "❌ FAILED (pos=" . (pos($input4) // 0) . ")";
print "\n";

print "\nNow testing with fresh position for each...\n";
print "Testing parse_number with fresh pos(0):\n";
my $input5 = $test_input;
pos($input5) = 0;
print "Before: pos=" . pos($input5) . "\n";
my $result5 = yapg::GeneratedParser::parse_number(\$input5);
print "After: pos=" . (pos($input5) // 0) . ", result=" . (defined $result5 ? "SUCCESS" : "FAILED") . "\n";

