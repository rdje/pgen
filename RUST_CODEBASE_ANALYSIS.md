# RUST_CODEBASE_ANALYSIS.md

Last updated: 2026-03-25

## Purpose
Live architecture and state assessment for the Rust codebase.

This document exists to preserve the high-level understanding needed to steer implementation, review future changes, and ramp up a new session without having to rediscover the whole Rust stack from scratch.

This is a live document, not an archival write-up. It should be amended whenever the Rust architecture, major risks, public integration surfaces, or codebase shape materially change.

## Session-Start Maintenance Rule
- Review this file at the start of any new session that materially touches the Rust codebase.
- Refresh it when the current codebase no longer matches this snapshot in a meaningful way.
- Prefer amending this file over scattering duplicate architectural assessments into ad hoc chat history.
- Keep historical detail in `CHANGES.md` / `DEVELOPMENT_NOTES.md`; keep this file focused on current structure, current risks, and the current steering picture.

## Scope Of This Assessment
- This is a source-structure and architecture assessment of the maintained Rust-first platform.
- It covers the main Rust crate, the generated-parser integration layer, the major Rust-owned binaries, and the Rust-owned gate/build ecosystem around them.
- It is not a claim that every parser family is closed.
- It is not a replacement for the live closure tracker in `LIVE_ACHIEVEMENT_STATUS.md`.

## Executive Summary
- PGEN's Rust codebase is not just a parser implementation. It is a parser-generation and parser-proof platform.
- The center of gravity is the AST pipeline in `rust/src/ast_pipeline/`, especially:
  - `mod.rs`
  - `ast_based_generator.rs`
  - `stimuli_generator.rs`
  - `annotation_validator.rs`
  - `semantic_runtime.rs`
- The generated parser path, stimuli/coverage closure path, semantic-steering path, and proof/gate path are deeply integrated rather than loosely bolted together.
- The strongest quality of the Rust codebase is coherence around determinism, observability, and machine-checkable proof.
- The main architectural risk is concentration of complexity in a few very large modules and a few repeated adapter seams.

## Snapshot Metrics
- Rust maintained source surface inspected in this pass: about `44k` lines.
- Biggest source hotspots:
  - `rust/src/ast_pipeline/stimuli_generator.rs`: `7907` lines
  - `rust/src/ast_pipeline/ast_based_generator.rs`: `7046` lines
  - `rust/src/ast_pipeline/annotation_validator.rs`: `4014` lines
  - `rust/src/main.rs`: `3183` lines
  - `rust/src/ast_pipeline/mod.rs`: `2920` lines
  - `rust/src/ast_pipeline/semantic_runtime.rs`: `2522` lines
  - `rust/src/ast_pipeline/unified_return_ast.rs`: `2625` lines
- Rust-owned shell gate scripts under `rust/scripts/`: `58`

## What The Rust Codebase Actually Is
- A grammar-to-parser pipeline:
  - `grammars/*.ebnf -> raw AST / JSON -> generated/*.rs`
- A parser generator:
  - AST-based Rust parser emission via `syn`, `quote`, and `prettyplease`
- A typed annotation platform:
  - return annotations
  - semantic annotations
  - validation and runtime-steering layers
- A stimuli-generation and coverage-closure platform:
  - in-memory stimuli generation
  - gap reporting
  - target planning
  - replay/closure-oriented telemetry
- A public integration surface:
  - parser registry
  - embedding API
  - grammar-profile-aware parse entrypoints
- A proof/gate system:
  - build orchestration
  - closure/status/contract sidecars
  - release/SOTA aggregate gates

## Major Architectural Layers

### 1. Core AST Pipeline And Grammar Normalization
Primary files:
- `rust/src/ast_pipeline/mod.rs`
- `rust/src/ast_pipeline/return_annotation_handler.rs`
- `rust/src/ast_pipeline/grouped_quantifier_parser.rs`
- `rust/src/ast_pipeline/mutual_recursion_handler.rs`
- `rust/src/ast_pipeline/ast_return_transform.rs`

Role:
- Defines the central IR for grammar transformation and parse-tree handling.
- Normalizes raw AST into the grammar tree used downstream.
- Handles branch/rule annotations.
- Performs left-recursion elimination and related normalization work.
- Provides shared runtime types:
  - parse node/content types
  - recursion/memoization machinery
  - trace/logging support

Assessment:
- This is the real heart of the crate.
- A lot of project doctrine is encoded here, not just in docs.
- It is powerful, but `mod.rs` itself is large enough that understanding the full transform pipeline now requires careful re-reading.

### 2. Parser Code Generation
Primary files:
- `rust/src/ast_pipeline/ast_based_generator.rs`
- `rust/src/ast_pipeline/ast_code_generator.rs`
- `rust/src/ast_pipeline/ast_generator_direct.rs`

Role:
- Turns normalized grammar AST into generated Rust parser source.
- Uses AST/token generation instead of raw string concatenation.
- Emits parser implementations with:
  - memoization and recursion guards
  - recovery telemetry
  - coverage-target telemetry
  - negative-case telemetry
  - deterministic-partition telemetry
  - semantic-runtime hooks

Assessment:
- The project is not generating “simple recognizers”; it is generating instrumented parsing systems.
- `ast_based_generator.rs` is one of the most important files in the repo.
- The AST-based codegen approach is a real strength because it reduces syntax-generation fragility.
- The downside is that too much emitted-parser policy is encoded in one giant generator module.

### 3. Stimuli, Coverage, Debt, And Closure Planning
Primary file:
- `rust/src/ast_pipeline/stimuli_generator.rs`

Role:
- Generates stimuli from grammar AST.
- Tracks coverage metrics across rules and branches.
- Computes reachable vs unreachable debt.
- Builds target plans for closure work.
- Supports recovery-biased and negative-ish generation modes.
- Integrates semantic steering into generation decisions.

Assessment:
- This is not a side tool; it is a second core engine beside the parser generator.
- It explains why PGEN should be thought of as a parser-proof platform, not only a parser generator.
- The module is very capable, but at nearly eight thousand lines it is a major maintainability hotspot.

### 4. Typed Annotation Model, Validation, And Runtime Semantics
Primary files:
- `rust/src/ast_pipeline/unified_return_ast.rs`
- `rust/src/ast_pipeline/unified_semantic_ast.rs`
- `rust/src/ast_pipeline/annotation_validator.rs`
- `rust/src/ast_pipeline/semantic_directive_registry.rs`
- `rust/src/ast_pipeline/semantic_runtime.rs`
- `rust/src/ast_pipeline/semantic_transform.rs`

Role:
- Parses and normalizes typed return/semantic annotation payloads.
- Validates annotation contracts against grammar structure.
- Defines semantic directive parsing and capability rules.
- Compiles semantic runtime annotations and executes them transactionally during parse.

Assessment:
- Return-annotation support appears deeper and more mature than the typed semantic AST layer.
- Semantic support is still substantial, but more directive-oriented and more spread across registry/runtime/validator seams.
- The validator is far beyond a “lint” layer; it is grammar-aware and contract-bearing.
- The semantic runtime is a meaningful subsystem in its own right.

### 5. Grammar-Specific Subsystems
Primary files:
- `rust/src/ebnf_frontend.rs`
- `rust/src/sv_preprocessor.rs`

Role:
- `ebnf_frontend.rs` provides a Rust-native `.ebnf -> raw_ast` frontend path.
- `sv_preprocessor.rs` implements a policyful SystemVerilog preprocessing stage with:
  - macro handling
  - include resolution
  - conditional compilation
  - diagnostics
  - source maps
  - event logging

Assessment:
- The Rust EBNF frontend is a real parser/tokenizer subsystem, not a small adapter.
- The SV preprocessor is substantial enough to deserve treatment as its own engine.
- The SV preprocessor’s explicit policies and observability surfaces are a strength.

### 6. Public Consumer Surfaces
Primary files:
- `rust/src/parser_registry.rs`
- `rust/src/embedding_api.rs`
- `rust/src/lib.rs`

Role:
- `parser_registry.rs` centralizes grammar-name dispatch across generated/bootstrap/profile-aware parsers.
- `embedding_api.rs` exposes a stable, versioned consumer contract with limits, result shapes, and AST-dump modes.
- `lib.rs` controls feature-gated exposure of the major subsystems.

Assessment:
- The embedding API is one of the cleaner and more disciplined pieces of the codebase.
- The registry layer is intentionally small and useful.
- There is still some repeated grammar/backend/profile branching across registry/API/binaries that could be unified further.

### 7. CLI, Build, And Operational Proof Layer
Primary files:
- `rust/src/main.rs`
- `rust/build.rs`
- `rust/Makefile`
- `rust/src/bin/*.rs`
- `rust/scripts/*.sh`

Role:
- `main.rs` is the large orchestration CLI for the core pipeline modes.
- `build.rs` resolves generated parser include paths at build time and emits `cfg` flags for available grammars.
- `rust/Makefile` coordinates bootstrap vs normal parser-generation flows.
- `rust/scripts/*.sh` provide the proof/gate ecosystem used for closure and release-grade validation.

Assessment:
- The build/gate layer is a major part of the product, not an afterthought.
- `build.rs` is strategically important because it lets the crate tolerate optional/generated parser availability.
- `main.rs` is functionally rich but overly large.
- The shell-gate surface is now big enough that architecture comprehension requires understanding both Rust and shell proof plumbing together.

## End-To-End Artifact Spine
1. Grammar/source input
   - Typical starting artifacts:
     - `grammars/*.ebnf`
     - generated grammar JSON inputs
     - real parser input samples
     - raw SystemVerilog source for preprocessing mode
2. Frontend / ingestion layer
   - `rust/src/ebnf_frontend.rs` can produce raw-AST envelopes directly from `.ebnf`
   - older or compatibility flows may still enter from precomputed JSON instead of live `.ebnf`
   - SystemVerilog preprocessing can branch here and emit expanded source plus source-map/diagnostic metadata before parsing
3. Normalization / transformation layer
   - `RustASTPipeline` in `rust/src/ast_pipeline/mod.rs` turns raw AST into the normalized grammar tree used for downstream generation
   - Important intermediate artifacts:
     - transformed / generation-input AST JSON
     - annotation metadata
     - normalization statistics
4. Generation layer
   - `ast_based_generator.rs` turns normalized grammar AST into generated Rust parser source
   - `stimuli_generator.rs` turns normalized grammar AST into:
     - in-memory stimuli
     - stimuli modules
     - coverage JSON
     - parseability reports
     - gap reports
     - target-drive telemetry
5. Runtime / consumer layer
   - Generated parser source becomes runtime parser modules through `build.rs` + `lib.rs`
   - Those runtime surfaces are then consumed by:
     - `parser_registry.rs`
     - `embedding_api.rs`
     - `parseability_probe`
     - `test_runner`
     - `perf_bench`
     - grammar-specific operational binaries
   - Parser-backed AST dumps can reappear here as a second artifact family, distinct from generation-input AST dumps
6. Proof / release layer
   - `rust/scripts/*.sh` collect upstream artifacts and emit machine-readable sidecars such as:
     - `summary.txt`
     - `summary.json`
     - `summary.csv`
   - Higher-level status / contract / combined-telemetry / SOTA gates then aggregate those sidecars into the project’s executable proof surface

Operational reading rule:
- Many bugs show up one stage later than where they originate.
- If a proof gate or parser runtime looks wrong, first identify which artifact family is wrong:
  - raw/frontend AST
  - normalized generation-input AST
  - generated parser source
  - runtime parser output
  - stimuli/coverage telemetry
  - proof sidecar summaries

## Main Rust Executables And Roles
- `ast_pipeline` / `ast_pipeline_bootstrap`
  - Both are wired to `rust/src/main.rs` via Cargo features.
  - This is the main orchestration CLI for:
    - AST transformation
    - parser generation
    - stimuli generation
    - stimuli-module generation
    - generation-input AST dumps
    - SystemVerilog preprocessing
  - If a task sounds like “run the Rust pipeline on a grammar or source file,” this is usually the first executable to inspect.
- `test_runner`
  - The main round-trip and suite-running harness for bootstrap/generated parser validation.
  - Important when the task is test-suite behavior, normalization in tests, or parser-family regression coverage.
- `parseability_probe`
  - The compact machine-facing probe for “does this grammar/profile parse this input?” and “dump the AST for this parse.”
  - This is one of the cleanest executable surfaces for external parseability contracts and AST-dump behavior.
- `ebnf_dual_run_diff`
  - A specialist diagnostic tool for the generated EBNF parser path.
  - It compares `parse` vs `parse_full` behavior and emits structured diagnostics for unconsumed tails and frontend drift.
- `perf_bench`
  - Benchmark/threshold executable for bootstrap-vs-generated parser throughput and latency.
  - Relevant when performance changes need proof, not just anecdotal timing.
- `pgen_ast`
  - A focused AST-based codegen CLI that reads transformed AST JSON and emits parser source.
  - It is narrower than `ast_pipeline`, but still useful for direct generator work or compatibility testing around AST-based emission.
- `return_annotation_generated_audit`
  - A small audit executable for generated return-annotation typed-AST serialization over sample lists.
  - Useful as a niche contract checker, not as a primary day-to-day workflow surface.
- `pgen`
  - An older parser smoke-test CLI for semantic/return/regex parser inputs with log-file output.
  - It is not the main modern operational surface, but it still exists and should be treated as a legacy-adjacent utility rather than deleted-by-assumption.

Assessment:
- Not every Rust executable here is equally strategic.
- The practical “primary” binaries are:
  - `ast_pipeline` / `ast_pipeline_bootstrap`
  - `test_runner`
  - `parseability_probe`
  - `ebnf_dual_run_diff`
  - `perf_bench`
- The smaller `pgen_ast`, `return_annotation_generated_audit`, and `pgen` executables are better thought of as specialist or legacy-support utilities.

## Where To Start By Task Type

### If the task is figuring out which Rust executable owns a workflow
Start here:
- `rust/Cargo.toml`
- `rust/src/main.rs`
- `rust/src/bin/test_runner.rs`
- `rust/src/bin/parseability_probe.rs`
- `rust/src/bin/ebnf_dual_run_diff.rs`
- `rust/src/bin/perf_bench.rs`
- `RUST_CODEBASE_ANALYSIS.md` section `Main Rust Executables And Roles`

Reason:
- Cargo wiring matters in this repo because feature-gated binaries share entrypoints.
- The fastest way to stop wandering is to identify whether a task belongs to the main pipeline CLI, a validation harness, a parseability contract tool, a frontend diagnostic, or a specialist audit utility.

### If the task is grammar normalization or parser-shape behavior
Start here:
- `rust/src/ast_pipeline/mod.rs`
- `rust/src/ast_pipeline/grouped_quantifier_parser.rs`
- `rust/src/ast_pipeline/mutual_recursion_handler.rs`
- `rust/src/ast_pipeline/return_annotation_handler.rs`

Reason:
- This is where raw grammar structure becomes the normalized grammar tree that the rest of the system depends on.
- Changes here can affect parser generation, stimuli generation, annotation validation, and closure metrics all at once.

### If the task is generated parser behavior or parser code shape
Start here:
- `rust/src/ast_pipeline/ast_based_generator.rs`
- `rust/src/ast_pipeline/ast_code_generator.rs`
- `rust/src/ast_pipeline/ast_generator_direct.rs`

Reason:
- This is the emitted-parser contract layer.
- Parser runtime telemetry, semantic-runtime ownership, recovery behavior, and branch ordering all converge here.

### If the task is stimuli, gap reports, or coverage closure
Start here:
- `rust/src/ast_pipeline/stimuli_generator.rs`

Then usually inspect:
- `rust/src/ast_pipeline/mod.rs`
- `rust/src/ast_pipeline/semantic_runtime.rs`

Reason:
- Stimuli generation is highly coupled to normalized grammar shape and semantic steering.
- Coverage/debt behavior is not a thin report layer; it is part of how closure work is directed.

### If the task is return/semantic annotation parsing or validation
Start here:
- `rust/src/ast_pipeline/unified_return_ast.rs`
- `rust/src/ast_pipeline/unified_semantic_ast.rs`
- `rust/src/ast_pipeline/annotation_validator.rs`
- `rust/src/ast_pipeline/semantic_directive_registry.rs`
- `rust/src/ast_pipeline/semantic_runtime.rs`

Reason:
- The typed annotation model, validator rules, directive registry, and runtime behavior are split across these files.
- It is easy to fix one layer and accidentally leave the others inconsistent.

### If the task is external integration or embedder-facing API behavior
Start here:
- `rust/src/embedding_api.rs`
- `rust/src/parser_registry.rs`
- `rust/src/lib.rs`
- `rust/build.rs`

Reason:
- Consumer-visible behavior depends on both runtime dispatch and build-time generated-parser availability.
- Feature/cfg/build-path interactions matter here as much as function signatures do.

### If the task is SystemVerilog preprocessing
Start here:
- `rust/src/sv_preprocessor.rs`

Then usually inspect:
- `rust/src/main.rs`
- relevant `rust/scripts/sv_*` gates

Reason:
- The SV preprocessor is its own subsystem with policies, diagnostics, event logs, and source maps.
- Its behavior is not just a helper phase before parsing.

### If the task is EBNF frontend conversion
Start here:
- `rust/src/ebnf_frontend.rs`
- `rust/src/main.rs`
- `rust/Makefile`
- relevant `rust/scripts/ebnf_*` gates

Reason:
- The Rust EBNF frontend sits at the start of the build/proof spine, not only as a parsing helper.
- Changes here can affect raw-AST conversion, dual-run differentials, and the generated-parser pipeline.

### If the task is CLI mode behavior or top-level orchestration
Start here:
- `rust/src/main.rs`
- `rust/src/bin/*.rs`
- `rust/Makefile`

Reason:
- The codebase has one large orchestration entrypoint plus several smaller utility binaries.
- The main risk is changing mode behavior without aligning the supporting build/gate surface.

### If the task is proof plumbing, contract sidecars, or release-gate behavior
Start here:
- `rust/scripts/*.sh`
- `rust/Makefile`
- `rust/src/bin/parseability_probe.rs`
- `rust/src/parser_registry.rs`
- `rust/src/embedding_api.rs`

Reason:
- A large amount of project truth now lives in the shell-gate layer and the artifacts it consumes/emits.
- These tasks often require understanding both machine-readable sidecars and the Rust producer/consumer seams behind them.

## High-Risk Change Zones
- `rust/src/ast_pipeline/mod.rs`
  - high blast radius because it changes the normalized grammar contract used by both parser and stimuli generation.
- `rust/src/ast_pipeline/ast_based_generator.rs`
  - high blast radius because emitted parser behavior, runtime telemetry, and semantic hooks all converge here.
- `rust/src/ast_pipeline/stimuli_generator.rs`
  - high blast radius because closure metrics, target planning, and semantic steering are co-located here.
- `rust/src/main.rs`
  - high coordination cost because many modes share one orchestration entrypoint.
- `rust/build.rs`
  - easy to underestimate; build-time parser-availability bugs can look like runtime/parser bugs elsewhere.
- `rust/src/embedding_api.rs` and `rust/src/parser_registry.rs`
  - small files relative to the engines, but they sit on public integration seams where drift is expensive.
- `rust/scripts/sota_exit_gate.sh` and sibling family aggregate/status gates
  - not Rust code, but they are part of the effective Rust-owned product contract.

## Change-Impact Checklist
Use this as a first-pass companion-check map, not as a complete proof checklist.

- If you change grammar normalization or core AST pipeline shape
  - Typical primary files:
    - `rust/src/ast_pipeline/mod.rs`
    - `rust/src/ast_pipeline/grouped_quantifier_parser.rs`
    - `rust/src/ast_pipeline/mutual_recursion_handler.rs`
  - Usually re-check:
    - `rust/src/ast_pipeline/ast_based_generator.rs`
    - `rust/src/ast_pipeline/stimuli_generator.rs`
    - `rust/src/ast_pipeline/annotation_validator.rs`
    - generation-input AST dump behavior in `rust/src/main.rs`
    - round-trip / parseability surfaces that implicitly depend on normalized rule shape
- If you change parser code generation
  - Typical primary files:
    - `rust/src/ast_pipeline/ast_based_generator.rs`
    - `rust/src/ast_pipeline/ast_code_generator.rs`
    - `rust/src/ast_pipeline/ast_generator_direct.rs`
  - Usually re-check:
    - generated parser compileability and include-path assumptions
    - `rust/src/parser_registry.rs`
    - `rust/src/embedding_api.rs`
    - `rust/src/bin/parseability_probe.rs`
    - `rust/src/bin/test_runner.rs`
    - `rust/src/bin/perf_bench.rs`
- If you change stimuli, coverage, or gap logic
  - Typical primary file:
    - `rust/src/ast_pipeline/stimuli_generator.rs`
  - Usually re-check:
    - `rust/src/main.rs` stimuli CLI/report wiring
    - parseability validation report behavior
    - coverage / gap / target-drive JSON artifacts
    - grammar-quality and family-contract gate expectations in `rust/scripts/*.sh`
- If you change annotation parsing, validation, or semantic runtime behavior
  - Typical primary files:
    - `rust/src/ast_pipeline/unified_return_ast.rs`
    - `rust/src/ast_pipeline/unified_semantic_ast.rs`
    - `rust/src/ast_pipeline/annotation_validator.rs`
    - `rust/src/ast_pipeline/semantic_runtime.rs`
    - `rust/src/ast_pipeline/semantic_directive_registry.rs`
  - Usually re-check:
    - generated parser conversion paths
    - `test_runner` bootstrap vs generated parity
    - annotation-focused suites and typed-AST consumers
    - any docs or gates that currently treat return-annotation support as closed and semantic support as still more fluid
- If you change build-script or generated-parser availability behavior
  - Typical primary files:
    - `rust/build.rs`
    - `rust/src/lib.rs`
    - `rust/src/parser_registry.rs`
  - Usually re-check:
    - Cargo feature combinations
    - `PGEN_*_PARSER_PATH` resolution behavior
    - `has_generated_*` cfg guards
    - binaries gated by `generated_parsers` or `ebnf_dual_run`
    - embedder-facing availability behavior in `embedding_api.rs`
- If you change embedder-facing or registry-facing parse surfaces
  - Typical primary files:
    - `rust/src/embedding_api.rs`
    - `rust/src/parser_registry.rs`
  - Usually re-check:
    - `rust/src/bin/parseability_probe.rs`
    - AST dump contract behavior
    - feature/cfg fallback behavior
    - any gates or tests that rely on registry exposure or parser support checks
- If you change EBNF frontend behavior
  - Typical primary files:
    - `rust/src/ebnf_frontend.rs`
    - `rust/src/main.rs`
    - `rust/src/bin/ebnf_dual_run_diff.rs`
  - Usually re-check:
    - raw-AST export behavior
    - dual-run drift reports
    - `ebnf_dual_run` build assumptions
    - readiness/quality gates that now rely on the Rust frontend path
- If you change SystemVerilog preprocessing behavior
  - Typical primary files:
    - `rust/src/sv_preprocessor.rs`
    - SV preprocess wiring in `rust/src/main.rs`
  - Usually re-check:
    - source-map and diagnostics behavior
    - strict-warning policy handling
    - downstream parseability expectations on preprocessed output
    - SV quality/aggregate proof gates in `rust/scripts/`
- If you change proof-sidecar shape or release-gate aggregation
  - Typical primary files:
    - `rust/scripts/*.sh`
    - sometimes `rust/src/bin/parseability_probe.rs` or `rust/src/embedding_api.rs`
  - Usually re-check:
    - `summary.txt` / `summary.json` parity
    - `ci_workflow_local_gate.sh`
    - higher aggregate readers like family-status, combined telemetry, and SOTA exit
    - `RUST_CODEBASE_ANALYSIS.md` if the effective operational contract changed

## Build And Feature Model
- The crate is feature-gated around bootstrap, normal, generated-parser, and EBNF-dual-run modes.
- Generated parser modules are not hardwired; `rust/build.rs` resolves them from environment-configured paths and only enables grammar-specific `cfg`s when files actually exist.
- This is a strength because it supports:
  - bootstrap cycles
  - clean checkout behavior
  - relocatable worktrees
  - partial grammar availability
- It also means more conditional complexity and more chances for path/feature divergence.

Feature/build matrix:
- `normal`
  - unlocks the `ast_pipeline` binary from `rust/src/main.rs`
  - represents the standard non-bootstrap orchestration path
- `bootstrap`
  - unlocks the `ast_pipeline_bootstrap` binary from the same `rust/src/main.rs` entrypoint
  - exists so the pipeline can keep functioning when generated-parser availability is intentionally reduced or absent
- `generated_parsers`
  - unlocks binaries and code paths that depend on the generated parser registry and generated parser includes
  - directly gates:
    - `parseability_probe`
    - `perf_bench`
    - generated-parser branches in the embedding/test surfaces
- `ebnf_dual_run`
  - unlocks the generated-EBNF differential tooling
  - directly gates:
    - `ebnf_dual_run_diff`
    - Rust-frontend/generated-frontend comparison flows in the CLI/build ecosystem

Build-time availability model:
- `rust/build.rs` does two distinct jobs:
  - resolves include paths from environment variables like `PGEN_SYSTEMVERILOG_PARSER_PATH` and `PGEN_VHDL_PARSER_PATH`
  - emits grammar-specific `cfg`s like `has_generated_systemverilog_parser` only when the resolved file actually exists
- That means feature enablement alone is not enough for every generated-parser behavior.
- In practice there are two layers of availability:
  - Cargo feature enabled
  - matching generated parser file found by `build.rs`

Generated parser env/cfg map:
- `PGEN_EBNF_PARSER_PATH`
  - resolved by `build.rs` into:
    - `PGEN_EBNF_PARSER_PATH_RESOLVED`
    - `PGEN_EBNF_PARSER_PATH_RESOLVED_BIN`
  - used by the `ebnf_dual_run` surface
  - important nuance: there is no `has_generated_ebnf_parser` cfg; EBNF availability is handled differently from the other grammar families
- `PGEN_JSON_PARSER_PATH`
  - drives `has_generated_json_parser`
  - controls `generated_parsers::json` and related parser-registry exposure
- `PGEN_REGEX_PARSER_PATH`
  - drives `has_generated_regex_parser`
  - controls `generated_parsers::regex` and related parser-registry exposure
- `PGEN_SYSTEMVERILOG_PARSER_PATH`
  - drives `has_generated_systemverilog_parser`
  - controls generated SystemVerilog registry, embedding, and parseability paths
- `PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_PATH`
  - drives `has_generated_systemverilog_preprocessor_parser`
  - controls generated SV-preprocessor parser availability
- `PGEN_VHDL_PARSER_PATH`
  - drives `has_generated_vhdl_parser`
  - controls generated VHDL registry, embedding, and parseability paths
- `PGEN_RTL_CONST_EXPR_PARSER_PATH`
  - drives `has_generated_rtl_const_expr_parser`
  - controls generated RTL-const-expr parser availability

Important asymmetries:
- `return_annotation` and `semantic_annotation`
  - live under `generated_parsers`, but are included from tracked generated sources rather than `build.rs` env-driven grammar-path discovery
- `ebnf`
  - uses `build.rs`-resolved include paths, but not the same `has_generated_*` cfg pattern used by the other generated grammar families
- `systemverilog`, `vhdl`, and the other env-driven grammar families
  - usually require both:
    - `feature = "generated_parsers"`
    - matching `has_generated_*` cfg emitted by `build.rs`

Operational takeaway:
- If a binary or API path appears to “exist but not really work,” check both:
  - the Cargo feature set
  - the relevant `PGEN_*_PARSER_PATH` resolution and resulting `has_generated_*` cfg
- A surprising amount of apparent parser/runtime breakage in this repo can actually be feature/build-shape drift.

## Testing And Verification Shape
- The test surface is not only `cargo test`.
- The codebase relies on:
  - unit/integration tests in Rust modules
  - JSON-driven round-trip suites
  - parser-family quality and contract gates
  - SOTA/aggregate proof surfaces
- `rust/src/test_runner/round_trip_tests.rs` is the more modern JSON-driven testing spine.
- `rust/src/test_registry.rs` and `rust/src/test_discovery.rs` look older and more limited by comparison.

Assessment:
- The repo is very strong on proof surfaces.
- The downside is that the test ecosystem is mixed-generation and not fully consolidated behind one obvious canonical layer.

## Strengths
- Strong architecture around determinism, observability, and machine-checkable proof.
- Clear Rust-first integration posture with explicit bootstrap escape hatches rather than hidden hand-written exceptions.
- Stable consumer-facing API design in `embedding_api.rs`.
- Sophisticated stimuli/coverage/gap machinery that matches the project’s closure doctrine.
- Good generated-parser integration model in `build.rs`.
- Real policyfulness in the SV preprocessor instead of a shallow text-prepass design.

## Main Risks And Technical Debt
- Complexity concentration in:
  - `stimuli_generator.rs`
  - `ast_based_generator.rs`
  - `annotation_validator.rs`
  - `mod.rs`
  - `main.rs`
- Repeated grammar/backend/profile adapter logic across:
  - `parser_registry.rs`
  - `embedding_api.rs`
  - selected binaries / CLI surfaces
- Bootstrap/generated duality remains necessary but expensive to reason about.
- Semantic support is powerful but distributed across several coupled files, which raises the cost of safe changes.
- The shell-gate layer is large enough that “the Rust codebase” now effectively includes a substantial Bash proof system.

## Steering Implications
- Future implementation should keep treating parser generation, stimuli closure, and proof/gate output as one system.
- Effort spent only on parser acceptance without preserving observability and proof surfaces will fight the project’s actual architecture.
- Refactors should aim to reduce concentration without weakening the current proof doctrine.

## Recommended Refactor Priorities
- Split `rust/src/main.rs` into subcommand or mode-focused modules.
- Break `rust/src/ast_pipeline/stimuli_generator.rs` into smaller policy/reporting/runtime units.
- Break `rust/src/ast_pipeline/ast_based_generator.rs` into emitter-focused submodules:
  - parser struct/runtime emission
  - semantic runtime emission
  - recovery/coverage telemetry emission
  - per-rule method emission
- Reduce repeated dispatch logic by introducing a more unified grammar/backend adapter layer shared by:
  - `parser_registry.rs`
  - `embedding_api.rs`
  - CLI/binary consumers
- Clarify which test layers are canonical and which are legacy carryovers.

## What To Re-Check At The Start Of A New Session
- Whether the hotspot files and their responsibilities have materially shifted.
- Whether new grammar families or generated parser integrations changed the build/registry shape.
- Whether bootstrap vs generated boundaries moved.
- Whether the public consumer seam changed:
  - embedding API
  - parser registry
  - grammar-profile coverage
- Whether the proof/gate layer changed enough that this document’s description of the operational surface is stale.
- Whether the main current risks are still:
  - concentrated module size
  - repeated adapter seams
  - bootstrap/generated maintenance cost
  - mixed-generation testing layers

## Session-Start Sanity Probes
Use these as cheap orientation probes before deeper Rust work, not as a replacement for task-specific validation.

- `git status --short`
  - Confirms whether unrelated dirt, generated artifacts, or untracked directories are already present before you start attributing odd behavior to the code.
- `rg -n "^\\[\\[bin\\]\\]|^\\[features\\]" rust/Cargo.toml`
  - Re-checks whether the binary and feature surface still matches this document’s assumptions.
- `rg -n "PGEN_[A-Z_]+_PARSER_PATH|has_generated_" rust/build.rs rust/src/lib.rs`
  - Re-checks the generated-parser availability contract quickly without re-reading the full files.
- `rg --files rust/src/bin`
  - Re-confirms the active Rust utility-binary surface.
- `sed -n '1,120p' RUST_CODEBASE_ANALYSIS.md`
  - Fast check that the live analysis doc still presents the same top-level structure and hasn’t fallen behind a major architectural shift.
- If the task is proof/gate-heavy:
  - `rg -n "summary\\.json|summary\\.txt|sota_exit_gate|combined_telemetry|family_status" rust/scripts`
  - Quick way to confirm whether the proof-sidecar vocabulary or aggregate-gate surface has drifted materially since the last session.

## Limits Of This Snapshot
- This assessment came from a deep source read and structural review.
- It is not a benchmark report.
- It is not a full dynamic validation run of all Rust binaries and all gates.
- It should therefore be refreshed when runtime evidence materially changes the picture.
