package yapg::GeneratedParser;
use strict;
use warnings;

# Compiled regex patterns for speed
my %REGEXES = (
    'token_comp_90_step3' => qr/\Q|\E/o,
    'token_comp_90_step4' => qr/\Qstart\E/o,
    'token_comp_90_step8' => qr/\Q|\E/o,
    'token_comp_90_step9' => qr/\Qfinish\E/o,
    'token_comp_90_step10' => qr/\Q|\E/o,
    'token_comp_90_step11' => qr/\Qbegin\E/o,
    'token_comp_90_step13' => qr/\Q|\E/o,
    'atom_node_36' => qr/\Qvoid|false|for|return|start|const\E/o
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
sub parse_element_node_4 {
    my ($input) = @_;
    return parse_function($input);
}


sub parse_atom_elem_56 {
    my ($input) = @_;
    return parse_while($input);
}


sub parse_token_comp_90 {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_value_def_72($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    my $result_2 = parse_atom_node_36($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    unless ($$input =~ /\G$REGEXES{'token_comp_90_step3'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'token_comp_90_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_5 = parse_element_node_4($input);
    unless (defined $result_5) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_5;
    my $result_6 = parse_token_comp_90($input);
    unless (defined $result_6) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_6;
    my $result_7 = parse_atom_elem_56($input);
    unless (defined $result_7) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_7;
    unless ($$input =~ /\G$REGEXES{'token_comp_90_step8'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'token_comp_90_step9'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'token_comp_90_step10'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'token_comp_90_step11'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_12 = parse_token_comp_90($input);
    unless (defined $result_12) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_12;
    unless ($$input =~ /\G$REGEXES{'token_comp_90_step13'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_14 = parse_atom_elem_56($input);
    unless (defined $result_14) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_14;
    
    return $results[3-1];
}


sub parse_value_def_72 {
    my ($input) = @_;
    return parse_null($input);
}


sub parse_atom_node_36 {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'atom_node_36'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return collect_quantified_results(1, \@results);
}


# Main entry point
sub parse {
    my ($input_ref) = @_;
    pos($$input_ref) = 0;
    my $result = parse_atom_elem_56($input_ref);
    
    # Check that entire input was consumed
    if (defined $result && pos($$input_ref) == length($$input_ref)) {
        return $result;
    } else {
        return undef;  # Partial match or unconsumed input
    }
}

1;
