#!/usr/bin/env perl
use strict;
use warnings;

# Test parse_value with exact same call pattern as the actual function
require './json_v2_parser.pm';

print "Testing parse_value with exact same pattern...\n\n";

# Simulate exactly what parse_value does
my $input = '0';
my $start_pos = pos($input) // 0;
print "start_pos: $start_pos\n";

# Test the actual if-statement pattern
print "Testing the if-statement pattern:\n";

if (my $alt_result = yapg::GeneratedParser::parse_object(\$input)) { 
    print "parse_object returned: $alt_result\n"; 
} elsif (my $alt_result = yapg::GeneratedParser::parse_array(\$input)) { 
    print "parse_array returned: $alt_result\n"; 
} elsif (my $alt_result = yapg::GeneratedParser::parse_string(\$input)) { 
    print "parse_string returned: $alt_result\n"; 
} elsif (my $alt_result = yapg::GeneratedParser::parse_number(\$input)) { 
    print "✅ parse_number returned: $alt_result\n"; 
} else {
    print "❌ All alternatives failed\n";
}

print "Final pos: " . (pos($input) // 0) . "\n";

# Now test the EXACT parse_value function
print "\nTesting actual parse_value function:\n";
my $input2 = '0';
my $result = yapg::GeneratedParser::parse_value(\$input2);
print "parse_value result: " . (defined $result ? "SUCCESS ($result)" : "FAILED") . "\n";

# Let's also add some debugging to see intermediate positions
print "\nStep-by-step position tracking:\n";
my $input3 = '0';
pos($input3) = 0;
print "Initial pos: " . pos($input3) . "\n";

my $obj_result = yapg::GeneratedParser::parse_object(\$input3);
print "After parse_object: pos=" . (pos($input3) // 0) . ", result=" . (defined $obj_result ? "SUCCESS" : "FAILED") . "\n";

my $arr_result = yapg::GeneratedParser::parse_array(\$input3);
print "After parse_array: pos=" . (pos($input3) // 0) . ", result=" . (defined $arr_result ? "SUCCESS" : "FAILED") . "\n";

my $str_result = yapg::GeneratedParser::parse_string(\$input3);
print "After parse_string: pos=" . (pos($input3) // 0) . ", result=" . (defined $str_result ? "SUCCESS" : "FAILED") . "\n";

my $num_result = yapg::GeneratedParser::parse_number(\$input3);
print "After parse_number: pos=" . (pos($input3) // 0) . ", result=" . (defined $num_result ? "SUCCESS ($num_result)" : "FAILED") . "\n";

