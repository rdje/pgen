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

## Primary Source Docs

- `docs/reference/PGEN_STIMULI_MODULE_NORMATIVE_SPEC.md`
- `docs/reference/STRESS_TEST_STANDARDIZATION.md`
- `docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`
- `regex_corpus_bundle/README.md`
