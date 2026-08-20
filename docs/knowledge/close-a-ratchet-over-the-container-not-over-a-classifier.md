---
id: close-a-ratchet-over-the-container-not-over-a-classifier
title: The closed side of a two-sided ratchet must be a filesystem fact, not a classifier verdict — a key-name census misses silently and in the passing direction
answers:
  - "how do I define the population a two-sided register has to cover"
  - "is it ok to derive a ratchet's population with a regex over field names"
  - "my census counted 15 files and the real number is different — where did it go wrong"
  - "why did my sweep miss a baseline that obviously holds an expectation"
  - "should a register enumerate the files it covers or derive them"
  - "how do I stop a tracked population from re-growing silently"
  - "my classifier over-counted and under-counted the same population at once"
  - "why is a *.json-only census dangerous"
tags: [ratchets, registers, censuses, instruments, evidence, doctrine, baselines]
date: 2026-08-20
status: current
evidence: SV-CORPUS-GRAD.13c.2x(d) sized the derived-expectation baseline population at **15 of 15** with a key-name classifier (`expected_*` / `min_*` / `max_*` / `*_count`) and declared itself a mechanical FLOOR owing per-row adjudication. `.13c.2x.2` slice 1 adjudicated all 53 entries of `rust/test_data/grammar_quality/` and the classifier was wrong in BOTH directions — it overcounted five `*_v0_contract` files whose "derived" fields are all run configuration and performance budgets (`sample_count`, `seed_base`, `max_depth`, ms/byte ceilings), and it undercounted because it read `*.json` only, missing `regex_pcre2_compile_oracle_lightweight_v0.env` and its SEVEN derived expectations (2189/1609/580/1845/344/299/48). The register `baseline_identity_register_v0.json` therefore closes over `os.listdir` of the directory.
reverify: "bash scripts/check_baseline_identity.sh --report   # 53 entries, one verdict each; an entry with no row FAILS and a row naming no entry FAILS"
---

**A two-sided ratchet is only as closed as its population.** The mechanism is sound — an entry with
no verdict fails, a verdict naming no entry fails — but both halves are computed against a set, and
if that set is produced by a *classifier* rather than by the *container*, the ratchet inherits the
classifier's blind spot. That blind spot is silent, and it is in the passing direction: a file the
classifier does not select is not reported as unclassified, it is simply absent from the population,
so it never needs a verdict and its absence never fails anything.

`SV-CORPUS-GRAD.13c.2x`(d) sized a population of baselines holding tree-derived expectations by
matching key names, got **15**, and was honest enough to label the result a floor. Adjudicating the
directory row by row showed the classifier failing twice over:

```text
OVERCOUNT   systemverilog_core_v0_contract.json          14 "derived" fields ->  0
            (max_depth, max_repeat, seed_base, target_max_attempts, ms/byte ceilings
             — every one of them what the run is TOLD, not what the tree PRODUCES)
UNDERCOUNT  regex_pcre2_compile_oracle_lightweight_v0.env  invisible -> 7 real expectations
            (the sweep globbed *.json; this baseline is an .env)
```

The overcount is the cheap error: it adds noise and a reviewer catches it. The undercount is the
expensive one, because nothing in the output says *"I did not look at .env files"* — the census
simply reports a number, and the number is confident.

⭐ **The fix is not a better classifier.** Close over the container. Membership in
`rust/test_data/grammar_quality/` is a filesystem fact that `os.listdir` re-derives on every run,
and a new file lands in the population whether or not anyone anticipated its shape or its extension.
The classifier then moves to where it belongs — it becomes the *disposition* on a row, a human
verdict that a reviewer can read and disagree with, rather than an invisible filter deciding what
gets reviewed at all.

The same shape appears wherever a register is derived: `check_gate_reachability.sh` derives its
universe from the Makefile and the script directory rather than from a list of "things that look
like gates", for exactly this reason. **Derive the container; adjudicate the members.**

See also [[a-check-whose-inputs-all-pass-has-not-been-tested]] and
[[check-whether-the-artifact-already-states-the-property-before-building-a-detector]].
