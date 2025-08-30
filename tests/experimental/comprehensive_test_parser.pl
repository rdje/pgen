package yapg::GeneratedParser;
use strict;
use warnings;

# Compiled regex patterns for speed
my %REGEXES = (
    'simple_value' => qr/\Qhello\E/o,
    'location_step1' => qr/([A-Z][a-z]+)/o,
    'value_step1' => qr/([0-9]+)/o,
    'id_step1' => qr/([A-Z0-9]+)/o,
    'name_step1' => qr/([A-Z][a-z]+)/o,
    'item_step1' => qr/(\w+)/o,
    'age_step1' => qr/([0-9]+)/o,
    'key_step1' => qr/([a-z_]+)/o,
    'word_step1' => qr/(\w+)/o,
    'count_step1' => qr/([0-9]+)/o
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
sub parse_simple_obj {
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
    my $result_2 = parse_value($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    
    return {"key" => ($results[0] // undef), "value" => ($results[1] // undef)};
}


sub parse_simple_value {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'simple_value'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return $results[0];
}


sub parse_triple_list {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_word($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    my $result_2 = parse_word($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    my $result_3 = parse_word($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    
    return [$results[0], $results[1], $results[2]];
}


sub parse_location {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'location_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return $results[0];
}


sub parse_value {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'value_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return $results[0];
}


sub parse_person {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_name($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    my $result_2 = parse_age($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    my $result_3 = parse_location($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    
    return {"name" => ($results[0] // undef), "age" => ($results[1] // undef), "location" => ($results[2] // undef)};
}


sub parse_id {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'id_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return $results[0];
}


sub parse_name {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'name_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return $results[0];
}


sub parse_list_items {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_ARRAY(0x7fcc3e4e3658)($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    
    return collect_quantified_results(1, \@results);
}


sub parse_item {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'item_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return $results[0];
}


sub parse_age {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'age_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return $results[0];
}


sub parse_key {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'key_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return $results[0];
}


sub parse_simple_list {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_word($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    my $result_2 = parse_word($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    
    return [$results[0], $results[1]];
}


sub parse_word {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'word_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return $results[0];
}


sub parse_data_structure {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_id($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    my $result_2 = parse_items($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    my $result_3 = parse_count($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless (parse_quantifier:
    id: $1,
    items: [$2, $3],
    metadata: count: $3, type: "list"($input)) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return \@results;
}


sub parse_count {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'count_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return $results[0];
}


# Main entry point
sub parse {
    my ($input_ref) = @_;
    pos($$input_ref) = 0;
    my $result = parse_simple_value($input_ref);
    
    # Check that entire input was consumed
    if (defined $result && pos($$input_ref) == length($$input_ref)) {
        return $result;
    } else {
        return undef;  # Partial match or unconsumed input
    }
}

1;
