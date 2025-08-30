# Rust Parser Generator Documentation

## Overview

The Rust Parser Generator (`rust_parser_gen`) is a complete implementation of a parser generator that converts JSON raw AST input into Rust parsing code. It implements a comprehensive 6-step AST transformation pipeline with advanced features including left recursion elimination and semantic actions through return annotations.

## Architecture

The generator follows a precise 6-step transformation pipeline:

```
Raw AST JSON → Step 2 → Step 2.5 → Step 3 → Step 4 → Step 5 → Step 6a → Step 6b → Rust Parser
               Group    Handle      Parse    Handle    Build     Left      Semantic
               by OR    Parens      Sequences Quantifiers Tree   Recursion  Actions
```

## Core Data Structures

### Return Annotation AST

The generator supports structured semantic actions through a comprehensive return annotation system:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ReturnAnnotation {
    ScalarRef(usize),                                    // $1, $2, etc.
    ArrayExpr(Vec<ReturnAnnotation>),                    // [$1, $2, "literal"]
    ObjectExpr(Vec<(String, ReturnAnnotation)>),         // {key: $1, value: $2}
    LiteralAnnotation(String),                           // "literal_value"
    DotAccess(Box<ReturnAnnotation>, String),            // $1.field
    QuantifiedAnnotation(Box<ReturnAnnotation>),         // $1* (quantified)
}
```

### Grammar Production System

For left recursion elimination, the generator uses a production-based representation:

```rust
#[derive(Debug, Clone)]
pub struct Production {
    pub symbols: Vec<String>,
    pub annotation: Option<ReturnAnnotation>,
}

#[derive(Debug, Clone)]
pub struct GrammarRule {
    pub name: String,
    pub productions: Vec<Production>,
}
```

### Semantic AST Nodes

The transformation pipeline builds a rich semantic AST:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ASTNode {
    Terminal(String),                                    // Literal strings/tokens
    NonTerminal(String),                                 // Rule references
    Sequence(Vec<ASTNode>),                              // Sequential elements
    Choice(String, Vec<ASTNode>),                        // Alternative branches
    QuantifiedNode(Box<ASTNode>, String),                // ?, *, + quantifiers
    GroupNode(Box<ASTNode>),                             // Parenthesized groups
    WithAnnotation(Box<ASTNode>, ReturnAnnotation),      // Nodes with semantic actions
}
```

## Implementation Features

### Step 6a: Left Recursion Elimination

Implements the complete **Aho-Sethi-Ullman left recursion elimination algorithm**:

#### Indirect Left Recursion Elimination
```rust
// For each rule Ai in order:
for i in 0..rule_names.len() {
    let ai = &rule_names[i];
    
    // Eliminate indirect left recursion with previous rules
    for j in 0..i {
        let aj = &rule_names[j];
        
        // Replace Ai -> Aj β with Ai -> γ₁ β | γ₂ β | ... for all Aj -> γₖ
        let mut new_productions = Vec::new();
        for prod in &productions[ai] {
            if !prod.symbols.is_empty() && prod.symbols[0] == *aj {
                // Substitute Aj productions
                for aj_prod in &productions[aj] {
                    let mut new_symbols = aj_prod.symbols.clone();
                    new_symbols.extend_from_slice(&prod.symbols[1..]);
                    new_productions.push(Production {
                        symbols: new_symbols,
                        annotation: prod.annotation.clone(),
                    });
                }
            } else {
                new_productions.push(prod.clone());
            }
        }
        productions.insert(ai.clone(), new_productions);
    }
    
    // Eliminate direct left recursion
    productions = eliminate_immediate_left_recursion(productions, ai)?;
}
```

#### Direct Left Recursion Elimination
```rust
// Transform: A -> Aα | β  into:  A -> βA'  and  A' -> αA' | ε
fn eliminate_immediate_left_recursion(
    mut productions: HashMap<String, Vec<Production>>,
    rule_name: &str
) -> Result<HashMap<String, Vec<Production>>, Box<dyn std::error::Error>> {
    let prods = productions.get(rule_name).unwrap().clone();
    
    // Separate left-recursive and non-left-recursive productions
    let mut left_recursive = Vec::new();
    let mut non_left_recursive = Vec::new();
    
    for prod in prods {
        if !prod.symbols.is_empty() && prod.symbols[0] == rule_name {
            left_recursive.push(prod);
        } else {
            non_left_recursive.push(prod);
        }
    }
    
    if left_recursive.is_empty() {
        return Ok(productions); // No left recursion
    }
    
    // Create auxiliary rule A'
    let aux_rule = format!("{}_prime", rule_name);
    
    // Transform productions
    let mut new_main_prods = Vec::new();
    let mut aux_prods = Vec::new();
    
    // A -> βA' for each non-left-recursive production A -> β
    for prod in non_left_recursive {
        let mut new_symbols = prod.symbols;
        new_symbols.push(aux_rule.clone());
        new_main_prods.push(Production {
            symbols: new_symbols,
            annotation: prod.annotation,
        });
    }
    
    // A' -> αA' for each left-recursive production A -> Aα
    for prod in left_recursive {
        if prod.symbols.len() > 1 {
            let alpha = &prod.symbols[1..];
            let mut new_symbols = alpha.to_vec();
            new_symbols.push(aux_rule.clone());
            aux_prods.push(Production {
                symbols: new_symbols,
                annotation: prod.annotation,
            });
        }
    }
    
    // A' -> ε (epsilon production)
    aux_prods.push(Production {
        symbols: vec![],
        annotation: None,
    });
    
    productions.insert(rule_name.to_string(), new_main_prods);
    productions.insert(aux_rule, aux_prods);
    
    Ok(productions)
}
```

### Step 6b: Return Annotation Processing

#### Annotation Extraction
```rust
fn extract_return_annotation(tokens: &[Token]) -> (Vec<Token>, Option<ReturnAnnotation>) {
    // Find "->" operator
    if let Some(arrow_pos) = tokens.iter().position(|t| t.token_type == "operator" && t.value == "->") {
        let clean_tokens = tokens[..arrow_pos].to_vec();
        let annotation_tokens = &tokens[arrow_pos + 1..];
        
        if !annotation_tokens.is_empty() {
            let annotation = parse_return_annotation_tokens(annotation_tokens)?;
            return (clean_tokens, Some(annotation));
        }
    }
    
    (tokens.to_vec(), None)
}
```

#### Annotation Parsing
Supports complex annotation expressions:

```rust
fn parse_return_annotation_tokens(tokens: &[Token]) -> Result<ReturnAnnotation, Box<dyn std::error::Error>> {
    let combined = tokens.iter().map(|t| &t.value).collect::<Vec<_>>().join(" ");
    
    // Scalar references: $1, $2, etc.
    if let Some(captures) = SCALAR_REF_REGEX.captures(&combined) {
        let index = captures[1].parse::<usize>()?;
        return Ok(ReturnAnnotation::ScalarRef(index));
    }
    
    // Array expressions: [$1, $2, "literal"]
    if combined.starts_with('[') && combined.ends_with(']') {
        let inner = &combined[1..combined.len()-1];
        let parts = parse_comma_separated(inner);
        let mut contents = Vec::new();
        
        for part in parts {
            let part = part.trim();
            if let Some(captures) = SCALAR_REF_REGEX.captures(part) {
                let index = captures[1].parse::<usize>()?;
                if part.ends_with('*') {
                    contents.push(ReturnAnnotation::QuantifiedAnnotation(
                        Box::new(ReturnAnnotation::ScalarRef(index))
                    ));
                } else {
                    contents.push(ReturnAnnotation::ScalarRef(index));
                }
            } else {
                contents.push(ReturnAnnotation::LiteralAnnotation(part.to_string()));
            }
        }
        
        return Ok(ReturnAnnotation::ArrayExpr(contents));
    }
    
    // Object expressions: {key: $1, value: $2}
    if combined.starts_with('{') && combined.ends_with('}') {
        let inner = &combined[1..combined.len()-1];
        let pairs = parse_object_pairs(inner);
        let mut contents = Vec::new();
        
        for (key, value) in pairs {
            if let Some(captures) = SCALAR_REF_REGEX.captures(&value) {
                let index = captures[1].parse::<usize>()?;
                contents.push((key, ReturnAnnotation::ScalarRef(index)));
            } else {
                contents.push((key, ReturnAnnotation::LiteralAnnotation(value)));
            }
        }
        
        return Ok(ReturnAnnotation::ObjectExpr(contents));
    }
    
    // Default: literal annotation
    Ok(ReturnAnnotation::LiteralAnnotation(combined))
}
```

### Code Generation with Semantic Actions

The generator produces Rust parsing functions that return structured data:

#### Basic Parsing Function
```rust
fn generate_parse_function(rule_name: &str, ast_node: &ASTNode) -> String {
    match ast_node {
        ASTNode::WithAnnotation(node, annotation) => {
            let inner_function = generate_parse_function(&format!("{}_inner", rule_name), node);
            let semantic_action = generate_semantic_action(annotation);
            
            format!(r#"
    // Parse {} rule (with semantic action)
    fn parse_{}(parser: &mut Parser) -> ParseResult<ParseValue> {{
        // Parse inner structure
        let inner_result = parse_{}_inner(parser)?;
        
        // Apply semantic action
        let result = {};
        Ok(result)
    }}
    
    {}"#, rule_name, rule_name, rule_name, semantic_action, inner_function)
        },
        
        ASTNode::Choice(_, alternatives) => {
            let mut code = format!(r#"
    // Parse {} rule
    fn parse_{}(parser: &mut Parser) -> ParseResult<ParseValue> {{
        let start_pos = parser.position();
        let mut errors = Vec::new();
        "#, rule_name, rule_name);
            
            for (i, alt) in alternatives.iter().enumerate() {
                code.push_str(&format!(r#"
        
        // Try alternative {}
        parser.set_position(start_pos);
        match {} {{
            Ok(result) => return Ok(result),
            Err(e) => errors.push(e),
        }}"#, i + 1, generate_alternative_code(alt)));
            }
            
            code.push_str(&format!(r#"
        
        // All alternatives failed
        Err(ParseError::new(
            format!("No valid alternative found for {}", rule_name),
            start_pos,
            vec!["valid {} alternative".to_string()]
        ))
    }}"#, rule_name, rule_name));
            
            code
        },
        
        // ... other AST node types
    }
}
```

#### Semantic Action Generation
```rust
fn generate_semantic_action(annotation: &ReturnAnnotation) -> String {
    match annotation {
        ReturnAnnotation::ScalarRef(index) => {
            format!("inner_result.get_element({})", index)
        },
        
        ReturnAnnotation::ArrayExpr(contents) => {
            let elements: Vec<String> = contents.iter()
                .map(|content| generate_semantic_action(content))
                .collect();
            format!("ParseValue::Array(vec![{}])", elements.join(", "))
        },
        
        ReturnAnnotation::ObjectExpr(pairs) => {
            let pairs_code: Vec<String> = pairs.iter()
                .map(|(key, value)| {
                    format!(r#"("{}".to_string(), {})"#, key, generate_semantic_action(value))
                })
                .collect();
            format!("ParseValue::Object(std::collections::HashMap::from([{}]))", 
                   pairs_code.join(", "))
        },
        
        ReturnAnnotation::LiteralAnnotation(value) => {
            format!(r#"ParseValue::String("{}".to_string())"#, value)
        },
        
        ReturnAnnotation::QuantifiedAnnotation(base) => {
            let base_action = generate_semantic_action(base);
            format!("ParseValue::Array(collect_quantified({}))", base_action)
        },
        
        ReturnAnnotation::DotAccess(base, field) => {
            format!("{}.get_field(\"{}\")", generate_semantic_action(base), field)
        }
    }
}
```

## Parser Output Format

The generated Rust parser includes:

### Rich Return Types
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ParseValue {
    String(String),
    Array(Vec<ParseValue>),
    Object(std::collections::HashMap<String, ParseValue>),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
}
```

### Comprehensive Error Handling
```rust
#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
    pub expected: Vec<String>,
}

pub type ParseResult<T> = Result<T, ParseError>;
```

### Parser State Management
```rust
pub struct Parser {
    input: String,
    position: usize,
}

impl Parser {
    pub fn new(input: String) -> Self {
        Self { input, position: 0 }
    }
    
    pub fn position(&self) -> usize {
        self.position
    }
    
    pub fn set_position(&mut self, pos: usize) {
        self.position = pos;
    }
    
    pub fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
    
    // ... other parser methods
}
```

## Command Line Interface

```bash
rust_parser_gen [OPTIONS] [JSON_FILE]

OPTIONS:
    -o, --output <FILE>     Write parser to <basename>.rs
    -m, --module <NAME>     Set module name (default: derived from grammar)
    -q, --quiet             Suppress progress messages
    -h, --help              Print help information

ARGUMENTS:
    <JSON_FILE>             Input JSON file (optional, reads from stdin if not provided)
```

## Usage Examples

### Basic Usage
```bash
# From file
rust_parser_gen grammar.json -o parser.rs

# From stdin
ebnf_to_json.pl grammar.ebnf | rust_parser_gen -o parser.rs

# With custom module name
rust_parser_gen grammar.json -o parser.rs -m MyCustomParser
```

### Grammar with Return Annotations
```ebnf
json_value := json_object | json_array | json_string | json_number -> $1
json_object := "{" json_members? "}" -> {type: "object", members: $2}
json_members := json_pair ("," json_pair)* -> [$1, $2*]
json_pair := json_string ":" json_value -> {key: $1, value: $3}
```

This generates a Rust parser that returns structured `ParseValue` objects instead of raw strings, enabling rich semantic processing of parsed content.

## Key Features

1. **Complete Left Recursion Elimination**: Handles both direct and indirect left recursion using the standard Aho-Sethi-Ullman algorithm
2. **Rich Semantic Actions**: Return annotations enable structured data construction during parsing
3. **Comprehensive Error Handling**: Detailed error reporting with position information and expected tokens
4. **Robust Code Generation**: Produces clean, efficient Rust parsing code with proper error handling
5. **Advanced Grammar Support**: Handles complex grammars with quantifiers, grouping, and nested structures
6. **Production System**: Internal representation using grammar productions for algorithmic transformations

## Implementation Status

✅ **Complete** - The Rust parser generator implements the full 6-step transformation pipeline with:
- Steps 1-5: Core AST transformation pipeline
- Step 6a: Complete left recursion elimination
- Step 6b: Full return annotation support and semantic action generation
- Production-based grammar representation for algorithmic processing
- Rich output format with structured data types
- Comprehensive error handling and reporting
