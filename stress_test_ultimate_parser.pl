#!/usr/bin/env perl
use strict;
use warnings;
use lib "tools/generators";
use Data::Dumper;

# Load the ultimate return annotation parser
require "./tools/generators/ultimate_return_annotation_perl_parser.pm";

print "🚀 ULTIMATE RETURN ANNOTATION PARSER STRESS TEST\n";
print "=" x 60 . "\n\n";

# Test cases organized by complexity
my @test_cases = (
    # BASIC SCALAR REFERENCES
    {
        category => "Basic Scalar References",
        tests => [
            '-> $1',
            '-> $2', 
            '-> $10',
            '-> $99'
        ]
    },
    
    # SIMPLE ARRAYS
    {
        category => "Simple Arrays",
        tests => [
            '-> [$1]',
            '-> [$1, $2]',
            '-> [$1, $2, $3]',
            '-> [$1*]',
            '-> [$2*]',
            '-> [$1+]',
            '-> [$2?]'
        ]
    },
    
    # MIXED ARRAYS (scalars + quantified)
    {
        category => "Mixed Arrays", 
        tests => [
            '-> [$1, $2*]',
            '-> [$1, $2*, $3]',
            '-> [$1*, $2]',
            '-> [$1, $2+, $3]',
            '-> [$1?, $2*, $3+]'
        ]
    },
    
    # SIMPLE OBJECTS
    {
        category => "Simple Objects",
        tests => [
            '-> {type: $1}',
            '-> {name: $1, value: $2}',
            '-> {left: $1, right: $2, op: $3}',
            '-> {type: "literal", value: $1}',
            '-> {type: "array", items: $2}'
        ]
    },
    
    # OBJECTS WITH ARRAYS
    {
        category => "Objects with Arrays",
        tests => [
            '-> {type: $1, items: [$2*]}',
            '-> {name: $1, args: [$2*], body: $3}',
            '-> {op: $1, operands: [$2+]}',
            '-> {type: "function", params: [$1*], body: [$2*]}'
        ]
    },
    
    # DOT NOTATION
    {
        category => "Dot Notation",
        tests => [
            '-> $1.name',
            '-> $1.value.type',
            '-> $2.items[0]',
            '-> $1.properties.name',
            '-> $1.left.value'
        ]
    },
    
    # COMPLEX NESTED STRUCTURES
    {
        category => "Complex Nested Structures",
        tests => [
            '-> {type: $1, children: [{name: $2, value: $3}]}',
            '-> {ast: {type: $1, body: [$2*]}}',
            '-> [$1, {type: $2, items: [$3*]}]',
            '-> {functions: [{name: $1, params: [$2*], body: [$3*]}]}'
        ]
    },
    
    # ARRAY INDEXING AND SLICING
    {
        category => "Array Indexing and Slicing",
        tests => [
            '-> $1[0]',
            '-> $1[1:3]', 
            '-> $1[:2]',
            '-> $1[1:]',
            '-> $1[::2]',
            '-> $1[-1]',
            '-> $2[1:5:2]'
        ]
    },
    
    # EDGE CASES AND STRESS TESTS
    {
        category => "Edge Cases and Stress Tests", 
        tests => [
            '-> {a: $1, b: $2, c: $3, d: $4, e: $5}',
            '-> [$1, $2, $3, $4, $5, $6, $7, $8, $9, $10]',
            '-> {nested: {deeply: {very: {deep: $1}}}}',
            '-> [{type: $1, items: [{name: $2, value: [$3*]}]}]',
            '-> {type: "complex", data: {items: [$1*], meta: {count: $2, flags: [$3*]}}}'
        ]
    },
    
    # QUANTIFIER COMBINATIONS
    {
        category => "Quantifier Combinations",
        tests => [
            '-> [$1?, $2*, $3+]',
            '-> {optional: $1?, repeated: [$2*], required: [$3+]}',
            '-> [$1{2,5}]',
            '-> [$1{0,}]',
            '-> [$1{,3}]'
        ]
    },
    
    # WHITESPACE VARIATIONS
    {
        category => "Whitespace Variations",
        tests => [
            '->$1',
            '-> { type : $1 }',
            '->[ $1 , $2 ]',
            '-> { type: $1 , items: [ $2* ] }',
            '->  $1  .  name  ',
            '-> [ $1 , { type : $2 , value : $3 } ]'
        ]
    }
);

# Run all tests
my $total_tests = 0;
my $passed_tests = 0;
my $failed_tests = 0;
my @failures = ();

foreach my $category (@test_cases) {
    print "📂 Testing: $category->{category}\n";
    print "-" x 40 . "\n";
    
    foreach my $test_case (@{$category->{tests}}) {
        $total_tests++;
        print sprintf("%-50s", "  $test_case");
        
        my $result;
        my $success = 0;
        my $error_msg = "";
        
        eval {
            $result = ultimate_return_annotation_perl_parser::parse(\$test_case);
            if (defined $result) {
                $success = 1;
                $passed_tests++;
                print " ✅ PASS\n";
            } else {
                $failed_tests++;
                $error_msg = "Parser returned undef";
                print " ❌ FAIL (undef)\n";
                push @failures, { test => $test_case, error => $error_msg, category => $category->{category} };
            }
        };
        
        if ($@) {
            $failed_tests++;
            $error_msg = $@;
            chomp $error_msg;
            print " ❌ FAIL ($error_msg)\n";
            push @failures, { test => $test_case, error => $error_msg, category => $category->{category} };
        }
        
        # For passed tests, optionally show structure (only for first few to avoid spam)
        if ($success && $passed_tests <= 3) {
            print "    📋 Result: " . Dumper($result);
        }
    }
    print "\n";
}

# Summary
print "=" x 60 . "\n";
print "🏁 STRESS TEST SUMMARY\n";
print "=" x 60 . "\n";
print "Total Tests:  $total_tests\n";
print "✅ Passed:     $passed_tests\n";
print "❌ Failed:     $failed_tests\n";
print "Success Rate: " . sprintf("%.1f%%", ($passed_tests / $total_tests) * 100) . "\n\n";

# Failure analysis
if (@failures) {
    print "🔍 FAILURE ANALYSIS\n";
    print "-" x 30 . "\n";
    
    my %failure_by_category = ();
    foreach my $failure (@failures) {
        push @{$failure_by_category{$failure->{category}}}, $failure;
    }
    
    foreach my $category (keys %failure_by_category) {
        print "\n📂 $category failures:\n";
        foreach my $failure (@{$failure_by_category{$category}}) {
            print "  ❌ '$failure->{test}'\n";
            print "     Error: $failure->{error}\n";
        }
    }
    
    print "\n🎯 RECOMMENDATIONS\n";
    print "-" x 20 . "\n";
    
    if ($failed_tests > $passed_tests) {
        print "⚠️  HIGH FAILURE RATE - Major parser issues detected\n";
        print "   - Check core grammar rules and token recognition\n";
        print "   - Verify basic scalar and array parsing\n";
    } elsif ($failed_tests > 0) {
        print "⚠️  Some failures detected - Focus on:\n";
        my %error_types = ();
        foreach my $failure (@failures) {
            if ($failure->{error} =~ /undef/) {
                $error_types{"Parser returning undef"}++;
            } elsif ($failure->{error} =~ /syntax/i) {
                $error_types{"Syntax errors"}++;
            } else {
                $error_types{"Other errors"}++;
            }
        }
        foreach my $type (keys %error_types) {
            print "   - $type: $error_types{$type} cases\n";
        }
    }
} else {
    print "🎉 ALL TESTS PASSED! Parser is working excellently.\n";
    print "✨ The ultimate return annotation parser handles all test cases correctly.\n";
}

print "\n" . "=" x 60 . "\n";
print "Stress test completed!\n";
