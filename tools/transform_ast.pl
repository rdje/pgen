#!/usr/bin/perl
use strict;
use warnings;
use lib 'perl';
use FindBin qw($RealBin);
use File::Spec;
use Getopt::Long;

# AST Transformation Tool - JSON Interface Bridge
#
# This demonstrates the cross-language interface:
# Reads raw AST JSON (from Perl ebnf_to_json.pl) 
# Outputs transformed AST JSON (for language-specific generators)
#
# In production, each target language (Rust, Julia, Go, etc.) would
# implement their own pipeline that reads raw AST JSON directly.
#
# Usage:
#   perl tools/transform_ast.pl input_raw.json output_transformed.json

use AST::Transform qw(process_to_final_ast);
use JSON::PP;

sub usage {
    print <<"EOF";
AST Transformation Tool - JSON Interface Bridge

Demonstrates the cross-language JSON interface for AST transformation.

Usage: $0 input_raw.json [output_transformed.json]

This tool shows how language-specific implementations should:
1. Read raw AST JSON (produced by Perl ebnf_to_json.pl)  
2. Transform AST using their own pipeline implementation
3. Generate code/data in-memory (no intermediate JSON needed)

For production use, each target language implements this directly.
EOF
}

sub main {
    my ($input_file, $output_file) = @ARGV;
    
    unless ($input_file && -f $input_file) {
        print STDERR "Error: Input raw AST JSON file required\n\n";
        usage();
        return 1;
    }
    
    $output_file ||= $input_file =~ s/\.json$/_transformed.json/r;
    
    print STDERR "=== Cross-Language AST Transformation Demo ===\n";
    print STDERR "Input:  $input_file (Raw AST JSON)\n";  
    print STDERR "Output: $output_file (Transformed AST JSON)\n\n";
    
    # Step 1: Read raw AST JSON (cross-language interface)
    print STDERR "Step 1: Reading raw AST JSON...\n";
    
    open my $fh, '<', $input_file or die "Cannot read $input_file: $!";
    local $/;
    my $json_text = <$fh>;
    close $fh;
    
    my $raw_data = JSON::PP->new->decode($json_text);
    
    unless ($raw_data->{raw_ast}) {
        die "Error: Input JSON missing 'raw_ast' field from ebnf_to_json.pl\n";
    }
    
    my $grammar_name = $raw_data->{grammar_name} || "unknown";
    my $raw_ast = $raw_data->{raw_ast};
    my $metadata = $raw_data->{metadata} || {};
    
    print STDERR "Loaded: '$grammar_name' with " . scalar(@$raw_ast) . " rules\n";
    
    # Step 2: Transform AST (language-specific implementation)  
    print STDERR "\nStep 2: Transforming AST (using Perl implementation)...\n";
    my ($grammar_tree, $rule_order) = process_to_final_ast($raw_ast);
    
    print STDERR "Transformed: " . scalar(keys %$grammar_tree) . " rules\n";
    print STDERR "Rule order: " . join(", ", @$rule_order) . "\n";
    
    # Step 3: Output transformed JSON (for generators)
    print STDERR "\nStep 3: Writing transformed AST JSON...\n";
    
    my $transformed_data = {
        grammar_name => $grammar_name,
        grammar_tree => $grammar_tree, 
        rule_order   => $rule_order,
        metadata     => {
            %$metadata,
            format         => "transformed_ast",
            transformed_at => scalar(localtime()),
            transformer    => "Perl AST::Transform (demo)",
            source_format  => "raw_ast"
        }
    };
    
    my $json_out = JSON::PP->new->pretty->canonical->encode($transformed_data);
    
    open my $out_fh, '>', $output_file or die "Cannot write $output_file: $!";
    print $out_fh $json_out;
    close $out_fh;
    
    print STDERR "Saved: $output_file\n";
    print STDERR "\n=== Ready for Language-Specific Generators ===\n";
    print STDERR "Next: Feed transformed JSON to Rust/Julia/Go/etc. generators\n";
    
    return 0;
}

exit main() if __FILE__ eq $0;
