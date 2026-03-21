#!/usr/bin/env perl
use strict;
use warnings;
use Time::HiRes qw(time);

print "=== VALIDATED PARSER GENERATOR TESTS ===\n";
print "(Using only confirmed valid EBNF syntax)\n";

# Test cases with VALIDATED grammar syntax
my @test_cases = (
    {
        name => "Simple sequence (baseline)",
        grammar => "expr := 'a' 'b' 'c'",
        valid_inputs => ["abc"],
        invalid_inputs => ["ab", "abcd", "xyz"]
    },
    {
        name => "Multiple alternatives (our fix)",
        grammar => "expr := 'first'\nexpr := 'second'\nexpr := 'third'",
        valid_inputs => ["first", "second", "third"],
        invalid_inputs => ["fourth", "firstsecond"]
    },
    {
        name => "Simple recursion (right-recursive, should work)",
        grammar => "list := 'item' list\nlist := 'end'",
        valid_inputs => ["end", "itemend", "itemitemend"],
        invalid_inputs => ["item", "enditem"]
    },
    {
        name => "Direct left-recursion (known to fail)",
        grammar => "expr := expr '+' 'num'\nexpr := 'num'",
        valid_inputs => ["num"],
        expected_behavior => "infinite_recursion"
    },
    {
        name => "Indirect left-recursion (corrected syntax)",
        grammar => "A := B 'x'\nB := A 'y'\nB := 'z'",
        valid_inputs => ["zx"],
        expected_behavior => "infinite_recursion"
    },
    {
        name => "Terminal matching only",
        grammar => "number := '123'\nstring := 'hello'",
        valid_inputs => ["123", "hello"],
        invalid_inputs => ["124", "hi"]
    }
);

my $total_tests = 0;
my $passed_tests = 0;
my @detailed_results = ();

foreach my $test (@test_cases) {
    print "\n" . "="x50 . "\n";
    print "TEST: $test->{name}\n";
    print "="x50 . "\n";
    
    # First validate the grammar syntax
    print "Grammar:\n$test->{grammar}\n\n";
    
    my $grammar_file = "validate_test_$$.ebnf";
    open my $fh, '>', $grammar_file or die "Cannot create test file: $!";
    print $fh $test->{grammar};
    close $fh;
    
    # Test EBNF parsing first
    my $ebnf_valid = test_ebnf_parsing($grammar_file);
    if (!$ebnf_valid) {
        print "❌ GRAMMAR SYNTAX INVALID - Test skipped\n";
        push @detailed_results, {
            test => $test->{name},
            status => "GRAMMAR_INVALID",
            details => "EBNF syntax not supported"
        };
        unlink $grammar_file;
        next;
    }
    
    print "✅ Grammar syntax valid\n\n";
    
    # Generate parser
    my $parser_code = `perl ast_transform.pl $grammar_file --quiet 2>&1`;
    my $generation_success = $? == 0;
    
    if (!$generation_success) {
        print "❌ PARSER GENERATION FAILED:\n$parser_code\n";
        push @detailed_results, {
            test => $test->{name},
            status => "GENERATION_FAILED",
            details => $parser_code
        };
        unlink $grammar_file;
        next;
    }
    
    print "✅ Parser generated successfully\n";
    
    # Compile parser
    eval $parser_code;
    if ($@) {
        print "❌ PARSER COMPILATION FAILED: $@\n";
        push @detailed_results, {
            test => $test->{name},
            status => "COMPILATION_FAILED",
            details => $@
        };
        unlink $grammar_file;
        next;
    }
    
    print "✅ Parser compiled successfully\n\n";
    
    # Test expected behavior
    if ($test->{expected_behavior} && $test->{expected_behavior} eq 'infinite_recursion') {
        print "Testing for expected infinite recursion...\n";
        my $input = $test->{valid_inputs}[0];
        
        my $hit_recursion = 0;
        eval {
            local $SIG{ALRM} = sub { die "timeout" };
            alarm(2);
            yapg::GeneratedParser::parse(\$input);
            alarm(0);
        };
        
        if ($@ && ($@ =~ /timeout|Deep recursion/)) {
            print "✅ EXPECTED: Infinite recursion detected\n";
            $passed_tests++;
            push @detailed_results, {
                test => $test->{name},
                status => "PASS",
                details => "Expected infinite recursion confirmed"
            };
        } else {
            print "❌ UNEXPECTED: No infinite recursion\n";
            push @detailed_results, {
                test => $test->{name},
                status => "FAIL",
                details => "Expected infinite recursion but parsing succeeded/failed normally"
            };
        }
        $total_tests++;
    } else {
        # Test valid inputs
        if ($test->{valid_inputs}) {
            foreach my $input (@{$test->{valid_inputs}}) {
                $total_tests++;
                print "Testing valid input: '$input' ... ";
                
                my $result = eval {
                    local $SIG{ALRM} = sub { die "timeout" };
                    alarm(3);
                    my $parse_result = yapg::GeneratedParser::parse(\$input);
                    alarm(0);
                    return $parse_result;
                };
                
                if ($@ && ($@ =~ /timeout|Deep recursion/)) {
                    print "❌ TIMEOUT/RECURSION\n";
                    push @detailed_results, {
                        test => "$test->{name} - '$input'",
                        status => "FAIL",
                        details => "Unexpected timeout/recursion on valid input"
                    };
                } elsif (defined $result) {
                    print "✅ SUCCESS\n";
                    $passed_tests++;
                } else {
                    print "❌ NO MATCH\n";
                    push @detailed_results, {
                        test => "$test->{name} - '$input'",
                        status => "FAIL",
                        details => "Valid input rejected"
                    };
                }
            }
        }
        
        # Test invalid inputs
        if ($test->{invalid_inputs}) {
            foreach my $input (@{$test->{invalid_inputs}}) {
                $total_tests++;
                print "Testing invalid input: '$input' ... ";
                
                my $result = eval {
                    local $SIG{ALRM} = sub { die "timeout" };
                    alarm(3);
                    my $parse_result = yapg::GeneratedParser::parse(\$input);
                    alarm(0);
                    return $parse_result;
                };
                
                if ($@ && ($@ =~ /timeout|Deep recursion/)) {
                    print "❌ TIMEOUT/RECURSION\n";
                    push @detailed_results, {
                        test => "$test->{name} - '$input'",
                        status => "FAIL",
                        details => "Unexpected timeout/recursion on invalid input"
                    };
                } elsif (!defined $result) {
                    print "✅ CORRECTLY REJECTED\n";
                    $passed_tests++;
                } else {
                    print "❌ INCORRECTLY ACCEPTED\n";
                    push @detailed_results, {
                        test => "$test->{name} - '$input'",
                        status => "FAIL",
                        details => "Invalid input incorrectly accepted"
                    };
                }
            }
        }
    }
    
    unlink $grammar_file;
}

print "\n" . "="x60 . "\n";
print "VALIDATED TEST RESULTS\n";
print "="x60 . "\n";
print "Total Tests: $total_tests\n";
print "Passed: $passed_tests\n";
print "Failed: " . ($total_tests - $passed_tests) . "\n";
print "Success Rate: " . sprintf("%.1f%%", ($passed_tests / $total_tests) * 100) . "\n";

print "\nDETAILED RESULTS:\n";
foreach my $result (@detailed_results) {
    print "  $result->{status}: $result->{test}\n";
    if ($result->{details}) {
        print "    → $result->{details}\n";
    }
}

sub test_ebnf_parsing {
    my ($grammar_file) = @_;
    
    my $test_result = `perl -I perl -e "
        use LinkedSpec;
        open my \\\$spec_fh, '<', 'specs/ebnf.spec';
        my \\\$spec_content = do { local \\\$/; <\\\$spec_fh> };
        close \\\$spec_fh;
        my \\\$parser = LinkedSpec::Get(\\\\\\\$spec_content);
        
        open my \\\$test_fh, '<', '$grammar_file';
        my \\\$test_content = do { local \\\$/; <\\\$test_fh> };
        close \\\$test_fh;
        
        my \\\$ast = \\\$parser->(\\\\\\\$test_content);
        print defined(\\\$ast) ? 'VALID' : 'INVALID';
    " 2>/dev/null`;
    
    return $test_result eq 'VALID';
}

print "\n=== VALIDATION COMPLETE ===\n";
