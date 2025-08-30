#!/usr/bin/env perl
use strict;
use warnings;

# Manual trace of parse_value logic
require './json_v2_parser.pm';

print "Manual trace of parse_value('0') logic...\n\n";

my $input = '0';
my $start_pos = pos($input) // 0;
print "Initial pos: $start_pos\n";

print "\n1. Trying parse_object...\n";
my $result1 = yapg::GeneratedParser::parse_object(\$input);
print "   Result: " . (defined $result1 ? "SUCCESS" : "FAILED") . "\n";
print "   Pos after: " . (pos($input) // 0) . "\n";

print "\n2. Trying parse_array...\n";
my $result2 = yapg::GeneratedParser::parse_array(\$input);
print "   Result: " . (defined $result2 ? "SUCCESS" : "FAILED") . "\n";
print "   Pos after: " . (pos($input) // 0) . "\n";

print "\n3. Trying parse_string...\n";  
my $result3 = yapg::GeneratedParser::parse_string(\$input);
print "   Result: " . (defined $result3 ? "SUCCESS" : "FAILED") . "\n";
print "   Pos after: " . (pos($input) // 0) . "\n";

print "\n4. Trying parse_number...\n";
my $result4 = yapg::GeneratedParser::parse_number(\$input);
print "   Result: " . (defined $result4 ? "SUCCESS" : "FAILED") . "\n";
print "   Pos after: " . (pos($input) // 0) . "\n";

if (defined $result4) {
    print "\n✅ parse_number succeeded, should return result!\n";
    print "Final result would be: '$result4'\n";
} else {
    print "\n❌ parse_number failed, would restore pos to $start_pos\n";
}

# Compare with working case
print "\n" . "="x50 . "\n";
print "Comparing with '123'...\n";
my $input2 = '123';
pos($input2) = 0;
my $result_num = yapg::GeneratedParser::parse_number(\$input2);
print "parse_number('123'): " . (defined $result_num ? "SUCCESS" : "FAILED") . " (pos=" . pos($input2) . ")\n";

my $input3 = '123';
my $result_val = yapg::GeneratedParser::parse_value(\$input3);
print "parse_value('123'): " . (defined $result_val ? "SUCCESS" : "FAILED") . "\n";

