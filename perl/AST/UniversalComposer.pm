package AST::UniversalComposer;

use strict;
use warnings;
use Data::Dumper;
use AST::UniversalReturnAnnotation;

=head1 DESCRIPTION

Universal composition algorithm that walks return annotation ASTs 
and generates language-agnostic semantic operations.

This is the core engine that converts return annotation ASTs into 
target language code using language-specific generators.

=cut

# ====================================================================
# CORE COMPOSITION ALGORITHM
# ====================================================================

=head2 compose_return_expression

Main entry point for composing a return expression AST into target language code.

Parameters:
- $ast_node: Return annotation AST node
- $generator: Language-specific code generator (implementing ReturnCodeGenerator interface)
- $results_var: Name of the results variable in the target language

Returns: String containing target language code

=cut

sub compose_return_expression {
    my ($ast_node, $generator, $results_var) = @_;
    
    # Validate inputs
    die "Invalid AST node" unless AST::UniversalReturnAnnotation::validate_ast($ast_node);
    die "Generator must implement ReturnCodeGenerator interface" unless $generator->can('generate_reference');
    
    my $type = $ast_node->{type};
    
    # Dispatch to appropriate composition method based on AST node type
    if ($type eq 'scalar_ref') {
        return compose_scalar_ref($ast_node, $generator, $results_var);
    }
    elsif ($type eq 'array') {
        return compose_array($ast_node, $generator, $results_var);
    }
    elsif ($type eq 'object') {
        return compose_object($ast_node, $generator, $results_var);
    }
    elsif ($type eq 'quantified_array') {
        return compose_quantified_array($ast_node, $generator, $results_var);
    }
    elsif ($type eq 'mixed_array') {
        return compose_mixed_array($ast_node, $generator, $results_var);
    }
    elsif ($type eq 'quantified_collection') {
        return compose_quantified_collection($ast_node, $generator, $results_var);
    }
    elsif ($type eq 'ultimate_dot_notation') {
        return compose_ultimate_dot_notation($ast_node, $generator, $results_var);
    }
    elsif ($type eq 'literal') {
        return compose_literal($ast_node, $generator, $results_var);
    }
    else {
        die "Unknown AST node type: $type";
    }
}

=head2 compose_scalar_ref

Compose scalar reference: $N -> results[N-1]

=cut

sub compose_scalar_ref {
    my ($ast_node, $generator, $results_var) = @_;
    
    return $generator->generate_reference($ast_node->{index}, $results_var);
}

=head2 compose_array

Compose array construction: [elem1, elem2, ...] -> [compose(elem1), compose(elem2), ...]

=cut

sub compose_array {
    my ($ast_node, $generator, $results_var) = @_;
    
    my @composed_elements = ();
    
    foreach my $element (@{$ast_node->{contents}}) {
        my $composed = compose_return_expression($element, $generator, $results_var);
        push @composed_elements, $composed;
    }
    
    return $generator->generate_array_construction(\@composed_elements);
}

=head2 compose_object

Compose object construction: {key1: value1, ...} -> {key1: compose(value1), ...}

=cut

sub compose_object {
    my ($ast_node, $generator, $results_var) = @_;
    
    my @composed_pairs = ();
    
    foreach my $pair (@{$ast_node->{contents}}) {
        my $key = $pair->{key};
        my $composed_value = compose_return_expression($pair->{value}, $generator, $results_var);
        push @composed_pairs, [$key, $composed_value];
    }
    
    return $generator->generate_object_construction(\@composed_pairs);
}

=head2 compose_quantified_array

Compose simple quantified array: [$1*] -> collect_quantified_results(1, results)

=cut

sub compose_quantified_array {
    my ($ast_node, $generator, $results_var) = @_;
    
    # The element should be a scalar_ref indicating which position to expand
    my $element = $ast_node->{element};
    
    if ($element->{type} eq 'scalar_ref') {
        return $generator->generate_collection_expansion($element->{index}, $results_var);
    } else {
        # More complex quantified element - compose it first then expand
        my $composed_element = compose_return_expression($element, $generator, $results_var);
        return $generator->generate_complex_collection_expansion($composed_element, $results_var);
    }
}

=head2 compose_mixed_array

Compose mixed array: [$1, $2*, $3] -> [results[0], *expand(2, results), results[2]]

=cut

sub compose_mixed_array {
    my ($ast_node, $generator, $results_var) = @_;
    
    my @composed_elements = ();
    
    foreach my $element (@{$ast_node->{contents}}) {
        if ($element->{type} eq 'quantified_collection') {
            # This is a $N* expansion within the mixed array
            my $expansion = compose_quantified_collection($element, $generator, $results_var);
            push @composed_elements, $expansion;
        } else {
            # Regular element
            my $composed = compose_return_expression($element, $generator, $results_var);
            push @composed_elements, $composed;
        }
    }
    
    return $generator->generate_mixed_array_construction(\@composed_elements);
}

=head2 compose_quantified_collection

Compose quantified collection expansion: $N* (used within mixed arrays)

=cut

sub compose_quantified_collection {
    my ($ast_node, $generator, $results_var) = @_;
    
    my $element = $ast_node->{element};
    
    if ($element->{type} eq 'scalar_ref') {
        return $generator->generate_expansion($element->{index}, $results_var);
    } else {
        # Complex element expansion
        my $composed_element = compose_return_expression($element, $generator, $results_var);
        return $generator->generate_complex_expansion($composed_element, $results_var);
    }
}

=head2 compose_ultimate_dot_notation

Compose dot notation access: $1.name.value[0] -> access_dot_path(results[0], path)

=cut

sub compose_ultimate_dot_notation {
    my ($ast_node, $generator, $results_var) = @_;
    
    my $base = compose_return_expression($ast_node->{base}, $generator, $results_var);
    my $path = $ast_node->{path};
    
    return $generator->generate_property_access($base, $path);
}

=head2 compose_literal

Compose literal value: "string" -> "string", 42 -> 42

=cut

sub compose_literal {
    my ($ast_node, $generator, $results_var) = @_;
    
    return $generator->generate_literal($ast_node->{value}, $ast_node->{value_type});
}

# ====================================================================
# BRANCH-SPECIFIC COMPOSITION
# ====================================================================

=head2 compose_branch_return

Compose return code for a specific branch/alternative.

Parameters:
- $branch_annotation: Branch annotation structure (from UniversalReturnAnnotation)
- $generator: Language-specific code generator
- $results_var: Name of results variable

Returns: Complete return statement for this branch

=cut

sub compose_branch_return {
    my ($branch_annotation, $generator, $results_var) = @_;
    
    my $return_ast = $branch_annotation->{return_ast};
    my $branch_id = $branch_annotation->{branch_id};
    
    # Compose the return expression
    my $composed_expression = compose_return_expression($return_ast, $generator, $results_var);
    
    # Wrap in return statement
    return $generator->generate_return_statement($composed_expression, $branch_id);
}

=head2 compose_rule_alternatives

Compose return code for all alternatives of a rule.

Parameters:
- $rule_name: Name of the rule
- $alternatives: Array of branch annotations
- $generator: Language-specific code generator
- $results_var: Name of results variable

Returns: Hash mapping alternative_index => return_code

=cut

sub compose_rule_alternatives {
    my ($rule_name, $alternatives, $generator, $results_var) = @_;
    
    my %alternative_returns = ();
    
    foreach my $branch_annotation (@$alternatives) {
        die "Branch annotation must be for rule $rule_name" 
            unless $branch_annotation->{rule_name} eq $rule_name;
        
        my $alt_index = $branch_annotation->{alternative_index};
        my $return_code = compose_branch_return($branch_annotation, $generator, $results_var);
        
        $alternative_returns{$alt_index} = $return_code;
    }
    
    return \%alternative_returns;
}

# ====================================================================
# DEBUGGING AND VALIDATION
# ====================================================================

=head2 trace_composition

Trace the composition process for debugging.

=cut

sub trace_composition {
    my ($ast_node, $generator, $results_var, $depth) = @_;
    $depth ||= 0;
    
    my $indent = '  ' x $depth;
    my $ast_str = AST::UniversalReturnAnnotation::ast_to_string($ast_node);
    
    print STDERR "${indent}Composing: $ast_str (type: $ast_node->{type})\n";
    
    my $result = compose_return_expression($ast_node, $generator, $results_var);
    
    print STDERR "${indent}Generated: $result\n";
    
    return $result;
}

1;

__END__

=head1 EXAMPLES

use AST::UniversalReturnAnnotation;
use AST::UniversalComposer;
use AST::PerlReturnCodeGenerator;  # Assuming this exists

# Example: Compose [$1*] for dot_path := accessor+ -> [$1*]
my $quantified_ast = AST::UniversalReturnAnnotation::new_quantified_array(
    AST::UniversalReturnAnnotation::new_scalar_ref(1)
);

my $generator = AST::PerlReturnCodeGenerator->new();
my $perl_code = AST::UniversalComposer::compose_return_expression(
    $quantified_ast, $generator, '\@results'
);
# Result: "collect_quantified_results(1, \@results)"

# Example: Compose {type: "property", name: $2}
my $object_ast = AST::UniversalReturnAnnotation::new_object(
    {key => 'type', value => AST::UniversalReturnAnnotation::new_literal('property')},
    {key => 'name', value => AST::UniversalReturnAnnotation::new_scalar_ref(2)}
);

my $perl_object_code = AST::UniversalComposer::compose_return_expression(
    $object_ast, $generator, '\@results'
);
# Result: "{type => \"property\", name => \$results[1]}"

=cut
