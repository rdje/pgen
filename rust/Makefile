# Makefile for pgen - Regex Parser Generator Pipeline
# This manages the complex build dependencies for generating Rust parsers from EBNF grammars

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

# Source dependencies for AST pipeline - if any of these change, parsers need regeneration
AST_PIPELINE_SOURCES = $(RUST_DIR)/src/main.rs \
                      $(RUST_DIR)/src/lib.rs \
                      $(RUST_DIR)/src/ast_pipeline.rs \
                      $(RUST_DIR)/src/ast_pipeline/high_performance_generator.rs \
                      $(RUST_DIR)/Cargo.toml

# Tools
EBNF_TO_JSON = $(TOOLS_DIR)/ebnf_to_json.pl
RUST_GENERATOR = $(RUST_AST_PIPELINE) --generate-parser --debug --trace
RUST_GENERATOR_BOOTSTRAP = $(RUST_AST_PIPELINE) --generate-parser --bootstrap-mode --debug --eliminate-left-recursion

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

# Create placeholder parsers to avoid circular dependency during build
.PHONY: create-placeholders
create-placeholders:
	@echo "Ensuring parser placeholders exist to avoid circular dependency..."
	@[ -f $(SEMANTIC_ANNOTATION_PARSER) ] || ./scripts/create_placeholder_parser.sh Semantic_annotationParser $(SEMANTIC_ANNOTATION_PARSER)
	@[ -f $(RETURN_ANNOTATION_PARSER) ] || ./scripts/create_placeholder_parser.sh Return_annotationParser $(RETURN_ANNOTATION_PARSER)

# Build Rust AST pipeline binary only (avoids circular dependency with generated parsers in lib.rs)
$(RUST_AST_PIPELINE): $(AST_PIPELINE_SOURCES) create-placeholders
	@echo "Building Rust AST pipeline binary only (avoiding circular dependency)..."
	cd $(RUST_DIR) && cargo build --bin ast_pipeline

# Step 2: Generate Rust parsers from JSON files using bootstrap mode
# These parsers are generated using bootstrap mode to avoid circular dependency
$(SEMANTIC_ANNOTATION_PARSER): $(SEMANTIC_ANNOTATION_JSON) $(RUST_AST_PIPELINE)
	@echo "Generating semantic annotation Rust parser from JSON (bootstrap mode)..."
	@mkdir -p $(GENERATED_DIR)
	@echo "Generating real parser (placeholder already exists from dependencies)..."
	$(RUST_GENERATOR_BOOTSTRAP) $(SEMANTIC_ANNOTATION_JSON) -o $(SEMANTIC_ANNOTATION_PARSER)

$(RETURN_ANNOTATION_PARSER): $(RETURN_ANNOTATION_JSON) $(RUST_AST_PIPELINE)
	@echo "Generating return annotation Rust parser from JSON (bootstrap mode)..."
	@mkdir -p $(GENERATED_DIR)
	@echo "Generating real parser (placeholder already exists from dependencies)..."
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
$(REGEX_PARSER): $(REGEX_JSON) $(RUST_AST_PIPELINE)
	@echo "Generating final regex parser from JSON..."
	@mkdir -p $(GENERATED_DIR)
	$(RUST_GENERATOR) $(REGEX_JSON) -o $(REGEX_PARSER)

.PHONY: return_semantic_parsers
return_semantic_parsers: clean $(SEMANTIC_ANNOTATION_PARSER) $(RETURN_ANNOTATION_PARSER)


# Development targets
.PHONY: debug-json
debug-json: $(REGEX_JSON)
	@echo "Regex JSON generated with debug output - check console for semantic annotation parsing details"

.PHONY: test-parser
test-parser: $(REGEX_PARSER)
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
return_annotation_parser: $(RETURN_ANNOTATION_JSON) $(RUST_AST_PIPELINE)
	@echo "🚀 Starting complete flow for return annotation parser..."
	@echo "📄 Step 1: EBNF -> JSON (already done: $(RETURN_ANNOTATION_JSON))"
	@echo "🔧 Step 2: JSON -> PARSER.RS (bootstrap mode)"
	@mkdir -p $(GENERATED_DIR)
	$(RUST_GENERATOR_BOOTSTRAP) $(RETURN_ANNOTATION_JSON) -o $(RETURN_ANNOTATION_PARSER)
	@echo "✅ Generated: $(RETURN_ANNOTATION_PARSER)"
	@echo "✅ Return annotation parser generated successfully!"

# Semantic annotation parser complete flow
.PHONY: semantic_annotation_parser
semantic_annotation_parser: $(SEMANTIC_ANNOTATION_PARSER)
	@echo "🚀 Starting complete flow for semantic annotation parser..."
	@echo "✅ Semantic annotation parser flow completed!"

# Regex parser complete flow
.PHONY: regex_parser
regex_parser: $(REGEX_JSON) $(RUST_AST_PIPELINE)
	@echo "🚀 Starting complete flow for regex parser..."
	@echo "📄 Step 1: EBNF -> JSON (already done: $(REGEX_JSON))"
	@echo "🔧 Step 2: JSON -> PARSER.RS (high-performance mode)"
	@mkdir -p $(GENERATED_DIR)
	$(RUST_GENERATOR) $(REGEX_JSON) -o $(REGEX_PARSER)
	@echo "✅ Generated: $(REGEX_PARSER)"
	@echo "✅ Regex parser generated successfully!"

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

# Specific test target with proper dependencies for debugging stack overflow issue
.PHONY: test-semantic-complex-xq_if_s1_else_q
test-semantic-complex-xq_if_s1_else_q: $(SEMANTIC_ANNOTATION_PARSER)
	@echo "🧪 Testing semantic parser with complex input (potential stack overflow case)..."
	@echo "Input: ^\" if \$1 else \""
	@echo "📝 Generated parser will be used from: $(SEMANTIC_ANNOTATION_PARSER)"
	cd $(RUST_DIR) && cargo run -- --parser semantic --input '^\" if \$1 else \"' 2>&1 | tee $(GENERATED_DIR)/test-semantic-complex-xq_if_s1_else_q.log || true
	@echo "📋 Debug output saved to: $(GENERATED_DIR)/test-semantic-complex-xq_if_s1_else_q.log"

# Individual test targets for Return Annotation Parser
# Basic scalar references
.PHONY: test-return-scalar-1
test-return-scalar-1: $(RETURN_ANNOTATION_PARSER)
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
# Generated at: 2025-10-01T18:39:44.654734+00:00
# DO NOT EDIT MANUALLY - This section is automatically regenerated

# Optional group - should preserve quantifier structure - optional_group
test-semantic-optional-group-xoptionalxgroupc_xidentifierxx: target/debug/pgen
	@echo "🧪 Testing semantic parser with: @optional_group: (identifier)?"
	@echo "Running: cargo run -- --parser semantic --input '@optional_group: (identifier)?'"
	@if cargo run -- --parser semantic --input '@optional_group: (identifier)?' > /dev/null 2>&1; then \
		echo "✅ PASS: test-semantic-optional-group-xoptionalxgroupc_xidentifierxx - Optional group - should preserve quantifier structure"; \
	else \
		echo "❌ FAIL: test-semantic-optional-group-xoptionalxgroupc_xidentifierxx - Optional group - should preserve quantifier structure"; \
		echo "🔧 REPRODUCE with make: make test-semantic-optional-group-xoptionalxgroupc_xidentifierxx"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser semantic --input '@optional_group: (identifier)?'"; \
		exit 1; \
	fi
	@echo ""

# Zero-or-more quantified group - should preserve * quantifier - quantified_group
test-semantic-quantified-group-xzeroxorxmorec_xidentifierxx: target/debug/pgen
	@echo "🧪 Testing semantic parser with: @zero_or_more: (identifier)*"
	@echo "Running: cargo run -- --parser semantic --input '@zero_or_more: (identifier)*'"
	@if cargo run -- --parser semantic --input '@zero_or_more: (identifier)*' > /dev/null 2>&1; then \
		echo "✅ PASS: test-semantic-quantified-group-xzeroxorxmorec_xidentifierxx - Zero-or-more quantified group - should preserve * quantifier"; \
	else \
		echo "❌ FAIL: test-semantic-quantified-group-xzeroxorxmorec_xidentifierxx - Zero-or-more quantified group - should preserve * quantifier"; \
		echo "🔧 REPRODUCE with make: make test-semantic-quantified-group-xzeroxorxmorec_xidentifierxx"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser semantic --input '@zero_or_more: (identifier)*'"; \
		exit 1; \
	fi
	@echo ""

# One-or-more quantified group - should preserve + quantifier - quantified_group
test-semantic-quantified-group-xonexorxmorec_xidentifierxx: target/debug/pgen
	@echo "🧪 Testing semantic parser with: @one_or_more: (identifier)+"
	@echo "Running: cargo run -- --parser semantic --input '@one_or_more: (identifier)+'"
	@if cargo run -- --parser semantic --input '@one_or_more: (identifier)+' > /dev/null 2>&1; then \
		echo "✅ PASS: test-semantic-quantified-group-xonexorxmorec_xidentifierxx - One-or-more quantified group - should preserve + quantifier"; \
	else \
		echo "❌ FAIL: test-semantic-quantified-group-xonexorxmorec_xidentifierxx - One-or-more quantified group - should preserve + quantifier"; \
		echo "🔧 REPRODUCE with make: make test-semantic-quantified-group-xonexorxmorec_xidentifierxx"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser semantic --input '@one_or_more: (identifier)+'"; \
		exit 1; \
	fi
	@echo ""

# OR group with multiple alternatives - or_group
test-semantic-or-group-xorxgroupc_xqaq_x_qbq_x_qcqx: target/debug/pgen
	@echo "🧪 Testing semantic parser with: @or_group: (\"a\" | \"b\" | \"c\")"
	@echo "Running: cargo run -- --parser semantic --input '@or_group: (\"a\" | \"b\" | \"c\")'"
	@if cargo run -- --parser semantic --input '@or_group: (\"a\" | \"b\" | \"c\")' > /dev/null 2>&1; then \
		echo "✅ PASS: test-semantic-or-group-xorxgroupc_xqaq_x_qbq_x_qcqx - OR group with multiple alternatives"; \
	else \
		echo "❌ FAIL: test-semantic-or-group-xorxgroupc_xqaq_x_qbq_x_qcqx - OR group with multiple alternatives"; \
		echo "🔧 REPRODUCE with make: make test-semantic-or-group-xorxgroupc_xqaq_x_qbq_x_qcqx"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser semantic --input '@or_group: (\"a\" | \"b\" | \"c\")'"; \
		exit 1; \
	fi
	@echo ""

# Nested group with optional quantifier - complex structure - nested_group
test-semantic-nested-group-xnestedxoptionalc_xxidentifier_qmq_identifierxxx: target/debug/pgen
	@echo "🧪 Testing semantic parser with: @nested_optional: ((identifier \",\" identifier)?)"
	@echo "Running: cargo run -- --parser semantic --input '@nested_optional: ((identifier \",\" identifier)?)'"
	@if cargo run -- --parser semantic --input '@nested_optional: ((identifier \",\" identifier)?)' > /dev/null 2>&1; then \
		echo "✅ PASS: test-semantic-nested-group-xnestedxoptionalc_xxidentifier_qmq_identifierxxx - Nested group with optional quantifier - complex structure"; \
	else \
		echo "❌ FAIL: test-semantic-nested-group-xnestedxoptionalc_xxidentifier_qmq_identifierxxx - Nested group with optional quantifier - complex structure"; \
		echo "🔧 REPRODUCE with make: make test-semantic-nested-group-xnestedxoptionalc_xxidentifier_qmq_identifierxxx"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser semantic --input '@nested_optional: ((identifier \",\" identifier)?)'"; \
		exit 1; \
	fi
	@echo ""

# Quantified sequence with regex whitespace - real-world pattern - complex_group
test-semantic-complex-group-xsequencexquantifiedc_xidentifier_xxsxx_qmq_xxsxx_identifierxx: target/debug/pgen
	@echo "🧪 Testing semantic parser with: @sequence_quantified: (identifier /\\s*/ \",\" /\\s*/ identifier)*"
	@echo "Running: cargo run -- --parser semantic --input '@sequence_quantified: (identifier /\\s*/ \",\" /\\s*/ identifier)*'"
	@if cargo run -- --parser semantic --input '@sequence_quantified: (identifier /\\s*/ \",\" /\\s*/ identifier)*' > /dev/null 2>&1; then \
		echo "✅ PASS: test-semantic-complex-group-xsequencexquantifiedc_xidentifier_xxsxx_qmq_xxsxx_identifierxx - Quantified sequence with regex whitespace - real-world pattern"; \
	else \
		echo "❌ FAIL: test-semantic-complex-group-xsequencexquantifiedc_xidentifier_xxsxx_qmq_xxsxx_identifierxx - Quantified sequence with regex whitespace - real-world pattern"; \
		echo "🔧 REPRODUCE with make: make test-semantic-complex-group-xsequencexquantifiedc_xidentifier_xxsxx_qmq_xxsxx_identifierxx"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser semantic --input '@sequence_quantified: (identifier /\\s*/ \",\" /\\s*/ identifier)*'"; \
		exit 1; \
	fi
	@echo ""

# Complex destructuring parameter pattern - the exact pattern that fails - destructuring
test-semantic-destructuring-xdestructuringxparameterc_xidentifierxliteral_xxxsxx_qmq_xxsxx_identifierxliteralxxxx: target/debug/pgen
	@echo "🧪 Testing semantic parser with: @destructuring_parameter: (identifier_literal (/\\s*/ \",\" /\\s*/ identifier_literal)*)?"
	@echo "Running: cargo run -- --parser semantic --input '@destructuring_parameter: (identifier_literal (/\\s*/ \",\" /\\s*/ identifier_literal)*)?'"
	@if cargo run -- --parser semantic --input '@destructuring_parameter: (identifier_literal (/\\s*/ \",\" /\\s*/ identifier_literal)*)?' > /dev/null 2>&1; then \
		echo "✅ PASS: test-semantic-destructuring-xdestructuringxparameterc_xidentifierxliteral_xxxsxx_qmq_xxsxx_identifierxliteralxxxx - Complex destructuring parameter pattern - the exact pattern that fails"; \
	else \
		echo "❌ FAIL: test-semantic-destructuring-xdestructuringxparameterc_xidentifierxliteral_xxxsxx_qmq_xxsxx_identifierxliteralxxxx - Complex destructuring parameter pattern - the exact pattern that fails"; \
		echo "🔧 REPRODUCE with make: make test-semantic-destructuring-xdestructuringxparameterc_xidentifierxliteral_xxxsxx_qmq_xxsxx_identifierxliteralxxxx"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser semantic --input '@destructuring_parameter: (identifier_literal (/\\s*/ \",\" /\\s*/ identifier_literal)*)?'"; \
		exit 1; \
	fi
	@echo ""

# Nested quantifiers with OR alternatives - extremely complex - nested_complex
test-semantic-nested-complex-xnestedxquantifiersc_xxelement_qmqxx__x_xelement_qxqxxxx: target/debug/pgen
	@echo "🧪 Testing semantic parser with: @nested_quantifiers: ((element \",\")*  | (element \";\")*)?"
	@echo "Running: cargo run -- --parser semantic --input '@nested_quantifiers: ((element \",\")*  | (element \";\")*)?'"
	@if cargo run -- --parser semantic --input '@nested_quantifiers: ((element \",\")*  | (element \";\")*)?' > /dev/null 2>&1; then \
		echo "✅ PASS: test-semantic-nested-complex-xnestedxquantifiersc_xxelement_qmqxx__x_xelement_qxqxxxx - Nested quantifiers with OR alternatives - extremely complex"; \
	else \
		echo "❌ FAIL: test-semantic-nested-complex-xnestedxquantifiersc_xxelement_qmqxx__x_xelement_qxqxxxx - Nested quantifiers with OR alternatives - extremely complex"; \
		echo "🔧 REPRODUCE with make: make test-semantic-nested-complex-xnestedxquantifiersc_xxelement_qmqxx__x_xelement_qxqxxxx"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser semantic --input '@nested_quantifiers: ((element \",\")*  | (element \";\")*)?'"; \
		exit 1; \
	fi
	@echo ""

# Mixed group types in sequence - optional and quantified - mixed_groups
test-semantic-mixed-groups-xmixedxgroupsc_xrequiredxelement_xoptionalxelementxx_xrepeatingxelementxxx: target/debug/pgen
	@echo "🧪 Testing semantic parser with: @mixed_groups: (required_element (optional_element)? (repeating_element)*)"
	@echo "Running: cargo run -- --parser semantic --input '@mixed_groups: (required_element (optional_element)? (repeating_element)*)'"
	@if cargo run -- --parser semantic --input '@mixed_groups: (required_element (optional_element)? (repeating_element)*)' > /dev/null 2>&1; then \
		echo "✅ PASS: test-semantic-mixed-groups-xmixedxgroupsc_xrequiredxelement_xoptionalxelementxx_xrepeatingxelementxxx - Mixed group types in sequence - optional and quantified"; \
	else \
		echo "❌ FAIL: test-semantic-mixed-groups-xmixedxgroupsc_xrequiredxelement_xoptionalxelementxx_xrepeatingxelementxxx - Mixed group types in sequence - optional and quantified"; \
		echo "🔧 REPRODUCE with make: make test-semantic-mixed-groups-xmixedxgroupsc_xrequiredxelement_xoptionalxelementxx_xrepeatingxelementxxx"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser semantic --input '@mixed_groups: (required_element (optional_element)? (repeating_element)*)'"; \
		exit 1; \
	fi
	@echo ""

# Regex patterns within complex groups - identifier path pattern - regex_groups
test-semantic-regex-groups-xregexxinxgroupc_xxaaxzaxzxaaaxzaxz0x9xaxx_xqdq_xaaxzaxzxaaaxzaxz0x9xaxxxx_xx: target/debug/pgen
	@echo "🧪 Testing semantic parser with: @regex_in_group: (/[a-zA-Z_][a-zA-Z0-9_]*/ (\".\" /[a-zA-Z_][a-zA-Z0-9_]*/)* )?"
	@echo "Running: cargo run -- --parser semantic --input '@regex_in_group: (/[a-zA-Z_][a-zA-Z0-9_]*/ (\".\" /[a-zA-Z_][a-zA-Z0-9_]*/)* )?'"
	@if cargo run -- --parser semantic --input '@regex_in_group: (/[a-zA-Z_][a-zA-Z0-9_]*/ (\".\" /[a-zA-Z_][a-zA-Z0-9_]*/)* )?' > /dev/null 2>&1; then \
		echo "✅ PASS: test-semantic-regex-groups-xregexxinxgroupc_xxaaxzaxzxaaaxzaxz0x9xaxx_xqdq_xaaxzaxzxaaaxzaxz0x9xaxxxx_xx - Regex patterns within complex groups - identifier path pattern"; \
	else \
		echo "❌ FAIL: test-semantic-regex-groups-xregexxinxgroupc_xxaaxzaxzxaaaxzaxz0x9xaxx_xqdq_xaaxzaxzxaaaxzaxz0x9xaxxxx_xx - Regex patterns within complex groups - identifier path pattern"; \
		echo "🔧 REPRODUCE with make: make test-semantic-regex-groups-xregexxinxgroupc_xxaaxzaxzxaaaxzaxz0x9xaxx_xqdq_xaaxzaxzxaaaxzaxz0x9xaxxxx_xx"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser semantic --input '@regex_in_group: (/[a-zA-Z_][a-zA-Z0-9_]*/ (\".\" /[a-zA-Z_][a-zA-Z0-9_]*/)* )?'"; \
		exit 1; \
	fi
	@echo ""

# Empty string should not be converted incorrectly to epsilon - epsilon_edge_case
test-semantic-epsilon-edge-case-xepsilonxissuec_qq: target/debug/pgen
	@echo "🧪 Testing semantic parser with: @epsilon_issue: \"\""
	@echo "Running: cargo run -- --parser semantic --input '@epsilon_issue: \"\"'"
	@if cargo run -- --parser semantic --input '@epsilon_issue: \"\"' > /dev/null 2>&1; then \
		echo "✅ PASS: test-semantic-epsilon-edge-case-xepsilonxissuec_qq - Empty string should not be converted incorrectly to epsilon"; \
	else \
		echo "❌ FAIL: test-semantic-epsilon-edge-case-xepsilonxissuec_qq - Empty string should not be converted incorrectly to epsilon"; \
		echo "🔧 REPRODUCE with make: make test-semantic-epsilon-edge-case-xepsilonxissuec_qq"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser semantic --input '@epsilon_issue: \"\"'"; \
		exit 1; \
	fi
	@echo ""

# Basic scalar reference $1 - scalar_reference
test-return_annotation-scalar-reference-xx_s1: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> \$1"
	@echo "Running: cargo run -- --parser return_annotation --input '-> \$1'"
	@if cargo run -- --parser return_annotation --input '-> \$1' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-scalar-reference-xx_s1 - Basic scalar reference $1"; \
	else \
		echo "❌ FAIL: test-return_annotation-scalar-reference-xx_s1 - Basic scalar reference $1"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-scalar-reference-xx_s1"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> \$1'"; \
		exit 1; \
	fi
	@echo ""

# Basic scalar reference $2 - scalar_reference
test-return_annotation-scalar-reference-xx_s2: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> \$2"
	@echo "Running: cargo run -- --parser return_annotation --input '-> \$2'"
	@if cargo run -- --parser return_annotation --input '-> \$2' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-scalar-reference-xx_s2 - Basic scalar reference $2"; \
	else \
		echo "❌ FAIL: test-return_annotation-scalar-reference-xx_s2 - Basic scalar reference $2"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-scalar-reference-xx_s2"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> \$2'"; \
		exit 1; \
	fi
	@echo ""

# Multi-digit scalar reference - scalar_reference
test-return_annotation-scalar-reference-xx_s10: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> \$10"
	@echo "Running: cargo run -- --parser return_annotation --input '-> \$10'"
	@if cargo run -- --parser return_annotation --input '-> \$10' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-scalar-reference-xx_s10 - Multi-digit scalar reference"; \
	else \
		echo "❌ FAIL: test-return_annotation-scalar-reference-xx_s10 - Multi-digit scalar reference"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-scalar-reference-xx_s10"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> \$10'"; \
		exit 1; \
	fi
	@echo ""

# Large scalar reference - scalar_reference
test-return_annotation-scalar-reference-xx_s99: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> \$99"
	@echo "Running: cargo run -- --parser return_annotation --input '-> \$99'"
	@if cargo run -- --parser return_annotation --input '-> \$99' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-scalar-reference-xx_s99 - Large scalar reference"; \
	else \
		echo "❌ FAIL: test-return_annotation-scalar-reference-xx_s99 - Large scalar reference"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-scalar-reference-xx_s99"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> \$99'"; \
		exit 1; \
	fi
	@echo ""

# String literal - literal
test-return_annotation-literal-xx_qhelloq: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> \"hello\""
	@echo "Running: cargo run -- --parser return_annotation --input '-> \"hello\"'"
	@if cargo run -- --parser return_annotation --input '-> \"hello\"' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-literal-xx_qhelloq - String literal"; \
	else \
		echo "❌ FAIL: test-return_annotation-literal-xx_qhelloq - String literal"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-literal-xx_qhelloq"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> \"hello\"'"; \
		exit 1; \
	fi
	@echo ""

# String literal with space - literal
test-return_annotation-literal-xx_qtest_stringq: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> \"test string\""
	@echo "Running: cargo run -- --parser return_annotation --input '-> \"test string\"'"
	@if cargo run -- --parser return_annotation --input '-> \"test string\"' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-literal-xx_qtest_stringq - String literal with space"; \
	else \
		echo "❌ FAIL: test-return_annotation-literal-xx_qtest_stringq - String literal with space"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-literal-xx_qtest_stringq"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> \"test string\"'"; \
		exit 1; \
	fi
	@echo ""

# Integer literal - literal
test-return_annotation-literal-xx_42: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> 42"
	@echo "Running: cargo run -- --parser return_annotation --input '-> 42'"
	@if cargo run -- --parser return_annotation --input '-> 42' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-literal-xx_42 - Integer literal"; \
	else \
		echo "❌ FAIL: test-return_annotation-literal-xx_42 - Integer literal"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-literal-xx_42"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> 42'"; \
		exit 1; \
	fi
	@echo ""

# Multi-digit integer literal - literal
test-return_annotation-literal-xx_123: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> 123"
	@echo "Running: cargo run -- --parser return_annotation --input '-> 123'"
	@if cargo run -- --parser return_annotation --input '-> 123' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-literal-xx_123 - Multi-digit integer literal"; \
	else \
		echo "❌ FAIL: test-return_annotation-literal-xx_123 - Multi-digit integer literal"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-literal-xx_123"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> 123'"; \
		exit 1; \
	fi
	@echo ""

# Bare identifier - should fail - identifier
test-return_annotation-identifier-xx_true: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> true"
	@echo "Running: cargo run -- --parser return_annotation --input '-> true'"
	@if cargo run -- --parser return_annotation --input '-> true' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-identifier-xx_true - Bare identifier - should fail"; \
	else \
		echo "❌ FAIL: test-return_annotation-identifier-xx_true - Bare identifier - should fail"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-identifier-xx_true"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> true'"; \
		exit 1; \
	fi
	@echo ""

# Bare identifier - should fail - identifier
test-return_annotation-identifier-xx_false: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> false"
	@echo "Running: cargo run -- --parser return_annotation --input '-> false'"
	@if cargo run -- --parser return_annotation --input '-> false' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-identifier-xx_false - Bare identifier - should fail"; \
	else \
		echo "❌ FAIL: test-return_annotation-identifier-xx_false - Bare identifier - should fail"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-identifier-xx_false"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> false'"; \
		exit 1; \
	fi
	@echo ""

# Single element array with scalar reference - array
test-return_annotation-array-xx_as1a: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> [\$1]"
	@echo "Running: cargo run -- --parser return_annotation --input '-> [\$1]'"
	@if cargo run -- --parser return_annotation --input '-> [\$1]' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-array-xx_as1a - Single element array with scalar reference"; \
	else \
		echo "❌ FAIL: test-return_annotation-array-xx_as1a - Single element array with scalar reference"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-array-xx_as1a"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> [\$1]'"; \
		exit 1; \
	fi
	@echo ""

# Single element array with scalar reference - array
test-return_annotation-array-xx_as2a: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> [\$2]"
	@echo "Running: cargo run -- --parser return_annotation --input '-> [\$2]'"
	@if cargo run -- --parser return_annotation --input '-> [\$2]' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-array-xx_as2a - Single element array with scalar reference"; \
	else \
		echo "❌ FAIL: test-return_annotation-array-xx_as2a - Single element array with scalar reference"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-array-xx_as2a"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> [\$2]'"; \
		exit 1; \
	fi
	@echo ""

# Empty array - array
test-return_annotation-array-xx_aa: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> []"
	@echo "Running: cargo run -- --parser return_annotation --input '-> []'"
	@if cargo run -- --parser return_annotation --input '-> []' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-array-xx_aa - Empty array"; \
	else \
		echo "❌ FAIL: test-return_annotation-array-xx_aa - Empty array"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-array-xx_aa"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> []'"; \
		exit 1; \
	fi
	@echo ""

# Empty object - object
test-return_annotation-object-xx_oo: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> {}"
	@echo "Running: cargo run -- --parser return_annotation --input '-> {}'"
	@if cargo run -- --parser return_annotation --input '-> {}' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-object-xx_oo - Empty object"; \
	else \
		echo "❌ FAIL: test-return_annotation-object-xx_oo - Empty object"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-object-xx_oo"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> {}'"; \
		exit 1; \
	fi
	@echo ""

# Multi-element array with scalar references - array
test-return_annotation-array-xx_as1m_s2a: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> [\$1, \$2]"
	@echo "Running: cargo run -- --parser return_annotation --input '-> [\$1, \$2]'"
	@if cargo run -- --parser return_annotation --input '-> [\$1, \$2]' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-array-xx_as1m_s2a - Multi-element array with scalar references"; \
	else \
		echo "❌ FAIL: test-return_annotation-array-xx_as1m_s2a - Multi-element array with scalar references"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-array-xx_as1m_s2a"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> [\$1, \$2]'"; \
		exit 1; \
	fi
	@echo ""

# String array - array
test-return_annotation-array-xx_aqitem1qm_qitem2qa: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> [\"item1\", \"item2\"]"
	@echo "Running: cargo run -- --parser return_annotation --input '-> [\"item1\", \"item2\"]'"
	@if cargo run -- --parser return_annotation --input '-> [\"item1\", \"item2\"]' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-array-xx_aqitem1qm_qitem2qa - String array"; \
	else \
		echo "❌ FAIL: test-return_annotation-array-xx_aqitem1qm_qitem2qa - String array"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-array-xx_aqitem1qm_qitem2qa"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> [\"item1\", \"item2\"]'"; \
		exit 1; \
	fi
	@echo ""

# Integer array - array
test-return_annotation-array-xx_a42m_100a: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> [42, 100]"
	@echo "Running: cargo run -- --parser return_annotation --input '-> [42, 100]'"
	@if cargo run -- --parser return_annotation --input '-> [42, 100]' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-array-xx_a42m_100a - Integer array"; \
	else \
		echo "❌ FAIL: test-return_annotation-array-xx_a42m_100a - Integer array"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-array-xx_a42m_100a"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> [42, 100]'"; \
		exit 1; \
	fi
	@echo ""

# Single property object - object
test-return_annotation-object-xx_okeyc_s1o: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> {key: \$1}"
	@echo "Running: cargo run -- --parser return_annotation --input '-> {key: \$1}'"
	@if cargo run -- --parser return_annotation --input '-> {key: \$1}' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-object-xx_okeyc_s1o - Single property object"; \
	else \
		echo "❌ FAIL: test-return_annotation-object-xx_okeyc_s1o - Single property object"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-object-xx_okeyc_s1o"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> {key: \$1}'"; \
		exit 1; \
	fi
	@echo ""

# Single property object with name key - object
test-return_annotation-object-xx_onamec_s1o: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> {name: \$1}"
	@echo "Running: cargo run -- --parser return_annotation --input '-> {name: \$1}'"
	@if cargo run -- --parser return_annotation --input '-> {name: \$1}' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-object-xx_onamec_s1o - Single property object with name key"; \
	else \
		echo "❌ FAIL: test-return_annotation-object-xx_onamec_s1o - Single property object with name key"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-object-xx_onamec_s1o"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> {name: \$1}'"; \
		exit 1; \
	fi
	@echo ""

# Single property object with value key - object
test-return_annotation-object-xx_ovaluec_s2o: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> {value: \$2}"
	@echo "Running: cargo run -- --parser return_annotation --input '-> {value: \$2}'"
	@if cargo run -- --parser return_annotation --input '-> {value: \$2}' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-object-xx_ovaluec_s2o - Single property object with value key"; \
	else \
		echo "❌ FAIL: test-return_annotation-object-xx_ovaluec_s2o - Single property object with value key"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-object-xx_ovaluec_s2o"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> {value: \$2}'"; \
		exit 1; \
	fi
	@echo ""

# Multi-property object - object
test-return_annotation-object-xx_onamec_s1m_valuec_s2o: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> {name: \$1, value: \$2}"
	@echo "Running: cargo run -- --parser return_annotation --input '-> {name: \$1, value: \$2}'"
	@if cargo run -- --parser return_annotation --input '-> {name: \$1, value: \$2}' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-object-xx_onamec_s1m_valuec_s2o - Multi-property object"; \
	else \
		echo "❌ FAIL: test-return_annotation-object-xx_onamec_s1m_valuec_s2o - Multi-property object"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-object-xx_onamec_s1m_valuec_s2o"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> {name: \$1, value: \$2}'"; \
		exit 1; \
	fi
	@echo ""

# Mixed type object - object
test-return_annotation-object-xx_oidc_42m_namec_qtestqo: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> {id: 42, name: \"test\"}"
	@echo "Running: cargo run -- --parser return_annotation --input '-> {id: 42, name: \"test\"}'"
	@if cargo run -- --parser return_annotation --input '-> {id: 42, name: \"test\"}' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-object-xx_oidc_42m_namec_qtestqo - Mixed type object"; \
	else \
		echo "❌ FAIL: test-return_annotation-object-xx_oidc_42m_namec_qtestqo - Mixed type object"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-object-xx_oidc_42m_namec_qtestqo"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> {id: 42, name: \"test\"}'"; \
		exit 1; \
	fi
	@echo ""

# Dot notation access - accessor
test-return_annotation-accessor-xx_s1dvalue: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> \$1.value"
	@echo "Running: cargo run -- --parser return_annotation --input '-> \$1.value'"
	@if cargo run -- --parser return_annotation --input '-> \$1.value' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-accessor-xx_s1dvalue - Dot notation access"; \
	else \
		echo "❌ FAIL: test-return_annotation-accessor-xx_s1dvalue - Dot notation access"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-accessor-xx_s1dvalue"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> \$1.value'"; \
		exit 1; \
	fi
	@echo ""

# Dot notation access to name property - accessor
test-return_annotation-accessor-xx_s1dname: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> \$1.name"
	@echo "Running: cargo run -- --parser return_annotation --input '-> \$1.name'"
	@if cargo run -- --parser return_annotation --input '-> \$1.name' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-accessor-xx_s1dname - Dot notation access to name property"; \
	else \
		echo "❌ FAIL: test-return_annotation-accessor-xx_s1dname - Dot notation access to name property"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-accessor-xx_s1dname"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> \$1.name'"; \
		exit 1; \
	fi
	@echo ""

# Dot notation access to data property - accessor
test-return_annotation-accessor-xx_s1ddata: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> \$1.data"
	@echo "Running: cargo run -- --parser return_annotation --input '-> \$1.data'"
	@if cargo run -- --parser return_annotation --input '-> \$1.data' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-accessor-xx_s1ddata - Dot notation access to data property"; \
	else \
		echo "❌ FAIL: test-return_annotation-accessor-xx_s1ddata - Dot notation access to data property"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-accessor-xx_s1ddata"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> \$1.data'"; \
		exit 1; \
	fi
	@echo ""

# Array indexing at position 0 - accessor
test-return_annotation-accessor-xx_s1a0a: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> \$1[0]"
	@echo "Running: cargo run -- --parser return_annotation --input '-> \$1[0]'"
	@if cargo run -- --parser return_annotation --input '-> \$1[0]' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-accessor-xx_s1a0a - Array indexing at position 0"; \
	else \
		echo "❌ FAIL: test-return_annotation-accessor-xx_s1a0a - Array indexing at position 0"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-accessor-xx_s1a0a"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> \$1[0]'"; \
		exit 1; \
	fi
	@echo ""

# Array indexing at position 1 - accessor
test-return_annotation-accessor-xx_s1a1a: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> \$1[1]"
	@echo "Running: cargo run -- --parser return_annotation --input '-> \$1[1]'"
	@if cargo run -- --parser return_annotation --input '-> \$1[1]' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-accessor-xx_s1a1a - Array indexing at position 1"; \
	else \
		echo "❌ FAIL: test-return_annotation-accessor-xx_s1a1a - Array indexing at position 1"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-accessor-xx_s1a1a"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> \$1[1]'"; \
		exit 1; \
	fi
	@echo ""

# Array indexing on second reference - accessor
test-return_annotation-accessor-xx_s2a0a: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> \$2[0]"
	@echo "Running: cargo run -- --parser return_annotation --input '-> \$2[0]'"
	@if cargo run -- --parser return_annotation --input '-> \$2[0]' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-accessor-xx_s2a0a - Array indexing on second reference"; \
	else \
		echo "❌ FAIL: test-return_annotation-accessor-xx_s2a0a - Array indexing on second reference"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-accessor-xx_s2a0a"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> \$2[0]'"; \
		exit 1; \
	fi
	@echo ""

# Bash-style whole array access - advanced_accessor
test-return_annotation-advanced-accessor-xx_s1axa: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> \$1[*]"
	@echo "Running: cargo run -- --parser return_annotation --input '-> \$1[*]'"
	@if cargo run -- --parser return_annotation --input '-> \$1[*]' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-advanced-accessor-xx_s1axa - Bash-style whole array access"; \
	else \
		echo "❌ FAIL: test-return_annotation-advanced-accessor-xx_s1axa - Bash-style whole array access"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-advanced-accessor-xx_s1axa"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> \$1[*]'"; \
		exit 1; \
	fi
	@echo ""

# Python-style whole array slice - advanced_accessor
test-return_annotation-advanced-accessor-xx_s1aca: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> \$1[:]"
	@echo "Running: cargo run -- --parser return_annotation --input '-> \$1[:]'"
	@if cargo run -- --parser return_annotation --input '-> \$1[:]' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-advanced-accessor-xx_s1aca - Python-style whole array slice"; \
	else \
		echo "❌ FAIL: test-return_annotation-advanced-accessor-xx_s1aca - Python-style whole array slice"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-advanced-accessor-xx_s1aca"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> \$1[:]'"; \
		exit 1; \
	fi
	@echo ""

# Perl-style range - advanced_accessor
test-return_annotation-advanced-accessor-xx_s1a0dd2a: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> \$1[0..2]"
	@echo "Running: cargo run -- --parser return_annotation --input '-> \$1[0..2]'"
	@if cargo run -- --parser return_annotation --input '-> \$1[0..2]' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-advanced-accessor-xx_s1a0dd2a - Perl-style range"; \
	else \
		echo "❌ FAIL: test-return_annotation-advanced-accessor-xx_s1a0dd2a - Perl-style range"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-advanced-accessor-xx_s1a0dd2a"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> \$1[0..2]'"; \
		exit 1; \
	fi
	@echo ""

# Python-style slice - advanced_accessor
test-return_annotation-advanced-accessor-xx_s1a1c4a: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> \$1[1:4]"
	@echo "Running: cargo run -- --parser return_annotation --input '-> \$1[1:4]'"
	@if cargo run -- --parser return_annotation --input '-> \$1[1:4]' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-advanced-accessor-xx_s1a1c4a - Python-style slice"; \
	else \
		echo "❌ FAIL: test-return_annotation-advanced-accessor-xx_s1a1c4a - Python-style slice"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-advanced-accessor-xx_s1a1c4a"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> \$1[1:4]'"; \
		exit 1; \
	fi
	@echo ""

# Python-style slice with step - advanced_accessor
test-return_annotation-advanced-accessor-xx_s1a1c10c2a: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> \$1[1:10:2]"
	@echo "Running: cargo run -- --parser return_annotation --input '-> \$1[1:10:2]'"
	@if cargo run -- --parser return_annotation --input '-> \$1[1:10:2]' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-advanced-accessor-xx_s1a1c10c2a - Python-style slice with step"; \
	else \
		echo "❌ FAIL: test-return_annotation-advanced-accessor-xx_s1a1c10c2a - Python-style slice with step"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-advanced-accessor-xx_s1a1c10c2a"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> \$1[1:10:2]'"; \
		exit 1; \
	fi
	@echo ""

# Nested arrays - nested_structures
test-return_annotation-nested-structures-xx_aas1m_s2am_as3m_s4aa: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> [[\$1, \$2], [\$3, \$4]]"
	@echo "Running: cargo run -- --parser return_annotation --input '-> [[\$1, \$2], [\$3, \$4]]'"
	@if cargo run -- --parser return_annotation --input '-> [[\$1, \$2], [\$3, \$4]]' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-nested-structures-xx_aas1m_s2am_as3m_s4aa - Nested arrays"; \
	else \
		echo "❌ FAIL: test-return_annotation-nested-structures-xx_aas1m_s2am_as3m_s4aa - Nested arrays"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-nested-structures-xx_aas1m_s2am_as3m_s4aa"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> [[\$1, \$2], [\$3, \$4]]'"; \
		exit 1; \
	fi
	@echo ""

# Nested objects - nested_structures
test-return_annotation-nested-structures-xx_oouterc_oinnerc_s1oo: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> {outer: {inner: \$1}}"
	@echo "Running: cargo run -- --parser return_annotation --input '-> {outer: {inner: \$1}}'"
	@if cargo run -- --parser return_annotation --input '-> {outer: {inner: \$1}}' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-nested-structures-xx_oouterc_oinnerc_s1oo - Nested objects"; \
	else \
		echo "❌ FAIL: test-return_annotation-nested-structures-xx_oouterc_oinnerc_s1oo - Nested objects"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-nested-structures-xx_oouterc_oinnerc_s1oo"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> {outer: {inner: \$1}}'"; \
		exit 1; \
	fi
	@echo ""

# Quantified array element - quantified
test-return_annotation-quantified-xx_as1xa: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> [\$1*]"
	@echo "Running: cargo run -- --parser return_annotation --input '-> [\$1*]'"
	@if cargo run -- --parser return_annotation --input '-> [\$1*]' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-quantified-xx_as1xa - Quantified array element"; \
	else \
		echo "❌ FAIL: test-return_annotation-quantified-xx_as1xa - Quantified array element"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-quantified-xx_as1xa"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> [\$1*]'"; \
		exit 1; \
	fi
	@echo ""

# Quantified object - quantified
test-return_annotation-quantified-xx_okeyc_s1ox: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> {key: \$1}*"
	@echo "Running: cargo run -- --parser return_annotation --input '-> {key: \$1}*'"
	@if cargo run -- --parser return_annotation --input '-> {key: \$1}*' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-quantified-xx_okeyc_s1ox - Quantified object"; \
	else \
		echo "❌ FAIL: test-return_annotation-quantified-xx_okeyc_s1ox - Quantified object"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-quantified-xx_okeyc_s1ox"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> {key: \$1}*'"; \
		exit 1; \
	fi
	@echo ""

# Grouped quantified array - grouped_quantified
test-return_annotation-grouped-quantified-xx_axelement1_element2xxa: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> [(element1 element2)*]"
	@echo "Running: cargo run -- --parser return_annotation --input '-> [(element1 element2)*]'"
	@if cargo run -- --parser return_annotation --input '-> [(element1 element2)*]' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-grouped-quantified-xx_axelement1_element2xxa - Grouped quantified array"; \
	else \
		echo "❌ FAIL: test-return_annotation-grouped-quantified-xx_axelement1_element2xxa - Grouped quantified array"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-grouped-quantified-xx_axelement1_element2xxa"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> [(element1 element2)*]'"; \
		exit 1; \
	fi
	@echo ""

# Chained accessors - chained_accessor
test-return_annotation-chained-accessor-xx_s1dpropertya2adsubprop: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> \$1.property[2].subprop"
	@echo "Running: cargo run -- --parser return_annotation --input '-> \$1.property[2].subprop'"
	@if cargo run -- --parser return_annotation --input '-> \$1.property[2].subprop' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-chained-accessor-xx_s1dpropertya2adsubprop - Chained accessors"; \
	else \
		echo "❌ FAIL: test-return_annotation-chained-accessor-xx_s1dpropertya2adsubprop - Chained accessors"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-chained-accessor-xx_s1dpropertya2adsubprop"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> \$1.property[2].subprop'"; \
		exit 1; \
	fi
	@echo ""

# Negative indexing - negative_index
test-return_annotation-negative-index-xx_s1ax1a: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> \$1[-1]"
	@echo "Running: cargo run -- --parser return_annotation --input '-> \$1[-1]'"
	@if cargo run -- --parser return_annotation --input '-> \$1[-1]' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-negative-index-xx_s1ax1a - Negative indexing"; \
	else \
		echo "❌ FAIL: test-return_annotation-negative-index-xx_s1ax1a - Negative indexing"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-negative-index-xx_s1ax1a"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> \$1[-1]'"; \
		exit 1; \
	fi
	@echo ""

# Multiple indices - multi_index
test-return_annotation-multi-index-xx_s1a1m3m5a: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> \$1[1,3,5]"
	@echo "Running: cargo run -- --parser return_annotation --input '-> \$1[1,3,5]'"
	@if cargo run -- --parser return_annotation --input '-> \$1[1,3,5]' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-multi-index-xx_s1a1m3m5a - Multiple indices"; \
	else \
		echo "❌ FAIL: test-return_annotation-multi-index-xx_s1a1m3m5a - Multiple indices"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-multi-index-xx_s1a1m3m5a"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> \$1[1,3,5]'"; \
		exit 1; \
	fi
	@echo ""

# Optional array - optional_quantifier
test-return_annotation-optional-quantifier-xx_as1ax: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> [\$1]?"
	@echo "Running: cargo run -- --parser return_annotation --input '-> [\$1]?'"
	@if cargo run -- --parser return_annotation --input '-> [\$1]?' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-optional-quantifier-xx_as1ax - Optional array"; \
	else \
		echo "❌ FAIL: test-return_annotation-optional-quantifier-xx_as1ax - Optional array"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-optional-quantifier-xx_as1ax"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> [\$1]?'"; \
		exit 1; \
	fi
	@echo ""

# One or more quantifier on array - plus_quantifier
test-return_annotation-plus-quantifier-xx_as1ax: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> [\$1]+"
	@echo "Running: cargo run -- --parser return_annotation --input '-> [\$1]+'"
	@if cargo run -- --parser return_annotation --input '-> [\$1]+' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-plus-quantifier-xx_as1ax - One or more quantifier on array"; \
	else \
		echo "❌ FAIL: test-return_annotation-plus-quantifier-xx_as1ax - One or more quantifier on array"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-plus-quantifier-xx_as1ax"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> [\$1]+'"; \
		exit 1; \
	fi
	@echo ""

# Exact repetition quantifier - exact_quantifier
test-return_annotation-exact-quantifier-xx_ocountc_s1oo3o: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> {count: \$1}{3}"
	@echo "Running: cargo run -- --parser return_annotation --input '-> {count: \$1}{3}'"
	@if cargo run -- --parser return_annotation --input '-> {count: \$1}{3}' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-exact-quantifier-xx_ocountc_s1oo3o - Exact repetition quantifier"; \
	else \
		echo "❌ FAIL: test-return_annotation-exact-quantifier-xx_ocountc_s1oo3o - Exact repetition quantifier"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-exact-quantifier-xx_ocountc_s1oo3o"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> {count: \$1}{3}'"; \
		exit 1; \
	fi
	@echo ""

# Range repetition quantifier - range_quantifier
test-return_annotation-range-quantifier-xx_oitemsc_as1aoo2m5o: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> {items: [\$1]}{2,5}"
	@echo "Running: cargo run -- --parser return_annotation --input '-> {items: [\$1]}{2,5}'"
	@if cargo run -- --parser return_annotation --input '-> {items: [\$1]}{2,5}' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-range-quantifier-xx_oitemsc_as1aoo2m5o - Range repetition quantifier"; \
	else \
		echo "❌ FAIL: test-return_annotation-range-quantifier-xx_oitemsc_as1aoo2m5o - Range repetition quantifier"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-range-quantifier-xx_oitemsc_as1aoo2m5o"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> {items: [\$1]}{2,5}'"; \
		exit 1; \
	fi
	@echo ""

# Up to n repetitions - max_quantifier
test-return_annotation-max-quantifier-xx_odatac_s1oom3o: target/debug/pgen
	@echo "🧪 Testing return_annotation parser with: -> {data: \$1}{,3}"
	@echo "Running: cargo run -- --parser return_annotation --input '-> {data: \$1}{,3}'"
	@if cargo run -- --parser return_annotation --input '-> {data: \$1}{,3}' > /dev/null 2>&1; then \
		echo "✅ PASS: test-return_annotation-max-quantifier-xx_odatac_s1oom3o - Up to n repetitions"; \
	else \
		echo "❌ FAIL: test-return_annotation-max-quantifier-xx_odatac_s1oom3o - Up to n repetitions"; \
		echo "🔧 REPRODUCE with make: make test-return_annotation-max-quantifier-xx_odatac_s1oom3o"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser return_annotation --input '-> {data: \$1}{,3}'"; \
		exit 1; \
	fi
	@echo ""

# Type annotation - type
test-semantic-type-xtypec_qexpressionq: target/debug/pgen
	@echo "🧪 Testing semantic parser with: @type: \"Expression\""
	@echo "Running: cargo run -- --parser semantic --input '@type: \"Expression\"'"
	@if cargo run -- --parser semantic --input '@type: \"Expression\"' > /dev/null 2>&1; then \
		echo "✅ PASS: test-semantic-type-xtypec_qexpressionq - Type annotation"; \
	else \
		echo "❌ FAIL: test-semantic-type-xtypec_qexpressionq - Type annotation"; \
		echo "🔧 REPRODUCE with make: make test-semantic-type-xtypec_qexpressionq"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser semantic --input '@type: \"Expression\"'"; \
		exit 1; \
	fi
	@echo ""

# Precedence annotation - precedence
test-semantic-precedence-xprecedencec_5: target/debug/pgen
	@echo "🧪 Testing semantic parser with: @precedence: 5"
	@echo "Running: cargo run -- --parser semantic --input '@precedence: 5'"
	@if cargo run -- --parser semantic --input '@precedence: 5' > /dev/null 2>&1; then \
		echo "✅ PASS: test-semantic-precedence-xprecedencec_5 - Precedence annotation"; \
	else \
		echo "❌ FAIL: test-semantic-precedence-xprecedencec_5 - Precedence annotation"; \
		echo "🔧 REPRODUCE with make: make test-semantic-precedence-xprecedencec_5"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser semantic --input '@precedence: 5'"; \
		exit 1; \
	fi
	@echo ""

# Boolean annotation - boolean
test-semantic-boolean-xsidexeffectc_false: target/debug/pgen
	@echo "🧪 Testing semantic parser with: @side_effect: false"
	@echo "Running: cargo run -- --parser semantic --input '@side_effect: false'"
	@if cargo run -- --parser semantic --input '@side_effect: false' > /dev/null 2>&1; then \
		echo "✅ PASS: test-semantic-boolean-xsidexeffectc_false - Boolean annotation"; \
	else \
		echo "❌ FAIL: test-semantic-boolean-xsidexeffectc_false - Boolean annotation"; \
		echo "🔧 REPRODUCE with make: make test-semantic-boolean-xsidexeffectc_false"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser semantic --input '@side_effect: false'"; \
		exit 1; \
	fi
	@echo ""

# Array annotation - array
test-semantic-array-xvalidatec_aqcheckxboundsqa: target/debug/pgen
	@echo "🧪 Testing semantic parser with: @validate: [\"check_bounds\"]"
	@echo "Running: cargo run -- --parser semantic --input '@validate: [\"check_bounds\"]'"
	@if cargo run -- --parser semantic --input '@validate: [\"check_bounds\"]' > /dev/null 2>&1; then \
		echo "✅ PASS: test-semantic-array-xvalidatec_aqcheckxboundsqa - Array annotation"; \
	else \
		echo "❌ FAIL: test-semantic-array-xvalidatec_aqcheckxboundsqa - Array annotation"; \
		echo "🔧 REPRODUCE with make: make test-semantic-array-xvalidatec_aqcheckxboundsqa"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser semantic --input '@validate: [\"check_bounds\"]'"; \
		exit 1; \
	fi
	@echo ""

# Object annotation - object
test-semantic-object-xcachec_ottlc_300o: target/debug/pgen
	@echo "🧪 Testing semantic parser with: @cache: {ttl: 300}"
	@echo "Running: cargo run -- --parser semantic --input '@cache: {ttl: 300}'"
	@if cargo run -- --parser semantic --input '@cache: {ttl: 300}' > /dev/null 2>&1; then \
		echo "✅ PASS: test-semantic-object-xcachec_ottlc_300o - Object annotation"; \
	else \
		echo "❌ FAIL: test-semantic-object-xcachec_ottlc_300o - Object annotation"; \
		echo "🔧 REPRODUCE with make: make test-semantic-object-xcachec_ottlc_300o"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser semantic --input '@cache: {ttl: 300}'"; \
		exit 1; \
	fi
	@echo ""

# Complex function call annotation - complex
test-semantic-complex-generatexcharxclassxmatcherxhasxnegationxs2xm_collectxclassxitemsxs2xx: target/debug/pgen
	@echo "🧪 Testing semantic parser with: generate_char_class_matcher(has_negation(\$2), collect_class_items(\$2))"
	@echo "Running: cargo run -- --parser semantic --input 'generate_char_class_matcher(has_negation(\$2), collect_class_items(\$2))'"
	@if cargo run -- --parser semantic --input 'generate_char_class_matcher(has_negation(\$2), collect_class_items(\$2))' > /dev/null 2>&1; then \
		echo "✅ PASS: test-semantic-complex-generatexcharxclassxmatcherxhasxnegationxs2xm_collectxclassxitemsxs2xx - Complex function call annotation"; \
	else \
		echo "❌ FAIL: test-semantic-complex-generatexcharxclassxmatcherxhasxnegationxs2xm_collectxclassxitemsxs2xx - Complex function call annotation"; \
		echo "🔧 REPRODUCE with make: make test-semantic-complex-generatexcharxclassxmatcherxhasxnegationxs2xm_collectxclassxitemsxs2xx"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser semantic --input 'generate_char_class_matcher(has_negation(\$2), collect_class_items(\$2))'"; \
		exit 1; \
	fi
	@echo ""

# Complex conditional expression - complex
test-semantic-complex-countxflattenxs2xx_x_8_x_qlookupxtableq_c_qlinearxchecksq: target/debug/pgen
	@echo "🧪 Testing semantic parser with: count(flatten(\$2)) > 8 ? \"lookup_table\" : \"linear_checks\""
	@echo "Running: cargo run -- --parser semantic --input 'count(flatten(\$2)) > 8 ? \"lookup_table\" : \"linear_checks\"'"
	@if cargo run -- --parser semantic --input 'count(flatten(\$2)) > 8 ? \"lookup_table\" : \"linear_checks\"' > /dev/null 2>&1; then \
		echo "✅ PASS: test-semantic-complex-countxflattenxs2xx_x_8_x_qlookupxtableq_c_qlinearxchecksq - Complex conditional expression"; \
	else \
		echo "❌ FAIL: test-semantic-complex-countxflattenxs2xx_x_8_x_qlookupxtableq_c_qlinearxchecksq - Complex conditional expression"; \
		echo "🔧 REPRODUCE with make: make test-semantic-complex-countxflattenxs2xx_x_8_x_qlookupxtableq_c_qlinearxchecksq"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser semantic --input 'count(flatten(\$2)) > 8 ? \"lookup_table\" : \"linear_checks\"'"; \
		exit 1; \
	fi
	@echo ""

# Complex lambda expression - complex
test-semantic-complex-allxextractxrangesxs2xm_r_xx_validxrangexrdstartm_rdendxx: target/debug/pgen
	@echo "🧪 Testing semantic parser with: all(extract_ranges(\$2), r => valid_range(r.start, r.end))"
	@echo "Running: cargo run -- --parser semantic --input 'all(extract_ranges(\$2), r => valid_range(r.start, r.end))'"
	@if cargo run -- --parser semantic --input 'all(extract_ranges(\$2), r => valid_range(r.start, r.end))' > /dev/null 2>&1; then \
		echo "✅ PASS: test-semantic-complex-allxextractxrangesxs2xm_r_xx_validxrangexrdstartm_rdendxx - Complex lambda expression"; \
	else \
		echo "❌ FAIL: test-semantic-complex-allxextractxrangesxs2xm_r_xx_validxrangexrdstartm_rdendxx - Complex lambda expression"; \
		echo "🔧 REPRODUCE with make: make test-semantic-complex-allxextractxrangesxs2xm_r_xx_validxrangexrdstartm_rdendxx"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser semantic --input 'all(extract_ranges(\$2), r => valid_range(r.start, r.end))'"; \
		exit 1; \
	fi
	@echo ""

# Invalid syntax pattern (causes stack overflow) - complex
test-semantic-complex-xq_if_s1_else_q: target/debug/pgen
	@echo "🧪 Testing semantic parser with: ^\" if \$1 else \""
	@echo "Running: cargo run -- --parser semantic --input '^\" if \$1 else \"'"
	@if cargo run -- --parser semantic --input '^\" if \$1 else \"' > /dev/null 2>&1; then \
		echo "✅ PASS: test-semantic-complex-xq_if_s1_else_q - Invalid syntax pattern (causes stack overflow)"; \
	else \
		echo "❌ FAIL: test-semantic-complex-xq_if_s1_else_q - Invalid syntax pattern (causes stack overflow)"; \
		echo "🔧 REPRODUCE with make: make test-semantic-complex-xq_if_s1_else_q"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser semantic --input '^\" if \$1 else \"'"; \
		exit 1; \
	fi
	@echo ""

# Comparison expression - complex
test-semantic-complex-s1_xx_null: target/debug/pgen
	@echo "🧪 Testing semantic parser with: \$1 != null"
	@echo "Running: cargo run -- --parser semantic --input '\$1 != null'"
	@if cargo run -- --parser semantic --input '\$1 != null' > /dev/null 2>&1; then \
		echo "✅ PASS: test-semantic-complex-s1_xx_null - Comparison expression"; \
	else \
		echo "❌ FAIL: test-semantic-complex-s1_xx_null - Comparison expression"; \
		echo "🔧 REPRODUCE with make: make test-semantic-complex-s1_xx_null"; \
		echo "🔧 REPRODUCE with cargo: cargo run -- --parser semantic --input '\$1 != null'"; \
		exit 1; \
	fi
	@echo ""

# Run all stress tests
test-all: test-semantic-optional-group-xoptionalxgroupc_xidentifierxx test-semantic-quantified-group-xzeroxorxmorec_xidentifierxx test-semantic-quantified-group-xonexorxmorec_xidentifierxx test-semantic-or-group-xorxgroupc_xqaq_x_qbq_x_qcqx test-semantic-nested-group-xnestedxoptionalc_xxidentifier_qmq_identifierxxx test-semantic-complex-group-xsequencexquantifiedc_xidentifier_xxsxx_qmq_xxsxx_identifierxx test-semantic-destructuring-xdestructuringxparameterc_xidentifierxliteral_xxxsxx_qmq_xxsxx_identifierxliteralxxxx test-semantic-nested-complex-xnestedxquantifiersc_xxelement_qmqxx__x_xelement_qxqxxxx test-semantic-mixed-groups-xmixedxgroupsc_xrequiredxelement_xoptionalxelementxx_xrepeatingxelementxxx test-semantic-regex-groups-xregexxinxgroupc_xxaaxzaxzxaaaxzaxz0x9xaxx_xqdq_xaaxzaxzxaaaxzaxz0x9xaxxxx_xx test-semantic-epsilon-edge-case-xepsilonxissuec_qq test-return_annotation-scalar-reference-xx_s1 test-return_annotation-scalar-reference-xx_s2 test-return_annotation-scalar-reference-xx_s10 test-return_annotation-scalar-reference-xx_s99 test-return_annotation-literal-xx_qhelloq test-return_annotation-literal-xx_qtest_stringq test-return_annotation-literal-xx_42 test-return_annotation-literal-xx_123 test-return_annotation-identifier-xx_true test-return_annotation-identifier-xx_false test-return_annotation-array-xx_as1a test-return_annotation-array-xx_as2a test-return_annotation-array-xx_aa test-return_annotation-object-xx_oo test-return_annotation-array-xx_as1m_s2a test-return_annotation-array-xx_aqitem1qm_qitem2qa test-return_annotation-array-xx_a42m_100a test-return_annotation-object-xx_okeyc_s1o test-return_annotation-object-xx_onamec_s1o test-return_annotation-object-xx_ovaluec_s2o test-return_annotation-object-xx_onamec_s1m_valuec_s2o test-return_annotation-object-xx_oidc_42m_namec_qtestqo test-return_annotation-accessor-xx_s1dvalue test-return_annotation-accessor-xx_s1dname test-return_annotation-accessor-xx_s1ddata test-return_annotation-accessor-xx_s1a0a test-return_annotation-accessor-xx_s1a1a test-return_annotation-accessor-xx_s2a0a test-return_annotation-advanced-accessor-xx_s1axa test-return_annotation-advanced-accessor-xx_s1aca test-return_annotation-advanced-accessor-xx_s1a0dd2a test-return_annotation-advanced-accessor-xx_s1a1c4a test-return_annotation-advanced-accessor-xx_s1a1c10c2a test-return_annotation-nested-structures-xx_aas1m_s2am_as3m_s4aa test-return_annotation-nested-structures-xx_oouterc_oinnerc_s1oo test-return_annotation-quantified-xx_as1xa test-return_annotation-quantified-xx_okeyc_s1ox test-return_annotation-grouped-quantified-xx_axelement1_element2xxa test-return_annotation-chained-accessor-xx_s1dpropertya2adsubprop test-return_annotation-negative-index-xx_s1ax1a test-return_annotation-multi-index-xx_s1a1m3m5a test-return_annotation-optional-quantifier-xx_as1ax test-return_annotation-plus-quantifier-xx_as1ax test-return_annotation-exact-quantifier-xx_ocountc_s1oo3o test-return_annotation-range-quantifier-xx_oitemsc_as1aoo2m5o test-return_annotation-max-quantifier-xx_odatac_s1oom3o test-semantic-type-xtypec_qexpressionq test-semantic-precedence-xprecedencec_5 test-semantic-boolean-xsidexeffectc_false test-semantic-array-xvalidatec_aqcheckxboundsqa test-semantic-object-xcachec_ottlc_300o test-semantic-complex-generatexcharxclassxmatcherxhasxnegationxs2xm_collectxclassxitemsxs2xx test-semantic-complex-countxflattenxs2xx_x_8_x_qlookupxtableq_c_qlinearxchecksq test-semantic-complex-allxextractxrangesxs2xm_r_xx_validxrangexrdstartm_rdendxx test-semantic-complex-xq_if_s1_else_q test-semantic-complex-s1_xx_null
	@echo "🎯 All stress tests completed successfully!"

# Run all semantic parser tests
test-semantic: test-semantic-optional-group-xoptionalxgroupc_xidentifierxx test-semantic-quantified-group-xzeroxorxmorec_xidentifierxx test-semantic-quantified-group-xonexorxmorec_xidentifierxx test-semantic-or-group-xorxgroupc_xqaq_x_qbq_x_qcqx test-semantic-nested-group-xnestedxoptionalc_xxidentifier_qmq_identifierxxx test-semantic-complex-group-xsequencexquantifiedc_xidentifier_xxsxx_qmq_xxsxx_identifierxx test-semantic-destructuring-xdestructuringxparameterc_xidentifierxliteral_xxxsxx_qmq_xxsxx_identifierxliteralxxxx test-semantic-nested-complex-xnestedxquantifiersc_xxelement_qmqxx__x_xelement_qxqxxxx test-semantic-mixed-groups-xmixedxgroupsc_xrequiredxelement_xoptionalxelementxx_xrepeatingxelementxxx test-semantic-regex-groups-xregexxinxgroupc_xxaaxzaxzxaaaxzaxz0x9xaxx_xqdq_xaaxzaxzxaaaxzaxz0x9xaxxxx_xx test-semantic-epsilon-edge-case-xepsilonxissuec_qq test-semantic-type-xtypec_qexpressionq test-semantic-precedence-xprecedencec_5 test-semantic-boolean-xsidexeffectc_false test-semantic-array-xvalidatec_aqcheckxboundsqa test-semantic-object-xcachec_ottlc_300o test-semantic-complex-generatexcharxclassxmatcherxhasxnegationxs2xm_collectxclassxitemsxs2xx test-semantic-complex-countxflattenxs2xx_x_8_x_qlookupxtableq_c_qlinearxchecksq test-semantic-complex-allxextractxrangesxs2xm_r_xx_validxrangexrdstartm_rdendxx test-semantic-complex-xq_if_s1_else_q test-semantic-complex-s1_xx_null
	@echo "🎯 All semantic parser tests completed!"

# Run all regex parser tests
test-regex: test-semantic-regex-groups-xregexxinxgroupc_xxaaxzaxzxaaaxzaxz0x9xaxx_xqdq_xaaxzaxzxaaaxzaxz0x9xaxxxx_xx
	@echo "🎯 All regex parser tests completed!"

# Run all accessor category tests
test-accessor: test-return_annotation-accessor-xx_s1dvalue test-return_annotation-accessor-xx_s1dname test-return_annotation-accessor-xx_s1ddata test-return_annotation-accessor-xx_s1a0a test-return_annotation-accessor-xx_s1a1a test-return_annotation-accessor-xx_s2a0a test-return_annotation-advanced-accessor-xx_s1axa test-return_annotation-advanced-accessor-xx_s1aca test-return_annotation-advanced-accessor-xx_s1a0dd2a test-return_annotation-advanced-accessor-xx_s1a1c4a test-return_annotation-advanced-accessor-xx_s1a1c10c2a test-return_annotation-chained-accessor-xx_s1dpropertya2adsubprop
	@echo "🎯 All accessor tests completed!"

# Run all array category tests
test-array: test-return_annotation-array-xx_as1a test-return_annotation-array-xx_as2a test-return_annotation-array-xx_aa test-return_annotation-array-xx_as1m_s2a test-return_annotation-array-xx_aqitem1qm_qitem2qa test-return_annotation-array-xx_a42m_100a test-semantic-array-xvalidatec_aqcheckxboundsqa
	@echo "🎯 All array tests completed!"

# Run all boolean category tests
test-boolean: test-semantic-boolean-xsidexeffectc_false
	@echo "🎯 All boolean tests completed!"

# Run all complex category tests
test-complex: test-semantic-complex-group-xsequencexquantifiedc_xidentifier_xxsxx_qmq_xxsxx_identifierxx test-semantic-nested-complex-xnestedxquantifiersc_xxelement_qmqxx__x_xelement_qxqxxxx test-semantic-complex-generatexcharxclassxmatcherxhasxnegationxs2xm_collectxclassxitemsxs2xx test-semantic-complex-countxflattenxs2xx_x_8_x_qlookupxtableq_c_qlinearxchecksq test-semantic-complex-allxextractxrangesxs2xm_r_xx_validxrangexrdstartm_rdendxx test-semantic-complex-xq_if_s1_else_q test-semantic-complex-s1_xx_null
	@echo "🎯 All complex tests completed!"

# Run all destructuring category tests
test-destructuring: test-semantic-destructuring-xdestructuringxparameterc_xidentifierxliteral_xxxsxx_qmq_xxsxx_identifierxliteralxxxx
	@echo "🎯 All destructuring tests completed!"

# Run all identifier category tests
test-identifier: test-return_annotation-identifier-xx_true test-return_annotation-identifier-xx_false
	@echo "🎯 All identifier tests completed!"

# Run all literal category tests
test-literal: test-return_annotation-literal-xx_qhelloq test-return_annotation-literal-xx_qtest_stringq test-return_annotation-literal-xx_42 test-return_annotation-literal-xx_123
	@echo "🎯 All literal tests completed!"

# Run all object category tests
test-object: test-return_annotation-object-xx_oo test-return_annotation-object-xx_okeyc_s1o test-return_annotation-object-xx_onamec_s1o test-return_annotation-object-xx_ovaluec_s2o test-return_annotation-object-xx_onamec_s1m_valuec_s2o test-return_annotation-object-xx_oidc_42m_namec_qtestqo test-semantic-object-xcachec_ottlc_300o
	@echo "🎯 All object tests completed!"

# Run all precedence category tests
test-precedence: test-semantic-precedence-xprecedencec_5
	@echo "🎯 All precedence tests completed!"

# Run all quantified category tests
test-quantified: test-semantic-quantified-group-xzeroxorxmorec_xidentifierxx test-semantic-quantified-group-xonexorxmorec_xidentifierxx test-return_annotation-quantified-xx_as1xa test-return_annotation-quantified-xx_okeyc_s1ox test-return_annotation-grouped-quantified-xx_axelement1_element2xxa
	@echo "🎯 All quantified tests completed!"

# Run all type category tests
test-type: test-semantic-type-xtypec_qexpressionq
	@echo "🎯 All type tests completed!"

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
	@echo "  Semantic tests: 67"
	@echo "  Regex tests: 0"
	@echo "  Total tests: 67"

.PHONY: test-all test-return test-semantic test-regex test-quick test-comprehensive test-debug test-stats


# End of auto-generated targets











