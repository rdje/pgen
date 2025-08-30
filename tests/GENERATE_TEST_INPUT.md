# LinkedSpec Test Input Generator Documentation

## Overview

The `generate_test_input.pl` script generates pseudo-random input files based on `.spec` file analysis for testing LinkedSpec parsers. It creates hierarchical, grammar-driven test inputs that match the expected structure of the target parser.

## Purpose and Benefits

### Why Use This Generator?

1. **Comprehensive Testing**: Generates diverse test cases covering various input structures
2. **Grammar-Driven**: Uses actual `.spec` file analysis to understand parser expectations
3. **Hierarchical Generation**: Creates nested structures that test parser recursion
4. **Configurable**: Fine-grained control over generation parameters
5. **Automated Testing**: Reduces manual test case creation effort

### Key Features

- **Spec File Analysis**: Parses `.spec` files to understand grammar structure
- **Hierarchical Generation**: Creates nested data structures with controlled depth
- **Content Type Variety**: Generates numbers, strings, identifiers, comments, and structures
- **Configurable Parameters**: Control depth, items per level, and content distribution
- **Multiple Output Formats**: Generate single files or multiple samples

## Installation

### Prerequisites

- Perl 5.10 or higher
- Standard Perl modules: `Data::Dumper`, `Getopt::Long`

### Setup

1. Ensure the script is executable:
   ```bash
   chmod +x generate_test_input.pl
   ```

2. Verify dependencies:
   ```bash
   perl -e "use Data::Dumper; use Getopt::Long; print 'Dependencies OK\n'"
   ```

## Usage

### Basic Usage

```bash
# Generate 3 samples using default settings
perl generate_test_input.pl specs/valid/basic.spec

# Generate to output file
perl generate_test_input.pl -o test_input.txt specs/valid/basic.spec

# Generate with verbose output
perl generate_test_input.pl -v specs/valid/basic.spec
```

### Advanced Usage

```bash
# Control structure generation
perl generate_test_input.pl -d 4 -min 2 -max 4 -p 0.7 specs/valid/basic.spec

# Control content type distribution
perl generate_test_input.pl -pn 0.4 -pd 0.3 -pi 0.2 specs/valid/basic.spec

# Generate multiple samples with specific controls
perl generate_test_input.pl -n 5 -d 3 -p 0.5 -o complex_tests.txt specs/valid/basic.spec
```

## Command-Line Options

### Structure Controls

| Option | Description | Default |
|--------|-------------|---------|
| `-d, --max-depth N` | Maximum nesting depth | 5 |
| `-min, --min-items N` | Minimum items per level | 1 |
| `-max, --max-items N` | Maximum items per level | 5 |
| `-p, --nesting-prob F` | Probability of nested vs terminal | 0.6 |

### Content Type Controls

| Option | Description | Default |
|--------|-------------|---------|
| `-pn, --prob-number F` | Probability of numbers | 0.2 |
| `-pd, --prob-dquotes F` | Probability of double-quoted strings | 0.2 |
| `-ps, --prob-squotes F` | Probability of single-quoted strings | 0.1 |
| `-pi, --prob-identifier F` | Probability of identifiers | 0.3 |
| `-pc, --prob-comment F` | Probability of comments | 0.1 |
| `-ps, --prob-simple F` | Probability of simple structures | 0.1 |

### General Options

| Option | Description | Default |
|--------|-------------|---------|
| `-v, --verbose` | Enable verbose output | Off |
| `-o, --output FILE` | Output file | stdout |
| `-n, --samples N` | Number of samples to generate | 3 |
| `-h, --help` | Show help message | - |

## Examples

### Example 1: Basic Generation

```bash
perl generate_test_input.pl specs/valid/basic.spec
```

**Output:**
```
=== Generated Test Inputs ===
(test_data
 (simple_value 123)
 (nested_data
  (inner_value "hello world")
 ))
---
(value
 (sample "CDEtpE")
 (setting
  (sample simple_value))
 "Tf8gS"
 account)
```

### Example 2: High Nesting Probability

```bash
perl generate_test_input.pl -p 0.8 -d 4 specs/valid/basic.spec
```

**Output:**
```
(data
 (config
  (simple_value 'kJMJHHCyh')
  (world
   (account ; for)
   (test_data "c3CJJf9klBIFJcg"))
  (inner_value
   'iCz2n7'
   "OpWhluLeEAGAG"))
 config
 ; testing)
```

### Example 3: Content Type Control

```bash
perl generate_test_input.pl -pn 0.4 -pd 0.3 -pi 0.2 specs/valid/basic.spec
```

**Output:**
```
(setting
 (value
  (inner_value "kC8p0Pc69dDohP")
  "VPntzViDB2F"
  (hello
   'qGc67YLWtlE1x'
   (inner_value "EpRyDrTvm")))
 (simple_value
  (inner_value
   (item 717))
  "7c9xy2NA")
 "ykcago2qPuZl")
```

## Architecture

### Core Components

1. **Spec File Parser**: Analyzes `.spec` files to extract grammar rules
2. **Grammar Analyzer**: Identifies patterns, actions, and dependencies
3. **Input Generator**: Creates hierarchical structures based on grammar
4. **Content Generator**: Generates various content types (numbers, strings, etc.)

### Generation Process

1. **Load Spec File**: Read and parse the `.spec` file
2. **Analyze Grammar**: Extract rules, patterns, and actions
3. **Determine Structure**: Decide on nesting depth and item counts
4. **Generate Content**: Create appropriate content for each item
5. **Format Output**: Apply proper formatting and indentation

## How It Works

### Step-by-Step Process

1. **Spec File Loading**
   ```perl
   my $spec_content = load_spec_file($spec_file);
   my $grammar = analyze_spec_grammar($spec_content);
   ```

2. **Grammar Analysis**
   - Extract top-level rule (e.g., `Lispish::`)
   - Parse rule definitions and patterns
   - Identify action blocks and dependencies

3. **Structure Generation**
   ```perl
   sub generate_lispish_input {
       my ($grammar, $depth, $max_depth) = @_;
       # Generate hierarchical structure with controlled parameters
   }
   ```

4. **Content Type Selection**
   ```perl
   sub generate_terminal_item {
       # Choose content type based on probabilities
       my $content_type = int(rand(6));
       # Generate appropriate content
   }
   ```

### Grammar Analysis

The generator analyzes `.spec` files to understand:

- **Top-level rules**: Entry points for parsing
- **Pattern definitions**: Regex patterns for matching
- **Action blocks**: Code executed during parsing
- **Dependencies**: Relationships between rules

### Input Generation Strategies

#### Hierarchical Structure Generation

```perl
# Level-specific controls
my %LEVEL_CONTROLS = (
    1 => { min_items => 2, max_items => 4, nesting_prob => 0.7 },
    2 => { min_items => 1, max_items => 3, nesting_prob => 0.5 },
    3 => { min_items => 1, max_items => 2, nesting_prob => 0.3 },
    4 => { min_items => 1, max_items => 1, nesting_prob => 0.1 },
    5 => { min_items => 1, max_items => 1, nesting_prob => 0.0 }
);
```

#### Content Type Distribution

```perl
# Weighted content selection
my $content_weights = [0.1, 0.2, 0.1, 0.4, 0.1, 0.1];
# [numbers, dquotes, squotes, identifiers, comments, simple_structs]
```

### Hierarchical Structure Generation

The generator creates nested structures with:

- **Controlled Depth**: Maximum nesting level specified by user
- **Variable Items**: Random number of items per level
- **Mixed Content**: Combination of nested structures and terminal items
- **Proper Formatting**: Correct indentation and structure

### Supported Grammar Types

#### Lispish Grammar

Supports the full Lispish grammar including:
- **Parentheses** `()`: Nested structures
- **Square brackets** `[]`: Array/list structures  
- **Curly braces** `{}`: Code blocks (recursive)
- **Double quotes** `"..."`: String literals
- **Single quotes** `'...'`: String literals
- **Comments** `;...`: Comment blocks
- **Identifiers**: Variable names and symbols

#### Basic Grammar

For simpler parsers like `basic.spec`:
- **Parentheses** `()`: Nested structures
- **Double quotes** `"..."`: String literals
- **Comments** `;...`: Comment blocks
- **Identifiers**: Variable names and symbols
- **Numbers**: Integer literals

## Configuration

### Default Settings

```perl
# Structure defaults
my $MAX_DEPTH = 5;
my $MIN_ITEMS_PER_LEVEL = 1;
my $MAX_ITEMS_PER_LEVEL = 5;
my $NESTING_PROBABILITY = 0.6;

# Content type defaults
my $PROB_NUMBER = 0.2;
my $PROB_DQUOTES = 0.2;
my $PROB_SQUOTES = 0.1;
my $PROB_IDENTIFIER = 0.3;
my $PROB_COMMENT = 0.1;
my $PROB_SIMPLE_STRUCT = 0.1;
```

### Custom Configuration

You can modify the script to add custom configurations:

```perl
# Add custom level controls
my %CUSTOM_LEVEL_CONTROLS = (
    1 => { min_items => 3, max_items => 6, nesting_prob => 0.8 },
    2 => { min_items => 2, max_items => 4, nesting_prob => 0.6 },
    # ... more levels
);
```

## Integration with Test Framework

### Test Workflow

1. **Generate Test Inputs**
   ```bash
   perl generate_test_input.pl -o test_inputs.txt specs/valid/basic.spec
   ```

2. **Run Parser Tests**
   ```bash
   perl run_parser.pl specs/valid/basic.spec test_inputs.txt
   ```

3. **Verify Results**
   ```bash
   # Check parser output and verify structure
   ```

### Automated Testing

```bash
#!/bin/bash
# Generate and test workflow
perl generate_test_input.pl -n 10 -o test_suite.txt specs/valid/basic.spec
perl run_parser.pl specs/valid/basic.spec test_suite.txt
```

## Troubleshooting

### Common Issues

#### 1. Spec File Not Found
```
Error: Cannot open spec file: specs/valid/basic.spec - No such file or directory
```
**Solution**: Verify the spec file path and ensure it exists.

#### 2. Empty Output
```
=== Generated Test Inputs ===
(empty output)
```
**Solution**: Check that the spec file contains valid grammar rules.

#### 3. Invalid Structure
```
Parse failed - returned undef
```
**Solution**: Verify the generated input matches the parser's expectations.

### Debugging

#### Enable Verbose Output
```bash
perl generate_test_input.pl -v specs/valid/basic.spec
```

#### Check Grammar Analysis
```bash
perl generate_test_input.pl -v specs/valid/basic.spec 2>&1 | grep "Spec Analysis"
```

#### Test Individual Components
```bash
# Test spec file loading
perl -e "use Data::Dumper; print Dumper(load_spec_file('specs/valid/basic.spec'))"

# Test grammar analysis
perl -e "print Dumper(analyze_spec_grammar(\$content))"
```

## Future Enhancements

### Planned Features

1. **Grammar-Specific Generators**: Specialized generators for different grammar types
2. **Template-Based Generation**: Use templates for more realistic test data
3. **Constraint-Based Generation**: Generate inputs that satisfy specific constraints
4. **Performance Testing**: Generate large inputs for performance testing
5. **Error Injection**: Generate inputs that test error handling

### API Reference

#### Main Functions

```perl
# Load and analyze spec file
load_spec_file($filename)
analyze_spec_grammar($content)

# Generate input
generate_input_from_grammar($grammar, $max_depth)
generate_lispish_input($grammar, $depth, $max_depth)

# Generate content
generate_terminal_item($grammar, $depth)
generate_random_string($min_len, $max_len)
generate_random_identifier()
generate_random_number()
```

#### Configuration Functions

```perl
# Set generation parameters
set_depth_controls($max_depth, $min_items, $max_items)
set_content_weights($weights_array)
set_nesting_probability($probability)
```

## Advanced Control Features

### Level-Specific Controls

The generator provides fine-grained control over each nesting level:

```perl
my %LEVEL_CONTROLS = (
    1 => { 
        min_items => 2, 
        max_items => 4, 
        nesting_prob => 0.7, 
        content_weights => [0.1, 0.2, 0.1, 0.4, 0.1, 0.1] 
    },
    2 => { 
        min_items => 1, 
        max_items => 3, 
        nesting_prob => 0.5, 
        content_weights => [0.2, 0.2, 0.1, 0.3, 0.1, 0.1] 
    },
    # ... more levels
);
```

### Content Type Distribution

Control the probability of different content types:

```bash
# Generate with more numbers and fewer strings
perl generate_test_input.pl -pn 0.4 -pd 0.1 -ps 0.05 specs/valid/basic.spec

# Generate with more identifiers
perl generate_test_input.pl -pi 0.5 -pn 0.1 -pd 0.1 specs/valid/basic.spec
```

### Structure Control Examples

#### High Nesting Probability
```bash
perl generate_test_input.pl -p 0.8 -d 4 specs/valid/basic.spec
```
**Result**: Deep, complex nested structures

#### Low Nesting Probability
```bash
perl generate_test_input.pl -p 0.2 -d 4 specs/valid/basic.spec
```
**Result**: Shallow, mostly flat structures

#### Controlled Item Counts
```bash
perl generate_test_input.pl -min 3 -max 5 specs/valid/basic.spec
```
**Result**: Each level has 3-5 items

## Advanced Usage Scenarios

### 1. Stress Testing
```bash
# Generate deep, complex structures
perl generate_test_input.pl -d 6 -p 0.9 -n 20 -o stress_tests.txt specs/valid/basic.spec
```

### 2. Edge Case Testing
```bash
# Generate minimal structures
perl generate_test_input.pl -d 1 -min 1 -max 1 -o edge_cases.txt specs/valid/basic.spec
```

### 3. Content-Specific Testing
```bash
# Generate mostly string content
perl generate_test_input.pl -pd 0.5 -ps 0.3 -pi 0.1 -pn 0.1 specs/valid/basic.spec

# Generate mostly numeric content
perl generate_test_input.pl -pn 0.6 -pi 0.2 -pd 0.1 -ps 0.1 specs/valid/basic.spec
```

### 4. Performance Testing
```bash
# Generate large test suite
perl generate_test_input.pl -n 100 -d 4 -o performance_tests.txt specs/valid/basic.spec
```

## Best Practices

### 1. Start Simple
Begin with default settings and gradually increase complexity:
```bash
# Start with defaults
perl generate_test_input.pl specs/valid/basic.spec

# Then add complexity
perl generate_test_input.pl -d 3 -p 0.5 specs/valid/basic.spec
```

### 2. Test Incrementally
Test generated inputs before using them in large test suites:
```bash
# Generate small test set first
perl generate_test_input.pl -n 3 -o test_sample.txt specs/valid/basic.spec

# Verify it works
perl run_parser.pl specs/valid/basic.spec test_sample.txt

# Then generate full suite
perl generate_test_input.pl -n 50 -o full_suite.txt specs/valid/basic.spec
```

### 3. Use Appropriate Controls
Match generation parameters to your testing needs:
- **Unit Testing**: Low depth, simple structures
- **Integration Testing**: Medium depth, mixed content
- **Stress Testing**: High depth, complex structures

### 4. Document Your Settings
Keep track of successful generation parameters:
```bash
# Save successful configuration
echo "perl generate_test_input.pl -d 4 -p 0.6 -pn 0.2 -pd 0.3 specs/valid/basic.spec" > successful_config.sh
```

## Conclusion

The LinkedSpec Test Input Generator provides powerful, flexible tools for creating comprehensive test suites. By understanding the grammar structure and using appropriate controls, you can generate test inputs that thoroughly exercise your parsers and help ensure robust, reliable parsing behavior.

**Note:** This documentation is maintained alongside the script. When the script is updated, this documentation should be updated to reflect the changes.
