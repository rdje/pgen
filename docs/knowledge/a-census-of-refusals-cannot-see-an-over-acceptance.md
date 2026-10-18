---
id: a-census-of-refusals-cannot-see-an-over-acceptance
title: A census that adjudicates only what a parser REFUSES cannot see what it wrongly ACCEPTS — join it to an answer key before claiming a release moved no correct verdict (`SV-CORPUS-GRAD.13c.2v.12`)
answers:
  - "does the svpp corpus census measure over-acceptance"
  - "why did the preprocessor census miss accepted error tests"
  - "how do I show a parser release introduced no over-acceptance"
  - "is zero pass to fail enough for a no-regression claim"
tags: [census, over-acceptance, answer-key, preprocessor, measurement]
date: 2026-10-18
provenance: SV-CORPUS-GRAD.13c.2v.12 / .13 (PGEN-SV-CORPUS-GRAD-0459); ledger SVPP-0010, SVPP-0011
reverify: "python3 rust/target/svpp9/unterminated_string_census.py | head -1   # scratch; the tracked census gains the key join in the next svpp release"
---

# A census of refusals cannot see an over-acceptance

**The fact.** `docs/tasks/artifacts/sv_corpus_grad/svpp_corpus_census/census.py` reads every corpus file through the
generated `systemverilog_preprocessor` parser and adjudicates each REJECTION in `residual_dispositions.tsv`; the census
refuses to run while one is unadjudicated. Release `1.0.8` was published on it with *"zero `pass → fail`, every residual
adjudicated"*. Both statements were true, and the release still accepted seven suite error tests for an unterminated
string (sv2v `test/error/string_literal_eof.sv`, verilator `t_parse_eof_str_bad.v`, `t_preproc_stringend_bad.v`, …) and
read a directive inside a block comment as real in four files (Surelog `tests/PPComment/dut.sv`, …). Nothing in the
census could see either: an accepted file never reaches the disposition table, and *"pass → fail = 0"* compares the
release with its predecessor, so a defect BOTH releases share is invisible to it.

**How it was found.** Not by the census — by designing the next release's skipped-group behaviour and testing the
neighbouring inputs by hand (`` `ifdef DNE `` around an unterminated comment, then an unterminated string), then
scanning the corpus for the shape. Of the 247 rows the SystemVerilog answer key files as `deferred:svpp_owned` — the
directive parser's own expectations — the release parser accepts 226, and none had been joined to its verdict.

**How to apply.** A parser census is two-sided or it is a census of one failure mode. Before a release claims it moved no
correct verdict: join the population to an answer key carrying `must_reject` rows (here the SV key's `must_reject` and
`deferred:svpp_owned` rows), report the over-acceptances by name, and treat *"pass → fail = 0"* as a statement about
movement between two releases, never about correctness.
