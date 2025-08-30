#!/usr/bin/perl
use strict;
use warnings;
use Data::Dumper;

print "=== DETAILED DEBUG: Simple Valid EBNF ===\n\n";

# Simple valid EBNF content
my $simple_ebnf = "word := 'hello'\n";

print "1. SIMPLE EBNF CONTENT:\n";
print Dumper(\$simple_ebnf);
print "\n";

# Save to file
open my $fh, '>', 'simple_test.ebnf' or die "Cannot create simple_test.ebnf: $!";
print $fh $simple_ebnf;
close $fh;

# Test LinkedSpec parsing
use lib 'fx/perl';
use LinkedSpec;

# Read ebnf.spec
my $spec_content;
{
    open my $fh, '<', "fx/specs/ebnf.spec" or die "Cannot read ebnf.spec: $!";
    local $/;
    $spec_content = <$fh>;
    close $fh;
}

print "2. LINKEDSPEC PARSING:\n";
my $parser = LinkedSpec::Get(\$spec_content);
print "Parser creation result: " . Dumper($parser) . "\n";

my $raw_ast = $parser->(\$simple_ebnf);
print "Raw AST from LinkedSpec: " . Dumper($raw_ast) . "\n";

print "3. AST ANALYSIS:\n";
print "- defined(\$raw_ast): " . (defined($raw_ast) ? "YES" : "NO") . "\n";
print "- \$raw_ast is truthy: " . ($raw_ast ? "YES" : "NO") . "\n";
print "- ref(\$raw_ast): " . (ref($raw_ast) || "NOT A REF") . "\n";

if (defined($raw_ast) && ref($raw_ast) eq 'ARRAY') {
    print "- Array length: " . scalar(@$raw_ast) . "\n";
    for my $i (0 .. $#$raw_ast) {
        print "- Rule $i: " . Dumper($raw_ast->[$i]) . "\n";
    }
}

# Cleanup
unlink 'simple_test.ebnf';


