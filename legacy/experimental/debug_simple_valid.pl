#!/usr/bin/perl
use strict;
use warnings;
use Data::Dumper;

print "=== Testing Simple Valid EBNF ===\n\n";

# Create a simple valid EBNF file
open my $fh, '>', 'test_simple_debug.ebnf' or die "Cannot create test file: $!";
print $fh "word := 'hello'\n";
close $fh;

print "Created test file with content: word := 'hello'\n\n";

# Test with the updated ebnf.spec
use lib 'fx/perl';
use LinkedSpec;

# Read spec content
my $spec_content;
{
    open my $fh, '<', "fx/specs/ebnf.spec" or die "Cannot read ebnf.spec: $!";
    local $/;
    $spec_content = <$fh>;
    close $fh;
}

# Read input content  
my $input_content;
{
    open my $fh, '<', 'test_simple_debug.ebnf' or die "Cannot read test file: $!";
    local $/;
    $input_content = <$fh>;
    close $fh;
}

print "Input content: " . Dumper(\$input_content);

# Try parsing
my $parser = LinkedSpec::Get(\$spec_content);
print "Parser creation: " . Dumper($parser);

my $raw_ast = $parser->(\$input_content);
print "Raw AST: " . Dumper($raw_ast);

# Cleanup
unlink 'test_simple_debug.ebnf';


