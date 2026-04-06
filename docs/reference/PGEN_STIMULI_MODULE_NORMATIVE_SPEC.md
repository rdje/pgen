# PGEN Stimuli Module Normative Specification (Living)

Last updated: 2026-02-22

## Purpose
This document defines the normative contract for generated Rust stimuli-module artifacts (`generated/<grammar>_stimuli.rs`) and their compatibility with in-memory stimuli generation.

It is binding for:
- CLI generation behavior (`ast_pipeline --generate-stimuli` and `ast_pipeline --generate-stimuli-module`),
- embedding consumers that import generated stimuli modules,
- parity/CI gates that enforce no-regression guarantees.

## Contract Scope
The stimuli-module contract covers three layers:

1. Artifact structure contract (module API shape and metadata constants).
2. Determinism/replay contract (seed and configuration compatibility).
3. In-memory vs module parity contract (sample/coverage/gap equivalence under matched config).

## Artifact Structure Contract
For `ast_pipeline INPUT --generate-stimuli-module`, generated module artifacts MUST contain:

- `pub const STIMULI_MODULE_API_VERSION: u32`
- `pub const GRAMMAR_NAME: &str`
- `pub const REQUESTED_SAMPLE_COUNT: usize`
- `pub const GENERATED_SAMPLE_COUNT: usize`
- `pub const GENERATION_SEED: u64`
- `pub const ENTRY_RULE: &str`
- `pub const STIMULI: [&str; N]`
- `pub fn generated_stimuli() -> &'static [&'static str]`

Output path contract:
- If `--output` is omitted, default artifact path is:
  - `generated/<sanitized_grammar_name>_stimuli.rs`.

Seed contract:
- If `--seed` is omitted for module mode, deterministic default seed is `1`.

## Determinism and Replay Contract
Given fixed inputs:
- same grammar content,
- same generation flags and values,
- same seed,
- same toolchain/pipeline code version,
the generated module source MUST be deterministic (byte-stable).

The replay identity tuple for stimuli generation is:
- grammar identity (`grammar_name` + content),
- `entry_rule`,
- `count`,
- `seed`,
- `max_depth`,
- `max_repeat`,
- `recovery_stimuli_mode`,
- parseability policy (`--validate-parseability` on/off),
- coverage merge input (`--coverage-input`, if any).

Implications:
- `--generate-stimuli` without `--seed` is entropy-based and not replay-stable.
- `--generate-stimuli-module` without `--seed` remains replay-stable via default seed `1`.
- Cross-mode deterministic replay requires explicitly matching all tuple fields above.

## In-Memory vs Module Parity Contract
When in-memory and module modes run with matched replay identity tuple:
- generated sample corpus MUST be equivalent,
- merged coverage JSON MUST be equivalent (canonicalized comparison),
- generated gap-report JSON MUST be equivalent (canonicalized comparison).

This contract is enforced by:
- `make -C rust SHELL=/bin/bash stimuli_module_parity_gate`
- contract manifest:
  - `rust/test_data/grammar_quality/stimuli_module_parity_contract.json`

The gate is promoted to aggregate required-check policy in:
- `rust/config/sota_exit_policy.env`
- `rust/scripts/sota_exit_gate.sh`

## Embedding Contract Guidance
Recommended import pattern in Rust:

```rust
#[path = "../generated/foolang_stimuli.rs"]
mod foolang_stimuli;
```

Use the exported metadata constants as compatibility guardrails:
- check `STIMULI_MODULE_API_VERSION` before consuming fields,
- use `GENERATION_SEED`/`ENTRY_RULE`/`REQUESTED_SAMPLE_COUNT` for deterministic replay in CI/debug flows.

## Non-Goals (Current Contract Boundary)
- This contract does not require zero parser rejects for arbitrary grammars unless parseability validation is explicitly enabled.
- This contract does not mandate semantic meaning of stimuli content beyond grammar-valid generation and gate-verified parity outcomes.

## Change Control
Any change to:
- exported module constant names/types,
- seed default behavior,
- parity equivalence semantics,
is a contract change and MUST update:
- this file,
- `PGEN_USER_GUIDE.md`,
- `PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`,
- associated parity gate tests/scripts.
