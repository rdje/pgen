# Standardized API Interfaces

This document defines the common API patterns that each language implementation should follow for both same-language optimization and cross-language compatibility.

## Core Principles

1. **Dual-Mode API**: Support both in-memory and JSON interfaces
2. **Consistent Naming**: Use similar method/function names across languages  
3. **Clear Separation**: Distinguish same-language vs cross-language usage
4. **Error Handling**: Consistent error reporting and validation
5. **Extensibility**: Easy to add new features while maintaining compatibility

## Pipeline Interface

### Constructor/Initialization

Each language should provide a pipeline class with configurable options:

**Python Example:**
```python
from ast_pipeline import PythonASTPipeline, PipelineConfig

config = PipelineConfig(debug=True, preserve_annotations=True)
pipeline = PythonASTPipeline(config)
```

**Rust Example (hypothetical):**
```rust
use ast_pipeline::{RustASTPipeline, PipelineConfig};

let config = PipelineConfig::new()
    .with_debug(true)
    .with_preserve_annotations(true);
let pipeline = RustASTPipeline::new(config);
```

### Same-Language API (Performance Optimized)

#### `transform_from_file(raw_ast_json_file, [output_json_file]) -> (AST, rule_order)`

Load raw AST JSON, transform in-memory, optionally save transformed JSON.

**Python:**
```python
# In-memory only (same-language optimization)
grammar_tree, rule_order = pipeline.transform_from_file("input.json")

# In-memory + JSON output (enables cross-language usage)
grammar_tree, rule_order = pipeline.transform_from_file("input.json", "output.json")
```

**Rust:**
```rust
// In-memory only
let (ast, rule_order) = pipeline.transform_from_file("input.json", None)?;

// In-memory + JSON output
let (ast, rule_order) = pipeline.transform_from_file("input.json", Some("output.json"))?;
```

#### `transform_raw_ast(raw_ast_data) -> (AST, rule_order)`

Transform raw AST data structures directly (no file I/O).

**Python:**
```python
raw_ast = load_raw_ast_data()  # From memory, network, etc.
grammar_tree, rule_order = pipeline.transform_raw_ast(raw_ast)
```

### Cross-Language API (JSON Interface)

#### `transform_to_json(raw_ast_json_file, output_json_file)`

Transform raw AST JSON file → transformed AST JSON file.

**Python:**
```python
pipeline.transform_to_json("raw_input.json", "transformed_output.json")
```

**Command Line:**
```bash
python ast_pipeline.py raw_input.json transformed_output.json
```

### Validation Methods

#### `validate_raw_ast(data) -> bool`
#### `validate_transformed_ast(data) -> bool`

**Python:**
```python
if pipeline.validate_raw_ast(raw_data):
    grammar_tree, rule_order = pipeline.transform_raw_ast(raw_data['raw_ast'])
```

## Generator Interface

### Constructor/Initialization

Each generator type (Code, Data) should provide configurable initialization:

**Python Example:**
```python
from syntactic_data_generator import SyntacticDataGenerator, GeneratorConfig

config = GeneratorConfig(seed=42, max_depth=10)
generator = SyntacticDataGenerator(grammar_tree, rule_order, config)
```

### Same-Language API

#### `generate_from_ast(grammar_tree, rule_order, config) -> output`

Generate directly from in-memory AST (same-language optimization).

**Python Data Generator:**
```python
# From in-memory AST
samples = data_generator.generate_from_ast(grammar_tree, rule_order, count=10)
```

**Python Code Generator (hypothetical):**
```python
# From in-memory AST
parser_code = code_generator.generate_from_ast(grammar_tree, rule_order, target="python")
```

### Cross-Language API

#### `generate_from_json(transformed_ast_json_file, config) -> output`

Generate from transformed AST JSON file (cross-language interface).

**Python:**
```python
samples = data_generator.generate_from_json("transformed.json", count=10)
parser_code = code_generator.generate_from_json("transformed.json", target="rust")
```

**Command Line:**
```bash
python syntactic_data_generator.py transformed.json --count 10 --rule expression
python code_generator.py transformed.json --target rust --output parser.rs
```

## Combined Same-Language Workflow

For maximum performance when using same language throughout:

**Python Example:**
```python
# Load and transform (JSON → in-memory)
pipeline = PythonASTPipeline()
grammar_tree, rule_order = pipeline.transform_from_file("raw.json")

# Generate code and data (in-memory → output)
code_gen = PythonCodeGenerator()
data_gen = SyntacticDataGenerator(grammar_tree, rule_order)

parser_code = code_gen.generate(target="python")  # In-memory
test_data = data_gen.generate(count=100)          # In-memory
```

## Error Handling Standards

### Common Exception Types

Each language should define equivalent exception types:

1. **InvalidRawASTError**: Malformed raw AST JSON
2. **InvalidTransformedASTError**: Malformed transformed AST JSON  
3. **TransformationError**: Error during AST transformation
4. **GenerationError**: Error during code/data generation
5. **ValidationError**: Schema validation failure

### Error Messages

Consistent error message patterns across languages:

```
"Raw AST JSON missing required field: {field}"
"Invalid token type in raw AST: {type}"
"Transformation failed at step {step}: {reason}"
"Cannot generate from rule '{rule}': {reason}"
```

## Configuration Standards

### Pipeline Configuration

Common configuration options across all implementations:

```json
{
  "debug": false,
  "preserve_annotations": true,
  "validate_input": true,
  "validate_output": true,
  "max_recursion_depth": 100,
  "enable_optimizations": true
}
```

### Data Generator Configuration

```json
{
  "seed": 42,
  "max_depth": 10,
  "recursion_limit": 5,
  "quantifier_min": {"*": 0, "+": 1, "?": 0},
  "quantifier_max": {"*": 5, "+": 5, "?": 1},
  "string_length_range": [3, 15],
  "pretty_print": true
}
```

### Code Generator Configuration

```json
{
  "target_language": "python",
  "output_format": "class",
  "include_tests": true,
  "enable_logging": false,
  "optimize_performance": true,
  "generate_documentation": true
}
```

## Language-Specific Adaptations

While maintaining API consistency, each language should follow its own idioms:

### Python
- Use snake_case method names
- Provide both class-based and functional APIs
- Support context managers where appropriate
- Use type hints and dataclasses

### Rust  
- Use snake_case method names
- Return `Result<T, E>` for error handling
- Provide builder pattern for configuration
- Use zero-copy optimizations where possible

### Go
- Use PascalCase for public methods
- Return `(result, error)` tuples
- Provide functional options for configuration  
- Use interfaces for extensibility

### Julia
- Use snake_case method names
- Support multiple dispatch
- Provide both mutable and immutable APIs
- Use parametric types for performance

## Testing Standards

### Unit Tests
Each implementation should provide equivalent test coverage:

1. **Raw AST Loading Tests**
2. **Transformation Pipeline Tests** (each stage)  
3. **JSON Schema Validation Tests**
4. **Error Handling Tests**
5. **Cross-Language Compatibility Tests**

### Integration Tests
Test cross-language workflows:

1. **Perl → Python**: Raw AST JSON → Python pipeline → Data generation
2. **Perl → Rust**: Raw AST JSON → Rust pipeline → Code generation
3. **Python → Go**: Transformed AST JSON → Go generators
4. **Round-trip**: Perl → Language X → JSON → Language Y → Output

### Performance Benchmarks
Compare same-language vs cross-language performance:

- **In-memory processing time**
- **JSON serialization/deserialization overhead**  
- **Memory usage patterns**
- **Scalability with grammar size**

## Documentation Standards

Each language implementation should provide:

1. **API Reference**: Complete method documentation
2. **Usage Examples**: Both same-language and cross-language  
3. **Performance Guide**: When to use each API mode
4. **Migration Guide**: Moving between language implementations
5. **Extension Guide**: Adding new features

## Version Compatibility

### JSON Schema Versioning
- Include schema version in JSON metadata
- Maintain backward compatibility for minor versions
- Provide migration tools for major version changes

### API Versioning
- Use semantic versioning for implementations
- Clearly document API changes and deprecations
- Provide compatibility layers when possible

This standardization enables a truly polyglot ecosystem where components can be mixed and matched while maintaining performance and reliability.
