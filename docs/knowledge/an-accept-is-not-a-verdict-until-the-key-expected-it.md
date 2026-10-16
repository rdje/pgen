---
id: an-accept-is-not-a-verdict-until-the-key-expected-it
title: An accept is not a verdict until something independent of the parser expected it — the must-accept behind a chained accept lives in the KEY (`chained_must_accept`), derived per file from the suite's own testimony, never from a suite list
answers:
  - "the pipeline accepts a row under its derived context and the coverage still says NO VERDICT — why"
  - "what is the expected verdict chained_must_accept and which lane cashes it"
  - "how does a file of a Surelog multi-file test unit get a per-file expectation"
  - "why not key every source of a unit whose golden log is clean"
  - "why was DESIGN_SUITES removed from the coverage instrument's reading"
  - "what in a Surelog .sl driver decides the key, and what makes a golden log testimony"
  - "a pre-screen predicted 79 rows and the instrument keyed 71 — what was the difference"
  - "a tuple under the pipeline fell by 3,700 rows after a column was added to the key — what happened"
tags: [systemverilog, answer-key, corpus, surelog, pipeline, coverage, done-bar]
date: 2026-10-17
provenance: SV-CORPUS-GRAD.57 (PGEN-SV-CORPUS-GRAD-0444); SVPP-EXPANSION.6c.1 (the design-corpus convention it generalises), SV-CORPUS-GRAD.8b.1 (the single-source Surelog key)
reverify: "python3 stimuli/sv/adjudicate_external_corpus.py --self-test   # surelog-units self-test: 14 ok, 0 failed; python3 stimuli/sv/corpus_conformance_compare.py --self-test   # corpus_conformance_compare: self-test OK; bash scripts/check_sv_corpus_denominator.sh   # the tuple under the pipeline, re-derived"
---

# An accept is not a verdict until something independent of the parser expected it

**The finding.** After release `1.0.253`, 266 rows outside the design corpora were ACCEPTED by the
preprocess→parse pipeline under their derived context and still carried **no verdict**: the coverage
instrument adjudicated a `chained:accepted_under_expansion` row only when its suite sat in
`DESIGN_SUITES` ("a shipping design's file is valid as part of its unit"), and every other suite's
accept was — correctly — nothing, because no per-file expectation existed. 161 of the 266 were Surelog
rows in multi-file test units, which the `.8b.1` key had left `chained_only` *as a whole* ("multi-file
test unit (N sources) — dir-level chained adjudication"), although the unit's committed golden log
completes with `[ SYNTAX] : 0` for 146 of them.

**The rule, in three parts.**
1. **The expectation lives in the key.** The adjudicator writes `expected = chained_must_accept` — must-accept
   *as part of its unit* — and the raw lane still classes the row `deferred:chained_only` (the file is parsed
   alone there; the expectation is cashed by the pipeline reading on an accepted-under-expansion /
   accepted-in-unit verdict). The design corpora get the same value from the same convention, so the
   coverage instrument, the done-bar assertion and the denominator gate read **one column** and never a
   suite set: a Surelog row and an OpenTitan row are adjudicated by the same rule.
2. **Testimony is per NAMED file, from a COMPLETED log.** A Surelog `.sl` driver names what Surelog compiled —
   a positional source, the argument of `-v`, an entry of an in-unit `-f` list. A `-y` library member is read
   only on demand and an unnamed sibling never, so a clean log says nothing about them; a config-driven
   driver (`-map` / `-cfg` / `-batch`) decides the set elsewhere; a log with `[SNT:]` names no carrier; and a
   log without its summary block is an aborted run (the `.8b.1` docstring promised "a completed log" and
   never checked it — closed in the same leaf, zero tracked rows moved). Only a named file under a complete,
   `[SNT:]`-free, `[FTL:]`-free log is keyed.
3. **The single-source arm is unchanged, and the external single-source unit joins the per-file arm.** A
   single source that is the whole compiled set keys the raw lane (`must_accept` / `must_reject`, `.8b.1`);
   a single source whose driver references out-of-unit paths (`+incdir+../../../UVM/…`) is not the whole
   set, so it keys per file like a multi-source unit — 8 rows the first cut had left out.

**Measured.** Expected moved on 4,699 rows (4,609 design-corpus rows `chained_only → chained_must_accept`
with the convention named in the basis, 90 Surelog rows from 25 clean units); observed and adjudication
moved on **0** rows on all three lanes; the pipeline re-adjudication table identical in every verdict.
Under the pipeline `11503/2943/1890/1247/0 → 11582/2943/1811/1247/0`: adjudicated +79, no-verdict −79,
dark and bar unmoved; the burnable rows 1,425 → **1,346** (sv_2017 1,411 → 1,332), unexplained 0 on every
lane. 79 of the 90 Surelog rows were accepted under expansion; 10 are rejected in their own file under the
derived context and 1 is refused for context — must-accept rows the pipeline does not accept, routed
(`SV-CORPUS-GRAD.57a`), never relabelled.

**Two readings the instruments caught before the commit.** The pre-screen predicted 79 keyable accepted
rows; the first cut keyed 71 — the 8 were single-source units with out-of-unit include paths, which the
pre-screen had run through the multi-source logic and the instrument through the `.8b.1` arm (part 3 of the
rule is the resolution, and the second run read 79 = 79). Then the coverage instrument's manifest reader
selected five named columns and silently dropped the new one: every chained accept fell to NO VERDICT and
the tuple under the pipeline read `7889/2943/5504/2748/0` — the denominator doctrine refused the publish,
which is exactly the drift it exists to stop.

**What remains with no per-file expectation** (187 accepted rows): Surelog 82 (49 files the driver does not
name, 13 config-driven, 6 under incomplete or aborted logs, 4 outside any `.sl` unit, …), verilator 68 (the
companion files of passing tests — a driver reading like `.6c.4`'s is the next convention), verible 17,
slang 14, iverilog 3, sv2v 2, sv-tests 1.

**The verilator reading (`SV-CORPUS-GRAD.58`).** The same rule, a different spelling of ownership: a
companion is compiled by the test whose driver names it (an SV flag list, a `-f` command file with
`${TEST_REGRESS}`), whose source `include`s it, or whose source instantiates a module of its name — resolvable
because the suite's own driver passes `+incdir+t -y t` to every test (`test_regress/driver.py:771`). 68 rows
keyed, 59 adjudicated; a `fails=True` co-owner keys nothing, and an orphan no test references keeps no verdict.
