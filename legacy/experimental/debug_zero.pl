#!/usr/bin/env perl
use strict;
use warnings;

# Debug why '0' fails to parse

print "Testing number regex pattern...\n\n";

my $pattern = '-?[0-9]+(\.[0-9]+)?';
my @test_numbers = ('0', '123', '-456', '12.34', '-56.78');

foreach my $num (@test_numbers) {
    print "Testing '$num': ";
    if ($num =~ /^$pattern$/) {
        print "✅ MATCHES\n";
    } else {
        print "❌ NO MATCH\n";
    }
}

print "\nTesting with \\G anchor (parser style):\n";
foreach my $num (@test_numbers) {
    my $input = $num;
    pos($input) = 0;
    print "Testing '$num': ";
    if ($input =~ /\G($pattern)/gc) {
        print "✅ MATCHES (captured: '$1')\n";
    } else {
        print "❌ NO MATCH\n";
    }
}

