# Makefile for pgen - Regex Parser Generator Pipeline
# This manages the complex build dependencies for generating Rust parsers from EBNF grammars

# Directories
GRAMMARS_DIR = grammars
GENERATED_DIR = generated
TOOLS_DIR = tools
RUST_DIR = rust

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
	@echo "pub struct Semantic_annotationsParser;" >> $(SEMANTIC_ANNOTATION_PARSER)
	@echo "impl Semantic_annotationsParser {" >> $(SEMANTIC_ANNOTATION_PARSER)
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
	@echo "Main targets:"
	@echo "  all          - Build complete pipeline (default)"
	@echo "  debug-json   - Generate regex JSON with debug output"
	@echo "  test-parser  - Test the generated parser"
	@echo "  rebuild      - Clean and rebuild everything"
	@echo ""
	@echo "Utility targets:"
	@echo "  clean        - Remove generated files"
	@echo "  clean-all    - Remove all build artifacts"
	@echo "  help         - Show this help"
	@echo ""
	@echo "Build order:"
	@echo "  1. Generate JSON files from EBNF grammars"
	@echo "  2. Build Rust AST pipeline (cargo build)"
	@echo "  3. Generate Rust parsers from JSON files"
	@echo "  4. Generate regex.json from regex.ebnf (with debug output)"
	@echo "  5. Generate final regex_parser.rs from regex.json"

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

.PHONY: debug-semantic-annotations
debug-semantic-annotations: $(SEMANTIC_ANNOTATION_EXE) $(RETURN_ANNOTATION_EXE)
	@echo "Testing semantic annotation parsing with debug output..."
	@mkdir -p $(GENERATED_DIR)
	@echo "Testing semantic annotation parser with sample input:" | tee $(GENERATED_DIR)/semantic_debug.log
	@echo 'codegen: "escape_literal_handling"' | $(SEMANTIC_ANNOTATION_EXE) --debug 2>&1 | tee $(GENERATED_DIR)/semantic_debug.log
	@echo "\n\nTesting return annotation parser with sample input:" | tee $(GENERATED_DIR)/semantic_debug.log
	@echo '{type: "escape", pattern: $$1}' | $(RETURN_ANNOTATION_EXE) --debug 2>&1 | tee $(GENERATED_DIR)/semantic_debug.log
	@echo "\nDebug output saved to $(GENERATED_DIR)/semantic_debug.log"

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
