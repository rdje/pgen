# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## 🚀 Quick Start (60 seconds)

Get productive immediately with the Makefile system:

```bash
# Check project status
make status

# Test complete parser generation flows
make return_parser      # Return annotation parser (202K file)
make semantic_parser    # Semantic annotation parser (382K file)  
make regex_tests       # Regex parser (172K file)

# Run all parser flows
make all_parser_tests

# Clean rebuild if needed
make clean && make bootstrap-test
```

## Repository Overview

**PGEN** is a Multi-Language EBNF Parser Generator with comprehensive AST transformation pipeline. It converts EBNF grammar specifications into executable parsers across multiple programming languages while preserving semantic and logging annotations.

**Current Status**: Rust implementation is production-ready with comprehensive Makefile system. Other languages (Julia, Go, Python) are complete but need testing.

### Core Architecture (Three-Phase Pipeline)

```
EBNF Grammar → Raw AST JSON → Transformed AST JSON → Parser Code
     ↓              ↓                  ↓               ↓
 (Perl tools)  (Multi-language)   (Multi-language)   (Rust)
```

**Key Innovation**: Bootstrap system solves circular dependencies - parsers that generate parsers.

## Essential Daily Commands

### Project Status & Help
```bash
make help           # Show all available targets with descriptions
make status         # Check which components are built
make bootstrap-status # Check bootstrap system state
```

### Parser Generation Flows
```bash
# Individual parser generation (complete EBNF → JSON → Rust flows)
make return_annotation_parser    # or: make return_parser
make semantic_annotation_parser  # or: make semantic_parser  
make regex_parser               # or: make regex_tests

# Run all parsers end-to-end
make all_parser_tests
```

### Build Management
```bash
make clean          # Remove generated files
make clean-all      # Remove all build artifacts
make rebuild        # Clean + build from scratch
make bootstrap-test # Full clean-to-build verification
```

### Debug & Development
```bash
make debug-json     # Generate regex JSON with debug output
make test-parser    # Test generated parsers
make force-debug-json # Force JSON regeneration ignoring deps
```

## Directory Structure & Key Files

### Build System
- **`Makefile`** - Complete build system with comprehensive flows and bootstrap handling
- **`generated/`** - All generated JSON and parser files (check file sizes: 100K-400K indicates success)

### Grammar Sources
- **`grammars/*.ebnf`** - Input EBNF grammar files
  - `return_annotation.ebnf` - Return annotation grammar
  - `semantic_annotation.ebnf` - Semantic annotation grammar  
  - `regex.ebnf` - Regex grammar
  - `ebnf.ebnf` - Self-hosting EBNF grammar

### Tools & Implementation
- **`tools/ebnf_to_json.pl`** - EBNF → JSON converter (Perl)
- **`rust/`** - Production Rust AST pipeline implementation
- **`perl/`** - Original Perl implementation (most tested)
- **`julia/`, `go/`, `python/`** - Other language implementations

### Documentation
- **`QUICKSTART_AI_ONBOARDING.md`** - Comprehensive AI onboarding guide
- **`PROJECT_OVERVIEW.md`** - Technical architecture details
- **`DEVELOPMENT_NOTES.md`** - Key technical insights and lessons learned

## Common Troubleshooting

### Build Failures
1. **Check status**: `make status` to identify missing components
2. **Bootstrap issues**: `make clean-all && make bootstrap-test`
3. **File sizes**: Generated parsers should be 100K+ (not empty stubs)

### Test Interface Mismatches
**Current Issue**: Comprehensive stress tests may fail due to interface expectations vs generated parser reality.
**Workaround**: Focus on parser generation (`make return_parser` etc.) - the Makefile flows work perfectly.

### Missing Dependencies  
- **Perl**: 5.20+, JSON module required for tools
- **Rust**: 1.70+, cargo (for AST pipeline)
- **Generated files**: Check `generated/` directory for proper parser files

## Multi-Language Status

| Language | AST Pipeline | Testing | Status |
|----------|-------------|---------|---------||
| **Perl** | ✅ Complete | ✅ Well Tested | **Most Reliable** |
| **Rust** | ✅ Complete | ✅ Production Ready | **Recommended** |
| **Julia** | ✅ Complete | ⚠️ Needs Testing | Ready for enhancement |
| **Go** | ✅ Complete | ⚠️ Needs Testing | Ready for enhancement |
| **Python** | ✅ Complete | ⚠️ Needs Testing | Ready for enhancement |
| **Zig** | ⚠️ Partial | ❌ Build Issues | In Development |

### Language-Specific Commands

#### Rust (Primary Implementation)
```bash
cd rust
cargo build        # Build AST pipeline
cargo test         # Run tests
cargo run input.json output.json  # Manual AST transformation
```

#### Julia
```bash
cd julia
julia ast_pipeline.jl input.json output.json
```

#### Go
```bash
cd go
go build ast_pipeline.go
./ast_pipeline input.json output.json
```

#### Python  
```bash
cd python
python ast_pipeline.py input.json output.json
```

## EBNF Grammar Development

### Critical Rules for Grammar Files
- **Regex Capturing Groups**: When using `$1, $2, etc.` in return annotations, regex patterns MUST have capturing groups
- **File Extensions**: `.ts` files are TableScript (NOT TypeScript)
- **Signal Naming**: 1-bit active low reset signals end with `_n` or `_b`

### Example EBNF with Annotations
```ebnf
@type: "Expression" 
@range: {min: 0, max: 1000}
expression := term ('+' term)*

@log: "Processing term"
term := factor ('*' factor)*

@examples: [42, 123, 999] 
factor := number | '(' expression ')'

number := /(\d+)/ -> $1  # Note: capturing group required
```

## Development Workflow Best Practices

### Documentation Maintenance
According to project rules, regularly update these files:
- **`CHANGES.md`** - Technical change history with root cause analysis
- **`DEVELOPMENT_NOTES.md`** - Architecture insights and design principles  
- **`git_message_brief.txt`** - Clean commit messages for git commit -F

### Git Version Control Best Practices
- **Moving files**: Use `git mv old_name new_name` instead of `mv` for Git-tracked files
- **Deleting files**: Use `git rm file_name` instead of `rm` for Git-tracked files  
- **Rationale**: Preserves file history and ensures Git properly tracks changes
- **New files**: Use normal `git add new_file` workflow

### Debug Practices  
- Add debug messages with file/function context: `[filename.pm][function_name]`
- Use `--debug` or `--quiet` flags in tools
- Comprehensive logging for all decision points and function calls

### Parser Debug Output Formatting Requirements (Universal Standard)
**Applies to ALL parser debug output**: stress tests, individual tests, Makefile targets, `--debug` option

#### **Core Principle: HUMAN-READABLE FIRST**
Debug output must be instantly understandable by humans, not technical blobs of text.

#### **Hierarchical Rule Processing**
- **Format**: For non-top rules, display as `rule-top → ... → RULE`
- **Unicode Arrow**: Use proper `→` (U+2192) symbol instead of ` > `
- **Visual Separation**: Empty line before each non-top rule
- **Top Rules**: `semantic_annotation →`, `return_annotation →`, `regex →`

#### **Rule Success/Failure Clarity**
- **Success Indicators**: Use clear ✅ symbols and descriptive text
- **Failure Indicators**: Use clear ❌ symbols with reason
- **Visual Impact**: Each outcome must be immediately obvious
- **Context Preservation**: Show what was being parsed when success/failure occurred

#### **Spacing and Visual Structure**
- **No Text Blobs**: Generous whitespace between logical sections
- **Consistent Indentation**: Clear visual hierarchy
- **Logical Grouping**: Related information grouped with spacing
- **Scannable Format**: Easy to skim and find specific information

#### **Example Top-Notch Debug Format**:
```
🔍 PARSING: "@type: \"Expression\""

   1: semantic_annotation
      ✅ Starting semantic annotation parse
      📍 Position: 0, Input: "@type: \"Expression\""

   2: semantic_annotation → annotation_name  
      🔍 Attempting to parse annotation name
      📍 Position: 1, Looking for: identifier after '@'
      ✅ SUCCESS: Found 'type'
      📊 Consumed: 4 characters

   3: semantic_annotation → colon
      🔍 Expecting ':' separator
      📍 Position: 5
      ✅ SUCCESS: Found ':'
      📊 Consumed: 1 character

   4: semantic_annotation → annotation_value → string_literal
      🔍 Parsing string literal value
      📍 Position: 7, Looking for: quoted string
      ✅ SUCCESS: Found '\"Expression\"'
      📊 Consumed: 12 characters

✅ PARSE COMPLETE: semantic_annotation
📊 Total consumed: 18/18 characters
🎯 Result: SemanticAnnotation { name: "type", value: "Expression" }
```

#### **Failure Example Format**:
```
🔍 PARSING: "@invalid_syntax"

   1: semantic_annotation
      ✅ Starting semantic annotation parse
      📍 Position: 0, Input: "@invalid_syntax"

   2: semantic_annotation → annotation_name
      🔍 Attempting to parse annotation name  
      📍 Position: 1, Looking for: identifier after '@'
      ✅ SUCCESS: Found 'invalid_syntax'
      📊 Consumed: 14 characters

   3: semantic_annotation → colon
      🔍 Expecting ':' separator
      📍 Position: 15
      ❌ FAILURE: Found end of input, expected ':'
      💡 Suggestion: Add ':' and value after annotation name

❌ PARSE FAILED: semantic_annotation
📍 Failed at: Position 15
🎯 Reason: Missing required ':' separator
💡 Fix: Use format "@name: value"
```

#### **Automatic Log File Creation**
- **File Naming**: `<parser>_<test>_<timestamp>.log` (e.g., `semantic_annotation_stress_test_1_20250926_143022.log`)
- **Auto-Generation**: Created automatically when using `with_debug_log()` constructor
- **Content**: Full debug trace with professional formatting and metadata header
- **Location**: Current working directory (ignored by .gitignore)

#### **Usage Examples**
```rust
// Automatic log file creation
let mut parser = SemanticAnnotationParser::with_debug_log(input, "basic_test");
let result = parser.parse(); // Automatically writes: semantic_annotation_basic_test_20250926_143022.log

// Traditional debug (console only)
let mut parser = SemanticAnnotationParser::with_debug(input);
```

#### **Universal Application**
This format applies to:
- Stress test debug output
- Individual parser test runs  
- `make test-*` targets
- Generated parser `with_debug()` and `with_debug_log()` output
- CLI `--debug` and `--trace` modes
- Any parser debugging context

### Grammar Testing
1. Start with simple patterns, add complexity incrementally
2. Test with `make debug-json` for detailed output
3. Use bootstrap mode for initial development
4. Validate with multiple input patterns

## Advanced Features

### Bootstrap System
- **Purpose**: Breaks circular dependency (parsers needed to generate parsers)
- **Mechanism**: Minimal placeholder parsers → Real parsers → Enhanced functionality
- **Usage**: Automatically handled by Makefile system

### Annotation System
- **Semantic Annotations**: `@type`, `@range`, `@validation` - static metadata
- **Logging Annotations**: `@log`, `@debug` - runtime logging during parsing
- **Return Annotations**: Specify parser return value transformations

### Self-Hosting Capability
- Parser generator parses its own EBNF grammar specifications
- No bootstrap limitations for grammar complexity
- Unlimited extensibility through self-modification

## Performance Baseline
- **Parser Generation**: 29K+ parses/second baseline performance  
- **Memory Usage**: ~1MB per 1000 grammar rules for raw AST
- **Build Time**: Complete parser generation typically under 30 seconds

## Related Documentation

For comprehensive technical details, see:
- **`QUICKSTART_AI_ONBOARDING.md`** - Essential information for immediate productivity
- **`PROJECT_OVERVIEW.md`** - Complete architecture and data flow pipeline  
- **`DEVELOPMENT_NOTES.md`** - Technical knowledge base and lessons learned
- **`docs/ast_transformation_pipeline.md`** - Detailed transformation algorithms
- **`docs/BOOTSTRAP_MODE_SPECIFICATION.md`** - Bootstrap system documentation

---

**Quick Reference**: The Makefile system (`make help`) is your primary interface. Start with `make return_parser` to verify the system works, then explore other flows.
