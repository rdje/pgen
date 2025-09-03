# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Repository Overview

This repository contains a **Multi-Language EBNF Parser Generator** with comprehensive AST transformation pipeline. The system converts EBNF grammar specifications into executable parsers across multiple programming languages while preserving semantic and logging annotations.

**Current Development Status**: Multi-language implementation with Perl being the most tested and reliable. Other implementations (Rust, Julia, Go, Python, Zig) are complete but need comprehensive testing.

**Core System Architecture**:
- **Three-Phase Pipeline**: EBNF → Raw AST JSON → Transformed AST JSON → Parser Code
- **Multi-Language Support**: 6 language implementations of AST transformation pipeline
- **JSON Interchange Format**: Universal data exchange between phases and languages
- **Annotation System**: Semantic, logging, and return annotations preserved through pipeline
- **Self-hosting capability**: Parser generator parses its own EBNF grammar specifications
- **Performance optimization**: Baseline 26K-29K parses/sec for production use

## Key Architecture Components

### Core Modules (perl/ directory)
- **`perl/AST/Transform.pm`**: Main AST transformation engine that converts EBNF AST into Perl parser code
- **`perl/AST/LeftRecursion.pm`**: Left recursion elimination for complex recursive grammars
- **`perl/AST/Performance.pm`**: Performance optimization utilities (29K+ parses/sec baseline)
- **`perl/Parser/ReturnAnnotation.pm`**: Self-hosting return annotation parser wrapper

### Command Line Tools (tools/ directory)

**Current Perl-Centric Tools:**
- **`tools/ast_transform.pl`**: Primary CLI wrapper for AST::Transform - generates parsers from EBNF
- **`tools/ebnf_generator.pl`**: EBNF parser generation utilities
- **`tools/ebnf_input_generator.pl`**: Probability-based input generation from EBNF grammars
- **`tools/analyze_spec.pl`**: Grammar analysis and validation tool
- **`tools/generate_parser.pl`**: Alternative parser generation interface
- **`tools/perl_parser_gen.pl`**: Perl-specific parser generation utilities

**Universal JSON-Based Tools:**
- **`tools/ebnf_to_json.pl`**: EBNF to JSON conversion tool - foundation of new architecture
- **`rust_parser_gen`**: (In development) Native Rust parser generator
- **`julia_parser_gen`**: (In development) Native Julia parser generator
- **Future generators**: Python, Go, C++, WebAssembly, and more

### Framework Integration (fx/ subdirectory)
The `fx/` directory contains the original 2008 consulting framework components, built around `LinkedSpec.pm`. This includes:
- **`LinkedSpec.pm`**: Original Perl package for parsing Lisp-like and other text formats using `.spec` files
- **`LinkedRE.pm`**: Regex orchestration utilities
- **TableScript files (`.ts`)**: Domain-specific language for table processing and formatting (NOT TypeScript)
- **Binary tools**: Various EDA and processing utilities from the original consulting work

### Bootstrap Process
- **`fx/specs/ebnf.spec`**: Bootstrap specification that enables EBNF support using the original LinkedSpec.pm system
- The system uses this bootstrap to parse EBNF grammars and generate new parsers
- Evolution: `.spec` format (2008) → `.ebnf` format (recent AI-assisted development)

## Setup and Dependencies

### Multi-Language Prerequisites
- **Perl**: 5.20+, JSON::PP module (Most reliable implementation)
- **Rust**: 1.70+, cargo
- **Julia**: 1.8+, JSON3 package (add if using Julia)
- **Go**: 1.19+ (uses standard library only)
- **Python**: 3.8+ (uses standard library only) 
- **Zig**: 0.15.1+ (⚠️ build system needs fixing)

### Quick Start
```bash
# Clone repository and navigate to it
cd /path/to/airefactored

# Verify Perl environment
perl -v

# Test the system with a simple grammar
echo 'rule := "token"' > test.ebnf
perl tools/ast_transform.pl test.ebnf

# Clean up
rm test.ebnf
```

### File Extensions Important for This Repository
- **`.ebnf`**: EBNF grammar files (modern format)
- **`.spec`**: Legacy LinkedSpec format files (2008 framework)
- **`.ts`**: TableScript files (NOT TypeScript - domain-specific table processing language)
- **`.pl`**: Perl scripts and generated parsers
- **`.pm`**: Perl modules

## Common Development Commands

### Generate Parser from EBNF Grammar
```bash
# Generate parser from EBNF grammar
perl tools/ast_transform.pl --input path/to/grammar.ebnf --output .

# Generate with custom package name
perl tools/ast_transform.pl --input grammar.ebnf --output . --package MyCustomParser
```

### Test Generated Parsers
```bash
# Quick test of a generated parser
perl -e "use lib '.'; use MyParser; my \$input = 'test_input'; my \$result = MyParser::parse(\$input); use Data::Dumper; print Dumper(\$result);"

# Test with external input file
perl MyParser.pl < input.txt
```

### Run Comprehensive Test Suite
```bash
# Run all tests from tests directory
cd tests
perl run_tests.pl

# Run single test with debugging
cd tests
export DUMP_VERBOSITY=200
./run_single_test.sh specs/valid/basic.spec tests/input/simple.txt

# Generate test input from grammar
cd tests
perl generate_test_input.pl specs/valid/basic.spec
```

### Performance Testing and Optimization
```bash
# The system includes built-in performance testing
# Generated parsers achieve 29K+ parses/second baseline performance
# No additional performance setup required - optimization is automatic
```

### Multi-Language Build and Test Commands

#### Rust
```bash
# Build Rust implementation
cd rust
cargo build

# Run tests
cargo test

# Process raw AST JSON to transformed AST
cargo run input.json output.json
```

#### Julia
```bash
# Set up environment (if needed)
cd julia
julia -e "using Pkg; Pkg.activate(.); Pkg.instantiate()"

# Run Julia implementation
julia ast_pipeline.jl input.json output.json
```

#### Go
```bash
# Build Go implementation
cd go
go build ast_pipeline.go

# Run tests
go test

# Run implementation
./ast_pipeline input.json output.json
```

#### Python
```bash
# Run Python implementation
cd python
python ast_pipeline.py input.json output.json
```

#### Zig
```bash
# Build Zig implementation (currently has build issues)
cd zig
zig build

# Run implementation (when fixed)
zig build run -- input.json output.json
```

### Multi-Language Implementation Status

| Language | AST Pipeline | Build System | Tests | Status |
|----------|-------------|-------------|-------|--------|
| **Perl**     | ✅ Complete  | ✅ Complete | ✅ Better tested | **Most Reliable** |
| **Rust**     | ✅ Complete  | ✅ Complete | ⚠️ Minimal | **Needs Testing** |
| **Julia**    | ✅ Complete  | ✅ Complete | ⚠️ Minimal | **Needs Testing** |
| **Go**       | ✅ Complete  | ✅ Complete | ⚠️ Minimal | **Needs Testing** |
| **Python**   | ✅ Complete  | ✅ Complete | ⚠️ Minimal | **Needs Testing** |
| **Zig**      | ⚠️ Partial  | ❌ Build issues | ⚠️ Minimal | **In Development** |

### JSON-Based Architecture Commands (New)

```bash
# Generate JSON AST from EBNF grammar (foundation of new architecture)
perl tools/ebnf_to_json.pl grammar.ebnf > grammar_ast.json

# Validate JSON output format
jq '.metadata.format' grammar_ast.json  # Should output "raw_ast"

# Examine raw AST structure
jq '.raw_ast[0]' grammar_ast.json  # View first rule's token structure

# Future: Generate native parsers from JSON (5-step transformation pipeline)
# rust_parser_gen < grammar_ast.json --output parser.rs
# julia_parser_gen < grammar_ast.json --output parser.jl
# python_parser_gen < grammar_ast.json --output parser.py
```

### Understanding the 5-Step AST Transformation Pipeline

Each language generator implements a **5-step transformation pipeline** to convert raw tokens into semantic AST:

```
Raw AST → Step 2 → Step 2.5 → Step 3 → Step 4 → Step 5 → Semantic Tree
         Group    Handle      Parse    Handle    Build
         by OR    Parens      Sequences Quantifiers Tree
```

**Pipeline Steps:**
1. **Step 2**: Group by OR operators (`|`) - separates rule alternatives
2. **Step 2.5**: Handle parentheses - resolves grouping and precedence
3. **Step 3**: Parse sequences - identifies ordered symbol sequences
4. **Step 4**: Handle quantifiers - processes `?`, `*`, `+` modifiers
5. **Step 5**: Build tree - creates final semantic AST structure

**For Language Implementers**: See `docs/ast_transformation_pipeline.md` for complete algorithmic specifications, worked examples, and implementation best practices.

## EBNF Grammar Syntax

### Critical Rule: Regex Capturing Groups
When using `$1, $2, etc.` in return annotations, regex patterns **MUST** have capturing groups:

```ebnf
# CORRECT - has capturing groups
identifier := /([a-zA-Z_]\w*)/ -> $1
string_literal := /"([^"]*)"/ -> $1

# WRONG - missing capturing groups (will cause undefined $1)
identifier := /[a-zA-Z_]\w*/ -> $1
```

### Return Annotation Patterns
```ebnf
# Simple values
name := identifier -> $1

# Arrays with quantifiers  
list := item (',' item)* -> [$1, $2*]

# Objects
declaration := 'var' identifier ':' type -> {name: $2, type: $4}

# Ultimate dot notation (advanced feature)
complex := data_structure -> {
    items: [$1.items*],
    metadata: $1.meta,
    first_item: $1.items[0]
}
```

## Advanced Features

### Ultimate Dot Notation System
Supports sophisticated parse-tree structure access:
- **Property access**: `$2.name`, `$1.data.items`
- **Positional access**: `$2.1`, `$2.2` (1-based for parse groups)
- **Array slicing**: `$2.items[1:4]`, `$2.items[0,2,4]`, `$2.items[-1]`
- **Mixed patterns**: `$2.items[1:3].name`, `$3.data[::2].value`

### Self-Hosting Capabilities
- The parser generator parses its own EBNF grammar specifications
- Return annotation DSL is parsed by generated parsers
- No bootstrap limitations - unlimited extensibility

### Performance Optimization
- Baseline performance: 29,665+ parses/sec
- Automatic optimizations: quantifier loops, regex caching, memory pooling
- Handles 100+ rule grammars, 50+ alternative branches
- Deep recursion support with memory management

## Testing Status and Development Priorities

### Critical Testing Gaps ⚠️

**What Works Well:**
- **Perl Implementation**: Better tested and most reliable implementation
- **Code Architecture**: All implementations follow correct design patterns
- **JSON Format**: Interchange format is well-defined and documented

**What Needs Work:**
- **Non-Perl Testing**: Rust, Julia, Go, Python implementations need comprehensive testing
- **Cross-Language Validation**: No systematic compatibility testing between languages
- **Integration Testing**: End-to-end pipeline validation needed
- **Zig Implementation**: Incomplete due to Zig 0.15.1 build system issues
- **Error Handling**: Inconsistent behavior across implementations

### Immediate Development Priorities

1. **Complete Zig Implementation** (1-2 weeks)
   - Fix build system issues with Zig 0.15.1
   - Complete partial implementation

2. **Create Comprehensive Test Suites** (2-3 weeks per language)
   - Bring non-Perl languages up to Perl's testing level
   - Unit tests for each pipeline step
   - Integration tests with real grammar files

3. **Cross-Language Validation** (2-4 weeks)
   - Ensure all languages produce equivalent JSON output
   - Systematic compatibility testing
   - Bug discovery and fixing process

4. **Bug Discovery and Fixing** (1-2 weeks per language after testing)
   - Address issues found during systematic testing
   - Stabilize implementations

### Realistic Timeline
- **Testing Phase**: 8-12 weeks total
- **Stabilization**: 4-6 weeks
- **Production Ready**: 12-18 weeks for all languages

**Note**: Only use Perl implementation for production work currently. Other implementations are coded but largely untested.

## Development Workflow

### Before Working with Grammars
1. Understand the difference between EBNF syntax (this system) and LinkedSpec.pm syntax (legacy .spec files)
2. Always use capturing groups `()` in regex patterns when referencing `$n`
3. Test grammars incrementally starting with simple patterns

### Debugging Parser Issues
1. Check `DEBUGGING_STARTUP_GUIDE.md` for common runtime errors
2. Use test framework with verbosity: `export DUMP_VERBOSITY=200`
3. Verify generated parser structure in `.pm` files
4. Test with simple inputs before complex patterns

### Grammar Development Best Practices
1. Start with terminal rules and basic patterns
2. Add return annotations incrementally
3. Use the test input generator to validate grammar coverage
4. Performance is automatically optimized - focus on correctness first

## File Structure Context

### Working Directory Layout
- Root contains generated parsers (`.pl`, `.pm` files) and test files
- `docs/` contains comprehensive feature documentation 
- `grammars/` contains example and test grammars
- `tests/` contains full test framework
- `legacy/` contains preserved development artifacts
- `archive/` contains historical development components

### Important Documentation Files
- `docs/PROJECT_STATUS_REPORT.md`: Complete system capabilities overview
- `docs/parser_architecture_evolution.md`: **Critical** - Documents the evolution from Perl-centric to JSON-based architecture
- `docs/ast_transformation_pipeline.md`: Detailed transformation pipeline algorithms
- `docs/ULTIMATE_DOT_NOTATION_DOCS.md`: Advanced data access patterns
- `docs/EBNF_GRAMMAR_RULES.md`: Grammar syntax rules and best practices
- `docs/PERFORMANCE_GUIDE.md`: Performance optimization details
- `tests/TEST_GUIDE.md`: Comprehensive testing documentation
- `docs/cursor_one_chat_session.md`: Complete Cursor/Sonnet 4 development session showing the detailed technical work on resolving EBNF groups with quantifiers and return annotations

## Architectural Evolution: From Perl-Centric to Universal JSON-Based System

### Current Architecture (Perl-Centric)
The system currently operates with Perl as the central transformation engine:

```
EBNF Grammar Files → LinkedSpec.pm → Raw AST → AST::Transform.pm → Language Generators → Target Parsers
```

**Current Capabilities:**
- **Production-ready Perl generator**: 29K+ parses/sec with comprehensive optimization
- **Rust/Julia generators**: Type-safe code generation for systems and scientific computing
- **Self-hosting**: Parser generator parses its own specifications
- **Advanced features**: Ultimate dot notation, left recursion elimination, grouped quantifiers

### JSON-Based Universal Architecture (In Development)

A **major architectural evolution** is underway to create a universal JSON-based system:

```
EBNF Grammar → ebnf_to_json.pl → Raw AST JSON → Language-Specific Generators → Native Parsers
```

**Key Benefits of New Architecture:**
- **Language Independence**: No Perl dependency for target language generators
- **Optimization Freedom**: Each language can optimize for its strengths (Rust zero-copy, Julia multiple dispatch)
- **Innovation Unlocked**: Languages can experiment with novel transformation approaches
- **Community Growth**: Lower barriers for language experts to contribute generators
- **Distribution Simplified**: Smaller, focused tools with native packaging

### Migration Status

**Phase 1: Foundation (✅ COMPLETE)**
- ✅ `ebnf_to_json.pl` tool created
- ✅ JSON interchange format documented
- ✅ Test suite with expected JSON outputs

**Phase 2: Reference Implementations (🔄 IN PROGRESS)**
- [ ] Native Rust generator (`rust_parser_gen`)
- [ ] Native Julia generator (`julia_parser_gen`)
- [ ] Cross-language validation testing

**Phase 3: Ecosystem Expansion (🔮 PLANNED)**
- [ ] Python, Go, C++ generators
- [ ] Package manager integration (Cargo, PyPI, npm)
- [ ] IDE/LSP integration

### Development Priorities (Updated)
1. **Complete JSON architecture migration**: Native language generators without Perl dependency
2. **Performance optimization**: Language-specific optimizations (2-5x improvements expected)
3. **Community ecosystem**: Enable contributions from language domain experts
4. **Advanced parsing features**: Streaming parsers, parallel parsing, error recovery
5. **Cross-language consistency**: Ensure identical behavior across all target languages

## Language Implementer's Guide

### Creating a New Language Generator

To implement a parser generator for your target language, follow this structured approach:

#### 1. **Setup and JSON Processing**
```bash
# Your generator should accept JSON from stdin
cat grammar.json | your_lang_parser_gen --output parser.ext
```

#### 2. **Raw AST Token Structure**
Understand the input format - each token is `[token_type, token_value]`:
- `["rule", "rule_name"]` - Rule definition
- `["identifier", "symbol"]` - Non-terminal reference
- `["quoted_string", "literal"]` - Terminal string
- `["operator", "|"]` - Choice operator
- `["quantifier", "*"]` - Quantifier (`?`, `*`, `+`)

#### 3. **Implement the 5-Step Pipeline**

**Step 2**: Group by OR operators
```python
def group_by_or(tokens):
    # Split on "|" operators, return list of alternative token lists
    # See ast_transformation_pipeline.md for full algorithm
```

**Step 2.5**: Handle parentheses
```python
def handle_parentheses(tokens):
    # Group tokens within () into nested structures
    # Preserve nesting depth and resolve precedence
```

**Step 3**: Parse sequences
```python
def parse_sequences(alternatives):
    # Convert token lists into sequence/single structures
    # Filter whitespace and comments
```

**Step 4**: Handle quantifiers
```python
def handle_quantifiers(alternatives):
    # Attach quantifiers to preceding symbols
    # Create quantified nodes with base symbol + quantifier
```

**Step 5**: Build semantic tree
```python
def build_tree_structure(rule_name, alternatives):
    # Create final AST with proper type information
    # Return structured tree ready for code generation
```

#### 4. **Language-Specific Optimizations**

**Rust Example**: Zero-copy with lifetimes
```rust
struct Token<'a> {
    token_type: &'a str,
    value: &'a str,
}
```

**Julia Example**: Multiple dispatch
```julia
transform(::Type{Sequence}, tokens) = # sequence handling
transform(::Type{Choice}, tokens) = # choice handling
```

**Python Example**: Generator-based processing
```python
def transform_stream(tokens):
    for rule in chunk_by_rules(tokens):
        yield transform_rule(rule)
```

#### 5. **Testing Strategy**
- **Unit tests**: Test each pipeline step independently
- **Integration tests**: Use complete grammar examples
- **Cross-validation**: Compare output with reference implementations
- **Performance tests**: Benchmark transformation speed

#### 6. **Code Generation**
After AST transformation, generate parser code optimized for your language:
- Use native data structures (HashMap, Vec, etc.)
- Leverage type system for compile-time safety
- Implement language-specific optimizations (SIMD, zero-copy, etc.)
- Generate idiomatic code following language conventions

### Implementation Resources
- **Complete algorithms**: `docs/ast_transformation_pipeline.md`
- **Worked examples**: Step-by-step transformations with real grammar rules
- **Common pitfalls**: Parentheses precedence, quantifier scope, token type mapping
- **Language optimizations**: Rust lifetimes, Julia dispatch, Python generators
- **Testing patterns**: Unit tests, integration tests, error cases

## Multi-Language Vision

### Historical Context
**18-Year Evolution (2006-2024)**:
- **Original Vision (2006)**: Create language-agnostic parser generators
- **Implemented**: LinkedSpec.pm (Perl), plus ports to Groovy, Ruby, Pnuts
- **Modern Enhancement (2024)**: Advanced parsing with EBNF + backtracking + memoization

### Architecture Insight: Engine vs Action Block Separation
**Key Realization**: Only LinkedSpec.pm needs porting → Everything else follows automatically
- **Phase 1**: Port the Engine (Complex but doable) - LinkedSpec.pm + LinkedRE.pm → LinkedSpec.py + LinkedRE.py
- **Phase 2**: Port Action Blocks (Simple syntax translation) - Same structure, different syntax

### Future Multi-Language Support
The system architecture supports expansion to:
- **Python**: Data science ecosystem integration
- **JavaScript**: Web development and Node.js services
- **Go**: Cloud services and microservices architecture
- **Rust**: Systems programming and WebAssembly targets

## Known System Capabilities

This production-ready system supports:
- **Grammar Types**: HDL (VHDL/Verilog), programming languages, configuration formats, protocols
- **Performance**: Excellent baseline (29K+ parses/sec) for production use
- **Complexity**: 100+ rule grammars, deep nesting, wide alternatives
- **Features**: Left recursion elimination, grouped quantifiers, probability-based generation
- **Architecture**: Clean modular design suitable for professional deployment
- **Error Reporting**: Professional-grade error system with context tracking
- **Self-Hosting**: Parser generator parses its own specifications

The system has evolved far beyond basic parsing into a comprehensive parser generation framework with advanced capabilities rivaling commercial tools.
