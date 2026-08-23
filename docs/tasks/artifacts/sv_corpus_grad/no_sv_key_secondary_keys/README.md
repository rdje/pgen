# `SV-CORPUS-GRAD.13e` — secondary keys for the `deferred:no_sv_key` corpus rows

`.13a` stratified this class and asked one question of its DARK members: **does iverilog offer any
secondary key**, and if not, what is the honest permanent deferral. The answer is **yes**, and the
largest sub-class turns out not to be a missing key at all.

| instrument | question | verdict at HEAD |
|---|---|---|
| `stimuli/sv/audit_no_sv_key.py` | is there a secondary key, and what does the unflagged dialect resolve to? | **`GN_VER2005`**; 37 of 52 basis-B DARK rows are named by another ivtest list |

Re-run — byte-identical across runs, no timestamps, no wall-clock:

```bash
python3 stimuli/sv/audit_no_sv_key.py > docs/tasks/artifacts/sv_corpus_grad/no_sv_key_secondary_keys/report.txt
```

## The finding — "the dialect is unresolved" was true about the DESCRIPTOR and false about the QUESTION

**147** rows (27 of them DARK) carry the adjudicator's basis *"vvp_tests descriptor(s) without an
explicit generation flag — dialect unresolved (the upstream default generation is not encoded in the
descriptor)"*. Every word of that is correct about the descriptor. It is the wrong place to look:
the default generation is compiled into `iverilog`, and **`iverilog` is vendored in this same
corpus**.

```text
stimuli/sv/subs/iverilog/compiler.h   enum generation_t { … GN_VER2005 = 4, … GN_DEFAULT = 4 }
stimuli/sv/subs/iverilog/main.cc      generation_t generation_flag = GN_DEFAULT;
```

⇒ a run with no `-g` flag compiles as **plain IEEE 1364-2005 Verilog**. The rows are
`v2005_profile_lane` (**ROUTED** — answered in the v2005 manifest), not `no_sv_key` (**NO VERDICT** —
answered nowhere).

⭐ **The contrast is what makes it safe to say.** `regress-sv.list`, the primary key, encodes the
dialect explicitly in **907 of its 922** entries (`-g2005-sv` / `-g2009` / `-g2012` / `-g2017` /
`-g2023`). That is *why* it is the SV key. A descriptor carrying no flag is not an SV entry missing
its label; it is a different lane.

## The one mechanism that would invert this, and it is checked on every run

`vvp_reg.py` has `force_gen()`, which strips any generation and inserts `-g2023`. If it ran by
default, every unflagged descriptor would be SystemVerilog and this verdict would be backwards. It
does not: it is guarded by `cfg['force-sv']`, `--force-sv` is an `action='store_true'` flag (default
off), and the vendored `Makefile.in`'s own `check-installed-vvp-py` target invokes
`python3 vvp_reg.py $(opts)` with no such flag. **All three facts are re-derived at run time and the
script REFUSES if any of them moves.**

## The other basis — enumerated, not argued

596 rows (52 DARK) carry *"no regress-sv.list entry"*. The vendored tree holds **11** list files and
the adjudicator reads **2**. Enumerating the rest:

```text
regress-ivl1.list                                          16
(NONE — no ivtest list names it)                           15
regress-ivl1.list, regress-vlog95.list                      4
regress-synth.list, vhdl_regress.list                       4
vpi_regress.list                                            4
blif.list                                                   2
regress-ivl1.list, vhdl_regress.list                        2
regress-synth.list, regress-vlog95.list                     2
regress-fsv.list, regress-ivl1.list, regress-vlog95.list    1
regress-synth.list                                          1
regress-fsv.list, regress-synth.list                        1
```

⇒ **37 of 52 are named by another list; 15 by none.** The honest permanent deferral is over **15**
rows, not 743 — which is the leaf's own stated ask, two orders of magnitude tighter.

## ⛔ What this audit deliberately does NOT conclude

- **It does not read a verdict out of a secondary list.** The lists are different back ends and they
  disagree about what their verdicts mean: `regress-fsv.list` is read **only** under `--force-sv`
  (so it testifies about *forced* SystemVerilog), `regress-vlog95.list` is a Verilog-95 **output**
  lane where a `CE` means the back end refused rather than the parser, and
  `vhdl_regress.list` / `blif.list` / `vpi_regress.list` are other back ends again. Reading a `CE`
  from one of those as `must_reject` would **manufacture a parser defect**.
- **It moves no row.** Reclassification edits `stimuli/sv/adjudicate_external_corpus.py`, the one
  file class that can move the SV graduation bar with zero parser change
  (`DOCTRINE-GAP-OWNERSHIP.6`), and it moves a published denominator. Sequenced as `.13e.1`.
- **It is not the relabelling `.13` forbids.** That warning is against pinning a deferred row to an
  expectation and calling it *adjudicated*. Nothing here becomes adjudicated: a row moves to a
  **weaker and checkable** deferral, because the stated ground for its original class is refuted by
  measurement.

## The refusal arms, all fired

A re-derivation that has only ever been seen green is an assertion. Each arm was driven red in a
throwaway mini-repo under `rust/target/` holding mutated copies of the four vendored inputs
(removed afterwards); the unmutated mini-repo reproduces the real verdict and returns to exit 0.

| arm | mutation | observed |
|---|---|---|
| A1 | `GN_DEFAULT` deleted from the enum | `REFUSE: GN_DEFAULT is not a member of enum generation_t` |
| A2 | `GN_DEFAULT = 99`, aliasing no generation | `REFUSE: … aliases 0 generations … this script will not pick one` |
| A3 | `generation_flag = GN_VER2023` instead of `GN_DEFAULT` | `REFUSE: … the enum value may be inert` |
| B1 | `--force-sv` becomes `store_false` | `REFUSE: … no longer an off-by-default store_true flag … INVERTS this audit's verdict` |
| B2 | `force_gen()` un-guarded | `REFUSE: … may now run unconditionally, which INVERTS this audit's verdict` |
| B3 | the Makefile target passes `--force-sv` | `REFUSE: … the upstream default run IS SystemVerilog` |
| C1 | one row given an unbucketed basis string | `REFUSE: … classify it rather than letting it fall into a total` |
| C2 | the class emptied | `REFUSE: no deferred:no_sv_key rows … the class moved or the run is stale` |

⭐ A3, B1, B2 and B3 exist because each is a single upstream edit that would flip the verdict
silently. The audit's value is not the number it prints today; it is that the number cannot survive
the premise changing underneath it.
