---
id: a-second-profile-lane-is-a-differential-and-the-differential-is-the-finding
title: A second profile lane is a differential, and the differential is the finding — join the two editions' verdicts per file, hold 2017_only_pass at zero, and read each 2023-only pass by its adjudication class
answers:
  - "how do I judge the corpus under the sv_2023 profile (the sv2023 corpus lane)"
  - "what does the sv_2017 x sv_2023 verdict census say and how do I re-derive it"
  - "is IEEE 1800-2023 a superset of 1800-2017 in PGEN's parser — does anything regress under the 2023 profile"
  - "a corpus row keyed must_accept parses under sv_2023 but not sv_2017 — edition contrast, gating defect, or key error"
  - "a must_reject row parses under sv_2023 — is that a 2023 over-acceptance"
  - "the first run of a new corpus lane refused on parameter drift or fell to the debug probe — how does provenance inheritance work"
  - "my census joined the raw dumps against the manifest and every row read (not in manifest)"
  - "how do I carry a must_reject ruling from the sv_2017 lane to the sv_2023 lane (production parity between IEEE 1800-2017 and 1800-2023 Annex A)"
tags: [sv-corpus, sv_2023, edition-contrast, profiles, census, provenance, systemverilog, lanes]
date: 2026-09-04
status: current
evidence: "SV-CORPUS-GRAD.14a + .14b.2 (PGEN-SV-CORPUS-GRAD-0359, -0361; the 2023 lane adjudicated: match 5918 unexplained 137, nine iverilog edition contrasts found by its own .12a, bar 151 -> 142): stimuli/run_external_corpus.sh sv2023 60 8 0 over 16 336 files (81 s); census both_pass=9831 both_nonpass=6477 2023_only_pass=28 2017_only_pass=0; the 28 = 15 bar rows (5 ispras 1800-2012 §12.6 nested tagged patterns, 10 verilator 1800-2023 constructs) + 11 must_reject rows + 1 routed + 1 macro"
reverify: "python3 docs/tasks/artifacts/sv_corpus_grad/sv2023_lane/census.py --check   # current, and 2017_only_pass=0"
---

**A second profile over the same files is not a second pass rate. It is a differential, and every row
where the two editions disagree is a question with a small answer set — read it by the row's
adjudication class on the lane that has a key.**

## The rules

- **Hold `2017_only_pass` at zero.** The newer edition is a superset; a file the older profile parses
  and the newer refuses is a regression in the newer profile's gating, and the differential is where
  it shows first. Measured 0 over 16 336 files on the first run.
- **Every `2023_only_pass` row is one of three things**, and its 2017-lane adjudication class says
  which: (1) `unexplained_rejects_valid` on text that is 2023-only syntax — a 2023 tool keyed it
  `must_accept` on the 2017 lane; it is an EDITION CONTRAST (`must_reject` on the 2017 lane with both
  editions' Annex A cited, `must_accept` on the 2023 lane — the `.13x.d` ruling at corpus scale);
  (2) `unexplained_rejects_valid` on text that is legal in the OLDER edition too — a gating defect
  in the older profile, a grammar leaf (the five §12.6 nested `tagged` patterns); (3) `match` on a
  `must_reject` row — a correct 2017 rejection the 2023 profile admits; on the 2023 lane it is the
  `.13x` census question (syntactically valid 2023 text whose failure is semantic, or a 2023
  production that refutes it).
- **A first run of a new lane inherits its sibling's provenance.** The runner refuses parameter drift
  against the lane's own report; with no report yet it would fall to debug-probe defaults. The
  `sv2023` lane reads `characterization.md` for its first run; a capped smoke run says
  `PGEN_CORPUS_REBASELINE=1` into `PGEN_CORPUS_OUT_DIR` scratch — the refusal working, not a bug.
- **The differential is not the whole yield — give the second lane its own key and let its own
  machinery judge.** Nine rows failed under both profiles and never appeared in the differential; the
  2023 lane's `.12a` refutation filed them `explained_svpp_macro_use` (the parse passes the 2023 token
  and stops at a macro-alterable point), which left their 2017 failure as the edition token itself —
  nine edition contrasts in the 2017 bar (`151 → 142`). An edition-relative key is the older key plus:
  contrasts flip to `must_accept`; explicit newer-edition readings for text the newer edition derives
  whose upstream failure is semantic; everything else inherited with a marker the summary COUNTS.
- **An inherited ruling carries over only when the production TEXT is compared.** A `must_reject` on a
  1800-2017 production is a 2023 ruling only if that production reads the same in 1800-2023 Annex A:
  `production_parity.py` extracts both bodies from the in-repo LRM texts, normalises PDF furniture,
  footnote digits and spacing, and compares — 33 of 38 identical (mechanical carry-over), 5 read by hand
  with the changed 2023 production cited. Hands only for the diffs.
- **Join on the producer's convention and assert the join.** The manifest keys a file relative to its
  suite; the raw dumps carry repo-relative paths. A census that joined on the wrong one printed
  *(not in manifest)* for every row — a number that read like a fact. A row absent from the manifest
  is a REFUSAL, never a cell; identity (same binary, same semantic grammar digest, dumps re-hashing
  to the reports' OUTPUT rows) is verified before a single row is counted.

## The measurement

`SV2023-LANE-CENSUS: rows=16336 both_pass=9831 both_nonpass=6477 2023_only_pass=28 2017_only_pass=0
population_breaches=0`; lane `16336 parsed — pass=9859 fail=6477 timeout=0 crash=0` under
`--profile sv_2023`. See `docs/tasks/SV-CORPUS-GRAD.md` `.14a`, `.14b`, `.3.34` and
`docs/tasks/artifacts/sv_corpus_grad/sv2023_lane/census.md`.
