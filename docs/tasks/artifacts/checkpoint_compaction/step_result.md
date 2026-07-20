# PGEN-RGX-0078-0193 result

## Verdict

**HOLD; no implementation.** `scope_len` is exactly redundant with
`chain_len`, so a lossless six-word checkpoint is feasible, but exact removable
traffic prices only **0.859310862 ns**. The deliberately optimistic paired-
store ceiling reaches only **0.880628388 ns**. This is too little margin to
license a candidate chain.

## Foundational audit correction

The preserved fused regions contain **40** checkpoint materializations:
39 trace-anchored `try_parse` sites plus one trace-less C3-B tournament site at
`0x100073f40`. The old `-0183` trace-based detector did exactly what it claimed
but its result was later overgeneralized as a complete checkpoint census.
Reclassification moves 0/1/1 samples from unmatched to required checkpoint
traffic and changes no prior price or total re-sum.

## Reproduction

```bash
python3 docs/tasks/artifacts/checkpoint_compaction/inspect_checkpoint_word.py \
  > /tmp/checkpoint_compaction.txt
cmp /tmp/checkpoint_compaction.txt \
  docs/tasks/artifacts/checkpoint_compaction/checkpoint_compaction.txt
```

The executable analysis pins source, probe, raw captures, compiler, layout
probe, source invariants, all 40 machine sites, the prior accepted/held
mechanisms through `-0192`, and the exact arithmetic.

## Scope and invariants

- Parser/runtime/emitter/generated artifact: unchanged.
- PCRE2 corpus verdicts, geomean floor, and settled MAX: unchanged.
- mdBook/contracts/reference architecture/LIVE tracker: unchanged; this is
  internal read-only pricing and a historical evidence correction.
- Next work must be separately task-tree-owned before any implementation.
