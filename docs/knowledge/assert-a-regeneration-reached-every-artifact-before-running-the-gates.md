---
id: assert-a-regeneration-reached-every-artifact-before-running-the-gates
title: A "regenerate everything" recipe's coverage is a census claim — assert the new emission is in EVERY artifact (and the old one in none) before running any gate that assumes it, and hold the gate's family list equal to the recipe's
answers:
  - "I changed the code generator and ran regenerate — how do I know every artifact was actually re-emitted"
  - "the reproducibility gate refuses to rebaseline after a regeneration — why"
  - "regenerate_generated_parsers exited 0 but some generated files still carry the old text"
  - "which generated artifacts does regenerate_generated_parsers not cover"
  - "how do I check an emitter edit landed in the generated parsers"
  - "why does the gate cover a family the Makefile does not regenerate"
tags: [codegen, regeneration, make, reproducibility, census, gates, ci-parity-gate-rot, sv-corpus-grad]
date: 2026-09-02
status: current
evidence: "SV-CORPUS-GRAD.13e.12(d) / PGEN-SV-CORPUS-GRAD-0336, re-measuring CI-PARITY-GATE-ROT.49. A comment-only edit to one `quote!`-emitted doc comment in `ast_based_generator.rs`, then `make regenerate_generated_parsers` (exit 0). `grep -l 'is NOT the failed-speculation' generated/*.rs | wc -l` read 9 and `grep -l 'FAILED-speculation entry count' generated/*.rs` named 2: `generated/ebnf.rs` (the seed — `regex_parser_bootstrap` guards it with `if [ ! -f generated/ebnf.rs ]`) and `generated/scratch_parser.rs` (`scratch` is in `check_generated_reproducibility.sh`'s FAMILIES but not in the Makefile's GENERATED_PARSER_FAMILIES). `GENERATED-REPRODUCIBILITY` tier 2 then refused to record, printing the fresh derivation's `+` lines the live artifact lacked — the same shape ENGINE-UNIVERSAL-SERVICES.66 measured one month earlier."
reverify: "grep -l 'is NOT the failed-speculation' generated/*.rs | wc -l; grep -l 'FAILED-speculation entry count' generated/*.rs | wc -l   # 11 and 0 after a complete regeneration; 9 and 2 after `regenerate_generated_parsers` alone"
---

**A recipe named *regenerate everything* makes a census claim about which artifacts it reaches, and
`exit 0` is not evidence for it.** Two independent measurements a month apart both read nine of
eleven.

## The one-line assertion that costs nothing

After any emitter edit, before any gate:

```bash
grep -l '<a sentence only the NEW emission contains>' generated/*.rs | wc -l   # must equal the artifact count
grep -l '<a sentence only the OLD emission contains>' generated/*.rs            # must name nothing
```

Two greps, both directions, over the WHOLE glob — not `*_parser.rs`, which is how the first census
of this class missed `ebnf.rs` ([[a-heading-census-is-only-as-good-as-the-heading-grammar]]).
It is the same shape as the `-0039` rule *assert the work happened (`grep -c Generating`), never
trust make's exit 0*, applied to the OUTPUT rather than the log.

## Why the recipe misses, twice, for two different reasons

| artifact | why `regenerate_generated_parsers` skips it | the fix shape |
|---|---|---|
| the `ebnf.rs` **seed** | an EXISTENCE guard (`[ ! -f … ]`) that is correct for the cold-clone bootstrap and wrong for a warm tree | `rm generated/ebnf.rs && make regex_parser_bootstrap`; long-term, a guard on freshness rather than existence — ⛔ not an mtime rule under GNU Make 3.81 (whole-second compare) |
| `scratch_parser.rs` | the reproducibility gate's family list has **eight** entries and the Makefile's has **seven** — two lists, maintained separately | `make focus_scratch`; long-term, hold the two lists equal, or derive one from the other |

⛔ **A gate that covers a family the recipe does not regenerate will breach on every emitter
change** — and a gate that breaches on every change is a gate people learn to bypass. The lists
must be held equal by a check, not by memory.

## What the gate is for

`GENERATED-REPRODUCIBILITY` tier 2 re-derives every artifact from HEAD and diffs; it caught both
misses, both times, and refused to record. That is the doctrine being load-bearing, not
ceremonial — and it is also the reason to run the two greps FIRST: the gate tells you *that* the
tree diverges, the greps tell you *which* artifacts and *why*, before an 18-minute release rebuild
has been spent on a tree that was never fully regenerated.
