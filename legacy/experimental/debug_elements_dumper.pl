#!/usr/bin/env perl
use strict;
use warnings;
use Data::Dumper;

# Debug with Data::Dumper to see what's happening
require './json_v5_parser.pm';

print "Debugging with Data::Dumper to see structures...\n\n";

my $test_input = '["hello", "world"]';
print "Input: '$test_input'\n\n";

# Test the problematic elements parsing
print "Testing parse_elements from position 1:\n";
my $input_copy = $test_input;
pos($input_copy) = 1;  # After '['
print "Starting position: " . pos($input_copy) . "\n";
print "Remaining input: '" . substr($input_copy, pos($input_copy)) . "'\n\n";

# Let's modify the parse_elements temporarily to add debug output
# First, let's see what the current elements function looks like
print "=== Current parse_elements function structure ===\n";

# Get the regex that should match comma
print "Let's check what the comma regex is:\n";
no strict 'refs';
my $regexes_ref = ${'yapg::GeneratedParser::REGEXES'};
use strict 'refs';

print "Available regexes:\n";
foreach my $key (sort keys %$regexes_ref) {
    if ($key =~ /elements/) {
        print "  $key: " . $regexes_ref->{$key} . "\n";
    }
}
print "\n";

# Now test the actual function
my $elements_result = yapg::GeneratedParser::parse_elements(\$input_copy);
print "Elements result: " . Dumper($elements_result);
print "Final position: " . pos($input_copy) . "\n";
print "Character at final position: '" . substr($input_copy, pos($input_copy), 1) . "'\n";