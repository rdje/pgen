#!/usr/bin/perl
use strict;
use warnings;
use lib '.';
use Data::Dumper;
use AST::Transform;

# Enable debug mode
$AST::Transform::quiet_mode = 0;
$AST::Transform::verbosity = 'debug';

# Test the actual grouped quantifier from our test grammar
# This simulates: expression_list = expression ( "," expression )*
my $test_rule = [
    ['rule', 'expression_list'],
    ['rule', 'expression'],
    ['('],
    ['quoted_string', ','],
    ['rule', 'expression'],
    [')'],
    ['operator', '*']
];

print "=== TESTING GROUPED QUANTIFIER PATTERN ===\n";
print "Input rule: expression_list = expression ( \",\" expression )*\n";
print Dumper($test_rule);

# Process through all transformation steps
my $step1_result = [$test_rule];
my $step2_result = AST::Transform::step2_group_by_or($step1_result);
my $step2_5_result = AST::Transform::step2_5_handle_parentheses($step2_result);
my $step3_result = AST::Transform::step3_parse_sequences($step2_5_result);

print "\n=== STEP 3 RESULT (BEFORE QUANTIFIERS) ===\n";
print Dumper($step3_result);

# Step 4: This should create grouped quantifiers
my $step4_result = AST::Transform::step4_handle_quantifiers($step3_result);

print "\n=== STEP 4 RESULT (AFTER QUANTIFIERS) ===\n";
print Dumper($step4_result);

# Analyze quantified elements
print "\n=== QUANTIFIED ELEMENT ANALYSIS ===\n";
foreach my $rule (@$step4_result) {
    if ($rule->{elements}) {
        my $elem_index = 0;
        foreach my $element (@{$rule->{elements}}) {
            $elem_index++;
            print "Element $elem_index:\n";
            print "  Type: " . ($element->{type} || 'unknown') . "\n";
            
            if ($element->{type} && $element->{type} eq 'quantified') {
                print "  This is a quantified element!\n";
                print "  Quantifier: " . ($element->{quantifier} || 'unknown') . "\n";
                print "  Element type: " . ref($element->{element}) . "\n";
                
                if (ref($element->{element}) eq 'HASH') {
                    print "  Element is hash - GOOD\n";
                    if ($element->{element}->{type} && $element->{element}->{type} eq 'sequence') {
                        print "    Sequence with " . scalar(@{$element->{element}->{elements}}) . " elements\n";
                    }
                } elsif (ref($element->{element}) eq 'ARRAY') {
                    print "  Element is array - analyzing...\n";
                    if (@{$element->{element}} >= 2 && $element->{element}->[0] eq 'GROUPED') {
                        print "    This looks like a GROUPED structure!\n";
                        print "    Group contents: " . Dumper($element->{element}->[1]);
                    }
                } elsif (!ref($element->{element})) {
                    print "  Element is scalar: " . $element->{element} . "\n";
                    if ($element->{element} =~ /^HASH\(/) {
                        print "    ERROR: Stringified hash detected!\n";
                    }
                } else {
                    print "  Element is other ref type: " . ref($element->{element}) . "\n";
                }
                
                print "  Full element:\n" . Dumper($element);
            }
            print "---\n";
        }
    }
}

# Test the generate_universal_quantified_step function directly
print "\n=== TESTING generate_universal_quantified_step DIRECTLY ===\n";
# Find a quantified element to test with
foreach my $rule (@$step4_result) {
    if ($rule->{elements}) {
        foreach my $element (@{$rule->{elements}}) {
            if ($element->{type} && $element->{type} eq 'quantified') {
                print "Testing generate_universal_quantified_step with this element:\n";
                print Dumper($element);
                
                # Create empty regexes array for testing
                my @test_regexes = ();
                
                # Call the function directly
                my $result = AST::Transform::generate_universal_quantified_step(
                    $element, "test_rule", 1, \@test_regexes
                );
                
                print "Result from generate_universal_quantified_step:\n";
                print ($result || "undef") . "\n";
                
                last;
            }
        }
    }
}
