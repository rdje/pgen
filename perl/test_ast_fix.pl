#!/usr/bin/env perl

use strict;
use warnings;
use lib '.';
use Data::Dumper;

# Load our modules
use AST::UniversalReturnAnnotation;
use AST::Transform qw(ebnf_to_universal_ast);

print "Testing AST::Transform fix for scalar_ref index extraction...\n\n";

# Test case 1: Simulate the problematic nested index structure
my $test_node = {
    type => 'scalar_ref',
    index => {
        value => '1', 
        type => 'positive'
    }
};

print "Input node structure:\n";
print Dumper($test_node);

# Test our transformation function
my $result = ebnf_to_universal_ast($test_node);

print "Result from ebnf_to_universal_ast:\n";
print Dumper($result);

# Test AST validation
if ($result) {
    print "\nTesting AST validation...\n";
    eval {
        my $validation_result = AST::UniversalReturnAnnotation::validate_ast($result);
        print "✅ AST validation passed!\n";
        print "Validation result: $validation_result\n";
    };
    if ($@) {
        print "❌ AST validation failed: $@\n";
    }
} else {
    print "❌ ebnf_to_universal_ast returned undef\n";
}

# Test case 2: Verify we don't break normal scalar_ref with simple index
print "\n" . "="x50 . "\n";
print "Testing with simple numeric index...\n";

my $simple_node = {
    type => 'scalar_ref',
    index => '2'
};

print "Input node structure:\n";
print Dumper($simple_node);

my $simple_result = ebnf_to_universal_ast($simple_node);

print "Result from ebnf_to_universal_ast:\n";
print Dumper($simple_result);

if ($simple_result) {
    print "\nTesting AST validation...\n";
    eval {
        my $validation_result = AST::UniversalReturnAnnotation::validate_ast($simple_result);
        print "✅ AST validation passed!\n";
        print "Validation result: $validation_result\n";
    };
    if ($@) {
        print "❌ AST validation failed: $@\n";
    }
}

print "\nDone!\n";
