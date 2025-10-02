# AST-Based Parser Generator

## Overview

The AST-based parser generator is a robust solution that leverages Rust's `syn` and `quote` crates to generate parser code through structured AST manipulation rather than error-prone string concatenation. This approach eliminates common issues with mismatched braces, unbalanced parentheses, and malformed syntax that plague string-based code generation.

## Architecture

### Core Components

1. **`ast_based_generator.rs`** - Main generator using syn/quote macros
2. **`ast_code_generator.rs`** - Helper for generating specific code patterns
3. **`ast_return_transform.rs`** - Enhanced return annotation handling
4. **`generator_adapter.rs`** - Unified interface for gradual migration
5. **`ast_generator_tests.rs`** - Comprehensive test suite

### Key Features

- **Compile-time syntax validation** - Invalid syntax is caught during macro expansion
- **Structured code generation** - Uses Rust AST nodes instead of strings
- **Automatic delimiter balancing** - Braces and parentheses are always matched
- **Type-safe code construction** - Leverages Rust's type system
- **Composable AST constructs** - Build complex structures from simple parts

## Migration Strategy

The `UnifiedGenerator` adapter provides a seamless migration path:

```rust
// Automatic backend selection based on grammar complexity
let generator = UnifiedGenerator::new_auto("my_parser", &grammar);

// Or explicit backend selection
let generator = UnifiedGenerator::new("my_parser", GeneratorBackend::AstBased);

// Generate parser with automatic fallback on error
let code = generator.generate_parser(&grammar, &rule_order, None)?;
```

### Backend Selection Criteria

The adapter automatically selects the appropriate backend:

- **String-based** (default for simple grammars):
  - Less than 10 rules
  - No complex nested structures
  - Simple alternatives and sequences

- **AST-based** (for complex grammars):
  - 10+ rules
  - Deep nesting (3+ levels)
  - Complex quantified groups
  - Many alternatives per rule

## Code Generation Examples

### Simple Rule
```rust
// Input: identifier := /[a-zA-Z_][a-zA-Z0-9_]*/
let ast = quote! {
    fn parse_identifier(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_IDENTIFIER, |parser| {
            let result = parser.match_regex(r"[a-zA-Z_][a-zA-Z0-9_]*")?;
            Ok(ParseNode {
                rule_name: "identifier",
                content: ParseContent::Terminal(result),
                span: start_pos..parser.position,
            })
        })
    }
};
```

### Alternatives
```rust
// Input: operator := '+' | '-' | '*' | '/'
let ast = quote! {
    fn parse_operator(&mut self) -> ParseResult<ParseNode<'input>> {
        self.memoized_call(Self::RULE_OPERATOR, |parser| {
            for (i, alt) in ["+", "-", "*", "/"].iter().enumerate() {
                if let Some(result) = parser.try_parse(|p| {
                    p.match_string(*alt)
                }) {
                    return Ok(ParseNode {
                        rule_name: "operator",
                        content: ParseContent::Terminal(result),
                        span: start_pos..parser.position,
                    });
                }
            }
            Err(ParseError::NoMatch)
        })
    }
};
```

### Return Annotations
```rust
// Input: expr := left operator right -> {op: $2, operands: [$1, $3]}
let return_transform = quote! {
    let mut json_obj = serde_json::json!({});
    json_obj["op"] = serde_json::json!(elements[1]);
    json_obj["operands"] = serde_json::json!([&elements[0], &elements[2]]);
    ParseContent::Terminal(serde_json::to_string(&json_obj)?)
};
```

## Benefits Over String-Based Generation

### 1. Syntax Correctness
- **String-based**: Can generate `}};};` or miss closing braces
- **AST-based**: Syntax errors caught at macro expansion time

### 2. Code Structure
- **String-based**: Manual indentation and formatting
- **AST-based**: Automatic proper formatting via rustfmt

### 3. Refactoring Safety
- **String-based**: Find-and-replace can break code
- **AST-based**: Structured transformations preserve correctness

### 4. Debugging
- **String-based**: Hard to trace where bad code originated
- **AST-based**: Clear stack traces to the macro that generated code

### 5. Type Safety
- **String-based**: Runtime errors from type mismatches
- **AST-based**: Type checking during code generation

## Testing

The comprehensive test suite in `ast_generator_tests.rs` covers:

- Basic grammar structures (atoms, sequences, alternatives)
- Quantifiers (*, +, ?)
- Return annotations (scalars, arrays, objects)
- Complex nested structures
- Single-branch edge cases
- Backend selection logic
- Output comparison between backends

Run tests with:
```bash
cargo test ast_generator_tests
```

## Performance Considerations

The AST-based generator has different performance characteristics:

- **Compilation**: Slightly slower due to macro expansion
- **Runtime**: Identical performance (generates same code patterns)
- **Memory**: Higher during generation (AST nodes vs strings)
- **Reliability**: Significantly better (no syntax errors)

For most use cases, the reliability improvement far outweighs the minimal compilation overhead.

## Future Enhancements

1. **Optimization passes** - AST-level optimizations before code generation
2. **Custom derive macros** - Generate parsers from struct definitions
3. **Incremental generation** - Only regenerate changed rules
4. **Cross-language targets** - Generate parsers for other languages
5. **Visual AST editor** - GUI for building parser grammars

## Troubleshooting

### Common Issues

1. **Macro recursion limit**
   - Solution: Add `#![recursion_limit = "256"]` to lib.rs

2. **Large generated files**
   - Solution: Use include! macro to keep generated code separate

3. **Slow compilation**
   - Solution: Use incremental compilation and cargo check

### Debug Tips

1. Enable macro expansion visualization:
   ```bash
   cargo expand --lib ast_based_generator
   ```

2. Check generated code structure:
   ```rust
   let tokens = generator.generate_tokens(&grammar);
   println!("{}", tokens);
   ```

3. Validate AST construction:
   ```rust
   let ast = syn::parse_str::<syn::File>(&generated_code)?;
   ```

## Integration Guide

### Step 1: Update Dependencies
```toml
[dependencies]
syn = { version = "2.0", features = ["full", "extra-traits"] }
quote = "1.0"
proc-macro2 = "1.0"
```

### Step 2: Configure Pipeline
```rust
use pgen::ast_pipeline::generator_adapter::{UnifiedGenerator, GeneratorBackend};

let mut pipeline = RustASTPipeline::new();
pipeline.config.use_ast_generator = true; // Future config option
```

### Step 3: Generate Parsers
```rust
let generator = UnifiedGenerator::new_auto(parser_name, &grammar);
let code = generator.generate_parser(&grammar, &rule_order, Some(&annotations))?;
std::fs::write(output_path, code)?;
```

### Step 4: Verify Output
```rust
// Compile and test generated parser
std::process::Command::new("rustc")
    .arg("--edition=2021")
    .arg("-o")
    .arg("/dev/null")
    .arg(&output_path)
    .status()?;
```

## Conclusion

The AST-based parser generator represents a significant improvement in code generation reliability and maintainability. By leveraging Rust's powerful macro system and AST manipulation capabilities, we eliminate entire classes of bugs while maintaining identical runtime performance. The gradual migration path through the UnifiedGenerator ensures existing code continues to work while new features benefit from the improved generation approach.