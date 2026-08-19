# Consult the tracked LRMs rather than asking a low-level standards question

**Director, 2026-08-19** (during `PGEN-SV-CORPUS-GRAD-0234`), on a callout that had offered a
`simple_type`-vs-`data_type` reading of IEEE 1800 §10.9.2 as *"your call"*:

> *"it is yours to make, it is a technical question. You need to make sure the PGEN SystemVerilog
> parser is fully compliant with LRM IEEE 1800-2017/2023. Your decision shall steer the parser in
> that direction, so it got to be sota, signoff and ensure full compliance."*

and, immediately after:

> *"Next time, don't ask me low level questions like this, please consult the LRMs, they are
> available so please use them."*

## What this binds

A question that the standards **answer** is not a director call, however consequential it feels.
Reading two normative clauses and deciding what the parser must accept IS the engineering work, not
a precondition for it. Escalating it spends a director turn on something the repository already
contains, and — worse — it frames a compliance decision as a preference.

This is the same directive as [[feedback_answer_your_own_technical_questions]], now with a named
source: **the LRM is the authority, and it is in the repo.** Escalate only what changes *what gets
built* — scope, priority, an objective — never *what the standard says*.

## Where they are

All three, tracked, each in three surfaces:

```
docs/systemverilog/2017/{md,txt}/  + SystemVerilog-LRM-IEEE-1800-2017.pdf
docs/systemverilog/2023/{md,txt}/  + SystemVerilog-LRM-IEEE-1800-2023.pdf
docs/verilog/2005/{md,txt}/        + Verilog-LRM-IEEE-1364-2005.pdf
```

⭐ `md/` is the better read for **clause text and worked examples** (it preserves structure);
`txt/` is the better read for **Annex A productions** (one production per line, greppable). Use
both — `-0234`'s decision turned on §10.9.2's prose in `md/` disagreeing with Annex A in `txt/`, and
either surface alone would have given a confident wrong answer.

## The standing rule this produced

⛔ **Compliance is the union of Annex A and the clause text, not Annex A alone.** They disagree, in
both editions, and PGEN's grammar was extracted from Annex A — so every disagreement is a latent
rejects-valid defect. When they conflict: accept what either normative clause licenses, reject only
what neither does. Tracked as a class in `SV-CORPUS-GRAD.13c.2v`.

Related: [[feedback_answer_your_own_technical_questions]],
[[feedback_sv_strict_lrm_compliance_default]], [[an-over-acceptance-can-be-load-bearing]].
