package yapg::GeneratedParser;
use strict;
use warnings;

# Compiled regex patterns for speed
my %REGEXES = (
    'port_clause_step1' => qr/port/o,
    'port_clause_step2' => qr/\(/o,
    'port_clause_step3' => qr/\)/o,
    'simple_entity_step1' => qr/entity/o,
    'simple_entity_step3' => qr/is/o,
    'simple_entity_step4' => qr/end/o,
    'simple_entity_step5' => qr/;/o,
    'identifier' => qr/([a-zA-Z][a-zA-Z0-9_]*)/o,
    'id_list_step3' => qr/\Q*\E/o,
    'mode_alt0' => qr/in/o,
    'mode_alt1' => qr/out/o,
    'optional_test_step1' => qr/([a-zA-Z][a-zA-Z0-9_]*)/o,
    'optional_test_step3' => qr/\Q?\E/o
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
sub parse_port_clause {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'port_clause_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'port_clause_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'port_clause_step3'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return $results["empty_ports"-1];
}


sub parse_simple_entity {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'simple_entity_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_2 = parse_identifier($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    unless ($$input =~ /\G$REGEXES{'simple_entity_step3'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'simple_entity_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'simple_entity_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return {"name" => ($results[2-1] // undef)};
}


sub parse_identifier {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'identifier'}/gc) {
        my @results = ($1);  # Capture regex result
        return $results[1-1];
    }
    return undef;
}


sub parse_id_list {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_identifier($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    unless ($$input =~ /\G$REGEXES{'id_list_step3'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return [$results[1-1], collect_quantified_results(2, \@results)];
}


sub parse_mode {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = $$input =~ /\G$REGEXES{'mode_alt0'}/gc) { return $alt_result; }
    if (my $alt_result = $$input =~ /\G$REGEXES{'mode_alt1'}/gc) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_optional_test {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'optional_test_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_2 = parse_port_clause($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    unless ($$input =~ /\G$REGEXES{'optional_test_step3'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return {"name" => ($results[1-1] // undef), "ports" => ($results[2-1] // undef)};
}


# Main entry point
sub parse {
    my ($input_ref) = @_;
    pos($$input_ref) = 0;
    my $result = parse_simple_entity($input_ref);
    
    # Check that entire input was consumed
    if (defined $result && pos($$input_ref) == length($$input_ref)) {
        return $result;
    } else {
        return undef;  # Partial match or unconsumed input
    }
}

1;
