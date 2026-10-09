---
id: two-artifacts-that-read-each-other-need-a-second-pass-and-a-check
title: Two artifacts that read each other need a second pass to reach their fixed point, and a check that the pair agrees — the sv_2023 carry-over reads the parity artifact that reads the manifest the adjudicator just wrote
answers:
  - "an sv_2023 manifest row says re-read is owed while parity.tsv says IDENTICAL — why"
  - "why does a new 2017 clause pin need two adjudication passes"
  - "what order do the adjudicator and production_parity.py run in the ceremony"
  - "what does CORPUS-KEY-INTEGRITY tier A5 check"
  - "what is PARITY-MANIFEST-AGREEMENT and what are its fields"
  - "how is the parity agreement's red control built"
  - "the parity self-test's A3 arm went red after editing production_parity.py — why"
tags: [systemverilog, answer-key, sv_2023, parity, adjudication, doctrine, fixed-point]
date: 2026-10-09
provenance: SV-CORPUS-GRAD.54 (PGEN-SV-CORPUS-GRAD-0441); SV-CORPUS-GRAD.53 (the finding, PGEN-SV-CORPUS-GRAD-0440); .14b.3 (the carry-over mechanism)
reverify: "python3 docs/tasks/artifacts/sv_corpus_grad/sv2023_lane/production_parity.py --check   # production-parity: current + PARITY-MANIFEST-AGREEMENT: … OK; bash scripts/check_corpus_key_integrity.sh --self-test   # 23 passed, 0 failed (A5 end-to-end: the -0439 pair reads rc 1)"
---

# Two artifacts that read each other need a second pass and a check

**The mechanism.** The sv_2023 lane inherits a 2017 `must_reject` ruling on a cited 1800-2017 production
only when every cited production reads identically in 1800-2023 Annex A. That verdict lives in
`docs/tasks/artifacts/sv_corpus_grad/sv2023_lane/parity.tsv`, derived by `production_parity.py` from the
**sv_2017 manifest's** citations; the adjudicator reads `parity.tsv` when it writes the **sv_2023 manifest**.
So a NEW 2017 clause pin reaches the artifact only after the manifest carrying its citation is written, and
the sv_2023 row adjudicated in that same pass reads `its 1800-2023 re-read is owed (.14b.3)`. The fixed point
is adjudicate → `parity --write` → adjudicate; the second adjudication costs seconds (the store-gate memo hits).

**The finding.** `PGEN-SV-CORPUS-GRAD-0439` ran one pass after adding three clause pins and committed the three
sv_2023 rows as owed beside a `parity.tsv` of the same commit reading IDENTICAL for them; the summary published
57 owed / 53 carried over where the artifact supported 54 / 56. The verdicts never moved — the basis and two
counters did — and nothing compared the pair: `production_parity.py --check` compared the artifact to a fresh
derivation only, and was run by nothing.

**The rule.** (1) The producer of the artifact that is READ owns the agreement arm: `--check` now prints
`PARITY-MANIFEST-AGREEMENT: identical=N carried_over=M owed_but_identical=K not_carried=J missing_in_sv2023=L OK|STALE`
and names each lagging row with the remedy. (2) A doctrine runs it on every commit — `CORPUS-KEY-INTEGRITY` tier A5,
corpus-independent (the tracked manifests and the two in-repo Annex A texts), 0.1 s. (3) **The red control is the
real lag**: the doctrine's self-test extracts the `-0439` pair with `git show` and requires rc 1 with
`owed_but_identical=3` — a synthetic shape would prove the shape, not the defect. (4) The ceremony's item 4 runs the
second pass before the summaries are published.

**A trap on the way.** `parity.md` records its producer's own digest in its identity table, so the first self-test
after editing the script read A3 red (the restored artifact "scored rc 1") — the new arm refusing correctly.
`--write` moved exactly that one row. Read the failing arm's name before reading the count.
