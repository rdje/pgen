---
id: declare-the-producers-not-the-consumers-in-a-dependency-set
title: A freshness block must name what PRODUCES its numbers, never what CHECKS them — declaring the checker manufactures false staleness, and a gate that cries wolf gets bypassed
answers:
  - "which files belong in an identity or provenance block"
  - "should a baseline hash the gate that reads it"
  - "my freshness check goes red on cosmetic edits — what did I over-declare"
  - "how wide should a dependency set be before it becomes noise"
  - "is it safer to over-declare or under-declare inputs to a staleness check"
  - "why did adding the harness to the identity block make the gate useless"
tags: [provenance, identity, baselines, gates, doctrine, false-positives]
date: 2026-08-20
status: current
evidence: SV-CORPUS-GRAD.13c.2x.2 slice 1. The first stamp of `systemverilog_recognized_cert_union_contract.json` declared SIX inputs, including `rust/scripts/sv_cert_recognized_union_gate.sh`; corrected to FIVE before commit. `ast_pipeline` emits both the `CERTIFICATE-COVERAGE:` and `CERTIFICATE-COVERAGE-UNION:` lines — the gate only asserts them — so the surviving set is the grammar, the generated parser, and the three engine surfaces that produce the counts (`grammar_wellformedness.rs`, `indirect_lr_elimination.rs`, `stimuli_generator.rs`). Wiring the reader into that same gate immediately staled the contract, which is how the over-declaration was noticed.
reverify: "python3 -c \"import json;print(sorted(json.load(open('rust/test_data/grammar_quality/systemverilog_recognized_cert_union_contract.json'))['identity']['inputs']))\"   # five PRODUCERS, no harness"
---

**Over-declaring a dependency set feels like the safe direction. It is not free, and the cost lands
on the check's credibility.**

A freshness block exists to answer one question: *did anything change that could have changed this
number?* Every path it names is a claim that a change there invalidates the baseline. Name a file
that does **not** produce the number and you have not tightened the check — you have taught it to
say "stale" when nothing about the measurement moved.

The first identity block written for PGEN's certificate-union contract declared the gate script
that asserts its `expected_*` fields. Within minutes the wiring commit — which added an identity
*reader* to that same gate — staled the contract it had just stamped, for a change that could not
possibly move a certificate count. The correction is a one-line rule:

```text
ast_pipeline  ── emits CERTIFICATE-COVERAGE: / CERTIFICATE-COVERAGE-UNION:  ← PRODUCER, declare it
     gate     ── parses those lines and compares to the contract            ← CONSUMER, do not
```

A 400-line shell harness attracts cosmetic edits — a comment, a log line, a message reword. Each one
would have demanded a re-measurement costing minutes per seed, for a number that provably did not
change. `PARSE-COST-RATCHET`'s own header already states the general form of this: it keeps its
wall-clock advisory band deliberately *wide* because "a tight band on a machine-dependent number
manufactures false alarms, and a gate that cries wolf is a gate people learn to bypass."

⚠️ **The rule does not license under-declaring, which is the failure `ENGINE-UNIVERSAL-SERVICES.21`
recorded**: the parse-cost identity named three inputs on the sound argument that the binding
counters are a function of exactly those, and it was insufficient — the *instrument's classifier*
produced published family numbers, so correcting it staled them while the identity tier still
reported `fresh`. **A baseline's identity must name everything its ARTIFACT depends on, not
everything its headline metric depends on — and nothing that merely reads it.**

The discriminating question is neither "could this file be relevant?" nor "is this file nearby?" but
**"if I change this file and nothing else, can the recorded value legitimately differ?"** If yes, it
is a producer. If no, leaving it out is not a hole — it is the difference between a check people run
and a check people skip.

See also [[a-cheap-tier-that-prescribes-a-remedy-inherits-the-remedys-blind-spot]].
