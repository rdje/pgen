#!/usr/bin/perl
use strict;
use warnings;
use FindBin qw($RealBin);
use lib "$RealBin/perl";
use Data::Dumper;

use AST::Transform qw(generate_parser_from_file get_error_context);

# Generate parser with debug info
print "=== DEBUGGING index_list rule processing ===\n";

my $parser_result = generate_parser_from_file('grammars/core/ultimate_dot_notation.ebnf', 
    quiet => 0, 
    verbosity => 'debug');

# Check the generated code for parse_index_list function
my $module_code = $parser_result->{module};

# Extract just the parse_index_list function
if ($module_code =~ /(sub parse_index_list \{.*?^\})/ms) {
    my $index_list_function = $1;
    print "\n=== GENERATED parse_index_list function ===\n";
    print "$index_list_function\n";
} else {
    print "\n❌ Could not find parse_index_list function in generated code!\n";
}

# Also extract parse_index function for comparison  
if ($module_code =~ /(sub parse_index \{.*?^\})/ms) {
    my $index_function = $1;
    print "\n=== GENERATED parse_index function ===\n";
    print "$index_function\n";
} else {
    print "\n❌ Could not find parse_index function in generated code!\n";
}
