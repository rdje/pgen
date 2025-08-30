#!/usr/bin/perl
use strict;
use warnings;
use Data::Dumper;
use lib 'fx/perl';
use LinkedSpec;

print "=== Debugging EBNF Parsing ===\n\n";

# Parse the malformed EBNF using LinkedSpec to see what AST we get
print "Parsing malformed.ebnf with LinkedSpec...\n";

my $raw_ast = LinkedSpec::Get("fx/specs/ebnf.spec", "stability_test_results/malformed.ebnf");

print "Raw AST from LinkedSpec:\n";
print Dumper($raw_ast);

if ($raw_ast) {
    print "\nAST contains " . scalar(@$raw_ast) . " rules\n";
    for my $i (0 .. $#$raw_ast) {
        print "Rule $i: " . Dumper($raw_ast->[$i]);
    }
} else {
    print "\nNo AST returned - parsing failed\n";
}


