# Multi-Language Architecture for EBNF Parser Generator

This document captures the architectural decisions and design principles for building a flexible, multi-language EBNF parser generator ecosystem.

## Overview

The system is designed to enable maximum flexibility by allowing different stages of the pipeline to be implemented in different programming languages, while optimizing performance when using the same language throughout.

## Architecture Principles

### 1. **Universal JSON Interface Boundaries**

JSON serves as the standard interchange format between different language implementations:

```
EBNF → [Perl Parser] → Raw AST JSON → [Any Language Pipeline] → Transformed AST JSON → [Any Language Generator] → Output
```

### 2. **In-Memory Optimization for Same Language**

When pipeline and generator are implemented in the same language, data exchange remains in-memory for performance:

```
EBNF → [Perl Parser] → Raw AST JSON → [Same Language: Pipeline + Generator] → Output
                                            ↑
                                    In-memory data structures
                                    (no JSON serialization overhead)
```

## Implementation Strategy

### Cross-Language Boundaries (JSON Required)
- **Perl Parser** outputs Raw AST JSON (universal input format)
- **Different Language Pipeline** reads Raw AST JSON → outputs Transformed AST JSON
- **Different Language Generator** reads Transformed AST JSON → produces final output

### Same-Language Optimization (In-Memory)
Language-specific implementations can:
1. Read Raw AST JSON once
2. Transform AST using in-memory data structures  
3. Generate code/data directly from in-memory AST (no JSON serialization)
4. Provide optional JSON output for cross-language usage

## Target Language Implementations

### Pipeline + Generator Combinations

#### **Same-Language (Optimized)**
- **Perl**: Raw AST JSON → Pipeline → Generator (in-memory AST) → Output
- **Python**: Raw AST JSON → Pipeline → Generator (in-memory objects) → Output  
- **Rust**: Raw AST JSON → Pipeline → Generator (in-memory structs) → Output
- **Julia**: Raw AST JSON → Pipeline → Generator (in-memory types) → Output
- **Go**: Raw AST JSON → Pipeline → Generator (in-memory structs) → Output
- **Ruby**: Raw AST JSON → Pipeline → Generator (in-memory objects) → Output

#### **Mixed-Language (JSON Interface)**
- Rust Pipeline → Transformed AST JSON → Python DataGenerator
- Python Pipeline → Transformed AST JSON → Go CodeGenerator  
- Julia Pipeline → Transformed AST JSON → Rust DataGenerator
- etc.

## API Design Standards

Each language implementation should provide:

### Same-Language API (Performance Optimized)
```python
# Direct in-memory processing
generate_code(raw_ast_json_file) → code_output
generate_data(raw_ast_json_file) → test_data
```

### Cross-Language API (JSON Interface)
```python  
# JSON interface for cross-language compatibility
transform_ast(raw_ast_json_file) → transformed_ast_json_file
generate_code(transformed_ast_json_file) → code_output
generate_data(transformed_ast_json_file) → test_data
```

### Hybrid API (Best of Both)
```python
# Flexible interface supporting both modes
pipeline = ASTPipeline(raw_ast_json_file)
transformed_ast = pipeline.transform()  # In-memory

# Same-language usage
code = CodeGenerator(transformed_ast).generate()  # In-memory
data = DataGenerator(transformed_ast).generate()  # In-memory

# Cross-language usage  
pipeline.save_json("transformed.json")  # Enable cross-language
```

## Benefits

### 1. **Performance Optimization**
- Same-language implementations avoid JSON serialization overhead
- In-memory data structures enable efficient processing
- Direct object/struct passing between pipeline stages

### 2. **Maximum Flexibility**
- Mix and match implementations across languages
- Use optimal language for each component (e.g., Rust for performance, Python for ML-based generation)
- Teams can work on different components independently

### 3. **Ecosystem Growth**
- Standard JSON interfaces enable easy integration
- Anyone can contribute generators in their preferred language
- Comprehensive testing via standard formats

### 4. **Developer Choice**
- Use single language for simplicity and performance
- Use multiple languages for specialized capabilities
- Migrate between approaches as needs evolve

## JSON Schema Standards

### Raw AST JSON Format
```json
{
  "grammar_name": "example_grammar",
  "raw_ast": [
    [
      ["rule", "rule_name"],
      ["rule_reference", "other_rule"],
      ["quoted_string", "literal"],
      ...
    ]
  ],
  "metadata": {
    "source_file": "grammar.ebnf",
    "format": "raw_ast",
    "generated_at": "timestamp",
    "parser": "ebnf_to_json.pl"
  }
}
```

### Transformed AST JSON Format
```json
{
  "grammar_name": "example_grammar", 
  "grammar_tree": {
    "rule_name": {
      "type": "sequence|atom|or|quantified",
      "elements": [...],
      "value": [...],
      "alternatives": [...],
      "quantifier": "*|+|?",
      ...
    }
  },
  "rule_order": ["rule1", "rule2", ...],
  "metadata": {
    "format": "transformed_ast",
    "transformed_at": "timestamp",
    "transformer": "language_implementation",
    "source_format": "raw_ast"
  }
}
```

## Implementation Roadmap

### Phase 1: Foundation
- ✅ Perl EBNF Parser → Raw AST JSON
- ✅ JSON Schema Specifications  
- ✅ Python DataGenerator (proof of concept)

### Phase 2: Python Ecosystem
- Python Pipeline Implementation
- Python CodeGenerator Implementation  
- Same-language optimization
- Cross-language JSON interface

### Phase 3: Multi-Language Expansion
- Rust implementations
- Julia implementations
- Go implementations
- Ruby implementations

### Phase 4: Advanced Features
- Performance benchmarking across languages
- ML-enhanced data generation
- Language-specific optimizations
- Comprehensive integration testing

## Design Insights

### Key Architectural Decisions

1. **JSON as Universal Interface**: Enables true polyglot ecosystem while maintaining compatibility
2. **Performance First**: In-memory optimization when using same language prevents unnecessary overhead
3. **Gradual Migration**: Teams can start with single-language implementation and add cross-language features as needed
4. **Standard APIs**: Consistent interface patterns across all language implementations
5. **Modular Design**: Each component (Parser, Pipeline, Generators) can be developed and optimized independently

### Trade-offs Considered

- **Flexibility vs Performance**: Solved by dual-mode API (in-memory + JSON)
- **Consistency vs Language Idioms**: Standard JSON format + language-specific APIs
- **Complexity vs Capability**: Start simple, grow complex as needed

This architecture provides a foundation for building a truly flexible, high-performance, multi-language EBNF parser generator ecosystem.
