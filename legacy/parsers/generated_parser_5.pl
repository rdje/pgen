package yapg::GeneratedParser;
use strict;
use warnings;

# Compiled regex patterns for speed
my %REGEXES = (
    'item_node_79' => qr/\Qnull*|while\E/o,
    'element_def_25_prime_alt0_1' => qr/\Qdo\E/o
);

# Runtime helper functions
sub quantified_match {
    my ($input, $regex, $min, $max) = @_;
    my $count = 0;
    my $pos = pos($$input);
    
    while ($count < $max && $$input =~ /\G$regex/gc) {
        $count++;
    }
    
    if ($count >= $min) {
        return $count;
    } else {
        # Restore position on failure
        pos($$input) = $pos;
        return undef;
    }
}

sub quantified_rule {
    my ($input, $rule_ref, $min, $max) = @_;
    my $count = 0;
    my $pos = pos($$input);
    my @results = ();
    
    while ($count < $max) {
        my $result = $rule_ref->($input);
        if (defined $result) {
            push @results, $result;
            $count++;
        } else {
            last;
        }
    }
    
    if ($count >= $min) {
        return \@results;
    } else {
        # Restore position on failure
        pos($$input) = $pos;
        return undef;
    }
}

sub collect_quantified_results {
    # Helper function to collect results from quantified elements
    my ($element_num, $results_ref) = @_;
    my $element_index = $element_num - 1;
    
    # If the element is a quantified result (array), return it
    # If it's undef (zero matches), return empty array
    # Otherwise return single element in array
    my $element = $results_ref->[$element_index];
    
    if (!defined $element) {
        return [];  # Zero matches
    } elsif (ref($element) eq 'ARRAY') {
        return $element;  # Already an array from quantifier
    } else {
        return [$element];  # Single element, wrap in array
    }
}

# Fast parsing subroutines
sub parse_atom_part_27 {
    my ($input) = @_;
    return parse_null($input);
}


sub parse_atom_def_3 {
    my ($input) = @_;
    return parse_for($input);
}


sub parse_item_node_79 {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'item_node_79'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return \@results;
}


sub parse_element_def_25 {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)

    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_element_comp_36 {
    my ($input) = @_;
    return parse_let($input);
}


sub parse_token_def_46 {
    my ($input) = @_;
    return parse_class($input);
}


sub parse_stmt_def_77 {
    my ($input) = @_;
    return parse_start($input);
}


sub parse_stmt_elem_65 {
    my ($input) = @_;
    return parse_end($input);
}


sub parse_element_def_25_prime {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_atom_part_27($input)) && ($$input =~ /\G$REGEXES{'element_def_25_prime_alt0_1'}/gc) && (parse_element_def_25_prime($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = 1) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

# Main entry point
sub parse {
    my ($input_ref) = @_;
    pos($$input_ref) = 0;
    my $result = parse_element_def_25($input_ref);
    
    # Check that entire input was consumed
    if (defined $result && pos($$input_ref) == length($$input_ref)) {
        return $result;
    } else {
        return undef;  # Partial match or unconsumed input
    }
}

1;
