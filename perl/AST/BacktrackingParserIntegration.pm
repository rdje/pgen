package AST::BacktrackingParserIntegration;

use strict;
use warnings;
use Data::Dumper;

=head1 NAME

AST::BacktrackingParserIntegration - Shared utilities for backtracking parser generation

=head1 DESCRIPTION

This module provides shared functionality for both the BacktrackingParserGenerator
and Transform modules to handle grouped quantifiers properly.

=cut

use Exporter 'import';
our @EXPORT_OK = qw(
    is_grouped_quantifier
    extract_grouped_elements
    parse_quantifier_bounds
    is_terminal
    is_literal
    is_regex
    is_rule_reference
    extract_rule_name
    extract_literal_value
    extract_regex_pattern
    detect_grouped_quantifier_in_element
);

=head2 is_grouped_quantifier($element)

Check if element is a grouped quantifier like (',' /\\s*/ expression)*.
Works with multiple AST formats.

=cut

sub is_grouped_quantifier {
    my ($element) = @_;
    
    return 0 unless defined $element;
    
    # Hash format with grouped_sequence type
    if (ref($element) eq 'HASH') {
        if (exists $element->{type} && $element->{type} eq 'grouped_sequence') {
            return 1;
        }
        
        # Check for nested grouped structure
        if (exists $element->{element}) {
            return is_grouped_quantifier($element->{element});
        }
        
        # Check for array of elements that form a group pattern
        if (exists $element->{elements} && ref($element->{elements}) eq 'ARRAY') {
            # Look for group patterns like [',', /\s*/, expr]
            my @elems = @{$element->{elements}};
            if (@elems >= 2) {
                # Check if this looks like a grouped sequence
                # Pattern: comma followed by optional whitespace followed by expression
                for my $elem (@elems) {
                    if (is_terminal($elem)) {
                        my $val = extract_literal_value($elem) || extract_regex_pattern($elem);
                        if ($val && ($val eq ',' || $val =~ /\\s/)) {
                            return 1;
                        }
                    }
                }
            }
        }
    }
    
    # Array format - check for grouped pattern
    if (ref($element) eq 'ARRAY') {
        # Look for patterns that suggest grouping
        for my $i (0..$#$element) {
            my $elem = $element->[$i];
            if (ref($elem) eq 'ARRAY' && @$elem >= 2) {
                if ($elem->[0] eq 'quoted_string' && $elem->[1] eq ',') {
                    return 1;  # Found comma pattern
                }
                if ($elem->[0] eq 'regex' && $elem->[1] =~ /\\s/) {
                    return 1;  # Found whitespace pattern
                }
            }
        }
    }
    
    return 0;
}

=head2 detect_grouped_quantifier_in_element($element)

Detect if an element contains a grouped quantifier and extract it.
This handles the case where the element might be nested.

=cut

sub detect_grouped_quantifier_in_element {
    my ($element) = @_;
    
    return undef unless defined $element;
    
    # Check if element itself is grouped quantifier
    if (is_grouped_quantifier($element)) {
        return {
            is_grouped => 1,
            group_element => $element,
            type => 'direct'
        };
    }
    
    # Check nested structures
    if (ref($element) eq 'HASH') {
        # Check for element.element pattern
        if (exists $element->{element}) {
            my $nested_result = detect_grouped_quantifier_in_element($element->{element});
            if ($nested_result) {
                return {
                    is_grouped => 1,
                    group_element => $nested_result->{group_element},
                    type => 'nested',
                    parent => $element
                };
            }
        }
        
        # Check for elements array
        if (exists $element->{elements}) {
            if (is_grouped_quantifier({elements => $element->{elements}})) {
                return {
                    is_grouped => 1,
                    group_element => {elements => $element->{elements}},
                    type => 'elements_array'
                };
            }
        }
    }
    
    return undef;
}

=head2 extract_grouped_elements($grouped_element)

Extract individual elements from a grouped quantifier.

=cut

sub extract_grouped_elements {
    my ($grouped_element) = @_;
    
    return () unless defined $grouped_element;
    
    # Hash format with elements
    if (ref($grouped_element) eq 'HASH') {
        if (exists $grouped_element->{elements}) {
            return @{$grouped_element->{elements}};
        }
        
        # Check nested element
        if (exists $grouped_element->{element} && ref($grouped_element->{element}) eq 'HASH') {
            return extract_grouped_elements($grouped_element->{element});
        }
    }
    
    # Array format - return as is
    if (ref($grouped_element) eq 'ARRAY') {
        return @$grouped_element;
    }
    
    return ();
}

=head2 parse_quantifier_bounds($quantifier)

Parse quantifier into min/max bounds.

=cut

sub parse_quantifier_bounds {
    my ($quantifier) = @_;
    
    return (0, 999999) unless defined $quantifier;
    
    if ($quantifier eq '*') {
        return (0, 999999);
    } elsif ($quantifier eq '+') {
        return (1, 999999);
    } elsif ($quantifier eq '?') {
        return (0, 1);
    } elsif ($quantifier =~ /^\{(\d+)\}$/) {
        return ($1, $1);
    } elsif ($quantifier =~ /^\{(\d+),\}$/) {
        return ($1, 999999);
    } elsif ($quantifier =~ /^\{(\d+),(\d+)\}$/) {
        return ($1, $2);
    } elsif ($quantifier =~ /^\{,(\d+)\}$/) {
        return (0, $1);
    }
    
    # Default fallback
    return (1, 1);
}

=head2 is_terminal($element)

Check if element is a terminal.

=cut

sub is_terminal {
    my ($element) = @_;
    
    return 0 unless defined $element;
    
    if (ref($element) eq 'ARRAY') {
        return $element->[0] eq 'quoted_string' || 
               $element->[0] eq 'regex' ||
               $element->[0] eq 'terminal';
    } elsif (ref($element) eq 'HASH') {
        return exists $element->{type} && 
               ($element->{type} eq 'terminal' || 
                $element->{type} eq 'literal' ||
                $element->{type} eq 'regex');
    }
    
    return 0;
}

=head2 is_literal($element)

Check if element is a literal terminal.

=cut

sub is_literal {
    my ($element) = @_;
    
    if (ref($element) eq 'ARRAY') {
        return $element->[0] eq 'quoted_string';
    } elsif (ref($element) eq 'HASH') {
        return exists $element->{type} && $element->{type} eq 'literal';
    }
    
    return 0;
}

=head2 is_regex($element)

Check if element is a regex pattern.

=cut

sub is_regex {
    my ($element) = @_;
    
    if (ref($element) eq 'ARRAY') {
        return $element->[0] eq 'regex';
    } elsif (ref($element) eq 'HASH') {
        return exists $element->{type} && $element->{type} eq 'regex';
    }
    
    return 0;
}

=head2 is_rule_reference($element)

Check if element is a rule reference.

=cut

sub is_rule_reference {
    my ($element) = @_;
    
    if (ref($element) eq 'ARRAY') {
        return $element->[0] eq 'rule_reference' || $element->[0] eq 'rule';
    } elsif (ref($element) eq 'HASH') {
        return exists $element->{type} && 
               ($element->{type} eq 'rule_reference' || $element->{type} eq 'rule');
    } elsif (!ref($element) && defined $element) {
        # Simple string that looks like a rule name
        return $element =~ /^[a-zA-Z_]\\w*$/;
    }
    
    return 0;
}

=head2 extract_rule_name($element)

Extract rule name from rule reference.

=cut

sub extract_rule_name {
    my ($element) = @_;
    
    if (ref($element) eq 'ARRAY') {
        return $element->[1];
    } elsif (ref($element) eq 'HASH') {
        return $element->{rule_name} || $element->{name} || $element->{value};
    } elsif (!ref($element)) {
        return $element;
    }
    
    return 'unknown_rule';
}

=head2 extract_literal_value($element)

Extract literal value from terminal element.

=cut

sub extract_literal_value {
    my ($element) = @_;
    
    if (ref($element) eq 'ARRAY') {
        return $element->[1];
    } elsif (ref($element) eq 'HASH') {
        return $element->{value};
    }
    
    return undef;
}

=head2 extract_regex_pattern($element)

Extract regex pattern from regex element.

=cut

sub extract_regex_pattern {
    my ($element) = @_;
    
    if (ref($element) eq 'ARRAY') {
        return $element->[1];
    } elsif (ref($element) eq 'HASH') {
        return $element->{pattern} || $element->{value};
    }
    
    return undef;
}

1;

__END__

=head1 EXAMPLE USAGE

    use AST::BacktrackingParserIntegration qw(is_grouped_quantifier extract_grouped_elements);
    
    # Check if element is grouped quantifier
    if (is_grouped_quantifier($element)) {
        my @group_elements = extract_grouped_elements($element);
        # Process grouped elements...
    }

=head1 PURPOSE

This module centralizes the logic for detecting and processing grouped quantifiers,
enabling both BacktrackingParserGenerator and Transform.pm to handle expressions like:

    array_contents := return_expression (',' /\\s*/ return_expression)*

Without this, multi-element arrays like `[$1, $2]` would fail with "SKIPPED: Unhandled quantified element type".

=cut
