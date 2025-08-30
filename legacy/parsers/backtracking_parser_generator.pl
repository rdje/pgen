#!/usr/bin/perl
use strict;
use warnings;
use Data::Dumper;
use Getopt::Long;
use File::Basename;

# New backtracking parser generator with memoization
# This generates proper recursive descent parsers

use lib 'fx/perl';
use LinkedSpec;

# Command line options
my $output_file = "";
my $verbose = 0;
my $help = 0;

GetOptions(
    'output|o=s' => \$output_file,
    'verbose|v' => \$verbose,
    'help|h' => \$help,
) or die "Error in command line arguments\n";

if ($help) {
    print <<EOF;
Usage: $0 [options] <ebnf_file>

Options:
  -o, --output FILE    Write parser to FILE (default: stdout)
  -v, --verbose        Enable verbose output
  -h, --help           Show this help message

Examples:
  $0 grammar.ebnf                        # Output to stdout
  $0 -o parser.pm grammar.ebnf           # Output to file
  $0 --verbose grammar.ebnf              # Show debug info
EOF
    exit 0;
}

# Parse EBNF file
my $ebnf_file = $ARGV[0] || die "Error: No EBNF file specified. Use --help for usage.\n";
open my $fh, "<", "fx/specs/ebnf.spec" or die "Cannot open ebnf.spec: $!";
my $spec_content = do { local $/; <$fh> };
close $fh;

open my $fh2, "<", $ebnf_file or die "Cannot open $ebnf_file: $!";
my $input_content = do { local $/; <$fh2> };
close $fh2;

my $parser = LinkedSpec::Get(\$spec_content);
my $raw_ast = $parser->(\$input_content);

unless ($raw_ast) {
    die "Error: Failed to parse EBNF file '$ebnf_file'. Check syntax.\n";
}

# Build grammar tree from AST
my %grammar = ();
my @rule_order = ();

for my $rule (@$raw_ast) {
    next if @$rule < 2;
    my $rule_name = $rule->[0];
    push @rule_order, $rule_name unless exists $grammar{$rule_name};
    
    # Collect rule elements (skip rule name and filter out probabilities)
    my @elements = @$rule[1..$#{$rule}];
    my @filtered_elements = ();
    
    for my $element (@elements) {
        # Skip probability annotations for parsing
        if (ref($element) eq 'ARRAY' && $element->[0] eq 'probability') {
            next;  # Skip probabilities during parser generation
        }
        push @filtered_elements, $element;
    }
    
    $grammar{$rule_name} = \@filtered_elements;
}

# Validate grammar completeness
validate_grammar_completeness(\%grammar);

# Validate grammar
unless (@rule_order) {
    die "Error: No valid rules found in EBNF file '$ebnf_file'.\n";
}

print STDERR "Found rules: " . join(", ", @rule_order) . "\n" if $verbose;

# Generate parser module
my $parser_code = generate_backtracking_parser(\%grammar, \@rule_order);

# Output to file or stdout
if ($output_file) {
    open my $out_fh, '>', $output_file or die "Cannot write to '$output_file': $!\n";
    print $out_fh $parser_code;
    close $out_fh;
    print STDERR "Parser written to: $output_file\n" if $verbose;
} else {
    print $parser_code;
}

sub generate_backtracking_parser {
    my ($grammar, $rule_order) = @_;
    
    my $main_rule = $rule_order->[0];
    
    my $module = qq{
package yapg::BacktrackingParser;
use strict;
use warnings;

# Memoization cache: rule_name => { position => [result, new_position] }
my \%memo_cache = ();

# Clear memoization cache  
sub clear_memo_cache { \%memo_cache = (); }

# Main parsing entry point
sub parse {
    my (\$input_ref) = \@_;
    clear_memo_cache();
    pos(\$\$input_ref) = 0;
    return parse_$main_rule(\$input_ref, 0);
}

# Try parsing alternatives with backtracking
sub try_alternatives {
    my (\$input_ref, \$pos, \@parsers) = \@_;
    
    for my \$parser (\@parsers) {
        my \$result = \$parser->(\$input_ref, \$pos);
        if (defined \$result) {
            return \$result;
        }
    }
    return undef;
}

# Parse sequence with backtracking
sub parse_sequence {
    my (\$input_ref, \$pos, \@parsers) = \@_;
    
    my \$current_pos = \$pos;
    my \@results = ();
    
    for my \$parser (\@parsers) {
        my \$result = \$parser->(\$input_ref, \$current_pos);
        if (!defined \$result) {
            return undef;  # Sequence failed
        }
        push \@results, \$result;
        \$current_pos = pos(\$\$input_ref);
    }
    
    return \\\@results;
}

# Parse literal terminal
sub parse_literal {
    my (\$input_ref, \$pos, \$literal) = \@_;
    pos(\$\$input_ref) = \$pos;
    
    if (\$\$input_ref =~ /\\G\\Q\$literal\\E/gc) {
        return \$literal;
    }
    return undef;
}

# Parse quantified expression (e.g., element+, element*, element?)
sub parse_quantified {
    my (\$input_ref, \$pos, \$parser_func, \$min, \$max) = \@_;
    
    my \$current_pos = \$pos;
    my \@results = ();
    my \$count = 0;
    
    while (\$count < \$max) {
        my \$result = \$parser_func->(\$input_ref, \$current_pos);
        if (defined \$result) {
            push \@results, \$result;
            \$current_pos = pos(\$\$input_ref);
            \$count++;
        } else {
            last;
        }
    }
    
    if (\$count >= \$min) {
        pos(\$\$input_ref) = \$current_pos;
        return \\\@results;
    } else {
        return undef;
    }
}

# Memoized rule parser
sub memoized_rule {
    my (\$rule_name, \$input_ref, \$pos, \$parser_func) = \@_;
    
    # Check memo cache
    if (exists \$memo_cache{\$rule_name}{\$pos}) {
        my (\$cached_result, \$cached_pos) = \@{\$memo_cache{\$rule_name}{\$pos}};
        pos(\$\$input_ref) = \$cached_pos;
        return \$cached_result;
    }
    
    # Parse and cache
    my \$result = \$parser_func->(\$input_ref, \$pos);
    my \$new_pos = pos(\$\$input_ref);
    
    \$memo_cache{\$rule_name}{\$pos} = [\$result, \$new_pos];
    return \$result;
}

};

    # Generate parsing functions for each rule
    for my $rule_name (@$rule_order) {
        my $elements = $grammar->{$rule_name};
        $module .= generate_rule_parser($rule_name, $elements);
    }
    
    $module .= "\n1;\n";
    return $module;
}

sub generate_rule_parser {
    my ($rule_name, $elements) = @_;
    
    # Analyze if this is an OR rule or sequence
    my @or_groups = ();
    my @current_group = ();
    
    for my $element (@$elements) {
        if (defined $element && $element eq '|') {
            if (@current_group) {
                push @or_groups, [@current_group];
                @current_group = ();
            }
        } else {
            push @current_group, $element;
        }
    }
    push @or_groups, [@current_group] if @current_group;
    
    my $parser_code = "sub parse_$rule_name {\n";
    $parser_code .= "    my (\$input_ref, \$pos) = \@_;\n";
    $parser_code .= "    return memoized_rule('$rule_name', \$input_ref, \$pos, sub {\n";
    $parser_code .= "        my (\$input_ref, \$pos) = \@_;\n";
    
    if (@or_groups > 1) {
        # OR rule - try each alternative
        $parser_code .= "        return try_alternatives(\$input_ref, \$pos,\n";
        for my $group (@or_groups) {
            $parser_code .= "            sub { " . generate_sequence_parser($group) . " },\n";
        }
        $parser_code .= "        );\n";
    } else {
        # Single sequence
        $parser_code .= "        " . generate_sequence_parser($or_groups[0]) . "\n";
    }
    
    $parser_code .= "    });\n";
    $parser_code .= "}\n\n";
    
    return $parser_code;
}

sub generate_sequence_parser {
    my ($elements) = @_;
    
    # Handle quantifiers by processing elements in pairs
    my @processed_elements = ();
    
    my $i = 0;
    while ($i <= $#{$elements}) {
        my $element = $elements->[$i];
        my $next_element = ($i < $#{$elements}) ? $elements->[$i + 1] : undef;
        
        if (defined $next_element && $next_element =~ /^[\+\*\?]$/) {
            # This element has a quantifier
            push @processed_elements, [$element, $next_element];
            $i += 2;  # Skip both element and quantifier
        } else {
            # Regular element without quantifier
            push @processed_elements, [$element, undef];
            $i++;
        }
    }
    
    if (@processed_elements == 1) {
        return generate_element_with_quantifier($processed_elements[0]);
    }
    
    my $code = "return parse_sequence(\$input_ref, \$pos,\n";
    for my $elem_pair (@processed_elements) {
        $code .= "            sub { " . generate_element_with_quantifier($elem_pair) . " },\n";
    }
    $code .= "        );";
    
    return $code;
}

sub generate_element_with_quantifier {
    my ($elem_pair) = @_;
    my ($element, $quantifier) = @$elem_pair;
    
    my $base_parser = generate_element_parser($element);
    
    if (!defined $quantifier) {
        return $base_parser;
    }
    
    if ($quantifier eq '+') {
        return "parse_quantified(\$input_ref, \$pos, sub { my (\$input_ref, \$pos) = \@_; $base_parser }, 1, 999)";
    } elsif ($quantifier eq '*') {
        return "parse_quantified(\$input_ref, \$pos, sub { my (\$input_ref, \$pos) = \@_; $base_parser }, 0, 999)";
    } elsif ($quantifier eq '?') {
        return "parse_quantified(\$input_ref, \$pos, sub { my (\$input_ref, \$pos) = \@_; $base_parser }, 0, 1)";
    }
    
    return $base_parser;
}

sub generate_element_parser {
    my ($element) = @_;
    
    if (ref($element) eq 'ARRAY' && $element->[0] eq 'terminal') {
        my $literal = $element->[1];
        return "parse_literal(\$input_ref, \$pos, '$literal')";
    } elsif (!ref($element) && $element =~ /^[a-zA-Z_]\w*$/) {
        # Valid rule name
        return "parse_$element(\$input_ref, \$pos)";
    } elsif (!ref($element) && $element =~ /^[\+\*\?]$/) {
        # Quantifier - should be handled by previous element
        return "undef";  # Skip quantifiers for now
    }
    
    return "undef";  # Unknown element type
}

sub validate_grammar_completeness {
    my ($grammar_tree) = @_;
    my %defined_rules = map { $_ => 1 } keys %$grammar_tree;
    my @errors = ();
    
    # Collect all referenced rule names
    my %referenced_rules = ();
    
    for my $rule_name (keys %$grammar_tree) {
        my $rule = $grammar_tree->{$rule_name};
        collect_referenced_rules($rule, \%referenced_rules);
    }
    
    # Check for undefined rules
    for my $ref_rule (keys %referenced_rules) {
        unless ($defined_rules{$ref_rule}) {
            push @errors, "Undefined rule referenced: '$ref_rule'";
        }
    }
    
    if (@errors) {
        die "Grammar validation errors:\n" . join("\n", map { "  $_" } @errors) . "\n";
    }
}

sub collect_referenced_rules {
    my ($node, $referenced) = @_;
    
    if (ref($node) eq 'ARRAY') {
        # Check if this is a terminal marker
        if (@$node == 2 && $node->[0] eq 'terminal') {
            # This is a terminal - don't process the content as a rule reference
            return;
        }
        
        # Handle array-based AST structure from EBNF parser
        for my $element (@$node) {
            collect_referenced_rules($element, $referenced);
        }
    } elsif (!ref($node) && $node =~ /^[a-zA-Z_]\w*$/) {
        # This looks like a rule reference (non-terminal)
        # But need to check if it's not a special token
        unless ($node eq 'terminal' || $node =~ /^['"]/ || $node =~ /[\+\*\?]$/) {
            $referenced->{$node} = 1;
        }
    }
}
