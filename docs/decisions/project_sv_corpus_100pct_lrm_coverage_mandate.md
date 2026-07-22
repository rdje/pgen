# ⛔ SV corpus mandate ESCALATED: top-of-class external corpus + measured 100% LRM coverage (director 2026-07-22, session #191)

- **Category:** project (standing directive; amends the `SV-CORPUS-GRAD` charter)
- **Date:** 2026-07-22 (session #191, mid-`.3.1`)
- **Directives (director, same session, escalating):**
  1. *"So please make sure our external test corpus is really top of the class
     and that they stress, exercize 100%, that is all of the SV LRMs!! We need
     that level of coverage !"*
  2. *"Please make sure we have the required, needed external test corpus !!"*

## What this changes

1. **The SV graduation bar gains a COVERAGE axis.** Zero unexplained
   divergences over whatever happens to be vendored is NOT enough; the corpus
   itself must be proven to exercise the FULL parseable surface of the SV LRMs
   (IEEE 1800-2017 + 1800-2023, and IEEE 1364-2005 for the `verilog_2005`
   profile). Corpus sufficiency is now a measured deliverable, on the same
   honesty standard as everything else — never asserted.
2. **Acquisition priority is OVERRIDDEN.** The engineer's earlier sequencing
   recommendation (burn down divergences first, vendor the ADD-v1 tier later)
   is superseded: acquiring the required corpora is a first-class immediate
   leaf of `SV-CORPUS-GRAD` (the director exercised the suite-acquisition
   ownership reserved in the RVDEF charter — this is the GO).

## The measurable definition of "100% of the LRM" (no BS clause)

A parser cannot "exercise" non-syntactic LRM text (scheduling semantics,
simulation algorithms, PLI/DPI C-side API chapters). The honest, mechanical
definition adopted — consistent with the director's ratified N/A-with-cause
principle ("If there is none, fine, we will live with it",
`PGEN-CORPUS-GRAD-ALL-0003`):

- **Rule-coverage instrument (grammar axis):** parse the ENTIRE vendored
  corpus with per-rule participation recording (the generated parser's
  transactional coverage testimony); union the fired-rule sets per profile;
  diff against the grammar's full rule inventory. **Uncovered rule ⇒ a
  measured corpus gap** (the grammar is the faithful image of LRM Annex A, so
  rule coverage IS clause coverage on the syntax surface).
- **Clause matrix (LRM axis):** the clause-keyed suites (sv-tests `:tags:`,
  ispras per-clause dirs, ivtest keys) map corpus cases to LRM
  clauses; chapters/clauses with a parse surface but no corpus case are gaps.
- **Negatives count:** where the LRM defines illegality with a parse-level
  surface, the corpus must contain reject cases (the accepts-invalid blind
  spot; keyed CE suites attack exactly this).
- **N/A-with-cause:** clauses with no parse surface are adjudicated out with
  a named cause, per the ratified principle — tracked, never silent.

## Execution (owned by `SV-CORPUS-GRAD`, new leaves `.7`/`.8`/`.9`)

- `.8` **ADD-v1 vendoring** (frozen roster v1, adjudicated 2026-07-22):
  ispras/sv-tests (~904 clause-keyed POS/NEG/VARYING), ivtest
  `regress-sv.list` (922, CE/EF/gold keys — the keyed-negatives axis), sv2v
  (paired goldens + error/), Surelog tests, OpenTitan + black-parrot
  (UVM-scale + macro stress); fold vendored uvm-core into the bulk runner.
  Roster-v2 candidate: **slang embedded-unittest extraction** (thousands of
  SV snippets inside slang's C++ unit tests — the sharpest open conformance
  oracle; needs an extraction tool + fragment entry-point mapping).
- `.7` **The coverage instrument** (rule-participation union + clause matrix
  + uncovered-rule report per profile).
- `.9` **Gap-driven acquisition loop**: every uncovered rule/clause gets a
  corpus case — sourced from the ADD tiers or crafted directly from the
  in-repo LRM markdown (`docs/systemverilog/2017`/`2023`) with the clause
  cited — until 100% of the parseable surface is exercised or N/A-with-cause.
- The `.5` graduation gate's criterion widens accordingly: zero unexplained
  divergences **AND** 100% measured corpus coverage of the parseable grammar
  surface per profile.

## Boundary kept

Corpus-sufficiency work must never dilute the honesty rules already in force:
expected verdicts stay spec/metadata-derived, exclusions stay named, and the
coverage instrument measures the corpus — it must never be gamed by generating
stimuli and calling them "external" (external means externally authored).
