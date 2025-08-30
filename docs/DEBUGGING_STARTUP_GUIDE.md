# Debugging Startup Guide for AST Transformation Pipeline

## Current Status
The AST transformation pipeline has been refactored to handle `*` quantifiers in sequences, but there's a recurring Perl runtime error: "Can't use string ("-> [1, 2, 3]") as a SCALAR ref while "strict refs" in use at MyParser.pm line 1666."

## Project Structure
```
perl/AST/Transform.pm          # Core AST transformation pipeline
perl/AST/LeftRecursion.pm      # Left recursion elimination
perl/Parser/ReturnAnnotation.pm # Return annotation parser wrapper
perl/Parser/ReturnAnnotationGenerated.pm # Generated return annotation parser
tools/ast_transform.pl         # CLI wrapper for AST::Transform
legacy/grammars/merged_ultimate_return_annotation.ebnf # Input EBNF grammar
```

## Key Files and Their Purpose

### 1. Core Transformation Module: `perl/AST/Transform.pm`
- **Purpose**: Converts EBNF AST into Perl parser code
- **Key Functions**:
  - `generate_sequence_parser`: Handles sequences with quantifiers (including `*`)
  - `generate_grouped_quantifier_sequence_loop`: Generates loops for `(X)*` patterns
  - `generate_fast_parser_sub`: Main parser generation function
  - `generate_parser_module`: Creates the complete parser module

### 2. Input Grammar: `legacy/grammars/merged_ultimate_return_annotation.ebnf`
- **Purpose**: EBNF grammar for return annotations with full dot notation support
- **Key Rules**:
  - `array_contents := return_expression (',' /\s*/ return_expression)*`
  - `return_expression := ...` (supports nested arrays/objects)

### 3. CLI Wrapper: `tools/ast_transform.pl`
- **Purpose**: Command-line interface to the transformation pipeline
- **Usage**: `perl tools/ast_transform.pl --input <ebnf_file> --output <output_dir>`

## How to Run AST Transform

### Basic Command
```bash
perl tools/ast_transform.pl --input legacy/grammars/merged_ultimate_return_annotation.ebnf --output .
```

### Command Options
- `--input` or `-i`: Input EBNF file
- `--output` or `-o`: Output directory
- `--package`: Custom package name (optional, defaults to filename-based name)

### What It Generates
1. **`.pm` file**: Perl module with parser functions
2. **`.pl` file**: Wrapper script for command-line usage

## Current Issue Details

### Error Description
```
Can't use string ("-> [1, 2, 3]") as a SCALAR ref while "strict refs" in use at MyParser.pm line 1666.
```

### Root Cause
The `MyParser::parse()` function expects a scalar reference (`\$input`) but is receiving a string value.

### Test Command That Fails
```bash
perl -e "use lib '.'; use MyParser; my \$input = '-> [1, 2, 3]'; my \$result = MyParser::parse(\$input); print \"Parse result: \"; use Data::Dumper; print Dumper(\$result);"
```

### Expected Test Command
```bash
perl -e "use lib '.'; use MyParser; my \$input = '-> [1, 2, 3]'; my \$result = MyParser::parse(\$input); print \"Parse result: \"; use Data::Dumper; print Dumper(\$result);"
```

## Recent Changes Made

### 1. Refactored `generate_sequence_parser`
- Added detection for grouped quantifiers ending with `(X)*`
- Implemented `generate_grouped_quantifier_sequence_loop` function
- Generates `while (1)` loops for repeated elements

### 2. Generated Parser Structure
The `parse_array_contents` function now correctly generates:
```perl
sub parse_array_contents {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();

    # Parse first required element
    my $result_1 = parse_return_expression($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;

    # Parse optional repeated grouped elements in loop
    while (1) {
        # Try to match the grouped pattern: (',' /\s*/ return_expression)
        my $loop_start_pos = pos($$input);

        # Match comma
        unless ($$input =~ /\G,/gc) { last; }

        # Match optional whitespace
        $$input =~ /\G\s*/gc;

        # Match return_expression
        my $loop_result = parse_return_expression($input);
        unless (defined $loop_result) {
            # Restore position and exit loop
            pos($$input) = $loop_start_pos;
            last;
        }

        # Successfully matched the group, add to results
        push @results, $loop_result;
    }

    return \@results;
}
```

## Debugging Steps to Take

### 1. Verify Generated Parser
```bash
# Check if parser was generated correctly
ls -la MyParser.pm MyParser.pl
```

### 2. Test Basic Parsing
```bash
# Test with simple input first
perl -e "use lib '.'; use MyParser; my \$input = '-> 42'; my \$result = MyParser::parse(\$input); print \"Result: \"; use Data::Dumper; print Dumper(\$result);"
```

### 3. Test Array Parsing
```bash
# Test with array input (this is currently failing)
perl -e "use lib '.'; use MyParser; my \$input = '-> [1, 2, 3]'; my \$result = MyParser::parse(\$input); print \"Result: \"; use Data::Dumper; print Dumper(\$result);"
```

### 4. Check Function Signature
Look at the generated `MyParser::parse` function to see what parameter type it expects.

## Key Functions to Examine

### In `perl/AST/Transform.pm`
1. `generate_sequence_parser` (lines ~1180-1250)
2. `generate_grouped_quantifier_sequence_loop` (lines ~1250-1300)
3. `generate_fast_parser_sub` (lines ~800-900)

### In Generated `MyParser.pm`
1. `parse` function (main entry point)
2. `parse_array_contents` function (handles comma-separated lists)
3. `parse_return_expression` function (parses individual elements)

## Expected Behavior
- `-> 42` should parse successfully
- `-> [42]` should parse successfully  
- `-> [1, 2, 3]` should parse successfully and return an array reference

## Current Problem
The parser generation is working (correct loop structure is generated), but there's a runtime issue with how the input is being passed to the `parse` function.

## Next Steps for Debugging
1. Verify the generated parser structure
2. Check the `parse` function signature
3. Fix the input parameter handling
4. Test the full parsing flow

## Dependencies
- Perl 5
- `LinkedSpec.pm` (for EBNF parsing)
- `Parser::ReturnAnnotationGenerated.pm` (for return annotation parsing)

## Working Directory
All commands should be run from `/Users/richarddje/Downloads/AFX/fsm/afx/cursor`
