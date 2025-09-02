#!/usr/bin/perl
use strict;
use warnings;
use lib 'perl';
use FindBin qw($RealBin);

# Test the complete pipeline: EBNF → Transform → DataGenerator

use AST::Transform qw(process_to_final_ast);
use JSON::PP;

print "=== Testing Complete DataGenerator Pipeline ===\n\n";

# Test 1: Create a simple grammar in memory
print "Step 1: Creating simple test grammar...\n";

my $simple_raw_ast = [
    [
        ["rule", "expression"],
        ["rule_reference", "term"],
        ["group_open", "("],
        ["quoted_string", "+"],
        ["rule_reference", "term"],
        ["group_close", ")"],
        ["operator", "*"]
    ],
    [
        ["rule", "term"],
        ["regex", "(\\d+)"]
    ]
];

print "Created simple arithmetic grammar with " . scalar(@$simple_raw_ast) . " rules.\n\n";

# Step 2: Transform to final AST
print "Step 2: Transforming to final AST...\n";

my ($grammar_tree, $rule_order) = process_to_final_ast($simple_raw_ast);

print "Transformation complete. Rules: " . join(", ", @$rule_order) . "\n";
print "Grammar tree keys: " . join(", ", keys %$grammar_tree) . "\n\n";

# Step 3: Save as JSON for DataGenerator
print "Step 3: Saving transformed grammar as JSON...\n";

my $json_data = {
    grammar_name => "simple_arithmetic",
    grammar_tree => $grammar_tree,
    rule_order => $rule_order,
    metadata => {
        source => "test_data_generator_pipeline.pl",
        format => "transformed_ast",
        generated_at => scalar(localtime())
    }
};

my $json = JSON::PP->new->pretty->encode($json_data);
my $json_file = "simple_arithmetic_grammar.json";

open my $fh, '>', $json_file or die "Cannot write $json_file: $!";
print $fh $json;
close $fh;

print "Saved transformed grammar to: $json_file\n\n";

# Step 4: Generate test data using Python DataGenerator
print "Step 4: Generating test data...\n";

print "\n--- Generating from 'expression' rule ---\n";
my $cmd1 = "python3 tools/syntactic_data_generator.py $json_file --rule expression --count 5 --stats";
print "Running: $cmd1\n";
system($cmd1);

print "\n--- Generating from 'term' rule ---\n";
my $cmd2 = "python3 tools/syntactic_data_generator.py $json_file --rule term --count 3 --stats";
print "Running: $cmd2\n";
system($cmd2);

print "\n=== Pipeline Test Complete ===\n";

# Step 5: Show the JSON structure for debugging
print "\nGenerated JSON structure preview:\n";
print "Grammar tree type for 'expression': " . ($grammar_tree->{expression}->{type} || 'unknown') . "\n";
print "Grammar tree type for 'term': " . ($grammar_tree->{term}->{type} || 'unknown') . "\n";

if ($grammar_tree->{expression}->{elements}) {
    print "Expression has " . scalar(@{$grammar_tree->{expression}->{elements}}) . " elements\n";
}
if ($grammar_tree->{expression}->{alternatives}) {
    print "Expression has " . scalar(@{$grammar_tree->{expression}->{alternatives}}) . " alternatives\n";
}
