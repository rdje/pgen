# Parser Generation Tools

This document describes the available tools for EBNF grammar processing and parser generation. These are the **ONLY** tools that should be used in the current architecture.

## Overview

The parser generation workflow follows this pipeline:

```
EBNF Grammar → [ebnf_to_json.pl] → Raw AST JSON → [perl_parser_gen] → Perl Parser (.pm/.pl)
```

## Available Tools

### 1. tools/ebnf_to_json.pl

**Purpose**: Converts EBNF grammar files to JSON Raw AST format for language-agnostic processing.

**Location**: `tools/ebnf_to_json.pl`

#### Usage

```bash
# Basic usage
perl tools/ebnf_to_json.pl <grammar.ebnf>

# With options
perl tools/ebnf_to_json.pl [options] <grammar.ebnf>
```

#### Positional Arguments

- `<grammar.ebnf>` - Input EBNF grammar file (required)

#### Options

| Option | Short | Type | Description |
|--------|-------|------|-------------|
| `--output` | `-o` | file | Write JSON to file instead of STDOUT |
| `--pretty` | | flag | Pretty-print JSON output with indentation |
| `--validate-only` | | flag | Only validate grammar, don't generate JSON |
| `--quiet` | `-q` | flag | Suppress progress messages |
| `--verbosity` | `-v` | level | Set verbosity: `normal`, `full`, `debug` |
| `--help` | `-h` | flag | Show help message |

#### Examples

```bash
# Generate JSON raw AST to STDOUT
perl tools/ebnf_to_json.pl json.ebnf

# Generate pretty-printed JSON to file
perl tools/ebnf_to_json.pl --pretty json.ebnf -o json_raw_ast.json

# Validate grammar only
perl tools/ebnf_to_json.pl --validate-only json.ebnf

# Quiet mode with output file
perl tools/ebnf_to_json.pl -q json.ebnf -o json.json
```

#### Output Format

The tool generates JSON with this structure:

```json
{
    "grammar_name": "json",
    "raw_ast": [
        // Raw EBNF parser tokens - exact output from ebnf.spec parser
        [["rule", "json"], ["quoted_string", "value"], ["operator", "|"], ...]
    ],
    "metadata": {
        "source_file": "json.ebnf",
        "generated_at": "2024-01-01T12:00:00Z",
        "generator": "ebnf_to_json.pl",
        "format": "raw_ast",
        "description": "Direct output from EBNF parser before transformations"
    }
}
```

#### Important Notes

- **Stops immediately** after EBNF parsing - no transformations applied
- Preserves exact token-level structure from EBNF parser
- Maximum flexibility for language-specific optimizations
- Universal interchange format for all language generators

---

### 2. tools/generators/perl_parser_gen

**Purpose**: Generates Perl parser modules (.pm/.pl files) from JSON Raw AST.

**Location**: `tools/generators/perl_parser_gen`

#### Usage

```bash
# From file
perl tools/generators/perl_parser_gen [options] [json_file]

# From pipeline
perl tools/ebnf_to_json.pl grammar.ebnf | perl tools/generators/perl_parser_gen [options]

# From STDIN
perl tools/generators/perl_parser_gen [options] -
```

#### Positional Arguments

- `[json_file]` - Input JSON file containing raw AST (optional, defaults to STDIN)
- `-` - Explicitly read from STDIN

#### Options

| Option | Short | Type | Description |
|--------|-------|------|-------------|
| `--output` | `-o` | basename | Write parser files as `basename.pm` and `basename.pl` |
| `--package` | `-p` | name | Set Perl package name (default: derived from grammar) |
| `--quiet` | `-q` | flag | Suppress progress messages |
| `--verbosity` | `-v` | level | Set verbosity: `normal`, `full`, `debug` |
| `--bootstrap-mode` | | flag | Enable bootstrap mode for parser regeneration |
| `--help` | `-h` | flag | Show help message |

#### Examples

```bash
# Generate from JSON file
perl tools/generators/perl_parser_gen json_raw_ast.json -o json_parser

# Generate from pipeline
perl tools/ebnf_to_json.pl json.ebnf | perl tools/generators/perl_parser_gen -o json_parser

# Custom package name
perl tools/generators/perl_parser_gen input.json -p MyParser -o my_parser

# Module only to STDOUT
perl tools/ebnf_to_json.pl simple.ebnf | perl tools/generators/perl_parser_gen

# From STDIN explicitly
cat input.json | perl tools/generators/perl_parser_gen -o parser
```

#### Output Files

When using `--output basename`, generates:

1. **`basename.pm`** - Perl module containing:
   - Optimized parsing functions
   - Pre-compiled regex patterns
   - Main `parse()` entry point
   - Runtime helper functions

2. **`basename.pl`** - Executable wrapper script with:
   - Command-line interface
   - File input/output handling
   - Pretty-printing options
   - Error reporting

#### Package Name Resolution

The Perl package name is determined in this order:

1. **Explicit**: `--package MyPackage`
2. **From output file**: `--output my_parser` → `my_parser`
3. **From JSON filename**: `input.json` → `input`
4. **From grammar name**: JSON `grammar_name` field

#### Bootstrap Mode

Use `--bootstrap-mode` when regenerating parsers to avoid self-hosting cycles:

```bash
# Regenerating the return annotation parser
perl tools/generators/perl_parser_gen --bootstrap-mode return_ast.json -o new_parser
```

#### Transformation Pipeline

The tool runs the complete 5-step transformation pipeline:

1. **Step 2**: Group by OR operators
2. **Step 2.5**: Handle parentheses grouping  
3. **Step 3**: Parse sequences
4. **Step 4**: Handle quantifiers
5. **Step 5**: Build tree structure
6. **Step 6**: Generate parser code

---

## Complete Workflow Examples

### Example 1: JSON Grammar Parser

```bash
# Step 1: Convert EBNF to JSON Raw AST
perl tools/ebnf_to_json.pl grammars/json.ebnf -o json_raw_ast.json

# Step 2: Generate Perl parser
perl tools/generators/perl_parser_gen json_raw_ast.json -o json_parser

# Result: json_parser.pm and json_parser.pl
```

### Example 2: Pipeline Mode

```bash
# Single pipeline command
perl tools/ebnf_to_json.pl grammars/calculator.ebnf | \
  perl tools/generators/perl_parser_gen -o calculator_parser
```

### Example 3: Custom Package Name

```bash
perl tools/ebnf_to_json.pl grammars/sql.ebnf | \
  perl tools/generators/perl_parser_gen -p SQLParser -o sql_parser
```

### Example 4: Testing Generated Parser

```bash
# Generate parser
perl tools/ebnf_to_json.pl grammars/json.ebnf | \
  perl tools/generators/perl_parser_gen -o json_parser

# Use generated parser
echo '{"key": "value"}' | perl json_parser.pl --pretty
```

---

## Deprecated Tools

### ⚠️ tools/ast_transform.pl - DO NOT USE

**Status**: Deprecated, reference only

This tool is kept for reference but **should not be used** in the current workflow. It represents the old monolithic architecture that has been replaced by the JSON-based pipeline.

**Why deprecated**:
- Monolithic Perl-specific design
- Not language-agnostic
- Difficult to maintain and extend
- Superseded by the cleaner JSON pipeline

---

## Architecture Benefits

### JSON-Based Pipeline

The current architecture provides:

1. **Language Independence**: Raw AST JSON can be consumed by generators for any target language
2. **Separation of Concerns**: EBNF parsing separated from language-specific transformations  
3. **Flexibility**: Each language generator can implement optimizations specific to its target
4. **Testability**: Each step can be tested independently
5. **Maintainability**: Clear boundaries between components

### Transformation Pipeline

The 5-step transformation pipeline is implemented natively in each language generator:

1. **Raw AST**: Direct EBNF parser output (ebnf_to_json.pl)
2. **OR Grouping**: Collect rule alternatives 
3. **Parentheses**: Handle grouping and nesting
4. **Sequences**: Parse element sequences
5. **Quantifiers**: Handle *, +, ?, {n,m}
6. **Tree Structure**: Build semantic grammar tree
7. **Code Generation**: Generate target language parser

---

## Error Handling

Both tools provide comprehensive error reporting:

### Verbosity Levels

- `normal`: Standard progress messages
- `full`: Detailed error summaries  
- `debug`: Complete debugging output

### Error Context

Tools track and report:
- Parse errors with line/column information
- Transformation failures with rule context
- Validation warnings and suggestions
- Performance metrics in debug mode

### Exit Codes

- `0`: Success
- `1`: Error (with detailed error output to STDERR)

---

## Best Practices

### File Organization

```
project/
├── grammars/           # .ebnf source files
├── generated/          # Generated parsers
│   ├── json_parser.pm
│   ├── json_parser.pl
│   └── *.json         # Intermediate JSON files
└── test/              # Test inputs
```

### Workflow Integration

```bash
# Development cycle
make_parser() {
    local grammar=$1
    local name=$2
    
    perl tools/ebnf_to_json.pl "grammars/${grammar}.ebnf" -o "generated/${name}_raw_ast.json"
    perl tools/generators/perl_parser_gen "generated/${name}_raw_ast.json" -o "generated/${name}_parser"
    
    echo "Generated: generated/${name}_parser.pm and generated/${name}_parser.pl"
}

# Usage: make_parser json json
```

### Testing

```bash
# Validate grammar
perl tools/ebnf_to_json.pl --validate-only grammars/new_grammar.ebnf

# Test generated parser
echo "test input" | perl generated/parser.pl --pretty
```

---

## Troubleshooting

### Common Issues

1. **File not found**: Check paths relative to project root
2. **JSON parsing errors**: Ensure ebnf_to_json.pl completed successfully  
3. **Package name conflicts**: Use `--package` to specify unique names
4. **Bootstrap cycles**: Use `--bootstrap-mode` when regenerating core parsers

### Debug Mode

Use `--verbosity debug` for detailed transformation tracing:

```bash
perl tools/generators/perl_parser_gen --verbosity debug input.json -o parser
```

This provides step-by-step transformation details and intermediate AST structures.

---

## Future Extensions

The JSON-based architecture enables future language generators:

- `tools/generators/python_parser_gen` - Python parser generation
- `tools/generators/rust_parser_gen` - Rust parser generation  
- `tools/generators/go_parser_gen` - Go parser generation

Each generator can implement language-specific optimizations while consuming the same Raw AST JSON format.
