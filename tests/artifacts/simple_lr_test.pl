#!/usr/bin/perl
use strict;
use warnings;
use lib '.';
use TestLRFixed;

# Test basic functionality
my $input1 = "num";
my $input2 = "num+num";

print "Testing: '$input1'\n";
my $result1 = TestLRFixed::parse(\$input1);
print "Result: " . (defined $result1 ? "\"$result1\"" : "undef") . "\n\n";

print "Testing: '$input2'\n"; 
my $result2 = TestLRFixed::parse(\$input2);
print "Result: " . (defined $result2 ? "\"$result2\"" : "undef") . "\n\n";
