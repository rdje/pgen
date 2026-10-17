---
id: a-check-no-gate-runs-is-decoration
title: A `--check` no gate runs is decoration, and a deferral by basename is a pass nobody issued (`SV-CORPUS-GRAD.58b`, `.58c` routed)
answers:
  - "the census.py --check would have said STALE — why did every gate pass for a week"
  - "what are arms (5) and (6) of PIPELINE-LANE-CURRENCY"
  - "IDENTITY-CARRIERS said a mismatch was deferred to check_corpus_key_integrity.sh — does that oracle read the file"
  - "how does check_identity_carriers.sh decide which oracle owns a carrier (owning_oracles, basename)"
  - "a tracked derived artifact went stale through three commits — which gate should have caught it"
  - "when a producer gets a --check, what else must the same leaf do"
tags: [systemverilog, pipeline, doctrine, currency, identity-carriers, gates]
date: 2026-10-17
provenance: SV-CORPUS-GRAD.58b (PGEN-SV-CORPUS-GRAD-0447); found by SVPP-EXPANSION.4c.3 (PGEN-SVPP-EXPANSION-0049); SV-CORPUS-GRAD.58c (routed, the ownership rule); SV-CORPUS-GRAD.13i (IDENTITY-CARRIER-CURRENCY)
reverify: "bash scripts/check_pipeline_lane_currency.sh   # six arms, PIPELINE-LANE-CURRENCY: OK; bash scripts/check_pipeline_lane_currency.sh --self-test   # PIPELINE-LANE-CURRENCY-SELF-TEST: arms=8 failed=0; git show 9e1ce4890:docs/tasks/artifacts/sv_corpus_grad/pipeline_sv2023_lane/census.md > rust/target/x58b/census.md && PGEN_IDENTITY_CARRIERS=rust/target/x58b/census.md bash scripts/check_identity_carriers.sh --report | grep -c 'deferred to check_corpus_key_integrity.sh'   # 6 rows deferred by basename"
---

# A `--check` no gate runs is decoration

**What happened.** The two profile censuses of the pipeline lane — `docs/tasks/artifacts/sv_corpus_grad/pipeline_sv2023_lane/census.md` (sv_2017 × sv_2023) and `…/pipeline_sv2023_tolerant_lane/census.md` (sv_2023 × sv_2023_tolerant) — join the lane dumps against the sv_2023 answer key. The key was re-derived three times (`PGEN-SV-CORPUS-GRAD-0444` … `-0446`); the censuses were not, and HEAD carried them with the key's pre-`.57` digest in their identity row and `chained_only` in their `expected` column where the key read `chained_must_accept`. `census.py --check` re-derives and diffs the bytes in 0.23 s and would have said `STALE`. Nothing ran it: `PIPELINE-LANE-CURRENCY` held four arms (context, report, re-adjudication, rejects-own-file census) and the censuses were the fifth and sixth. `SVPP-EXPANSION.4c.3`'s lane chain re-derived them and noticed the `expected` column move with zero verdict movers.

**The second mechanism, measured while fixing.** The censuses ARE identity carriers (`IDENTITY-CARRIERS` takes every tracked `docs/tasks/artifacts/**/*.md` with an identity heading). Replaying `-0446`'s census through the gate (the `PGEN_IDENTITY_CARRIERS` seam) shows it SAW the `sv_2023 answer key` row's mismatch and filed a NOTE: *deferred to `check_corpus_key_integrity.sh`, which judges this carrier under its own row definitions*. It does not. `owning_oracles` matches the carrier's BASENAME — `census.md` — against every registered oracle's text, and that oracle names its own `census.md` (`sv2023_lane/`'s). Six `census.md` carriers under `sv_corpus_grad/` are "owned" that way by an oracle that never opens them. The gate's honest limit (*the guard on carriers NOBODY else checks*) was defeated by a deferral keyed on a common filename.

**The fix (`.58b`).** Arms (5) and (6) of `scripts/check_pipeline_lane_currency.sh` run each reading's `census.py --check` (`STALE` rc 1 on drift; `NOT EVALUATED` rc 0, loudly, without the .gitignored dumps). Two self-test arms set one `expected` cell of a census back to the pre-`.57` reading and expect arm (5), resp. arm (6), alone to refuse it (`arms=8 failed=0`). A faithful replay of `-0446`'s census through the new arm exits 1 where every gate at `-0446` had exited 0. **Routed (`.58c`)**: ownership by repo-relative path or directory, never basename — with the one live note that rests on the old rule (`parse_cost_ratchet/cost.md`) re-read before the change lands.

**The rules.** (1) The leaf that gives a producer a `--check` wires it into the gate that runs on the staged set, in the same commit. (2) A deferral is an assertion about another oracle — key it on something that oracle provably reads. (3) Run the check before writing the claim: `.4c.3`'s leaf said "not among the identity carriers" without replaying the file; one run refuted it, and the sentence had already been committed (corrected forward).

**Related.** [[a-pick-among-an-ips-real-builds-is-still-a-guess]] (the leaf that found the staleness); [[an-accept-is-not-a-verdict-until-the-key-expected-it]] (why the `expected` column moving matters).
