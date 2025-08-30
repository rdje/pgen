package yapg::GeneratedParser;
use strict;
use warnings;

# Compiled regex patterns for speed
my %REGEXES = (
    'simple_array_step1' => qr/\Q[\E/o,
    'simple_array_step5' => qr/\Q]\E/o,
    'quantifier' => qr/\Q*\E/o,
    'simple_object_step1' => qr/\Q{\E/o,
    'simple_object_step5' => qr/\Q}\E/o,
    'identifier' => qr/([a-zA-Z_][a-zA-Z0-9_]*)/o,
    'whitespace' => qr/\s+/o,
    'quoted_string' => qr/"[^"]*"/o,
    'number' => qr/(\d+)/o,
    'return_expression_alt3' => qr/"[^"]*"/o,
    'return_expression_alt4' => qr/(\d+)/o,
    'return_annotation_step1' => qr/\Q->\E/o,
    'object_pair_step1' => qr/([a-zA-Z_][a-zA-Z0-9_]*)/o,
    'object_pair_step3' => qr/\Q:\E/o,
    'scalar_ref_step1' => qr/\Q$\E/o
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
    my $result_2 = parse_ARRAY(0x7fe1a597e310)($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    my $result_3 = parse_array_element($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    my $result_4 = parse_ARRAY(0x7fe1a597e3d0)($input);
    unless (defined $result_4) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_4;
    unless ($$input =~ /\G$REGEXES{'simple_array_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return {"type" => "array", "contents" => [($results[3-1] // undef)]};
}


sub parse_quantifier {
    my ($input) = @_;
    return 1 if $$input =~ /\G$REGEXES{'quantifier'}/gc;
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
    my $result_2 = parse_ARRAY(0x7fe1a597e6d0)($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    my $result_3 = parse_object_pair($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    my $result_4 = parse_ARRAY(0x7fe1a597e790)($input);
    unless (defined $result_4) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_4;
    unless ($$input =~ /\G$REGEXES{'simple_object_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return {"type" => "object", "contents" => [($results[3-1] // undef)]};
}


sub parse_identifier {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'identifier'}/gc) {
        my @results = ($1);  # Capture regex result
        return $results[1-1];
    }
    return undef;
}


sub parse_whitespace {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'whitespace'}/gc) {
        my @results = ($1);  # Capture regex result
        return 1;
    }
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


sub parse_array_element {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_scalar_ref($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_literal($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_number {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'number'}/gc) {
        my @results = ($1);  # Capture regex result
        return $results[1-1];
    }
    return undef;
}


sub parse_return_expression {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_scalar_ref($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_simple_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_simple_object($input))) { return $alt_result; }
    if (my $alt_result = $$input =~ /\G$REGEXES{'return_expression_alt3'}/gc) { return $alt_result; }
    if (my $alt_result = $$input =~ /\G$REGEXES{'return_expression_alt4'}/gc) { return $alt_result; }
    
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
    my $result_2 = parse_ARRAY(0x7fe1a5933ab8)($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    my $result_3 = parse_return_expression($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    
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

sub parse_object_pair {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'object_pair_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_2 = parse_ARRAY(0x7fe1a597df50)($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    unless ($$input =~ /\G$REGEXES{'object_pair_step3'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_4 = parse_ARRAY(0x7fe1a597e0b8)($input);
    unless (defined $result_4) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_4;
    my $result_5 = parse_return_expression($input);
    unless (defined $result_5) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_5;
    
    return {"key" => ($results[1-1] // undef), "value" => ($results[5-1] // undef)};
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
    my $result_3 = parse_ARRAY(0x7fe1a5911908)($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    
    return {"type" => "scalar_ref", "index" => ($results[2-1] // undef), "quantified" => ($results[3-1] // undef)};
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
