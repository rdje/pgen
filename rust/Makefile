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
SYSTEMVERILOG_EBNF = $(GRAMMARS_DIR)/systemverilog.ebnf
JSON_EBNF = $(GRAMMARS_DIR)/json.ebnf
RTL_CONST_EXPR_EBNF = $(GRAMMARS_DIR)/rtl_const_expr.ebnf
SVPP_EBNF = $(GRAMMARS_DIR)/systemverilog_preprocessor.ebnf
RTL_FRONTEND_EBNF = $(GRAMMARS_DIR)/rtl_frontend.ebnf
VHDL_EBNF = $(GRAMMARS_DIR)/vhdl.ebnf
# PARSE-HARNESS.2 — the blessed scratch-register slot (arbitrary-grammar probe).
SCRATCH_EBNF = $(GRAMMARS_DIR)/scratch/scratch.ebnf

# Generated JSON files (intermediate)
SEMANTIC_ANNOTATION_JSON = $(GENERATED_DIR)/semantic_annotation.json
RETURN_ANNOTATION_JSON = $(GENERATED_DIR)/return_annotation.json
REGEX_JSON = $(GENERATED_DIR)/regex.json
SYSTEMVERILOG_JSON = $(GENERATED_DIR)/systemverilog.json
JSON_JSON = $(GENERATED_DIR)/json.json
RTL_CONST_EXPR_JSON = $(GENERATED_DIR)/rtl_const_expr.json
SVPP_JSON = $(GENERATED_DIR)/systemverilog_preprocessor.json
RTL_FRONTEND_JSON = $(GENERATED_DIR)/rtl_frontend.json
VHDL_JSON = $(GENERATED_DIR)/vhdl.json
SCRATCH_JSON = $(GENERATED_DIR)/scratch.json

# Generated Rust parser files (final output)
SEMANTIC_ANNOTATION_PARSER = $(GENERATED_DIR)/semantic_annotation_parser.rs
RETURN_ANNOTATION_PARSER = $(GENERATED_DIR)/return_annotation_parser.rs
REGEX_PARSER = $(GENERATED_DIR)/regex_parser.rs
SYSTEMVERILOG_PARSER = $(GENERATED_DIR)/systemverilog_parser.rs
JSON_PARSER = $(GENERATED_DIR)/json_parser.rs
RTL_CONST_EXPR_PARSER = $(GENERATED_DIR)/rtl_const_expr_parser.rs
SVPP_PARSER = $(GENERATED_DIR)/systemverilog_preprocessor_parser.rs
RTL_FRONTEND_PARSER = $(GENERATED_DIR)/rtl_frontend_parser.rs
VHDL_PARSER = $(GENERATED_DIR)/vhdl_parser.rs
SCRATCH_PARSER = $(GENERATED_DIR)/scratch_parser.rs

# Compiled Rust parser executables
SEMANTIC_ANNOTATION_EXE = $(GENERATED_DIR)/semantic_annotation_parser
RETURN_ANNOTATION_EXE = $(GENERATED_DIR)/return_annotation_parser

# Rust build artifacts
RUST_AST_PIPELINE = $(RUST_DIR)/target/debug/ast_pipeline
RUST_AST_PIPELINE_BOOTSTRAP = $(RUST_DIR)/target/debug/ast_pipeline_bootstrap
RUST_EBNF_FRONTEND_TARGET_DIR = $(RUST_DIR)/target/ebnf_frontend_build
RUST_EBNF_FRONTEND_BIN = $(RUST_EBNF_FRONTEND_TARGET_DIR)/debug/ast_pipeline
DIFFERENTIAL_REPORT_DIR = $(RUST_DIR)/target/differential_harness
DIFFERENTIAL_BASELINE_DIR = $(RUST_DIR)/test_data/differential_baseline
GENERATED_LOG_DIR = $(RUST_DIR)/target/generated_logs

# Differential harness policy:
# - default reports mismatches without failing local flows
# - set DIFFERENTIAL_STRICT=1 to make mismatches fail the target
DIFFERENTIAL_STRICT ?= 0

PERFORMANCE_GATE_REPORT_DIR = $(RUST_DIR)/target/performance_gate

# Source dependencies for AST pipeline - if any of these change, parsers need regeneration.
# DERIVED via $(wildcard), never hand-enumerated (RGX-0078.5.i.7.t1): a hand list drifted
# behind the crate (first_set.rs / fusibility_census.rs et al. were omitted), so a make-only
# flow after editing an unlisted codegen source skipped the rebuild AND the parser regen —
# silently stale artifacts. Over-approximation is safe: cargo no-ops fast and codegen is
# deterministic, so a spurious regen is byte-identical. Covers every module compiled into the
# three pipeline binaries (ast_pipeline / ast_pipeline_bootstrap / the ebnf frontend build).
AST_PIPELINE_SOURCES = $(wildcard $(RUST_DIR)/src/*.rs) \
                      $(wildcard $(RUST_DIR)/src/ast_pipeline/*.rs) \
                      $(wildcard $(RUST_DIR)/src/ast_pipeline/ast_based_generator/*.rs) \
                      $(wildcard $(RUST_DIR)/src/ast_pipeline/ast_based_generator/cascade/*.rs) \
                      $(wildcard $(RUST_DIR)/src/bin/*.rs) \
                      $(wildcard $(RUST_DIR)/src/test_runner/*.rs) \
                      $(RUST_DIR)/build.rs \
                      $(RUST_DIR)/Cargo.toml

# Tools
#
# ⛔⛔ CI-PARITY-GATE-ROT.31 — THE SHIPPING GENERATION PATH IS QUIET BY DEFAULT.
# Director ruling 2026-08-14: *"remove `--debug --trace` for CI streams … these 2 options are only
# for debug, tracing purposes … CI are not debug runs."* `RUST_GENERATOR` carried `--debug --trace`
# and `RUST_GENERATOR_BOOTSTRAP` carried `--debug`, so every `focus_<family>` and every
# `regenerate_generated_parsers` — locally AND in the 11 hosted workflows that reach this recipe
# through `.github/actions/regenerate-parsers` — asked the generator to narrate itself. Measured on
# this tree, one full regeneration (annotation pair + 7 families):
# **6 894 576 244 B → 14 194 408 B**, 485.7× smaller, wall time 154 s → 131 s. Of that AFTER total
# 14 189 570 B is ONE `ast_pipeline` cargo rebuild both sides paid; net of it the generator itself
# went from ~6.88 GB to 4 838 B. Census: `docs/tasks/artifacts/ci_parity_gate_rot/generator_trace_volume.txt`.
#
# ⭐ IT CANNOT CHANGE AN ARTIFACT, AND THAT IS STRUCTURAL RATHER THAN LUCKY. The two flags reach the
# pipeline only through `resolve_trace_verbosity()` → the global verbosity that gates the
# `pgen_trace!` macros (`rust/src/main.rs:936-938`). The `PipelineConfig` fields they also set —
# `debug` and `trace` — are WRITTEN AND NEVER READ anywhere in the crate, so `--trace`'s own CLI help
# (*"Enable trace mode in generated parser"*) describes a wire that is not connected. Verified
# empirically all the same: all 9 regenerated parsers plus `generated/ebnf.rs` are byte-identical
# across this change, against a same-session determinism control.
#
# ⭐ THE TRACE IS NOT DELETED — IT IS OPT-IN, THROUGH THE KNOB THE ENGINE ALREADY HAS rather than a
# new one invented beside it (`DESIGN-PRIOR-ART`): `resolve_trace_verbosity` consults the
# environment whenever no `--verbosity` is passed, and no recipe here passes one. So
#     PGEN_TRACE_VERBOSITY=debug make -C rust SHELL=/bin/bash focus_systemverilog
# restores the full stream these flags used to force, and `=high` reproduces the old
# `RUST_GENERATOR_BOOTSTRAP` level. See `TOOLBOX.md` §2.1 and `docs/book/src/gate-flow.md`.
#
# ⛔ `FLOW-INTEGRITY` invariant (10) holds this: no `rust/Makefile` line that invokes
# `--generate-parser` may carry `--debug` or `--trace` again.
RUST_GENERATOR = $(RUST_AST_PIPELINE) --generate-parser --eliminate-left-recursion
RUST_GENERATOR_BOOTSTRAP = $(RUST_AST_PIPELINE_BOOTSTRAP) --generate-parser --bootstrap-mode --eliminate-left-recursion
RUST_FRONTEND_TO_JSON = $(RUST_EBNF_FRONTEND_BIN)

# ⛔⛔ CI-PARITY-GATE-ROT.32 (d) — EVERY OPERATOR ENTRY POINT GUARDS MAKE 3.81's WHOLE-SECOND BLIND
# SPOT BEFORE IT BUILDS ANYTHING. Read `scripts/make_freshness_guard.sh` for the full mechanism; the
# short version is that this host's only `make` is GNU Make **3.81**, which compares mtimes at
# WHOLE-SECOND granularity, so a prerequisite rewritten in the same second as its target is invisible
# and the rule is skipped at exit 0 — silently, in the PASSING direction.
#
# ⭐ THE SWEEP WAS MEASURED, NOT ESTIMATED, AND IT REFUTED THE ACUTE FIX'S OWN PREMISE. `.32` closed
# the scratch path and recorded that the minute-long families were "unlikely" to be hit. The census
# (`docs/tasks/artifacts/ci_parity_gate_rot/make_freshness_window_census.txt`) says:
#   - EDGE A  json ← grammar : the gap a driver must beat is the GENERATOR step ⇒ 5 of 10 exposed
#                              (return_annotation 0.33 s, json 0.14 s, scratch 0.06 s,
#                               rtl_const_expr 0.33 s, systemverilog_preprocessor 0.47 s).
#   - EDGE B  parser ← json  : the gap is the FRONTEND step, which is 0.006-0.111 s for EVERY family
#                              ⇒ **10 of 10 exposed, SystemVerilog included (0.054 s)**. A family can
#                              pass edge A and still hand you a FRESH json with a STALE parser.
# ⇒ slowness protected nobody on edge B, and the exposure is a property of today's grammars and
# today's machine, not of the rules.
#
# ⛔ THE GUARD IS NOT `rm`, DELIBERATELY. It removes an artifact ONLY when make's own comparison is
# provably wrong (same whole second AND the prerequisite genuinely newer), so an up-to-date tree pays
# nothing and `focus_systemverilog` keeps its no-op — which matters because three tracked gates call
# `focus_*` purely to ENSURE an artifact exists (`sv_cert_recognized_union_gate.sh`,
# `verilog_2005_conformance_gate.sh`, `rtl_const_expr_cert_gate.sh`) and would otherwise pay a full
# 28.5 s SV regeneration every run.
#
# ⛔ RECIPE STEP + RECURSIVE $(MAKE), NEVER A SIBLING PREREQUISITE — sibling prerequisites have no
# guaranteed order under `-j`, so the guard could race the build it must precede. Same reasoning as
# `focus_scratch`, which keeps its stronger unconditional `rm` because a probe caller has just edited
# the slot by construction.
#
# `FLOW-INTEGRITY` invariant (11) holds this: every `$(<FAM>_JSON): $(<FAM>_EBNF)` rule must be
# covered by a guard invocation or by an unconditional `rm`.
FRESHNESS_GUARD = bash ../scripts/make_freshness_guard.sh

# Default target
.PHONY: all
all: $(REGEX_PARSER)

# Step 1: Generate JSON from EBNF files
$(SEMANTIC_ANNOTATION_JSON): $(SEMANTIC_ANNOTATION_EBNF) $(RUST_EBNF_FRONTEND_BIN)
	@echo "Generating semantic annotation JSON from EBNF (Rust frontend)..."
	@mkdir -p $(GENERATED_DIR)
	$(RUST_FRONTEND_TO_JSON) $(SEMANTIC_ANNOTATION_EBNF) --emit-raw-ast-json $(SEMANTIC_ANNOTATION_JSON)

$(RETURN_ANNOTATION_JSON): $(RETURN_ANNOTATION_EBNF) $(RUST_EBNF_FRONTEND_BIN)
	@echo "Generating return annotation JSON from EBNF (Rust frontend)..."
	@mkdir -p $(GENERATED_DIR)
	$(RUST_FRONTEND_TO_JSON) $(RETURN_ANNOTATION_EBNF) --emit-raw-ast-json $(RETURN_ANNOTATION_JSON)

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

# CI-PARITY-GATE-ROT.24 slice 2: this rule OWNS `rust/target/debug/ast_pipeline`, and it declares the
# FULL feature set — `generated_parsers` AND `ebnf_dual_run` — so the path's capability is a property
# of the path, not of whichever target happened to run last.
#
# ⛔ WHY THIS IS NOT THE MICRO-OPTIMISATION IT LOOKS LIKE. The rule carried `generated_parsers` alone
# from 2026-02-19 (36678059) — one day BEFORE `ebnf_dual_run` existed (de8ca8bc, 2026-02-20). Nobody
# ever decided to omit it; the rule simply predated the feature and was never revisited. Meanwhile 21
# targets depend on this rule, so ANY of them rebuilt it to the stale, under-featured set whenever a
# source changed. An under-featured binary does not crash — it errors on stderr and yields an EMPTY
# metric row on stdout, and two empty result sets DIFF CLEAN: a measured before/after cert sweep
# produced 18 empty rows per side and nearly published "zero drift" backed by nothing.
#
# ⭐ COST IS GENERATION-TIME ONLY, and this was measured, not assumed. `ebnf_dual_run` has ZERO
# `cfg(feature)` sites in the parser codegen (`ast_based_generator.rs`); every site in the generator
# crate is on a `#[test]`. Proven by construction: the regex parser regenerated by a single-feature
# and a dual-feature generator is BYTE-IDENTICAL (md5 943829436b0d628fa8ab21d12f0962c0 both ways).
# Release runtime binaries (`parseability_probe`, `regex_perf_probe`) are SEPARATE cargo invocations
# with their own `--features`, and Cargo features are per-invocation — so no parser's runtime can be
# reached from here. The regex parser's sub-1us runtime is untouchable by this rule, by construction.
$(RUST_AST_PIPELINE): $(AST_PIPELINE_SOURCES) $(SEMANTIC_ANNOTATION_PARSER) $(RETURN_ANNOTATION_PARSER)
	@echo "Building Rust AST pipeline binary (generated parsers + .ebnf frontend)..."
	cd $(RUST_DIR) && cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline

$(RUST_EBNF_FRONTEND_BIN): $(AST_PIPELINE_SOURCES) $(GENERATED_DIR)/ebnf.rs $(RUST_DIR)/build.rs
	@echo "Building dedicated Rust EBNF frontend binary..."
	cd $(RUST_DIR) && cargo build --target-dir target/ebnf_frontend_build --features ebnf_dual_run --bin ast_pipeline

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
$(REGEX_JSON): $(REGEX_EBNF) $(RUST_EBNF_FRONTEND_BIN)
	@echo "Generating regex JSON with debug output (Rust frontend)..."
	@mkdir -p $(GENERATED_DIR)
	$(RUST_FRONTEND_TO_JSON) $(REGEX_EBNF) --emit-raw-ast-json $(REGEX_JSON)

# Generate final regex parser from JSON
$(REGEX_PARSER): $(REGEX_JSON) $(RUST_AST_PIPELINE)
	@echo "Generating final regex parser from JSON..."
	@mkdir -p $(GENERATED_DIR)
	$(RUST_GENERATOR) $(REGEX_JSON) -o $(REGEX_PARSER)

# Generate json raw-AST JSON from EBNF (GRAMMAR-WELLFORMED.H.3 — Phase H cert-coverage wiring)
$(JSON_JSON): $(JSON_EBNF) $(RUST_EBNF_FRONTEND_BIN)
	@echo "Generating json JSON with debug output (Rust frontend)..."
	@mkdir -p $(GENERATED_DIR)
	$(RUST_FRONTEND_TO_JSON) $(JSON_EBNF) --emit-raw-ast-json $(JSON_JSON)

# Generate final json parser from JSON
$(JSON_PARSER): $(JSON_JSON) $(RUST_AST_PIPELINE)
	@echo "Generating final json parser from JSON..."
	@mkdir -p $(GENERATED_DIR)
	$(RUST_GENERATOR) $(JSON_JSON) -o $(JSON_PARSER)

# json parser complete flow
.PHONY: json_parser
json_parser: $(JSON_JSON) $(RUST_AST_PIPELINE)
	@echo "🚀 Starting complete flow for json parser..."
	@echo "📄 Step 1: EBNF -> JSON via Rust frontend (already done: $(JSON_JSON))"
	@echo "🔧 Step 2: JSON -> PARSER.RS (high-performance mode)"
	@mkdir -p $(GENERATED_DIR)
	$(RUST_GENERATOR) $(JSON_JSON) -o $(JSON_PARSER)
	@echo "✅ Generated: $(JSON_PARSER)"
	@echo "✅ json parser generated successfully!"

# `make focus_json` — regenerate the json artifact so subsequent
# `cargo test --features generated_parsers` invocations exercise the json
# family (build.rs sets has_generated_json_parser when the artifact exists).
.PHONY: focus_json
focus_json:
	@$(FRESHNESS_GUARD) --chain $(JSON_EBNF) $(JSON_JSON) $(JSON_PARSER) --tools $(RUST_EBNF_FRONTEND_BIN) $(RUST_AST_PIPELINE)
	@$(MAKE) $(JSON_PARSER)
	@echo ""
	@echo "✅ json parser artifact ready: $(JSON_PARSER)"
	@echo ""

# Generate scratch raw-AST JSON from EBNF (PARSE-HARNESS.2 — the blessed
# scratch-register slot; the grammar body is meant to be overwritten freely
# for arbitrary-grammar probing).
$(SCRATCH_JSON): $(SCRATCH_EBNF) $(RUST_EBNF_FRONTEND_BIN)
	@echo "Generating scratch JSON with debug output (Rust frontend)..."
	@mkdir -p $(GENERATED_DIR)
# ⛔ PARSE-HARNESS.11 — the slot's HEADER is not part of the throwaway. This rule depends on
# $(SCRATCH_EBNF), so it re-runs exactly when the slot has changed since the last generation —
# i.e. at the one moment a destroyed header is still visible. It CANNOT be caught later: the
# correct probe workflow ends in `git checkout`, so the loss never reaches a commit.
	@bash ../scripts/check_scratch_slot_header.sh --probe-time
	$(RUST_FRONTEND_TO_JSON) $(SCRATCH_EBNF) --emit-raw-ast-json $(SCRATCH_JSON)

# Generate final scratch parser from JSON
$(SCRATCH_PARSER): $(SCRATCH_JSON) $(RUST_AST_PIPELINE)
	@echo "Generating final scratch parser from JSON..."
	@mkdir -p $(GENERATED_DIR)
	$(RUST_GENERATOR) $(SCRATCH_JSON) -o $(SCRATCH_PARSER)

# ⛔⛔ CI-PARITY-GATE-ROT.32 — THE SCRATCH ARTIFACTS ARE DELETED BEFORE EVERY REGENERATION, AND
# THAT IS A CORRECTNESS FIX, NOT A BIG HAMMER.
#
# This repository's only `make` is Apple's **GNU Make 3.81**, which compares prerequisite and
# target timestamps at **WHOLE-SECOND** granularity. Measured on this volume (APFS, nanosecond
# mtimes): with `grammars/scratch/scratch.ebnf` written 0.8 s AFTER `generated/scratch.json`, make
# reports *"is up to date"* and the frontend runs **0** times — so the probe is driven against the
# PREVIOUS grammar. Reproduced deterministically: the slot said `bravo`, the regenerated parser
# still declared `alpha`.
#
# ⭐ WHY IT MATTERS HERE SPECIFICALLY. A human editing a grammar and typing `make` takes longer than
# a second, so this is invisible in hand use. A SCRIPTED or AGENT-DRIVEN probe loop — the exact way
# this slot is meant to be used — hits it constantly, and it fails SILENTLY and in the PASSING
# direction: you get a green run against the wrong grammar. It is a second, independent mechanism
# for the identical symptom `GENERATED-LINT-CORRECTNESS.13` recorded (a run judging the previous
# grammar), which was diagnosed then as a build-ORDER problem.
#
# ⛔ The `rm` is scoped to this PHONY operator entry point, deliberately. Making $(SCRATCH_PARSER)
# itself always-rebuild would rewrite a file `build.rs` keys on and drag cargo into a rebuild on
# every unrelated `make` invocation. Here the caller has just edited the grammar and a regeneration
# is exactly what they asked for. Cost: `focus_scratch` is now always ~2-4 s instead of ~0 s when
# up to date — the right trade for a probe target where a wrong answer costs a session.
#
# `make focus_scratch` — regenerate the scratch artifact so subsequent
# `cargo build/test --features generated_parsers` invocations pick up the
# scratch slot (build.rs sets has_generated_scratch_parser when the artifact
# exists). Edit the grammar BODY of grammars/scratch/scratch.ebnf, run this,
# rebuild the probe, then drive `parseability_probe --parse scratch <input>`.
# ⛔ The removal is a RECIPE STEP followed by a recursive $(MAKE), not a sibling prerequisite.
# Sibling prerequisites have no guaranteed order under `-j`, so `rm` could race the generation it
# is supposed to precede — a fix that reintroduces a nondeterministic version of the same defect.
.PHONY: focus_scratch
focus_scratch:
	@rm -f $(SCRATCH_JSON) $(SCRATCH_PARSER)
	@$(MAKE) $(SCRATCH_PARSER)
	@echo ""
	@echo "✅ scratch parser artifact ready: $(SCRATCH_PARSER)"
	@echo ""
	@echo "Rebuild the probe, then drive the full toolbox on your probe grammar:"
	@echo "  (cd $(RUST_DIR) && cargo build --release --features generated_parsers --bin parseability_probe)"
	@echo "  ./rust/target/release/parseability_probe --parse scratch <input_file>"
	@echo ""
	@echo "⛔ cert-coverage on scratch ALSO needs ast_pipeline rebuilt AFTER this target:"
	@echo "  (cd $(RUST_DIR) && cargo build --features \"generated_parsers ebnf_dual_run\" --bin ast_pipeline)"
	@echo "  this target builds that binary BEFORE regenerating the parser, so one built by this run"
	@echo "  judges the PREVIOUS grammar (session #218: UNKNOWN=9 / every probe parsed=false, on a"
	@echo "  correct grammar). GENERATED-LINT-CORRECTNESS.13."
	@echo ""
	@echo "⭐ If this probe backs a task-leaf claim, PRESERVE IT BEFORE RESTORING the slot:"
	@echo "  scripts/preserve_scratch_probe.sh <TREE-ID> <probe_name>"
	@echo "  then: git checkout grammars/scratch/scratch.ebnf"
	@echo ""

# Generate rtl_const_expr raw-AST JSON from EBNF (GRAMMAR-WELLFORMED.H.4 — Phase H cert-coverage wiring)
$(RTL_CONST_EXPR_JSON): $(RTL_CONST_EXPR_EBNF) $(RUST_EBNF_FRONTEND_BIN)
	@echo "Generating rtl_const_expr JSON with debug output (Rust frontend)..."
	@mkdir -p $(GENERATED_DIR)
	$(RUST_FRONTEND_TO_JSON) $(RTL_CONST_EXPR_EBNF) --emit-raw-ast-json $(RTL_CONST_EXPR_JSON)

# Generate final rtl_const_expr parser from JSON
$(RTL_CONST_EXPR_PARSER): $(RTL_CONST_EXPR_JSON) $(RUST_AST_PIPELINE)
	@echo "Generating final rtl_const_expr parser from JSON..."
	@mkdir -p $(GENERATED_DIR)
	$(RUST_GENERATOR) $(RTL_CONST_EXPR_JSON) -o $(RTL_CONST_EXPR_PARSER)

# rtl_const_expr parser complete flow
.PHONY: rtl_const_expr_parser
rtl_const_expr_parser: $(RTL_CONST_EXPR_JSON) $(RUST_AST_PIPELINE)
	@echo "🚀 Starting complete flow for rtl_const_expr parser..."
	@echo "📄 Step 1: EBNF -> JSON via Rust frontend (already done: $(RTL_CONST_EXPR_JSON))"
	@echo "🔧 Step 2: JSON -> PARSER.RS (high-performance mode)"
	@mkdir -p $(GENERATED_DIR)
	$(RUST_GENERATOR) $(RTL_CONST_EXPR_JSON) -o $(RTL_CONST_EXPR_PARSER)
	@echo "✅ Generated: $(RTL_CONST_EXPR_PARSER)"
	@echo "✅ rtl_const_expr parser generated successfully!"

# `make focus_rtl_const_expr` — regenerate the rtl_const_expr artifact so subsequent
# `cargo test --features generated_parsers` invocations exercise the rtl_const_expr
# family (build.rs sets has_generated_rtl_const_expr_parser when the artifact exists).
.PHONY: focus_rtl_const_expr
focus_rtl_const_expr:
	@$(FRESHNESS_GUARD) --chain $(RTL_CONST_EXPR_EBNF) $(RTL_CONST_EXPR_JSON) $(RTL_CONST_EXPR_PARSER) --tools $(RUST_EBNF_FRONTEND_BIN) $(RUST_AST_PIPELINE)
	@$(MAKE) $(RTL_CONST_EXPR_PARSER)
	@echo ""
	@echo "✅ rtl_const_expr parser artifact ready: $(RTL_CONST_EXPR_PARSER)"
	@echo ""

# Generate systemverilog_preprocessor raw-AST JSON from EBNF (GRAMMAR-WELLFORMED.H.5 — Phase H cert-coverage)
$(SVPP_JSON): $(SVPP_EBNF) $(RUST_EBNF_FRONTEND_BIN)
	@echo "Generating systemverilog_preprocessor JSON with debug output (Rust frontend)..."
	@mkdir -p $(GENERATED_DIR)
	$(RUST_FRONTEND_TO_JSON) $(SVPP_EBNF) --emit-raw-ast-json $(SVPP_JSON)

# Generate final systemverilog_preprocessor parser from JSON
$(SVPP_PARSER): $(SVPP_JSON) $(RUST_AST_PIPELINE)
	@echo "Generating final systemverilog_preprocessor parser from JSON..."
	@mkdir -p $(GENERATED_DIR)
	$(RUST_GENERATOR) $(SVPP_JSON) -o $(SVPP_PARSER)

# systemverilog_preprocessor parser complete flow
.PHONY: systemverilog_preprocessor_parser
systemverilog_preprocessor_parser: $(SVPP_JSON) $(RUST_AST_PIPELINE)
	@echo "🚀 Starting complete flow for systemverilog_preprocessor parser..."
	@echo "📄 Step 1: EBNF -> JSON via Rust frontend (already done: $(SVPP_JSON))"
	@echo "🔧 Step 2: JSON -> PARSER.RS (high-performance mode)"
	@mkdir -p $(GENERATED_DIR)
	$(RUST_GENERATOR) $(SVPP_JSON) -o $(SVPP_PARSER)
	@echo "✅ Generated: $(SVPP_PARSER)"
	@echo "✅ systemverilog_preprocessor parser generated successfully!"

# `make focus_systemverilog_preprocessor` — regenerate the svpp artifact so subsequent
# `cargo test --features generated_parsers` invocations exercise the systemverilog_preprocessor
# family (build.rs sets has_generated_systemverilog_preprocessor_parser when the artifact exists).
.PHONY: focus_systemverilog_preprocessor
focus_systemverilog_preprocessor:
	@$(FRESHNESS_GUARD) --chain $(SVPP_EBNF) $(SVPP_JSON) $(SVPP_PARSER) --tools $(RUST_EBNF_FRONTEND_BIN) $(RUST_AST_PIPELINE)
	@$(MAKE) $(SVPP_PARSER)
	@echo ""
	@echo "✅ systemverilog_preprocessor parser artifact ready: $(SVPP_PARSER)"
	@echo ""
	@echo "Default cargo test will pick this up automatically."
	@echo "  cd $(RUST_DIR) && cargo test --features generated_parsers"
	@echo ""

# Generate rtl_frontend raw-AST JSON from EBNF (GRAMMAR-WELLFORMED.H.6 — Phase H cert-coverage)
$(RTL_FRONTEND_JSON): $(RTL_FRONTEND_EBNF) $(RUST_EBNF_FRONTEND_BIN)
	@echo "Generating rtl_frontend JSON with debug output (Rust frontend)..."
	@mkdir -p $(GENERATED_DIR)
	$(RUST_FRONTEND_TO_JSON) $(RTL_FRONTEND_EBNF) --emit-raw-ast-json $(RTL_FRONTEND_JSON)

# Generate final rtl_frontend parser from JSON
$(RTL_FRONTEND_PARSER): $(RTL_FRONTEND_JSON) $(RUST_AST_PIPELINE)
	@echo "Generating final rtl_frontend parser from JSON..."
	@mkdir -p $(GENERATED_DIR)
	$(RUST_GENERATOR) $(RTL_FRONTEND_JSON) -o $(RTL_FRONTEND_PARSER)

# rtl_frontend parser complete flow
.PHONY: rtl_frontend_parser
rtl_frontend_parser: $(RTL_FRONTEND_JSON) $(RUST_AST_PIPELINE)
	@echo "🚀 Starting complete flow for rtl_frontend parser..."
	@echo "📄 Step 1: EBNF -> JSON via Rust frontend (already done: $(RTL_FRONTEND_JSON))"
	@echo "🔧 Step 2: JSON -> PARSER.RS (high-performance mode)"
	@mkdir -p $(GENERATED_DIR)
	$(RUST_GENERATOR) $(RTL_FRONTEND_JSON) -o $(RTL_FRONTEND_PARSER)
	@echo "✅ Generated: $(RTL_FRONTEND_PARSER)"
	@echo "✅ rtl_frontend parser generated successfully!"

# `make focus_rtl_frontend` — regenerate the rtl_frontend artifact so subsequent
# `cargo test --features generated_parsers` invocations exercise the rtl_frontend
# family (build.rs sets has_generated_rtl_frontend_parser when the artifact exists).
.PHONY: focus_rtl_frontend
focus_rtl_frontend:
	@$(FRESHNESS_GUARD) --chain $(RTL_FRONTEND_EBNF) $(RTL_FRONTEND_JSON) $(RTL_FRONTEND_PARSER) --tools $(RUST_EBNF_FRONTEND_BIN) $(RUST_AST_PIPELINE)
	@$(MAKE) $(RTL_FRONTEND_PARSER)
	@echo ""
	@echo "✅ rtl_frontend parser artifact ready: $(RTL_FRONTEND_PARSER)"
	@echo ""
	@echo "Default cargo test will pick this up automatically."
	@echo "  cd $(RUST_DIR) && cargo test --features generated_parsers"
	@echo ""

# Generate vhdl raw-AST JSON from EBNF (GRAMMAR-WELLFORMED.H.2 — Phase H cert-coverage)
$(VHDL_JSON): $(VHDL_EBNF) $(RUST_EBNF_FRONTEND_BIN)
	@echo "Generating vhdl JSON with debug output (Rust frontend)..."
	@mkdir -p $(GENERATED_DIR)
	$(RUST_FRONTEND_TO_JSON) $(VHDL_EBNF) --emit-raw-ast-json $(VHDL_JSON)

# Generate final vhdl parser from JSON
$(VHDL_PARSER): $(VHDL_JSON) $(RUST_AST_PIPELINE)
	@echo "Generating final vhdl parser from JSON..."
	@mkdir -p $(GENERATED_DIR)
	$(RUST_GENERATOR) $(VHDL_JSON) -o $(VHDL_PARSER)

# vhdl parser complete flow
.PHONY: vhdl_parser
vhdl_parser: $(VHDL_JSON) $(RUST_AST_PIPELINE)
	@echo "🚀 Starting complete flow for vhdl parser..."
	@echo "📄 Step 1: EBNF -> JSON via Rust frontend (already done: $(VHDL_JSON))"
	@echo "🔧 Step 2: JSON -> PARSER.RS (high-performance mode)"
	@mkdir -p $(GENERATED_DIR)
	$(RUST_GENERATOR) $(VHDL_JSON) -o $(VHDL_PARSER)
	@echo "✅ Generated: $(VHDL_PARSER)"
	@echo "✅ vhdl parser generated successfully!"

# `make focus_vhdl` — regenerate the vhdl artifact so subsequent
# `cargo test --features generated_parsers` invocations exercise the vhdl
# family (build.rs sets has_generated_vhdl_parser when the artifact exists).
.PHONY: focus_vhdl
focus_vhdl:
	@$(FRESHNESS_GUARD) --chain $(VHDL_EBNF) $(VHDL_JSON) $(VHDL_PARSER) --tools $(RUST_EBNF_FRONTEND_BIN) $(RUST_AST_PIPELINE)
	@$(MAKE) $(VHDL_PARSER)
	@echo ""
	@echo "✅ vhdl parser artifact ready: $(VHDL_PARSER)"
	@echo ""
	@echo "Default cargo test will pick this up automatically."
	@echo "  cd $(RUST_DIR) && cargo test --features generated_parsers"
	@echo ""

# Generate systemverilog JSON from EBNF (parallel to regex; depends on Rust frontend only)
$(SYSTEMVERILOG_JSON): $(SYSTEMVERILOG_EBNF) $(RUST_EBNF_FRONTEND_BIN)
	@echo "Generating systemverilog JSON from EBNF (Rust frontend)..."
	@mkdir -p $(GENERATED_DIR)
	$(RUST_FRONTEND_TO_JSON) $(SYSTEMVERILOG_EBNF) --emit-raw-ast-json $(SYSTEMVERILOG_JSON)

# Generate final systemverilog parser from JSON (parallel to regex_parser).
# The 47MB output and 30-60s codegen time is why this isn't tracked in git
# nor wired into the default `cargo test`. Use `make focus_systemverilog`
# to regen + drop a hint about how to opt-in to SV-gated tests.
$(SYSTEMVERILOG_PARSER): $(SYSTEMVERILOG_JSON) $(RUST_AST_PIPELINE)
	@echo "Generating systemverilog parser from JSON..."
	@mkdir -p $(GENERATED_DIR)
	$(RUST_GENERATOR) $(SYSTEMVERILOG_JSON) -o $(SYSTEMVERILOG_PARSER)

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
# OPS-MEMSAFE.3 (director directive 2026-07-19, incident 2026-07-18): this
# recipe DELETES every generated artifact AND runs `cargo clean` — which
# removes the ENTIRE rust/target/ tree (build caches, generated_logs, any
# preserved binaries; the incident cost 102.8 GiB incl. every preserved
# perf probe). It now REFUSES unless explicitly confirmed, so no alias,
# dep chain, script, or habit can trigger it accidentally. The guard is
# doctrine-enforced by scripts/check_destructive_target_guard.sh.
clean:
	@if [ "$$PGEN_CONFIRM_CLEAN" != "1" ]; then \
		echo "⛔ REFUSED: 'make clean' deletes ALL generated artifacts ($(GENERATED_DIR)/*) AND runs 'cargo clean'" >&2; \
		echo "   (the ENTIRE rust/target/ — build caches, generated_logs, preserved binaries)." >&2; \
		echo "   This destroyed 102.8 GiB incl. every preserved perf probe on 2026-07-18 (OPS-MEMSAFE.3)." >&2; \
		echo "   If you REALLY mean it:  PGEN_CONFIRM_CLEAN=1 make clean" >&2; \
		exit 96; \
	fi
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
	@echo "🔬 Focus on a specific parser family for cargo test (per-grammar default-parser switch):"
	@echo "  focus_regex                - Regen regex parser artifact + activate has_generated_regex_parser cfg"
	@echo "  focus_systemverilog        - Regen SV parser artifact + activate has_generated_systemverilog_parser cfg"
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
	@echo "  regenerate_generated_parsers - Cold-clone regeneration of every generated/ artifact (the SINGLE home of that recipe; used by .github/actions/regenerate-parsers and by ci_workflow_local_gate)"
	@echo "  clean        - Remove generated files"
	@echo "  clean-all    - Remove all build artifacts"
	@echo "  fixed_point_gate - Verify deterministic bootstrap artifacts across cycles"
	@echo "  branch_protection_contract_gate - Validate tracked branch-protection required-check contract vs workflows"
	@echo "  ci_workflow_local_gate - Run tracked GitHub workflow gate commands from a tracked-only local export"
	@echo "  bin_build_integrity_gate - Prove every declared binary compiles in every configuration this repo builds (heavy; run under the memory guard)"
	@echo "  mdbook_docs_gate - Build the live mdBook documentation surface"
	@echo "  regex_parser_book_gate - Build the standalone regex parser mdBook (RGX integration reference)"
	@echo "  systemverilog_parser_book_gate - Build the standalone SystemVerilog parser mdBook (Nexsim integration reference)"
	@echo "  clippy_on_rust_change - Run cargo clippy flow automatically when Rust/generated Rust files changed"
	@echo "  sota_exit_gate - Run release-grade aggregate gate (fixed-point, annotation, differential, performance, embedding, EBNF readiness report)"
	@echo "  sota_release_policy - Show active SOTA release policy file"
	@echo "  ebnf_frontend_readiness - Report EBNF frontend flow readiness for ebnf/json/regex grammars"
	@echo "  ebnf_frontend_gate - Strict EBNF frontend gate (fail on any grammar flow failure)"
	@echo "  ebnf_frontend_dual_run_diff - Report the frontend<->meta-parser RAW-AST ENVELOPE differential across all 14 diffable grammars"
	@echo "  ebnf_frontend_dual_run_gate - Strict frontend<->meta-parser envelope differential gate (self-hosting + per-grammar divergence ratchet)"
	@echo "  hdl_frontend_readiness - Report HDL frontend flow readiness for systemverilog/vhdl grammars"
	@echo "  hdl_frontend_gate - Strict HDL frontend readiness gate (fails on missing/failing HDL grammar flows)"
	@echo "  sv_preprocessor_quality_gate - Enforce deterministic closed-loop quality checks for systemverilog_preprocessor grammar"
	@echo "  sv_preprocessor_aggregate_contract_gate - Enforce repeatable aggregate-report contract checks for focused preprocessor evidence surfaces"
	@echo "  sv_preprocessor_reachability_closure_gate - Enforce zero-target/full-reachable-coverage preprocessor closure under checked-in policy"
	@echo "  sv_preprocessor_zero_plausible_gap_proof_gate - Prove the SV-preprocessor helper-only syntax-unreachable whitelist over the retained aggregate/reachability sidecars"
	@echo "  sv_preprocessor_formal_exhaustive_closure_gate - Compute the explicit SV-preprocessor exhaustive-closure proof surface status from syntax, aggregate, and reachability sidecars"
	@echo "  sv_preprocessor_curated_differential_gate - Run curated expected-artifact differential corpus and classify expected-vs-bug taxonomy deltas"
	@echo "  sv_preprocessor_template_differential_gate - Run dynamic template-based preprocessor oracle differential (fully offline)"
	@echo "  sv_stimuli_quality_gate - Enforce preprocess-first SystemVerilog closed-loop quality gate (coverage/gap/replay -> preprocess -> parse_full -> semantic baseline)"
	@echo "  sv_parser_aggregate_contract_gate - Enforce repeatable aggregate-report contract checks for focused SystemVerilog parser evidence surfaces"
	@echo "  sv_parser_family_status_gate - Compute machine-checkable SV-family status rationale and verify live-tracker alignment"
	@echo "  sv_parser_family_status_contract_gate - Enforce schema/consistency contract for the SV-family status sidecar"
	@echo "  sv_external_corpus_triage_gate - Run deterministic external SystemVerilog corpus preprocess/parse triage over tracked SV/UVM sources"
	@echo "  sv_formal_exhaustive_closure_gate - Compute the explicit SV exhaustive-closure proof surface status from family-status and external-corpus sidecars"
	@echo "  sv_cert_recognized_union_gate - Lock the recognized SV multi-config certificate-coverage UNION accounting basis (canonical + union UNKNOWN + residual, deterministic across seeds)"
	@echo "  rtl_const_expr_cert_gate - Lock the rtl_const_expr canonical certificate-coverage baseline (default entry, depth 32, 48/0 fully_certified, deterministic across seeds)"
	@echo "  verilog_2005_conformance_gate - Enforce the strict verilog_2005 (IEEE 1364-2005) profile conformance surface (corpus accept/reject matrix, 0-profile-orphan lint lock, profiled cert baseline)"
	@echo "  regex_parser_family_contract_gate - Enforce combined contract for the current regex family proof surfaces"
	@echo "  regex_broader_corpus_proof_gate - Run deterministic broader regex corpus proof over the checked-in regex stress corpus"
	@echo "  regex_pcre2_textsafe_corpus_gate - Normalize canonical PCRE2 testdata into a conservative text-safe slice and measure generated regex parser acceptance"
	@echo "  regex_pcre2_compile_oracle_gate - Normalize canonical PCRE2 compile-oracle cases from testinput2/testoutput2 and enforce tracked mismatch ceilings"
	@echo "  regex_formal_exhaustive_closure_gate - Compute the explicit regex exhaustive-closure proof surface status from the family sidecar"
	@echo "  regex_parser_family_status_gate - Compute machine-checkable regex family status rationale and verify live-tracker alignment"
	@echo "  regex_parser_family_status_contract_gate - Enforce schema/consistency contract for the regex family-status sidecar"
	@echo "  regex_combined_telemetry_contract_gate - Enforce aggregate regex-family telemetry consistency against regex family sidecars"
	@echo "  vhdl_parser_family_contract_gate - Enforce combined contract for the current VHDL family proof surfaces"
	@echo "  vhdl_formal_exhaustive_closure_gate - Compute the explicit VHDL exhaustive-closure proof surface status from family and external-corpus sidecars"
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
	@echo "  stimuli_cross_family_platform_gate - Validate bounded shared stimuli-platform replay across regex, VHDL, and SystemVerilog"
	@echo "  duality_hunt_gate - Enforce the pinned plain-config duality-break signature sets (regex + svpp, canonical + scaled lanes)"
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
	@echo "  sc01_contract_gate - Enforce SC-01 canonical-transform Tier-4 contract slices + differential taxonomy checks"
	@echo "  sc02_contract_gate - Enforce SC-02 raw literal sample-hint Tier-4 contract slices + differential taxonomy checks"
	@echo "  sc13_contract_gate - Enforce SC-13 profiles/runtime-scaffold Tier-4 contract slices + differential taxonomy checks"
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
	@echo "  rtl_frontend_generated_contract_gate - Validate curated rtl_frontend generated and handwritten contract samples"
	@echo "  generated_clippy_correctness_gate - Hold the generated-parser clippy::correctness floor at 0 (refuses if generated/ is absent)"
	@echo "  generated_clippy_correctness_policy - The no-cargo subset: artifact presence + pinned correctness-roster integrity"
	@echo "  regex_parser_integration_contract_gate - Validate regex parser integration contract (consumer-facing convenience API + diagnostics)"
	@echo "  regex_embedded_code_block_contract_gate - Validate regex embedded code-block structural contract over the checked-in synthetic corpus"
	@echo "  regex_corpus_bundle_contract_gate - Validate the tracked PCRE2-first regex corpus acquisition bundle contract"
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
	@echo "  1. Generate JSON files from EBNF grammars (Rust frontend)"
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
	@echo "Force generating regex JSON with debug output via Rust frontend (ignoring dependencies)..."
	@mkdir -p $(GENERATED_DIR)
	$(MAKE) $(RUST_EBNF_FRONTEND_BIN)
	$(RUST_FRONTEND_TO_JSON) $(REGEX_EBNF) --emit-raw-ast-json $(REGEX_JSON)

.PHONY: regex-parser-with-log
regex-parser-with-log:
	@echo "Generating regex parser with full debug logging..."
	@mkdir -p $(GENERATED_DIR) $(GENERATED_LOG_DIR)
	@rm -f $(REGEX_JSON) $(REGEX_PARSER)
	@echo "Step 1: Generate regex JSON via Rust frontend..." | tee $(GENERATED_LOG_DIR)/regex_parser.log
	$(MAKE) $(RUST_EBNF_FRONTEND_BIN) 2>&1 | tee -a $(GENERATED_LOG_DIR)/regex_parser.log
	$(RUST_FRONTEND_TO_JSON) $(REGEX_EBNF) --emit-raw-ast-json $(REGEX_JSON) 2>&1 | tee -a $(GENERATED_LOG_DIR)/regex_parser.log
	@echo "\nStep 2: Generate regex parser from JSON..." | tee -a $(GENERATED_LOG_DIR)/regex_parser.log
	$(RUST_GENERATOR) $(REGEX_JSON) -o $(REGEX_PARSER) 2>&1 | tee -a $(GENERATED_LOG_DIR)/regex_parser.log
	@echo "\nRegex parser generation complete! Check $(GENERATED_LOG_DIR)/regex_parser.log for details"

.PHONY: return-annotation-parser-with-log
return-annotation-parser-with-log:
	@echo "Generating return annotation parser with full debug logging..."
	@mkdir -p $(GENERATED_DIR) $(GENERATED_LOG_DIR)
	@rm -f $(RETURN_ANNOTATION_JSON) $(RETURN_ANNOTATION_PARSER)
	@echo "Step 1: Generate return annotation JSON via Rust frontend..." | tee $(GENERATED_LOG_DIR)/return_annotation_parser.log
	$(MAKE) $(RUST_EBNF_FRONTEND_BIN) 2>&1 | tee -a $(GENERATED_LOG_DIR)/return_annotation_parser.log
	$(RUST_FRONTEND_TO_JSON) $(RETURN_ANNOTATION_EBNF) --emit-raw-ast-json $(RETURN_ANNOTATION_JSON) 2>&1 | tee -a $(GENERATED_LOG_DIR)/return_annotation_parser.log
	@echo "\nStep 2: Generate return annotation parser from JSON..." | tee -a $(GENERATED_LOG_DIR)/return_annotation_parser.log
	$(RUST_GENERATOR_BOOTSTRAP) $(RETURN_ANNOTATION_JSON) -o $(RETURN_ANNOTATION_PARSER) 2>&1 | tee -a $(GENERATED_LOG_DIR)/return_annotation_parser.log
	@echo "\nReturn annotation parser generation complete! Check $(GENERATED_LOG_DIR)/return_annotation_parser.log for details"

.PHONY: semantic-annotation-parser-with-log
semantic-annotation-parser-with-log:
	@echo "Generating semantic annotation parser with full debug logging..."
	@mkdir -p $(GENERATED_DIR) $(GENERATED_LOG_DIR)
	@rm -f $(SEMANTIC_ANNOTATION_JSON) $(SEMANTIC_ANNOTATION_PARSER)
	@echo "Step 1: Generate semantic annotation JSON via Rust frontend..." | tee $(GENERATED_LOG_DIR)/semantic_annotation_parser.log
	$(MAKE) $(RUST_EBNF_FRONTEND_BIN) 2>&1 | tee -a $(GENERATED_LOG_DIR)/semantic_annotation_parser.log
	$(RUST_FRONTEND_TO_JSON) $(SEMANTIC_ANNOTATION_EBNF) --emit-raw-ast-json $(SEMANTIC_ANNOTATION_JSON) 2>&1 | tee -a $(GENERATED_LOG_DIR)/semantic_annotation_parser.log
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
# CI-PARITY-GATE-ROT.32 (d) — `--json-only`: this flow's recipe runs the generator unconditionally
# (a recipe line, not a file rule), so edge B does not exist here and removing the parser would only
# let $(RUST_AST_PIPELINE)'s placeholder logic drop a stub in its place. Edge A is real and MEASURED:
# the return_annotation generator step is 0.33 s, well inside the one-second window.
return_annotation_parser:
	@$(FRESHNESS_GUARD) --chain $(RETURN_ANNOTATION_EBNF) $(RETURN_ANNOTATION_JSON) $(RETURN_ANNOTATION_PARSER) --tools $(RUST_EBNF_FRONTEND_BIN) $(RUST_AST_PIPELINE_BOOTSTRAP) --json-only
	@$(MAKE) $(RETURN_ANNOTATION_JSON) $(RUST_AST_PIPELINE_BOOTSTRAP)
	@echo "🚀 Starting complete flow for return annotation parser..."
	@echo "📄 Step 1: EBNF -> JSON via Rust frontend (already done: $(RETURN_ANNOTATION_JSON))"
	@echo "🔧 Step 2: JSON -> PARSER.RS (bootstrap mode)"
	@mkdir -p $(GENERATED_DIR)
	$(RUST_GENERATOR_BOOTSTRAP) $(RETURN_ANNOTATION_JSON) -o $(RETURN_ANNOTATION_PARSER)
	@echo "✅ Generated: $(RETURN_ANNOTATION_PARSER)"
	@echo "✅ Return annotation parser generated successfully!"

# Semantic annotation parser complete flow
.PHONY: semantic_annotation_parser
# CI-PARITY-GATE-ROT.32 (d) — `--json-only`, for the reason given on return_annotation_parser above.
semantic_annotation_parser:
	@$(FRESHNESS_GUARD) --chain $(SEMANTIC_ANNOTATION_EBNF) $(SEMANTIC_ANNOTATION_JSON) $(SEMANTIC_ANNOTATION_PARSER) --tools $(RUST_EBNF_FRONTEND_BIN) $(RUST_AST_PIPELINE_BOOTSTRAP) --json-only
	@$(MAKE) $(SEMANTIC_ANNOTATION_JSON) $(RUST_AST_PIPELINE_BOOTSTRAP)
	@echo "🚀 Starting complete flow for semantic annotation parser..."
	@echo "📄 Step 1: EBNF -> JSON via Rust frontend (already done: $(SEMANTIC_ANNOTATION_JSON))"
	@echo "🔧 Step 2: JSON -> PARSER.RS (bootstrap mode)"
	@mkdir -p $(GENERATED_DIR)
	$(RUST_GENERATOR_BOOTSTRAP) $(SEMANTIC_ANNOTATION_JSON) -o $(SEMANTIC_ANNOTATION_PARSER)
	@echo "✅ Generated: $(SEMANTIC_ANNOTATION_PARSER)"
	@echo "✅ Semantic annotation parser generated successfully!"

# Regex parser complete flow
.PHONY: regex_parser
regex_parser: $(REGEX_JSON) $(RUST_AST_PIPELINE)
	@echo "🚀 Starting complete flow for regex parser..."
	@echo "📄 Step 1: EBNF -> JSON via Rust frontend (already done: $(REGEX_JSON))"
	@echo "🔧 Step 2: JSON -> PARSER.RS (high-performance mode)"
	@mkdir -p $(GENERATED_DIR)
	$(RUST_GENERATOR) $(REGEX_JSON) -o $(REGEX_PARSER)
	@echo "✅ Generated: $(REGEX_PARSER)"
	@echo "✅ Regex parser generated successfully!"

# `make focus_<grammar>` — equal first-class status for any parser family.
#
# build.rs already supports per-parser cfg activation: when
# `../generated/<grammar>_parser.rs` exists (or `PGEN_<GRAMMAR>_PARSER_PATH`
# is set), the matching `has_generated_<grammar>_parser` cfg fires and
# any test gated on that cfg compiles in.
#
# `focus_<grammar>` regenerates the artifact for the named grammar from
# the current grammar source so subsequent `cargo test --features
# generated_parsers` invocations exercise that family. The default
# `cargo test` still skips parsers whose artifact isn't on disk, so
# focusing one grammar doesn't slow iteration on other grammars.
#
# Currently wired: `focus_regex` (regex artifact tracked-in-git but this
# refreshes from grammar source) and `focus_systemverilog` (~3-5min on
# first run; the SV parser is ~47MB and not tracked in git).
.PHONY: focus_regex
focus_regex:
	@$(FRESHNESS_GUARD) --chain $(REGEX_EBNF) $(REGEX_JSON) $(REGEX_PARSER) --tools $(RUST_EBNF_FRONTEND_BIN) $(RUST_AST_PIPELINE)
	@$(MAKE) $(REGEX_PARSER)
	@echo ""
	@echo "✅ Regex parser artifact ready: $(REGEX_PARSER)"
	@echo ""
	@echo "Default cargo test will pick this up automatically."
	@echo "  cd $(RUST_DIR) && cargo test --features generated_parsers"
	@echo ""

.PHONY: focus_systemverilog
focus_systemverilog:
	@$(FRESHNESS_GUARD) --chain $(SYSTEMVERILOG_EBNF) $(SYSTEMVERILOG_JSON) $(SYSTEMVERILOG_PARSER) --tools $(RUST_EBNF_FRONTEND_BIN) $(RUST_AST_PIPELINE)
	@$(MAKE) $(SYSTEMVERILOG_PARSER)
	@echo ""
	@echo "✅ SystemVerilog parser artifact ready: $(SYSTEMVERILOG_PARSER)"
	@echo ""
	@echo "build.rs sets has_generated_systemverilog_parser cfg when the artifact exists."
	@echo "SV-gated tests now compile in:"
	@echo "  cd $(RUST_DIR) && cargo test --features generated_parsers"
	@echo ""
	@echo "Quick SV calibration check (rebuild parseability_probe with the SV"
	@echo "path so SV grammar registers):"
	@echo "  cd $(RUST_DIR) && cargo build --release --features generated_parsers --bin parseability_probe"
	@echo "  echo 'module m; endmodule' | $(RUST_DIR)/target/release/parseability_probe --parse-dump-ast-pretty systemverilog /dev/stdin"
	@echo ""
	@echo "Full SV slice gate (book gate + manifest cross-check):"
	@echo "  make systemverilog_parser_book_gate"
	@echo ""

# Regex parser FULL bootstrap — single-command path for cold clones.
#
# This is the entry point downstream consumers like RGX should use when
# their PGEN submodule has been freshly cloned (or `make clean-all`'d) and
# `generated/` is empty. It handles the chicken-and-egg between
# `$(RUST_EBNF_FRONTEND_BIN)` (which compiles `generated/ebnf.rs` into
# itself via the `ebnf_dual_run` feature) and the rest of the pipeline.
#
# Two-step strategy:
#   1. If `generated/ebnf.rs` is missing, seed it via the RUST frontend
#      (`ast_pipeline --emit-raw-ast-json` -> `generated/ebnf.json`) and the
#      bootstrap-mode AST pipeline binary. This works with no `generated/ebnf.rs`
#      present because the generated-parser cross-check is cfg-gated on
#      `has_generated_ebnf_parser` — that gate IS the chicken-and-egg breaker.
#      (This comment used to say "via the Perl frontend (tools/ebnf_to_json.pl)".
#      The recipe below has used the Rust frontend for a long time; the text was
#      stale, and LANG-CAPABILITY-AUDIT.10.6 retired the Perl frontend outright.)
#   2. Run the standard `regex_parser` target, which transitively pulls
#      in everything else through the normal Make dependency graph.
#
# Idempotent. Running it again with `generated/ebnf.rs` already in place
# skips the bootstrap seed and just runs the standard regen.
# Regex parser CLEAN bootstrap — wipe everything then rebuild from
# absolute scratch. Useful for downstream consumers like RGX when their
# subs/pgen tree has gotten into a weird intermediate state and they want
# a guaranteed-fresh rebuild. Combines `clean-all` + `regex_parser_bootstrap`.
#
# Sequence:
#   1. `make clean-all` — removes the entire generated/ directory and
#      runs `cargo clean` (drops all build artifacts under rust/target).
#   2. `make regex_parser_bootstrap` — full cold-clone bootstrap.
#
# After this target completes, generated/regex_parser.rs is fresh and
# RGX can `cargo build --features generated_parsers` against it. Both
# steps together typically take 4-6 minutes on a modern laptop because
# of the cargo full rebuild.
.PHONY: regex_parser_fresh
regex_parser_fresh:
	@echo "🧹 Wiping all generated artifacts and Rust build cache..."
	$(MAKE) clean-all
	@echo "🚀 Bootstrapping regex parser from absolute scratch..."
	$(MAKE) regex_parser_bootstrap
	@echo "✅ Fresh rebuild complete. generated/regex_parser.rs is ready."

.PHONY: regex_parser_bootstrap
regex_parser_bootstrap:
	@echo "🚀 Bootstrap regex parser from cold clone..."
	@mkdir -p $(GENERATED_DIR)
	@set -e; if [ ! -f $(GENERATED_DIR)/ebnf.rs ]; then \
		echo "🌱 Seeding generated/ebnf.rs (one-time bootstrap, Rust frontend)..."; \
		echo "  Step A: build ast_pipeline (compiles without generated/ebnf.rs"; \
		echo "          because the cross-check is gated on has_generated_ebnf_parser)..."; \
		cd $(RUST_DIR) && cargo build --features ebnf_dual_run --bin ast_pipeline; \
		echo "  Step B: Rust EBNF frontend (grammars/ebnf.ebnf -> generated/ebnf.json)"; \
		$(RUST_AST_PIPELINE) $(GRAMMARS_DIR)/ebnf.ebnf --emit-raw-ast-json $(GENERATED_DIR)/ebnf.json; \
		echo "  Step C: ast_pipeline JSON -> generated/ebnf.rs (bootstrap-mode seed:"; \
		echo "          the annotation pair does not exist yet, so annotations route"; \
		echo "          through the hand-rolled bootstrap surface BY DESIGN here —"; \
		echo "          without the flag the RGX-0078.5.i.1.t2 guard refuses, see PGEN-RGX-0090)"; \
		$(RUST_AST_PIPELINE) --generate-parser --bootstrap-mode --eliminate-left-recursion $(GENERATED_DIR)/ebnf.json -o $(GENERATED_DIR)/ebnf.rs; \
		echo "🌱 generated/ebnf.rs seeded."; \
		echo "  Rebuilding ast_pipeline now that has_generated_ebnf_parser is set..."; \
		cd $(RUST_DIR) && cargo build --features ebnf_dual_run --bin ast_pipeline; \
	else \
		echo "✓ generated/ebnf.rs already present — skipping seed step."; \
		echo "  (verifying it still compiles against these sources...)"; \
		( cd $(RUST_DIR) && cargo build --features ebnf_dual_run --bin ast_pipeline ) || { \
			echo ""; \
			echo "⚠️  generated/ebnf.rs exists but no longer compiles against the current sources."; \
			echo "    This is the STALE-ARTIFACT trap (PGEN-RGX-0090): the artifact was generated"; \
			echo "    by an OLDER pin and the Rust runtime types have since changed."; \
			echo "    Fix: remove the stale artifact and re-run the bootstrap to reseed:"; \
			echo "      rm $(GENERATED_DIR)/ebnf.rs   # (or: rm -rf $(GENERATED_DIR))"; \
			echo "      make -C rust regex_parser_bootstrap"; \
			exit 1; \
		}; \
	fi
	@echo "🔁 Step D: standard regex_parser chain..."
	$(MAKE) regex_parser
	@echo "✅ Bootstrap complete. generated/regex_parser.rs is ready for --features generated_parsers builds."

# PARSER-NEUTRALITY.1 (director ruling 2026-07-20): the former
# `regex_typed_differential_gate` and `regex_typed_perf_probe` targets — and
# the `--enable-parser-hooks` per-grammar hook mechanism they exercised — are
# REMOVED. The AST pipeline takes no per-parser inputs beyond the grammar
# itself; typed-output equivalence remains covered by the differential-
# equivalence, AST-shape, duality, and PCRE2 compile-oracle gates.
# See docs/decisions/feedback_no_parser_hooks_full_neutrality.md.

# Convenience aliases for easier typing
.PHONY: return_parser
return_parser: return_annotation_parser

.PHONY: semantic_parser
semantic_parser: semantic_annotation_parser

.PHONY: annotation_parsers
# RGX-0078.5.j.4 (-0141 ops rider) — this alias used to point at
# `return_semantic_parsers`, which hard-depends on `clean` (rm -f
# generated/* + `cargo clean`). Invoked as an innocent "regenerate the
# annotation pair" step it destroyed all 11 generated artifacts AND the
# whole cargo target dir (102.8 GiB incl. every preserved perf probe;
# 2026-07-18 session #155). The alias now routes to the two direct,
# NON-destructive per-parser flows; the full clean+rebuild semantic
# remains available explicitly as `return_semantic_parsers`.
annotation_parsers: return_annotation_parser semantic_annotation_parser

# CI-PARITY-GATE-ROT.4 — THE SINGLE HOME OF THE COLD-CLONE REGENERATION RECIPE.
#
# `generated/` is untracked (repository policy, `0ed2b2ad` 2026-04-29), so a fresh
# checkout — `actions/checkout` on a hosted runner, or the local parity gate's
# `copy_tracked_worktree` export, which are the SAME tracked-files-only shape — has no
# generated parsers at all. `rust/src/lib.rs` includes the two annotation parsers by
# LITERAL path with no `has_generated_*` cfg, so under `--features generated_parsers`
# their absence is a hard rustc error that takes the whole crate down rather than a
# disabled feature. Measured: 11 of the 15 tracked workflows cannot run without this.
#
# ⛔ This target exists so that sequence has EXACTLY ONE definition. Before it, the
# recipe was written out twice (in `.github/workflows/generated-clippy-correctness-gate.yml`
# and in `prepare_generated_artifacts` inside `rust/scripts/ci_workflow_local_gate.sh`),
# and wiring the other ten workflows would have made twelve copies of a sequence whose
# drift nothing could detect. Every consumer now calls this target:
#   - `.github/actions/regenerate-parsers/action.yml` (the composite action the hosted
#     workflows use), and
#   - `prepare_generated_artifacts` in the local workflow-parity gate.
# `audit_workflow_regeneration_surface` in that gate asserts the wiring still holds.
#
# Measured cost from a bare tracked tree: ~258 s (CI-PARITY-GATE-ROT.3 prepared census).
# ✅ QUIET SINCE CI-PARITY-GATE-ROT.31 (2026-08-14, director-ruled). It used to be verbose BY
# CONSTRUCTION — `RUST_GENERATOR` ran `--debug --trace`, and a full capture of this sequence
# measured 7.1 GB, re-measured at 6 894 576 244 B the day it was fixed. It is now 14 194 408 B, of
# which 14 189 570 B is one `ast_pipeline` cargo rebuild — the generator's own share is 4 838 B. The
# log therefore needs no bounding to be safe; the parity gate still bounds its capture, which is now
# a cheap belt-and-braces rather than the thing standing between a hosted runner and its disk.
# ⭐ To get the old stream back for an actual debugging run:
#     PGEN_TRACE_VERBOSITY=debug make -C rust SHELL=/bin/bash regenerate_generated_parsers
GENERATED_PARSER_FAMILIES = json regex systemverilog systemverilog_preprocessor vhdl rtl_const_expr rtl_frontend

.PHONY: regenerate_generated_parsers
regenerate_generated_parsers:
	@echo "🌱 Regenerating generated/ from a cold clone (CI-PARITY-GATE-ROT.4)..."
	$(MAKE) regex_parser_bootstrap
	$(MAKE) annotation_parsers
	@set -e; for grammar in $(GENERATED_PARSER_FAMILIES); do \
		$(MAKE) "focus_$$grammar"; \
	done
	@echo "✅ generated/ regenerated: annotation pair + $(words $(GENERATED_PARSER_FAMILIES)) grammar families."

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

# BIN-BUILD-INTEGRITY.2 — every declared binary must still COMPILE, in every
# configuration this repository actually builds. HEAVY (one cargo check per
# configuration): run it under the memory guard per the host-RAM directive.
.PHONY: bin_build_integrity_gate
bin_build_integrity_gate:
	@echo "🔩 Running binary build-integrity gate (every bin, every configuration)..."
	cd $(RUST_DIR) && ./scripts/bin_build_integrity_gate.sh
	@echo "✅ Binary build-integrity gate passed."

# CI-PARITY-GATE-ROT.2 — the per-parser book gates are DERIVED, not hand-listed. Every
# `rust/scripts/*_parser_book_gate.sh` becomes a target here, so a book added tomorrow is covered
# without anyone remembering to add it. The reachability inventory found all ten of them orphaned:
# nothing ran them, while README carries the standing directive that every PGEN parser ships its own
# live mdBook. Measured 0.3 s each (they build mdBook only — no cargo, no generated parsers).
PARSER_BOOK_GATES = $(patsubst $(RUST_DIR)/scripts/%.sh,%,$(wildcard $(RUST_DIR)/scripts/*_parser_book_gate.sh))

# ⛔ The book gates are PREREQUISITES, not a shell loop, and that is deliberate. A
# `for g in $(PARSER_BOOK_GATES); do $(MAKE) "$$g"; done` recipe runs them correctly but makes the
# edges INVISIBLE to any static reader — including the reachability inventory this leaf ships, which
# then still reported all ten orphaned. A gate that is invoked only in a way nothing can see is a
# weaker version of the very problem `.2` exists to solve.
.PHONY: parser_books_gate
parser_books_gate: $(PARSER_BOOK_GATES)
	@echo "✅ All $(words $(PARSER_BOOK_GATES)) per-parser book gates passed."

# CI-PARITY-GATE-ROT.2 — expose any variable's expanded value so a tool can ask MAKE what a
# derived list contains instead of re-implementing make's own semantics:
#   make -C rust --no-print-directory -s print-PARSER_BOOK_GATES
# Read-only; it echoes and does nothing else.
print-%:
	@echo '$($*)'

# ⭐ CI-PARITY-GATE-ROT.2 — `parser_books_gate` hangs off `mdbook_docs_gate` DELIBERATELY rather
# than getting its own workflow step. The platform book and the ten per-parser books are ONE
# documentation surface, `mdbook_docs_gate` is already invoked by a tracked workflow, by the commit
# workflow and by the local parity gate's replay, and the whole addition measures ~3 s. Wiring it
# this way means the existing invokers cover it with no new plumbing that could itself rot.
.PHONY: mdbook_docs_gate
mdbook_docs_gate: parser_books_gate
	@echo "📚 Running mdBook docs gate..."
	cd $(RUST_DIR) && ./scripts/mdbook_docs_gate.sh
	@echo "✅ mdBook docs gate passed."

.PHONY: regex_parser_book_gate
regex_parser_book_gate:
	@echo "📖 Running regex parser book gate..."
	cd $(RUST_DIR) && ./scripts/regex_parser_book_gate.sh
	@echo "✅ regex parser book gate passed."

.PHONY: systemverilog_parser_book_gate
systemverilog_parser_book_gate:
	@echo "📖 Running systemverilog parser book gate..."
	cd $(RUST_DIR) && ./scripts/systemverilog_parser_book_gate.sh
	@echo "✅ systemverilog parser book gate passed."

.PHONY: vhdl_parser_book_gate
vhdl_parser_book_gate:
	@echo "📖 Running vhdl parser book gate..."
	cd $(RUST_DIR) && ./scripts/vhdl_parser_book_gate.sh
	@echo "✅ vhdl parser book gate passed."

.PHONY: rtl_frontend_parser_book_gate
rtl_frontend_parser_book_gate:
	@echo "📖 Running rtl_frontend parser book gate..."
	cd $(RUST_DIR) && ./scripts/rtl_frontend_parser_book_gate.sh
	@echo "✅ rtl_frontend parser book gate passed."

.PHONY: rtl_const_expr_parser_book_gate
rtl_const_expr_parser_book_gate:
	@echo "📖 Running rtl_const_expr parser book gate..."
	cd $(RUST_DIR) && ./scripts/rtl_const_expr_parser_book_gate.sh
	@echo "✅ rtl_const_expr parser book gate passed."

.PHONY: systemverilog_preprocessor_parser_book_gate
systemverilog_preprocessor_parser_book_gate:
	@echo "📖 Running systemverilog_preprocessor parser book gate..."
	cd $(RUST_DIR) && ./scripts/systemverilog_preprocessor_parser_book_gate.sh
	@echo "✅ systemverilog_preprocessor parser book gate passed."

.PHONY: json_parser_book_gate
json_parser_book_gate:
	@echo "📖 Running json parser book gate..."
	cd $(RUST_DIR) && ./scripts/json_parser_book_gate.sh
	@echo "✅ json parser book gate passed."

.PHONY: return_annotation_parser_book_gate
return_annotation_parser_book_gate:
	@echo "📖 Running return_annotation parser book gate..."
	cd $(RUST_DIR) && ./scripts/return_annotation_parser_book_gate.sh
	@echo "✅ return_annotation parser book gate passed."

.PHONY: semantic_annotation_parser_book_gate
semantic_annotation_parser_book_gate:
	@echo "📖 Running semantic_annotation parser book gate..."
	cd $(RUST_DIR) && ./scripts/semantic_annotation_parser_book_gate.sh
	@echo "✅ semantic_annotation parser book gate passed."

.PHONY: ebnf_parser_book_gate
ebnf_parser_book_gate:
	@echo "📖 Running ebnf parser book gate..."
	cd $(RUST_DIR) && ./scripts/ebnf_parser_book_gate.sh
	@echo "✅ ebnf parser book gate passed."

.PHONY: clippy_on_rust_change
clippy_on_rust_change:
	@echo "🧹 Running clippy flow on Rust/generated Rust changes..."
	cd $(RUST_DIR) && ./scripts/clippy_on_rust_change.sh
	@echo "✅ clippy_on_rust_change completed."

.PHONY: silent_success_sentinel_gate
silent_success_sentinel_gate:
	@echo "🕵️  Running silent-success sentinel gate..."
	cd $(RUST_DIR) && ./scripts/silent_success_sentinel_gate.sh
	@echo "✅ silent-success sentinel gate passed."

# ⭐ DONE-BAR.5c — `silent_success_sentinel_gate` hangs off `sota_exit_gate` as a
# PREREQUISITE, not as a line inside the aggregate's shell script, so the edge is
# VISIBLE to any static reader (the CI-PARITY-GATE-ROT.2 lesson: a `for … $(MAKE)`
# loop makes the edges invisible to `check_gate_reachability.sh`). It is measured at
# ~40 s against a 5 h aggregate, so it costs nothing meaningful and runs FIRST —
# a silent-success path invalidates every "did it parse?" verdict downstream.
.PHONY: sota_exit_gate
sota_exit_gate: silent_success_sentinel_gate
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

.PHONY: sv_preprocessor_zero_plausible_gap_proof_gate
sv_preprocessor_zero_plausible_gap_proof_gate:
	@echo "🧪 Running SystemVerilog preprocessor zero-plausible-gap proof gate..."
	cd $(RUST_DIR) && ./scripts/sv_preprocessor_zero_plausible_gap_proof_gate.sh
	@echo "✅ SystemVerilog preprocessor zero-plausible-gap proof gate completed."

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

.PHONY: regex_formal_exhaustive_closure_gate
regex_formal_exhaustive_closure_gate:
	@echo "🧪 Running regex formal exhaustive closure gate..."
	cd $(RUST_DIR) && ./scripts/regex_formal_exhaustive_closure_gate.sh
	@echo "✅ Regex formal exhaustive closure gate passed."

.PHONY: regex_broader_corpus_proof_gate
regex_broader_corpus_proof_gate:
	@echo "🧪 Running regex broader corpus proof gate..."
	cd $(RUST_DIR) && ./scripts/regex_broader_corpus_proof_gate.sh
	@echo "✅ Regex broader corpus proof gate passed."

.PHONY: regex_pcre2_textsafe_corpus_gate
regex_pcre2_textsafe_corpus_gate:
	@echo "🧪 Running regex PCRE2 text-safe corpus gate..."
	cd $(RUST_DIR) && ./scripts/regex_pcre2_textsafe_corpus_gate.sh
	@echo "✅ Regex PCRE2 text-safe corpus gate passed."

.PHONY: regex_pcre2_compile_oracle_gate
regex_pcre2_compile_oracle_gate:
	@echo "🧪 Running regex PCRE2 compile oracle gate..."
	cd $(RUST_DIR) && ./scripts/regex_pcre2_compile_oracle_gate.sh
	@echo "✅ Regex PCRE2 compile oracle gate passed."

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

.PHONY: sv_formal_exhaustive_closure_gate
sv_formal_exhaustive_closure_gate:
	@echo "🧪 Running SV formal exhaustive closure gate..."
	cd $(RUST_DIR) && ./scripts/sv_formal_exhaustive_closure_gate.sh
	@echo "✅ SV formal exhaustive closure gate passed."

.PHONY: sv_cert_recognized_union_gate
sv_cert_recognized_union_gate:
	@echo "🧪 Running SV recognized certificate-coverage union gate..."
	cd $(RUST_DIR) && ./scripts/sv_cert_recognized_union_gate.sh
	@echo "✅ SV recognized certificate-coverage union gate passed."

.PHONY: rtl_const_expr_cert_gate
rtl_const_expr_cert_gate:
	@echo "🧪 Running rtl_const_expr canonical certificate-coverage gate..."
	cd $(RUST_DIR) && ./scripts/rtl_const_expr_cert_gate.sh
	@echo "✅ rtl_const_expr canonical certificate-coverage gate passed."

# GENERATED-LINT-CORRECTNESS.3 — hold the generated-parser correctness-lint floor at 0.
# Requires the generated/ artifacts on disk (they are untracked; regenerate with focus_<grammar>);
# REFUSES with exit 2 rather than passing vacuously when they are absent.
.PHONY: generated_clippy_correctness_gate
generated_clippy_correctness_gate:
	@echo "🧪 Running generated-parser clippy correctness gate..."
	cd $(RUST_DIR) && ./scripts/generated_clippy_correctness_gate.sh
	@echo "✅ generated-parser clippy correctness gate passed."

# The no-cargo subset: artifact presence + pinned clippy::correctness roster integrity.
.PHONY: generated_clippy_correctness_policy
generated_clippy_correctness_policy:
	cd $(RUST_DIR) && ./scripts/generated_clippy_correctness_gate.sh --policy-only

.PHONY: verilog_2005_conformance_gate
verilog_2005_conformance_gate:
	@echo "🧪 Running verilog_2005 (IEEE 1364-2005) conformance gate..."
	cd $(RUST_DIR) && ./scripts/verilog_2005_conformance_gate.sh
	@echo "✅ verilog_2005 conformance gate passed."

.PHONY: sv_preprocessor_formal_exhaustive_closure_gate
sv_preprocessor_formal_exhaustive_closure_gate:
	@echo "🧪 Running SV preprocessor formal exhaustive closure gate..."
	cd $(RUST_DIR) && ./scripts/sv_preprocessor_formal_exhaustive_closure_gate.sh
	@echo "✅ SV preprocessor formal exhaustive closure gate passed."

.PHONY: vhdl_parser_family_contract_gate
vhdl_parser_family_contract_gate:
	@echo "🧪 Running VHDL parser-family contract gate..."
	cd $(RUST_DIR) && ./scripts/vhdl_parser_family_contract_gate.sh
	@echo "✅ VHDL parser-family contract gate passed."

.PHONY: vhdl_formal_exhaustive_closure_gate
vhdl_formal_exhaustive_closure_gate:
	@echo "🧪 Running VHDL formal exhaustive closure gate..."
	cd $(RUST_DIR) && ./scripts/vhdl_formal_exhaustive_closure_gate.sh
	@echo "✅ VHDL formal exhaustive closure gate passed."

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
	@$(MAKE) -C $(RUST_DIR) sc01_contract_gate
	@$(MAKE) -C $(RUST_DIR) sc02_contract_gate
	@$(MAKE) -C $(RUST_DIR) sc13_contract_gate
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

.PHONY: sc01_contract_gate
sc01_contract_gate:
	@echo "🧪 Running SC-01 canonical-transform contract gate..."
	cd $(RUST_DIR) && ./scripts/sc01_contract_gate.sh
	@echo "✅ SC-01 canonical-transform contract gate passed."

.PHONY: sc02_contract_gate
sc02_contract_gate:
	@echo "🧪 Running SC-02 raw literal sample-hint contract gate..."
	cd $(RUST_DIR) && ./scripts/sc02_contract_gate.sh
	@echo "✅ SC-02 raw literal sample-hint contract gate passed."

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

.PHONY: sc13_contract_gate
sc13_contract_gate:
	@echo "🧪 Running SC-13 profiles/runtime-scaffold contract gate..."
	cd $(RUST_DIR) && ./scripts/sc13_contract_gate.sh
	@echo "✅ SC-13 profiles/runtime-scaffold contract gate passed."

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

.PHONY: stimuli_cross_family_platform_gate
stimuli_cross_family_platform_gate:
	@echo "🧪 Running stimuli cross-family platform gate..."
	cd $(RUST_DIR) && ./scripts/stimuli_cross_family_platform_gate.sh
	@echo "✅ Stimuli cross-family platform gate passed."

.PHONY: duality_hunt_gate
duality_hunt_gate:
	@echo "🎯 Running plain-config duality-hunt gate (STIMULI-SIGNOFF.13.3)..."
	cd $(RUST_DIR) && ./scripts/duality_hunt_gate.sh
	@echo "✅ Duality-hunt gate passed."

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

.PHONY: ast_shape_contract_gate
ast_shape_contract_gate:
	@echo "🧬 Running per-parser-family AST-shape contract gate..."
	cd $(RUST_DIR) && cargo test --features generated_parsers --lib ast_shape_contract --quiet
	@echo "✅ AST-shape contract gate passed."

.PHONY: parse_harness_equivalence_gate
parse_harness_equivalence_gate:
	@echo "🪞 Running parse-harness differential-equivalence gate (PARSE-HARNESS.5)..."
	@echo "   interpreter (approach 1) == generated parser, byte-identical over the deterministic"
	@echo "   stimuli corpus (seeds 0/7/42); CERTIFIED byte-identical + DEFERRED ratchet + completeness."
	cd $(RUST_DIR) && cargo test --features "generated_parsers ebnf_dual_run" --lib parse_harness_equivalence::gate -- --nocapture
	@echo "✅ Parse-harness differential-equivalence gate passed."

.PHONY: parse_harness_combinator_gate
parse_harness_combinator_gate:
	@echo "🧩 Running parse-harness structural combinator suite (PARSE-HARNESS.6.1)..."
	@echo "   interpreter (approach 1) == compile-and-run oracle (approach 3), byte-identical over a"
	@echo "   per-combinator isolating-grammar suite (choice/branch_policy, quantifiers, lookahead,"
	@echo "   sequence, atoms, rule-ref, LR); includes the A2.2/A2.3 branch_policy discrimination proof."
	cd $(RUST_DIR) && cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline
	cd $(RUST_DIR) && cargo test --features "generated_parsers ebnf_dual_run" --lib parse_harness_combinator_suite::gate -- --nocapture
	@echo "✅ Parse-harness structural combinator suite passed."

.PHONY: parse_harness_semantic_gate
parse_harness_semantic_gate:
	@echo "🧿 Running parse-harness semantic-directive orchestration suite (PARSE-HARNESS.6.2)..."
	@echo "   interpreter (approach 1) == compile-and-run oracle (approach 3), byte-identical over a"
	@echo "   per-construct isolating-grammar suite for the store-gated-outcome surface (pre/branch/post"
	@echo "   @predicate gates, @emit_fact + the query vocabulary, scope tree, C3-B rollback, \$$ref"
	@echo "   resolution, branch-start inline actions, library no-op parity, memoization x store)."
	cd $(RUST_DIR) && cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline
	cd $(RUST_DIR) && cargo test --features "generated_parsers ebnf_dual_run" --lib parse_harness_semantic_suite::gate -- --nocapture
	@echo "✅ Parse-harness semantic-directive orchestration suite passed."

.PHONY: regex_ast_shape_contract_gate
regex_ast_shape_contract_gate:
	@echo "🧬 Running regex AST-shape contract gate..."
	cd $(RUST_DIR) && cargo test --features generated_parsers --lib ast_shape_contract::tests::regex_ast_shape_contract_holds_against_running_generated_parser -- --nocapture
	@echo "✅ Regex AST-shape contract gate passed."

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
	@$(MAKE) -C $(RUST_DIR) regex_parser_integration_contract_gate
	@echo "✅ Embedding API gate passed."

.PHONY: nexsim_parser_embedding_contract_gate
nexsim_parser_embedding_contract_gate:
	@echo "🧪 Running Nexsim parser embedding profile contract gate..."
	cd $(RUST_DIR) && ./scripts/nexsim_parser_embedding_contract_gate.sh
	@echo "✅ Nexsim parser embedding profile contract gate passed."

.PHONY: rtl_frontend_generated_contract_gate
rtl_frontend_generated_contract_gate:
	@echo "🧪 Running rtl_frontend generated contract gate..."
	cd $(RUST_DIR) && ./scripts/rtl_frontend_generated_contract_gate.sh
	@echo "✅ rtl_frontend generated contract gate passed."

.PHONY: regex_parser_integration_contract_gate
regex_parser_integration_contract_gate:
	@echo "🧪 Running regex parser integration contract gate..."
	cd $(RUST_DIR) && ./scripts/regex_parser_integration_contract_gate.sh
	@echo "✅ Regex parser integration contract gate passed."

.PHONY: regex_embedded_code_block_contract_gate
regex_embedded_code_block_contract_gate:
	@echo "🧪 Running regex embedded code-block contract gate..."
	cd $(RUST_DIR) && ./scripts/regex_embedded_code_block_contract_gate.sh
	@echo "✅ Regex embedded code-block contract gate passed."

.PHONY: regex_corpus_bundle_contract_gate
regex_corpus_bundle_contract_gate:
	@echo "🧪 Running regex corpus bundle contract gate..."
	cd $(RUST_DIR) && ./scripts/regex_corpus_bundle_contract_gate.sh
	@echo "✅ Regex corpus bundle contract gate passed."

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

# ---------------------------------------------------------------------------
# PGEN-RGX-0078 vendored compile-time perf gate (rust/perf/ — see VENDORED.md):
# PGEN regex parse p50 vs PCRE2 full-compile p50 on the frozen 8-pattern
# corpus; closure criterion = geomean ratio < 5x (the RGX ROADMAP target).
# Requires system libpcre2-8 (brew install pcre2) for the pcre2 crate + the
# optional C cross-check baselines.
# ---------------------------------------------------------------------------
PCRE2_PREFIX ?= $(shell brew --prefix pcre2 2>/dev/null || echo /usr/local)

.PHONY: regex_pcre2_compile_perf_gate
regex_pcre2_compile_perf_gate:
	@echo "🧱 Running regex compile-time perf gate (PGEN parse vs PCRE2 compile)..."
	cd $(RUST_DIR) && cargo run --release \
	    --features generated_parsers \
	    --example pgen_pcre2_compile_ratio
	@echo "✅ regex compile-time perf gate completed."

.PHONY: regex_pcre2_compile_perf_gate_cross_check
regex_pcre2_compile_perf_gate_cross_check:
	@echo "🧪 Building + running C-side PCRE2 baselines (cross-check)..."
	cc -O2 -I$(PCRE2_PREFIX)/include -L$(PCRE2_PREFIX)/lib \
	    -o $(RUST_DIR)/target/pcre2_compile_baseline \
	    $(RUST_DIR)/perf/pcre2_compile_baseline.c -lpcre2-8
	cc -O2 -I$(PCRE2_PREFIX)/include -L$(PCRE2_PREFIX)/lib \
	    -o $(RUST_DIR)/target/pcre2_compile_jit_baseline \
	    $(RUST_DIR)/perf/pcre2_compile_jit_baseline.c -lpcre2-8
	@echo "=== PCRE2 compile (no JIT) ==="
	@$(RUST_DIR)/target/pcre2_compile_baseline
	@echo "=== PCRE2 compile + JIT ==="
	@$(RUST_DIR)/target/pcre2_compile_jit_baseline

.PHONY: regex_pcre2_compile_perf_gate_full
regex_pcre2_compile_perf_gate_full: regex_pcre2_compile_perf_gate regex_pcre2_compile_perf_gate_cross_check

# ── ENGINE-UNIVERSAL-SERVICES.20 acceptance (d) — the SV parse-cost ratchet ─────────────────
#
# ⛔ WHY THIS TARGET EXISTS. `.17` slice 9's guarded left-recursion admission cost +24.3 %
# parse time and EVERY gate in the repository stayed green across it, because nothing measured
# parse cost at all. This is the re-measure entry point: it re-runs the pinned sample and
# refuses if any binding counter ROSE.
#
# The doctrine enforcer (`scripts/check_parse_cost_ratchet.sh`, registered in
# scripts/check_doctrines.sh) runs the cheap IDENTITY tier on every commit; this target runs
# the full ratchet (~2.5 min) and is what the enforcer tells you to run when the parser moves.
.PHONY: sv_parse_cost_ratchet
sv_parse_cost_ratchet:
	@echo "⏱  Re-measuring the SV parse-cost ratchet (pinned sample, ~2.5 min)..."
	cd $(RUST_DIR)/.. && PGEN_PARSE_COST_REMEASURE=1 bash scripts/check_parse_cost_ratchet.sh
	@echo "✅ parse-cost ratchet holds."

# Promote a NEW baseline — deliberate, never automatic. An improvement only tightens the
# ratchet once somebody says so; a silent auto-promote would let a regression ride in behind
# an unrelated improvement in the same run.
.PHONY: sv_parse_cost_rebaseline
sv_parse_cost_rebaseline:
	@echo "⏱  Re-measuring and PROMOTING a new SV parse-cost baseline..."
	cd $(RUST_DIR)/.. && PGEN_PARSE_COST_REBASELINE=1 bash scripts/check_parse_cost_ratchet.sh
	@echo "✅ parse-cost baseline promoted — review the diff before committing."
