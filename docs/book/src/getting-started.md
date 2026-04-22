# Getting Started

The quickest way to get productive in PGEN is to treat the repository as a Rust-first active platform with strong documentation discipline.

The maintained Rust toolchain floor is explicit now: the repo's Cargo packages declare an MSRV of `1.95`. If you are building the Rust-owned surfaces directly, start from Rust `1.95` or newer.

## First Files To Read

1. `README.md`
2. `QUICKSTART_AI_ONBOARDING.md`
3. `PGEN_USER_GUIDE.md`
4. `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
5. `LIVE_ACHIEVEMENT_STATUS.md`

## First Commands To Know

```bash
make -C rust SHELL=/bin/bash sota_exit_gate
make -C rust SHELL=/bin/bash ci_workflow_local_gate
make -C rust SHELL=/bin/bash annotation_contract_gate
make -C rust SHELL=/bin/bash stimuli_cross_family_platform_gate
```

## What To Expect

- generated artifacts are version controlled,
- the maintained Rust MSRV is `1.95`,
- quality gates matter,
- continuity docs matter,
- user-facing claims are expected to align with executable proof.

## Primary Source Docs

- `README.md`
- `QUICKSTART_AI_ONBOARDING.md`
- `SESSION_BOOTSTRAP.md`
- `PGEN_USER_GUIDE.md`
