#!/usr/bin/perl
use strict;
use warnings;
use Data::Dumper;

print "=== VERBOSE DEBUG: EBNF Context Validation ===\n\n";

# Simple test case
my $test_input = "word := 'hello'\n";
print "Testing with: " . Dumper(\$test_input);

# Read the current ebnf.spec to see the validation logic
print "Current ebnf.spec validation logic:\n";
print "=" x 50 . "\n";

open my $fh, '<', 'fx/specs/ebnf.spec' or die "Cannot read ebnf.spec: $!";
my $line_num = 1;
while (my $line = <$fh>) {
    printf "%3d: %s", $line_num++, $line;
    if ($line =~ /\$on/ || $line =~ /if.*\(/ || $line =~ /say/) {
        print "      ^^^ VALIDATION LOGIC\n";
    }
}
close $fh;

print "=" x 50 . "\n\n";

print "Key questions to investigate:\n";
print "1. Is \$on being properly initialized?\n";
print "2. Are the if(\$on) conditions blocking?\n";  
print "3. Is there a recursive call loop?\n";
print "4. Are the 'say' statements causing issues?\n";
print "5. Is the variable scope correct?\n\n";

# Let's examine the specific validation pattern
print "Pattern analysis:\n";
print "- \$on declared in grammar_file:: scope\n";
print "- \$on set to 1 when grammar_rule found\n";
print "- Each token handler checks if(\$on)\n";
print "- If not \$on, prints error and returns undef\n\n";

print "This could hang if:\n";
print "- \$on is undefined/uninitialized in token handlers\n";
print "- if(\$on) condition evaluates unexpectedly\n";
print "- 'say' statements block or loop\n";
print "- return undef causes parsing retry loop\n";


