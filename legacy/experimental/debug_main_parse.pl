#!/usr/bin/env perl
use strict;
use warnings;
use Data::Dumper;

require './json_fixed_truthiness_parser.pm';

print "Debugging main parse function...\n\n";

my $simple_input = '{}';
print "Testing: '$simple_input'\n\n";

print "1. Testing parse_json (entry rule):\n";
my $input_copy = $simple_input;
my $json_result = yapg::GeneratedParser::parse_json(\$input_copy);
print "   Result: " . Dumper($json_result);
print "   Position: " . pos($input_copy) . "/" . length($simple_input) . "\n\n";

print "2. Testing main parse function:\n";
$input_copy = $simple_input;
my $main_result = yapg::GeneratedParser::parse(\$input_copy);
print "   Result: " . Dumper($main_result);
print "   Position: " . pos($input_copy) . "/" . length($simple_input) . "\n\n";

print "3. Analysis:\n";
if (defined $json_result && pos($input_copy) == length($simple_input)) {
    print "   parse_json works and consumes all input\n";
} else {
    print "   Issue with parse_json or input consumption\n";
}

if (defined $main_result) {
    print "   Main parse succeeds\n";
} else {
    print "   Main parse fails\n";
}