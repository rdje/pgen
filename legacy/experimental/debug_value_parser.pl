#!/usr/bin/env perl
use strict;
use warnings;

# Debug the value parser chain
require './json_v2_parser.pm';

print "Testing the value parser chain for '0'...\n\n";

my $test_input = '0';

# Test each level
print "1. Testing parse_json('0'): ";
my $input1 = $test_input;
my $result1 = yapg::GeneratedParser::parse_json(\$input1);
print defined $result1 ? "✅ SUCCESS\n" : "❌ FAILED\n";

print "2. Testing parse_value('0'): ";
my $input2 = $test_input;
my $result2 = yapg::GeneratedParser::parse_value(\$input2);
print defined $result2 ? "✅ SUCCESS\n" : "❌ FAILED\n";

print "3. Testing parse_number('0'): ";
my $input3 = $test_input;
my $result3 = yapg::GeneratedParser::parse_number(\$input3);
print defined $result3 ? "✅ SUCCESS\n" : "❌ FAILED\n";

# Also test with a working number
print "\nTesting with '123' for comparison...\n";
print "1. Testing parse_json('123'): ";
my $input4 = '123';
my $result4 = yapg::GeneratedParser::parse_json(\$input4);
print defined $result4 ? "✅ SUCCESS\n" : "❌ FAILED\n";

print "2. Testing parse_value('123'): ";
my $input5 = '123';
my $result5 = yapg::GeneratedParser::parse_value(\$input5);
print defined $result5 ? "✅ SUCCESS\n" : "❌ FAILED\n";

