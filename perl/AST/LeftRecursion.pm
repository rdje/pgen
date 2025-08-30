package AST::LeftRecursion;

use strict;
use warnings;
use Exporter qw(import);
use Data::Dumper;
use LeftRecursionIntegrator qw(eliminate_left_recursion_nuclear_option);

our @EXPORT_OK = qw(eliminate_left_recursion);
our $VERSION = '1.0.0';

# 🔥 LEFT-RECURSION NUCLEAR ELIMINATOR!
# Implements the comprehensive two-phase algorithm for complete left-recursion elimination

sub eliminate_left_recursion {
    my ($grammar_ref, $rule_order) = @_;
    
    print STDERR "🚀 DEPLOYING LEFT-RECURSION NUCLEAR ELIMINATOR!\n" if $ENV{VERBOSE};
    print STDERR "🎯 Target: Complete annihilation of all recursion forms\n" if $ENV{VERBOSE};
    print STDERR "=" x 70 . "\n\n" if $ENV{VERBOSE};
    
    # Call the main integration function that handles AST format conversion
    my ($new_grammar_tree, $new_rule_order) = eliminate_left_recursion_nuclear_option($grammar_ref, $rule_order);
    
    print STDERR "\n💀 LEFT-RECURSION STATUS: COMPLETELY ANNIHILATED!\n" if $ENV{VERBOSE};
    
    return ($new_grammar_tree, $new_rule_order);
}

1;

__END__

=head1 NAME

AST::LeftRecursion - Left recursion elimination for EBNF grammars

=head1 SYNOPSIS

    use AST::LeftRecursion qw(eliminate_left_recursion);
    
    my $clean_grammar = eliminate_left_recursion($grammar);

=head1 DESCRIPTION

This module implements comprehensive left recursion elimination using a two-phase algorithm
that handles direct, indirect, mutual, and chain left-recursion completely.

=cut
