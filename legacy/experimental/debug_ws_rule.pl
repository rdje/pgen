#!/usr/bin/env perl
use strict;
use warnings;

require './json_ws_fixed_parser.pm';

print "Debugging whitespace rule...\n\n";

# Test the ws rule directly if possible
print "1. Testing simple value without whitespace:\n";
my $simple = '"hello"';
my $input_copy = $simple;
my $result = yapg::GeneratedParser::parse(\$input_copy);
print "   '$simple': " . (defined $result ? "SUCCESS" : "FAILED") . "\n";
print "   Position: " . (pos($input_copy) // 0) . "/" . length($simple) . "\n\n";

# Test basic object without spaces (should work)
print "2. Testing basic object:\n";
my $basic_obj = '{}';
$input_copy = $basic_obj;
$result = yapg::GeneratedParser::parse(\$input_copy);
print "   '$basic_obj': " . (defined $result ? "SUCCESS" : "FAILED") . "\n";
print "   Position: " . (pos($input_copy) // 0) . "/" . length($basic_obj) . "\n\n";

# Test if ws rule is even working
print "3. Testing if ws rule exists:\n";
if (yapg::GeneratedParser->can('parse_ws')) {
    print "   ✅ parse_ws function exists\n";
    my $ws_test = "   ";
    my $ws_result = yapg::GeneratedParser::parse_ws(\$ws_test);
    print "   Testing '   ' (3 spaces): " . (defined $ws_result ? "SUCCESS" : "FAILED") . "\n";
} else {
    print "   ❌ parse_ws function does not exist\n";
}

# Let's look at what functions are available
print "\n4. Available parse functions:\n";
no strict 'refs';
my @functions = grep { /^parse_/ } keys %{yapg::GeneratedParser::};
use strict 'refs';
foreach my $func (sort @functions) {
    print "   $func\n";
}

