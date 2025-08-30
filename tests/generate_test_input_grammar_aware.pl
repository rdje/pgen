#!/usr/bin/env perl

use strict;
use warnings;
use Data::Dumper;
use Getopt::Long;

# Grammar-Aware Test Input Generator for LinkedSpec
# Reads the actual .spec file to extract sub-rule relationships

my $VERBOSE = 0;
my $OUTPUT_FILE = '';
my $NUM_SAMPLES = 3;

# Structure controls
my $MAX_DEPTH = 4;
my $MIN_ITEMS_PER_LEVEL = 1;
my $MAX_ITEMS_PER_LEVEL = 3;
my $NESTING_PROBABILITY = 0.4;

# Structural decision percentages (what type of rule to inject)
my $PROB_TERMINAL = 60;      # 60% chance of terminal rule vs container
my $PROB_CONTAINER = 30;     # 30% chance of container rule
my $PROB_RECURSIVE = 10;     # 10% chance of recursive rule

# Level-specific structural controls
my %LEVEL_STRUCTURAL_CONTROLS = (
    1 => { prob_terminal => 40, prob_container => 50, prob_recursive => 10 },
    2 => { prob_terminal => 50, prob_container => 35, prob_recursive => 15 },
    3 => { prob_terminal => 60, prob_container => 25, prob_recursive => 15 },
    4 => { prob_terminal => 80, prob_container => 15, prob_recursive => 5 },
    5 => { prob_terminal => 95, prob_container => 5, prob_recursive => 0 }
);

GetOptions(
    'verbose|v' => \$VERBOSE,
    'output|o=s' => \$OUTPUT_FILE,
    'samples|n=i' => \$NUM_SAMPLES,
    'max-depth|d=i' => \$MAX_DEPTH,
    'min-items|min=i' => \$MIN_ITEMS_PER_LEVEL,
    'max-items|max=i' => \$MAX_ITEMS_PER_LEVEL,
    'nesting-prob|p=f' => \$NESTING_PROBABILITY,
    'prob-terminal|pt=i' => \$PROB_TERMINAL,
    'prob-container|pc=i' => \$PROB_CONTAINER,
    'prob-recursive|pr=i' => \$PROB_RECURSIVE,
    'help|h' => sub { show_help(); exit 0; }
);

sub show_help {
    print <<EOF;
Grammar-Aware LinkedSpec Test Input Generator

Usage: perl generate_test_input_grammar_aware.pl [options] <spec_file>

This generator analyzes the grammar structure by READING THE SPEC FILE
and extracting actual sub-rule relationships from -> action blocks.

Structure Controls:
    -d, --max-depth N     Maximum nesting depth (default: 4)
    -min, --min-items N   Minimum items per level (default: 1)
    -max, --max-items N   Maximum items per level (default: 3)
    -p, --nesting-prob F  Probability of nested vs terminal (default: 0.4)

Structural Decision Controls (percentages 0-100):
    -pt, --prob-terminal N   Probability of terminal rules (default: 60)
    -pc, --prob-container N  Probability of container rules (default: 30)
    -pr, --prob-recursive N  Probability of recursive rules (default: 10)

Examples:
    perl generate_test_input_grammar_aware.pl specs/valid/basic.spec
    perl generate_test_input_grammar_aware.pl -pt 40 -pc 50 -pr 10 fx/specs/Lispish.spec
    perl generate_test_input_grammar_aware.pl -v -d 5 -pt 30 -pc 60 -pr 10 any_other.spec

EOF
}

die "Usage: perl generate_test_input_grammar_aware.pl [options] <spec_file>\n" unless @ARGV;
my $spec_file = shift @ARGV;

# Validate structural probabilities
my $total_structural = $PROB_TERMINAL + $PROB_CONTAINER + $PROB_RECURSIVE;
if ($total_structural != 100) {
    warn "Warning: Structural probabilities sum to ${total_structural}% (should be 100%). Normalizing...\n";
    my $factor = 100.0 / $total_structural;
    $PROB_TERMINAL = int($PROB_TERMINAL * $factor);
    $PROB_CONTAINER = int($PROB_CONTAINER * $factor);
    $PROB_RECURSIVE = int($PROB_RECURSIVE * $factor);
}

# Load and analyze the spec file
my $spec_content = load_spec_file($spec_file);
my $grammar_analysis = analyze_grammar_structure($spec_content);

if ($VERBOSE) {
    print "=== Grammar Structure Analysis ===\n";
    print Dumper($grammar_analysis);
    print "=== Generation Parameters ===\n";
    print "Max Depth: $MAX_DEPTH\n";
    print "Items per level: $MIN_ITEMS_PER_LEVEL to $MAX_ITEMS_PER_LEVEL\n";
    print "Nesting probability: $NESTING_PROBABILITY\n";
    print "Structural probabilities:\n";
    print "  Terminal rules: $PROB_TERMINAL%\n";
    print "  Container rules: $PROB_CONTAINER%\n";
    print "  Recursive rules: $PROB_RECURSIVE%\n";
}

# Generate test inputs based on grammar analysis
my @generated_inputs;
for (my $i = 1; $i <= $NUM_SAMPLES; $i++) {
    my $input = generate_grammar_aware_input($grammar_analysis, $MAX_DEPTH);
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

sub analyze_grammar_structure {
    my ($content) = @_;
    my %analysis;
    
    # Extract top-level rule
    if ($content =~ /^(\w+)::\s*$/m) {
        $analysis{top_rule} = $1;
    }
    
    # Extract all rules and their sub-rule relationships
    while ($content =~ /^(\w+):\s*(.+?)(?=^\w+:|$)/gms) {
        my ($rule_name, $rule_content) = ($1, $2);
        next if $rule_name eq $analysis{top_rule};  # Skip top rule
        
        $analysis{rules}{$rule_name} = {
            patterns => parse_regex_patterns($rule_content),
            sub_rules => extract_sub_rules($rule_content),
            actions => extract_actions($rule_content),
            rule_type => classify_rule_type($rule_content, $rule_name),
            structure_type => determine_structure_type($rule_content)
        };
    }
    
    # Also extract rules that use :: format (like top_expression::)
    while ($content =~ /^(\w+)::\s*(.+?)(?=^\w+:|$)/gms) {
        my ($rule_name, $rule_content) = ($1, $2);
        next if $rule_name eq $analysis{top_rule};  # Skip top rule
        
        $analysis{rules}{$rule_name} = {
            patterns => parse_regex_patterns($rule_content),
            sub_rules => extract_sub_rules($rule_content),
            actions => extract_actions($rule_content),
            rule_type => classify_rule_type($rule_content, $rule_name),
            structure_type => determine_structure_type($rule_content)
        };
    }
    
    # Extract top rule
    if (exists $analysis{top_rule}) {
        my $top_rule = $analysis{top_rule};
        my $top_rule_content = extract_top_rule_content($content, $top_rule);
        
        if ($VERBOSE) {
            print "Top rule content for $top_rule:\n'$top_rule_content'\n";
        }
        
        $analysis{rules}{$top_rule} = {
            patterns => [],
            sub_rules => extract_sub_rules($top_rule_content),
            actions => extract_actions($top_rule_content),
            rule_type => 'container',  # Top rules are always containers
            structure_type => 'container'
        };
    }
    
    # Analyze recursion patterns
    $analysis{recursion_patterns} = analyze_recursion_patterns($analysis{rules});
    
    # Categorize rules by type based on actual sub-rule relationships
    categorize_rules_by_type($analysis{rules});
    
    return \%analysis;
}

sub extract_top_rule_content {
    my ($content, $top_rule) = @_;
    
    # Find the section starting with the top rule definition
    # Look for everything after Lispish:: until the next rule definition
    if ($content =~ /^$top_rule::\s*\n(.+?)(?=^\w+:\s*\/)/ms) {
        return $1;
    }
    
    # Alternative: find everything after the top rule definition until the next rule
    if ($content =~ /^$top_rule::\s*\n(.+?)(?=^\w+:|$)/ms) {
        return $1;
    }
    
    # If still not found, try a simpler approach
    if ($content =~ /^$top_rule::\s*\n(.+)/ms) {
        return $1;
    }
    
    return '';
}

sub parse_regex_patterns {
    my ($rule_content) = @_;
    my @parsed_patterns;
    
    # Extract regex patterns like /pattern/
    while ($rule_content =~ /\/([^\/]+)\//g) {
        push @parsed_patterns, $1;
    }
    
    return \@parsed_patterns;
}

sub extract_sub_rules {
    my ($rule_content) = @_;
    my @sub_rules;
    
    # Extract -> action blocks to find sub-rules
    while ($rule_content =~ /->\s+(\w+)(?:\[(\d+)\])?\s*\{([^}]+)\}/g) {
        my ($target_rule, $index, $code) = ($1, $2 || 0, $3);
        
        # Skip if it's a self-reference (recursion)
        next if $target_rule eq 'parenthesis' && $index == 1;  # Closing parenthesis
        
        push @sub_rules, {
            target => $target_rule,
            index => $index,
            code => $code
        };
    }
    
    # Also extract simple -> calls without code blocks
    while ($rule_content =~ /->\s+(\w+)(?:\[(\d+)\])?\s*$/gm) {
        my ($target_rule, $index) = ($1, $2 || 0);
        
        # Skip if it's a self-reference (recursion)
        next if $target_rule eq 'parenthesis' && $index == 1;  # Closing parenthesis
        
        push @sub_rules, {
            target => $target_rule,
            index => $index,
            code => ''
        };
    }
    
    # Debug: print what we found
    if ($VERBOSE && @sub_rules > 0) {
        print "Found sub-rules: " . join(', ', map { $_->{target} } @sub_rules) . "\n";
    }
    
    return \@sub_rules;
}

sub extract_actions {
    my ($rule_content) = @_;
    my @actions;
    
    # Extract -> action blocks
    while ($rule_content =~ /->\s+(\w+)(?:\[(\d+)\])?\s*\{([^}]+)\}/g) {
        push @actions, {
            target => $1,
            index => $2 || 0,
            code => $3
        };
    }
    
    return \@actions;
}

sub classify_rule_type {
    my ($rule_content, $rule_name) = @_;
    
    # Check if this rule has -> action blocks (container rule)
    if ($rule_content =~ /->\s+\w+/) {
        return 'container';  # Has action blocks, can contain other rules
    }
    
    # Check if it has regex patterns but no action blocks (terminal rule)
    if ($rule_content =~ /\/([^\/]+)\//) {
        return 'terminal';  # Has patterns but no sub-rules
    }
    
    return 'unknown';
}

sub determine_structure_type {
    my ($rule_content) = @_;
    
    # Check for structural patterns
    if ($rule_content =~ /\/\\\(/ || $rule_content =~ /\/\\\)/) {
        return 'parentheses';
    } elsif ($rule_content =~ /\/\\\[/ || $rule_content =~ /\/\\\]/) {
        return 'brackets';
    } elsif ($rule_content =~ /\/\\\{/ || $rule_content =~ /\/\\\}/) {
        return 'braces';
    } elsif ($rule_content =~ /\/".*"/) {
        return 'quoted';
    } elsif ($rule_content =~ /\/\\s\+/) {
        return 'whitespace';
    } elsif ($rule_content =~ /\/[^\\s"\{\}\\(\\)\\[\\];]+/) {
        return 'identifier';
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
        foreach my $sub_rule (@{$rule->{sub_rules}}) {
            if ($sub_rule->{target} eq $rule_name) {
                push @recursive_calls, { type => 'direct', target => $rule_name };
            }
        }
        
        $recursion{$rule_name} = \@recursive_calls if @recursive_calls;
    }
    
    return \%recursion;
}

sub categorize_rules_by_type {
    my ($rules) = @_;
    
    my %terminal_rules;
    my %container_rules;
    my %recursive_rules;
    
    foreach my $rule_name (keys %$rules) {
        my $rule = $rules->{$rule_name};
        my $rule_type = $rule->{rule_type};
        
        if ($rule_type eq 'terminal') {
            $terminal_rules{$rule_name} = $rule;
        } elsif ($rule_type eq 'container') {
            $container_rules{$rule_name} = $rule;
        }
        
        # Check if recursive
        if (exists $rule->{recursive} && $rule->{recursive}) {
            $recursive_rules{$rule_name} = $rule;
        }
    }
    
    $rules->{_terminal_rules} = \%terminal_rules;
    $rules->{_container_rules} = \%container_rules;
    $rules->{_recursive_rules} = \%recursive_rules;
}

sub generate_grammar_aware_input {
    my ($analysis, $max_depth, $current_depth) = @_;
    $current_depth ||= 0;
    
    return '' if $current_depth >= $max_depth;
    
    my $top_rule = $analysis->{top_rule};
    return generate_rule_content($analysis, $top_rule, $current_depth, $max_depth);
}

sub generate_rule_content {
    my ($analysis, $rule_name, $depth, $max_depth) = @_;
    
    my $rule_data = $analysis->{rules}{$rule_name};
    my $rule_type = $rule_data->{rule_type};
    my $sub_rules = $rule_data->{sub_rules};
    
    # Get level-specific controls
    my $level = $depth + 1;
    my $controls = $LEVEL_STRUCTURAL_CONTROLS{$level} || $LEVEL_STRUCTURAL_CONTROLS{5};
    
    # Decide whether to nest or use terminal content
    if ($depth < $max_depth - 1 && rand() < $NESTING_PROBABILITY) {
        # Choose what type of rule to inject
        my $rand_val = rand(100);
        my $cumulative = 0;
        
        # Check for recursive rules first
        my $recursion = $analysis->{recursion_patterns}{$rule_name};
        if ($recursion && $rand_val <= ($cumulative += $controls->{prob_recursive})) {
            return generate_recursive_content($analysis, $rule_name, $depth, $max_depth);
        }
        
        # Check for container rules (rules with sub-rules)
        if (@$sub_rules > 0 && $rand_val <= ($cumulative += $controls->{prob_container})) {
            return generate_container_content($analysis, $rule_name, $sub_rules, $depth, $max_depth);
        }
        
        # Fall back to terminal content
        return generate_terminal_content($analysis, $depth);
    } else {
        # Use terminal content
        return generate_terminal_content($analysis, $depth);
    }
}

sub generate_recursive_content {
    my ($analysis, $rule_name, $depth, $max_depth) = @_;
    
    # Generate content using the same rule (recursion)
    my $rule_data = $analysis->{rules}{$rule_name};
    my $structure_type = $rule_data->{structure_type};
    
    if ($structure_type eq 'parentheses') {
        return generate_parentheses_structure($analysis, $rule_name, $depth, $max_depth);
    } elsif ($structure_type eq 'brackets') {
        return generate_bracket_structure($analysis, $rule_name, $depth, $max_depth);
    } elsif ($structure_type eq 'braces') {
        return generate_brace_structure($analysis, $rule_name, $depth, $max_depth);
    } else {
        # Default recursive structure
        return generate_container_structure($analysis, $rule_name, $depth, $max_depth);
    }
}

sub generate_container_content {
    my ($analysis, $rule_name, $sub_rules, $depth, $max_depth) = @_;
    
    # Choose a random sub-rule from the available ones
    if (@$sub_rules == 0) {
        return generate_terminal_content($analysis, $depth);
    }
    
    my $chosen_sub_rule = $sub_rules->[int(rand(@$sub_rules))];
    my $target_rule = $chosen_sub_rule->{target};
    
    # Check if the target rule exists
    if (exists $analysis->{rules}{$target_rule}) {
        return generate_rule_content($analysis, $target_rule, $depth + 1, $max_depth);
    } else {
        return generate_terminal_content($analysis, $depth);
    }
}

sub generate_parentheses_structure {
    my ($analysis, $rule_name, $depth, $max_depth) = @_;
    
    my $output = '(';
    $output .= generate_random_identifier();
    $output .= "\n";
    
    my $num_items = int(rand($MAX_ITEMS_PER_LEVEL - $MIN_ITEMS_PER_LEVEL + 1)) + $MIN_ITEMS_PER_LEVEL;
    
    for (my $i = 0; $i < $num_items; $i++) {
        $output .= ' ';
        
        if ($depth < $max_depth - 1 && rand() < $NESTING_PROBABILITY) {
            $output .= generate_rule_content($analysis, $rule_name, $depth + 1, $max_depth);
        } else {
            $output .= generate_terminal_content($analysis, $depth);
        }
        
        $output .= "\n" if $i < $num_items - 1;
    }
    
    $output .= ')';
    return $output;
}

sub generate_bracket_structure {
    my ($analysis, $rule_name, $depth, $max_depth) = @_;
    
    my $output = '[';
    $output .= generate_random_identifier();
    $output .= "\n";
    
    my $num_items = int(rand($MAX_ITEMS_PER_LEVEL - $MIN_ITEMS_PER_LEVEL + 1)) + $MIN_ITEMS_PER_LEVEL;
    
    for (my $i = 0; $i < $num_items; $i++) {
        $output .= '  ';
        
        if ($depth < $max_depth - 1 && rand() < $NESTING_PROBABILITY) {
            $output .= generate_rule_content($analysis, $rule_name, $depth + 1, $max_depth);
        } else {
            $output .= generate_terminal_content($analysis, $depth);
        }
        
        $output .= "\n" if $i < $num_items - 1;
    }
    
    $output .= ' ]';
    return $output;
}

sub generate_brace_structure {
    my ($analysis, $rule_name, $depth, $max_depth) = @_;
    
    my $output = '{';
    $output .= generate_random_identifier();
    $output .= "\n";
    
    my $num_items = int(rand($MAX_ITEMS_PER_LEVEL - $MIN_ITEMS_PER_LEVEL + 1)) + $MIN_ITEMS_PER_LEVEL;
    
    for (my $i = 0; $i < $num_items; $i++) {
        $output .= '  ';
        
        if ($depth < $max_depth - 1 && rand() < $NESTING_PROBABILITY) {
            $output .= generate_rule_content($analysis, $rule_name, $depth + 1, $max_depth);
        } else {
            $output .= generate_terminal_content($analysis, $depth);
        }
        
        $output .= "\n" if $i < $num_items - 1;
    }
    
    $output .= ' }';
    return $output;
}

sub generate_container_structure {
    my ($analysis, $rule_name, $depth, $max_depth) = @_;
    
    my $output = '(';
    $output .= generate_random_identifier();
    $output .= "\n";
    
    my $num_items = int(rand($MAX_ITEMS_PER_LEVEL - $MIN_ITEMS_PER_LEVEL + 1)) + $MIN_ITEMS_PER_LEVEL;
    
    for (my $i = 0; $i < $num_items; $i++) {
        $output .= ' ';
        
        if ($depth < $max_depth - 1 && rand() < $NESTING_PROBABILITY) {
            $output .= generate_rule_content($analysis, $rule_name, $depth + 1, $max_depth);
        } else {
            $output .= generate_terminal_content($analysis, $depth);
        }
        
        $output .= "\n" if $i < $num_items - 1;
    }
    
    $output .= ')';
    return $output;
}

sub generate_terminal_content {
    my ($analysis, $depth) = @_;
    
    # Get available terminal rules from the grammar
    my $terminal_rules = $analysis->{rules}{_terminal_rules};
    my @available_terminals = keys %$terminal_rules;
    
    if (@available_terminals == 0) {
        # Fallback to basic content types
        return generate_fallback_content($analysis);
    }
    
    # Choose a random terminal rule
    my $chosen_rule = $available_terminals[int(rand(@available_terminals))];
    my $rule_data = $terminal_rules->{$chosen_rule};
    my $structure_type = $rule_data->{structure_type};
    
    # Generate content based on the terminal rule's structure type
    if ($structure_type eq 'quoted') {
        return '"' . generate_random_string(5, 15) . '"';
    } elsif ($structure_type eq 'identifier') {
        return generate_random_identifier();
    } elsif ($structure_type eq 'whitespace') {
        return '; ' . generate_random_comment();
    } else {
        return generate_random_identifier();
    }
}

sub generate_fallback_content {
    my ($analysis) = @_;
    
    # Generate basic content when no terminal rules are available
    return generate_random_identifier();
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

sub generate_random_comment {
    my @comments = qw(this is a test comment generated for testing purposes);
    return $comments[rand(@comments)];
}

# Example usage and testing
if ($VERBOSE) {
    print "=== Grammar-Aware Test Input Generator ===\n";
    print "Spec file: $spec_file\n";
    print "Samples: $NUM_SAMPLES\n";
    print "Max depth: $MAX_DEPTH\n";
    print "Items per level: $MIN_ITEMS_PER_LEVEL to $MAX_ITEMS_PER_LEVEL\n";
    print "Nesting probability: $NESTING_PROBABILITY\n";
    print "Output: " . ($OUTPUT_FILE || "stdout") . "\n";
}
