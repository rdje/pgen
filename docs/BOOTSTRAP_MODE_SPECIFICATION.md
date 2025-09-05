# Bootstrap Mode Specification

## Overview
Bootstrap mode provides built-in annotation parsing for the Rust AST pipeline to break circular dependencies during initial system builds.

## Purpose
- Enable clean builds from scratch without requiring external annotation parsers
- Provide essential annotation parsing capabilities during bootstrap phase
- Graceful fallback when full parsers are unavailable

## Semantic Annotation Bootstrap Parser

### ✅ Supported Patterns
- **Simple name:value pairs**: `generate: some_function()`
- **Simple identifiers**: `type: escape_literal_handling`
- **Function calls**: `validate(args...)` with up to 4 arguments
  - `process($1)`
  - `check($1, $2)`  
  - `transform($1, $2, $3)`
  - `analyze($1, $2, $3, $4)` (maximum)

### ❌ Unsupported Patterns (Fall Back to Raw)
- **Functions with >4 arguments**: `complex($1, $2, $3, $4, $5)`
- **Nested function calls**: `outer(inner($1))`
- **Complex expressions**: Method calls, chained operations, etc.

## Return Annotation Bootstrap Parser

### ✅ Supported Patterns - FLAT STRUCTURES ONLY

#### Scalars
- `$1`, `$2`, `$3`, etc. (any number)

#### Arrays  
- **Simple arrays**: `[$1, $2, $3, $4, ...]` (unlimited elements)
- **Quantified arrays**: `[$1*]`, `[$2+]`
- **Mixed arrays**: `[$1, $2*]`

#### Objects
- **Simple objects**: `{key1: $1, key2: $2, key3: $3, ...}` (unlimited keys)
- **Keys must be simple identifiers**: `name`, `type`, `value`, etc.
- **Values must be scalars or simple quantified references**: `$1`, `$2*`

### ❌ Unsupported Patterns (Fall Back to Raw)

#### Nesting (Strictly Forbidden)
- **Nested objects**: `{outer: {inner: $1}}` ❌
- **Objects in arrays**: `[{name: $1}, {name: $2}]` ❌  
- **Arrays in objects**: `{items: [$1, $2]}` ❌
- **Complex nesting**: `{data: {list: [$1*]}}` ❌

#### Complex Values
- **Dot notation**: `[$1.name, $2.value]` ❌
- **Dynamic keys**: `{$1: $2}` ❌
- **Function calls in values**: `{result: func($1)}` ❌

## Key Design Principles

### 1. **Flat Structures Only**
Bootstrap mode supports unlimited elements in arrays and objects but **ZERO nesting**. This keeps the parser simple while being practical for most annotation needs.

### 2. **Graceful Degradation**
When patterns exceed bootstrap capabilities, they are stored as raw strings with clear warning messages. The system continues functioning.

### 3. **Clear Boundaries**
Bootstrap mode has well-defined limits. Complex annotation parsing should use the full generated parsers.

### 4. **Practical Utility**
Despite limitations, bootstrap mode handles the majority of common annotation patterns used in real grammars.

## Implementation Notes

### Bootstrap Mode Activation - Automatic Fallback

**Important**: Bootstrap mode is **NOT** activated via command-line flags. It operates as an **automatic fallback mechanism**:

1. **Primary Attempt**: AST pipeline first tries to use external annotation parsers
2. **Automatic Fallback**: When external parsers fail/unavailable, bootstrap mode activates automatically
3. **No User Intervention**: This happens transparently - no special flags needed
4. **Logged Behavior**: You'll see messages like:
   ```
   Warning: External return parser failed, falling back to bootstrap mode: {...}
   ```

### Bootstrap Detection
```rust
fn should_use_bootstrap_mode(&self) -> bool {
    self.config.bootstrap_mode || !self.external_parsers_available()
}
```

### How It Actually Works in Practice

```bash
# This is all you need - bootstrap happens automatically as fallback:
./ast_pipeline --generate-parser input.json -o parser.rs

# The AST pipeline will:
# 1. Try external parsers first
# 2. Fall back to bootstrap mode if external parsers fail  
# 3. Log the fallback with clear warning messages
# 4. Continue processing successfully
```

### Fallback Strategy
```rust
// When pattern not recognized:
if self.config.debug {
    println!("WARNING: Pattern not recognized in bootstrap mode");
    println!("  Pattern: {}", annotation_value);
    println!("  Bootstrap mode supports FLAT structures only");
    println!("  Stored as raw string - use full parser mode for complete support");
}
Ok(format!("raw:{}", annotation_value))
```

### Error Handling
Bootstrap mode never fails - it either parses successfully or falls back to raw storage, ensuring the build process can always continue.

## Usage Example

### Build Process
```makefile
# Create placeholders first
$(BOOTSTRAP_PARSER_PLACEHOLDERS): 
    # Create minimal placeholder parsers

# Build AST pipeline with placeholders  
$(RUST_AST_PIPELINE): $(BOOTSTRAP_PARSER_PLACEHOLDERS)
    cd rust && cargo build

# Generate parsers using bootstrap mode
$(PARSERS): $(JSON_FILES) $(RUST_AST_PIPELINE)
    rust/target/debug/ast_pipeline --generate-parser --bootstrap-mode input.json -o parser.rs
```

### Command Line
```bash
# Use bootstrap mode explicitly
./ast_pipeline --bootstrap-mode --generate-parser input.json -o parser.rs

# Bootstrap mode activates automatically if external parsers unavailable
./ast_pipeline --generate-parser input.json -o parser.rs
```

## Benefits

1. **Eliminates Circular Dependencies**: System can build from completely clean state
2. **Practical Coverage**: Handles majority of real-world annotation patterns  
3. **Reliable Fallback**: Never blocks the build process
4. **Clear Boundaries**: Well-defined limits prevent complexity creep
5. **Unlimited Flat Elements**: Arrays and objects can have any number of top-level elements

This specification ensures bootstrap mode remains simple, reliable, and practical for real-world usage.
