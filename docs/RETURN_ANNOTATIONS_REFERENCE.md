# Return Annotations Reference Guide

> **The definitive guide to return annotations in pgen parser generator**

**Status:** This document describes features under active development. The syntax and semantics are being designed and refined.

Return annotations are a powerful feature that allows you to transform the raw parse tree into meaningful data structures. This document serves as the complete reference for understanding, writing, and debugging return annotations.

## Table of Contents

1. [Introduction](#introduction)
2. [Basic Concepts](#basic-concepts)
3. [Syntax Reference](#syntax-reference)
4. [Positional References](#positional-references)
5. [Arrays and Spreading](#arrays-and-spreading)
6. [Objects and Properties](#objects-and-properties)
7. [Quantified Groups and Extraction](#quantified-groups-and-extraction)
8. [Bootstrap Mode Support](#bootstrap-mode-support)
9. [Advanced Patterns](#advanced-patterns)
10. [Common Pitfalls](#common-pitfalls)
11. [Migration Guide](#migration-guide)
12. [Implementation Details](#implementation-details)

---

## Introduction

Return annotations transform parse results at parse time, eliminating the need for separate AST transformation passes. They are specified using the `->` operator after rule definitions.

### Quick Example
```ebnf
# Without return annotation - returns raw parse tree
identifier := /[a-zA-Z_]\w*/

# With return annotation - returns just the matched text
identifier := /([a-zA-Z_]\w*)/ -> $1
```

### Why Return Annotations Matter

1. **Simplify AST construction** - Build the exact data structure you need
2. **Reduce post-processing** - Transform data during parsing, not after
3. **Self-documenting grammar** - The return structure is visible in the grammar
4. **Type-safe generation** - Generate parsers with predictable return types

---

## Basic Concepts

### The Arrow Operator (`->`)

The `->` operator separates the pattern from its return annotation:

```ebnf
rule_name := pattern -> return_annotation
```

### Implicit Passthrough

When no return annotation is specified, or when using branch alternatives, the default behavior is to pass through the entire matched content (implicit `-> $1`).

```ebnf
# These are equivalent:
simple := "hello"
simple := "hello" -> $1
```

---

## Syntax Reference

### Core Return Types

| Syntax | Description | Example |
|--------|-------------|---------|
| `$N` | Positional reference to Nth element | `$1`, `$2`, `$3` |
| `"string"` | String literal | `"array"`, `"object"` |
| `number` | Number literal | `42`, `3.14` |
| `true`/`false` | Boolean literal | `true`, `false` |
| `[...]` | Array construction | `[$1, $2]` |
| `{...}` | Object construction | `{type: "node", value: $1}` |

### Special Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `*` | Spread operator | `$2*` spreads all elements of $2 |
| `::` | Extraction operator | `$2::3` extracts element 3 from each in $2 |
| `.` | Property access | `$1.value` accesses the value property |
| `[N]` | Array indexing | `$1[0]` accesses first element |

---

## Positional References

Positional references (`$N`) refer to elements in the parsed sequence, using **1-based indexing**.

### Basic Positional References

```ebnf
# Pattern with 3 elements
pair := key ':' value -> {key: $1, value: $3}
#       $1   $2  $3

# Pattern with whitespace
spaced := word /\s+/ word -> [$1, $3]
#         $1    $2    $3
```

### Counting Elements

Each element in the pattern gets a position number:

```ebnf
complex := '(' identifier operator number ')' -> {
    op: $3,      # The operator
    left: $2,    # The identifier  
    right: $4    # The number
}
# Positions: $1='(', $2=identifier, $3=operator, $4=number, $5=')'
```

### Important Notes

- **Whitespace patterns count**: `/\s*/` is an element and gets a position
- **Literals count**: Every quoted string gets a position
- **Groups count as one**: `(a b c)` is one element, not three

---

## Arrays and Spreading

### Array Construction

Arrays are created using square brackets:

```ebnf
# Simple array
list := item item item -> [$1, $2, $3]

# Mixed types
mixed := number identifier string -> [$1, $2, $3]
```

### Spread Operator (`*`)

The spread operator unpacks array/sequence contents:

```ebnf
# Spreading a quantified group
items := first rest* -> [$1, $2*]
# If rest matches [a, b, c], result is [first, a, b, c]

# Combining spreads
merged := list1 list2 -> [...$1, ...$2]  # Explicit spread syntax (future)
merged := list1 list2 -> [$1*, $2*]      # Current syntax
```

### Nested Arrays

Arrays can contain any return expression:

```ebnf
matrix := row row row -> [
    [$1],
    [$2],
    [$3]
]

nested := item (',' item)* -> [$1, [$2*]]
```

---

## Objects and Properties

### Object Construction

Objects are created using curly braces with key-value pairs:

```ebnf
# Simple object
node := type ':' value -> {type: $1, value: $3}

# Multiple properties
detailed := name age city -> {
    name: $1,
    age: $2,
    location: $3
}
```

### Property Keys

Property keys can be:
- **Identifiers**: `{type: $1}`
- **String literals**: `{"type": $1}` 
- **Computed** (future): `{[$1]: $2}`

### Nested Objects

Objects can contain any return expression:

```ebnf
person := name address -> {
    name: $1,
    address: {
        street: $2.street,
        city: $2.city
    }
}
```

---

## Quantified Groups and Extraction

This is where return annotations become incredibly powerful.

### Understanding Quantified Groups

When a quantified group contains a sequence, each repetition includes all elements in that sequence:

```ebnf
# Pattern with quantified group
index_list := index (',' /\s*/ index)*
#             $1     $2 (entire quantified group)

# Each repetition of $2 contains 3 elements:
# [1] ','
# [2] /\s*/  
# [3] index  <-- Often we want just these
```

### Extraction Operators

Use the double colon operator (`::`) to extract specific elements from quantified groups:

```ebnf
# Extract element at index 2 from each repetition
index_list := index (',' /\s*/ index)* -> [$1, $2::3*]

# Without spreading (keeps as nested array)
index_list := index (',' /\s*/ index)* -> [$1, $2::2]

# Extract last element from each repetition
items := item (sep meta data)* -> [$1, $2::last*]
```

#### How It Works

- `$2::1` - Extract first element from each repetition (array index 0)
- `$2::2` - Extract second element from each repetition (array index 1)
- `$2::1*` - Extract first element and spread into parent array
- `$2::first` - Extract first element from each repetition
- `$2::last` - Extract last element from each repetition

### Extraction Examples

```ebnf
# Extract fourth element (the second index) from each repetition
list := item (',' /\s*/ item)* -> [$1, $2::4*]
# Input: "a, b, c, d"
# Result: ["a", "b", "c", "d"]

# Keep extracted elements as nested array
grouped := item (',' item)* -> [$1, $2::1]
# Input: "a, b, c"
# Result: ["a", [",", ","]]

# Extract from complex patterns
params := param (',' param)* -> {
    params: [$1, $2::2*]  # Extract second element (param) from each repetition
}
```

---

## Bootstrap Mode Support

The bootstrap parser supports an essential subset of return annotation features for self-hosting.

### Supported Features

```ebnf
# Positional references
simple := a b c -> $2

# Arrays with spreading
list := first rest* -> [$1, $2*]

# Simple objects (≤3 properties)
node := type value -> {type: $1, value: $2}

# Basic extraction with double colon operator
indices := num (',' num)* -> [$1, $2::2*]

# String literals
constant := anything -> "fixed_value"
```

### Bootstrap Mode Limitations

- Nested objects limited to depth 1
- Maximum 3 properties per object  
- Complex property access chains not supported
- Only `::` extraction operator supported

---

## Planned Features

### Conditional Returns
```ebnf
value := number | string | 'null' -> $1 ? $1 : "default"
```

### Computed Property Names  
```ebnf
pair := key '=' value -> {[$1]: $2}
```

### Transform Functions
```ebnf
number := /\d+/ -> parseInt($1)
upper := /\w+/ -> toUpper($1)  
```

---

## Implementation Details

### AST Node Types

Return annotations are parsed into a unified AST with these node types:

```rust
enum UnifiedReturnAST {
    // Basic types
    PositionalRef { index: usize },
    StringLiteral { value: String },
    NumberLiteral { value: f64 },
    BooleanLiteral { value: bool },
    
    // Collections
    Array { elements: Vec<UnifiedReturnAST> },
    Object { properties: HashMap<String, Box<UnifiedReturnAST>> },
    
    // Operators
    Spread { base: Box<UnifiedReturnAST> },
    PropertyAccess { base: Box<UnifiedReturnAST>, property: String },
    ArrayAccess { base: Box<UnifiedReturnAST>, index: Box<UnifiedReturnAST> },
    
    // Extraction from quantified groups
    QuantifiedExtraction {
        base: Box<UnifiedReturnAST>,
        extraction_type: ExtractionType,
        target: ExtractionTarget,
    },
    
    // Default
    Passthrough,
}
```

### Code Generation

Return annotations generate runtime code that transforms parse results:

```rust
// For: {type: "object", key: $3, value: $7}
let mut json_obj = serde_json::json!({});
json_obj[r#"type"#] = serde_json::json!(r#"object"#);
json_obj[r#"key"#] = serde_json::json!(
    match &sequence_elements[2].content {
        ParseContent::Terminal(s) => s.to_string(),
        // ... extraction logic
    }
);
// ... etc
```

### Captured Variables

The system tracks available captured variables:

- **Sequences**: `sequence_elements[0]`, `sequence_elements[1]`, etc.
- **Alternatives**: `result` for each branch
- **Quantified**: Special handling for `*`, `+`, `?`

---

## Examples Gallery

### Basic Patterns

```ebnf
# Simple passthrough
identifier := /\w+/ -> $1

# String literal
keyword := 'class' | 'function' -> "keyword"

# Array construction
pair := left right -> [$1, $2]

# Object construction  
node := op arg1 arg2 -> {operation: $1, args: [$2, $3]}
```

### Working with Lists

```ebnf
# Extract items from comma-separated list
list := '[' item (',' item)* ']' -> [$2, $3::1*]

# Build structured result
function_call := name '(' arg (',' arg)* ')' -> {
    name: $1,
    args: [$3, $4::1*]
}
```

### Complex Structures

```ebnf
# Parse key-value pairs
object := '{' pair (',' pair)* '}' -> {
    type: "object",
    pairs: [$2, $3::1*]
}

# Extract from nested patterns
statement := 'if' '(' condition ')' block 'else' block -> {
    type: "if_else",
    condition: $3,
    then_block: $5,
    else_block: $7
}
```

---

## Quick Reference

### Syntax Summary

| Pattern | Description | Example |
|---------|-------------|----------|
| `$N` | Position reference | `$1`, `$2`, `$3` |
| `[...]` | Array | `[$1, $2]` |
| `{...}` | Object | `{key: $1, val: $2}` |
| `*` | Spread | `$2*` |
| `::` | Extract from quantified | `$2::1*` |
| `"..."` | String literal | `"keyword"` |

---

## File Locations

- **Implementation**: `rust/src/ast_pipeline/unified_return_ast.rs`
- **Tests**: `tests/test_universal_return_annotations.pl`
- **Bootstrap parser**: `rust/src/ast_pipeline/unified_return_ast.rs`

---

*This document is the authoritative reference for return annotations in pgen.*
