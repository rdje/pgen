#!/usr/bin/env perl
use strict;
use warnings;

# Debug JSON value parsing
require './json_parser.pm';

print "Debugging JSON value parsing...\n\n";

# Test individual parsing functions
my @tests = (
    {input => 'true', func => 'parse_value', desc => 'value(true)'},
    {input => '0', func => 'parse_value', desc => 'value(0)'},
    {input => '0', func => 'parse_digit', desc => 'digit(0)'},
    {input => '0', func => 'parse_number', desc => 'number(0)'},
);

foreach my $test (@tests) {
    my $input = $test->{input};
    my $func = $test->{func};
    my $desc = $test->{desc};
    
    my $input_copy = $input;
    my $result = eval {
        no strict 'refs';
        my $parser_func = "yapg::GeneratedParser::$func";
        $parser_func->(\$input_copy);
    };
    
    print "$desc: ";
    if (defined $result) {
        print "✅ SUCCESS (result: '$result')\n";
    } else {
        print "❌ FAILED\n";
    }
    
    if ($@) {
        print "  Error: $@\n";
    }
}

# Also test the regexes directly
print "\nTesting regexes directly...\n";
my $test_input = 'true';
if ($test_input =~ /\Qtrue\E/) {
    print "✅ Direct regex /\\Qtrue\\E/ matches 'true'\n";
} else {
    print "❌ Direct regex /\\Qtrue\\E/ fails on 'true'\n";
}

# Test with actual regex from parser
print "\nTesting actual parser regex...\n";
my $input_ref = \$test_input;
pos($$input_ref) = 0;
if ($$input_ref =~ /\G(qr\/\Qtrue\E\/o)/gc) {
    print "✅ Parser regex matches\n";
} else {
    print "❌ Parser regex fails\n";
}

