#!/usr/bin/env perl

use strict;
use warnings;
use Data::Dumper;

# Just test the AST transform pipeline with debug output focused on return annotations
require './ast_transform.pl';

my $file = 'test_return_annotations.ebnf';

print "🔍 DEBUGGING Return Annotation Generation\n";
print "=" x 50 . "\n";

# Parse the EBNF file
my $ast = parse_ebnf_file($file);
print "Raw AST scalar_test rule:\n";
print Dumper($ast->[0]); # First rule (scalar_test)
print "\n" . "=" x 50 . "\n";

# Apply all steps and see what happens to return annotations
my ($processed_ast, $regexes) = step1_clean_ast($ast);
my ($normalized_ast) = step2_normalize_rules($processed_ast);
my ($recursion_free_ast) = step3_eliminate_left_recursion($normalized_ast);
my ($optimized_ast) = step4_apply_optimizations($recursion_free_ast, $regexes);
my ($grammar_tree, $rule_order) = step5_build_tree_structure($optimized_ast);

print "After Step 5 - scalar_test structure:\n";
print Dumper($grammar_tree->{scalar_test});
print "\n" . "=" x 50 . "\n";

# Now test the parser generation for scalar_test specifically
print "Testing generate_sequence_parser for scalar_test...\n";
my $scalar_rule = $grammar_tree->{scalar_test};
if ($scalar_rule && $scalar_rule->{type} eq 'sequence') {
    my $parser_code = generate_sequence_parser('scalar_test', $scalar_rule, $regexes);
    print "Generated parser code:\n";
    print $parser_code;
} else {
    print "ERROR: scalar_test is not a sequence type!\n";
    print "Type: " . ($scalar_rule->{type} || 'undefined') . "\n";
}

