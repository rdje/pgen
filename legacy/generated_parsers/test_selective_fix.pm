package yapg::GeneratedParser;
use strict;
use warnings;

# Compiled regex patterns for speed
my %REGEXES = (
    'list_prime_alt0_0' => qr/\s*,\s*/o,
    'factor_alt0_0' => qr/\s*\(\s*/o,
    'factor_alt0_2' => qr/\s*\)\s*/o,
    'factor_alt1' => qr/(\w+)\s*/o,
    'factor_alt2' => qr/(\d+)\s*/o,
    'term_alt0_0' => qr/\s*\(\s*/o,
    'term_alt0_2' => qr/\s*\)\s*/o,
    'term_alt1_0' => qr/(\w+)\s*/o,
    'term_alt2_0' => qr/(\d+)\s*/o,
    'term_prime_alt0_0' => qr/\s*\*\s*/o,
    'term_prime_alt1_0' => qr/\s*\/\s*/o,
    'item_alt0_0' => qr/\s*\[\s*/o,
    'item_alt0_2' => qr/\s*\]\s*/o,
    'item_alt1' => qr/(\w+)\s*/o,
    'item_alt2' => qr/(\d+)\s*/o,
    'expr_prime_alt0_0' => qr/\s*\+\s*/o,
    'expr_prime_alt1_0' => qr/\s*-\s*/o,
    'list_alt0_0' => qr/\s*\[\s*/o,
    'list_alt0_2' => qr/\s*\]\s*/o,
    'list_alt1_0' => qr/(\w+)\s*/o,
    'list_alt2_0' => qr/(\d+)\s*/o
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
sub parse_expr {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_term($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    my $result_2 = parse_expr_prime($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    
    return {"left" => ($results[1-1] // undef), "op" => ($results[2-1] // undef), "right" => ($results[3-1] // undef)};
}


sub parse_list_prime {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); my @seq_results = (); my $step_result_0 = ($$input =~ /\G$REGEXES{'list_prime_alt0_0'}/gc); return (pos($$input) = $seq_pos, undef) unless $step_result_0; push @seq_results, $step_result_0; my $step_result_1 = (parse_item($input)); return (pos($$input) = $seq_pos, undef) unless $step_result_1; push @seq_results, $step_result_1; my $step_result_2 = (parse_list_prime($input)); return (pos($$input) = $seq_pos, undef) unless $step_result_2; push @seq_results, $step_result_2; \@seq_results }) { return $alt_result; }
    if (my $alt_result = 1) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_factor {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); my @seq_results = (); my $step_result_0 = ($$input =~ /\G$REGEXES{'factor_alt0_0'}/gc); return (pos($$input) = $seq_pos, undef) unless $step_result_0; push @seq_results, $step_result_0; my $step_result_1 = (parse_expr($input)); return (pos($$input) = $seq_pos, undef) unless $step_result_1; push @seq_results, $step_result_1; my $step_result_2 = ($$input =~ /\G$REGEXES{'factor_alt0_2'}/gc); return (pos($$input) = $seq_pos, undef) unless $step_result_2; push @seq_results, $step_result_2; \@seq_results }) { return $alt_result; }
    if (my $alt_result = do { if ($$input =~ /\G$REGEXES{'factor_alt1'}/gc) { {"type" => "identifier", "name" => $1} } else { undef } }) { return $alt_result; }
    if (my $alt_result = do { if ($$input =~ /\G$REGEXES{'factor_alt2'}/gc) { {"type" => "number", "value" => $1} } else { undef } }) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_term {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); my @seq_results = (); my $step_result_0 = ($$input =~ /\G$REGEXES{'term_alt0_0'}/gc); return (pos($$input) = $seq_pos, undef) unless $step_result_0; push @seq_results, $step_result_0; my $step_result_1 = (parse_expr($input)); return (pos($$input) = $seq_pos, undef) unless $step_result_1; push @seq_results, $step_result_1; my $step_result_2 = ($$input =~ /\G$REGEXES{'term_alt0_2'}/gc); return (pos($$input) = $seq_pos, undef) unless $step_result_2; push @seq_results, $step_result_2; my $step_result_3 = (parse_term_prime($input)); return (pos($$input) = $seq_pos, undef) unless $step_result_3; push @seq_results, $step_result_3; \@seq_results }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); my @seq_results = (); my $step_result_0 = ($$input =~ /\G$REGEXES{'term_alt1_0'}/gc); return (pos($$input) = $seq_pos, undef) unless $step_result_0; push @seq_results, $step_result_0; my $step_result_1 = (parse_term_prime($input)); return (pos($$input) = $seq_pos, undef) unless $step_result_1; push @seq_results, $step_result_1; \@seq_results }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); my @seq_results = (); my $step_result_0 = ($$input =~ /\G$REGEXES{'term_alt2_0'}/gc); return (pos($$input) = $seq_pos, undef) unless $step_result_0; push @seq_results, $step_result_0; my $step_result_1 = (parse_term_prime($input)); return (pos($$input) = $seq_pos, undef) unless $step_result_1; push @seq_results, $step_result_1; \@seq_results }) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_term_prime {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); my @seq_results = (); my $step_result_0 = ($$input =~ /\G$REGEXES{'term_prime_alt0_0'}/gc); return (pos($$input) = $seq_pos, undef) unless $step_result_0; push @seq_results, $step_result_0; my $step_result_1 = (parse_factor($input)); return (pos($$input) = $seq_pos, undef) unless $step_result_1; push @seq_results, $step_result_1; my $step_result_2 = (parse_term_prime($input)); return (pos($$input) = $seq_pos, undef) unless $step_result_2; push @seq_results, $step_result_2; \@seq_results }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); my @seq_results = (); my $step_result_0 = ($$input =~ /\G$REGEXES{'term_prime_alt1_0'}/gc); return (pos($$input) = $seq_pos, undef) unless $step_result_0; push @seq_results, $step_result_0; my $step_result_1 = (parse_factor($input)); return (pos($$input) = $seq_pos, undef) unless $step_result_1; push @seq_results, $step_result_1; my $step_result_2 = (parse_term_prime($input)); return (pos($$input) = $seq_pos, undef) unless $step_result_2; push @seq_results, $step_result_2; \@seq_results }) { return $alt_result; }
    if (my $alt_result = 1) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_item {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); my @seq_results = (); my $step_result_0 = ($$input =~ /\G$REGEXES{'item_alt0_0'}/gc); return (pos($$input) = $seq_pos, undef) unless $step_result_0; push @seq_results, $step_result_0; my $step_result_1 = (parse_list($input)); return (pos($$input) = $seq_pos, undef) unless $step_result_1; push @seq_results, $step_result_1; my $step_result_2 = ($$input =~ /\G$REGEXES{'item_alt0_2'}/gc); return (pos($$input) = $seq_pos, undef) unless $step_result_2; push @seq_results, $step_result_2; \@seq_results }) { return $alt_result; }
    if (my $alt_result = do { if ($$input =~ /\G$REGEXES{'item_alt1'}/gc) { {"type" => "id", "value" => $1} } else { undef } }) { return $alt_result; }
    if (my $alt_result = do { if ($$input =~ /\G$REGEXES{'item_alt2'}/gc) { {"type" => "num", "value" => $1} } else { undef } }) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_expr_prime {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); my @seq_results = (); my $step_result_0 = ($$input =~ /\G$REGEXES{'expr_prime_alt0_0'}/gc); return (pos($$input) = $seq_pos, undef) unless $step_result_0; push @seq_results, $step_result_0; my $step_result_1 = (parse_term($input)); return (pos($$input) = $seq_pos, undef) unless $step_result_1; push @seq_results, $step_result_1; my $step_result_2 = (parse_expr_prime($input)); return (pos($$input) = $seq_pos, undef) unless $step_result_2; push @seq_results, $step_result_2; \@seq_results }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); my @seq_results = (); my $step_result_0 = ($$input =~ /\G$REGEXES{'expr_prime_alt1_0'}/gc); return (pos($$input) = $seq_pos, undef) unless $step_result_0; push @seq_results, $step_result_0; my $step_result_1 = (parse_term($input)); return (pos($$input) = $seq_pos, undef) unless $step_result_1; push @seq_results, $step_result_1; my $step_result_2 = (parse_expr_prime($input)); return (pos($$input) = $seq_pos, undef) unless $step_result_2; push @seq_results, $step_result_2; \@seq_results }) { return $alt_result; }
    if (my $alt_result = 1) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_list {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); my @seq_results = (); my $step_result_0 = ($$input =~ /\G$REGEXES{'list_alt0_0'}/gc); return (pos($$input) = $seq_pos, undef) unless $step_result_0; push @seq_results, $step_result_0; my $step_result_1 = (parse_list($input)); return (pos($$input) = $seq_pos, undef) unless $step_result_1; push @seq_results, $step_result_1; my $step_result_2 = ($$input =~ /\G$REGEXES{'list_alt0_2'}/gc); return (pos($$input) = $seq_pos, undef) unless $step_result_2; push @seq_results, $step_result_2; my $step_result_3 = (parse_list_prime($input)); return (pos($$input) = $seq_pos, undef) unless $step_result_3; push @seq_results, $step_result_3; \@seq_results }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); my @seq_results = (); my $step_result_0 = ($$input =~ /\G$REGEXES{'list_alt1_0'}/gc); return (pos($$input) = $seq_pos, undef) unless $step_result_0; push @seq_results, $step_result_0; my $step_result_1 = (parse_list_prime($input)); return (pos($$input) = $seq_pos, undef) unless $step_result_1; push @seq_results, $step_result_1; \@seq_results }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); my @seq_results = (); my $step_result_0 = ($$input =~ /\G$REGEXES{'list_alt2_0'}/gc); return (pos($$input) = $seq_pos, undef) unless $step_result_0; push @seq_results, $step_result_0; my $step_result_1 = (parse_list_prime($input)); return (pos($$input) = $seq_pos, undef) unless $step_result_1; push @seq_results, $step_result_1; \@seq_results }) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

# Main entry point
sub parse {
    my ($input_ref) = @_;
    pos($$input_ref) = 0;
    my $result = parse_expr($input_ref);
    
    # Check that entire input was consumed
    if (defined $result && pos($$input_ref) == length($$input_ref)) {
        return $result;
    } else {
        return undef;  # Partial match or unconsumed input
    }
}

1;
