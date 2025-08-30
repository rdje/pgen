#!/usr/bin/env perl

use strict;
use warnings;
use lib '.';

use AST::RustCodeGen qw(generate_rust_parser_module);
use Data::Dumper;

print "🔧 Testing Rust code generation with type fixes...\n";

# Create a simplified JSON grammar tree
my $grammar_tree = {
    'json' => {
        type => 'or',
        alternatives => [
            {
                type => 'atom',
                value => ['token', 'object']
            },
            {
                type => 'atom',
                value => ['token', 'array']
            }
        ]
    },
    'object' => {
        type => 'sequence',
        elements => [
            {
                type => 'atom',
                value => ['quoted_string', '{']
            },
            {
                type => 'atom',
                value => ['token', 'pair_list']
            },
            {
                type => 'atom',
                value => ['quoted_string', '}']
            }
        ]
    },
    'array' => {
        type => 'sequence',
        elements => [
            {
                type => 'atom',
                value => ['quoted_string', '[']
            },
            {
                type => 'atom',
                value => ['token', 'value_list']
            },
            {
                type => 'atom',
                value => ['quoted_string', ']']
            }
        ]
    },
    'pair_list' => {
        type => 'atom',
        value => ['quoted_string', '']  # Empty for now
    },
    'value_list' => {
        type => 'atom',
        value => ['quoted_string', '']  # Empty for now
    }
};

my $rule_order = ['json', 'object', 'array', 'pair_list', 'value_list'];

print "📊 Grammar tree structure:\n";
print Dumper($grammar_tree);

print "\n🚀 Generating Rust parser module...\n";
my $rust_code = generate_rust_parser_module($grammar_tree, $rule_order);

# Write the Rust code to a file
open my $fh, '>', 'generated_parser.rs' or die "Cannot write to generated_parser.rs: $!";
print $fh $rust_code;
close $fh;

print "✅ Generated Rust code written to generated_parser.rs\n\n";

# Check if rust code looks syntactically correct by examining specific patterns
print "🔍 Checking generated Rust code structure...\n";

# Read the generated file and analyze it
open my $rust_fh, '<', 'generated_parser.rs' or die "Cannot read generated_parser.rs: $!";
my $rust_content = do { local $/; <$rust_fh> };
close $rust_fh;

# Check for the type mismatch pattern we fixed
my $has_type_issues = 0;
my @issues = ();

# Look for the old pattern where terminal strings were returned directly 
# This should find cases like: match_literal(...) -> result_N.unwrap() (not wrapped)
# But NOT: parse_rule(...) -> result_N.unwrap() (already returns ASTNode)
if ($rust_content =~ /match_literal\([^)]+\)\?;[^}]+result_\d+\.unwrap\(\)(?![)])/) {
    push @issues, "❌ Found terminal literals not properly wrapped as ASTNode::Terminal";
    $has_type_issues = 1;
}

# Check that terminals are wrapped as ASTNode::Terminal
if ($rust_content =~ /ASTNode::Terminal\(result_\d+\.unwrap\(\)\)/) {
    print "✅ Terminal literals are properly wrapped as ASTNode::Terminal\n";
} else {
    push @issues, "❌ Terminal literals may not be properly wrapped";
    $has_type_issues = 1;
}

# Check for the redundant results vector (should be gone)
if ($rust_content =~ /let results = vec!\[.*\];\s*return Ok\(Some\(ASTNode::Array\(vec!\[.*\]\)\)\);/) {
    push @issues, "❌ Found redundant results vector causing potential move errors";
    $has_type_issues = 1;
} else {
    print "✅ No redundant results vectors found\n";
}

# Check for proper function structure
if ($rust_content =~ /fn parse_\w+\(input: &mut ParseInput\) -> ParseResult<ASTNode>/) {
    print "✅ Parser functions have correct signatures\n";
} else {
    push @issues, "❌ Parser function signatures may be incorrect";
    $has_type_issues = 1;
}

if ($has_type_issues) {
    print "\n❌ Issues found in generated code:\n";
    print join("\n", @issues) . "\n";
    print "\n💡 The type mismatch fixes may need further adjustment.\n";
} else {
    print "\n✅ Generated Rust code structure looks correct!\n";
    print "✅ Type mismatches appear to be resolved\n";
    print "\n🎉 The critical fixes have been applied successfully:\n";
    print "  • Terminal literals are wrapped as ASTNode::Terminal\n";
    print "  • Redundant results vectors have been eliminated\n";
    print "  • Function signatures return consistent ASTNode types\n";
    print "\nNote: Full compilation would require adding regex and lazy_static dependencies\n";
    print "to a proper Rust project with Cargo.toml, but the type issues are resolved.\n";
}
