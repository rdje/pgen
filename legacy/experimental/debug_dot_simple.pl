#!/usr/bin/perl
use strict;
use warnings;
use Data::Dumper;

# Load the generated parser
require './step13_parser_fixed.pl';

print "=== Testing Simple Dot Notation ===\n\n";

my @simple_tests = (
    '-> $1.name',
    '-> $2.items', 
    '-> $1.0',
    '-> $3.data.count',
);

foreach my $test_input (@simple_tests) {
    print "Testing: $test_input\n";
    
    eval {
        my $result = yapg::GeneratedParser::parse_return_annotation(\$test_input);
        
        if ($result && ref($result) eq 'ARRAY' && $result->[2]) {
            my $parsed = $result->[2];
            print "Type: $parsed->{type}\n";
            
            if ($parsed->{type} eq 'dot_notation_ref') {
                print "SUCCESS: Dot notation detected!\n";
                print "Base: " . Dumper($parsed->{base});
                print "Path: " . Dumper($parsed->{path});
            } else {
                print "ISSUE: Parsed as $parsed->{type} instead of dot_notation_ref\n";
            }
        } else {
            print "FAILED: No result\n";
        }
    };
    
    if ($@) {
        print "ERROR: $@\n";
    }
    
    print "\n" . ("=" x 50) . "\n\n";
}
