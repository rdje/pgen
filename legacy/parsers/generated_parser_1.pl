package yapg::GeneratedParser;
use strict;
use warnings;

# Compiled regex patterns for speed
my %REGEXES = (
    'stmt_elem_91_step1' => qr/\Qdo\E/o,
    'atom_elem_14_step3' => qr/\Qthen\E/o,
    'atom_elem_14_step4' => qr/\Q|\E/o,
    'atom_elem_14_step8' => qr/\Q|\E/o,
    'expr_elem_34' => qr/\Qwhile\E/o,
    'value_def_26_step1' => qr/\Qvoid\E/o,
    'value_def_26_step2' => qr/\Q|\E/o,
    'value_def_26_step3' => qr/\Qelse\E/o,
    'value_def_26_step4' => qr/\Q|\E/o,
    'value_def_26_step5' => qr/\Qthen\E/o,
    'value_def_26_step6' => qr/\Q|\E/o,
    'value_def_26_step9' => qr/\Q|\E/o,
    'value_def_26_step10' => qr/\Qint\E/o,
    'value_def_26_step11' => qr/\Q|\E/o,
    'value_def_26_step12' => qr/\Qconst\E/o,
    'value_def_26_step13' => qr/\Q|\E/o,
    'value_def_26_step15' => qr/\Qvar\E/o,
    'value_def_26_step16' => qr/\Q|\E/o,
    'value_def_26_step17' => qr/\Qclass\E/o,
    'value_def_26_step18' => qr/\Q|\E/o,
    'value_def_26_step19' => qr/\Qfunction\E/o,
    'value_def_26_step20' => qr/\Q|\E/o,
    'value_def_26_step21' => qr/\Qend\E/o,
    'value_def_26_step22' => qr/\Q|\E/o,
    'value_def_26_step23' => qr/\Qelse\E/o
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
sub parse_stmt_elem_91 {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'stmt_elem_91_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_2 = parse_ARRAY(0x7fe7f099bf88)($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    
    return \@results;
}


sub parse_item_rule_1 {
    my ($input) = @_;
    return parse_stmt_comp_36($input);
}


sub parse_atom_elem_10 {
    my ($input) = @_;
    return parse_stmt_comp_36($input);
}


sub parse_element_node_39 {
    my ($input) = @_;
    return parse_then($input);
}


sub parse_atom_elem_14 {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_expr_elem_34($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    my $result_2 = parse_element_node_39($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    unless ($$input =~ /\G$REGEXES{'atom_elem_14_step3'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'atom_elem_14_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_5 = parse_element_node_39($input);
    unless (defined $result_5) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_5;
    my $result_6 = parse_stmt_comp_36($input);
    unless (defined $result_6) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_6;
    my $result_7 = parse_atom_elem_14($input);
    unless (defined $result_7) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_7;
    unless ($$input =~ /\G$REGEXES{'atom_elem_14_step8'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_9 = parse_atom_elem_14($input);
    unless (defined $result_9) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_9;
    my $result_10 = parse_ARRAY(0x7fe7f099bd60)($input);
    unless (defined $result_10) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_10;
    
    return \@results;
}


sub parse_element_node_7 {
    my ($input) = @_;
    return parse_function($input);
}


sub parse_expr_elem_34 {
    my ($input) = @_;
    return 1 if $$input =~ /\G$REGEXES{'expr_elem_34'}/gc;
    return undef;
}


sub parse_stmt_comp_36 {
    my ($input) = @_;
    return parse_while($input);
}


sub parse_value_def_26 {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'value_def_26_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'value_def_26_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'value_def_26_step3'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'value_def_26_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'value_def_26_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'value_def_26_step6'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_7 = parse_value_def_26($input);
    unless (defined $result_7) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_7;
    my $result_8 = parse_item_rule_1($input);
    unless (defined $result_8) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_8;
    unless ($$input =~ /\G$REGEXES{'value_def_26_step9'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'value_def_26_step10'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'value_def_26_step11'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'value_def_26_step12'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'value_def_26_step13'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_14 = parse_value_def_26($input);
    unless (defined $result_14) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_14;
    unless ($$input =~ /\G$REGEXES{'value_def_26_step15'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'value_def_26_step16'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'value_def_26_step17'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'value_def_26_step18'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'value_def_26_step19'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'value_def_26_step20'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'value_def_26_step21'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'value_def_26_step22'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'value_def_26_step23'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return collect_quantified_results(2, \@results);
}


sub parse_element_elem_9 {
    my ($input) = @_;
    return parse_else($input);
}


# Main entry point
sub parse {
    my ($input_ref) = @_;
    pos($$input_ref) = 0;
    my $result = parse_stmt_comp_36($input_ref);
    
    # Check that entire input was consumed
    if (defined $result && pos($$input_ref) == length($$input_ref)) {
        return $result;
    } else {
        return undef;  # Partial match or unconsumed input
    }
}

1;
