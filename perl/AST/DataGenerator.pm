package AST::DataGenerator;

use strict;
use warnings;
use Data::Dumper;
use Carp;
use lib 'perl';
# Note: We don't import extract_token_value from AST::Transform anymore
# as it serves a different purpose there (raw EBNF parsing vs final AST processing)

# Export main functions
use Exporter 'import';
our @EXPORT_OK = qw(
    new
    generate_data
    set_seed
    set_max_depth
    set_recursion_tracker
    generate_from_rule
    generate_from_atom
    generate_from_sequence
    generate_from_or
    generate_from_quantified
    extract_probability_annotations
    select_weighted_alternative
);

# Constructor
sub new {
    my ($class, %options) = @_;
    
    my $self = {
        # Core data
        final_ast => $options{final_ast} || {},
        rule_order => $options{rule_order} || [],
        
        # Configuration
        max_depth => $options{max_depth} || 10,
        seed => $options{seed},
        
        # Runtime state
        recursion_depth => 0,
        recursion_tracker => {},
        debug => $options{debug} || 0,
        
        # Statistics
        generation_stats => {
            rules_expanded => 0,
            terminals_generated => 0,
            max_depth_reached => 0,
        }
    };
    
    # Set random seed if provided
    if (defined $self->{seed}) {
        srand($self->{seed});
        print STDERR "🎲 Random seed set to: $self->{seed}\n" if $self->{debug};
    }
    
    bless $self, $class;
    return $self;
}

# Main entry point: generate test data from grammar
sub generate_data {
    my ($self, %options) = @_;
    
    my $start_rule = $options{start_rule} || $self->{rule_order}->[0];
    my $count = $options{count} || 1;
    
    unless ($start_rule) {
        croak "No start rule specified and no rules available in grammar";
    }
    
    unless (exists $self->{final_ast}->{$start_rule}) {
        croak "Start rule '$start_rule' not found in grammar";
    }
    
    print STDERR "🚀 Generating $count test inputs from rule '$start_rule'\n" if $self->{debug};
    
    my @generated_data = ();
    
    for my $i (1..$count) {
        # Reset state for each generation
        $self->{recursion_depth} = 0;
        $self->{recursion_tracker} = {};
        
        print STDERR "\n--- Generation $i/$count ---\n" if $self->{debug};
        
        my $data = $self->generate_from_rule($start_rule);
        if (defined $data) {
            push @generated_data, $data;
            print STDERR "✅ Generated: '$data'\n" if $self->{debug};
        } else {
            print STDERR "❌ Failed to generate data for attempt $i\n" if $self->{debug};
        }
    }
    
    # Print statistics
    if ($self->{debug}) {
        print STDERR "\n📊 Generation Statistics:\n";
        print STDERR "  Rules expanded: $self->{generation_stats}->{rules_expanded}\n";
        print STDERR "  Terminals generated: $self->{generation_stats}->{terminals_generated}\n";
        print STDERR "  Max depth reached: $self->{generation_stats}->{max_depth_reached}\n";
    }
    
    return \@generated_data;
}

# Generate data from a specific rule
sub generate_from_rule {
    my ($self, $rule_name) = @_;
    
    # Check recursion depth
    if ($self->{recursion_depth} >= $self->{max_depth}) {
        print STDERR "⚠️  Max recursion depth ($self->{max_depth}) reached for rule '$rule_name'\n" if $self->{debug};
        $self->{generation_stats}->{max_depth_reached}++;
        return ""; # Return empty string to avoid infinite recursion
    }
    
    # Check for excessive recursion of the same rule
    $self->{recursion_tracker}->{$rule_name} = ($self->{recursion_tracker}->{$rule_name} || 0) + 1;
    if ($self->{recursion_tracker}->{$rule_name} > 3) {
        print STDERR "⚠️  Rule '$rule_name' recursed too many times, terminating\n" if $self->{debug};
        return "";
    }
    
    # Get rule definition
    my $rule_def = $self->{final_ast}->{$rule_name};
    unless ($rule_def) {
        croak "Rule '$rule_name' not found in grammar";
    }
    
    print STDERR "  " x $self->{recursion_depth} . "📋 Expanding rule: $rule_name\n" if $self->{debug};
    
    $self->{recursion_depth}++;
    $self->{generation_stats}->{rules_expanded}++;
    
    my $result = $self->generate_from_ast_node($rule_def);
    
    $self->{recursion_depth}--;
    $self->{recursion_tracker}->{$rule_name}--;
    
    return $result;
}

# Generate data from an AST node (dispatch based on type)
sub generate_from_ast_node {
    my ($self, $node) = @_;
    
    unless (ref($node) eq 'HASH' && $node->{type}) {
        croak "Invalid AST node: " . Dumper($node);
    }
    
    my $type = $node->{type};
    
    if ($type eq 'atom') {
        return $self->generate_from_atom($node);
    } elsif ($type eq 'sequence') {
        return $self->generate_from_sequence($node);
    } elsif ($type eq 'or') {
        return $self->generate_from_or($node);
    } elsif ($type eq 'quantified') {
        return $self->generate_from_quantified($node);
    } else {
        croak "Unknown AST node type: $type";
    }
}

# Generate data from an atom node (terminal or rule reference)
sub generate_from_atom {
    my ($self, $node) = @_;
    
    if ($self->{debug}) {
        print STDERR "  " x $self->{recursion_depth} . "🔍 DEBUG: generate_from_atom called with node:\n";
        print STDERR "  " x $self->{recursion_depth} . "    node ref type: " . ref($node) . "\n";
        print STDERR "  " x $self->{recursion_depth} . "    node dump: " . Dumper($node) . "\n";
    }
    
    my $value = $node->{value};
    
    if ($self->{debug}) {
        print STDERR "  " x $self->{recursion_depth} . "🔍 DEBUG: node value extracted:\n";
        print STDERR "  " x $self->{recursion_depth} . "    value ref type: " . ref($value) . "\n";
        print STDERR "  " x $self->{recursion_depth} . "    value dump: " . Dumper($value) . "\n";
    }
    
    # Handle nested atom structures
    if (ref($value) eq 'HASH' && $value->{type} eq 'atom') {
        print STDERR "  " x $self->{recursion_depth} . "🔄 DEBUG: Handling nested atom structure\n" if $self->{debug};
        return $self->generate_from_atom($value);
    }
    
    # CRITICAL FIX: Handle sequence structures that are embedded in atom nodes
    if (ref($value) eq 'HASH' && $value->{type} eq 'sequence') {
        print STDERR "  " x $self->{recursion_depth} . "🔄 DEBUG: Atom contains sequence - delegating to generate_from_sequence\n" if $self->{debug};
        return $self->generate_from_sequence($value);
    }
    
    # Handle OR structures that are embedded in atom nodes  
    if (ref($value) eq 'HASH' && $value->{type} eq 'or') {
        print STDERR "  " x $self->{recursion_depth} . "🔄 DEBUG: Atom contains OR - delegating to generate_from_or\n" if $self->{debug};
        return $self->generate_from_or($value);
    }
    
    # Check if this is a terminal
    my $is_terminal = $self->is_terminal($value);
    if ($self->{debug}) {
        print STDERR "  " x $self->{recursion_depth} . "🔍 DEBUG: is_terminal check result: " . ($is_terminal ? "TRUE" : "FALSE") . "\n";
    }
    
    if ($is_terminal) {
        print STDERR "  " x $self->{recursion_depth} . "📝 DEBUG: Generating terminal value\n" if $self->{debug};
        return $self->generate_terminal_value($value);
    } else {
        # This is a rule reference - use the specialized extractor
        print STDERR "  " x $self->{recursion_depth} . "🔗 DEBUG: Processing rule reference, calling extract_rule_name\n" if $self->{debug};
        my $rule_name = $self->extract_rule_name($value);
        
        if ($self->{debug}) {
            print STDERR "  " x $self->{recursion_depth} . "🔍 DEBUG: extract_rule_name result:\n";
            print STDERR "  " x $self->{recursion_depth} . "    rule_name: '" . (defined $rule_name ? $rule_name : 'undef') . "'\n";
            print STDERR "  " x $self->{recursion_depth} . "    defined: " . (defined $rule_name ? "YES" : "NO") . "\n";
            print STDERR "  " x $self->{recursion_depth} . "    looks valid: " . ((defined $rule_name && $rule_name && $rule_name !~ /^(?:HASH|COMPLEX|ERROR)/) ? "YES" : "NO") . "\n";
        }
        
        if (defined $rule_name && $rule_name && $rule_name !~ /^(?:HASH|COMPLEX|ERROR)/) {
            print STDERR "  " x $self->{recursion_depth} . "✅ DEBUG: Valid rule name extracted: '$rule_name'\n" if $self->{debug};
            return $self->generate_from_rule($rule_name);
        } else {
            print STDERR "  " x $self->{recursion_depth} . "❌ DEBUG: Failed to extract valid rule name\n" if $self->{debug};
            print STDERR "  " x $self->{recursion_depth} . "    Original value: " . Dumper($value) . "\n" if $self->{debug};
            print STDERR "  " x $self->{recursion_depth} . "    Extracted rule_name: '" . (defined $rule_name ? $rule_name : 'undef') . "'\n" if $self->{debug};
            print STDERR "  " x $self->{recursion_depth} . "    This indicates the atom contains a complex structure, not a simple rule reference\n" if $self->{debug};
            return "";
        }
    }
}

# Generate data from a sequence node
sub generate_from_sequence {
    my ($self, $node) = @_;
    
    if ($self->{debug}) {
        print STDERR "  " x $self->{recursion_depth} . "📝 DEBUG: generate_from_sequence called with node:\n";
        print STDERR "  " x $self->{recursion_depth} . "    node dump: " . Dumper($node) . "\n";
    }
    
    my $elements = $node->{elements} || [];
    my @parts = ();
    
    print STDERR "  " x $self->{recursion_depth} . "📝 Generating sequence with " . scalar(@$elements) . " elements\n" if $self->{debug};
    
    for my $i (0 .. $#$elements) {
        my $element = $elements->[$i];
        
        if ($self->{debug}) {
            print STDERR "  " x $self->{recursion_depth} . "  🔍 Processing element $i:\n";
            print STDERR "  " x $self->{recursion_depth} . "      element ref type: " . ref($element) . "\n";
            print STDERR "  " x $self->{recursion_depth} . "      element type: " . (ref($element) eq 'HASH' ? ($element->{type} || 'NO_TYPE') : 'NOT_HASH') . "\n";
            print STDERR "  " x $self->{recursion_depth} . "      element dump: " . Dumper($element) . "\n";
        }
        
        my $part;
        
        # Handle raw array elements that need to be converted to AST node format
        if (ref($element) eq 'ARRAY') {
            # Convert array to atom node
            my $atom_node = {
                type => 'atom',
                value => $element
            };
            $part = $self->generate_from_ast_node($atom_node);
        } else {
            # Standard AST node
            $part = $self->generate_from_ast_node($element);
        }
        
        if ($self->{debug}) {
            print STDERR "  " x $self->{recursion_depth} . "  ✅ Element $i generated: '" . (defined $part ? $part : 'undef') . "'\n";
        }
        
        if (defined $part) {
            push @parts, $part unless $part eq ''; # Skip empty strings but keep defined values
        }
    }
    
    my $result = join('', @parts);
    print STDERR "  " x $self->{recursion_depth} . "📝 Sequence result: '$result'\n" if $self->{debug};
    
    return $result;
}

# Generate data from an OR node (alternatives)
sub generate_from_or {
    my ($self, $node) = @_;
    
    my $alternatives = $node->{alternatives} || [];
    unless (@$alternatives) {
        print STDERR "⚠️  No alternatives found in OR node\n" if $self->{debug};
        return "";
    }
    
    print STDERR "  " x $self->{recursion_depth} . "🎯 Selecting from " . scalar(@$alternatives) . " alternatives\n" if $self->{debug};
    
    # Extract probability annotations if present
    my $selected_alt = $self->select_weighted_alternative($alternatives);
    
    return $self->generate_from_ast_node($selected_alt);
}

# Generate data from a quantified node (*, +, ?, {n,m})
sub generate_from_quantified {
    my ($self, $node) = @_;
    
    my $element = $node->{element};
    my $quantifier = $node->{quantifier} || '*';
    
    # Parse quantifier to get min/max bounds
    my ($min, $max) = $self->parse_quantifier($quantifier);
    
    # Choose actual repetition count within bounds
    my $count = $min + int(rand($max - $min + 1));
    
    # Special case: limit repetitions to prevent excessive output
    if ($count > 5) {
        $count = 3 + int(rand(3)); # Limit to 3-5 repetitions max
    }
    
    print STDERR "  " x $self->{recursion_depth} . "🔢 Quantifier '$quantifier' → generating $count repetitions\n" if $self->{debug};
    
    my @parts = ();
    for my $i (1..$count) {
        my $part = $self->generate_from_ast_node($element);
        push @parts, $part if defined $part && $part ne '';
    }
    
    return join('', @parts);
}

# Select alternative from OR group with probability weights
sub select_weighted_alternative {
    my ($self, $alternatives) = @_;
    
    # For now, implement simple uniform random selection
    # TODO: Add probability annotation parsing and weighted selection
    
    my $selected_index = int(rand(scalar(@$alternatives)));
    my $selected = $alternatives->[$selected_index];
    
    print STDERR "  " x $self->{recursion_depth} . "🎲 Selected alternative $selected_index\n" if $self->{debug};
    
    return $selected;
}

# Parse quantifier string to min/max bounds
sub parse_quantifier {
    my ($self, $quantifier) = @_;
    
    if ($quantifier eq '*') {
        return (0, 3); # 0-3 repetitions
    } elsif ($quantifier eq '+') {
        return (1, 3); # 1-3 repetitions  
    } elsif ($quantifier eq '?') {
        return (0, 1); # 0-1 repetitions
    } elsif ($quantifier =~ /^(\d+),(\d+)$/) {
        return ($1, $2); # Explicit bounds
    } elsif ($quantifier =~ /^(\d+),$/) {
        return ($1, $1 + 2); # Min specified, reasonable max
    } elsif ($quantifier =~ /^,(\d+)$/) {
        return (0, $1); # Max specified
    } else {
        # Unknown quantifier, default to single occurrence
        return (1, 1);
    }
}

# Check if a value represents a terminal
sub is_terminal {
    my ($self, $value) = @_;
    
    # Handle nested structures
    if (ref($value) eq 'HASH' && $value->{type} eq 'atom' && ref($value->{value}) eq 'ARRAY') {
        $value = $value->{value};
    }
    
    if (ref($value) eq 'ARRAY' && @$value >= 2) {
        my $type = $value->[0];
        return ($type eq 'quoted_string' || $type eq 'regex' || $type eq 'terminal' || 
                $type eq 'number' || $type eq 'epsilon');
    }
    
    return 0;
}

# Extract rule name from final AST structures (specialized for data generation)
sub extract_rule_name {
    my ($self, $value) = @_;
    
    if ($self->{debug}) {
        print STDERR "  " x $self->{recursion_depth} . "    🔧 extract_rule_name processing: " . ref($value) . "\n";
    }
    
    # Handle different final AST value structures
    if (ref($value) eq 'ARRAY' && @$value >= 2) {
        # Simple array format: ['rule_reference', 'rule_name']
        if ($value->[0] eq 'rule_reference' || $value->[0] eq 'rule') {
            return $value->[1];
        }
        # Terminal array format: ['quoted_string', 'content'] - not a rule reference
        return "ERROR_NOT_A_RULE";
    }
    elsif (ref($value) eq 'HASH') {
        # Handle different hash structures in final AST
        
        # Nested atom structure: {type => 'atom', value => [...]}
        if ($value->{type} eq 'atom' && ref($value->{value}) eq 'ARRAY') {
            return $self->extract_rule_name($value->{value});
        }
        
        # Complex AST node structures (sequence, or, quantified)
        elsif ($value->{type} eq 'sequence' || $value->{type} eq 'or' || $value->{type} eq 'quantified') {
            if ($self->{debug}) {
                print STDERR "  " x $self->{recursion_depth} . "    ⚠️  Attempted to extract rule name from complex AST node type: $value->{type}\n";
            }
            return "COMPLEX_AST_NODE_" . $value->{type};
        }
        
        # Other hash structures - might contain rule references  
        elsif (exists $value->{rule_name}) {
            return $value->{rule_name};
        }
        elsif (exists $value->{name}) {
            return $value->{name};
        }
        
        # Unhandled hash structure
        if ($self->{debug}) {
            print STDERR "  " x $self->{recursion_depth} . "    ⚠️  Unhandled hash structure in extract_rule_name\n";
        }
        return "HASH_STRUCTURE_UNHANDLED";
    }
    elsif (!ref($value)) {
        # Simple scalar - assume it's already a rule name
        return $value;
    }
    else {
        # Other reference type
        if ($self->{debug}) {
            print STDERR "  " x $self->{recursion_depth} . "    ⚠️  Unexpected reference type in extract_rule_name: " . ref($value) . "\n";
        }
        return "ERROR_UNEXPECTED_REF_" . ref($value);
    }
}

# Generate actual terminal value
sub generate_terminal_value {
    my ($self, $value) = @_;
    
    # Handle nested structures
    if (ref($value) eq 'HASH' && $value->{type} eq 'atom' && ref($value->{value}) eq 'ARRAY') {
        $value = $value->{value};
    }
    
    unless (ref($value) eq 'ARRAY' && @$value >= 2) {
        print STDERR "⚠️  Invalid terminal value structure: " . Dumper($value) . "\n" if $self->{debug};
        return "";
    }
    
    my $type = $value->[0];
    my $content = $value->[1];
    
    $self->{generation_stats}->{terminals_generated}++;
    
    if ($type eq 'quoted_string' || $type eq 'terminal') {
        # Literal string - return as-is
        print STDERR "  " x $self->{recursion_depth} . "💬 Terminal: '$content'\n" if $self->{debug};
        return $content;
    } elsif ($type eq 'regex') {
        # Regex pattern - generate matching string
        print STDERR "  " x $self->{recursion_depth} . "🔍 Regex: /$content/\n" if $self->{debug};
        return $self->generate_from_regex($content);
    } elsif ($type eq 'epsilon') {
        # Empty production
        print STDERR "  " x $self->{recursion_depth} . "∅ Epsilon\n" if $self->{debug};
        return "";
    } elsif ($type eq 'number') {
        # Numeric literal
        print STDERR "  " x $self->{recursion_depth} . "🔢 Number: $content\n" if $self->{debug};
        return $content;
    } else {
        print STDERR "⚠️  Unknown terminal type: $type\n" if $self->{debug};
        return $content; # Fallback
    }
}

# Generate string that matches regex pattern (basic implementation)
sub generate_from_regex {
    my ($self, $pattern) = @_;
    
    # Basic regex pattern matching - this is a simplified implementation
    # TODO: Implement full regex-to-string generation
    
    # Handle common simple patterns
    if ($pattern eq '\\d+') {
        # One or more digits
        my $length = 1 + int(rand(3)); # 1-3 digits
        return join('', map { int(rand(10)) } 1..$length);
    } elsif ($pattern eq '\\d') {
        # Single digit
        return int(rand(10));
    } elsif ($pattern eq '\\w+') {
        # One or more word characters
        my @chars = ('a'..'z', 'A'..'Z', '0'..'9', '_');
        my $length = 1 + int(rand(4)); # 1-4 characters
        return join('', map { $chars[rand @chars] } 1..$length);
    } elsif ($pattern eq '\\w') {
        # Single word character
        my @chars = ('a'..'z', 'A'..'Z', '0'..'9', '_');
        return $chars[rand @chars];
    } elsif ($pattern eq '\\s+') {
        # One or more whitespace
        return ' ' x (1 + int(rand(2))); # 1-2 spaces
    } elsif ($pattern eq '\\s') {
        # Single whitespace
        return ' ';
    } elsif ($pattern eq '[a-zA-Z]+') {
        # One or more letters
        my @chars = ('a'..'z', 'A'..'Z');
        my $length = 1 + int(rand(4)); # 1-4 characters
        return join('', map { $chars[rand @chars] } 1..$length);
    } elsif ($pattern eq '[a-zA-Z]') {
        # Single letter
        my @chars = ('a'..'z', 'A'..'Z');
        return $chars[rand @chars];
    } elsif ($pattern eq '[0-9]+') {
        # One or more digits
        my $length = 1 + int(rand(3)); # 1-3 digits
        return join('', map { int(rand(10)) } 1..$length);
    } elsif ($pattern eq '[0-9]') {
        # Single digit
        return int(rand(10));
    }
    
    # Fallback for unhandled patterns - return something plausible
    print STDERR "⚠️  Unhandled regex pattern: /$pattern/ - using fallback\n" if $self->{debug};
    return "placeholder";
}

# Set random seed
sub set_seed {
    my ($self, $seed) = @_;
    $self->{seed} = $seed;
    srand($seed) if defined $seed;
}

# Set maximum recursion depth
sub set_max_depth {
    my ($self, $max_depth) = @_;
    $self->{max_depth} = $max_depth;
}

# Get generation statistics
sub get_stats {
    my ($self) = @_;
    return $self->{generation_stats};
}

# Enable/disable debug output
sub set_debug {
    my ($self, $debug) = @_;
    $self->{debug} = $debug;
}

1;

__END__

=head1 NAME

AST::DataGenerator - Generate pseudo-random test data from EBNF grammar AST

=head1 SYNOPSIS

    use AST::DataGenerator;
    use AST::Transform qw(process_to_final_ast);
    
    # Process EBNF grammar to final AST
    my ($final_ast, $rule_order) = process_to_final_ast($ebnf_content);
    
    # Create data generator
    my $generator = AST::DataGenerator->new(
        final_ast => $final_ast,
        rule_order => $rule_order,
        max_depth => 10,
        seed => 42,
        debug => 1
    );
    
    # Generate test data
    my $data = $generator->generate_data(
        count => 10,
        start_rule => 'expression'
    );
    
    foreach my $test_input (@$data) {
        print "Generated: $test_input\n";
    }

=head1 DESCRIPTION

AST::DataGenerator is the core module for generating pseudo-random test data from EBNF grammars. 
It consumes the final AST produced by AST::Transform::process_to_final_ast() and generates 
realistic test inputs that conform to the grammar specification.

=head1 KEY FEATURES

=over 4

=item * Consumes final AST from the transformation pipeline

=item * Supports all EBNF constructs: sequences, alternatives, quantifiers

=item * Handles terminal generation (strings, regexes, numbers)  

=item * Recursion depth control to prevent infinite loops

=item * Configurable random seed for reproducible generation

=item * Debug output for generation process visibility

=item * Generation statistics tracking

=back

=head1 METHODS

=head2 new(%options)

Create a new data generator instance.

Options:
- final_ast: Final AST from AST::Transform::process_to_final_ast()
- rule_order: Rule processing order 
- max_depth: Maximum recursion depth (default: 10)
- seed: Random seed for reproducible generation
- debug: Enable debug output (default: 0)

=head2 generate_data(%options)

Generate test data from the grammar.

Options:
- count: Number of test inputs to generate (default: 1)
- start_rule: Starting rule name (default: first rule)

Returns: Array reference of generated test strings

=head2 set_seed($seed)

Set random seed for reproducible generation.

=head2 set_max_depth($depth)

Set maximum recursion depth limit.

=head2 get_stats()

Get generation statistics hash reference.

=head1 ARCHITECTURE INTEGRATION

This module is part of the hybrid AST processing architecture:

    EBNF Grammar
         ↓
    AST::Transform::process_to_final_ast()
         ↓
    Final AST + Rule Order
         ↓
    AST::DataGenerator
         ↓
    Generated Test Data

=head1 FUTURE ENHANCEMENTS

=over 4

=item * Probability annotation support (@n% weights)

=item * Advanced regex pattern generation  

=item * Terminal value constraints and validation

=item * Custom generation strategies per rule type

=item * Performance optimization for large grammars

=back

=head1 SEE ALSO

L<AST::Transform>, L<AST::UniversalReturnAnnotation>

=cut
