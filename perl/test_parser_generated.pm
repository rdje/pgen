package Test_parser_generated; # Placeholder, will be replaced by tools/ast_transform.pl
use strict;
use warnings;

# Compiled regex patterns for speed
our %REGEXES = (
    'word' => qr/[a-zA-Z]+/o,
    'identifier' => qr/[a-zA-Z_]\w*/o,
    'number' => qr/\d+/o
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

sub parse_word {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'word'}/gc) {
        my @results = ($1);  # Capture regex result
        return 1;
    }
    return undef;
}


sub parse_expression_list {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    # Universal quantified sequence: parse all elements in order
    # Rule call: expression
    my $atom_result_1 = parse_expression($input);
    unless (defined $atom_result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $atom_result_1;

    # FIXED: Enhanced fallback for element type 


    
    return \@results;
}


sub parse_expression {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_identifier($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_number($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
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


sub parse_word_sequence {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    # Universal quantified sequence: parse all elements in order
    # Rule call: word
    my $atom_result_1 = parse_word($input);
    unless (defined $atom_result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $atom_result_1;

    # FIXED: Enhanced fallback for element type 


    
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


sub parse_number_list {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    # Universal quantified sequence: parse all elements in order
    # Rule call: number
    my $atom_result_1 = parse_number($input);
    unless (defined $atom_result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $atom_result_1;

    # FIXED: Enhanced fallback for element type 


    
    return \@results;
}


# Main entry point
sub parse {
    my ($input) = @_;
    pos($$input) = 0;
    return parse_number_list($input);
}

1;
