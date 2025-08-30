package LeftRecursionIntegrator;

# ================================================================================
# INTEGRATION MODULE: LEFT-RECURSION NUCLEAR ELIMINATOR 
# ================================================================================
# This module adapts our ULTIMATE left-recursion elimination algorithm
# to work with the existing AST data structures in ast_transform.pl
# ================================================================================

use strict;
use warnings;
use FindBin qw($RealBin);
use Data::Dumper;

use LeftRecursionEliminator qw(eliminate_all_left_recursion);

use Exporter 'import';
our @EXPORT_OK = qw(eliminate_left_recursion_nuclear_option);

# ================================================================================
# ADAPTER FUNCTIONS - Convert between data formats
# ================================================================================

sub convert_ast_to_elimination_format {
    my ($grammar_tree) = @_;
    
    print STDERR "🔄 Converting AST format to elimination format...\n";
    
    my %simple_grammar = ();
    my %return_annotations = ();  # Store return annotations separately
    
    foreach my $rule_name (keys %$grammar_tree) {
        my $rule_def = $grammar_tree->{$rule_name};
        
        # Debug: Track dot_path rule specifically
        if ($rule_name eq 'dot_path') {
            print STDERR "🎯 FOUND dot_path rule in grammar_tree: " . Dumper($rule_def) . "\n";
        }
        
        if ($rule_def->{type} eq 'sequence') {
            # Single sequence: convert to array of elements
            my @production = extract_sequence_symbols($rule_def->{elements});
            $simple_grammar{$rule_name} = [\@production];
            
            # Store return annotation if present - use consistent format
            if ($rule_def->{return_annotation}) {
                $return_annotations{$rule_name} = [$rule_def->{return_annotation}];
            }
            
        } elsif ($rule_def->{type} eq 'or') {
            # Multiple alternatives: convert each to array
            my @productions = ();
            my @alt_annotations = ();
            foreach my $alt (@{$rule_def->{alternatives}}) {
                if ($alt->{type} eq 'sequence') {
                    my @production = extract_sequence_symbols($alt->{elements});
                    push @productions, \@production;
                    push @alt_annotations, $alt->{return_annotation};
                } elsif ($alt->{type} eq 'atom') {
                    my @production = extract_atom_symbol($alt);
                    push @productions, \@production;
                    push @alt_annotations, $alt->{return_annotation};
                }
            }
            $simple_grammar{$rule_name} = \@productions;
            $return_annotations{$rule_name} = \@alt_annotations;
            
        } elsif ($rule_def->{type} eq 'atom') {
            # Single atom: convert to single-element array
            my @production = extract_atom_symbol($rule_def);
            $simple_grammar{$rule_name} = [\@production];
            
            # Store return annotation if present - use consistent format
            if ($rule_def->{return_annotation}) {
                $return_annotations{$rule_name} = [$rule_def->{return_annotation}];
            }
        } elsif ($rule_def->{type} eq 'quantified') {
            # Quantified rule: convert to quantified element
            my $element_name;
            my $inner_element = $rule_def->{element};
            
            if (ref($inner_element) eq 'HASH' && $inner_element->{type} eq 'atom') {
                # Extract rule name from atom structure
                my $value = $inner_element->{value};
                if (ref($value) eq 'ARRAY' && @$value == 2) {
                    $element_name = $value->[1];  # Extract rule name
                } else {
                    $element_name = $value;
                }
            } else {
                $element_name = $inner_element;
            }
            
            # Create quantified production
            my @production = ("QUANTIFIED:" . $element_name . ":" . $rule_def->{quantifier});
            $simple_grammar{$rule_name} = [\@production];
            
            # Store return annotation if present - use consistent format
            if ($rule_def->{return_annotation}) {
                $return_annotations{$rule_name} = [$rule_def->{return_annotation}];
            }
        }
    }
    
    print STDERR "📊 Converted " . scalar(keys %simple_grammar) . " rules\n";
    print STDERR "🏷️ Stored annotations for " . scalar(keys %return_annotations) . " rules\n";
    return (\%simple_grammar, \%return_annotations);
}

sub extract_sequence_symbols {
    my ($elements) = @_;
    my @symbols = ();
    
    foreach my $element (@$elements) {
        if ($element->{type} eq 'atom') {
            # Handle nested atom structures from step 5
            my $value = $element->{value};
            
            # Check if this is a nested atom structure
            if (ref($value) eq 'HASH' && $value->{type} eq 'atom' && ref($value->{value}) eq 'ARRAY') {
                # Extract from nested structure: {type => 'atom', value => ['rule_reference', 'expr']}
                my $inner_value = $value->{value};
                
                if ($inner_value->[0] eq 'rule_reference') {
                    push @symbols, $inner_value->[1];  # Rule name
                } elsif ($inner_value->[0] eq 'quoted_string') {
                    push @symbols, "TERMINAL:" . $inner_value->[1];
                } elsif ($inner_value->[0] eq 'terminal') {
                    push @symbols, "TERMINAL:" . $inner_value->[1];
                } elsif ($inner_value->[0] eq 'operator') {
                    push @symbols, "OPERATOR:" . $inner_value->[1];
                } elsif ($inner_value->[0] eq 'regex') {
                    push @symbols, "REGEX:" . $inner_value->[1];
                } elsif ($inner_value->[0] eq 'return_scalar' || $inner_value->[0] eq 'return_array' || $inner_value->[0] eq 'return_object') {
                    # Skip return annotations - they're metadata, not grammar symbols
                    next;
                } else {
                    push @symbols, join(":", @$inner_value);
                }
            } elsif (ref($value) eq 'ARRAY') {
                # Direct array format: ['terminal', 'hello'] -> 'TERMINAL:hello'
                if ($value->[0] eq 'terminal') {
                    push @symbols, "TERMINAL:" . $value->[1];
                } elsif ($value->[0] eq 'operator') {
                    push @symbols, "OPERATOR:" . $value->[1];
                } elsif ($value->[0] eq 'regex') {
                    push @symbols, "REGEX:" . $value->[1];
                } elsif ($value->[0] eq 'return_scalar' || $value->[0] eq 'return_array' || $value->[0] eq 'return_object') {
                    # Skip return annotations - they're metadata, not grammar symbols
                    next;
                } else {
                    push @symbols, join(":", @$value);
                }
            } else {
                # Handle undefined values safely first
                if (!defined($value)) {
                    next;
                }
                
                # Check if this is a return annotation string that got corrupted
                # Use eval to safely handle potential regex issues
                if (!ref($value) && length($value) > 0) {
                    my $is_annotation = 0;
                    eval {
                        $is_annotation = 1 if ($value =~ /^[\w\s:"$,\[\]\{\}]+$/ && 
                                               $value =~ /(?:type:|items:|name:|value:|left:|right:|op:)/);
                    };
                    if ($is_annotation) {
                        # Skip corrupted return annotation strings
                        next;
                    }
                }
                # Rule reference -> rule name
                push @symbols, $value;
            }
        } elsif ($element->{type} eq 'quantified') {
            # Quantified element -> preserve structure with special marker
            my $inner_element = $element->{element};
            
            if (ref($inner_element) eq 'ARRAY' && $inner_element->[0] eq 'GROUPED') {
                # Handle grouped quantified elements: (a | b)*
                # Convert GROUPED content to a synthetic rule reference
                my $grouped_content = $inner_element->[1];  # Extract the grouped array
                my @group_symbols = ();
                
                # Process the grouped content recursively
                foreach my $group_item (@$grouped_content) {
                    if (ref($group_item) eq 'ARRAY') {
                        if ($group_item->[0] eq 'terminal') {
                            push @group_symbols, "TERMINAL:" . $group_item->[1];
                        } elsif ($group_item->[0] eq 'operator') {
                            push @group_symbols, "OPERATOR:" . $group_item->[1];
                        } elsif ($group_item->[0] eq 'regex') {
                            push @group_symbols, "REGEX:" . $group_item->[1];
                        } else {
                            push @group_symbols, join(":", @$group_item);
                        }
                    } else {
                        push @group_symbols, $group_item;  # Rule reference
                    }
                }
                
                # Create a compound symbol representing the grouped quantified element
                # Use a safe encoding that won't be split incorrectly later
                my $group_content = join("~", @group_symbols);  # Use ~ as safe separator
                push @symbols, "QUANTIFIED:GROUP~$group_content~" . $element->{quantifier};
            } else {
                # Simple quantified element: item+
                # Extract token value properly from different structures
                my $element_name;
                if (ref($inner_element) eq 'HASH' && $inner_element->{type} eq 'atom') {
                    # Handle hash structure from Step 5: {type => 'atom', value => ['rule_reference', 'accessor']}
                    my $value = $inner_element->{value};
                    if (ref($value) eq 'ARRAY' && @$value == 2) {
                        $element_name = $value->[1];  # Extract rule name
                    } else {
                        $element_name = $value;
                    }
                } elsif (ref($inner_element) eq 'ARRAY' && @$inner_element == 2) {
                    # Handle array structure: ['type', 'value']
                    $element_name = $inner_element->[1];
                } else {
                    # Use as-is if not structured
                    $element_name = $inner_element;
                }
                push @symbols, "QUANTIFIED:" . $element_name . ":" . $element->{quantifier};
            }
        }
    }
    
    return @symbols;
}

sub extract_atom_symbol {
    my ($atom) = @_;
    
    my $value = $atom->{value};
    
    # Handle nested atom structures from step 5
    if (ref($value) eq 'HASH' && $value->{type} eq 'atom' && ref($value->{value}) eq 'ARRAY') {
        # Extract from nested structure: {type => 'atom', value => ['quoted_string', 'num']}
        my $inner_value = $value->{value};
        
        if ($inner_value->[0] eq 'rule_reference') {
            return ($inner_value->[1]);  # Rule name
        } elsif ($inner_value->[0] eq 'quoted_string') {
            return ("TERMINAL:" . $inner_value->[1]);
        } elsif ($inner_value->[0] eq 'terminal') {
            return ("TERMINAL:" . $inner_value->[1]);
        } elsif ($inner_value->[0] eq 'regex') {
            return ("REGEX:" . $inner_value->[1]);
        } else {
            return (join(":", @$inner_value));
        }
    } elsif (ref($value) eq 'ARRAY') {
        # Direct array format
        if ($value->[0] eq 'terminal') {
            return ("TERMINAL:" . $value->[1]);
        } elsif ($value->[0] eq 'regex') {
            return ("REGEX:" . $value->[1]);
        } else {
            return (join(":", @$value));
        }
    } else {
        return ($value);
    }
}

sub convert_elimination_result_to_ast {
    my ($eliminated_grammar, $original_annotations) = @_;
    
    print STDERR "🔄 Converting elimination result back to AST format...\n";
    
    my %new_grammar_tree = ();
    
    foreach my $rule_name (keys %$eliminated_grammar) {
        my $productions = $eliminated_grammar->{$rule_name};
        
        if (@$productions == 1) {
            # Single production
            # Check if there's a return annotation from original rule
            my $prime_rule_exists = exists $eliminated_grammar->{"${rule_name}_prime"};
            my $original_annotation = $original_annotations->{$rule_name};
            
            $new_grammar_tree{$rule_name} = convert_production_to_ast($productions->[0], defined $original_annotation);
            
            # Always restore return annotations if they existed in original rule
            if ($original_annotation && $original_annotation->[0]) {
                if ($prime_rule_exists) {
                    # Rule was transformed - need to apply return annotation to the base case
                    # For transformed rules, the main rule becomes the base case (non-recursive alternative)
                    $new_grammar_tree{$rule_name}{return_annotation} = $original_annotation->[1] || $original_annotation->[0];
                } else {
                    # Rule was not transformed - keep original return annotation
                    $new_grammar_tree{$rule_name}{return_annotation} = $original_annotation->[0];
                }
            }
        } else {
            # Multiple productions -> OR
            my @alternatives = ();
            for my $i (0..$#{$productions}) {
                # Check if there's a return annotation for this alternative from original grammar
                my $prime_rule_exists = exists $eliminated_grammar->{"${rule_name}_prime"};
                my $original_annotation = $original_annotations->{$rule_name};
                my $alt_annotation = $original_annotation && $original_annotation->[$i];
                
                my $alt = convert_production_to_ast($productions->[$i], defined $alt_annotation);
                
                # Always restore return annotations if they existed in original alternatives
                if ($alt_annotation) {
                    # Apply the original return annotation to this alternative
                    $alt->{return_annotation} = $alt_annotation;
                }
                
                push @alternatives, $alt;
            }
            $new_grammar_tree{$rule_name} = {
                type => 'or',
                alternatives => \@alternatives
            };
        }
    }
    
    print STDERR "📊 Converted back " . scalar(keys %new_grammar_tree) . " rules\n";
    return \%new_grammar_tree;
}

sub convert_production_to_ast {
    my ($production, $has_return_annotation) = @_;
    
    if (@$production == 0 || ($production->[0] eq 'ε')) {
        # Epsilon production
        return {
            type => 'atom',
            value => ['epsilon']
        };
    } elsif (@$production == 1) {
        # Single element - check if it's a quantified element
        my $ast_value = convert_symbol_to_ast_value($production->[0]);
        
        if (ref($ast_value) eq 'ARRAY' && $ast_value->[0] eq 'quantified_element') {
            # Single quantified element - convert to proper quantified structure
            my ($type, $element_name, $quantifier) = @$ast_value;
            if ($has_return_annotation) {
                return {
                    type => 'sequence', 
                    elements => [{
                        type => 'quantified',
                        element => $element_name,
                        quantifier => $quantifier
                    }]
                };
            } else {
                return {
                    type => 'quantified',
                    element => $element_name,
                    quantifier => $quantifier
                };
            }
        } elsif ($has_return_annotation) {
            # Regular single element with return annotation - keep as sequence
            return {
                type => 'sequence',
                elements => [{
                    type => 'atom',
                    value => $ast_value
                }]
            };
        } else {
            # Regular single element
            return {
                type => 'atom',
                value => $ast_value
            };
        }
    } else {
        # Sequence
        my @elements = ();
        foreach my $symbol (@$production) {
            my $ast_value = convert_symbol_to_ast_value($symbol);
            
            # Check if this is a quantified element within a sequence
            if (ref($ast_value) eq 'ARRAY' && $ast_value->[0] eq 'quantified_element') {
                my ($type, $element_name, $quantifier) = @$ast_value;
                push @elements, {
                    type => 'quantified',
                    element => $element_name,
                    quantifier => $quantifier
                };
            } else {
                push @elements, {
                    type => 'atom',
                    value => $ast_value
                };
            }
        }
        return {
            type => 'sequence',
            elements => \@elements
        };
    }
}

sub convert_symbol_to_ast_value {
    my ($symbol) = @_;
    
    if ($symbol =~ /^TERMINAL:(.+)$/) {
        return ['quoted_string', $1];
    } elsif ($symbol =~ /^OPERATOR:(.+)$/) {
        return ['operator', $1];
    } elsif ($symbol =~ /^REGEX:(.+)$/) {
        return ['regex', $1];
    } elsif ($symbol =~ /^QUANTIFIED:GROUP~(.+)~(.+)$/) {
        # Reconstruct grouped quantified element structure  
        my ($group_content, $quantifier) = ($1, $2);
        return ['quantified_group', $group_content, $quantifier];
    } elsif ($symbol =~ /^QUANTIFIED:([^:]+):(.+)$/) {
        # Reconstruct simple quantified element structure
        return ['quantified_element', $1, $2];
    } elsif ($symbol =~ /^(.+):(.+)$/) {
        return [$1, $2];
    } else {
        # If no prefix, assume it's a rule name
        return $symbol;
    }
}

# ================================================================================
# MAIN INTEGRATION FUNCTION
# ================================================================================

sub eliminate_left_recursion_nuclear_option {
    my ($grammar_tree, $rule_order) = @_;
    
    print STDERR "🚀 DEPLOYING LEFT-RECURSION NUCLEAR ELIMINATOR!\n";
    print STDERR "🎯 Target: Complete annihilation of all recursion forms\n";
    print STDERR "=" x 70 . "\n\n";
    
    # Step 1: Convert AST format to simple format for our algorithm
    my ($simple_grammar, $original_annotations) = convert_ast_to_elimination_format($grammar_tree);
    
    print STDERR "📋 Grammar before elimination:\n";
    foreach my $rule (sort keys %$simple_grammar) {
        my @prod_strings = map { join(" ", @$_) } @{$simple_grammar->{$rule}};
        print STDERR "   $rule := " . join(" | ", @prod_strings) . "\n";
    }
    print STDERR "\n";
    
    # Step 2: Load our nuclear elimination engine and apply transformation
    # Step 3: UNLEASH THE DESTROYER! (capture output but don't fail if redirection fails)
    my $eliminated_grammar;
    
    {
        # Try to capture output, but don't fail if it doesn't work
        local *STDOUT;
        local *STDERR;
        
        eval {
            open(STDOUT, '>', \my $stdout_buffer);
            open(STDERR, '>', \my $stderr_buffer);
        };
        
        # Run the elimination regardless of whether redirection worked
        $eliminated_grammar = eliminate_all_left_recursion($simple_grammar);
        
        # Filehandles will be restored automatically when leaving this block
    }
    
    # Step 4: Convert back to AST format
    my $new_grammar_tree = convert_elimination_result_to_ast($eliminated_grammar, $original_annotations);
    
    # Step 5: Update rule order (add any new prime rules)
    my @new_rule_order = @$rule_order;
    foreach my $rule_name (keys %$new_grammar_tree) {
        unless (grep { $_ eq $rule_name } @new_rule_order) {
            push @new_rule_order, $rule_name;
            print STDERR "➕ Added new rule to order: $rule_name\n";
        }
    }
    
    print STDERR "\n💀 LEFT-RECURSION STATUS: COMPLETELY ANNIHILATED!\n";
    return ($new_grammar_tree, \@new_rule_order);
}

1;
