#!/usr/bin/env perl
use strict;
use warnings;
use Data::Dumper;

require './json_v5_parser.pm';

print "Testing parse_value with leading whitespace...\n\n";

my @tests = (
    '"world"',     # No leading space
    ' "world"',    # Leading space  
    '  "world"',   # Multiple leading spaces
    "\t\"world\"", # Leading tab
    "\n\"world\"", # Leading newline
);

foreach my $test (@tests) {
    print "Testing parse_value on: " . Dumper($test);
    my $input_copy = $test;
    pos($input_copy) = 0;
    my $result = yapg::GeneratedParser::parse_value(\$input_copy);
    print "  Result: " . (defined $result ? "SUCCESS ($result)" : "FAILED") . "\n";
    print "  Position: " . pos($input_copy) . "\n\n";
}

print "Conclusion: parse_value fails with leading whitespace!\n";
print "This is why the recursive elements call fails.\n";

