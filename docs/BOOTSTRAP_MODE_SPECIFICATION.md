# Bootstrap Mode Specification

## Overview
Bootstrap mode provides built-in annotation parsing for the Rust AST pipeline to break circular dependencies during initial system builds.

## Purpose
- Enable clean builds from scratch without requiring external annotation parsers
- Provide essential annotation parsing capabilities during bootstrap phase
- Graceful fallback when full parsers are unavailable

## Two Surfaces, One Language

There are two distinct parser surfaces for semantic-annotation source. Both are kept consistent in lockstep — what one accepts the other accepts.

1. **EBNF-language surface — `grammars/semantic_annotation.ebnf` → `generated/semantic_annotation_parser.rs`.** Parses semantic-annotation `*.ebnf` source text and freestanding annotation strings passed through the `parse_annotation` embedding API. This is the formal language definition.

2. **Grammar directive-payload runtime — `rust/src/ast_pipeline/unified_semantic_ast.rs::StructuredSemanticValueParser` (hand-rolled).** Invoked via `UnifiedSemanticAST::parse_bootstrap` → `parse_structured_payload` whenever the AST pipeline reads a grammar's `@directive: { … }` annotations during parser generation. This is the surface real grammar authors hit when adding directives to their `.ebnf`.

When the two diverge, the runtime is what actually parses grammar payloads — surface (1) is silent. Changes to "what `$<ref>` accepts" must touch BOTH surfaces in lockstep. The `SV-EXH-PROOF.3.3.4.a.1` slice (`PGEN-SV-EXH-PROOF-0026`, 2026-05-20) surfaced this asymmetry — regex-only edits to the EBNF were no-ops for grammar directive payloads until the hand-rolled `parse_rule_reference` was also extended.

## Semantic Annotation Bootstrap Parser

### ✅ Supported Patterns
- **Simple name:value pairs**: `generate: some_function()`
- **Simple identifiers**: `type: escape_literal_handling`
- **Function calls**: `validate(args...)` with up to 4 arguments
  - `process($1)`
  - `check($1, $2)`  
  - `transform($1, $2, $3)`
  - `analyze($1, $2, $3, $4)` (maximum)
- **Rule references — depth-unbounded** (SV-EXH-PROOF.3.3.4.a.1 / .a.2, 2026-05-20):
  - Simple named: `$name`, `$body`, `$pkg`
  - Simple positional (1-indexed): `$1`, `$42`
  - Dotted property-access chains, unbounded depth: `$name.body`, `$1.body.subkey`, `$a.b.c.d.e.f.g.h…`
  - Non-negative integer indexed-access chains, unbounded depth: `$items[0]`, `$1[0]`, `$matrix[0][1][2]`
  - Mixed dotted + indexed chains, unbounded depth: `$items[0].name`, `$a.b[0].c[1].d.e[2].r.z`
  - Strict-bracket / strict-trailing-dot policy: malformed forms (bare `.`, `[` with no `<digits>]`) roll back to before the offending segment; the surrounding payload parser then handles the leftover or falls back to `Raw`.
  - Durable no-depth-limit guarantee locked by two regression tests in `rust/src/ast_pipeline/unified_semantic_ast.rs::tests` exercising 64 segments each (one pure-dotted, one mixed dotted + indexed).
  - Subset boundary: dotted property + non-negative integer indexing ONLY. NOT full JSONPath (no filters `[?(@.foo)]`, no wildcards `*`, no recursive descent `..`).

### ❌ Unsupported Patterns (Fall Back to Raw)
- **Functions with >4 arguments**: `complex($1, $2, $3, $4, $5)`
- **Nested function calls**: `outer(inner($1))`
- **Complex expressions**: Method calls, chained operations, etc.
- **JSONPath features beyond the subset**: filters `[?(@.foo > 0)]`, wildcards `*`, recursive descent `..`, negative indices `[-1]`, range slices `[0:5]`.

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

**Important**: Bootstrap mode normally operates as an **automatic fallback mechanism**, but the CLI still exposes `--bootstrap-mode` as a forcing/debug path for regeneration workflows and proof gates.

1. **Primary Attempt**: AST pipeline first tries to use external annotation parsers
2. **Automatic Fallback**: When external parsers fail/unavailable, bootstrap mode activates automatically
3. **No User Intervention**: This happens transparently - no special flags needed
4. **Logged Behavior**: You'll see messages like:
   ```
   Warning: External return parser failed, falling back to bootstrap mode: {...}
   ```

## Bootstrap Architecture Notes

This document now also absorbs the useful architecture residue from the retired `docs/BOOTSTRAP_SYSTEM_COMPLETE.md`.

### Three-Level Bootstrap Shape

1. **Built-in Rust fallback parsers**
   - small semantic / return-annotation support used when generated parsers are unavailable
2. **Generated special parsers**
   - `grammars/semantic_annotation.ebnf`
   - `grammars/return_annotation.ebnf`
3. **Everything else**
   - the wider grammar surface generated once the special parsers are available again

### Critical Files

- `rust/src/ast_pipeline/mod.rs`
  - bootstrap-mode configuration and fallback flow
- `rust/src/ast_pipeline/unified_return_ast.rs`
  - bootstrap return-annotation parsing support
- `grammars/semantic_annotation.ebnf`
  - semantic-annotation grammar used by the generated path
- `grammars/return_annotation.ebnf`
  - return-annotation grammar used by the generated path

### Practical Build Summary

1. Start from a clean state or placeholder-generated state.
2. Build the Rust AST pipeline.
3. Use bootstrap fallback or explicit `--bootstrap-mode` where needed to regenerate the special parsers.
4. Rebuild with the real generated parsers available.
5. Regenerate the wider parser surface and verify it through the retained proof gates.

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
