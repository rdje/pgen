---
name: two-escaping-rules-that-look-alike-and-only-one-is-real
description: "DISCIPLINE (2026-08-27, LIVE-DOC-CONTAINMENT.11 / PGEN-LIVE-DOC-CONTAINMENT-0017, found by a director re-verification challenge) — when a format has SEVERAL parsing passes, a backslash means different things to each, and generalising one pass's escaping rule to another invents defects in correct text. MEASURED in one markdown table row: `a\\|b` renders as a|b, because GFM splits table cells BEFORE inline parsing, so the pipe rule really does honour the backslash and consumes it; while a backtick-delimited span holding a single backslash renders as a code span containing that backslash, because CommonMark parses code spans BEFORE backslash escapes and states that backslash escapes do not work inside them. A tracked instrument here applied the FIRST rule to the second case — skipping any backtick preceded by an odd number of backslashes — and read a shipped contract at 4 cells with an unpaired backtick where both renderers see 0, every one of them a correct span. It was one step from becoming the second arm of a commit-blocking gate that would have refused four correct rows in a document this project ships. ⇒ before reusing an escaping rule across a format's passes, name WHICH pass it belongs to and check the other pass against a renderer; two rules that look identical in the source can be opposite in effect."
id: two-escaping-rules-that-look-alike-and-only-one-is-real
title: "Two escaping rules that look alike, and only one is real — name the parsing pass before you reuse a backslash rule"
date: 2026-08-27
evidence: "docs/tasks/LIVE-DOC-CONTAINMENT.md leaf .11, field `RE_VERIFICATION_CHALLENGE_2026-08-27` item (1) (the 4-versus-0 measurement and the four rows it would have refused); the corrected `code_spans()` docstring in docs/tasks/artifacts/live_means_live/table_pipe_repair.py; the self-test regression pin in scripts/check_table_arity.sh"
reverify: "bash docs/tasks/artifacts/live_means_live/backslash_rule_demo.sh   # one row, two backslashes: the code-span one survives as content, the table-pipe one is consumed; exits non-zero if either stops holding"
answers:
  - "does a backslash escape a backtick in markdown"
  - "why does my code-span scanner disagree with the renderer"
  - "can I reuse the same escaping rule across a format's passes"
  - "why did my lint report defects in text that renders correctly"
  - "what order does GFM parse table cells and inline code in"
metadata:
  node_type: memory
  type: feedback
  created: 2026-08-27
---

## The rule

A format with several parsing passes has several escaping rules, and they are not the same rule.
Before reusing one, name the **pass** it belongs to, and check the other pass against a renderer
rather than against intuition. Two backslashes that look identical in the source can be opposite in
effect.

## Why — the measurement

One markdown table row contains both rules:

| written | renders as | what the backslash did |
| --- | --- | --- |
| `a\|b` | `a\|b` in one cell | **escaped the pipe, and was consumed** — GFM splits cells *before* inline parsing, so the pipe rule runs first |
| a backtick span holding one backslash | a code span containing `\` | **nothing** — it is the span's *content*; CommonMark parses code spans *before* backslash escapes, and escapes do not work inside them |

A tracked instrument here generalised the first rule to the second: its code-span scanner skipped any
backtick preceded by an odd number of backslashes. Under that rule the released-parser bug ledger
reads **4** cells carrying an unpaired backtick run. Both renderers read **0**. All four were correct
spans.

That mis-rule was one step from becoming the second arm of a commit-blocking gate — which would have
refused four correct rows in a document this project ships to a downstream consumer, and sent an
author to "repair" text that was already right.

## What caught it

Not re-reading the scanner, and not a second run of it. A **re-verification challenge** forced the
question *"do you still stand by this?"*, and answering it honestly meant re-deriving the numbers
against a renderer instead of against the model that produced them. The disagreement was one row in
2 573 — invisible in a summary, decisive once compared.

## The transferable shape

Look for this wherever a document or protocol is parsed in stages:

- shell quoting inside a `Makefile` recipe (make expands, then the shell parses);
- a regex written inside a YAML string (YAML unescapes, then the regex engine does);
- SQL inside an ORM string literal; a URL inside a JSON body inside a shell heredoc;
- any templating layer above a language that also has escapes.

The tell is that the two rules are spelled the same and the failure only shows on inputs where a
backslash appears in the *other* pass's territory — which is rare, so the wrong rule looks right for
a long time.

⚠️ Note what is NOT the lesson: masking `\|` before splitting table cells is **correct** and is load
bearing here — ignoring it reads the same ledger at 94 defects instead of 26. The defect was
extending a real rule past its own pass.

Related: [[the-optional-half-of-a-delimiter-is-where-a-counter-goes-blind]] and
[[a-delimiter-imbalance-is-not-a-local-defect]] — the other two ways a hand-written model of a
markdown rule went wrong in this leaf.
