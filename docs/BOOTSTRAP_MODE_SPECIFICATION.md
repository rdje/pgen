# Bootstrap Mode Annotation Support Specification

## Overview

Bootstrap mode enables the Rust AST pipeline to build itself without requiring external semantic and return annotation parsers. This breaks the circular dependency where the pipeline needs annotation parsers to build, but the annotation parsers need the pipeline to be generated.

## Bootstrap Mode Activation

**Command Line:**
```bash
ast_pipeline input.json --bootstrap-mode --generate-parser -o output_parser.rs
```

**Rust API:**
```rust
let config = PipelineConfig {
    bootstrap_mode: true,
    // ... other options
};
```

## Built-in Annotation Support Level

### Semantic Annotations (Built-in Support)

The bootstrap mode provides **BASIC** built-in parsing for common semantic annotations:

#### ✅ Supported Patterns:
1. **Simple name-value pairs**: `@codegen: "escape_literal_handling"`
2. **Type annotations**: `@type: "context_sensitive_construct"`
3. **Debug annotations**: `@debug: "trace_parsing"`
4. **Simple flags**: `@flag: "terminal_handling"`
5. **Function calls with simple arguments**: 
   - Single arg: `@transform: uppercase($1)`
   - Multiple args: `@format: concat($1, "_", $2)`
   - Mixed args: `@validate: check_range($1, "min", $2)`

#### ✅ Supported Function Call Patterns:
```
@transform: uppercase($1)                    // Single argument
@format: concat($1, "_suffix")              // Two arguments  
@validate: check_range($1, "0", "100")      // Three arguments
@generate: make_class($1, $2, "default")   // Mixed argument types
@convert: to_type($1, $2, $3, $4)          // Up to 4 arguments (bootstrap limit)
```

#### ✅ Supported Function Argument Types:
- **Scalar references**: `$1`, `$2`, `$3`
- **String literals**: `"constant_string"`, `"_suffix"`
- **Simple identifiers**: `default`, `min`, `max` (no quotes)
- **Numbers**: `0`, `100`, `42`

#### ✅ Supported Format:
- **Input**: `["semantic_annotation", ["name", "value"]]` from JSON AST
- **Output**: Stored as `"name:value"` in semantic_annotations HashMap
- **Example**: `@transform: uppercase($1)` → `"transform:uppercase($1)"`

#### ❌ NOT Supported in Bootstrap Mode:
- **Complex nested structures**: `@config: {parser: {mode: "strict", options: [...]}}`
- **Array values**: `@include: ["module1", "module2", "module3"]`
- **Conditional expressions**: `@when: ($target == "rust")`
- **Mathematical expressions**: `@priority: (base_priority + 10)`
- **Nested function calls**: `@transform: uppercase(concat($1, $2))`
- **Functions with >4 arguments**: `@complex: func($1, $2, $3, $4, $5)` (exceeds bootstrap limit)

### Return Annotations (Built-in Support)

The bootstrap mode provides **MINIMAL** built-in parsing for essential return annotations with **STRICT NESTING LIMITS**:

#### ✅ Supported Patterns (FLAT STRUCTURES ONLY):
1. **Simple scalar refs**: `$1`, `$2`, `$3`
2. **Basic arrays**: `[$1, $2]`, `[$1*]`
3. **Simple objects (1-3 keys)**: 
   - Single key: `{key: $1}`
   - Two keys: `{name: $1, value: $2}`
   - Three keys: `{type: $1, name: $2, value: $3}`
4. **Quantified arrays**: `[$1*]`, `[$2+]` (treated as `[$N*]`)

#### ✅ Supported Object Patterns (FLAT ONLY):
```
{key: $1}                           // Single property
{name: $1, value: $2}               // Two properties  
{type: $1, name: $2, value: $3}     // Three properties (bootstrap limit)
{id: $1, items: [$2*]}              // Property with quantified array (FLAT)
{result: $1, data: [$2, $3]}        // Property with simple array (FLAT)
```

#### ❌ STRICT NESTING BOUNDARY - NOT Supported:
```
// NO nested objects
{outer: {inner: $1}}                // REJECTED - nested object
{data: {items: [$1*]}}              // REJECTED - object inside object

// NO nested arrays  
[[$1, $2], [$3, $4]]               // REJECTED - array of arrays
{items: [[$1*], [$2*]]}            // REJECTED - array of arrays in object

// NO mixed complex nesting
{groups: [{id: $1, items: [$2*]}]} // REJECTED - object in array in object
[{items: [$1*]}, {data: [$2*]}]    // REJECTED - objects with arrays in array

// Objects with >3 keys
{a: $1, b: $2, c: $3, d: $4}       // REJECTED - exceeds 3-key limit
```

#### ✅ Supported Format:
- **Input**: `["return_scalar", "content"]`, `["return_array", "content"]`, `["return_object", "content"]`
- **Output**: Stored as parsed JSON structure for code generator use
- **Example**: `return_object: "{name: $1, value: $2}"` → `{"type": "object", "properties": [{"key": "name", "value": {"type": "scalar_ref", "index": 1}}, {"key": "value", "value": {"type": "scalar_ref", "index": 2}}]}`

## Bootstrap vs Full Parser Comparison

| Feature | Bootstrap Mode | Full Parser Mode |
|---------|---------------|------------------|
| **Semantic Annotations** | name:value + simple functions (≤4 args) | Full EBNF grammar support |
| **Semantic Functions** | Simple calls, no nesting | Nested calls, complex expressions |
| **Return Objects** | 1-3 keys, FLAT only | Unlimited keys, deep nesting |
| **Return Arrays** | Basic $N, [$N*], NO nesting | Complex slicing, nested structures |
| **Return Scalars** | Simple $N references | Mathematical expressions, conditionals |
| **Nesting Level** | **ZERO** (strictly flat) | Unlimited depth |
| **Dependency** | Self-contained | Requires external parsers |
| **Use Case** | Initial build, simple grammars | Production parsing |
| **Completeness** | ~30% of full grammar | 100% of grammar |

## Implementation Strategy

### Bootstrap Mode Detection

```rust
impl RustASTPipeline {
    fn should_use_bootstrap_mode(&self) -> bool {
        self.config.bootstrap_mode || 
        !self.external_parsers_available()
    }
    
    fn external_parsers_available(&self) -> bool {
        // Check if generated parser files exist
        std::path::Path::new("../../generated/semantic_annotation_parser.rs").exists() &&
        std::path::Path::new("../../generated/return_annotation_parser.rs").exists()
    }
}
```

### Built-in Parsing Functions

```rust
impl RustASTPipeline {
    /// Bootstrap-only semantic annotation parser
    /// Supports simple name:value and function calls with ≤4 simple arguments
    fn parse_semantic_annotation_bootstrap(&self, annotation_value: &str) -> Result<String> {
        // Parse "name: value" and "function_name(arg1, arg2, ...)" patterns
        // Maximum 4 arguments per function call
        // Return format: "name:value" or "name:function_name(args...)" for storage
    }
    
    /// Bootstrap-only return annotation parser  
    /// STRICTLY FLAT - no nesting allowed
    fn parse_return_annotation_bootstrap(&self, annotation_value: &str) -> Result<String> {
        // Parse ONLY flat structures: $N, [$N], {key1: $N, key2: $M, key3: $P}
        // REJECT any nested [] or {} patterns
        // Maximum 3 properties per object
        // Return JSON representation for code generator
    }
}
```

## Bootstrap Function Call Parsing Rules

### Argument Limits
- **Maximum 4 arguments** per function call in bootstrap mode
- Functions with >4 arguments will be stored as raw strings with warning
- Arguments must be simple types (no nested function calls)

### Supported Argument Types
- **Scalar references**: `$1`, `$2`, `$3`
- **String literals**: `"constant"`, `"_suffix"` (quoted)
- **Simple identifiers**: `default`, `min`, `max` (unquoted)
- **Numbers**: `0`, `100`, `42`, `-5`

### Function Call Examples

```rust
// Bootstrap mode can handle:
"@transform: uppercase($1)"              → Simple function, 1 arg
"@format: concat($1, \"_\", $2)"        → Function with 3 args
"@validate: range_check($1, 0, 100)"    → Mixed argument types
"@generate: make_obj($1, $2, $3, \"d\")" → 4 arguments (bootstrap limit)

// Bootstrap mode CANNOT handle:
"@complex: func($1, $2, $3, $4, $5)"    → >4 arguments (stored as raw)
"@nested: outer(inner($1))"             → Nested calls (stored as raw)
"@expr: calc($1 + $2)"                  → Expression in args (stored as raw)
```

## Bootstrap Return Annotation Nesting Rules

### ZERO NESTING POLICY
- **Objects**: Can contain scalars, simple arrays, or quantified arrays ONLY
- **Arrays**: Can contain scalars ONLY (no objects or nested arrays)
- **No Exceptions**: Any nesting beyond one level is REJECTED

### Nesting Examples

```rust
// ✅ ALLOWED (flat structures):
"$1"                           → Scalar
"[$1, $2, $3]"                → Simple array
"{name: $1, items: [$2*]}"    → Object with quantified array property
"{id: $1, data: [$2, $3]}"    → Object with simple array property

// ❌ REJECTED (nesting detected):
"{outer: {inner: $1}}"        → Object nesting
"[{id: $1}, {id: $2}]"       → Array of objects  
"[[1, 2], [3, 4]]"          → Array nesting
"{items: [{id: $1}]}"        → Object in array in object
```

## Error Handling

### Bootstrap Mode Warnings

When bootstrap mode is active:

```
WARNING: Bootstrap mode active - limited annotation parsing
  - Semantic annotations: name:value + simple functions (≤4 args)
  - Return annotations: FLAT structures only (≤3 object keys, NO nesting)
  - Complex patterns will be stored as raw strings
```

### Function Argument Limit Warnings

```
WARNING: Function with 5 arguments exceeds bootstrap limit (max: 4)
  - Pattern: @complex: func($1, $2, $3, $4, $5)
  - Stored as raw string - use full parser mode for complete support
```

### Nesting Detection Warnings

```
WARNING: Nested structure detected - exceeds bootstrap flat-only policy
  - Pattern: {outer: {inner: $1}}
  - Bootstrap mode supports FLAT structures only
  - Stored as raw string - use full parser mode for nesting support
```

## Usage Guidelines

### When to Use Bootstrap Mode

1. **Initial system build**: Building the AST pipeline for the first time
2. **Clean builds**: After `make clean-all` when generated parsers don't exist
3. **Simple grammars**: Using only flat annotations and simple functions
4. **Emergency recovery**: When external parsers are corrupted

### When NOT to Use Bootstrap Mode

1. **Complex grammars**: Any use of nested objects/arrays
2. **Advanced functions**: Functions with >4 arguments or nesting
3. **Production parsing**: Always prefer full parser mode
4. **Development/testing**: Full parser mode provides complete functionality

## Testing Strategy

### Test Cases

```rust
#[test]
fn test_bootstrap_semantic_function_call() {
    let pipeline = RustASTPipeline::new(PipelineConfig { bootstrap_mode: true, ..Default::default() });
    let result = pipeline.parse_semantic_annotation_bootstrap("transform: uppercase($1)").unwrap();
    assert_eq!(result, "transform:uppercase($1)");
}

#[test]
fn test_bootstrap_function_four_args() {
    let pipeline = RustASTPipeline::new(PipelineConfig { bootstrap_mode: true, ..Default::default() });
    let result = pipeline.parse_semantic_annotation_bootstrap("generate: make_obj($1, $2, $3, \"default\")").unwrap();
    assert!(result.contains("make_obj"));
}

#[test]
fn test_bootstrap_function_exceeds_limit() {
    let pipeline = RustASTPipeline::new(PipelineConfig { bootstrap_mode: true, ..Default::default() });
    let result = pipeline.parse_semantic_annotation_bootstrap("complex: func($1, $2, $3, $4, $5)").unwrap();
    assert!(result.starts_with("raw:"));
}

#[test]
fn test_bootstrap_return_flat_object() {
    let pipeline = RustASTPipeline::new(PipelineConfig { bootstrap_mode: true, ..Default::default() });
    let result = pipeline.parse_return_annotation_bootstrap("{name: $1, items: [$2*]}").unwrap();
    assert!(result.contains("\"properties\""));
}

#[test]
fn test_bootstrap_return_rejects_nesting() {
    let pipeline = RustASTPipeline::new(PipelineConfig { bootstrap_mode: true, ..Default::default() });
    let result = pipeline.parse_return_annotation_bootstrap("{outer: {inner: $1}}").unwrap();
    assert!(result.starts_with("raw:"));
}
```

## Clear Boundaries Summary

### Semantic Annotations Bootstrap Support:
- ✅ Simple name:value pairs
- ✅ Function calls with ≤4 simple arguments  
- ❌ Nested function calls
- ❌ Complex expressions
- ❌ Array/object values

### Return Annotations Bootstrap Support:
- ✅ Scalars: `$1`, `$2`
- ✅ Simple arrays: `[$1, $2]`, `[$1*]`
- ✅ Flat objects: `{key: $1}` (≤3 keys)
- ❌ **ZERO NESTING**: No `{a: {b: $1}}` or `[[...]]`
- ❌ **ZERO MULTI-LEVEL**: Strictly one level deep maximum

This specification draws clear, enforceable boundaries that make bootstrap mode implementation straightforward while covering the majority of real-world use cases.
