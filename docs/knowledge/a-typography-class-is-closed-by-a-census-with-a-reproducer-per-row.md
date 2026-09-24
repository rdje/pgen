---
id: a-typography-class-is-closed-by-a-census-with-a-reproducer-per-row
title: A typography class is closed by a census with clause evidence and a reproducer per row — the extraction cannot tell bold from plain, so the population is measured, not remembered
answers:
  - "why does the LRM text under docs/ show literal tokens and EBNF optionality with the same brackets"
  - "how do I decide whether an Annex A bracket pair is a literal token or optionality"
  - "which Annex A productions have literal brackets and does the grammar render them"
  - "why does $width with two arguments reject on sv_2017 but accept on verilog_2005"
  - "where are the literal-bracket reproducers pinned and how do I re-run the census"
tags: [sv-corpus, lrm-fidelity, annex-a, census, reproducers, measurement-discipline]
date: 2026-09-24
status: current
evidence: "SV-CORPUS-GRAD.21 (PGEN-SV-CORPUS-GRAD-0418): 1178 bracket pairs across IEEE 1800-2017 A, 1800-2023 A and 1364-2005 A; 88 LITERAL over 59 edition x production combinations, each with clause evidence and a reproducer; 33/38/17 ACCEPT on sv_2017/sv_2023/verilog_2005, zero rendered as optional; $width's threshold optional under 1364-2005 A.7.5.1, mandatory under 1800 A.7.5.1."
reverify: "python3 docs/tasks/artifacts/sv_corpus_grad/annex_a_brackets/census.py --check"
---

**IEEE prints a literal token in bold and EBNF optionality in plain brackets; the PDF-to-text
extraction under `docs/` keeps only the characters. A bold `[` and an optional `[` are the same byte,
so a production transcribed by reading is a coin toss the grammar pays for later — twice, before the
population was measured.**

## What decides a pair

- **The characters alone**, where they can: an empty `[ ]` is an unsized dimension; an operator-led
  `[$`, `[*`, `[+`, `[->` is a queue, a repetition, a goto; `[=` is a repetition only inside a
  repetition production. These rules are generic and never wrong.
- **The clause**, everywhere else: dimensions, selects, ranges, `new [ n ]`, enum ranges, bins, value
  ranges, the sequence and property operators, `with [ ]`, the specify terminals, `edge [ ]`. Each row
  cites the clause's own example that spells the bracket (`[7:0]`, `nexttime [2] p`, `bins b[] = …`),
  never Annex A itself, and where no example carries the bracket the provenance of the reading is named.
- **EBNF optionality by default** for the other 1 090 pairs — `[ : module_identifier ]`, `[ = expression ]`,
  `[ virtual ]` — which is what the extraction faithfully rendered.

## Why the probe is the proof and the rule text is a hint

A rule-text search for `lbrack` within three references misses arms that live beyond an LR-eliminated
chain (`property_expr`'s operators sit in `prop_primary_sv_2017`); a reproducer parsed on the edition's
profile does not. Every literal row carries one, pinned in `stimuli/sv/adjudication_repros/` so the
release ceremony reads it each release. A construct that parses through the WRONG production still
accepts — the rule-fire partition is the instrument that reads the arm
([[a-ratchet-ceiling-goes-stale-from-below-when-the-instrument-improves]] for the same shape of
"which instrument answers which question").

## A reject is a question for the standard first

`$width(posedge clk, 1)` rejected on the SV profiles and accepted on verilog_2005. The brackets were
innocent: IEEE 1364-2005 A.7.5.1 writes `[ , threshold [ , notifier ] ]` and IEEE 1800 A.7.5.1 writes
`, threshold [ , [ notifier ] ]` — the threshold became mandatory — and the grammar already carried both
as profile twins. The two-sided pin keeps that a fact rather than a re-derivation.
