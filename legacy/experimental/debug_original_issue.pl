#!/usr/bin/env perl
use strict;
use warnings;
use Data::Dumper;

# Go back to the ORIGINAL working parser before we broke it
require './json_final_parser.pm';

print "Debugging the ORIGINAL issue with deep JSON...\n\n";

# Start with what WAS working
print "=== Testing what we know works ===\n";
my @working_cases = (
    '{}',
    '{"name": "John"}',
    '["hello", "world"]',
);

foreach my $test (@working_cases) {
    print "Testing: '$test'\n";
    my $input_copy = $test;
    my $result = yapg::GeneratedParser::parse(\$input_copy);
    my $success = defined $result && pos($input_copy) == length($test);
    print "  Result: " . ($success ? "✅ SUCCESS" : "❌ FAILED") . "\n\n";
}

# Now test the problematic deep JSON
print "=== Testing the ORIGINAL problem case ===\n";

# Let's start with a minimal version of the deep JSON
my $minimal_deep = '{
  "level1": {
    "simple": "value"
  }
}';

print "Testing minimal nested object:\n";
print $minimal_deep . "\n\n";

my $input_copy = $minimal_deep;
my $result = yapg::GeneratedParser::parse(\$input_copy);
my $success = defined $result && pos($input_copy) == length($minimal_deep);

print "Result: " . ($success ? "✅ SUCCESS" : "❌ FAILED") . "\n";
print "Position: " . (pos($input_copy) // 0) . "/" . length($minimal_deep) . "\n";

if (!$success) {
    my $pos = pos($input_copy) // 0;
    print "Failed at position $pos\n";
    print "Character at failure: '" . substr($minimal_deep, $pos, 1) . "'\n";
    print "Context: '" . substr($minimal_deep, max(0, $pos-5), 10) . "'\n";
}

sub max { $_[0] > $_[1] ? $_[0] : $_[1] }

