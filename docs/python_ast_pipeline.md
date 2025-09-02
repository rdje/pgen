# Python AST Pipeline

## Module: `python.ast_pipeline`

The Python AST Pipeline provides a complete implementation of the EBNF AST transformation pipeline with dual-mode API for both same-language optimization and cross-language interoperability.

## Classes

### `PipelineConfig`

Configuration dataclass for AST transformation pipeline.

#### Attributes

- `debug: bool = False` - Enable debug output during transformation
- `preserve_annotations: bool = True` - Preserve semantic and logging annotations
- `validate_input: bool = True` - Validate raw AST JSON format
- `validate_output: bool = True` - Validate transformed AST format

#### Usage

```python
config = PipelineConfig(
    debug=True,
    preserve_annotations=True,
    validate_input=True
)
```

### `ASTNode`

Represents a node in the transformed AST structure.

#### Constructor

```python
ASTNode(node_type: str, **kwargs)
```

#### Parameters

- `node_type: str` - Node type (`"atom"`, `"sequence"`, `"or"`, `"quantified"`)
- `**kwargs` - Additional node-specific attributes

#### Methods

##### `to_dict() -> Dict[str, Any]`

Convert AST node to dictionary for JSON serialization.

**Returns:** Dictionary representation suitable for JSON output.

#### Usage

```python
atom_node = ASTNode("atom", value=["quoted_string", "literal"])
sequence_node = ASTNode("sequence", elements=[atom_node])
dict_repr = sequence_node.to_dict()
```

### `PythonASTPipeline`

Main pipeline class implementing the 5-stage AST transformation process.

#### Constructor

```python
PythonASTPipeline(config: PipelineConfig = None)
```

#### Parameters

- `config: PipelineConfig` - Pipeline configuration. Defaults to `PipelineConfig()` if None.

#### Attributes

- `config: PipelineConfig` - Pipeline configuration
- `debug: bool` - Debug flag from configuration
- `stats: Dict[str, int]` - Transformation statistics
- `annotations: Dict[str, Dict]` - Preserved annotations

#### Methods

##### `load_raw_ast(json_file: str) -> Dict[str, Any]`

Load raw AST JSON from file with validation.

**Parameters:**
- `json_file: str` - Path to raw AST JSON file

**Returns:** Dictionary containing raw AST data

**Raises:**
- `ValueError` - If JSON format is invalid
- `IOError` - If file cannot be read

##### `transform_raw_ast(raw_ast: List[List[List[str]]]) -> Tuple[Dict[str, ASTNode], List[str]]`

Transform raw AST to semantic AST using the 5-stage pipeline.

**Parameters:**
- `raw_ast: List[List[List[str]]]` - Raw AST rule definitions

**Returns:** Tuple of (grammar_tree, rule_order)
- `grammar_tree: Dict[str, ASTNode]` - Transformed AST nodes by rule name
- `rule_order: List[str]` - Ordered list of rule names

##### `save_transformed_ast(grammar_tree: Dict[str, ASTNode], rule_order: List[str], grammar_name: str, output_file: str)`

Save transformed AST to JSON file.

**Parameters:**
- `grammar_tree: Dict[str, ASTNode]` - Transformed AST nodes
- `rule_order: List[str]` - Rule order
- `grammar_name: str` - Grammar name for metadata
- `output_file: str` - Output file path

##### `transform_from_file(raw_ast_json_file: str, output_json_file: Optional[str] = None) -> Tuple[Dict[str, ASTNode], List[str]]`

Same-language API: Transform raw AST JSON file to in-memory AST with optional JSON output.

**Parameters:**
- `raw_ast_json_file: str` - Input raw AST JSON file
- `output_json_file: Optional[str]` - Optional output JSON file for cross-language usage

**Returns:** Tuple of (grammar_tree, rule_order)

**Usage:**
```python
# In-memory only
grammar_tree, rule_order = pipeline.transform_from_file("input.json")

# In-memory + JSON output  
grammar_tree, rule_order = pipeline.transform_from_file("input.json", "output.json")
```

##### `transform_to_json(raw_ast_json_file: str, output_json_file: str)`

Cross-language API: Transform raw AST JSON file to transformed AST JSON file.

**Parameters:**
- `raw_ast_json_file: str` - Input raw AST JSON file
- `output_json_file: str` - Output transformed AST JSON file

**Usage:**
```python
pipeline.transform_to_json("raw_input.json", "transformed_output.json")
```

## Pipeline Stages

The transformation pipeline implements the following stages:

### Stage 1: Extract Annotations

Extracts and preserves semantic annotations, logging annotations, and return annotations from raw AST.

**Preserved Annotations:**
- `semantic_annotations: Dict[str, List[str]]` - Semantic metadata by rule name
- `logging_annotations: Dict[str, List[str]]` - Logging metadata by rule name  
- `return_annotations: Dict[str, str]` - Return type annotations by rule name

### Stage 2: Group by OR Operators

Groups rule definitions by OR operators (`|`), splitting alternatives into separate branches.

### Stage 2.5: Handle Parentheses

Processes parentheses and grouping constructs, creating nested group structures.

### Stage 3: Parse Sequences

Converts token sequences into structured sequence nodes and atomic elements.

### Stage 4: Handle Quantifiers

Processes quantifiers (`*`, `+`, `?`), creating quantified nodes with associated elements.

### Stage 5: Build Tree Structure

Constructs final AST tree with proper node hierarchy and rule references.

## Node Types

### Atom Node

Represents atomic elements (terminals or rule references).

**Structure:**
```json
{
  "type": "atom",
  "value": ["token_type", "token_value"] | nested_node
}
```

### Sequence Node

Represents ordered sequences of elements.

**Structure:**
```json
{
  "type": "sequence", 
  "elements": [node1, node2, ...]
}
```

### OR Node

Represents alternative choices.

**Structure:**
```json
{
  "type": "or",
  "alternatives": [node1, node2, ...]
}
```

### Quantified Node

Represents elements with quantifiers.

**Structure:**
```json
{
  "type": "quantified",
  "element": node,
  "quantifier": "*" | "+" | "?"
}
```

## Command Line Interface

```bash
python python/ast_pipeline.py input_raw.json [output_transformed.json] [options]
```

### Arguments

- `input_json` - Raw AST JSON input file
- `output_json` - Optional transformed AST JSON output file

### Options

- `--debug, -d` - Enable debug output
- `--stats, -s` - Show transformation statistics

### Examples

```bash
# Transform to in-memory (same-language mode)
python python/ast_pipeline.py grammar_raw.json --debug --stats

# Transform to JSON file (cross-language mode)  
python python/ast_pipeline.py grammar_raw.json grammar_transformed.json --stats
```

## Error Handling

The pipeline implements comprehensive error handling:

### Exception Types

- `ValueError` - Invalid input format or data
- `IOError` - File access errors
- `TypeError` - Type mismatches in AST processing

### Validation

Input validation checks:
- Required JSON fields (`grammar_name`, `raw_ast`, `metadata`)
- Proper metadata format (`format: "raw_ast"`)
- Array structure for `raw_ast` field
- Token format validation (2-element arrays)

## Performance Characteristics

### Same-Language Mode

- In-memory AST processing
- No JSON serialization overhead
- Direct object passing between pipeline stages
- Optimal for single-language workflows

### Cross-Language Mode

- JSON serialization/deserialization overhead
- File I/O operations
- Process isolation
- Maximum interoperability with other tools

## Integration

### With Existing Tools

The pipeline integrates with existing tools through:

1. **Input Compatibility**: Consumes raw AST JSON from `tools/ebnf_to_json.pl`
2. **Output Compatibility**: Produces transformed AST JSON for `tools/syntactic_data_generator.py`
3. **API Compatibility**: Follows standardized API patterns for cross-language usage

### Usage Patterns

#### Production Pipeline
```python
config = PipelineConfig(debug=False, validate_input=True)
pipeline = PythonASTPipeline(config)
grammar_tree, rule_order = pipeline.transform_from_file("production_grammar.json")
# Use grammar_tree directly with Python generators
```

#### Development/Testing
```python
config = PipelineConfig(debug=True, preserve_annotations=True)
pipeline = PythonASTPipeline(config)
pipeline.transform_to_json("test_grammar.json", "debug_output.json")
# Inspect JSON output for debugging
```

#### Cross-Language Workflow
```python
pipeline = PythonASTPipeline()
pipeline.transform_to_json("raw.json", "transformed.json")
# transformed.json can be consumed by Rust, Go, Julia tools
```

## Statistics and Monitoring

The pipeline tracks the following metrics:

- `rules_processed: int` - Number of rules successfully transformed
- `annotations_preserved: int` - Number of annotations extracted and preserved
- `transformations_applied: int` - Number of pipeline stages completed

Access statistics via `pipeline.stats` after transformation completion.

## Thread Safety

The `PythonASTPipeline` class is not thread-safe. Create separate instances for concurrent processing or implement external synchronization.
