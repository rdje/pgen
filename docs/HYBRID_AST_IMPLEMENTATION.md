# Hybrid AST Implementation Plan

## Architecture Overview

Support both direct Perl consumption and JSON serialization from a single pipeline:

```
EBNF → AST::Transform → [Direct Perl Objects | JSON File] → [Perl Tools | Multi-Language Tools]
```

## Implementation Steps

### Step 1: Extract Core Pipeline (Non-Breaking)

Add to `AST::Transform.pm`:

```perl
sub process_to_final_ast {
    my ($input, %options) = @_;
    
    # Set global options
    $quiet_mode = $options{quiet} // 0;
    $verbosity = $options{verbosity} // 'normal';
    $ERROR_CONTEXT->{verbosity} = $verbosity;
    
    # Handle both string content and pre-parsed AST
    my $raw_ast;
    if (ref($input) eq 'ARRAY') {
        $raw_ast = $input;
    } else {
        $raw_ast = load_ebnf_spec_from_content($input);
        unless ($raw_ast) {
            die "Failed to parse EBNF content";
        }
    }
    
    # Execute transformation pipeline
    print STDERR "\n=== Processing to Final AST ===\n" unless $quiet_mode;
    
    my $step2_result = step2_group_by_or($raw_ast);
    my $step2_5_result = step2_5_handle_parentheses($step2_result);
    my $step3_result = step3_parse_sequences($step2_5_result);
    my $step4_result = step4_handle_quantifiers($step3_result);
    my ($final_ast, $rule_order) = step5_build_tree_structure($step4_result);
    
    print STDERR "✅ Final AST ready with " . scalar(keys %$final_ast) . " rules\n" unless $quiet_mode;
    
    # Optional JSON serialization
    if ($options{output_json}) {
        serialize_final_ast_to_json($final_ast, $rule_order, $options{output_json}, %options);
        print STDERR "💾 Final AST serialized to: $options{output_json}\n" unless $quiet_mode;
    }
    
    # Always return Perl structures for direct consumption
    return ($final_ast, $rule_order);
}

sub serialize_final_ast_to_json {
    my ($final_ast, $rule_order, $output_file, %options) = @_;
    
    my $json_data = {
        format => "final_ast",
        version => "1.0",
        metadata => {
            generated_at => strftime("%Y-%m-%dT%H:%M:%SZ", gmtime()),
            generator => "AST::Transform::process_to_final_ast",
            source_file => $options{source_file} || "unknown",
            transformation_complete => JSON::PP::true,
            ready_for => ["code_generation", "data_generation", "analysis"]
        },
        final_ast => $final_ast,
        rule_order => $rule_order
    };
    
    my $json = JSON::PP->new;
    if ($options{pretty}) {
        $json->pretty->canonical;
    } else {
        $json->canonical;
    }
    
    open my $fh, '>', $output_file or die "Cannot write to $output_file: $!";
    print $fh $json->encode($json_data);
    close $fh;
}
```

### Step 2: Update Existing Code Generation (Backward Compatible)

```perl
sub process_transformation_phases {
    my ($input, %options) = @_;
    
    # Get final AST using new function
    my ($final_ast, $rule_order) = process_to_final_ast($input, %options);
    
    # Continue to step 6 (existing code generation)
    print STDERR "\n=== Step 6: Generate parser code ===\n" unless $options{quiet};
    my $step6_result = step6_generate_parser_code($final_ast, $rule_order);
    
    return $step6_result;
}
```

### Step 3: Create CLI Tool for JSON Export

`tools/ebnf_to_final_ast.pl`:

```perl
#!/usr/bin/perl
use strict;
use warnings;
use FindBin qw($RealBin);
use lib "$RealBin/../perl";
use AST::Transform qw(process_to_final_ast);
use Getopt::Long;

my $grammar_file = $ARGV[0] or die "Usage: $0 <grammar.ebnf> [options]\n";
die "Grammar file '$grammar_file' not found\n" unless -f $grammar_file;

GetOptions(
    'output-json|o=s' => \(my $output_json = ''),
    'pretty' => \(my $pretty = 0),
    'quiet|q' => \(my $quiet = 0),
    'help|h' => \&show_help,
) or die "Error in command line arguments\n";

# Read EBNF content
my $content = do {
    open my $fh, '<', $grammar_file or die "Cannot open $grammar_file: $!";
    local $/; <$fh>;
};

# Process to final AST
my ($final_ast, $rule_order) = process_to_final_ast($content, 
    output_json => $output_json,
    pretty => $pretty,
    quiet => $quiet,
    source_file => $grammar_file
);

print STDERR "🎉 Final AST processing complete!\n" unless $quiet;

sub show_help {
    print <<'EOF';
USAGE:
    ebnf_to_final_ast.pl <grammar.ebnf> [options]

REQUIRED:
    grammar.ebnf        EBNF grammar file to process

OPTIONS:
    --output-json, -o   Output final AST as JSON file
    --pretty            Pretty-print JSON output
    --quiet, -q         Suppress progress messages
    --help, -h          Show this help message

EXAMPLES:
    # Process to final AST (Perl structures only)
    ebnf_to_final_ast.pl json.ebnf
    
    # Export final AST as JSON
    ebnf_to_final_ast.pl json.ebnf --output-json json_final.json --pretty
    
    # Quiet mode
    ebnf_to_final_ast.pl json.ebnf -q -o final_ast.json
EOF
    exit 0;
}
```

### Step 4: Create DataGenerator Module

`perl/AST/DataGenerator.pm`:

```perl
package AST::DataGenerator;
use strict;
use warnings;
use JSON::PP;
use Exporter 'import';

our @EXPORT_OK = qw(generate_test_data generate_from_json);

sub generate_test_data {
    my ($final_ast, $rule_order, %options) = @_;
    
    # Generate test data from Perl AST structures
    # This is the high-performance path for Perl workflows
    
    # ... implementation ...
}

sub generate_from_json {
    my ($json_file, %options) = @_;
    
    # Load JSON final AST
    open my $fh, '<', $json_file or die "Cannot open $json_file: $!";
    my $json_content = do { local $/; <$fh> };
    close $fh;
    
    my $json_data = JSON::PP->new->decode($json_content);
    
    # Validate format
    die "Invalid final AST JSON format" unless $json_data->{format} eq 'final_ast';
    
    # Extract components
    my $final_ast = $json_data->{final_ast};
    my $rule_order = $json_data->{rule_order};
    
    # Generate using same logic as direct path
    return generate_test_data($final_ast, $rule_order, %options);
}

1;
```

### Step 5: Create Unified CLI for Data Generation

`tools/generate_test_data.pl`:

```perl
#!/usr/bin/perl
use strict;
use warnings;
use FindBin qw($RealBin);
use lib "$RealBin/../perl";
use AST::Transform qw(process_to_final_ast);
use AST::DataGenerator qw(generate_test_data generate_from_json);
use Getopt::Long;

my $input_file = $ARGV[0] or die "Usage: $0 <grammar.ebnf|final_ast.json> [options]\n";
die "Input file '$input_file' not found\n" unless -f $input_file;

GetOptions(
    'count=i' => \(my $count = 10),
    'output-dir=s' => \(my $output_dir = '.'),
    'max-depth=i' => \(my $max_depth = 5),
    'seed=i' => \(my $seed = time()),
    'quiet|q' => \(my $quiet = 0),
    'help|h' => \&show_help,
) or die "Error in command line arguments\n";

# Detect input type and choose path
if ($input_file =~ /\.ebnf$/i) {
    # Direct Perl path: EBNF → Final AST → Test Data (high performance)
    my $content = do {
        open my $fh, '<', $input_file or die "Cannot open $input_file: $!";
        local $/; <$fh>;
    };
    
    my ($final_ast, $rule_order) = process_to_final_ast($content, 
        quiet => $quiet,
        source_file => $input_file
    );
    
    my $test_data = generate_test_data($final_ast, $rule_order,
        count => $count,
        output_dir => $output_dir,
        max_depth => $max_depth,
        seed => $seed,
        quiet => $quiet
    );
    
} elsif ($input_file =~ /\.json$/i) {
    # JSON path: JSON → Test Data (multi-language integration)
    my $test_data = generate_from_json($input_file,
        count => $count,
        output_dir => $output_dir,
        max_depth => $max_depth,
        seed => $seed,
        quiet => $quiet
    );
    
} else {
    die "Unsupported input file type. Use .ebnf or .json\n";
}

print STDERR "🎉 Test data generation complete!\n" unless $quiet;
```

## Usage Examples

### Direct Perl Workflow (Fast)
```bash
# Single command: EBNF → Test Data
perl tools/generate_test_data.pl grammar.ebnf --count 100 --output-dir tests/
```

### Multi-Language Workflow (Flexible)
```bash
# Step 1: EBNF → Final AST JSON
perl tools/ebnf_to_final_ast.pl grammar.ebnf --output-json final.json --pretty

# Step 2: Any language can consume
perl tools/generate_test_data.pl final.json --count 100
python tools/analyze_grammar.py final.json
rust_tool generate_parser final.json --output parser.rs
```

### Pipeline Integration
```bash
# Export for external tools
perl tools/ebnf_to_final_ast.pl complex_grammar.ebnf -o final_ast.json

# Multiple consumers
parallel -j 4 ::: \
    "perl tools/generate_test_data.pl final_ast.json --count 1000" \
    "python tools/analyze_complexity.py final_ast.json" \
    "rust_generator final_ast.json --target wasm" \
    "go_generator final_ast.json --target native"
```

## Benefits Summary

### For Perl Development
- ✅ **High Performance**: Direct memory structures, no serialization overhead
- ✅ **Easy Debugging**: Native Perl data structures with Data::Dumper
- ✅ **Simple Integration**: Direct function calls between modules

### For Multi-Language Integration  
- ✅ **Universal Format**: JSON final AST can be consumed by any language
- ✅ **Persistence**: Can save and reuse final AST for multiple consumers
- ✅ **Distribution**: Can process on different machines/containers
- ✅ **Inspection**: Human-readable JSON for debugging and analysis

### For System Architecture
- ✅ **Flexibility**: Choose the right path for each use case
- ✅ **Future-Proof**: Easy to add new language consumers
- ✅ **Consistency**: Both paths use identical AST processing
- ✅ **Performance**: Optimal path selection based on needs

This hybrid approach gives us maximum flexibility while maintaining optimal performance for the primary Perl workflow.
