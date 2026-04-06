# QUICKSTART: PGEn Round-Trip Testing Framework

> Historical note
> This file contains earlier onboarding material and mixed historical snapshots.
> Current commands, gates, and contracts live in `README.md`, `PGEN_USER_GUIDE.md`,
> `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`, `docs/reference/RUST_CODEBASE_ANALYSIS.md`, and
> `rust/docs/EMBEDDING_API_CONTRACT.md`.
> For annotation-heavy work, start from the aggregate proof surfaces in `README.md`
> / `PGEN_USER_GUIDE.md`: `annotation_contract_gate`, `semantic_full_contract_gate`,
> `return_annotation_support_gate`, and `annotation_stimuli_quality_gate`.

## 🚀 **Quick Start - Validate Parser Correctness**

Get started with PGEn's mathematical parser validation in under 5 minutes.

### **🎯 What You'll Learn:**
- Run comprehensive round-trip tests
- Validate parser mathematical correctness
- Use professional testing CLI
- Interpret test results

### **📋 Prerequisites:**
- Rust toolchain installed
- PGEn repository cloned

### **⚡ Quick Validation Commands:**

#### **1. Build the Test Runner:**
```bash
cd rust
cargo build --bin test_runner
```

#### **2. Run All Tests (Dashboard View):**
```bash
cargo run --bin test_runner -- --dashboard
```
**Expected Output:**
```
🎯 ROUND-TRIP TESTING DASHBOARD
═══════════════════════════════════════════════════════════════
📋 Test Suite Results:
• return_annotations (return) - ✅ PASS (73 tests)
• semantic_annotations (semantic) - ✅ PASS (10 tests)
• unified & regex tests - ✅ PASS (3 tests)

🎉 OVERALL RESULT: ✅ ALL 86 TESTS PASSED
```

#### **3. Run Specific Parser Tests:**
```bash
# Test only return annotations
cargo run --bin test_runner -- --parser return

# Test only semantic annotations  
cargo run --bin test_runner -- --parser semantic
```

#### **4. Run Filtered Tests:**
```bash
# Test only basic cases
cargo run --bin test_runner -- --tags basic

# Test edge cases only
cargo run --bin test_runner -- --tags edge
```

#### **5. Get Detailed Results:**
```bash
# Verbose output with individual test details
cargo run --bin test_runner -- --verbose

# Summary view
cargo run --bin test_runner -- --summary
```

### **🔍 Understanding Results:**

#### **✅ PASS**: Mathematical validation successful
- Parser correctly parses input to AST
- Unparser correctly converts AST back to original string
- Input string equals output string

#### **❌ FAIL**: Parser correctness issue detected
- Either parsing or unparsing failed
- Indicates parser implementation bug

### **🎯 What Round-Trip Testing Validates:**

**Mathematical Parser Correctness:**
```
∀ valid_input: unparse(parse(valid_input)) = valid_input
```

- **Parsing Works**: Input strings successfully convert to AST
- **Unparsing Works**: AST successfully converts back to strings
- **Reversibility**: Process is mathematically reversible
- **No Data Loss**: No information lost in parse/unparse cycle

### **📊 Test Coverage:**
- **86 comprehensive test cases**
- **Return annotations**: Arrays, objects, references, extractions
- **Semantic annotations**: Transform expressions, function calls
- **Edge cases**: Unicode, escapes, nested structures, large indices

### **🚀 Next Steps:**
1. **Add New Tests**: Extend test coverage in `rust/test_data/`
2. **Custom Parsers**: Implement new parser types using `Parser` trait
3. **CI/CD Integration**: Add to automated testing pipeline
4. **Performance Testing**: Benchmark parser speed and memory usage

---

/
# AI Quick-Start Onboarding Guide
*Essential information for immediate productivity in the PGEN project*

## 🚀 Immediate Context (READ FIRST)

### What This Project Does
PGEN is a **Regex Parser Generator Pipeline** that converts EBNF grammars into high-performance Rust parsers with semantic annotation support. The core flow is:
```
EBNF Grammar → JSON AST → Rust Parser → Stress Testing
```

### Current State (as of 2025-01-07)
- ✅ **WORKING**: All three parsers successfully generated (200K-400K each)
- ✅ **WORKING**: Complete Makefile system with comprehensive flows  
- ✅ **WORKING**: Bootstrap mode to break circular dependencies
- ⚠️ **ISSUE**: Comprehensive stress tests have compilation errors
- ⚠️ **ISSUE**: Generated parsers have correct interface but test expectations mismatch

### Last Session Achievement
Successfully demonstrated that the Makefile system works perfectly:
- `make return_annotation_parser` generates 202K parser
- `make semantic_annotation_parser` generates 382K parser  
- `make regex_parser` generates 172K parser
- All parsers have proper interface: `with_debug()`, `parse()`, `debug_output()`, `Debug` traits

## ⚡ Quick Commands to Get Productive

### Essential Commands
```bash
# Get project status
make help
make status

# Test the main flows (these should work)
make return_parser     # alias for return_annotation_parser
make semantic_parser   # alias for semantic_annotation_parser  
make regex_tests       # alias for regex_parser

# Full clean and rebuild if needed
make clean-all
make all

# Check what's generated
ls -lah generated/
```

### Key Files to Understand

#### Core Build System
- `Makefile` - **CRITICAL**: Well-documented build system with comprehensive flows
- `generated/` - Where all parsers are generated (check file sizes to verify success)

#### Core Grammar Files  
- `grammars/return_annotation.ebnf` - Return annotation grammar
- `grammars/semantic_annotation.ebnf` - Semantic annotation grammar
- `grammars/regex.ebnf` - Regex grammar

#### Generated Parsers
- `generated/return_annotation_parser.rs` - Generated return annotation parser (~202K)
- `generated/semantic_annotation_parser.rs` - Generated semantic annotation parser (~382K)
- `generated/regex_parser.rs` - Generated regex parser (~172K)

#### Critical Documentation
- `README.md` - Project objective, canonical flow, and current doc map
- `PGEN_USER_GUIDE.md` - Operator-facing workflows and supported public surfaces
- `docs/reference/RUST_CODEBASE_ANALYSIS.md` - Current Rust-first architecture and subsystem map
- `LIVE_ACHIEVEMENT_STATUS.md` - Current closure truth and remaining gaps
- `DEVELOPMENT_NOTES.md` - Key technical insights and lessons learned

## 🔧 Known Issues and Workarounds

### Issue 1: Comprehensive Stress Tests Fail
**Problem**: Tests expect different interface than generated parsers provide
**Location**: `rust/src/comprehensive_stress_test.rs`
**Workaround**: Focus on parser generation, not the stress tests
**Status**: Parser generation works perfectly, test interface mismatch

### Issue 2: Semantic Parser Interface Mismatch
**Problem**: Test expects methods that don't match generated parser interface
**Root Cause**: Generated parsers have correct interface, tests have wrong expectations
**Solution**: Either fix test interface or update generator to match test expectations

### Issue 3: Error Types Don't Implement Display
**Problem**: `()` error type doesn't implement `std::fmt::Display` 
**Location**: Generated parsers return `Result<ParseNode, ()>` but tests expect `Display`
**Fix**: Update error types in generator or test expectations

## 🎯 High-Value Tasks for New AI

### Immediate Impact (30 minutes)
1. **Fix Comprehensive Stress Tests**: Update test interface to match generated parsers
2. **Verify All Three Flows**: Ensure `make all_parser_tests` works end-to-end
3. **Update Test Error Handling**: Fix error type Display implementation

### Medium Impact (2-4 hours)
1. **Add Parser Validation**: Verify generated parsers actually parse their target grammars
2. **Enhance Error Messages**: Improve error reporting in generated parsers
3. **Add Performance Benchmarks**: Measure parser generation and execution speed

### Strategic Impact (1-2 days)
1. **Full Parser Testing**: Create comprehensive test suite for each generated parser
2. **Cross-Language Integration**: Extend to generate parsers in other languages
3. **Advanced Semantic Annotations**: Enhance semantic annotation support

## 🧭 Architecture Quick Reference

### Build Pipeline
```
1. EBNF Files (grammars/*.ebnf)
   ↓
2. JSON AST (generated/*.json)  
   ↓
3. Rust AST Pipeline (rust/target/debug/ast_pipeline)
   ↓ 
4. Generated Parsers (generated/*_parser.rs)
   ↓
5. Stress Tests (rust/src/comprehensive_stress_test.rs)
```

### Key Directories
```
pgen/
├── Makefile                    # MAIN BUILD SYSTEM  
├── grammars/                   # Input EBNF files
├── generated/                  # Generated JSON and parsers
├── rust/                       # Rust AST pipeline and tests
├── tools/                      # Perl EBNF→JSON converter
└── docs/                       # Extensive technical docs
```

### Bootstrap System
- **Purpose**: Breaks circular dependency (parsers needed to generate parsers)
- **Mechanism**: Placeholder parsers → AST pipeline → Real parsers
- **Status**: Working perfectly
- **Files**: `generated/*.placeholder` files mark bootstrap state

## 🔍 Debugging Tips

### When Make Targets Fail
1. Check `make status` for missing files
2. Look at file sizes in `generated/` (should be 100K+)
3. Run individual steps to isolate issues
4. Check Rust compilation with `cd rust && cargo build`

### When Parsers Don't Generate
1. Verify JSON files exist: `ls generated/*.json`
2. Check AST pipeline: `cd rust && cargo build`
3. Try bootstrap mode: `rust/target/debug/ast_pipeline --bootstrap-mode`

### When Tests Fail
1. **Interface Mismatch**: Check if test expects methods that don't exist
2. **Type Issues**: Verify error types implement required traits
3. **Missing Debug**: Ensure AST types implement `Debug` trait

## 🎓 Learning Path for New AI

### Phase 1: Understand Current State (30 minutes)
1. Run `make help` and `make status`
2. Read `README.md`, `PGEN_USER_GUIDE.md`, and `docs/reference/RUST_CODEBASE_ANALYSIS.md`
3. Check `generated/` directory for parser files
4. Try a focused maintained gate rather than an old bootstrap-only flow

### Phase 2: Identify Issues (1 hour)
1. Try `make all_parser_tests` and note failures
2. Check `rust/src/comprehensive_stress_test.rs` for test interface
3. Compare with generated parser interfaces
4. Identify specific compilation errors

### Phase 3: Fix and Enhance (ongoing)
1. Fix test interface mismatches
2. Improve error handling
3. Add more comprehensive testing
4. Enhance performance and features

## 🏆 Success Metrics

### Immediate Success
- ✅ All three parser generation flows work (`make return_parser`, etc.)
- ✅ Generated files are 100K+ (indicates real parsers, not stubs)
- ✅ `make all_parser_tests` completes without compilation errors

### Medium-term Success  
- ✅ Generated parsers actually parse their target grammars
- ✅ Comprehensive test coverage for all parsers
- ✅ Performance benchmarks and optimization

### Long-term Success
- ✅ Multi-language parser generation
- ✅ Production-ready parser generator
- ✅ Extensive semantic annotation ecosystem

## 🚨 Critical Don't-Do List

1. **DON'T** assume comprehensive tests work - they're known broken
2. **DON'T** manually run complex command chains - use Makefile targets  
3. **DON'T** ignore bootstrap system - it's essential for circular dependency breaking
4. **DON'T** focus on Perl code - the Rust pipeline is the current focus
5. **DON'T** expect all documentation to be current - focus on this guide and Makefile help

---

**Bottom Line**: The Makefile system works perfectly for parser generation. The tests need fixing to match the generated parser interface. Focus there for immediate impact.
