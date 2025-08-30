#!/usr/bin/perl
use strict;
use warnings;

print "FIRST PRINT TEST\n";
$| = 1;  # Force flush output buffer
print "SECOND PRINT TEST\n";

use Data::Dumper;
print "AFTER Data::Dumper LOAD\n";

print "=== STEP-BY-STEP DEBUG WITH LOGGING ===\n";

print "Step 1: Testing basic Perl functionality...\n";
print "Perl is working: " . (2 + 2) . "\n";
# 
print "Step 2: Loading LinkedSpec module...\n";
use lib 'fx/perl';
eval {
    require LinkedSpec;
    print "LinkedSpec module loaded successfully\n";
} or do {
    print "ERROR loading LinkedSpec: $@\n";
    exit 1;
};
# 
print "Step 3: Reading ebnf.spec file...\n";
my $spec_content;
eval {
    open my $fh, '<', "fx/specs/ebnf.spec" or die "Cannot read ebnf.spec: $!";
    local $/;
    $spec_content = <$fh>;
    close $fh;
    print "ebnf.spec read successfully (" . length($spec_content) . " bytes)\n";
} or do {
    print "ERROR reading ebnf.spec: $@\n";
    exit 1;
};
# 
print "Step 4: Creating parser from ebnf.spec...\n";
print "About to call LinkedSpec::Get with spec_content length: " . length($spec_content) . "\n";
print "First 200 chars of spec_content:\n" . substr($spec_content, 0, 200) . "\n";

my $parser;
eval {
    print "Calling LinkedSpec::Get...\n";
    $parser = LinkedSpec::Get(\$spec_content);
    print "LinkedSpec::Get returned\n";
    print "Parser result: " . Dumper($parser);
    print "Parser creation: " . (defined($parser) ? "SUCCESS" : "FAILED") . "\n";
} or do {
    print "ERROR creating parser: $@\n";
    exit 1;
};
# 
print "Step 5: Testing simple input parsing...\n";
my $simple_input = "word := 'hello'\n";
print "Input to parse: " . Dumper(\$simple_input);
print "Input length: " . length($simple_input) . "\n";
print "Input bytes: " . join(" ", map { ord($_) } split //, $simple_input) . "\n";

my $result;
eval {
    print "About to call parser with input...\n";
    print "Parser reference: " . ref($parser) . "\n";
    print "Calling parser function...\n";
    $result = $parser->(\$simple_input);
    print "Parser function returned!\n";
    print "Result type: " . (ref($result) || "SCALAR") . "\n";
    print "Result defined: " . (defined($result) ? "YES" : "NO") . "\n";
} or do {
    print "ERROR during parsing: $@\n";
    print "Error details: " . Dumper(\$@);
    exit 1;
};
# 
print "Step 6: Final Results\n";
print "Raw result: " . Dumper($result);
if (defined($result) && ref($result) eq 'ARRAY') {
    print "Result is array with " . scalar(@$result) . " elements\n";
    for my $i (0 .. $#$result) {
        print "Element $i: " . Dumper($result->[$i]);
    }
}
print "=== DEBUG COMPLETE ===\n";
