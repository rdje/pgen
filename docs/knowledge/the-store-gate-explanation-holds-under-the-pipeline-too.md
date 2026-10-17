---
id: the-store-gate-explanation-holds-under-the-pipeline-too
title: The store-gate explanation holds under the pipeline too — `chained:explained_store_gate` reads the dark worklist's proof instead of re-running it (`SV-CORPUS-GRAD.57b`)
answers:
  - "a chained_must_accept row rejects in its own file on an undeclared type — is that a defect, a barrier or the store gate"
  - "what is chained:explained_store_gate and which conditions give it"
  - "why does the pipeline re-adjudication read dark_chained_only/adjudication.tsv"
  - "why are FACT-GATED-CROSS-FILE and PRE-TICK rows not explained by .57b"
  - "why does a wildcard import after the refused use not keep the row dark"
tags: [systemverilog, pipeline, re-adjudication, store-gate, answer-key, doctrine]
date: 2026-10-17
provenance: SV-CORPUS-GRAD.57b (PGEN-SV-CORPUS-GRAD-0451); SV-CORPUS-GRAD.13w (the raw lane's class and its three soundness conditions); SV-CORPUS-GRAD.13c / .13c.2 (the strata and the fixed-point proof); SV-CORPUS-GRAD.17(a) (the positional reading of §26.3); SV-CORPUS-GRAD.57c (routed)
reverify: "python3 stimuli/sv/pipeline_readjudication.py --self-test   # PIPELINE-READJUDICATION-SELF-TEST: checks=10 failed=0; grep -c explained_store_gate stimuli/sv/characterization/pipeline_readjudication.tsv   # 3"
---

# The store-gate explanation holds under the pipeline too

**The class.** `SV-PARSE-STRICT.2` made the parser reject a provably-undeclared type identifier at parse time. On the
raw lane `.13w` turned the `must_accept` rows that refusal catches into `divergence:explained_store_gate` — by parsing:
the file parses once a minimal prelude declares the names the parser demanded and lacked, and three soundness
conditions hold (no unresolved wildcard import before the use, no package `export`, no LRM-predefined name). Under the
pipeline the same rows are keyed `chained_must_accept` and sat `chained:rejects_own_file`, NO VERDICT, dark. Since
`.57b` the re-adjudication reads the same proof there: a row earns `chained:explained_store_gate` when the key expects
it must-accept as part of its unit, `.13c`'s audit stratum is `NO-ALTERABLE-TICK/UNDECLARED-ANYWHERE` (no alterable
directive before the failure — so the lane's expanded text is the file's own text and the raw-text proof is the
pipeline's — and the demanded name declared in no file of the suite, hence none of the unit), the dark worklist's
adjudication reads `CROSS-FILE-FACTS` with its minimal prelude (not overridden by a `RESIDUAL_ROWS.tsv` ruling), and
`soundness_reason` returns nothing at the audit's `stuck` offset. ADJUDICATED-explained, never a pass; the prelude is
published in the row's `unit` column (`parses once declared: DD,id_t1 (class)`).

**Read the proof, do not re-run it.** The route asked for "the `.13w` proof over the lane's expanded text"; for a
`NO-ALTERABLE-TICK` row the expanded text IS the raw text, and `adjudicate_dark_worklist.py` had already proved those
rows. The arm reads a tracked table (`dark_chained_only/adjudication.tsv`, now an identity-block input of the
re-adjudication) and adds no prover.

**What the strata keep out, and why.** `FACT-GATED-CROSS-FILE` means the name IS declared in another file of the
suite: inside the row's unit it is the `.6d` declaration barrier (`SVPP-EXPANSION.6d.9`'s population), outside it the
gate's refusal — the unit's source list decides, and `.57b` does not read it (19 rows, `tests/ClassFuncProto/top.v`
among them). `PRE-TICK` means an alterable directive precedes the failure, so the proof must run on the lane's
expanded text (210 rows). Both are `.57c`. Measured on the first reading: 3 rows explained (`ModPortTest`, `UnitClass`,
ibex `spec_api.sv`), `rejects_own_file` 299 → 296, the tuple under the pipeline `11648 → 11651` adjudicated.

**Related.** [[an-accept-is-not-a-verdict-until-the-key-expected-it]] (the key side of the same discipline);
[[a-delay-on-a-member-select-needs-parentheses]] (the `.58a` reading that classed its three barrier rows).
