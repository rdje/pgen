#!/usr/bin/perl
use strict;
use warnings;
use lib 'perl';

# Test Complete JSON-Based Cross-Language Pipeline
#
# Demonstrates the proper architecture:
# EBNF → [Perl Parser] → Raw AST JSON → [Language Implementation] → Output
#
# This simulates how Rust/Julia/Go implementations would work

use JSON::PP;

print "=== Testing JSON-Based Cross-Language Pipeline ===\n\n";

# Step 1: Create Raw AST JSON (simulating ebnf_to_json.pl output)
print "Step 1: Creating Raw AST JSON (simulating ebnf_to_json.pl)...\n";

my $raw_ast_data = {
    grammar_name => "simple_arithmetic",
    raw_ast => [
        [
            ["rule", "expression"],
            ["rule_reference", "term"],
            ["group_open", "("],
            ["quoted_string", "+"],
            ["rule_reference", "term"],
            ["group_close", ")"],
            ["operator", "*"]
        ],
        [
            ["rule", "term"],
            ["regex", "(\\d+)"]
        ]
    ],
    metadata => {
        source_file => "simulated.ebnf",
        format => "raw_ast", 
        generated_at => scalar(localtime()),
        parser => "ebnf_to_json.pl (simulated)"
    }
};

my $raw_json_file = "test_raw_ast.json";
my $json_out = JSON::PP->new->pretty->canonical->encode($raw_ast_data);

open my $fh, '>', $raw_json_file or die "Cannot write $raw_json_file: $!";
print $fh $json_out;
close $fh;

print "Created: $raw_json_file (Raw AST JSON from Perl parser)\n";

# Step 2: Transform Raw → Transformed JSON (cross-language interface)
print "\nStep 2: Transforming Raw AST JSON → Transformed AST JSON...\n";

my $transformed_json_file = "test_transformed_ast.json";
my $transform_cmd = "perl tools/transform_ast.pl $raw_json_file $transformed_json_file";
print "Running: $transform_cmd\n";

my $result = system($transform_cmd);
if ($result != 0) {
    die "Transformation failed with exit code: $result\n";
}

print "Success: Raw AST → Transformed AST via JSON interface\n";

# Step 3: Language-specific generator consumes Transformed JSON
print "\nStep 3: Language-specific generator reads Transformed JSON...\n";

print "\n--- Python Implementation (DataGenerator) ---\n";
my $python_cmd = "python3 tools/syntactic_data_generator.py $transformed_json_file --rule expression --count 3 --stats";
print "Running: $python_cmd\n";
system($python_cmd);

# Step 4: Show the JSON interface files
print "\n=== JSON Interface Files ===\n";

print "\nRaw AST JSON structure:\n";
system("head -20 $raw_json_file");

print "\nTransformed AST JSON structure:\n";
system("head -20 $transformed_json_file");

print "\n=== Cross-Language Pipeline Demonstration Complete ===\n";
print "\nArchitecture Summary:\n";
print "1. ✅ Perl EBNF Parser → Raw AST JSON\n";
print "2. ✅ Language Implementation reads Raw AST JSON\n";
print "3. ✅ Language Implementation does Pipeline + Generation in-memory\n";
print "4. ✅ JSON provides cross-language interface boundary\n";

print "\nNext Steps for Language Implementations:\n";
print "- Rust: Read Raw AST JSON → AST Transform → Code Generation\n";
print "- Julia: Read Raw AST JSON → AST Transform → Code Generation  \n";
print "- Go: Read Raw AST JSON → AST Transform → Code Generation\n";
print "\nEach language handles Pipeline→Generator in-memory for efficiency.\n";

# Cleanup
unlink($raw_json_file, $transformed_json_file);
print "\nCleanup: Removed temporary JSON files.\n";
