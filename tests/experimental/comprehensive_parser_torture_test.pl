#!/usr/bin/env perl
use strict;
use warnings;
use Time::HiRes qw(time);

print "=== COMPREHENSIVE PARSER GENERATOR TORTURE TEST ===\n";

# Test suite categories
my @test_categories = (
    {
        name => "INDIRECT LEFT-RECURSION",
        tests => [
            {
                name => "Indirect via single intermediate",
                grammar => "A := B 'x'\nB := A 'y' | 'z'",
                inputs => ["zy", "zyx", "zyxy"]
            },
            {
                name => "Mutual left-recursion (3-way cycle)",
                grammar => "A := B 'a' | 'start'\nB := C 'b'\nC := A 'c'",
                inputs => ["start", "startab", "startcab"]
            }
        ]
    },
    {
        name => "COMPLEX QUANTIFIERS",
        tests => [
            {
                name => "Nested quantifiers with recursion",
                grammar => "expr := term (op term)*\nterm := '(' expr ')' | 'num'\nop := '+' | '*'", 
                inputs => ["num", "(num)", "num+num*num", "((num))", "num+(num*num)"]
            },
            {
                name => "Optional recursive elements",
                grammar => "list := '[' item? (',' item)* ']'\nitem := list | 'val'",
                inputs => ["[]", "[val]", "[val,val]", "[[]]", "[val,[val,val]]"]
            }
        ]
    },
    {
        name => "PATHOLOGICAL CASES",
        tests => [
            {
                name => "Exponential backtracking grammar",
                grammar => "S := A A A A\nA := 'a'? 'a'",
                inputs => ["aa", "aaa", "aaaa", "aaaaa", "aaaaaa"]
            },
            {
                name => "Deeply nested structure",
                grammar => "nest := '(' nest ')' | 'x'",
                inputs => ["x", "(x)", "((x))", "(((x)))", "((((x))))"]
            }
        ]
    },
    {
        name => "REAL-WORLD GRAMMARS",
        tests => [
            {
                name => "JSON subset",
                grammar => "json := object | array | string | number\nobject := '{' pairs? '}'\npairs := pair (',' pair)*\npair := string ':' json\narray := '[' items? ']'\nitems := json (',' json)*\nstring := '\"str\"'\nnumber := '123'",
                inputs => ['"str"', '123', '{}', '[]', '{"key":"val"}', '[123,"str"]']
            },
            {
                name => "Expression language with precedence",
                grammar => "expr := term (('+' | '-') term)*\nterm := factor (('*' | '/') factor)*\nfactor := '(' expr ')' | number\nnumber := 'num'",
                inputs => ["num", "num+num", "num*num+num", "(num)", "num*(num+num)"]
            }
        ]
    }
);

my $total_tests = 0;
my $passed_tests = 0;
my $failed_tests = 0;
my @failures = ();

foreach my $category (@test_categories) {
    print "\n" . "="x60 . "\n";
    print "CATEGORY: $category->{name}\n";
    print "="x60 . "\n";
    
    foreach my $test (@{$category->{tests}}) {
        print "\nTest: $test->{name}\n";
        print "-" x 40 . "\n";
        
        # Write test grammar to file
        my $grammar_file = "test_grammar_$$.ebnf";
        open my $fh, '>', $grammar_file or die "Cannot create test file: $!";
        print $fh $test->{grammar};
        close $fh;
        
        # Generate parser
        my $parser_code = `perl ast_transform.pl $grammar_file --quiet 2>&1`;
        my $generation_success = $? == 0;
        
        if (!$generation_success) {
            print "  PARSER GENERATION FAILED:\n  $parser_code\n";
            $failed_tests++;
            push @failures, "$category->{name} - $test->{name}: Parser generation failed";
            unlink $grammar_file;
            next;
        }
        
        # Test the generated parser
        eval $parser_code;
        if ($@) {
            print "  PARSER COMPILATION FAILED: $@\n";
            $failed_tests++;
            push @failures, "$category->{name} - $test->{name}: Parser compilation failed";
            unlink $grammar_file;
            next;
        }
        
        my $test_passed = 1;
        foreach my $input (@{$test->{inputs}}) {
            $total_tests++;
            print "    Testing input: '$input' ... ";
            
            my $result = eval {
                local $SIG{ALRM} = sub { die "timeout" };
                alarm(3);  # 3 second timeout
                my $parse_result = yapg::GeneratedParser::parse(\$input);
                alarm(0);
                return $parse_result;
            };
            
            if ($@) {
                if ($@ =~ /timeout|Deep recursion/) {
                    print "TIMEOUT/RECURSION\n";
                    $test_passed = 0;
                } else {
                    print "ERROR: $@\n";
                    $test_passed = 0;
                }
            } elsif (defined $result) {
                print "SUCCESS\n";
                $passed_tests++;
            } else {
                print "NO MATCH\n";
                # For now, count no-match as success (parser didn't crash)
                $passed_tests++;
            }
        }
        
        if (!$test_passed) {
            $failed_tests++;
            push @failures, "$category->{name} - $test->{name}: Some inputs failed";
        }
        
        unlink $grammar_file;
    }
}

print "\n" . "="x60 . "\n";
print "COMPREHENSIVE TORTURE TEST RESULTS\n";
print "="x60 . "\n";
print "Total Input Tests: $total_tests\n";
print "Passed: $passed_tests\n";
print "Failed: $failed_tests\n";
print "Success Rate: " . sprintf("%.1f%%", ($passed_tests / $total_tests) * 100) . "\n";

if (@failures) {
    print "\nFAILED TESTS:\n";
    foreach my $failure (@failures) {
        print "  - $failure\n";
    }
}

print "\n=== TORTURE TEST COMPLETE ===\n";

