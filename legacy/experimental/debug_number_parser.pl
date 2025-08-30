#!/usr/bin/env perl
use strict;
use warnings;

# Debug the specific number parser function
require './json_v2_parser.pm';

print "Testing parse_number function directly...\n\n";

my @test_numbers = ('0', '123', '-456', '12.34', '-56.78');

foreach my $num (@test_numbers) {
    my $input_copy = $num;
    print "Testing parse_number('$num'): ";
    
    my $result = eval {
        yapg::GeneratedParser::parse_number(\$input_copy);
    };
    
    if (defined $result) {
        print "✅ SUCCESS (result: '$result')\n";
    } else {
        print "❌ FAILED\n";
    }
    
    if ($@) {
        print "  Error: $@\n";
    }
}

print "\nTesting main parse function (which should use entry point)...\n";
foreach my $num (@test_numbers) {
    my $input_copy = $num;
    print "Testing parse('$num'): ";
    
    my $result = eval {
        yapg::GeneratedParser::parse(\$input_copy);
    };
    
    if (defined $result) {
        print "✅ SUCCESS\n";
    } else {
        print "❌ FAILED\n";
    }
}

