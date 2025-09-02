# Implementation Guide: Multi-Language AST Pipeline

## Quick Start for New Developers

### Prerequisites
- Git repository cloned: `git clone https://github.com/rdje/pgen`
- Programming language environments set up (Perl 5.20+, Rust 1.70+, Julia 1.8+, Go 1.19+, Python 3.8+)
- Basic understanding of AST transformations and parser generators

### Initial Setup
```bash
# Clone and explore
git clone https://github.com/rdje/pgen
cd pgen
find . -name "*.md" -exec head -5 {} \; # Read documentation
```

## Understanding the Pipeline

### Step 1: Examine Input/Output Examples
```bash
# Look at test grammars
ls test_grammars/
cat test_grammars/arithmetic.ebnf

# Look at expected outputs  
ls test_results/
cat test_results/arithmetic_raw.json
cat test_results/arithmetic_transformed.json
```

### Step 2: Run Existing Implementation
```bash
# Generate raw AST from EBNF
cd perl
perl ebnf_to_json.pl ../test_grammars/arithmetic.ebnf output_raw.json

# Transform raw AST (multiple language options)
cd ../rust && cargo run ../output_raw.json output_transformed.json
cd ../julia && julia ast_pipeline.jl ../output_raw.json output_transformed.json
cd ../go && go run ast_pipeline.go ../output_raw.json output_transformed.json
cd ../python && python ast_pipeline.py ../output_raw.json output_transformed.json
```

### Step 3: Study Core Data Structures

#### Raw AST Format
```json
{
  "grammar_name": "example",
  "raw_ast": [
    [
      ["rule", "rule_name"],
      ["semantic_annotation", "[\"name\", \"value\"]"],
      ["logging_annotation", "[\"name\", [\"arg1\", \"arg2\"]]"],
      ["identifier", "token"],
      ["operator", "|"],
      ["quoted_string", "literal"]
    ]
  ],
  "metadata": {"format": "raw_ast"}
}
```

#### Transformed AST Format
```json
{
  "grammar_name": "example",
  "grammar_tree": {
    "rule_name": {
      "type": "or|sequence|atom|quantified",
      "elements": [...],        
      "alternatives": [...],    
      "value": [...],          
      "element": {...},        
      "quantifier": "*|+|?"    
    }
  },
  "rule_order": ["rule1", "rule2"],
  "metadata": {
    "annotations": {
      "semantic_annotations": {"rule": ["name:value"]},
      "logging_annotations": {"rule": ["name(args)"]},
      "return_annotations": {"rule": "type"}
    }
  }
}
```

## Five-Stage Transformation Pipeline

### Stage 1: Extract Annotations
**Purpose**: Separate annotations from grammar elements, preserve metadata

**Input**: Raw AST array of token sequences
**Output**: Cleaned AST + populated annotations structure

**Key Logic**:
```
for each rule_definition in raw_ast:
    rule_name = find_rule_token(rule_definition)
    for each token in rule_definition:
        if token.type == "semantic_annotation":
            parse_annotation(token.value) → store in semantic_annotations[rule_name]
        elif token.type == "logging_annotation":
            parse_annotation(token.value) → store in logging_annotations[rule_name]  
        elif token.type == "return_*":
            store in return_annotations[rule_name]
        else:
            add to cleaned_rule
```

**Annotation Parsing**:
```
# Semantic: ['semantic_annotation', '["name", "value"]']
parsed = json_parse(token_value)  # → ["name", "value"]
formatted = f"{parsed[0]}:{parsed[1]}"  # → "name:value"

# Logging: ['logging_annotation', '["name", ["arg1", "arg2"]]']  
parsed = json_parse(token_value)  # → ["name", ["arg1", "arg2"]]
args = ",".join(parsed[1])       # → "arg1,arg2"
formatted = f"{parsed[0]}({args})" # → "name(arg1,arg2)"
```

### Stage 2: Group by OR Operators
**Purpose**: Split rule alternatives separated by "|" operators

**Input**: Cleaned AST token sequences
**Output**: Map of rule_name → list of alternative token sequences

### Stage 2.5: Handle Parentheses  
**Purpose**: Process grouping constructs (parentheses)

### Stage 3: Parse Sequences
**Purpose**: Convert token sequences to structured AST nodes

### Stage 4: Handle Quantifiers
**Purpose**: Process quantifier operators (*, +, ?)

### Stage 5: Build Tree Structure
**Purpose**: Assemble final grammar tree

## Language-Specific Implementation Notes

### Rust Implementation
- Use `serde` for JSON serialization
- Implement `enum ASTNode` for type safety
- Use `Result<T, E>` for error handling
- Memory management via ownership system

### Julia Implementation  
- Use `JSON3.jl` for JSON handling
- Define `struct` types for AST nodes
- Use multiple dispatch for type-specific operations

### Go Implementation
- Use `encoding/json` for JSON operations
- Define `struct` types with JSON tags
- Standard error handling with explicit checks

### Python Implementation
- Use standard `json` library
- Define classes with `dataclasses` or manual `__init__`
- Type hints for better documentation

### Zig Implementation
- Use `std.json` for JSON operations
- Define `union(enum)` for AST node variants
- Manual memory management with allocators

## Common Implementation Patterns

### Configuration Structure
```
struct Config {
    debug: bool = false,
    preserve_annotations: bool = true,
    validate_input: bool = true,
    validate_output: bool = true,
    max_recursion_depth: int = 100
}
```

### Error Handling Strategy
- Validate JSON structure on input
- Graceful fallbacks for malformed annotations
- Clear error messages with context
- Non-fatal warnings for recoverable issues

### Testing Strategy
1. **Unit tests**: Each transformation stage independently
2. **Integration tests**: Full pipeline with real grammar files  
3. **Edge case tests**: Empty grammars, malformed JSON, complex nesting
4. **Cross-language tests**: Same input produces equivalent output

## Extension Guidelines

### Adding New Language Implementation
1. Copy closest existing implementation as template
2. Implement the five core stages
3. Add JSON I/O handling with proper error handling  
4. Create comprehensive test suite
5. Add build system integration
6. Update PROJECT_OVERVIEW.md with new language status

### Adding New AST Node Type
1. Update the AST node type definitions in all languages
2. Add handling in relevant transformation stages
3. Update JSON serialization/deserialization
4. Add test cases covering the new node type
5. Update documentation with node type specification

This guide provides the implementation details needed for productive development and extension of the multi-language AST pipeline system.
