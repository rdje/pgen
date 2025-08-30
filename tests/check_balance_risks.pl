#!/usr/bin/env perl
use strict;
use warnings;
use Data::Dumper;

# --- check_balance_risks.pl ---
# Check for potential unbalanced delimiter risks in .spec files
# Usage: perl check_balance_risks.pl <spec_file>

die "Usage: perl check_balance_risks.pl <spec_file>\n" unless @ARGV;
my $spec_file = shift @ARGV;

my $spec_content = do {
    local $/;
    open my $fh, '<', $spec_file or die "Cannot open spec file: $spec_file - $!\n";
    <$fh>;
};

print "=== Balance Risk Analysis for $spec_file ===\n";
analyze_balance_risks($spec_content);

sub analyze_balance_risks {
    my ($content) = @_;
    
    # Find all rules with their patterns
    my %rules;
    
    while ($content =~ /^(\w+):\s*(.+?)(?=^\w+:|$)/gms) {
        my ($rule_name, $rule_content) = ($1, $2);
        
        # Extract regex patterns
        my @patterns;
        while ($rule_content =~ /\/([^\/]+)\//g) {
            push @patterns, $1;
        }
        
        # Extract sub-rules
        my @sub_rules;
        while ($rule_content =~ /->\s*([a-zA-Z_]\w*)/g) {
            push @sub_rules, $1;
        }
        
        $rules{$rule_name} = {
            patterns => \@patterns,
            sub_rules => \@sub_rules
        };
    }
    
    # Check for balanced delimiter patterns
    foreach my $rule_name (keys %rules) {
        my $rule = $rules{$rule_name};
        my @patterns = @{$rule->{patterns}};
        my @sub_rules = @{$rule->{sub_rules}};
        
        # Check for opening/closing delimiter pairs
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
            
            my $is_balanced = 0;
            foreach my $open (keys %delimiters) {
                my $close = $delimiters{$open};
                if ($open_pattern =~ /$open/ && $close_pattern =~ /$close/) {
                    $is_balanced = 1;
                    print "✅ $rule_name: Balanced delimiters ($open_pattern <-> $close_pattern)\n";
                    
                    # Check if this rule can recurse on itself between delimiters
                    if (grep { $_ eq $rule_name } @sub_rules) {
                        print "   🔄 Can recurse on itself - this is SAFE (controlled by delimiters)\n";
                    }
                    last;
                }
            }
            
            if (!$is_balanced && @sub_rules > 0) {
                print "⚠️  $rule_name: Unbalanced patterns ($open_pattern, $close_pattern) with sub-rules\n";
                if (grep { $_ eq $rule_name } @sub_rules) {
                    print "   🚨 RISK: Can recurse without clear termination condition!\n";
                }
            }
        }
        elsif (@patterns == 1 && @sub_rules > 0) {
            print "ℹ️  $rule_name: Single pattern ($patterns[0]) with sub-rules\n";
            if (grep { $_ eq $rule_name } @sub_rules) {
                print "   ⚠️  POTENTIAL RISK: Self-recursion with single pattern - check termination logic\n";
            }
        }
        elsif (@patterns == 0 && @sub_rules > 0) {
            print "📝 $rule_name: Container rule (no patterns, only sub-rules)\n";
            if (grep { $_ eq $rule_name } @sub_rules) {
                print "   🚨 HIGH RISK: Self-recursion in pure container rule!\n";
            }
        }
    }
}





