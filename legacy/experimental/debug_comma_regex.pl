#!/usr/bin/env perl
use strict;
use warnings;
use Data::Dumper;

print "Testing comma regex matching directly...\n\n";

# Test the exact comma regex from the generated parser
my $comma_regex = qr/\Q,\E/o;
print "Comma regex: $comma_regex\n\n";

# Test different comma scenarios
my @comma_tests = (
    ',"world"',     # comma followed by quote (no space)
    ', "world"',    # comma followed by space then quote
    ',',            # just comma
    ' ,',           # space before comma
);

foreach my $test (@comma_tests) {
    print "Testing: '$test'\n";
    pos($test) = 0;
    if ($test =~ /\G$comma_regex/gc) {
        print "  ✅ MATCH at position " . (pos($test) - 1) . "\n";
        print "  Position after: " . pos($test) . "\n";
        print "  Remaining: '" . substr($test, pos($test)) . "'\n";
    } else {
        print "  ❌ NO MATCH\n";
    }
    print "\n";
}

# Now test the actual problematic sequence
print "=== Testing actual problematic sequence ===\n";
my $problem_input = '"hello", "world"';
print "Input: '$problem_input'\n";

# Step 1: parse_value should consume "hello"
pos($problem_input) = 0;
print "1. Should parse '\"hello\"' and leave position at " . length('"hello"') . "\n";
print "   Character at pos 7: '" . substr($problem_input, 7, 1) . "'\n";

# Step 2: comma should match
pos($problem_input) = 7;  # After "hello"
print "2. Testing comma match from position 7:\n";
if ($problem_input =~ /\G$comma_regex/gc) {
    print "   ✅ Comma matches, now at position " . pos($problem_input) . "\n";
    print "   Next character: '" . substr($problem_input, pos($problem_input), 1) . "'\n";
} else {
    print "   ❌ Comma doesn't match\n";
}

print "\nThe real question: Why does the full sequence fail when individual parts work?\n";

