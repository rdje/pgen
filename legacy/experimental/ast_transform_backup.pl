#!/usr/bin/env perl
use strict;
use warnings;
use Data::Dumper;
use lib 'fx/perl';

# AST Transformation Pipeline for := grammar rules
# Takes raw token AST and transforms it into structured parse tree

sub step2_group_by_or {
    my ($raw_ast) = @_;
    my @transformed_rules;
    
    foreach my $rule_tokens (@$raw_ast) {
        my ($rule_name, @tokens) = @$rule_tokens;
        
        # Split tokens on | operators
        my @or_groups = ();
        my @current_group = ();
        
        foreach my $token (@tokens) {
            if ($token eq '|') {
                # Save current group and start new one
                push @or_groups, [@current_group] if @current_group;
                @current_group = ();
            } else {
                push @current_group, $token;
            }
        }
        
        # Don't forget the last group
        push @or_groups, [@current_group] if @current_group;
        
        # If only one group, no OR operation
        if (@or_groups == 1) {
            push @transformed_rules, [$rule_name, 'SEQUENCE', $or_groups[0]];
        } else {
            push @transformed_rules, [$rule_name, 'OR', \@or_groups];
        }
    }
    
    return \@transformed_rules;
}

# Get real AST from EBNF parser
use LinkedSpec;
open my $fh, "<", "fx/specs/ebnf.spec" or die "Cannot open ebnf.spec: $!";
my $spec_content = do { local $/; <$fh> };
close $fh;
my $quiet_mode = 0;
my $ebnf_file = "test_grammar.ebnf";

# Parse command line arguments  
for my $arg (@ARGV) {
    if ($arg eq '--quiet') {
        $quiet_mode = 1;
    } else {
        $ebnf_file = $arg;
    }
}

open my $fh2, "<", $ebnf_file or die "Cannot open $ebnf_file: $!";
my $input_content = do { local $/; <$fh2> };
close $fh2;
print "=== Parsing $ebnf_file ===\n" unless $quiet_mode;
my $parser = LinkedSpec::Get(\$spec_content);
my $raw_ast = $parser->(\$input_content);
print "RAW AST from EBNF parser:\n" . Dumper($raw_ast) unless $quiet_mode;

sub step2_5_handle_parentheses {
    my ($grouped_ast) = @_;
    my @result = ();
    
    foreach my $rule (@$grouped_ast) {
        my ($rule_name, $type, $data) = @$rule;
        
        if ($type eq 'OR') {
            # Process each OR alternative for parentheses
            my @processed_alternatives = ();
            foreach my $alternative (@$data) {
                my $processed = process_parentheses_in_sequence($alternative);
                push @processed_alternatives, $processed;
            }
            push @result, [$rule_name, $type, \@processed_alternatives];
        } else {
            # Non-OR rules: process the data directly  
            my $processed = process_parentheses_in_sequence($data);
            push @result, [$rule_name, $type, $processed];
        }
    }
    
    return \@result;
}

sub process_parentheses_in_sequence {
    my ($sequence) = @_;
    return $sequence unless ref($sequence) eq 'ARRAY';
    
    my @result = ();
    my @stack = ();
    my $depth = 0;
    
    for my $token (@$sequence) {
        if ($token eq '(') {
            $depth++;
            push @stack, [];
        } elsif ($token eq ')') {
            $depth--;
            if (@stack) {
                my $group_content = pop @stack;
                if ($depth == 0) {
                    # Top-level group - add to result as grouped
                    push @result, ['GROUPED', $group_content];
                } else {
                    # Nested group - add to parent group
                    push @{$stack[-1]}, ['GROUPED', $group_content];
                }
            }
        } else {
            if ($depth > 0 && @stack) {
                # Inside parentheses - add to current group
                push @{$stack[-1]}, $token;
            } else {
                # Outside parentheses - add to result
                push @result, $token;
            }
        }
    }
    
    return \@result;
}

sub step3_parse_sequences {
    my ($grouped_ast) = @_;
    my @transformed_rules;
    
    foreach my $rule (@$grouped_ast) {
        my ($rule_name, $type, $data) = @$rule;
        
        if ($type eq 'OR') {
            # Process each OR alternative as a sequence
            my @or_alternatives = ();
            foreach my $alternative (@$data) {
                if (@$alternative == 1) {
                    # Single token
                    push @or_alternatives, ['ATOM', $alternative->[0]];
                } else {
                    # Multiple tokens = sequence
                    push @or_alternatives, ['SEQUENCE', $alternative];
                }
            }
            push @transformed_rules, [$rule_name, 'OR', \@or_alternatives];
        } elsif ($type eq 'SEQUENCE') {
            # Already a sequence, just wrap atoms
            if (@$data == 1) {
                push @transformed_rules, [$rule_name, 'ATOM', $data->[0]];
            } else {
                push @transformed_rules, [$rule_name, 'SEQUENCE', $data];
            }
        }
    }
    
    return \@transformed_rules;
}

print "\n=== Step 2: Group by OR operators ===\n" unless $quiet_mode;
my $step2_result = step2_group_by_or($raw_ast);
print "STEP 2 RESULT (Grouped by OR):\n" . Dumper($step2_result) unless $quiet_mode;

sub step4_handle_quantifiers {
    my ($sequence_ast) = @_;
    my @transformed_rules;
    
    foreach my $rule (@$sequence_ast) {
        my ($rule_name, $type, $data) = @$rule;
        
        if ($type eq 'OR') {
            # Process each OR alternative
            my @or_alternatives = ();
            foreach my $alternative (@$data) {
                my ($alt_type, $alt_data) = @$alternative;
                if ($alt_type eq 'SEQUENCE') {
                    push @or_alternatives, ['SEQUENCE', process_quantifiers_in_sequence($alt_data)];
                } elsif ($alt_type eq 'ATOM') {
                    push @or_alternatives, ['ATOM', $alt_data];
                }
            }
            push @transformed_rules, [$rule_name, 'OR', \@or_alternatives];
        } elsif ($type eq 'SEQUENCE') {
            push @transformed_rules, [$rule_name, 'SEQUENCE', process_quantifiers_in_sequence($data)];
        } elsif ($type eq 'ATOM') {
            push @transformed_rules, [$rule_name, 'ATOM', $data];
        }
    }
    
    return \@transformed_rules;
}

sub process_quantifiers_in_sequence {
    my ($tokens) = @_;
    my @result = ();
    
    for (my $i = 0; $i < @$tokens; $i++) {
        my $token = $tokens->[$i];
        
        if (is_quantifier($token)) {
            # Validate: quantifier must have valid left element
            if (@result == 0) {
                die "Syntax error: Quantifier '$token' at beginning of sequence";
            }
            
            my $prev_element = pop @result;
            if (ref($prev_element) && $prev_element->[0] eq 'QUANTIFIED') {
                die "Syntax error: Double quantifier '$token' after '$prev_element->[2]'";
            }
            
            # Attach quantifier to previous element
            push @result, ['QUANTIFIED', $prev_element, $token];
        } else {
            push @result, $token;
        }
    }
    
    return \@result;
}

sub is_quantifier {
    my ($token) = @_;
    return $token =~ /^[\d,]+$/ ||  # Matches patterns like "1,3", "1,", "0,3"
           $token =~ /^[\+\*\?]$/;  # Matches +, *, ?
}

print "\n=== Step 2.5: Handle parentheses grouping ===\n" unless $quiet_mode;
my $step2_5_result = step2_5_handle_parentheses($step2_result);
print "STEP 2.5 RESULT (Parentheses handled):\n" . Dumper($step2_5_result) unless $quiet_mode;

print "\n=== Step 3: Parse sequences ===\n" unless $quiet_mode;
my $step3_result = step3_parse_sequences($step2_5_result);
print "STEP 3 RESULT (Sequences parsed):\n" . Dumper($step3_result) unless $quiet_mode;

sub step5_build_tree_structure {
    my ($quantified_ast) = @_;
    my %grammar_tree = ();
    my @rule_order = ();
    
    foreach my $rule (@$quantified_ast) {
        my ($rule_name, $type, $data) = @$rule;
        
        # Add to rule order only on first occurrence
        push @rule_order, $rule_name unless exists $grammar_tree{$rule_name};
        
        # Convert current rule to a standard alternative structure
        my $new_alternative;
        if ($type eq 'OR') {
            # OR node with multiple alternatives - flatten them
            my @alternatives = ();
            foreach my $alternative (@$data) {
                my ($alt_type, $alt_data) = @$alternative;
                if ($alt_type eq 'SEQUENCE') {
                    push @alternatives, {
                        type => 'sequence',
                        elements => build_sequence_elements($alt_data)
                    };
                } elsif ($alt_type eq 'ATOM') {
                    push @alternatives, {
                        type => 'atom',
                        value => $alt_data
                    };
                }
            }
            $new_alternative = {
                type => 'or',
                alternatives => \@alternatives
            };
        } elsif ($type eq 'SEQUENCE') {
            my ($elements, $return_annotation) = build_sequence_elements($data);
            $new_alternative = {
                type => 'sequence',
                elements => $elements,
                return_annotation => $return_annotation
            };
        } elsif ($type eq 'ATOM') {
            $new_alternative = {
                type => 'atom',
                value => $data
            };
        }
        
        # Handle multiple rules with same name by combining into OR
        if (exists $grammar_tree{$rule_name}) {
            my $existing = $grammar_tree{$rule_name};
            
            # If existing rule is already an OR, add to its alternatives
            if ($existing->{type} eq 'or') {
                if ($new_alternative->{type} eq 'or') {
                    # Both are OR - merge their alternatives
                    push @{$existing->{alternatives}}, @{$new_alternative->{alternatives}};
                } else {
                    # New is single alternative - add it
                    push @{$existing->{alternatives}}, $new_alternative;
                }
            } else {
                # Existing is single rule - convert to OR with both alternatives
                if ($new_alternative->{type} eq 'or') {
                    # New is OR - prepend existing as first alternative
                    unshift @{$new_alternative->{alternatives}}, $existing;
                    $grammar_tree{$rule_name} = $new_alternative;
                } else {
                    # Both are single - create new OR with both
                    $grammar_tree{$rule_name} = {
                        type => 'or',
                        alternatives => [$existing, $new_alternative]
                    };
                }
            }
        } else {
            # First occurrence of this rule name
            $grammar_tree{$rule_name} = $new_alternative;
        }
    }
    
    return (\%grammar_tree, \@rule_order);
}

sub validate_grammar_completeness {
    my ($grammar_tree) = @_;
    my %defined_rules = map { $_ => 1 } keys %$grammar_tree;
    my @errors = ();
    
    # Collect all referenced rule names
    my %referenced_rules = ();
    
    for my $rule_name (keys %$grammar_tree) {
        my $rule = $grammar_tree->{$rule_name};
        collect_referenced_rules($rule, \%referenced_rules);
    }
    
    # Check for undefined rules
    for my $ref_rule (keys %referenced_rules) {
        unless ($defined_rules{$ref_rule}) {
            push @errors, "Undefined rule referenced: '$ref_rule'";
        }
    }
    
    if (@errors) {
        die "Grammar validation errors:\n" . join("\n", map { "  $_" } @errors) . "\n";
    }
}

sub collect_referenced_rules {
    my ($node, $referenced) = @_;
    
    if (ref($node) eq 'HASH') {
        if ($node->{type} eq 'atom' && !is_terminal($node->{value})) {
            # This is a rule reference (non-terminal)
            $referenced->{$node->{value}} = 1;
        } elsif ($node->{type} eq 'sequence' || $node->{type} eq 'or') {
            for my $element (@{$node->{elements} || $node->{alternatives} || []}) {
                collect_referenced_rules($element, $referenced);
            }
        } elsif ($node->{type} eq 'quantified') {
            collect_referenced_rules($node->{element}, $referenced);
        }
    }
}

sub build_sequence_elements {
    my ($elements) = @_;
    my @result = ();
    my $return_annotation = undef;
    
    foreach my $element (@$elements) {
        if (ref($element) && $element->[0] eq 'QUANTIFIED') {
            # Quantified element
            my ($type, $value, $quantifier) = @$element;
            push @result, {
                type => 'quantified',
                element => $value,
                quantifier => $quantifier
            };
        } elsif (ref($element) eq 'ARRAY' && is_return_annotation($element)) {
            # Return annotation - extract but don't add to sequence elements
            $return_annotation = $element;
        } else {
            # Regular element
            push @result, {
                type => 'atom', 
                value => $element
            };
        }
    }
    
    return (\@result, $return_annotation);
}

print "\n=== Step 4: Handle quantifiers ===\n" unless $quiet_mode;
my $step4_result = step4_handle_quantifiers($step3_result);
print "STEP 4 RESULT (Quantifiers handled):\n" . Dumper($step4_result) unless $quiet_mode;

sub step6_generate_parser_code {
    my ($grammar_tree, $rule_order) = @_;
    
    # First pass: validate all referenced rules exist
    validate_grammar_completeness($grammar_tree);
    
    # Second pass: detect and eliminate left recursion
    my ($transformed_grammar, $new_rule_order) = eliminate_left_recursion($grammar_tree, $rule_order);
    
    # Generate a complete Perl module with fast parsing subroutines
    my $module_code = generate_parser_module($transformed_grammar, $new_rule_order);
    
    return $module_code;
}

sub generate_parser_module {
    my ($grammar_tree, $rule_order) = @_;
    
    my @subroutines = ();
    my @regex_definitions = ();
    
    # Generate fast parsing subroutines for each rule
    foreach my $rule_name (keys %$grammar_tree) {
        my $rule_def = $grammar_tree->{$rule_name};
        my ($sub_code, $regexes) = generate_fast_parser_sub($rule_name, $rule_def);
        push @subroutines, $sub_code;
        push @regex_definitions, @$regexes if $regexes;
    }
    
    # Build complete module  
    my $main_rule = $rule_order->[0];  # First rule is always the main entry point
    
    my $regex_definitions = join(",\n", @regex_definitions);
    my $subroutines = join("\n\n", @subroutines);
    
    my $module = <<"EOF";
package yapg::GeneratedParser;
use strict;
use warnings;

# Compiled regex patterns for speed
my \%REGEXES = (
$regex_definitions
);

# Runtime helper functions
sub quantified_match {
    my (\$input, \$regex, \$min, \$max) = \@_;
    my \$count = 0;
    my \$pos = pos(\$\$input);
    
    while (\$count < \$max && \$\$input =~ /\\G\$regex/gc) {
        \$count++;
    }
    
    if (\$count >= \$min) {
        return \$count;
    } else {
        # Restore position on failure
        pos(\$\$input) = \$pos;
        return undef;
    }
}

sub quantified_rule {
    my (\$input, \$rule_ref, \$min, \$max) = \@_;
    my \$count = 0;
    my \$pos = pos(\$\$input);
    my \@results = ();
    
    while (\$count < \$max) {
        my \$result = \$rule_ref->(\$input);
        if (defined \$result) {
            push \@results, \$result;
            \$count++;
        } else {
            last;
        }
    }
    
    if (\$count >= \$min) {
        return \\\@results;
    } else {
        # Restore position on failure
        pos(\$\$input) = \$pos;
        return undef;
    }
}

sub collect_quantified_results {
    # Helper function to collect results from quantified elements
    my (\$element_num, \$results_ref) = \@_;
    my \$element_index = \$element_num - 1;
    
    # If the element is a quantified result (array), return it
    # If it's undef (zero matches), return empty array
    # Otherwise return single element in array
    my \$element = \$results_ref->[\$element_index];
    
    if (!defined \$element) {
        return [];  # Zero matches
    } elsif (ref(\$element) eq 'ARRAY') {
        return \$element;  # Already an array from quantifier
    } else {
        return [\$element];  # Single element, wrap in array
    }
}

# Fast parsing subroutines
$subroutines

# Main entry point
sub parse {
    my (\$input_ref) = \@_;
    pos(\$\$input_ref) = 0;
    my \$result = parse_$main_rule(\$input_ref);
    
    # Check that entire input was consumed
    if (defined \$result && pos(\$\$input_ref) == length(\$\$input_ref)) {
        return \$result;
    } else {
        return undef;  # Partial match or unconsumed input
    }
}

1;
EOF
    
    return $module;
}

sub generate_fast_parser_sub {
    my ($rule_name, $rule_def) = @_;
    my $type = $rule_def->{type};
    my @regexes = ();
    
    if ($type eq 'or') {
        return generate_or_parser($rule_name, $rule_def, \@regexes);
    } elsif ($type eq 'sequence') {
        return generate_sequence_parser($rule_name, $rule_def, \@regexes);
    } elsif ($type eq 'atom') {
        return generate_atom_parser($rule_name, $rule_def, \@regexes);
    }
}

sub generate_or_parser {
    my ($rule_name, $rule_def, $regexes) = @_;
    
    # Check if all alternatives are pure literals (optimization opportunity)
    my @literal_alternatives = ();
    my $all_literals = 1;
    
    foreach my $alt (@{$rule_def->{alternatives}}) {
        if ($alt->{type} eq 'atom' && is_terminal($alt->{value})) {
            if (ref($alt->{value}) eq 'ARRAY' && $alt->{value}->[0] eq 'epsilon') {
                # Epsilon production breaks pure literal optimization
                $all_literals = 0;
                last;
            } elsif (ref($alt->{value}) eq 'ARRAY' && $alt->{value}->[0] eq 'regex') {
                # Regex patterns break pure literal optimization (for now)
                $all_literals = 0;
                last;
            } else {
                # This is a literal alternative
                push @literal_alternatives, $alt->{value}[1];
            }
        } elsif ($alt->{type} eq 'sequence') {
            # Check if sequence contains only literals
            my @seq_literals = ();
            my $seq_is_all_literals = 1;
            foreach my $element (@{$alt->{elements}}) {
                if ($element->{type} eq 'atom' && is_terminal($element->{value})) {
                    push @seq_literals, $element->{value}[1];
                } else {
                    $seq_is_all_literals = 0;
                    last;
                }
            }
            if ($seq_is_all_literals) {
                # Sequence of literals - combine into single literal
                push @literal_alternatives, join('', @seq_literals);
            } else {
                $all_literals = 0;
                last;
            }
        } else {
            # Non-literal alternative
            $all_literals = 0;
            last;
        }
    }
    
    if ($all_literals && @literal_alternatives > 0) {
        # Generate single optimized OR regex for all literal alternatives
        my $regex_pattern = join('|', map { "\\Q$_\\E" } @literal_alternatives);
        my $regex_name = $rule_name;
        push @$regexes, "    '$regex_name' => qr/$regex_pattern/o";
        
        my $sub_code = "sub parse_$rule_name {\n" .
                       "    my (\$input) = \@_;\n" .
                       "    return 1 if \$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc;\n" .
                       "    return undef;\n" .
                       "}";
        return ($sub_code, $regexes);
    }
    
    # Fall back to individual alternative processing
    my @alternatives = ();
    foreach my $alt (@{$rule_def->{alternatives}}) {
        if ($alt->{type} eq 'atom') {
            if (is_terminal($alt->{value})) {
                if (ref($alt->{value}) eq 'ARRAY' && $alt->{value}->[0] eq 'epsilon') {
                    # Epsilon production - always succeeds
                    push @alternatives, "1";  # Always succeeds
                } elsif (ref($alt->{value}) eq 'ARRAY' && $alt->{value}->[0] eq 'regex') {
                    # Regex pattern - use regex directly
                    my $pattern = $alt->{value}[1];
                    my $regex_name = "${rule_name}_alt" . @alternatives;
                    push @$regexes, "    '$regex_name' => qr/$pattern/o";
                    push @alternatives, "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
                } else {
                    # For terminal atoms, generate literal match
                    my $literal = $alt->{value}[1];
                    my $regex_name = "${rule_name}_alt" . @alternatives;
                    push @$regexes, "    '$regex_name' => qr/\\Q$literal\\E/o";
                    push @alternatives, "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
                }
            } else {
                # For non-terminal atoms, call parser function
                push @alternatives, "parse_$alt->{value}(\$input)";
            }
        } elsif ($alt->{type} eq 'sequence') {
            # For sequences, we need to generate inline matching code
            my @seq_steps = ();
            my $alt_num = @alternatives;
            foreach my $element (@{$alt->{elements}}) {
                if ($element->{type} eq 'atom') {
                    if (is_terminal($element->{value})) {
                        if (ref($element->{value}) eq 'ARRAY' && $element->{value}->[0] eq 'regex') {
                            # Regex pattern
                            my $pattern = $element->{value}[1];
                            my $step_regex = "${rule_name}_alt${alt_num}_" . @seq_steps;
                            push @$regexes, "    '$step_regex' => qr/$pattern/o";
                            push @seq_steps, "\$\$input =~ /\\G\$REGEXES{'$step_regex'}/gc";
                        } else {
                            # Literal terminal
                            my $literal = $element->{value}[1];
                            my $step_regex = "${rule_name}_alt${alt_num}_" . @seq_steps;
                            push @$regexes, "    '$step_regex' => qr/\\Q$literal\\E/o";
                            push @seq_steps, "\$\$input =~ /\\G\$REGEXES{'$step_regex'}/gc";
                        }
                    } else {
                        push @seq_steps, "parse_$element->{value}(\$input)";
                    }
                } elsif ($element->{type} eq 'quantified') {
                    # Handle quantified elements in sequences
                    my $quant_code = generate_quantified_code($element, "${rule_name}_alt${alt_num}", @seq_steps, $regexes);
                    push @seq_steps, $quant_code;
                }
            }
            # Join sequence steps with proper backtracking
            my $sequence_code = "do { my \$seq_pos = pos(\$\$input); ";
            for my $step (@seq_steps) {
                $sequence_code .= "($step) && ";
            }
            $sequence_code .= "1 || (pos(\$\$input) = \$seq_pos, 0) }";
            push @alternatives, $sequence_code;
        }
    }
    
    # Alternatives array is now properly populated
    
    # Generate the alternative checking code 
    # Use defined() only for function calls, not for regex/boolean expressions
    my @alt_lines = ();
    foreach my $alt (@alternatives) {
        next unless defined $alt && $alt ne "";
        if ($alt =~ /^parse_\w+\(/) {
            # Function call - use defined() to handle '0' return values
            push @alt_lines, "    if (defined(my \$alt_result = $alt)) { return \$alt_result; }";
        } else {
            # Regex or boolean expression - use truthiness
            push @alt_lines, "    if (my \$alt_result = $alt) { return \$alt_result; }";
        }
    }
    my $alt_code = join("\n", @alt_lines);
    
    my $sub_code = "sub parse_$rule_name {\n" .
                   "    my (\$input) = \@_;\n" .
                   "    my \$start_pos = pos(\$\$input);\n" .
                   "    \n" .
                   "    # Try alternatives in order (fast backtracking)\n" .
                   $alt_code . "\n" .
                   "    \n" .
                   "    # No match - restore position\n" .
                   "    pos(\$\$input) = \$start_pos;\n" .
                   "    return undef;\n" .
                   "}";
    
    return ($sub_code, $regexes);
}

sub generate_sequence_parser {
    my ($rule_name, $rule_def, $regexes) = @_;
    
    my @sequence_steps = ();
    my $return_annotation = $rule_def->{return_annotation};
    
    # Use the elements directly (return annotation already extracted in Step 5)
    my @filtered_elements = @{$rule_def->{elements}};
    
    # Check if this is a pure literal sequence that can be optimized
    my @literal_parts = ();
    my $can_optimize = 1;
    
    foreach my $element (@filtered_elements) {
        if ($element->{type} eq 'atom' && is_terminal($element->{value})) {
            if (ref($element->{value}) eq 'ARRAY' && $element->{value}->[0] eq 'regex') {
                # Regex pattern - breaks literal optimization
                $can_optimize = 0;
                last;
            } else {
                # Terminal literal - can be part of optimized regex
                push @literal_parts, $element->{value}[1];
            }
        } elsif ($element->{type} eq 'atom' && !is_terminal($element->{value})) {
            # Rule reference - breaks optimization
            $can_optimize = 0;
            last;
        } elsif ($element->{type} eq 'quantified') {
            # Quantifier - breaks optimization  
            $can_optimize = 0;
            last;
        } else {
            # Unknown type - breaks optimization
            $can_optimize = 0;
            last;
        }
    }
    
    if ($can_optimize && @literal_parts > 0) {
        # Generate single optimized regex for entire literal sequence
        my $combined_literal = join('', @literal_parts);
        my $regex_name = $rule_name;
        push @$regexes, "    '$regex_name' => qr/\\Q$combined_literal\\E/o";
        push @sequence_steps, "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
    } else {
        # Fall back to individual element processing
        my $step_num = 0;
        foreach my $element (@filtered_elements) {
            $step_num++;
            if ($element->{type} eq 'atom') {
                if (is_terminal($element->{value})) {
                    if (ref($element->{value}) eq 'ARRAY' && $element->{value}->[0] eq 'regex') {
                        # Direct regex match for regex pattern
                        my $pattern = $element->{value}[1];  # Extract regex pattern
                        my $regex_name = "${rule_name}_step${step_num}";
                        push @$regexes, "    '$regex_name' => qr/$pattern/o";
                        push @sequence_steps, "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
                    } else {
                        # Direct regex match for terminal literal
                        my $literal = $element->{value}[1];  # Extract terminal content
                        my $regex_name = "${rule_name}_step${step_num}";
                        push @$regexes, "    '$regex_name' => qr/\\Q$literal\\E/o";  # Escape literal
                        push @sequence_steps, "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
                    }
                } else {
                    # Rule call
                    push @sequence_steps, "parse_$element->{value}(\$input)";
                }
            } elsif ($element->{type} eq 'quantified') {
                # Generate quantified parsing code
                my $quant_code = generate_quantified_code($element, $rule_name, $step_num, $regexes);
                push @sequence_steps, $quant_code;
            }
        }
    }
    
    # Sequence steps array is now properly populated
    
    # Generate the sequence checking code with result capture
    my @seq_lines = ();
    my $step_counter = 0;
    foreach my $step (@sequence_steps) {
        next unless defined $step && $step ne "";
        $step_counter++;
        
        if ($step =~ /^parse_(\w+)\(/) {
            # Function call - capture result
            push @seq_lines, "    my \$result_$step_counter = $step;";
            push @seq_lines, "    unless (defined \$result_$step_counter) {";
            push @seq_lines, "        pos(\$\$input) = \$start_pos;";
            push @seq_lines, "        return undef;";
            push @seq_lines, "    }";
            push @seq_lines, "    push \@results, \$result_$step_counter;";
        } elsif ($step =~ /^quantified_rule\(/) {
            # Quantified rule call - capture array result
            push @seq_lines, "    my \$result_$step_counter = $step;";
            push @seq_lines, "    unless (defined \$result_$step_counter) {";
            push @seq_lines, "        pos(\$\$input) = \$start_pos;";
            push @seq_lines, "        return undef;";
            push @seq_lines, "    }";
            push @seq_lines, "    push \@results, \$result_$step_counter;";
        } else {
            # Regex match - just check success
            push @seq_lines, "    unless ($step) {";
            push @seq_lines, "        pos(\$\$input) = \$start_pos;";
            push @seq_lines, "        return undef;";
            push @seq_lines, "    }";
            push @seq_lines, "    push \@results, 1;  # Terminal match success";
        }
    }
    my $seq_code = join("\n", @seq_lines);
    
    # Generate return code based on annotation
    my $return_code;
    if ($return_annotation) {
        $return_code = generate_return_code($return_annotation, \@filtered_elements);
    } else {
        $return_code = "return \\\@results;";
    }
    
    my $sub_code = <<"EOF";
sub parse_$rule_name {
    my (\$input) = \@_;
    my \$start_pos = pos(\$\$input);
    my \@results = ();
    
    # Parse sequence elements in order
$seq_code
    
    $return_code
}
EOF
    
    return ($sub_code, $regexes);
}

sub generate_return_code {
    my ($return_annotation, $filtered_elements) = @_;
    my ($type, $annotation) = @$return_annotation;
    
    if ($type eq 'return_scalar') {
        # Handle $1, $2, etc.
        my $var_num = $annotation;
        $var_num =~ s/\$//;  # Remove $ sign
        my $element_value = element_value($var_num, $filtered_elements);
        return "return $element_value;";
    } elsif ($type eq 'return_array') {
        # Handle [$1, $3, etc.] and [$1*] collection syntax
        my $array_content = $annotation;
        $array_content =~ s/^\[|\]$//g;  # Remove brackets
        
        # Check if this is a collection pattern like $1*
        if ($array_content =~ /^\s*\$(\d+)\*\s*$/) {
            # Single quantifier collection: [$1*]
            my $element_num = $1;
            return "return collect_quantified_results($element_num, \\\@results);";
        } else {
            # Mixed array: [$1, $3, $2*] - handle both regular and quantified elements
            my $perl_array = $array_content;
            # Handle quantified elements: $N* -> collect_quantified_results(N, \@results)
            $perl_array =~ s/\$(\d+)\*/collect_quantified_results($1, \\\@results)/g;
            # Handle regular elements: $N -> $results[N-1]  
            $perl_array =~ s/\$(\d+)/element_value($1, $filtered_elements)/ge;
            return "return [$perl_array];";
        }
    } elsif ($type eq 'return_object') {
        # Handle {key: $1, value: $3} and {items: [$1*]}
        my $object_content = $annotation;
        $object_content =~ s/^\{|\}$//g;  # Remove braces
        
        # Convert to Perl hash syntax and substitute $N references
        my $perl_hash = $object_content;
        $perl_hash =~ s/(\w+):\s*/"$1" => /g;  # key: -> "key" =>
        
        # Handle collection patterns like [$1*] within object values
        $perl_hash =~ s/\[\s*\$(\d+)\*\s*\]/collect_quantified_results($1, \\\@results)/g;
        
        # Handle regular $N references
        $perl_hash =~ s/\$(\d+)/element_value($1, $filtered_elements)/ge;
        
        return "return {$perl_hash};";
    }
    
    return "return \\\@results;  # Fallback";
}

sub element_value {
    my ($var_num, $filtered_elements) = @_;
    my $element_index = $var_num - 1;  # Convert to 0-based
    
    if ($element_index >= 0 && $element_index < @$filtered_elements) {
        return "\$results[$element_index]";  # Reference captured result
    } else {
        return "undef";  # Invalid index
    }
}



sub generate_atom_parser {
    my ($rule_name, $rule_def, $regexes) = @_;
    my $value = $rule_def->{value};
    
    if (is_terminal($value)) {
        if (ref($value) eq 'ARRAY' && $value->[0] eq 'epsilon') {
            # Epsilon production - always succeeds without consuming input
            my $sub_code = <<"EOF";
sub parse_$rule_name {
    my (\$input) = \@_;
    return [];  # Epsilon - empty match, always succeeds
}
EOF
            return ($sub_code, []);
        } elsif (ref($value) eq 'ARRAY' && $value->[0] eq 'regex') {
            # Regex pattern - use the regex directly
            my $pattern = $value->[1];  # Extract regex pattern
            push @$regexes, "    '$rule_name' => qr/$pattern/o";
            my $sub_code = <<"EOF";
sub parse_$rule_name {
    my (\$input) = \@_;
    return 1 if \$\$input =~ /\\G\$REGEXES{'$rule_name'}/gc;
    return undef;
}
EOF
            return ($sub_code, $regexes);
        } else {
            # Regular terminal (quoted string)
            my $literal = $value->[1];  # Extract terminal content
            push @$regexes, "    '$rule_name' => qr/\\Q$literal\\E/o";
            my $sub_code = <<"EOF";
sub parse_$rule_name {
    my (\$input) = \@_;
    return 1 if \$\$input =~ /\\G\$REGEXES{'$rule_name'}/gc;
    return undef;
}
EOF
            return ($sub_code, $regexes);
        }
    } else {
        my $sub_code = <<"EOF";
sub parse_$rule_name {
    my (\$input) = \@_;
    return parse_$value(\$input);
}
EOF
        return ($sub_code, []);
    }
}

sub generate_quantified_code {
    my ($element, $rule_name, $step_num, $regexes) = @_;
    my $quant = parse_quantifier($element->{quantifier});
    
    if (is_terminal($element->{element})) {
        if (ref($element->{element}) eq 'ARRAY' && $element->{element}->[0] eq 'regex') {
            # Regex pattern
            my $pattern = $element->{element}[1];
            my $regex_name = "${rule_name}_quant${step_num}";
            push @$regexes, "    '$regex_name' => qr/$pattern/o";
            return "quantified_match(\$input, \$REGEXES{'$regex_name'}, $quant->{min}, $quant->{max})";
        } else {
            # Literal terminal
            my $literal = $element->{element}[1];  # Extract terminal content
            my $regex_name = "${rule_name}_quant${step_num}";
            push @$regexes, "    '$regex_name' => qr/\\Q$literal\\E/o";
            return "quantified_match(\$input, \$REGEXES{'$regex_name'}, $quant->{min}, $quant->{max})";
        }
    } else {
        return "quantified_rule(\$input, \\&parse_$element->{element}, $quant->{min}, $quant->{max})";
    }
}

sub generate_rule_code {
    my ($rule_name, $rule_def) = @_;
    my $type = $rule_def->{type};
    
    if ($type eq 'or') {
        return generate_or_rule($rule_name, $rule_def);
    } elsif ($type eq 'sequence') {
        return generate_sequence_rule($rule_name, $rule_def);
    } elsif ($type eq 'atom') {
        return generate_atom_rule($rule_name, $rule_def);
    } else {
        die "Unknown rule type: $type";
    }
}

sub generate_or_rule {
    my ($rule_name, $rule_def) = @_;
    my @actions = ();
    my $alt_num = 0;
    
    foreach my $alternative (@{$rule_def->{alternatives}}) {
        $alt_num++;
        if ($alternative->{type} eq 'atom') {
            push @actions, "-> $alternative->{value}";
        } elsif ($alternative->{type} eq 'sequence') {
            # For sequences in OR, we need to call sub-rules in order
            my @seq_calls = ();
            foreach my $element (@{$alternative->{elements}}) {
                if ($element->{type} eq 'atom') {
                    push @seq_calls, $element->{value};
                } elsif ($element->{type} eq 'quantified') {
                    push @seq_calls, "$element->{element}\\{$element->{quantifier}\\}";
                }
            }
            # Create a synthetic rule for this sequence
            my $seq_rule = "${rule_name}_alt${alt_num}";
            push @actions, "-> $seq_rule";
        }
    }
    
    my $rule_header = "$rule_name:";
    return $rule_header . "\n" . join("\n", @actions);
}

sub generate_sequence_rule {
    my ($rule_name, $rule_def) = @_;
    my @regex_patterns = ();
    my @actions = ();
    
    foreach my $element (@{$rule_def->{elements}}) {
        if ($element->{type} eq 'atom') {
            if (is_terminal($element->{value})) {
                # Terminal - generate regex pattern
                push @regex_patterns, "/$element->{value}/";
            } else {
                # Non-terminal - generate action
                push @actions, "-> $element->{value}";
            }
        } elsif ($element->{type} eq 'quantified') {
            # Handle quantified elements
            my $quant = parse_quantifier($element->{quantifier});
            if (is_terminal($element->{element})) {
                push @regex_patterns, "/$element->{element}\\{$quant->{min},$quant->{max}\\}/";
            } else {
                # Quantified non-terminal - this is tricky!
                push @actions, "-> $element->{element}  # TODO: quantifier $element->{quantifier}";
            }
        }
    }
    
    my $rule_header = "$rule_name:";
    $rule_header .= " " . join(" ", @regex_patterns) if @regex_patterns;
    
    my $result = $rule_header;
    $result .= "\n" . join("\n", @actions) if @actions;
    
    return $result;
}

sub generate_atom_rule {
    my ($rule_name, $rule_def) = @_;
    my $value = $rule_def->{value};
    
    if (is_terminal($value)) {
        return "$rule_name: /$value/";
    } else {
        return "$rule_name:\n-> $value";
    }
}

sub is_terminal {
    my ($value) = @_;
    # Check if value is explicitly marked as terminal, regex, or epsilon
    return ref($value) eq 'ARRAY' && ($value->[0] eq 'terminal' || $value->[0] eq 'regex' || $value->[0] eq 'epsilon');
}

sub is_return_annotation {
    my ($value) = @_;
    # Check if value is a return annotation (rule metadata, not grammar symbol)
    return ref($value) eq 'ARRAY' && ($value->[0] eq 'return_scalar' || $value->[0] eq 'return_array' || $value->[0] eq 'return_object');
}

sub parse_quantifier {
    my ($quant_str) = @_;
    if ($quant_str =~ /^(\d+),(\d+)$/) {
        return {min => $1, max => $2};
    } elsif ($quant_str =~ /^(\d+),$/) {
        return {min => $1, max => 999};
    } elsif ($quant_str =~ /^,(\d+)$/) {
        return {min => 0, max => $1};
    } elsif ($quant_str eq '+') {
        return {min => 1, max => 999};  # one or more
    } elsif ($quant_str eq '*') {
        return {min => 0, max => 999};  # zero or more
    } elsif ($quant_str eq '?') {
        return {min => 0, max => 1};    # zero or one
    } else {
        return {min => 1, max => 1};
    }
}

# Left-recursion elimination algorithm
sub eliminate_left_recursion {
    my ($grammar_tree, $rule_order) = @_;
    
    print "\n=== Eliminating Left-Recursion ===\n" unless $quiet_mode;
    
    my %new_grammar = %$grammar_tree;  # Copy original grammar
    my @new_order = @$rule_order;      # Copy original order
    
    # Step 1: Detect left-recursive rules
    my @left_recursive_rules = ();
    
    foreach my $rule_name (keys %new_grammar) {
        if (is_left_recursive($rule_name, $new_grammar{$rule_name})) {
            push @left_recursive_rules, $rule_name;
            print "Found left-recursive rule: $rule_name\n" unless $quiet_mode;
        }
    }
    
    # Step 2: Transform each left-recursive rule
    foreach my $rule_name (@left_recursive_rules) {
        my $rule = $new_grammar{$rule_name};
        
        if ($rule->{type} eq 'or') {
            # Split alternatives into left-recursive and non-left-recursive
            my @left_recursive_alts = ();
            my @non_left_recursive_alts = ();
            
            foreach my $alt (@{$rule->{alternatives}}) {
                if (starts_with_rule($alt, $rule_name)) {
                    push @left_recursive_alts, $alt;
                } else {
                    push @non_left_recursive_alts, $alt;
                }
            }
            
            if (@left_recursive_alts > 0 && @non_left_recursive_alts > 0) {
                # Classic transformation: A := A α | β  ->  A := β A'  and  A' := α A' | ε
                my $tail_rule_name = "${rule_name}_tail";
                
                # Transform main rule: A := β A'
                my @main_alternatives = ();
                foreach my $non_left_alt (@non_left_recursive_alts) {
                    # Add A' call to end of each non-left-recursive alternative
                    my $new_alt = add_tail_call($non_left_alt, $tail_rule_name);
                    push @main_alternatives, $new_alt;
                }
                
                $new_grammar{$rule_name} = {
                    type => (@main_alternatives == 1) ? $main_alternatives[0]->{type} : 'or',
                    (@main_alternatives == 1) ? %{$main_alternatives[0]} : (alternatives => \@main_alternatives)
                };
                
                # Create tail rule: A' := α A' | ε
                my @tail_alternatives = ();
                foreach my $left_alt (@left_recursive_alts) {
                    # Remove the left-recursive reference and add tail call
                    my $alpha = remove_left_recursion($left_alt, $rule_name);
                    my $tail_alt = add_tail_call($alpha, $tail_rule_name);
                    push @tail_alternatives, $tail_alt;
                }
                
                # Add epsilon (empty) alternative
                push @tail_alternatives, {
                    type => 'atom',
                    value => ['epsilon']  # Special marker for empty production
                };
                
                $new_grammar{$tail_rule_name} = {
                    type => 'or',
                    alternatives => \@tail_alternatives
                };
                
                # Add tail rule to order (after main rule)
                my $main_rule_index = 0;
                for my $i (0..$#new_order) {
                    if ($new_order[$i] eq $rule_name) {
                        $main_rule_index = $i;
                        last;
                    }
                }
                splice @new_order, $main_rule_index + 1, 0, $tail_rule_name;
                
                print "Transformed $rule_name -> $rule_name + $tail_rule_name\n" unless $quiet_mode;
            }
        }
    }
    
    return (\%new_grammar, \@new_order);
}

# Check if a rule is left-recursive (directly)
sub is_left_recursive {
    my ($rule_name, $rule) = @_;
    
    if ($rule->{type} eq 'or') {
        # Check if any alternative starts with the rule itself
        foreach my $alt (@{$rule->{alternatives}}) {
            if (starts_with_rule($alt, $rule_name)) {
                return 1;
            }
        }
    } elsif ($rule->{type} eq 'sequence') {
        # Check if sequence starts with the rule itself
        return starts_with_rule($rule, $rule_name);
    } elsif ($rule->{type} eq 'atom') {
        # Check if atom is the rule itself
        return !is_terminal($rule->{value}) && $rule->{value} eq $rule_name;
    }
    
    return 0;
}

# Check if an alternative/sequence starts with a specific rule
sub starts_with_rule {
    my ($alt, $rule_name) = @_;
    
    if ($alt->{type} eq 'sequence') {
        # Check first element of sequence
        my $first_element = $alt->{elements}->[0];
        if ($first_element && $first_element->{type} eq 'atom' && 
            !is_terminal($first_element->{value}) && 
            $first_element->{value} eq $rule_name) {
            return 1;
        }
    } elsif ($alt->{type} eq 'atom') {
        # Check if atom is the rule
        return !is_terminal($alt->{value}) && $alt->{value} eq $rule_name;
    }
    
    return 0;
}

# Add a tail rule call to the end of an alternative
sub add_tail_call {
    my ($alt, $tail_rule_name) = @_;
    
    if ($alt->{type} eq 'sequence') {
        # Add tail call to end of sequence
        my @new_elements = @{$alt->{elements}};
        push @new_elements, {
            type => 'atom',
            value => $tail_rule_name
        };
        return {
            type => 'sequence',
            elements => \@new_elements
        };
    } elsif ($alt->{type} eq 'atom') {
        # Convert atom to sequence with tail call
        return {
            type => 'sequence',
            elements => [
                $alt,
                {
                    type => 'atom',
                    value => $tail_rule_name
                }
            ]
        };
    }
    
    return $alt;  # Fallback
}

# Remove left-recursive reference from alternative (get α from A α)
sub remove_left_recursion {
    my ($alt, $rule_name) = @_;
    
    if ($alt->{type} eq 'sequence') {
        my @elements = @{$alt->{elements}};
        # Remove first element (should be the left-recursive reference)
        if (@elements > 1 && 
            $elements[0]->{type} eq 'atom' && 
            !is_terminal($elements[0]->{value}) && 
            $elements[0]->{value} eq $rule_name) {
            
            shift @elements;  # Remove first element
            
            if (@elements == 1) {
                return $elements[0];  # Single element - return as atom
            } else {
                return {
                    type => 'sequence',
                    elements => \@elements
                };
            }
        }
    }
    
    # Fallback - return empty (this shouldn't happen in well-formed left recursion)
    return {
        type => 'atom',
        value => ['epsilon']
    };
}

print "\n=== Step 5: Build tree structure ===\n" unless $quiet_mode;
my ($step5_result, $rule_order) = step5_build_tree_structure($step4_result);
print "STEP 5 RESULT (Tree structure):\n" . Dumper($step5_result) unless $quiet_mode;
print "RULE ORDER: " . join(", ", @$rule_order) . "\n" unless $quiet_mode;

print "\n=== Step 6: Generate parser code ===\n" unless $quiet_mode;
my $step6_result = step6_generate_parser_code($step5_result, $rule_order);
print $step6_result;
