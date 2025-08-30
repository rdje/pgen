#!/usr/bin/perl
use strict;
use warnings;
use FindBin qw($RealBin);
use Getopt::Long;
use Pod::Usage;
use File::Temp qw(tempfile);
use IPC::Open3;

# 🎯 UNIVERSAL PARSER GENERATOR - Backward Compatible Wrapper
# Sequences ebnf_to_json.pl -> language_parser_gen for all languages

my %options = (
    quiet => 0,
    verbosity => 'normal',
    help => 0,
    validate_only => 0,
    output_file => '',
    package_name => '',
    rust => 0,
    julia => 0,
    perl => 0,
    # Future languages
    python => 0,
    go => 0,
    cpp => 0,
);

GetOptions(
    'quiet|q'        => \$options{quiet},
    'verbosity|v=s'  => \$options{verbosity},
    'help|h'         => \$options{help},
    'validate-only'  => \$options{validate_only},
    'output|o=s'     => \$options{output_file},
    'package|p=s'    => \$options{package_name},
    'rust|r'         => \$options{rust},
    'julia|j'        => \$options{julia},
    'perl'           => \$options{perl},
    'python'         => \$options{python},
    'go'             => \$options{go},
    'cpp'            => \$options{cpp},
) or pod2usage(2);

pod2usage(1) if $options{help};
pod2usage("Error: No EBNF file specified") unless @ARGV;

my $ebnf_file = $ARGV[0];

# Determine target language (default to Perl for backward compatibility)
my $target_language = 'perl';  # Default
$target_language = 'rust' if $options{rust};
$target_language = 'julia' if $options{julia};
$target_language = 'python' if $options{python};
$target_language = 'go' if $options{go};
$target_language = 'cpp' if $options{cpp};

# 🛡️ ENHANCED ERROR HANDLING
eval {
    if ($options{validate_only}) {
        # Just validate the grammar using ebnf_to_json.pl
        print STDERR "🔍 Validating grammar: $ebnf_file\n" unless $options{quiet};
        
        my $cmd = "$RealBin/ebnf_to_json.pl --validate-only";
        $cmd .= " --quiet" if $options{quiet};
        $cmd .= " --verbosity $options{verbosity}" if $options{verbosity} ne 'normal';
        $cmd .= " '$ebnf_file'";
        
        system($cmd);
        exit($? >> 8) if $?;
        
        print STDERR "✅ Grammar validation passed!\n" unless $options{quiet};
        
    } else {
        # Generate the parser using the new pipeline
        generate_parser_via_pipeline($ebnf_file, $target_language, %options);
    }
};

if ($@) {
    print STDERR "\n🚨 PARSER GENERATION FAILED!\n";
    print STDERR $@;
    exit 1;
}

print STDERR "🎉 Success!\n" unless $options{quiet};

# 🔧 CORE PIPELINE FUNCTION
sub generate_parser_via_pipeline {
    my ($ebnf_file, $target_language, %options) = @_;
    
    # Step 1: Generate JSON raw AST using ebnf_to_json.pl
    print STDERR "Step 1: Converting EBNF to JSON...\n" unless $options{quiet};
    
    my $json_cmd = "$RealBin/ebnf_to_json.pl";
    $json_cmd .= " --quiet" if $options{quiet};
    $json_cmd .= " --verbosity $options{verbosity}" if $options{verbosity} ne 'normal';
    $json_cmd .= " '$ebnf_file'";
    
    my $json_output = `$json_cmd`;
    my $json_exit_code = $? >> 8;
    
    if ($json_exit_code != 0) {
        die "Failed to convert EBNF to JSON (exit code: $json_exit_code)\n";
    }
    
    # Step 2: Generate parser using language-specific generator
    my $language_emoji = get_language_emoji($target_language);
    print STDERR "Step 2: ${language_emoji} Generating $target_language parser...\n" unless $options{quiet};
    
    my $gen_cmd = get_generator_command($target_language, %options);
    
    # Use IPC::Open3 for better control
    my ($gen_in, $gen_out, $gen_err);
    my $gen_pid = open3($gen_in, $gen_out, $gen_err, $gen_cmd);
    
    # Send JSON to generator
    print $gen_in $json_output;
    close $gen_in;
    
    # Read generator output
    my $gen_result = do { local $/; <$gen_out> };
    my $gen_error = do { local $/; <$gen_err> };
    
    waitpid($gen_pid, 0);
    my $gen_exit_code = $? >> 8;
    
    if ($gen_exit_code != 0) {
        print STDERR $gen_error if $gen_error;
        die "Failed to generate $target_language parser (exit code: $gen_exit_code)\n";
    }
    
    # Print generator errors/progress (but not to STDERR if quiet)
    print STDERR $gen_error if $gen_error && !$options{quiet};
    
    # Handle output based on target language
    if ($options{output_file}) {
        # Language generators handle their own file output
        # The result is already written to files by the generators
        print STDERR "✅ Parser written to output files\n" unless $options{quiet};
    } else {
        # Print result to STDOUT
        print $gen_result;
    }
}

# 🎨 LANGUAGE CONFIGURATION
sub get_language_emoji {
    my ($language) = @_;
    
    my %emojis = (
        'perl'   => '🐪',
        'rust'   => '🦀', 
        'julia'  => '🔷',
        'python' => '🐍',
        'go'     => '🐹',
        'cpp'    => '⚙️',
    );
    
    return $emojis{$language} || '🔧';
}

sub get_generator_command {
    my ($language, %options) = @_;
    
    my $cmd;
    
    if ($language eq 'perl') {
        $cmd = "$RealBin/perl_parser_gen.pl";
        $cmd .= " --output '$options{output_file}'" if $options{output_file};
        $cmd .= " --package '$options{package_name}'" if $options{package_name};
        
    } elsif ($language eq 'rust') {
        $cmd = "$RealBin/rust_parser_gen";  # Future implementation
        $cmd .= " --output '$options{output_file}'" if $options{output_file};
        
    } elsif ($language eq 'julia') {
        $cmd = "$RealBin/julia_parser_gen";  # Future implementation
        $cmd .= " --output '$options{output_file}'" if $options{output_file};
        
    } elsif ($language eq 'python') {
        $cmd = "$RealBin/python_parser_gen";  # Future implementation
        $cmd .= " --output '$options{output_file}'" if $options{output_file};
        
    } elsif ($language eq 'go') {
        $cmd = "$RealBin/go_parser_gen";  # Future implementation
        $cmd .= " --output '$options{output_file}'" if $options{output_file};
        
    } elsif ($language eq 'cpp') {
        $cmd = "$RealBin/cpp_parser_gen";  # Future implementation
        $cmd .= " --output '$options{output_file}'" if $options{output_file};
        
    } else {
        die "Unsupported language: $language\n";
    }
    
    # Add common options
    $cmd .= " --quiet" if $options{quiet};
    $cmd .= " --verbosity $options{verbosity}" if $options{verbosity} ne 'normal';
    
    return $cmd;
}

__END__

=head1 NAME

generate_parser.pl - Universal Parser Generator with JSON-based Architecture

=head1 SYNOPSIS

    generate_parser.pl [options] grammar.ebnf

=head1 OPTIONS

=over 4

=item B<--rust, -r>

Generate Rust parser (.rs file)

=item B<--julia, -j>

Generate Julia parser (.jl file)  

=item B<--perl>

Generate Perl parser (.pm/.pl files) [DEFAULT]

=item B<--python>

Generate Python parser (.py file) [FUTURE]

=item B<--go>

Generate Go parser (.go file) [FUTURE]

=item B<--cpp>

Generate C++ parser (.cpp/.hpp files) [FUTURE]

=item B<--output, -o> basename

Output file basename (extensions added automatically)

=item B<--package, -p> name  

Package/module name (Perl only)

=item B<--validate-only>

Only validate grammar, don't generate parser

=item B<--quiet, -q>

Suppress progress messages

=item B<--verbosity, -v> level

Set verbosity level: normal, full, debug

=item B<--help, -h>

Show this help message

=back

=head1 EXAMPLES

    # Generate Perl parser (default)  
    generate_parser.pl json.ebnf -o json_parser
    
    # Generate Rust parser
    generate_parser.pl --rust json.ebnf -o json_parser
    
    # Generate Julia parser  
    generate_parser.pl --julia json.ebnf -o json_parser
    
    # Validate only
    generate_parser.pl --validate-only json.ebnf
    
    # Quiet mode
    generate_parser.pl -q --rust json.ebnf -o json_parser

=head1 DESCRIPTION

This is the universal wrapper for the new JSON-based parser generation 
architecture. It provides backward compatibility with the original
ast_transform.pl interface while using the new pipeline:

    EBNF → [ebnf_to_json.pl] → Raw AST JSON → [language_parser_gen] → Target Parser

=head2 Two-Stage Pipeline

B<Stage 1: EBNF to JSON>
- Parses EBNF grammar using ebnf_to_json.pl
- Produces universal raw AST JSON interchange format
- Language-agnostic, minimal Perl dependency

B<Stage 2: JSON to Target Parser>  
- Uses language-specific generator (perl_parser_gen.pl, rust_parser_gen, etc.)
- Implements 5-step transformation pipeline natively
- Generates optimized parser in target language

=head2 Language Support

=over 4

=item B<Perl> ✅ READY

Full support via perl_parser_gen.pl
- Generates .pm module + .pl wrapper
- Optimized parsing functions  
- Backward compatible with existing workflow

=item B<Rust> 🔄 IN DEVELOPMENT

Via rust_parser_gen (future implementation)
- Zero-copy string processing
- Memory-safe AST handling
- Compile-time validation

=item B<Julia> 🔄 IN DEVELOPMENT  

Via julia_parser_gen (future implementation)
- Multiple dispatch transformations
- LLVM-optimized parsing
- Interactive development support

=item B<Others> 🔮 PLANNED

Python, Go, C++ generators planned for future releases

=back

=head1 ARCHITECTURE BENEFITS

=over 4

=item B<Language Independence>

No Perl runtime dependency for target parsers

=item B<Optimization Freedom>

Each language can optimize for its strengths

=item B<Parallel Development>

Language generators can be developed independently  

=item B<Innovation Friendly>

New transformation approaches can be explored

=item B<Backward Compatibility>

Existing workflows continue to work unchanged

=back

=head1 MIGRATION FROM ast_transform.pl

Old command:
    perl ast_transform.pl --rust json.ebnf -o json_parser.rs

New equivalent:  
    generate_parser.pl --rust json.ebnf -o json_parser

The interface is nearly identical - just replace ast_transform.pl with 
generate_parser.pl.

=head1 FILES

This wrapper uses the following tools:

=over 4

=item F<ebnf_to_json.pl>

Converts EBNF to JSON raw AST

=item F<perl_parser_gen.pl>  

Generates Perl parsers from JSON

=item F<rust_parser_gen>

Generates Rust parsers from JSON [FUTURE]

=item F<julia_parser_gen>

Generates Julia parsers from JSON [FUTURE]

=back

=head1 SEE ALSO

L<ebnf_to_json.pl>, L<perl_parser_gen.pl>, 
F<docs/parser_architecture_evolution.md>,
F<docs/ast_transformation_pipeline.md>

=cut
