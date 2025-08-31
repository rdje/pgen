#!/usr/bin/perl
use strict;
use warnings;
use lib '.';
use Data::Dumper;
use AST::Transform;

# Enable debug mode
$AST::Transform::quiet_mode = 0;
$AST::Transform::verbosity = 'debug';

# Test input that should create grouped structure
my $test_rule = [
    ['rule', 'expression_list'],
    ['rule', 'expression'],
    ['('],
    ['quoted_string', ','],
    ['rule', 'expression'],
    [')'],
    ['operator', '*']
];

print "=== TESTING STEP 2.5 PARENTHESES HANDLING ===\n";
print "Input: expression_list = expression ( \",\" expression )*\n\n";

# Step 1 & 2
my $step1_result = [$test_rule];
my $step2_result = AST::Transform::step2_group_by_or($step1_result);

print "After Step 2 (group by OR):\n";
print Dumper($step2_result);

# Check the or_groups structure
foreach my $rule (@$step2_result) {
    print "Rule: " . $rule->{name} . "\n";
    print "OR groups count: " . scalar(@{$rule->{or_groups}}) . "\n";
    for my $i (0..$#{$rule->{or_groups}}) {
        print "  OR group $i:\n";
        my @group = @{$rule->{or_groups}->[$i]};
        for my $j (0..$#group) {
            my $token = $group[$j];
            if (ref($token) eq 'ARRAY') {
                print "    Token $j: [" . join(", ", @$token) . "]\n";
            } else {
                print "    Token $j: $token\n";
            }
        }
    }
}

print "\n=== NOW TESTING STEP 2.5 ===\n";

# Step 2.5: Handle parentheses
my $step2_5_result = AST::Transform::step2_5_handle_parentheses($step2_result);

print "After Step 2.5 (handle parentheses):\n";
print Dumper($step2_5_result);

# Analyze the result
foreach my $rule (@$step2_5_result) {
    print "Rule: " . $rule->{name} . "\n";
    print "OR groups count: " . scalar(@{$rule->{or_groups}}) . "\n";
    for my $i (0..$#{$rule->{or_groups}}) {
        print "  OR group $i:\n";
        my $group_ref = $rule->{or_groups}->[$i];
        if (ref($group_ref) eq 'ARRAY') {
            my @group = @$group_ref;
            for my $j (0..$#group) {
                my $token = $group[$j];
                if (ref($token) eq 'ARRAY') {
                    if (@$token >= 2 && $token->[0] eq 'GROUPED') {
                        print "    Token $j: GROUPED structure with " . scalar(@{$token->[1]}) . " elements\n";
                        print "      Group content:\n";
                        my $group_content = $token->[1];
                        if (ref($group_content) eq 'ARRAY') {
                            for my $k (0..$#{$group_content}) {
                                my $inner_token = $group_content->[$k];
                                if (ref($inner_token) eq 'ARRAY') {
                                    print "        Element $k: [" . join(", ", @$inner_token) . "]\n";
                                } else {
                                    print "        Element $k: $inner_token\n";
                                }
                            }
                        }
                    } else {
                        print "    Token $j: [" . join(", ", @$token) . "]\n";
                    }
                } else {
                    print "    Token $j: $token\n";
                }
            }
        }
    }
}

print "\n=== TESTING process_parentheses_in_sequence DIRECTLY ===\n";

# Test the function directly on our token sequence
my @test_tokens = (
    ['rule', 'expression'],
    ['('],
    ['quoted_string', ','],
    ['rule', 'expression'],
    [')'],
    ['operator', '*']
);

print "Direct input tokens:\n";
for my $i (0..$#test_tokens) {
    my $token = $test_tokens[$i];
    print "  Token $i: [" . join(", ", @$token) . "]\n";
}

my $direct_result = AST::Transform::process_parentheses_in_sequence(\@test_tokens);

print "\nDirect result from process_parentheses_in_sequence:\n";
print Dumper($direct_result);
