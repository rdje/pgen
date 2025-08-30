# Universal Return Annotation Composition System

## Philosophy
Return annotations describe **what** to return, not **how** to implement it in a specific language. The composition rules are universal mathematical/structural operations.

## Core Semantic Operations

### 1. Reference Operation
- **Semantics**: Access element at position N from results
- **Notation**: `$N`
- **Languages**: 
  - Perl: `$results[N-1]`
  - Rust: `results[N-1].clone()`
  - Python: `results[N-1]`
  - TypeScript: `results[N-1]`
  - Julia: `results[N]`
  - Ruby: `results[N-1]`

### 2. Collection Operation  
- **Semantics**: Expand quantified results from position N
- **Notation**: `$N*`
- **Languages**:
  - Perl: `@{collect_quantified_results(N, \@results)}`
  - Rust: `expand_quantified(N, &results)`
  - Python: `*collect_quantified(N, results)`
  - TypeScript: `...expandQuantified(N, results)`
  - Julia: `collect_quantified(N, results)...`
  - Ruby: `*collect_quantified(N, results)`

### 3. Array Construction
- **Semantics**: Create array from elements
- **Notation**: `[elem1, elem2, ...]`
- **Languages**:
  - Perl: `[compose(elem1), compose(elem2), ...]`
  - Rust: `vec![compose(elem1), compose(elem2), ...]`
  - Python: `[compose(elem1), compose(elem2), ...]`
  - TypeScript: `[compose(elem1), compose(elem2), ...]`
  - Julia: `[compose(elem1), compose(elem2), ...]`
  - Ruby: `[compose(elem1), compose(elem2), ...]`

### 4. Object Construction
- **Semantics**: Create key-value mapping
- **Notation**: `{key1: value1, key2: value2}`
- **Languages**:
  - Perl: `{key1 => compose(value1), key2 => compose(value2)}`
  - Rust: `HashMap::from([(key1, compose(value1)), (key2, compose(value2))])`
  - Python: `{"key1": compose(value1), "key2": compose(value2)}`
  - TypeScript: `{key1: compose(value1), key2: compose(value2)}`
  - Julia: `Dict("key1" => compose(value1), "key2" => compose(value2))`
  - Ruby: `{key1: compose(value1), key2: compose(value2)}`

### 5. Property Access
- **Semantics**: Navigate object/array structure
- **Notation**: `$N.property.path[index]`
- **Languages**: Language-specific property/index access patterns

## Universal Composition Algorithm

```pseudocode
function compose_return_expression(ast_node, results_var):
    match ast_node.type:
        case "scalar_ref":
            return generate_reference(ast_node.index, results_var)
        case "array":
            elements = []
            for element in ast_node.contents:
                if element.quantified:
                    elements.append(generate_expansion(element, results_var))
                else:
                    elements.append(compose_return_expression(element, results_var))
            return generate_array_construction(elements)
        case "object":
            pairs = []
            for pair in ast_node.contents:
                key = compose_key(pair.key)
                value = compose_return_expression(pair.value, results_var)
                pairs.append((key, value))
            return generate_object_construction(pairs)
        case "quantified_array":
            return generate_collection_expansion(ast_node.element, results_var)
        case "ultimate_dot_notation":
            base = compose_return_expression(ast_node.base, results_var)
            return generate_property_access(base, ast_node.path)
        case "literal":
            return generate_literal(ast_node.value)
```

## Language-Specific Code Generators

### Interface
```pseudocode
interface ReturnCodeGenerator:
    function generate_reference(index, results_var) -> string
    function generate_expansion(element, results_var) -> string  
    function generate_array_construction(elements) -> string
    function generate_object_construction(pairs) -> string
    function generate_property_access(base, path) -> string
    function generate_literal(value) -> string
```

### Example: Rust Implementation
```rust
impl ReturnCodeGenerator for RustGenerator {
    fn generate_reference(&self, index: usize, results_var: &str) -> String {
        format!("{}.get({}).cloned().unwrap_or_default()", results_var, index - 1)
    }
    
    fn generate_array_construction(&self, elements: Vec<String>) -> String {
        format!("vec![{}]", elements.join(", "))
    }
    
    fn generate_object_construction(&self, pairs: Vec<(String, String)>) -> String {
        let entries: Vec<String> = pairs.into_iter()
            .map(|(k, v)| format!("({}, {})", k, v))
            .collect();
        format!("HashMap::from([{}])", entries.join(", "))
    }
    // ... etc
}
```

### Example: Python Implementation  
```python
class PythonGenerator(ReturnCodeGenerator):
    def generate_reference(self, index: int, results_var: str) -> str:
        return f"{results_var}[{index - 1}]"
    
    def generate_array_construction(self, elements: List[str]) -> str:
        return f"[{', '.join(elements)}]"
    
    def generate_object_construction(self, pairs: List[Tuple[str, str]]) -> str:
        entries = [f'"{k}": {v}' for k, v in pairs]
        return f"{{{', '.join(entries)}}}"
    # ... etc
```

## Complex Example Translations

### Input: `{items: [$1*], count: $2, nested: {data: $3.values[0]}}`

#### Perl:
```perl
{
    items => collect_quantified_results(1, \@results),
    count => $results[1], 
    nested => {
        data => access_dot_path($results[2], ['values', {type: 'index', value: 0}])
    }
}
```

#### Rust:
```rust
HashMap::from([
    ("items", expand_quantified(1, &results)),
    ("count", results[1].clone()),
    ("nested", HashMap::from([
        ("data", access_dot_path(&results[2], vec![
            PathSegment::Property("values"),
            PathSegment::Index(0)
        ]))
    ]))
])
```

#### Python:
```python
{
    "items": expand_quantified(1, results),
    "count": results[1],
    "nested": {
        "data": access_dot_path(results[2], [
            ("property", "values"),
            ("index", 0)
        ])
    }
}
```

## Benefits of This Approach

1. **Language Agnostic**: Same semantic model works for any target language
2. **Compositional**: Complex structures built from simple operations
3. **Extensible**: New operations added by extending the interface
4. **Testable**: Can verify semantic correctness independent of syntax
5. **Maintainable**: Single source of truth for return annotation semantics

## Implementation Strategy

1. **Phase 1**: Define universal AST for return annotations
2. **Phase 2**: Implement language-agnostic composition algorithm  
3. **Phase 3**: Create language-specific code generators
4. **Phase 4**: Test with increasingly complex examples across all languages
5. **Phase 5**: Integrate with existing parser generators
