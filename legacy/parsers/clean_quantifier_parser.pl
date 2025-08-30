package yapg::GeneratedParser;
use strict;
use warnings;

# Compiled regex patterns for speed
my %REGEXES = (
    'signal_step1' => qr/([a-zA-Z_][a-zA-Z0-9_]*)/o,
    'signal_step2' => qr/\Q:\E/o,
    'item' => qr/\Qtoken\E/o,
    'zero_or_more' => qr/\Qtoken*\E/o,
    'item_list' => qr/\Qtoken+\E/o,
    'optional_group' => qr/\QARRAY(0x7fd82308f790)?\E/o,
    'identifier' => qr/([a-zA-Z_][a-zA-Z0-9_]*)/o,
    'grouped_items' => qr/\QARRAY(0x7fd821984938)+\E/o,
    'direction' => qr/\Qin|out\E/o,
    'optional_item' => qr/\Qtoken?\E/o,
    'signal_decl_step1' => qr/([a-zA-Z_][a-zA-Z0-9_]*)/o,
    'signal_decl_step2' => qr/\Q:\E/o,
    'signal_decl_step4' => qr/\Q*\E/o,
    'port_list_step1' => qr/([a-zA-Z_][a-zA-Z0-9_]*)/o,
    'port_list_step2' => qr/\Q:\E/o,
    'port_list_step4' => qr/\Q+\E/o,
    'signal_type' => qr/\Qstd_logic|integer\E/o,
    'other' => qr/\Qalt\E/o,
    'port_step1' => qr/([a-zA-Z_][a-zA-Z0-9_]*)/o,
    'port_step2' => qr/\Q:\E/o
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
sub parse_signal {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'signal_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'signal_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_3 = parse_signal_type($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    
    return {"name" => ($results[1-1] // undef), "type" => ($results[3-1] // undef)};
}


sub parse_item {
    my ($input) = @_;
    return 1 if $$input =~ /\G$REGEXES{'item'}/gc;
    return undef;
}


sub parse_zero_or_more {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'zero_or_more'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return collect_quantified_results(1, \@results);
}


sub parse_item_list {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'item_list'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return collect_quantified_results(1, \@results);
}


sub parse_optional_group {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'optional_group'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return $results[1-1];
}


sub parse_identifier {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'identifier'}/gc) {
        my @results = ($1);  # Capture regex result
        return $results[1-1];
    }
    return undef;
}


sub parse_grouped_items {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'grouped_items'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return collect_quantified_results(1, \@results);
}


sub parse_direction {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'direction'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return $results["output"-1];
}


sub parse_optional_item {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'optional_item'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return $results[1-1];
}


sub parse_signal_decl {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'signal_decl_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'signal_decl_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_3 = parse_signal_type($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'signal_decl_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return collect_quantified_results(1, \@results);
}


sub parse_port_list {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'port_list_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    unless ($$input =~ /\G$REGEXES{'port_list_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_3 = parse_direction($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'port_list_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return collect_quantified_results(1, \@results);
}


sub parse_signal_type {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'signal_type'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    
    return $results["int"-1];
}


sub parse_other {
    my ($input) = @_;
    return 1 if $$input =~ /\G$REGEXES{'other'}/gc;
    return undef;
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
    unless ($$input =~ /\G$REGEXES{'port_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, 1;  # Terminal match success
    my $result_3 = parse_direction($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    
    return {"name" => ($results[1-1] // undef), "dir" => ($results[3-1] // undef)};
}


# Main entry point
sub parse {
    my ($input_ref) = @_;
    pos($$input_ref) = 0;
    my $result = parse_item_list($input_ref);
    
    # Check that entire input was consumed
    if (defined $result && pos($$input_ref) == length($$input_ref)) {
        return $result;
    } else {
        return undef;  # Partial match or unconsumed input
    }
}

1;
