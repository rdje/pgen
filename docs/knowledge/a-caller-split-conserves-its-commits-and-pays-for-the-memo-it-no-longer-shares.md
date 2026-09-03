---
id: a-caller-split-conserves-its-commits-and-pays-for-the-memo-it-no-longer-shares
title: A caller split conserves its commits and pays for the memo it no longer shares — widen one caller with a twin rule, read the twins' memo_hits before blaming the operand, and give the rise class its own conservation law
answers:
  - "one caller of a grammar rule needs a wider operand but the other callers must stay narrow — how do I widen it without over-accepting elsewhere"
  - "why does clist[b[0]++].m reject while clist[b[0]++] = null and clist[b[0]+1].m parse (SV-CORPUS-GRAD.3.31)"
  - "the parse-cost ratchet rose after a grammar split although committed did not move — where did the entries go"
  - "why does splitting a rule into two twins cost memo hits — the memo is keyed by rule id, not by what the rule derives"
  - "neither contained_in_introduced_subgraph nor profile_split_respelling admits my rise — what invariant fits a caller split"
  - "the LR-family share changed although the family's entries did not — why must the share be re-derived first"
  - "sed -i '' did nothing on macOS with GNU sed — how do I verify an in-place edit"
tags: [parse-cost, ratchet, grammar-authoring, memoization, systemverilog, sv-corpus, invariants, containment]
date: 2026-09-03
status: current
evidence: "SV-CORPUS-GRAD.3.31 (PGEN-SV-CORPUS-GRAD-0357, release 1.0.208, ledger SV-0081): hierarchical_bit_select split from constant_bit_select at ten sites; entries +1,344,489 with committed conserved exactly (constant_bit_select −881,589 / −38,027 ↔ hierarchical_bit_select +881,591 / +38,027); the twins' memo_hits net −298,210 replays = the loop-body risers lbrack +357,103, trivia +120,109, rbrack +65,125; accepted under the new caller_split_containment invariant (8 self-test arms); ENGINE-UNIVERSAL-SERVICES.92 prices the elimination"
reverify: "python3 scripts/parse_cost_containment.py --self-test | tail -1 && bash scripts/check_parse_cost_ratchet.sh | tail -1   # CALLER-SPLIT-CONTAINMENT-SELFTEST: arms=8 failed=0, then parse-cost-ratchet: OK"
---

**When one caller of a rule needs a wider operand and the others must stay narrow, the fix is a twin
rule at that caller — and the twin's cost is not its operand, it is the memo the two twins no longer
share. Price a split by the twins' `memo_hits`, and give the rise class its own conservation law.**

## The rules

- **Widen the caller, not the rule.** `clist[b[0]++].m` rejected because the index of an element whose
  member is then selected was the one index path spelled through `constant_bit_select`, whose
  `constant_expression` lacks the two A.8.3 `expression` operands `inc_or_dec_expression` and
  `( operator_assignment )` (IEEE 1800-2017 `section-41-data-read-api.md:3026` vs `:2982`). Widening
  `constant_bit_select` itself would have admitted `++` in a `constant_select` and a `bind` target —
  over-acceptance. A twin (`hierarchical_bit_select`) at the ten hierarchical sites widens exactly the
  path that was inconsistent and leaves the constant contexts as the letter has them.
- **Refuse the cheaper spellings by their mechanism before measuring them.** An ordered choice
  `narrow | wide` commits on the `b[0]` prefix under commit-once and never sees the `++`; a lookahead
  cannot see a postfix `++` without parsing the lvalue. Both are refused in a sentence, not a build.
- **A split moves its work one for one; read the twins' memo before blaming the operand.** The
  ratchet's per-rule table showed `constant_bit_select` −881,589 entries / −38,027 committed against
  `hierarchical_bit_select` +881,591 / +38,027 — conserved to within two entries. The rise was the memo:
  the memo is keyed by `(rule id, position)`, so callers that used to replay one cell now execute two
  rules; the twins' `memo_hits` net −298,210 replays, and those executions re-ran the loop body
  (`lbrack` +357,103, `trivia` +120,109, `rbrack` +65,125). The operand tournament is the rest and is
  `ENGINE-UNIVERSAL-SERVICES.90`'s mechanism (the ordered choice enters every alternative).
- **A rise class needs its own conservation law.** `contained_in_introduced_subgraph` refuses any
  faller and a split ALWAYS has one (the narrow twin); `profile_split_respelling` refuses any
  pre-existing riser and a split always has those (the loop's terminals). Widening either would admit
  what it was written to refuse. `caller_split_containment` states what a split IS — total `committed`
  moved by 0, no pre-existing rule's `committed` rose, the `committed` lost by pre-existing rules equals
  the `committed` gained by the introduced rules, every entries-riser inside the sub-graph reachable
  from the introduced rules — and eight self-test arms each name a shape it must refuse.
- **A family SHARE moves when the DENOMINATOR moves.** The LR family's entries fell by 1 279 while the
  corpus total rose by 2 M: `2.750 → 2.744 %`. The share is re-derived FIRST and its published anchors
  are the total's currency, not the family's — the gate names every anchor that must move.
- **Verify an in-place edit by re-counting the text, never by the exit status.** The BSD spelling
  `sed -i '' 's/a/b/' f` on a GNU sed takes `''` as the script and `s/a/b/` as a file name: it edits
  nothing and the loop it sat in exited 0. The `grep -c` after the edit is what caught it.

## The measurement

`entries` 412,979,398 → 414,323,887 (+0.33 %), `memo_hits` 184,638,132 → 185,332,574 (+0.38 %),
`committed` 6,759,873 exactly unchanged; the axis-2 bar `162 → 161` by exactly the predicted corpus row,
`pass → fail` 0. See `docs/tasks/SV-CORPUS-GRAD.md` `.3.31`, the ROUTING EVIDENCE `.3.31 → .92`, and
`docs/tasks/ENGINE-UNIVERSAL-SERVICES.md` `.92`.
