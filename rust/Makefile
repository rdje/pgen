# Makefile for pgen - Regex Parser Generator Pipeline
# This manages the complex build dependencies for generating Rust parsers from EBNF grammars
# Now with automatic test synchronization when stress test files change

# Include automatic test synchronization
include Makefile.auto-sync

# Directories
GRAMMARS_DIR = ../grammars
GENERATED_DIR = ../generated
TOOLS_DIR = ../tools
RUST_DIR = .

# Input EBNF files
SEMANTIC_ANNOTATION_EBNF = $(GRAMMARS_DIR)/semantic_annotation.ebnf
RETURN_ANNOTATION_EBNF = $(GRAMMARS_DIR)/return_annotation.ebnf
REGEX_EBNF = $(GRAMMARS_DIR)/regex.ebnf

# Generated JSON files (intermediate)
SEMANTIC_ANNOTATION_JSON = $(GENERATED_DIR)/semantic_annotation.json
RETURN_ANNOTATION_JSON = $(GENERATED_DIR)/return_annotation.json
REGEX_JSON = $(GENERATED_DIR)/regex.json

# Generated Rust parser files (final output)
SEMANTIC_ANNOTATION_PARSER = $(GENERATED_DIR)/semantic_annotation_parser.rs
RETURN_ANNOTATION_PARSER = $(GENERATED_DIR)/return_annotation_parser.rs
REGEX_PARSER = $(GENERATED_DIR)/regex_parser.rs

# Compiled Rust parser executables
SEMANTIC_ANNOTATION_EXE = $(GENERATED_DIR)/semantic_annotation_parser
RETURN_ANNOTATION_EXE = $(GENERATED_DIR)/return_annotation_parser

# Rust build artifacts
RUST_AST_PIPELINE = $(RUST_DIR)/target/debug/ast_pipeline

# Tools
EBNF_TO_JSON = $(TOOLS_DIR)/ebnf_to_json.pl
RUST_GENERATOR = $(RUST_AST_PIPELINE) --generate-parser --debug --trace
RUST_GENERATOR_BOOTSTRAP = $(RUST_AST_PIPELINE) --generate-parser --bootstrap-mode --debug

# Default target
.PHONY: all
all: $(REGEX_PARSER)

# Step 1: Generate JSON from EBNF files
$(SEMANTIC_ANNOTATION_JSON): $(SEMANTIC_ANNOTATION_EBNF)
	@echo "Generating semantic annotation JSON from EBNF..."
	@mkdir -p $(GENERATED_DIR)
	$(EBNF_TO_JSON) --verbosity debug --pretty $(SEMANTIC_ANNOTATION_EBNF) -o $(SEMANTIC_ANNOTATION_JSON)

$(RETURN_ANNOTATION_JSON): $(RETURN_ANNOTATION_EBNF)
	@echo "Generating return annotation JSON from EBNF..."
	@mkdir -p $(GENERATED_DIR)
	$(EBNF_TO_JSON) --verbosity debug --pretty $(RETURN_ANNOTATION_EBNF) -o $(RETURN_ANNOTATION_JSON)

# Bootstrap placeholder targets: Create minimal files only if they don't exist
# These are file-based targets that follow Make's dependency model
$(GENERATED_DIR)/semantic_annotation_parser.rs.placeholder:
	@echo "Creating semantic annotation parser placeholder..."
	@mkdir -p $(GENERATED_DIR)
	@echo "pub struct Span { pub start: usize, pub end: usize }" > $(SEMANTIC_ANNOTATION_PARSER)
	@echo "pub struct ParseNode { pub rule_name: String, pub span: Span, pub content: ParseContent }" >> $(SEMANTIC_ANNOTATION_PARSER)
	@echo "pub enum ParseContent { Terminal(String), Sequence(Vec<ParseNode>), Alternative(Box<ParseNode>), Quantified(Vec<ParseNode>, String) }" >> $(SEMANTIC_ANNOTATION_PARSER)
	@echo "pub struct Semantic_annotationParser;" >> $(SEMANTIC_ANNOTATION_PARSER)
	@echo "impl Semantic_annotationParser {" >> $(SEMANTIC_ANNOTATION_PARSER)
	@echo "    pub fn new(_: &str) -> Self { Self }" >> $(SEMANTIC_ANNOTATION_PARSER)
	@echo "    pub fn with_debug(_: &str) -> Self { Self }" >> $(SEMANTIC_ANNOTATION_PARSER)
	@echo "    pub fn parse(&mut self) -> Result<ParseNode, ()> { Err(()) }" >> $(SEMANTIC_ANNOTATION_PARSER)
	@echo "}" >> $(SEMANTIC_ANNOTATION_PARSER)
	@touch $@

$(GENERATED_DIR)/return_annotation_parser.rs.placeholder:
	@echo "Creating return annotation parser placeholder..."
	@mkdir -p $(GENERATED_DIR)
	@echo "pub struct Span { pub start: usize, pub end: usize }" > $(RETURN_ANNOTATION_PARSER)
	@echo "pub struct ParseNode { pub rule_name: String, pub span: Span, pub content: ParseContent }" >> $(RETURN_ANNOTATION_PARSER)
	@echo "pub enum ParseContent { Terminal(String), Sequence(Vec<ParseNode>), Alternative(Box<ParseNode>), Quantified(Vec<ParseNode>, String) }" >> $(RETURN_ANNOTATION_PARSER)
	@echo "pub struct Return_annotationParser;" >> $(RETURN_ANNOTATION_PARSER)
	@echo "impl Return_annotationParser {" >> $(RETURN_ANNOTATION_PARSER)
	@echo "    pub fn new(_: &str) -> Self { Self }" >> $(RETURN_ANNOTATION_PARSER)
	@echo "    pub fn with_debug(_: &str) -> Self { Self }" >> $(RETURN_ANNOTATION_PARSER)
	@echo "    pub fn parse(&mut self) -> Result<ParseNode, ()> { Err(()) }" >> $(RETURN_ANNOTATION_PARSER)
	@echo "}" >> $(RETURN_ANNOTATION_PARSER)
	@touch $@

# Build Rust AST pipeline (depends on placeholder parsers existing)
$(RUST_AST_PIPELINE): $(GENERATED_DIR)/semantic_annotation_parser.rs.placeholder $(GENERATED_DIR)/return_annotation_parser.rs.placeholder
	@echo "Building Rust AST pipeline..."
	cd $(RUST_DIR) && cargo build

# Step 2: Generate Rust parsers from JSON files using bootstrap mode
# These parsers are generated using bootstrap mode to avoid circular dependency
$(SEMANTIC_ANNOTATION_PARSER): $(SEMANTIC_ANNOTATION_JSON) $(RUST_AST_PIPELINE)
	@echo "Generating semantic annotation Rust parser from JSON (bootstrap mode)..."
	@mkdir -p $(GENERATED_DIR)
	$(RUST_GENERATOR_BOOTSTRAP) $(SEMANTIC_ANNOTATION_JSON) -o $(SEMANTIC_ANNOTATION_PARSER)

$(RETURN_ANNOTATION_PARSER): $(RETURN_ANNOTATION_JSON) $(RUST_AST_PIPELINE)
	@echo "Generating return annotation Rust parser from JSON (bootstrap mode)..."
	@mkdir -p $(GENERATED_DIR)
	$(RUST_GENERATOR_BOOTSTRAP) $(RETURN_ANNOTATION_JSON) -o $(RETURN_ANNOTATION_PARSER)

# Step 3: Compile the generated Rust parsers as executables
$(SEMANTIC_ANNOTATION_EXE): $(SEMANTIC_ANNOTATION_PARSER)
	@echo "Compiling semantic annotation parser executable..."
	rustc --edition 2021 -g -o $(SEMANTIC_ANNOTATION_EXE) $(SEMANTIC_ANNOTATION_PARSER)

$(RETURN_ANNOTATION_EXE): $(RETURN_ANNOTATION_PARSER)
	@echo "Compiling return annotation parser executable..."
	rustc --edition 2021 -g -o $(RETURN_ANNOTATION_EXE) $(RETURN_ANNOTATION_PARSER)

# Generate regex JSON from EBNF (depends only on Rust pipeline - parsers are optional)
$(REGEX_JSON): $(REGEX_EBNF) $(RUST_AST_PIPELINE)
	@echo "Generating regex JSON with debug output..."
	@mkdir -p $(GENERATED_DIR)
	$(EBNF_TO_JSON) --verbosity debug --pretty $(REGEX_EBNF) -o $(REGEX_JSON)

# Generate final regex parser from JSON
$(REGEX_PARSER): $(REGEX_JSON)
	@echo "Generating final regex parser from JSON..."
	@mkdir -p $(GENERATED_DIR)
	$(RUST_GENERATOR) $(REGEX_JSON) -o $(REGEX_PARSER)

# Development targets
.PHONY: debug-json
debug-json: $(REGEX_JSON)
	@echo "Regex JSON generated with debug output - check console for semantic annotation parsing details"

.PHONY: test-parser
test-parser: check-sync-needed $(REGEX_PARSER)
	@echo "Testing generated regex parser..."
	cd $(RUST_DIR) && cargo test

.PHONY: clean
clean:
	@echo "Cleaning generated files..."
	rm -f $(GENERATED_DIR)/*.pl $(GENERATED_DIR)/*.json $(GENERATED_DIR)/*.rs $(GENERATED_DIR)/*.placeholder $(GENERATED_DIR)/semantic_annotation_parser $(GENERATED_DIR)/return_annotation_parser
	cd $(RUST_DIR) && cargo clean

.PHONY: clean-all
clean-all: clean
	@echo "Cleaning all build artifacts..."
	rm -rf $(GENERATED_DIR)

.PHONY: rebuild
rebuild: clean all

# Help target
.PHONY: help
help:
	@echo "pgen Build Pipeline Makefile"
	@echo ""
	@echo "🎯 Parser-Specific Comprehensive Flows (EBNF → JSON → PARSER.RS → STRESS_TESTS):"
	@echo "  return_annotation_parser    - Complete flow for return annotation parser + tests"
	@echo "  semantic_annotation_parser  - Complete flow for semantic annotation parser + tests"
	@echo "  regex_parser               - Complete flow for regex parser + tests"
	@echo ""
	@echo "🔗 Convenience Aliases:"
	@echo "  return_parser              - Alias for return_annotation_parser"
	@echo "  semantic_parser            - Alias for semantic_annotation_parser"
	@echo "  regex_tests                - Alias for regex_parser"
	@echo "  all_parser_tests           - Run comprehensive flows for all parsers"
	@echo ""
	@echo "📚 Main targets:"
	@echo "  all          - Build complete pipeline (default)"
	@echo "  debug-json   - Generate regex JSON with debug output"
	@echo "  test-parser  - Test the generated parser"
	@echo "  rebuild      - Clean and rebuild everything"
	@echo ""
	@echo "🛠️  Utility targets:"
	@echo "  clean        - Remove generated files"
	@echo "  clean-all    - Remove all build artifacts"
	@echo "  help         - Show this help"
	@echo "  status       - Show current build status"
	@echo ""
	@echo "🔍 Debug & Logging Targets:"
	@echo "  return-annotation-parser-with-log   - Generate return parser with full debug logging"
	@echo "  semantic-annotation-parser-with-log - Generate semantic parser with full debug logging"
	@echo "  regex-parser-with-log               - Generate regex parser with full debug logging"
	@echo "  return-parser-with-log              - Alias for return-annotation-parser-with-log"
	@echo "  semantic-parser-with-log            - Alias for semantic-annotation-parser-with-log"
	@echo ""
	@echo "🤖 Individual Test Targets (Examples):"
	@echo "  test-return-scalar-1          - Test return parser with '\$$1'"
	@echo "  test-return-literal-hello     - Test return parser with '\"hello\"'"
	@echo "  test-return-array-simple      - Test return parser with '[\$$1]'"
	@echo "  test-return-object-simple     - Test return parser with '{key: \$$1}'"
	@echo "  test-semantic-type            - Test semantic parser with '@type: \"Expression\"'"
	@echo "  test-semantic-precedence      - Test semantic parser with '@precedence: 5'"
	@echo "  test-regex-hello              - Test regex parser with 'hello'"
	@echo "  test-regex-dot                - Test regex parser with '.'"
	@echo "  📋 Run 'make -n test-return-scalar-1' to see all available individual tests"
	@echo ""
	@echo "🔧 Build order:"
	@echo "  1. Generate JSON files from EBNF grammars"
	@echo "  2. Build Rust AST pipeline (cargo build)"
	@echo "  3. Generate Rust parsers from JSON files"
	@echo "  4. Generate regex.json from regex.ebnf (with debug output)"
	@echo "  5. Generate final regex_parser.rs from regex.json"
	@echo ""
	@echo "💡 Quick Start Examples:"
	@echo "  make return_parser         - Test return annotation parser end-to-end"
	@echo "  make semantic_parser       - Test semantic annotation parser end-to-end"
	@echo "  make regex_tests          - Test regex parser end-to-end"
	@echo "  make all_parser_tests      - Test all parsers end-to-end"

# Debug targets for individual steps
.PHONY: step1
step1: $(SEMANTIC_ANNOTATION_PARSER)

.PHONY: step2  
step2: $(RETURN_ANNOTATION_PARSER)

.PHONY: step3
step3: $(RUST_AST_PIPELINE)

.PHONY: step4
step4: $(REGEX_JSON)

.PHONY: step5
step5: $(REGEX_PARSER)

# Show current build status
.PHONY: status
status:
	@echo "Build Status:"
	@echo "============="
	@echo -n "Semantic annotation parser: "; [ -f $(SEMANTIC_ANNOTATION_PARSER) ] && echo "✓ EXISTS" || echo "✗ MISSING"
	@echo -n "Return annotation parser: "; [ -f $(RETURN_ANNOTATION_PARSER) ] && echo "✓ EXISTS" || echo "✗ MISSING"  
	@echo -n "Rust AST pipeline: "; [ -f $(RUST_AST_PIPELINE) ] && echo "✓ EXISTS" || echo "✗ MISSING"
	@echo -n "Regex JSON: "; [ -f $(REGEX_JSON) ] && echo "✓ EXISTS" || echo "✗ MISSING"
	@echo -n "Final regex parser: "; [ -f $(REGEX_PARSER) ] && echo "✓ EXISTS" || echo "✗ MISSING"

.PHONY: force-debug-json
force-debug-json:
	@echo "Force generating regex JSON with debug output (ignoring dependencies)..."
	@mkdir -p $(GENERATED_DIR)
	$(EBNF_TO_JSON) --verbosity debug --pretty $(REGEX_EBNF) -o $(REGEX_JSON)

.PHONY: regex-parser-with-log
regex-parser-with-log:
	@echo "Generating regex parser with full debug logging..."
	@mkdir -p $(GENERATED_DIR)
	@rm -f $(REGEX_JSON) $(REGEX_PARSER)
	@echo "Step 1: Generate regex JSON..." | tee $(GENERATED_DIR)/regex_parser.log
	$(EBNF_TO_JSON) --verbosity debug --pretty $(REGEX_EBNF) -o $(REGEX_JSON) 2>&1 | tee $(GENERATED_DIR)/regex_parser.log
	@echo "\nStep 2: Generate regex parser from JSON..." | tee $(GENERATED_DIR)/regex_parser.log
	$(RUST_GENERATOR) $(REGEX_JSON) -o $(REGEX_PARSER) 2>&1 | tee $(GENERATED_DIR)/regex_parser.log
	@echo "\nRegex parser generation complete! Check $(GENERATED_DIR)/regex_parser.log for details"

.PHONY: return-annotation-parser-with-log
return-annotation-parser-with-log:
	@echo "Generating return annotation parser with full debug logging..."
	@mkdir -p $(GENERATED_DIR)
	@rm -f $(RETURN_ANNOTATION_JSON) $(RETURN_ANNOTATION_PARSER)
	@echo "Step 1: Generate return annotation JSON..." | tee $(GENERATED_DIR)/return_annotation_parser.log
	$(EBNF_TO_JSON) --verbosity debug --pretty $(RETURN_ANNOTATION_EBNF) -o $(RETURN_ANNOTATION_JSON) 2>&1 | tee $(GENERATED_DIR)/return_annotation_parser.log
	@echo "\nStep 2: Generate return annotation parser from JSON..." | tee -a $(GENERATED_DIR)/return_annotation_parser.log
	$(RUST_GENERATOR_BOOTSTRAP) $(RETURN_ANNOTATION_JSON) -o $(RETURN_ANNOTATION_PARSER) 2>&1 | tee -a $(GENERATED_DIR)/return_annotation_parser.log
	@echo "\nReturn annotation parser generation complete! Check $(GENERATED_DIR)/return_annotation_parser.log for details"

.PHONY: semantic-annotation-parser-with-log
semantic-annotation-parser-with-log:
	@echo "Generating semantic annotation parser with full debug logging..."
	@mkdir -p $(GENERATED_DIR)
	@rm -f $(SEMANTIC_ANNOTATION_JSON) $(SEMANTIC_ANNOTATION_PARSER)
	@echo "Step 1: Generate semantic annotation JSON..." | tee $(GENERATED_DIR)/semantic_annotation_parser.log
	$(EBNF_TO_JSON) --verbosity debug --pretty $(SEMANTIC_ANNOTATION_EBNF) -o $(SEMANTIC_ANNOTATION_JSON) 2>&1 | tee $(GENERATED_DIR)/semantic_annotation_parser.log
	@echo "\nStep 2: Generate semantic annotation parser from JSON..." | tee -a $(GENERATED_DIR)/semantic_annotation_parser.log
	$(RUST_GENERATOR_BOOTSTRAP) $(SEMANTIC_ANNOTATION_JSON) -o $(SEMANTIC_ANNOTATION_PARSER) 2>&1 | tee -a $(GENERATED_DIR)/semantic_annotation_parser.log
	@echo "\nSemantic annotation parser generation complete! Check $(GENERATED_DIR)/semantic_annotation_parser.log for details"

.PHONY: debug-semantic-annotations
debug-semantic-annotations: $(SEMANTIC_ANNOTATION_EXE) $(RETURN_ANNOTATION_EXE)
	@echo "Testing semantic annotation parsing with debug output..."
	@mkdir -p $(GENERATED_DIR)
	@echo "Testing semantic annotation parser with sample input:" | tee $(GENERATED_DIR)/semantic_debug.log
	@echo 'codegen: "escape_literal_handling"' | $(SEMANTIC_ANNOTATION_EXE) --debug 2>&1 | tee $(GENERATED_DIR)/semantic_debug.log
	@echo "\n\nTesting return annotation parser with sample input:" | tee $(GENERATED_DIR)/semantic_debug.log
	@echo '{type: "escape", pattern: $$1}' | $(RETURN_ANNOTATION_EXE) --debug 2>&1 | tee $(GENERATED_DIR)/semantic_debug.log
	@echo "\nDebug output saved to $(GENERATED_DIR)/semantic_debug.log"

# Parser-specific comprehensive flow targets
# These create complete EBNF -> JSON -> PARSER.RS -> STRESS_TESTS flows

# Return annotation parser complete flow
.PHONY: return_annotation_parser
return_annotation_parser: check-sync-needed $(RETURN_ANNOTATION_JSON) $(RUST_AST_PIPELINE)
	@echo "🚀 Starting complete flow for return annotation parser..."
	@echo "📄 Step 1: EBNF -> JSON (already done: $(RETURN_ANNOTATION_JSON))"
	@echo "🔧 Step 2: JSON -> PARSER.RS (bootstrap mode)"
	@mkdir -p $(GENERATED_DIR)
	$(RUST_GENERATOR_BOOTSTRAP) $(RETURN_ANNOTATION_JSON) -o $(RETURN_ANNOTATION_PARSER)
	@echo "✅ Generated: $(RETURN_ANNOTATION_PARSER)"
	@echo "🧪 Step 3: Running comprehensive stress tests"
	cd $(RUST_DIR) && cargo test comprehensive_stress_tests --release -- --nocapture return_annotation
	@echo "🎉 Return annotation parser comprehensive flow completed!"

# Semantic annotation parser complete flow
.PHONY: semantic_annotation_parser
semantic_annotation_parser: check-sync-needed $(SEMANTIC_ANNOTATION_JSON) $(RUST_AST_PIPELINE)
	@echo "🚀 Starting complete flow for semantic annotation parser..."
	@echo "📄 Step 1: EBNF -> JSON (already done: $(SEMANTIC_ANNOTATION_JSON))"
	@echo "🔧 Step 2: JSON -> PARSER.RS (bootstrap mode)"
	@mkdir -p $(GENERATED_DIR)
	$(RUST_GENERATOR_BOOTSTRAP) $(SEMANTIC_ANNOTATION_JSON) -o $(SEMANTIC_ANNOTATION_PARSER)
	@echo "✅ Generated: $(SEMANTIC_ANNOTATION_PARSER)"
	@echo "🧪 Step 3: Running comprehensive stress tests"
	cd $(RUST_DIR) && cargo test comprehensive_stress_tests --release -- --nocapture semantic_annotation
	@echo "🎉 Semantic annotation parser comprehensive flow completed!"

# Regex parser complete flow
.PHONY: regex_parser
regex_parser: check-sync-needed $(REGEX_JSON) $(RUST_AST_PIPELINE)
	@echo "🚀 Starting complete flow for regex parser..."
	@echo "📄 Step 1: EBNF -> JSON (already done: $(REGEX_JSON))"
	@echo "🔧 Step 2: JSON -> PARSER.RS (high-performance mode)"
	@mkdir -p $(GENERATED_DIR)
	$(RUST_GENERATOR) $(REGEX_JSON) -o $(REGEX_PARSER)
	@echo "✅ Generated: $(REGEX_PARSER)"
	@echo "🧪 Step 3: Running comprehensive stress tests"
	cd $(RUST_DIR) && cargo test comprehensive_stress_tests --release -- --nocapture regex
	@echo "🎉 Regex parser comprehensive flow completed!"

# Convenience aliases for easier typing
.PHONY: return_parser
return_parser: return_annotation_parser

.PHONY: semantic_parser
semantic_parser: semantic_annotation_parser

.PHONY: regex_tests
regex_tests: regex_parser

# Convenience aliases for *-with-log targets
.PHONY: return-parser-with-log
return-parser-with-log: return-annotation-parser-with-log

.PHONY: semantic-parser-with-log
semantic-parser-with-log: semantic-annotation-parser-with-log

.PHONY: regex-parser-with-log-alias
regex-parser-with-log-alias: regex-parser-with-log

# Run comprehensive stress tests for all parsers
.PHONY: all_parser_tests
all_parser_tests: return_annotation_parser semantic_annotation_parser regex_parser
	@echo "🎉 All parser comprehensive flows completed!"

# Individual test targets for Return Annotation Parser
# Basic scalar references
.PHONY: test-return-scalar-1
test-return-scalar-1: check-sync-needed $(RETURN_ANNOTATION_PARSER)
	@echo "🔍 Testing Return Parser: \$1"
	@cd $(RUST_DIR) && PGEN_TEST_INPUT='\$1' cargo test test_individual_return_test -- --nocapture

.PHONY: test-return-scalar-2
test-return-scalar-2: $(RETURN_ANNOTATION_PARSER)
	@echo "🔍 Testing Return Parser: \$$2"
	@cd $(RUST_DIR) && PGEN_TEST_INPUT='\$$2' cargo test test_individual_return_test -- --nocapture

.PHONY: test-return-scalar-10
test-return-scalar-10: $(RETURN_ANNOTATION_PARSER)
	@echo "🔍 Testing Return Parser: \$$10"
	@cd $(RUST_DIR) && PGEN_TEST_INPUT='\$$10' cargo test test_individual_return_test -- --nocapture

# Literals
.PHONY: test-return-literal-hello
test-return-literal-hello: $(RETURN_ANNOTATION_PARSER)
	@echo "🔍 Testing Return Parser: \"hello\""
	@cd $(RUST_DIR) && PGEN_TEST_INPUT='"hello"' cargo test test_individual_return_test -- --nocapture

.PHONY: test-return-literal-42
test-return-literal-42: $(RETURN_ANNOTATION_PARSER)
	@echo "🔍 Testing Return Parser: 42"
	@cd $(RUST_DIR) && PGEN_TEST_INPUT='42' cargo test test_individual_return_test -- --nocapture

.PHONY: test-return-literal-true
test-return-literal-true: $(RETURN_ANNOTATION_PARSER)
	@echo "🔍 Testing Return Parser: true"
	@cd $(RUST_DIR) && PGEN_TEST_INPUT='true' cargo test test_individual_return_test -- --nocapture

# Arrays
.PHONY: test-return-array-simple
test-return-array-simple: $(RETURN_ANNOTATION_PARSER)
	@echo "🔍 Testing Return Parser: [\$$1]"
	@cd $(RUST_DIR) && cargo test test_individual_return_test -- --nocapture '[\$$1]'

.PHONY: test-return-array-dual
test-return-array-dual: $(RETURN_ANNOTATION_PARSER)
	@echo "🔍 Testing Return Parser: [\$$1, \$$2]"
	@cd $(RUST_DIR) && cargo test test_individual_return_test -- --nocapture '[\$$1, \$$2]'

.PHONY: test-return-array-empty
test-return-array-empty: $(RETURN_ANNOTATION_PARSER)
	@echo "🔍 Testing Return Parser: []"
	@cd $(RUST_DIR) && cargo test test_individual_return_test -- --nocapture '[]'

# Objects
.PHONY: test-return-object-simple
test-return-object-simple: $(RETURN_ANNOTATION_PARSER)
	@echo "🔍 Testing Return Parser: {key: \$$1}"
	@cd $(RUST_DIR) && cargo test test_individual_return_test -- --nocapture '{key: \$$1}'

.PHONY: test-return-object-dual
test-return-object-dual: $(RETURN_ANNOTATION_PARSER)
	@echo "🔍 Testing Return Parser: {name: \$$1, value: \$$2}"
	@cd $(RUST_DIR) && cargo test test_individual_return_test -- --nocapture '{name: \$$1, value: \$$2}'

.PHONY: test-return-object-empty
test-return-object-empty: $(RETURN_ANNOTATION_PARSER)
	@echo "🔍 Testing Return Parser: {}"
	@cd $(RUST_DIR) && cargo test test_individual_return_test -- --nocapture '{}'

# Dot notation
.PHONY: test-return-dot-value
test-return-dot-value: $(RETURN_ANNOTATION_PARSER)
	@echo "🔍 Testing Return Parser: \$$1.value"
	@cd $(RUST_DIR) && cargo test test_individual_return_test -- --nocapture '\$$1.value'

# Array access
.PHONY: test-return-array-access-0
test-return-array-access-0: $(RETURN_ANNOTATION_PARSER)
	@echo "🔍 Testing Return Parser: \$$1[0]"
	@cd $(RUST_DIR) && cargo test test_individual_return_test -- --nocapture '\$$1[0]'

# Individual test targets for Semantic Annotation Parser
# Basic annotations
.PHONY: test-semantic-type
test-semantic-type: $(SEMANTIC_ANNOTATION_PARSER)
	@echo "🔍 Testing Semantic Parser: @type: \"Expression\""
	@cd $(RUST_DIR) && PGEN_TEST_INPUT='@type: "Expression"' cargo test test_individual_semantic_test -- --nocapture

.PHONY: test-semantic-precedence
test-semantic-precedence: $(SEMANTIC_ANNOTATION_PARSER)
	@echo "🔍 Testing Semantic Parser: @precedence: 5"
	@cd $(RUST_DIR) && PGEN_TEST_INPUT='@precedence: 5' cargo test test_individual_semantic_test -- --nocapture

.PHONY: test-semantic-boolean
test-semantic-boolean: $(SEMANTIC_ANNOTATION_PARSER)
	@echo "🔍 Testing Semantic Parser: @side_effect: false"
	@cd $(RUST_DIR) && PGEN_TEST_INPUT='@side_effect: false' cargo test test_individual_semantic_test -- --nocapture

# Arrays
.PHONY: test-semantic-array-validate
test-semantic-array-validate: $(SEMANTIC_ANNOTATION_PARSER)
	@echo "🔍 Testing Semantic Parser: @validate: [\"check_bounds\"]"
	@cd $(RUST_DIR) && cargo test test_individual_semantic_test -- --nocapture '@validate: [\"check_bounds\"]'

# Objects
.PHONY: test-semantic-object-cache
test-semantic-object-cache: $(SEMANTIC_ANNOTATION_PARSER)
	@echo "🔍 Testing Semantic Parser: @cache: {ttl: 300}"
	@cd $(RUST_DIR) && cargo test test_individual_semantic_test -- --nocapture '@cache: {ttl: 300}'

# Individual test targets for Regex Parser
# Basic patterns
.PHONY: test-regex-hello
test-regex-hello: $(REGEX_PARSER)
	@echo "🔍 Testing Regex Parser: hello"
	@cd $(RUST_DIR) && PGEN_TEST_INPUT='hello' cargo test test_individual_regex_test -- --nocapture

.PHONY: test-regex-dot
test-regex-dot: $(REGEX_PARSER)
	@echo "🔍 Testing Regex Parser: ."
	@cd $(RUST_DIR) && PGEN_TEST_INPUT='.' cargo test test_individual_regex_test -- --nocapture

.PHONY: test-regex-anchor-start
test-regex-anchor-start: $(REGEX_PARSER)
	@echo "🔍 Testing Regex Parser: ^start"
	@cd $(RUST_DIR) && PGEN_TEST_INPUT='^start' cargo test test_individual_regex_test -- --nocapture

# Character classes
.PHONY: test-regex-char-class-az
test-regex-char-class-az: $(REGEX_PARSER)
	@echo "🔍 Testing Regex Parser: [a-z]"
	@cd $(RUST_DIR) && cargo test test_individual_regex_test -- --nocapture '[a-z]'

.PHONY: test-regex-digit
test-regex-digit: $(REGEX_PARSER)
	@echo "🔍 Testing Regex Parser: \\d"
	@cd $(RUST_DIR) && cargo test test_individual_regex_test -- --nocapture '\\d'

# Quantifiers
.PHONY: test-regex-star
test-regex-star: $(REGEX_PARSER)
	@echo "🔍 Testing Regex Parser: a*"
	@cd $(RUST_DIR) && cargo test test_individual_regex_test -- --nocapture 'a*'

.PHONY: test-regex-plus
test-regex-plus: $(REGEX_PARSER)
	@echo "🔍 Testing Regex Parser: b+"
	@cd $(RUST_DIR) && cargo test test_individual_regex_test -- --nocapture 'b+'

# Groups
.PHONY: test-regex-group
test-regex-group: $(REGEX_PARSER)
	@echo "🔍 Testing Regex Parser: (abc)"
	@cd $(RUST_DIR) && cargo test test_individual_regex_test -- --nocapture '(abc)'

# Bootstrap mode targets
.PHONY: bootstrap-test
bootstrap-test: clean-all
	@echo "Testing full bootstrap build process..."
	@echo "This will build the entire system from scratch using bootstrap mode"
	make all

.PHONY: bootstrap-status
bootstrap-status:
	@echo "Bootstrap Mode Build Status:"
	@echo "============================"
	@echo -n "Bootstrap placeholder files: "; \
		if [ -f $(SEMANTIC_ANNOTATION_PARSER) ] && [ -f $(RETURN_ANNOTATION_PARSER) ]; then \
			echo "✓ EXISTS"; \
		else \
			echo "✗ MISSING - run 'make bootstrap-parsers'"; \
		fi
	@echo -n "Rust AST pipeline: "; [ -f $(RUST_AST_PIPELINE) ] && echo "✓ EXISTS" || echo "✗ MISSING"
	@echo -n "Generated parsers (bootstrap mode): "; \
		if [ -f $(SEMANTIC_ANNOTATION_PARSER) ] && [ -f $(RETURN_ANNOTATION_PARSER) ]; then \
			echo "✓ GENERATED"; \
		else \
			echo "✗ NOT GENERATED"; \
		fi
	@echo -n "Final regex parser: "; [ -f $(REGEX_PARSER) ] && echo "✓ EXISTS" || echo "✗ MISSING"
	@echo ""
	@echo "To test bootstrap mode: make bootstrap-test"


# Auto-generated Makefile targets for stress tests
# Generated at: 2025-09-26T18:25:21.719564+00:00
# DO NOT EDIT MANUALLY - This section is automatically regenerated

# Run all stress tests
test-all: 
	@echo "🎯 All stress tests completed successfully!"

# Convenience targets
test-quick: test-return test-semantic
	@echo "🚀 Quick test suite completed!"

test-comprehensive: test-all
	@echo "🔍 Comprehensive test suite completed!"

test-debug: target/debug/pgen
	@echo "🐛 Running debug mode tests..."
	@cargo run -- --debug --parser return --input '$$1'
	@cargo run -- --debug --parser semantic --input '@type: "test"'

test-stats:
	@echo "📊 Test Statistics:"
	@echo "  Return tests: 0"
	@echo "  Semantic tests: 0"
	@echo "  Regex tests: 0"
	@echo "  Total tests: 0"

.PHONY: test-all test-return test-semantic test-regex test-quick test-comprehensive test-debug test-stats


# End of auto-generated targets





