# PGEN Stimuli Module Normative Specification (Living)

Last updated: 2026-04-09

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
- `stimuli_constraint_profile`,
- `stimuli_mutation_mode`,
- parseability policy (`--validate-parseability` on/off),
- coverage merge input (`--coverage-input`, if any).

Implications:
- `--generate-stimuli` without `--seed` is entropy-based and not replay-stable.
- `--generate-stimuli-module` without `--seed` remains replay-stable via default seed `1`.
- Cross-mode deterministic replay requires explicitly matching all tuple fields above.
- The replay identity tuple is intentionally larger than the exported module metadata constant set.
- Consumers that need exact replay under non-default stimuli controls MUST retain the full invocation config out of band rather than relying only on generated module constants.

## Current Constrained-Random Steering Contract (2026-04-09)
- `--stimuli-constraint-profile baseline` preserves the existing weighting behavior.
- `--stimuli-constraint-profile rare_branch_biased` increases pressure toward under-hit OR branches, especially when they still have:
  - zero or low success counts
  - zero or low selection counts
  - remaining target deficit
  - uncovered referenced-rule debt
- `--stimuli-constraint-profile deep_nesting_biased` biases generation toward deeper structures by:
  - preferring higher quantifier repeat counts
  - boosting branches that keep recursive or nested structure alive when depth budget still allows it
- The currently landed steering slice is intentionally bounded:
  - it is global profile steering, not a full rule-specific constraint DSL yet
  - it composes with the existing stimuli generator rather than replacing it
  - it is validated through bounded real-family replay on:
    - `regex`
    - `vhdl`
    - `systemverilog`
- This should be treated as the second executed item from the preserved stimuli-strengthening backlog, not as the finished end-state of constrained-random support.

## Current Mutation Contract (2026-04-09)
- `--stimuli-mutation-mode baseline` preserves the existing stimuli behavior.
- `--stimuli-mutation-mode grammar_aware_local` performs one local grammar-aware trace/replay perturbation over an otherwise valid sample.
- The currently landed local mutation sites are:
  - alternate OR-branch selection
  - alternate quantifier repeat counts
- The current grammar-aware mutation slice is intentionally bounded:
  - it activates only when `--recovery-stimuli-mode baseline` is in effect
  - non-baseline recovery modes retain their existing recovery semantics
- This first landed slice is validated through bounded real-family replay on:
  - `regex`
  - `vhdl`
  - `systemverilog`
- This should be treated as the first executed item from the preserved stimuli-strengthening backlog, not as the finished end-state of mutation support.

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

## Non-Normative Strategic Guidance (Verbatim, 2026-04-08)
This section intentionally preserves the full strategic guidance from the 2026-04-08 stimuli-generation discussion so future sessions do not have to reconstruct the steering from chat history. The prose below is preserved in full; only interactive file-link formatting was normalized to repo-local path references for repository readability.

> Yes, I think your updated read is right.
>
> PGEN’s real advantage is not “random input generation.” It is that we already have a grammar-aware engine in `rust/src/ast_pipeline/stimuli_generator.rs`, plus coverage/gap/replay machinery in `rust/src/main.rs`. That is a much better foundation than starting from generic fuzz crates and trying to claw grammar-awareness back afterward.
>
> If we want to make it meaningfully stronger, these are the highest-value upgrades:
>
> 1. Add grammar-aware mutation, not just fresh generation.
> A very strong next step is to mutate existing valid samples while preserving local structure:
> - swap one alternative for another
> - perturb quantifier counts
> - splice subtrees from other valid samples
> - rename identifiers consistently within a sample
> - keep delimiters balanced while stressing nearby syntax
>
> 2. Add a real constrained-random layer.
> Your SystemVerilog-style instinct was good. PGEN could let grammars or profiles express preferences like:
> - bias toward rarely hit branches
> - force mixed forms such as “named + positional”
> - cap or encourage nesting
> - require certain rule families to co-occur
> - steer toward parser-history hotspots
>
> 3. Get better at negative generation.
> Today the strongest bugs often come from “almost valid” inputs. PGEN should generate:
> - single-token deletions
> - trailing separators
> - wrong-closing delimiters
> - off-by-one quantifier forms
> - scope/flag mismatches
> - near-sync recovery probes
>
> 4. Add metamorphic invariants.
> This is very SOTA and fits PGEN well. Example checks:
> - parse -> pretty/normalize -> parse should preserve key AST facts
> - alpha-renaming should not change structure
> - whitespace/comment injection should preserve shape where grammar says it should
> - minimized counterexamples should still fail for the same reason family
>
> 5. Promote coverage-guided replay into a first-class corpus system.
> The existing coverage-guided fuzz loop is already promising. I’d build on it by making it easy to:
> - export minimized seed corpora
> - tag seeds with rule/branch novelty
> - promote good seeds into contract suites
> - replay exact corpora deterministically in CI
>
> 6. Add better shrinkers.
> Shrinking is where a lot of value appears. PGEN can do better than generic byte minimization because it knows grammar structure. Shrink by:
> - dropping optional nodes
> - collapsing alternations
> - reducing repetition counts
> - pruning whole subtrees while preserving the failing property
>
> 7. Add oracle-strength, not just stimulus-strength.
> A lot of recent regex misses were accepted-tree shape bugs, not parse/no-parse bugs. So stronger stimuli should be paired with stronger checks:
> - required/forbidden rule names
> - retained rule texts
> - targeted AST-shape locks
> - parser-registry and embedding-path parity
>
> So yes: the EBNF-based stimuli generator was a very good idea. I do agree. It was the right architectural move, because it gives PGEN something generic fuzzing alone does not: valid structure, replayability, and grammar-specific steering.
>
> If I were prioritizing, I’d do:
> 1. grammar-aware mutation
> 2. constrained-random steering
> 3. stronger near-valid negative generation
> 4. corpus export/promotion
> 5. smarter shrinkers
>
> Validation policy for that future work:
> - treat stimuli-generator upgrades as shared platform changes, not grammar-local experiments
> - major generator improvements should be exercised against at least:
>   - `systemverilog`
>   - `vhdl`
>   - `regex`
> - active families should benefit from the stronger generator, and already-closed families should serve as regression sentinels
> - the current executable cross-family enforcement surface for that policy is:
>   - `make -C rust SHELL=/bin/bash stimuli_cross_family_platform_gate`
>   - this bounded gate intentionally reuses the real family lanes instead of inventing a separate toy harness
>   - the current bounded SystemVerilog slice in that wrapper uses `sv_parseable_file` rather than the much heavier `sv_file` surface, so the shared platform lane stays representative without drifting into full-family replay cost
>
> That path gives you a genuinely stronger PGEN-native fuzz/stimuli system without needing `libfuzzer-sys` or `arbitrary` right now.

## Change Control
Any change to:
- exported module constant names/types,
- seed default behavior,
- parity equivalence semantics,
- replay identity tuple fields or active steering/mutation semantics,
is a contract change and MUST update:
- this file,
- `PGEN_USER_GUIDE.md`,
- `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`,
- associated parity gate tests/scripts.
