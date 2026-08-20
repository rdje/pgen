---
id: a-coverage-report-cannot-see-a-production-the-grammar-never-declares
title: Coverage measures reach over the DECLARED rule set, so a construct the grammar never spells is invisible to it — a missing rule cannot be UNKNOWN, only absent
answers:
  - "my certificate/coverage report is clean — does that mean the grammar is right"
  - "what class of grammar defect can coverage instruments never report"
  - "two coverage instruments agree — how much confidence does that buy me"
  - "the residual UNKNOWN list is adjudicated benign — is there anything left to do"
  - "should I rebaseline a red gate once every number checks out"
  - "where do I look when a keyword is rejected but no rule is missing from the report"
tags: [coverage, certificates, oracles, grammar-fidelity, gates, baselines, diagnosis]
date: 2026-08-20
status: current
evidence: "SV-CORPUS-GRAD.13c.2x(a) (`PGEN-SV-CORPUS-GRAD-0259`). Adjudicating `sv_cert_recognized_union_gate`'s 53 residual union `UNKNOWN`s found all 53 benign — 43 LR-eliminator-synthesised `property_expr_lr_*` rules (independently `GAP` 0-of-184-files in `stimuli/sv/characterization/rule_coverage_sv_2017.tsv`) and 10 witness-planner reach gaps whose constructs each PARSED. Reading `prop_primary_sv_2017` to explain the 10 found `grammars/systemverilog.ebnf:6666` — `implies := trivia \"->\"`, a token rule named after an IEEE 1800 keyword — so `a implies b` (A.2.10) is REJECTED while `a -> b`, `-> b` and `a |= b` are ACCEPTED though A.2.10 defines none. There is no `kw_implies_*` anywhere, so the missing production has no rule, hence no row, in either instrument."
reverify: "printf 'kw_implies in the SV grammar: %s\\nrows naming it in the corpus coverage TSV: %s\\nwhat the rule called implies actually is: %s\\n' \"$(grep -c kw_implies grammars/systemverilog.ebnf)\" \"$(grep -c kw_implies stimuli/sv/characterization/rule_coverage_sv_2017.tsv)\" \"$(grep -m1 '^implies :=' grammars/systemverilog.ebnf)\""
---

**A coverage report answers *"is every rule I declared reachable and exercised?"* It never answers
*"did I declare the right rules?"*** Those look like the same question from inside the report,
because the report's row set *is* the declared rule set. A production the grammar never spells
produces no row — not an `UNKNOWN` row, not a `GAP` row, no row at all — so a perfectly clean
report and a grammar missing an operator are indistinguishable from the inside.

PGEN measured this on SystemVerilog. `sv_cert_recognized_union_gate` had 53 residual `UNKNOWN`s and
every one of them adjudicated benign:

| class | count | verdict |
|---|---:|---|
| `property_expr_lr_*` — authored by PGEN's own left-recursion eliminator | 43 | cannot be witnessed; corroborated `GAP` (0 files of 184) by an independent corpus instrument |
| `kw_*` tokens for A.2.10 property operators + one property identifier | 10 | witness-planner reach gaps; **each construct parses** |

That is a clean result, and it is also the end of what the instrument can say. The defect sitting
in the same rule — `prop_primary` binding an *arrow token named `implies`* where IEEE 1800 spells
the *keyword* `implies` — has no row in either report, because the keyword has no rule.

⛔ **Two instruments agreeing does not widen the window when both look through the same frame.** The
certificate pass and the corpus rule-coverage TSV have different inputs, different code and
different authors, and they agreed exactly on all 43 rules. They share the blind spot anyway,
because they share the row set. Only an oracle *outside* the grammar — the standard's own text, an
external corpus, a hand-written probe — can testify that the declared grammar is the language.

⭐ **The residual roster is a POINTER, not a conclusion.** The nine unwitnessable keywords did not
report the defect; they said *look at `prop_primary`*, and reading the rule found it. Treat an
adjudicated-benign residual as an instruction to read the rule the residual lives in, not as a file
to close.

⛔⛔ **Which is why "adjudicate, then rebaseline" is an ordering and not a formality.** A rebaseline
does not merely record a number — it closes the question that produced it. Had the red gate been
re-derived first, the contract would today record `union UNKNOWN = 53` with every field correct, the
gate would be green, and a standard-legal operator would still be rejected, with the green gate
standing as evidence that the area had been examined.
