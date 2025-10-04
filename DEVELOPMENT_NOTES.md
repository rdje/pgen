# DEVELOPMENT_NOTES.md

## 2025-10-04 - Semantic Annotations Architecture: JSON AST Extraction

### Clean Architecture: EBNF Parser → JSON → Rust Extraction

**Successfully implemented semantic annotation extraction from JSON AST tokens.**

#### Architecture Overview

##### 1. EBNF Parser Integration
The EBNF parser correctly recognizes `@transform:` annotations and embeds them in JSON as:
```json
[
  "semantic_annotation",
  [
    "transform", 
    "str::parse::<f64>().unwrap_or(0.0)"
  ]
]
```

##### 2. JSON AST Structure
- **Rule tokens**: `["rule", "float"]`
- **Semantic annotation tokens**: `["semantic_annotation", ["transform", "str::parse::<f64>().unwrap_or(0.0)"]]`  
- **Regex tokens**: `["regex", "[-+]?[0-9]+\\.[0-9]+(?:[eE][-+]?[0-9]+)?"]`

##### 3. Rust Pipeline Extraction
The AST pipeline extracts semantic annotations during the `extract_annotations` phase:
```rust
if let TokenValue::Array(annotation_data) = &token[1] {
    let annotation_name_str = annotation_data[0].clone();  // "transform"
    let annotation_value_str = annotation_data[1].clone(); // "str::parse::<f64>().unwrap_or(0.0)"
    // Store in pipeline.annotations.semantic_annotations
}
```

##### 4. Bootstrap System
Cargo features resolve circular dependencies:
```toml
[features]
default = []
bootstrap = []
```

- **Compile time**: `cargo build --features bootstrap` excludes external parsers
- **Runtime**: `ast_pipeline --bootstrap-mode` uses built-in parsers

#### Implementation Status

##### ✅ Completed
- EBNF parser correctly embeds semantic annotations in JSON
- JSON AST uses structured `["semantic_annotation", [<name>, <value>]]` tokens
- Rust pipeline extracts annotations from JSON AST
- Bootstrap compilation works for circular dependency resolution
- Semantic annotations stored in `pipeline.annotations.semantic_annotations`

##### ⏳ Next Steps  
- AST-based generator needs to use semantic annotations for code generation
- Generate transformation code like `str::parse::<f64>().unwrap_or(0.0)` for matched terminals
- Add debug logging for semantic annotation code generation

#### Technical Details

##### TokenValue Enum
```rust
pub enum TokenValue {
    String(String),
    Array(Vec<String>),  // Semantic annotations use this format
}
```

##### Annotation Storage
```rust
pub struct Annotations {
    pub semantic_annotations: HashMap<String, Vec<String>>,  // rule_name -> [annotation_strings]
    // ...
}
```

##### Bootstrap Makefile
```makefile
$(RUST_AST_PIPELINE): $(AST_PIPELINE_SOURCES)
cd $(RUST_DIR) && cargo build --bin ast_pipeline --features bootstrap
```
