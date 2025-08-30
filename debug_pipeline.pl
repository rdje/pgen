#!/usr/bin/perl
use strict;
use warnings;
use FindBin qw($RealBin);
use lib "$RealBin/perl";
use JSON::PP;
use Data::Dumper;
use AST::Transform qw(step2_group_by_or step3_parse_sequences step4_handle_quantifiers step5_build_tree_structure);

# Read the JSON file
my $json_content = do {
    open my $fh, '<', 'json_grammar.json' or die "Cannot open: $!";
    local $/;
    <$fh>;
};

my $json = JSON::PP->new;
my $json_data = $json->decode($json_content);
my $raw_ast = $json_data->{raw_ast};

print "=== RAW AST ===\n";
print "Total rules: " . scalar(@$raw_ast) . "\n";

# Count value rules in raw AST
my $value_count = 0;
foreach my $rule_tokens (@$raw_ast) {
    my ($rule_name_token, @tokens) = @$rule_tokens;
    my $rule_name = ref($rule_name_token) eq 'ARRAY' ? $rule_name_token->[1] : $rule_name_token;
    if ($rule_name eq 'value') {
        $value_count++;
        print "Value rule $value_count: " . join(', ', map { ref($_) eq 'ARRAY' ? "[" . join(',', @$_) . "]" : $_ } @tokens) . "\n";
    }
}
print "Total 'value' rules in raw AST: $value_count\n\n";

print "=== STEP 2: GROUP BY OR ===\n";
my $step2_result = step2_group_by_or($raw_ast);
print "Total rules after step 2: " . scalar(@$step2_result) . "\n";

# Check value rule in step2 result
foreach my $rule (@$step2_result) {
    if ($rule->{name} eq 'value') {
        print "Value rule OR groups: " . scalar(@{$rule->{or_groups}}) . "\n";
        for my $i (0..$#{$rule->{or_groups}}) {
            my $group = $rule->{or_groups}->[$i];
            print "  Group $i: " . join(', ', map { ref($_) eq 'ARRAY' ? "[" . join(',', @$_) . "]" : $_ } @$group) . "\n";
        }
        last;
    }
}
print "\n";

print "=== STEP 3: PARSE SEQUENCES ===\n";
my $step3_result = step3_parse_sequences($step2_result);
print "Total rules after step 3: " . scalar(@$step3_result) . "\n";

# Check value rule in step3 result
foreach my $rule (@$step3_result) {
    if ($rule->{name} eq 'value') {
        print "Value rule after step 3:\n";
        print Dumper($rule);
        last;
    }
}
