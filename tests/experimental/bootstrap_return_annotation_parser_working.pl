package yapg::GeneratedParser;
use strict;
use warnings;

# Compiled regex patterns for speed
my %REGEXES = (
    'ARRAY(0x7fa6f2816a10)' => qr/"[^"]*"/o,
    'ARRAY(0x7fa6f2816b48)' => qr/(\d+)/o
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
sub parse_ARRAY(0x7fa6f2816080) {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_ARRAY(0x7fa6f3062540)($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    my $result_2 = parse_ARRAY(0x7fa6f3062588)($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    
    return \@results;
}


sub parse_ARRAY(0x7fa6f28165a8) {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_ARRAY(0x7fa6f3062360)($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    my $result_2 = parse_ARRAY(0x7fa6f30623a8)($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    
    return {"type" => "scalar_ref", "index" => ($results[2-1] // undef)};
}


sub parse_ARRAY(0x7fa6f2816a10) {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'ARRAY(0x7fa6f2816a10)'}/gc) {
        my @results = ($1);  # Capture regex result
        return $results[1-1];
    }
    return undef;
}


sub parse_ARRAY(0x7fa6f2816980) {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_ARRAY(0x7fa6f30621f8)($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_ARRAY(0x7fa6f3062150)($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_ARRAY(0x7fa6f2816b48) {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'ARRAY(0x7fa6f2816b48)'}/gc) {
        my @results = ($1);  # Capture regex result
        return $results[1-1];
    }
    return undef;
}


sub parse_ARRAY(0x7fa6f20bc530) {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_ARRAY(0x7fa6f3062108)($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_ARRAY(0x7fa6f3061b08)($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

# Main entry point
sub parse {
    my ($input_ref) = @_;
    pos($$input_ref) = 0;
    my $result = parse_ARRAY(0x7fa6f2816080)($input_ref);
    
    # Check that entire input was consumed
    if (defined $result && pos($$input_ref) == length($$input_ref)) {
        return $result;
    } else {
        return undef;  # Partial match or unconsumed input
    }
}

1;
