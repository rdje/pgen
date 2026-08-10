---
id: annex-a-footnotes-license-derivations-the-productions-cannot-derive
title: Before ruling "no derivation in Annex A", read the annex's FOOTNOTES — a footnote that presupposes a construct is evidence the PRODUCTION is the transcription error, and the parser's rejection is a defect, not fidelity
answers:
  - "the corpus rejects a construct the LRM's own examples use — is that a parser bug or a correct rejection"
  - "how do I decide must_accept vs must_reject when the standard contradicts itself"
  - "Annex A cannot derive this but clause text says it is legal — which one wins"
  - "why does q[1:$] reject when constant_expression has no $ primary"
  - "how narrow should a grammar fix be when the standard's evident intent is wider"
  - "how do I avoid injecting over-acceptance while repairing an LRM-extraction gap"
  - "what distinguishes SV-CORPUS-GRAD.3.25 from .3.23 — both were 'the LRM cannot derive it'"
tags: [lrm-fidelity, adjudication, grammar-authoring, over-acceptance, systemverilog, root-cause]
date: 2026-08-10
status: current
evidence: docs/tasks/SV-CORPUS-GRAD.md leaf .3.25 (the queue-slice `q[a:$]` repair) and leaf .3.23 (the opposite verdict on `enum [N:M]`); docs/systemverilog/2017/md/section-41-data-read-api.md:3564 (A.8.4 footnote 42), :2907 + :3551 (A.8.1 + footnote 35), :3076/:3019 (part_select_range / constant_range), :2652 (indexed_range, the asymmetry); docs/systemverilog/2017/md/section-16-777-216-224-elements.md §7.10.1/§7.10.4; grammars/systemverilog.ebnf `part_select_range` / `queue_slice_range_sv_only`
reverify: "grep -q 'The .\\$ primary shall be legal only in a select for a queue variable' docs/systemverilog/2017/md/section-41-data-read-api.md && printf 'module m; int q[$]; initial q = q[1:$]; endmodule' > /tmp/pgen_kc_qslice.sv && ./rust/target/release/parseability_probe --parse systemverilog /tmp/pgen_kc_qslice.sv --profile sv_2017 >/dev/null 2>&1 && echo FOOTNOTE-42-LICENSED-AND-PARSES"
---

**`must_reject` means "the standard cannot derive this". That test is only as good as your reading
of the standard — and Annex A keeps half its rules in numbered footnotes.**

The productions tell you what can be *built*; the footnotes tell you where a construct is *allowed*.
A footnote that constrains where something may appear **presupposes that it can appear there at
all** — so when the production tree cannot produce it, the footnote is the stronger witness and the
production is the transcription error.

## The worked example

`q = q[1:$]` is written five times in IEEE 1800-2017 §7.10.4's own normative example block, and
§7.10.1 states the bounds of `Q[a:b]` *"may be arbitrary integral expressions and, in particular,
are not required to be constant expressions."* Annex A still cannot derive it:

```ebnf
select            ::= … bit_select [ [ part_select_range ] ]
part_select_range ::= constant_range | indexed_range
constant_range    ::= constant_expression : constant_expression
constant_primary  ::= …                       -- no `$` alternative anywhere
```

**A.8.4 footnote 42** settles it: *"The `$` primary shall be legal only in **a select for a queue
variable**, in an open_value_range, …"* — a sentence with no meaning unless a select can contain
`$`. ⇒ the rejection was a defect.

## The three checks, in order

1. **Search the footnotes before concluding "no derivation."** They are where the annex records the
   constraints it could not express in BNF.
2. **Look for a local asymmetry with a neighbouring production.** One line below `constant_range`,
   Annex A writes `indexed_range ::= expression +: constant_expression` against
   `constant_indexed_range ::= constant_expression +: constant_expression` — it *does* separate the
   select form from the constant-select form on the `expression` vs `constant_expression` axis, and
   then lets `part_select_range` share `constant_range` anyway. An inconsistency with the line next
   to it is far stronger evidence than any reading of the prose.
3. **Repair only what the contradiction licenses, and MEASURE the wider fix before rejecting it.**
   Following the "evident intent" (make both bounds `expression`) also takes `v[a++ : b]` and
   `v[a inside {1,2} : 0]` from REJECT to PASS. Adding `$` and nothing else leaves every non-`$`
   bound on the original path, so no input changes verdict in either direction.

## ⛔ The opposite verdict exists, and the difference is a citation

`SV-CORPUS-GRAD.3.23` asked the same question about `enum [N:M] { … }` and answered the other way:
**nothing** in the LRM licensed it, so all nine rows were pinned `must_reject`. The two leaves are
separated by whether a named clause or footnote can be quoted — never by how plausible the construct
looks or by how many suites write it. *A construct every vendor accepts is still `must_reject` if no
sentence of the standard licenses it.*

## Two riders

- **Profile-gate the repair.** `$` and queues are IEEE 1800 only; the fix is declared
  `@profiles: ["sv_2017", "sv_2023"]`, and the `verilog_2005` manifest staying byte-identical is
  what proves the gate held rather than the claim that it should.
- **A fix that moves `furthest_position` deeper is a partial fix.** Two of the seven rows kept
  rejecting after the repair — they had cleared the `$` bound and stopped at the *next* construct.
  Report five, route the rest; see [[a-rising-pass-rate-is-not-evidence-of-correctness]].

See also [[a-rising-pass-rate-is-not-evidence-of-correctness]],
[[a-column-0-comment-inside-a-rule-body-deletes-the-following-alternatives]].
