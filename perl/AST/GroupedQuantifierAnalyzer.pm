package AST::GroupedQuantifierAnalyzer;

use strict;
use warnings;
use Data::Dumper;

=head1 NAME

AST::GroupedQuantifierAnalyzer - Analyze and extract grouped quantifier patterns

=head1 DESCRIPTION

This module analyzes AST structures to identify and extract grouped quantifier
patterns like (',' /\s*/ return_expression)* that were causing SKIPPED errors
in the old parser generation system.

=cut

=head2 analyze_rule_for_grouped_quantifiers($rule_name, $rule_def)

Analyze a rule definition to find grouped quantifier patterns.
Returns hash with analysis results.

=cut

sub analyze_rule_for_grouped_quantifiers {
    my ($rule_name, $rule_def) = @_;
    
    my $analysis = {
        rule_name => $rule_name,
        has_grouped_quantifiers => 0,
        grouped_patterns => [],
        needs_special_handling => 0,
    };
    
    # Check if this rule has OR groups
    if (exists $rule_def->{or_groups} && ref($rule_def->{or_groups}) eq 'ARRAY') {
        for my $group_idx (0..$#{$rule_def->{or_groups}}) {
            my $or_group = $rule_def->{or_groups}[$group_idx];
            my $group_analysis = analyze_group_for_quantifiers($or_group, $group_idx);
            
            if ($group_analysis->{has_grouped_quantifiers}) {
                $analysis->{has_grouped_quantifiers} = 1;
                push @{$analysis->{grouped_patterns}}, $group_analysis;
            }
        }
    }
    
    # Check if this rule has elements (sequence)
    if (exists $rule_def->{elements} && ref($rule_def->{elements}) eq 'ARRAY') {
        my $elements_analysis = analyze_elements_for_quantifiers($rule_def->{elements});
        
        if ($elements_analysis->{has_grouped_quantifiers}) {
            $analysis->{has_grouped_quantifiers} = 1;
            push @{$analysis->{grouped_patterns}}, $elements_analysis;
        }
    }
    
    # Determine if special handling is needed
    $analysis->{needs_special_handling} = $analysis->{has_grouped_quantifiers};
    
    return $analysis;
}

=head2 analyze_group_for_quantifiers($group_elements, $group_idx)

Analyze a group of elements for quantifier patterns.

=cut

sub analyze_group_for_quantifiers {
    my ($group_elements, $group_idx) = @_;
    
    my $analysis = {
        group_index => $group_idx,
        has_grouped_quantifiers => 0,
        quantifier_patterns => [],
        pattern_type => 'unknown',
    };
    
    return $analysis unless ref($group_elements) eq 'ARRAY';
    
    # Look for patterns that suggest grouped quantifiers
    my @elements = @$group_elements;
    
    # Pattern detection: Look for sequences with operators followed by quantifiers
    for my $i (0..$#elements) {
        my $element = $elements[$i];
        
        # Look for quantifier operators (* + ?)
        if (is_quantifier_operator($element)) {
            # Check if this quantifier is applied to a preceding group pattern
            my $group_pattern = detect_preceding_group_pattern(\@elements, $i);
            
            if ($group_pattern) {
                $analysis->{has_grouped_quantifiers} = 1;
                push @{$analysis->{quantifier_patterns}}, {
                    quantifier => extract_quantifier_type($element),
                    group_pattern => $group_pattern,
                    position => $i,
                };
                
                # Classify the pattern type
                if ($group_pattern->{type} eq 'comma_separated') {
                    $analysis->{pattern_type} = 'comma_separated_list';
                } elsif ($group_pattern->{type} eq 'sequence') {
                    $analysis->{pattern_type} = 'grouped_sequence';
                }
            }
        }
    }
    
    return $analysis;
}

=head2 analyze_elements_for_quantifiers($elements)

Analyze a list of elements for quantifier patterns.

=cut

sub analyze_elements_for_quantifiers {
    my ($elements) = @_;
    
    my $analysis = {
        has_grouped_quantifiers => 0,
        quantifier_patterns => [],
        element_count => scalar(@$elements),
    };
    
    # Look for grouped parentheses with quantifiers
    my $in_group = 0;
    my $group_start = -1;
    my @group_elements = ();
    
    for my $i (0..$#{$elements}) {
        my $element = $elements->[$i];
        
        if (is_group_open($element)) {
            $in_group = 1;
            $group_start = $i;
            @group_elements = ();
        } elsif (is_group_close($element)) {
            if ($in_group && @group_elements > 0) {
                # Check if next element is a quantifier
                if ($i + 1 <= $#{$elements} && is_quantifier_operator($elements->[$i + 1])) {
                    my $quantifier = extract_quantifier_type($elements->[$i + 1]);
                    
                    $analysis->{has_grouped_quantifiers} = 1;
                    push @{$analysis->{quantifier_patterns}}, {
                        quantifier => $quantifier,
                        group_elements => [@group_elements],
                        start_pos => $group_start,
                        end_pos => $i,
                        quantifier_pos => $i + 1,
                    };
                }
            }
            $in_group = 0;
        } elsif ($in_group) {
            push @group_elements, $element;
        }
    }
    
    return $analysis;
}

=head2 detect_preceding_group_pattern($elements, $quantifier_pos)

Detect if there's a group pattern preceding a quantifier.

=cut

sub detect_preceding_group_pattern {
    my ($elements, $quantifier_pos) = @_;
    
    return undef if $quantifier_pos == 0;
    
    # Look backwards from quantifier position
    my $pattern = {
        elements => [],
        type => 'unknown',
        start_pos => $quantifier_pos - 1,
        end_pos => $quantifier_pos - 1,
    };
    
    # Check for explicit group close before quantifier
    if (is_group_close($elements->[$quantifier_pos - 1])) {
        # Find matching group open
        my $paren_count = 1;
        my $group_start = -1;
        
        for my $i (reverse 0..$quantifier_pos - 2) {
            if (is_group_close($elements->[$i])) {
                $paren_count++;
            } elsif (is_group_open($elements->[$i])) {
                $paren_count--;
                if ($paren_count == 0) {
                    $group_start = $i;
                    last;
                }
            }
        }
        
        if ($group_start >= 0) {
            # Extract elements between parentheses
            for my $i ($group_start + 1..$quantifier_pos - 2) {
                push @{$pattern->{elements}}, $elements->[$i];
            }
            
            $pattern->{start_pos} = $group_start;
            $pattern->{end_pos} = $quantifier_pos - 1;
            
            # Analyze pattern type
            if (has_comma_pattern($pattern->{elements})) {
                $pattern->{type} = 'comma_separated';
            } else {
                $pattern->{type} = 'sequence';
            }
            
            return $pattern;
        }
    }
    
    return undef;
}

=head2 convert_to_packrat_structure($analysis)

Convert grouped quantifier analysis to PackratParser-compatible structure.

=cut

sub convert_to_packrat_structure {
    my ($analysis) = @_;
    
    return undef unless $analysis->{has_grouped_quantifiers};
    
    my @packrat_patterns = ();
    
    for my $pattern (@{$analysis->{grouped_patterns}}) {
        if (exists $pattern->{quantifier_patterns}) {
            for my $quant_pattern (@{$pattern->{quantifier_patterns}}) {
                push @packrat_patterns, {
                    type => 'grouped_quantified',
                    quantifier => $quant_pattern->{quantifier},
                    group_elements => convert_elements_to_packrat($quant_pattern->{group_elements} || $quant_pattern->{group_pattern}{elements}),
                    min => quantifier_to_min_max($quant_pattern->{quantifier})->[0],
                    max => quantifier_to_min_max($quant_pattern->{quantifier})->[1],
                };
            }
        }
    }
    
    return {
        rule_name => $analysis->{rule_name},
        patterns => \@packrat_patterns,
    };
}

=head2 convert_elements_to_packrat($elements)

Convert AST elements to PackratParser format.

=cut

sub convert_elements_to_packrat {
    my ($elements) = @_;
    
    my @packrat_elements = ();
    
    for my $element (@$elements) {
        if (is_terminal_element($element)) {
            push @packrat_elements, {
                type => 'terminal',
                value => extract_terminal_value($element),
                terminal_type => extract_terminal_type($element),
            };
        } elsif (is_rule_reference_element($element)) {
            push @packrat_elements, {
                type => 'rule_reference',
                rule_name => extract_rule_name($element),
            };
        } else {
            # Unknown element - add as generic
            push @packrat_elements, {
                type => 'unknown',
                raw_data => $element,
            };
        }
    }
    
    return \@packrat_elements;
}

# =============================================================================
# UTILITY FUNCTIONS
# =============================================================================

=head2 is_quantifier_operator($element)

Check if element is a quantifier operator.

=cut

sub is_quantifier_operator {
    my ($element) = @_;
    
    if (ref($element) eq 'ARRAY') {
        return $element->[0] eq 'operator' && $element->[1] =~ /^[*+?]$/;
    } elsif (ref($element) eq 'HASH') {
        return $element->{type} eq 'operator' && 
               exists $element->{value} && 
               $element->{value} =~ /^[*+?]$/;
    } elsif (!ref($element)) {
        return $element =~ /^[*+?]$/;
    }
    
    return 0;
}

=head2 is_group_open($element)

Check if element is a group opening parenthesis.

=cut

sub is_group_open {
    my ($element) = @_;
    
    if (ref($element) eq 'ARRAY') {
        return $element->[0] eq 'group_open' || 
               ($element->[0] eq 'quoted_string' && $element->[1] eq '(');
    } elsif (ref($element) eq 'HASH') {
        return $element->{type} eq 'group_open' ||
               ($element->{type} eq 'terminal' && $element->{value} eq '(');
    } elsif (!ref($element)) {
        return $element eq '(' || $element eq 'group_open';
    }
    
    return 0;
}

=head2 is_group_close($element)

Check if element is a group closing parenthesis.

=cut

sub is_group_close {
    my ($element) = @_;
    
    if (ref($element) eq 'ARRAY') {
        return $element->[0] eq 'group_close' || 
               ($element->[0] eq 'quoted_string' && $element->[1] eq ')');
    } elsif (ref($element) eq 'HASH') {
        return $element->{type} eq 'group_close' ||
               ($element->{type} eq 'terminal' && $element->{value} eq ')');
    } elsif (!ref($element)) {
        return $element eq ')' || $element eq 'group_close';
    }
    
    return 0;
}

=head2 has_comma_pattern($elements)

Check if elements contain comma-separated pattern.

=cut

sub has_comma_pattern {
    my ($elements) = @_;
    
    for my $element (@$elements) {
        if (ref($element) eq 'ARRAY') {
            return 1 if $element->[0] eq 'quoted_string' && $element->[1] eq ',';
        } elsif (ref($element) eq 'HASH') {
            return 1 if $element->{type} eq 'terminal' && $element->{value} eq ',';
        } elsif (!ref($element)) {
            return 1 if $element eq ',';
        }
    }
    
    return 0;
}

=head2 extract_quantifier_type($element)

Extract quantifier type from element.

=cut

sub extract_quantifier_type {
    my ($element) = @_;
    
    if (ref($element) eq 'ARRAY') {
        return $element->[1];
    } elsif (ref($element) eq 'HASH') {
        return $element->{value};
    } elsif (!ref($element)) {
        return $element;
    }
    
    return '*';  # Default
}

=head2 is_terminal_element($element)

Check if element is a terminal.

=cut

sub is_terminal_element {
    my ($element) = @_;
    
    if (ref($element) eq 'ARRAY') {
        return $element->[0] =~ /^(quoted_string|regex|number)$/;
    } elsif (ref($element) eq 'HASH') {
        return exists $element->{type} && 
               $element->{type} =~ /^(terminal|literal|regex)$/;
    }
    
    return 0;
}

=head2 is_rule_reference_element($element)

Check if element is a rule reference.

=cut

sub is_rule_reference_element {
    my ($element) = @_;
    
    if (ref($element) eq 'ARRAY') {
        return $element->[0] eq 'rule_reference';
    } elsif (ref($element) eq 'HASH') {
        return exists $element->{type} && $element->{type} eq 'rule_reference';
    } elsif (!ref($element)) {
        return $element =~ /^[a-zA-Z_]\w*$/;
    }
    
    return 0;
}

=head2 extract_terminal_value($element)

Extract terminal value.

=cut

sub extract_terminal_value {
    my ($element) = @_;
    
    if (ref($element) eq 'ARRAY') {
        return $element->[1];
    } elsif (ref($element) eq 'HASH') {
        return $element->{value} || $element->{pattern};
    }
    
    return '';
}

=head2 extract_terminal_type($element)

Extract terminal type.

=cut

sub extract_terminal_type {
    my ($element) = @_;
    
    if (ref($element) eq 'ARRAY') {
        return $element->[0];
    } elsif (ref($element) eq 'HASH') {
        return $element->{type};
    }
    
    return 'unknown';
}

=head2 extract_rule_name($element)

Extract rule name from rule reference.

=cut

sub extract_rule_name {
    my ($element) = @_;
    
    if (ref($element) eq 'ARRAY') {
        return $element->[1];
    } elsif (ref($element) eq 'HASH') {
        return $element->{rule_name} || $element->{name};
    } elsif (!ref($element)) {
        return $element;
    }
    
    return 'unknown_rule';
}

=head2 quantifier_to_min_max($quantifier)

Convert quantifier to min/max bounds.

=cut

sub quantifier_to_min_max {
    my ($quantifier) = @_;
    
    return [0, 999999] if $quantifier eq '*';
    return [1, 999999] if $quantifier eq '+';
    return [0, 1] if $quantifier eq '?';
    
    # Handle {n}, {n,}, {n,m} patterns
    if ($quantifier =~ /^\{(\d+)\}$/) {
        return [$1, $1];
    } elsif ($quantifier =~ /^\{(\d+),\}$/) {
        return [$1, 999999];
    } elsif ($quantifier =~ /^\{(\d+),(\d+)\}$/) {
        return [$1, $2];
    }
    
    return [1, 1];  # Default
}

1;

__END__

=head1 EXAMPLE USAGE

    use AST::GroupedQuantifierAnalyzer;
    
    # Analyze a rule for grouped quantifiers
    my $analysis = AST::GroupedQuantifierAnalyzer::analyze_rule_for_grouped_quantifiers(
        'array_contents', $rule_def
    );
    
    if ($analysis->{has_grouped_quantifiers}) {
        print "Rule has grouped quantifiers!\n";
        
        # Convert to PackratParser format
        my $packrat_structure = AST::GroupedQuantifierAnalyzer::convert_to_packrat_structure($analysis);
        
        # Use in parser generation...
    }

=head1 PURPOSE

This module is specifically designed to fix the "SKIPPED: Unhandled quantified element type"
error that was preventing multi-element array parsing like `[$1, $2]` from working correctly.

By properly analyzing and extracting grouped quantifier patterns from the AST, we can generate
the correct PackratParser code to handle expressions like:

    array_contents := return_expression (',' /\s*/ return_expression)*

=cut
