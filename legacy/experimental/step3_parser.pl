package yapg::GeneratedParser;
use strict;
use warnings;

# Compiled regex patterns for speed
my %REGEXES = (
    'identifier' => qr/([a-zA-Z_]\w*)/o,
    'return_annotation_step1' => qr/\Q->\E/o,
    'return_annotation_step2' => qr/\s*/o,
    'number' => qr/(\d+)/o,
    'simple_array_step1' => qr/\Q[\E/o,
    'simple_array_step2' => qr/\s*/o,
    'simple_array_step4' => qr/\s*/o,
    'simple_array_step5' => qr/\Q]\E/o,
    'simple_object_step1' => qr/\Q{\E/o,
    'simple_object_step2' => qr/\s*/o,
    'simple_object_step4' => qr/\s*/o,
    'simple_object_step5' => qr/\Q:\E/o,
    'simple_object_step6' => qr/\s*/o,
    'simple_object_step8' => qr/\s*/o,
    'simple_object_step9' => qr/\Q}\E/o,
    'quoted_string' => qr/"([^"]*)"/o,
    'scalar_ref_step1' => qr/\$/o,
    'quantified_array_step1' => qr/\Q[\E/o,
    'quantified_array_step2' => qr/\s*/o,
    'quantified_array_step4' => qr/\s*/o,
    'quantified_array_step5' => qr/\Q]\E/o,
    'quantifier' => qr/([\*\+\?])/o
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

sub parse_identifier {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'identifier'}/gc) {
        my @results = ($1);  # Capture regex result
        return $results[1-1];
    }
    return undef;
}


sub parse_object_value {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_scalar_ref($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_literal($input))) { return $alt_result; }
    
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


sub parse_number {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'number'}/gc) {
        my @results = ($1);  # Capture regex result
        return $results[1-1];
    }
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
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'simple_array_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_3 = parse_scalar_ref($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'simple_array_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'simple_array_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return {"type" => "array", "element" => ($results[3-1] // undef)};
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

sub parse_simple_object {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'simple_object_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'simple_object_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
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
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'simple_object_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'simple_object_step6'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
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
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'simple_object_step9'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return {"type" => "object", "key" => ($results[3-1] // undef), "value" => ($results[6-1] // undef)};
}


sub parse_return_expression {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_quantified_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_simple_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_simple_object($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_scalar_ref($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_quoted_string {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'quoted_string'}/gc) {
        my @results = ($1);  # Capture regex result
        return $results[1-1];
    }
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
    
    return {"type" => "scalar_ref", "index" => ($results[2-1] // undef)};
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
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'quantified_array_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_3 = parse_scalar_ref($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'quantified_array_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'quantified_array_step5'}/gc) {
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
    
    return {"type" => "array", "element" => ($results[3-1] // undef), "quantified" => ($results[6-1] // undef)};
}


sub parse_quantifier {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'quantifier'}/gc) {
        my @results = ($1);  # Capture regex result
        return $results[1-1];
    }
    return undef;
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
