---
id: a-specialised-rule-masked-by-a-general-fallback-is-untestable-from-source
title: When a GENERAL production accepts the same text as the specialised one, a defect in the specialised rule is invisible from source — enter the rule directly, then find the input the fallback cannot rescue
answers:
  - "a grammar rule looks broken but every source-level probe parses — how do I test it"
  - "how do I probe a mid-grammar rule in isolation without writing a whole source file"
  - "my census flagged candidate rules but they all seem to work — how do I confirm or clear them"
  - "is it safe to fix a rule that no source-level input reaches"
  - "how do I know whether a specialised production is reachable at all"
  - "why did a defect only appear for one particular argument shape"
  - "a fix changes no verdict on any real input — should I still land it"
tags: [grammar-authoring, instrument-soundness, reachability, adjudication, toolbox, systemverilog, over-rejection]
date: 2026-08-19
status: current
evidence: docs/tasks/SV-CORPUS-GRAD.md leaf .13c.2s (PGEN-SV-CORPUS-GRAD-0242); docs/tasks/artifacts/sv_corpus_grad/systf_clocking_event/; ledger rows SV-0063/SV-0064; grammars/systemverilog.ebnf `let_list_of_arguments` / `property_list_of_arguments` / `sequence_list_of_arguments` versus `list_of_arguments`
reverify: "printf 'a, .b(2)' > /tmp/p.txt && ./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --interpret-parse /tmp/p.txt --grammar-profile sv_2017 --interpret-entry-rule sequence_list_of_arguments   # then swap the entry rule for list_of_arguments and watch the same text be accepted by the general route"
---

A grammar that has both a **specialised** production and a **general** one over overlapping text
has a blind spot: a defect in the specialised rule produces no observable symptom for as long as
the general rule can parse the same input. Every source-level probe passes. Every corpus number is
unmoved. The rule is broken and nothing says so.

## Measured

`SV-CORPUS-GRAD.13c.2r`'s starving-star census flagged `let_list_of_arguments`,
`property_list_of_arguments` and `sequence_list_of_arguments` as carrying a defective shape, then
had to file them **unconfirmed**: source-level probes showed the construct parsing.

Entering each rule directly (`ast_pipeline --interpret-entry-rule`) settles it in one command each:

| entry rule | `a, .b(2)` | `.b(2)` | `a, c` |
|---|---|---|---|
| `let_list_of_arguments` | **REJECT** | ACCEPT | ACCEPT |
| `property_list_of_arguments` | **REJECT** | ACCEPT | ACCEPT |
| `sequence_list_of_arguments` | **REJECT** | ACCEPT | ACCEPT |
| `list_of_arguments` — the general one | **ACCEPT** | — | — |

All three starve; the general production rescues the call site. Two controls per row (named-only,
positional-only) attribute the rejection to the *mix* alone rather than to named arguments or to
commas.

## The rescue has a boundary, and the boundary is where the defect becomes real

A masked defect is not automatically a harmless one. Ask: **what can the specialised rule parse that
the general one cannot?** Feed exactly that.

- `sq(a, .q(b))` — every positional argument is also a plain expression ⇒ the general route rescues
  it ⇒ parses, before and after any fix.
- `sq(a ##1 b, .q(d))` — `a ##1 b` is a `sequence_actual_arg` and **not** an expression ⇒ neither
  route can parse it ⇒ **REJECT**, a live rejects-valid defect.
- `sq(a ##1 b, d)` — the same sequence-shaped argument with the named one removed ⇒ parses on both
  sides, which is what makes the flip attributable to the mix.

That third row is the control that turns the second from an anecdote into an isolation.

## Then check reachability before you fix

⛔ A fix to a rule nothing reaches is a widening with a risk profile and no benefit. Establish that
the rule is live, from the parser's own output rather than from the reference graph: on
`sq(a ##1 b)` the tree carries `sequence_list_of_arguments`' own `ordered_tail`/`named_tail` fields;
on `sq(a, .q(b))` it carries the general `list_of_arguments` shape instead. The rule *is* on the
path — just not for the masked form.

## And say plainly when a site has no demonstrated flip

The same fix applied to `let_list_of_arguments` moves nothing at source level, because
`let_actual_arg := expression` and the general route therefore always rescues it. It shipped anyway
— the defect is identical and leaving one member of a swept class unfixed is what creates the *next*
one-clause-away recurrence — but the contract, the ledger row and the grammar comment all say **"no
demonstrated source-level flip"** rather than implying one. A swept class and an evidenced claim are
different things and both are worth having.

Related: [[a-catch-all-alternative-makes-an-accept-meaningless]] (the same masking one level down,
between alternatives of a single rule, where the fix is to read the ARM),
[[an-accept-set-watch-cannot-see-a-replaced-ast-shape]] (the third member of this family: the text
parses, the tree differs), [[a-check-whose-inputs-all-pass-has-not-been-tested]].
