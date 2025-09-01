# EBNF Data Generator Architecture

## Overview

This document outlines the architecture for generating pseudo-random test data from EBNF grammar specifications, with support for probability annotations and controlled recursion depth.

## System Goals

1. **Automated Test Generation**: Create valid test files from any EBNF grammar
2. **Probability Support**: Honor `@prob` annotations or use uniform distribution
3. **Recursion Control**: Intelligent depth limiting for recursive grammars
4. **Round-Trip Validation**: Generate → Parse → Validate consistency

## CLI Interface Design

### Generate Test Data Tool

**Usage**: `perl tools/generate_test_data.pl <grammar.ebnf> [options]`

**Examples**:
```bash
# Basic usage - grammar file is required positional argument
perl tools/generate_test_data.pl json.ebnf --count 100 --output tests/json/

# Multiple options
perl tools/generate_test_data.pl expr.ebnf --count 50 --max-depth 3 --seed 12345

# Simple case - defaults for everything except the required grammar
perl tools/generate_test_data.pl simple.ebnf
```

### Validation Tool

**Usage**: `perl tools/validate_grammar.pl <grammar.ebnf> [options]`

**Examples**:
```bash
# Full validation suite - grammar is positional, required
perl tools/validate_grammar.pl complex.ebnf --test-count 1000

# Performance benchmarking
perl tools/validate_grammar.pl hdl.ebnf --benchmark --iterations 5000

# Quick validation with defaults
perl tools/validate_grammar.pl test.ebnf
```

## Architecture Components

### 1. Grammar Analyzer (`EBNFAnalyzer.pm`)

**Purpose**: Parse EBNF grammar and extract generation metadata

**Key Functions**:
```perl
sub analyze_grammar($ebnf_file) -> $grammar_metadata
sub extract_probabilities($rule) -> %probability_weights  
sub identify_recursion_patterns($grammar) -> @recursive_rules
sub calculate_default_weights($alternatives) -> %weights
```

**Probability Handling**:
```ebnf
# Explicit probabilities
rule := option_a @30 | option_b @70     # 30%, 70%

# Mixed explicit/implicit
rule := option_x @50 | option_y | option_z   # 50%, 25%, 25%

# Uniform distribution (default)
rule := option_a | option_b | option_c       # 33.33% each
```

### 2. Data Generator (`EBNFDataGenerator.pm`)

**Purpose**: Generate pseudo-random text conforming to grammar

**Key Functions**:
```perl
sub generate_text($grammar_metadata, %options) -> $generated_text
sub generate_rule($rule_name, $depth, $max_depth) -> $text_fragment
sub select_alternative($alternatives, %weights) -> $chosen_alternative
sub generate_quantified($pattern, $quantifier, $depth) -> $text_list
```

**Generation Options**:
```perl
my %options = (
    max_depth => 5,           # Maximum recursion depth
    seed => 12345,            # Reproducible randomness
    min_quantifier => 0,      # Minimum for * and +
    max_quantifier => 5,      # Maximum for * and +
    prefer_shorter => 0.7,    # Bias towards shorter generations
);
```

### 3. CLI Argument Processing

**For generate_test_data.pl**:
```perl
# Command line parsing
use Getopt::Long;

my $grammar_file = $ARGV[0] or die "Usage: $0 <grammar.ebnf> [options]\n";
die "Grammar file '$grammar_file' not found\n" unless -f $grammar_file;
die "Grammar file must have .ebnf extension\n" unless $grammar_file =~ /\.ebnf$/;

# Process options with defaults
GetOptions(
    'count=i' => \(my $count = 10),
    'output=s' => \(my $output_dir = '.'),
    'max-depth=i' => \(my $max_depth = 5),
    'seed=i' => \(my $seed = time()),
    'prefix=s' => \(my $prefix = 'test'),
    'help|h' => \&show_help,
) or die "Error in command line arguments\n";

sub show_help {
    print <<'EOF';
USAGE:
    generate_test_data.pl <grammar.ebnf> [options]

REQUIRED:
    grammar.ebnf    EBNF grammar file to generate test data from

OPTIONS:
    --count N       Number of test files to generate (default: 10)
    --output DIR    Output directory (default: current directory)
    --max-depth N   Maximum recursion depth (default: 5)
    --seed N        Random seed for reproducible generation (default: current time)
    --prefix STR    Prefix for generated files (default: 'test')
    --help, -h      Show this help message

EXAMPLES:
    generate_test_data.pl json.ebnf --count 100
    generate_test_data.pl expr.ebnf --count 50 --max-depth 3 --output tests/
    generate_test_data.pl simple.ebnf
EOF
    exit 0;
}
```

**For validate_grammar.pl**:
```perl
# Command line parsing
use Getopt::Long;

my $grammar_file = $ARGV[0] or die "Usage: $0 <grammar.ebnf> [options]\n";
die "Grammar file '$grammar_file' not found\n" unless -f $grammar_file;
die "Grammar file must have .ebnf extension\n" unless $grammar_file =~ /\.ebnf$/;

GetOptions(
    'test-count=i' => \(my $test_count = 100),
    'input-dir=s' => \(my $input_dir = '.'),
    'benchmark' => \(my $benchmark = 0),
    'iterations=i' => \(my $iterations = 1000),
    'full-report' => \(my $full_report = 0),
    'output-dir=s' => \(my $report_dir = '.'),
    'help|h' => \&show_help,
) or die "Error in command line arguments\n";

sub show_help {
    print <<'EOF';
USAGE:
    validate_grammar.pl <grammar.ebnf> [options]

REQUIRED:
    grammar.ebnf    EBNF grammar file to validate

OPTIONS:
    --test-count N      Number of test files to generate and validate (default: 100)
    --input-dir DIR     Directory containing existing test files to validate
    --benchmark         Run performance benchmarking
    --iterations N      Number of iterations for benchmarking (default: 1000)
    --full-report       Generate comprehensive validation report
    --output-dir DIR    Output directory for reports (default: current directory)
    --help, -h          Show this help message

EXAMPLES:
    validate_grammar.pl json.ebnf --test-count 500
    validate_grammar.pl hdl.ebnf --benchmark --iterations 5000
    validate_grammar.pl complex.ebnf --full-report --output-dir results/
EOF
    exit 0;
}
```

## Implementation Plan

### Phase 1: Core Generator

**Files to Create**:
- `perl/EBNFAnalyzer.pm` - Grammar analysis and metadata extraction
- `perl/EBNFDataGenerator.pm` - Core text generation engine
- `perl/RecursionController.pm` - Depth and cycle management
- `tools/generate_test_data.pl` - CLI interface with positional grammar argument

**Updated Sample Usage**:
```bash
# Generate 100 test files from JSON grammar - grammar is positional
perl tools/generate_test_data.pl json.ebnf --count 100 --output tests/json/

# Generate with specific parameters - grammar comes first
perl tools/generate_test_data.pl expr.ebnf --count 50 --max-depth 3 --seed 12345

# Simplest case - just provide the grammar file
perl tools/generate_test_data.pl simple.ebnf
```

### Phase 2: Validation Framework

**Files to Create**:
- `perl/ValidationPipeline.pm` - Round-trip testing coordinator
- `perl/TestReporter.pm` - Results analysis and reporting
- `tools/validate_grammar.pl` - Full validation CLI with positional grammar argument

**Updated Sample Usage**:
```bash
# Full validation suite - grammar is positional
perl tools/validate_grammar.pl complex.ebnf --test-count 1000

# Performance benchmarking - grammar comes first
perl tools/validate_grammar.pl hdl.ebnf --benchmark --iterations 5000

# Quick validation with defaults - just the grammar
perl tools/validate_grammar.pl test.ebnf
```

## Error Handling for Positional Arguments

```perl
# Robust argument validation
sub validate_grammar_file {
    my ($file) = @_;
    
    die "Usage: $0 <grammar.ebnf> [options]\n" unless defined $file;
    die "Grammar file '$file' not found\n" unless -f $file;
    die "Grammar file '$file' is not readable\n" unless -r $file;
    die "Grammar file must have .ebnf extension\n" unless $file =~ /\.ebnf$/i;
    
    # Additional validation
    die "Grammar file '$file' is empty\n" unless -s $file;
    
    return $file;
}
```

## Output File Naming

**For generate_test_data.pl**:
```perl
sub generate_output_filename {
    my ($prefix, $index, $grammar_file) = @_;
    
    # Extract base name from grammar file
    my ($base) = $grammar_file =~ /([^\/\\]+?)\.ebnf$/i;
    
    # Generate filename: test_json_001.txt, test_json_002.txt, etc.
    return sprintf("%s_%s_%03d.txt", $prefix, $base, $index);
}

# Example outputs:
# json.ebnf → test_json_001.txt, test_json_002.txt, ...
# math_expr.ebnf → test_math_expr_001.txt, test_math_expr_002.txt, ...
```

## Integration Examples

### 1. JSON Grammar Testing
```bash
# Generate JSON test files - grammar is positional, required
perl tools/generate_test_data.pl json.ebnf --count 500 --max-depth 4

# Validate all generated files parse correctly
perl tools/validate_grammar.pl json.ebnf --input-dir tests/json/
```

### 2. Expression Grammar Benchmarking
```bash
# Generate mathematical expressions - simple positional usage
perl tools/generate_test_data.pl math_expr.ebnf --count 10000 --max-depth 6

# Performance test - grammar file is first argument
perl tools/validate_grammar.pl math_expr.ebnf --benchmark --iterations 10000
```

### 3. HDL Grammar Validation
```bash
# Generate VHDL test cases - grammar file required
perl tools/generate_test_data.pl vhdl.ebnf --count 200 --prefer-shorter 0.8

# Comprehensive validation - positional grammar argument
perl tools/validate_grammar.pl vhdl.ebnf --full-report --output-dir results/
```

## CLI Help Integration

Both tools will show proper usage when called incorrectly:

```bash
# Missing grammar file
$ perl tools/generate_test_data.pl
Usage: generate_test_data.pl <grammar.ebnf> [options]

# Help flag
$ perl tools/generate_test_data.pl --help
USAGE:
    generate_test_data.pl <grammar.ebnf> [options]

REQUIRED:
    grammar.ebnf    EBNF grammar file to generate test data from
    
# ... rest of help text
```

This design makes the tools much more intuitive to use since the EBNF grammar file is always the first, required argument, followed by optional configuration flags.
