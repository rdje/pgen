---
id: which-pgen-grammars-are-certified-and-which-are-not
title: Which PGEN grammars are certified, which are not, and why — 2 of 9 shipped families even have a certification oracle, and 0 of 9 are certified AND fresh
answers:
  - "which grammars are certified"
  - "is the SystemVerilog parser fully certified"
  - "is the SV parser certified now"
  - "which families have full certificate coverage"
  - "how many grammars are fully certified"
  - "which grammar is not certified and why"
  - "does certified mean the parser is correct"
  - "where do I find per-grammar certification status"
  - "is a green certification still valid after a regeneration"
tags: [certification, certificate-coverage, status, grammars, freshness, signoff]
date: 2026-08-23
status: current
evidence: |
  Derived by `scripts/report_grammar_certification.sh` over the tracked contracts and the parsers in
  the tree, 2026-08-23. Published as `docs/book/src/grammar-certification-status.md`, which the same
  script re-derives and diffs (`--check`), so the page cannot drift from the tree.

  **certified_and_fresh = 0 / 9.** Nine shipped families (`generated/*_parser.rs`, minus the
  `scratch` slot). Only TWO carry a certificate-coverage contract at all:

    systemverilog   UNVERIFIED / STALE     1385 rules · 1378 witness · 7 proof · 0 unknown
                                           (canonical unknown 11)
    rtl_const_expr  UNVERIFIED / UNPINNED    48 rules ·   48 witness · 0 proof · 0 unknown

  The other seven — `json`, `regex`, `return_annotation`, `rtl_frontend`, `semantic_annotation`,
  `systemverilog_preprocessor`, `vhdl` — have NO ORACLE: nothing has ever scored their rule
  reachability, so there is no certification claim to be true or false.

  SystemVerilog is STALE for a precise reason: its contract pins
  `generated/systemverilog_parser.rs = cc874b60…`; the parser in the tree is `e53cb4a2…`. The gate's
  `summary.json` is dated 2026-08-22 03:14 and the parser was regenerated 2026-08-23 12:16 — the
  proof predates its own subject by a day. The GRAMMAR did not move (`git log 767a1b37..HEAD --
  grammars/systemverilog.ebnf` = 0 commits); engine-universal codegen did.

  `rtl_const_expr` is UNPINNED: its contract carries no `identity` block, so nothing can say which
  tree produced its 48/48.
reverify: "bash scripts/report_grammar_certification.sh; bash scripts/report_grammar_certification.sh --check docs/book/src/grammar-certification-status.md"
---

## The short answer

**No PGEN grammar is currently certified-and-fresh.** Two of nine have ever been scored; both of
those scores are real but cannot be trusted at HEAD, for different reasons.

| grammar | certification | why |
|---|---|---|
| `systemverilog` | ⚠️ UNVERIFIED (STALE) | reached 0 unknown on 2026-08-22, but the contract pins a different parser than the one in the tree |
| `rtl_const_expr` | ⚠️ UNVERIFIED (UNPINNED) | 48/48 witnessed, but no identity block says which tree it scored |
| the other seven | ⛔ NO ORACLE | nothing has ever scored their rule reachability |

## Three things "certified" does not mean

1. **It is not correctness.** Certification asks *can the generator reach every rule of our
   grammar* — a claim about internal reachability, not about the language. SystemVerilog's corpus
   axis separately reads 46.3 % adjudicated with 275 known defects.
2. **It is not "reached by a generated string" for every rule.** Of SystemVerilog's 1 385 rules,
   1 378 are credited by a **witness** (a generated string really reaches them) and **7 by proof** —
   argued unreachable under their profile, with no string produced. And the 1 378 are a **union over
   four** entry/profile configurations; under the single canonical config, **11** rules are unknown.
3. **It is not durable.** A proof describes one specific generated parser. Codegen moves, the parser
   moves, and the proof keeps reading green while describing something that no longer exists.

## Why this page had to be derived

The previous answer was a **doc-asserted roster** — `CHANGES.md` records one family's membership as
*"doc-asserted only"*, i.e. a claim with no oracle behind it. It rotted exactly as you would expect:
asked directly whether SV was certified, the honest answer took a dozen commands and came back *no,
and the last yes was scored against a bar we wrote and never re-ran* — while every registered
doctrine reported PASS, because a stale baseline inside its commit budget is a NOTE, not a failure.

⇒ **a status line without its freshness is not a status.** Every row on the published page carries
both, and the page is regenerated and diffed rather than edited.

Related: [[a-corpus-generated-from-the-artifact-under-test-is-part-of-the-measurement]] ·
[[a-control-that-cannot-fail-is-not-a-control]]
