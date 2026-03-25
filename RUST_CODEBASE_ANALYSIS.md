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

## Where To Start By Task Type

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

## Build And Feature Model
- The crate is feature-gated around bootstrap, normal, generated-parser, and EBNF-dual-run modes.
- Generated parser modules are not hardwired; `rust/build.rs` resolves them from environment-configured paths and only enables grammar-specific `cfg`s when files actually exist.
- This is a strength because it supports:
  - bootstrap cycles
  - clean checkout behavior
  - relocatable worktrees
  - partial grammar availability
- It also means more conditional complexity and more chances for path/feature divergence.

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

## Limits Of This Snapshot
- This assessment came from a deep source read and structural review.
- It is not a benchmark report.
- It is not a full dynamic validation run of all Rust binaries and all gates.
- It should therefore be refreshed when runtime evidence materially changes the picture.
