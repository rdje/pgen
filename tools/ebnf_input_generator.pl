#!/usr/bin/perl
use strict;
use warnings;
use Data::Dumper;

# EBNF-based input generator with probability support
# Usage: perl ebnf_input_generator.pl <grammar.ebnf> [options]

use lib 'fx/perl';
use LinkedSpec;

my $ebnf_file = $ARGV[0] || die "Usage: $0 <grammar.ebnf> [--count N] [--max-depth N]\n";

# Parse command line options
my %opts = (
    count => 10,        # Number of inputs to generate
    max_depth => 5,     # Maximum recursion depth
    seed => time(),     # Random seed
);

for my $i (1..$#ARGV) {
    if ($ARGV[$i] eq '--count' && $i < $#ARGV) {
        $opts{count} = $ARGV[$i+1];
    } elsif ($ARGV[$i] eq '--max-depth' && $i < $#ARGV) {
        $opts{max_depth} = $ARGV[$i+1];
    } elsif ($ARGV[$i] eq '--seed' && $i < $#ARGV) {
        $opts{seed} = $ARGV[$i+1];
    }
}

srand($opts{seed});
print "# Generated with seed: $opts{seed}\n";

# Parse the EBNF grammar
open my $fh, "<", "fx/specs/ebnf.spec" or die "Cannot open ebnf.spec: $!";
my $spec_content = do { local $/; <$fh> };
close $fh;

open my $fh2, "<", $ebnf_file or die "Cannot open $ebnf_file: $!";
my $input_content = do { local $/; <$fh2> };
close $fh2;

my $parser = LinkedSpec::Get(\$spec_content);
my $raw_ast = $parser->(\$input_content);

# Build grammar structure with probabilities
my %grammar = ();
my @rule_order = ();

for my $rule (@$raw_ast) {
    next if @$rule < 2;
    my $rule_name = $rule->[0];
    push @rule_order, $rule_name unless exists $grammar{$rule_name};
    
    # Parse rule elements with probability support
    my @elements = @$rule[1..$#{$rule}];
    $grammar{$rule_name} = parse_rule_with_probabilities(\@elements);
}

my $main_rule = $rule_order[0];
print "# Main rule: $main_rule\n";

# Generate input files
for my $i (1..$opts{count}) {
    print "# Input $i:\n";
    my $generated = generate_from_rule($main_rule, 0, \%opts);
    print "$generated\n\n";
}

sub parse_rule_with_probabilities {
    my ($elements) = @_;
    
    # Split by pipe operator to get alternatives
    my @alternatives = ();
    my @current_alt = ();
    
    for my $element (@$elements) {
        if (defined $element && $element eq '|') {
            if (@current_alt) {
                push @alternatives, parse_alternative_with_probability(\@current_alt);
                @current_alt = ();
            }
        } else {
            push @current_alt, $element;
        }
    }
    push @alternatives, parse_alternative_with_probability(\@current_alt) if @current_alt;
    
    # Validate and normalize probabilities for OR alternatives
    if (@alternatives > 1) {
        @alternatives = validate_and_normalize_probabilities(\@alternatives);
        return { type => 'or', alternatives => \@alternatives };
    } else {
        return $alternatives[0];
    }
}

sub parse_alternative_with_probability {
    my ($elements) = @_;
    
    my $probability = 100;  # Default probability
    my @content = ();
    
    # Extract probability if present (should be last element)
    for my $element (@$elements) {
        if (ref($element) eq 'ARRAY' && $element->[0] eq 'probability') {
            $probability = $element->[1];
        } else {
            push @content, $element;
        }
    }
    
    if (@content == 1) {
        return { type => 'element', content => $content[0], probability => $probability };
    } else {
        return { type => 'sequence', elements => \@content, probability => $probability };
    }
}

sub generate_from_rule {
    my ($rule_name, $depth, $opts) = @_;
    
    # Prevent infinite recursion
    if ($depth > $opts->{max_depth}) {
        return "";
    }
    
    my $rule_def = $grammar{$rule_name};
    return generate_from_definition($rule_def, $depth, $opts);
}

sub generate_from_definition {
    my ($def, $depth, $opts) = @_;
    
    if ($def->{type} eq 'or') {
        # Choose alternative based on probabilities
        return generate_from_or_alternatives($def->{alternatives}, $depth, $opts);
    } elsif ($def->{type} eq 'element') {
        return generate_from_element($def->{content}, $depth, $opts);
    } elsif ($def->{type} eq 'sequence') {
        my @parts = ();
        for my $element (@{$def->{elements}}) {
            push @parts, generate_from_element($element, $depth, $opts);
        }
        return join(" ", @parts);
    }
    
    return "";
}

sub generate_from_or_alternatives {
    my ($alternatives, $depth, $opts) = @_;
    
    # Calculate total probability
    my $total_prob = 0;
    for my $alt (@$alternatives) {
        $total_prob += $alt->{probability} || 100;
    }
    
    # Choose randomly based on probabilities
    my $rand_val = rand($total_prob);
    my $cumulative = 0;
    
    for my $alt (@$alternatives) {
        $cumulative += $alt->{probability} || 100;
        if ($rand_val <= $cumulative) {
            return generate_from_definition($alt, $depth, $opts);
        }
    }
    
    # Fallback to first alternative
    return generate_from_definition($alternatives->[0], $depth, $opts);
}

sub generate_from_element {
    my ($element, $depth, $opts) = @_;
    
    if (ref($element) eq 'ARRAY' && $element->[0] eq 'terminal') {
        return $element->[1];
    } elsif (!ref($element) && exists $grammar{$element}) {
        return generate_from_rule($element, $depth + 1, $opts);
    }
    
    return "";
}

sub validate_and_normalize_probabilities {
    my ($alternatives) = @_;
    
    # Calculate total of specified probabilities
    my $specified_total = 0;
    my $unspecified_count = 0;
    
    for my $alt (@$alternatives) {
        if (defined $alt->{probability} && $alt->{probability} != 100) {
            $specified_total += $alt->{probability};
        } else {
            $unspecified_count++;
        }
    }
    
    # Case 1: All probabilities specified
    if ($unspecified_count == 0) {
        if ($specified_total != 100) {
            die "ERROR: Probabilities sum to $specified_total%, must equal 100%\n";
        }
        return @$alternatives;  # Already valid
    }
    
    # Case 2: Some probabilities unspecified
    if ($specified_total >= 100) {
        die "ERROR: Specified probabilities ($specified_total%) exceed 100%\n";
    }
    
    # Distribute remaining probability equally among unspecified alternatives
    my $remaining = 100 - $specified_total;
    my $default_prob = int($remaining / $unspecified_count);
    my $extra = $remaining % $unspecified_count;
    
    my @normalized = ();
    my $extra_distributed = 0;
    
    for my $alt (@$alternatives) {
        my %new_alt = %$alt;  # Copy
        
        if (!defined $alt->{probability} || $alt->{probability} == 100) {
            # Assign calculated default probability
            $new_alt{probability} = $default_prob;
            
            # Distribute remainder to first few unspecified alternatives
            if ($extra_distributed < $extra) {
                $new_alt{probability}++;
                $extra_distributed++;
            }
        }
        
        push @normalized, \%new_alt;
    }
    
    # Verify final sum
    my $final_total = 0;
    for my $alt (@normalized) {
        $final_total += $alt->{probability};
    }
    
    if ($final_total != 100) {
        die "ERROR: Normalized probabilities sum to $final_total%, should be 100%\n";
    }
    
    print STDERR "INFO: Normalized probabilities to sum 100%\n" if $unspecified_count > 0;
    return @normalized;
}
