#!/usr/bin/env perl
use strict;
use warnings;
use Data::Dumper;

# --- check_balanced_actions.pl ---
# Check for balanced action patterns in recursive .spec rules
# Usage: perl check_balanced_actions.pl <spec_file>

die "Usage: perl check_balanced_actions.pl <spec_file>\n" unless @ARGV;
my $spec_file = shift @ARGV;

my $spec_content = do {
    local $/;
    open my $fh, '<', $spec_file or die "Cannot open spec file: $spec_file - $!\n";
    <$fh>;
};

print "=== Balanced Action Analysis for $spec_file ===\n";
analyze_balanced_actions($spec_content);

sub analyze_balanced_actions {
    my ($content) = @_;
    
    # Find all rules with their patterns and actions
    my %rules;
    
    while ($content =~ /^(\w+):\s*(.+?)(?=^\w+:|$)/gms) {
        my ($rule_name, $rule_content) = ($1, $2);
        
        # Extract regex patterns
        my @patterns;
        while ($rule_content =~ /\/([^\/]+)\//g) {
            push @patterns, $1;
        }
        
        # Extract action blocks with their indices
        my @actions;
        # First, extract actions with multi-line code blocks
        while ($rule_content =~ /->\s*([a-zA-Z_]\w*)(?:\[(\d+)\])?\s*\{(.*?)\}/gs) {
            my ($target, $index, $code) = ($1, $2 // 0, $3);
            push @actions, {
                target => $target,
                index => $index,
                code => $code
            };
        }
        
        # Then extract simple actions without code blocks
        my $temp_content = $rule_content;
        $temp_content =~ s/->\s*[a-zA-Z_]\w*(?:\[\d+\])?\s*\{.*?\}//gs; # Remove actions with code blocks
        while ($temp_content =~ /->\s*([a-zA-Z_]\w*)(?:\[(\d+)\])?/g) {
            my ($target, $index) = ($1, $2 // 0);
            push @actions, {
                target => $target,
                index => $index,
                code => ''
            };
        }
        
        $rules{$rule_name} = {
            patterns => \@patterns,
            actions => \@actions
        };
    }
    
    # Check each rule for balanced action patterns
    foreach my $rule_name (keys %rules) {
        my $rule = $rules{$rule_name};
        my @patterns = @{$rule->{patterns}};
        my @actions = @{$rule->{actions}};
        
        print "\n--- Rule: $rule_name ---\n";
        print "Patterns: " . join(", ", map { "/$_/" } @patterns) . "\n";
        
        # Check if this rule has balanced delimiter patterns
        if (@patterns >= 2) {
            my ($open_pattern, $close_pattern) = ($patterns[0], $patterns[1]);
            
            # Common delimiter patterns
            my %delimiters = (
                '\\(' => '\\)',
                '\\[' => '\\]',
                '\\{' => '\\}',
                '"' => '"',
                "'" => "'",
            );
            
            my $has_balanced_delimiters = 0;
            foreach my $open (keys %delimiters) {
                my $close = $delimiters{$open};
                if ($open_pattern =~ /$open/ && $close_pattern =~ /$close/) {
                    $has_balanced_delimiters = 1;
                    print "✅ Balanced delimiters detected: $open <-> $close\n";
                    last;
                }
            }
            
            if ($has_balanced_delimiters) {
                # Check for balanced actions
                my %action_indices;
                my @self_actions;
                
                foreach my $action (@actions) {
                    if ($action->{target} eq $rule_name) {
                        push @self_actions, $action;
                        $action_indices{$action->{index}}++;
                    }
                }
                
                if (@self_actions) {
                    print "🔄 Self-referencing actions found:\n";
                    foreach my $action (@self_actions) {
                        my $code_summary = $action->{code} ? 
                            (length($action->{code}) > 50 ? substr($action->{code}, 0, 47) . "..." : $action->{code}) : 
                            "(no code)";
                        $code_summary =~ s/\s+/ /g;  # Clean up whitespace
                        print "   -> $action->{target}\[$action->{index}\] $code_summary\n";
                    }
                    
                    # Check if we have both [0] and [1] actions (balanced)
                    if (exists $action_indices{0} && exists $action_indices{1}) {
                        print "✅ BALANCED: Has both opening [0] and closing [1] actions\n";
                    } elsif (exists $action_indices{0} && !exists $action_indices{1}) {
                        print "⚠️  UNBALANCED: Has opening [0] but missing closing [1] action\n";
                    } elsif (!exists $action_indices{0} && exists $action_indices{1}) {
                        print "⚠️  UNBALANCED: Has closing [1] but missing opening [0] action\n";
                    } else {
                        print "ℹ️  Non-standard indexing pattern\n";
                    }
                } else {
                    print "ℹ️  No self-referencing actions (non-recursive)\n";
                }
            }
        } else {
            print "ℹ️  Not a balanced delimiter rule\n";
        }
        
        # Show all actions for this rule
        if (@actions) {
            print "Actions:\n";
            foreach my $action (@actions) {
                my $code_summary = $action->{code} ? 
                    (length($action->{code}) > 30 ? substr($action->{code}, 0, 27) . "..." : $action->{code}) : 
                    "(no code)";
                $code_summary =~ s/\s+/ /g;  # Clean up whitespace
                print "   -> $action->{target}\[$action->{index}\] $code_summary\n";
            }
        }
    }
}
