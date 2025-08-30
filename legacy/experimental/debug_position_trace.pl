#!/usr/bin/env perl
use strict;
use warnings;

# Detailed position trace
require './json_v3_parser.pm';

print "Detailed position trace for '0'...\n\n";

my $input = '0';
pos($input) = 0;
print "1. Initial state: input='$input', pos=" . pos($input) . "\n";

print "\n2. Calling parse_json(\\\$input)...\n";
my $result = yapg::GeneratedParser::parse_json(\$input);
print "   Result: " . (defined $result ? "'$result'" : "undef") . "\n";
print "   Pos after parse_json: " . pos($input) . "\n";

print "\n3. Testing parse_number directly...\n";
my $input2 = '0';
pos($input2) = 0;
print "   Before parse_number: pos=" . pos($input2) . "\n";
my $num_result = yapg::GeneratedParser::parse_number(\$input2);
print "   After parse_number: pos=" . pos($input2) . ", result=" . (defined $num_result ? "'$num_result'" : "undef") . "\n";

print "\n4. Testing parse_value directly...\n";
my $input3 = '0';
pos($input3) = 0;
print "   Before parse_value: pos=" . pos($input3) . "\n";
my $val_result = yapg::GeneratedParser::parse_value(\$input3);
print "   After parse_value: pos=" . pos($input3) . ", result=" . (defined $val_result ? "'$val_result'" : "undef") . "\n";

print "\n5. Manual step-by-step in parse_value...\n";
my $input4 = '0';
pos($input4) = 0;
my $start_pos = pos($input4);
print "   start_pos: $start_pos\n";

# Test each alternative and track position
print "   Trying parse_object: ";
my $obj_res = yapg::GeneratedParser::parse_object(\$input4);
print (defined $obj_res ? "SUCCESS" : "FAILED") . " (pos=" . pos($input4) . ")\n";

print "   Trying parse_array: ";
my $arr_res = yapg::GeneratedParser::parse_array(\$input4);
print (defined $arr_res ? "SUCCESS" : "FAILED") . " (pos=" . pos($input4) . ")\n";

print "   Trying parse_string: ";
my $str_res = yapg::GeneratedParser::parse_string(\$input4);
print (defined $str_res ? "SUCCESS" : "FAILED") . " (pos=" . pos($input4) . ")\n";

print "   Trying parse_number: ";
my $num_res = yapg::GeneratedParser::parse_number(\$input4);
print (defined $num_res ? "SUCCESS ('$num_res')" : "FAILED") . " (pos=" . pos($input4) . ")\n";

if (defined $num_res) {
    print "   ✅ parse_number succeeded, parse_value should return without restoring position\n";
    print "   Final position should be: " . pos($input4) . "\n";
}

