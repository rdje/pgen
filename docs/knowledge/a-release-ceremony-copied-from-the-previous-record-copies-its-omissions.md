---
id: a-release-ceremony-copied-from-the-previous-record-copies-its-omissions
title: A release ceremony copied from the previous record copies its omissions — keep the gate list in the book, name every producer in a memo key, and let a doctrine be the bank's invoker
answers:
  - "which gates must a SystemVerilog grammar release run on its final bytes (the release ceremony)"
  - "sv_syntax_closure_gate is RED with an unreachable rule SET that differs from allowed_unreachable_rules — suppress, list, or delete"
  - "BASELINE-IDENTITY says a contract has been STALE across more commits than its budget — what do I do"
  - "why did the store-gate memo not re-measure after the instrument changed (memo key = file sha + binary sha only)"
  - "a probe bank went RED for several commits and nobody noticed — who runs it now (SV-STORE-GATE-BANK)"
  - "check_doctrines.sh passed in CI but the release probe is not built there — which doctrines evaluated nothing"
  - "what does PGEN_RELEASE_PROBE do"
tags: [release-ceremony, gates, doctrines, sv-corpus, memoization, baseline-identity, syntax-closure, systemverilog]
date: 2026-09-04
status: current
evidence: "SV-CORPUS-GRAD.15 + .16 (PGEN-SV-CORPUS-GRAD-0358): the .13w bank had no invoker (census 0) and sat RED six commits; the memo key (file, binary) missed the .13x.e instrument change (186 rows hand-pruned); releases 1.0.207 and 1.0.208 both omitted sv_syntax_closure_gate, whose contract went STALE across 21 commits (budget 20) with ten kw_* terminals orphaned by .3.30, each referenced once in the grammar"
reverify: "bash scripts/check_sv_store_gate_bank.sh && bash scripts/check_baseline_identity.sh && echo OK   # the bank 10/10 under its doctrine; every tracked baseline within budget and re-hashing equal"
---

**A ceremony that exists only as the previous release's record is copied forward with its omissions;
a memo key that omits a producer is stale the moment that producer moves; a bank nothing invokes is
no check. Each of the three was found by a gate this slice added or ran, not by a reader.**

## The rules

- **The release ceremony is a list in the book, cited by the record — never a record copied from its
  predecessor.** `docs/systemverilog_parser_book/src/schema-versioning.md`, *The release ceremony*,
  holds the eleven items a grammar revision runs on its FINAL bytes (lint and parity, regenerate and
  build, pins, both corpus lanes, both certificates, syntax closure, shape and reproducibility, parse
  cost in its fixed order, the downstream derivations, the lockstep documents, the doctrines). Two
  releases in a row omitted the syntax-closure gate because each copied the one before.
- **A newly unreachable rule is adjudicated by a grep, then listed with the reason or deleted in a
  grammar slice — never suppressed.** `grep -c` per name in the grammar: exactly ONE occurrence means
  dead (its own definition), not gated; the contract's allow-list is re-derived to the observed set
  with the measured reason, and `max_unreachable_rules` must equal its length. Deleting the dead
  rules moves the grammar digest and needs the full ceremony (`SV-CORPUS-GRAD.3.33`).
- **A staleness budget is the doctrine that catches a skipped gate.** `BASELINE-IDENTITY` counts the
  commits touching a contract's declared inputs since its stamp; over budget it fails for everyone at
  HEAD until the owning gate is run — GREEN re-stamps, RED needs a person. Run the gate; never
  hand-edit the stamp.
- **A memo key names every producer.** The store-gate memo now hashes the two producer files
  (NUL-framed, path-prefixed) into `instrument_sha256`; a row measured by other producer bytes, or a
  legacy row without the column, is re-measured. Re-stamp a tracked memo by a full re-measure and
  prove zero verdicts moved — 231 rows here.
- **The invoker of a bank is a doctrine, and it says when it cannot judge.** `SV-STORE-GATE-BANK`
  requires exit 0, `N/N as declared (fail=0)` with `N` equal to the bank's own `DECLARED`, and no
  `FAIL` line — three conditions because each can fail alone; its `--self-test` proves each refusal.
  Without the release probe it exits 0 SAYING SO, and the driver's scope note (derived from each
  enforcer's source, `PGEN_RELEASE_PROBE` to prove it) names every doctrine that judged nothing live.

## The measurement

Bank 9/9 → 10/10 under its doctrine; memo 231 rows re-measured, 0 verdicts moved; closure contract
allow-list ONE → ELEVEN with the ten `kw_*` terminals each `grep -c` = 1; `sv_syntax_closure_gate`
GREEN and re-stamped (defined_rule_count=1533 reachable_rules=1553 unreachable_rules=11 unreachable_branches=2 unresolved_rule_reference_count=0); 37 doctrines. See `docs/tasks/SV-CORPUS-GRAD.md` `.15`, `.16`, `.3.33`.
