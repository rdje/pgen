package TestLRFixed; # Placeholder, will be replaced by tools/ast_transform.pl
use strict;
use warnings;

# Compiled regex patterns for speed
our %REGEXES = (
    'expr_step1' => qr/(num)/o,
    'expr_prime_alt0_0' => qr/(\+)/o,
    'expr_prime_alt0_1' => qr/(num)/o
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

sub parse_expr {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'expr_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_2 = parse_expr_prime($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    
    return $1;
}


sub parse_expr_prime {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'expr_prime_alt0_0'}/gc) && ($$input =~ /\G$REGEXES{'expr_prime_alt0_1'}/gc) && (parse_expr_prime($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = 1) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

# Main entry point
sub parse {
    my ($input) = @_;
    pos($$input) = 0;
    return parse_expr($input);
}

1;
