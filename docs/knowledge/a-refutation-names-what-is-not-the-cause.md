---
id: a-refutation-names-what-is-not-the-cause
title: Refuting a failure's label from the input's own bytes narrows the cause — it never names it, and it refutes only the labels whose claim those bytes actually bear on
answers:
  - "this corpus row is deferred as needing the preprocessor — can I disprove that from the file itself"
  - "the file contains no directive at all and the parse still fails — is the deferral label wrong"
  - "my falsification test flagged N rows across several buckets — can I publish that as N misclassifications"
  - "how do I tell a multi-file TEXT dependency from a multi-file FACT dependency"
  - "why does the parse fail on a type name that is declared in a different file"
  - "does a zero-directive file mean the same thing in every deferral class"
  - "how do I decide whether expansion or unit-level fact continuity unblocks a deferred corpus row"
tags: [corpus-adjudication, instrument-soundness, semantic-store, systemverilog, measurement, toolbox]
date: 2026-08-11
status: current
evidence: docs/tasks/SV-CORPUS-GRAD.md leaf .13a (the 530-row census, the 3-row probe, the 8 rows read before publishing); docs/tasks/artifacts/sv_corpus_grad/verdict_coverage/{coverage.md,strata.tsv}; stimuli/sv/corpus_verdict_coverage.py (TICK_MEANING + the import-time refusal); stimuli/sv/subs/opentitan/hw/dv/sv/csrng_agent/csrng_if.sv (the canonical row — 0 backticks, dies on a sibling file's type name)
reverify: "F=stimuli/sv/subs/opentitan/hw/dv/sv/csrng_agent/csrng_if.sv; python3 -c \"print('backticks:', open('$F','rb').read().count(b'\\`'))\" && ./rust/target/release/parseability_probe --parse systemverilog $F --profile sv_2017 2>&1 | tail -1 && PGEN_TRACE_VERBOSITY=high ./rust/target/release/parseability_probe --parse systemverilog $F --profile sv_2017 --trace 2>&1 | grep -c 'has_fact(kind=type_name'"
---

A failure carrying an *explanation* — "this row is deferred because it needs the preprocessor", "this
file only parses as part of a multi-file unit" — can often be **refuted from the input's own bytes**,
with no re-run and no new instrument. The refutation is cheap, sound and worth doing. It is also
routinely over-read.

## The falsification, and its strongest form

A preprocessor's output is byte-identical to its input **up to the first token it can alter**;
`` `define `` lines are deleted, macros expand and `` `ifdef `` arms vanish at or after that offset,
never before it. So a parse that dies *before* that offset dies identically on the expanded text.

The degenerate case is the sharpest: **a file containing no `` ` `` byte at all cannot be altered by
expansion, conditional resolution or `` `include `` inlining in any way whatsoever.** Measured over
the 4 398 unadjudicated-and-failing rows of the SV corpus, **530 `deferred:chained_only` rows are in
exactly that state** (friscv 346, opentitan 152, uvm-core 13, Surelog 12, …) — each one deferred on a
text-substitution argument its own bytes cannot support.

## ⛔ But a refutation names the absence of a cause, never the cause

*"Expansion is not what is failing here"* is not *"nothing legitimate is failing here"*, and treating
the two as the same manufactures findings. The next step is the toolbox, not a conclusion. Three of
the 530, probed:

```
./rust/target/release/parseability_probe --parse systemverilog \
    stimuli/sv/subs/opentitan/hw/dv/sv/csrng_agent/csrng_if.sv --profile sv_2017
# did not consume full input at position 279 [furthest_position=468]
#   text at 468:   wire csrng_req_t   cmd_req;
PGEN_TRACE_VERBOSITY=high … --trace | grep -E "🚫|has_fact"
# 🔍 has_fact(kind=type_name, name=Identifier("clk")) → false
# 🚫 Rule 'checked_type_identifier' rejected by post predicate 'has_fact [type_name, …]'
```

All three die on a **user-defined type name a sibling file declares** (`csrng_req_t`,
`flash_phy_prim_flash_req_t`, a parameterized `dv_lc_tx_if_cov #(…)`). The multi-file dependency is
real — but it is a **cross-file FACT** dependency, not a text dependency: in a store-gated parser a
`typedef` in another file supplies the `type_name` a predicate requires. Both dependencies wear the
label *"needs the other files"*, and they are satisfied by completely different capabilities —
**text expansion** versus **parsing a file list into one fact store**.

⇒ the sequencing consequence, which is the reason this is worth a record: a standing claim that the
whole 5 276-row class unblocks on the preprocessor was **corrected by measurement**. Sized properly
(`SV-CORPUS-GRAD.13c`, one traced parse per refuted row): **2 057 rows unblock on unit-level fact
continuity and need no preprocessor at all**, 1 527 genuinely need expansion, 346 are `$readmemh` hex
images that can never parse, and 57 are candidate parser defects.
**A convergence claim is only as good as the population it was measured over.**

## ⛔⛔ The same test can be decisive for one label and irrelevant for five

The zero-directive test also flags 8 rows labelled `deferred:svpp_owned` — an inviting
`575 rows misclassified` headline, one `grep` away. Reading all 8 refutes it: 5 are verible
`// verilog_syntax: parse-as-module-body` excerpt fixtures and 3 are verilator `t_preproc_*_bad`
unterminated-string/EOF cases. **Their preprocessor relevance is the test's purpose, not a directive
in its text**, so the bytes bear on nothing they claim. Same for `no_sv_key` (the label is the absence
of an upstream answer key), `impl_varying` (the LRM leaves the verdict open) and `verilog_ams_lane`
(a different dialect).

A measurement whose *meaning* varies per class must carry that meaning **in the instrument**, not in
the reader's head. `stimuli/sv/corpus_verdict_coverage.py` pairs every no-verdict class with an
explicit reading (`TICK_MEANING`) and **refuses at import** if a class has none — so a new class
cannot arrive and silently inherit the one interpretation that happens to be exciting.

## ⛔⛔ And the classifier that sorts the refuted rows must be CALIBRATED, not designed

Refuting the label is the easy half. Sorting *what is actually going on* in the refuted rows is where
a heuristic sneaks in, and it announces itself by moving too much. Sizing this population
(`SV-CORPUS-GRAD.13c`, 4 158 rows) the candidate-defect bucket read, in order:

**1 902 → 1 933 → 53 → 1 885 → 335 → 228 → 57.**

Every intermediate answer was produced by a defensible improvement, and every one but the last was
wrong. The instructive four:

- **Three probed rows are a shape, not a distribution.** The first classifier generalised from the
  three rows whose failure landed *after* the unknown type name, and mis-sorted the corpus norm
  (`sw_region_cfg_t [MpRegions-1:0] region_i`) because a packed dimension sat in between.
- **Enumerate the tool's vocabulary from a real run.** Rewriting it to use the parser's own predicate
  trace was right — and it matched `has_fact` but not `fact_attribute_equals`, the spelling that gates
  `class X extends BASE`. That single missing arm put **1 203 rows, 63 % of the worklist**, into
  "candidate defect" *while the trace named the cross-file base class one line above*.
- **A PEG parser speculates, so a whole-run signal excuses everything.** It tries a type reading on
  ordinary tokens: `static task host();` yields demanded-and-missing "type names" `static` and `host`.
  Only a signal intersected with the FAILURE REGION is evidence about the failure.
- **If a claim is checkable, check it.** "A sibling file would supply this name" became a corpus-wide
  declaration index (11 359 files, ~2 s): `mem_model_base_test` resolves to `dccm_base_test.sv`,
  `host` resolves to nothing. Building it exposed two more defects in the reader itself — a type
  *used inside* a `typedef struct` body being read as the typedef's own name, and a bounded
  `[^;]{0,300}` scan that can never reach a struct typedef's name at all.

⭐ **The habit that caught all six: before believing a bucket, run the real tool on ONE member of
it.** Not a sample, not a code review — one probe, about two minutes. Publishing the third answer
would have claimed 1 885 hidden parser defects and been wrong by 33×.

## The general rule

**Refute from the input where you can; then name the mechanism with a tool — and calibrate the sorter
against the tool before you publish its counts.** Publish the two
separately, because they have different strengths: the refutation is a proof about what cannot be the
cause, and the mechanism is an observation that has to be earned per row. And before you aggregate a
falsification across buckets, ask of each one whether its label's claim is *about* the bytes you
tested — the answer is usually no for most of them, and one number covering all of them is wrong
everywhere except where it was born.

See also [[a-furthest-position-names-a-region-not-a-token]] (resolve the layout gap before comparing
a parser position to a source offset — the same audit family's first, artefactual answer),
[[a-fused-counter-is-not-evidence-about-any-of-its-parts]] (split an aggregate before concluding),
[[a-rising-pass-rate-is-not-evidence-of-correctness]] and
[[a-shared-predicate-may-answer-two-questions]].
