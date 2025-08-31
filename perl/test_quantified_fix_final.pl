#!/usr/bin/env perl
use strict;
use warnings;
use FindBin qw($RealBin);
use Data::Dumper;

# Add current directory to path
use lib $RealBin;

# Load all required modules
use AST::Transform qw(load_ebnf_spec_from_content step2_group_by_or step2_5_handle_parentheses step3_parse_sequences step4_handle_quantifiers step5_build_tree_structure);
use LeftRecursionIntegrator qw(eliminate_left_recursion_nuclear_option);

print "=== FINAL TEST: QUANTIFIED SEQUENCE IN LEFT-RECURSION ELIMINATION ===\n\n";

# Test grammar with both left-recursion and quantified sequences
my $test_grammar = q{
expr_list := expr ( "," expr )*
expr := 'number'
};

print "Input grammar:\n$test_grammar\n";

# Parse the grammar using the transformation pipeline
my ($grammar_tree, $rule_order) = step5_build_tree_structure(
    step4_handle_quantifiers(
        step3_parse_sequences(
            step2_5_handle_parentheses(
                step2_group_by_or(
                    load_ebnf_spec_from_content($test_grammar)
                )
            )
        )
    )
);

print "\n📋 Parsed grammar structure for expr_list:\n";
print Dumper($grammar_tree->{expr_list});

print "\n🚀 Running left-recursion elimination with nuclear option...\n";

# Apply left-recursion elimination
my ($eliminated_tree, $new_rule_order) = eliminate_left_recursion_nuclear_option($grammar_tree, $rule_order);

print "\n✅ Left-recursion elimination completed successfully!\n";

print "\n📊 Final grammar structure for expr_list:\n";
if (exists $eliminated_tree->{expr_list}) {
    print Dumper($eliminated_tree->{expr_list});
} else {
    print "expr_list rule not found in eliminated grammar\n";
    print "Available rules: " . join(", ", sort keys %$eliminated_tree) . "\n";
}

print "\n📈 Rule count: " . scalar(keys %$eliminated_tree) . " rules\n";
print "Rules: " . join(", ", sort keys %$eliminated_tree) . "\n";

print "\n🎯 TEST RESULT: ";
my $success = 0;
if (exists $eliminated_tree->{expr_list} && ref($eliminated_tree->{expr_list}) eq 'HASH') {
    my $expr_list = $eliminated_tree->{expr_list};
    
    # Check if it has the right structure
    if ($expr_list->{type} eq 'sequence' && 
        ref($expr_list->{elements}) eq 'ARRAY' &&
        @{$expr_list->{elements}} >= 2) {
        
        # Check if second element is properly reconstructed quantified element
        my $second_elem = $expr_list->{elements}->[1];
        if (ref($second_elem) eq 'HASH' &&
            $second_elem->{type} eq 'quantified' &&
            $second_elem->{quantifier} eq '*' &&
            ref($second_elem->{element}) eq 'HASH' &&
            $second_elem->{element}->{type} eq 'sequence') {
            $success = 1;
        }
    }
}

if ($success) {
    print "✅ SUCCESS - Quantified sequences preserved and reconstructed correctly!\n";
} else {
    print "❌ FAILED - Issues with quantified sequence structure\n";
}

print "\n" . "=" x 60 . "\n";
