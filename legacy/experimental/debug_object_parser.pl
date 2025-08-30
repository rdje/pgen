#!/usr/bin/env perl
use strict;
use warnings;

# Debug why parse_object is succeeding on '0'
require './json_v3_parser.pm';

print "Debug parse_object('0') issue...\n\n";

my $input = '0';
pos($input) = 0;
print "Before parse_object: input='$input', pos=" . pos($input) . "\n";

my $result = yapg::GeneratedParser::parse_object(\$input);
print "After parse_object: result=" . (defined $result ? "'$result'" : "undef") . ", pos=" . pos($input) . "\n";

# Check what object alternatives exist
print "\nChecking object regex patterns...\n";
my $input2 = '0';
if ($input2 =~ /\G\{/gc) {
    print "✅ '0' matches opening brace regex\n";
} else {
    print "❌ '0' does not match opening brace regex\n";
}

if ($input2 =~ /\G\}/gc) {
    print "✅ '0' matches closing brace regex\n";
} else {
    print "❌ '0' does not match closing brace regex\n";
}

# Test individual components
print "\nTesting individual object components...\n";
my $input3 = '{}';
pos($input3) = 0;
my $obj_result = yapg::GeneratedParser::parse_object(\$input3);
print "parse_object('{}'): " . (defined $obj_result ? "SUCCESS" : "FAILED") . "\n";

my $input4 = '0';
pos($input4) = 0;
my $obj_result2 = yapg::GeneratedParser::parse_object(\$input4);
print "parse_object('0'): " . (defined $obj_result2 ? "SUCCESS" : "FAILED") . "\n";

