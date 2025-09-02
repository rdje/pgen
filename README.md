# Multi-Language EBNF Parser Generator

A comprehensive EBNF (Extended Backus-Naur Form) parser generator with multi-language AST transformation pipeline. Converts EBNF grammar specifications into executable parsers across multiple programming languages while preserving semantic and logging annotations.

## 🚀 Quick Start

```bash
# Clone the repository
git clone https://github.com/rdje/pgen
cd pgen

# Generate Raw AST from EBNF (using Perl)
cd perl
perl ebnf_to_json.pl ../test_grammars/arithmetic.ebnf raw_ast.json

# Transform Raw AST to final AST (choose your language)
cd ../rust && cargo run ../raw_ast.json transformed.json
cd ../julia && julia ast_pipeline.jl ../raw_ast.json transformed.json
cd ../go && go run ast_pipeline.go ../raw_ast.json transformed.json
cd ../python && python ast_pipeline.py ../raw_ast.json transformed.json
```

## 📋 Project Status

### Implementation Status

| Language | AST Pipeline | Build System | Testing Level | Status |
|----------|-------------|-------------|---------------|---------|
| **Perl**     | ✅ Complete | ✅ Complete | ✅ Better Tested | **Most Reliable** |
| **Rust**     | ✅ Complete | ✅ Complete | ⚠️ Minimal Testing | **Needs Testing** |
| **Julia**    | ✅ Complete | ✅ Complete | ⚠️ Minimal Testing | **Needs Testing** |
| **Go**       | ✅ Complete | ✅ Complete | ⚠️ Minimal Testing | **Needs Testing** |
| **Python**   | ✅ Complete | ✅ Complete | ⚠️ Minimal Testing | **Needs Testing** |
| **Zig**      | ⚠️ Partial  | ❌ Build Issues | ⚠️ Minimal Testing | **In Development** |

**⚠️ Important**: While all implementations are coded, only Perl has sufficient testing. Other implementations need comprehensive testing and validation.

## 🏗️ Architecture

### Three-Phase Pipeline

```
EBNF Grammar → Raw AST JSON → Transformed AST JSON → Parser Code
     ↓              ↓                  ↓               ↓
  (Perl)      (Multi-language)   (Multi-language)   (Perl)
```

### Data Flow

1. **EBNF Input** → `perl/ebnf_to_json.pl` → **Raw AST JSON**
2. **Raw AST JSON** → Multi-language implementations → **Transformed AST JSON** 
3. **Transformed AST JSON** → `perl/AST/Transform.pm` → **Parser Code**

### Five-Stage Transformation Pipeline

All language implementations follow the same transformation stages:

1. **Extract Annotations**: Preserve semantic/logging metadata, clean grammar tokens
2. **Group by OR**: Split rule alternatives on "|" operators
3. **Handle Parentheses**: Process grouping constructs and nested structures
4. **Parse Sequences**: Build structured AST nodes from token sequences
5. **Build Tree**: Assemble final grammar tree with proper node relationships

## 📝 Annotation System

The system supports three types of annotations:

### Semantic Annotations
```ebnf
@type: "Expression"
@range: {min: 0, max: 1000}
expression := term ('+' term)*
```
**Format**: `['semantic_annotation', [<name>, <value>]]`
**Purpose**: Static metadata about rule semantics

### Logging Annotations  
```ebnf
@log: "Processing term"
@debug: "Captured value", "$1"
term := factor ('*' factor)*
```
**Format**: `['logging_annotation', [<name>, [<arg1>, <arg2>, ...]]]`
**Purpose**: Dynamic runtime logging during parsing

### Return Annotations
```ebnf
@return_scalar: "number"
number := /(\d+)/
```
**Format**: `['return_scalar'|'return_array'|'return_object', <type>]`
**Purpose**: Specify parser return value transformation

## 🔧 Installation & Usage

### Prerequisites
- **Perl**: 5.20+, JSON module
- **Rust**: 1.70+, cargo
- **Julia**: 1.8+, JSON3 package
- **Go**: 1.19+
- **Python**: 3.8+
- **Zig**: 0.15.1+ (build system needs fixing)

### Build Commands

#### Rust
```bash
cd rust
cargo build
cargo test
cargo run input.json output.json
```

#### Julia
```bash
cd julia
julia -e "using Pkg; Pkg.activate(.); Pkg.instantiate()"
julia ast_pipeline.jl input.json output.json
```

#### Go
```bash
cd go
go build ast_pipeline.go
go test
./ast_pipeline input.json output.json
```

#### Python
```bash
cd python
python ast_pipeline.py input.json output.json
python -m pytest tests/ # (when tests are created)
```

#### Perl
```bash
cd perl
perl ebnf_to_json.pl input.ebnf output.json
perl -Mlib=. -MAST::Transform -e "test_suite()" # (existing tests)
```

## 📊 Example

### Input EBNF
```ebnf
@type: "Arithmetic"
expression := term ('+' term)*

@log: "Processing term"
term := factor ('*' factor)*

@examples: [42, 123, 999]
factor := number | '(' expression ')'

number := /(\d+)/
```

### Output Transformed AST (excerpt)
```json
{
  "grammar_name": "arithmetic",
  "grammar_tree": {
    "expression": {
      "type": "sequence",
      "elements": [
        {"type": "atom", "value": ["identifier", "term"]},
        {
          "type": "quantified",
          "element": {
            "type": "sequence", 
            "elements": [
              {"type": "atom", "value": ["operator", "+"]},
              {"type": "atom", "value": ["identifier", "term"]}
            ]
          },
          "quantifier": "*"
        }
      ]
    }
  },
  "metadata": {
    "annotations": {
      "semantic_annotations": {
        "expression": ["type:Arithmetic"],
        "factor": ["examples:42,123,999"]
      },
      "logging_annotations": {
        "term": ["log(Processing term)"]
      }
    }
  }
}
```

## 🧪 Testing Status

### Current State
- **Perl**: Has validation tests and real-world usage examples
- **Other Languages**: Basic compilation tests only
- **Cross-Language**: No systematic compatibility validation
- **Integration**: Limited end-to-end testing

### Testing Needs
- [ ] Comprehensive unit tests for all non-Perl implementations
- [ ] Integration tests with complex grammar files  
- [ ] Cross-language JSON compatibility validation
- [ ] Error handling and edge case coverage
- [ ] Performance benchmarks and scalability testing

## 🗺️ Roadmap

### Phase 1: Stabilization (High Priority)
- [ ] **Complete Zig implementation** (fix build system issues)
- [ ] **Create comprehensive test suites** for all languages
- [ ] **Cross-language validation** (ensure equivalent JSON output)
- [ ] **Bug discovery and fixing** through systematic testing

### Phase 2: Parser Generation (Medium Priority)  
- [ ] **Complete parser code generation** (Perl-based)
- [ ] **Left-recursion elimination** for grammar transformation
- [ ] **Optimization passes** for generated parser performance

### Phase 3: Advanced Features (Lower Priority)
- [ ] **Syntactic data generation** from grammar + semantic annotations
- [ ] **Grammar analysis tools** (validation, conflict detection)
- [ ] **Performance profiling** and optimization

### Phase 4: Extended Support (Future)
- [ ] **Additional languages** (C/C++, C#, Java, Swift)
- [ ] **IDE integration** (Language Server Protocol)
- [ ] **Web interface** for grammar editing
- [ ] **Package manager distribution** (cargo, npm, pip)

## 🤝 Contributing

### High-Impact Opportunities
1. **Testing**: Create comprehensive test suites for non-Perl implementations
2. **Zig Completion**: Fix build system and complete implementation
3. **Cross-Language Validation**: Ensure JSON compatibility between languages
4. **Bug Discovery**: Run systematic tests to find implementation issues

### Getting Started
1. Read [`IMPLEMENTATION_GUIDE.md`](IMPLEMENTATION_GUIDE.md) for detailed technical guide
2. Check [`CURRENT_STATUS.md`](CURRENT_STATUS.md) for accurate project status
3. Review [`PROJECT_OVERVIEW.md`](PROJECT_OVERVIEW.md) for complete technical details

### Development Workflow
- **Most Reliable**: Use Perl implementation for reference
- **Testing First**: Create tests before claiming implementations work
- **Cross-Language**: Validate JSON compatibility between implementations
- **Documentation**: Update docs for any API changes

## 📚 Documentation

- **[PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md)**: Complete technical architecture
- **[IMPLEMENTATION_GUIDE.md](IMPLEMENTATION_GUIDE.md)**: Developer guide with implementation details  
- **[CURRENT_STATUS.md](CURRENT_STATUS.md)**: Accurate project status and testing gaps
- **[docs/](docs/)**: Additional technical documentation

## ⚠️ Important Notes

1. **Testing Required**: Only Perl implementation has adequate testing
2. **Expect Bugs**: Non-Perl implementations are largely untested
3. **Cross-Language Compatibility**: Designed but not validated
4. **Production Use**: Not recommended without comprehensive testing
5. **Zig Issues**: Build system needs updating for Zig 0.15.1

## 📄 License

[License information to be added]

## 🏷️ Keywords

`EBNF` `parser-generator` `AST` `multi-language` `Perl` `Rust` `Julia` `Go` `Python` `Zig` `JSON` `annotations` `grammar` `compilation` `transformation-pipeline`
