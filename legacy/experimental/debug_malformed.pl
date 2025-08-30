#!/usr/bin/perl
use strict;
use warnings;
use Data::Dumper;

print "=== Debugging Malformed EBNF Test ===\n\n";

# Check what's in the malformed file
print "Contents of malformed.ebnf:\n";
print "-" x 40 . "\n";
if (open my $fh, '<', 'stability_test_results/malformed.ebnf') {
    while (my $line = <$fh>) {
        print "$.: $line";
    }
    close $fh;
} else {
    print "ERROR: Cannot read malformed.ebnf: $!\n";
}
print "-" x 40 . "\n\n";

# Run the parser generator and capture everything
print "Running parser generator on malformed file...\n";
my $output = `perl backtracking_parser_generator.pl stability_test_results/malformed.ebnf 2>&1`;
my $exit_code = $?;

print "Raw output:\n";
print Dumper(\$output);
print "\nExit code: $exit_code\n";

# Check if the exit code indicates failure
if ($exit_code == 0) {
    print "ISSUE: Exit code is 0 (success) but should be non-zero (failure)\n";
} else {
    print "GOOD: Exit code is non-zero ($exit_code), indicating failure\n";
}

# Check what the test condition is looking for
print "\nTest condition analysis:\n";
print "- Exit code != 0? " . ($exit_code != 0 ? "YES" : "NO") . "\n";
print "- Test expects failure, got: " . ($exit_code == 0 ? "success" : "failure") . "\n";


