package AST::Transform;

use strict;
use warnings;
use Data::Dumper;
use lib 'fx/perl';
use lib 'perl';
use LinkedSpec;
use AST::LeftRecursion;
# BOOTSTRAP FIX: Use require instead of use to avoid compile-time loading
# This allows us to conditionally load the parser and break the self-hosting cycle
# use ultimate_return_annotation_perl_parser;  # DISABLED FOR BOOTSTRAP
use AST::UniversalReturnAnnotation;
use AST::UniversalComposer;
use AST::PerlReturnCodeGenerator;
use AST::BacktrackingParserIntegration qw(is_grouped_quantifier extract_grouped_elements detect_grouped_quantifier_in_element parse_quantifier_bounds);
use lib '.';  # Add current directory to include path

# Helper function to parse annotations into universal AST
sub parse_annotation_to_universal_ast {
    my ($annotation) = @_;
    
    # Try to parse with EBNF first
    my $annotation_string = "$annotation";
    if ($annotation_string =~ /^->/) {
        # Full annotation with arrow
        my $parsed_ast = parse_return_annotation_with_ebnf($annotation_string);
        if ($parsed_ast) {
            # Convert EBNF parser result to universal AST
            return convert_ebnf_ast_to_universal($parsed_ast);
        }
    } else {
        # Just the expression part, add arrow
        my $full_annotation = "-> $annotation_string";
        my $parsed_ast = parse_return_annotation_with_ebnf($full_annotation);
        if ($parsed_ast) {
            # Convert EBNF parser result to universal AST
            return convert_ebnf_ast_to_universal($parsed_ast);
        }
    }
    
    # Fallback: try to parse common patterns directly
    return parse_common_annotation_patterns($annotation);
}

# Helper to convert EBNF parser AST to universal AST
sub convert_ebnf_ast_to_universal {
    my ($ebnf_ast) = @_;
    
    # Extract the return expression from the EBNF parser result
    my $return_expr;
    if (ref($ebnf_ast) eq 'ARRAY' && @$ebnf_ast >= 3 && defined $ebnf_ast->[2]) {
        $return_expr = $ebnf_ast->[2];
    } else {
        $return_expr = $ebnf_ast;
    }
    
    # Convert to universal AST format
    return ebnf_to_universal_ast($return_expr);
}

# Helper to convert EBNF AST nodes to universal AST nodes
sub ebnf_to_universal_ast {
    my ($node) = @_;
    
    return undef unless defined $node && ref($node) eq 'HASH';
    
    my $type = $node->{type} || '';
    
    if ($type eq 'scalar_ref') {
        my $index = $node->{index};
        # Handle parsed number objects: {value => '1', type => 'positive'}
        if (ref($index) eq 'HASH' && exists $index->{value}) {
            $index = $index->{value};
        }
        return AST::UniversalReturnAnnotation::new_scalar_ref(
            $index
        );
    }
    elsif ($type eq 'quantified_array') {
        my $element = $node->{element};
        if ($element && $element->{scalar} && $element->{scalar}{type} eq 'scalar_ref') {
            return AST::UniversalReturnAnnotation::new_quantified_array(
                AST::UniversalReturnAnnotation::new_scalar_ref(
                    $element->{scalar}{index}
                )
            );
        }
    }
    elsif ($type eq 'object') {
        my $key = $node->{key};
        my $value = $node->{value};
        
        my $value_ast = ebnf_to_universal_ast($value);
        if ($value_ast) {
            return AST::UniversalReturnAnnotation::new_object(
                {key => $key, value => $value_ast}
            );
        }
    }
    elsif ($type eq 'multi_object') {
        my %properties;
        
        # Process each property
        for my $i (1..10) {
            my $prop_key = "prop$i";
            last unless exists $node->{$prop_key};
            
            my $prop = $node->{$prop_key};
            my $key = $prop->{key};
            my $value_ast = ebnf_to_universal_ast($prop->{value});
            
            if ($value_ast) {
                $properties{$key} = $value_ast;
            }
        }
        
        if (keys %properties) {
            my @pairs = map { {key => $_, value => $properties{$_}} } keys %properties;
            return AST::UniversalReturnAnnotation::new_object(@pairs);
        }
    }
    elsif ($type eq 'string') {
        return AST::UniversalReturnAnnotation::new_literal(
            $node->{value}
        );
    }
    
    return undef;
}

# Fallback parser for common annotation patterns
sub parse_common_annotation_patterns {
    my ($annotation) = @_;
    
    # Remove surrounding whitespace
    $annotation =~ s/^\s+|\s+$//g;
    
    # Pattern: $N (scalar reference)
    if ($annotation =~ /^\$(\d+)$/) {
        return AST::UniversalReturnAnnotation::new_scalar_ref(
            $1
        );
    }
    
    # Pattern: $N.property (dot notation)
    if ($annotation =~ /^\$(\d+)\.(\w+)$/) {
        # Note: DotNotation not implemented in function-based API, fall back to simple scalar
        return AST::UniversalReturnAnnotation::new_scalar_ref(
            $1
        );
    }
    
    # Pattern: $N[index] (array access)
    if ($annotation =~ /^\$(\d+)\[(\d+)\]$/) {
        # Note: Array access not directly supported in function-based API, fall back to simple scalar
        return AST::UniversalReturnAnnotation::new_scalar_ref(
            $1
        );
    }
    
    # Pattern: [$N*] (quantified array)
    if ($annotation =~ /^\[\s*\$(\d+)\*\s*\]$/) {
        return AST::UniversalReturnAnnotation::new_quantified_array(
            AST::UniversalReturnAnnotation::new_scalar_ref(
                $1
            )
        );
    }
    
    # Pattern: [$N+] (quantified array with +)
    if ($annotation =~ /^\[\s*\$(\d+)\+\s*\]$/) {
        # Note: Quantifier '+' not directly supported in function-based API, fall back to basic quantified array
        return AST::UniversalReturnAnnotation::new_quantified_array(
            AST::UniversalReturnAnnotation::new_scalar_ref(
                $1
            )
        );
    }
    
    # Pattern: [$N?] (quantified array with ?)
    if ($annotation =~ /^\[\s*\$(\d+)\?\s*\]$/) {
        # Note: Quantifier '?' not directly supported in function-based API, fall back to basic quantified array
        return AST::UniversalReturnAnnotation::new_quantified_array(
            AST::UniversalReturnAnnotation::new_scalar_ref(
                $1
            )
        );
    }
    
    # Pattern: [$N, $M] (mixed array)
    if ($annotation =~ /^\[\s*\$(\d+)\s*,\s*\$(\d+)\s*\]$/) {
        # Note: MixedArray not directly supported in function-based API
        # Fall back to simple quantified array for first element
        return AST::UniversalReturnAnnotation::new_quantified_array(
            AST::UniversalReturnAnnotation::new_scalar_ref(
                $1
            )
        );
    }
    
    # Pattern: [$N, $M*] (mixed array with quantified)
    if ($annotation =~ /^\[\s*\$(\d+)\s*,\s*\$(\d+)\*\s*\]$/) {
        # Note: MixedArray with quantified not directly supported in function-based API
        # Fall back to simple quantified array for first element
        return AST::UniversalReturnAnnotation::new_quantified_array(
            AST::UniversalReturnAnnotation::new_scalar_ref(
                $1
            )
        );
    }
    
    # Pattern: {key: $N} (simple object)
    if ($annotation =~ /^\{\s*(\w+)\s*:\s*\$(\d+)\s*\}$/) {
        return AST::UniversalReturnAnnotation::new_object(
            {key => $1, value => AST::UniversalReturnAnnotation::new_scalar_ref($2)}
        );
    }
    
    # Pattern: {key: $N.property} (object with dot notation)
    if ($annotation =~ /^\{\s*(\w+)\s*:\s*\$(\d+)\.(\w+)\s*\}$/) {
        # Note: DotNotation not supported in function-based API, fall back to simple object with scalar
        return AST::UniversalReturnAnnotation::new_object(
            {key => $1, value => AST::UniversalReturnAnnotation::new_scalar_ref($2)}
        );
    }
    
    # Pattern: {key: [$N*]} (object with quantified array)
    if ($annotation =~ /^\{\s*(\w+)\s*:\s*\[\s*\$(\d+)\*\s*\]\s*\}$/) {
        return AST::UniversalReturnAnnotation::new_object(
            {key => $1, value => AST::UniversalReturnAnnotation::new_quantified_array(
                AST::UniversalReturnAnnotation::new_scalar_ref($2)
            )}
        );
    }
    
    # Pattern: {key1: $N, key2: $M} (two-property object)
    if ($annotation =~ /^\{\s*(\w+)\s*:\s*\$(\d+)\s*,\s*(\w+)\s*:\s*\$(\d+)\s*\}$/) {
        return AST::UniversalReturnAnnotation::new_object(
            {key => $1, value => AST::UniversalReturnAnnotation::new_scalar_ref($2)},
            {key => $3, value => AST::UniversalReturnAnnotation::new_scalar_ref($4)}
        );
    }
    
    # Pattern: {key1: $N, key2: [$M*]} (mixed object)
    if ($annotation =~ /^\{\s*(\w+)\s*:\s*\$(\d+)\s*,\s*(\w+)\s*:\s*\[\s*\$(\d+)\*\s*\]\s*\}$/) {
        return AST::UniversalReturnAnnotation::new_object(
            {key => $1, value => AST::UniversalReturnAnnotation::new_scalar_ref($2)},
            {key => $3, value => AST::UniversalReturnAnnotation::new_quantified_array(
                AST::UniversalReturnAnnotation::new_scalar_ref($4)
            )}
        );
    }
    
    # Pattern: {key1: $N, key2: $M, key3: $P} (three-property object)
    if ($annotation =~ /^\{\s*(\w+)\s*:\s*\$(\d+)\s*,\s*(\w+)\s*:\s*\$(\d+)\s*,\s*(\w+)\s*:\s*\$(\d+)\s*\}$/) {
        return AST::UniversalReturnAnnotation::new_object(
            {key => $1, value => AST::UniversalReturnAnnotation::new_scalar_ref($2)},
            {key => $3, value => AST::UniversalReturnAnnotation::new_scalar_ref($4)},
            {key => $5, value => AST::UniversalReturnAnnotation::new_scalar_ref($6)}
        );
    }
    
    return undef;
}

# Global variables for configuration
our $quiet_mode = 0;
our $verbosity = 'normal';
our $bootstrap_mode = 0;
our $ERROR_CONTEXT = {
    verbosity => 'normal',
    errors => []
};

# Export main functions
use Exporter 'import';
our @EXPORT_OK = qw(
    generate_parser_from_file
    generate_parser_from_grammar
    process_transformation_phases
    load_ebnf_spec
    load_ebnf_spec_from_content
    step2_group_by_or
    step2_5_handle_parentheses
    step3_parse_sequences
    step4_handle_quantifiers
    step5_build_tree_structure
    get_error_context
    validate_grammar
    ebnf_to_universal_ast
    parse_annotation_to_universal_ast
);

# Helper function to extract values from structured tokens
sub extract_token_value {
    my ($token) = @_;
    
    # Handle quantifier tokens specifically to prevent HASH stringification
    if (ref($token) eq 'HASH') {
        # Check for quantifier hash structures
        if (exists $token->{quantifier} || exists $token->{min} || exists $token->{max}) {
            # This looks like a quantifier object - extract the quantifier symbol
            if (exists $token->{quantifier}) {
                return $token->{quantifier};
            } elsif (exists $token->{symbol}) {
                return $token->{symbol};
            } elsif (exists $token->{type} && $token->{type} eq 'quantifier' && exists $token->{value}) {
                return $token->{value};
            } else {
                # Fallback - derive quantifier from min/max
                my $min = $token->{min} || 0;
                my $max = $token->{max} || 999;
                if ($min == 0 && $max == 999) {
                    return '*';
                } elsif ($min == 1 && $max == 999) {
                    return '+';
                } elsif ($min == 0 && $max == 1) {
                    return '?';
                } else {
                    return "$min,$max";
                }
            }
        }
        
        # Handle atom tokens
        if ($token->{type} eq 'atom' && ref($token->{value}) eq 'ARRAY' && @{$token->{value}} == 2) {
            # New hash format: {type => 'atom', value => ['rule', 'name']}
            return $token->{value}->[1];
        }
        
        # Handle operator tokens that might be quantifiers
        if ($token->{type} eq 'operator' && exists $token->{value}) {
            # Check if this is a quantifier operator
            my $op = $token->{value};
            if ($op eq '*' || $op eq '+' || $op eq '?') {
                return $op;
            }
            # For array-based operator tokens
            if (ref($op) eq 'ARRAY' && @$op >= 2) {
                my $op_symbol = $op->[1];
                if ($op_symbol eq '*' || $op_symbol eq '+' || $op_symbol eq '?') {
                    return $op_symbol;
                }
            }
            return $op;
        }
        
        # Other hash formats - return as is to avoid stringification
        print STDERR "WARNING: Unhandled hash token structure in extract_token_value: " . Dumper($token) . "\n" if !$quiet_mode && $verbosity eq 'debug';
        return "HASH_TOKEN_" . (ref($token) || 'UNKNOWN');
    } elsif (ref($token) eq 'ARRAY' && @$token == 2) {
        # Legacy array format: ['rule', 'name'] or ['quoted_string', 'value'] or ['operator', '*']
        if ($token->[0] eq 'operator' && ($token->[1] eq '*' || $token->[1] eq '+' || $token->[1] eq '?')) {
            # This is a quantifier operator
            return $token->[1];
        }
        return $token->[1];
    } else {
        # Already extracted or other format
        return $token;
    }
}

# Helper function to safely escape literals for regex
sub escape_regex_literal {
    my ($literal) = @_;
    # Handle special cases that need careful escaping
    if ($literal eq '$') {
        return '\\$';  # Double escape for dollar sign
    } elsif ($literal eq "'") {
        return "\\'";  # Escape single quote
    } elsif ($literal eq '\\') {
        return '\\\\\\\\';  # Quadruple escape for backslash
    } elsif (!defined $literal) {
        return '';  # Handle undefined literals
    } else {
        return "\\Q$literal\\E";  # Standard quotemeta escaping
    }
}

sub step2_group_by_or {
    my ($raw_ast) = @_;
    my %rules_by_name = (); # Hash to collect rules by name
    
    # DEBUG: Track dot_path rule
    print STDERR "\n🔍 STEP 2 DEBUG: Looking for dot_path rule in input...\n" unless $quiet_mode;
    my $dot_path_found_input = 0;
    foreach my $rule_tokens (@$raw_ast) {
        my ($rule_name_token, @tokens) = @$rule_tokens;
        my $rule_name = extract_token_value($rule_name_token);
        if ($rule_name eq 'dot_path') {
            $dot_path_found_input = 1;
            print STDERR "🎯 STEP 2: Found dot_path rule in input: " . join(", ", map { ref($_) eq 'ARRAY' ? "[" . join(",", @$_) . "]" : $_ } @$rule_tokens) . "\n" unless $quiet_mode;
            last;
        }
    }
    print STDERR "❌ STEP 2: dot_path rule NOT found in input!\n" unless $quiet_mode || $dot_path_found_input;
    
    # First pass: collect all alternatives for each rule name
    foreach my $rule_tokens (@$raw_ast) {
        my ($rule_name_token, @tokens) = @$rule_tokens;
        my $rule_name = extract_token_value($rule_name_token);
        
        # Split tokens on | operators, but respect parentheses nesting
        my @or_groups = ();
        my @current_group = ();
        my $paren_depth = 0;
        
        foreach my $token (@tokens) {
            if (ref($token) eq 'ARRAY') {
                # Track parentheses depth to avoid splitting inside nested groups
                if (($token->[0] eq 'group_open' || $token->[0] eq 'operator') && $token->[1] eq '(') {
                    $paren_depth++;
                } elsif (($token->[0] eq 'group_close' || $token->[0] eq 'operator') && $token->[1] eq ')') {
                    $paren_depth--;
                }
                
                # Only split on | when not inside parentheses
                if ($token->[0] eq 'operator' && $token->[1] eq '|' && $paren_depth == 0) {
                    # Save current group and start new one
                    push @or_groups, [@current_group] if @current_group;
                    @current_group = ();
                } else {
                    push @current_group, $token;
                }
            } else {
                push @current_group, $token;
            }
        }
        
        # Add final group
        push @or_groups, [@current_group] if @current_group;
        
        # Collect OR groups for this rule name
        if (!exists $rules_by_name{$rule_name}) {
            $rules_by_name{$rule_name} = [];
        }
        
        # Add all OR groups from this rule definition to the collected groups
        push @{$rules_by_name{$rule_name}}, @or_groups;
    }
    
    # Second pass: create merged rule structures
    my @transformed_rules = ();
    foreach my $rule_name (keys %rules_by_name) {
        my $rule = {
            name => $rule_name,
            or_groups => $rules_by_name{$rule_name}
        };
        push @transformed_rules, $rule;
    }
    
    # DEBUG: Check if dot_path made it through step 2
    my $dot_path_found_output = 0;
    foreach my $rule (@transformed_rules) {
        if ($rule->{name} eq 'dot_path') {
            $dot_path_found_output = 1;
            print STDERR "✅ STEP 2: dot_path rule found in output: " . Dumper($rule) . "\n" unless $quiet_mode;
            last;
        }
    }
    print STDERR "❌ STEP 2: dot_path rule LOST in step 2!\n" unless $quiet_mode || $dot_path_found_output;
    
    return \@transformed_rules;
}

sub step2_5_handle_parentheses {
    my ($step2_result) = @_;
    my @transformed_rules = ();
    
    # DEBUG: Track dot_path rule input to step 2.5
    print STDERR "\n🔍 STEP 2.5 DEBUG: Looking for dot_path rule in input...\n" unless $quiet_mode;
    my $dot_path_found_input = 0;
    foreach my $rule (@$step2_result) {
        if ($rule->{name} eq 'dot_path') {
            $dot_path_found_input = 1;
            print STDERR "🎯 STEP 2.5: Found dot_path rule in input: " . Dumper($rule) . "\n" unless $quiet_mode;
            last;
        }
    }
    print STDERR "❌ STEP 2.5: dot_path rule NOT found in input!\n" unless $quiet_mode || $dot_path_found_input;
    
    foreach my $rule (@$step2_result) {
        my $rule_name = $rule->{name};
        my @processed_or_groups = ();
        
        foreach my $or_group (@{$rule->{or_groups}}) {
            my $processed_group = process_parentheses_in_sequence($or_group);
            push @processed_or_groups, $processed_group;
        }
        
        push @transformed_rules, {
            name => $rule_name,
            or_groups => \@processed_or_groups
        };
    }
    
    # DEBUG: Check if dot_path made it through step 2.5
    my $dot_path_found_output = 0;
    foreach my $rule (@transformed_rules) {
        if ($rule->{name} eq 'dot_path') {
            $dot_path_found_output = 1;
            print STDERR "✅ STEP 2.5: dot_path rule found in output: " . Dumper($rule) . "\n" unless $quiet_mode;
            last;
        }
    }
    print STDERR "❌ STEP 2.5: dot_path rule LOST in step 2.5!\n" unless $quiet_mode || $dot_path_found_output;
    
    return \@transformed_rules;
}

sub is_group_open {
    my ($token) = @_;
    return ref($token) eq 'ARRAY' && (
        ($token->[0] eq 'operator' && $token->[1] eq '(') ||
        ($token->[0] eq 'group_open' && $token->[1] eq '(') ||
        ($token->[0] eq '(')  # Handle single-element array format
    );
}

sub is_group_close {
    my ($token) = @_;
    return ref($token) eq 'ARRAY' && (
        ($token->[0] eq 'operator' && $token->[1] eq ')') ||
        ($token->[0] eq 'group_close' && $token->[1] eq ')') ||
        ($token->[0] eq ')')  # Handle single-element array format
    );
}

sub process_parentheses_in_sequence {
    my ($tokens) = @_;
    my @result = ();
    my @group_stack = ();
    my $group_depth = 0;
    
    foreach my $token (@$tokens) {
        if (is_group_open($token)) {
            $group_depth++;
            push @group_stack, [];
        } elsif (is_group_close($token)) {
            if ($group_depth > 0) {
                $group_depth--;
                my $group_content = pop @group_stack;
                
                # Process nested OR alternatives within the group
                my $processed_group_content = process_nested_or_alternatives($group_content);
                
                # Mark this as a grouped sequence
                my $grouped_token = ['GROUPED', $processed_group_content];
                
                if (@group_stack) {
                    # We're still inside another group
                    push @{$group_stack[-1]}, $grouped_token;
                } else {
                    # We're at the top level
                    push @result, $grouped_token;
                }
            } else {
                # Unmatched closing parenthesis - treat as literal
                push @result, $token;
            }
        } else {
            if (@group_stack) {
                # We're inside a group
                push @{$group_stack[-1]}, $token;
            } else {
                # We're at the top level
                push @result, $token;
            }
        }
    }
    
    # Handle unmatched opening parentheses
    while (@group_stack) {
        my $unmatched_group = pop @group_stack;
        my $processed_group_content = process_nested_or_alternatives($unmatched_group);
        push @result, ['GROUPED', $processed_group_content];
    }
    
    return \@result;
}

# New function to handle nested OR alternatives within groups
sub process_nested_or_alternatives {
    my ($group_content) = @_;
    
    # Check if group contains OR operators (|) that need special processing
    my @or_groups = ();
    my @current_group = ();
    
    foreach my $token (@$group_content) {
        if (ref($token) eq 'ARRAY' && $token->[0] eq 'operator' && $token->[1] eq '|') {
            # Found OR operator - start new alternative
            push @or_groups, [@current_group] if @current_group;
            @current_group = ();
        } else {
            push @current_group, $token;
        }
    }
    
    # Add final group
    push @or_groups, [@current_group] if @current_group;
    
    # If we found multiple OR groups, create an OR structure
    if (@or_groups > 1) {
        return {
            type => 'or',
            alternatives => [map { { type => 'sequence', elements => $_ } } @or_groups]
        };
    } else {
        # No OR alternatives - return original content
        return $group_content;
    }
}

sub is_grouped_alternative {
    my ($token) = @_;
    return ref($token) eq 'ARRAY' && $token->[0] eq 'GROUPED';
}

sub step3_parse_sequences {
    my ($step2_5_result) = @_;
    my @transformed_rules = ();
    
    # DEBUG: Track dot_path rule input to step 3
    print STDERR "\n🔍 STEP 3 DEBUG: Looking for dot_path rule in input...\n" unless $quiet_mode;
    my $dot_path_found_input = 0;
    foreach my $rule (@$step2_5_result) {
        if ($rule->{name} eq 'dot_path') {
            $dot_path_found_input = 1;
            print STDERR "🎯 STEP 3: Found dot_path rule in input: " . Dumper($rule) . "\n" unless $quiet_mode;
            last;
        }
    }
    print STDERR "❌ STEP 3: dot_path rule NOT found in input!\n" unless $quiet_mode || $dot_path_found_input;
    
    foreach my $rule (@$step2_5_result) {
        my $rule_name = $rule->{name};
        my @parsed_alternatives = ();
        
        foreach my $or_group (@{$rule->{or_groups}}) {
            my $parsed_alternative = {
                type => 'sequence',
                elements => $or_group
            };
            push @parsed_alternatives, $parsed_alternative;
        }
        
        # Build rule structure based on number of alternatives
        my $parsed_rule;
        if (@parsed_alternatives == 1) {
            # Single alternative - use the alternative directly
            $parsed_rule = {
                name => $rule_name,
                %{$parsed_alternatives[0]}
            };
        } else {
            # Multiple alternatives - create OR structure
            $parsed_rule = {
                name => $rule_name,
                type => 'or',
                alternatives => \@parsed_alternatives
            };
        }
        
        push @transformed_rules, $parsed_rule;
    }
    
    # DEBUG: Check if dot_path made it through step 3
    my $dot_path_found_output = 0;
    foreach my $rule (@transformed_rules) {
        if ($rule->{name} eq 'dot_path') {
            $dot_path_found_output = 1;
            print STDERR "✅ STEP 3: dot_path rule found in output: " . Dumper($rule) . "\n" unless $quiet_mode;
            last;
        }
    }
    print STDERR "❌ STEP 3: dot_path rule LOST in step 3!\n" unless $quiet_mode || $dot_path_found_output;
    
    return \@transformed_rules;
}

sub step4_handle_quantifiers {
    my ($step3_result) = @_;
    my @transformed_rules = ();
    
    # DEBUG: Track dot_path rule input to step 4
    print STDERR "\n🔍 STEP 4 DEBUG: Looking for dot_path rule in input...\n" unless $quiet_mode;
    my $dot_path_found_input = 0;
    foreach my $rule (@$step3_result) {
        if ($rule->{name} eq 'dot_path') {
            $dot_path_found_input = 1;
            print STDERR "🎯 STEP 4: Found dot_path rule in input: " . Dumper($rule) . "\n" unless $quiet_mode;
            last;
        }
    }
    print STDERR "❌ STEP 4: dot_path rule NOT found in input!\n" unless $quiet_mode || $dot_path_found_input;
    
    foreach my $rule (@$step3_result) {
        my $rule_name = $rule->{name};
        my $transformed_rule;
        
        if ($rule->{type} eq 'or') {
            # Process each alternative
            my @processed_alternatives = ();
            foreach my $alternative (@{$rule->{alternatives}}) {
                my $processed_alt = process_quantifiers_in_sequence($alternative);
                push @processed_alternatives, $processed_alt;
            }
            
            $transformed_rule = {
                name => $rule_name,
                type => 'or',
                alternatives => \@processed_alternatives
            };
        } elsif ($rule->{type} eq 'sequence') {
            # Process the sequence
            $transformed_rule = process_quantifiers_in_sequence($rule);
            $transformed_rule->{name} = $rule_name;
        } else {
            # Other types pass through unchanged
            $transformed_rule = $rule;
        }
        
        push @transformed_rules, $transformed_rule;
    }
    
    # DEBUG: Check if dot_path made it through step 4
    my $dot_path_found_output = 0;
    foreach my $rule (@transformed_rules) {
        if ($rule->{name} eq 'dot_path') {
            $dot_path_found_output = 1;
            print STDERR "✅ STEP 4: dot_path rule found in output: " . Dumper($rule) . "\n" unless $quiet_mode;
            last;
        }
    }
    print STDERR "❌ STEP 4: dot_path rule LOST in step 4!\n" unless $quiet_mode || $dot_path_found_output;
    
    return \@transformed_rules;
}

sub process_quantifiers_in_sequence {
    my ($sequence) = @_;
    my @processed_elements = ();
    my @elements = @{$sequence->{elements}};
    
    # First, filter out return annotations
    my ($return_annotation, $filtered_elements) = extract_return_annotation(\@elements);
    
    my $i = 0;
    while ($i < @$filtered_elements) {
        my $element = $filtered_elements->[$i];
        my $next_element = ($i < $#{$filtered_elements}) ? $filtered_elements->[$i + 1] : undef;
        
        # Check if next element is a quantifier
        if ($next_element && is_quantifier($next_element)) {
            # Handle grouped quantifiers specially
            if (is_grouped_alternative($element)) {
                # This is a grouped quantifier: (pattern)*
                my $group_content = $element->[1];
                # Create a proper quantified group structure
                my $quantified = {
                    type => 'quantified',
                    element => {
                        type => 'sequence',
                        elements => $group_content  # Store the actual elements, not the GROUPED wrapper
                    },
                    quantifier => extract_token_value($next_element)
                };
                push @processed_elements, $quantified;
            } else {
                # Regular quantified element
                print STDERR "DEBUG: Creating quantified element from element = " . Dumper($element) . "\n" if !$quiet_mode && $verbosity eq 'debug';
                print STDERR "DEBUG: element ref type = '" . ref($element) . "'\n" if !$quiet_mode && $verbosity eq 'debug';
                my $quantified = {
                    type => 'quantified',
                    element => $element,
                    quantifier => extract_token_value($next_element)
                };
                print STDERR "DEBUG: Created quantified = " . Dumper($quantified) . "\n" if !$quiet_mode && $verbosity eq 'debug';
                push @processed_elements, $quantified;
            }
            
            # Skip both current element and quantifier token
            $i += 2;
        } else {
            # Regular element
            if (is_grouped_alternative($element)) {
                # Process grouped content recursively (non-quantified groups)
                my $group_content = $element->[1];
                my $processed_group = process_parentheses_in_sequence($group_content);
                push @processed_elements, ['GROUPED', $processed_group];
            } else {
                # Convert array elements to hash format
                if (ref($element) eq 'ARRAY') {
                    push @processed_elements, {
                        type => 'atom',
                        value => $element
                    };
                } else {
                    push @processed_elements, $element;
                }
            }
            $i++;
        }
    }
    
    my $result = {
        type => 'sequence',
        elements => \@processed_elements
    };
    
    # Preserve return annotation if present
    $result->{return_annotation} = $return_annotation if $return_annotation;
    
    return $result;
}

sub is_quantifier {
    my ($token) = @_;
    # Handle both old 'quantifier' token type and new 'operator' token type for quantifiers
    return (ref($token) eq 'ARRAY' && 
           ($token->[0] eq 'quantifier' || 
            ($token->[0] eq 'operator' && 
             ($token->[1] eq '*' || $token->[1] eq '+' || $token->[1] eq '?'))
           ));
}

sub step5_build_tree_structure {
    my ($step4_result) = @_;
    my %grammar_tree = ();
    my @rule_order = ();
    
    # DEBUG: Track dot_path rule input to step 5
    print STDERR "\n🔍 STEP 5 DEBUG: Looking for dot_path rule in input...\n" unless $quiet_mode;
    my $dot_path_found_input = 0;
    foreach my $rule (@$step4_result) {
        if ($rule->{name} eq 'dot_path') {
            $dot_path_found_input = 1;
            print STDERR "🎯 STEP 5: Found dot_path rule in input: " . Dumper($rule) . "\n" unless $quiet_mode;
            last;
        }
    }
    print STDERR "❌ STEP 5: dot_path rule NOT found in input!\n" unless $quiet_mode || $dot_path_found_input;
    
    # First pass: collect all rule names for validation
    my %all_rules = ();
    foreach my $rule (@$step4_result) {
        $all_rules{$rule->{name}} = 1;
        push @rule_order, $rule->{name};
    }
    
    # Second pass: collect all referenced rule names to prevent unwrapping
    my %referenced_rules = ();
    foreach my $rule (@$step4_result) {
        my @refs = collect_referenced_rules($rule);
        foreach my $ref (@refs) {
            $referenced_rules{$ref} = 1;
        }
    }
    
    # Validate grammar completeness
    validate_grammar_completeness($step4_result, \%all_rules);
    
    # Second pass: build the tree structure
    foreach my $rule (@$step4_result) {
        my $rule_name = $rule->{name};
        my $tree_node = build_sequence_elements($rule, \%all_rules, \%referenced_rules);
        $grammar_tree{$rule_name} = $tree_node;
    }
    
    # DEBUG: Check if dot_path made it through step 5
    my $dot_path_found_output = 0;
    if (exists $grammar_tree{dot_path}) {
        $dot_path_found_output = 1;
        print STDERR "✅ STEP 5: dot_path rule found in output: " . Dumper($grammar_tree{dot_path}) . "\n" unless $quiet_mode;
    }
    print STDERR "❌ STEP 5: dot_path rule LOST in step 5!\n" unless $quiet_mode || $dot_path_found_output;
    
    return (\%grammar_tree, \@rule_order);
}

sub validate_grammar_completeness {
    my ($rules, $all_rules) = @_;
    my @referenced_rules = ();
    
    foreach my $rule (@$rules) {
        my @refs = collect_referenced_rules($rule);
        push @referenced_rules, @refs;
    }
    
    # Check for undefined rule references
    my @undefined_rules = ();
    foreach my $ref (@referenced_rules) {
        unless (exists $all_rules->{$ref}) {
            push @undefined_rules, $ref;
        }
    }
    
    if (@undefined_rules) {
        my $undefined = join(', ', @undefined_rules);
        print STDERR "WARNING: Referenced but undefined rules: $undefined\n";
    }
}

sub collect_referenced_rules {
    my ($rule) = @_;
    my @references = ();
    
    if ($rule->{type} eq 'or') {
        foreach my $alt (@{$rule->{alternatives}}) {
            push @references, collect_referenced_rules($alt);
        }
    } elsif ($rule->{type} eq 'sequence') {
        foreach my $element (@{$rule->{elements}}) {
            if (ref($element) eq 'ARRAY') {
                if ($element->[0] eq 'GROUPED') {
                    # Process grouped content
                    foreach my $sub_element (@{$element->[1]}) {
                        if (ref($sub_element) eq 'ARRAY' && $sub_element->[0] eq 'rule') {
                            push @references, $sub_element->[1];
                        }
                    }
                } elsif ($element->[0] eq 'rule') {
                    push @references, $element->[1];
                }
            } elsif (ref($element) eq 'HASH') {
                if ($element->{type} eq 'quantified') {
                    if (ref($element->{element}) eq 'ARRAY' && $element->{element}->[0] eq 'rule') {
                        push @references, $element->{element}->[1];
                    }
                }
            }
        }
    }
    
    return @references;
}

sub build_sequence_elements {
    my ($rule, $all_rules, $referenced_rules) = @_;
    
    if ($rule->{type} eq 'or') {
        # OR rule - build alternatives
        my @alternatives = ();
        foreach my $alternative (@{$rule->{alternatives}}) {
            my $alt_node = build_sequence_elements($alternative, $all_rules, $referenced_rules);
            
            # Check for return annotations in the elements
            my ($return_annotation, $filtered_elements) = extract_return_annotation($alternative->{elements});
            if ($return_annotation) {
                $alt_node->{return_annotation} = $return_annotation;
                $alt_node->{elements} = $filtered_elements if $alt_node->{type} eq 'sequence';
            }
            
            push @alternatives, $alt_node;
        }
        
        return {
            type => 'or',
            alternatives => \@alternatives
        };
    } elsif ($rule->{type} eq 'sequence') {
        # Sequence rule - process elements
        my ($return_annotation, $filtered_elements) = extract_return_annotation($rule->{elements});
        
        # Use the rule's own return_annotation if it exists, otherwise use extracted one
        if ($rule->{return_annotation}) {
            $return_annotation = $rule->{return_annotation};
        }
        
        # Check if this rule is referenced by other rules - if so, DON'T unwrap single elements
        my $rule_name = $rule->{name};
        my $is_referenced = 0;
        if (defined $rule_name && $referenced_rules) {
            $is_referenced = exists $referenced_rules->{$rule_name};
        }
        
        if (@$filtered_elements == 1 && !$is_referenced) {
            # Single element sequence AND not referenced by other rules - unwrap to atom
            my $element = $filtered_elements->[0];
            my $atom_node = process_single_element($element, $all_rules, $referenced_rules);
            # Properly attach return annotation to the node
            if ($return_annotation) {
                $atom_node->{return_annotation} = $return_annotation;
            }
            return $atom_node;
        } else {
            # Multi-element sequence OR referenced by other rules - keep as sequence
            my @processed_elements = ();
            foreach my $element (@$filtered_elements) {
                my $processed = process_single_element($element, $all_rules, $referenced_rules);
                push @processed_elements, $processed;
            }
            
            my $seq_node = {
                type => 'sequence',
                elements => \@processed_elements
            };
            $seq_node->{return_annotation} = $return_annotation if $return_annotation;
            return $seq_node;
        }
    } else {
        # Other types (shouldn't happen in normal cases)
        return $rule;
    }
}

sub extract_return_annotation {
    my ($elements) = @_;
    my @filtered_elements = ();
    my $return_annotation = undef;
    
    # Handle case where elements might not be an array reference
    if (!defined $elements || ref($elements) ne 'ARRAY') {
        return (undef, []);
    }
    
    foreach my $element (@$elements) {
        if (is_return_annotation($element)) {
            # This is a return annotation
            $return_annotation = $element;
        } else {
            # Regular element
            push @filtered_elements, $element;
        }
    }
    
    return ($return_annotation, \@filtered_elements);
}

sub process_single_element {
    my ($element, $all_rules, $referenced_rules) = @_;
    
    if (ref($element) eq 'HASH' && $element->{type} eq 'quantified') {
        # Quantified element
        my $inner_element = process_single_element($element->{element}, $all_rules, $referenced_rules);
        return {
            type => 'quantified',
            element => $inner_element,
            quantifier => $element->{quantifier}
        };
    } elsif (ref($element) eq 'ARRAY' && $element->[0] eq 'GROUPED') {
        # Grouped element - process as mini-sequence or OR structure
        my $group_content = $element->[1];
        
        # Check if the group content is an OR structure created by process_nested_or_alternatives
        if (ref($group_content) eq 'HASH' && $group_content->{type} eq 'or') {
            # This is an OR structure - process its alternatives
            my @processed_alternatives = ();
            foreach my $alternative (@{$group_content->{alternatives}}) {
                my $processed_alt = build_sequence_elements($alternative, $all_rules, $referenced_rules);
                push @processed_alternatives, $processed_alt;
            }
            
            return {
                type => 'or',
                alternatives => \@processed_alternatives
            };
        } else {
            # Regular grouped content - process as sequence
            my ($return_annotation, $filtered_elements) = extract_return_annotation($group_content);
            
            if (@$filtered_elements == 1) {
                # Single element in group
                my $inner = process_single_element($filtered_elements->[0], $all_rules, $referenced_rules);
                $inner->{return_annotation} = $return_annotation if $return_annotation;
                return $inner;
            } else {
                # Multi-element group
                my @processed_elements = ();
                foreach my $sub_element (@$filtered_elements) {
                    push @processed_elements, process_single_element($sub_element, $all_rules, $referenced_rules);
                }
                
                my $group_node = {
                    type => 'sequence',
                    elements => \@processed_elements
                };
                $group_node->{return_annotation} = $return_annotation if $return_annotation;
                return $group_node;
            }
        }
    } else {
        # Atomic element
        # Check if this is already a structured element with a return annotation
        if (ref($element) eq 'HASH' && $element->{type} eq 'atom' && $element->{return_annotation}) {
            # Already has a return annotation, just return it
            return $element;
        }
        return {
            type => 'atom',
            value => $element
        };
    }
}

sub is_return_annotation {
    my ($value) = @_;
    # Check if value is a return annotation (rule metadata, not grammar symbol)
    return ref($value) eq 'ARRAY' && ($value->[0] eq 'return_scalar' || $value->[0] eq 'return_array' || $value->[0] eq 'return_object');
}

sub is_return_annotation_string {
    my ($value) = @_;
    # Check if value is a string that looks like a return annotation
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

sub step6_generate_parser_code {
    my ($grammar_tree, $rule_order) = @_;
    
    # Apply left-recursion elimination
    my ($transformed_grammar, $final_rule_order) = AST::LeftRecursion::eliminate_left_recursion($grammar_tree, $rule_order);
    
    # Generate parser module
    my $result = generate_parser_module($transformed_grammar, $final_rule_order);
    
    return $result;
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
    
    # Generate the .pm module (package name will be replaced later)
    my $module = <<"EOF";
package PACKAGE_NAME_PLACEHOLDER; # Placeholder, will be replaced by tools/ast_transform.pl
use strict;
use warnings;

# Compiled regex patterns for speed
our \%REGEXES = (
$regex_definitions
);

# Runtime helper functions
sub quantified_match {
    my (\$input, \$regex, \$min, \$max) = \@_;
    my \$count = 0;
    my \$pos = pos(\$\$input);
    
    # Optimized: Pre-compile regex with cache
    my \$compiled_regex = qr/\$regex/o;
    
    # Optimized: Tighter loop with fewer operations
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
    
    # Optimized: Pre-allocate array for better performance
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
        # Optimized: Trim array to actual size
        \$#results = \$count - 1;
        return \\\@results;
    } else {
        # Restore position on failure
        pos(\$\$input) = \$checkpoint;
        return undef;
    }
}

sub collect_quantified_results {
    my (\$element_num, \$results_ref) = \@_;
    my \$element_index = \$element_num - 1;
    
    # If the element is a quantified result (array), return it
    # If it's undef (zero matches), return empty array
    # Otherwise return single element in array
    my \$element = \$results_ref->[\$element_index];
    
    if (!defined \$element) {
        return [];
    } elsif (ref(\$element) eq 'ARRAY') {
        return \$element;
    } else {
        return [\$element];
    }
}

$subroutines

# Main entry point
sub parse {
    my (\$input) = \@_;
    pos(\$\$input) = 0;
    return parse_$main_rule(\$input);
}

1;
EOF

    # Generate the .pl wrapper script
    my $wrapper = <<"EOF";
#!/usr/bin/env perl
use strict;
use warnings;
use FindBin qw(\$RealBin);
use lib \$RealBin;
use PACKAGE_NAME_PLACEHOLDER; # Placeholder, will be replaced by tools/ast_transform.pl
use Getopt::Long;
use Pod::Usage;

# Command-line options
my \%options = (
    input_file => '',
    output_file => '',
    pretty => 0,
    help => 0,
);

GetOptions(
    'input|i=s'    => \\\$options{input_file},
    'output|o=s'   => \\\$options{output_file},
    'pretty|p'     => \\\$options{pretty},
    'help|h'       => \\\$options{help},
) or pod2usage(2);

pod2usage(1) if \$options{help};

# Get input file from positional argument or --input option
my \$input_file = \$options{input_file} || \$ARGV[0];
pod2usage("Error: No input file specified") unless \$input_file;

# Read input file
open my \$fh, '<', \$input_file or die "Cannot open \$input_file: \$!";
my \$content = do { local \$/; <\$fh> };
close \$fh;

# Parse the content
my \$result = PACKAGE_NAME_PLACEHOLDER::parse(\\\$content); # Placeholder, will be replaced by tools/ast_transform.pl

# Output result
if (\$result) {
    if (\$options{output_file}) {
        open my \$out_fh, '>', \$options{output_file} or die "Cannot write to \$options{output_file}: \$!";
        if (\$options{pretty}) {
            require Data::Dumper;
            print \$out_fh Data::Dumper->Dump([\$result], ['result']);
        } else {
            print \$out_fh "\$result\\n";
        }
        close \$out_fh;
        print STDERR "✅ Parse result written to: \$options{output_file}\\n";
    } else {
        if (\$options{pretty}) {
            require Data::Dumper;
            print Data::Dumper->Dump([\$result], ['result']);
        } else {
            print "\$result\\n";
        }
    }
} else {
    print STDERR "❌ Parse failed\\n";
    exit 1;
}

__END__

=head1 NAME

generated_parser.pl - Parse input using generated parser

=head1 SYNOPSIS

generated_parser.pl [options] input_file

Options:
  -i, --input     Input file to parse
  -o, --output    Output file for parse result
  -p, --pretty    Pretty-print output using Data::Dumper
  -h, --help      Show this help message

=head1 DESCRIPTION

This script uses the generated parser to parse input files and output the results.

=cut
EOF

    return { module => $module, wrapper => $wrapper, main_rule => $main_rule };
}

sub generate_fast_parser_sub {
    my ($rule_name, $rule_def) = @_;
    my @regexes = ();
    
    # DEBUG: Track what happens to index_list specifically
    if ($rule_name eq 'index_list' && !$quiet_mode && $verbosity eq 'debug') {
        print STDERR "DEBUG: Generating parser for index_list with rule_def:\n";
        print STDERR Dumper($rule_def);
    }
    
    my $type = $rule_def->{type};
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
    print STDERR "DEBUG: Entered generate_or_parser for $rule_name\n" if !$quiet_mode && $verbosity eq 'debug';
    
    # Check if all alternatives are pure literals (optimization opportunity)
    my @literal_alternatives = ();
    my $all_literals = 1;
    
    # Apply YOUR OPTIMIZATION RULE:
    # Optimize ONLY if ALL alternatives are (literals OR regexes) AND NONE have return annotations
    foreach my $alt (@{$rule_def->{alternatives}}) {

        
        # Check if this alternative has a return annotation - if so, CANNOT optimize
        if ($alt->{return_annotation}) {
            print STDERR "DEBUG: Found return annotation - disabling optimization\n" if !$quiet_mode && $verbosity eq 'debug';
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
                    my $alt_value = $alt->{value};
                    my $literal;
                    
                    # Handle both new hash format and legacy array format
                    if (ref($alt_value) eq 'HASH' && $alt_value->{type} eq 'atom' && ref($alt_value->{value}) eq 'ARRAY') {
                        # New hash format: {type => 'atom', value => ['quoted_string', 'literal']}
                        $literal = $alt_value->{value}->[1];
                    } elsif (ref($alt_value) eq 'ARRAY') {
                        # Legacy array format: ['quoted_string', 'literal']
                        $literal = $alt_value->[1];
                    } else {
                        # Fallback
                        $literal = $alt_value;
                    }
                    
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
            
            # Debug: check what we're working with
            unless (ref($alt->{elements}) eq 'ARRAY') {
                print STDERR "ERROR: alt->{elements} is not an array reference: " . Dumper($alt->{elements}) . "\n";
                next;
            }
            
            foreach my $element (@{$alt->{elements}}) {
                # Debug: check element structure
                unless (ref($element) eq 'HASH') {
                    print STDERR "ERROR: element is not a hash reference: " . Dumper($element) . "\n";
                    next;
                }
                
                if ($element->{type} eq 'atom') {
                    my $element_value = $element->{value};
                    if (is_terminal($element->{value})) {
                        # Handle both new hash format and legacy array format
                        if (ref($element_value) eq 'HASH' && $element_value->{type} eq 'atom' && ref($element_value->{value}) eq 'ARRAY') {
                            # New hash format: {type => 'atom', value => ['regex', 'pattern']}
                            if ($element_value->{value}->[0] eq 'regex') {
                                # Regex pattern
                                my $pattern = $element_value->{value}->[1];
                                my $step_regex = "${rule_name}_alt${alt_num}_" . @seq_steps;
                                push @$regexes, "    '$step_regex' => qr/$pattern/o";
                                push @seq_steps, "\$\$input =~ /\\G\$REGEXES{'$step_regex'}/gc";
                            } else {
                                # Literal terminal
                                my $literal = $element_value->{value}->[1];
                                my $step_regex = "${rule_name}_alt${alt_num}_" . @seq_steps;
                                if ($literal =~ m{/}) {
                                    push @$regexes, "    '$step_regex' => qr{\\Q$literal\\E}o";
                                } else {
                                    my $escaped_literal = escape_regex_literal($literal);
                            push @$regexes, "    '$step_regex' => qr/$escaped_literal/o";
                                }
                                push @seq_steps, "\$\$input =~ /\\G\$REGEXES{'$step_regex'}/gc";
                            }
                        } elsif (ref($element_value) eq 'ARRAY') {
                            # Legacy array format: ['regex', 'pattern']
                            if ($element_value->[0] eq 'regex') {
                                # Regex pattern
                                my $pattern = $element_value->[1];
                                my $step_regex = "${rule_name}_alt${alt_num}_" . @seq_steps;
                                push @$regexes, "    '$step_regex' => qr/$pattern/o";
                                push @seq_steps, "\$\$input =~ /\\G\$REGEXES{'$step_regex'}/gc";
                            } else {
                                # Literal terminal
                                my $literal = $element_value->[1];
                                my $step_regex = "${rule_name}_alt${alt_num}_" . @seq_steps;
                                if ($literal =~ m{/}) {
                                    push @$regexes, "    '$step_regex' => qr{\\Q$literal\\E}o";
                                } else {
                                    my $escaped_literal = escape_regex_literal($literal);
                            push @$regexes, "    '$step_regex' => qr/$escaped_literal/o";
                                }
                                push @seq_steps, "\$\$input =~ /\\G\$REGEXES{'$step_regex'}/gc";
                            }
                        }
                    } else {
                        my $rule_name_to_call = extract_token_value($element->{value});
                        push @seq_steps, "parse_$rule_name_to_call(\$input)";
                    }
                } elsif ($element->{type} eq 'quantified') {
                    # Handle quantified elements in sequences
                    my $quant_code = generate_quantified_code($element, "${rule_name}_alt${alt_num}", scalar(@seq_steps), $regexes);
                    push @seq_steps, $quant_code;
                }
            }
            # Join sequence steps with proper backtracking
            my $sequence_code = "do { my \$seq_pos = pos(\$\$input); ";
            for my $step (@seq_steps) {
                $sequence_code .= "($step) && ";
            }
            
            # For sequence alternatives, we need to handle return values properly
            # Just return success for now - complex return handling will be done in sequence parser
            $sequence_code .= "1 || (pos(\$\$input) = \$seq_pos, 0) }";
            
            # Store the return annotation for later processing if needed
            if ($alt->{return_annotation}) {
                # Return annotations for sequence alternatives are complex
                # For now, just ensure we have valid syntax
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
    
    print STDERR "DEBUG: Entered generate_sequence_parser for $rule_name\n" if !$quiet_mode && $verbosity eq 'debug';
    
    my @sequence_steps = ();
    my $return_annotation = $rule_def->{return_annotation};
    
    # Use the elements directly (return annotation already extracted in Step 5)
    my @filtered_elements = @{$rule_def->{elements}};
    
    # Check if this is a pure literal sequence that can be optimized
    my @literal_parts = ();
    my $can_optimize = 1;
    
    foreach my $element (@filtered_elements) {
        if ($element->{type} eq 'atom' && is_terminal($element->{value})) {
            my $element_value = $element->{value};
            # Handle both new hash format and legacy array format
            if (ref($element_value) eq 'HASH' && $element_value->{type} eq 'atom' && ref($element_value->{value}) eq 'ARRAY') {
                # New hash format: {type => 'atom', value => ['regex', 'pattern']}
                if ($element_value->{value}->[0] eq 'regex') {
                    # Regex pattern - breaks literal optimization
                    $can_optimize = 0;
                    last;
                } else {
                    # Terminal literal - can be part of optimized regex
                    push @literal_parts, $element_value->{value}->[1];
                }
            } elsif (ref($element_value) eq 'ARRAY') {
                # Legacy array format: ['regex', 'pattern'] or ['quoted_string', 'value']
                if ($element_value->[0] eq 'regex') {
                    # Regex pattern - breaks literal optimization
                    $can_optimize = 0;
                    last;
                } else {
                    # Terminal literal - can be part of optimized regex
                    push @literal_parts, $element_value->[1];
                }
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
        # Fall back to individual element processing with proper quantifier handling
        print STDERR "DEBUG generate_sequence_rule: processing sequence with filtered_elements=" . Dumper(\@filtered_elements) . "\n" if !$quiet_mode && $verbosity eq 'debug';
        
        # Check if this sequence ends with a * quantifier applied to a grouped expression
        my $has_grouped_quantifier = 0;
        print STDERR "DEBUG: Checking for grouped quantifier, filtered_elements count: " . @filtered_elements . "\n" if !$quiet_mode && $verbosity eq 'debug';
        if (@filtered_elements >= 7) {
            my $last_element = $filtered_elements[-1];
            my $second_last_element = $filtered_elements[-2];
            
            print STDERR "DEBUG: Last element: " . Dumper($last_element) . "\n" if !$quiet_mode && $verbosity eq 'debug';
            print STDERR "DEBUG: Second last element: " . Dumper($second_last_element) . "\n" if !$quiet_mode && $verbosity eq 'debug';
            
            # Check if last element is * quantifier and second last is group_close
            if ($last_element->{type} eq 'atom' && 
                ref($last_element->{value}) eq 'HASH' && 
                ref($last_element->{value}->{value}) eq 'ARRAY' && 
                $last_element->{value}->{value}->[0] eq 'operator' && 
                $last_element->{value}->{value}->[1] eq '*' &&
                $second_last_element->{type} eq 'atom' && 
                ref($second_last_element->{value}) eq 'HASH' && 
                ref($second_last_element->{value}->{value}) eq 'ARRAY' && 
                $second_last_element->{value}->{value}->[0] eq 'group_close') {
                
                $has_grouped_quantifier = 1;
                print STDERR "DEBUG: Detected grouped quantifier pattern in sequence\n" if !$quiet_mode && $verbosity eq 'debug';
            } else {
                print STDERR "DEBUG: Grouped quantifier pattern not detected\n" if !$quiet_mode && $verbosity eq 'debug';
                print STDERR "DEBUG: last_element->{type} eq 'atom': " . ($last_element->{type} eq 'atom' ? 'true' : 'false') . "\n" if !$quiet_mode && $verbosity eq 'debug';
                print STDERR "DEBUG: ref(last_element->{value}) eq 'HASH': " . (ref($last_element->{value}) eq 'HASH' ? 'true' : 'false') . "\n" if !$quiet_mode && $verbosity eq 'debug';
                
                # Only access hash elements if the value is actually a hash
                if (ref($last_element->{value}) eq 'HASH') {
                    print STDERR "DEBUG: ref(last_element->{value}->{value}) eq 'ARRAY': " . (ref($last_element->{value}->{value}) eq 'ARRAY' ? 'true' : 'false') . "\n" if !$quiet_mode && $verbosity eq 'debug';
                    if (ref($last_element->{value}->{value}) eq 'ARRAY') {
                        print STDERR "DEBUG: last_element->{value}->{value}->[0] eq 'operator': " . ($last_element->{value}->{value}->[0] eq 'operator' ? 'true' : 'false') . "\n" if !$quiet_mode && $verbosity eq 'debug';
                        print STDERR "DEBUG: last_element->{value}->{value}->[1] eq '*': " . ($last_element->{value}->{value}->[1] eq '*' ? 'true' : 'false') . "\n" if !$quiet_mode && $verbosity eq 'debug';
                    }
                }
                
                print STDERR "DEBUG: second_last_element->{type} eq 'atom': " . ($second_last_element->{type} eq 'atom' ? 'true' : 'false') . "\n" if !$quiet_mode && $verbosity eq 'debug';
                print STDERR "DEBUG: ref(second_last_element->{value}) eq 'HASH': " . (ref($second_last_element->{value}) eq 'HASH' ? 'true' : 'false') . "\n" if !$quiet_mode && $verbosity eq 'debug';
                
                # Only access hash elements if the value is actually a hash
                if (ref($second_last_element->{value}) eq 'HASH') {
                    print STDERR "DEBUG: ref(second_last_element->{value}->{value}) eq 'ARRAY': " . (ref($second_last_element->{value}->{value}) eq 'ARRAY' ? 'true' : 'false') . "\n" if !$quiet_mode && $verbosity eq 'debug';
                    if (ref($second_last_element->{value}->{value}) eq 'ARRAY') {
                        print STDERR "DEBUG: second_last_element->{value}->{value}->[0] eq 'group_close': " . ($second_last_element->{value}->{value}->[0] eq 'group_close' ? 'true' : 'false') . "\n" if !$quiet_mode && $verbosity eq 'debug';
                    }
                }
            }
        } else {
            print STDERR "DEBUG: Not enough elements for grouped quantifier check\n" if !$quiet_mode && $verbosity eq 'debug';
        }
        
        if ($has_grouped_quantifier) {
            # Generate loop-based sequence for grouped quantifier
            push @sequence_steps, generate_grouped_quantifier_sequence_loop($rule_name, \@filtered_elements, $regexes);
        } else {
            # Check if this sequence contains any quantified elements that need loop generation
            my $has_quantified_elements = 0;
            foreach my $element (@filtered_elements) {
                if ($element->{type} eq 'quantified') {
                    $has_quantified_elements = 1;
                    print STDERR "DEBUG: Found quantified element in sequence: " . Dumper($element) . "\n" if !$quiet_mode && $verbosity eq 'debug';
                    last;
                }
            }
            
            if ($has_quantified_elements) {
                # Generate loop-based sequence for quantified elements
                print STDERR "DEBUG: Generating quantified sequence loop\n" if !$quiet_mode && $verbosity eq 'debug';
                push @sequence_steps, generate_quantified_sequence_loop($rule_name, \@filtered_elements, $regexes);
            } else {
                # Generate standard step-by-step sequence
                my $step_num = 0;
                foreach my $element (@filtered_elements) {
                    $step_num++;
                    print STDERR "DEBUG generate_sequence_rule: processing element $step_num: " . Dumper($element) . "\n" if !$quiet_mode && $verbosity eq 'debug';
                    if ($element->{type} eq 'atom') {
                        my $element_value = $element->{value};
                        # Check if this is a nested atom structure
                        my $is_terminal_element = 0;
                        if (ref($element_value) eq 'HASH' && $element_value->{type} eq 'atom' && ref($element_value->{value}) eq 'ARRAY') {
                            # Nested structure: {type => 'atom', value => ['quoted_string', 'value']}
                            $is_terminal_element = is_terminal($element_value);
                        } elsif (ref($element->{value}) eq 'ARRAY') {
                            # Direct structure: ['quoted_string', 'value']
                            $is_terminal_element = is_terminal($element->{value});
                        } else {
                            # Other structure
                            $is_terminal_element = is_terminal($element->{value});
                        }
                        
                        if ($is_terminal_element) {
                            # Handle both new hash format and legacy array format
                            if (ref($element_value) eq 'HASH' && $element_value->{type} eq 'atom' && ref($element_value->{value}) eq 'ARRAY') {
                                # New hash format: {type => 'atom', value => ['regex', 'pattern']}
                                if ($element_value->{value}->[0] eq 'regex') {
                                    # Direct regex match for regex pattern
                                    my $pattern = $element_value->{value}->[1];  # Extract regex pattern
                                    my $regex_name = "${rule_name}_step${step_num}";
                                    push @$regexes, "    '$regex_name' => qr/$pattern/o";
                                    push @sequence_steps, "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
                                } elsif ($element_value->{value}->[0] eq 'GROUPED') {
                                    # GROUPED elements should be skipped/ignored - they represent parentheses grouping 
                                    # which has already been processed into the structure
                                    print STDERR "WARNING: GROUPED element should have been processed earlier: " . join(", ", @{$element_value->{value}}) . "\n";
                                    # For now, skip this element
                                    next;
                                } else {
                                    # Direct regex match for terminal literal
                                    my $literal = $element_value->{value}->[1];  # Extract terminal content
                                    my $regex_name = "${rule_name}_step${step_num}";
                                    my $escaped_literal = escape_regex_literal($literal);
                                    if ($literal =~ m{/}) {
                                        push @$regexes, "    '$regex_name' => qr{$escaped_literal}o";
                                    } else {
                                        push @$regexes, "    '$regex_name' => qr/$escaped_literal/o";
                                    }  # Escape literal
                                    push @sequence_steps, "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
                                }
                            } elsif (ref($element_value) eq 'ARRAY') {
                                # Legacy array format: ['regex', 'pattern']
                                if ($element_value->[0] eq 'regex') {
                                    # Direct regex match for regex pattern
                                    my $pattern = $element_value->[1];  # Extract regex pattern
                                    my $regex_name = "${rule_name}_step${step_num}";
                                    push @$regexes, "    '$regex_name' => qr/$pattern/o";
                                    push @sequence_steps, "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
                                } elsif ($element_value->[0] eq 'GROUPED') {
                                    # GROUPED elements should be skipped/ignored - they represent parentheses grouping 
                                    # which has already been processed into the structure
                                    print STDERR "WARNING: GROUPED element should have been processed earlier: " . join(", ", @{$element_value}) . "\n";
                                    # For now, skip this element
                                    next;
                                } else {
                                    # Direct regex match for terminal literal
                                    my $literal = $element_value->[1];  # Extract terminal content
                                    my $regex_name = "${rule_name}_step${step_num}";
                                    my $escaped_literal = escape_regex_literal($literal);
                                    if ($literal =~ m{/}) {
                                        push @$regexes, "    '$regex_name' => qr{$escaped_literal}o";
                                    } else {
                                        push @$regexes, "    '$regex_name' => qr/$escaped_literal/o";
                                    }  # Escape literal
                                    push @sequence_steps, "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
                                }
                            }
                        } elsif (ref($element_value) eq 'HASH' && $element_value->{type} eq 'atom' && ref($element_value->{value}) eq 'ARRAY' && $element_value->{value}->[0] eq 'quantified_group') {
                            # Special handling for quantified group atoms in sequences (new format)
                            my ($type, $group_content, $quantifier) = @{$element_value->{value}};
                            my $quant = parse_quantifier($quantifier);
                        } elsif (ref($element_value) eq 'ARRAY' && $element_value->[0] eq 'quantified_group') {
                            # Special handling for quantified group atoms in sequences (legacy format)
                            my ($type, $group_content, $quantifier) = @{$element_value};
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
                            print STDERR "DEBUG: element->{value} = " . Dumper($element->{value}) . "\n" if !$quiet_mode && $verbosity eq 'debug';
                            print STDERR "DEBUG: rule_name_to_call = '$rule_name_to_call'\n" if !$quiet_mode && $verbosity eq 'debug';
                            
                            # Check for invalid rule names and handle them
                            if (!defined $rule_name_to_call || $rule_name_to_call eq '' || $rule_name_to_call =~ /[^\w]/) {
                                # Invalid rule name - treat as a literal terminal
                                my $literal = ref($element->{value}) eq 'ARRAY' ? $element->{value}->[1] : $element->{value};
                                my $regex_name = "${rule_name}_step${step_num}";
                                my $escaped_literal = escape_regex_literal($literal);
                                if ($literal =~ m{/}) {
                                    push @$regexes, "    '$regex_name' => qr{$escaped_literal}o";
                                } else {
                                    push @$regexes, "    '$regex_name' => qr/$escaped_literal/o";
                                }
                                push @sequence_steps, "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
                            } else {
                                push @sequence_steps, "parse_$rule_name_to_call(\$input)";
                            }
                        }
                    } elsif ($element->{type} eq 'quantified') {
                        # Generate quantified parsing code
                        print STDERR "DEBUG: Found quantified element: " . Dumper($element) . "\n" if !$quiet_mode && $verbosity eq 'debug';
                        my $quant_code = generate_quantified_code($element, $rule_name, $step_num, $regexes);
                        print STDERR "DEBUG: Generated quantified code: $quant_code\n" if !$quiet_mode && $verbosity eq 'debug';
                        push @sequence_steps, $quant_code;
                    } elsif ($element->{type} eq 'quantified_group') {
                        # Generate quantified group parsing code  
                        print STDERR "DEBUG: Found quantified group: " . Dumper($element) . "\n" if !$quiet_mode && $verbosity eq 'debug';
                        my $group_code = generate_grouped_quantifier_code($element, $rule_name, $regexes);
                        push @sequence_steps, $group_code;
                    }
                }
            }
        }
    }
    
    # Sequence steps array is now properly populated
    
    # Generate the sequence checking code with result capture
    my @seq_lines = ();
    my $step_counter = 0;
    foreach my $step (@sequence_steps) {
        # Skip empty or undefined steps
        next unless defined $step && $step ne "" && $step !~ /^\s*$/;
        
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
        } elsif ($step =~ /^QUANTIFIED_LOOP:/) {
            # Special marker for quantified loop - extract the actual loop code
            my $loop_code = $step;
            $loop_code =~ s/^QUANTIFIED_LOOP://;
            # Only add if loop code is not empty
            if ($loop_code && $loop_code !~ /^\s*$/) {
                push @seq_lines, $loop_code;
            }
        } elsif ($step =~ /^GROUPED_QUANTIFIED_LOOP:/) {
            # Special marker for grouped quantifier loop - extract the actual loop code
            my $loop_code = $step;
            $loop_code =~ s/^GROUPED_QUANTIFIED_LOOP://;
            # Only add if loop code is not empty
            if ($loop_code && $loop_code !~ /^\s*$/) {
                push @seq_lines, $loop_code;
            }
        } else {
            # Regex match - capture the regex result
            push @seq_lines, "    unless ($step) {";
            push @seq_lines, "        pos(\$\$input) = \$start_pos;";
            push @seq_lines, "        return undef;";
            push @seq_lines, "    }";
            push @seq_lines, "    push \@results, \$1;  # Capture regex result";
        }
    }
    
    # Handle case where no sequence steps were generated
    my $seq_code;
    if (@seq_lines == 0) {
        $seq_code = "    # No sequence elements to process";
    } else {
        $seq_code = join("\n", @seq_lines);
    }
    
    # Generate return code based on annotation
    my $return_code;
    if ($return_annotation) {
        print STDERR "\n==== STAGE 2: RETURN CODE GENERATION ====\n" unless $quiet_mode;
        print STDERR "Rule: $rule_name\n" unless $quiet_mode;
        print STDERR "Return annotation input: " . Dumper($return_annotation) . "\n" unless $quiet_mode;
        
        $return_code = generate_return_code_enhanced($return_annotation, \@filtered_elements);
        
        print STDERR "Generated return code output: '$return_code'\n" unless $quiet_mode;
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

# New helper function to generate loop-based sequences for grouped quantifiers
sub generate_grouped_quantifier_sequence_loop {
    my ($rule_name, $filtered_elements, $regexes) = @_;
    
    # The structure is: regex_pattern, (comma, whitespace, regex_pattern)*
    # We need to parse the first regex, then loop for the grouped pattern
    
    my $first_element = $filtered_elements->[0];
    
    # Generate code for the first element (should be a regex pattern)
    my $first_element_code;
    if ($first_element->{type} eq 'atom' && is_terminal($first_element->{value})) {
        my $element_value = $first_element->{value};
        
        # Handle nested atom structure properly
        if (ref($element_value) eq 'HASH' && $element_value->{type} eq 'atom' && ref($element_value->{value}) eq 'ARRAY') {
            # New hash format: {type => 'atom', value => ['regex', 'pattern']}
            if ($element_value->{value}->[0] eq 'regex') {
                my $pattern = $element_value->{value}->[1];
                my $regex_name = "${rule_name}_first";
                push @$regexes, "    '$regex_name' => qr/$pattern/o";
                $first_element_code = "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
            } else {
                # Other terminal type
                my $literal = $element_value->{value}->[1];
                my $regex_name = "${rule_name}_first";
                my $escaped_literal = escape_regex_literal($literal);
                push @$regexes, "    '$regex_name' => qr/$escaped_literal/o";
                $first_element_code = "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
            }
        } elsif (ref($element_value) eq 'ARRAY' && $element_value->[0] eq 'regex') {
            # Legacy array format: ['regex', 'pattern']
            my $pattern = $element_value->[1];
            my $regex_name = "${rule_name}_first";
            push @$regexes, "    '$regex_name' => qr/$pattern/o";
            $first_element_code = "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
        } else {
            # Fallback for other terminal types
            my $rule_name_to_call = extract_token_value($element_value);
            $first_element_code = "parse_$rule_name_to_call(\$input)";
        }
    } else {
        # Non-terminal, call parser function
        my $rule_name_to_call = extract_token_value($first_element->{value});
        $first_element_code = "parse_$rule_name_to_call(\$input)";
    }
    
    # Find the quantified group element and extract its pattern
    my $quantified_element = $filtered_elements->[1]; # Should be the quantified group
    
    # Generate regex patterns for the grouped sequence elements
    my @group_patterns = ();
    if ($quantified_element->{type} eq 'quantified' && 
        ref($quantified_element->{element}) eq 'HASH' && 
        $quantified_element->{element}->{type} eq 'sequence') {
        
        my $group_elements = $quantified_element->{element}->{elements};
        my $step_counter = 0;
        
        foreach my $group_elem (@$group_elements) {
            $step_counter++;
            
            if ($group_elem->{type} eq 'atom' && is_terminal($group_elem->{value})) {
                my $elem_value = $group_elem->{value};
                
                if (ref($elem_value) eq 'ARRAY') {
                    if ($elem_value->[0] eq 'quoted_string') {
                        # Terminal like ","
                        my $literal = $elem_value->[1];
                        my $regex_name = "${rule_name}_group_step${step_counter}";
                        my $escaped_literal = escape_regex_literal($literal);
                        push @$regexes, "    '$regex_name' => qr/$escaped_literal/o";
                        push @group_patterns, "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
                    } elsif ($elem_value->[0] eq 'regex') {
                        # Regex pattern like /\s*/
                        my $pattern = $elem_value->[1];
                        # Fix common regex patterns that cause warnings
                        $pattern =~ s/\\\\s/\\s/g;  # Fix double-escaped \s
                        my $regex_name = "${rule_name}_group_step${step_counter}";
                        push @$regexes, "    '$regex_name' => qr/$pattern/o";
                        push @group_patterns, "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
                    }
                }
            } elsif ($group_elem->{type} eq 'atom' && !is_terminal($group_elem->{value})) {
                # Rule reference
                my $rule_to_call = extract_token_value($group_elem->{value});
                push @group_patterns, "my \$group_result = parse_$rule_to_call(\$input); defined \$group_result";
            }
        }
    }
    
    # Generate the complete loop structure
    my $group_matching_code = join(" && ", @group_patterns);
    
    # If no group matching code was generated, provide a fallback exit
    if (!$group_matching_code) {
        $group_matching_code = "0";
    }
    
    my $loop_structure = <<"EOF";
    # Parse first required element (regex pattern)
    unless ($first_element_code) {
        pos(\$\$input) = \$start_pos;
        return undef;
    }
    push \@results, \$1;  # Capture from first regex
    
    # Parse optional repeated grouped elements in loop
    while (1) {
        my \$loop_start_pos = pos(\$\$input);
        
        # Try to match the grouped pattern - exit loop if no match
        if (!($group_matching_code)) {
            pos(\$\$input) = \$loop_start_pos;
            last;
        }
        
        # Successfully matched the group, add captured result if any
        push \@results, \$1 if defined \$1;
    }
EOF
    
    # Debug output to see what's being generated
    print STDERR "DEBUG: first_element_code = '$first_element_code'\n" if !$quiet_mode && $verbosity eq 'debug';
    print STDERR "DEBUG: group_matching_code = '$group_matching_code'\n" if !$quiet_mode && $verbosity eq 'debug';
    
    return "GROUPED_QUANTIFIED_LOOP:" . $loop_structure;
}

# UNIVERSAL quantified sequence generator
# Handles ANY sequence of terminals/non-terminals with ANY quantifiers
sub generate_quantified_sequence_loop {
    my ($rule_name, $filtered_elements, $regexes) = @_;
    
    # Handle different quantified sequence patterns universally:
    # - Simple quantified: accessor+
    # - Mixed sequences: term ("+" term)* literal?
    # - Multiple quantified: element* rule+ "end"?
    
    my @sequence_steps = ();
    my $step_num = 0;
    
    foreach my $element (@$filtered_elements) {
        $step_num++;
        
        if ($element->{type} eq 'quantified') {
            # QUANTIFIED ELEMENT - generate quantified parsing logic
            my $quant_code = generate_universal_quantified_step(
                $element, $rule_name, $step_num, $regexes
            );
            push @sequence_steps, $quant_code if $quant_code;
            
        } elsif ($element->{type} eq 'atom') {
            # NON-QUANTIFIED ELEMENT - generate single parsing step
            my $atom_code = generate_universal_atom_step(
                $element, $rule_name, $step_num, $regexes
            );
            push @sequence_steps, $atom_code if $atom_code;
            
        } else {
            # OTHER ELEMENT TYPES (sequences, groups, etc.)
            my $other_code = generate_universal_other_step(
                $element, $rule_name, $step_num, $regexes
            );
            push @sequence_steps, $other_code if $other_code;
        }
    }
    
    # If no valid steps were generated, return empty 
    return "" unless @sequence_steps;
    
    # Combine all steps into a unified parsing sequence
    my $combined_steps = join("\n", @sequence_steps);
    
    my $loop_structure = <<"EOF";
    # Universal quantified sequence: parse all elements in order
$combined_steps
EOF
    
    return "QUANTIFIED_LOOP:" . $loop_structure;
}

# Generate parsing code for a quantified element (*, +, ?, {n,m})
sub generate_universal_quantified_step {
    my ($element, $rule_name, $step_num, $regexes) = @_;
    
    # DEBUG: Check the actual element structure
    print STDERR "DEBUG generate_universal_quantified_step: element = " . Dumper($element) . "\n" if !$quiet_mode && $verbosity eq 'debug';
    
    my $quantifier = $element->{quantifier};
    my $quant = parse_quantifier($quantifier);
    my $element_value = $element->{element};
    
    # DEBUG: Check element_value type and content
    print STDERR "DEBUG generate_universal_quantified_step: element_value ref = '" . ref($element_value) . "'\n" if !$quiet_mode && $verbosity eq 'debug';
    print STDERR "DEBUG generate_universal_quantified_step: element_value = " . Dumper($element_value) . "\n" if !$quiet_mode && $verbosity eq 'debug';
    
    # CRITICAL FIX: Check for grouped quantifiers first!
    my $grouped_info = detect_grouped_quantifier_in_element($element_value);
    if ($grouped_info && $grouped_info->{is_grouped}) {
        print STDERR "DEBUG: Detected grouped quantifier in step $step_num: " . Dumper($grouped_info) . "\n" if !$quiet_mode && $verbosity eq 'debug';
        
        # Extract the grouped elements
        my @group_elements = extract_grouped_elements($grouped_info->{group_element});
        
        if (@group_elements) {
            print STDERR "DEBUG: Extracted " . scalar(@group_elements) . " group elements\n" if !$quiet_mode && $verbosity eq 'debug';
            
            # Generate PackratParser code for grouped quantifier
            my @group_parser_code = ();
            my $group_step = 0;
            
            foreach my $group_elem (@group_elements) {
                $group_step++;
                my $parser_code = generate_element_parser_code($group_elem, "${rule_name}_group${step_num}_${group_step}", $regexes);
                push @group_parser_code, "        sub { $parser_code }" if $parser_code;
            }
            
            my $group_parsers = join(",\n", @group_parser_code);
            
            return <<"EOF";
    # Grouped quantified sequence: (...)$quantifier
    my \@group_parsers_$step_num = (
$group_parsers
    );
    my \$grouped_result_$step_num = AST::PackratParser::parse_grouped_quantified(\$input, pos(\$\$input), \\\@group_parsers_$step_num, $quant->{min}, $quant->{max});
    unless (defined \$grouped_result_$step_num) {
        pos(\$\$input) = \$start_pos;
        return undef;
    }
    push \@results, \$grouped_result_$step_num;
EOF
        }
    }
    
    # Handle different types of quantified elements
    if (ref($element_value) eq 'ARRAY' && $element_value->[0] eq 'rule_reference') {
        # Rule reference: parse_rule_name
        my $rule_to_call = $element_value->[1];
        return <<"EOF";
    # Quantified rule: $rule_to_call$quantifier
    my \$quant_result_$step_num = quantified_rule(\$input, \\&parse_$rule_to_call, $quant->{min}, $quant->{max});
    unless (defined \$quant_result_$step_num) {
        pos(\$\$input) = \$start_pos;
        return undef;
    }
    push \@results, \$quant_result_$step_num;
EOF
    } elsif (!ref($element_value) && $element_value !~ /HASH\(/) {
        # Simple rule name string
        return <<"EOF";
    # Quantified rule: $element_value$quantifier  
    my \$quant_result_$step_num = quantified_rule(\$input, \\&parse_$element_value, $quant->{min}, $quant->{max});
    unless (defined \$quant_result_$step_num) {
        pos(\$\$input) = \$start_pos;
        return undef;
    }
    push \@results, \$quant_result_$step_num;
EOF
    } elsif (ref($element_value) eq 'ARRAY' && is_terminal($element_value)) {
        # Terminal (literal/regex) quantified
        my $regex_name = "${rule_name}_quant_step${step_num}";
        if ($element_value->[0] eq 'regex') {
            my $pattern = $element_value->[1];
            push @$regexes, "    '$regex_name' => qr/$pattern/o";
        } else {
            my $literal = $element_value->[1];
            my $escaped = escape_regex_literal($literal);
            push @$regexes, "    '$regex_name' => qr/$escaped/o";
        }
        
        return <<"EOF";
    # Quantified terminal: $element_value->[1]$quantifier
    my \$quant_result_$step_num = quantified_match(\$input, \$REGEXES{'$regex_name'}, $quant->{min}, $quant->{max});
    unless (defined \$quant_result_$step_num) {
        pos(\$\$input) = \$start_pos;
        return undef;
    }
    push \@results, \$quant_result_$step_num;
EOF
    }
    
    # Enhanced fallback with detailed debugging
    print STDERR "WARNING: Unhandled quantified element in generate_universal_quantified_step:\n" if !$quiet_mode;
    print STDERR "  element_value type: " . ref($element_value) . "\n" if !$quiet_mode;
    print STDERR "  element_value: " . Dumper($element_value) . "\n" if !$quiet_mode;
    return "    # FIXED: Enhanced fallback for element type " . ref($element_value) . "\n";
}

# Generate parsing code for a non-quantified atom element
sub generate_universal_atom_step {
    my ($element, $rule_name, $step_num, $regexes) = @_;
    
    my $element_value = $element->{value};
    
    if (is_terminal($element_value)) {
        # Terminal element (literal/regex)
        my $regex_name = "${rule_name}_atom_step${step_num}";
        
        # Handle nested atom structures
        my $actual_value = $element_value;
        if (ref($element_value) eq 'HASH' && $element_value->{type} eq 'atom') {
            $actual_value = $element_value->{value};
        }
        
        if (ref($actual_value) eq 'ARRAY') {
            if ($actual_value->[0] eq 'regex') {
                my $pattern = $actual_value->[1];
                push @$regexes, "    '$regex_name' => qr/$pattern/o";
            } else {
                my $literal = $actual_value->[1];
                my $escaped = escape_regex_literal($literal);
                push @$regexes, "    '$regex_name' => qr/$escaped/o";
            }
            
            return <<"EOF";
    # Terminal: $actual_value->[1]
    unless (\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc) {
        pos(\$\$input) = \$start_pos;
        return undef;
    }
    push \@results, \$1;  # Capture terminal result
EOF
        }
    } else {
        # Non-terminal (rule reference)
        my $rule_to_call = extract_token_value($element_value);
        return <<"EOF";
    # Rule call: $rule_to_call
    my \$atom_result_$step_num = parse_$rule_to_call(\$input);
    unless (defined \$atom_result_$step_num) {
        pos(\$\$input) = \$start_pos;
        return undef;
    }
    push \@results, \$atom_result_$step_num;
EOF
    }
    
    return "";
}

# Generate parsing code for other element types (sequences, groups, etc.)
sub generate_universal_other_step {
    my ($element, $rule_name, $step_num, $regexes) = @_;
    
    # Handle sequences, groups, or other complex structures
    # For now, provide a placeholder
    return "    # TODO: Handle other element type: $element->{type}\n";
}

# Helper function to generate parser code for individual elements in grouped quantifiers
sub generate_element_parser_code {
    my ($element, $element_name, $regexes) = @_;
    
    # Handle different element types
    if (ref($element) eq 'ARRAY') {
        # Array format like ['quoted_string', ','] or ['regex', '\\s*'] or ['rule', 'expr']
        if ($element->[0] eq 'quoted_string') {
            # Terminal literal
            my $literal = $element->[1];
            my $escaped = escape_regex_literal($literal);
            push @$regexes, "    '$element_name' => qr/$escaped/o";
            return "AST::PackratParser::parse_literal(\$input_ref, pos(\$\$input_ref), '$literal')";
        } elsif ($element->[0] eq 'regex') {
            # Regex pattern
            my $pattern = $element->[1];
            push @$regexes, "    '$element_name' => qr/$pattern/o";
            return "AST::PackratParser::parse_regex(\$input_ref, pos(\$\$input_ref), qr/$pattern/)";
        } elsif ($element->[0] eq 'rule' || $element->[0] eq 'rule_reference') {
            # Rule reference
            my $rule_name = $element->[1];
            return "parse_$rule_name(\$input_ref, pos(\$\$input_ref))";
        }
    } elsif (ref($element) eq 'HASH') {
        # Hash format - check for different structures
        if ($element->{type} eq 'atom' && ref($element->{value}) eq 'ARRAY') {
            # Nested atom structure
            return generate_element_parser_code($element->{value}, $element_name, $regexes);
        } elsif ($element->{type} eq 'terminal' || $element->{type} eq 'literal') {
            # Terminal element
            my $value = $element->{value};
            my $escaped = escape_regex_literal($value);
            push @$regexes, "    '$element_name' => qr/$escaped/o";
            return "AST::PackratParser::parse_literal(\$input_ref, pos(\$\$input_ref), '$value')";
        } elsif ($element->{type} eq 'regex') {
            # Regex element
            my $pattern = $element->{value} || $element->{pattern};
            push @$regexes, "    '$element_name' => qr/$pattern/o";
            return "AST::PackratParser::parse_regex(\$input_ref, pos(\$\$input_ref), qr/$pattern/)";
        } elsif ($element->{type} eq 'rule_reference') {
            # Rule reference
            my $rule_name = $element->{rule_name} || $element->{name};
            return "parse_$rule_name(\$input_ref, pos(\$\$input_ref))";
        }
    } elsif (!ref($element)) {
        # Simple string - assume it's a rule name
        return "parse_$element(\$input_ref, pos(\$\$input_ref))";
    }
    
    # Fallback for unhandled element types
    print STDERR "WARNING: Unhandled element type in generate_element_parser_code: " . Dumper($element) . "\n" if !$quiet_mode;
    return "AST::PackratParser::parse_epsilon(\$input_ref, pos(\$\$input_ref))";
}

sub generate_atom_parser {
    my ($rule_name, $rule_def, $regexes) = @_;
    my $value = $rule_def->{value};
    my $return_annotation = $rule_def->{return_annotation};
    
    # Handle both new hash format and legacy array format
    my $actual_value = $value;
    if (ref($value) eq 'HASH' && $value->{type} eq 'atom' && ref($value->{value}) eq 'ARRAY') {
        # New hash format: {type => 'atom', value => ['quoted_string', 'value']}
        $actual_value = $value->{value};
    }
    
    if (is_terminal($value)) {
        if (ref($actual_value) eq 'ARRAY' && $actual_value->[0] eq 'epsilon') {
            # Epsilon production - always succeeds without consuming input
            my $sub_code = <<"EOF";
sub parse_$rule_name {
    my (\$input) = \@_;
    return [];  # Epsilon - empty match, always succeeds
}
EOF
            return ($sub_code, []);
        } elsif (ref($actual_value) eq 'ARRAY' && $actual_value->[0] eq 'regex') {
            # Regex pattern - use the regex directly
            my $pattern = $actual_value->[1];  # Extract regex pattern
            push @$regexes, "    '$rule_name' => qr/$pattern/o";
            
            # Generate return code
            my $return_code;
            if ($return_annotation) {
                print STDERR "DEBUG: Processing return annotation for $rule_name: " . Dumper($return_annotation) . "\n" if !$quiet_mode && $verbosity eq 'debug';
                $return_code = generate_return_code_enhanced($return_annotation, [{ type => 'atom', value => $actual_value }]);
                print STDERR "DEBUG: Generated return code for $rule_name: '$return_code'\n" if !$quiet_mode && $verbosity eq 'debug';
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
            my $literal = $actual_value->[1];  # Extract terminal content
            
            # Handle special cases
            if (!defined $literal || $literal eq '') {
                # Empty string terminal - always matches without consuming input
                my $sub_code = <<"EOF";
sub parse_$rule_name {
    my (\$input) = \@_;
    return 1;  # Empty string always matches
}
EOF
                return ($sub_code, []);
            }
            
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
    } else {
        # Rule reference - call the appropriate parser function
        my $return_code;
        if ($return_annotation) {
            $return_code = generate_return_code_enhanced($return_annotation, [{ type => 'atom', value => $actual_value }]);
        } else {
            $return_code = "return \$result;";
        }
        
        # Extract the actual rule name to call
        my $rule_to_call;
        if (ref($actual_value) eq 'ARRAY' && @$actual_value == 2) {
            # Format: ['rule_reference', 'rule_name']
            $rule_to_call = $actual_value->[1];
        } else {
            # Fallback - use the value directly
            $rule_to_call = $actual_value;
        }
        
        my $sub_code = <<"EOF";
sub parse_$rule_name {
    my (\$input) = \@_;
    my \$result = parse_$rule_to_call(\$input);
    if (defined \$result) {
        $return_code
    }
    return undef;
}
EOF
        return ($sub_code, $regexes);
    }
}

sub is_terminal {
    my ($value) = @_;
    # Check if value is explicitly marked as terminal type
    # Handle both new hash format and legacy array format
    if (ref($value) eq 'HASH' && $value->{type} eq 'atom' && ref($value->{value}) eq 'ARRAY') {
        # New hash format: {type => 'atom', value => ['quoted_string', 'value']}
        return ($value->{value}->[0] eq 'quoted_string' || $value->{value}->[0] eq 'number' || 
                $value->{value}->[0] eq 'regex' || $value->{value}->[0] eq 'operator' || 
                $value->{value}->[0] eq 'GROUPED' || $value->{value}->[0] eq 'epsilon' ||
                $value->{value}->[0] eq 'group_open' || $value->{value}->[0] eq 'group_close');
    } elsif (ref($value) eq 'ARRAY') {
        # Legacy array format: ['quoted_string', 'value']
        return ($value->[0] eq 'quoted_string' || $value->[0] eq 'number' || 
                $value->[0] eq 'regex' || $value->[0] eq 'operator' || 
                $value->[0] eq 'GROUPED' || $value->[0] eq 'epsilon' ||
                $value->[0] eq 'group_open' || $value->[0] eq 'group_close');
    }
    return 0;
}

# New function to handle quantified sequences (grouped quantifiers)
sub generate_quantified_sequence_code {
    my ($element, $rule_name, $step_num, $regexes) = @_;
    my $quant = parse_quantifier($element->{quantifier});
    my $sequence = $element->{element};
    
    print STDERR "DEBUG generate_quantified_sequence_code: element=" . Dumper($element) . "\n" if !$quiet_mode && $verbosity eq 'debug';
    
    # Generate code to match the sequence pattern quantified times
    my @seq_conditions = ();
    my $seq_step_num = 0;
    
    foreach my $seq_element (@{$sequence->{elements}}) {
        $seq_step_num++;
        
        if ($seq_element->{type} eq 'atom' && is_terminal($seq_element->{value})) {
            my $element_value = $seq_element->{value};
            
            if (ref($element_value) eq 'HASH' && $element_value->{type} eq 'atom' && ref($element_value->{value}) eq 'ARRAY') {
                # New hash format
                if ($element_value->{value}->[0] eq 'regex') {
                    my $pattern = $element_value->{value}->[1];
                    my $regex_name = "${rule_name}_quantseq${step_num}_${seq_step_num}";
                    push @$regexes, "    '$regex_name' => qr/$pattern/o";
                    push @seq_conditions, "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
                } else {
                    # Literal terminal
                    my $literal = $element_value->{value}->[1];
                    my $regex_name = "${rule_name}_quantseq${step_num}_${seq_step_num}";
                    my $escaped_literal = escape_regex_literal($literal);
                    push @$regexes, "    '$regex_name' => qr/$escaped_literal/o";
                    push @seq_conditions, "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
                }
            } elsif (ref($element_value) eq 'ARRAY') {
                # Legacy array format
                if ($element_value->[0] eq 'regex') {
                    my $pattern = $element_value->[1];
                    my $regex_name = "${rule_name}_quantseq${step_num}_${seq_step_num}";
                    push @$regexes, "    '$regex_name' => qr/$pattern/o";
                    push @seq_conditions, "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
                } else {
                    # Literal terminal
                    my $literal = $element_value->[1];
                    my $regex_name = "${rule_name}_quantseq${step_num}_${seq_step_num}";
                    my $escaped_literal = escape_regex_literal($literal);
                    push @$regexes, "    '$regex_name' => qr/$escaped_literal/o";
                    push @seq_conditions, "\$\$input =~ /\\G\$REGEXES{'$regex_name'}/gc";
                }
            }
        } elsif ($seq_element->{type} eq 'atom' && !is_terminal($seq_element->{value})) {
            # Rule reference
            my $rule_name_to_call = extract_token_value($seq_element->{value});
            push @seq_conditions, "parse_$rule_name_to_call(\$input)";
        }
    }
    
    # Join sequence conditions
    my $sequence_condition = join(' && ', @seq_conditions);
    
    # Generate quantified matching code
    my $quantified_code = <<"EOF";
do {
    my \$count = 0;
    my \$start_pos = pos(\$\$input);
    my \@matches = ();
    
    # Try to match the sequence pattern up to max times
    while (\$count < $quant->{max}) {
        my \$seq_pos = pos(\$\$input);
        
        if ($sequence_condition) {
            \$count++;
            push \@matches, 1;  # Record successful match
        } else {
            # Restore position and exit loop
            pos(\$\$input) = \$seq_pos;
            last;
        }
    }
    
    # Check if we met the minimum requirement
    if (\$count >= $quant->{min}) {
        \$count;  # Return count of matches
    } else {
        # Restore original position on failure
        pos(\$\$input) = \$start_pos;
        undef;
    }
}
EOF
    
    return $quantified_code;
}

sub generate_quantified_code {
    my ($element, $rule_name, $step_num, $regexes) = @_;
    my $quant = parse_quantifier($element->{quantifier});
    
    print STDERR "DEBUG generate_quantified_code: element=" . Dumper($element) . "\n" if !$quiet_mode && $verbosity eq 'debug';
    print STDERR "DEBUG generate_quantified_code: quantifier=$element->{quantifier}, parsed quant=" . Dumper($quant) . "\n" if !$quiet_mode && $verbosity eq 'debug';
    
    my $element_value = $element->{element};
    
    # Handle different element formats
    if (ref($element_value) eq 'ARRAY') {
        # Array format like ['GROUPED', [elements]] or ['rule_reference', 'name'] or ['quoted_string', 'literal']
        if ($element_value->[0] eq 'GROUPED') {
            # Grouped quantifier like ("," expression)*
            return generate_quantified_sequence_code($element, $rule_name, $step_num, $regexes);
        } elsif ($element_value->[0] eq 'rule_reference') {
            my $element_name = $element_value->[1];
            return "quantified_rule(\$input, \\&parse_$element_name, $quant->{min}, $quant->{max})";
        } elsif (is_terminal($element_value)) {
            # Terminal element like ['quoted_string', 'literal'] or ['regex', 'pattern']
            if ($element_value->[0] eq 'regex') {
                my $pattern = $element_value->[1];
                my $regex_name = "${rule_name}_quant${step_num}";
                push @$regexes, "    '$regex_name' => qr/$pattern/o";
                return "quantified_match(\$input, \$REGEXES{'$regex_name'}, $quant->{min}, $quant->{max})";
            } else {
                # Literal terminal
                my $literal = $element_value->[1];
                my $regex_name = "${rule_name}_quant${step_num}";
                my $escaped_literal = escape_regex_literal($literal);
                push @$regexes, "    '$regex_name' => qr/$escaped_literal/o";
                return "quantified_match(\$input, \$REGEXES{'$regex_name'}, $quant->{min}, $quant->{max})";
            }
        }
    } elsif (ref($element_value) eq 'HASH') {
        # Hash format like {type => 'atom', value => [...]} or {type => 'sequence', elements => [...]}
        if ($element_value->{type} eq 'atom' && ref($element_value->{value}) eq 'ARRAY') {
            # Nested hash format: {type => 'atom', value => ['regex', 'pattern']}
            if (is_terminal($element_value->{value})) {
                if ($element_value->{value}->[0] eq 'regex') {
                    my $pattern = $element_value->{value}->[1];
                    my $regex_name = "${rule_name}_quant${step_num}";
                    push @$regexes, "    '$regex_name' => qr/$pattern/o";
                    return "quantified_match(\$input, \$REGEXES{'$regex_name'}, $quant->{min}, $quant->{max})";
                } else {
                    # Literal terminal
                    my $literal = $element_value->{value}->[1];
                    my $regex_name = "${rule_name}_quant${step_num}";
                    my $escaped_literal = escape_regex_literal($literal);
                    push @$regexes, "    '$regex_name' => qr/$escaped_literal/o";
                    return "quantified_match(\$input, \$REGEXES{'$regex_name'}, $quant->{min}, $quant->{max})";
                }
            } else {
                # Rule reference in hash format
                my $element_name = extract_token_value($element_value->{value});
                if (defined $element_name && $element_name !~ /HASH\(/) {
                    return "quantified_rule(\$input, \\&parse_$element_name, $quant->{min}, $quant->{max})";
                }
            }
        } elsif ($element_value->{type} eq 'sequence') {
            # Sequence quantifier
            return generate_quantified_sequence_code($element, $rule_name, $step_num, $regexes);
        } elsif ($element_value->{type} eq 'atom' && ref($element_value->{value}) eq 'HASH') {
            # Deeply nested hash format: {type => 'atom', value => {type => 'atom', value => [...]}}
            return generate_quantified_code({
                type => 'quantified',
                element => $element_value->{value},
                quantifier => $element->{quantifier}
            }, $rule_name, $step_num, $regexes);
        }
    } else {
        # Simple string or other format
        my $element_name = extract_token_value($element_value);
        if (defined $element_name && $element_name !~ /HASH\(/) {
            return "quantified_rule(\$input, \\&parse_$element_name, $quant->{min}, $quant->{max})";
        }
    }
    
    # Fallback - provide a working default instead of error
    print STDERR "WARNING: Unhandled quantified element type in generate_quantified_code: " . Dumper($element_value) . "\n" if !$quiet_mode;
    return "1";  # Fallback: assume successful match
}

sub generate_return_code_enhanced {
    my ($return_annotation, $filtered_elements) = @_;
    my ($type, $annotation) = @$return_annotation;
    
    print STDERR "DEBUG: Using universal return annotation system for: $annotation\n" unless $quiet_mode;
    
    # Parse the annotation into universal AST
    my $universal_ast = parse_annotation_to_universal_ast($annotation);
    if ($universal_ast) {
        # Use universal composition system
        my $generator = AST::PerlReturnCodeGenerator->new();
        my $composed_expression = AST::UniversalComposer::compose_return_expression(
            $universal_ast, $generator, 'results'
        );
        print STDERR "DEBUG: Universal system generated: $composed_expression\n" unless $quiet_mode;
        return "return $composed_expression;";
    }
    
    # Fallback to legacy regex-based generation
    print STDERR "DEBUG: Falling back to legacy system\n" unless $quiet_mode;
    return generate_return_code_legacy($return_annotation, $filtered_elements);
}

# Legacy regex-based return code generation (renamed for clarity)
sub generate_return_code_legacy {
    my ($return_annotation, $filtered_elements) = @_;
    my ($type, $annotation) = @$return_annotation;
    
    if ($type eq 'return_scalar') {
        # Handle $1, $2, etc.
        if ($annotation =~ /^\$(\d+)$/) {
            my $var_num = $1;
            # For simple regex captures, directly return the captured variable
            return "return \$$var_num;";
        } else {
            # For other scalar returns, use the results array
            my $var_num = $annotation;
            $var_num =~ s/\$//;  # Remove $ sign
            return "return \$results[$var_num-1];";
        }
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
        
        # Handle object property access: $1.name -> ($results[0] // {})->{name}
        $perl_hash =~ s/\$(\d+)\.(\w+)/(\$results[$1-1] \/\/ {})->{$2}/g;
        
        # Handle regular $N references with bounds checking
        $perl_hash =~ s/\$(\d+)/(\$results[$1-1] \/\/ undef)/g;
        
        return "return {$perl_hash};";
    }
    
    return "return \\\@results;  # Fallback";
}

# Ultimate return annotation parser integration with full grammar support
sub parse_return_annotation_with_ebnf {
    my ($annotation_string) = @_;
    
    # BOOTSTRAP MODE: Skip ultimate parser loading if in bootstrap mode
    if ($bootstrap_mode) {
        print STDERR "DEBUG: Bootstrap mode enabled - skipping ultimate parser, falling back to legacy\n" unless $quiet_mode;
        return undef;
    }
    
    print STDERR "DEBUG: Attempting to load ultimate return annotation parser...\n" unless $quiet_mode;
    
    # BOOTSTRAP FIX: Use conditional runtime require to avoid self-hosting cycle
    my $parser_found = 0;
    eval {
        require 'ultimate_return_annotation_perl_parser.pm';
        $parser_found = 1;
        print STDERR "DEBUG: Successfully loaded ultimate return annotation parser\n" unless $quiet_mode;
    };
    
    if ($@) {
        print STDERR "DEBUG: Failed to load ultimate return annotation parser: $@\n" unless $quiet_mode;
        print STDERR "DEBUG: This is expected during bootstrap - falling back to legacy parser\n" unless $quiet_mode;
        return undef;
    }
    
    unless ($parser_found) {
        print STDERR "DEBUG: Ultimate return annotation parser not found, falling back to legacy\n" unless $quiet_mode;
        return undef;
    }
    
    # Try to parse the annotation
    my $result;
    eval {
        print STDERR "DEBUG: Attempting to parse annotation: '$annotation_string'\n" unless $quiet_mode;
        # The generated parser uses the package name from the grammar file
        $result = ultimate_return_annotation_perl_parser::parse(\$annotation_string);
        print STDERR "DEBUG: Parse call completed, result defined: " . (defined($result) ? "yes" : "no") . "\n" unless $quiet_mode;
    };
    
    if ($@) {
        print STDERR "DEBUG: Ultimate parser FAILED for '$annotation_string': $@\n" unless $quiet_mode;
        return undef;
    }
    
    if (!defined($result)) {
        print STDERR "DEBUG: Ultimate parser returned undef for '$annotation_string'\n" unless $quiet_mode;
        return undef;
    }
    
    print STDERR "\n==== STAGE 1: RAW PARSER OUTPUT ====\n" unless $quiet_mode;
    print STDERR "Input annotation: '$annotation_string'\n" unless $quiet_mode;
    print STDERR "Raw parser result:\n" . Dumper($result) . "\n" unless $quiet_mode;
    
    # Check if result contains any JavaScript-like constructs
    my $result_str = Dumper($result);
    if ($result_str =~ /\.map\(|Math\.|\.\.\.|length/) {
        print STDERR "\n*** STAGE 1 CRITICAL: JAVASCRIPT DETECTED IN RAW PARSER OUTPUT! ***\n" unless $quiet_mode;
        print STDERR "The ultimate return annotation parser is outputting JavaScript instead of a proper AST\n" unless $quiet_mode;
    }
    
    return $result;
}

sub generate_return_code_from_ast {
    my ($ast, $filtered_elements) = @_;
    
    # Extract the actual return expression AST from the parsed structure
    # The return annotation parser produces [undef, undef, {...}] where the third element is our AST
    my $return_expr;
    if (ref($ast) eq 'ARRAY' && @$ast >= 3 && defined $ast->[2]) {
        $return_expr = $ast->[2];
    } else {
        $return_expr = $ast;  # If already unwrapped
    }
    
    # Debug output
    print STDERR "\n==== STAGE 3: AST TO CODE CONVERSION ====\n" unless $quiet_mode;
    print STDERR "Return expression AST: " . Dumper($return_expr) . "\n" unless $quiet_mode;
    print STDERR "Return expression AST type: " . (ref($return_expr) eq 'HASH' ? ($return_expr->{type} || 'NO_TYPE') : ref($return_expr)) . "\n" unless $quiet_mode;
    
    # Handle different AST node types
    if (ref($return_expr) eq 'HASH') {
        my $type = $return_expr->{type} || '';
        
        print STDERR "DEBUG: Processing AST node type: '$type'\n" unless $quiet_mode;
        
        # Handle scalar references: $1, $2
        if ($type eq 'scalar_ref') {
            my $index = $return_expr->{index};
            my $code = "return \$results[" . ($index-1) . "];";
            print STDERR "DEBUG: Generated scalar_ref code: '$code'\n" unless $quiet_mode;
            return $code;
        }
        
        # Handle simple quantified arrays: [$1*]
        elsif ($type eq 'quantified_array') {
            my $element = $return_expr->{element};
            if ($element && $element->{scalar} && $element->{scalar}{type} eq 'scalar_ref') {
                my $index = $element->{scalar}{index};
                my $code = "return collect_quantified_results($index, \\\@results);";
                print STDERR "DEBUG: Generated quantified_array code: '$code'\n" unless $quiet_mode;
                return $code;
            }
        }
        
        # Handle simple objects: {key: $1}
        elsif ($type eq 'object') {
            my $key = $return_expr->{key};
            my $value = $return_expr->{value};
            
            my $value_code;
            if (ref($value) eq 'HASH' && $value->{type} eq 'scalar_ref') {
                my $index = $value->{index};
                $value_code = "\$results[" . ($index-1) . "] // undef";
            } else {
                $value_code = "undef";
            }
            
            my $code = "return {\"$key\" => $value_code};";
            print STDERR "DEBUG: Generated object code: '$code'\n" unless $quiet_mode;
            return $code;
        }
        
        # Handle multi-property objects: {key: $1, items: [$2*]}
        elsif ($type eq 'multi_object') {
            my @props;
            
            # Process each property
            for my $i (1..10) {  # Support up to 10 properties
                my $prop_key = "prop$i";
                last unless exists $return_expr->{$prop_key};
                
                my $prop = $return_expr->{$prop_key};
                my $key = $prop->{key};
                my $value = $prop->{value};
                
                if (ref($value) eq 'HASH') {
                    my $value_type = $value->{type} || '';
                    
                    if ($value_type eq 'scalar_ref') {
                        my $index = $value->{index};
                        push @props, "\"$key\" => \$results[" . ($index-1) . "] // undef";
                    }
                    elsif ($value_type eq 'quantified_array' && $value->{element} && $value->{element}{scalar}) {
                        my $index = $value->{element}{scalar}{index};
                        push @props, "\"$key\" => collect_quantified_results($index, \\\@results)";
                    }
                    elsif ($value_type eq 'string') {
                        # Handle string literals
                        my $string_value = $value->{value};
                        push @props, "\"$key\" => \"$string_value\"";
                    }
                    # Add more value type handlers as needed
                }
            }
            
            my $code = "return {" . join(", ", @props) . "};";
            print STDERR "DEBUG: Generated multi_object code: '$code'\n" unless $quiet_mode;
            return $code;
        }
        
        # Handle unknown types - emit debug info and check structure
        else {
            print STDERR "DEBUG: Unknown AST type '$type', full AST structure:\n" unless $quiet_mode;
            print STDERR Dumper($return_expr) unless $quiet_mode;
            
            # Try to handle as a raw JavaScript/object-like structure
            if (ref($return_expr) eq 'HASH') {
                # Look for any pattern that could be JavaScript code that needs conversion
                my $ast_str = Dumper($return_expr);
                if ($ast_str =~ /\.map\(|Math\.|\[\.\.\./) {
                    print STDERR "DEBUG: DETECTED JavaScript-like constructs in AST!\n" unless $quiet_mode;
                    print STDERR "DEBUG: This is the problematic AST that contains JS code\n" unless $quiet_mode;
                    # This is the bug! The parser is outputting JavaScript instead of Perl AST
                }
            }
        }
        
        # Add more node type handlers as needed
    }
    
    # Fallback to legacy mode for unsupported AST structures
    print STDERR "DEBUG: Unsupported AST structure in return_code_from_ast, falling back to legacy\n" unless $quiet_mode;
    return "return \\\@results;  # Fallback for unsupported AST";
}

sub collect_quantified_results {
    my ($element_num, $results_ref) = @_;
    my $element_index = $element_num - 1;
    
    # If the element is a quantified result (array), return it
    # If it's undef (zero matches), return empty array
    # Otherwise return single element in array
    my $element = $results_ref->[$element_index];
    
    if (!defined $element) {
        return [];
    } elsif (ref($element) eq 'ARRAY') {
        return $element;
    } else {
        return [$element];
    }
}

sub generate_grouped_quantifier_code {
    my ($element, $rule_name, $regexes) = @_;
    
    # Simple implementation for now
    return "1";  # Placeholder
}

sub load_ebnf_spec {
    my ($ebnf_file) = @_;
    
    # Load and parse the input EBNF file
    open my $fh2, "<", $ebnf_file or die "Cannot open $ebnf_file: $!";
    my $input_content = do { local $/; <$fh2> };
    close $fh2;
    
    # Parse the content using helper function
    return load_ebnf_spec_from_content($input_content);
}

sub load_ebnf_spec_from_content {
    my ($input_content) = @_;
    
    # Load the EBNF parser specification - try different paths
    my $spec_file;
    for my $path ("fx/specs/ebnf.spec", "../fx/specs/ebnf.spec", "../../fx/specs/ebnf.spec") {
        if (-f $path) {
            $spec_file = $path;
            last;
        }
    }
    
    die "Cannot find ebnf.spec file" unless $spec_file;
    
    open my $fh, "<", $spec_file or die "Cannot open $spec_file: $!";
    my $spec_content = do { local $/; <$fh> };
    close $fh;
    
    # Create parser from the spec
    my $parser = LinkedSpec::Get(\$spec_content);
    unless ($parser) {
        die "Failed to create EBNF parser from fx/specs/ebnf.spec";
    }
    
    # Parse the EBNF content
    my $raw_ast = $parser->(\$input_content);
    unless ($raw_ast) {
        die "Failed to parse EBNF content";
    }
    
    return $raw_ast;
}

sub process_transformation_phases {
    my ($input, %options) = @_;
    
    # Set global options
    $quiet_mode = $options{quiet} // 0;
    $verbosity = $options{verbosity} // 'normal';
    $ERROR_CONTEXT->{verbosity} = $verbosity;
    
    # Handle both string content and pre-parsed AST
    my $raw_ast;
    if (ref($input) eq 'ARRAY') {
        # Already parsed AST
        $raw_ast = $input;
    } else {
        # String content - need to parse first
        $raw_ast = load_ebnf_spec_from_content($input);
        unless ($raw_ast) {
            die "Failed to parse EBNF content";
        }
    }
    
    print STDERR "\n=== Step 2: Group by OR ===\n" unless $quiet_mode;
    my $step2_result = step2_group_by_or($raw_ast);
    print STDERR "STEP 2 RESULT (OR groups):\n" . Dumper($step2_result) unless $quiet_mode;

    print STDERR "\n=== Step 2.5: Handle parentheses ===\n" unless $quiet_mode;
    my $step2_5_result = step2_5_handle_parentheses($step2_result);
    print STDERR "STEP 2.5 RESULT (Parentheses handled):\n" . Dumper($step2_5_result) unless $quiet_mode;

    print STDERR "\n=== Step 3: Parse sequences ===\n" unless $quiet_mode;
    
    # DEBUG: Track index_list before step 3
    print STDERR "DEBUG: index_list before step3:\n" if !$quiet_mode && $verbosity eq 'debug';
    foreach my $rule (@$step2_5_result) {
        if ($rule->{name} eq 'index_list') {
            print STDERR Dumper($rule) if !$quiet_mode && $verbosity eq 'debug';
            last;
        }
    }
    
    my $step3_result = step3_parse_sequences($step2_5_result);
    print STDERR "STEP 3 RESULT (Sequences parsed):\n" . Dumper($step3_result) unless $quiet_mode;
    
    # DEBUG: Track index_list after step 3
    print STDERR "DEBUG: index_list after step3:\n" if !$quiet_mode && $verbosity eq 'debug';
    foreach my $rule (@$step3_result) {
        if ($rule->{name} eq 'index_list') {
            print STDERR Dumper($rule) if !$quiet_mode && $verbosity eq 'debug';
            last;
        }
    }

    print STDERR "\n=== Step 4: Handle quantifiers ===\n" unless $quiet_mode;
    
    # DEBUG: Track index_list before step 4
    print STDERR "DEBUG: index_list before step4:\n" if !$quiet_mode && $verbosity eq 'debug';
    foreach my $rule (@$step3_result) {
        if ($rule->{name} eq 'index_list') {
            print STDERR Dumper($rule) if !$quiet_mode && $verbosity eq 'debug';
            last;
        }
    }
    
    my $step4_result = step4_handle_quantifiers($step3_result);
    print STDERR "STEP 4 RESULT (Quantifiers handled):\n" . Dumper($step4_result) unless $quiet_mode;
    
    # DEBUG: Track index_list after step 4
    print STDERR "DEBUG: index_list after step4:\n" if !$quiet_mode && $verbosity eq 'debug';
    foreach my $rule (@$step4_result) {
        if ($rule->{name} eq 'index_list') {
            print STDERR Dumper($rule) if !$quiet_mode && $verbosity eq 'debug';
            last;
        }
    }

    print STDERR "\n=== Step 5: Build tree structure ===\n" unless $quiet_mode;
    
    # DEBUG: Track index_list before step 5
    print STDERR "DEBUG: index_list before step5:\n" if !$quiet_mode && $verbosity eq 'debug';
    foreach my $rule (@$step4_result) {
        if ($rule->{name} eq 'index_list') {
            print STDERR Dumper($rule) if !$quiet_mode && $verbosity eq 'debug';
            last;
        }
    }
    
    my ($step5_result, $rule_order) = step5_build_tree_structure($step4_result);
    print STDERR "STEP 5 RESULT (Tree structure):\n" . Dumper($step5_result) unless $quiet_mode;
    print STDERR "RULE ORDER: " . join(", ", @$rule_order) . "\n" unless $quiet_mode;
    
    # DEBUG: Track index_list after step 5
    print STDERR "DEBUG: index_list after step5:\n" if !$quiet_mode && $verbosity eq 'debug';
    if (exists $step5_result->{index_list}) {
        print STDERR Dumper($step5_result->{index_list}) if !$quiet_mode && $verbosity eq 'debug';
    } else {
        print STDERR "DEBUG: index_list NOT FOUND in step5 result\n" if !$quiet_mode && $verbosity eq 'debug';
    }
    
    # Also debug 'index' rule for comparison
    print STDERR "DEBUG: index rule after step5:\n" if !$quiet_mode && $verbosity eq 'debug';
    if (exists $step5_result->{index}) {
        print STDERR Dumper($step5_result->{index}) if !$quiet_mode && $verbosity eq 'debug';
    } else {
        print STDERR "DEBUG: index rule NOT FOUND in step5 result\n" if !$quiet_mode && $verbosity eq 'debug';
    }

    print STDERR "\n=== Step 6: Generate parser code ===\n" unless $quiet_mode;
    
    # DEBUG: Final check on what's being passed to parser generation
    print STDERR "DEBUG: Keys in step5_result before step6: " . join(", ", sort keys %$step5_result) . "\n" if !$quiet_mode && $verbosity eq 'debug';
    
    my $step6_result = step6_generate_parser_code($step5_result, $rule_order);
    
    return $step6_result;
}

sub generate_parser_from_file {
    my ($filename, %options) = @_;
    
    # Set global options
    $quiet_mode = $options{quiet} // 0;
    $verbosity = $options{verbosity} // 'normal';
    
    # Load and parse the EBNF file
    my $raw_ast = load_ebnf_spec($filename);
    unless ($raw_ast) {
        $ERROR_CONTEXT->{errors} = ["Failed to load EBNF specification"];
        return undef;
    }
    
    # Process the transformation phases
    my $result = process_transformation_phases($raw_ast, %options);
    
    return $result;
}

sub generate_parser_from_grammar {
    my ($grammar_tree, $rule_order, %options) = @_;
    
    # Set global options
    $quiet_mode = $options{quiet} // 0;
    $verbosity = $options{verbosity} // 'normal';
    
    print STDERR "\n=== Step 6: Generate parser code ===\n" unless $quiet_mode;
    my $step6_result = step6_generate_parser_code($grammar_tree, $rule_order);
    
    return $step6_result;
}

sub get_error_context {
    return $ERROR_CONTEXT;
}

1;
