---
id: which-pgen-grammars-are-certified-and-which-are-not
title: Which PGEN grammars are certified, which are not, and why — 6 of 9 shipped families are fully certified; return_annotation and semantic_annotation are not, and SystemVerilog's proof is stale
answers:
  - "which grammars are certified"
  - "is the SystemVerilog parser fully certified"
  - "is the SV parser certified now"
  - "which families have full certificate coverage"
  - "how many grammars are fully certified"
  - "which grammar is not certified and why"
  - "what is the definition of certified in PGEN"
  - "what does fully_certified mean"
  - "does certified mean the parser is correct"
  - "where do I find per-grammar certification status"
  - "is a green certification still valid after a regeneration"
tags: [certification, certificate-coverage, status, grammars, freshness, signoff]
date: 2026-08-23
status: current
evidence: |
  MEASURED at HEAD 2026-08-23 by `scripts/report_grammar_certification.sh`, which RUNS
  `ast_pipeline --report-certificate-coverage` per family against the generated parsers in the tree.
  Published as `docs/book/src/grammar-certification-status.md`, which the same script re-derives and
  diffs (`--check`).

    json                        9 /  9 witness / 0 proof /  0 unknown   fully_certified=true
    regex                     269 /260 witness / 9 proof /  0 unknown   fully_certified=true
    rtl_const_expr             48 / 48 witness / 0 proof /  0 unknown   fully_certified=true  (--max-depth 32)
    rtl_frontend              169 /168 witness / 1 proof /  0 unknown   fully_certified=true
    systemverilog_preprocessor 74 / 74 witness / 0 proof /  0 unknown   fully_certified=true
    vhdl                      225 /225 witness / 0 proof /  0 unknown   fully_certified=true
    return_annotation          35 / 33 witness / 0 proof /  2 unknown   fully_certified=false
    semantic_annotation       119 / 90 witness / 0 proof / 29 unknown   fully_certified=false
    systemverilog            1385 /1378 witness/ 7 proof /  0 unknown   UNION over 4 configs; canonical unknown 11; proof STALE

  ⛔ THE FIRST VERSION OF THIS CARD SAID 0/9 WITH SEVEN FAMILIES "NO ORACLE". It was wrong: the
  producing script asked whether a `*cert*contract*.json` file existed rather than running the
  oracle. Five families that certify cleanly were reported as never scored.

  ⛔ AND THE FIX FOR THAT SHIPPED A SECOND DEFECT: it deleted `--check`'s implementation while
  leaving the FLAG, so for one commit `--check` printed a table and exited 0 on every input,
  including a path that does not exist. Restored with five observed arms
  (GRAMMAR-CERT-STATUS.1b) -> [[deleting-an-implementation-while-leaving-its-flag-fails-silently]].
reverify: "bash scripts/report_grammar_certification.sh"
---

## The definition (PGEN's own, `GRAMMAR-WELLFORMED.G.4`)

Quoted from the oracle's own help text:

> For every rule, is it covered by a verified unreachability **PROOF** or a verified reachability
> **WITNESS** (a clean diverse `--count` sample that parses through the real parser and exercises
> it)? `UNKNOWN`=0 with no failures = the objective *"trustworthy on this grammar"* number.

A grammar is **certified** iff every rule is either proven unreachable or witnessed by a generated
sample that parses **through the real generated parser**, with `UNKNOWN=0`,
`sample_parse_failures=0`, `proof_reverify_failures=0`.

## The answer

**6 of 9 shipped grammars are certified**: `json`, `regex`, `rtl_const_expr`, `rtl_frontend`,
`systemverilog_preprocessor`, `vhdl`.

**Two are not**: `return_annotation` (2 unknown of 35) and `semantic_annotation` (29 unknown of 119).

**One is unverified**: `systemverilog`. Its union over four entry/profile configs last reached 0
unknown, but the proof pins a different parser digest than the tree holds — engine-universal codegen
moved under it, while the grammar itself did not.

## Three things it does not mean

1. **Not correctness.** Certification asks whether the generator can reach every rule of *our*
   grammar. A certified grammar can still accept invalid input or reject valid input — that is the
   corpus axis (SystemVerilog: 46.3 % adjudicated, 275 known defects).
2. **Not "a string for every rule".** A rule may be credited by a **proof** that no input reaches it.
   SystemVerilog credits 7 rules that way.
3. **Not durable.** A proof describes one specific generated parser. When codegen moves, the parser
   moves and the proof keeps reading green while describing something that no longer exists.

## How to ask a family under its own parameters

`rtl_const_expr` certifies at `--max-depth 32`, which its own contract declares; at the CLI default
of 24 its generator produces nothing at all. **A verdict is only as good as whose parameters it was
taken under** — that error produced a false `NO ORACLE` table, a false "this family is broken"
finding, and a certification answer read out of a self-authored contract, all in one session.

Related: [[a-control-that-cannot-fail-is-not-a-control]] ·
[[a-corpus-generated-from-the-artifact-under-test-is-part-of-the-measurement]]
