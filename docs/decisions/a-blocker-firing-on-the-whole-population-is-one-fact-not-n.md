---
name: a-blocker-firing-on-the-whole-population-is-one-fact-not-n
description: "DISCIPLINE (2026-08-27, ENGINE-UNIVERSAL-SERVICES.44(b) step 2 / PGEN-ENGINE-UNIVERSAL-SERVICES-0102) — when a capability census reports ZERO across several independent gates, read each blocker count against its own population size BEFORE reading the verdict. A count that EQUALS the population is not N per-item verdicts; it is ONE global property reported N times, and the verdict says more about the gate's model than about the thing measured. MEASURED on the shipped SystemVerilog grammar: four dispatch gates read 0 of 535 top-level sites, 0 of 535, 0 of 1 934 branches and 0 of 1 157 min-0 sites — four models, four zeros, reading like 'this grammar is structurally undispatchable'. Every top blocker count equalled its own population exactly and every one was the SAME blocker, `terminals skip leading layout (R2 raw-byte peek unsound)`: all four gates model an emission that peeks the RAW byte at the current position, and SV terminals skip layout, so the census was not measuring SV's dispatchability — it was declining to look. The gate that made it unmissable was the one with the FEWEST blockers (FIRSTK: exactly one histogram entry, covering all 1 934 branches), not the one under study."
id: a-blocker-firing-on-the-whole-population-is-one-fact-not-n
title: "A blocker firing on 100 % of a population is ONE fact, not N — read the histogram against the population before reading the verdict"
date: 2026-08-27
evidence: docs/tasks/artifacts/engine_universal_services/deep_nesting_cliff/census_attempt_elision.sh (arms A and B) + logs/attempt_elision_census.txt; docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .44 (b) STEP 2 (PGEN-ENGINE-UNIVERSAL-SERVICES-0102); the gate source is rust/src/ast_pipeline/fusibility_census.rs:1789/:1863/:2288, each guarded on `classifier.layout.terminals`
reverify: "./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --report-fusibility-census 2>&1 | grep -E '^(DEGENERACY-CENSUS|PREFIX2-CENSUS|FIRSTK-CENSUS|QUANT-SITE-CENSUS)|R2 raw-byte peek unsound'   # every gate reads 0, and each layout-blocker count must EQUAL the site/branch total on its own gate's line — if a count ever falls below its population, the blocker has stopped being global and the verdict has become a per-site fact"
answers:
  - "my capability census reports zero everywhere, is the grammar really that bad"
  - "how do I tell a real negative result from a gate that cannot see my input"
  - "why do four independent gates all report zero on the same grammar"
  - "what does 'terminals skip leading layout (R2 raw-byte peek unsound)' actually mean for my grammar"
  - "how should I read a blocker histogram"
  - "does PGEN's fusibility census price attempt elision on SystemVerilog"
metadata:
  node_type: memory
  type: feedback
  created: 2026-08-27
---

## The rule

A capability census prints, per gate, a verdict count and a blocker histogram. The verdict is the
line people quote. **The histogram is the line that says whether the verdict is about your input at
all.**

Before reading any verdict, divide each blocker count by that gate's own population:

- **count < population** → a per-item fact. Some sites qualify, some do not. The verdict measures
  the thing you asked about.
- **count == population** → **one global property**, reported once per item. The verdict measures
  the gate's *model*, and it would read the same for every input with that property.

The second case is not a smaller version of the first. It is a different kind of statement, and it
is the one that gets mis-quoted, because `0 of 1 157` looks like a thousand careful verdicts.

## The measurement it comes from

`ENGINE-UNIVERSAL-SERVICES.44`(b) asked whether PGEN's existing attempt-elision machinery already
prices the fix on SystemVerilog. Four gates answered:

| gate | population | verdict | top blocker | fires on |
|---|---:|---:|---|---:|
| `DEGENERACY-CENSUS` | 535 top-level Or sites | **0** | `terminals skip leading layout` | 535 (**100 %**) |
| `PREFIX2-CENSUS` | 535 blocked sites | **0** | same | 535 (**100 %**) |
| `FIRSTK-CENSUS` | 1 934 top-level branches | **0** | same | 1 934 (**100 %**) |
| `QUANT-SITE-CENSUS` | 1 157 min-0 sites | **0** | `terminal layout skipping` | 1 157 (**100 %**) |

Four gates, four models, four zeros — and one cause. Each gate emits a guard that peeks the **raw
byte at the current position**; SystemVerilog's terminals skip leading layout, so that byte is not
the branch's first significant byte and the peek would be unsound. **The gates are correct. The
report is useless**: it prices SystemVerilog under an emission SystemVerilog can never use, and that
price is necessarily zero.

## Two things that made it visible, both cheap

**Look at the gate with the FEWEST blockers first, not the one you care about.** Three of the four
histograms carry co-firing blockers (`branch not first-byte-decided` at 1 934 of 1 934,
`element not first-byte-decided` at 1 157 of 1 157 — themselves universal), which is enough noise to
argue about. `FIRSTK` has **exactly one entry**, covering every branch. The gate you care least
about is often the cleanest instrument.

**Run the same binary on a grammar where the gate is known to fire.** `0` from a census and `0` from
a dead census are the same bytes. On `grammars/regex.ebnf` the identical invocation reads
**60 / 33 / 853 / 102**. That is what converts *"the census returned zero"* into *"SystemVerilog has
no surface here"* — and it is [[feedback_an_instrument_that_can_only_return_one_reading_is_not_a_measurement]]
applied to a negative result rather than a defect.

## What it does NOT license

A universal blocker says the census cannot see your input. It does **not** say the capability is
available under a different emission, and it must never be read as one. Whether a layout-aware
variant — skip layout once per dispatch, then peek — is sound and worth emitting is a design
question the census has not been asked. Reading *"the gate's model does not fit"* as *"so a better
model would work"* replaces a measured zero with an unmeasured hope.

Related: [[a-healthy-cache-is-not-evidence-the-cache-is-the-bottleneck]] is the same mistake one
level up — a number that looks like an answer about the workload while it is really an answer about
the instrument's own question.
