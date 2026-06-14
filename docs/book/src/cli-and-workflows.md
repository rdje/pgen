# CLI and Workflows

PGEN becomes much easier to reason about once you separate its surfaces into three layers:

1. direct CLI work through `ast_pipeline`,
2. repeatable workflow entrypoints through `make -C rust ...`,
3. policy and parity enforcement through tracked gates and workflows.

## `ast_pipeline` Is The Main Tooling Surface

The central CLI can drive:

- raw EBNF AST export,
- parser generation,
- in-memory stimuli generation,
- generated stimuli-module export,
- parseability-aware generation,
- target-driven replay and coverage reporting,
- newer stimuli controls such as:
  - grammar-aware mutation,
  - constrained-random steering,
  - near-valid negative generation,
  - corpus bundle export.

In practice, this means `ast_pipeline` is the shortest path from a grammar change to concrete evidence.

## Make Targets Are The Main Operational Surface

PGEN deliberately exposes most serious workflows through Make wrappers so users and contributors do not have to reconstruct long command lines every time.

Important examples include:

- `sota_exit_gate`
- `ci_workflow_local_gate`
- `annotation_contract_gate`
- `rtl_frontend_generated_contract_gate`
- `stimuli_cross_family_platform_gate`
- `mdbook_docs_gate`

`rtl_frontend_generated_contract_gate` proves three related surfaces over one curated 130-sample manifest (typed-AST-era contract `0.2.0`; parser release `1.0.5` / AST-dump schema `3`):

1. **Generated-parser contract behavior** — parse-acceptance for all 130 samples (98 accepts / 32 rejects — including bare ANSI ports `input R` / `output R` / `inout R`, accepted since the `1.0.4` `RTL-FE-CLOSURE.9` fix that gave `port_group` a no-type branch, and keyword-prefixed identifiers such as `input_data` / `reg_file` / `format`, accepted since the `1.0.5` `RTL-FE-CLOSURE.10` keyword word-boundary (`\b`) fix); rule participation for accepted parses via the generated parser's **transactional coverage record** (`required_rule_names` / `forbidden_rule_names` — the parser's own entry testimony, sound under PEG backtracking and complete under return-annotation folding); and 232 curated retained-text locks expressed as exact string values of the released schema-3 typed JSON carrier (`required_typed_string_values`).
2. **Handwritten-baseline parse replay** over the same manifest, currently with no divergence overrides.
3. **A ratcheted handwritten elaboration replay layer** for samples carrying `expected_elaboration` (59 curated semantic samples: 46 accepts / 13 rejects), with ratcheted minimums on top-parameter, child-path, child-parameter, and child-port-binding checks — so hierarchy/package-constant/instance-array/typed-actual cases prove more than "it elaborated", and non-constant override forms prove they fail in the expected way.

The `0.1.0` raw-envelope checks (walking the dumped AST for `rule_name`/`span` keys) were retired in the `0.2.0` migration: they were authored before the rtl_frontend typing campaign and became structurally unsatisfiable once return annotations folded the structural rules into the typed carrier — the proof-surface lesson being that a gate's assertion language must be migrated in the same wave as the representation it asserts over.

These wrappers matter because they become the stable shared vocabulary for:

- local development,
- CI workflows,
- release policy,
- continuity docs,
- user-facing reports.

## Local Workflow Parity Matters

One of the distinctive operational features in PGEN is the local workflow-parity lane:

```bash
make -C rust SHELL=/bin/bash ci_workflow_local_gate
```

This exists to approximate the tracked GitHub workflow surface from a tracked-only local export. That reduces the risk of local-only files, stale paths, or documentation drift hiding CI failures until after a push.

As of 2026-04-14, the hosted GitHub Actions workflows are intentionally manual-only (`workflow_dispatch`) to conserve account Actions minutes. The workflows still exist and can be started manually from GitHub when needed, but routine validation should use the local Make gates and `ci_workflow_local_gate` until hosted auto-runs are explicitly restored.

By default, successful `ci_workflow_local_gate` runs now delete their own scratch `run.*` export directories under `rust/target/ci_workflow_local_gate` after the selected workflows complete. Failed runs are intentionally retained so the exported tracked tree and logs remain available for diagnosis. Set `PGEN_CI_WORKFLOW_LOCAL_KEEP_RUNS=1` when you want to preserve a successful run on purpose.

The same “keep the evidence, drop the disposable build cache” rule now also applies to the direct VHDL quality lane. `vhdl_stimuli_quality_gate` still isolates its adapter-backed Rust build under a gate-local `cargo_target` so nested runs do not clobber each other, but that default `rust/target/vhdl_stimuli_quality_gate/cargo_target` directory is pruned automatically when the gate exits. The retained proof surface remains `work/` plus `logs/`. Set `PGEN_VHDL_STIMULI_KEEP_CARGO_TARGET=1` only when you intentionally want to keep that gate-local cache around.

## Debugging A Parser With `parseability_probe`

For any work involving a parser's behavior — investigating a parse failure, identifying performance hot spots, understanding why a `@predicate` rejected a branch — `parseability_probe` is the dedicated diagnostic CLI. See the [**Debugging With `parseability_probe`**](parseability-probe-debug.md) chapter for the complete reference, covering:

- the five trace verbosity levels (`none`/`low`/`medium`/`high`/`debug`) and their additivity guarantees,
- rule-scoped tracing via `--trace-rules <list>` (100-1000× volume reduction vs full `--trace`),
- the live per-rule call-count dashboard (`--dump-rule-call-counts [N]`) with rule-exclusion filtering,
- the always-on `furthest_position` error diagnostic that points to the actual defect locus (not the surface failure position),
- the self-explaining predicate trace (`🛡️ predicate 'X' PASSED/REJECTED/INAPPLICABLE branch K/N`),
- workflow recipes for stuck parses, deep failures, and predicate-rejection mysteries.

## Working Style That Fits PGEN Best

The most reliable pattern is:

1. use `ast_pipeline` for focused development and diagnosis,
2. use the relevant Make target for proof,
3. use `ci_workflow_local_gate` when the change affects tracked workflow surfaces,
4. update docs and contracts when a user-facing surface changed.

## Primary Source Docs

- `PGEN_USER_GUIDE.md`
- `README.md`
- `rust/docs/CLI_REFERENCE.md`
- `rust/scripts/ci_workflow_local_gate.sh`
