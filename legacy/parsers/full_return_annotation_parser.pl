package yapg::GeneratedParser;
use strict;
use warnings;

# Compiled regex patterns for speed
my %REGEXES = (
    'boolean' => qr/(true|false)/o,
    'grouped_expr_step1' => qr/\Q(\E/o,
    'grouped_expr_step2' => qr/\s*/o,
    'grouped_expr_step4' => qr/\s*/o,
    'grouped_expr_step5' => qr/\Q)\E/o,
    'object_contents_step3' => qr/\Q,\E/o,
    'object_contents_step4' => qr/\s*/o,
    'dot_path_step2' => qr/\Q.\E/o,
    'number' => qr/(\d+)/o,
    'nested_array_step1' => qr/\Q[\E/o,
    'nested_array_step2' => qr/\s*/o,
    'nested_array_step4' => qr/\s*/o,
    'nested_array_step5' => qr/\Q]\E/o,
    'quoted_string' => qr/"([^"]*)"/o,
    'array_contents_step3' => qr/\Q,\E/o,
    'array_contents_step4' => qr/\s*/o,
    'simple_quantifier' => qr/[\*\+\?]/o,
    'object_pair_step2' => qr/\Q:\E/o,
    'object_pair_step3' => qr/\s*/o,
    'identifier' => qr/([a-zA-Z_]\w*)/o,
    'return_annotation_step1' => qr/\Q->\E/o,
    'return_annotation_step2' => qr/\s*/o,
    'array_expr_step1' => qr/\Q[\E/o,
    'array_expr_step2' => qr/\s*/o,
    'array_expr_step4' => qr/\s*/o,
    'array_expr_step5' => qr/\Q]\E/o,
    'object_expr_step1' => qr/\Q{\E/o,
    'object_expr_step2' => qr/\s*/o,
    'object_expr_step4' => qr/\s*/o,
    'object_expr_step5' => qr/\Q}\E/o,
    'range_quantifier_step1' => qr/\Q{\E/o,
    'range_quantifier_step2' => qr/\s*/o,
    'range_quantifier_step5' => qr/\s*/o,
    'range_quantifier_step6' => qr/\Q,\E/o,
    'range_quantifier_step7' => qr/\s*/o,
    'range_quantifier_step10' => qr/\s*/o,
    'range_quantifier_step11' => qr/\Q}\E/o,
    'scalar_ref_step1' => qr/\$/o,
    'nested_object_step1' => qr/\Q{\E/o,
    'nested_object_step2' => qr/\s*/o,
    'nested_object_step4' => qr/\s*/o,
    'nested_object_step5' => qr/\Q}\E/o
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
sub parse_boolean {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'boolean'}/gc) {
        my @results = ($1);  # Capture regex result
        return $results[1-1];
    }
    return undef;
}


sub parse_grouped_expr {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'grouped_expr_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'grouped_expr_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_3 = parse_return_expression($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'grouped_expr_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'grouped_expr_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_6 = parse_quantifier($input);
    unless (defined $result_6) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_6;
    
    return {"type" => "grouped", "expr" => ($results[3-1] // undef), "quantified" => ($results[6-1] // undef)};
}


sub parse_object_value {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_scalar_ref($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_literal($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_nested_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_nested_object($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_array_expr($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_object_expr($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_object_contents {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_object_pair($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    unless (parse_(($input)) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'object_contents_step3'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'object_contents_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_5 = parse_object_pair($input);
    unless (defined $result_5) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_5;
    my $result_6 = parse_ARRAY(0x7fedef86f820)($input);
    unless (defined $result_6) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_6;
    
    return [$results[1-1], collect_quantified_results(3, \@results)];
}


sub parse_dot_path {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless (parse_(($input)) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'dot_path_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_3 = parse_path_element($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    my $result_4 = parse_ARRAY(0x7fedef82e360)($input);
    unless (defined $result_4) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_4;
    
    return collect_quantified_results(2, \@results);
}


sub parse_number {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'number'}/gc) {
        my @results = ($1);  # Capture regex result
        return $results[1-1];
    }
    return undef;
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
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'nested_array_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_3 = parse_ARRAY(0x7fedef84d668)($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'nested_array_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'nested_array_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return {"type" => "array", "contents" => ($results[3-1] // undef)};
}


sub parse_quoted_string {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'quoted_string'}/gc) {
        my @results = ($1);  # Capture regex result
        return $results[1-1];
    }
    return undef;
}


sub parse_array_contents {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_array_element($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    unless (parse_(($input)) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'array_contents_step3'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'array_contents_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_5 = parse_array_element($input);
    unless (defined $result_5) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_5;
    my $result_6 = parse_ARRAY(0x7fedef8677f0)($input);
    unless (defined $result_6) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_6;
    
    return [$results[1-1], collect_quantified_results(3, \@results)];
}


sub parse_simple_quantifier {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'simple_quantifier'}/gc) {
        my @results = ($1);  # Capture regex result
        return $results[1-1];
    }
    return undef;
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
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'object_pair_step3'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_4 = parse_object_value($input);
    unless (defined $result_4) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_4;
    
    return {"key" => ($results[1-1] // undef), "value" => ($results[4-1] // undef)};
}


sub parse_identifier {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'identifier'}/gc) {
        my @results = ($1);  # Capture regex result
        return $results[1-1];
    }
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
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'return_annotation_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_3 = parse_return_expression($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    
    return \@results;
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

sub parse_return_expression {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_array_expr($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_object_expr($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_scalar_ref($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_literal($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_grouped_expr($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_path_element {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_number($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_identifier($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_array_expr {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'array_expr_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'array_expr_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_3 = parse_ARRAY(0x7fedef866ed8)($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'array_expr_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'array_expr_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_6 = parse_ARRAY(0x7fedef8255c0)($input);
    unless (defined $result_6) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_6;
    
    return {"type" => "array", "contents" => ($results[3-1] // undef), "quantified" => ($results[6-1] // undef)};
}


sub parse_object_expr {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'object_expr_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'object_expr_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_3 = parse_ARRAY(0x7fedef84a9f8)($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'object_expr_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'object_expr_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_6 = parse_ARRAY(0x7fedef831be8)($input);
    unless (defined $result_6) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_6;
    
    return {"type" => "object", "contents" => ($results[3-1] // undef), "quantified" => ($results[6-1] // undef)};
}


sub parse_quantifier {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_simple_quantifier($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_range_quantifier($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_array_element {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_scalar_ref($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_literal($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_nested_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_nested_object($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_range_quantifier {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'range_quantifier_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'range_quantifier_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_3 = parse_number($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless (parse_(($input)) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'range_quantifier_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'range_quantifier_step6'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'range_quantifier_step7'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_8 = parse_ARRAY(0x7fedef830f18)($input);
    unless (defined $result_8) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_8;
    my $result_9 = parse_ARRAY(0x7fedef860620)($input);
    unless (defined $result_9) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_9;
    unless ($$input =~ /\G$REGEXES{'range_quantifier_step10'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'range_quantifier_step11'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return {"min" => ($results[3-1] // undef), "max" => ($results[6-1] // undef)};
}


sub parse_literal {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_quoted_string($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_number($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_boolean($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
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
    push @results, 1;  # Terminal match success
    my $result_2 = parse_number($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    my $result_3 = parse_ARRAY(0x7fedef851020)($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    my $result_4 = parse_ARRAY(0x7fedef84f368)($input);
    unless (defined $result_4) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_4;
    
    return {"type" => "scalar_ref", "index" => ($results[2-1] // undef), "path" => ($results[3-1] // undef), "quantified" => ($results[4-1] // undef)};
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
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'nested_object_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_3 = parse_ARRAY(0x7fedef84d038)($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'nested_object_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'nested_object_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return {"type" => "object", "contents" => ($results[3-1] // undef)};
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
