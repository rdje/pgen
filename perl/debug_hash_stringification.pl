#!/usr/bin/perl
use strict;
use warnings;
use lib '.';
use Data::Dumper;
use AST::Transform qw(step2_group_by_or step2_5_handle_parentheses step3_parse_sequences step4_handle_quantifiers);

# Enable debug mode
$AST::Transform::quiet_mode = 0;
$AST::Transform::verbosity = 'debug';

# Create a simple test case that exhibits the issue
my $test_rule = [
    ['rule', 'test_rule'],
    ['('],
    ['rule', 'expression'],
    [')'],
    ['operator', '*']
];

print "=== INPUT TEST RULE ===\n";
print Dumper($test_rule);

# Step 1: Group by OR (this should work fine)
my $step1_result = [$test_rule];  # Wrap in array as expected
my $step2_result = step2_group_by_or($step1_result);

print "\n=== STEP 2 RESULT ===\n";
print Dumper($step2_result);

# Step 2.5: Handle parentheses
my $step2_5_result = step2_5_handle_parentheses($step2_result);

print "\n=== STEP 2.5 RESULT ===\n";
print Dumper($step2_5_result);

# Step 3: Parse sequences
my $step3_result = step3_parse_sequences($step2_5_result);

print "\n=== STEP 3 RESULT ===\n";
print Dumper($step3_result);

# Step 4: Handle quantifiers (this is where the issue occurs)
my $step4_result = step4_handle_quantifiers($step3_result);

print "\n=== STEP 4 RESULT ===\n";
print Dumper($step4_result);

# Analyze the quantified elements in detail
print "\n=== DETAILED ANALYSIS OF QUANTIFIED ELEMENTS ===\n";
foreach my $rule (@$step4_result) {
    if ($rule->{elements}) {
        foreach my $element (@{$rule->{elements}}) {
            if ($element->{type} && $element->{type} eq 'quantified') {
                print "Found quantified element:\n";
                print "  Type: " . ref($element) . "\n";
                print "  Element type: " . ref($element->{element}) . "\n";
                print "  Element value: " . ($element->{element} || 'undef') . "\n";
                if (ref($element->{element}) eq 'HASH') {
                    print "  Element is a hash reference - GOOD\n";
                } elsif (!ref($element->{element})) {
                    if ($element->{element} =~ /^HASH\(/) {
                        print "  Element is a stringified hash reference - BAD!\n";
                        print "  Raw element: " . $element->{element} . "\n";
                    } else {
                        print "  Element is a simple scalar\n";
                    }
                } else {
                    print "  Element is a different ref type: " . ref($element->{element}) . "\n";
                }
                print "  Full element structure:\n";
                print Dumper($element);
                print "---\n";
            }
        }
    }
}
