# EBNF Parser Generator - Complete Guide

This document provides comprehensive information for generating EBNF-based parsers that produce ASTs using the `tools/ast_transform.pl` system.

## Overview

The EBNF Parser Generator is a Perl-based system that transforms Extended Backus-Naur Form (EBNF) grammar files into working Perl parser modules (.pm files) that can parse input text and generate Abstract Syntax Trees (ASTs).

## Quick Start

1. Create an EBNF grammar file (`.ebnf`)
2. Run `perl tools/ast_transform.pl your_grammar.ebnf`
3. Use the generated `.pm` module in your code

## Core Tool: `tools/ast_transform.pl`

### Basic Usage
```bash
perl tools/ast_transform.pl <grammar_file.ebnf>
```

### Command Line Options

#### Required Arguments
- `<grammar_file.ebnf>`: Path to your EBNF grammar file

#### Optional Flags
- `--verbose` or `-v`: Enable verbose debug output showing transformation steps
- `--debug`: Enable detailed debugging information
- `--output <filename>`: Specify output filename (default: derived from grammar filename)

### Examples
```bash
# Basic generation
perl tools/ast_transform.pl my_grammar.ebnf

# With verbose output for debugging
perl tools/ast_transform.pl --verbose my_grammar.ebnf

# Specify custom output filename
perl tools/ast_transform.pl --output MyCustomParser.pm my_grammar.ebnf
```

### Output Files
The tool generates two files:
- `<GrammarName>.pm`: The parser module
- `<GrammarName>.pl`: A test script for the parser

## EBNF Grammar Syntax

### Basic Structure
```ebnf
# Comments start with #
rule_name := definition -> return_annotation
```

### Include System

The EBNF parser generator supports a powerful include system that allows you to split large grammars across multiple files and reuse common grammar components.

#### Include Directives

You can include other EBNF files using several directive forms:

```ebnf
# Include specific files
include_file("common_rules.ebnf", "operators.ebnf")
include("tokens.ebnf")  # Short form
file("literals")       # Will auto-add .ebnf extension

# Include all .ebnf files from directories
include_dir("../shared", "./components")
dir("common")  # Short form
```

#### File Extension Handling

File includes are flexible with extensions:
- `include("grammar")` → searches for `grammar.ebnf`
- `include("grammar.ebnf")` → searches for `grammar.ebnf`
- Both forms are equivalent

#### Directory Search Path

The system searches for included files in the following order:

1. **Base directory** (directory containing the main grammar file)
2. **Explicit directories** (from `include_dir()` directives)
3. **Environment directories**:
   - `$EBNF_INCLUDES` (colon-separated paths)
   - `$EBNFLIB` (colon-separated paths)
4. **Current directory** (fallback)

#### Environment Variables

Set up library paths using environment variables:

```bash
# Unix/Linux/macOS
export EBNF_INCLUDES="/usr/local/lib/ebnf:/opt/grammars"
export EBNFLIB="$HOME/.ebnf/lib:/shared/grammars"

# Windows
set EBNF_INCLUDES="C:\grammars\lib;D:\shared\ebnf"
set EBNFLIB="%USERPROFILE%\.ebnf\lib"
```

#### Recursive Includes

Included files can themselves contain include directives, creating a tree of grammar components:

```ebnf
# main.ebnf
include("tokens", "expressions", "statements")

program := statement* -> {type: "program", body: [$1*]}
```

```ebnf
# tokens.ebnf
include("common/whitespace")

identifier := /([a-zA-Z_]\w*)/ -> $1
number := /(\d+)/ -> $1
```

```ebnf
# expressions.ebnf
include("tokens")  # Can reference already included files

expression := term (("+" | "-") term)*
term := factor (("*" | "/") factor)*
factor := number | identifier | "(" expression ")"
```

#### Best Practices for Includes

1. **Organize by functionality**:
   ```
   grammars/
   ├── main.ebnf           # Top-level grammar
   ├── tokens/
   │   ├── identifiers.ebnf
   │   ├── literals.ebnf
   │   └── operators.ebnf
   ├── expressions/
   │   ├── arithmetic.ebnf
   │   └── boolean.ebnf
   └── statements/
       ├── control.ebnf
       └── declarations.ebnf
   ```

2. **Use environment variables for shared libraries**:
   ```bash
   export EBNFLIB="/usr/local/share/ebnf:/opt/parsers/common"
   ```

3. **Document include dependencies**:
   ```ebnf
   # arithmetic.ebnf
   # Requires: tokens/numbers.ebnf, tokens/operators.ebnf
   include("numbers", "operators")
   
   expression := term (additive_op term)*
   ```

4. **Avoid circular includes** - the system handles them gracefully but they indicate design issues

#### Example: Modular Language Grammar

```ebnf
# language.ebnf - Main grammar file
include_dir("tokens", "expressions", "statements")
include("whitespace")  # From EBNFLIB

program := statement* -> {type: "program", statements: [$1*]}
```

```ebnf
# tokens/identifiers.ebnf
identifier := /([a-zA-Z_]\w*)/ -> {type: "identifier", name: $1}
keyword := "if" | "while" | "function" | "return"
```

```ebnf
# expressions/arithmetic.ebnf
include("../tokens/identifiers", "../tokens/numbers")

expression := term (("+" | "-") term)*
          -> {type: "expression", left: $1, operations: [$2*]}
```

### Terminal Elements

#### String Literals
```ebnf
# Exact string match
keyword := "function"
operator := "+"
```

#### Regular Expressions
```ebnf
# Regex patterns (use capturing groups for return annotations)
identifier := /([a-zA-Z_][a-zA-Z0-9_]*)/
number := /(\d+)/
whitespace := /\s*/
```

#### Character Classes
```ebnf
letter := /[a-zA-Z]/
digit := /[0-9]/
```

### Non-Terminal Elements

#### Simple Rule References
```ebnf
expression := term
statement := expression
```

#### Sequences
```ebnf
# Space-separated elements in sequence
assignment := identifier "=" expression
function_call := identifier "(" argument_list ")"
```

#### Alternatives (OR)
```ebnf
# Pipe-separated alternatives
operator := "+" | "-" | "*" | "/"
statement := assignment | function_call | if_statement
```

#### Grouping
```ebnf
# Parentheses for grouping
expression := term (("+" | "-") term)*
```

### Quantifiers

#### Zero or More (`*`)
```ebnf
# Matches zero or more occurrences
parameter_list := parameter ("," parameter)*
statements := statement*
```

#### One or More (`+`)
```ebnf
# Matches one or more occurrences
digits := /[0-9]/+
```

#### Optional (`?`)
```ebnf
# Matches zero or one occurrence
return_type := (":" type)?
```

#### Specific Counts (`{n}`, `{n,m}`)
```ebnf
# Exactly n occurrences
hex_digit := /[0-9A-Fa-f]/{2}

# Between n and m occurrences
variable_args := parameter{1,5}
```

### Complex Patterns

#### Grouped Quantifiers
```ebnf
# Quantifiers applied to groups
array_elements := expression ("," expression)*
parameter_list := "(" (parameter ("," parameter)*)? ")"
```

#### Nested Structures
```ebnf
block := "{" statement* "}"
if_statement := "if" "(" expression ")" block ("else" block)?
```

## Return Annotations - The Power Feature

Return annotations transform parsing results into structured AST nodes. They follow the pattern `-> annotation` at the end of rules.

### Scalar References

#### Basic Captures
```ebnf
# $1 refers to first capturing group or first element
identifier := /([a-zA-Z_]\w*)/ -> $1
number := /(\d+)/ -> $1
```

#### Multiple Elements
```ebnf
# $1, $2, $3... refer to sequential elements
assignment := identifier "=" expression -> $3  # Returns the expression
binary_op := expression operator expression -> $2  # Returns the operator
```

### Array Construction

#### Simple Arrays
```ebnf
# Square brackets create arrays
parameter_list := param ("," param)* -> [$1, $2*]
# $1 = first param, $2* = all remaining params as array
```

#### Quantified Arrays
```ebnf
# Collect all matches into array
statements := statement* -> [$1*]
arguments := expression ("," expression)* -> [$1, $2*]
```

### Object Construction

#### Simple Objects
```ebnf
# Curly braces create hash/object structures
assignment := identifier "=" expression -> {name: $1, value: $3}
function_def := "function" identifier "(" params ")" block 
            -> {type: "function", name: $2, params: $4, body: $6}
```

#### Complex Objects
```ebnf
if_statement := "if" "(" condition ")" then_block ("else" else_block)?
            -> {
                type: "if_statement",
                condition: $3,
                then_branch: $5,
                else_branch: $7
            }
```

### Advanced Return Annotations

#### Dot Notation
```ebnf
# Access nested properties
method_call := object "." method "(" args ")"
           -> {
               type: "method_call",
               target: $1.name,
               method: $3,
               arguments: $5.items
           }
```

#### Array Slicing
```ebnf
# Python-style slicing
function_params := "(" param_list ")" -> $2[1:-1]  # Skip first and last
multiple_assigns := targets "=" values -> {targets: $1[::2], values: $3}
```

#### Advanced Nested Structures
```ebnf
class_def := "class" name ("extends" parent)? "{" members "}"
         -> {
             type: "class",
             name: $2,
             parent: $4,
             members: [$6*]
         }
```

### Quantifier-Specific Annotations

#### Handling Repeated Elements
```ebnf
# * quantifier - collect all matches
array_literal := "[" (element ("," element)*)? "]" 
             -> {type: "array", elements: [$2, $3*]}

# + quantifier - at least one
non_empty_list := element ("," element)+ -> [$1, $2*]

# ? quantifier - optional
typed_param := name (":" type)? -> {name: $1, type: $3}
```

### Real-World Examples

#### JSON Parser
```ebnf
json_value := string | number | boolean | null | object | array

object := "{" (pair ("," pair)*)? "}" 
       -> {type: "object", properties: [$2, $3*]}

array := "[" (json_value ("," json_value)*)? "]"
      -> {type: "array", elements: [$2, $3*]}

pair := string ":" json_value -> {key: $1, value: $3}

string := /"([^"\\]|\\.)*"/ -> $1
number := /(-?\d+(\.\d+)?([eE][+-]?\d+)?)/ -> $1
boolean := "true" | "false" -> $1
null := "null" -> null
```

#### Expression Parser
```ebnf
expression := term (("+" | "-") term)*
          -> {left: $1, operations: [$2*]}

term := factor (("*" | "/") factor)*
    -> {left: $1, operations: [$2*]}

factor := number | "(" expression ")" -> $2
number := /(\d+)/ -> {type: "number", value: $1}
```

## Generated Parser Usage

### Basic Parser Structure
```perl
use MyGeneratedParser;

# Create parser instance (if constructor provided)
my $parser = MyGeneratedParser->new();

# Parse input
my $result = $parser->parse_start_rule($input_string);

# Or call parsing functions directly
my $result = MyGeneratedParser::parse_start_rule($input_string);
```

### Error Handling
```perl
my $result = eval { parse_my_grammar($input) };
if ($@) {
    print "Parse error: $@\n";
} else {
    # Process successful result
    print "Parsed: ", Dumper($result), "\n";
}
```

### Working with Results
```perl
use Data::Dumper;

my $ast = parse_expression("2 + 3 * 4");
print Dumper($ast);

# Expected output (example):
# {
#   'op' => '+',
#   'left' => { 'type' => 'number', 'value' => '2' },
#   'right' => {
#     'op' => '*',
#     'left' => { 'type' => 'number', 'value' => '3' },
#     'right' => { 'type' => 'number', 'value' => '4' }
#   }
# }
```

## Advanced Features

### Left Recursion Handling
The system automatically eliminates left recursion:

```ebnf
# This left-recursive grammar:
expression := expression "+" term | term

# Is automatically transformed to:
expression := term expression_prime
expression_prime := "+" term expression_prime | ε
```

### Error Recovery
Generated parsers include basic error recovery and reporting.

### Performance Considerations
- Use specific regex patterns rather than overly general ones
- Minimize deep nesting in return annotations
- Consider rule ordering for performance (most common alternatives first)

## Troubleshooting

### Common Issues

#### 1. Regex Patterns Not Matching
```ebnf
# Wrong - no capturing groups
identifier := /[a-zA-Z]\w*/

# Correct - with capturing groups for return annotations
identifier := /([a-zA-Z]\w*)/ -> $1
```

#### 2. Return Annotation Errors
```ebnf
# Wrong - referring to non-existent captures
rule := "keyword" -> $2  # Only one element, no $2

# Correct
rule := "keyword" -> $1
```

#### 3. Quantifier Issues
```ebnf
# Wrong - quantifier on wrong element
list := (item ","*)  # Quantifies comma, not the pair

# Correct
list := item ("," item)*  # Quantifies the pair
```

### Debug Techniques

#### 1. Use Verbose Mode
```bash
perl tools/ast_transform.pl --verbose my_grammar.ebnf
```

#### 2. Test Incrementally
Start with simple rules and gradually add complexity.

#### 3. Check Generated Code
Examine the generated `.pm` file for issues.

## Directory Structure

After generation, organize your files:
```
project/
├── grammars/           # Your .ebnf files
├── parsers/            # Generated .pm files  
├── tests/              # Test scripts and data
└── tools/
    └── ast_transform.pl # The generator tool
```

## Best Practices

### 1. Grammar Design
- Start with a minimal grammar and expand
- Use descriptive rule names
- Group related rules together
- Comment complex patterns

### 2. Return Annotations
- Design AST structure before writing grammar
- Use consistent object structures
- Prefer explicit over implicit captures
- Test return annotations early

### 3. Testing
- Create comprehensive test cases
- Test edge cases and error conditions
- Validate AST structure matches expectations
- Use the generated test script as starting point

### 4. Maintenance
- Version control your grammar files
- Document grammar changes
- Keep generated parsers in separate directory
- Regenerate parsers after grammar changes

## Example: Complete Mini Language

Here's a complete example showing a simple programming language grammar:

```ebnf
# Simple programming language grammar

program := statement* -> {type: "program", statements: [$1*]}

statement := assignment | if_statement | while_loop | expression_statement

assignment := identifier "=" expression ";" 
          -> {type: "assignment", target: $1, value: $3}

if_statement := "if" "(" expression ")" block ("else" block)?
            -> {
                type: "if",
                condition: $3,
                then_branch: $5,
                else_branch: $7
            }

while_loop := "while" "(" expression ")" block
          -> {type: "while", condition: $3, body: $5}

expression_statement := expression ";" -> $1

block := "{" statement* "}" -> {type: "block", statements: [$2*]}

expression := term (("+" | "-") term)*
          -> {type: "expression", left: $1, operations: [$2*]}

term := factor (("*" | "/") factor)*
    -> {type: "term", left: $1, operations: [$2*]}

factor := number | identifier | "(" expression ")" -> $2

identifier := /([a-zA-Z_]\w*)/ -> {type: "identifier", name: $1}
number := /(\d+)/ -> {type: "number", value: $1}
```

This grammar generates a parser that can handle programs like:
```
x = 42;
if (x > 10) {
    y = x * 2;
    while (y > 0) {
        y = y - 1;
    }
}
```

And produces a structured AST with all the semantic information needed for further processing.

## Conclusion

This EBNF Parser Generator provides a powerful, flexible way to create parsers that produce rich ASTs. The return annotation system allows you to shape your AST exactly as needed, while the EBNF syntax provides familiar, readable grammar definitions.

Key strengths:
- Intuitive EBNF syntax
- Powerful return annotation system
- Automatic left recursion elimination
- Generated Perl modules ready to use
- Comprehensive debugging support

Start simple, test frequently, and gradually build up to complex grammars. The system is designed to handle real-world parsing tasks efficiently and maintainably.
