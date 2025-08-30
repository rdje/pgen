package Json; # Placeholder, will be replaced by tools/ast_transform.pl
use strict;
use warnings;

# Compiled regex patterns for speed
our %REGEXES = (
    'object_step1' => qr/\s*\{\s*/o,
    'object_step3' => qr/\s*\}\s*/o,
    'string' => qr/\s*"[^"]*"\s*/o,
    'array_step1' => qr/\s*\[\s*/o,
    'array_step3' => qr/\s*\]\s*/o,
    'number' => qr/\s*-?[0-9]+(\.[0-9]+)?\s*/o,
    'value' => qr/\s*null\s*/o,
    'pair_step2' => qr/\s*:\s*/o
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

sub parse_members {
    my ($input) = @_;
    my $result = parse_pair($input);
    if (defined $result) {
        return $result;
    }
    return undef;
}


sub parse_object {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'object_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_2 = parse_members($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    unless ($$input =~ /\G$REGEXES{'object_step3'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return \@results;
}


sub parse_string {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'string'}/gc) {
        my @results = ($1);  # Capture regex result
        return 1;
    }
    return undef;
}


sub parse_array {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'array_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_2 = parse_elements($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    unless ($$input =~ /\G$REGEXES{'array_step3'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return \@results;
}


sub parse_number {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'number'}/gc) {
        my @results = ($1);  # Capture regex result
        return 1;
    }
    return undef;
}


sub parse_json {
    my ($input) = @_;
    my $result = parse_value($input);
    if (defined $result) {
        return $result;
    }
    return undef;
}


sub parse_value {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'value'}/gc) {
        my @results = ($1);  # Capture regex result
        return 1;
    }
    return undef;
}


sub parse_elements {
    my ($input) = @_;
    my $result = parse_value($input);
    if (defined $result) {
        return $result;
    }
    return undef;
}


sub parse_pair {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_string($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    unless ($$input =~ /\G$REGEXES{'pair_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_value($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    
    return \@results;
}


# Main entry point
sub parse {
    my ($input) = @_;
    pos($$input) = 0;
    return parse_json($input);
}

1;
