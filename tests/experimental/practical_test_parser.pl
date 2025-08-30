package yapg::GeneratedParser;
use strict;
use warnings;

# Compiled regex patterns for speed
my %REGEXES = (
    'name_step1' => qr/([A-Z][a-z]+)/o,
    'age_step1' => qr/([0-9]+)/o,
    'simple_value' => qr/\Qhello\E/o,
    'location_step1' => qr/([A-Z][a-z]+)/o,
    'value_step1' => qr/([0-9]+)/o,
    'word_step1' => qr/(\w+)/o,
    'debug_step1' => qr/(true|false)/o,
    'port_step1' => qr/([0-9]+)/o,
    'key_step1' => qr/([a-z_]+)/o
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


sub parse_config {
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
    my $result_2 = parse_port($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    my $result_3 = parse_debug($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    
    return {"service_name" => ($results[0] // undef), "port_number" => ($results[1] // undef), "debug_enabled" => ($results[2] // undef), "version" => "1.0"};
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


sub parse_debug {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'debug_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return $results[0];
}


sub parse_port {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'port_step1'}/gc) {
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
