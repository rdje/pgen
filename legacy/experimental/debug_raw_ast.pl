#!/usr/bin/perl
use strict;
use warnings;
use Data::Dumper;

print "=== Debugging Raw AST from Parser Generator ===\n\n";

# Let's reproduce exactly what backtracking_parser_generator.pl does
my $ebnf_file = "stability_test_results/malformed.ebnf";

# Read the spec content
my $spec_content;
{
    open my $fh, '<', "fx/specs/ebnf.spec" or die "Cannot read ebnf.spec: $!";
    local $/;
    $spec_content = <$fh>;
    close $fh;
}

# Read the input content
my $input_content;
{
    open my $fh, '<', $ebnf_file or die "Cannot read $ebnf_file: $!";
    local $/;
    $input_content = <$fh>;
    close $fh;
}

print "Input content:\n";
print Dumper(\$input_content);

# Now try the parsing
use lib 'fx/perl';
use LinkedSpec;

my $parser = LinkedSpec::Get(\$spec_content);
print "\nParser creation result:\n";
print Dumper($parser);

my $raw_ast = $parser->(\$input_content);
print "\nRaw AST result:\n";
print Dumper($raw_ast);

print "\nAST analysis:\n";
print "- defined(\$raw_ast): " . (defined($raw_ast) ? "YES" : "NO") . "\n";
print "- \$raw_ast is truthy: " . ($raw_ast ? "YES" : "NO") . "\n";
print "- ref(\$raw_ast): " . (ref($raw_ast) || "NOT A REF") . "\n";

if (ref($raw_ast) eq 'ARRAY') {
    print "- Array length: " . scalar(@$raw_ast) . "\n";
}


