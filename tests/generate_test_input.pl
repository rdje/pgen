#!/usr/bin/env perl

use strict;
use warnings;
use Data::Dumper;
use Getopt::Long;

# Test Input Generator for LinkedSpec
# Generates pseudo-random input files based on .spec file analysis

my $VERBOSE = 0;
my $OUTPUT_FILE = '';
my $NUM_SAMPLES = 3;
my $MAX_DEPTH = 5;
my $MIN_ITEMS_PER_LEVEL = 1;
my $MAX_ITEMS_PER_LEVEL = 5;
my $NESTING_PROBABILITY = 0.6;  # 60% chance of nested structure vs terminal

GetOptions(
    'verbose|v' => \$VERBOSE,
    'output|o=s' => \$OUTPUT_FILE,
    'samples|n=i' => \$NUM_SAMPLES,
    'max-depth|d=i' => \$MAX_DEPTH,
    'min-items|min=i' => \$MIN_ITEMS_PER_LEVEL,
    'max-items|max=i' => \$MAX_ITEMS_PER_LEVEL,
    'nesting-prob|p=f' => \$NESTING_PROBABILITY,
    'help|h' => sub { show_help(); exit 0; }
);

sub show_help {
    print <<EOF;
LinkedSpec Test Input Generator

Usage: perl generate_test_input.pl [options] <spec_file>

Options:
    -v, --verbose       Enable verbose output
    -o, --output FILE   Output file (default: stdout)
    -n, --samples N     Number of samples to generate (default: 3)
    -d, --max-depth N   Maximum nesting depth (default: 5)
    -min, --min-items N Minimum items per level (default: 1)
    -max, --max-items N Maximum items per level (default: 5)
    -p, --nesting-prob F Probability of nested vs terminal (default: 0.6)
    -h, --help          Show this help

Examples:
    perl generate_test_input.pl specs/valid/basic.spec
    perl generate_test_input.pl -o test_input.txt -n 5 specs/valid/basic.spec
    perl generate_test_input.pl -v -d 3 -min 2 -max 4 -p 0.7 specs/valid/basic.spec

EOF
}

die "Usage: perl generate_test_input.pl [options] <spec_file>\n" unless @ARGV;
my $spec_file = shift @ARGV;

# Load and parse the spec file
my $spec_content = load_spec_file($spec_file);
my $grammar = analyze_spec_grammar($spec_content);

if ($VERBOSE) {
    print "=== Spec Analysis ===\n";
    print Dumper($grammar);
    print "=== Generation Parameters ===\n";
    print "Max Depth: $MAX_DEPTH\n";
    print "Items per level: $MIN_ITEMS_PER_LEVEL to $MAX_ITEMS_PER_LEVEL\n";
    print "Nesting probability: $NESTING_PROBABILITY\n";
}

# Generate test inputs
my @generated_inputs;
for (my $i = 1; $i <= $NUM_SAMPLES; $i++) {
    my $input = generate_input_from_grammar($grammar, $MAX_DEPTH);
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

sub analyze_spec_grammar {
    my ($content) = @_;
    my %grammar;
    
    # Extract top-level rule first
    if ($content =~ /^(\w+)::\s*$/m) {
        $grammar{'_top_rule'} = $1;
    }
    
    # Extract rules and their patterns
    while ($content =~ /^(\w+):\s*(.+)$/gm) {
        my ($rule_name, $patterns) = ($1, $2);
        next if $rule_name eq $grammar{'_top_rule'};  # Skip top rule
        
        $grammar{$rule_name} = {
            patterns => parse_regex_patterns($patterns),
            actions => extract_actions($content, $rule_name),
            dependencies => extract_dependencies($content, $rule_name)
        };
    }
    
    # Extract top rule actions
    if (exists $grammar{'_top_rule'}) {
        my $top_rule = $grammar{'_top_rule'};
        $grammar{$top_rule} = {
            patterns => [],
            actions => extract_actions($content, $top_rule),
            dependencies => extract_dependencies($content, $top_rule)
        };
    }
    
    return \%grammar;
}

sub parse_regex_patterns {
    my ($patterns) = @_;
    my @parsed_patterns;
    
    # Extract regex patterns like /pattern/
    while ($patterns =~ /\/([^\/]+)\//g) {
        push @parsed_patterns, $1;
    }
    
    return \@parsed_patterns;
}

sub extract_actions {
    my ($content, $rule_name) = @_;
    my @actions;
    
    # Find action blocks for this rule
    my $rule_section = extract_rule_section($content, $rule_name);
    
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

sub extract_rule_section {
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

sub extract_dependencies {
    my ($content, $rule_name) = @_;
    my @deps;
    
    my $rule_section = extract_rule_section($content, $rule_name);
    
    # Find call() dependencies
    while ($rule_section =~ /call\((\w+)\)/g) {
        push @deps, $1;
    }
    
    return \@deps;
}

sub generate_input_from_grammar {
    my ($grammar, $max_depth, $current_depth) = @_;
    $current_depth ||= 0;
    
    return '' if $current_depth >= $max_depth;
    
    my $top_rule = $grammar->{'_top_rule'} || 'Lispish';
    
    # Generate based on the grammar type
    if ($top_rule eq 'Lispish') {
        return generate_lispish_input($grammar, $current_depth, $max_depth);
    } elsif ($top_rule eq 'TestRule') {
        return generate_test_rule_input($grammar, $current_depth);
    } else {
        return generate_generic_input($grammar, $top_rule, $current_depth);
    }
}

sub generate_lispish_input {
    my ($grammar, $depth, $max_depth) = @_;
    
    # Generate hierarchical Lispish-style input with controlled structure
    my $output = '(';
    $output .= generate_random_identifier();
    $output .= "\n";
    
    # Determine number of items at this level
    my $num_items = int(rand($MAX_ITEMS_PER_LEVEL - $MIN_ITEMS_PER_LEVEL + 1)) + $MIN_ITEMS_PER_LEVEL;
    
    for (my $i = 0; $i < $num_items; $i++) {
        $output .= ' ';  # Indent
        
        # Decide whether this item should be nested or terminal
        if ($depth < $max_depth - 1 && rand() < $NESTING_PROBABILITY) {
            # Generate nested structure
            $output .= generate_nested_structure($grammar, $depth + 1, $max_depth);
        } else {
            # Generate terminal item
            $output .= generate_terminal_item($grammar, $depth);
        }
        
        $output .= "\n" if $i < $num_items - 1;
    }
    
    $output .= ')';
    return $output;
}

sub generate_nested_structure {
    my ($grammar, $depth, $max_depth) = @_;
    
    # Choose structure type based on Lispish grammar
    my $structure_type = int(rand(4));  # 0=parentheses, 1=square brackets, 2=curly braces, 3=mixed
    
    if ($structure_type == 0) {
        return generate_parentheses_structure($grammar, $depth, $max_depth);
    } elsif ($structure_type == 1) {
        return generate_square_bracket_structure($grammar, $depth, $max_depth);
    } elsif ($structure_type == 2) {
        return generate_curly_brace_structure($grammar, $depth, $max_depth);
    } else {
        return generate_mixed_structure($grammar, $depth, $max_depth);
    }
}

sub generate_parentheses_structure {
    my ($grammar, $depth, $max_depth) = @_;
    
    my $output = '(';
    $output .= generate_random_identifier();
    $output .= "\n";
    
    # Determine number of items
    my $num_items = int(rand($MAX_ITEMS_PER_LEVEL - $MIN_ITEMS_PER_LEVEL + 1)) + $MIN_ITEMS_PER_LEVEL;
    
    for (my $i = 0; $i < $num_items; $i++) {
        $output .= '  ';  # Double indent for nested
        
        if ($depth < $max_depth - 1 && rand() < $NESTING_PROBABILITY) {
            $output .= generate_nested_structure($grammar, $depth + 1, $max_depth);
        } else {
            $output .= generate_terminal_item($grammar, $depth);
        }
        
        $output .= "\n" if $i < $num_items - 1;
    }
    
    $output .= ' )';
    return $output;
}

sub generate_square_bracket_structure {
    my ($grammar, $depth, $max_depth) = @_;
    
    my $output = '[';
    $output .= generate_random_identifier();
    $output .= "\n";
    
    my $num_items = int(rand($MAX_ITEMS_PER_LEVEL - $MIN_ITEMS_PER_LEVEL + 1)) + $MIN_ITEMS_PER_LEVEL;
    
    for (my $i = 0; $i < $num_items; $i++) {
        $output .= '  ';
        
        if ($depth < $max_depth - 1 && rand() < $NESTING_PROBABILITY) {
            $output .= generate_nested_structure($grammar, $depth + 1, $max_depth);
        } else {
            $output .= generate_terminal_item($grammar, $depth);
        }
        
        $output .= "\n" if $i < $num_items - 1;
    }
    
    $output .= ' ]';
    return $output;
}

sub generate_curly_brace_structure {
    my ($grammar, $depth, $max_depth) = @_;
    
    my $output = '{';
    $output .= generate_random_identifier();
    $output .= "\n";
    
    my $num_items = int(rand($MAX_ITEMS_PER_LEVEL - $MIN_ITEMS_PER_LEVEL + 1)) + $MIN_ITEMS_PER_LEVEL;
    
    for (my $i = 0; $i < $num_items; $i++) {
        $output .= '  ';
        
        if ($depth < $max_depth - 1 && rand() < $NESTING_PROBABILITY) {
            $output .= generate_nested_structure($grammar, $depth + 1, $max_depth);
        } else {
            $output .= generate_terminal_item($grammar, $depth);
        }
        
        $output .= "\n" if $i < $num_items - 1;
    }
    
    $output .= ' }';
    return $output;
}

sub generate_mixed_structure {
    my ($grammar, $depth, $max_depth) = @_;
    
    # Randomly choose structure type for each item
    my $structure_type = int(rand(3));
    
    if ($structure_type == 0) {
        return generate_parentheses_structure($grammar, $depth, $max_depth);
    } elsif ($structure_type == 1) {
        return generate_square_bracket_structure($grammar, $depth, $max_depth);
    } else {
        return generate_curly_brace_structure($grammar, $depth, $max_depth);
    }
}

sub generate_terminal_item {
    my ($grammar, $depth) = @_;
    
    # Choose terminal content type based on Lispish grammar
    my $content_type = int(rand(6));  # 0=number, 1=dquotes, 2=squotes, 3=identifier, 4=comment, 5=simple
    
    if ($content_type == 0) {
        return generate_random_number();
    } elsif ($content_type == 1) {
        return '"' . generate_random_string(5, 15) . '"';
    } elsif ($content_type == 2) {
        return "'" . generate_random_string(5, 15) . "'";
    } elsif ($content_type == 3) {
        return generate_random_identifier();
    } elsif ($content_type == 4) {
        return '; ' . generate_random_comment();
    } else {
        # Simple structure
        return '(' . generate_random_identifier() . ' ' . generate_random_identifier() . ')';
    }
}

sub generate_test_rule_input {
    my ($grammar, $depth) = @_;
    
    # Generate simple pattern-based input
    my $output = '';
    
    # Check what patterns are available
    if (exists $grammar->{'rule_a'}) {
        $output .= 'a';
    }
    if (exists $grammar->{'rule_b'}) {
        $output .= 'b';
    }
    
    # If no specific rules, generate generic
    if ($output eq '') {
        $output = generate_random_string(3, 8);
    }
    
    return $output;
}

sub generate_generic_input {
    my ($grammar, $top_rule, $depth) = @_;
    
    # Generate generic input based on available rules
    my $output = '';
    
    foreach my $rule_name (keys %$grammar) {
        next if $rule_name eq '_top_rule' || $rule_name eq $top_rule;
        
        my $rule = $grammar->{$rule_name};
        foreach my $pattern (@{$rule->{patterns}}) {
            $output .= generate_from_pattern($pattern);
        }
    }
    
    return $output || generate_random_string(5, 10);
}

sub generate_from_pattern {
    my ($pattern) = @_;
    
    # Simple pattern generation based on regex
    if ($pattern =~ /^[\(\)]$/) {
        return $pattern;  # Literal parentheses
    }
    elsif ($pattern =~ /^"(.*?)"$/) {
        return '"' . generate_random_string(5, 15) . '"';  # Quoted strings
    }
    elsif ($pattern =~ /^\s+$/) {
        return ' ' x int(rand(3) + 1);  # Spaces
    }
    elsif ($pattern =~ /^[^\s"\{\}\(\)\[\];]+$/) {
        return generate_random_identifier();  # Identifiers
    }
    elsif ($pattern =~ /^;.*\\n$/) {
        return '; ' . generate_random_comment() . "\n";  # Comments
    }
    elsif ($pattern =~ /^\\\($/) {
        return '(';  # Escaped parentheses
    }
    elsif ($pattern =~ /^\\\)$/) {
        return ')';  # Escaped parentheses
    }
    elsif ($pattern =~ /^\\s\+$/) {
        return ' ' x int(rand(3) + 1);  # Spaces
    }
    elsif ($pattern =~ /^\[.*\]\+$/) {
        return generate_random_identifier();  # Character classes
    }
    else {
        # Generic pattern - generate random alphanumeric
        return generate_random_string(3, 8);
    }
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
    print "=== Test Input Generator ===\n";
    print "Spec file: $spec_file\n";
    print "Samples: $NUM_SAMPLES\n";
    print "Max depth: $MAX_DEPTH\n";
    print "Items per level: $MIN_ITEMS_PER_LEVEL to $MAX_ITEMS_PER_LEVEL\n";
    print "Nesting probability: $NESTING_PROBABILITY\n";
    print "Output: " . ($OUTPUT_FILE || "stdout") . "\n";
}
