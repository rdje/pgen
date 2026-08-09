---
name: feedback-unsampled-rule-is-checked-for-parseability-only
description: DISCIPLINE (2026-08-09, session #219, SV-CORPUS-GRAD.3.18) — a grammar rule with no ast_shape_contract sample is checked for PARSEABILITY only, never for CORRECTNESS. Every routine oracle here (corpus pass count, adjudication manifest, triage gate, syntax-closure gate) asks "did it parse", so a construct that parses into the WRONG TREE reports green everywhere. Measured on SV at HEAD, 22 of 1055 annotated rules carry a sample — 2.09%. When a burn-down leaf touches an unsampled rule, adding a sample is not polish; it is the only thing that makes the fix verifiable in the direction that matters.
metadata:
  node_type: memory
  type: feedback
---

**The founding case.** `SV-CORPUS-GRAD.3.18` fixed one dropped delimiter in `expression_or_dist`:
IEEE 1800-2017 A.2.10's `expression_or_dist ::= expression [ dist { dist_list } ]` had been
transcribed with the **literal** braces read as EBNF repetition (`dist_list*`). Two of the three
resulting symptoms were the familiar kind — legal input rejected, illegal input accepted — and both
were visible to an exit code.

The third was not. `soft x dist {5, 8};` **parsed**, because `dist_item → value_range → expression`
can itself begin with `{` as a **concatenation**. So the LRM's *two* distribution items arrived at
the consumer as *one* item whose value was the concatenation expression `{5, 8}`. The parse
succeeded, the corpus counted it as a pass, the adjudication manifest classed it `match`, and the
tree was wrong. The only visible difference was one field in the AST dump:
`"kind": "concat"` where the LRM requires two separate items.

**Why no instrument caught it.** Every routine oracle in this repository answers *did it parse*:

| oracle | question it asks |
|---|---|
| `run_external_corpus.sh` pass/fail counts | did it parse |
| `adjudicate_external_corpus.py` manifest | did it parse, vs. did we expect it to |
| `sv_external_corpus_triage_gate` | did it parse |
| `sv_syntax_closure_gate` | is the grammar reachable/closed |
| **`ast_shape_contract_gate`** | **is the emitted tree the right shape** |

Only the last one asks the question that was failing, and `dist` appeared in none of its samples.

**The measurement (HEAD, 2026-08-09):**

| quantity | value |
|---|---|
| return-annotation entries, `generated/systemverilog_return_annotations.json` | 2 284 |
| distinct SV rules carrying a return annotation | 1 055 |
| distinct rules named by an `ast_shape_contract` sample | 30 (32 samples) |
| …of those, rules that carry a return annotation | **22** (the other 8 are un-annotated named lifts) |
| **shape-locked share of the annotated surface (22 / 1 055)** | **2.09 %** |

⛔ **This is not the claim that 97.9 % of the contract is missing.** The contract was designed as a
*regression lock for known corruption sites* (`SV-0014`…`SV-0020`), and 30 targeted samples are the
right shape for that job. The narrower, worse finding is that **nothing else in the repository asks
whether the emitted tree is right**, for any of the other 1 033 rules — and `dist` is the existence
proof that the gap is not theoretical. Tracked as `SV-AST-SHAPE-FIDELITY.4`.

**How to apply.**

1. **Touching an unsampled rule?** Add a sample in the same leaf. `.3.18` did this by hand; it costs
   one manifest entry plus one dispatch arm, and it is the only artifact that would catch a
   recurrence.
2. **Never accept a green exit code as proof a construct is correct.** For any fix whose defect
   *could* have a wrong-tree arm, dump the AST before and after and diff the discriminating field.
   A pass/fail probe cannot distinguish "right" from "parsed".
3. **Prove the new lock is live.** Break it deliberately, watch the gate go red, then restore.
   `.3.18` did exactly this (`missing required key 'THIS_KEY_DOES_NOT_EXIST'`). A green gate never
   shown to go red is a claim, not a proof — see [[feedback_instrument_needs_ground_truth]].
4. **Beware the shape-checking instrument that agrees with you.** `.3.18`'s own checker first used a
   depth-first search for the discriminating `"kind"` and returned the LRM-correct answer *on the
   corrupted tree*, because the wrapper node holding `"kind": "concat"` also holds the operand list
   whose members each carry `"kind": "number"`. It was rewritten breadth-first so the shallowest
   match wins.

Related: [[feedback_correctness_before_speed]] (accuracy is the immovable floor),
[[a-rising-pass-rate-is-not-evidence-of-correctness]] (the pass count is the least trustworthy
signal), [[feedback_sv_strict_lrm_compliance_default]] (over-acceptance is a defect too).
