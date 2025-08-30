package Test_debug; # Placeholder, will be replaced by tools/ast_transform.pl
use strict;
use warnings;

# Compiled regex patterns for speed
our %REGEXES = (
    'return_expression' => qr/([a-zA-Z_][a-zA-Z0-9_]*)/o,
    'array_contents_group_step1' => qr/\Q,\E/o
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

sub parse_return_expression {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'return_expression'}/gc) {
        my @results = ($1);  # Capture regex result
        return 1;
    }
    return undef;
}


sub parse_array_contents {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    # Parse first required element
    my $result_1 = parse_return_expression($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    
    # Parse optional repeated grouped elements in loop
    while (1) {
        my $loop_start_pos = pos($$input);
        
        unless ($$input =~ /\G$REGEXES{'array_contents_group_step1'}/gc) {
            pos($$input) = $loop_start_pos;
            last;
        }
        my $group_result = parse_return_expression($input);
        unless (defined $group_result) {
            pos($$input) = $loop_start_pos;
            last;
        }
        push @results, $group_result;
    }

    
    return \@results;
}


# Main entry point
sub parse {
    my ($input) = @_;
    pos($$input) = 0;
    return parse_array_contents($input);
}

1;
