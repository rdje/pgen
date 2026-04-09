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
