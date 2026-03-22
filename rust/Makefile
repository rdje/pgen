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
GENERATED_LOG_DIR = $(RUST_DIR)/target/generated_logs

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

# Build Rust AST pipeline binaries.
# - bootstrap binary intentionally avoids generated parser dependency to break initial cycle
# - normal binary is built with generated_parsers so non-bootstrap annotation parsing uses generated backends
$(RUST_AST_PIPELINE_BOOTSTRAP): $(AST_PIPELINE_SOURCES)
	@echo "Ensuring parser placeholders exist to avoid circular dependency..."
	@[ -f $(SEMANTIC_ANNOTATION_PARSER) ] || source ./scripts/create_placeholder_parser.sh Semantic_annotationParser $(SEMANTIC_ANNOTATION_PARSER)
	@[ -f $(RETURN_ANNOTATION_PARSER) ] || source ./scripts/create_placeholder_parser.sh Return_annotationParser $(RETURN_ANNOTATION_PARSER)
	@echo "Building Rust AST pipeline binary only (avoiding circular dependency)..."
	cd $(RUST_DIR) && cargo build --bin ast_pipeline_bootstrap  --no-default-features --features bootstrap

$(RUST_AST_PIPELINE): $(AST_PIPELINE_SOURCES) $(SEMANTIC_ANNOTATION_PARSER) $(RETURN_ANNOTATION_PARSER)
	@echo "Building Rust AST pipeline binary with generated parsers enabled..."
	cd $(RUST_DIR) && cargo build --features generated_parsers --bin ast_pipeline

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
	@echo "  branch_protection_contract_gate - Validate tracked branch-protection required-check contract vs workflows"
	@echo "  ci_workflow_local_gate - Run tracked GitHub workflow gate commands from a tracked-only local export"
	@echo "  clippy_on_rust_change - Run cargo clippy flow automatically when Rust/generated Rust files changed"
	@echo "  sota_exit_gate - Run release-grade aggregate gate (fixed-point, annotation, differential, performance, embedding, EBNF readiness report)"
	@echo "  sota_release_policy - Show active SOTA release policy file"
	@echo "  ebnf_frontend_readiness - Report EBNF frontend flow readiness for ebnf/json/regex grammars"
	@echo "  ebnf_frontend_gate - Strict EBNF frontend gate (fail on any grammar flow failure)"
	@echo "  ebnf_frontend_dual_run_diff - Report Perl-vs-Rust EBNF frontend differential across ebnf/json/regex"
	@echo "  ebnf_frontend_dual_run_gate - Strict Perl-vs-Rust EBNF frontend differential gate"
	@echo "  hdl_frontend_readiness - Report HDL frontend flow readiness for systemverilog/vhdl grammars"
	@echo "  hdl_frontend_gate - Strict HDL frontend readiness gate (fails on missing/failing HDL grammar flows)"
	@echo "  sv_preprocessor_quality_gate - Enforce deterministic closed-loop quality checks for systemverilog_preprocessor grammar"
	@echo "  sv_preprocessor_aggregate_contract_gate - Enforce repeatable aggregate-report contract checks for focused preprocessor evidence surfaces"
	@echo "  sv_preprocessor_reachability_closure_gate - Enforce zero-target/full-reachable-coverage preprocessor closure under checked-in policy"
	@echo "  sv_preprocessor_curated_differential_gate - Run curated expected-artifact differential corpus and classify expected-vs-bug taxonomy deltas"
	@echo "  sv_preprocessor_template_differential_gate - Run dynamic template-based preprocessor oracle differential (fully offline)"
	@echo "  sv_stimuli_quality_gate - Enforce preprocess-first SystemVerilog closed-loop quality gate (coverage/gap/replay -> preprocess -> parse_full -> semantic baseline)"
	@echo "  sv_parser_aggregate_contract_gate - Enforce repeatable aggregate-report contract checks for focused SystemVerilog parser evidence surfaces"
	@echo "  sv_parser_family_status_gate - Compute machine-checkable SV-family status rationale and verify live-tracker alignment"
	@echo "  sv_parser_family_status_contract_gate - Enforce schema/consistency contract for the SV-family status sidecar"
	@echo "  sv_external_corpus_triage_gate - Run deterministic external SystemVerilog corpus preprocess/parse triage over tracked SV/UVM sources"
	@echo "  regex_parser_family_contract_gate - Enforce combined contract for the current regex family proof surfaces"
	@echo "  regex_parser_family_status_gate - Compute machine-checkable regex family status rationale and verify live-tracker alignment"
	@echo "  regex_parser_family_status_contract_gate - Enforce schema/consistency contract for the regex family-status sidecar"
	@echo "  regex_combined_telemetry_contract_gate - Enforce aggregate regex-family telemetry consistency against regex family sidecars"
	@echo "  vhdl_parser_family_contract_gate - Enforce combined contract for the current VHDL family proof surfaces"
	@echo "  vhdl_combined_telemetry_contract_gate - Enforce aggregate VHDL-family telemetry consistency against the VHDL family sidecar"
	@echo "  vhdl_parser_family_status_gate - Compute machine-checkable VHDL family status rationale and verify live-tracker alignment"
	@echo "  vhdl_parser_family_status_contract_gate - Enforce schema/consistency contract for the VHDL family-status sidecar"
	@echo "  vhdl_external_corpus_triage_gate - Run deterministic external VHDL corpus parse triage over tracked VHDL submodule sources"
	@echo "  sv_replay_gap_target_triage - Surface deterministic triage for the remaining focused SystemVerilog replay-gap targets"
	@echo "  sv_failure_context_contract_gate - Enforce lightweight source-level failure-context proof across both SV parser families"
	@echo "  sv_roundtrip_contract_gate - Enforce lightweight parser/stimuli roundtrip proof across both SV parser families"
	@echo "  sv_combined_telemetry_contract_gate - Enforce aggregate SV-family telemetry consistency against combined proof sidecars"
	@echo "  sv_semantic_scope_contract_gate - Enforce lightweight parser-backed SV semantic-scope reduced proof cases"
	@echo "  sv_declared_shadow_promotion_gate - Run strict declared-shadow promotion trials and emit runtime-enforcement readiness report"
	@echo "  sv_parse_full_ratio_promotion_gate - Run strict parse-full ratio promotion trials and emit threshold-ratchet readiness report"
	@echo "  vhdl_stimuli_quality_gate - Enforce deterministic VHDL closed-loop stimuli quality gate (coverage/gap/replay -> parse_full)"
	@echo "  vhdl_strict_promotion_gate - Run deterministic VHDL strict-promotion trials and emit required-mode readiness report"
	@echo "  sv_syntax_closure_gate - Enforce deterministic SystemVerilog syntax-closure burn-down no-regression contract"
	@echo "  sv_preprocessor_syntax_closure_gate - Enforce deterministic SystemVerilog preprocessor syntax-closure burn-down no-regression contract"
	@echo "  ast_dump_contract_gate - Enforce deterministic/bounded AST dump contract checks (generation + parser dump surfaces)"
	@echo "  annotation_contract_gate - Enforce normative bootstrap annotation contracts + validator diagnostics"
	@echo "  annotation_robustness_gate - Enforce advanced annotation suites + generated parseability/coverage checks"
	@echo "  annotation_stimuli_quality_gate - Enforce strict closed-loop stimuli/coverage/gap checks (no-regression) for annotation grammars"
	@echo "  ebnf_stimuli_quality_gate - Enforce strict closed-loop parser/stimuli quality checks for non-annotation tracked EBNFs"
	@echo "  stimuli_module_parity_gate - Enforce in-memory vs generated stimuli-module parity (samples + coverage + gap)"
	@echo "  return_annotation_exhaustiveness_gate - Enforce auto-derived return-annotation closure proof (coverage + parity + typed AST)"
	@echo "  annotation_nonbootstrap_e2e_gate - Enforce non-bootstrap annotation parser/stimuli end-to-end checks"
	@echo "  annotation_shared_contract_gate - Enforce shared bootstrap/generated annotation contracts"
	@echo "  sc08_contract_gate - Enforce SC-08 value-domain Tier-4 contract slices + differential taxonomy checks"
	@echo "  sc05_contract_gate - Enforce SC-05 precedence/associativity Tier-4 contract slices + differential taxonomy checks"
	@echo "  sc12_contract_gate - Enforce SC-12 deterministic-partition Tier-4 contract slices + differential taxonomy checks"
	@echo "  sc11_contract_gate - Enforce SC-11 negative-case Tier-4 contract slices + differential taxonomy checks"
	@echo "  sc10_contract_gate - Enforce SC-10 coverage-target Tier-4 contract slices + differential taxonomy checks"
	@echo "  sc09_contract_gate - Enforce SC-09 relational-constraint Tier-4 contract slices + differential taxonomy checks"
	@echo "  sc07_contract_gate - Enforce SC-07 recovery/sync Tier-4 contract slices + differential taxonomy checks"
	@echo "  sc06_contract_gate - Enforce SC-06 branch weighting/selection Tier-4 contract slices + differential taxonomy checks"
	@echo "  sc03_contract_gate - Enforce SC-03 directive routing + strict policy contract slices and differential taxonomy checks"
	@echo "  sc04_contract_gate - Enforce SC-04 token steering Tier-4 contract slices + differential taxonomy checks"
	@echo "  semantic_usage_gate - Enforce semantic-annotation leverage contract (parser/stimuli)"
	@echo "  semantic_runtime_contract_gate - Enforce semantic runtime/typed-AST contract checks"
	@echo "  semantic_ast_roundtrip_gate - Enforce semantic AST round-trip contract checks"
	@echo "  semantic_full_contract_gate - Enforce aggregate semantic contract gate (runtime + round-trip + regression)"
	@echo "  return_annotation_support_gate - Enforce aggregate 100% return-annotation support proof (audit + contract + stimuli)"
	@echo "  return_runtime_semantics_gate - Enforce typed return AST/runtime transform contract checks"
	@echo "  return_ast_roundtrip_gate - Enforce canonical return AST round-trip contract checks"
	@echo "  return_full_contract_gate - Enforce aggregate return contract gate (runtime + round-trip + parity)"
	@echo "  annotation_100_gate - Enforce aggregate annotation 100% proof gate"
	@echo "  regression_gate - Run whitespace+dotted-identifier regression suites (bootstrap + generated)"
	@echo "  embedding_api_gate - Validate stable embedding API contract/tests (bootstrap + generated)"
	@echo "  nexsim_parser_embedding_contract_gate - Validate Nexsim parser-profile embedding contract (SV/VHDL profiles + diagnostics)"
	@echo "  performance_gate - Run benchmark thresholds for bootstrap vs generated parsers"
	@echo "  differential_report - Compare generated vs bootstrap parser behavior and emit JSON reports"
	@echo "  differential_refresh_baseline - Refresh tracked differential mismatch baselines"
	@echo "  differential_regression_gate - Fail only on NEW mismatches vs tracked differential baselines"
	@echo "  return_parity_gate - Enforce zero return mismatches on comparable (expectation-aligned) differential corpus"
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
	@mkdir -p $(GENERATED_DIR) $(GENERATED_LOG_DIR)
	@rm -f $(REGEX_JSON) $(REGEX_PARSER)
	@echo "Step 1: Generate regex JSON..." | tee $(GENERATED_LOG_DIR)/regex_parser.log
	$(EBNF_TO_JSON) --verbosity debug --pretty $(REGEX_EBNF) -o $(REGEX_JSON) 2>&1 | tee $(GENERATED_LOG_DIR)/regex_parser.log
	@echo "\nStep 2: Generate regex parser from JSON..." | tee -a $(GENERATED_LOG_DIR)/regex_parser.log
	$(RUST_GENERATOR) $(REGEX_JSON) -o $(REGEX_PARSER) 2>&1 | tee -a $(GENERATED_LOG_DIR)/regex_parser.log
	@echo "\nRegex parser generation complete! Check $(GENERATED_LOG_DIR)/regex_parser.log for details"

.PHONY: return-annotation-parser-with-log
return-annotation-parser-with-log:
	@echo "Generating return annotation parser with full debug logging..."
	@mkdir -p $(GENERATED_DIR) $(GENERATED_LOG_DIR)
	@rm -f $(RETURN_ANNOTATION_JSON) $(RETURN_ANNOTATION_PARSER)
	@echo "Step 1: Generate return annotation JSON..." | tee $(GENERATED_LOG_DIR)/return_annotation_parser.log
	$(EBNF_TO_JSON) --verbosity debug --pretty $(RETURN_ANNOTATION_EBNF) -o $(RETURN_ANNOTATION_JSON) 2>&1 | tee $(GENERATED_LOG_DIR)/return_annotation_parser.log
	@echo "\nStep 2: Generate return annotation parser from JSON..." | tee -a $(GENERATED_LOG_DIR)/return_annotation_parser.log
	$(RUST_GENERATOR_BOOTSTRAP) $(RETURN_ANNOTATION_JSON) -o $(RETURN_ANNOTATION_PARSER) 2>&1 | tee -a $(GENERATED_LOG_DIR)/return_annotation_parser.log
	@echo "\nReturn annotation parser generation complete! Check $(GENERATED_LOG_DIR)/return_annotation_parser.log for details"

.PHONY: semantic-annotation-parser-with-log
semantic-annotation-parser-with-log:
	@echo "Generating semantic annotation parser with full debug logging..."
	@mkdir -p $(GENERATED_DIR) $(GENERATED_LOG_DIR)
	@rm -f $(SEMANTIC_ANNOTATION_JSON) $(SEMANTIC_ANNOTATION_PARSER)
	@echo "Step 1: Generate semantic annotation JSON..." | tee $(GENERATED_LOG_DIR)/semantic_annotation_parser.log
	$(EBNF_TO_JSON) --verbosity debug --pretty $(SEMANTIC_ANNOTATION_EBNF) -o $(SEMANTIC_ANNOTATION_JSON) 2>&1 | tee $(GENERATED_LOG_DIR)/semantic_annotation_parser.log
	@echo "\nStep 2: Generate semantic annotation parser from JSON..." | tee -a $(GENERATED_LOG_DIR)/semantic_annotation_parser.log
	$(RUST_GENERATOR_BOOTSTRAP) $(SEMANTIC_ANNOTATION_JSON) -o $(SEMANTIC_ANNOTATION_PARSER) 2>&1 | tee -a $(GENERATED_LOG_DIR)/semantic_annotation_parser.log
	@echo "\nSemantic annotation parser generation complete! Check $(GENERATED_LOG_DIR)/semantic_annotation_parser.log for details"

.PHONY: debug-semantic-annotations
debug-semantic-annotations: $(SEMANTIC_ANNOTATION_EXE) $(RETURN_ANNOTATION_EXE)
	@echo "Testing semantic annotation parsing with debug output..."
	@mkdir -p $(GENERATED_LOG_DIR)
	@echo "Testing semantic annotation parser with sample input:" | tee $(GENERATED_LOG_DIR)/semantic_debug.log
	@echo 'codegen: "escape_literal_handling"' | $(SEMANTIC_ANNOTATION_EXE) --debug 2>&1 | tee -a $(GENERATED_LOG_DIR)/semantic_debug.log
	@echo "\n\nTesting return annotation parser with sample input:" | tee -a $(GENERATED_LOG_DIR)/semantic_debug.log
	@echo '{type: "escape", pattern: $$1}' | $(RETURN_ANNOTATION_EXE) --debug 2>&1 | tee -a $(GENERATED_LOG_DIR)/semantic_debug.log
	@echo "\nDebug output saved to $(GENERATED_LOG_DIR)/semantic_debug.log"

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

.PHONY: branch_protection_contract_gate
branch_protection_contract_gate:
	@echo "🛡️  Running branch protection contract gate..."
	cd $(RUST_DIR) && ./scripts/branch_protection_contract_gate.sh
	@echo "✅ Branch protection contract gate passed."

.PHONY: ci_workflow_local_gate
ci_workflow_local_gate:
	@echo "🧪 Running local GitHub workflow parity gate..."
	cd $(RUST_DIR) && ./scripts/ci_workflow_local_gate.sh
	@echo "✅ Local GitHub workflow parity gate passed."

.PHONY: clippy_on_rust_change
clippy_on_rust_change:
	@echo "🧹 Running clippy flow on Rust/generated Rust changes..."
	cd $(RUST_DIR) && ./scripts/clippy_on_rust_change.sh
	@echo "✅ clippy_on_rust_change completed."

.PHONY: sota_exit_gate
sota_exit_gate:
	@echo "🏁 Running SOTA exit aggregate gate..."
	cd $(RUST_DIR) && ./scripts/sota_exit_gate.sh
	@echo "✅ SOTA exit aggregate gate passed."

.PHONY: sota_release_policy
sota_release_policy:
	@echo "📜 Active SOTA release policy:"
	@cat $(RUST_DIR)/config/sota_exit_policy.env

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

.PHONY: ebnf_frontend_dual_run_diff
ebnf_frontend_dual_run_diff:
	@echo "🧪 Running EBNF frontend dual-run differential (report mode)..."
	cd $(RUST_DIR) && ./scripts/ebnf_frontend_dual_run_diff_gate.sh
	@echo "✅ EBNF frontend dual-run differential report generated."

.PHONY: ebnf_frontend_dual_run_gate
ebnf_frontend_dual_run_gate:
	@echo "🧱 Running strict EBNF frontend dual-run differential gate..."
	cd $(RUST_DIR) && PGEN_EBNF_DUAL_RUN_STRICT=1 ./scripts/ebnf_frontend_dual_run_diff_gate.sh
	@echo "✅ EBNF frontend dual-run differential gate passed."

.PHONY: hdl_frontend_readiness
hdl_frontend_readiness:
	@echo "🧭 Running HDL frontend readiness check (report mode)..."
	cd $(RUST_DIR) && ./scripts/hdl_frontend_readiness_gate.sh
	@echo "✅ HDL frontend readiness report generated."

.PHONY: hdl_frontend_gate
hdl_frontend_gate:
	@echo "🧱 Running strict HDL frontend readiness gate..."
	cd $(RUST_DIR) && PGEN_HDL_FRONTEND_STRICT=1 ./scripts/hdl_frontend_readiness_gate.sh
	@echo "✅ HDL frontend gate passed."

.PHONY: sv_preprocessor_quality_gate
sv_preprocessor_quality_gate:
	@echo "🧪 Running SystemVerilog preprocessor quality gate..."
	cd $(RUST_DIR) && ./scripts/sv_preprocessor_quality_gate.sh
	@echo "✅ SystemVerilog preprocessor quality gate passed."

.PHONY: sv_preprocessor_aggregate_contract_gate
sv_preprocessor_aggregate_contract_gate:
	@echo "🧪 Running SystemVerilog preprocessor aggregate-report contract gate..."
	cd $(RUST_DIR) && ./scripts/sv_preprocessor_aggregate_contract_gate.sh
	@echo "✅ SystemVerilog preprocessor aggregate-report contract gate passed."

.PHONY: sv_preprocessor_reachability_closure_gate
sv_preprocessor_reachability_closure_gate:
	@echo "🧪 Running SystemVerilog preprocessor reachability-closure gate..."
	cd $(RUST_DIR) && ./scripts/sv_preprocessor_reachability_closure_gate.sh
	@echo "✅ SystemVerilog preprocessor reachability-closure gate passed."

.PHONY: sv_preprocessor_curated_differential_gate
sv_preprocessor_curated_differential_gate:
	@echo "🧪 Running SystemVerilog preprocessor curated differential gate..."
	cd $(RUST_DIR) && ./scripts/sv_preprocessor_curated_differential_gate.sh
	@echo "✅ SystemVerilog preprocessor curated differential gate passed."

.PHONY: sv_preprocessor_template_differential_gate
sv_preprocessor_template_differential_gate:
	@echo "🧪 Running SystemVerilog preprocessor template differential gate..."
	cd $(RUST_DIR) && ./scripts/sv_preprocessor_template_differential_gate.sh
	@echo "✅ SystemVerilog preprocessor template differential gate passed."

.PHONY: sv_stimuli_quality_gate
sv_stimuli_quality_gate:
	@echo "🧪 Running SystemVerilog stimuli closed-loop quality gate..."
	cd $(RUST_DIR) && ./scripts/sv_stimuli_quality_gate.sh
	@echo "✅ SystemVerilog stimuli closed-loop quality gate passed."

.PHONY: sv_parser_aggregate_contract_gate
sv_parser_aggregate_contract_gate:
	@echo "🧪 Running SystemVerilog aggregate-report contract gate..."
	cd $(RUST_DIR) && ./scripts/sv_parser_aggregate_contract_gate.sh
	@echo "✅ SystemVerilog aggregate-report contract gate passed."

.PHONY: sv_parser_family_status_gate
sv_parser_family_status_gate:
	@echo "🧪 Running SystemVerilog parser-family status gate..."
	cd $(RUST_DIR) && ./scripts/sv_parser_family_status_gate.sh
	@echo "✅ SystemVerilog parser-family status gate passed."

.PHONY: sv_parser_family_status_contract_gate
sv_parser_family_status_contract_gate:
	@echo "🧪 Running SystemVerilog parser-family status contract gate..."
	cd $(RUST_DIR) && ./scripts/sv_parser_family_status_contract_gate.sh
	@echo "✅ SystemVerilog parser-family status contract gate passed."

.PHONY: sv_external_corpus_triage_gate
sv_external_corpus_triage_gate:
	@echo "🧪 Running SystemVerilog external corpus triage gate..."
	cd $(RUST_DIR) && ./scripts/sv_external_corpus_triage_gate.sh
	@echo "✅ SystemVerilog external corpus triage gate completed."

.PHONY: regex_parser_family_contract_gate
regex_parser_family_contract_gate:
	@echo "🧪 Running regex parser-family contract gate..."
	cd $(RUST_DIR) && ./scripts/regex_parser_family_contract_gate.sh
	@echo "✅ Regex parser-family contract gate passed."

.PHONY: regex_parser_family_status_gate
regex_parser_family_status_gate:
	@echo "🧪 Running regex parser-family status gate..."
	cd $(RUST_DIR) && ./scripts/regex_parser_family_status_gate.sh
	@echo "✅ Regex parser-family status gate passed."

.PHONY: regex_parser_family_status_contract_gate
regex_parser_family_status_contract_gate:
	@echo "🧪 Running regex parser-family status contract gate..."
	cd $(RUST_DIR) && ./scripts/regex_parser_family_status_contract_gate.sh
	@echo "✅ Regex parser-family status contract gate passed."

.PHONY: regex_combined_telemetry_contract_gate
regex_combined_telemetry_contract_gate:
	@echo "🧪 Running regex combined telemetry contract gate..."
	cd $(RUST_DIR) && ./scripts/regex_combined_telemetry_contract_gate.sh
	@echo "✅ Regex combined telemetry contract gate passed."

.PHONY: vhdl_parser_family_contract_gate
vhdl_parser_family_contract_gate:
	@echo "🧪 Running VHDL parser-family contract gate..."
	cd $(RUST_DIR) && ./scripts/vhdl_parser_family_contract_gate.sh
	@echo "✅ VHDL parser-family contract gate passed."

.PHONY: vhdl_combined_telemetry_contract_gate
vhdl_combined_telemetry_contract_gate:
	@echo "🧪 Running VHDL combined telemetry contract gate..."
	cd $(RUST_DIR) && ./scripts/vhdl_combined_telemetry_contract_gate.sh
	@echo "✅ VHDL combined telemetry contract gate passed."

.PHONY: vhdl_parser_family_status_gate
vhdl_parser_family_status_gate:
	@echo "🧪 Running VHDL parser-family status gate..."
	cd $(RUST_DIR) && ./scripts/vhdl_parser_family_status_gate.sh
	@echo "✅ VHDL parser-family status gate passed."

.PHONY: vhdl_parser_family_status_contract_gate
vhdl_parser_family_status_contract_gate:
	@echo "🧪 Running VHDL parser-family status contract gate..."
	cd $(RUST_DIR) && ./scripts/vhdl_parser_family_status_contract_gate.sh
	@echo "✅ VHDL parser-family status contract gate passed."

.PHONY: vhdl_external_corpus_triage_gate
vhdl_external_corpus_triage_gate:
	@echo "🧪 Running VHDL external corpus triage gate..."
	cd $(RUST_DIR) && ./scripts/vhdl_external_corpus_triage_gate.sh
	@echo "✅ VHDL external corpus triage gate completed."

.PHONY: sv_replay_gap_target_triage
sv_replay_gap_target_triage:
	@echo "🧪 Running SystemVerilog replay-gap target triage proof..."
	cd $(RUST_DIR) && ./scripts/sv_parser_aggregate_contract_gate.sh
	@echo "✅ SystemVerilog replay-gap target triage proof passed."

.PHONY: sv_failure_context_contract_gate
sv_failure_context_contract_gate:
	@echo "🧪 Running SystemVerilog failure-context contract gate..."
	cd $(RUST_DIR) && ./scripts/sv_failure_context_contract_gate.sh
	@echo "✅ SystemVerilog failure-context contract gate passed."

.PHONY: sv_roundtrip_contract_gate
sv_roundtrip_contract_gate:
	@echo "🧪 Running SystemVerilog roundtrip contract gate..."
	cd $(RUST_DIR) && ./scripts/sv_roundtrip_contract_gate.sh
	@echo "✅ SystemVerilog roundtrip contract gate passed."

.PHONY: sv_combined_telemetry_contract_gate
sv_combined_telemetry_contract_gate:
	@echo "🧪 Running SystemVerilog combined telemetry contract gate..."
	cd $(RUST_DIR) && ./scripts/sv_combined_telemetry_contract_gate.sh
	@echo "✅ SystemVerilog combined telemetry contract gate passed."

.PHONY: sv_semantic_scope_contract_gate
sv_semantic_scope_contract_gate:
	@echo "🧪 Running SystemVerilog semantic-scope contract gate..."
	cd $(RUST_DIR) && ./scripts/sv_semantic_scope_contract_gate.sh
	@echo "✅ SystemVerilog semantic-scope contract gate passed."

.PHONY: sv_declared_shadow_promotion_gate
sv_declared_shadow_promotion_gate:
	@echo "🧪 Running SystemVerilog declared-shadow promotion gate..."
	cd $(RUST_DIR) && ./scripts/sv_declared_shadow_promotion_gate.sh
	@echo "✅ SystemVerilog declared-shadow promotion gate completed."

.PHONY: sv_parse_full_ratio_promotion_gate
sv_parse_full_ratio_promotion_gate:
	@echo "🧪 Running SystemVerilog parse-full ratio promotion gate..."
	cd $(RUST_DIR) && ./scripts/sv_parse_full_ratio_promotion_gate.sh
	@echo "✅ SystemVerilog parse-full ratio promotion gate completed."

.PHONY: vhdl_stimuli_quality_gate
vhdl_stimuli_quality_gate:
	@echo "🧪 Running VHDL stimuli closed-loop quality gate..."
	cd $(RUST_DIR) && ./scripts/vhdl_stimuli_quality_gate.sh
	@echo "✅ VHDL stimuli closed-loop quality gate passed."

.PHONY: vhdl_strict_promotion_gate
vhdl_strict_promotion_gate:
	@echo "🧪 Running VHDL strict-promotion gate..."
	cd $(RUST_DIR) && ./scripts/vhdl_strict_promotion_gate.sh
	@echo "✅ VHDL strict-promotion gate completed."

.PHONY: sv_syntax_closure_gate
sv_syntax_closure_gate:
	@echo "🧪 Running SystemVerilog syntax closure gate..."
	cd $(RUST_DIR) && ./scripts/sv_syntax_closure_gate.sh
	@echo "✅ SystemVerilog syntax closure gate passed."

.PHONY: sv_preprocessor_syntax_closure_gate
sv_preprocessor_syntax_closure_gate:
	@echo "🧪 Running SystemVerilog preprocessor syntax closure gate..."
	cd $(RUST_DIR) && ./scripts/sv_preprocessor_syntax_closure_gate.sh
	@echo "✅ SystemVerilog preprocessor syntax closure gate passed."

.PHONY: ast_dump_contract_gate
ast_dump_contract_gate:
	@echo "🧪 Running AST dump contract gate..."
	cd $(RUST_DIR) && ./scripts/ast_dump_contract_gate.sh
	@echo "✅ AST dump contract gate passed."

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
	@$(MAKE) -C $(RUST_DIR) sc12_contract_gate
	@$(MAKE) -C $(RUST_DIR) sc11_contract_gate
	@$(MAKE) -C $(RUST_DIR) sc10_contract_gate
	@$(MAKE) -C $(RUST_DIR) sc09_contract_gate
	@$(MAKE) -C $(RUST_DIR) sc07_contract_gate
	@$(MAKE) -C $(RUST_DIR) sc06_contract_gate
	@$(MAKE) -C $(RUST_DIR) sc08_contract_gate
	@$(MAKE) -C $(RUST_DIR) sc05_contract_gate
	@$(MAKE) -C $(RUST_DIR) sc03_contract_gate
	@$(MAKE) -C $(RUST_DIR) sc04_contract_gate
	@$(MAKE) -C $(RUST_DIR) return_full_contract_gate
	@$(MAKE) -C $(RUST_DIR) semantic_full_contract_gate
	@$(MAKE) -C $(RUST_DIR) annotation_robustness_gate
	@$(MAKE) -C $(RUST_DIR) annotation_stimuli_quality_gate
	@echo "✅ Annotation normative contract gate passed."

.PHONY: sc06_contract_gate
sc06_contract_gate:
	@echo "🧪 Running SC-06 branch weighting/selection contract gate..."
	cd $(RUST_DIR) && ./scripts/sc06_contract_gate.sh
	@echo "✅ SC-06 branch weighting/selection contract gate passed."

.PHONY: sc08_contract_gate
sc08_contract_gate:
	@echo "🧪 Running SC-08 value-domain contract gate..."
	cd $(RUST_DIR) && ./scripts/sc08_contract_gate.sh
	@echo "✅ SC-08 value-domain contract gate passed."

.PHONY: sc05_contract_gate
sc05_contract_gate:
	@echo "🧪 Running SC-05 precedence/associativity contract gate..."
	cd $(RUST_DIR) && ./scripts/sc05_contract_gate.sh
	@echo "✅ SC-05 precedence/associativity contract gate passed."

.PHONY: sc12_contract_gate
sc12_contract_gate:
	@echo "🧪 Running SC-12 deterministic partition contract gate..."
	cd $(RUST_DIR) && ./scripts/sc12_contract_gate.sh
	@echo "✅ SC-12 deterministic partition contract gate passed."

.PHONY: sc11_contract_gate
sc11_contract_gate:
	@echo "🧪 Running SC-11 negative-case contract gate..."
	cd $(RUST_DIR) && ./scripts/sc11_contract_gate.sh
	@echo "✅ SC-11 negative-case contract gate passed."

.PHONY: sc10_contract_gate
sc10_contract_gate:
	@echo "🧪 Running SC-10 coverage-target contract gate..."
	cd $(RUST_DIR) && ./scripts/sc10_contract_gate.sh
	@echo "✅ SC-10 coverage-target contract gate passed."

.PHONY: sc09_contract_gate
sc09_contract_gate:
	@echo "🧪 Running SC-09 relational constraint contract gate..."
	cd $(RUST_DIR) && ./scripts/sc09_contract_gate.sh
	@echo "✅ SC-09 relational constraint contract gate passed."

.PHONY: sc07_contract_gate
sc07_contract_gate:
	@echo "🧪 Running SC-07 recovery/sync contract gate..."
	cd $(RUST_DIR) && ./scripts/sc07_contract_gate.sh
	@echo "✅ SC-07 recovery/sync contract gate passed."

.PHONY: sc03_contract_gate
sc03_contract_gate:
	@echo "🧪 Running SC-03 directive routing contract gate..."
	cd $(RUST_DIR) && ./scripts/sc03_contract_gate.sh
	@echo "✅ SC-03 directive routing contract gate passed."

.PHONY: sc04_contract_gate
sc04_contract_gate:
	@echo "🧪 Running SC-04 token steering contract gate..."
	cd $(RUST_DIR) && ./scripts/sc04_contract_gate.sh
	@echo "✅ SC-04 token steering contract gate passed."

.PHONY: annotation_robustness_gate
annotation_robustness_gate:
	@echo "🧪 Running annotation robustness gate..."
	cd $(RUST_DIR) && ./scripts/annotation_robustness_gate.sh
	@echo "✅ Annotation robustness gate passed."

.PHONY: annotation_stimuli_quality_gate
annotation_stimuli_quality_gate:
	@echo "🧪 Running annotation stimuli quality gate..."
	cd $(RUST_DIR) && ./scripts/annotation_stimuli_quality_gate.sh
	@echo "✅ Annotation stimuli quality gate passed."

.PHONY: ebnf_stimuli_quality_gate
ebnf_stimuli_quality_gate:
	@echo "🧪 Running non-annotation EBNF stimuli quality gate..."
	cd $(RUST_DIR) && ./scripts/ebnf_stimuli_quality_gate.sh
	@echo "✅ Non-annotation EBNF stimuli quality gate passed."

.PHONY: stimuli_module_parity_gate
stimuli_module_parity_gate:
	@echo "🧪 Running stimuli-module parity gate..."
	cd $(RUST_DIR) && ./scripts/stimuli_module_parity_gate.sh
	@echo "✅ Stimuli-module parity gate passed."

.PHONY: annotation_nonbootstrap_e2e_gate
annotation_nonbootstrap_e2e_gate:
	@echo "🧪 Running annotation non-bootstrap end-to-end gate..."
	cd $(RUST_DIR) && ./scripts/annotation_nonbootstrap_e2e_gate.sh
	@echo "✅ Annotation non-bootstrap end-to-end gate passed."

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

.PHONY: semantic_runtime_contract_gate
semantic_runtime_contract_gate:
	@echo "🧪 Running semantic runtime contract gate..."
	cd $(RUST_DIR) && cargo test --lib semantic_validator_
	cd $(RUST_DIR) && cargo test --features generated_parsers --lib generated_semantic_tree_to_ast_
	@$(MAKE) -C $(RUST_DIR) semantic_usage_gate
	@echo "✅ Semantic runtime contract gate passed."

.PHONY: semantic_ast_roundtrip_gate
semantic_ast_roundtrip_gate:
	@echo "🔁 Running semantic AST round-trip gate..."
	cd $(RUST_DIR) && cargo run --bin test_runner -- --parser semantic --suite semantic_annotation_normative_shared_contract
	cd $(RUST_DIR) && cargo run --features generated_parsers --bin test_runner -- --parser semantic --suite semantic_annotation_normative_shared_contract
	@echo "✅ Semantic AST round-trip gate passed."

.PHONY: semantic_differential_regression_gate
semantic_differential_regression_gate:
	@echo "🧩 Running semantic differential regression gate..."
	@mkdir -p $(DIFFERENTIAL_REPORT_DIR)
	cd $(RUST_DIR) && cargo build --features generated_parsers --bin test_runner
	cd $(RUST_DIR) && ./target/debug/test_runner --differential --parser semantic --differential-comparable-only --differential-baseline-json test_data/differential_baseline/semantic_annotation_baseline.json --differential-regression-only --differential-report-json target/differential_harness/semantic_annotation_diff_report.json
	@echo "✅ Semantic differential regression gate passed."

.PHONY: semantic_full_contract_gate
semantic_full_contract_gate:
	@echo "📐 Running semantic full-contract gate..."
	@$(MAKE) -C $(RUST_DIR) semantic_runtime_contract_gate
	@$(MAKE) -C $(RUST_DIR) semantic_ast_roundtrip_gate
	@$(MAKE) -C $(RUST_DIR) semantic_differential_regression_gate
	@echo "✅ Semantic full-contract gate passed."

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
	cd $(RUST_DIR) && ./target/debug/test_runner --differential --parser semantic --differential-comparable-only --differential-write-baseline-json test_data/differential_baseline/semantic_annotation_baseline.json; rc_semantic=$$?; \
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
	cd $(RUST_DIR) && ./target/debug/test_runner --differential --parser semantic --differential-comparable-only --differential-baseline-json test_data/differential_baseline/semantic_annotation_baseline.json --differential-regression-only --differential-report-json target/differential_harness/semantic_annotation_diff_report.json
	@echo "✅ Differential regression gate passed (no new mismatches vs baseline)."

.PHONY: return_parity_gate
return_parity_gate:
	@echo "🧩 Running return parity gate (comparable corpus)..."
	@mkdir -p $(DIFFERENTIAL_REPORT_DIR)
	cd $(RUST_DIR) && cargo build --features generated_parsers --bin test_runner
	cd $(RUST_DIR) && ./target/debug/test_runner --differential --parser return --differential-comparable-only --differential-report-json target/differential_harness/return_annotation_parity_report.json
	@echo "✅ Return parity gate passed (comparable return corpus has zero mismatches)."

.PHONY: return_annotation_support_gate
return_annotation_support_gate:
	@echo "🧱 Running return annotation support gate..."
	cd $(RUST_DIR) && cargo test --features generated_parsers --lib parser_registry --quiet
	@$(MAKE) -C $(RUST_DIR) return_full_contract_gate
	@$(MAKE) -C $(RUST_DIR) return_annotation_exhaustiveness_gate
	@echo "✅ Return annotation support gate passed."

.PHONY: return_annotation_exhaustiveness_gate
return_annotation_exhaustiveness_gate:
	@echo "🧮 Running return annotation exhaustiveness gate..."
	cd $(RUST_DIR) && ./scripts/return_annotation_exhaustiveness_gate.sh
	@echo "✅ Return annotation exhaustiveness gate passed."

.PHONY: return_runtime_semantics_gate
return_runtime_semantics_gate:
	@echo "🧪 Running return runtime semantics gate..."
	cd $(RUST_DIR) && cargo test --lib unified_return_ast
	cd $(RUST_DIR) && cargo test --features generated_parsers --lib generated_return_tree_to_typed_ast_
	cd $(RUST_DIR) && cargo test --lib return_validator
	@echo "✅ Return runtime semantics gate passed."

.PHONY: return_ast_roundtrip_gate
return_ast_roundtrip_gate:
	@echo "🔁 Running return AST round-trip gate..."
	cd $(RUST_DIR) && cargo test --lib test_round_trip_runner
	cd $(RUST_DIR) && cargo run --bin test_runner -- --parser return --suite return_annotation_normative_shared_contract
	@echo "✅ Return AST round-trip gate passed."

.PHONY: return_full_contract_gate
return_full_contract_gate:
	@echo "📐 Running return full-contract gate..."
	@$(MAKE) -C $(RUST_DIR) return_runtime_semantics_gate
	@$(MAKE) -C $(RUST_DIR) return_ast_roundtrip_gate
	@$(MAKE) -C $(RUST_DIR) return_parity_gate
	@echo "✅ Return full-contract gate passed."

.PHONY: annotation_construct_coverage_gate
annotation_construct_coverage_gate:
	@echo "🧮 Running annotation construct coverage gate..."
	@$(MAKE) -C $(RUST_DIR) annotation_stimuli_quality_gate
	@echo "✅ Annotation construct coverage gate passed."

.PHONY: annotation_typed_ast_gate
annotation_typed_ast_gate:
	@echo "🧬 Running annotation typed-AST gate..."
	@$(MAKE) -C $(RUST_DIR) return_runtime_semantics_gate
	@$(MAKE) -C $(RUST_DIR) semantic_runtime_contract_gate
	@$(MAKE) -C $(RUST_DIR) annotation_nonbootstrap_e2e_gate
	@echo "✅ Annotation typed-AST gate passed."

.PHONY: annotation_runtime_intent_gate
annotation_runtime_intent_gate:
	@echo "🎯 Running annotation runtime-intent gate..."
	@$(MAKE) -C $(RUST_DIR) return_full_contract_gate
	@$(MAKE) -C $(RUST_DIR) semantic_full_contract_gate
	@echo "✅ Annotation runtime-intent gate passed."

.PHONY: annotation_determinism_gate
annotation_determinism_gate:
	@echo "🔁 Running annotation determinism gate..."
	@$(MAKE) -C $(RUST_DIR) fixed_point_gate
	@echo "✅ Annotation determinism gate passed."

.PHONY: annotation_differential_parity_gate
annotation_differential_parity_gate:
	@echo "🧩 Running annotation differential parity gate..."
	@$(MAKE) -C $(RUST_DIR) return_parity_gate
	@$(MAKE) -C $(RUST_DIR) semantic_differential_regression_gate
	@echo "✅ Annotation differential parity gate passed."

.PHONY: annotation_100_gate
annotation_100_gate:
	@echo "💯 Running annotation 100% aggregate gate..."
	@$(MAKE) -C $(RUST_DIR) annotation_contract_gate
	@$(MAKE) -C $(RUST_DIR) annotation_construct_coverage_gate
	@$(MAKE) -C $(RUST_DIR) annotation_typed_ast_gate
	@$(MAKE) -C $(RUST_DIR) annotation_runtime_intent_gate
	@$(MAKE) -C $(RUST_DIR) annotation_determinism_gate
	@$(MAKE) -C $(RUST_DIR) annotation_differential_parity_gate
	@echo "✅ Annotation 100% aggregate gate passed."

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
	@$(MAKE) -C $(RUST_DIR) nexsim_parser_embedding_contract_gate
	@echo "✅ Embedding API gate passed."

.PHONY: nexsim_parser_embedding_contract_gate
nexsim_parser_embedding_contract_gate:
	@echo "🧪 Running Nexsim parser embedding profile contract gate..."
	cd $(RUST_DIR) && ./scripts/nexsim_parser_embedding_contract_gate.sh
	@echo "✅ Nexsim parser embedding profile contract gate passed."

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
