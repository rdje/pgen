---
id: an-assertion-whose-zero-no-work-can-reach-is-not-a-bar-it-is-a-label
title: An assertion whose zero no work can reach is not a bar, it is a label — leg 3 demanded a verdict for rows the project had already ruled unadjudicable, so no parser could ever meet it; read each row under its declared reading and bound what you exclude
answers:
  - "why could the SystemVerilog Done bar's leg 3 never be met by any parser work"
  - "what does full_pass mean in the SV external-corpus conformance gate since DONE-BAR.3f"
  - "how does the leg-3 assertion treat permanently unadjudicable corpus rows"
  - "what is burnable_no_verdict and how does it relate to the denominator gate's tuple under the pipeline"
  - "what is full_pass_raw_lane in the conformance gate's summary"
  - "why does the conformance comparator read the pipeline re-adjudication table"
  - "why are the permanent-deferral ceilings per lane"
  - "how do I check whether a bar's assertion is reachable at all"
tags: [done-bar, leg-3, conformance-gate, sv-corpus, permanent-deferral, pipeline-lane, reachability, instruments]
date: 2026-10-08
status: current
evidence: "DONE-BAR.3f (`PGEN-DONE-BAR-0032`). `corpus_conformance_compare.py` defined `full_pass = unexplained == 0 and no_verdict == 0` over the raw sv_2017 manifest; the register's leg-3 `pass_query` read that bit. Decomposing the raw lane's 6 245 no-verdict rows with the coverage instrument's own class map: 716 in the five PERMANENT classes of `SV-CORPUS-GRAD.13f` (each exactly at its recorded ceiling), 4 933 `deferred:chained_only` rows the raw lane defers to the preprocess→parse pipeline by design (`SVPP-EXPANSION.6c.1`), 596 `deferred:no_sv_key` — 5 649 of 6 245 rows the bit demanded a verdict for that no parser work could supply; the verilog_2005 lane held 29 more (impl_varying / NI) with no bound at all. The comparator now reads every row through `corpus_verdict_coverage.pipeline_reading` (one definition — the same function the denominator doctrine's tuple under the pipeline comes from), names and bounds the permanent classes of both lanes (over a ceiling: REFUSED), counts a stale re-adjudication row and a pipeline bar row against the pass, and keeps the old bit as `full_pass_raw_lane`. On the tracked manifests: declared_adjudicated 13 794 + permanent 500 + burnable 2 042 = 16 336 rows, unexplained 0, declared bar 0 — `full_pass` false and reachable; the sv_2017 reading reproduces the denominator gate's 11259/2606/2471/bar 0 exactly. 13 new self-test arms, every refusal driven RED."
reverify: "python3 stimuli/sv/corpus_conformance_compare.py --self-test (expect OK, the 3f arms among them); python3 stimuli/sv/corpus_conformance_compare.py --assert-tracked stimuli/sv/characterization/adjudication_manifest.tsv --assert-tracked-v2005 stimuli/sv/characterization/adjudication_manifest_v2005.tsv | jq '[.rows, .declared_adjudicated + .permanent_no_verdict + .burnable_no_verdict, .full_pass_raw_lane]' (expect the first two equal and the third false); bash scripts/check_sv_corpus_denominator.sh | grep -o 'under the pipeline [0-9/]*' and compare with the assertion's sv_2017 declared_adjudicated/declared_routed/declared_no_verdict"
---

**Ask of every bar: which work would make it true?** If the honest answer is *none*, the bar is a
label — it says where a family sits, never how far it has to climb — and every instrument that counts
it (the release distance, the three-leg block) publishes a distance to a point nobody can reach.
Leg 3 was such a label for 35 days: `no_verdict == 0` on the raw lane, while the project's own
rulings held 716 rows permanently unadjudicable and 4 933 rows answerable only under the pipeline.

- **Read each row under its declared reading, with one definition.** The coverage instrument already
  said how a deferred row is read under the pipeline; the comparator imports that function rather
  than restating it, so the denominator doctrine's tuple and the bar's assertion cannot disagree.
- **Exclude only what is bounded, and refuse when it grows.** A class the assertion stops demanding
  must sit under a recorded ceiling in a register that names its argument — on *every* lane that
  carries it, because the same class name can hold a different population per lane. Above the
  ceiling, the comparator refuses rather than scores: an unbounded exclusion is a hole, not a ruling.
- **Keep the retired bit by name.** `full_pass_raw_lane` survives in the artifact and the tracked
  assertion so the change of meaning is visible to every reader, and nothing moves from no-verdict to
  pass in any published table — the question changed, the data did not.
- **Publish the remainder as the distance.** `burnable_no_verdict` is the rows that still stand
  between the corpus and the bar; the audit's tracked-derivation line prints it, and the three terms
  sum to the denominator.
