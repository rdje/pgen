#!/usr/bin/perl
use strict;
use warnings;
use Data::Dumper;

# Load ast_transform.pl to access its functions
require "./ast_transform.pl";

# Test with a simple case
my $test_grammar = "test_current_complex.ebnf";

# Override the file check in ast_transform.pl
$ARGV[0] = $test_grammar;

print "=== DEBUGGING RETURN ANNOTATION PROCESSING ===\n";

# Let me examine the tree structure right before code generation
my $grammar_tree = read_ebnf_grammar($test_grammar);

print "=== FINAL TREE BEFORE CODE GENERATION ===\n";
print "expr rule: " . Dumper($grammar_tree->{expr});

if ($grammar_tree->{expr}) {
    my $expr_rule = $grammar_tree->{expr};
    print "expr rule type: " . $expr_rule->{type} . "\n";
    
    if ($expr_rule->{type} eq 'sequence') {
        print "expr is a sequence\n";
        print "return_annotation: " . Dumper($expr_rule->{return_annotation});
    } elsif ($expr_rule->{type} eq 'or') {
        print "expr is an OR rule with " . scalar(@{$expr_rule->{alternatives}}) . " alternatives\n";
        for my $i (0..$#{$expr_rule->{alternatives}}) {
            my $alt = $expr_rule->{alternatives}[$i];
            print "Alternative $i: " . Dumper($alt->{return_annotation});
        }
    }
}
