#!/usr/bin/perl
use strict;
use warnings;
use Data::Dumper;

print "=== RAW EBNF PARSER OUTPUT (Before Any Validation) ===\n\n";

# Simple valid EBNF
my $simple_ebnf = "word := 'hello'\n";
print "Input EBNF content:\n";
print Dumper(\$simple_ebnf);

# Just use LinkedSpec directly - NO validation logic
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

print "\n=== LINKEDSPEC PARSING ONLY ===\n";

# Create parser from ebnf.spec
my $parser = LinkedSpec::Get(\$spec_content);
print "Parser creation: " . (defined($parser) ? "SUCCESS" : "FAILED") . "\n";

if (defined($parser)) {
    # Apply parser to simple EBNF content  
    my $raw_ast = $parser->(\$simple_ebnf);
    
    print "\nRaw AST from LinkedSpec (BEFORE any validation):\n";
    print Dumper($raw_ast);
    
    print "\nAST Analysis:\n";
    print "- defined: " . (defined($raw_ast) ? "YES" : "NO") . "\n";
    print "- truthy: " . ($raw_ast ? "YES" : "NO") . "\n"; 
    print "- type: " . (ref($raw_ast) || "SCALAR") . "\n";
    
    if (defined($raw_ast) && ref($raw_ast) eq 'ARRAY') {
        print "- length: " . scalar(@$raw_ast) . "\n";
        for my $i (0 .. $#$raw_ast) {
            print "- rule[$i]: " . Dumper($raw_ast->[$i]);
        }
    }
} else {
    print "ERROR: Parser creation failed!\n";
}

print "\n=== END RAW OUTPUT ===\n";


