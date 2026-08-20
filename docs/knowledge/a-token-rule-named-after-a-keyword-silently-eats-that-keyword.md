---
id: a-token-rule-named-after-a-keyword-silently-eats-that-keyword
title: A token rule named after a keyword will silently swallow that keyword when a grammar is imported — and no coverage instrument can report it, because the keyword never becomes a rule
answers:
  - "my grammar rejects a keyword the standard defines — where do I look first"
  - "why would a parser accept an operator the standard does not define in that position"
  - "how can a defect exist that certificate coverage and corpus coverage both miss"
  - "I am importing a standard's BNF into an existing grammar — what collides"
  - "is a rule named after a keyword safe if it predates the import"
  - "what should I check before minting a keyword token that did not exist"
tags: [grammar-fidelity, extraction, tokens, keywords, coverage, oracles, diagnosis]
date: 2026-08-20
status: current
evidence: "SV-CORPUS-GRAD.13c.2y (`PGEN-SV-CORPUS-GRAD-0261`, ledger `SV-0066`). `grammars/systemverilog.ebnf` defined `implies := trivia \"->\"` — the ARROW — years before IEEE 1800 A.2.10's property operator `implies` was extracted. Extraction resolved the keyword to the existing rule, so `a implies b` was REJECTED at every release while `-> b` and `a |= b` were ACCEPTED in property position. `grep -c kw_implies grammars/systemverilog.ebnf` returned 0 at every release. Fixed by minting `kw_implies_470cec58` and moving the operator to its Table 16-3 precedence group; union certificate `UNKNOWN` fell 53 -> 1 and the generated parser shed 8.41 MB, because the mis-encoded alternative was also the only left-recursive one in its rule."
reverify: "printf 'kw_implies present: %s\\nthe rule named implies is: %s\\n' \"$(grep -c kw_implies grammars/systemverilog.ebnf)\" \"$(grep -m1 '^implies :=' grammars/systemverilog.ebnf)\""
---

**When a grammar mints operator tokens with human-readable names, it creates a namespace that a
later import can collide with — silently.** `implies`, `assign`, `star`, `not`, `or` are all
plausible names for punctuation rules *and* real keywords in some standard. The moment a BNF import
resolves an identifier, an existing rule of that name wins, and nothing warns: the reference
resolves, the grammar lints clean, the parser builds.

The failure is two-sided and both sides are quiet:

| | what happens | why nothing sees it |
|---|---|---|
| **over-rejection** | the standard's keyword is unparseable | the keyword never becomes a rule, so it has no row in any coverage report |
| **over-acceptance** | the punctuation is accepted where the standard puts the keyword | a rule that *is* covered, exercised through the wrong terminal |

⛔ **The over-rejection half is invisible to every coverage instrument by construction.** Certificate
coverage and corpus rule-coverage both enumerate rows from the *declared* rule set. A production the
grammar never spells produces no row — not `UNKNOWN`, not `GAP`, nothing. See
[[a-coverage-report-cannot-see-a-production-the-grammar-never-declares]].

**What finds it.** Only an oracle outside the grammar: the standard's own text, an external corpus,
or a hand-written probe for each operator the standard lists. In the SystemVerilog case the pointer
was indirect — nine keyword tokens in one rule could not be witnessed, which said *read that rule*,
and reading it found a tenth operator that was not a token at all.

⭐ **Before minting the keyword, check the standard's reserved-word list.** If it is reserved, a
`/keyword\b/` terminal cannot capture a legal identifier. If it is **not** reserved, the same edit
introduces a new defect — a rule matching it bare will eat a user's identifier. Both cases have been
measured in this repository one release apart: `implies` is reserved (IEEE 1800-2023 Annex B) and was
safe; `randomize` is not, and placing its rule mid-list captured legal identifiers until the
alternative was moved last.

⭐⭐ **A mis-encoded alternative can also be carrying engine cost.** Here the bogus
`property_expr implies property_expr` was the only left-recursive alternative in its rule, so it had
been driving a whole left-recursion-elimination knot: removing it deleted 96 synthesised rules and
8.41 MB of generated parser. Fidelity and cost are not always a trade.
