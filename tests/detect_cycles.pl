#!/usr/bin/env perl
use strict;
use warnings;
use Data::Dumper;

# --- detect_cycles.pl ---
# Detect potential infinite loops in .spec files
# Usage: perl detect_cycles.pl <spec_file>

die "Usage: perl detect_cycles.pl <spec_file>\n" unless @ARGV;
my $spec_file = shift @ARGV;

my $spec_content = do {
    local $/;
    open my $fh, '<', $spec_file or die "Cannot open spec file: $spec_file - $!\n";
    <$fh>;
};

my $analysis = analyze_grammar($spec_content);
my $cycles = detect_cycles($analysis);

print "=== Cycle Detection Results for $spec_file ===\n";
if (@$cycles) {
    print "⚠️  POTENTIAL INFINITE LOOPS DETECTED:\n";
    foreach my $cycle (@$cycles) {
        print "  🔄 " . join(" -> ", @$cycle) . " -> " . $cycle->[0] . "\n";
    }
} else {
    print "✅ No obvious cycles detected.\n";
}

print "\n=== Self-Referencing Rules ===\n";
my $self_refs = find_self_references($analysis);
foreach my $rule (keys %$self_refs) {
    print "  🔁 $rule calls itself " . $self_refs->{$rule} . " time(s)\n";
}

sub analyze_grammar {
    my ($content) = @_;
    my %analysis = (
        rules => {},
        top_rule => undef,
    );

    # First, find the top-level rule (e.g., "Lispish::")
    if ($content =~ /^(\w+)::/m) {
        $analysis{top_rule} = $1;
    }

    # Split the content into rule sections
    my @rule_sections = split /(?=^[a-zA-Z_]\w*::?)/m, $content;

    foreach my $section (@rule_sections) {
        next unless $section =~ /^\s*(\w+)::?/m;
        my $rule_name = $1;

        my $rule_data = {
            name => $rule_name,
            sub_rules => [],
            is_container => 0,
        };

        # Find all sub-rules referenced in "->" action blocks within this section
        while ($section =~ /->\s*([a-zA-Z_]\w*)/g) {
            push @{$rule_data->{sub_rules}}, $1;
        }

        # A rule is a container if it has sub-rules.
        if (@{$rule_data->{sub_rules}}) {
            $rule_data->{is_container} = 1;
        }

        $analysis{rules}{$rule_name} = $rule_data;
    }

    return \%analysis;
}

sub detect_cycles {
    my ($analysis) = @_;
    my @cycles;
    my %visited;
    my %path;

    foreach my $rule_name (keys %{$analysis->{rules}}) {
        next if $visited{$rule_name};
        my $cycle = find_cycle_from($analysis, $rule_name, \%visited, \%path, []);
        push @cycles, $cycle if $cycle;
    }

    return \@cycles;
}

sub find_cycle_from {
    my ($analysis, $current_rule, $visited, $path, $current_path) = @_;
    
    return undef unless exists $analysis->{rules}{$current_rule};
    
    # If we've seen this rule in the current path, we found a cycle
    if ($path->{$current_rule}) {
        # Extract the cycle from current_path
        my $cycle_start = -1;
        for my $i (0..$#{$current_path}) {
            if ($current_path->[$i] eq $current_rule) {
                $cycle_start = $i;
                last;
            }
        }
        return [@{$current_path}[$cycle_start..$#{$current_path}]] if $cycle_start >= 0;
    }
    
    # Mark as visited in current path
    $path->{$current_rule} = 1;
    push @{$current_path}, $current_rule;
    
    # Check all sub-rules
    my $rule_data = $analysis->{rules}{$current_rule};
    foreach my $sub_rule (@{$rule_data->{sub_rules}}) {
        my $cycle = find_cycle_from($analysis, $sub_rule, $visited, $path, $current_path);
        if ($cycle) {
            # Clean up and return the cycle
            delete $path->{$current_rule};
            pop @{$current_path};
            return $cycle;
        }
    }
    
    # Mark as globally visited and clean up current path
    $visited->{$current_rule} = 1;
    delete $path->{$current_rule};
    pop @{$current_path};
    
    return undef;
}

sub find_self_references {
    my ($analysis) = @_;
    my %self_refs;
    
    foreach my $rule_name (keys %{$analysis->{rules}}) {
        my $rule_data = $analysis->{rules}{$rule_name};
        my $count = 0;
        foreach my $sub_rule (@{$rule_data->{sub_rules}}) {
            $count++ if $sub_rule eq $rule_name;
        }
        $self_refs{$rule_name} = $count if $count > 0;
    }
    
    return \%self_refs;
}





