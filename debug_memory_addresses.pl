#!/usr/bin/perl
use strict;
use warnings;
use Data::Dumper;
use FindBin qw($RealBin);
use lib "$RealBin/perl";
use AST::Transform qw(load_ebnf_spec process_transformation_phases);

# Simple test case
my $test_content = <<'EOF';
list := item (',' item)*
item := /[a-z]+/
EOF

# Write test file
open my $fh, '>', 'debug_test.ebnf' or die $!;
print $fh $test_content;
close $fh;

# Load and process with detailed debugging
my $raw_ast = AST::Transform::load_ebnf_spec('debug_test.ebnf');
print "RAW AST:\n" . Dumper($raw_ast) . "\n";

# Process step by step
my $step2_result = AST::Transform::step2_group_by_or($raw_ast);
print "STEP 2 RESULT:\n" . Dumper($step2_result) . "\n";

my $step2_5_result = AST::Transform::step2_5_handle_parentheses($step2_result);
print "STEP 2.5 RESULT:\n" . Dumper($step2_5_result) . "\n";

my $step3_result = AST::Transform::step3_parse_sequences($step2_5_result);
print "STEP 3 RESULT:\n" . Dumper($step3_result) . "\n";

my $step4_result = AST::Transform::step4_handle_quantifiers($step3_result);
print "STEP 4 RESULT:\n" . Dumper($step4_result) . "\n";

# Clean up
unlink 'debug_test.ebnf';
