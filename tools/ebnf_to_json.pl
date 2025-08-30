#!/usr/bin/perl
use strict;
use warnings;
use FindBin qw($RealBin);
use lib "$RealBin/../perl";

use AST::Transform qw(load_ebnf_spec_from_content get_error_context);
use JSON::PP;
use Getopt::Long;
use Pod::Usage;
use POSIX qw(strftime);

# 🎯 EBNF TO JSON CONVERTER - Raw AST Generator
# Stops immediately after EBNF parsing, before any transformations

my %options = (
    output_file => '',
    pretty => 0,
    validate_only => 0,
    quiet => 0,
    verbosity => 'normal',
    help => 0,
);

GetOptions(
    'output|o=s'     => \$options{output_file},
    'pretty'         => \$options{pretty},
    'validate-only'  => \$options{validate_only},
    'quiet|q'        => \$options{quiet},
    'verbosity|v=s'  => \$options{verbosity},
    'help|h'         => \$options{help},
) or pod2usage(2);

pod2usage(1) if $options{help};
pod2usage("Error: No EBNF file specified") unless @ARGV;

my $ebnf_file = $ARGV[0];

# 🛡️ ENHANCED ERROR HANDLING
eval {
    if ($options{validate_only}) {
        # Just validate the grammar
        print STDERR "🔍 Validating grammar: $ebnf_file\n" unless $options{quiet};
        
        open my $fh, '<', $ebnf_file or die "Cannot open $ebnf_file: $!";
        my $content = do { local $/; <$fh> };
        close $fh;
        
        # Parse to validate
        load_ebnf_spec_from_content($content);
        print STDERR "✅ Grammar validation passed!\n" unless $options{quiet};
        
    } else {
        # Generate JSON raw AST
        print STDERR "📄 Converting EBNF to JSON Raw AST: $ebnf_file\n" unless $options{quiet};
        
        my $json_ast = ebnf_to_raw_json($ebnf_file, %options);
        
        if ($options{output_file}) {
            # Write to file
            open my $out_fh, '>', $options{output_file} or die "Cannot write to $options{output_file}: $!";
            print $out_fh $json_ast;
            close $out_fh;
            print STDERR "✅ JSON Raw AST written to: $options{output_file}\n" unless $options{quiet};
        } else {
            # Write to STDOUT
            print $json_ast;
        }
    }
};

if ($@) {
    print STDERR "\n🚨 EBNF TO JSON CONVERSION FAILED!\n";
    print STDERR $@;
    
    # Show error summary
    my $context = get_error_context();
    if (@{$context->{errors}}) {
        print STDERR "\n📊 ERROR SUMMARY:\n";
        print STDERR "  Total Errors: " . scalar(@{$context->{errors}}) . "\n";
        print STDERR "  Total Warnings: " . scalar(@{$context->{warnings}}) . "\n";
        
        if ($options{verbosity} eq 'full') {
            print STDERR "\n🔍 DETAILED ERROR LOG:\n";
            foreach my $error (@{$context->{errors}}) {
                print STDERR "  • $error->{type}: $error->{message}\n";
            }
        }
    }
    
    exit 1;
}

print STDERR "🎉 Raw AST JSON generated successfully!\n" unless $options{quiet};

# 🔧 CORE CONVERSION FUNCTION
sub ebnf_to_raw_json {
    my ($ebnf_file, %options) = @_;
    
    # Set global options for the modules
    $AST::Transform::quiet_mode = $options{quiet};
    $AST::Transform::verbosity = $options{verbosity};
    
    # Load and parse the EBNF grammar
    my $content = do {
        open my $fh, '<', $ebnf_file or die "Cannot open $ebnf_file: $!";
        local $/;
        <$fh>;
    };
    
    # Parse the EBNF content to get raw AST - STOP HERE!
    # This is the direct output from the EBNF parser before any transformations
    my $raw_ast = load_ebnf_spec_from_content($content);
    
    print STDERR "🎯 Raw AST extracted from EBNF parser (before transformations)\n" unless $options{quiet};
    
    # Convert raw AST directly to JSON format
    my $json_data = package_raw_ast_as_json($raw_ast, $ebnf_file);
    
    # Serialize to JSON
    my $json = JSON::PP->new;
    if ($options{pretty}) {
        $json->pretty->canonical;
    } else {
        $json->canonical;
    }
    
    return $json->encode($json_data);
}

# 🔄 PACKAGING FUNCTION
sub package_raw_ast_as_json {
    my ($raw_ast, $ebnf_file) = @_;
    
    # Extract grammar name from filename
    my $grammar_name = $ebnf_file;
    $grammar_name =~ s/.*\///;      # Remove directory path
    $grammar_name =~ s/\.ebnf$//;   # Remove .ebnf extension
    
    # Package raw AST with metadata
    my $json_data = {
        grammar_name => $grammar_name,
        raw_ast => $raw_ast,
        metadata => {
            source_file => $ebnf_file,
            generated_at => strftime("%Y-%m-%dT%H:%M:%SZ", gmtime()),
            generator => "ebnf_to_json.pl",
            ebnf_version => "1.0",
            format => "raw_ast",
            description => "Direct output from EBNF parser (ebnf.spec via LinkedSpec.pm) before any AST transformations",
            next_step => "Apply transformation pipeline: step2_group_by_or → step2_5_handle_parentheses → step3_parse_sequences → step4_handle_quantifiers → step5_build_tree_structure",
            documentation => "See docs/ast_transformation_pipeline.md for detailed transformation algorithms"
        }
    };
    
    return $json_data;
}

__END__

=head1 NAME

ebnf_to_json.pl - EBNF Grammar to JSON Raw AST Converter

=head1 SYNOPSIS

    ebnf_to_json.pl [options] <grammar.ebnf>

=head1 OPTIONS

=over 4

=item B<--output, -o> file

Write JSON to file instead of STDOUT

=item B<--pretty>

Pretty-print JSON output with indentation

=item B<--validate-only>

Only validate grammar, don't generate JSON

=item B<--quiet, -q>

Suppress progress messages

=item B<--verbosity, -v> level

Set verbosity level: normal, full, debug

=item B<--help, -h>

Show this help message

=back

=head1 EXAMPLES

    # Generate JSON raw AST to STDOUT
    ebnf_to_json.pl json.ebnf

    # Generate pretty-printed JSON to file
    ebnf_to_json.pl --pretty json.ebnf -o json_raw_ast.json
    
    # Validate grammar only
    ebnf_to_json.pl --validate-only json.ebnf
    
    # Quiet mode with output file
    ebnf_to_json.pl -q json.ebnf -o json.json

=head1 DESCRIPTION

This tool parses EBNF grammar files using the core EBNF parser and outputs 
the RAW AST as JSON. This is the universal interchange format for the new 
JSON-based parser generation architecture.

IMPORTANT: This tool stops IMMEDIATELY after EBNF parsing, before any AST
transformations. The raw AST preserves the exact token-level structure 
returned by the EBNF parser for maximum flexibility.

Language-specific generators are responsible for implementing their own
transformation pipelines optimized for their target language.

=head1 ARCHITECTURE DECISION

This implements Option A of the JSON-based architecture:

    EBNF → Raw AST → JSON → Language Transformations → Target Parser

Benefits:
- Maximum flexibility for language-specific optimizations
- No dependency on Perl transformation logic
- Language generators can innovate on transformation approaches
- True language independence

See docs/parser_architecture_evolution.md for full rationale.

=head1 JSON FORMAT

The output JSON contains the raw AST exactly as returned by the EBNF parser:

    {
        "grammar_name": "json",
        "raw_ast": [
            // Each rule as array: [rule_name_token, ...rule_tokens...]
            [["rule", "json"], ["quoted_string", "value"], ["operator", "|"], ...]
        ],
        "metadata": {
            "source_file": "json.ebnf",
            "format": "raw_ast",
            "description": "Direct EBNF parser output before transformations"
        }
    }

=head1 TRANSFORMATION PIPELINE

Language generators must implement the 5-step transformation pipeline:

    1. Raw AST (this tool's output)
    2. Group by OR operators  
    3. Handle parentheses grouping
    4. Parse sequences
    5. Handle quantifiers
    6. Build semantic tree structure

See docs/ast_transformation_pipeline.md for detailed algorithms.

=cut
