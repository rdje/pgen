# Bootstrap Architecture & Annotation Parsing System

## Summary
The bootstrap build system for the pgen parser generator is now fully implemented and tested. This system solves the circular dependency problem that occurs when building annotation parsers from scratch.

## Key Achievements

### 1. File-Based Placeholder Targets ✅
- Converted from phony to file-based targets in Makefile
- Added `.placeholder` marker files to track generation state
- Placeholders created only when missing (follows Make dependency model)
- Updated `clean` target to remove marker files

### 2. Complete Bootstrap Mode ✅
- Added `--bootstrap-mode` CLI flag to Rust AST pipeline
- Built-in semantic annotation parser (name:value patterns + function calls ≤4 args)
- Built-in return annotation parser (flat structures: scalars, arrays, objects ≤3 keys)
- Automatic fallback when external parsers fail
- Proper warning messages for unsupported patterns in bootstrap mode

### 3. Configuration Integration ✅
- Fixed missing `trace` field in `PipelineConfig` initialization
- All CLI arguments properly passed to pipeline configuration
- Bootstrap mode flag correctly propagated through the system

### 4. Build Process Verification ✅
- Full bootstrap build test passes from completely clean state
- All components build successfully: placeholders → Rust pipeline → final parser
- Build status shows all files exist and are properly generated

## Bootstrap Test Results
```bash
$ make bootstrap-test
Cleaning generated files...
Cleaning all build artifacts...
Testing full bootstrap build process...
[Creating placeholders...]
[Building Rust AST pipeline...] ✓
[Generating with bootstrap mode...] ✓
[Final regex parser generated] ✓
```

## File Structure After Bootstrap
```
generated/
├── semantic_annotation_parser.rs           ✓ Generated
├── semantic_annotation_parser.rs.placeholder ✓ Marker
├── return_annotation_parser.rs             ✓ Generated  
├── return_annotation_parser.rs.placeholder ✓ Marker
├── regex.json                               ✓ Generated
└── regex_parser.rs                         ✓ Generated
```

## Bootstrap Mode Capabilities

### Semantic Annotations
- Simple name:value patterns: `generate: some_function()`
- Function calls with up to 4 arguments: `validate: check($1, $2, $3, $4)`
- Complex patterns fall back to raw strings with warnings

### Return Annotations  
- Scalar references: `$1`, `$2`
- Simple arrays: `[$1, $2]`, `[$1*]`
- Simple objects: `{type: $1, value: $2}` (max 3 keys)
- Complex nested structures fall back to raw strings

## Three-Level Bootstrap Architecture

### Level 1: Core Built-in Parsers (Hardcoded in Rust)

#### Built-in Semantic Annotation Parser
- **Location**: `ast_pipeline.rs::parse_semantic_annotation_bootstrap()`
- **Capabilities**: Simple name:value patterns, function calls with ≤4 arguments
- **Output**: String representation (e.g., "generate:some_function()")
- **Purpose**: Parse semantic annotations in bootstrap mode when external parser unavailable

#### Built-in Return Annotation Parser  
- **Location**: `unified_return_ast.rs::UnifiedReturnAST::parse_bootstrap()`
- **Capabilities**: 
  - Scalar references: `$1`, `$2`, etc.
  - Arrays with spreads: `[$1, $3*]`
  - Objects: `{type: "array", element: $3}`
  - Nested structures supported
- **Output**: `UnifiedReturnAST` enum (semantic AST)
- **Purpose**: Parse return annotations to control parser output structure

### Level 2: Special Self-Hosted Parsers (Generated in Bootstrap Mode)

#### semantic_annotation.ebnf → Semantic_annotationParser
- **Special Status**: Must be parseable by Level 1 built-in parser
- **Generation**: Uses bootstrap mode (built-in parsers)
- **Location**: Included via `include!("../../generated/semantic_annotation_parser.rs")`
- **Purpose**: Full-featured semantic annotation parsing for Level 3 grammars

#### return_annotation.ebnf → Return_annotationParser
- **Special Status**: Must be parseable by Level 1 built-in parser  
- **Generation**: Uses bootstrap mode (built-in parsers)
- **Location**: Included via `include!("../../generated/return_annotation_parser.rs")`
- **Grammar Features**: Recursive structures, dot notation, array slicing, quantifiers
- **Purpose**: Full-featured return annotation parsing for Level 3 grammars

### Level 3: User Grammars (Generated in Non-Bootstrap Mode)

#### All Other EBNF Files
- **Examples**: `json.ebnf`, `regex.ebnf`, `ebnf.ebnf`, user grammars
- **Generation**: Uses Level 2 parsers (full features)
- **Capabilities**: Can use all semantic and return annotation features
- **Purpose**: Generate production parsers with full annotation support

## Data Flow Architecture

### Return Annotation Processing Flow

#### Path 1: Bootstrap Mode
```
Text annotation (e.g., "-> [$1, $3*]")
    ↓
UnifiedReturnAST::parse_bootstrap() [Level 1]
    ↓  
UnifiedReturnAST (semantic AST)
    ↓
Code Generator
    ↓
Rust code
```

#### Path 2: Non-Bootstrap Mode (Full Features)
```
Text annotation
    ↓
Return_annotationParser::parse() [Level 2]
    ↓
ParseNode with ParseContent (syntactic AST)
    ↓
convert_parse_node_to_unified_ast()
    ↓
UnifiedReturnAST (semantic AST)  
    ↓
Code Generator
    ↓
Rust code
```

### Key Design Principles

1. **Separation of Concerns**
   - **Syntactic Parsing**: Structure according to grammar (ParseNode)
   - **Semantic Interpretation**: Meaning extraction (UnifiedReturnAST)
   - **Code Generation**: Single path using UnifiedReturnAST

2. **Bootstrap Hierarchy**
   - Level 1 parsers are simple enough to be hand-written
   - Level 2 parsers use Level 1 to bootstrap themselves
   - Level 3 parsers use Level 2 for full features

3. **Unified AST Benefits**
   - Code generator only knows one AST format
   - Easy to test (bootstrap and full parser produce same AST)
   - Clear upgrade path from bootstrap to full parser

## AST Types Explained

### ParseNode (Syntactic AST)
```rust
pub struct ParseNode {
    pub rule_name: String,      // Grammar rule that matched
    pub content: ParseContent,   // Parsed content
    pub span: Range<usize>,      // Source location for errors
}
```
- Represents HOW text was parsed according to grammar
- Contains spans for error reporting
- Wraps the actual parsed content

### UnifiedReturnAST (Semantic AST)
```rust
pub enum UnifiedReturnAST {
    PositionalRef { index: usize },           // $1, $2, etc.
    StringLiteral { value: String },          // "text"
    Array { elements: Vec<UnifiedReturnAST> }, // [...]
    Object { properties: HashMap<...> },      // {...}
    Spread { base: Box<UnifiedReturnAST> },   // $3*
    // ... other variants
}
```
- Represents WHAT the annotation means semantically
- Direct mapping to code generation needs
- Same structure from both bootstrap and full parser

## Critical Files

### Core Implementation
- `rust/src/ast_pipeline.rs` - Main pipeline, includes special parsers
- `rust/src/ast_pipeline/unified_return_ast.rs` - UnifiedReturnAST definition and bootstrap parser
- `rust/src/ast_pipeline/return_annotation_handler.rs` - Old handler (being phased out)
- `rust/src/ast_pipeline/high_performance_generator.rs` - Code generator using UnifiedReturnAST

### Special Grammars (Level 2)
- `grammars/semantic_annotation.ebnf` - Defines semantic annotation syntax
- `grammars/return_annotation.ebnf` - Defines return annotation syntax (195 lines, full features)

### Generated Parsers (Bootstrap Artifacts)
- `rust/generated/semantic_annotation_parser.rs` - Generated from semantic_annotation.ebnf
- `rust/generated/return_annotation_parser.rs` - Generated from return_annotation.ebnf

## Bootstrap Process Summary

1. **Clean State**: No generated parsers exist
2. **Create Placeholders**: Makefile creates `.placeholder` files
3. **Build Pipeline**: Rust compiles with placeholder includes (empty modules)
4. **Generate Level 2**: Use built-in parsers to generate special parsers
5. **Rebuild Pipeline**: Now includes real Level 2 parsers
6. **Generate Level 3**: Use Level 2 parsers for all other grammars

This architecture ensures:
- No circular dependencies
- Clean bootstrap from nothing
- Progressive enhancement of features
- Single code generation path
- Clear error messages at each level
