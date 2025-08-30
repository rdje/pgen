#!/usr/bin/perl
use strict;
use warnings;
use FindBin qw($RealBin);
use lib "$RealBin/../perl";

use AST::Transform qw(generate_parser_from_file get_error_context process_transformation_phases load_ebnf_spec_from_content step2_group_by_or step2_5_handle_parentheses step3_parse_sequences step4_handle_quantifiers step5_build_tree_structure);
use AST::RustCodeGen qw(generate_rust_parser_module);
use AST::JuliaCodeGen qw(generate_julia_parser_module);
use LeftRecursionEliminator qw(eliminate_all_left_recursion); # Add missing import
use Getopt::Long;
use Pod::Usage;

# 🎯 CLEAN CLI INTERFACE FOR AST TRANSFORMATION

my %options = (
    quiet => 0,
    verbosity => 'normal',
    help => 0,
    validate_only => 0,
    output_file => '',
    package_name => '',
    rust => 0,
    julia => 0,
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
        
        AST::Transform::validate_grammar($content);
        print STDERR "✅ Grammar validation passed!\n" unless $options{quiet};
        
    } else {
        # Generate the parser
        if ($options{rust}) {
            print STDERR "🦀 Generating Rust parser from: $ebnf_file\n" unless $options{quiet};
        } elsif ($options{julia}) {
            print STDERR "🔷 Generating Julia parser from: $ebnf_file\n" unless $options{quiet};
        } else {
            print STDERR "🚀 Generating Perl parser from: $ebnf_file\n" unless $options{quiet};
        }
        
        my $parser_result;
        if ($options{rust}) {
            # Generate Rust parser
            $parser_result = generate_rust_parser_from_file($ebnf_file, %options);
        } elsif ($options{julia}) {
            # Generate Julia parser
            $parser_result = generate_julia_parser_from_file($ebnf_file, %options);
        } else {
            # Generate Perl parser
            $parser_result = generate_parser_from_file($ebnf_file, %options);
        }
        
        if ($options{output_file}) {
            if ($options{rust}) {
                # For Rust output, write single .rs file
                my $rs_file = $options{output_file};
                $rs_file .= '.rs' unless $rs_file =~ /\.rs$/;
                
                open my $rs_fh, '>', $rs_file or die "Cannot write to $rs_file: $!";
                print $rs_fh $parser_result;
                close $rs_fh;
                print STDERR "✅ Rust parser written to: $rs_file\n" unless $options{quiet};
            } elsif ($options{julia}) {
                # For Julia output, write single .jl file
                my $jl_file = $options{output_file};
                $jl_file .= '.jl' unless $jl_file =~ /\.jl$/;
                
                open my $jl_fh, '>', $jl_file or die "Cannot write to $jl_file: $!";
                print $jl_fh $parser_result;
                close $jl_fh;
                print STDERR "✅ Julia parser written to: $jl_file\n" unless $options{quiet};
            } else {
                # Split the base filename to create .pm and .pl files
                my $base_name = $options{output_file};
                $base_name =~ s/\.(pl|pm)$//; # Remove extension if present
            
            # Determine package name: CLI option > output filename > EBNF filename
            my $package_name = $options{package_name};
            if (!$package_name) {
                if ($base_name) {
                    $package_name = $base_name;
                    $package_name =~ s/.*\///; # Remove directory path
                } else {
                    # Derive from EBNF filename
                    $package_name = $ebnf_file;
                    $package_name =~ s/.*\///; # Remove directory path
                    $package_name =~ s/\.ebnf$//; # Remove .ebnf extension
                }
                # Clean up package name
                $package_name =~ s/[^a-zA-Z0-9_]/_/g; # Replace non-alphanumeric with underscore
                $package_name = ucfirst($package_name); # Capitalize first letter
            }
            
            my $pm_file = "${base_name}.pm";
            my $pl_file = "${base_name}.pl";
            
            # Update module and wrapper with correct package name
            my $module_content = $parser_result->{module};
            my $wrapper_content = $parser_result->{wrapper};
            
            $module_content =~ s/package PACKAGE_NAME_PLACEHOLDER;/package $package_name;/;
            $wrapper_content =~ s/use PACKAGE_NAME_PLACEHOLDER;/use $package_name;/;
            $wrapper_content =~ s/PACKAGE_NAME_PLACEHOLDER::parse/${package_name}::parse/g;
            
            # Write the .pm module file
            open my $pm_fh, '>', $pm_file or die "Cannot write to $pm_file: $!";
            print $pm_fh $module_content;
            close $pm_fh;
            print STDERR "✅ Parser module written to: $pm_file\n" unless $options{quiet};
            
            # Write the .pl wrapper script
            open my $pl_fh, '>', $pl_file or die "Cannot write to $pl_file: $!";
            print $pl_fh $wrapper_content;
            close $pl_fh;
                chmod 0755, $pl_file; # Make executable
                print STDERR "✅ Parser script written to: $pl_file\n" unless $options{quiet};
            }
        } else {
            # If no output file specified, print to stdout
            if ($options{rust}) {
                # For Rust, just print the generated code directly
                print $parser_result;
            } elsif ($options{julia}) {
                # For Julia, just print the generated code directly
                print $parser_result;
            } else {
                # For Perl, need to determine package name and update module content
                my $package_name = $options{package_name};
                if (!$package_name) {
                    # Derive from EBNF filename
                    $package_name = $ebnf_file;
                    $package_name =~ s/.*\///; # Remove directory path
                    $package_name =~ s/\.ebnf$//; # Remove .ebnf extension
                    # Clean up package name
                    $package_name =~ s/[^a-zA-Z0-9_]/_/g; # Replace non-alphanumeric with underscore
                    $package_name = ucfirst($package_name); # Capitalize first letter
                }
                
                # Update module with correct package name
                my $module_content = $parser_result->{module};
                $module_content =~ s/package PACKAGE_NAME_PLACEHOLDER;/package $package_name;/;
                
                print $module_content;
            }
        }
    }
};

if ($@) {
    print STDERR "\n🚨 PARSER GENERATION FAILED!\n";
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

print STDERR "🎉 Success!\n" unless $options{quiet};

# Function to generate Rust parser from file
sub generate_rust_parser_from_file {
    my ($ebnf_file, %options) = @_;
    
    # Set global options for the modules
    $AST::Transform::quiet_mode = $options{quiet};
    $AST::Transform::verbosity = $options{verbosity};
    $AST::RustCodeGen::quiet_mode = $options{quiet};
    $AST::RustCodeGen::verbosity = $options{verbosity};
    
    # Load and parse the EBNF grammar
    my $content = do {
        open my $fh, '<', $ebnf_file or die "Cannot open $ebnf_file: $!";
        local $/;
        <$fh>;
    };
    
    # Parse the EBNF content to get raw AST first
    my $raw_ast = AST::Transform::load_ebnf_spec_from_content($content);
    
    # Process transformation phases up to step 5 to get grammar tree and rule order
    print STDERR "\n=== Processing EBNF for Rust generation ===\n" unless $options{quiet};
    
    my $step2_result = AST::Transform::step2_group_by_or($raw_ast);
    my $step2_5_result = AST::Transform::step2_5_handle_parentheses($step2_result);
    my $step3_result = AST::Transform::step3_parse_sequences($step2_5_result);
    my $step4_result = AST::Transform::step4_handle_quantifiers($step3_result);
    my ($grammar_tree, $rule_order) = AST::Transform::step5_build_tree_structure($step4_result);
    
    # TODO: Fix grammar tree format conversion - currently corrupts rule references
    # For now, use grammar tree directly (LR elimination validated but bypassed)
    # Convert grammar tree format for left-recursion elimination
    # my $lr_compatible_grammar = convert_grammar_tree_to_lr_format($grammar_tree);
    # Apply left-recursion elimination  
    # my $eliminated_grammar = eliminate_all_left_recursion($lr_compatible_grammar);
    # Convert back to original format for Rust generation
    # my $transformed_grammar = convert_lr_format_to_grammar_tree($eliminated_grammar);
    my $transformed_grammar = $grammar_tree; # Use original until conversion fixed
    my $final_rule_order = $rule_order; # Keep original rule order
    
    # Generate Rust code
    my $rust_code = generate_rust_parser_module($transformed_grammar, $final_rule_order);
    
    return $rust_code;
}

# Function to generate Julia parser from file
sub generate_julia_parser_from_file {
    my ($ebnf_file, %options) = @_;
    
    # Set global options for the modules
    $AST::Transform::quiet_mode = $options{quiet};
    $AST::Transform::verbosity = $options{verbosity};
    $AST::JuliaCodeGen::quiet_mode = $options{quiet};
    $AST::JuliaCodeGen::verbosity = $options{verbosity};
    
    # Load and parse the EBNF grammar
    my $content = do {
        open my $fh, '<', $ebnf_file or die "Cannot open $ebnf_file: $!";
        local $/;
        <$fh>;
    };
    
    # Parse the EBNF content to get raw AST first
    my $raw_ast = AST::Transform::load_ebnf_spec_from_content($content);
    
    # Process transformation phases up to step 5 to get grammar tree and rule order
    print STDERR "\n=== Processing EBNF for Julia generation ===\n" unless $options{quiet};
    
    my $step2_result = AST::Transform::step2_group_by_or($raw_ast);
    my $step2_5_result = AST::Transform::step2_5_handle_parentheses($step2_result);
    my $step3_result = AST::Transform::step3_parse_sequences($step2_5_result);
    my $step4_result = AST::Transform::step4_handle_quantifiers($step3_result);
    my ($grammar_tree, $rule_order) = AST::Transform::step5_build_tree_structure($step4_result);
    
    # Use same approach as Rust generation - bypass LR elimination for now
    my $transformed_grammar = $grammar_tree; # Use original until conversion fixed
    my $final_rule_order = $rule_order; # Keep original rule order
    
    # Generate Julia code
    my $julia_code = generate_julia_parser_module($transformed_grammar, $final_rule_order);
    
    return $julia_code;
}

# Convert grammar tree from step5 format to LeftRecursionEliminator format
sub convert_grammar_tree_to_lr_format {
    my ($grammar_tree) = @_;
    my %lr_grammar = ();
    
    foreach my $rule_name (keys %$grammar_tree) {
        my $rule = $grammar_tree->{$rule_name};
        my @productions = extract_productions_from_rule($rule);
        $lr_grammar{$rule_name} = \@productions;
    }
    
    return \%lr_grammar;
}

# Extract productions from a rule (recursive)
sub extract_productions_from_rule {
    my ($rule) = @_;
    my @productions = ();
    
    if ($rule->{type} eq 'or') {
        # OR rule: each alternative becomes a separate production
        foreach my $alternative (@{$rule->{alternatives}}) {
            my @alt_productions = extract_productions_from_rule($alternative);
            push @productions, @alt_productions;
        }
    } elsif ($rule->{type} eq 'sequence') {
        # Sequence rule: convert elements to symbol array
        my @symbols = ();
        foreach my $element (@{$rule->{elements}}) {
            my @element_symbols = extract_symbols_from_element($element);
            push @symbols, @element_symbols;
        }
        push @productions, \@symbols;
    } elsif ($rule->{type} eq 'atom') {
        # Atom rule: single symbol production
        my @symbols = (extract_symbol_from_atom($rule->{value}));
        push @productions, \@symbols;
    } else {
        # Fallback: treat as epsilon production
        push @productions, ['ε'];
    }
    
    return @productions;
}

# Extract symbols from an element (handles quantified, grouped, etc.)
# Always returns an array of symbols
sub extract_symbols_from_element {
    my ($element) = @_;
    
    if (ref($element) eq 'HASH') {
        if ($element->{type} eq 'atom') {
            return (extract_symbol_from_atom($element->{value}));
        } elsif ($element->{type} eq 'quantified') {
            # For now, treat quantified elements as optional (simplified)
            # TODO: More sophisticated handling of quantifiers in LR elimination
            return extract_symbols_from_element($element->{element});
        } elsif ($element->{type} eq 'sequence') {
            # Nested sequence
            my @symbols = ();
            foreach my $sub_element (@{$element->{elements}}) {
                my @sub_symbols = extract_symbols_from_element($sub_element);
                push @symbols, @sub_symbols;
            }
            return @symbols;
        } elsif ($element->{type} eq 'or') {
            # This is complex - for now, just take first alternative
            # TODO: More sophisticated handling of nested OR in LR elimination
            if (@{$element->{alternatives}} > 0) {
                return extract_symbols_from_element($element->{alternatives}->[0]);
            }
            return ('ε');
        }
    } elsif (ref($element) eq 'ARRAY' && $element->[0] eq 'GROUPED') {
        # Grouped element - process contents
        my @symbols = ();
        foreach my $sub_element (@{$element->[1]}) {
            my @sub_symbols = extract_symbols_from_element($sub_element);
            push @symbols, @sub_symbols;
        }
        return @symbols;
    } else {
        # Direct element
        return (extract_symbol_from_atom($element));
    }
}

# Extract symbol name from atomic value
sub extract_symbol_from_atom {
    my ($atom) = @_;
    
    if (ref($atom) eq 'ARRAY') {
        if ($atom->[0] eq 'rule') {
            return $atom->[1];  # Rule reference
        } elsif ($atom->[0] eq 'quoted_string') {
            return $atom->[1];  # Terminal string
        } elsif ($atom->[0] eq 'regex') {
            return "REGEX:$atom->[1]";  # Regex pattern
        } elsif ($atom->[0] eq 'terminal') {
            return $atom->[1];  # Terminal symbol
        } else {
            return $atom->[1] // $atom->[0];  # Fallback
        }
    } else {
        return $atom;  # Plain string
    }
}

# Convert back from LeftRecursionEliminator format to grammar tree format
sub convert_lr_format_to_grammar_tree {
    my ($lr_grammar) = @_;
    my %grammar_tree = ();
    
    foreach my $rule_name (keys %$lr_grammar) {
        my $productions = $lr_grammar->{$rule_name};
        my $rule_node = convert_productions_to_rule($productions);
        $grammar_tree{$rule_name} = $rule_node;
    }
    
    return \%grammar_tree;
}

# Convert array of productions back to rule structure
sub convert_productions_to_rule {
    my ($productions) = @_;
    
    if (@$productions == 1) {
        # Single production
        my $production = $productions->[0];
        if (@$production == 1) {
            # Single symbol - atom
            return {
                type => 'atom',
                value => convert_symbol_to_atom($production->[0])
            };
        } else {
            # Multiple symbols - sequence
            my @elements = map { 
                { type => 'atom', value => convert_symbol_to_atom($_) } 
            } @$production;
            return {
                type => 'sequence',
                elements => \@elements
            };
        }
    } else {
        # Multiple productions - OR
        my @alternatives = ();
        foreach my $production (@$productions) {
            if (@$production == 1) {
                # Single symbol alternative
                push @alternatives, {
                    type => 'atom',
                    value => convert_symbol_to_atom($production->[0])
                };
            } else {
                # Multi-symbol alternative
                my @elements = map { 
                    { type => 'atom', value => convert_symbol_to_atom($_) } 
                } @$production;
                push @alternatives, {
                    type => 'sequence',
                    elements => \@elements
                };
            }
        }
        return {
            type => 'or',
            alternatives => \@alternatives
        };
    }
}

# Convert symbol string back to atom format
sub convert_symbol_to_atom {
    my ($symbol) = @_;
    
    # Handle special case where symbol might be undefined or empty
    return ['epsilon'] unless defined $symbol && $symbol ne '';
    
    if ($symbol eq 'ε') {
        return ['epsilon'];
    } elsif ($symbol =~ /^REGEX:(.+)$/) {
        return ['regex', $1];
    } elsif ($symbol =~ /^".*"$/ || $symbol =~ /^'.*'$/) {
        return ['quoted_string', $symbol];
    } else {
        # Assume it's a rule reference - make sure it's a simple string
        return ['rule', "$symbol"];
    }
}

__END__

=head1 NAME

ast_transform.pl - EBNF Grammar to Parser Generator

=head1 SYNOPSIS

    ast_transform.pl [options] <grammar.ebnf>

=head1 OPTIONS

=over 4

=item B<--quiet, -q>

Suppress progress messages

=item B<--verbosity, -v> level

Set verbosity level: normal, full, debug

=item B<--validate-only>

Only validate grammar, don't generate parser

=item B<--output, -o> file

Write output to file instead of STDOUT

=item B<--package, -p> name

Package name for the generated parser module. If not specified, derives from the output filename or EBNF filename (without .ebnf extension). (Perl only)

=item B<--rust, -r>

Generate Rust parser instead of Perl parser

=item B<--julia, -j>

Generate Julia parser instead of Perl parser

=item B<--help, -h>

Show this help message

=back

=head1 EXAMPLES

    # Generate parser to STDOUT  
    ast_transform.pl grammar.ebnf > parser.pm

    # Generate .pm/.pl files with custom package name
    ast_transform.pl grammar.ebnf -o my_parser --package MyParser

    # Generate with package name derived from EBNF filename
    ast_transform.pl my_grammar.ebnf -o parser  # Creates package My_grammar

    # Generate with verbose output
    ast_transform.pl -v full grammar.ebnf -o parser.pl
    
    # Generate Rust parser
    ast_transform.pl --rust grammar.ebnf -o parser
    
    # Generate Julia parser
    ast_transform.pl --julia grammar.ebnf -o parser

    # Just validate grammar
    ast_transform.pl --validate-only grammar.ebnf

=head1 DESCRIPTION

This tool transforms EBNF grammar definitions into executable parsers (Perl, Rust, or Julia)
with comprehensive error reporting and validation.

=cut
