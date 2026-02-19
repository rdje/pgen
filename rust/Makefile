# Makefile for pgen - Regex Parser Generator Pipeline
# This manages the complex build dependencies for generating Rust parsers from EBNF grammars
SHELL=/opt/homebrew/bin/bash

# Directories
GRAMMARS_DIR = ../grammars
GENERATED_DIR = ../generated
TOOLS_DIR = ../tools
RUST_DIR = .

# Gate defaults:
# - local runs keep faster 2-cycle checks
# - CI enforces >=3 cycles
FIXED_POINT_CYCLES ?= 2
ifeq ($(CI),true)
FIXED_POINT_CYCLES := 3
endif

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
RUST_AST_PIPELINE_BOOTSTRAP = $(RUST_DIR)/target/debug/ast_pipeline_bootstrap
DIFFERENTIAL_REPORT_DIR = $(RUST_DIR)/target/differential_harness
DIFFERENTIAL_BASELINE_DIR = $(RUST_DIR)/test_data/differential_baseline

# Differential harness policy:
# - default reports mismatches without failing local flows
# - set DIFFERENTIAL_STRICT=1 to make mismatches fail the target
DIFFERENTIAL_STRICT ?= 0

PERFORMANCE_GATE_REPORT_DIR = $(RUST_DIR)/target/performance_gate

# Source dependencies for AST pipeline - if any of these change, parsers need regeneration
AST_PIPELINE_SOURCES = $(RUST_DIR)/src/main.rs \
                      $(RUST_DIR)/src/lib.rs \
                      $(RUST_DIR)/src/ast_pipeline/mod.rs \
                      $(RUST_DIR)/src/ast_pipeline/ast_based_generator.rs \
                      $(RUST_DIR)/src/ast_pipeline/ast_code_generator.rs \
                      $(RUST_DIR)/src/ast_pipeline/ast_generator_direct.rs \
                      $(RUST_DIR)/src/ast_pipeline/ast_return_transform.rs \
                      $(RUST_DIR)/src/ast_pipeline/grouped_quantifier_parser.rs \
                      $(RUST_DIR)/src/ast_pipeline/mutual_recursion_handler.rs \
                      $(RUST_DIR)/src/ast_pipeline/return_annotation_handler.rs \
                      $(RUST_DIR)/src/ast_pipeline/unified_return_ast.rs \
                      $(RUST_DIR)/src/ast_pipeline/unified_semantic_ast.rs \
                      $(RUST_DIR)/Cargo.toml

# Tools
EBNF_TO_JSON = $(TOOLS_DIR)/ebnf_to_json.pl
RUST_GENERATOR = $(RUST_AST_PIPELINE) --generate-parser --debug --trace --eliminate-left-recursion
RUST_GENERATOR_BOOTSTRAP = $(RUST_AST_PIPELINE_BOOTSTRAP) --generate-parser --bootstrap-mode --debug --eliminate-left-recursion

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
$(RUST_AST_PIPELINE_BOOTSTRAP): $(AST_PIPELINE_SOURCES)
	@echo "Ensuring parser placeholders exist to avoid circular dependency..."
	@source ./scripts/create_placeholder_parser.sh Semantic_annotationParser $(SEMANTIC_ANNOTATION_PARSER)
	@source ./scripts/create_placeholder_parser.sh Return_annotationParser $(RETURN_ANNOTATION_PARSER)
	@echo "Building Rust AST pipeline binary only (avoiding circular dependency)..."
	cd $(RUST_DIR) && cargo build --bin ast_pipeline_bootstrap  --no-default-features --features bootstrap

$(RUST_AST_PIPELINE): $(AST_PIPELINE_SOURCES)
	@echo "Building Rust AST pipeline binary only (avoiding circular dependency)..."
	cd $(RUST_DIR) && cargo build --bin ast_pipeline

# Step 2: Generate Rust parsers from JSON files using bootstrap mode
# These parsers are generated using bootstrap mode to avoid circular dependency
$(SEMANTIC_ANNOTATION_PARSER): $(SEMANTIC_ANNOTATION_JSON) $(RUST_AST_PIPELINE_BOOTSTRAP)
	@echo "Generating semantic annotation Rust parser from JSON (bootstrap mode)..."
	@mkdir -p $(GENERATED_DIR)
	@echo "Generating real parser (placeholder already exists from dependencies)..."
	$(RUST_GENERATOR_BOOTSTRAP) $(SEMANTIC_ANNOTATION_JSON) -o $(SEMANTIC_ANNOTATION_PARSER)

$(RETURN_ANNOTATION_PARSER): $(RETURN_ANNOTATION_JSON) $(RUST_AST_PIPELINE_BOOTSTRAP)
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
$(REGEX_JSON): $(REGEX_EBNF)
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
	@echo "  fixed_point_gate - Verify deterministic bootstrap artifacts across cycles"
	@echo "  ebnf_frontend_readiness - Report EBNF frontend flow readiness for ebnf/json/regex grammars"
	@echo "  ebnf_frontend_gate - Strict EBNF frontend gate (fail on any grammar flow failure)"
	@echo "  annotation_contract_gate - Enforce normative bootstrap annotation contracts + validator diagnostics"
	@echo "  annotation_robustness_gate - Enforce advanced annotation suites + generated parseability/coverage checks"
	@echo "  annotation_shared_contract_gate - Enforce shared bootstrap/generated annotation contracts"
	@echo "  semantic_usage_gate - Enforce semantic-annotation leverage contract (parser/stimuli)"
	@echo "  regression_gate - Run whitespace+dotted-identifier regression suites (bootstrap + generated)"
	@echo "  embedding_api_gate - Validate stable embedding API contract/tests (bootstrap + generated)"
	@echo "  performance_gate - Run benchmark thresholds for bootstrap vs generated parsers"
	@echo "  differential_report - Compare generated vs bootstrap parser behavior and emit JSON reports"
	@echo "  differential_refresh_baseline - Refresh tracked differential mismatch baselines"
	@echo "  differential_regression_gate - Fail only on NEW mismatches vs tracked differential baselines"
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
return_annotation_parser: $(RETURN_ANNOTATION_JSON) $(RUST_AST_PIPELINE_BOOTSTRAP)
	@echo "🚀 Starting complete flow for return annotation parser..."
	@echo "📄 Step 1: EBNF -> JSON (already done: $(RETURN_ANNOTATION_JSON))"
	@echo "🔧 Step 2: JSON -> PARSER.RS (bootstrap mode)"
	@mkdir -p $(GENERATED_DIR)
	$(RUST_GENERATOR_BOOTSTRAP) $(RETURN_ANNOTATION_JSON) -o $(RETURN_ANNOTATION_PARSER)
	@echo "✅ Generated: $(RETURN_ANNOTATION_PARSER)"
	@echo "✅ Return annotation parser generated successfully!"

# Semantic annotation parser complete flow
.PHONY: semantic_annotation_parser
semantic_annotation_parser: $(SEMANTIC_ANNOTATION_JSON) $(RUST_AST_PIPELINE_BOOTSTRAP)
	@echo "🚀 Starting complete flow for semantic annotation parser..."
	@echo "📄 Step 1: EBNF -> JSON (already done: $(SEMANTIC_ANNOTATION_JSON))"
	@echo "🔧 Step 2: JSON -> PARSER.RS (bootstrap mode)"
	@mkdir -p $(GENERATED_DIR)
	$(RUST_GENERATOR_BOOTSTRAP) $(SEMANTIC_ANNOTATION_JSON) -o $(SEMANTIC_ANNOTATION_PARSER)
	@echo "✅ Generated: $(SEMANTIC_ANNOTATION_PARSER)"
	@echo "✅ Semantic annotation parser generated successfully!"

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

.PHONY: annotation_parsers
annotation_parsers: return_semantic_parsers

.PHONY: regex_tests
regex_tests: regex_parser

.PHONY: fixed_point_gate
fixed_point_gate:
	@echo "🔁 Running fixed-point bootstrap reproducibility gate..."
	cd $(RUST_DIR) && PGEN_STRICT_ANNOTATION_VALIDATION=$${PGEN_STRICT_ANNOTATION_VALIDATION:-1} ./scripts/fixed_point_bootstrap_gate.sh --cycles $(FIXED_POINT_CYCLES)
	@echo "✅ Fixed-point gate passed."

.PHONY: ebnf_frontend_readiness
ebnf_frontend_readiness:
	@echo "🧭 Running EBNF frontend readiness check (report mode)..."
	cd $(RUST_DIR) && ./scripts/ebnf_frontend_readiness_gate.sh
	@echo "✅ EBNF frontend readiness report generated."

.PHONY: ebnf_frontend_gate
ebnf_frontend_gate:
	@echo "🧱 Running strict EBNF frontend readiness gate..."
	cd $(RUST_DIR) && PGEN_EBNF_FRONTEND_STRICT=1 ./scripts/ebnf_frontend_readiness_gate.sh
	@echo "✅ EBNF frontend gate passed."

.PHONY: regression_gate
regression_gate:
	@echo "🔒 Running regression gate (whitespace + dotted identifiers) across bootstrap and generated parsers..."
	cd $(RUST_DIR) && cargo run --bin test_runner -- --parser return --suite return_annotation_generated_whitespace_regression
	cd $(RUST_DIR) && cargo run --features generated_parsers --bin test_runner -- --parser return --suite return_annotation_generated_whitespace_regression
	cd $(RUST_DIR) && cargo run --bin test_runner -- --parser semantic --suite semantic_annotation_generated_whitespace_and_dotted_regression
	cd $(RUST_DIR) && cargo run --features generated_parsers --bin test_runner -- --parser semantic --suite semantic_annotation_generated_whitespace_and_dotted_regression
	@echo "✅ Regression gate passed."

.PHONY: annotation_contract_gate
annotation_contract_gate:
	@echo "📏 Running annotation normative contract gate..."
	cd $(RUST_DIR) && cargo test --lib annotation_validator
	cd $(RUST_DIR) && cargo run --bin test_runner -- --parser return --suite return_annotation_builtin_contract
	cd $(RUST_DIR) && cargo run --bin test_runner -- --parser semantic --suite semantic_annotation_builtin_contract
	@$(MAKE) -C $(RUST_DIR) annotation_shared_contract_gate
	@$(MAKE) -C $(RUST_DIR) semantic_usage_gate
	@$(MAKE) -C $(RUST_DIR) annotation_robustness_gate
	@echo "✅ Annotation normative contract gate passed."

.PHONY: annotation_robustness_gate
annotation_robustness_gate:
	@echo "🧪 Running annotation robustness gate..."
	cd $(RUST_DIR) && ./scripts/annotation_robustness_gate.sh
	@echo "✅ Annotation robustness gate passed."

.PHONY: annotation_shared_contract_gate
annotation_shared_contract_gate:
	@echo "🤝 Running shared bootstrap/generated annotation contract gate..."
	cd $(RUST_DIR) && cargo run --bin test_runner -- --parser return --suite return_annotation_normative_shared_contract
	cd $(RUST_DIR) && cargo run --features generated_parsers --bin test_runner -- --parser return --suite return_annotation_normative_shared_contract
	cd $(RUST_DIR) && cargo run --bin test_runner -- --parser semantic --suite semantic_annotation_normative_shared_contract
	cd $(RUST_DIR) && cargo run --features generated_parsers --bin test_runner -- --parser semantic --suite semantic_annotation_normative_shared_contract
	@echo "✅ Shared annotation contract gate passed."

.PHONY: semantic_usage_gate
semantic_usage_gate:
	@echo "🧠 Running semantic annotation leverage usage gate..."
	cd $(RUST_DIR) && cargo test --lib semantic_usage_
	@echo "✅ Semantic annotation usage gate passed."

.PHONY: differential_report
differential_report:
	@echo "🧪 Running differential harness (generated vs bootstrap)..."
	@mkdir -p $(DIFFERENTIAL_REPORT_DIR)
	cd $(RUST_DIR) && cargo build --features generated_parsers --bin test_runner
	@set +e; \
	cd $(RUST_DIR) && ./target/debug/test_runner --differential --parser return --differential-report-json target/differential_harness/return_annotation_diff_report.json; rc_return=$$?; \
	cd $(RUST_DIR) && ./target/debug/test_runner --differential --parser semantic --differential-report-json target/differential_harness/semantic_annotation_diff_report.json; rc_semantic=$$?; \
	set -e; \
	echo "Differential report exit codes: return=$$rc_return semantic=$$rc_semantic"; \
	if [ $$rc_return -ne 0 ] || [ $$rc_semantic -ne 0 ]; then \
		echo "⚠️ Differential mismatches detected. Reports in $(DIFFERENTIAL_REPORT_DIR)."; \
		if [ "$(DIFFERENTIAL_STRICT)" = "1" ]; then \
			echo "❌ DIFFERENTIAL_STRICT=1: failing due to mismatches."; \
			exit 1; \
		fi; \
	else \
		echo "✅ Differential harness found no mismatches."; \
	fi

.PHONY: differential_refresh_baseline
differential_refresh_baseline:
	@echo "🧷 Refreshing differential mismatch baseline snapshots..."
	@mkdir -p $(DIFFERENTIAL_BASELINE_DIR)
	cd $(RUST_DIR) && cargo build --features generated_parsers --bin test_runner
	@set +e; \
	cd $(RUST_DIR) && ./target/debug/test_runner --differential --parser return --differential-write-baseline-json test_data/differential_baseline/return_annotation_baseline.json; rc_return=$$?; \
	cd $(RUST_DIR) && ./target/debug/test_runner --differential --parser semantic --differential-write-baseline-json test_data/differential_baseline/semantic_annotation_baseline.json; rc_semantic=$$?; \
	set -e; \
	if [ $$rc_return -gt 1 ] || [ $$rc_semantic -gt 1 ]; then \
		echo "❌ differential_refresh_baseline failed (unexpected harness error)."; \
		exit 1; \
	fi
	@echo "✅ Differential baselines refreshed in $(DIFFERENTIAL_BASELINE_DIR)."

.PHONY: differential_regression_gate
differential_regression_gate:
	@echo "🚧 Running differential regression gate (new mismatches only)..."
	@mkdir -p $(DIFFERENTIAL_REPORT_DIR)
	cd $(RUST_DIR) && cargo build --features generated_parsers --bin test_runner
	cd $(RUST_DIR) && ./target/debug/test_runner --differential --parser return --differential-baseline-json test_data/differential_baseline/return_annotation_baseline.json --differential-regression-only --differential-report-json target/differential_harness/return_annotation_diff_report.json
	cd $(RUST_DIR) && ./target/debug/test_runner --differential --parser semantic --differential-baseline-json test_data/differential_baseline/semantic_annotation_baseline.json --differential-regression-only --differential-report-json target/differential_harness/semantic_annotation_diff_report.json
	@echo "✅ Differential regression gate passed (no new mismatches vs baseline)."

.PHONY: performance_gate
performance_gate:
	@echo "⚡ Running performance benchmark threshold gate..."
	@mkdir -p $(PERFORMANCE_GATE_REPORT_DIR)
	cd $(RUST_DIR) && ./scripts/performance_gate.sh
	@echo "✅ Performance gate passed."

.PHONY: embedding_api_gate
embedding_api_gate:
	@echo "🔌 Running embedding API stability gate..."
	cd $(RUST_DIR) && cargo test --lib embedding_api
	cd $(RUST_DIR) && cargo test --features generated_parsers --lib embedding_api
	@echo "✅ Embedding API gate passed."

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
