#!/usr/bin/env perl
use strict;
use warnings;

# --- check_infinite_loop_risk.pl ---
# Check for the specific infinite loop risk pattern:
# Rule with two patterns + self-recursion BUT missing closing action
# Usage: perl check_infinite_loop_risk.pl <spec_file>

die "Usage: perl check_infinite_loop_risk.pl <spec_file>\n" unless @ARGV;
my $spec_file = shift @ARGV;

my $spec_content = do {
    local $/;
    open my $fh, '<', $spec_file or die "Cannot open spec file: $spec_file - $!\n";
    <$fh>;
};

print "=== Infinite Loop Risk Analysis for $spec_file ===\n";
check_infinite_loop_risks($spec_content);

sub check_infinite_loop_risks {
    my ($content) = @_;
    
    # Parse each rule - need to be more careful about rule boundaries
    my @rule_sections = split /(?=^[a-zA-Z_]\w*::?)/m, $content;
    
    foreach my $section (@rule_sections) {
        next unless $section =~ /^\s*(\w+)::?\s/m;
        my $rule_name = $1;
        next if $rule_name =~ /^(TEST_MODE|EXPECT)$/; # Skip test directives
        
        # Extract the rule content (everything after the rule name line)
        my $rule_content = $section;
        
        # Extract patterns
        my @patterns;
        while ($rule_content =~ /\/([^\/]+)\//g) {
            push @patterns, $1;
        }
        
        # Skip rules that don't have exactly 2 patterns (open/close)
        next unless @patterns == 2;
        
        # Extract all actions for this rule
        my @actions;
        while ($rule_content =~ /->\s*([a-zA-Z_]\w*)(?:\[(\d+)\])?/g) {
            my ($target, $index) = ($1, $2 // 0);
            push @actions, { target => $target, index => $index };
        }
        
        # Check if this rule has self-recursion
        my $has_self_call_0 = 0;  # -> rule or -> rule[0]
        my $has_self_call_1 = 0;  # -> rule[1]
        
        foreach my $action (@actions) {
            if ($action->{target} eq $rule_name) {
                if ($action->{index} == 0) {
                    $has_self_call_0 = 1;
                } elsif ($action->{index} == 1) {
                    $has_self_call_1 = 1;
                }
            }
        }
        
        # Analyze the risk
        print "\n--- Rule: $rule_name ---\n";
        print "Patterns: /$patterns[0]/ /$patterns[1]/\n";
        
        if ($has_self_call_0 && $has_self_call_1) {
            print "✅ SAFE: Has both -> $rule_name (opens) and -> $rule_name\[1\] (closes)\n";
        } elsif ($has_self_call_0 && !$has_self_call_1) {
            print "🚨 INFINITE LOOP RISK: Has -> $rule_name but missing -> $rule_name\[1\]\n";
            print "   The parser will get stuck because:\n";
            print "   1. Matches opening pattern /$patterns[0]/\n";
            print "   2. Recursively calls -> $rule_name\n";
            print "   3. Has NO way to handle closing pattern /$patterns[1]/\n";
            print "   4. Gets stuck in infinite recursion!\n";
        } elsif (!$has_self_call_0 && $has_self_call_1) {
            print "⚠️  UNUSUAL: Has -> $rule_name\[1\] but no -> $rule_name\n";
        } elsif (!$has_self_call_0 && !$has_self_call_1) {
            print "ℹ️  NON-RECURSIVE: No self-calls (safe)\n";
        }
        
        # Show all self-referencing actions
        my @self_actions = grep { $_->{target} eq $rule_name } @actions;
        if (@self_actions) {
            print "Self-actions:\n";
            foreach my $action (@self_actions) {
                print "   -> $action->{target}\[$action->{index}\]\n";
            }
        }
    }
}
