#!/usr/bin/env perl

use strict;
use warnings;
use Data::Dumper;
use Getopt::Long;

# Generic Test Input Generator for LinkedSpec
# Works with ANY .spec file by analyzing its grammar structure

my $VERBOSE = 0;
my $OUTPUT_FILE = '';
my $NUM_SAMPLES = 3;
# Depth and structure controls - reasonable defaults
my $MAX_DEPTH = 4;                    # 4 levels deep is usually sufficient for most grammars
my $MIN_ITEMS_PER_LEVEL = 1;          # At least 1 item per level
my $MAX_ITEMS_PER_LEVEL = 3;          # Max 3 items per level to avoid overwhelming output
my $NESTING_PROBABILITY = 0.4;        # 40% chance of nesting vs 60% terminal content

# Content type distribution controls - reasonable defaults (percentages 0-100)
my $PROB_NUMBER = 15;        # 15% chance of numbers (common in data structures)
my $PROB_DQUOTES = 25;       # 25% chance of double-quoted strings (most common)
my $PROB_SQUOTES = 10;       # 10% chance of single-quoted strings (less common)
my $PROB_IDENTIFIER = 35;    # 35% chance of identifiers (very common)
my $PROB_COMMENT = 5;        # 5% chance of comments (occasional)
my $PROB_SIMPLE_STRUCT = 10; # 10% chance of simple structures

GetOptions(
    'verbose|v' => \$VERBOSE,
    'output|o=s' => \$OUTPUT_FILE,
    'samples|n=i' => \$NUM_SAMPLES,
    'max-depth|d=i' => \$MAX_DEPTH,
    'min-items|min=i' => \$MIN_ITEMS_PER_LEVEL,
    'max-items|max=i' => \$MAX_ITEMS_PER_LEVEL,
    'nesting-prob|p=f' => \$NESTING_PROBABILITY,
    'prob-number|pn=f' => \$PROB_NUMBER,
    'prob-dquotes|pd=f' => \$PROB_DQUOTES,
    'prob-squotes|ps=f' => \$PROB_SQUOTES,
    'prob-identifier|pi=f' => \$PROB_IDENTIFIER,
    'prob-comment|pc=f' => \$PROB_COMMENT,
    'prob-simple|ps=f' => \$PROB_SIMPLE_STRUCT,
    'help|h' => sub { show_help(); exit 0; }
);

sub show_help {
    print <<EOF;
Generic LinkedSpec Test Input Generator

Usage: perl generate_test_input_generic.pl [options] <spec_file>

This generator works with ANY .spec file by analyzing its grammar structure
and generating appropriate test input based on the rules defined in that spec.

Options:
    -v, --verbose       Enable verbose output
    -o, --output FILE   Output file (default: stdout)
    -n, --samples N     Number of samples to generate (default: 3)
    -d, --max-depth N   Maximum nesting depth (default: 4)
    -min, --min-items N Minimum items per level (default: 1)
    -max, --max-items N Maximum items per level (default: 3)
    -p, --nesting-prob F Probability of nested vs terminal (default: 0.4)
    -pn, --prob-number N    Distribution of numbers (0-100, default: 15)
    -pd, --prob-dquotes N   Distribution of double-quoted strings (0-100, default: 25)
    -ps, --prob-squotes N   Distribution of single-quoted strings (0-100, default: 10)
    -pi, --prob-identifier N Distribution of identifiers (0-100, default: 35)
    -pc, --prob-comment N   Distribution of comments (0-100, default: 5)
    -ps, --prob-simple N    Distribution of simple structures (0-100, default: 10)
    
Note: Content type distributions should ideally sum to 100%. If they don't,
probabilities will be automatically normalized to create a proper distribution.
    -h, --help          Show this help

Examples:
    perl generate_test_input_generic.pl specs/valid/basic.spec
    perl generate_test_input_generic.pl specs/Lispish.spec
    perl generate_test_input_generic.pl -v -d 4 any_other.spec
    
    # Generate with more numbers and fewer strings
    perl generate_test_input_generic.pl -pn 40 -pd 20 specs/valid/basic.spec
    
    # Generate with high nesting probability
    perl generate_test_input_generic.pl -p 0.7 -d 5 specs/valid/basic.spec

EOF
}

die "Usage: perl generate_test_input_generic.pl [options] <spec_file>\n" unless @ARGV;
my $spec_file = shift @ARGV;

# Validate that content type probabilities are reasonable
my $total_prob = $PROB_NUMBER + $PROB_DQUOTES + $PROB_SQUOTES + $PROB_IDENTIFIER + $PROB_COMMENT + $PROB_SIMPLE_STRUCT;
if ($total_prob > 100) {
    warn "Warning: Content type probabilities sum to ${total_prob}% (over 100%). Probabilities will be normalized.\n";
} elsif ($total_prob < 100) {
    warn "Warning: Content type probabilities sum to ${total_prob}% (under 100%). Probabilities will be normalized.\n";
}

# Load and analyze the spec file
my $spec_content = load_spec_file($spec_file);
my $grammar_analysis = analyze_spec_grammar_generic($spec_content);

if ($VERBOSE) {
    print "=== Generic Spec Analysis ===\n";
    print Dumper($grammar_analysis);
    print "=== Generation Parameters ===\n";
    print "Max Depth: $MAX_DEPTH\n";
    print "Items per level: $MIN_ITEMS_PER_LEVEL to $MAX_ITEMS_PER_LEVEL\n";
    print "Nesting probability: $NESTING_PROBABILITY\n";
    print "Content probabilities:\n";
    print "  Numbers: $PROB_NUMBER\n";
    print "  Double quotes: $PROB_DQUOTES\n";
    print "  Single quotes: $PROB_SQUOTES\n";
    print "  Identifiers: $PROB_IDENTIFIER\n";
    print "  Comments: $PROB_COMMENT\n";
    print "  Simple structures: $PROB_SIMPLE_STRUCT\n";
}

# Generate test inputs based on the analyzed grammar
my @generated_inputs;
for (my $i = 1; $i <= $NUM_SAMPLES; $i++) {
    my $input = generate_input_from_grammar_analysis($grammar_analysis, $MAX_DEPTH);
    push @generated_inputs, $input;
    
    if ($VERBOSE) {
        print "=== Generated Input $i ===\n";
        print $input . "\n";
    }
}

# Output results
if ($OUTPUT_FILE) {
    open(my $fh, '>', $OUTPUT_FILE) or die "Cannot open output file: $!\n";
    print $fh join("\n---\n", @generated_inputs) . "\n";
    close($fh);
    print "Generated $NUM_SAMPLES test inputs to: $OUTPUT_FILE\n";
} else {
    print "=== Generated Test Inputs ===\n";
    print join("\n---\n", @generated_inputs) . "\n";
}

sub load_spec_file {
    my ($filename) = @_;
    open(my $fh, '<', $filename) or die "Cannot open spec file: $filename - $!\n";
    my $content = do { local $/; <$fh> };
    close($fh);
    return $content;
}

sub analyze_spec_grammar_generic {
    my ($content) = @_;
    my %analysis;
    
    # Extract top-level rule
    if ($content =~ /^(\w+)::\s*$/m) {
        $analysis{top_rule} = $1;
    }
    
    # Extract all rules and their patterns
    while ($content =~ /^(\w+):\s*(.+)$/gm) {
        my ($rule_name, $patterns) = ($1, $2);
        next if $rule_name eq $analysis{top_rule};  # Skip top rule
        
        $analysis{rules}{$rule_name} = {
            patterns => parse_regex_patterns_generic($patterns),
            actions => extract_actions_generic($content, $rule_name),
            dependencies => extract_dependencies_generic($content, $rule_name),
            structure_type => determine_structure_type($patterns, $content, $rule_name)
        };
    }
    
    # Extract top rule actions and structure
    if (exists $analysis{top_rule}) {
        my $top_rule = $analysis{top_rule};
        $analysis{rules}{$top_rule} = {
            patterns => [],
            actions => extract_actions_generic($content, $top_rule),
            dependencies => extract_dependencies_generic($content, $top_rule),
            structure_type => 'container'  # Top rules are typically containers
        };
    }
    
    # Analyze recursion patterns
    $analysis{recursion_patterns} = analyze_recursion_patterns($analysis{rules});
    
    # Determine content types supported
    $analysis{content_types} = determine_supported_content_types($analysis{rules});
    
    return \%analysis;
}

sub parse_regex_patterns_generic {
    my ($patterns) = @_;
    my @parsed_patterns;
    
    # Extract regex patterns like /pattern/
    while ($patterns =~ /\/([^\/]+)\//g) {
        push @parsed_patterns, $1;
    }
    
    return \@parsed_patterns;
}

sub extract_actions_generic {
    my ($content, $rule_name) = @_;
    my @actions;
    
    # Find action blocks for this rule
    my $rule_section = extract_rule_section_generic($content, $rule_name);
    
    # Extract -> action blocks
    while ($rule_section =~ /->\s+(\w+)(?:\[(\d+)\])?\s*\{([^}]+)\}/g) {
        push @actions, {
            target => $1,
            index => $2 || 0,
            code => $3
        };
    }
    
    return \@actions;
}

sub extract_rule_section_generic {
    my ($content, $rule_name) = @_;
    
    # Find the section starting with the rule definition
    if ($content =~ /^$rule_name:\s*(.+?)(?=^\w+:|$)/ms) {
        return $1;
    }
    
    # For top-level rules, look for the :: section
    if ($content =~ /^$rule_name::\s*(.+?)(?=^\w+:|$)/ms) {
        return $1;
    }
    
    return '';
}

sub extract_dependencies_generic {
    my ($content, $rule_name) = @_;
    my @deps;
    
    my $rule_section = extract_rule_section_generic($content, $rule_name);
    
    # Find call() dependencies
    while ($rule_section =~ /call\((\w+)\)/g) {
        push @deps, $1;
    }
    
    return \@deps;
}

sub determine_structure_type {
    my ($patterns, $content, $rule_name) = @_;
    
    # Analyze patterns to determine structure type
    my $pattern_str = $patterns;  # $patterns is already a string
    
    if ($pattern_str =~ /\\\\\(.*\\\\\)/) {
        return 'parentheses';  # Parentheses structure
    } elsif ($pattern_str =~ /\\\\\[.*\\\\\]/) {
        return 'brackets';     # Square bracket structure
    } elsif ($pattern_str =~ /\\\\\{.*\\\\\}/) {
        return 'braces';       # Curly brace structure
    } elsif ($pattern_str =~ /".*"/) {
        return 'quoted';       # Quoted string structure
    } elsif ($pattern_str =~ /\\\\s\+/) {
        return 'whitespace';   # Whitespace structure
    } elsif ($pattern_str =~ /[^\\\\s"\{\}\\(\\)\\[\\];]+/) {
        return 'identifier';   # Identifier structure
    } else {
        return 'unknown';
    }
}

sub analyze_recursion_patterns {
    my ($rules) = @_;
    my %recursion;
    
    foreach my $rule_name (keys %$rules) {
        my $rule = $rules->{$rule_name};
        my @recursive_calls;
        
        # Check if this rule calls itself (direct recursion)
        foreach my $action (@{$rule->{actions}}) {
            if ($action->{target} eq $rule_name) {
                push @recursive_calls, { type => 'direct', target => $rule_name };
            }
        }
        
        # Check for indirect recursion through dependencies
        foreach my $dep (@{$rule->{dependencies}}) {
            if (exists $rules->{$dep}) {
                # Check if the dependency can lead back to this rule
                my $dep_rule = $rules->{$dep};
                foreach my $dep_action (@{$dep_rule->{actions}}) {
                    if ($dep_action->{target} eq $rule_name) {
                        push @recursive_calls, { type => 'indirect', target => $dep };
                    }
                }
            }
        }
        
        $recursion{$rule_name} = \@recursive_calls if @recursive_calls;
    }
    
    return \%recursion;
}

sub determine_supported_content_types {
    my ($rules) = @_;
    my %content_types;
    
    foreach my $rule_name (keys %$rules) {
        my $rule = $rules->{$rule_name};
        my $structure_type = $rule->{structure_type};
        
        $content_types{$structure_type} = 1;
        
        # Analyze patterns for specific content types
        foreach my $pattern (@{$rule->{patterns}}) {
            if ($pattern =~ /".*"/) {
                $content_types{quoted_strings} = 1;
            }
            if ($pattern =~ /'.*'/) {
                $content_types{single_quoted} = 1;
            }
            if ($pattern =~ /\\\\d\+/) {
                $content_types{numbers} = 1;
            }
            if ($pattern =~ /\\\\s\+/) {
                $content_types{whitespace} = 1;
            }
            if ($pattern =~ /[^\\\\s"\{\}\\(\\)\\[\\];]+/) {
                $content_types{identifiers} = 1;
            }
        }
    }
    
    return \%content_types;
}

sub generate_input_from_grammar_analysis {
    my ($analysis, $max_depth, $current_depth) = @_;
    $current_depth ||= 0;
    
    return '' if $current_depth >= $max_depth;
    
    my $top_rule = $analysis->{top_rule};
    my $top_rule_data = $analysis->{rules}{$top_rule};
    
    # Generate based on the top rule's structure type
    return generate_structure_based_input($analysis, $top_rule, $current_depth, $max_depth);
}

sub generate_structure_based_input {
    my ($analysis, $rule_name, $depth, $max_depth) = @_;
    
    my $rule_data = $analysis->{rules}{$rule_name};
    my $structure_type = $rule_data->{structure_type};
    
    if ($structure_type eq 'parentheses') {
        return generate_parentheses_structure_generic($analysis, $rule_name, $depth, $max_depth);
    } elsif ($structure_type eq 'brackets') {
        return generate_bracket_structure_generic($analysis, $rule_name, $depth, $max_depth);
    } elsif ($structure_type eq 'braces') {
        return generate_brace_structure_generic($analysis, $rule_name, $depth, $max_depth);
    } elsif ($structure_type eq 'quoted') {
        return generate_quoted_content_generic($analysis, $rule_name);
    } elsif ($structure_type eq 'identifier') {
        return generate_identifier_content_generic($analysis, $rule_name);
    } elsif ($structure_type eq 'whitespace') {
        return generate_whitespace_content_generic();
    } else {
        # Default to container structure
        return generate_container_structure_generic($analysis, $rule_name, $depth, $max_depth);
    }
}

sub generate_parentheses_structure_generic {
    my ($analysis, $rule_name, $depth, $max_depth) = @_;
    
    my $output = '(';
    $output .= generate_random_identifier();
    $output .= "\n";
    
    # Determine number of items
    my $num_items = int(rand($MAX_ITEMS_PER_LEVEL - $MIN_ITEMS_PER_LEVEL + 1)) + $MIN_ITEMS_PER_LEVEL;
    
    for (my $i = 0; $i < $num_items; $i++) {
        $output .= ' ';  # Indent
        
        # Decide whether to nest or use terminal content
        if ($depth < $max_depth - 1 && rand() < $NESTING_PROBABILITY) {
            # Check for recursion patterns
            my $recursion = $analysis->{recursion_patterns}{$rule_name};
            if ($recursion && rand() < 0.3) {  # 30% chance of recursion
                $output .= generate_structure_based_input($analysis, $rule_name, $depth + 1, $max_depth);
            } else {
                # Generate nested content based on dependencies
                my $deps = $analysis->{rules}{$rule_name}{dependencies};
                if (@$deps) {
                    my $random_dep = $deps->[int(rand(@$deps))];
                    $output .= generate_structure_based_input($analysis, $random_dep, $depth + 1, $max_depth);
                } else {
                    $output .= generate_terminal_content_generic($analysis, $depth);
                }
            }
        } else {
            $output .= generate_terminal_content_generic($analysis, $depth);
        }
        
        $output .= "\n" if $i < $num_items - 1;
    }
    
    $output .= ')';
    return $output;
}

sub generate_bracket_structure_generic {
    my ($analysis, $rule_name, $depth, $max_depth) = @_;
    
    my $output = '[';
    $output .= generate_random_identifier();
    $output .= "\n";
    
    my $num_items = int(rand($MAX_ITEMS_PER_LEVEL - $MIN_ITEMS_PER_LEVEL + 1)) + $MIN_ITEMS_PER_LEVEL;
    
    for (my $i = 0; $i < $num_items; $i++) {
        $output .= '  ';
        
        if ($depth < $max_depth - 1 && rand() < $NESTING_PROBABILITY) {
            my $deps = $analysis->{rules}{$rule_name}{dependencies};
            if (@$deps) {
                my $random_dep = $deps->[int(rand(@$deps))];
                $output .= generate_structure_based_input($analysis, $random_dep, $depth + 1, $max_depth);
            } else {
                $output .= generate_terminal_content_generic($analysis, $depth);
            }
        } else {
            $output .= generate_terminal_content_generic($analysis, $depth);
        }
        
        $output .= "\n" if $i < $num_items - 1;
    }
    
    $output .= ' ]';
    return $output;
}

sub generate_brace_structure_generic {
    my ($analysis, $rule_name, $depth, $max_depth) = @_;
    
    my $output = '{';
    $output .= generate_random_identifier();
    $output .= "\n";
    
    my $num_items = int(rand($MAX_ITEMS_PER_LEVEL - $MIN_ITEMS_PER_LEVEL + 1)) + $MIN_ITEMS_PER_LEVEL;
    
    for (my $i = 0; $i < $num_items; $i++) {
        $output .= '  ';
        
        if ($depth < $max_depth - 1 && rand() < $NESTING_PROBABILITY) {
            my $deps = $analysis->{rules}{$rule_name}{dependencies};
            if (@$deps) {
                my $random_dep = $deps->[int(rand(@$deps))];
                $output .= generate_structure_based_input($analysis, $random_dep, $depth + 1, $max_depth);
            } else {
                $output .= generate_terminal_content_generic($analysis, $depth);
            }
        } else {
            $output .= generate_terminal_content_generic($analysis, $depth);
        }
        
        $output .= "\n" if $i < $num_items - 1;
    }
    
    $output .= ' }';
    return $output;
}

sub generate_container_structure_generic {
    my ($analysis, $rule_name, $depth, $max_depth) = @_;
    
    # Default container structure
    my $output = '(';
    $output .= generate_random_identifier();
    $output .= "\n";
    
    my $num_items = int(rand($MAX_ITEMS_PER_LEVEL - $MIN_ITEMS_PER_LEVEL + 1)) + $MIN_ITEMS_PER_LEVEL;
    
    for (my $i = 0; $i < $num_items; $i++) {
        $output .= ' ';
        
        if ($depth < $max_depth - 1 && rand() < $NESTING_PROBABILITY) {
            my $deps = $analysis->{rules}{$rule_name}{dependencies};
            if (@$deps) {
                my $random_dep = $deps->[int(rand(@$deps))];
                $output .= generate_structure_based_input($analysis, $random_dep, $depth + 1, $max_depth);
            } else {
                $output .= generate_terminal_content_generic($analysis, $depth);
            }
        } else {
            $output .= generate_terminal_content_generic($analysis, $depth);
        }
        
        $output .= "\n" if $i < $num_items - 1;
    }
    
    $output .= ')';
    return $output;
}

sub generate_terminal_content_generic {
    my ($analysis, $depth) = @_;
    
    # Check what content types are supported by this grammar
    my $content_types = $analysis->{content_types};
    
    # Build distribution based on supported types and configured probabilities
    my @distribution;
    my $total_prob = 0;
    
    # Only include supported types in the distribution
    if (exists $content_types->{numbers}) {
        push @distribution, { type => 'number', prob => $PROB_NUMBER };
        $total_prob += $PROB_NUMBER;
    }
    if (exists $content_types->{quoted_strings}) {
        push @distribution, { type => 'dquotes', prob => $PROB_DQUOTES };
        $total_prob += $PROB_DQUOTES;
    }
    if (exists $content_types->{single_quoted}) {
        push @distribution, { type => 'squotes', prob => $PROB_SQUOTES };
        $total_prob += $PROB_SQUOTES;
    }
    if (exists $content_types->{identifiers}) {
        push @distribution, { type => 'identifier', prob => $PROB_IDENTIFIER };
        $total_prob += $PROB_IDENTIFIER;
    }
    if (exists $content_types->{whitespace}) {
        push @distribution, { type => 'comment', prob => $PROB_COMMENT };
        $total_prob += $PROB_COMMENT;
    }
    push @distribution, { type => 'simple', prob => $PROB_SIMPLE_STRUCT };
    $total_prob += $PROB_SIMPLE_STRUCT;
    
    # If no supported types or total is 0, fallback to identifier
    if (@distribution == 0 || $total_prob == 0) {
        return generate_random_identifier();
    }
    
    # Normalize probabilities to sum to 100 if they don't already
    my $normalization_factor = 100.0 / $total_prob;
    my $cumulative = 0;
    my $rand_val = rand(100);
    
    foreach my $item (@distribution) {
        my $normalized_prob = $item->{prob} * $normalization_factor;
        if ($rand_val <= ($cumulative + $normalized_prob)) {
            # Return the appropriate content type
            if ($item->{type} eq 'number') {
                return generate_random_number();
            } elsif ($item->{type} eq 'dquotes') {
                return '"' . generate_random_string(5, 15) . '"';
            } elsif ($item->{type} eq 'squotes') {
                return "'" . generate_random_string(5, 15) . "'";
            } elsif ($item->{type} eq 'identifier') {
                return generate_random_identifier();
            } elsif ($item->{type} eq 'comment') {
                return '; ' . generate_random_comment();
            } elsif ($item->{type} eq 'simple') {
                return '(' . generate_random_identifier() . ' ' . generate_random_identifier() . ')';
            }
        }
        $cumulative += $normalized_prob;
    }
    
    # Fallback to identifier if no specific type matched
    return generate_random_identifier();
}

sub generate_quoted_content_generic {
    my ($analysis, $rule_name) = @_;
    return '"' . generate_random_string(5, 15) . '"';
}

sub generate_identifier_content_generic {
    my ($analysis, $rule_name) = @_;
    return generate_random_identifier();
}

sub generate_whitespace_content_generic {
    return ' ' x int(rand(3) + 1);
}

sub generate_random_string {
    my ($min_len, $max_len) = @_;
    my $len = int(rand($max_len - $min_len + 1)) + $min_len;
    my @chars = ('a'..'z', 'A'..'Z', '0'..'9');
    return join('', map { $chars[rand(@chars)] } 1..$len);
}

sub generate_random_identifier {
    my @identifiers = qw(test_data simple_value nested_data inner_value hello world example sample data value item config setting user profile account);
    return $identifiers[rand(@identifiers)];
}

sub generate_random_number {
    return int(rand(1000)) + 1;
}

# Example usage and testing
if ($VERBOSE) {
    print "=== Generic Test Input Generator ===\n";
    print "Spec file: $spec_file\n";
    print "Samples: $NUM_SAMPLES\n";
    print "Max depth: $MAX_DEPTH\n";
    print "Items per level: $MIN_ITEMS_PER_LEVEL to $MAX_ITEMS_PER_LEVEL\n";
    print "Nesting probability: $NESTING_PROBABILITY\n";
    print "Output: " . ($OUTPUT_FILE || "stdout") . "\n";
}
