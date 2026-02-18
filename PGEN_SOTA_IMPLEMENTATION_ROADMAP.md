# PGEN SOTA Implementation Roadmap (Living)

Last updated: 2026-02-18

## Mission
Build PGEN into a state-of-the-art parser and stimuli generation platform with production-grade return/semantic annotation support, suitable for embedding in high-rigor systems (SystemVerilog/VHDL tooling, regex engines, and similar domains).

## Status Legend
- `Not Started`
- `In Progress`
- `Blocked`
- `Done`

## Pillar Tracker

| Pillar | Current Status | Target Outcome |
|---|---|---|
| 1. Bootstrap Trust and Fixed-Point Reproducibility | In Progress | Repeated bootstrap cycles are byte-identical for annotation grammars. |
| 2. Normative Annotation Specification | In Progress | One normative spec for return/semantic annotation syntax + semantics. |
| 3. Typed Annotation Validation | In Progress | Compile-time validation of references/transforms with precise diagnostics. |
| 4. Bootstrap vs Generated Behavioral Contract | In Progress | Explicitly tracked differences with required tests and closure plan. |
| 5. Industrial Frontend Support (SV/VHDL Readiness) | Not Started | Preprocess/lex/parse pipeline robust for real-world HDL sources. |
| 6. Ambiguity Handling and Recovery | Not Started | Deterministic branch resolution and production-grade error recovery. |
| 7. Coverage-Guided Semantic Stimuli | In Progress | Feedback loop that drives branch/rule/annotation coverage upward. |
| 8. Differential Validation vs External Parsers | In Progress | Continuous mismatch detection against trusted external tools. |
| 9. Performance and Scalability SLAs | In Progress | Enforced throughput/memory/latency budgets in CI. |
| 10. Embedding-Grade APIs and Contracts | In Progress | Stable crate API, deterministic behavior, and versioned contracts. |
| 11. Security and Robustness Hardening | In Progress | Fuzzed, bounded, and resilient parser/stimuli runtime. |
| 12. SOTA Exit Criteria Gate | Not Started | CI-enforced release gates with objective pass thresholds. |

## Execution Plan (Ordered)

### Phase A (Now)
- [x] Create living roadmap document and track implementation status here.
- [x] Add fixed-point bootstrap gate script for return/semantic parser artifacts.
- [x] Add `make fixed_point_gate` target for local and CI usage.
- [x] Wire `fixed_point_gate` into CI as required pre-merge gate.
- [x] Increase gate strictness from 2-cycle to 3-cycle minimum in CI.

### Phase B (Next)
- [x] Implement typed return annotation validator (`$n`, extraction, spread, object/array shape checks).
- [x] Implement typed semantic annotation validator (transform function signatures and argument checks).
- [x] Emit structured diagnostics with stable error codes.
- [x] Extend validator coverage to include rule-aware capture bounds and branch-shape compatibility.
- [x] Add strict-mode failure policy into standard CI gates (not only env-triggered mode).

### Phase C
- [x] Add coverage-guided semantic fuzzing loop with seed replay and corpus minimization.
- [x] Add shrinking for failing stimuli and parseability counterexamples.
- [x] Add gap-driven generator priorities (rule/branch/annotation coverage targets).

### Phase D
- [x] Add differential harness against external parser/tool baselines.
- [x] Add performance benchmark suite and CI thresholds.
- [x] Finalize embedding API stability and versioning policy.

### Phase E
- [x] Add differential mismatch taxonomy + baseline regression gate (`new mismatch only`) with tracked baseline snapshots.
- [x] Wire `differential_regression_gate` into CI as required pre-merge check with report artifact retention.
- [x] Author and maintain comprehensive PGEN User Guide (EBNF, return/semantic annotations, coverage flows, stimuli generation, automation workflows, troubleshooting).

### Phase F (Current)
- [x] Publish living normative annotation specification with explicit bootstrap/full/validator contracts.
- [x] Add executable bootstrap contract suites for inferred built-in return/semantic parser behaviors.
- [x] Add `make annotation_contract_gate` to enforce validator + built-in contract suites.
- [x] Wire `annotation_contract_gate` into CI as required pre-merge check.
- [x] Add shared bootstrap/generated annotation contract suites and enforce them in gate paths.

### Phase G (Current)
- [x] Add embedding API input bounds (`ParseLimits`) with stable diagnostics for oversized/invalid inputs.
- [x] Extend embedding API contract docs with limit behavior and new diagnostic codes.

## Current Sprint: Pillar 1

### Completed in this sprint
- Added a reproducibility gate that regenerates return/semantic annotation JSON and parser outputs for multiple cycles and asserts byte-identical outputs between cycle 1 and subsequent cycles.

### Remaining for Pillar 1 completion
- Branch protection rule should require the `fixed-point-gate` check before merge.

## Risks and Mitigations
- Risk: Non-deterministic codegen details (ordering, paths, timestamps) can create false drifts.
  - Mitigation: Fixed output paths per cycle, byte-level comparisons, and explicit diff output on mismatch.
- Risk: Gate bypass in local workflows.
  - Mitigation: Add Make target now; enforce in CI next.
- Risk: Bootstrap/generated behavior drifts without visibility.
  - Mitigation: Maintain conformance tests and feature matrix tracking as required checklists.

## Change Log (Roadmap Updates)
- 2026-02-18: Initialized roadmap and marked Pillar 1 implementation started.
- 2026-02-18: Added GitHub Actions `fixed-point-gate` workflow and started Phase B validator implementation with structured diagnostics.
- 2026-02-18: Extended annotation validator with grammar-aware branch/capture checks and integrated grammar-aware validation into parser generation.
- 2026-02-18: Enforced strict annotation validation policy in standard CI gate path and switched CI fixed-point runs to 3 cycles by default.
- 2026-02-18: Added fixed-point drift artifact upload/retention policy in CI (failure-only upload of `rust/target/fixed_point_gate` snapshots and diffs).
- 2026-02-18: Added coverage-guided fuzz loop mode with deterministic per-seed replay and greedy corpus minimization for stimuli generation.
- 2026-02-18: Added shrinking for parseability counterexamples and failing stimuli traces (delta-debug-style minimization in replay and parseability-failure diagnostics).
- 2026-02-18: Added gap-priority generation mode that applies reachable targets from a gap report as branch/rule bias in count-based stimuli generation.
- 2026-02-18: Started Phase D by adding generated-vs-bootstrap differential harness mode in `test_runner` with JSON mismatch reports and Makefile automation (`make differential_report`).
- 2026-02-18: Completed Phase D performance gate by adding `perf_bench`, `make performance_gate`, threshold policy (`rust/perf/thresholds.json`), and CI workflow wiring (`performance-gate`).
- 2026-02-18: Completed Phase D embedding API stabilization with versioned `pgen::embedding_api` contracts, deterministic structured parse outcomes, and `make embedding_api_gate`.
- 2026-02-18: Started Phase E by adding mismatch taxonomy + baseline closure tracking in differential mode and a regression-only gate (`make differential_regression_gate`) backed by tracked baseline snapshots under `rust/test_data/differential_baseline/`.
- 2026-02-18: Added CI workflow `differential-regression-gate` to enforce `make differential_regression_gate` on PR/main and retain differential report artifacts.
- 2026-02-18: Published initial comprehensive end-user guide in `PGEN_USER_GUIDE.md` and linked it from `README.md` (living document for onboarding + full feature usage).
- 2026-02-18: Started Pillar 2 by publishing `PGEN_ANNOTATION_NORMATIVE_SPEC.md`, adding bootstrap contract suites (`builtin_contract.json`), and wiring `make annotation_contract_gate`.
- 2026-02-18: Added CI workflow `annotation-contract-gate` to enforce `make annotation_contract_gate` on PR/main.
- 2026-02-18: Added shared bootstrap/generated contract suites (`normative_shared_contract.json`) and extended `annotation_contract_gate` with `annotation_shared_contract_gate`.
- 2026-02-18: Started Pillar 11 by hardening embedding API parsing with bounded input limits (`ParseLimits`, `E_INPUT_TOO_LARGE`, `E_INVALID_LIMITS`) and updated contract docs.
