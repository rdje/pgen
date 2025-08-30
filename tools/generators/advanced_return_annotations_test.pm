package Advanced_return_annotations_test; # Placeholder, will be replaced by tools/ast_transform.pl
use strict;
use warnings;

# Compiled regex patterns for speed
our %REGEXES = (
    'number_literal_step1' => qr/(-?\d+\.?\d*)/o,
    'array_def_first' => qr/\Q[\E/o,
    'config_value_alt1_0' => qr/\Q[\E/o,
    'config_value_alt1_2' => qr/\Q]\E/o,
    'item_alt1_0' => qr/\Q(\E/o,
    'item_alt1_2' => qr/\Q)\E/o,
    'property_step2' => qr/\s*/o,
    'property_step3' => qr/\Q:\E/o,
    'property_step4' => qr/\s*/o,
    'string_literal_step1' => qr/\"([^\"]*)\"/o,
    'object_def_first' => qr/\Q{\E/o,
    'ultimate_item_step2' => qr/\Q:\E/o,
    'function_call_first' => qr/([a-zA-Z_]\w*)/o,
    'config_entry_step2' => qr/\Q=\E/o,
    'config_entry_step4' => qr/\Q;\E/o,
    'ultimate_value_alt1_0' => qr/\Q[\E/o,
    'ultimate_value_alt1_2' => qr/\Q]\E/o,
    'ultimate_value_alt2_0' => qr/\Q{\E/o,
    'ultimate_value_alt2_2' => qr/\Q}\E/o,
    'config_key_step1' => qr/([a-zA-Z_]\w*)/o,
    'boolean_literal_alt0_0' => qr/\Qtrue\E/o,
    'boolean_literal_alt1_0' => qr/\Qfalse\E/o,
    'ultimate_structure_step1' => qr/\Qultimate\E/o,
    'ultimate_structure_step2' => qr/\Q{\E/o,
    'ultimate_structure_step4' => qr/\Q}\E/o,
    'accessor_alt0_0' => qr/\Q.\E/o,
    'accessor_alt0_1' => qr/([a-zA-Z_]\w*)/o,
    'accessor_alt1_0' => qr/\Q[\E/o,
    'accessor_alt1_1' => qr/\s*/o,
    'accessor_alt1_2' => qr/(\\d+)/o,
    'accessor_alt1_3' => qr/\s*/o,
    'accessor_alt1_4' => qr/\Q]\E/o,
    'accessor_alt2_0' => qr/\Q[\E/o,
    'accessor_alt2_1' => qr/\s*/o,
    'accessor_alt2_3' => qr/\s*/o,
    'accessor_alt2_4' => qr/\Q]\E/o,
    'key_alt0_0' => qr/\"([^\"]+)\"/o,
    'key_alt1_0' => qr/([a-zA-Z_]\w*)/o,
    'quantifier_alt0_0' => qr/\Q*\E/o,
    'quantifier_alt1_0' => qr/\Q+\E/o,
    'quantifier_alt2_0' => qr/\Q?\E/o,
    'quantifier_alt3_0' => qr/\Q{\E/o,
    'quantifier_alt3_1' => qr/\s*/o,
    'quantifier_alt3_2' => qr/(\d+)/o,
    'quantifier_alt3_3' => qr/\s*/o,
    'quantifier_alt3_4' => qr/\Q}\E/o,
    'quantifier_alt4_0' => qr/\Q{\E/o,
    'quantifier_alt4_1' => qr/\s*/o,
    'quantifier_alt4_2' => qr/(\d+)/o,
    'quantifier_alt4_3' => qr/\s*/o,
    'quantifier_alt4_4' => qr/\Q,\E/o,
    'quantifier_alt4_5' => qr/\s*/o,
    'quantifier_alt4_6' => qr/\Q}\E/o,
    'quantifier_alt5_0' => qr/\Q{\E/o,
    'quantifier_alt5_1' => qr/\s*/o,
    'quantifier_alt5_2' => qr/(\d+)/o,
    'quantifier_alt5_3' => qr/\s*/o,
    'quantifier_alt5_4' => qr/\Q,\E/o,
    'quantifier_alt5_5' => qr/\s*/o,
    'quantifier_alt5_6' => qr/(\d+)/o,
    'quantifier_alt5_7' => qr/\s*/o,
    'quantifier_alt5_8' => qr/\Q}\E/o,
    'configuration_first' => qr/\Qconfig\E/o
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

sub parse_number_literal {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'number_literal_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return $1;
}


sub parse_config_value_list {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    # Parse first required element
    my $result_1 = parse_config_value($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    
    # Parse optional repeated grouped elements in loop
    while (1) {
        my $loop_start_pos = pos($$input);
        
        # Exit loop if no group code available
        if (!q{}) {
            pos($$input) = $loop_start_pos;
            last;
        }
        

    }

    
    return [$results[1-1], collect_quantified_results(3, \@results)];
}


sub parse_parameter {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_value($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    
    return {"param_value" => ($results[1-1] // undef), "param_index" => "auto"};
}


sub parse_array_def {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    # Parse first required element (regex match)
    unless ($$input =~ /\G$REGEXES{'array_def_first'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture from first regex
    
    # Parse optional repeated grouped elements in loop
    while (1) {
        my $loop_start_pos = pos($$input);
        
        # Exit loop if no group code available
        if (!q{}) {
            pos($$input) = $loop_start_pos;
            last;
        }
        

    }

    
    return {"type" => "array", "elements" => ($results[3-1] // undef) || [], "count" => (($results[3-1] // undef) || []).length};
}


sub parse_config_value {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_simple_value($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'config_value_alt1_0'}/gc) && (parse_config_value_list($input)) && ($$input =~ /\G$REGEXES{'config_value_alt1_2'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_item {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_simple_value($input))) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'item_alt1_0'}/gc) && (parse_value($input)) && ($$input =~ /\G$REGEXES{'item_alt1_2'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_property {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_key($input);
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
    my $result_5 = parse_value($input);
    unless (defined $result_5) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_5;
    
    return {"key" => ($results[1-1] // undef), "value" => ($results[5-1] // undef), "metadata" => {"line" => 1}};
}


sub parse_string_literal {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'string_literal_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return $1;
}


sub parse_ultimate_array_content {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    # Parse first required element
    my $result_1 = parse_ultimate_value($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    
    # Parse optional repeated grouped elements in loop
    while (1) {
        my $loop_start_pos = pos($$input);
        
        # Exit loop if no group code available
        if (!q{}) {
            pos($$input) = $loop_start_pos;
            last;
        }
        

    }

    
    return {
        "elements" => [($results[1-1] // undef), ($results[3-1] // undef)*],
        "count" => 1 + (($results[3-1] // undef)*).length,
        "types" => [($results[1-1] // {})->{type}, ...(($results[3-1] // undef)*).map(x => x.type)],
        "max_complexity" => Math.max(($results[1-1] // {})->{complexity}, ...(($results[3-1] // undef)*).map(x => x.complexity))
    };
}


sub parse_ultimate_content {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    # Parse first required element
    my $result_1 = parse_ultimate_item($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    
    # Parse optional repeated grouped elements in loop
    while (1) {
        my $loop_start_pos = pos($$input);
        
        # Exit loop if no group code available
        if (!q{}) {
            pos($$input) = $loop_start_pos;
            last;
        }
        

    }

    
    return {
        "primary" => ($results[1-1] // undef),
        "additional" => collect_quantified_results(3, \@results),
        "total_items" => 1 + (($results[3-1] // undef)*).length,
        "structure_analysis" => {
            "has_primary" => true,
            "has_additional" => (($results[3-1] // undef)*).length > 0,
            "item_types" => [($results[1-1] // {})->{type}, ...(($results[3-1] // undef)*).map(x => x.type)]
        }
    };
}


sub parse_value {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_object_def($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_array_def($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_simple_value($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_function_call($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_object_def {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    # Parse first required element (regex match)
    unless ($$input =~ /\G$REGEXES{'object_def_first'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture from first regex
    
    # Parse optional repeated grouped elements in loop
    while (1) {
        my $loop_start_pos = pos($$input);
        
        # Exit loop if no group code available
        if (!q{}) {
            pos($$input) = $loop_start_pos;
            last;
        }
        

    }

    
    return {"type" => "object", "properties" => ($results[3-1] // undef) || [], "source_location" => {"start" => 1, "end" => 5}};
}


sub parse_ultimate_item {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_key($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    unless ($$input =~ /\G$REGEXES{'ultimate_item_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_ultimate_value($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    
    return {
        "item_key" => ($results[1-1] // undef),
        "item_value" => ($results[3-1] // undef),
        "type" => ($results[3-1] // {})->{type} || "unknown",
        "path" => [($results[1-1] // undef)]
    };
}


sub parse_data_structure {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_object_def($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_array_def($input))) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_simple_value($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_function_call {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    # Parse first required element (regex match)
    unless ($$input =~ /\G$REGEXES{'function_call_first'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture from first regex
    
    # Parse optional repeated grouped elements in loop
    while (1) {
        my $loop_start_pos = pos($$input);
        
        # Exit loop if no group code available
        if (!q{}) {
            pos($$input) = $loop_start_pos;
            last;
        }
        

    }

    
    return {"function_name" => ($results[1-1] // undef), "parameters" => ($results[4-1] // undef) || [], "call_type" => "function_invocation"};
}


sub parse_repeated_pattern {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_item($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    my $result_2 = parse_quantifier($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    
    return {"pattern" => ($results[1-1] // undef), "quantifier" => ($results[2-1] // undef), "collection_type" => "sequence"};
}


sub parse_config_entry {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_config_key($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    unless ($$input =~ /\G$REGEXES{'config_entry_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_config_value($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'config_entry_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return {"key" => ($results[1-1] // undef), "value" => ($results[3-1] // undef), "entry_type" => "setting"};
}


sub parse_nested_access {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    # Parse first required element
    my $result_1 = parse_value($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    
    # Parse optional repeated grouped elements in loop
    while (1) {
        my $loop_start_pos = pos($$input);
        
        # Exit loop if no group code available
        if (!q{}) {
            pos($$input) = $loop_start_pos;
            last;
        }
        

    }

    
    return {"base" => ($results[1-1] // undef), "accessors" => collect_quantified_results(2, \@results), "result_type" => ($results[1-1] // {})->{type}};
}


sub parse_simple_value {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_string_literal($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_number_literal($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_boolean_literal($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_parameter_list {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    # Parse first required element
    my $result_1 = parse_parameter($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    
    # Parse optional repeated grouped elements in loop
    while (1) {
        my $loop_start_pos = pos($$input);
        
        # Exit loop if no group code available
        if (!q{}) {
            pos($$input) = $loop_start_pos;
            last;
        }
        

    }

    
    return [$results[1-1], collect_quantified_results(3, \@results)];
}


sub parse_ultimate_value {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_simple_value($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'ultimate_value_alt1_0'}/gc) && (parse_ultimate_array_content($input)) && ($$input =~ /\G$REGEXES{'ultimate_value_alt1_2'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'ultimate_value_alt2_0'}/gc) && (parse_ultimate_object_content($input)) && ($$input =~ /\G$REGEXES{'ultimate_value_alt2_2'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_config_key {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'config_key_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return $1;
}


sub parse_element_list {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    # Parse first required element
    my $result_1 = parse_value($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    
    # Parse optional repeated grouped elements in loop
    while (1) {
        my $loop_start_pos = pos($$input);
        
        # Exit loop if no group code available
        if (!q{}) {
            pos($$input) = $loop_start_pos;
            last;
        }
        

    }

    
    return [$results[1-1], collect_quantified_results(3, \@results)];
}


sub parse_property_list {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    # Parse first required element
    my $result_1 = parse_property($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    
    # Parse optional repeated grouped elements in loop
    while (1) {
        my $loop_start_pos = pos($$input);
        
        # Exit loop if no group code available
        if (!q{}) {
            pos($$input) = $loop_start_pos;
            last;
        }
        

    }

    
    return [$results[1-1], collect_quantified_results(3, \@results)];
}


sub parse_boolean_literal {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'boolean_literal_alt0_0'}/gc) && (parse_true($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'boolean_literal_alt1_0'}/gc) && (parse_false($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_ultimate_structure {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'ultimate_structure_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'ultimate_structure_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_ultimate_content($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'ultimate_structure_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return {
        "structure_type" => "ultimate",
        "content" => ($results[3-1] // undef),
        "metadata" => {
            "parser_version" => "2.0",
            "features_used" => ["nested_objects", "quantified_arrays", "dot_notation"],
            "complexity_score" => 10
        }
    };
}


sub parse_accessor {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'accessor_alt0_0'}/gc) && ($$input =~ /\G$REGEXES{'accessor_alt0_1'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'accessor_alt1_0'}/gc) && ($$input =~ /\G$REGEXES{'accessor_alt1_1'}/gc) && ($$input =~ /\G$REGEXES{'accessor_alt1_2'}/gc) && ($$input =~ /\G$REGEXES{'accessor_alt1_3'}/gc) && ($$input =~ /\G$REGEXES{'accessor_alt1_4'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'accessor_alt2_0'}/gc) && ($$input =~ /\G$REGEXES{'accessor_alt2_1'}/gc) && (parse_string_literal($input)) && ($$input =~ /\G$REGEXES{'accessor_alt2_3'}/gc) && ($$input =~ /\G$REGEXES{'accessor_alt2_4'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_key {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'key_alt0_0'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'key_alt1_0'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_ultimate_object_content {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    # Parse first required element
    my $result_1 = parse_ultimate_item($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    
    # Parse optional repeated grouped elements in loop
    while (1) {
        my $loop_start_pos = pos($$input);
        
        # Exit loop if no group code available
        if (!q{}) {
            pos($$input) = $loop_start_pos;
            last;
        }
        

    }

    
    return {
        "properties" => [($results[1-1] // undef), ($results[3-1] // undef)*],
        "property_count" => 1 + (($results[3-1] // undef)*).length,
        "property_names" => [($results[1-1] // {})->{item_key}, ...(($results[3-1] // undef)*).map(x => x.item_key)],
        "nested_complexity" => ($results[1-1] // {})->{item_value}.complexity + (($results[3-1] // undef)*).reduce((sum, x) => sum + x.item_value.complexity, 0)
    };
}


sub parse_quantifier {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'quantifier_alt0_0'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'quantifier_alt1_0'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'quantifier_alt2_0'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'quantifier_alt3_0'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt3_1'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt3_2'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt3_3'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt3_4'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'quantifier_alt4_0'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt4_1'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt4_2'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt4_3'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt4_4'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt4_5'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt4_6'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'quantifier_alt5_0'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt5_1'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt5_2'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt5_3'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt5_4'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt5_5'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt5_6'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt5_7'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt5_8'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_configuration {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    # Parse first required element (regex match)
    unless ($$input =~ /\G$REGEXES{'configuration_first'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture from first regex
    
    # Parse optional repeated grouped elements in loop
    while (1) {
        my $loop_start_pos = pos($$input);
        
        # Exit loop if no group code available
        if (!q{}) {
            pos($$input) = $loop_start_pos;
            last;
        }
        

    }

    
    return {"type" => "configuration", "entries" => collect_quantified_results(3, \@results), "entry_count" => (($results[3-1] // undef)*).length, "valid" => true};
}


# Main entry point
sub parse {
    my ($input) = @_;
    pos($$input) = 0;
    return parse_data_structure($input);
}

1;
