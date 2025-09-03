# Rust AST Pipeline Semantic Annotation Support

## Overview

This document details the implementation of semantic annotation parsing support in the Rust AST transformation pipeline. This enhancement enables the Rust pipeline to correctly parse and preserve semantic annotations from annotated EBNF grammars, maintaining compatibility with the broader semantic annotation system documented in `SEMANTIC_ANNOTATIONS_ANALYSIS.md`.

## Technical Architecture

### TokenValue Enum Enhancement

The core enhancement introduces a flexible token representation system to handle mixed content types in raw AST structures.

#### Definition

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TokenValue {
    String(String),
    Array(Vec<String>),
}
```

#### Key Features

- **Untagged Serialization**: Uses `#[serde(untagged)]` for seamless JSON deserialization
- **Mixed Content Support**: Handles both traditional string tokens and array-structured annotation data
- **Trait Implementations**: Full trait support for common operations

#### Trait Implementations

```rust
impl TokenValue {
    /// Get as string reference if this is a String variant
    pub fn as_str(&self) -> Option<&str> {
        match self {
            TokenValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }
    
    /// Check if this is an empty string
    pub fn is_empty(&self) -> bool {
        match self {
            TokenValue::String(s) => s.is_empty(),
            TokenValue::Array(v) => v.is_empty(),
        }
    }
}

impl std::fmt::Display for TokenValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenValue::String(s) => write!(f, "{}", s),
            TokenValue::Array(v) => write!(f, "{:?}", v),
        }
    }
}

impl PartialEq<&str> for TokenValue {
    fn eq(&self, other: &&str) -> bool {
        match self {
            TokenValue::String(s) => s == *other,
            _ => false,
        }
    }
}
```

### Type System Changes

#### Updated Type Definitions

```rust
pub type Token = Vec<TokenValue>;
pub type TokenSequence = Vec<Token>;
pub type RawAST = Vec<TokenSequence>;
```

These changes cascade through the entire pipeline, requiring updates to all transformation stages.

### Annotation Extraction Enhancement

#### Core Extraction Logic

The `extract_annotations()` method has been enhanced to handle the new semantic annotation format:

```rust
"semantic_annotation" | "logging_annotation" => {
    if let Some(ref name) = rule_name {
        if self.config.preserve_annotations {
            // New format: token[1] is the annotation array [name, value]
            if let TokenValue::Array(annotation_data) = &token[1] {
                if annotation_data.len() >= 2 {
                    let annotation_name = &annotation_data[0];
                    let annotation_value = &annotation_data[1];
                    
                    match token_type.as_str() {
                        "semantic_annotation" => {
                            let formatted_annotation = format!("{}:{}", annotation_name, annotation_value);
                            self.annotations.semantic_annotations
                                .entry(name.clone())
                                .or_insert_with(Vec::new)
                                .push(formatted_annotation);
                        }
                        "logging_annotation" => {
                            let formatted_annotation = format!("{}({})", annotation_name, annotation_value);
                            self.annotations.logging_annotations
                                .entry(name.clone())
                                .or_insert_with(Vec::new)
                                .push(formatted_annotation);
                        }
                        _ => unreachable!(),
                    }
                    
                    if self.config.debug {
                        println!("Parsed {} annotation: {} = {}", token_type, annotation_name, annotation_value);
                    }
                }
            }
            // Fallback for old string format omitted for brevity
            self.stats.annotations_preserved += 1;
        }
    }
}
```

#### Annotation Format Support

**Input Format (JSON AST):**
```json
[
   "semantic_annotation", 
   [
      "type",
      "\"context_sensitive_construct\""
   ]
]
```

**Parsed Format (Internal Storage):**
```rust
// Stored as: "type:\"context_sensitive_construct\""
// In: self.annotations.semantic_annotations["rule_name"]
```

### Pipeline Stage Updates

#### Stage 2: OR Operator Grouping

```rust
fn group_by_or_operators(&self, ast: &RawAST) -> Result<HashMap<String, Vec<TokenSequence>>> {
    // Updated to handle TokenValue enum in rule detection and operator matching
    for token in rule_def {
        if token.len() == 2 {
            if let (TokenValue::String(type_str), TokenValue::String(name_str)) = (&token[0], &token[1]) {
                if type_str == "rule" {
                    rule_name = Some(name_str.clone());
                    break;
                }
            }
        }
    }
    // ... operator matching logic updated similarly
}
```

#### Stage 2.5: Parentheses Processing

```rust
fn process_parentheses_in_sequence(&self, sequence: &TokenSequence) -> Result<TokenSequence> {
    // Updated token comparison logic
    if token.len() == 2 && token[0] == "group_open" {
        // Find matching close with updated string access
        while j < sequence.len() && paren_count > 0 {
            if sequence[j].len() == 2 {
                if let Some(token_str) = sequence[j][0].as_str() {
                    match token_str {
                        "group_open" => paren_count += 1,
                        "group_close" => paren_count -= 1,
                        _ => {}
                    }
                }
            }
            // ...
        }
        
        // Generate group token with proper TokenValue construction
        result.push(vec![
            TokenValue::String("group".to_string()), 
            TokenValue::String(content_json)
        ]);
    }
}
```

#### Stage 3: Sequence Parsing

```rust
fn parse_single_element(&self, element: &Token) -> Result<ASTNode> {
    let token_type = &element[0];
    let token_value = &element[1];

    match token_type.as_str() {
        Some("group") => {
            // Deserialize group content with proper string handling
            if let TokenValue::String(json_str) = token_value {
                let group_content: TokenSequence = serde_json::from_str(json_str)
                    .context("Failed to deserialize group content")?;
                // ... rest of group processing
            } else {
                Ok(ASTNode::Atom { value: ASTValue::Token(element.clone()) })
            }
        }
        _ => Ok(ASTNode::Atom { value: ASTValue::Token(element.clone()) })
    }
}
```

#### Stage 4: Quantifier Handling

```rust
fn apply_quantifiers_to_node(&self, node: ASTNode) -> Result<ASTNode> {
    // Updated quantifier detection with proper TokenValue handling
    if let ASTNode::Atom { value: ASTValue::Token(token) } = &elements[i + 1] {
        if token.len() == 2 && token[0] == "operator" {
            if let Some(op_str) = token[1].as_str() {
                if ["*", "+", "?"].contains(&op_str) {
                    let quantified_node = ASTNode::Quantified {
                        element: Box::new(element.clone()),
                        quantifier: op_str.to_string(),
                    };
                    // ...
                }
            }
        }
    }
}
```

## Data Flow Architecture

### Input Processing

1. **Raw JSON Loading**: JSON with mixed string/array tokens deserialized via TokenValue enum
2. **Annotation Detection**: Array-formatted annotations identified during token processing  
3. **Extraction**: Annotation name/value pairs extracted and formatted for storage
4. **Metadata Preservation**: Annotations stored in structured format in transformation metadata

### Pipeline Compatibility

```
Raw AST JSON → TokenValue Parsing → Annotation Extraction → 
Pipeline Stages (2-5) → Final AST → Metadata Preservation
```

Each stage handles TokenValue enum consistently, maintaining annotation integrity throughout.

### Output Structure

**Transformed AST Metadata:**
```rust
pub struct TransformMetadata {
    pub format: String,
    pub source_format: String,
    pub transformed_at: String,
    pub transformer: String,
    pub pipeline_stage: String,
    pub annotations: Annotations,  // ← Semantic annotations preserved here
    pub stats: TransformStats,
}

pub struct Annotations {
    pub semantic_annotations: HashMap<String, Vec<String>>, // rule_name -> [formatted_annotations]
    pub logging_annotations: HashMap<String, Vec<String>>,
    pub return_annotations: HashMap<String, String>,
}
```

## Testing and Validation

### Test Configuration

**Debug Mode Activation:**
```bash
cargo run -- ../generated/regex_raw_cleaned.json ../generated/regex_transformed_fixed.json --debug --stats
```

**Expected Debug Output:**
```
=== Rust AST Transformation Pipeline ===
Stage 1: Extracting annotations...
Parsed semantic_annotation annotation: type = "context_sensitive_construct"
Parsed semantic_annotation annotation: description = "Character class with proper bracket handling"
Parsed semantic_annotation annotation: constraint = "must properly handle ']' as literal vs terminator"
Preserved 18 annotations
```

### Validation Metrics

- **Annotation Recognition**: ✅ Correctly identifies array-format semantic annotations
- **Format Handling**: ✅ Parses `["type", "\"value\""]` format properly  
- **Pipeline Integration**: ✅ All 5 transformation stages handle TokenValue enum
- **Metadata Preservation**: ✅ Annotations available in final transformed AST
- **Statistics**: ✅ Accurate counts of preserved annotations
- **Backward Compatibility**: ✅ Fallback handling for legacy formats

### Error Handling

**Malformed Annotation Handling:**
```rust
// Fallback for old string format or malformed data
if self.config.debug {
    println!("Warning: Unexpected annotation format for {}: {:?}", token_type, token[1]);
}
match token_type.as_str() {
    "semantic_annotation" => {
        self.annotations.semantic_annotations
            .entry(name.clone())
            .or_insert_with(Vec::new)
            .push(format!("raw:{:?}", token[1]));
    }
    // ... similar for logging annotations
}
```

## Integration with Domain-Specific DataGenerators

### Annotation Access Pattern

```rust
// Access semantic annotations from transformed AST
let transformed_ast: TransformedASTJson = load_transformed_ast()?;

for (rule_name, rule_node) in &transformed_ast.grammar_tree {
    if let Some(rule_annotations) = transformed_ast.metadata.annotations.semantic_annotations.get(rule_name) {
        for annotation in rule_annotations {
            // Parse annotation: "name:value" format
            if let Some((name, value)) = annotation.split_once(':') {
                match name {
                    "type" => handle_type_annotation(rule_name, value),
                    "constraint" => handle_constraint_annotation(rule_name, value), 
                    "description" => handle_description_annotation(rule_name, value),
                    // ... domain-specific handling
                }
            }
        }
    }
}
```

### Domain-Specific Extensions

As outlined in `SEMANTIC_ANNOTATIONS_ANALYSIS.md`, this infrastructure supports:

- **SystemVerilog DataGenerator**: Using annotations like `@bit_width`, `@clock_domain`
- **VHDL DataGenerator**: Using annotations like `@signal_type`, `@range_direction`  
- **Generic DataGenerator**: Using annotations like `@range`, `@format`, `@examples`

## Future Enhancements

### Immediate Improvements

1. **High-Performance Generator**: Complete TokenValue support in `high_performance_generator.rs`
2. **Comprehensive Testing**: Full test suite with various annotation formats
3. **Performance Optimization**: Minimize string allocations during token processing

### Long-term Extensions

1. **Streaming Processing**: Support for very large AST structures
2. **Validation**: Schema validation for annotation values
3. **Compression**: Compact storage for annotation metadata
4. **IDE Integration**: Language server protocol support for annotation tooltips

## Compatibility Matrix

| Component | TokenValue Support | Annotation Preservation | Status |
|-----------|-------------------|------------------------|---------|
| Raw AST Loading | ✅ Complete | ✅ Complete | ✅ Working |
| Annotation Extraction | ✅ Complete | ✅ Complete | ✅ Working |
| Pipeline Stage 2 | ✅ Complete | ✅ Preserved | ✅ Working |
| Pipeline Stage 2.5 | ✅ Complete | ✅ Preserved | ✅ Working |
| Pipeline Stage 3 | ✅ Complete | ✅ Preserved | ✅ Working |
| Pipeline Stage 4 | ✅ Complete | ✅ Preserved | ✅ Working |
| Pipeline Stage 5 | ✅ Complete | ✅ Preserved | ✅ Working |
| Metadata Output | ✅ Complete | ✅ Complete | ✅ Working |
| High-Performance Generator | 🔄 In Progress | ✅ Available | 🔄 Needs Completion |

## Conclusion

The semantic annotation parsing implementation in the Rust AST pipeline represents a critical infrastructure component for enabling domain-specific parser generation. By providing robust support for array-formatted semantic annotations while maintaining backward compatibility, this enhancement enables the full semantic annotation system as described in the project's broader architecture documents.

The implementation follows the principle that semantic annotations are **static metadata** that should be preserved throughout the transformation pipeline, making them available for downstream tools like DataGenerators while not interfering with the core parsing logic generation.

This foundation enables the next phase of development: building domain-specific EBNF grammars with appropriate semantic annotation vocabularies and implementing DataGenerators that can leverage this preserved metadata for intelligent parser generation.
