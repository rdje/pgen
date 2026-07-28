# PGEN

PGEN is a production-focused parser and stimuli generator platform.

## Project Objective
- Build **state-of-the-art, EBNF-driven parser/stimuli generation** for serious language tooling.
- Parser-construction doctrine:
  - every parser that counts as a PGEN deliverable shall be EBNF-backed,
  - there are no exceptions to this rule,
  - handwritten parsers may exist only as bootstrap/prototyping scaffolding and do not count as final closure.
- Annotation doctrine:
  - every generated parser returns an AST,
  - return annotations are the normative mechanism for shaping that returned AST,
  - semantic annotations are the normative mechanism for steering parser-generation behavior.
- Parser proof doctrine:
  - for a deliverable grammar `grammars/foolang.ebnf`, closure expects a generated parser path (`generated/foolang_parser.rs`) plus a stimuli path,
  - that stimuli path may be the default in-memory generator, a generated module artifact (`generated/foolang_stimuli.rs`), or both,
  - when both stimuli forms exist, parity between them is part of the contract,
  - parser closure requires objective roundtrip and coverage proof for both parsing and stimuli generation rather than narrative confidence,
  - this doctrine applies to any PGEN EBNF-based parser family with no exception: SystemVerilog, VHDL, regex, annotation grammars, Phase S parser families, and future grammar families are all judged against the same professional-grade closure standard,
  - the live tracker differs by how much of that common proof doctrine has been landed for a given parser family, not by using different quality bars for different grammars.
- Support advanced **return annotations** and **semantic annotations** with contract-grade validation.
- Deliver parser/stimuli quality via deterministic gates, coverage/gap analysis, and closed-loop replay.
- Treat parser quality as the product:
  - generated parsers must be correct, fast, accurate, predictable, observable, and trustworthy in real systems.
- Rust toolchain floor:
  - the maintained Cargo packages in this repository now declare an explicit MSRV of `1.95`
  - if you are building the Rust-owned surfaces directly, assume Rust `1.95` or newer
- Tracing doctrine:
  - every new tool or operational surface added to PGEN should expose the same trace-verbosity contract:
    - `none`
    - `low`
    - `medium`
    - `high`
    - `debug`
  - prefer one shared tracing model over per-tool ad hoc debug flags or bespoke log levels,
  - route instrumentation through shared trace helpers/macros rather than scattered `println!`-style debugging,
  - instrument the real control flow:
    - entry/exit,
    - important branch decisions,
    - fallbacks,
    - retries,
    - timeouts,
    - and failure boundaries,
  - the current Rust AST pipeline already provides the maintained reference shape for this doctrine through `TraceVerbosity`, `PGEN_TRACE_VERBOSITY`, and the `pgen_trace*` macros; future tools should align with that surface instead of inventing incompatible tracing schemes.
- North-star trust goal:
  - make PGEN the de facto go-to platform for parsers because projects can trust it,
  - make PGEN sign-off-grade when parsing correctness materially affects downstream flows.
- Primary near-term integration targets:
  - **Nexsim** (SystemVerilog + VHDL parsing)
  - **RGX** (regex parsing)
  - **PNR** (staged LEF / Liberty / DEF / Verilog structural netlist / SDC / SPEF parser-family demand captured as a downstream contract)

## Canonical Flow
- `grammars/foolang.ebnf -> raw_ast/json -> generated/foolang_parser.rs`
- `grammars/foolang.ebnf -> in-memory stimuli and/or generated/foolang_stimuli.rs`
- Rust-native EBNF frontend now also supports direct `raw_ast` export:
  - `ast_pipeline INPUT.ebnf --emit-raw-ast-json RAW.json`
- Annotation parsers (`return_annotation_parser`, `semantic_annotation_parser`) are generated with bootstrap mode only.
- `grammars/builtin_return_annotation.ebnf` and `grammars/builtin_semantic_annotation.ebnf` are the bootstrap-safe annotation grammar contracts used for that bootstrap generation path, so the annotation parsers can be generated without depending on themselves.
- All other grammars use the non-bootstrap path.
- `grammars/return_annotation.ebnf` with `generated/return_annotation_parser.rs` defines the supported AST-shaping language for parser return values.
- `grammars/semantic_annotation.ebnf` with `generated/semantic_annotation_parser.rs` defines the supported steering language for parser-generation behavior.
- `make -C rust SHELL=/bin/bash annotation_contract_gate` is the aggregate annotation contract spine for validator coverage, built-in/shared annotation suites, SC semantic contract slices, aggregate semantic/return contract gates, and annotation robustness/stimuli verification.
- `make -C rust SHELL=/bin/bash annotation_stimuli_quality_gate` is the required closed-loop proof surface for annotation stimuli quality, including the return-annotation generator/parser loop.
- `make -C rust SHELL=/bin/bash semantic_full_contract_gate` is the focused aggregate proof surface for semantic annotation runtime, round-trip, and comparable differential-regression evidence.
- `make -C rust SHELL=/bin/bash return_annotation_support_gate` is the focused aggregate proof surface for return-annotation closure in the Rust AST pipeline; it now includes the auto-derived `return_annotation_exhaustiveness_gate` (grammar-driven coverage closure, stimuli-module parity, and generated-parse-tree to typed-AST audit) and is the formal `Done` gate for the currently tracked return-annotation claim.
- In general, PGEN supports two stimuli-delivery modes for a grammar:
  - default in-memory generation via `--generate-stimuli`,
  - optional generated module artifacts via `--generate-stimuli-module` (for example `generated/foolang_stimuli.rs`).
- For serious parser closure claims, the expected evidence is:
  - EBNF-backed parser generation,
  - return-AST shaping through return annotations,
  - parser/stimuli roundtrip proof,
  - parser coverage proof,
  - stimuli-generation coverage/gap proof,
  - and repeatable machine-checkable gates behind every claim.
  - This is the repository-wide closure doctrine for any PGEN EBNF-based parser, not an SV-only or annotation-only rule.

## Fast Ramp-Up (Read In This Order)
> ⛔ **Before debugging ANY issue** (an `UNKNOWN`, a rejected parse, a hang, a reach gap, a "why isn't this witnessed"), read **`TOOLBOX.md`** and run the debug tools FIRST — never eyeball a grammar or guess a root cause. This is a standing director directive **and it is mechanically ENFORCED**: a code change cannot land without tool-backed WHY+WHERE diagnosis + measured before→after verification in its owning task leaf (`scripts/check_doctrines.sh` via `.githooks/pre-commit`; see `DOCTRINE_ENFORCEMENT.md`).
1. `README.md` (this file)
2. `docs/book/` (`mdBook` live mastery surface)
3. `TOOLBOX.md` (the diagnostic & debug toolbox — when/how for every debug tool; READ BEFORE debugging)
4. `QUICKSTART_AI_ONBOARDING.md`
5. `PGEN_USER_GUIDE.md`
6. `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
7. `LIVE_ACHIEVEMENT_STATUS.md`
8. `docs/reference/RUST_CODEBASE_ANALYSIS.md`
9. `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`
10. `docs/reference/PGEN_LINTER_ENABLEMENT_ROADMAP.md`
11. `docs/reference/PGEN_COMPILER_ELABORATOR_ENABLEMENT_ROADMAP.md`
12. `docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`
13. `docs/TASK_TREE.md` (active task trees + PNT selection rules)
14. `docs/TASK_TREE_README.md` (reusable workflow installation guide)
15. `MEMORY_ARCHITECTURE.md` (how durable, harness-agnostic agent memory + continuity work here — the 4 layers + enforcement)
16. `DOCTRINE_ENFORCEMENT.md` (how EVERY mechanizable doctrine is enforced — the portable enforcer kit)
17. `MEMORY.md` (layer A — the bounded resume pointer; read first on resume)
18. `CHANGES.md`
19. `DEVELOPMENT_NOTES.md`
20. `COMMIT.md`

## Key Project Paths
- `grammars/`: EBNF sources (`*.ebnf`)
- `grammars/builtin_return_annotation.ebnf`, `grammars/builtin_semantic_annotation.ebnf`: bootstrap-safe annotation grammar contracts that break the annotation-parser chicken-and-egg cycle
- `grammars/scratch/scratch.ebnf`: the **parse-harness scratch slot** (PARSE-HARNESS approach 3) — a blessed, throwaway probe grammar (registered as `scratch`) whose body you overwrite freely to parse an *arbitrary* grammar's input through the whole `parseability_probe` toolbox, authoritative by construction. Rebuild with `make -C rust focus_scratch`, then `parseability_probe --parse scratch <input>`. See the *The Parse Harness* book chapter, `grammars/scratch/README.md`, and `docs/tasks/PARSE-HARNESS.md`.
- `generated/`: pipeline-output artifacts (parser sources, AST JSON dumps, return-annotation inventories) consumed by compile-time includes; **not tracked in git** — regenerate locally with the per-grammar `make` targets (e.g. `make -C rust focus_<grammar>`)
- `rust/target/generated_logs/`: scratch generation/debug logs kept out of `generated/`
- `rust/src/`: Rust AST pipeline, generators, parser registry, embedding API
- `rtl_const_expr/`: standalone constant-expression parser/evaluator bootstrap baseline crate for planned RTL frontend/elaboration work, including dotted and package-qualified (`pkg::NAME`) identifier lookup; it is now paired with tracked grammar `grammars/rtl_const_expr.ebnf` and generated parser `generated/rtl_const_expr_parser.rs` because RTLSyn needs deterministic parameter/width/generate evaluation before elaboration can be trusted
- `rtl_frontend/`: initial synthesizable-RTL frontend bootstrap baseline crate wired to `rtl_const_expr` for module/instance parsing, typed port actuals (including member-path/expression/repetition forms), unpacked-array port/net declarations, struct-aware validation through indexed unpacked-array elements, enum and union data types with typedef/import visibility including generate-body-local alias scope, builtin integral atom types (`byte`, `shortint`, `longint`) in declarations and enum base-width handling, `always_ff` edge-event controls and `always_latch` procedural blocks in addition to `always_comb` / `always @(*)`, typed assignment targets for `assign` and procedural statements (including signal/member/select/part-select/concatenation forms), structured assignment values (including signal/member/select/concat/repeat forms), syntax-only selector/concat-rich expression text for generated-contract parity, bounded selector-only syntax parameter-override evaluation when parent constant symbols, including package-qualified constant symbols, make signal/bit/part-select extraction unambiguous, scoped package-qualified selector-actual handling in the handwritten semantic lane, elaboration-time procedural validation for known identifiers plus `always_ff` nonblocking-assignment policy, packed-union width-coherence validation, instance-array expansion, inline aggregate-aware member validation, file-scope/module-local/package typedef-backed named types, package-backed constant declarations plus package-qualified/body-import/header-import constant visibility, and first-pass elaboration helpers; it is now also paired with tracked bootstrap grammar `grammars/rtl_frontend.ebnf`, generated artifacts `generated/rtl_frontend_parser.rs` / `generated/rtl_frontend.json`, curated generated-contract manifest `rust/test_data/grammar_quality/rtl_frontend_generated_parity_contract_v0.json`, and gate `make -C rust SHELL=/bin/bash rtl_frontend_generated_contract_gate` (typed-AST-era contract `0.2.0`; parser release `1.0.5` / AST-dump schema `3`), which proves parse-acceptance for all `130` curated samples (98 accepts / 32 rejects — including bare ANSI ports `input R` / `output R` / `inout R`, accepted since the `1.0.4` `RTL-FE-CLOSURE.9` fix that gave `port_group` a no-type branch, and keyword-prefixed identifiers such as `input_data` / `reg_file` / `format`, accepted since the `1.0.5` `RTL-FE-CLOSURE.10` keyword word-boundary (`\b`) fix), rule participation for the accepted parses via the generated parser's transactional coverage record (`required_rule_names` / `forbidden_rule_names` — entry testimony that is sound under PEG backtracking and complete under return-annotation folding), `232` curated retained-text locks re-expressed as exact string values of the released schema-3 typed JSON carrier (`required_typed_string_values`), handwritten-baseline parse replay over the same manifest with no current `expected_handwritten_parse_ok` divergence overrides, and the ratcheted optional `expected_elaboration` replay layer across 59 curated semantic samples (46 accepts / 13 rejects, with ratcheted minimums on top-parameter, child-path, child-parameter, and child-port-binding checks enforced by the handwritten replay tests); the `0.1.0` raw-envelope `rule_name`/`span` retention checks were retired in the `0.2.0` migration — they became structurally unsatisfiable once the released typed-AST campaign (schema 3) folded the structural rules into the typed carrier — with multi-token span evidence superseded by the coverage testimony plus the handwritten parity and elaboration layers; rtl_frontend has now reached its per-family PGEN closure bar — certificate-coverage `fully_certified=true` (`UNKNOWN=0`, deterministic at seeds 0/7/42) plus the green generated-contract/elaboration gate — so its LIVE row is `Done` (leaf `RTL-FE-CLOSURE.8`); the remaining Phase S work is the broader parser-stack (the Liberty/SDC companion crates) and deeper elaboration parity as the synthesizable subset widens
- Current `rtl_frontend` generated-contract note: the curated manifest (`rust/test_data/grammar_quality/rtl_frontend_generated_parity_contract_v0.json`) is the authoritative per-sample detail behind that gate — its samples cover module/port/net declaration shells, typed aggregate (struct/enum/union) surfaces, procedural `always_ff` / `always_comb` / `always @(*)` / `always_latch` lanes, generate `if`/`else`/`for` structures, hierarchy/instance-array/parameter-override and package-qualified constant flows, unpacked-array and member-path actuals, and curated near-miss rejects; consult the manifest and the rtl_frontend parser book for the exact retained evidence per sample (the pre-`0.2.0` per-lock prose notes formerly kept here described the retired raw-envelope span-lock layer and were removed with it).
- `rust/build.rs`: compile-time generated-parser include path resolver; emits relative `include!(env!(...))` paths from `rust/src/` so clean checkouts and relocated worktrees do not depend on absolute filesystem paths
- `rust/config/branch_protection_policy.json`: tracked minimum branch-protection required-check contract
- `rust/scripts/`: executable quality gates and policy runners
- `rust/test_data/grammar_quality/`: gate contracts, corpora, deterministic case manifests
- `rust/docs/`: Rust-specific architecture/API/test docs
- `docs/contracts/`: downstream parser integration contracts, issue-reporting protocol, and released-parser bug ledger
- `docs/reference/`: normative specs, matrices, closure roadmaps, release policy, and other maintained deep-reference docs
- `docs/tcl/md/tcl.md`: local Tcl syntax note for the pending SDC/Tcl-shaped PNR parser lane; reference input only, not a shipped SDC grammar
- `regex_corpus_bundle/`: PCRE2-first regex corpus acquisition/inventory starter for future regex hardening; keeps immutable upstream snapshots separate from normalized corpus/oracle outputs, with maintained gates `make -C rust regex_corpus_bundle_contract_gate`, `make -C rust regex_pcre2_textsafe_corpus_gate`, and `make -C rust regex_pcre2_compile_oracle_gate`
- `json_corpus_bundle/`: the EXTERNAL test-corpus surface for the `json` parser (the JSON analogue of `regex_corpus_bundle/`), per the external-corpus doctrine (every parser proven by BOTH the stimuli generator AND an officially-recognized external corpus). Vendors an immutable JSONTestSuite snapshot (MIT, 318 `y_`/`n_`/`i_` files) under `third_party/upstream/`, a reproducible runner `scripts/run_json_corpus.sh`, and `results/characterization.md` (the simplified `grammars/json.ebnf` measured against the standard: `y_` 81/95, `n_` 158/188, 3 deep-nesting crashes — a characterization, not a conformance gate, because `json.ebnf` is a deliberately simplified subset)
- `tools/`: conversion/extraction and support workflows
- `perl/`: legacy/frontend EBNF-to-JSON path (`ebnf_to_json.pl`) still used in hybrid flow
- `docs/systemverilog/2017`, `docs/systemverilog/2023`: SV LRM conversion workspaces
- `docs/vhdl/2019`: VHDL LRM conversion workspace
- `docs/verilog/2005`: Verilog LRM conversion workspace
- `grammars/verilog_2005_lrm_extracted.ebnf`: canonical extracted Verilog 2005 grammar snapshot from the tracked LRM workspace
- `grammars/systemverilog.ebnf`: active flattened profile-aware full-SV grammar synthesized from the IEEE 1800-2017/2023 markdown workspaces (`sv_2017`, `sv_2023`)
- `grammars/systemverilog_2017_lrm_extracted.ebnf`, `grammars/systemverilog_2023_lrm_extracted.ebnf`: full extracted SV EBNF snapshots from the versioned markdown workspaces
- `grammars/systemverilog_lrm_profiled_generated.ebnf`, `grammars/systemverilog_lrm_profiled_wrapper.ebnf`: profiled synthesis artifacts retained for regeneration traceability
- `grammars/rtl_const_expr.ebnf`: tracked Phase S constant-expression grammar already paired with `generated/rtl_const_expr_parser.rs`
- `grammars/rtl_frontend.ebnf`: tracked bootstrap EBNF for the current RTLSyn-facing synthesizable RTL subset, now paired with generated parser artifacts, registry wiring, and a curated generated-contract gate; next step is broader parity/proof closure against the handwritten baseline
- `docs/systemverilog/profiled_generation_report.json`: structured report for staged dual-LRM profile synthesis
- `tests/`: test how-to and test guides

## Diagnostic & Debug Toolbox (use FIRST, every time)
- `TOOLBOX.md` is the authoritative, meticulous catalog of every debug surface — WHAT each tool is, WHEN to reach for it, and HOW to run it. It is **ramp-up item #3** and the first thing to open for any `UNKNOWN`, rejected parse, hang, reach gap, or "why isn't this witnessed" question. Do **not** eyeball a grammar or guess a root cause.
- The systematic **3-step certificate-coverage `UNKNOWN` protocol**:
  - `PGEN_CERT_COVERAGE_DUMP_ALL=1 … --report-certificate-coverage …` → the full residual list (+ dead-rule candidates);
  - `PGEN_CERT_COVERAGE_DEBUG_PROBES=1 …` → per-rule `[plannable-probe]` `parsed`/`witnessed_target` verdicts (the WHY);
  - scoped `PGEN_TRACE_VERBOSITY=debug … --trace-rules <rule>` → the exact `🚫 rejected by post predicate …` line.
- Mirrored surfaces (kept in lockstep): book chapters `docs/book/src/diagnosing-unknowns.md` + `docs/book/src/parseability-probe-debug.md`; KM cards `docs/knowledge/cert-coverage-unknown-diagnostics.md` + `ast-pipeline-cli-reference`; decision `docs/decisions/feedback_systematically_use_debug_toolbox.md`.
- **Enforcement (not optional).** Using the toolbox is mechanically enforced, not a "trust me" claim: a code change cannot commit unless its owning task leaf carries tool-backed WHY+WHERE diagnosis **and** a measured before→after verification — checked by `scripts/check_diagnosis_evidence.sh`, run by the general doctrine enforcer `scripts/check_doctrines.sh` via `.githooks/pre-commit` (E3) and CI (E4). The portable enforcer model is `DOCTRINE_ENFORCEMENT.md`.

## Standard Commands
- General doctrine enforcer (runs every mechanizable doctrine check):
  - `bash scripts/check_doctrines.sh`
- Gate reachability inventory (which tracked gates does anything actually RUN?):
  - `bash scripts/check_gate_reachability.sh --report`
  - the `GATE-REACHABILITY` doctrine: **a check that nothing invokes is indistinguishable from a
    check that does not exist.** Every gate target must be reachable from an aggregate, a CI
    workflow or a git hook — or carry a deliberate disposition in
    `rust/test_data/grammar_quality/gate_reachability_register_v0.json`
  - it is a **ratchet, not a report**: the orphan set is re-derived on every run, an untriaged
    orphan fails, and a register entry that no longer names an orphan fails too
- All per-parser mdBooks in one lane (also run by `mdbook_docs_gate`):
  - `make -C rust SHELL=/bin/bash parser_books_gate`
- Memory guard (MANDATORY for heavy/background jobs — HOST-RAM BUDGET DIRECTIVE,
  `docs/decisions/feedback_host_ram_budget_all_jobs.md`):
  - `scripts/run_with_memory_guard.sh [--budget-mb N] [--floor-pct N] [--disk-floor-gb N] [--interval-s N] [--timeout-s N] [--marker FILE] -- <command> [args...]`
  - pre-flights system free RAM AND free disk on the working filesystem, samples the
    job's process-tree RSS, and kills the whole tree (TERM→grace→KILL, always-written
    marker) on budget breach (default 12288 MB), system-free floor breach (default
    10%), disk-floor breach (default 8 GB free; 0 disables), or timeout
  - exit codes are mechanically branchable: the child's own code on normal
    completion; 96 preflight-refused (RAM or disk — marker `reason=` disambiguates) /
    97 rss-budget / 98 free-floor / 95 disk-floor / 99 timeout /
    130 guard-interrupted
  - example: `scripts/run_with_memory_guard.sh --budget-mb 12288 --timeout-s 7200 -- make -C rust SHELL=/bin/bash sota_exit_gate`
- Aggregate policy gate:
  - `make -C rust SHELL=/bin/bash sota_exit_gate`
- Branch-protection contract gate:
  - `make -C rust SHELL=/bin/bash branch_protection_contract_gate`
- Cold-clone regeneration of `generated/` (the single home of that recipe):
  - `make -C rust SHELL=/bin/bash regenerate_generated_parsers`
  - seeds `generated/ebnf.rs`, emits the annotation pair, then the seven grammar families —
    measured **236 s** from a bare tracked tree
  - `generated/` is untracked, so any checkout that has not run this cannot compile the crate with
    `--features generated_parsers`: `rust/src/lib.rs:72,78` include the two annotation parsers by
    literal path with no `has_generated_*` cfg, so their absence is a hard rustc error
  - the hosted workflows reach it through the composite action `.github/actions/regenerate-parsers`,
    and the local parity gate through `PGEN_CI_WORKFLOW_LOCAL_PREPARE`; both call this one target
- Hosted GitHub Actions pause:
  - hosted workflows are temporarily manual-only (`workflow_dispatch`) to conserve account Actions minutes
  - routine proof should use the local `make -C rust ...` gates until hosted auto-runs are re-enabled
  - **11 of the 15** tracked workflows regenerate `generated/` first (the 3 that provably do not need
    it — `branch-protection-contract-gate`, `fixed-point-gate`, `mdbook-docs-gate` — deliberately do
    not pay for it, and `memory-architecture-gate` runs no `make -C rust` at all);
    `ci_workflow_local_gate`'s `audit_workflow_regeneration_surface` fails a workflow that runs a
    `make -C rust` gate without declaring the step
- Local workflow parity gate:
  - `make -C rust SHELL=/bin/bash ci_workflow_local_gate`
  - focused replay example:
    - `PGEN_CI_WORKFLOW_LOCAL_FILTER=annotation-contract-gate make -C rust SHELL=/bin/bash ci_workflow_local_gate`
  - successful runs under `rust/target/ci_workflow_local_gate/run.*` are removed automatically after analysis; failed runs are retained for triage
  - set `PGEN_CI_WORKFLOW_LOCAL_KEEP_RUNS=1` when a successful export/log bundle should be preserved deliberately
  - ⛔ the gate exports `git ls-files` output ONLY, and `generated/` is untracked — so the **workflow
    replay phase** needs the generated parsers materialised first. `rust/src/lib.rs:72,78` include the
    two annotation parsers by literal path with no `has_generated_*` cfg, so their absence is a hard
    rustc error rather than a disabled feature, and **8 of the 11 replays cannot run without them**
    (the 3 that can are `branch-protection-contract-gate`, `mdbook-docs-gate`, `fixed-point-gate`)
  - `PGEN_CI_WORKFLOW_LOCAL_PREPARE` **defaults to `true`**: the gate replays the repository's own
    cold-clone bootstrap (`make -C rust regenerate_generated_parsers`) inside the export dir before
    the replays, measured at ≈236 s, after which the full workflow phase runs. It engages only when
    the export dir is missing an artifact `rust/src/lib.rs` includes by literal path.
    Set `PGEN_CI_WORKFLOW_LOCAL_PREPARE=0` to skip it deliberately — worth doing for a narrowed run
    whose selected replays do not compile the crate (`branch-protection-contract-gate`,
    `mdbook-docs-gate`, `fixed-point-gate`); the gate then warns instead of preparing
  - ⛔ this default was `false` until the hosted workflows were fixed: a local green over a broken
    hosted side is **false parity, worse than the visible red the gate reported**
  - an unknown `PGEN_CI_WORKFLOW_LOCAL_FILTER` entry, or a run that ends up replaying zero workflows,
    is now **refused** — it previously reported `✅ … parity gate passed` having replayed nothing
- mdBook docs gate:
  - `make -C rust SHELL=/bin/bash mdbook_docs_gate`
- Generated-parser clippy correctness gate:
  - `make -C rust SHELL=/bin/bash generated_clippy_correctness_gate`
  - holds the generated parsers' `clippy::correctness` finding count at **0**; the tracked
    contract `rust/test_data/grammar_quality/generated_clippy_correctness_contract_v0.json`
    pins the 68-lint roster **by name and by group**, so the subset cannot narrow silently
  - gates on the **correctness category only** — the ~78.8k style/complexity warnings over
    210.8 MB of emitted code are deliberately not gated (noise, not signal)
  - ⛔ **refuses (exit 2) rather than passing** when the `generated/` artifacts are absent or
    when cargo's own build-script cfg census does not confirm they were compiled into the
    linted unit: `generated/` is untracked, so a naive run in a clean checkout would lint
    nothing and exit 0. A skip is never a pass.
  - cheap no-cargo subset (artifact presence + roster integrity):
    - `make -C rust SHELL=/bin/bash generated_clippy_correctness_policy`
- `rtl_frontend` generated contract gate:
  - `make -C rust SHELL=/bin/bash rtl_frontend_generated_contract_gate`
  - focused workflow-parity replay example:
    - `PGEN_CI_WORKFLOW_LOCAL_FILTER=rtl-frontend-generated-contract-gate make -C rust SHELL=/bin/bash ci_workflow_local_gate`
- Plain-config duality-hunt gate:
  - `make -C rust SHELL=/bin/bash duality_hunt_gate`
  - asserts the pinned per-(grammar, seed) duality-break signature sets (regex + systemverilog_preprocessor, canonical 100-sample + scaled 2000-sample budgets, seeds 0/7/42) from `rust/test_data/grammar_quality/duality_hunt_gate_contract_v0.json`, with a byte-identical determinism tripwire
  - a NOVEL signature = a new break class to route to a task-tree leaf; a VANISHED pinned signature = re-baseline the contract same-commit with the change that closed it
- Cross-family stimuli platform gate:
  - `make -C rust SHELL=/bin/bash stimuli_cross_family_platform_gate`
  - bounded shared replay over:
    - regex via the regex-only EBNF stimuli contract
    - VHDL via bounded closed-loop replay
    - SystemVerilog via bounded single-profile (`2017`) `sv_parseable_file` closed-loop replay
  - emits:
    - `rust/target/stimuli_cross_family_platform_gate/summary.txt`
    - `rust/target/stimuli_cross_family_platform_gate/summary.json`
- SV quality gate:
  - `make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
  - gate-local default:
    - the shell workflow now applies `closed_loop_target_generation_timeout_ms=5` unless overridden
    - the underlying CLI/runtime default is still `0`
    - set `PGEN_SV_STIMULI_QUALITY_TARGET_GENERATION_TIMEOUT_MS=0` to restore the legacy unbounded shell-gate posture deliberately
  - bounded replay rerun example:
    - `PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS=100 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
  - explicit primary-attempt containment example for stubborn replay triage:
    - `PGEN_SV_STIMULI_QUALITY_TARGET_MAX_ATTEMPTS=16 PGEN_SV_STIMULI_QUALITY_TARGET_GENERATION_TIMEOUT_MS=5 make -C rust SHELL=/bin/bash sv_stimuli_quality_gate`
  - helper probes still use their separate maintained budget surface:
    - `PGEN_SV_STIMULI_QUALITY_TARGET_HELPER_TIMEOUT_MS=<ms>`
- Verilog-2005 dialect-profile conformance gate:
  - `make -C rust SHELL=/bin/bash verilog_2005_conformance_gate`
  - asserts the strict `verilog_2005` (IEEE 1364-2005) profile surface against the tracked contract `rust/test_data/grammar_quality/verilog_2005_conformance_contract_v0.json`: the `--lint-grammar` 0-profile-orphan lock, the curated accept/reject corpus matrix across `verilog_2005`/`sv_2017`/`sv_2023`, and the profiled certificate-coverage baseline (deterministic across seeds 0/7/42)
- VHDL quality gate:
  - `make -C rust SHELL=/bin/bash vhdl_stimuli_quality_gate`
  - the default gate-local Rust build cache under `rust/target/vhdl_stimuli_quality_gate/cargo_target` is pruned automatically when the gate exits; the retained evidence remains in `work/` and `logs/`
  - set `PGEN_VHDL_STIMULI_KEEP_CARGO_TARGET=1` if you want to keep that default gate-local cache deliberately
  - if you point `PGEN_VHDL_STIMULI_CARGO_TARGET_DIR` at a custom/shared target dir, that directory remains user-managed rather than being auto-pruned
- VHDL strict-promotion trials:
  - `make -C rust SHELL=/bin/bash vhdl_strict_promotion_gate`
- EBNF dual-run gate:
  - `make -C rust SHELL=/bin/bash ebnf_frontend_dual_run_gate`
- Return-annotation support gate:
  - `make -C rust SHELL=/bin/bash return_annotation_support_gate`
- Stimuli module parity gate:
  - `make -C rust SHELL=/bin/bash stimuli_module_parity_gate`
- EBNF frontend readiness (Rust path):
  - `PGEN_EBNF_FRONTEND_IMPL=rust make -C rust SHELL=/bin/bash ebnf_frontend_readiness`
- EBNF closed-loop quality (Rust path):
  - `PGEN_EBNF_FRONTEND_IMPL=rust PGEN_EBNF_STIMULI_QUALITY_COUNT=3 bash rust/scripts/ebnf_stimuli_quality_gate.sh`
- Regex external hardening lanes:
  - `make -C rust regex_corpus_bundle_contract_gate`
  - `make -C rust regex_pcre2_textsafe_corpus_gate`
  - `make -C rust regex_pcre2_compile_oracle_gate`

## Documentation Book
- The curated live book source is under:
  - `docs/book/`
- Build it locally with:
  - `mdbook build docs/book`
- Serve it locally with live reload:
  - `mdbook serve docs/book --open`
- Gate it with the repo-standard wrapper:
  - `make -C rust SHELL=/bin/bash mdbook_docs_gate`
- Intent:
  - the book is the primary public documentation surface for users and developers,
  - the book itself should explain the documentation split between public chapters, deep reference/contracts, and internal continuity docs,
  - the book should grow until every important aspect of PGEN is documented there with rationale and transparency,
  - continuity docs are internal session/continuity surfaces,
  - contracts/reference docs remain the deep authoritative detail behind the book.

## Per-Parser Integration Reference Books
- Alongside the platform mastery book (`docs/book/`), **every PGEN parser has
  its own live mdBook** — the canonical AST-integration reference for that
  parser. The standing directive (director 2026-06-08) is that every parser
  shall have its own per-parser mdBook, referenced from the top-level book's
  Parser Families chapter. Both the `src/*.md` source and the rendered
  `*-html/` are tracked in git so they are browsable directly on GitHub
  without an mdbook install.
- Books and their repo-standard gates:
  - regex — `docs/regex_parser_book/` — `make -C rust SHELL=/bin/bash regex_parser_book_gate`
  - systemverilog — `docs/systemverilog_parser_book/` — `make -C rust SHELL=/bin/bash systemverilog_parser_book_gate`
  - systemverilog_preprocessor — `docs/systemverilog_preprocessor_parser_book/` — `make -C rust SHELL=/bin/bash systemverilog_preprocessor_parser_book_gate`
  - vhdl — `docs/vhdl_parser_book/` — `make -C rust SHELL=/bin/bash vhdl_parser_book_gate`
  - rtl_frontend — `docs/rtl_frontend_parser_book/` — `make -C rust SHELL=/bin/bash rtl_frontend_parser_book_gate`
  - rtl_const_expr — `docs/rtl_const_expr_parser_book/` — `make -C rust SHELL=/bin/bash rtl_const_expr_parser_book_gate`
  - json (built-in, simplified) — `docs/json_parser_book/` — `make -C rust SHELL=/bin/bash json_parser_book_gate`
  - return_annotation — `docs/return_annotation_parser_book/` — `make -C rust SHELL=/bin/bash return_annotation_parser_book_gate`
  - semantic_annotation — `docs/semantic_annotation_parser_book/` — `make -C rust SHELL=/bin/bash semantic_annotation_parser_book_gate`
  - ebnf (meta-grammar) — `docs/ebnf_parser_book/` — `make -C rust SHELL=/bin/bash ebnf_parser_book_gate`
- Each shipped-family per-parser book is paired with the matching downstream
  contract under `docs/contracts/` (the deep authoritative integration
  surface) and the family's AST shape-contract manifest under
  `rust/test_data/ast_shape_contract/`. (The `json` book documents a built-in
  simplified grammar and is paired instead with `json_corpus_bundle/`. The
  `return_annotation` and `semantic_annotation` books are paired with
  `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md` and
  `docs/contracts/PGEN_SEMANTIC_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`.)
- The every-parser-book directive is now **complete**: every PGEN grammar —
  the shipped/annotation families above plus the `ebnf` meta-grammar — has its
  own live, gated mdBook. The `ebnf` book is the grammar-author's reference for
  the EBNF *input* language (tracked by the `EBNF-BOOK` task tree).

## Documentation Status
- Current authoritative docs for the active Rust-first platform:
  - `README.md`
  - `docs/book/`
  - `PGEN_USER_GUIDE.md`
  - `QUICKSTART_AI_ONBOARDING.md`
  - `docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md`
  - `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`
  - `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`
  - `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`
  - `docs/contracts/PGEN_PNR_PARSER_INTEGRATION_CONTRACT.md`
  - `docs/tcl/md/tcl.md`
  - `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
  - `LIVE_ACHIEVEMENT_STATUS.md`
  - `docs/reference/RUST_CODEBASE_ANALYSIS.md`
  - `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`
  - `docs/reference/PGEN_LINTER_ENABLEMENT_ROADMAP.md`
  - `docs/reference/PGEN_COMPILER_ELABORATOR_ENABLEMENT_ROADMAP.md`
  - `docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`
  - `rust/docs/EMBEDDING_API_CONTRACT.md`
- Historical/reference docs are still tracked for context, but some describe superseded workflows or earlier project phases.
- In particular, treat these as archival unless they are explicitly refreshed:
  - `rust/docs/TECHNICAL_ARCHITECTURE.md`
  - `rust/docs/CLI_REFERENCE.md`
- The complete markdown index below is a repository navigation index, not a claim that every listed document is equally current.
- Commit-workflow continuity rule:
  - `COMMIT.md` is binding operational policy for post-task commits,
  - post-commit user-facing reports must include the commit ID, exact commit message, the list of tracked files included in the commit, and the current live-status snapshot.

## Documentation Structure
- Curated live mastery book:
  - `docs/book/`
- Project governance, release policy, and live status:
  - `docs/reference/PGEN_RELEASE_POLICY.md`, `LIVE_ACHIEVEMENT_STATUS.md`, `CHANGES.md`
- Rust architecture/state assessment:
  - `docs/reference/RUST_CODEBASE_ANALYSIS.md`
- Core contracts and roadmaps:
  - `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`, `docs/reference/PGEN_ANNOTATION_100_PERCENT_CLOSURE_ROADMAP.md`, `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`, `docs/reference/PGEN_STIMULI_MODULE_NORMATIVE_SPEC.md`, `docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`, `docs/reference/SV_GRAMMAR_COVERAGE_MATRIX.md`
- Downstream parser integration contracts:
  - `docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md`, `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`, `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`, `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`, `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md`, `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md`, `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`, `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`, `docs/contracts/PGEN_SEMANTIC_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`, `docs/contracts/PGEN_PNR_PARSER_INTEGRATION_CONTRACT.md`
- Future downstream parser source notes:
  - `docs/tcl/md/tcl.md`
- Regex corpus acquisition and hardening:
  - `regex_corpus_bundle/README.md`, `regex_corpus_bundle/docs/regex_corpus_plan.md`, `regex_corpus_bundle/corpus/pcre2/invalid/README.md`, `regex_corpus_bundle/corpus/pcre2/quarantine/README.md`, `regex_corpus_bundle/oracle/pcre2/README.md`
- Operational continuity:
  - `LIVE_ACHIEVEMENT_STATUS.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `COMMIT.md`
- User/developer onboarding:
  - `SESSION_BOOTSTRAP.md`, `README.md`, `QUICKSTART_AI_ONBOARDING.md`, `PGEN_USER_GUIDE.md`, `docs/reference/STRESS_TEST_STANDARDIZATION.md`

## Active Markdown Index
The list below is the current high-signal markdown surface for active work. A 2026-04-06 audit found that most top-level `docs/*.md` files are legacy implementation notes, historical status snapshots, or duplicate design writeups and should not be treated as equal-priority sources of truth.
- `CHANGES.md`
- `COMMIT.md`
- `DEVELOPMENT_NOTES.md`
- `LIVE_ACHIEVEMENT_STATUS.md`
- `MEMORY_ARCHITECTURE.md` (durable harness-agnostic agent-memory standard — the memory/continuity system of record)
- `DOCTRINE_ENFORCEMENT.md` (portable doctrine-enforcement standard + kit — the general "rule enforcer" framework)
- `TOOLBOX.md` (the diagnostic & debug toolbox — when/how for every debug tool; read before debugging)
- `MEMORY.md` (layer A resume pointer)
- `PGEN_USER_GUIDE.md`
- `QUICKSTART_AI_ONBOARDING.md`
- `SESSION_BOOTSTRAP.md`
- `docs/book/book.toml`
- `docs/book/src/SUMMARY.md`
- `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
- `docs/reference/RUST_CODEBASE_ANALYSIS.md`
- `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`
- `docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`
- `docs/reference/PGEN_STIMULI_MODULE_NORMATIVE_SPEC.md`
- `docs/reference/PGEN_RELEASE_POLICY.md`
- `docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md`
- `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`
- `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`
- `regex_corpus_bundle/README.md`
- `regex_corpus_bundle/docs/regex_corpus_plan.md`
- `regex_corpus_bundle/corpus/pcre2/invalid/README.md`
- `regex_corpus_bundle/corpus/pcre2/quarantine/README.md`
- `regex_corpus_bundle/oracle/pcre2/README.md`
- `docs/AST_GENERATOR_ARCHITECTURE.md`
- `docs/ast_transformation_pipeline.md`
- `docs/BOOTSTRAP_MODE_SPECIFICATION.md`
- `docs/EBNF_INCLUDE_SYSTEM.md`
- `docs/parser_architecture_evolution.md`
- `docs/RETURN_ANNOTATIONS_REFERENCE.md`
- `docs/systemverilog/README.md`
- `docs/TEST_INFRASTRUCTURE.md`
- `docs/verilog/README.md`
- `docs/vhdl/README.md`

The top-level `docs/*.md` surface has now been pruned down to the maintained active reference set. The full audit trail and removal rationale remain recorded in `DEVELOPMENT_NOTES.md`.
- `make -C rust SHELL=/bin/bash ci_workflow_local_gate` now audits the tracked top-level `docs/*.md` allowlist so this surface does not silently drift back upward.
- `make -C rust SHELL=/bin/bash ci_workflow_local_gate` now also audits the tracked `docs/contracts/*.md` and `docs/reference/*.md` allowlists so the curated contract/reference buckets do not silently drift.
- `make -C rust SHELL=/bin/bash ci_workflow_local_gate` now also audits the curated `docs/book/` surface and replays the tracked `mdbook-docs-gate` workflow command so the live book stays buildable.
- `make -C rust SHELL=/bin/bash ci_workflow_local_gate` now also audits active operator/reference docs for stale pre-rehome path mentions, so live docs keep pointing at the canonical `docs/contracts/...` and `docs/reference/...` locations.

Root markdown policy note:
- the repository root should be reserved for entrypoint docs, live continuity docs, and tool/session-control docs
- tool-specific editor/assistant docs that no longer serve the active workflow should be removed rather than kept as root clutter
- the parser integration contract surface now lives under `docs/contracts/` instead of consuming repo-root markdown slots
- the maintained spec / matrix / policy reference surface now also lives under `docs/reference/` instead of consuming repo-root markdown slots
- the active roadmap and the live Rust architecture/state assessment now also live under `docs/reference/` instead of consuming repo-root markdown slots
- stale historical root overview/status/guidance docs have now been removed instead of being kept as dead navigation noise
- the remaining root markdown set is now the intentionally minimal entrypoint / continuity / active-operator surface, while deep-reference docs like the roadmap, Rust analysis, and regex bootstrap architecture live under `docs/reference/`
- a separate root `*.md` audit/classification now also lives in `DEVELOPMENT_NOTES.md`
- `make -C rust SHELL=/bin/bash ci_workflow_local_gate` now audits the tracked root markdown allowlist so this surface does not silently drift

Read SESSION_BOOTSTRAP.md and start from there.
