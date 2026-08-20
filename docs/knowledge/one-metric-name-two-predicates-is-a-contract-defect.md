---
id: one-metric-name-two-predicates-is-a-contract-defect
title: Two instruments reporting different values for the same metric name are usually both right — the defect is the contract that never said which predicate it meant
answers:
  - "two tools disagree about the same metric on the same input — which one is wrong"
  - "my gate fails on a metric and the obvious diagnostic tool reports zero"
  - "how do I settle a disagreement between two tracked instruments"
  - "is an entry-scoped reachability count comparable to a global one"
  - "what should a contract record besides the threshold"
  - "why does my unreachable-rules count change when I change the entry rule"
tags: [instruments, contracts, metrics, diagnosis, gates, evidence]
date: 2026-08-20
status: current
evidence: "SV-CORPUS-GRAD.13c.2x.3 (`PGEN-SV-CORPUS-GRAD-0250`). `sv_syntax_closure_gate` failed with `unreachable_rules=3 > max_unreachable_rules=0`; `ast_pipeline --lint-grammar` reported `unreachable_rules=0` over the same 1610-rule post-elimination universe (1607 + 3 = 1610). Measured: the gate's number is ENTRY-SCOPED and moves 3 / 14 / 1058 for entries `sv_multi_entry_root` / `systemverilog_file` / `library_text`, while `grammar_wellformedness.rs:341` documents the lint as treating an unreferenced orphan as a ROOT by design — 'only referenced-but-unreachable dead ISLANDS are caught'. Neither is wrong; the contract never named a predicate."
reverify: "for e in sv_multi_entry_root systemverilog_file library_text; do ./rust/target/debug/ast_pipeline <grammar-json> --generate-stimuli --count 1 --entry-rule $e --gap-report-json /tmp/g.json >/dev/null 2>&1; python3 -c \"import json;print('$e', json.load(open('/tmp/g.json'))['summary']['unreachable_rules'])\"; done"
---

**When two maintained instruments disagree about a metric, the instinct is to find the buggy one.
Check first whether they are computing different predicates under the same word.** Often both are
correct, deliberately so, and the disagreement is a naming failure in whatever consumes them.

PGEN had `unreachable_rules` mean two things at once:

| instrument | the question it answers | orphans |
|---|---|---|
| `--lint-grammar` | *is there a **referenced**-but-unreachable dead island?* | treated as ROOTS — invisible **by design** |
| the syntax-probe gap report | *what is reachable from **this declared entry**?* | unreachable from every entry |

Both choices are defensible and both are documented in their own source. The lint is deliberately
false-negative-safe: flagging an unreferenced orphan would reject a good grammar that legitimately
exposes several start symbols. The gap report is entry-scoped because coverage is only meaningful
relative to an entry — and that scoping is not a detail, it moved the same number **3 → 14 → 1058**
across three entries of one grammar.

The damage lands on the reader. A gate fails on `unreachable_rules`; the reader reaches for the
obvious diagnostic; it reports **0**; and the reasonable conclusion — *"the gate is broken"* — is
wrong. Worse, the *actually* stale thing (a threshold authored two months before the pass that
produces the residue even existed) stays hidden behind an argument about which tool to trust.

⭐ **What a contract owes, then, is not just a threshold but the predicate behind it:**

- which instrument produces the number, and under which parameters (here: which `--entry-rule`);
- a **named** allow-list rather than a bare count where the exceptions are knowable — `["casting_type", …]` with a reason each, not `max: 3`, because a count says nothing about *why* and re-breaks silently when the population shifts;
- the inputs whose change should invalidate it, so the threshold goes *stale* instead of *wrong* → [[a-provenance-block-must-say-whether-the-numbers-were-ever-right]].

⚠️ **And the general move that resolved this in one step: read the instrument's own source comment
before theorising about its behaviour.** The lint's root definition was written out in full, in
prose, at the top of the function. A hypothesis was already formed and about to be tested
expensively; the answer had been checked into the repository for months →
[[check-whether-the-artifact-already-states-the-property-before-building-a-detector]].
