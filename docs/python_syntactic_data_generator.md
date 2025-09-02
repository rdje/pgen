# Python Syntactic Data Generator

## Module: `tools.syntactic_data_generator`

The Python Syntactic Data Generator produces syntactically valid input files from transformed AST grammars. It operates without domain-specific knowledge, generating purely syntactic outputs based on grammar structure.

## Classes

### `GeneratorConfig`

Configuration dataclass for syntactic data generation.

#### Attributes

- `seed: int = 42` - Random seed for reproducible results
- `max_depth: int = 10` - Maximum recursion depth to prevent infinite expansion
- `recursion_limit: int = 5` - Maximum repetitions of same rule in call stack
- `quantifier_min: Dict[str, int]` - Minimum repetitions for quantifiers
- `quantifier_max: Dict[str, int]` - Maximum repetitions for quantifiers  
- `prefer_shorter_alternatives: bool = True` - Prefer shorter alternatives at higher depths
- `string_length_range: Tuple[int, int] = (3, 15)` - Range for generated string lengths
- `regex_generation_attempts: int = 10` - Maximum attempts for regex matching
- `pretty_print: bool = True` - Add spacing between elements
- `indent_size: int = 2` - Indentation size for pretty printing

#### Default Quantifier Behavior

```python
quantifier_min = {'*': 0, '+': 1, '?': 0}
quantifier_max = {'*': 5, '+': 5, '?': 1}
```

#### Usage

```python
config = GeneratorConfig(
    seed=123,
    max_depth=15,
    quantifier_max={'*': 3, '+': 3, '?': 1},
    string_length_range=(5, 20)
)
```

### `SyntacticDataGenerator`

Main generator class for producing syntactic test data from transformed AST grammars.

#### Constructor

```python
SyntacticDataGenerator(grammar_tree: Dict, rule_order: List[str], config=None)
```

#### Parameters

- `grammar_tree: Dict` - Transformed AST tree with rule definitions
- `rule_order: List[str]` - Ordered list of rule names
- `config: GeneratorConfig` - Generator configuration. Defaults to `GeneratorConfig()` if None.

#### Attributes

- `grammar_tree: Dict` - Grammar tree structure
- `rule_order: List[str]` - Rule ordering
- `config: GeneratorConfig` - Configuration settings
- `random: random.Random` - Random number generator instance
- `call_stack: List[str]` - Current rule call stack for recursion tracking
- `stats: Dict[str, int]` - Generation statistics

#### Methods

##### `generate(start_rule: str = None, count: int = 1) -> List[str]`

Generate multiple input samples from the grammar.

**Parameters:**
- `start_rule: str` - Starting rule name. Defaults to first rule in rule_order
- `count: int` - Number of samples to generate

**Returns:** List of generated sample strings

**Usage:**
```python
generator = SyntacticDataGenerator(grammar_tree, rule_order)
samples = generator.generate(start_rule="expression", count=10)
```

##### `generate_rule(rule_name: str, depth: int) -> str`

Generate content for a specific grammar rule.

**Parameters:**
- `rule_name: str` - Name of rule to generate
- `depth: int` - Current recursion depth

**Returns:** Generated string content

**Raises:**
- `RecursionError` - If maximum depth or recursion limits exceeded

##### `generate_to_file(output_file: str, start_rule: str = None, count: int = 10) -> int`

Generate samples and write to file.

**Parameters:**
- `output_file: str` - Output file path
- `start_rule: str` - Starting rule name
- `count: int` - Number of samples to generate

**Returns:** Number of samples successfully generated

**File Format:**
```
# Generated test data from grammar
# Generated N samples  
# Configuration: seed=42, max_depth=10

# Sample 1
generated_content_1

# Sample 2  
generated_content_2
```

## Generation Algorithm

### Rule Processing

The generator processes rules based on their AST node type:

#### Atom Nodes
- `quoted_string` tokens: Return literal string content
- `regex` tokens: Generate matching string using regex analysis
- `rule_reference` tokens: Recursively generate referenced rule
- Nested nodes: Process recursively

#### Sequence Nodes
- Generate each element in order
- Add spacing between elements if `pretty_print` enabled
- Return concatenated result

#### OR Nodes (Alternatives)
- Select one alternative using weighted random selection
- Apply complexity-based weighting at higher depths
- Generate content from selected alternative

#### Quantified Nodes
- Determine repetition count based on quantifier and configuration
- Reduce maximum count at higher depths to prevent explosion
- Generate repeated content with optional separators

### Recursion Control

The generator implements multiple recursion control mechanisms:

1. **Depth Limiting**: Stops expansion beyond `max_depth`
2. **Rule Repetition Limiting**: Prevents same rule appearing more than `recursion_limit` times in call stack
3. **Minimal Generation**: Falls back to minimal valid content when limits hit

### Terminal Generation

#### String Literals
- Remove surrounding quotes
- Return literal content

#### Regular Expressions
Supports common regex patterns:
- `\d+`, `(\d+)`: Generate random integers (1-999)
- `[a-zA-Z_]\w*`: Generate valid identifiers (3-12 characters)
- `"[^"]*"`, `'[^']*'`: Generate quoted strings
- `\s+`: Generate single space
- Fallback: Pattern analysis for character classes

#### Identifier Generation
- First character: `[a-zA-Z_]`
- Remaining characters: `[a-zA-Z0-9_]`
- Length: 3-12 characters

## Command Line Interface

```bash
python tools/syntactic_data_generator.py grammar.json [options]
```

### Arguments

- `grammar_json` - Transformed grammar JSON file

### Options

- `--output, -o FILE` - Output file (default: stdout)
- `--count, -n INT` - Number of samples (default: 10)
- `--rule, -r RULE` - Start rule (default: first rule)
- `--seed INT` - Random seed (default: 42)
- `--max-depth INT` - Max recursion depth (default: 10)  
- `--config FILE` - Configuration JSON file
- `--stats` - Show generation statistics

### Examples

```bash
# Generate to stdout
python tools/syntactic_data_generator.py grammar.json --count 5

# Generate to file with specific rule
python tools/syntactic_data_generator.py grammar.json -o test_data.txt --rule expression --count 100

# Use custom configuration
python tools/syntactic_data_generator.py grammar.json --config custom_config.json --stats
```

### Configuration File Format

```json
{
  "seed": 42,
  "max_depth": 15,
  "recursion_limit": 8,
  "quantifier_min": {"*": 0, "+": 1, "?": 0},
  "quantifier_max": {"*": 3, "+": 3, "?": 1},
  "string_length_range": [5, 25],
  "pretty_print": true,
  "prefer_shorter_alternatives": true
}
```

## Statistics

The generator tracks the following metrics during generation:

- `rules_generated: int` - Total number of rule expansions
- `alternatives_chosen: int` - Number of OR node selections
- `quantifiers_expanded: int` - Number of quantifier expansions
- `max_depth_reached: int` - Maximum depth achieved during generation

Access statistics via `generator.stats` after generation completion.

## Error Handling

### Exception Handling

The generator handles errors gracefully:

- `RecursionError`: Caught and logged, generation continues with next sample
- `General Exception`: Caught and logged, generation continues with next sample
- Missing rules: Returns placeholder content `<rule_name>`
- Invalid node types: Returns placeholder content `<unknown:type>`

### Graceful Degradation

When encountering issues:
1. **Missing Rules**: Generate placeholder content
2. **Depth Limits**: Use minimal generation strategy
3. **Recursion Limits**: Use minimal generation strategy
4. **Invalid Patterns**: Use fallback content generation

## Performance Characteristics

### Complexity
- Time complexity: O(depth * rules * samples)
- Space complexity: O(depth) for call stack
- Memory usage scales with grammar complexity and depth limits

### Optimization Features
- Weighted alternative selection reduces deep recursion
- Depth-based quantifier reduction prevents exponential growth
- Minimal generation fallbacks ensure termination
- String caching for repeated literal generation

## Input Format

The generator expects transformed AST JSON format:

```json
{
  "grammar_name": "example",
  "grammar_tree": {
    "rule_name": {
      "type": "sequence|atom|or|quantified",
      "elements": [...],
      "value": [...],
      "alternatives": [...],
      "quantifier": "*|+|?"
    }
  },
  "rule_order": ["rule1", "rule2", ...],
  "metadata": {
    "format": "transformed_ast"
  }
}
```

## Integration

### With AST Pipeline

The generator integrates directly with the Python AST Pipeline:

```python
from python.ast_pipeline import PythonASTPipeline
from tools.syntactic_data_generator import SyntacticDataGenerator

# Transform grammar
pipeline = PythonASTPipeline()
grammar_tree, rule_order = pipeline.transform_from_file("raw.json")

# Convert to dict format
grammar_dict = {name: node.to_dict() for name, node in grammar_tree.items()}

# Generate data
generator = SyntacticDataGenerator(grammar_dict, rule_order)
samples = generator.generate(count=50)
```

### Cross-Language Usage

The generator accepts transformed AST JSON from any pipeline implementation:

```bash
# Perl pipeline → JSON → Python generator
perl tools/transform_ast.pl raw.json transformed.json
python tools/syntactic_data_generator.py transformed.json --count 100

# Python pipeline → JSON → Python generator  
python python/ast_pipeline.py raw.json transformed.json
python tools/syntactic_data_generator.py transformed.json --count 100
```

## Thread Safety

The `SyntacticDataGenerator` class is not thread-safe due to shared state (`call_stack`, `stats`, `random`). Create separate instances for concurrent processing or implement external synchronization.

## Limitations

### Current Limitations
- No semantic understanding of grammar meaning
- Limited regex pattern support (extensible)
- No context-sensitive generation
- Fixed terminal generation strategies

### Future Enhancements
- Semantic annotation integration
- Advanced regex generation
- Context-aware generation
- Machine learning-based pattern recognition
- Domain-specific generation strategies
