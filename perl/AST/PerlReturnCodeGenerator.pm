package AST::PerlReturnCodeGenerator;

use strict;
use warnings;
use Data::Dumper;

=head1 DESCRIPTION

Perl-specific implementation of return code generation using universal composition concepts.

This generates Perl code from universal return annotation ASTs.
Other languages would have their own equivalent generators.

=cut

sub new {
    my ($class) = @_;
    return bless {}, $class;
}

=head1 CORE GENERATION METHODS

These implement the universal semantic operations for Perl.

=cut

=head2 generate_reference($index, $results_var)

Generate Perl code to reference element at position $index from $results_var.
Perl uses 0-based arrays, so $1 becomes $results[0].

=cut

sub generate_reference {
    my ($self, $index, $results_var) = @_;
    my $array_index = $index - 1;  # Convert from 1-based to 0-based
    return "\$results[$array_index]";
}

=head2 generate_array_construction($elements_ref)

Generate Perl array construction: [elem1, elem2, ...]

=cut

sub generate_array_construction {
    my ($self, $elements_ref) = @_;
    return "[" . join(", ", @$elements_ref) . "]";
}

=head2 generate_object_construction($pairs_ref)

Generate Perl hash construction: {key1 => value1, key2 => value2}
$pairs_ref contains [key, composed_value] pairs.

=cut

sub generate_object_construction {
    my ($self, $pairs_ref) = @_;
    
    my @perl_pairs = ();
    foreach my $pair (@$pairs_ref) {
        my ($key, $value) = @$pair;
        # Keys are always strings in Perl hashes
        push @perl_pairs, "\"$key\" => $value";
    }
    
    return "{" . join(", ", @perl_pairs) . "}";
}

=head2 generate_collection_expansion($index, $results_var)

Generate Perl code for simple quantified collection: [$1*]
Uses the existing collect_quantified_results helper function.

=cut

sub generate_collection_expansion {
    my ($self, $index, $results_var) = @_;
    return "collect_quantified_results($index, \\\@$results_var)";
}

=head2 generate_expansion($index, $results_var)

Generate Perl code for quantified expansion within mixed arrays: $N*
Uses array spreading with @{...}.

=cut

sub generate_expansion {
    my ($self, $index, $results_var) = @_;
    return "\@{collect_quantified_results($index, \\\@$results_var)}";
}

=head2 generate_mixed_array_construction($elements_ref)

Generate Perl code for mixed arrays where some elements are expansions.
This is tricky in Perl because @{} expansions need special handling.

=cut

sub generate_mixed_array_construction {
    my ($self, $elements_ref) = @_;
    
    # For mixed arrays with expansions, we need to flatten at runtime
    # Example: [$1, $2*] becomes [$results[0], @{collect_quantified_results(2, \@results)}]
    return "[" . join(", ", @$elements_ref) . "]";
}

=head2 generate_property_access($base_code, $path_segments)

Generate Perl code for dot notation property access.
Uses a helper function to navigate the path.

=cut

sub generate_property_access {
    my ($self, $base_code, $path_segments) = @_;
    
    # Convert path segments to Perl-friendly format
    my @perl_path = ();
    foreach my $segment (@$path_segments) {
        if ($segment->{type} eq 'property') {
            push @perl_path, "{type => 'property', name => '$segment->{name}'}";
        }
        elsif ($segment->{type} eq 'index') {
            push @perl_path, "{type => 'index', value => $segment->{value}}";
        }
        elsif ($segment->{type} eq 'slice') {
            push @perl_path, "{type => 'slice', start => $segment->{start}, end => $segment->{end}}";
        }
    }
    
    my $path_array = "[" . join(", ", @perl_path) . "]";
    return "access_dot_path($base_code, $path_array)";
}

=head2 generate_literal($value, $value_type)

Generate Perl code for literal values.

=cut

sub generate_literal {
    my ($self, $value, $value_type) = @_;
    
    if ($value_type eq 'string') {
        # Escape quotes in string literals
        my $escaped = $value;
        $escaped =~ s/\"/\\"/g;
        return "\"$escaped\"";
    }
    elsif ($value_type eq 'number') {
        return $value;  # Numbers don't need quotes
    }
    elsif ($value_type eq 'boolean') {
        # Perl doesn't have native booleans, use 1/0
        return $value eq 'true' ? '1' : '0';
    }
    else {
        # Default to string
        return "\"$value\"";
    }
}

=head2 generate_return_statement($expression, $branch_id)

Generate complete Perl return statement.

=cut

sub generate_return_statement {
    my ($self, $expression, $branch_id) = @_;
    return "return $expression;";
}

=head1 COMPLEX OPERATIONS

These handle more advanced cases.

=cut

=head2 generate_complex_collection_expansion($composed_element, $results_var)

Handle complex quantified elements that aren't simple scalar references.

=cut

sub generate_complex_collection_expansion {
    my ($self, $composed_element, $results_var) = @_;
    # For complex elements, we'd need a more sophisticated expansion function
    return "expand_complex_collection($composed_element, $results_var)";
}

=head2 generate_complex_expansion($composed_element, $results_var)

Handle complex quantified expansions within mixed arrays.

=cut

sub generate_complex_expansion {
    my ($self, $composed_element, $results_var) = @_;
    return "\@{expand_complex_collection($composed_element, \\$results_var)}";
}

=head1 DEBUGGING AND UTILITIES

=cut

=head2 trace_generation($operation, $input, $output)

Debug helper to trace code generation.

=cut

sub trace_generation {
    my ($self, $operation, $input, $output) = @_;
    print STDERR "PERL_GEN: $operation($input) -> $output\n";
    return $output;
}

1;

__END__

=head1 EXAMPLES

use AST::PerlReturnCodeGenerator;
use AST::UniversalReturnAnnotation;
use AST::UniversalComposer;

# Create generator
my $generator = AST::PerlReturnCodeGenerator->new();

# Example 1: Simple scalar reference $1
my $scalar_ast = AST::UniversalReturnAnnotation::new_scalar_ref(1);
my $perl_code = AST::UniversalComposer::compose_return_expression(
    $scalar_ast, $generator, '\\@results'
);
# Result: "$results[0]"

# Example 2: Quantified array [$1*] (for dot_path := accessor+ -> [$1*])
my $quantified_ast = AST::UniversalReturnAnnotation::new_quantified_array(
    AST::UniversalReturnAnnotation::new_scalar_ref(1)
);
my $quantified_code = AST::UniversalComposer::compose_return_expression(
    $quantified_ast, $generator, '\\@results'
);
# Result: "collect_quantified_results(1, \\@results)"

# Example 3: Object {type: "property", name: $2}
my $object_ast = AST::UniversalReturnAnnotation::new_object(
    {key => 'type', value => AST::UniversalReturnAnnotation::new_literal('property')},
    {key => 'name', value => AST::UniversalReturnAnnotation::new_scalar_ref(2)}
);
my $object_code = AST::UniversalComposer::compose_return_expression(
    $object_ast, $generator, '\\@results'
);
# Result: {"type" => "property", "name" => $results[1]}

# Example 4: Mixed array [$1, $2*]
my $mixed_ast = AST::UniversalReturnAnnotation::new_mixed_array(
    AST::UniversalReturnAnnotation::new_scalar_ref(1),
    AST::UniversalReturnAnnotation::new_quantified_collection(
        AST::UniversalReturnAnnotation::new_scalar_ref(2)
    )
);
my $mixed_code = AST::UniversalComposer::compose_return_expression(
    $mixed_ast, $generator, '\\@results'
);
# Result: [$results[0], @{collect_quantified_results(2, \\@results)}]

=cut
