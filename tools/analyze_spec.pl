#!/usr/bin/env perl
use strict;
use warnings;
use Data::Dumper;

# --- analyze_spec.pl ---
# A dedicated script to parse a LinkedSpec .spec file and dump its structure.
# GOAL: Perfect the grammar analysis logic before integrating it into the generator.

# Usage: perl analyze_spec.pl <path_to_spec_file>

die "Usage: perl analyze_spec.pl <spec_file>\n" unless @ARGV;
my $spec_file = shift @ARGV;

my $spec_content = do {
    local $/;
    open my $fh, '<', $spec_file or die "Cannot open spec file: $spec_file - $!\n";
    <$fh>;
};

my $analysis = analyze_grammar($spec_content);
print Dumper($analysis);

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

    # Split the content into rule sections. A rule starts with "rulename:" or "rulename::"
    # at the beginning of a line.
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






