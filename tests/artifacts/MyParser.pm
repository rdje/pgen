package MyParser; # Placeholder, will be replaced by tools/ast_transform.pl
use strict;
use warnings;

# Compiled regex patterns for speed
our %REGEXES = (
    'three_property_object_step1' => qr/\Q{\E/o,
    'three_property_object_step2' => qr/\s*/o,
    'three_property_object_step4' => qr/\Q,\E/o,
    'three_property_object_step5' => qr/\s*/o,
    'three_property_object_step7' => qr/\Q,\E/o,
    'three_property_object_step8' => qr/\s*/o,
    'three_property_object_step10' => qr/\s*/o,
    'three_property_object_step11' => qr/\Q}\E/o,
    'quoted_string' => qr/"([^"]*)"/o,
    'property_accessor_step1' => qr/\Q.\E/o,
    'grouped_quantified_array_step1' => qr/\Q[\E/o,
    'grouped_quantified_array_step2' => qr/\s*/o,
    'grouped_quantified_array_step4' => qr/\s*/o,
    'grouped_quantified_array_step5' => qr/\Q]\E/o,
    'property_step2' => qr/\s*/o,
    'property_step3' => qr/\Q:\E/o,
    'property_step4' => qr/\s*/o,
    'negative_number_step1' => qr/\Q-\E/o,
    'array_accessor_step1' => qr/\Q[\E/o,
    'array_accessor_step3' => qr/\Q]\E/o,
    'simple_array_step1' => qr/\Q[\E/o,
    'simple_array_step2' => qr/\s*/o,
    'simple_array_step4' => qr/\s*/o,
    'simple_array_step5' => qr/\Q]\E/o,
    'empty_slice_part' => qr/(?=:)/o,
    'positive_number' => qr/(\d+)/o,
    'python_slice_step2' => qr/\Q:\E/o,
    'object_pair_step2' => qr/\s*/o,
    'object_pair_step3' => qr/\Q:\E/o,
    'object_pair_step4' => qr/\s*/o,
    'simple_object_step1' => qr/\Q{\E/o,
    'simple_object_step2' => qr/\s*/o,
    'simple_object_step4' => qr/\s*/o,
    'simple_object_step5' => qr/\Q:\E/o,
    'simple_object_step6' => qr/\s*/o,
    'simple_object_step8' => qr/\s*/o,
    'simple_object_step9' => qr/\Q}\E/o,
    'grouped_element_item_alt0_0' => qr/\Q(\E/o,
    'grouped_element_item_alt0_1' => qr/\s*/o,
    'grouped_element_item_alt0_3' => qr/\s*/o,
    'grouped_element_item_alt0_4' => qr/\Q)\E/o,
    'perl_range_step2' => qr/\Q..\E/o,
    'two_property_object_step1' => qr/\Q{\E/o,
    'two_property_object_step2' => qr/\s*/o,
    'two_property_object_step4' => qr/\Q,\E/o,
    'two_property_object_step5' => qr/\s*/o,
    'two_property_object_step7' => qr/\s*/o,
    'two_property_object_step8' => qr/\Q}\E/o,
    'element_sequence_alt0_1' => qr/\Q(\E/o,
    'element_sequence_alt0_2' => qr/\s+/o,
    'element_sequence_alt0_4' => qr/\Q)\E/o,
    'element_sequence_alt0_5' => qr/\Q*\E/o,
    'positional_accessor_step1' => qr/\Q.\E/o,
    'quantifier_alt0' => qr/\Q*\E/o,
    'quantifier_alt1' => qr/\Q+\E/o,
    'quantifier_alt2' => qr/\Q?\E/o,
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
    'quantifier_alt6_0' => qr/\Q{\E/o,
    'quantifier_alt6_1' => qr/\s*/o,
    'quantifier_alt6_2' => qr/\Q,\E/o,
    'quantifier_alt6_3' => qr/\s*/o,
    'quantifier_alt6_5' => qr/\s*/o,
    'quantifier_alt6_6' => qr/\Q}\E/o,
    'simple_nested_object_step1' => qr/\Q{\E/o,
    'simple_nested_object_step2' => qr/\s*/o,
    'simple_nested_object_step4' => qr/\s*/o,
    'simple_nested_object_step5' => qr/\Q:\E/o,
    'simple_nested_object_step6' => qr/\s*/o,
    'simple_nested_object_step8' => qr/\s*/o,
    'simple_nested_object_step9' => qr/\Q}\E/o,
    'python_slice_with_step_step2' => qr/\Q:\E/o,
    'python_slice_with_step_step4' => qr/\Q:\E/o,
    'quantified_array_step1' => qr/\Q[\E/o,
    'quantified_array_step2' => qr/\s*/o,
    'quantified_array_step4' => qr/\s*/o,
    'quantified_array_step5' => qr/\Q]\E/o,
    'nested_object_step1' => qr/\Q{\E/o,
    'nested_object_step2' => qr/\s*/o,
    'nested_object_step4' => qr/\Q?\E/o,
    'nested_object_step5' => qr/\s*/o,
    'nested_object_step6' => qr/\Q}\E/o,
    'nested_object_step8' => qr/\Q?\E/o,
    'dot_path_step2' => qr/\Q+\E/o,
    'empty_spec' => qr/(?=\])/o,
    'grouped_element_list_alt0_1' => qr/\Q(\E/o,
    'grouped_element_list_alt0_2' => qr/\Q,\E/o,
    'grouped_element_list_alt0_3' => qr/\s*/o,
    'grouped_element_list_alt0_5' => qr/\Q)\E/o,
    'grouped_element_list_alt0_6' => qr/\Q*\E/o,
    'star_spec' => qr/\Q*\E/o,
    'return_annotation_step1' => qr/\Q->\E/o,
    'return_annotation_step2' => qr/\s*/o,
    'nested_array_step1' => qr/\Q[\E/o,
    'nested_array_step2' => qr/\s*/o,
    'nested_array_step4' => qr/\Q?\E/o,
    'nested_array_step5' => qr/\s*/o,
    'nested_array_step6' => qr/\Q]\E/o,
    'nested_array_step8' => qr/\Q?\E/o,
    'colon_spec' => qr/\Q:\E/o,
    'number' => qr/(\d+)/o,
    'identifier' => qr/([a-zA-Z_]\w*)/o,
    'inner_object_step1' => qr/\Q{\E/o,
    'inner_object_step2' => qr/\s*/o,
    'inner_object_step4' => qr/\s*/o,
    'inner_object_step5' => qr/\Q:\E/o,
    'inner_object_step6' => qr/\s*/o,
    'inner_object_step8' => qr/\s*/o,
    'inner_object_step9' => qr/\Q}\E/o,
    'scalar_ref_step1' => qr/\$/o
);

# Runtime helper functions
sub quantified_match {
    my ($input, $regex, $min, $max) = @_;
    my $count = 0;
    my $pos = pos($$input);
    
    # Optimized: Pre-compile regex with cache
    my $compiled_regex = qr/$regex/o;
    
    # Optimized: Tighter loop with fewer operations
    while ($count < $max) {
        if ($$input =~ /\G$compiled_regex/gc) {
            $count++;
        } else {
            last;
        }
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
    my $checkpoint = pos($$input);
    
    # Optimized: Pre-allocate array for better performance
    my @results;
    $#results = $max - 1 if $max < 1000; # Pre-allocate for reasonable sizes
    
    my $result_idx = 0;
    while ($count < $max) {
        my $result = $rule_ref->($input);
        if (defined $result) {
            $results[$result_idx++] = $result;
            $count++;
        } else {
            last;
        }
    }
    
    if ($count >= $min) {
        # Optimized: Trim array to actual size
        $#results = $count - 1;
        return \@results;
    } else {
        # Restore position on failure
        pos($$input) = $checkpoint;
        return undef;
    }
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
    
    return \@results;
}


sub parse_quoted_string {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'quoted_string'}/gc) {
        my @results = ($1);  # Capture regex result
        return 1;
    }
    return undef;
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
    
    return \@results;
}


sub parse_object_value {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_ultimate_dot_notation($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_scalar_ref($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_literal($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_nested_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_nested_object($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
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
    
    return \@results;
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
    
    return \@results;
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
    
    return \@results;
}


sub parse_array_spec {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_empty_spec($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_star_spec($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_colon_spec($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_single_index($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_perl_range($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_python_slice($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_python_slice_with_step($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_index_list($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_mixed_expression($input))) { return $alt_result; }
    
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
    
    return \@results;
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
    
    return \@results;
}


sub parse_empty_slice_part {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'empty_slice_part'}/gc) {
        my @results = ($1);  # Capture regex result
        return 1;
    }
    return undef;
}


sub parse_positive_number {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'positive_number'}/gc) {
        my @results = ($1);  # Capture regex result
        return 1;
    }
    return undef;
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
    
    return \@results;
}


sub parse_object_pair {
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
    unless ($$input =~ /\G$REGEXES{'object_pair_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'object_pair_step3'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'object_pair_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_5 = parse_return_expression($input);
    unless (defined $result_5) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_5;
    
    return \@results;
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
    
    return \@results;
}


sub parse_property_value {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_nested_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_nested_object($input))) { return $alt_result; }
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
    
    return \@results;
}


sub parse_group_content {
    my ($input) = @_;
    my $result = parse_element_sequence($input);
    if (defined $result) {
        return $result;
    }
    return undef;
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
    
    return \@results;
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

sub parse_inner_value {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_nested_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_nested_object($input))) { return $alt_result; }
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

sub parse_array_element {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_ultimate_dot_notation($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_scalar_ref($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_literal($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_nested_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_nested_object($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_array_contents {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    # Parse first required element
    my $result_1 = parse_return_expression($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    
    # Parse optional repeated grouped elements in loop
    while (1) {
        # Try to match the grouped pattern: (',' /s*/ return_expression)
        my $loop_start_pos = pos($$input);
        
        # Match comma
        unless ($$input =~ /\G,/gc) { last; }
        
        # Match optional whitespace
        $$input =~ /\G\s*/gc;
        
        # Match return_expression
        my $loop_result = parse_return_expression($input);
        unless (defined $loop_result) {
            # Restore position and exit loop
            pos($$input) = $loop_start_pos;
            last;
        }
        
        # Successfully matched the group, add to results
        push @results, $loop_result;
    }

    
    return \@results;
}


sub parse_return_expression {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_nested_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_nested_object($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_grouped_quantified_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_simple_nested_object($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_multi_property_object($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_quantified_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_simple_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_simple_object($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_ultimate_dot_notation($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_scalar_ref($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_literal($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_element_sequence {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_element_item($input)) && ($$input =~ /\G$REGEXES{'element_sequence_alt0_1'}/gc) && ($$input =~ /\G$REGEXES{'element_sequence_alt0_2'}/gc) && (parse_element_item($input)) && ($$input =~ /\G$REGEXES{'element_sequence_alt0_4'}/gc) && ($$input =~ /\G$REGEXES{'element_sequence_alt0_5'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (defined(my $alt_result = parse_element_item($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
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
    
    return \@results;
}


sub parse_quantifier {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = $$input =~ /\G$REGEXES{'quantifier_alt0'}/gc) { return $alt_result; }
    if (my $alt_result = $$input =~ /\G$REGEXES{'quantifier_alt1'}/gc) { return $alt_result; }
    if (my $alt_result = $$input =~ /\G$REGEXES{'quantifier_alt2'}/gc) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'quantifier_alt3_0'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt3_1'}/gc) && (parse_positive_number($input)) && ($$input =~ /\G$REGEXES{'quantifier_alt3_3'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt3_4'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'quantifier_alt4_0'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt4_1'}/gc) && (parse_positive_number($input)) && ($$input =~ /\G$REGEXES{'quantifier_alt4_3'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt4_4'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt4_5'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt4_6'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'quantifier_alt5_0'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt5_1'}/gc) && (parse_positive_number($input)) && ($$input =~ /\G$REGEXES{'quantifier_alt5_3'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt5_4'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt5_5'}/gc) && (parse_positive_number($input)) && ($$input =~ /\G$REGEXES{'quantifier_alt5_7'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt5_8'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'quantifier_alt6_0'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt6_1'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt6_2'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt6_3'}/gc) && (parse_positive_number($input)) && ($$input =~ /\G$REGEXES{'quantifier_alt6_5'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt6_6'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

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
    
    return \@results;
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
    
    return \@results;
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
    
    return \@results;
}


sub parse_nested_object {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'nested_object_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'nested_object_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_object_contents($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'nested_object_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'nested_object_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'nested_object_step6'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_7 = parse_quantifier($input);
    unless (defined $result_7) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_7;
    unless ($$input =~ /\G$REGEXES{'nested_object_step8'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return \@results;
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

sub parse_dot_path {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_accessor($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    unless ($$input =~ /\G$REGEXES{'dot_path_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return \@results;
}


sub parse_empty_spec {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'empty_spec'}/gc) {
        my @results = ($1);  # Capture regex result
        return 1;
    }
    return undef;
}


sub parse_grouped_element_list {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_grouped_element_item($input)) && ($$input =~ /\G$REGEXES{'grouped_element_list_alt0_1'}/gc) && ($$input =~ /\G$REGEXES{'grouped_element_list_alt0_2'}/gc) && ($$input =~ /\G$REGEXES{'grouped_element_list_alt0_3'}/gc) && (parse_grouped_element_item($input)) && ($$input =~ /\G$REGEXES{'grouped_element_list_alt0_5'}/gc) && ($$input =~ /\G$REGEXES{'grouped_element_list_alt0_6'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (defined(my $alt_result = parse_grouped_element_item($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_star_spec {
    my ($input) = @_;
    return 1 if $$input =~ /\G$REGEXES{'star_spec'}/gc;
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


sub parse_step {
    my ($input) = @_;
    my $result = parse_index($input);
    if (defined $result) {
        return $result;
    }
    return undef;
}


sub parse_mixed_expression {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    # Parse first required element
    my $result_1 = parse_mixed_element($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    
    # Parse optional repeated grouped elements in loop
    while (1) {
        # Try to match the grouped pattern: (',' /s*/ return_expression)
        my $loop_start_pos = pos($$input);
        
        # Match comma
        unless ($$input =~ /\G,/gc) { last; }
        
        # Match optional whitespace
        $$input =~ /\G\s*/gc;
        
        # Match return_expression
        my $loop_result = parse_return_expression($input);
        unless (defined $loop_result) {
            # Restore position and exit loop
            pos($$input) = $loop_start_pos;
            last;
        }
        
        # Successfully matched the group, add to results
        push @results, $loop_result;
    }

    
    return \@results;
}


sub parse_nested_array {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'nested_array_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'nested_array_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_array_contents($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'nested_array_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'nested_array_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'nested_array_step6'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_7 = parse_quantifier($input);
    unless (defined $result_7) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_7;
    unless ($$input =~ /\G$REGEXES{'nested_array_step8'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return \@results;
}


sub parse_colon_spec {
    my ($input) = @_;
    return 1 if $$input =~ /\G$REGEXES{'colon_spec'}/gc;
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

sub parse_object_contents {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    # Parse first required element
    my $result_1 = parse_object_pair($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    
    # Parse optional repeated grouped elements in loop
    while (1) {
        # Try to match the grouped pattern: (',' /s*/ return_expression)
        my $loop_start_pos = pos($$input);
        
        # Match comma
        unless ($$input =~ /\G,/gc) { last; }
        
        # Match optional whitespace
        $$input =~ /\G\s*/gc;
        
        # Match return_expression
        my $loop_result = parse_return_expression($input);
        unless (defined $loop_result) {
            # Restore position and exit loop
            pos($$input) = $loop_start_pos;
            last;
        }
        
        # Successfully matched the group, add to results
        push @results, $loop_result;
    }

    
    return \@results;
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
    
    return \@results;
}


sub parse_index_list {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    # Parse first required element
    my $result_1 = parse_index($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    
    # Parse optional repeated grouped elements in loop
    while (1) {
        # Try to match the grouped pattern: (',' /s*/ return_expression)
        my $loop_start_pos = pos($$input);
        
        # Match comma
        unless ($$input =~ /\G,/gc) { last; }
        
        # Match optional whitespace
        $$input =~ /\G\s*/gc;
        
        # Match return_expression
        my $loop_result = parse_return_expression($input);
        unless (defined $loop_result) {
            # Restore position and exit loop
            pos($$input) = $loop_start_pos;
            last;
        }
        
        # Successfully matched the group, add to results
        push @results, $loop_result;
    }

    
    return \@results;
}


sub parse_single_index {
    my ($input) = @_;
    my $result = parse_index($input);
    if (defined $result) {
        return $result;
    }
    return undef;
}


sub parse_number {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'number'}/gc) {
        my @results = ($1);  # Capture regex result
        return 1;
    }
    return undef;
}


sub parse_identifier {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'identifier'}/gc) {
        my @results = ($1);  # Capture regex result
        return 1;
    }
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
    
    return \@results;
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
    
    return \@results;
}


# Main entry point
sub parse {
    my ($input) = @_;
    pos($$input) = 0;
    return parse_return_annotation($input);
}

1;
