# Stimuli and Quality

The stimuli system is one of PGEN's defining features.

## Why It Matters

PGEN is not satisfied with "the parser compiles." It aims for:

- grammar-aware stimuli generation,
- parseability-aware generation,
- target-driven replay,
- coverage and gap analysis,
- promotion and counterexample retention,
- cross-family quality proof.

## Current Stimuli Doctrine

The live direction for stimuli work now includes the first five planned upgrades in bounded initial form:

1. grammar-aware mutation
2. constrained-random steering
3. stronger near-valid negative generation
4. corpus export / promotion groundwork
5. smarter shrinkers, starting with delimiter-aware structural minimization

The shrinker work is deliberately not complete yet. The first landed slice teaches the existing counterexample minimizer to try balanced `()`, `[]`, and `{}` reductions before and after generic chunk minimization. Future work should push deeper into grammar-tree-aware shrinkers that can drop optional nodes, collapse alternations, reduce repetitions, and prune subtrees while preserving the failing property.

## Cross-Family Rule

Major stimuli-generator upgrades should prove themselves on at least:

- `regex`
- `vhdl`
- `systemverilog`

That rule keeps stimuli work platform-grade instead of grammar-specific.

## Key Quality Lanes

- `stimuli_cross_family_platform_gate`
- family-specific quality gates
- parseability reports and target-driven replay
- bounded contract files and summary artifacts

## Probe-Only Steering

When a family is down to a stubborn replay frontier, PGEN now distinguishes between two kinds of literal steering:

- `@sample` for ordinary always-on literalish steering
- `@probe_sample` for active-entry-only target-drive replay

That split matters because a hint that is useful when probing a single dependency rule can be harmful if it fires everywhere during normal top-level generation. The current maintained rule is:

- use `@sample` when the grammar really should always short-circuit to that literal shape
- use `@probe_sample` when the literal is meant to accelerate targeted replay of a specific rule without flattening ordinary coverage

## Replay Progress Tracing

The heavy replay gates are intentionally quiet by default, but the retained SystemVerilog replay lane now has an opt-in progress surface when a closed-loop stage is CPU-hot and needs inspection.

- `rust/scripts/sv_stimuli_quality_gate.sh` accepts `PGEN_SV_STIMULI_QUALITY_REPLAY_TRACE_VERBOSITY`
- keep the default at `none` for ordinary runs
- use `low` when you want replay-stage start/progress/completion lines from the Rust target-drive loop without turning the whole gate into an always-on debug dump

That trace is meant for honest progress visibility during stubborn replay work, not as a replacement for the gate's final summary artifacts.

## Primary Source Docs

- `docs/reference/PGEN_STIMULI_MODULE_NORMATIVE_SPEC.md`
- `docs/reference/STRESS_TEST_STANDARDIZATION.md`
- `docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`
- `regex_corpus_bundle/README.md`
