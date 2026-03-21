#!/usr/bin/env perl
use strict;
use warnings;
use Data::Dumper;
use Getopt::Long;

# Generic Grammar-Driven Test Input Generator for LinkedSpec
# Analyzes any .spec file and generates valid input based purely on its grammar rules

my $VERBOSE = 0;
my $OUTPUT_FILE = '';
my $NUM_SAMPLES = 3;
my $MAX_DEPTH = 4;
my $MIN_ITEMS = 1;
my $MAX_ITEMS = 3;
my $MAX_SIZE = 0;  # 0 = no size limit

GetOptions(
    'verbose|v' => \$VERBOSE,
    'output|o=s' => \$OUTPUT_FILE,
    'samples|n=i' => \$NUM_SAMPLES,
    'max-depth|d=i' => \$MAX_DEPTH,
    'min-items|i=i' => \$MIN_ITEMS,
    'max-items|x=i' => \$MAX_ITEMS,
    'max-size|s=i' => \$MAX_SIZE,
    'help|h' => sub { show_help(); exit 0; }
);

sub show_help {
    print <<EOF;
Generic Grammar-Driven LinkedSpec Test Input Generator

Analyzes any .spec file and generates valid input based purely on its grammar rules.
No assumptions made about rule names or content - everything extracted from the .spec.

Usage: perl generate_input_generic.pl [options] <spec_file>

Options:
    -v, --verbose        Show detailed analysis
    -o, --output FILE    Output to file instead of stdout
    -n, --samples N      Number of samples to generate (default: 3)
    -d, --max-depth N    Maximum nesting depth (default: 4)
    -i, --min-items N    Minimum items per container (default: 1)
    -x, --max-items N    Maximum items per container (default: 3)
    -s, --max-size N     Maximum output size in characters (0=unlimited, default: 0)
    -h, --help          Show this help

Examples:
    perl generate_input_generic.pl specs/Lispish.spec
    perl generate_input_generic.pl -v -n 5 -d 6 specs/ifelse.spec
    perl generate_input_generic.pl specs/regdef.spec

EOF
}

die "Usage: perl generate_input_generic.pl [options] <spec_file>\n" unless @ARGV;
my $spec_file = shift @ARGV;

# Load and analyze the spec file
my $spec_content = load_spec_file($spec_file);
my $analysis = analyze_grammar($spec_content);

if ($VERBOSE) {
    print "=== Grammar Analysis ===\n";
    print Dumper($analysis);
}

# Generate test inputs
my @generated_inputs;
for (my $i = 1; $i <= $NUM_SAMPLES; $i++) {
    my $input = generate_input($analysis, $MAX_DEPTH);
    
    # Apply size limit if specified
    if ($MAX_SIZE > 0 && length($input) > $MAX_SIZE) {
        $input = substr($input, 0, $MAX_SIZE);
        # Try to end at a reasonable boundary
        if ($input =~ /^(.*[)}\]"'\s])/s) {
            $input = $1;
        }
    }
    
    push @generated_inputs, $input;
    
    if ($VERBOSE) {
        print "=== Generated Input $i (length: " . length($input) . ") ===\n";
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

sub analyze_grammar {
    my ($content) = @_;
    my %analysis = (
        rules => {},
        top_rule => undef,
        regex_patterns => {},
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
            regex_patterns => [],
        };

        # Extract regex patterns for this rule
        my @patterns = extract_regex_patterns($section);
        $rule_data->{regex_patterns} = \@patterns;
        $analysis{regex_patterns}{$rule_name} = \@patterns;

        # Find all sub-rules referenced in "->" action blocks within this section
        while ($section =~ /->\s*([a-zA-Z_]\w*)(?:\[\d+\])?/g) {
            push @{$rule_data->{sub_rules}}, $1;
        }

        # A rule is a container if it has sub-rules
        if (@{$rule_data->{sub_rules}}) {
            $rule_data->{is_container} = 1;
        }

        $analysis{rules}{$rule_name} = $rule_data;
    }

    return \%analysis;
}

sub extract_regex_patterns {
    my ($section) = @_;
    my @patterns;
    
    # Find regex patterns like /pattern/ or qr/pattern/
    while ($section =~ m{(?:^|\s+)(?:qr)?/((?:[^/\\]|\\.)*)(?<!\\)/[gimsx]*}gm) {
        my $pattern = $1;
        push @patterns, $pattern;
    }
    
    return @patterns;
}

sub generate_input {
    my ($analysis, $max_depth, $current_depth) = @_;
    $current_depth ||= 0;
    
    return generate_simple_content() if $current_depth >= $max_depth;
    
    my $top_rule = $analysis->{top_rule};
    
    # Generate the main content
    my $result = generate_from_rule($analysis, $top_rule, $current_depth, $max_depth);
    
    return $result;
}

sub generate_from_rule {
    my ($analysis, $rule_name, $depth, $max_depth) = @_;
    
    my $rule = $analysis->{rules}{$rule_name};
    return generate_simple_content() unless $rule;
    
    # Avoid infinite recursion
    return generate_simple_content() if $depth >= $max_depth;
    
    my $patterns = $rule->{regex_patterns} || [];
    
    # If this rule has two patterns, it's likely a balanced structure (open/close)
    if (@$patterns == 2 && $rule->{is_container}) {
        return generate_balanced_structure($analysis, $rule_name, $depth, $max_depth);
    }
    
    # If this is a container rule with sub-rules
    if ($rule->{is_container} && @{$rule->{sub_rules}} > 0) {
        # Choose a random sub-rule to generate
        my @available_sub_rules = filter_safe_sub_rules($rule->{sub_rules}, $depth, $analysis);
        
        if (@available_sub_rules > 0) {
            my $chosen_sub_rule = $available_sub_rules[int(rand(@available_sub_rules))];
            return generate_from_rule($analysis, $chosen_sub_rule, $depth + 1, $max_depth);
        }
    }
    
    # For terminal rules, generate content that matches their regex pattern
    return generate_content_matching_pattern($patterns);
}

sub generate_balanced_structure {
    my ($analysis, $rule_name, $depth, $max_depth) = @_;
    
    my $rule = $analysis->{rules}{$rule_name};
    my $patterns = $rule->{regex_patterns} || [];
    
    # Generate opening delimiter from first pattern
    my $opening = generate_content_matching_pattern([$patterns->[0]]);
    
    # Generate closing delimiter from second pattern  
    my $closing = generate_content_matching_pattern([$patterns->[1]]);
    
    # Generate content inside
    my $content = '';
    if ($depth < $max_depth - 1 && @{$rule->{sub_rules}} > 0) {
        my @available_sub_rules = filter_safe_sub_rules($rule->{sub_rules}, $depth, $analysis);
        
        if (@available_sub_rules > 0) {
            my $num_items = int(rand($MAX_ITEMS - $MIN_ITEMS + 1)) + $MIN_ITEMS;
            
            for my $i (0..$num_items-1) {
                $content .= ' ' if $i > 0;
                my $chosen_sub_rule = $available_sub_rules[int(rand(@available_sub_rules))];
                $content .= generate_from_rule($analysis, $chosen_sub_rule, $depth + 1, $max_depth);
            }
        }
    }
    
    return $opening . $content . $closing;
}

sub generate_content_matching_pattern {
    my ($patterns) = @_;
    
    return generate_simple_content() unless $patterns && @$patterns > 0;
    
    my $pattern = $patterns->[0];
    
    # Handle common regex patterns and generate matching content
    # Look for key characters in the pattern, ignoring complex lookarounds
    if ($pattern =~ /\\\(/) {
        return '(';
    } elsif ($pattern =~ /\\\)/) {
        return ')';
    } elsif ($pattern =~ /\\\{/) {
        return '{';
    } elsif ($pattern =~ /\\\}/) {
        return '}';
    } elsif ($pattern =~ /\\\[/) {
        return '[';
    } elsif ($pattern =~ /\\\]/) {
        return ']';
    } elsif ($pattern =~ /\\d\+/) {
        return int(rand(1000));
    } elsif ($pattern =~ /\[a-zA-Z_\].*\\w/ || $pattern =~ /\\w\\S/) {
        return generate_identifier_content();
    } elsif ($pattern =~ /\\\+/) {
        return '+';
    } elsif ($pattern =~ /\\\-\\\-/) {
        return '--';
    } elsif ($pattern =~ /\\\+\\\+/) {
        return '++';
    } elsif ($pattern =~ /\\\-/) {
        return '-';
    } elsif ($pattern =~ /\\\*/) {
        return '*';
    } elsif ($pattern =~ /\\\//) {
        return '/';
    } elsif ($pattern =~ /\\\./) {
        return '.';
    } elsif ($pattern =~ /".*"/ || $pattern =~ /^"/) {
        return '"hello"';
    } elsif ($pattern =~ /'.*'/ || $pattern =~ /^'/) {
        return "'hello'";
    } elsif ($pattern =~ /#/) {
        return '# comment';
    } elsif ($pattern =~ /;/) {
        return '; comment';
    } elsif ($pattern =~ /\\w/) {
        return generate_word_content();
    } elsif ($pattern =~ /\\s/) {
        return ' ';
    } else {
        # For complex patterns, try to generate reasonable content
        return generate_simple_content();
    }
}

sub generate_identifier_content {
    my @identifiers = ('variable', 'func', 'data', 'value', 'x', 'y', 'result');
    return $identifiers[int(rand(@identifiers))];
}



sub generate_word_content {
    my @words = ('hello', 'world', 'test', 'data', 'value', 'example', 'sample');
    return $words[int(rand(@words))];
}

sub generate_simple_content {
    my @content = ('hello', 'test', 'data', '42', 'example');
    return $content[int(rand(@content))];
}

sub filter_safe_sub_rules {
    my ($sub_rules, $depth, $analysis) = @_;
    
    # Filter out rules that can break structure integrity based on their regex patterns
    my @safe_rules = grep { 
        my $rule_name = $_;
        my $patterns = $analysis->{regex_patterns}{$rule_name} || [];
        
        # Check if any pattern looks like a line comment (could break structure)
        my $has_line_comment = 0;
        for my $pattern (@$patterns) {
            if ($pattern =~ /^[;#\/]/ && @$patterns == 1) {
                # Single pattern that starts with comment chars - likely line comment
                $has_line_comment = 1;
                last;
            }
        }
        
        !$has_line_comment;  # Exclude rules with line comments
    } @{$sub_rules};
    
    return @safe_rules;
}

sub generate_random_terminal {
    my @terminals = (
        'hello',
        'world', 
        'test',
        'data',
        'value',
        'example',
        '"quoted string"',
        'identifier',
        '42',
        'sample'
    );
    
    return $terminals[int(rand(@terminals))];
}
