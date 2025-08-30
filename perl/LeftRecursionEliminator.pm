package LeftRecursionEliminator;

# ================================================================================
# LEFT-RECURSION ELIMINATION - STANDARD ALGORITHM
# ================================================================================
# Implementation of the classic left-recursion elimination algorithm from:
# "Compilers: Principles, Techniques, and Tools" by Aho, Sethi, and Ullman
# 
# Algorithm 4.19: Eliminating left recursion from a grammar
# ================================================================================

use strict;
use warnings;
use Data::Dumper;

use Exporter 'import';
our @EXPORT_OK = qw(eliminate_all_left_recursion print_grammar);

# ================================================================================
# MAIN ELIMINATION FUNCTION
# ================================================================================

sub eliminate_all_left_recursion {
    my ($grammar) = @_;
    
    print STDERR "🔥 Applying standard left-recursion elimination algorithm\n";
    print STDERR "📚 Based on Aho-Sethi-Ullman Algorithm 4.19\n";
    print STDERR "📝 Input grammar has " . scalar(keys %$grammar) . " rules\n";
    
    # Step 1: Order the nonterminals (rules) arbitrarily as A1, A2, ..., An
    my @nonterminals = sort keys %$grammar;
    print STDERR "📋 Processing nonterminals in order: " . join(", ", @nonterminals) . "\n";
    
    my %working_grammar = %$grammar;
    
    # Step 2: for i = 1 to n do begin
    for my $i (0..$#nonterminals) {
        my $Ai = $nonterminals[$i];
        print STDERR "\n🎯 Processing $Ai (step " . ($i+1) . "/" . scalar(@nonterminals) . ")\n";
        
        # Step 2a: for j = 1 to i-1 do begin
        for my $j (0..$i-1) {
            my $Aj = $nonterminals[$j];
            
            # Check if substitution is actually needed to prevent left recursion
            # Only substitute if this could lead to indirect left recursion
            if (could_create_indirect_left_recursion(\%working_grammar, $Ai, $Aj)) {
                print STDERR "   🔄 Substituting $Aj in productions of $Ai (needed to break potential left recursion)\n";
                
                # Replace each production of the form Ai → Aj γ 
                # by the productions Ai → δ1 γ | δ2 γ | ... | δk γ
                # where Aj → δ1 | δ2 | ... | δk are all current Aj-productions
                
                my $ai_productions = $working_grammar{$Ai};
                my @new_ai_productions = ();
                
                foreach my $production (@$ai_productions) {
                    if (@$production > 0 && $production->[0] eq $Aj) {
                        # This production starts with Aj, substitute it
                        my @gamma = @$production[1..$#$production];  # Rest after Aj
                        my $aj_productions = $working_grammar{$Aj};
                        
                        # Replace Ai → Aj γ with Ai → δ γ for each Aj → δ
                        foreach my $aj_prod (@$aj_productions) {
                            # Skip epsilon productions to avoid infinite loops
                            if (@$aj_prod == 1 && $aj_prod->[0] eq 'ε') {
                                # Aj → ε, so Ai → Aj γ becomes Ai → γ
                                push @new_ai_productions, [@gamma];
                            } else {
                                # Aj → δ, so Ai → Aj γ becomes Ai → δ γ
                                my @new_production = (@$aj_prod, @gamma);
                                push @new_ai_productions, \@new_production;
                            }
                        }
                    } else {
                        # Keep production as-is
                        push @new_ai_productions, $production;
                    }
                }
                
                $working_grammar{$Ai} = \@new_ai_productions;
            } else {
                print STDERR "   ✅ Skipping substitution of $Aj in $Ai (not needed for left recursion elimination)\n";
            }
        }
        
        # Step 2b: eliminate the immediate left recursion among the Ai-productions
        print STDERR "   ⚡ Eliminating immediate left recursion in $Ai\n";
        eliminate_immediate_left_recursion(\%working_grammar, $Ai, \@nonterminals);
    }
    
    print STDERR "\n✅ Left-recursion elimination complete!\n";
    print STDERR "🎯 Final grammar has " . scalar(keys %working_grammar) . " rules\n";
    
    return \%working_grammar;
}

# ================================================================================
# IMMEDIATE LEFT-RECURSION ELIMINATION
# ================================================================================
# Algorithm 4.18: Eliminating immediate left recursion
# 
# If A → A α1 | A α2 | ... | A αm | β1 | β2 | ... | βn
# where no βi begins with A, then replace the A-productions by:
# A → β1 A' | β2 A' | ... | βn A'
# A' → α1 A' | α2 A' | ... | αm A' | ε

sub eliminate_immediate_left_recursion {
    my ($grammar_ref, $nonterminal, $nonterminals_ref) = @_;
    
    my $productions = $grammar_ref->{$nonterminal};
    
    # Separate left-recursive and non-left-recursive productions
    my @left_recursive = ();    # A → A αi
    my @non_left_recursive = (); # A → βi
    
    foreach my $production (@$productions) {
        if (@$production > 0 && $production->[0] eq $nonterminal) {
            # Direct left-recursion: A → A α
            push @left_recursive, $production;
        } else {
            # Non-left-recursive: A → β
            push @non_left_recursive, $production;
        }
    }
    
    if (@left_recursive > 0) {
        print STDERR "     🔥 Found " . scalar(@left_recursive) . " left-recursive productions\n";
        
        # Create new nonterminal A'
        my $prime_nonterminal = "${nonterminal}_prime";
        
        # Ensure the prime nonterminal name is unique
        my $counter = 1;
        while (exists $grammar_ref->{$prime_nonterminal} || 
               grep { $_ eq $prime_nonterminal } @$nonterminals_ref) {
            $prime_nonterminal = "${nonterminal}_prime_$counter";
            $counter++;
        }
        
        print STDERR "     ➕ Creating new nonterminal: $prime_nonterminal\n";
        
        # Create A-productions: A → β1 A' | β2 A' | ... | βn A'
        my @new_main_productions = ();
        if (@non_left_recursive > 0) {
            foreach my $beta (@non_left_recursive) {
                my @new_production = (@$beta, $prime_nonterminal);
                push @new_main_productions, \@new_production;
            }
        } else {
            # If there are no non-left-recursive productions, add A → A'
            push @new_main_productions, [$prime_nonterminal];
        }
        
        # Create A'-productions: A' → α1 A' | α2 A' | ... | αm A' | ε
        my @new_prime_productions = ();
        foreach my $left_prod (@left_recursive) {
            # Remove the left-recursive symbol (first element) to get αi
            my @alpha = @$left_prod[1..$#$left_prod];
            my @new_production = (@alpha, $prime_nonterminal);
            push @new_prime_productions, \@new_production;
        }
        # Add epsilon production: A' → ε
        push @new_prime_productions, ['ε'];
        
        # Update grammar
        $grammar_ref->{$nonterminal} = \@new_main_productions;
        $grammar_ref->{$prime_nonterminal} = \@new_prime_productions;
        
        # Add the new nonterminal to the list for further processing
        push @$nonterminals_ref, $prime_nonterminal;
        
        print STDERR "     ✅ Transformed $nonterminal into $nonterminal + $prime_nonterminal\n";
    } else {
        print STDERR "     ✅ No immediate left recursion found in $nonterminal\n";
    }
}

# ================================================================================
# LEFT-RECURSION DETECTION FUNCTIONS
# ================================================================================

sub could_create_indirect_left_recursion {
    my ($grammar_ref, $current_rule, $referenced_rule) = @_;
    
    # Check if substituting referenced_rule could lead to left recursion
    # This happens when:
    # 1. referenced_rule has productions that start with current_rule (creating a cycle)
    # 2. referenced_rule has productions that start with rules that eventually lead back to current_rule
    
    my $referenced_productions = $grammar_ref->{$referenced_rule};
    
    foreach my $prod (@$referenced_productions) {
        if (@$prod > 0) {
            my $first_symbol = $prod->[0];
            
            # Direct cycle: referenced_rule → current_rule ...
            if ($first_symbol eq $current_rule) {
                return 1;
            }
            
            # Check for indirect cycles through other nonterminals
            # For now, use a simple heuristic: if the first symbol is a nonterminal,
            # we might need substitution to avoid indirect left recursion
            if (exists $grammar_ref->{$first_symbol} && $first_symbol ne $referenced_rule) {
                # This could potentially create indirect left recursion
                # However, for most cases like "index_list → index ...", this is safe
                # Only substitute if we detect a real risk
                if (has_path_back_to_rule($grammar_ref, $first_symbol, $current_rule, {$referenced_rule => 1})) {
                    return 1;
                }
            }
        }
    }
    
    return 0;  # Safe to keep as rule reference
}

sub has_path_back_to_rule {
    my ($grammar_ref, $start_rule, $target_rule, $visited) = @_;
    
    # Avoid infinite recursion
    return 0 if $visited->{$start_rule};
    $visited->{$start_rule} = 1;
    
    my $productions = $grammar_ref->{$start_rule};
    return 0 unless $productions;
    
    foreach my $prod (@$productions) {
        if (@$prod > 0) {
            my $first_symbol = $prod->[0];
            
            # Direct match
            if ($first_symbol eq $target_rule) {
                return 1;
            }
            
            # Recursive check for indirect paths
            if (exists $grammar_ref->{$first_symbol} && !$visited->{$first_symbol}) {
                if (has_path_back_to_rule($grammar_ref, $first_symbol, $target_rule, {%$visited})) {
                    return 1;
                }
            }
        }
    }
    
    return 0;
}

# ================================================================================
# UTILITY FUNCTIONS
# ================================================================================

sub print_grammar {
    my ($grammar, $title) = @_;
    print STDERR "\n$title:\n";
    foreach my $rule (sort keys %$grammar) {
        my @prod_strings = map { 
            join(" ", map { $_ eq 'ε' ? 'ε' : $_ } @$_) 
        } @{$grammar->{$rule}};
        print STDERR "   $rule → " . join(" | ", @prod_strings) . "\n";
    }
    print STDERR "\n";
}

1;
