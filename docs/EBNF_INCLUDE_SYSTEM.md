# EBNF Include System - Technical Reference

This document provides comprehensive technical details about the include system in the EBNF Parser Generator.

## Overview

The EBNF include system allows you to:
- Split large grammars across multiple files
- Create reusable grammar libraries
- Organize grammar components by functionality
- Maintain complex grammar projects efficiently

## Include Directives

### File Includes

#### Syntax
```ebnf
# Full forms
include_file("file1.ebnf", "file2.ebnf", "file3")
file("grammar1", "grammar2.ebnf")

# Short form (most common)
include("tokens", "expressions", "statements")
```

#### Behavior
- Includes specific files by name
- Automatically adds `.ebnf` extension if not present
- Multiple files can be specified in a single directive
- Files are processed in the order listed

#### Examples
```ebnf
# These are all equivalent:
include("common")
include("common.ebnf")
include_file("common")
file("common.ebnf")

# Multiple includes
include("tokens", "operators", "expressions")
```

### Directory Includes

#### Syntax
```ebnf
# Full forms
include_dir("directory1", "directory2", "../shared")
dir("components", "./local")
```

#### Behavior
- Includes ALL `.ebnf` files from specified directories
- Uses `*.ebnf` pattern automatically
- Processes files in alphabetical order
- Searches relative to the search path

#### Examples
```ebnf
# Include all .ebnf files from these directories
include_dir("tokens", "expressions", "statements")
dir("../shared", "./components")
```

## File Resolution

### Search Algorithm

The system searches for included files using the following algorithm:

1. **Direct path check**: If the path is absolute, use it directly
2. **Search path traversal**: For relative paths, search through the search path in order:
   - Base directory (containing the main grammar file)
   - Explicit include directories (from `include_dir()` directives)
   - Environment include directories (`$EBNF_INCLUDES`, `$EBNFLIB`)
   - Current working directory (fallback)

### Search Path Construction

```perl
# Pseudo-code for search path construction:
@search_path = (
    $base_directory,                    # Directory of main grammar file
    @explicit_directories,              # From include_dir() directives
    split(':', $ENV{EBNF_INCLUDES}),   # Environment variable
    split(':', $ENV{EBNFLIB}),         # Environment variable
    '.'                                # Current directory fallback
);
```

### File Extension Handling

```perl
# Extension logic:
sub resolve_filename {
    my $filename = shift;
    
    # Add .ebnf if no extension present
    $filename .= '.ebnf' unless $filename =~ /\.ebnf$/;
    
    return $filename;
}
```

## Environment Variables

### `EBNF_INCLUDES`

Primary environment variable for EBNF library paths.

```bash
# Unix/Linux/macOS
export EBNF_INCLUDES="/usr/local/lib/ebnf:/opt/grammars:/home/user/.ebnf"

# Windows
set EBNF_INCLUDES="C:\grammars\lib;D:\shared\ebnf;%USERPROFILE%\.ebnf"
```

### `EBNFLIB`

Alternative environment variable (for compatibility/convenience).

```bash
# Can be used instead of or in addition to EBNF_INCLUDES
export EBNFLIB="/shared/grammars:/usr/local/share/ebnf"
```

### Platform Differences

- **Unix/Linux/macOS**: Uses colon (`:`) as path separator
- **Windows**: Uses semicolon (`;`) as path separator
- The system automatically detects the platform and uses the correct separator

## Recursive Processing

### How It Works

1. **Initial parsing**: Main grammar file is parsed to extract includes and rules
2. **Include resolution**: Each include directive is resolved to actual file paths
3. **Recursive parsing**: Each included file is parsed and processed for its own includes
4. **Cycle detection**: The system prevents infinite recursion from circular includes
5. **Rule aggregation**: All rules from all files are combined into a single grammar

### Example Flow

```
main.ebnf
├─ include("tokens", "expressions")
│   ├─ tokens.ebnf
│   │  └─ include("common/whitespace")
│   │     └─ common/whitespace.ebnf
│   └─ expressions.ebnf
│      └─ include("tokens")  # Already processed, skipped
└─ [final combined grammar]
```

### Circular Include Handling

The system handles circular includes gracefully:

```ebnf
# file1.ebnf
include("file2")
rule1 := "a"

# file2.ebnf  
include("file1")  # Circular reference
rule2 := "b"
```

Result: Both files are processed once, all rules are included.

## Implementation Details

### AST Structure

Includes are parsed into AST nodes:

```perl
# include_file directive produces:
["include_file", ["file1", "file2", "file3"]]

# include_dir directive produces: 
["include_dir", ["dir1", "dir2", "dir3"]]
```

### Processing Function

The main processing function `process_ast_includes` handles:

```perl
sub process_ast_includes {
    my ($raw_ast, $base_dir, $include_dirs) = @_;
    
    # 1. Build comprehensive search path
    # 2. Separate includes from rules in AST
    # 3. Process each include directive
    # 4. Recursively process included files
    # 5. Combine all rules
    
    return \@all_rules;
}
```

### Error Handling

The system provides detailed error reporting:

- **File not found**: Lists searched directories
- **Parse errors**: Shows file path and error details
- **Circular includes**: Warns but continues processing
- **Permission errors**: Reports access issues

## Best Practices

### 1. Organization Strategies

#### Functional Organization
```
grammars/
├── main.ebnf
├── tokens/
│   ├── keywords.ebnf
│   ├── operators.ebnf
│   └── literals.ebnf
├── expressions/
│   ├── arithmetic.ebnf
│   ├── logical.ebnf
│   └── assignment.ebnf
└── statements/
    ├── control_flow.ebnf
    ├── declarations.ebnf
    └── blocks.ebnf
```

#### Layered Organization
```
grammars/
├── main.ebnf          # Top level
├── syntax/
│   ├── expressions.ebnf
│   └── statements.ebnf
├── lexical/
│   ├── tokens.ebnf
│   └── whitespace.ebnf
└── common/
    ├── numbers.ebnf
    └── strings.ebnf
```

### 2. Environment Setup

#### Development Environment
```bash
# .bashrc or .zshrc
export EBNFLIB="$HOME/.ebnf/lib:$HOME/projects/grammars/common"
export EBNF_INCLUDES="/usr/local/share/ebnf:$EBNFLIB"
```

#### Project-Specific Setup
```bash
# project/.env
export EBNF_INCLUDES="./grammars/common:./grammars/shared:$EBNF_INCLUDES"
```

### 3. Documentation Conventions

#### File Headers
```ebnf
# tokens/operators.ebnf
# Binary and unary operators for arithmetic expressions
# Dependencies: none
# Used by: expressions/arithmetic.ebnf, expressions/assignment.ebnf

plus_op := "+" -> "plus"
minus_op := "-" -> "minus"
```

#### Dependency Comments
```ebnf
# expressions/arithmetic.ebnf
# Requires: tokens/operators.ebnf, tokens/numbers.ebnf
include("operators", "numbers")

expression := term (additive_op term)*
```

### 4. Testing Strategies

#### Isolated Testing
Test individual grammar files in isolation:

```bash
# Test tokens separately
perl tools/ast_transform.pl grammars/tokens/operators.ebnf

# Test expressions with tokens
perl tools/ast_transform.pl grammars/expressions/arithmetic.ebnf
```

#### Integration Testing
Test the complete grammar with all includes:

```bash
# Test full grammar
perl tools/ast_transform.pl grammars/main.ebnf --verbose
```

## Common Patterns

### 1. Base + Extensions

```ebnf
# base.ebnf - Core language features
include_dir("core")
program := statement* -> {type: "program", body: [$1*]}

# extensions/objects.ebnf - Object-oriented extension
include("../base")
class_def := "class" identifier "{" member* "}"
```

### 2. Layered Includes

```ebnf
# main.ebnf
include("syntax")  # High-level syntax

# syntax.ebnf
include("expressions", "statements")  # Mid-level constructs

# expressions.ebnf  
include("tokens")  # Low-level tokens
```

### 3. Conditional Includes

While not directly supported, you can simulate conditional includes:

```ebnf
# main.ebnf
include_dir("base")
# Manually choose extensions based on target language
# include("extensions/objects")      # For OOP version
# include("extensions/functional")   # For functional version
```

## Troubleshooting

### Common Issues

#### 1. File Not Found
```
Error: Cannot open included file grammar.ebnf: No such file or directory
```

**Solutions:**
- Check file path and spelling
- Verify file exists in search directories
- Add directory to `EBNF_INCLUDES`
- Use absolute path for testing

#### 2. Circular Dependencies
```
Warning: Circular include detected: file1.ebnf -> file2.ebnf -> file1.ebnf
```

**Solutions:**
- Redesign grammar structure
- Extract common rules to separate file
- Use more specific includes

#### 3. Parse Errors in Included Files
```
Error in included file tokens.ebnf: Unexpected token at line 15
```

**Solutions:**
- Test included file independently
- Check for syntax errors
- Verify rule completeness

### Debugging Techniques

#### 1. Verbose Mode
```bash
perl tools/ast_transform.pl --verbose main.ebnf
```

Shows detailed include processing steps.

#### 2. Path Debugging
Add temporary debugging to see search paths:

```bash
# Check environment
echo $EBNF_INCLUDES
echo $EBNFLIB

# Test file resolution manually
find $EBNF_INCLUDES -name "*.ebnf" | head -10
```

#### 3. Incremental Inclusion
Start with minimal includes and add gradually:

```ebnf
# Start with this
include("tokens")

# Then add
include("tokens", "operators")  

# Finally
include("tokens", "operators", "expressions")
```

## Performance Considerations

### File System Impact
- Each include directive causes file system access
- Directory includes scan entire directories
- Large numbers of includes can slow down parsing

### Optimization Strategies
1. **Group related rules**: Fewer, larger files vs. many small files
2. **Minimize directory includes**: Use specific file includes when possible
3. **Cache parsed results**: Consider implementing parsed grammar caching

### Memory Usage
- All included rules are loaded into memory
- Large grammar projects may require significant RAM
- Consider splitting very large projects if memory becomes an issue

## Advanced Features

### Environment Variable Expansion

The system supports environment variable expansion in paths:

```bash
export GRAMMAR_ROOT="/opt/grammars"
export EBNF_INCLUDES="$GRAMMAR_ROOT/common:$GRAMMAR_ROOT/extensions"
```

### Cross-Platform Compatibility

Handles platform-specific path conventions:

```perl
# Automatic platform detection
my $separator = ($^O eq 'MSWin32') ? ';' : ':';
my @paths = split /$separator/, $ENV{EBNF_INCLUDES};
```

### Future Enhancements

Planned features:
- **Namespace support**: Avoid rule name conflicts
- **Version constraints**: Require specific versions of included grammars  
- **Conditional includes**: Include based on feature flags
- **Remote includes**: Include grammars from URLs
- **Compressed includes**: Support for `.ebnf.gz` files

## API Reference

### Core Functions

#### `process_ast_includes($raw_ast, $base_dir, $include_dirs)`
Main include processing function.

**Parameters:**
- `$raw_ast`: Raw AST from EBNF parser
- `$base_dir`: Base directory for relative path resolution
- `$include_dirs`: Additional search directories

**Returns:**
- `\@all_rules`: Array reference containing all rules from all files

#### `resolve_include_files($file_spec, $include_dirs)`
Resolves file include specifications to actual file paths.

#### `resolve_include_directory($dir_spec, $pattern, $include_dirs)`
Resolves directory includes to lists of matching files.

#### `load_and_process_included_file($file_path, $include_dirs)`
Loads and recursively processes a single included file.

### Internal Functions

#### `extract_includes_and_rules($raw_ast)`
Separates include directives from grammar rules in the AST.

#### `process_ast_includes_from_content($content, $include_dirs)`  
Processes includes from EBNF content string.

## Conclusion

The EBNF include system provides powerful capabilities for organizing and managing complex grammar projects. By understanding its features and following best practices, you can build maintainable, modular grammars that scale to real-world parsing requirements.

Key benefits:
- **Modularity**: Split grammars into logical components
- **Reusability**: Share common grammar elements across projects
- **Maintainability**: Easier to update and debug smaller files
- **Scalability**: Handle large grammar projects efficiently
- **Collaboration**: Multiple developers can work on different grammar components

The system is designed to be robust, flexible, and developer-friendly while handling the complexities of file resolution, recursive processing, and cross-platform compatibility transparently.
