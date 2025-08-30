#!/usr/bin/env perl
use strict;
use warnings;
use Data::Dumper;
use ultimate_return_annotation_perl_parser;

print "Successfully loaded ultimate parser!\n\n";

# Test cases
my @test_cases = (
    '-> $1',
    '-> [$1*]', 
    '-> {items: [$1*]}',
    '-> {key: $1, value: $2}',
    '-> $1.name'
);

print "Testing ultimate return annotation parser...\n\n";

foreach my $test (@test_cases) {
    print "Testing: '$test'\n";
    
    my $result;
    eval {
        $result = ultimate_return_annotation_perl_parser::parse(\$test);
    };
    
    if ($@) {
        print "  ERROR: $@\n";
    } elsif (!defined $result) {
        print "  RESULT: undef (parse failed)\n";
    } else {
        print "  RESULT: " . Dumper($result);
    }
    print "\n";
}
