package yapg::GeneratedParser;
use strict;
use warnings;

# Compiled regex patterns for speed
my %REGEXES = (
    'simple_nested_object_step1' => qr/\Q{\E/o,
    'simple_nested_object_step2' => qr/\s*/o,
    'simple_nested_object_step4' => qr/\s*/o,
    'simple_nested_object_step5' => qr/\Q:\E/o,
    'simple_nested_object_step6' => qr/\s*/o,
    'simple_nested_object_step8' => qr/\s*/o,
    'simple_nested_object_step9' => qr/\Q}\E/o,
    'empty_slice_part' => qr/(?=:)/o,
    'grouped_quantified_array_step1' => qr/\Q[\E/o,
    'grouped_quantified_array_step2' => qr/\s*/o,
    'grouped_quantified_array_step4' => qr/\s*/o,
    'grouped_quantified_array_step5' => qr/\Q]\E/o,
    'negative_number_step1' => qr/\Q-\E/o,
    'perl_range_step2' => qr/\Q..\E/o,
    'property_step2' => qr/\s*/o,
    'property_step3' => qr/\Q:\E/o,
    'property_step4' => qr/\s*/o,
    'number_step1' => qr/(\d+)/o,
    'scalar_ref_step1' => qr/\$/o,
    'array_accessor_step1' => qr/\Q[\E/o,
    'array_accessor_step3' => qr/\Q]\E/o,
    'property_accessor_step1' => qr/\Q.\E/o,
    'quoted_string_step1' => qr/"([^"]*)"/o,
    'quantified_array_step1' => qr/\Q[\E/o,
    'quantified_array_step2' => qr/\s*/o,
    'quantified_array_step4' => qr/\s*/o,
    'quantified_array_step5' => qr/\Q]\E/o,
    'positional_accessor_step1' => qr/\Q.\E/o,
    'whitespace_step1' => qr/\s+/o,
    'three_property_object_step1' => qr/\Q{\E/o,
    'three_property_object_step2' => qr/\s*/o,
    'three_property_object_step4' => qr/\Q,\E/o,
    'three_property_object_step5' => qr/\s*/o,
    'three_property_object_step7' => qr/\Q,\E/o,
    'three_property_object_step8' => qr/\s*/o,
    'three_property_object_step10' => qr/\s*/o,
    'three_property_object_step11' => qr/\Q}\E/o,
    'grouped_element_item_alt0_0' => qr/\Q(\E/o,
    'grouped_element_item_alt0_1' => qr/\s*/o,
    'grouped_element_item_alt0_3' => qr/\s*/o,
    'grouped_element_item_alt0_4' => qr/\Q)\E/o,
    'simple_array_step1' => qr/\Q[\E/o,
    'simple_array_step2' => qr/\s*/o,
    'simple_array_step4' => qr/\s*/o,
    'simple_array_step5' => qr/\Q]\E/o,
    'empty_spec' => qr/(?=\])/o,
    'python_slice_step2' => qr/\Q:\E/o,
    'return_annotation_step1' => qr/\Q->\E/o,
    'return_annotation_step2' => qr/\s*/o,
    'positive_number_step1' => qr/(\d+)/o,
    'star_spec' => qr/\Q*\E/o,
    'colon_spec' => qr/\Q:\E/o,
    'inner_object_step1' => qr/\Q{\E/o,
    'inner_object_step2' => qr/\s*/o,
    'inner_object_step4' => qr/\s*/o,
    'inner_object_step5' => qr/\Q:\E/o,
    'inner_object_step6' => qr/\s*/o,
    'inner_object_step8' => qr/\s*/o,
    'inner_object_step9' => qr/\Q}\E/o,
    'simple_object_step1' => qr/\Q{\E/o,
    'simple_object_step2' => qr/\s*/o,
    'simple_object_step4' => qr/\s*/o,
    'simple_object_step5' => qr/\Q:\E/o,
    'simple_object_step6' => qr/\s*/o,
    'simple_object_step8' => qr/\s*/o,
    'simple_object_step9' => qr/\Q}\E/o,
    'identifier_step1' => qr/([a-zA-Z_]\w*)/o,
    'quantifier_alt0_0' => qr/\Q*\E/o,
    'quantifier_alt1_0' => qr/\Q+\E/o,
    'quantifier_alt2_0' => qr/\Q?\E/o,
    'quantifier_alt3_0' => qr/\Q{\E/o,
    'quantifier_alt3_1' => qr/\s*/o,
    'quantifier_alt3_3' => qr/\s*/o,
    'quantifier_alt3_4' => qr/\Q}\E/o,
    'quantifier_alt4_0' => qr/\Q{\E/o,
    'quantifier_alt4_1' => qr/\s*/o,
    'quantifier_alt4_3' => qr/\s*/o,
    'quantifier_alt4_4' => qr/\Q,\E/o,
    'quantifier_alt4_5' => qr/\s*/o,
    'quantifier_alt4_6' => qr/\Q}\E/o,
    'quantifier_alt5_0' => qr/\Q{\E/o,
    'quantifier_alt5_1' => qr/\s*/o,
    'quantifier_alt5_3' => qr/\s*/o,
    'quantifier_alt5_4' => qr/\Q,\E/o,
    'quantifier_alt5_5' => qr/\s*/o,
    'quantifier_alt5_7' => qr/\s*/o,
    'quantifier_alt5_8' => qr/\Q}\E/o,
    'two_property_object_step1' => qr/\Q{\E/o,
    'two_property_object_step2' => qr/\s*/o,
    'two_property_object_step4' => qr/\Q,\E/o,
    'two_property_object_step5' => qr/\s*/o,
    'two_property_object_step7' => qr/\s*/o,
    'two_property_object_step8' => qr/\Q}\E/o,
    'python_slice_with_step_step2' => qr/\Q:\E/o,
    'python_slice_with_step_step4' => qr/\Q:\E/o
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
sub parse_simple_nested_object {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'simple_nested_object_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'simple_nested_object_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_outer_key($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'simple_nested_object_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'simple_nested_object_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'simple_nested_object_step6'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_7 = parse_inner_object($input);
    unless (defined $result_7) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_7;
    unless ($$input =~ /\G$REGEXES{'simple_nested_object_step8'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'simple_nested_object_step9'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return {"type" => "nested_object", "key" => ($results[2] // undef), "value" => ($results[6] // undef)};
}


sub parse_object_value {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_ultimate_dot_notation($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_scalar_ref($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_literal($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_empty_slice_part {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'empty_slice_part'}/gc) {
        my @results = ($1);  # Capture regex result
        return 1;
    }
    return undef;
}


sub parse_grouped_quantified_array {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'grouped_quantified_array_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'grouped_quantified_array_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_grouped_element_list($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'grouped_quantified_array_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'grouped_quantified_array_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return {"type" => "grouped_quantified_array", "groups" => ($results[2] // undef)};
}


sub parse_negative_number {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'negative_number_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_2 = parse_positive_number($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    
    return {"type" => "negative", "value" => ($results[1] // undef)};
}


sub parse_array_spec {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_empty_spec($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_star_spec($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_colon_spec($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_single_index($input)) && ($1) || (pos($$input) = $seq_pos, undef) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_perl_range($input)) && ($1) || (pos($$input) = $seq_pos, undef) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_python_slice($input)) && ($1) || (pos($$input) = $seq_pos, undef) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_python_slice_with_step($input)) && ($1) || (pos($$input) = $seq_pos, undef) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_index_list($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_mixed_expression($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_perl_range {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_index($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    unless ($$input =~ /\G$REGEXES{'perl_range_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_index($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    
    return {"type" => "perl_range", "start" => ($results[0] // undef), "end" => ($results[2] // undef)};
}


sub parse_property {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_object_key($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    unless ($$input =~ /\G$REGEXES{'property_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'property_step3'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'property_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_5 = parse_property_value($input);
    unless (defined $result_5) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_5;
    
    return {"key" => ($results[0] // undef), "value" => ($results[4] // undef)};
}


sub parse_grouped_element_list {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_grouped_element_item($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_grouped_element_item($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_number {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'number_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return $results[0];
}


sub parse_index {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_positive_number($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_negative_number($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_inner_value {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_grouped_quantified_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_quantified_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_simple_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_ultimate_dot_notation($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_scalar_ref($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_literal($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_index_list {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_index($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    
    return [$results[1-1], collect_quantified_results(3, \@results)];
}


sub parse_scalar_ref {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'scalar_ref_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_2 = parse_positive_number($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    
    return {"type" => "scalar_ref", "index" => ($results[1] // undef)};
}


sub parse_element_item {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_identifier($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_literal($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_object_key {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_identifier($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_quoted_string($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_array_accessor {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'array_accessor_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_2 = parse_array_spec($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    unless ($$input =~ /\G$REGEXES{'array_accessor_step3'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return {"type" => "array_access", "spec" => ($results[1] // undef)};
}


sub parse_property_accessor {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'property_accessor_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_2 = parse_identifier($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    
    return {"type" => "property", "name" => ($results[1] // undef)};
}


sub parse_quoted_string {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'quoted_string_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return $results[0];
}


sub parse_quantified_element {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_ultimate_dot_notation($input)) && (parse_quantifier($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_scalar_ref($input)) && (parse_quantifier($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_python_slice_end {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_index($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_empty_slice_part($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_property_value {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_grouped_quantified_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_quantified_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_simple_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_ultimate_dot_notation($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_scalar_ref($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_literal($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_quantified_array {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'quantified_array_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'quantified_array_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_quantified_element($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'quantified_array_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'quantified_array_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return {"type" => "quantified_array", "element" => ($results[2] // undef)};
}


sub parse_literal {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_quoted_string($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_number($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_outer_key {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_identifier($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_quoted_string($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_array_element {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_ultimate_dot_notation($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_scalar_ref($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_single_index {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_index($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    
    return {"type" => "single_index", "value" => ($results[0] // undef)};
}


sub parse_positional_accessor {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'positional_accessor_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_2 = parse_positive_number($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    
    return {"type" => "position", "index" => ($results[1] // undef)};
}


sub parse_whitespace {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'whitespace_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return $results[" "-1];
}


sub parse_three_property_object {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'three_property_object_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'three_property_object_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_property($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'three_property_object_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'three_property_object_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_6 = parse_property($input);
    unless (defined $result_6) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_6;
    unless ($$input =~ /\G$REGEXES{'three_property_object_step7'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'three_property_object_step8'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_9 = parse_property($input);
    unless (defined $result_9) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_9;
    unless ($$input =~ /\G$REGEXES{'three_property_object_step10'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'three_property_object_step11'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return {"type" => "multi_object", "prop1" => ($results[3-1] // undef), "prop2" => ($results[6-1] // undef), "prop3" => ($results[9-1] // undef)};
}


sub parse_grouped_element_item {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'grouped_element_item_alt0_0'}/gc) && ($$input =~ /\G$REGEXES{'grouped_element_item_alt0_1'}/gc) && (parse_group_content($input)) && ($$input =~ /\G$REGEXES{'grouped_element_item_alt0_3'}/gc) && ($$input =~ /\G$REGEXES{'grouped_element_item_alt0_4'}/gc) && (parse_quantifier($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_element_sequence($input)) && (parse_quantifier($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_simple_array {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'simple_array_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'simple_array_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_array_element($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'simple_array_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'simple_array_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return {"type" => "array", "element" => ($results[2] // undef)};
}


sub parse_element_sequence {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_element_item($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_element_item($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_empty_spec {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'empty_spec'}/gc) {
        my @results = ($1);  # Capture regex result
        return 1;
    }
    return undef;
}


sub parse_step {
    my ($input) = @_;
    return parse_index($input);
}


sub parse_python_slice {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_python_slice_start($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    unless ($$input =~ /\G$REGEXES{'python_slice_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_python_slice_end($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    
    return {"type" => "python_slice", "start" => ($results[0] // undef), "end" => ($results[2] // undef)};
}


sub parse_return_annotation {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'return_annotation_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'return_annotation_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_return_expression($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    
    return \@results;
}


sub parse_return_expression {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_grouped_quantified_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_simple_nested_object($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_multi_property_object($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_quantified_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_simple_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_simple_object($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_ultimate_dot_notation($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_scalar_ref($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_positive_number {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'positive_number_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return {"type" => "positive", "value" => ($results[0] // undef)};
}


sub parse_ultimate_dot_notation {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_scalar_ref($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    my $result_2 = parse_dot_path($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    
    return {"type" => "ultimate_dot_notation", "base" => ($results[0] // undef), "path" => ($results[1] // undef)};
}


sub parse_star_spec {
    my ($input) = @_;
    return 1 if $$input =~ /\G$REGEXES{'star_spec'}/gc;
    return undef;
}


sub parse_colon_spec {
    my ($input) = @_;
    return 1 if $$input =~ /\G$REGEXES{'colon_spec'}/gc;
    return undef;
}


sub parse_mixed_expression {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_mixed_element($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    
    return [$results[1-1], collect_quantified_results(3, \@results)];
}


sub parse_inner_key {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_identifier($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_quoted_string($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_inner_object {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'inner_object_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'inner_object_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_inner_key($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'inner_object_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'inner_object_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'inner_object_step6'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_7 = parse_inner_value($input);
    unless (defined $result_7) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_7;
    unless ($$input =~ /\G$REGEXES{'inner_object_step8'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'inner_object_step9'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return {"type" => "inner_object", "key" => ($results[2] // undef), "value" => ($results[6] // undef)};
}


sub parse_simple_object {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'simple_object_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'simple_object_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_object_key($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'simple_object_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'simple_object_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'simple_object_step6'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_7 = parse_object_value($input);
    unless (defined $result_7) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_7;
    unless ($$input =~ /\G$REGEXES{'simple_object_step8'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'simple_object_step9'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return {"type" => "object", "key" => ($results[2] // undef), "value" => ($results[6] // undef)};
}


sub parse_identifier {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'identifier_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return $results[0];
}


sub parse_quantifier {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'quantifier_alt0_0'}/gc) && ("*") || (pos($$input) = $seq_pos, undef) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'quantifier_alt1_0'}/gc) && ("+") || (pos($$input) = $seq_pos, undef) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'quantifier_alt2_0'}/gc) && ("?") || (pos($$input) = $seq_pos, undef) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'quantifier_alt3_0'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt3_1'}/gc) && (parse_positive_number($input)) && ($$input =~ /\G$REGEXES{'quantifier_alt3_3'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt3_4'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'quantifier_alt4_0'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt4_1'}/gc) && (parse_positive_number($input)) && ($$input =~ /\G$REGEXES{'quantifier_alt4_3'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt4_4'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt4_5'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt4_6'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'quantifier_alt5_0'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt5_1'}/gc) && (parse_positive_number($input)) && ($$input =~ /\G$REGEXES{'quantifier_alt5_3'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt5_4'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt5_5'}/gc) && (parse_positive_number($input)) && ($$input =~ /\G$REGEXES{'quantifier_alt5_7'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt5_8'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_multi_property_object {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_two_property_object($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_three_property_object($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_dot_path {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = quantified_rule($input, \&parse_accessor, 1, 999);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    
    return collect_quantified_results(1, \@results);
}


sub parse_two_property_object {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'two_property_object_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'two_property_object_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_property($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'two_property_object_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'two_property_object_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_6 = parse_property($input);
    unless (defined $result_6) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_6;
    unless ($$input =~ /\G$REGEXES{'two_property_object_step7'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'two_property_object_step8'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return {"type" => "multi_object", "prop1" => ($results[2] // undef), "prop2" => ($results[5] // undef)};
}


sub parse_python_slice_with_step {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_python_slice_start($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    unless ($$input =~ /\G$REGEXES{'python_slice_with_step_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_python_slice_end($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'python_slice_with_step_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_5 = parse_step($input);
    unless (defined $result_5) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_5;
    
    return {"type" => "python_slice_step", "start" => ($results[1-1] // undef), "end" => ($results[3-1] // undef), "step" => ($results[5-1] // undef)};
}


sub parse_mixed_element {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_single_index($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_perl_range($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_python_slice($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_python_slice_with_step($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_accessor {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_property_accessor($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_positional_accessor($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_array_accessor($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_python_slice_start {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_index($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_empty_slice_part($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_group_content {
    my ($input) = @_;
    return parse_element_sequence($input);
}


# Main entry point
sub parse {
    my ($input_ref) = @_;
    pos($$input_ref) = 0;
    my $result = parse_return_annotation($input_ref);
    
    # Check that entire input was consumed
    if (defined $result && pos($$input_ref) == length($$input_ref)) {
        return $result;
    } else {
        return undef;  # Partial match or unconsumed input
    }
}

1;
