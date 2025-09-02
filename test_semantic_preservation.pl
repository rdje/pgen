#!/usr/bin/env perl

use strict;
use warnings;
use Data::Dumper;
use JSON;
use lib 'perl';

# Test semantic annotation preservation through the transformation pipeline

use AST::Transform qw(process_to_final_ast);

# Read the JSON output from ebnf_to_json.pl to get the raw AST
my $json_file = 'test_semantic_annotations.json';

# First generate the JSON if it doesn't exist
system("perl tools/ebnf_to_json.pl test_semantic_annotations.ebnf > $json_file");

# Read the JSON
open my $fh, '<', $json_file or die "Cannot open $json_file: $!";
my $json_content = do { local $/; <$fh> };
close $fh;

my $data = decode_json($json_content);
my $raw_ast = $data->{raw_ast};

print "=== TESTING SEMANTIC ANNOTATION PRESERVATION ===\n\n";
print "Raw AST contains " . scalar(@$raw_ast) . " rules\n\n";

# Show first rule with semantic annotations
print "Example raw rule with semantic annotations:\n";
print Dumper($raw_ast->[0]);
print "\n";

# Transform the AST
my ($grammar_tree, $rule_order) = process_to_final_ast($raw_ast);

print "=== TRANSFORMED GRAMMAR TREE ===\n";
print Dumper($grammar_tree);
print "\n";

print "=== RULE ORDER ===\n";
print Dumper($rule_order);
print "\n";

# Function to search for semantic annotations in the transformed output
sub check_semantic_annotations {
    my ($node, $path, $found_count) = @_;
    $path = $path || "root";
    $found_count = $found_count || 0;
    
    if (ref($node) eq 'HASH') {
        if (exists $node->{semantic_annotations}) {
            $found_count++;
            print "✅ FOUND semantic_annotations at path: $path\n";
            print "    Count: " . scalar(@{$node->{semantic_annotations}}) . " annotations\n";
            print "    Content: " . Dumper($node->{semantic_annotations}) . "\n";
        }
        
        # Recursively check all hash values
        foreach my $key (keys %$node) {
            if (ref($node->{$key})) {
                $found_count = check_semantic_annotations($node->{$key}, "$path.$key", $found_count);
            }
        }
    } elsif (ref($node) eq 'ARRAY') {
        for my $i (0 .. $#$node) {
            if (ref($node->[$i])) {
                $found_count = check_semantic_annotations($node->[$i], "$path\[$i\]", $found_count);
            }
        }
    }
    
    return $found_count;
}

# Check for preserved semantic annotations
print "=== SEARCHING FOR PRESERVED SEMANTIC ANNOTATIONS ===\n";
my $annotations_found = check_semantic_annotations($grammar_tree);

print "\n=== ANALYSIS ===\n";
if ($annotations_found == 0) {
    print "❌ NO semantic_annotations fields found in transformed output!\n";
    print "   This means semantic annotations are NOT being preserved for DataGenerator use.\n\n";
    
    # Check if they might be in raw form somewhere
    sub find_raw_annotations {
        my ($node, $path) = @_;
        $path = $path || "root";
        my $found = 0;
        
        if (ref($node) eq 'ARRAY' && @$node >= 2 && $node->[0] eq 'semantic_annotation') {
            print "🔍 Found RAW semantic annotation at $path: ";
            print "[$node->[0], [$node->[1]->[0], $node->[1]->[1]]]\\n";
            $found = 1;
        } elsif (ref($node) eq 'HASH') {
            foreach my $key (keys %$node) {
                if (ref($node->{$key})) {
                    $found += find_raw_annotations($node->{$key}, "$path.$key");
                }
            }
        } elsif (ref($node) eq 'ARRAY') {
            for my $i (0 .. $#$node) {
                if (ref($node->[$i])) {
                    $found += find_raw_annotations($node->[$i], "$path\[$i\]");
                }
            }
        }
        return $found;
    }
    
    print "Searching for any traces of semantic annotations...\n";
    my $raw_found = find_raw_annotations($grammar_tree);
    if ($raw_found == 0) {
        print "❌ No traces of semantic annotations found anywhere in the output!\n";
        print "   The semantic annotation filtering/preservation system may not be working.\n";
    } else {
        print "⚠️  Found $raw_found raw semantic annotations still present.\n";
        print "   This suggests they're being preserved but not properly structured.\n";
    }
} else {
    print "✅ SUCCESS: Found $annotations_found semantic annotation fields preserved!\n";
    print "   Semantic annotations ARE available for DataGenerator use in the 'semantic_annotations' field.\n";
}

# Count semantic annotations in input vs output
my $input_annotations = 0;
for my $rule (@$raw_ast) {
    for my $element (@$rule) {
        if (ref($element) eq 'ARRAY' && $element->[0] eq 'semantic_annotation') {
            $input_annotations++;
        }
    }
}

print "\n=== SUMMARY ===\n";
print "Input semantic annotations: $input_annotations\n";
print "Output preserved annotations: $annotations_found\n";

if ($annotations_found > 0 && $annotations_found >= $input_annotations) {
    print "✅ CONCLUSION: Semantic annotations are properly preserved and available for DataGenerator!\n";
} elsif ($annotations_found > 0 && $annotations_found < $input_annotations) {
    print "⚠️  CONCLUSION: Some semantic annotations preserved, but not all. Partial success.\n";
} else {
    print "❌ CONCLUSION: Semantic annotations are NOT preserved. DataGenerator cannot access them.\n";
    print "   The implementation needs to be fixed to enable constrained data generation.\n";
}

# Clean up
unlink $json_file;
