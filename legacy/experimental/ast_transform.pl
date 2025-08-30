#!/usr/bin/env perl
use strict;
use warnings;
use Data::Dumper;
use lib 'fx/perl';
use lib 'perl';

# AST Transformation Pipeline for := grammar rules
# Takes raw token AST and transforms it into structured parse tree

# Helper function to extract values from structured tokens
sub extract_token_value {
    my ($token) = @_;
    if (ref($token) eq 'ARRAY' && @$token == 2) {
        # Structured token like ['rule', 'name'] or ['quoted_string', 'value']
        return $token->[1];
    } else {
        # Legacy format or already extracted
        return $token;
    }
}

# Helper function to safely escape literals for regex
sub escape_regex_literal {
    my ($literal) = @_;
    # Handle special case of $ which can cause interpolation issues
    if ($literal eq '$') {
        return '\\$';  # Double escape for dollar sign
    } else {
        return "\\Q$literal\\E";  # Standard quotemeta escaping
    }
}

sub step2_group_by_or {
    my ($raw_ast) = @_;
    my @transformed_rules;
    
    foreach my $rule_tokens (@$raw_ast) {
        my ($rule_name_token, @tokens) = @$rule_tokens;
        my $rule_name = extract_token_value($rule_name_token);
        
        # Split tokens on | operators
        my @or_groups = ();
        my @current_group = ();
        
        foreach my $token (@tokens) {
            if (ref($token) eq 'ARRAY' && $token->[0] eq 'operator' && $token->[1] eq '|') {
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
my $ebnf_file;

# Parse command line arguments  
for my $arg (@ARGV) {
    if ($arg eq '--quiet') {
        $quiet_mode = 1;
    } else {
        $ebnf_file = $arg;
    }
}

# Ensure we have an input file
unless ($ebnf_file) {
    die "Usage: $0 [--quiet] <ebnf_file>\n";
}

open my $fh2, "<", $ebnf_file or die "Cannot open $ebnf_file: $!";
my $input_content = do { local $/; <$fh2> };
close $fh2;
print STDERR "=== Parsing $ebnf_file ===\n" unless $quiet_mode;
my $parser = LinkedSpec::Get(\$spec_content);
my $raw_ast = $parser->(\$input_content);
print STDERR "RAW AST from EBNF parser:\n" . Dumper($raw_ast) unless $quiet_mode;

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

sub is_group_open {
    my ($token) = @_;
    return (ref($token) eq 'ARRAY' && $token->[0] eq 'group_open') || $token eq '(';
}

sub is_group_close {
    my ($token) = @_;
    return (ref($token) eq 'ARRAY' && $token->[0] eq 'group_close') || $token eq ')';
}

sub process_parentheses_in_sequence {
    my ($sequence) = @_;
    return $sequence unless ref($sequence) eq 'ARRAY';
    
    my @result = ();
    my @stack = ();
    my $depth = 0;
    
    for my $token (@$sequence) {
        if (is_group_open($token)) {
            $depth++;
            push @stack, [];
        } elsif (is_group_close($token)) {
            $depth--;
            if (@stack) {
                my $group_content = pop @stack;
                if ($depth == 0) {
                    # Top-level group - check if it's a grouped alternative
                    if (is_grouped_alternative($group_content)) {
                        push @result, ['GROUPED_ALTERNATIVE', $group_content];
                    } else {
                        push @result, ['GROUPED', $group_content];
                    }
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

# Check if a group contains alternatives (pipe operators)
sub is_grouped_alternative {
    my ($group_content) = @_;
    
    for my $element (@$group_content) {
        if (ref($element) eq 'ARRAY' && @$element >= 2 && $element->[0] eq 'operator' && $element->[1] eq '|') {
            return 1;
        }
    }
    return 0;
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

print STDERR "\n=== Step 2: Group by OR operators ===\n" unless $quiet_mode;
my $step2_result = step2_group_by_or($raw_ast);
print STDERR "STEP 2 RESULT (Grouped by OR):\n" . Dumper($step2_result) unless $quiet_mode;

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
            
            # Attach quantifier to previous element - extract operator string if it's an array
            my $quantifier_str = (ref($token) eq 'ARRAY' && $token->[0] eq 'operator') ? $token->[1] : $token;
            
            # Check if the previous element is a GROUPED construct
            if (ref($prev_element) eq 'ARRAY' && $prev_element->[0] eq 'GROUPED') {
                push @result, ['QUANTIFIED_GROUP', $prev_element->[1], $quantifier_str];
            } else {
                push @result, ['QUANTIFIED', $prev_element, $quantifier_str];
            }
        } else {
            push @result, $token;
        }
    }
    
    return \@result;
}

sub is_quantifier {
    my ($token) = @_;
    # Handle both simple strings and operator arrays
    if (ref($token) eq 'ARRAY' && $token->[0] eq 'operator') {
        my $op = $token->[1];
        return $op =~ /^[\+\*\?]$/ || $op =~ /^[\d,]+$/;
    } else {
        return $token =~ /^[\d,]+$/ ||  # Matches patterns like "1,3", "1,", "0,3"
               $token =~ /^[\+\*\?]$/;  # Matches +, *, ?
    }
}

print STDERR "\n=== Step 2.5: Handle parentheses grouping ===\n" unless $quiet_mode;
my $step2_5_result = step2_5_handle_parentheses($step2_result);
print STDERR "STEP 2.5 RESULT (Parentheses handled):\n" . Dumper($step2_5_result) unless $quiet_mode;

print STDERR "\n=== Step 3: Parse sequences ===\n" unless $quiet_mode;
my $step3_result = step3_parse_sequences($step2_5_result);
print STDERR "STEP 3 RESULT (Sequences parsed):\n" . Dumper($step3_result) unless $quiet_mode;

sub step5_build_tree_structure {
    my ($quantified_ast) = @_;
    my %grammar_tree = ();
    my @rule_order = ();
    
    foreach my $rule (@$quantified_ast) {
        my ($rule_name, $type, $data, @extra_elements) = @$rule;
        
        # Add to rule order only on first occurrence
        push @rule_order, $rule_name unless exists $grammar_tree{$rule_name};
        
        # Check for return annotations in data elements (they moved during earlier steps)
        my $rule_return_annotation;
        if (ref($data) eq 'ARRAY') {
            foreach my $element (@$data) {
                if (ref($element) eq 'ARRAY' && is_return_annotation($element)) {
                    $rule_return_annotation = $element;
                    last;
                }
            }
        }
        
        # Convert current rule to a standard alternative structure
        my $new_alternative;
        if ($type eq 'OR') {
            # OR node with multiple alternatives - flatten them
            my @alternatives = ();
            foreach my $alternative (@$data) {
                my ($alt_type, $alt_data) = @$alternative;
                if ($alt_type eq 'SEQUENCE') {
                    my ($elements, $return_annotation) = build_sequence_elements($alt_data);
                    push @alternatives, {
                        type => 'sequence',
                        elements => $elements,
                        return_annotation => $return_annotation
                    };
                } elsif ($alt_type eq 'ATOM') {
                    # For atoms, check if there are additional elements like return annotations
                    my $atom_alternative = {
                        type => 'atom',
                        value => $alt_data
                    };
                    
                    # Check if there are more elements after the atom (return annotations)
                    if (@$alternative > 2) {
                        # There are additional elements - check for return annotations
                        for my $i (2..$#$alternative) {
                            my $extra_element = $alternative->[$i];
                            if (ref($extra_element) eq 'ARRAY' && is_return_annotation($extra_element)) {
                                $atom_alternative->{return_annotation} = $extra_element;
                                last; # Only take the first return annotation
                            }
                        }
                    }
                    
                    push @alternatives, $atom_alternative;
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
            # Add return annotation if present
            if ($rule_return_annotation) {
                $new_alternative->{return_annotation} = $rule_return_annotation;
            }
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
        print STDERR "Grammar validation errors:\n" . join("\n", map { "  $_" } @errors) . "\n";
        exit 1;
    }
}

sub collect_referenced_rules {
    my ($node, $referenced) = @_;
    
    if (ref($node) eq 'HASH') {
        if ($node->{type} eq 'atom') {
            my $value = $node->{value};
            
            # Only plain strings that are valid rule names should be considered rule references
            if (!ref($value) && $value =~ /^[a-zA-Z_][a-zA-Z0-9_]*$/) {
                $referenced->{$value} = 1;
            }
            # Skip all array references (terminals, operators, return annotations, etc.)
            # Skip strings that don't match rule name pattern (return annotation content, etc.)
            
        } elsif ($node->{type} eq 'sequence' || $node->{type} eq 'or') {
            # Recursively check elements/alternatives - but skip return_annotation fields
            for my $element (@{$node->{elements} || $node->{alternatives} || []}) {
                collect_referenced_rules($element, $referenced);
            }
        } elsif ($node->{type} eq 'quantified') {
            # Recursively check the quantified element
            collect_referenced_rules($node->{element}, $referenced);
        }
        
        # IMPORTANT: Never traverse into return_annotation fields
        # They contain metadata, not grammar structure to be validated
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
        } elsif (ref($element) eq 'ARRAY' && $element->[0] eq 'quantified_element') {
            # Quantified element from left-recursion eliminator
            my ($type, $rule_name, $quantifier) = @$element;
            push @result, {
                type => 'quantified',
                element => $rule_name,
                quantifier => $quantifier
            };
        } elsif (ref($element) eq 'ARRAY' && $element->[0] eq 'quantified_group') {
            # Quantified group from left-recursion eliminator: (pattern)*
            my ($type, $group_content, $quantifier) = @$element;
            push @result, {
                type => 'quantified_group',
                group_content => $group_content,
                quantifier => $quantifier
            };
        } elsif (ref($element) eq 'ARRAY' && $element->[0] eq 'QUANTIFIED_GROUP') {
            # Quantified group from Step 4: (pattern)*
            my ($type, $group_content, $quantifier) = @$element;
            push @result, {
                type => 'quantified_group',
                group_content => $group_content,
                quantifier => $quantifier
            };
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

print STDERR "\n=== Step 4: Handle quantifiers ===\n" unless $quiet_mode;
my $step4_result = step4_handle_quantifiers($step3_result);
print STDERR "STEP 4 RESULT (Quantifiers handled):\n" . Dumper($step4_result) unless $quiet_mode;

sub step6_generate_parser_code {
    my ($grammar_tree, $rule_order) = @_;
    
    # First pass: validate all referenced rules exist
    validate_grammar_completeness($grammar_tree);
    
    # Second pass: detect and eliminate left recursion using NUCLEAR OPTION
    require './integrate_left_recursion_killer.pl';
    my ($transformed_grammar, $new_rule_order) = eliminate_left_recursion_nuclear_option($grammar_tree, $rule_order);
    
    print STDERR "DEBUG: Grammar after left-recursion elimination:\n" . Dumper($transformed_grammar) unless $quiet_mode;
    
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
    
    # 🚀 OPTIMIZED: Pre-compile regex with cache
    my \$compiled_regex = qr/\$regex/o;
    
    # 🚀 OPTIMIZED: Tighter loop with fewer operations
    while (\$count < \$max) {
        if (\$\$input =~ /\\G\$compiled_regex/gc) {
            \$count++;
        } else {
            last;
        }
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
    my \$checkpoint = pos(\$\$input);
    
    # 🚀 OPTIMIZED: Pre-allocate array for better performance
    my \@results;
    \$#results = \$max - 1 if \$max < 1000; # Pre-allocate for reasonable sizes
    
    my \$result_idx = 0;
    while (\$count < \$max) {
        my \$result = \$rule_ref->(\$input);
        if (defined \$result) {
            \$results[\$result_idx++] = \$result;
            \$count++;
        } else {
            last;
        }
    }
    
    if (\$count >= \$min) {
        # 🚀 OPTIMIZED: Trim array to actual size
        \$#results = \$count - 1;
        return \\\@results;
    } else {
        # Restore position on failure
        pos(\$\$input) = \$checkpoint;
        return undef;
    }
}

sub collect_quantified_results {
    # 🚀 OPTIMIZED: Ultra-fast quantified results collection
    my (\$element_num, \$results_ref) = \@_;
    my \$element = \$results_ref->[\$element_num - 1];
    
    # 🚀 OPTIMIZED: Fast path for most common case (array reference)
    return \$element if ref(\$element) eq 'ARRAY';
    
    # Handle undefined (zero matches)
    return [] unless defined \$element;
    
    # Single element case
    return [\$element];
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
    print STDERR "DEBUG: Entered generate_or_parser for $rule_name\n" unless $quiet_mode;
    
    # Check if all alternatives are pure literals (optimization opportunity)
    my @literal_alternatives = ();
    my $all_literals = 1;
    
    # Apply YOUR OPTIMIZATION RULE:
    # Optimize ONLY if ALL alternatives are (literals OR regexes) AND NONE have return annotations
    foreach my $alt (@{$rule_def->{alternatives}}) {

        
        # Check if this alternative has a return annotation - if so, CANNOT optimize
        if ($alt->{return_annotation}) {
            print STDERR "DEBUG: Found return annotation - disabling optimization\n" unless $quiet_mode;
            $all_literals = 0;
            last;
        }
        
        # Check if alternative is literal or regex
        if ($alt->{type} eq 'atom' && is_terminal($alt->{value})) {
            if (ref($alt->{value}) eq 'ARRAY' && $alt->{value}->[0] eq 'epsilon') {
                # Epsilon production - not a simple literal/regex
                $all_literals = 0;
                last;
            } elsif (ref($alt->{value}) eq 'ARRAY' && ($alt->{value}->[0] eq 'regex' || $alt->{value}->[0] eq 'terminal')) {
                # This is a regex or literal - OK for optimization IF no return annotation
                push @literal_alternatives, $alt->{value}[1];
            } else {
                # Unknown terminal type
                $all_literals = 0;
                last;
            }
        } elsif ($alt->{type} eq 'sequence') {
            # Sequences are more complex - check if they're simple literals only
            my @seq_literals = ();
            my $seq_is_simple = 1;
            foreach my $element (@{$alt->{elements}}) {
                if ($element->{type} eq 'atom' && is_terminal($element->{value})) {
                    if (ref($element->{value}) eq 'ARRAY' && ($element->{value}->[0] eq 'terminal' || $element->{value}->[0] eq 'regex')) {
                        push @seq_literals, $element->{value}[1];
                    } else {
                        $seq_is_simple = 0;
                        last;
                    }
                } else {
                    $seq_is_simple = 0;
                    last;
                }
            }
            if ($seq_is_simple) {
                # Simple sequence of literals/regexes - OK for optimization IF no return annotation
                push @literal_alternatives, join('', @seq_literals);
            } else {
                $all_literals = 0;
                last;
            }
        } else {
            # Non-literal/regex alternative
            $all_literals = 0;
            last;
        }
    }
    
    if (0) {  # TEMPORARILY DISABLE OPTIMIZATION TO FIX RETURN ANNOTATIONS
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
                    
                    # Check if this alternative has a return annotation
                    if ($alt->{return_annotation}) {
                        my ($type, $annotation) = @{$alt->{return_annotation}};
                        if ($type eq 'return_object') {
                            # For object returns, substitute $1 with the captured value
                            my $object_content = $annotation;
                            $object_content =~ s/^\{|\}$//g;  # Remove braces
                            my $perl_hash = $object_content;
                            $perl_hash =~ s/(\w+):\s*/"$1" => /g;  # key: -> "key" =>
                            $perl_hash =~ s/\$1/\$1/g;  # Keep $1 as is for regex capture
                            push @alternatives, "do { if (\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc) { {$perl_hash} } else { undef } }";
                        } elsif ($type eq 'return_scalar') {
                            if ($annotation =~ /^\$\d+$/) {
                                # Variable reference like $1 - use regex capture
                                push @alternatives, "do { if (\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc) { \$1 } else { undef } }";
                            } else {
                                # Literal value like "input" - return the literal (removing quotes)
                                my $literal_value = $annotation;
                                $literal_value =~ s/^["']|["']$//g;  # Remove surrounding quotes
                                push @alternatives, "do { if (\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc) { \"$literal_value\" } else { undef } }";
                            }
                        } else {
                            push @alternatives, "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
                        }
                    } else {
                        push @alternatives, "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
                    }
                } else {
                    # For terminal atoms, generate literal match
                    my $literal = $alt->{value}[1];
                    my $regex_name = "${rule_name}_alt" . @alternatives;
                    my $escaped_literal = escape_regex_literal($literal);
                    if ($literal =~ m{/}) {
                        push @$regexes, "    '$regex_name' => qr{$escaped_literal}o";
                    } else {
                        push @$regexes, "    '$regex_name' => qr/$escaped_literal/o";
                    }
                    
                    # Check if this alternative has a return annotation
                    if ($alt->{return_annotation}) {
                        my ($type, $annotation) = @{$alt->{return_annotation}};
                        if ($type eq 'return_scalar') {
                            if ($annotation =~ /^\$\d+$/) {
                                # Variable reference like $1 - use regex capture (but terminals don't capture)
                                push @alternatives, "do { if (\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc) { \$1 } else { undef } }";
                            } else {
                                # Literal value like "input" - return the literal (removing quotes)
                                my $literal_value = $annotation;
                                $literal_value =~ s/^["']|["']$//g;  # Remove surrounding quotes
                                push @alternatives, "do { if (\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc) { \"$literal_value\" } else { undef } }";
                            }
                        } else {
                            push @alternatives, "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
                        }
                    } else {
                        push @alternatives, "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
                    }
                }
            } else {
                # For non-terminal atoms, call parser function
                my $rule_name_to_call = extract_token_value($alt->{value});
                push @alternatives, "parse_$rule_name_to_call(\$input)";
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
                            if ($literal =~ m{/}) {
                                push @$regexes, "    '$step_regex' => qr{\\Q$literal\\E}o";
                            } else {
                                my $escaped_literal = escape_regex_literal($literal);
                        push @$regexes, "    '$step_regex' => qr/$escaped_literal/o";
                            }
                            push @seq_steps, "\$\$input =~ /\\G\$REGEXES{'$step_regex'}/gc";
                        }
                    } else {
                        my $rule_name_to_call = extract_token_value($element->{value});
                        push @seq_steps, "parse_$rule_name_to_call(\$input)";
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
            
            # Handle return annotation for the sequence
            if ($alt->{return_annotation}) {
                my ($type, $annotation) = @{$alt->{return_annotation}};
                if ($type eq 'return_scalar') {
                    if ($annotation =~ /^\$\d+$/) {
                        # Variable reference like $1 - but sequences don't capture directly
                        $sequence_code .= "(\$1) || (pos(\$\$input) = \$seq_pos, undef) }";
                    } else {
                        # Literal value like "input" - return the literal (removing quotes)
                        my $literal_value = $annotation;
                        $literal_value =~ s/^["']|["']$//g;  # Remove surrounding quotes
                        $sequence_code .= "(\"$literal_value\") || (pos(\$\$input) = \$seq_pos, undef) }";
                    }
                } else {
                    # For other return types, default to success
                    $sequence_code .= "1 || (pos(\$\$input) = \$seq_pos, 0) }";
                }
            } else {
                $sequence_code .= "1 || (pos(\$\$input) = \$seq_pos, 0) }";
            }
            
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
        my $escaped_combined = escape_regex_literal($combined_literal);
        push @$regexes, "    '$regex_name' => qr/$escaped_combined/o";
        push @sequence_steps, "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
    } else {
        # Fall back to individual element processing
        print STDERR "DEBUG generate_sequence_rule: processing sequence with filtered_elements=" . Dumper(\@filtered_elements) . "\n" unless $quiet_mode;
        my $step_num = 0;
        foreach my $element (@filtered_elements) {
            $step_num++;
            print STDERR "DEBUG generate_sequence_rule: processing element $step_num: " . Dumper($element) . "\n" unless $quiet_mode;
            if ($element->{type} eq 'atom') {
                if (is_terminal($element->{value})) {
                    if (ref($element->{value}) eq 'ARRAY' && $element->{value}->[0] eq 'regex') {
                        # Direct regex match for regex pattern
                        my $pattern = $element->{value}[1];  # Extract regex pattern
                        my $regex_name = "${rule_name}_step${step_num}";
                        push @$regexes, "    '$regex_name' => qr/$pattern/o";
                        push @sequence_steps, "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
                    } elsif (ref($element->{value}) eq 'ARRAY' && $element->{value}->[0] eq 'GROUPED') {
                        # GROUPED elements should be skipped/ignored - they represent parentheses grouping 
                        # which has already been processed into the structure
                        print STDERR "WARNING: GROUPED element should have been processed earlier: " . join(", ", @{$element->{value}}) . "\n";
                        # For now, skip this element
                        next;
                    } else {
                        # Direct regex match for terminal literal
                        my $literal = $element->{value}[1];  # Extract terminal content
                        my $regex_name = "${rule_name}_step${step_num}";
                        my $escaped_literal = escape_regex_literal($literal);
                        if ($literal =~ m{/}) {
                        push @$regexes, "    '$regex_name' => qr{$escaped_literal}o";
                    } else {
                        push @$regexes, "    '$regex_name' => qr/$escaped_literal/o";
                    }  # Escape literal
                        push @sequence_steps, "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
                    }
                } elsif (ref($element->{value}) eq 'ARRAY' && $element->{value}->[0] eq 'quantified_group') {
                    # Special handling for quantified group atoms in sequences
                    my ($type, $group_content, $quantifier) = @{$element->{value}};
                    my $quant = parse_quantifier($quantifier);
                    
                    # Generate inline grouped quantifier parsing logic
                    my @group_elements = split(/~/, $group_content);
                    my $group_func_name = "${rule_name}_group_${step_num}";
                    
                    # Create inline parsing logic for the grouped pattern
                    my @inline_group_steps = ();
                    my $sub_step = 0;
                    foreach my $group_element (@group_elements) {
                        $sub_step++;
                        if ($group_element =~ /^TERMINAL:(.+)$/) {
                            my $terminal = $1;
                            my $regex_name = "${group_func_name}_step${sub_step}";
                            my $escaped_terminal = escape_regex_literal($terminal);
                            push @$regexes, "    '$regex_name' => qr/$escaped_terminal/o";
                            push @inline_group_steps, "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
                        } elsif ($group_element =~ /^REGEX:(.+)$/) {
                            my $pattern = $1;
                            my $regex_name = "${group_func_name}_step${sub_step}";
                            push @$regexes, "    '$regex_name' => qr/$pattern/o";
                            push @inline_group_steps, "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
                        } elsif ($group_element =~ /^OPERATOR:(.+)$/) {
                            # Skip operators for now
                            next;
                        } elsif ($group_element ne '') {
                            # Rule reference
                            push @inline_group_steps, "parse_$group_element(\$input)";
                        }
                    }
                    
                    my $group_pattern = join(' && ', @inline_group_steps);
                    push @sequence_steps, "quantified_match(\$input, sub { $group_pattern }, $quant->{min}, $quant->{max})";
                } else {
                    # Rule call
                    my $rule_name_to_call = extract_token_value($element->{value});
                    push @sequence_steps, "parse_$rule_name_to_call(\$input)";
                }
            } elsif ($element->{type} eq 'quantified') {
                # Generate quantified parsing code
                print STDERR "DEBUG: Found quantified element: " . Dumper($element) . "\n" unless $quiet_mode;
                my $quant_code = generate_quantified_code($element, $rule_name, $step_num, $regexes);
                print STDERR "DEBUG: Generated quantified code: $quant_code\n" unless $quiet_mode;
                push @sequence_steps, $quant_code;
            } elsif ($element->{type} eq 'quantified_group') {
                # Generate quantified group parsing code  
                print STDERR "DEBUG: Found quantified group: " . Dumper($element) . "\n" unless $quiet_mode;
                my $group_code = generate_grouped_quantifier_code($element, $rule_name, $regexes);
                push @sequence_steps, $group_code;
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
            # Regex match - capture the regex result
            push @seq_lines, "    unless ($step) {";
            push @seq_lines, "        pos(\$\$input) = \$start_pos;";
            push @seq_lines, "        return undef;";
            push @seq_lines, "    }";
            push @seq_lines, "    push \@results, \$1;  # Capture regex result";
        }
    }
    my $seq_code = join("\n", @seq_lines);
    
    # Generate return code based on annotation
    my $return_code;
    if ($return_annotation) {
        $return_code = generate_return_code_enhanced($return_annotation, \@filtered_elements);
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

# Self-hosting return annotation parser integration
sub parse_return_annotation_with_ebnf {
    my ($annotation_string) = @_;
    
    # Use our self-hosting EBNF parser
    use Parser::ReturnAnnotation qw(parse_annotation);
    
    my $result = parse_annotation($annotation_string);
    if (defined $result) {
        # Extract the actual annotation structure (skip the parsing metadata)
        return $result->[2];  # Position 2 contains the parsed return_expression
    } else {
        # Fallback to legacy regex-based parsing for compatibility
        warn "EBNF parser failed for annotation: $annotation_string, falling back to regex";
        return undef;
    }
}

# Generate Perl code from parsed return annotation AST
sub generate_ultimate_dot_notation_access {
    my ($ultimate_dot_ast) = @_;
    
    # Start with the base scalar reference  
    my $base = $ultimate_dot_ast->{base};
    my $index = extract_number_value($base->{index}) - 1;  # Convert to 0-based for internal access
    my $access_code = "(\$results[$index] // undef)";
    
    # Apply each path accessor
    foreach my $accessor (@{$ultimate_dot_ast->{path}}) {
        my $accessor_type = $accessor->{type};
        
        if ($accessor_type eq 'property') {
            # Property access: .name, .items
            my $prop_name = $accessor->{name};
            $access_code = "(ref($access_code) eq 'HASH') ? $access_code" . "->{$prop_name} : undef";
            
        } elsif ($accessor_type eq 'position') {
            # Positional access: .1, .2, .3 (1-based positions in parse tree)
            my $pos_index = extract_number_value($accessor->{index}) - 1;  # Convert to 0-based for array access
            $access_code = "(ref($access_code) eq 'ARRAY' && \@\{$access_code\} > $pos_index) ? $access_code" . "->[$pos_index] : undef";
            
        } elsif ($accessor_type eq 'array_access') {
            # Array access with comprehensive slice support
            $access_code = generate_array_access_code($access_code, $accessor->{spec});
        }
    }
    
    return $access_code;
}

sub extract_number_value {
    my ($num_obj) = @_;
    if (ref($num_obj) eq 'HASH' && $num_obj->{type} eq 'positive') {
        return $num_obj->{value};
    } elsif (ref($num_obj) eq 'HASH' && $num_obj->{type} eq 'negative') {
        return "-" . extract_number_value($num_obj->{value});
    } else {
        return $num_obj;  # Fallback for simple numbers
    }
}

sub generate_array_access_code {
    my ($base_code, $array_spec) = @_;
    my $spec_type = $array_spec->{type};
    
    if ($spec_type eq 'whole_array') {
        # Whole array access: [], [*], [:]
        return "\@\{(ref($base_code) eq 'ARRAY') ? $base_code : []\}";
        
    } elsif ($spec_type eq 'single_index') {
        # Single index: [2], [-1]
        my $index_val = extract_index_value($array_spec->{value});
        return "(ref($base_code) eq 'ARRAY') ? $base_code" . "->[$index_val] : undef";
        
    } elsif ($spec_type eq 'perl_range') {
        # Perl5 range: [0..2], [1..-1]
        my $start = extract_index_value($array_spec->{start});
        my $end = extract_index_value($array_spec->{end});
        return "\@\{(ref($base_code) eq 'ARRAY') ? $base_code : []\}[$start..$end]";
        
    } elsif ($spec_type eq 'python_slice') {
        # Python slice: [1:4], [:2], [3:]
        my $start = extract_slice_value($array_spec->{start});
        my $end = extract_slice_value($array_spec->{end});
        return generate_python_slice_code($base_code, $start, $end);
        
    } elsif ($spec_type eq 'python_slice_step') {
        # Python slice with step: [1:10:2]
        my $start = extract_slice_value($array_spec->{start});
        my $end = extract_slice_value($array_spec->{end});
        my $step = extract_index_value($array_spec->{step});
        return generate_python_slice_step_code($base_code, $start, $end, $step);
        
    } elsif ($spec_type eq 'multi_index') {
        # Multiple indices: [1,3,5]
        my @indices = map { extract_index_value($_) } @{$array_spec->{indices}};
        my $indices_str = join(',', @indices);
        return "\@\{(ref($base_code) eq 'ARRAY') ? $base_code : []\}[$indices_str]";
        
    } elsif ($spec_type eq 'mixed_expression') {
        # Mixed expressions: [0,2..4,7] - convert to individual indices
        return generate_mixed_expression_code($base_code, $array_spec->{elements});
        
    } else {
        # Unknown spec type
        return "undef";
    }
}

sub extract_index_value {
    my ($index_obj) = @_;
    if (ref($index_obj) eq 'HASH' && $index_obj->{type} eq 'positive') {
        return $index_obj->{value};
    } elsif (ref($index_obj) eq 'HASH' && $index_obj->{type} eq 'negative') {
        return "-" . extract_number_value($index_obj->{value});
    } else {
        return $index_obj;
    }
}

sub extract_slice_value {
    my ($slice_part) = @_;
    if ($slice_part eq 'default') {
        return undef;  # Empty slice part
    } else {
        return extract_index_value($slice_part);
    }
}

sub generate_python_slice_code {
    my ($base_code, $start, $end) = @_;
    
    # Convert Python slice to Perl array slice
    my $slice_expr;
    if (!defined $start && !defined $end) {
        # [:]
        $slice_expr = "0..\$\#{$base_code}";
    } elsif (!defined $start) {
        # [:end]
        my $perl_end = ($end > 0) ? $end - 1 : $end;  # Python end is exclusive
        $slice_expr = "0..$perl_end";
    } elsif (!defined $end) {
        # [start:]
        $slice_expr = "$start..\$\#{$base_code}";
    } else {
        # [start:end]
        my $perl_end = ($end > 0) ? $end - 1 : $end;  # Python end is exclusive
        $slice_expr = "$start..$perl_end";
    }
    
    return "\@\{(ref($base_code) eq 'ARRAY') ? $base_code : []\}[$slice_expr]";
}

sub generate_python_slice_step_code {
    my ($base_code, $start, $end, $step) = @_;
    
    # For stepped slices, we need to generate more complex Perl code
    return "do { my \$arr = (ref($base_code) eq 'ARRAY') ? $base_code : []; " .
           "my \$start = " . (defined $start ? $start : 0) . "; " .
           "my \$end = " . (defined $end ? $end : "\$\#\$arr + 1") . "; " .
           "my \$step = $step; " .
           "my \@result; " .
           "for (my \$i = \$start; \$i < \$end; \$i += \$step) { " .
           "push \@result, \$arr->[\$i] if \$i >= 0 && \$i <= \$\#\$arr; } " .
           "\\\@result; }";
}

sub generate_mixed_expression_code {
    my ($base_code, $elements) = @_;
    
    # Convert all mixed elements to individual indices
    my @all_indices;
    foreach my $element (@$elements) {
        my $elem_type = $element->{type};
        if ($elem_type eq 'single_index') {
            push @all_indices, extract_index_value($element->{value});
        } elsif ($elem_type eq 'perl_range') {
            my $start = extract_index_value($element->{start});
            my $end = extract_index_value($element->{end});
            push @all_indices, "$start..$end";
        } elsif ($elem_type eq 'python_slice') {
            # Convert Python slice to range
            my $start = extract_slice_value($element->{start}) // 0;
            my $end = extract_slice_value($element->{end});
            if (defined $end) {
                my $perl_end = ($end > 0) ? $end - 1 : $end;
                push @all_indices, "$start..$perl_end";
            } else {
                push @all_indices, "$start..\$\#{$base_code}";
            }
        }
    }
    
    my $indices_str = join(',', @all_indices);
    return "\@\{(ref($base_code) eq 'ARRAY') ? $base_code : []\}[$indices_str]";
}

sub generate_dot_notation_access {
    my ($dot_notation_ast) = @_;
    
    # Start with the base scalar reference
    my $base = $dot_notation_ast->{base};
    my $index = $base->{index} - 1;  # Convert to 0-based
    my $access_code = "(\$results[$index] // undef)";
    
    # Apply each path accessor
    foreach my $accessor (@{$dot_notation_ast->{path}}) {
        my $accessor_type = $accessor->{type};
        
        if ($accessor_type eq 'property') {
            my $prop_name = $accessor->{name};
            $access_code = "(ref($access_code) eq 'HASH') ? $access_code" . "->{$prop_name} : undef";
        } elsif ($accessor_type eq 'index') {
            my $index_val = $accessor->{value};
            if (ref($index_val) eq 'HASH' && exists $index_val->{sign} && $index_val->{sign} eq 'negative') {
                # Negative indexing: -1 means last element
                my $neg_index = $index_val->{value};
                $access_code = "(ref($access_code) eq 'ARRAY' && \@\{$access_code\} >= $neg_index) ? $access_code" . "->[\@\{$access_code\} - $neg_index] : undef";
            } else {
                # Positive indexing
                $access_code = "(ref($access_code) eq 'ARRAY' && \@\{$access_code\} > $index_val) ? $access_code" . "->[$index_val] : undef";
            }
        } elsif ($accessor_type eq 'bracket_index') {
            my $index_val = $accessor->{value};
            if (ref($index_val) eq 'HASH' && exists $index_val->{sign} && $index_val->{sign} eq 'negative') {
                # Negative bracket indexing: [-1]
                my $neg_index = $index_val->{value};
                $access_code = "(ref($access_code) eq 'ARRAY' && \@\{$access_code\} >= $neg_index) ? $access_code" . "->[\@\{$access_code\} - $neg_index] : undef";
            } else {
                # Positive bracket indexing: [2]
                $access_code = "(ref($access_code) eq 'ARRAY' && \@\{$access_code\} > $index_val) ? $access_code" . "->[$index_val] : undef";
            }
        }
    }
    
    return $access_code;
}

sub generate_return_code_from_ast {
    my ($annotation_ast, $filtered_elements) = @_;
    
    if (!defined $annotation_ast) {
        return "return \\\@results;";
    }
    
    if (ref($annotation_ast) ne 'HASH') {
        # Simple scalar or literal
        if ($annotation_ast =~ /^\$(\d+)$/) {
            my $index = $1 - 1;  # Convert to 0-based
            return "return \$results[$index];";
        } else {
            # Literal value
            return "return " . $annotation_ast . ";";
        }
    }
    
    my $type = $annotation_ast->{type};
    
    if ($type eq 'scalar_ref') {
        my $index = $annotation_ast->{index} - 1;  # Convert to 0-based
        return "return \$results[$index];";
    } elsif ($type eq 'dot_notation_ref') {
        my $base_code = generate_dot_notation_access($annotation_ast);
        return "return $base_code;";
    } elsif ($type eq 'ultimate_dot_notation') {
        my $base_code = generate_ultimate_dot_notation_access($annotation_ast);
        return "return $base_code;";
    } elsif ($type eq 'array') {
        if (exists $annotation_ast->{element}) {
            # Simple array: [$1]
            my $element = $annotation_ast->{element};
            if ($element->{type} eq 'scalar_ref') {
                my $index = $element->{index} - 1;
                if ($annotation_ast->{quantified}) {
                    return "return \$results[$index];";  # Already an array from quantified_rule
                } else {
                    return "return [\$results[$index]];";
                }
            }
        } elsif (exists $annotation_ast->{contents}) {
            # Multi-element array: [$1, $2, "literal"]
            my $contents = $annotation_ast->{contents};
            my @array_elements = ();
            
            # First element
            if ($contents->[0]->{type} eq 'scalar_ref') {
                my $index = $contents->[0]->{index} - 1;
                push @array_elements, "\$results[$index]";
            }
            
            # Rest elements (in array format)
            if (ref($contents->[1]) eq 'ARRAY' && @{$contents->[1]} > 0) {
                foreach my $rest_element (@{$contents->[1]}) {
                    if (ref($rest_element) eq 'HASH' && $rest_element->{type} eq 'scalar_ref') {
                        my $index = $rest_element->{index} - 1;
                        push @array_elements, "\$results[$index]";
                    } elsif (!ref($rest_element)) {
                        # Literal
                        push @array_elements, "\"$rest_element\"";
                    }
                }
            }
            
            return "return [" . join(", ", @array_elements) . "];";
        }
    } elsif ($type eq 'quantified_array') {
        # Quantified array: [$1*], [$2+], [$3{2,5}]
        my $element = $annotation_ast->{element};
        my $scalar = $element->{scalar};
        my $quantifier = $element->{quantifier};
        
        if ($scalar->{type} eq 'scalar_ref') {
            my $index = $scalar->{index};
            # Generate the collection call that matches our runtime helper
            return "return collect_quantified_results($index, \\\@results);";
        }
    } elsif ($type eq 'nested_object') {
        # Nested object with quantified arrays: {items: [$2*]}
        my $properties = $annotation_ast->{properties};
        my @object_pairs = ();
        
        foreach my $property (@$properties) {
            if (ref($property) eq 'HASH' && $property->{key}) {
                my $key = $property->{key};
                my $value = $property->{value};
                
                my $value_code;
                if (ref($value) eq 'HASH') {
                    if ($value->{type} eq 'quantified_array') {
                        # Handle nested quantified array
                        my $element = $value->{element};
                        my $scalar = $element->{scalar};
                        if ($scalar->{type} eq 'scalar_ref') {
                            my $index = $scalar->{index};
                            $value_code = "collect_quantified_results($index, \\\@results)";
                        }
                    } elsif ($value->{type} eq 'scalar_ref') {
                        my $index = $value->{index} - 1;
                        $value_code = "(\$results[$index] // undef)";
                    } elsif ($value->{type} eq 'dot_notation_ref') {
                        $value_code = generate_dot_notation_access($value);
                    } else {
                        $value_code = "undef";  # TODO: Handle other nested types
                    }
                } elsif (!ref($value)) {
                    $value_code = "\"$value\"";
                } else {
                    $value_code = "undef";
                }
                
                push @object_pairs, "\"$key\" => $value_code";
            }
        }
        
        return "return {" . join(", ", @object_pairs) . "};";
    } elsif ($type eq 'multi_object') {
        # Multi-property object: {header: $1, items: [$2*], footer: $3}
        my @object_pairs = ();
        
        # Handle each property (prop1, prop2, prop3, etc.)
        foreach my $prop_key (sort keys %$annotation_ast) {
            next if $prop_key eq 'type';  # Skip the type field
            
            my $property = $annotation_ast->{$prop_key};
            if (ref($property) eq 'HASH' && $property->{key} && $property->{value}) {
                my $key = $property->{key};
                my $value = $property->{value};
                
                my $value_code;
                if (ref($value) eq 'HASH') {
                    if ($value->{type} eq 'quantified_array') {
                        # Handle quantified array: [$2*]
                        my $element = $value->{element};
                        my $scalar = $element->{scalar};
                        if ($scalar->{type} eq 'scalar_ref') {
                            my $index = $scalar->{index};
                            $value_code = "collect_quantified_results($index, \\\@results)";
                        }
                    } elsif ($value->{type} eq 'scalar_ref') {
                        # Handle scalar reference: $1
                        my $index = $value->{index} - 1;
                        $value_code = "(\$results[$index] // undef)";
                    } elsif ($value->{type} eq 'dot_notation_ref') {
                        # Handle dot notation reference: $1.items.count
                        $value_code = generate_dot_notation_access($value);
                    } elsif ($value->{type} eq 'array') {
                        # Handle simple array: [$1]
                        my $element = $value->{element};
                        if ($element->{type} eq 'scalar_ref') {
                            my $index = $element->{index} - 1;
                            $value_code = "[\$results[$index]]";
                        }
                    } else {
                        $value_code = "undef";  # TODO: Handle other nested types
                    }
                } elsif (!ref($value)) {
                    $value_code = "\"$value\"";
                } else {
                    $value_code = "undef";
                }
                
                push @object_pairs, "\"$key\" => $value_code";
            }
        }
        
        return "return {" . join(", ", @object_pairs) . "};";
    } elsif ($type eq 'object') {
        if (exists $annotation_ast->{properties}) {
            # Multi-property object: {name: $1, value: $2}
            my $properties = $annotation_ast->{properties};
            my @object_pairs = ();
            
            # First property
            if ($properties->[0] && ref($properties->[0]) eq 'HASH' && $properties->[0]->{key}) {
                my $key = $properties->[0]->{key};
                my $value = $properties->[0]->{value};
                
                my $value_code;
                if (ref($value) eq 'HASH' && $value->{type} eq 'scalar_ref') {
                    my $index = $value->{index} - 1;
                    $value_code = "(\$results[$index] // undef)";
                } elsif (ref($value) eq 'HASH' && $value->{type} eq 'dot_notation_ref') {
                    $value_code = generate_dot_notation_access($value);
                } elsif (!ref($value)) {
                    $value_code = "\"$value\"";
                } else {
                    $value_code = "undef";
                }
                
                push @object_pairs, "\"$key\" => $value_code";
            }
            
            # Rest properties (in array format)
            if (ref($properties->[1]) eq 'ARRAY' && @{$properties->[1]} > 0) {
                foreach my $rest_property (@{$properties->[1]}) {
                    if (ref($rest_property) eq 'HASH' && $rest_property->{key}) {
                        my $key = $rest_property->{key};
                        my $value = $rest_property->{value};
                        
                        my $value_code;
                        if (ref($value) eq 'HASH' && $value->{type} eq 'scalar_ref') {
                            my $index = $value->{index} - 1;
                            $value_code = "(\$results[$index] // undef)";
                        } elsif (ref($value) eq 'HASH' && $value->{type} eq 'dot_notation_ref') {
                            $value_code = generate_dot_notation_access($value);
                        } elsif (!ref($value)) {
                            $value_code = "\"$value\"";
                        } else {
                            $value_code = "undef";
                        }
                        
                        push @object_pairs, "\"$key\" => $value_code";
                    }
                }
            }
            
            return "return {" . join(", ", @object_pairs) . "};";
        } else {
            # Simple object (backwards compatibility): {key: value}
            my $key = $annotation_ast->{key};
            my $value = $annotation_ast->{value};
            
            my $value_code;
            if (ref($value) eq 'HASH' && $value->{type} eq 'scalar_ref') {
                my $index = $value->{index} - 1;
                $value_code = "(\$results[$index] // undef)";
            } elsif (ref($value) eq 'HASH' && $value->{type} eq 'dot_notation_ref') {
                $value_code = generate_dot_notation_access($value);
            } elsif (!ref($value)) {
                $value_code = "\"$value\"";
            } else {
                $value_code = "undef";
            }
            
            return "return {\"$key\" => $value_code};";
        }
    }
    
    # Fallback
    return "return \\\@results;";
}

# Enhanced return code generation using EBNF parser
sub generate_return_code_enhanced {
    my ($return_annotation, $filtered_elements) = @_;
    my ($type, $annotation) = @$return_annotation;
    
    # Try EBNF parser first
    my $annotation_string = "$annotation";
    if ($annotation_string =~ /^->/) {
        # Full annotation with arrow
        my $parsed_ast = parse_return_annotation_with_ebnf($annotation_string);
        if ($parsed_ast) {
            return generate_return_code_from_ast($parsed_ast, $filtered_elements);
        }
    } else {
        # Just the expression part, add arrow
        my $full_annotation = "-> $annotation_string";
        my $parsed_ast = parse_return_annotation_with_ebnf($full_annotation);
        if ($parsed_ast) {
            return generate_return_code_from_ast($parsed_ast, $filtered_elements);
        }
    }
    
    # Fallback to legacy regex-based generation
    return generate_return_code_legacy($return_annotation, $filtered_elements);
}

# Legacy regex-based return code generation (renamed for clarity)
sub generate_return_code_legacy {
    my ($return_annotation, $filtered_elements) = @_;
    my ($type, $annotation) = @$return_annotation;
    
    if ($type eq 'return_scalar') {
        # Handle $1, $2, etc.
        my $var_num = $annotation;
        $var_num =~ s/\$//;  # Remove $ sign
        return "return \$results[$var_num-1];";
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
            $perl_array =~ s/\$(\d+)/\$results[$1-1]/g;
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
        
        # Handle regular $N references with bounds checking
        $perl_hash =~ s/\$(\d+)/(\$results[$1-1] \/\/ undef)/g;
        
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
    my $return_annotation = $rule_def->{return_annotation};
    
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
            
            # Generate return code
            my $return_code;
            if ($return_annotation) {
                $return_code = generate_return_code_enhanced($return_annotation, [{ type => 'atom', value => $value }]);
            } else {
                $return_code = "return 1;";
            }
            
            my $sub_code = <<"EOF";
sub parse_$rule_name {
    my (\$input) = \@_;
    if (\$\$input =~ /\\G\$REGEXES{'$rule_name'}/gc) {
        my \@results = (\$1);  # Capture regex result
        $return_code
    }
    return undef;
}
EOF
            return ($sub_code, $regexes);
        } else {
            # Regular terminal (quoted string)
            my $literal = $value->[1];  # Extract terminal content
            my $escaped_literal = escape_regex_literal($literal);
            if ($literal =~ m{/}) {
                push @$regexes, "    '$rule_name' => qr{$escaped_literal}o";
            } else {
                push @$regexes, "    '$rule_name' => qr/$escaped_literal/o";
            }
            my $sub_code = <<"EOF";
sub parse_$rule_name {
    my (\$input) = \@_;
    return 1 if \$\$input =~ /\\G\$REGEXES{'$rule_name'}/gc;
    return undef;
}
EOF
            return ($sub_code, $regexes);
        }
    } elsif (ref($value) eq 'ARRAY' && $value->[0] eq 'quantified_group') {
        # Handle quantified group from left-recursion eliminator: (pattern)*
        my ($type, $group_content, $quantifier) = @$value;
        my $quant = parse_quantifier($quantifier);
        
        # Parse the group content to generate parsing logic
        # Format: "TERMINAL:,~item" -> ["TERMINAL:,", "item"]
        my @group_elements = split(/~/, $group_content);
        
        # Generate parsing code for each element in the group
        my @group_parsing_steps = ();
        my $step_counter = 0;
        foreach my $element (@group_elements) {
            $step_counter++;
            if ($element =~ /^TERMINAL:(.+)$/) {
                my $terminal = $1;
                my $regex_name = "${rule_name}_group_step${step_counter}";
                my $escaped_terminal = escape_regex_literal($terminal);
                push @$regexes, "    '$regex_name' => qr/$escaped_terminal/o";
                push @group_parsing_steps, "        unless (\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc) { return undef; }";
            } elsif ($element =~ /^REGEX:(.+)$/) {
                my $pattern = $1;
                my $regex_name = "${rule_name}_group_step${step_counter}";
                push @$regexes, "    '$regex_name' => qr/$pattern/o";
                push @group_parsing_steps, "        unless (\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc) { return undef; }";
            } elsif ($element =~ /^OPERATOR:(.+)$/) {
                # Skip operators in grouped content for now
                next;
            } elsif ($element ne '') {
                # Rule reference
                push @group_parsing_steps, "        my \$group_result = parse_$element(\$input);";
                push @group_parsing_steps, "        unless (defined \$group_result) { return undef; }";
                push @group_parsing_steps, "        push \@group_results, \$group_result;";
            }
        }
        
        my $group_parsing_code = join("\n", @group_parsing_steps);
        
        # Generate return code with annotation support
        my $return_code;
        if ($return_annotation) {
            my ($annot_type, $annot_content) = @$return_annotation;
            if ($annot_type eq 'return_array' && $annot_content =~ /^\[\s*\$1,\s*\$2\*\s*\]$/) {
                # Special case for [$1, $2*] pattern common in HDL
                $return_code = "return [\$first_result, \@results];";
            } else {
                # General case - return the collected results
                $return_code = "return \\\@results;";
            }
        } else {
            $return_code = "return \\\@results;";
        }
        
        my $sub_code = <<"EOF";
sub parse_$rule_name {
    my (\$input) = \@_;
    my \@results = ();
    my \$count = 0;
    my \$pos = pos(\$\$input);
    
    # Parse the grouped pattern repeatedly
    while (\$count < $quant->{max}) {
        my \$start_pos = pos(\$\$input);
        my \@group_results = ();
        
        # Try to match the group pattern
$group_parsing_code
        
        # If we got here, the group matched
        push \@results, \\\@group_results;
        \$count++;
        
        # Update position for next iteration
        \$pos = pos(\$\$input);
    }
    
    # Check minimum requirement
    if (\$count < $quant->{min}) {
        pos(\$\$input) = \$pos;
        return undef;
    }
    
    $return_code
}
EOF
        return ($sub_code, $regexes);
    } elsif (ref($value) eq 'ARRAY' && $value->[0] eq 'quantified_element') {
        # Handle quantified element from left-recursion eliminator
        my ($type, $element_name_token, $quantifier) = @$value;
        my $element_name = extract_token_value($element_name_token);
        my $quant = parse_quantifier($quantifier);
        
        # Generate return code with annotation support
        my $return_code;
        if ($return_annotation) {
            my ($annot_type, $annot_content) = @$return_annotation;
            if ($annot_type eq 'return_array' && $annot_content =~ /^\[\s*\$1\*\s*\]$/) {
                # For quantified collections like [$1*], $result is already the array
                $return_code = "return \$result;";
            } elsif ($annot_type eq 'return_scalar' && $annot_content eq '$1') {
                # For single quantified results like $1, $result might be an array - return first element or undef
                $return_code = "return (ref(\$result) eq 'ARRAY' && \@\$result > 0) ? \$result->[0] : \$result;";
            } else {
                # For other annotations, wrap $result in @results context for generate_return_code
                $return_code = generate_return_code_enhanced($return_annotation, [{ type => 'quantified', element => $element_name, quantifier => $quantifier }]);
                $return_code =~ s/\\\@results/[\$result]/g;  # Replace @results with [$result]
            }
        } else {
            $return_code = "return \$result;";
        }
        
        my $sub_code = <<"EOF";
sub parse_$rule_name {
    my (\$input) = \@_;
    my \$result = quantified_rule(\$input, \\&parse_$element_name, $quant->{min}, $quant->{max});
    return undef unless defined \$result;
    $return_code
}
EOF
        return ($sub_code, []);
    } else {
        my $rule_name_to_call = extract_token_value($value);
        my $sub_code = <<"EOF";
sub parse_$rule_name {
    my (\$input) = \@_;
    return parse_$rule_name_to_call(\$input);
}
EOF
        return ($sub_code, []);
    }
}

sub generate_grouped_quantifier_code {
    my ($element, $rule_name, $regexes) = @_;
    my $quant = parse_quantifier($element->{quantifier});
    
    print STDERR "DEBUG generate_grouped_quantifier_code: element=" . Dumper($element) . "\n" unless $quiet_mode;
    print STDERR "DEBUG generate_grouped_quantifier_code: quantifier=$element->{quantifier}, parsed quant=" . Dumper($quant) . "\n" unless $quiet_mode;
    
    # Generate parsing logic for the group content
    my @group_steps = ();
    my $step_num = 0;
    foreach my $group_element (@{$element->{group_content}}) {
        $step_num++;
        if (ref($group_element) eq 'ARRAY' && @$group_element == 2) {
            my ($type, $value) = @$group_element;
            
            if ($type eq 'quoted_string') {
                # Literal string match
                my $escaped_literal = escape_regex_literal($value);
                my $regex_name = "${rule_name}_group_literal_${step_num}";
                push @$regexes, "    '$regex_name' => qr/$escaped_literal/o";
                push @group_steps, "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
            } elsif ($type eq 'regex') {
                # Regex pattern match
                my $regex_name = "${rule_name}_group_regex_${step_num}";
                push @$regexes, "    '$regex_name' => qr/$value/o";
                push @group_steps, "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
            } elsif ($type eq 'rule_reference') {
                # Rule reference
                push @group_steps, "parse_$value(\$input)";
            }
        }
    }
    
    # Generate quantified loop for the group
    my $group_check = join(' && ', @group_steps);
    
    if ($quant->{min} == 0 && $quant->{max} == 999) { # * quantifier
        return "quantified_rule(\$input, sub { my \$group_pos = pos(\$\$input); ($group_check) || (pos(\$\$input) = \$group_pos, 0) }, 0, 999)";
    } elsif ($quant->{min} == 1 && $quant->{max} == 999) { # + quantifier  
        return "quantified_rule(\$input, sub { my \$group_pos = pos(\$\$input); ($group_check) || (pos(\$\$input) = \$group_pos, 0) }, 1, 999)";
    } elsif ($quant->{min} == 0 && $quant->{max} == 1) { # ? quantifier
        return "quantified_rule(\$input, sub { my \$group_pos = pos(\$\$input); ($group_check) || (pos(\$\$input) = \$group_pos, 0) }, 0, 1)";
    } else {
        # Custom quantifier {n,m}
        return "quantified_rule(\$input, sub { my \$group_pos = pos(\$\$input); ($group_check) || (pos(\$\$input) = \$group_pos, 0) }, $quant->{min}, $quant->{max})";
    }
}

sub generate_quantified_code {
    my ($element, $rule_name, $step_num, $regexes) = @_;
    my $quant = parse_quantifier($element->{quantifier});
    
    print STDERR "DEBUG generate_quantified_code: element=" . Dumper($element) . "\n" unless $quiet_mode;
    print STDERR "DEBUG generate_quantified_code: quantifier=$element->{quantifier}, parsed quant=" . Dumper($quant) . "\n" unless $quiet_mode;
    
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
            my $escaped_literal = escape_regex_literal($literal);
            push @$regexes, "    '$regex_name' => qr/$escaped_literal/o";
            return "quantified_match(\$input, \$REGEXES{'$regex_name'}, $quant->{min}, $quant->{max})";
        }
    } else {
        my $element_name = extract_token_value($element->{element});
        return "quantified_rule(\$input, \\&parse_$element_name, $quant->{min}, $quant->{max})";
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
    # Check if value is explicitly marked as terminal type
    # Updated for new consistent token format: quoted_string, number, regex
    return ref($value) eq 'ARRAY' && ($value->[0] eq 'quoted_string' || $value->[0] eq 'number' || 
                                      $value->[0] eq 'regex' || $value->[0] eq 'operator' || 
                                      $value->[0] eq 'GROUPED' || $value->[0] eq 'epsilon');
}

sub is_return_annotation {
    my ($value) = @_;
    # Check if value is a return annotation (rule metadata, not grammar symbol)
    return ref($value) eq 'ARRAY' && ($value->[0] eq 'return_scalar' || $value->[0] eq 'return_array' || $value->[0] eq 'return_object');
}

sub is_return_annotation_string {
    my ($value) = @_;
    # Check if value is a return annotation string that got incorrectly treated as a rule name
    return defined($value) && !ref($value) && ($value =~ /^[\w\s:"$,\[\]\{\}]+$/ && ($value =~ /type:|items:|name:|value:|left:|right:|op:/));
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
    
    print STDERR "\n=== Eliminating Left-Recursion ===\n" unless $quiet_mode;
    
    my %new_grammar = %$grammar_tree;  # Copy original grammar
    my @new_order = @$rule_order;      # Copy original order
    
    # Step 1: Detect left-recursive rules
    my @left_recursive_rules = ();
    
    foreach my $rule_name (keys %new_grammar) {
        if (is_left_recursive($rule_name, $new_grammar{$rule_name})) {
            push @left_recursive_rules, $rule_name;
            print STDERR "Found left-recursive rule: $rule_name\n" unless $quiet_mode;
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
                
                print STDERR "Transformed $rule_name -> $rule_name + $tail_rule_name\n" unless $quiet_mode;
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

print STDERR "\n=== Step 5: Build tree structure ===\n" unless $quiet_mode;
my ($step5_result, $rule_order) = step5_build_tree_structure($step4_result);
print STDERR "STEP 5 RESULT (Tree structure):\n" . Dumper($step5_result) unless $quiet_mode;
print STDERR "RULE ORDER: " . join(", ", @$rule_order) . "\n" unless $quiet_mode;

print STDERR "\n=== Step 6: Generate parser code ===\n" unless $quiet_mode;
my $step6_result = step6_generate_parser_code($step5_result, $rule_order);
print $step6_result;
