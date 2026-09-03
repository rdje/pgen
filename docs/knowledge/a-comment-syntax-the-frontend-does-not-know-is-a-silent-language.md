---
id: a-comment-syntax-the-frontend-does-not-know-is-a-silent-language
title: A comment syntax the frontend does not know is a silent language — implement the meta-grammar's comment rule, not one spelling of it, and diff the WHOLE envelope after a frontend change
answers:
  - "why did commented-out alternatives end up live in the generated parser"
  - "the meta-grammar allows // comments — does the EBNF frontend"
  - "why did a comment-only edit move the ebnf_raw_ast digest"
  - "how do I know a frontend fix changed only what I meant to change"
  - "cargo test --lib passed with 0 tests — is the module feature-gated"
tags: [ebnf-frontend, comments, meta-grammar, raw-ast, envelope, regression, feature-gates]
date: 2026-09-03
status: current
evidence: "PGEN-EBNF-FRONTEND-SILENT-TRUNCATION-0002: grammars/ebnf.ebnf:583-586 defines `#` and `//` as line comments; rust/src/ebnf_frontend.rs knew `#`. 77 `//` lines in grammars/systemverilog.ebnf survived by being dropped or swallowed, and `parseable_source_item`'s three `//`-commented alternatives were LIVE in the released parser (each with an empty regex token), reachable via --entry-rule systemverilog_parseable_file. 14 return-annotation payloads in 11 rules carried the comment that followed them (return_scalar). Fix: comments cut from payloads, `//` a comment in all four frontend sites; one generated parser moved in one rule, ten byte-identical; SV release 1.0.205, ledger SV-0078."
reverify: "bash docs/tasks/artifacts/ebnf_frontend_silent_truncation/comment_payload/probe.sh   # census over every readable grammar, the founding synthetic in both spellings, the comment-edit invariance, // in every position"
---

**If the meta-grammar says it is a comment and the frontend does not, the text is silently something else.**

## What happened

| where | what the old frontend did with a `//` line | consequence |
|---|---|---|
| top level, after a blank line (35 runs) | dropped as an unrecognised line | nothing visible |
| column 0 inside a body, after `-> {…}` (2) | swallowed into the annotation payload up to the first `\|` | `return_scalar`, digest keyed on comment text |
| the `\|`-carrying lines of `parseable_source_item` | payload stopped at `\|`; the next `//` became an EMPTY regex literal | three commented-out alternatives **live in the shipped parser** for four months |

## The rules

1. **Implement the meta-grammar's rule, not one spelling of it.** `line_comment := ("#" | "//") …` means every
   scanner in the frontend — top-level, body collector, tokenizer, delimiter tracker, payload extractor — treats
   both the same. Fix the frontend; do not rewrite the 77 lines (the grammar is right by its own spec).
2. **Comments are cut out of a payload, not "tracked" around.** Tracking a delimiter tells you where `|` is
   safe; it does not remove the comment from the slice you return.
3. **After a frontend change, diff the WHOLE envelope.** Scrub the tokens you meant to change and ask what else
   moved. The leaf's 14 tokens were the intended change; the rule that moved outside them was the real defect.
4. **A test filter that matches nothing is a green run of nothing.** Assert the number of tests you expect, and
   know which modules are feature-gated (`cargo test --lib --features ebnf_dual_run -- ebnf_frontend::tests`).
5. **Unreachable from the entry is not unobservable.** `--entry-rule` reaches it; pin the repro entry-relative
   (the repro ratchet's `entry` column exists for this).

## Related
[[a-missing-alternative-is-invisible-to-every-instrument-that-reads-only-what-exists]] — the mirror image: here
an EXTRA alternative was invisible to every instrument that reads only the registered entry.
