#!/usr/bin/env perl
use strict;
use warnings;
use lib "tools/generators";
use Data::Dumper;

require "./tools/generators/ultimate_return_annotation_perl_parser.pm";

print "🔍 DEBUGGING ARRAY PARSING ISSUES\n";
print "=" x 50 . "\n\n";

# Progressive array complexity tests
my @debug_tests = (
    # Single element arrays (known to work)
    "-> [\$1]",
    
    # Two element arrays (known to fail)  
    "-> [\$1, \$2]",
    
    # Variations to isolate the issue
    "-> [\$1,\$2]",           # No spaces
    "-> [\$1 , \$2]",         # Spaces around comma
    "-> [ \$1, \$2 ]",        # Spaces inside brackets
    "-> [ \$1 , \$2 ]",       # All spaces
    
    # Try with different numbers
    "-> [\$1, \$3]",
    "-> [\$2, \$1]",
    
    # Three elements
    "-> [\$1, \$2, \$3]",
    
    # Mixed with quantified (known to fail)
    "-> [\$1, \$2*]",
    
    # Objects that work
    "-> {a: \$1, b: \$2}",     # This works - why?
    "-> {a: \$1, b: \$2, c: \$3}",  # This works too
);

foreach my $test (@debug_tests) {
    print sprintf("%-30s", $test);
    
    my $result;
    eval {
        $result = ultimate_return_annotation_perl_parser::parse(\$test);
    };
    
    if ($@) {
        print " ❌ ERROR: $@\n";
    } elsif (defined $result) {
        print " ✅ SUCCESS\n";
        # Show abbreviated result
        if (ref($result) eq 'ARRAY' && @$result >= 3) {
            my $expr = $result->[2];
            if (ref($expr) eq 'HASH') {
                print "    Type: $expr->{type}\n";
                if ($expr->{type} eq 'array') {
                    print "    Elements: " . scalar(@{$expr->{contents} || []}) . "\n";
                }
            }
        }
    } else {
        print " ❌ FAILED (undef)\n";
    }
}

print "\n" . "=" x 50 . "\n";

# Let's also test some working cases for comparison
print "\n🔍 TESTING WORKING PATTERNS FOR COMPARISON\n";
print "-" x 30 . "\n";

my @working_tests = (
    "-> [\$1*]",                    # Quantified array - works
    "-> {a: \$1, b: \$2}",          # Multi-property object - works  
    "-> {type: \$1, items: [\$2*]}", # Object with array - works
);

foreach my $test (@working_tests) {
    print sprintf("%-30s", $test);
    
    my $result;
    eval {
        $result = ultimate_return_annotation_perl_parser::parse(\$test);
    };
    
    if ($@ || !defined $result) {
        print " ❌ UNEXPECTED FAILURE\n";
    } else {
        print " ✅ WORKS AS EXPECTED\n";
        if (ref($result) eq 'ARRAY' && @$result >= 3) {
            my $expr = $result->[2];
            if (ref($expr) eq 'HASH') {
                print "    Type: $expr->{type}\n";
            }
        }
    }
}

print "\n🎯 ANALYSIS\n";
print "-" x 10 . "\n";
print "The issue appears to be with parsing comma-separated elements in arrays.\n";
print "Multi-property objects work fine, but multi-element arrays fail.\n";
print "This suggests the grammar rule for array_contents or array parsing\n";
print "may not be correctly handling comma-separated scalar references.\n";
