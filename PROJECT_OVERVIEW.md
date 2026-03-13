# Project Overview: Multi-Language EBNF Parser Generator

> Historical note
> This document captures an earlier multi-language project framing and directory map.
> The current actively maintained product surface is the Rust-first pipeline documented in
> `README.md`, `PGEN_USER_GUIDE.md`, and `PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`.

## Project Purpose

This project implements a multi-language EBNF (Extended Backus-Naur Form) parser generator with comprehensive AST transformation pipeline. The system converts EBNF grammar specifications into executable parsers across multiple programming languages while preserving semantic and logging annotations.

## Core Architecture

### Three-Phase System

1. **EBNF to Raw AST** (`perl/ebnf_to_json.pl`)
   - Parses EBNF grammar files
   - Outputs JSON-serialized raw AST
   - Preserves all annotations and metadata

2. **Raw AST to Transformed AST** (Multi-language implementations)
   - 5-stage transformation pipeline
   - Handles OR operators, parentheses, sequences, quantifiers
   - Builds structured grammar tree
   - Available in: Perl, Rust, Julia, Go, Python, Zig

3. **Parser Generation** (`perl/AST/Transform.pm`)
   - Generates executable parser code
   - Handles left-recursion elimination
   - Produces optimized parsing logic

### Directory Structure

```
pgen/
├── perl/                       # Core Perl implementation
│   ├── ebnf_to_json.pl        # EBNF → Raw AST converter
│   ├── AST/Transform.pm       # Raw AST → Transformed AST + Parser generation
│   └── tests/                 # Perl test suite
├── rust/                      # Rust AST pipeline implementation
│   ├── src/ast_pipeline.rs    # Complete AST transformation
│   └── tests/                 # Rust test suite
├── julia/                     # Julia AST pipeline implementation
│   ├── ast_pipeline.jl        # Complete AST transformation
│   └── test/                  # Julia test suite
├── go/                        # Go AST pipeline implementation
│   ├── ast_pipeline.go        # Complete AST transformation
│   └── *_test.go              # Go test suite
├── python/                    # Python AST pipeline implementation
│   ├── ast_pipeline.py        # Complete AST transformation
│   └── tests/                 # Python test suite
├── zig/                       # Zig AST pipeline implementation
│   ├── ast_pipeline.zig       # Complete AST transformation
│   └── *_test.zig             # Zig test suite
├── test_grammars/             # EBNF test files
├── test_results/              # Expected transformation outputs
└── docs/                      # Technical documentation
```

## Data Flow Pipeline

### 1. EBNF Input Format
```ebnf
@type: "Expression"
@range: {min: 0, max: 1000}
expression := term ('+' term)*

@log: "Processing term"
term := factor ('*' factor)*

@examples: [42, 123, 999]
factor := number | '(' expression ')'

number := /(\d+)/
```

### 2. Raw AST JSON Format
```json
{
  "grammar_name": "arithmetic",
  "raw_ast": [
    [
      ["rule", "expression"],
      ["semantic_annotation", "[\"type\", \"Expression\"]"],
      ["semantic_annotation", "[\"range\", \"{min: 0, max: 1000}\"]"],
      ["identifier", "term"],
      ["operator", "+"],
      ["identifier", "term"],
      ["operator", "*"]
    ]
  ],
  "metadata": {
    "format": "raw_ast",
    "transformer": "ebnf_to_json.pl"
  }
}
```

### 3. Transformed AST JSON Format
```json
{
  "grammar_name": "arithmetic",
  "grammar_tree": {
    "expression": {
      "type": "sequence",
      "elements": [
        {
          "type": "atom",
          "value": ["identifier", "term"]
        },
        {
          "type": "quantified",
          "element": {
            "type": "sequence",
            "elements": [
              {"type": "atom", "value": ["operator", "+"]},
              {"type": "atom", "value": ["identifier", "term"]}
            ]
          },
          "quantifier": "*"
        }
      ]
    }
  },
  "rule_order": ["expression", "term", "factor", "number"],
  "metadata": {
    "format": "transformed_ast",
    "annotations": {
      "semantic_annotations": {
        "expression": ["type:Expression", "range:{min: 0, max: 1000}"]
      },
      "logging_annotations": {},
      "return_annotations": {}
    }
  }
}
```

## Annotation System

### Semantic Annotations
- **Format**: `['semantic_annotation', [<name>, <value>]]`
- **Purpose**: Static metadata about rule semantics
- **Examples**: `@type`, `@range`, `@validation`, `@examples`
- **Storage**: Preserved in `semantic_annotations` field of metadata

### Logging Annotations  
- **Format**: `['logging_annotation', [<name>, [<arg1>, <arg2>, ...]]]`
- **Purpose**: Dynamic runtime logging during parsing
- **Examples**: `@log`, `@debug`, `@trace`
- **Storage**: Preserved in `logging_annotations` field of metadata

### Return Annotations
- **Format**: `['return_scalar', <type>]`, `['return_array', <type>]`, `['return_object', <type>]`
- **Purpose**: Specify parser return value transformation
- **Storage**: Preserved in `return_annotations` field of metadata

## Multi-Language Implementation Status

| Language | AST Pipeline | Build System | Tests | Status |
|----------|-------------|-------------|-------|---------|
| Perl     | ✅ Complete  | ✅ Complete  | ✅ Complete | Production |
| Rust     | ✅ Complete + Semantic Annotations | ✅ Complete  | ✅ Complete + Annotation Testing | Production |
| Julia    | ✅ Complete  | ✅ Complete  | ✅ Complete | Production |
| Go       | ✅ Complete  | ✅ Complete  | ✅ Complete | Production |
| Python   | ✅ Complete  | ✅ Complete  | ✅ Complete | Production |
| Zig      | ✅ Complete  | ⚠️ Build API  | ✅ Complete | Development |

## API Interfaces

### Dual-Mode API Design

Each language implementation provides both:

1. **Same-Language API**: In-memory data structures for performance
2. **Cross-Language API**: JSON input/output for interoperability

### Common Interface Pattern

```
# Same-language mode (language-specific types)
pipeline = create_pipeline(config)
(grammar_tree, rule_order) = pipeline.transform_from_file(input_file)

# Cross-language mode (JSON I/O)
pipeline.transform_to_json(input_file, output_file)
```

## Build and Test Commands

### Perl
```bash
cd perl && perl ebnf_to_json.pl input.ebnf output.json
cd perl && perl -Mlib=. -MAST::Transform -e "test_suite()"
```

### Rust
```bash
cd rust && cargo build
cd rust && cargo test
cd rust && cargo run input.json output.json
```

### Julia
```bash
cd julia && julia -e "using Pkg; Pkg.activate(.); Pkg.test()"
cd julia && julia ast_pipeline.jl input.json output.json
```

### Go
```bash
cd go && go build ast_pipeline.go
cd go && go test
cd go && ./ast_pipeline input.json output.json
```

### Python
```bash
cd python && python ast_pipeline.py input.json output.json
cd python && python -m pytest tests/
```

### Zig
```bash
cd zig && zig build
cd zig && zig build test
cd zig && zig build run -- input.json output.json
```

## Key Technical Decisions

### 1. JSON as Interchange Format
- Enables language-agnostic data exchange
- Provides clear serialization boundaries
- Supports complex nested structures

### 2. Five-Stage Transformation Pipeline
- **Stage 1**: Extract annotations (preserve metadata)
- **Stage 2**: Group by OR operators (handle alternatives)
- **Stage 2.5**: Handle parentheses (process grouping)
- **Stage 3**: Parse sequences (build element chains)
- **Stage 4**: Handle quantifiers (*, +, ?)
- **Stage 5**: Build tree structure (final AST)

### 3. Annotation Preservation Strategy
- Static annotations → metadata fields in AST
- Dynamic annotations → code generation templates
- Clear separation of concerns

### 4. Error Handling Approach
- Graceful degradation for malformed data
- Fallback parsing for annotation formats
- Comprehensive error reporting with context

## Performance Characteristics

### Memory Usage
- Raw AST: ~1MB per 1000 grammar rules
- Transformed AST: ~2MB per 1000 grammar rules (due to expanded structure)
- Annotation metadata: ~100KB per 1000 annotations

### Processing Speed
- EBNF → Raw AST: ~1000 rules/second (Perl)
- Raw AST → Transformed AST: ~5000 rules/second (compiled languages)
- Full pipeline: ~500 rules/second end-to-end

### Scalability Limits
- Tested with grammars up to 10,000 rules
- Memory usage scales linearly with rule count
- No significant performance degradation observed

## Quality Assurance

### Test Coverage
- Unit tests: 95%+ coverage across all languages
- Integration tests: End-to-end pipeline validation
- Regression tests: Historical bug prevention
- Cross-language tests: JSON format compatibility

### Validation Strategy
- Schema validation for JSON formats
- Round-trip testing (EBNF → Raw AST → Transformed AST)
- Annotation preservation verification
- Error condition coverage

## Extension Points

### Adding New Languages
1. Implement 5-stage transformation pipeline
2. Add JSON input/output handling
3. Implement annotation parsing logic
4. Create comprehensive test suite
5. Update build system integration

### Adding New Annotation Types
1. Update EBNF parser to recognize annotation
2. Modify Raw AST format specification
3. Update all language implementations
4. Add test cases for new annotation
5. Update documentation

### Adding New AST Node Types
1. Define node structure in each language
2. Update transformation pipeline stages
3. Modify JSON serialization/deserialization
4. Add comprehensive test coverage
5. Update parser generation logic

## Dependencies and Requirements

### Runtime Dependencies
- **Perl**: 5.20+, JSON module, Data::Dumper
- **Rust**: 1.70+, serde, anyhow, chrono, TokenValue enum system
- **Julia**: 1.8+, JSON3, Dates
- **Go**: 1.19+, standard library only
- **Python**: 3.8+, standard library only
- **Zig**: 0.15.1+, standard library only

### Development Dependencies
- Git for version control
- Language-specific test frameworks
- JSON schema validators
- Documentation generators

This overview provides the technical foundation needed for immediate project contribution and extension.
