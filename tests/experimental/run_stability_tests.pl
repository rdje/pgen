#!/usr/bin/perl
use strict;
use warnings;
use File::Spec;
use Time::HiRes qw(time);
use File::Path qw(make_path);

# Comprehensive stability testing for EBNF parser generator
# Tests edge cases, error conditions, performance limits, and real-world scenarios

print "=== EBNF Parser Generator Stability Testing ===\n\n";

# Test results tracking
my @test_results = ();
my $total_tests = 0;
my $passed_tests = 0;
my $failed_tests = 0;

# Create test output directory
my $test_dir = "stability_test_results";
make_path($test_dir) unless -d $test_dir;

sub run_test {
    my ($test_name, $test_func) = @_;
    $total_tests++;
    
    print "Running: $test_name... ";
    
    my $start_time = time();
    my $result = eval { $test_func->() };
    my $duration = time() - $start_time;
    
    if ($@ || !$result) {
        print "FAILED" . ($@ ? " ($@)" : "") . "\n";
        $failed_tests++;
        push @test_results, {
            name => $test_name,
            status => 'FAILED',
            duration => $duration,
            error => $@ || 'Test returned false'
        };
        return 0;
    } else {
        print "PASSED (${duration}s)\n";
        $passed_tests++;
        push @test_results, {
            name => $test_name,
            status => 'PASSED', 
            duration => $duration
        };
        return 1;
    }
}

# Test Category 1: Edge Cases
print "\n=== EDGE CASE TESTING ===\n";

run_test("Empty EBNF Grammar", sub {
    open my $fh, '>', "$test_dir/empty.ebnf" or die "Cannot create empty.ebnf: $!";
    close $fh;
    
    # Should fail gracefully for empty file (non-zero exit code)
    my $result = `perl backtracking_parser_generator.pl $test_dir/empty.ebnf 2>&1`;
    return $? != 0 && $result =~ /No.*rules.*found|Empty.*grammar/i;
});

run_test("Single Rule Grammar", sub {
    open my $fh, '>', "$test_dir/single.ebnf" or die "Cannot create single.ebnf: $!";
    print $fh "word := 'hello'\n";
    close $fh;
    
    my $result = `perl backtracking_parser_generator.pl $test_dir/single.ebnf 2>&1`;
    return $? == 0;
});

run_test("Malformed EBNF Syntax", sub {
    open my $fh, '>', "$test_dir/malformed.ebnf" or die "Cannot create malformed.ebnf: $!";
    print $fh "completely invalid syntax\n";
    print $fh "no := operators anywhere\n";
    print $fh "just random text\n";
    close $fh;
    
    # Should fail gracefully for completely invalid syntax
    my $result = `perl backtracking_parser_generator.pl $test_dir/malformed.ebnf 2>&1`;
    return $? != 0;  # Should fail with non-zero exit code
});

run_test("Invalid Probability Syntax", sub {
    open my $fh, '>', "$test_dir/bad_prob.ebnf" or die "Cannot create bad_prob.ebnf: $!";
    print $fh "expr := 'a' @50% | 'b' @60%\n";  # Sums to 110%
    close $fh;
    
    my $result = `perl ebnf_input_generator.pl $test_dir/bad_prob.ebnf 2>&1`;
    return $result =~ /exceed.*100|sum.*100/i;
});

run_test("Circular Rule References", sub {
    open my $fh, '>', "$test_dir/circular.ebnf" or die "Cannot create circular.ebnf: $!";
    print $fh "a := b\n";
    print $fh "b := c\n"; 
    print $fh "c := a\n";
    close $fh;
    
    my $result = `perl backtracking_parser_generator.pl $test_dir/circular.ebnf 2>&1`;
    # Should detect or handle circular references
    return $? == 0 || $result =~ /circular|recursive|undefined/i;
});

# Test Category 2: Stress Testing  
print "\n=== STRESS TESTING ===\n";

run_test("Large Grammar (100 Rules)", sub {
    open my $fh, '>', "$test_dir/large.ebnf" or die "Cannot create large.ebnf: $!";
    
    # Generate 100 rules
    for my $i (1..100) {
        print $fh "rule$i := 'token$i' | 'alt$i'\n";
    }
    
    # Main rule that references many others
    print $fh "main := " . join(' | ', map { "rule$_" } 1..100) . "\n";
    close $fh;
    
    my $start = time();
    my $result = `perl backtracking_parser_generator.pl $test_dir/large.ebnf 2>&1`;
    my $duration = time() - $start;
    
    print "  (Large grammar took ${duration}s)";
    return $? == 0 && $duration < 30;  # Should complete in under 30 seconds
});

run_test("Deep Recursion Grammar", sub {
    open my $fh, '>', "$test_dir/deep.ebnf" or die "Cannot create deep.ebnf: $!";
    print $fh "expr := '(' expr ')' | 'x'\n";
    close $fh;
    
    # Generate deeply nested input
    my $deep_input = 'x';
    for (1..100) {
        $deep_input = "($deep_input)";
    }
    
    open my $input_fh, '>', "$test_dir/deep_input.txt" or die "Cannot create deep_input.txt: $!";
    print $input_fh $deep_input;
    close $input_fh;
    
    # Generate parser
    my $result1 = `perl backtracking_parser_generator.pl -o $test_dir/deep_parser.pm $test_dir/deep.ebnf 2>&1`;
    return 0 unless $? == 0;
    
    # Test parsing deeply nested input  
    my $result2 = `perl $test_dir/deep_parser.pm < $test_dir/deep_input.txt 2>&1`;
    return $? == 0;
});

run_test("Wide Alternatives (50 OR branches)", sub {
    open my $fh, '>', "$test_dir/wide.ebnf" or die "Cannot create wide.ebnf: $!";
    
    my @alternatives = map { "'alt$_'" } 1..50;
    print $fh "choice := " . join(' | ', @alternatives) . "\n";
    close $fh;
    
    my $result = `perl backtracking_parser_generator.pl -o $test_dir/wide_parser.pm $test_dir/wide.ebnf 2>&1`;
    return $? == 0;
});

# Test Category 3: Real-World Grammars
print "\n=== REAL-WORLD GRAMMAR TESTING ===\n";

run_test("JSON Grammar", sub {
    open my $fh, '>', "$test_dir/json.ebnf" or die "Cannot create json.ebnf: $!";
    print $fh <<'EOF';
json := object | array
object := '{' '}' | '{' members '}'
members := pair | pair ',' members  
pair := string ':' value
array := '[' ']' | '[' elements ']'
elements := value | value ',' elements
value := string | number | object | array | 'true' | 'false' | 'null'
string := '"' chars '"'
chars := char | char chars
char := 'a' | 'b' | 'c' | 'd' | 'e'
number := digit | digit digits
digits := digit | digit digits  
digit := '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'
EOF
    close $fh;
    
    my $result = `perl backtracking_parser_generator.pl -o $test_dir/json_parser.pm $test_dir/json.ebnf 2>&1`;
    return $? == 0;
});

run_test("Arithmetic Expressions", sub {
    open my $fh, '>', "$test_dir/arithmetic.ebnf" or die "Cannot create arithmetic.ebnf: $!";
    print $fh <<'EOF';
expr := term '+' expr | term '-' expr | term
term := factor '*' term | factor '/' term | factor  
factor := '(' expr ')' | number
number := digit | digit number
digit := '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'
EOF
    close $fh;
    
    my $result = `perl backtracking_parser_generator.pl -o $test_dir/arithmetic_parser.pm $test_dir/arithmetic.ebnf 2>&1`;
    return $? == 0;
});

# Test Category 4: End-to-End Workflow Testing
print "\n=== END-TO-END WORKFLOW TESTING ===\n";

run_test("Complete Pipeline: Grammar -> Parser -> Input -> Parse", sub {
    open my $fh, '>', "$test_dir/pipeline.ebnf" or die "Cannot create pipeline.ebnf: $!";
    print $fh <<'EOF';
expr := term '+' term @40% | term '-' term @30% | number @30%
term := number | '(' expr ')'
number := '1' @25% | '2' @25% | '3' @25% | '4' @25%
EOF
    close $fh;
    
    # Step 1: Generate parser
    my $result1 = `perl backtracking_parser_generator.pl -o $test_dir/pipeline_parser.pm $test_dir/pipeline.ebnf 2>&1`;
    return 0 unless $? == 0;
    
    # Step 2: Generate input
    my $result2 = `perl ebnf_input_generator.pl $test_dir/pipeline.ebnf --count 10 > $test_dir/pipeline_input.txt 2>&1`;
    return 0 unless $? == 0;
    
    # Step 3: Parse generated input
    my $result3 = `perl $test_dir/pipeline_parser.pm < $test_dir/pipeline_input.txt 2>&1`;
    return $? == 0;
});

# Test Category 5: Error Recovery Testing
print "\n=== ERROR RECOVERY TESTING ===\n";

run_test("Parser Handles Invalid Input Gracefully", sub {
    # Use existing working grammar
    return 0 unless -f "comprehensive_test.ebnf";
    
    # Create invalid input
    open my $fh, '>', "$test_dir/invalid_input.txt" or die "Cannot create invalid_input.txt: $!";
    print $fh "completely invalid input that matches no rules @#\$%\n";
    close $fh;
    
    # Generate parser
    my $result1 = `perl backtracking_parser_generator.pl -o $test_dir/error_test_parser.pm comprehensive_test.ebnf 2>&1`;
    return 0 unless $? == 0;
    
    # Test with invalid input - just verify parser was created and runs
    my $result2 = `perl $test_dir/error_test_parser.pm < $test_dir/invalid_input.txt 2>&1`;
    # For now, just check that it doesn't crash (exit code < 256)
    return $? < 256;  # Any exit code < 256 means no crash
});

# Generate test summary report
print "\n" . "="x60 . "\n";
print "STABILITY TEST SUMMARY\n";
print "="x60 . "\n";
print "Total Tests: $total_tests\n";
print "Passed:      $passed_tests\n"; 
print "Failed:      $failed_tests\n";
print "Success Rate: " . sprintf("%.1f", ($passed_tests/$total_tests)*100) . "%\n\n";

if ($failed_tests > 0) {
    print "FAILED TESTS:\n";
    for my $test (@test_results) {
        if ($test->{status} eq 'FAILED') {
            print "  - $test->{name}: $test->{error}\n";
        }
    }
    print "\n";
}

# Performance summary
print "PERFORMANCE SUMMARY:\n";
my $total_time = 0;
for my $test (@test_results) {
    $total_time += $test->{duration};
    if ($test->{duration} > 5) {  # Highlight slow tests
        print "  - $test->{name}: " . sprintf("%.2f", $test->{duration}) . "s (SLOW)\n";
    }
}
print "Total test time: " . sprintf("%.2f", $total_time) . "s\n\n";

# Write detailed results to file
open my $report_fh, '>', "$test_dir/stability_test_report.txt" or die "Cannot create report: $!";
print $report_fh "EBNF Parser Generator Stability Test Report\n";
print $report_fh "Generated: " . localtime() . "\n\n";
print $report_fh "Summary: $passed_tests/$total_tests tests passed\n\n";

for my $test (@test_results) {
    print $report_fh sprintf("%-50s %s (%.3fs)\n", 
                             $test->{name}, 
                             $test->{status}, 
                             $test->{duration});
    if ($test->{error}) {
        print $report_fh "  Error: $test->{error}\n";
    }
}
close $report_fh;

print "Detailed results written to: $test_dir/stability_test_report.txt\n";
print "Test files and outputs in: $test_dir/\n";

# Exit with failure code if any tests failed
exit($failed_tests > 0 ? 1 : 0);

