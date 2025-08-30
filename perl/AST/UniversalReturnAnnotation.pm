package AST::UniversalReturnAnnotation;

use strict;
use warnings;
use Data::Dumper;

# Universal AST data structures for return annotations
# These structures are language-agnostic and represent semantic operations

=head1 DESCRIPTION

Universal return annotation system that separates semantics from syntax.
Return annotations apply to alternatives/branches, not to rules as a whole.
Each alternative can have its own return annotation.

=cut

# ====================================================================
# AST NODE TYPES
# ====================================================================

=head2 scalar_ref

Represents a reference to a captured element: $1, $2, etc.

Structure:
{
    type => 'scalar_ref',
    index => N  # 1-based position in results array
}

=cut

sub new_scalar_ref {
    my ($index) = @_;
    return {
        type => 'scalar_ref',
        index => $index
    };
}

=head2 array

Represents array construction: [elem1, elem2, ...]

Structure:
{
    type => 'array',
    contents => [element1, element2, ...]  # Each element is a return annotation AST
}

=cut

sub new_array {
    my (@contents) = @_;
    return {
        type => 'array',
        contents => \@contents
    };
}

=head2 object

Represents object construction: {key1: value1, key2: value2}

Structure:
{
    type => 'object',
    contents => [
        {key => key1, value => value1_ast},
        {key => key2, value => value2_ast},
        ...
    ]
}

=cut

sub new_object {
    my (@pairs) = @_;  # Array of {key => $key, value => $value_ast}
    return {
        type => 'object',
        contents => \@pairs
    };
}

=head2 quantified_array

Represents quantified collection: [$1*], [$2*], etc.

Structure:
{
    type => 'quantified_array',
    element => scalar_ref_ast  # What to expand
}

=cut

sub new_quantified_array {
    my ($element_ast) = @_;
    return {
        type => 'quantified_array',
        element => $element_ast
    };
}

=head2 mixed_array

Represents mixed arrays: [$1, $2*, $3]

Structure:
{
    type => 'mixed_array',
    contents => [
        {type => 'scalar_ref', index => 1},
        {type => 'quantified_collection', element => {type => 'scalar_ref', index => 2}},
        {type => 'scalar_ref', index => 3}
    ]
}

=cut

sub new_mixed_array {
    my (@contents) = @_;
    return {
        type => 'mixed_array',
        contents => \@contents
    };
}

=head2 quantified_collection

Represents quantified expansion: $N* (used inside mixed arrays)

Structure:
{
    type => 'quantified_collection',
    element => scalar_ref_ast
}

=cut

sub new_quantified_collection {
    my ($element_ast) = @_;
    return {
        type => 'quantified_collection',
        element => $element_ast
    };
}

=head2 ultimate_dot_notation

Represents dot path access: $1.name.value[0]

Structure:
{
    type => 'ultimate_dot_notation',
    base => scalar_ref_ast,
    path => [path_segment1, path_segment2, ...]
}

Path segments:
- {type => 'property', name => 'name'}
- {type => 'index', value => 0}
- {type => 'slice', start => 1, end => 3}

=cut

sub new_ultimate_dot_notation {
    my ($base_ast, @path) = @_;
    return {
        type => 'ultimate_dot_notation',
        base => $base_ast,
        path => \@path
    };
}

=head2 literal

Represents literal values: "string", 42, true

Structure:
{
    type => 'literal',
    value => $value,
    value_type => 'string' | 'number' | 'boolean'
}

=cut

sub new_literal {
    my ($value, $value_type) = @_;
    $value_type ||= (
        $value =~ /^\d+$/ ? 'number' :
        $value =~ /^(true|false)$/ ? 'boolean' :
        'string'
    );
    
    return {
        type => 'literal',
        value => $value,
        value_type => $value_type
    };
}

# ====================================================================
# BRANCH ANNOTATION STRUCTURE
# ====================================================================

=head2 branch_annotation

Represents a return annotation for a specific alternative/branch.

Structure:
{
    branch_id => "rule_name_alt_0",  # Unique identifier for this branch
    rule_name => "accessor",         # Name of the rule this branch belongs to
    alternative_index => 0,          # Index of this alternative (0-based)
    return_ast => {...}              # The return annotation AST
}

=cut

sub new_branch_annotation {
    my ($rule_name, $alternative_index, $return_ast) = @_;
    my $branch_id = "${rule_name}_alt_${alternative_index}";
    
    return {
        branch_id => $branch_id,
        rule_name => $rule_name,
        alternative_index => $alternative_index,
        return_ast => $return_ast
    };
}

# ====================================================================
# VALIDATION AND UTILITIES
# ====================================================================

=head2 validate_ast

Validates that an AST node has the correct structure.

=cut

sub validate_ast {
    my ($ast) = @_;
    
    return 0 unless ref($ast) eq 'HASH';
    return 0 unless exists $ast->{type};
    
    my $type = $ast->{type};
    
    if ($type eq 'scalar_ref') {
        return exists $ast->{index} && $ast->{index} =~ /^\d+$/ && $ast->{index} > 0;
    }
    elsif ($type eq 'array') {
        return exists $ast->{contents} && ref($ast->{contents}) eq 'ARRAY';
    }
    elsif ($type eq 'object') {
        return exists $ast->{contents} && ref($ast->{contents}) eq 'ARRAY';
    }
    elsif ($type eq 'quantified_array') {
        return exists $ast->{element} && validate_ast($ast->{element});
    }
    elsif ($type eq 'mixed_array') {
        return exists $ast->{contents} && ref($ast->{contents}) eq 'ARRAY';
    }
    elsif ($type eq 'quantified_collection') {
        return exists $ast->{element} && validate_ast($ast->{element});
    }
    elsif ($type eq 'ultimate_dot_notation') {
        return exists $ast->{base} && exists $ast->{path} && 
               validate_ast($ast->{base}) && ref($ast->{path}) eq 'ARRAY';
    }
    elsif ($type eq 'literal') {
        return exists $ast->{value};
    }
    
    return 0;  # Unknown type
}

=head2 ast_to_string

Converts AST back to readable return annotation syntax for debugging.

=cut

sub ast_to_string {
    my ($ast) = @_;
    
    return 'undef' unless defined $ast;
    return 'invalid' unless validate_ast($ast);
    
    my $type = $ast->{type};
    
    if ($type eq 'scalar_ref') {
        return '$' . $ast->{index};
    }
    elsif ($type eq 'array') {
        my @elements = map { ast_to_string($_) } @{$ast->{contents}};
        return '[' . join(', ', @elements) . ']';
    }
    elsif ($type eq 'object') {
        my @pairs = map { 
            my $key = $_->{key};
            my $value = ast_to_string($_->{value});
            "$key: $value"
        } @{$ast->{contents}};
        return '{' . join(', ', @pairs) . '}';
    }
    elsif ($type eq 'quantified_array') {
        return '[' . ast_to_string($ast->{element}) . '*]';
    }
    elsif ($type eq 'mixed_array') {
        my @elements = map { 
            $_->{type} eq 'quantified_collection' ?
                ast_to_string($_->{element}) . '*' :
                ast_to_string($_)
        } @{$ast->{contents}};
        return '[' . join(', ', @elements) . ']';
    }
    elsif ($type eq 'quantified_collection') {
        return ast_to_string($ast->{element}) . '*';
    }
    elsif ($type eq 'ultimate_dot_notation') {
        my $base = ast_to_string($ast->{base});
        my $path = join('', map {
            $_->{type} eq 'property' ? '.' . $_->{name} :
            $_->{type} eq 'index' ? '[' . $_->{value} . ']' :
            $_->{type} eq 'slice' ? '[' . $_->{start} . ':' . $_->{end} . ']' :
            '.unknown'
        } @{$ast->{path}});
        return $base . $path;
    }
    elsif ($type eq 'literal') {
        my $value = $ast->{value};
        return $ast->{value_type} eq 'string' ? "\"$value\"" : $value;
    }
    
    return 'unknown';
}

1;

__END__

=head1 EXAMPLES

# Simple scalar reference: $1
my $scalar = AST::UniversalReturnAnnotation::new_scalar_ref(1);

# Simple array: [$1, $2]  
my $array = AST::UniversalReturnAnnotation::new_array(
    AST::UniversalReturnAnnotation::new_scalar_ref(1),
    AST::UniversalReturnAnnotation::new_scalar_ref(2)
);

# Quantified array: [$1*]
my $quantified = AST::UniversalReturnAnnotation::new_quantified_array(
    AST::UniversalReturnAnnotation::new_scalar_ref(1)
);

# Mixed array: [$1, $2*]
my $mixed = AST::UniversalReturnAnnotation::new_mixed_array(
    AST::UniversalReturnAnnotation::new_scalar_ref(1),
    AST::UniversalReturnAnnotation::new_quantified_collection(
        AST::UniversalReturnAnnotation::new_scalar_ref(2)
    )
);

# Object: {type: "property", name: $2}
my $object = AST::UniversalReturnAnnotation::new_object(
    {key => 'type', value => AST::UniversalReturnAnnotation::new_literal('property')},
    {key => 'name', value => AST::UniversalReturnAnnotation::new_scalar_ref(2)}
);

# Branch annotation for: accessor := property_accessor -> {type: "property", name: $1}
my $branch = AST::UniversalReturnAnnotation::new_branch_annotation(
    'accessor', 0, $object
);

=cut
