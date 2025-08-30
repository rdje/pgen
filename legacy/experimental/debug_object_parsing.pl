#!/usr/bin/env perl
use strict;
use warnings;

require './json_final_parser.pm';

print "Debugging object parsing step by step...\n\n";

# Test simple cases first
my @simple_tests = (
    '{}',                    # Empty object
    '{"key": "value"}',      # Simple object
    '{"a": 1}',             # Object with number
);

foreach my $test (@simple_tests) {
    print "Testing: '$test'\n";
    my $input_copy = $test;
    my $result = yapg::GeneratedParser::parse(\$input_copy);
    my $success = defined $result && pos($input_copy) == length($test);
    print "  Result: " . ($success ? "✅ SUCCESS" : "❌ FAILED") . "\n";
    
    if (!$success) {
        print "  Position: " . (pos($input_copy) // 0) . "/" . length($test) . "\n";
        # Try parsing object directly
        my $input_copy2 = $test;
        my $obj_result = yapg::GeneratedParser::parse_object(\$input_copy2);
        print "  Direct parse_object: " . (defined $obj_result ? "SUCCESS" : "FAILED") . "\n";
        
        # Try parsing value directly  
        my $input_copy3 = $test;
        my $val_result = yapg::GeneratedParser::parse_value(\$input_copy3);
        print "  Direct parse_value: " . (defined $val_result ? "SUCCESS" : "FAILED") . "\n";
    }
    print "\n";
}

# Check what the main parse function is doing
print "=== Checking main parse function ===\n";
my $simple_obj = '{}';
print "Testing main parse on '$simple_obj':\n";

my $input_copy = $simple_obj;
print "1. Initial position: " . pos($input_copy) . "\n";

# The main parse function should call the first rule
# Let's see what the first rule is in our JSON grammar
print "2. First rule in JSON grammar should be 'value'\n";

my $result = yapg::GeneratedParser::parse(\$input_copy);
print "3. Main parse result: " . (defined $result ? "SUCCESS" : "FAILED") . "\n";
print "4. Position after: " . (pos($input_copy) // 0) . "\n";

