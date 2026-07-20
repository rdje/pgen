# Member-5 trace-off rule-context latch — measured result

Work unit: `PGEN-RGX-0078-0176` (`RGX-0078.5.j.4`)

## Outcome

**MEASURED-FALSIFIED and reverted.** The safe per-outermost transaction latch
was implemented exactly as designed and passed the complete correctness
battery, but its performance effect was inside the pre-registered 2.28%
same-binary noise span:

- 8-pattern bench, alternated 5x2000, geomean of per-pattern best minima:
  **1,922.6 -> 1,906.9 ns (-0.82%, candidate/base 0.991839)**;
- full 2,189-cell PCRE2 corpus geomean:
  **1,252.9 -> 1,241.4 ns (-0.92%)**;
- corpus candidate max:
  **477,667 ns**, below the settled 483,583 ns bound;
- corpus verdict flips: **0**.

The pre-registered land bound was candidate/base `<= 0.9772`. The measured
`0.991839` ratio failed it, so the Rust change was reverted. No campaign floor
number is banked from this candidate.

## Candidate and custody

- preserved floor probe sha256: `1d3fa0eebb4de19ff003ace38721d80d71bd94653f8ea3801c0f29ca9581f9b8`;
- candidate probe sha256: `d73d292919c2dc715a6a803649724dd778dfec594fd853402bcbc765f29c7f50`;
- generated regex artifact sha256: `e4924024a4bf7a91a75bb8b0dfb8460c71f19ef83e08313661fbd5d4e1ad590f`
  before and after (lib-only change; no regeneration);
- after revert, `rust/target/release/regex_perf_probe` was restored to the
  source/artifact-coherent `b33387f3...` baseline binary.

`run_ab.sh` asserts the full base hash, the generated-artifact prefix, and a
candidate-newer-than-source check before measuring. Raw benchmark and corpus
rows are in this directory; `analyze_ab.py` reproduces `acceptance.txt`.

## Correctness and trace proof before the performance decision

- focused rule-context tests: **4 passed** (three candidate tests plus the
  existing context test);
- full dual-feature lib suite: **1,002 passed, 0 failed, 29 ignored**,
  including all-11 generated/interpreter byte equivalence, combinator, and
  semantic gates;
- certificate seeds 0/7/42: each exactly
  `total=268 proof=9 witness=259 UNKNOWN=0 fully_certified=true`;
- PCRE2 compile oracle: exact `2189/1879/262/48` tuple;
- regex AST-shape: **4 aligned, 0 drift**;
- duality hunt: all **9 lanes** plus determinism repeat passed, with the pinned
  signature sets unchanged;
- strict source Clippy: **pass**; generated stage retained its known non-gating
  290-error baseline;
- trace contract: focused tests pinned trace-off empty-stack behavior, exact
  nested path `outer > inner`, mid-tree latch symmetry, and between-tree
  resampling. Message formatting and trace call sites were unchanged.

## What the result establishes

The reader audit and defect remain true: the `Vec<Cow>` stack is maintained on
ordinary static rule entries even though only diagnostics consume it. What is
falsified is its value as a standalone regex speed lever. The A/B measures the
whole safe mechanism, so the only defensible conclusion is that removing the
stack traffic **minus the required depth/latch bookkeeping** is less than the
campaign's measurable noise floor on both steering populations. The individual
costs must not be inferred from this aggregate.

The campaign floor therefore remains approximately **1,263.4 ns corpus
geomean** (bench approximately 1,937.4 ns; settled corpus max 483,583 ns).
The next slice must re-price the residual BATCH-1 emitter members before
spending a regeneration chain.
