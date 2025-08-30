#!/usr/bin/perl
use strict;
use warnings;
use Data::Dumper;

# Load the generated parser
require "./test_final_regex_fixed_parser.pm";

my $input = "x + y * 2";
print "=== DEBUGGING PARSE FAILURE ===\n";
print "Input: '$input'\n";
print "Input length: " . length($input) . "\n";

# Let me examine what the generated parser actually looks like
print "\n=== EXAMINING GENERATED PARSER STRUCTURE ===\n";

# Try to call individual parse functions to see what works
my $input_ref = \$input;

print "\n=== TESTING INDIVIDUAL COMPONENTS ===\n";

# Test if basic identifier parsing works
my $test_id = "x";
my $test_id_ref = \$test_id;
print "Testing parse_factor with 'x': ";
pos($$test_id_ref) = 0;
my $result_id = yapg::GeneratedParser::parse_factor($test_id_ref);
print Dumper($result_id);

# Test if basic number parsing works  
my $test_num = "2";
my $test_num_ref = \$test_num;
print "Testing parse_factor with '2': ";
pos($$test_num_ref) = 0;
my $result_num = yapg::GeneratedParser::parse_factor($test_num_ref);
print Dumper($result_num);

# Test full parsing with position tracking
print "\n=== TESTING FULL PARSE WITH POSITION TRACKING ===\n";
pos($$input_ref) = 0;
print "Starting position: " . (pos($$input_ref) // 0) . "\n";

my $result = yapg::GeneratedParser::parse($input_ref);
print "Final position: " . (pos($$input_ref) // 0) . "\n";
print "Result: " . Dumper($result);

if (!$result) {
    print "Parse failed. Position stopped at: " . (pos($$input_ref) // 0) . "\n";
    my $consumed = substr($input, 0, pos($$input_ref) // 0);
    my $remaining = substr($input, pos($$input_ref) // 0);
    print "Consumed: '$consumed'\n";
    print "Remaining: '$remaining'\n";
}

