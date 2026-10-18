---
id: a-generated-sample-the-parser-refuses-is-a-boundary-or-a-defect
title: A generated sample the parser refuses is either a lexical boundary the grammar never declared or a parser defect — read every one three ways before discounting any (`SV-CORPUS-GRAD.13c.2v.16`)
answers:
  - "what do certificate sample_parse_failures mean"
  - "is a sample parse failure a generator bug or a parser bug"
  - "why did the svpp certificate regress when the corpus census passed"
  - "how do I declare a token boundary for the stimuli generator"
  - "why is an end-of-input anchor generated in the middle of a sample"
tags: [certificate, stimuli-generator, lexical-annotations, follow-restriction, preprocessor, measurement]
date: 2026-10-18
provenance: SV-CORPUS-GRAD.13c.2v.16 (PGEN-SV-CORPUS-GRAD-0464); ledger SVPP-0015; scratch rust/target/svpp9/certfail_extract.py and candidates g10 … g15
reverify: "rust/target/debug/ast_pipeline grammars/systemverilog_preprocessor.ebnf --report-certificate-coverage --entry-rule systemverilog_preprocessor_file --count 40 --seed 0 | grep -m1 CERTIFICATE-COVERAGE   # fully_certified=true sample_parse_failures=0"
---

# A generated sample the parser refuses is a boundary or a defect

**The certificate's generator writes text no corpus holds, so its refused samples reach forms a 16,336-file census
never sees. Each refused sample is one of exactly two things, and the two need opposite fixes.**

1. **A lexical boundary the grammar never declared.** The parser is right: the generator composed two tokens whose
   concatenation re-reads differently — a `/` before `/*` becomes `//*`, an escaped identifier swallows the `(` after
   it, a `//` comment swallows the `` `endif `` rendered on its line, a zero-width `\z` drawn mid-file. The fix is to
   DECLARE the boundary the parser already holds: a `[>! … ]` follow restriction (a parse-direction no-op; the
   generator appends the separator the forbidden set cannot start), each a true statement of the clause it cites —
   and, where no directive can say it, a neutral generator service that reads the grammar's own construct (the
   end-of-input anchor: an alternative that mandatorily renders `\z` is not drawn while a non-empty element follows).
   A rule-level `@sample` renders the rule's text and nothing after it; it bypasses every restriction below it.
2. **A parser defect.** The text is valid and the parser refuses it. In `systemverilog_preprocessor` `1.0.9`'s first
   build the samples found three the census had not: a define ending the file with trailing white space, a comment-only
   formal default, and macro actuals read without lexing (`` `include `M(")") ``, ledger `SVPP-0015`, present since
   `1.0.8`).

## How to tell them apart

Extract each refused sample byte-exact (keep CR/LF — a text-mode read folds `\r\n` and shifts every offset), then
parse it three ways: the generated parser, the interpreter on the committed grammar, the interpreter on the candidate
fix. The first two must agree (if not, it is an engine divergence — a third kind). A sample the candidate's own
interpreter accepts is fixed by the rebuild; one it still refuses is unexplained, and is read until it is attributed.
Bisect each to a minimal reproducer and run it on the released parser too: that decides whether a defect needs a
ledger row (released) or is this build's regression.

## Why the census cannot replace it

The census measures what the corpus contains. `` `define A `` with a trailing space and no final newline, a default
that is only a comment, `( … )` inside a string in an `` `include `` actual — no file of 16,336 holds them, and the
census passed while the certificate failed. Both instruments are needed; neither discounts the other.

Related: [[a-census-of-refusals-cannot-see-an-over-acceptance]],
[[a-fallback-arm-goes-inside-the-shared-rule-as-an-ordered-last-alternative]].
