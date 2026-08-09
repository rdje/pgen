---
id: a-rule-with-no-shape-sample-is-checked-for-parseability-only
title: A construct can be broken in a direction no pass-rate can see — a rule with no ast_shape_contract sample is checked for PARSEABILITY only, never for correctness
answers:
  - "the file parses and the corpus is green — is the AST actually right"
  - "how can a grammar bug survive when every gate is passing"
  - "my grammar fix made input parse but did it parse into the right tree"
  - "how do I check a parser produced the correct AST shape and not just a successful parse"
  - "which SystemVerilog rules are shape-locked and which are only checked for parseability"
  - "how do I add an ast_shape_contract sample for a rule"
  - "how do I prove a newly added gate assertion actually fails when it should"
  - "I transcribed an LRM production and the construct parses — how do I know I read the braces right"
  - "a construct parses but the consumer gets the wrong number of list items — where do I look"
  - "why did a mis-parse survive the corpus, the adjudication manifest and the triage gate"
  - "when is it mandatory to add a shape sample during a burn-down leaf"
  - "my shape-checking script agrees with my fix — how do I know the script is right"
tags: [ast-shape, correctness, verification, instrument-honesty, systemverilog, lrm, burn-down, gates]
date: 2026-08-09
status: current
evidence: docs/tasks/SV-CORPUS-GRAD.md leaf .3.18 (the `dist` defect and its three arms); grammars/systemverilog.ebnf `expression_or_dist` (IEEE 1800-2017 A.2.10, literal braces restored); rust/test_data/ast_shape_contract/systemverilog_v1.json sample `expression_or_dist_braced_list` + its dispatch arm in rust/src/ast_shape_contract.rs; docs/decisions/feedback_unsampled_rule_is_checked_for_parseability_only.md; docs/tasks/SV-AST-SHAPE-FIDELITY.md leaf .4 (the 22/1055 measurement)
reverify: "python3 -c \"import json; inv=json.load(open('generated/systemverilog_return_annotations.json')); con=json.load(open('rust/test_data/ast_shape_contract/systemverilog_v1.json')); ann={a['rule'] for a in inv['annotations']}; s={x['rule_under_test'] for x in con['samples']}; print('shape-locked %d/%d annotated rules'%(len(s&ann),len(ann))); assert 'expression_or_dist' in s, 'the dist shape lock is GONE'\""
---

**The defect that motivates this card parsed cleanly.** `SV-CORPUS-GRAD.3.18` found that
`expression_or_dist` had transcribed IEEE 1800-2017 A.2.10 —
`expression_or_dist ::= expression [ dist { dist_list } ]` — with the **literal** SystemVerilog
braces read as EBNF repetition (`dist_list*`). Two of the three resulting arms were the familiar
kind, visible to an exit code: legal input rejected (`x dist {100 := 1, 200 := 2}`), illegal input
accepted (the brace-less `x dist 100 := 1;`).

The third arm was invisible. `soft x dist {5, 8};` **parsed**, because
`dist_item → value_range → expression` can itself begin with `{` as a **concatenation**. The LRM's
*two* distribution items reached the consumer as *one* item whose value was the concatenation
expression `{5, 8}`. Exit code 0. Corpus: pass. Adjudication manifest: `match`. The only trace was
one field in the AST dump — `"kind": "concat"` where two separate items belong.

## Rank your oracles by what they cannot see

| oracle | the question it actually asks |
|---|---|
| `run_external_corpus.sh` pass/fail counts | did it parse |
| `adjudicate_external_corpus.py` manifest | did it parse, vs. did we expect it to |
| `sv_external_corpus_triage_gate` | did it parse |
| `sv_syntax_closure_gate` | is the grammar reachable / closed |
| **`ast_shape_contract_gate`** | **is the emitted tree the right shape** |

Only the last asks the failing question — and `dist` appeared in none of its samples. Measured at
HEAD on 2026-08-09: **22 of 1 055 annotated SystemVerilog rules carry a sample — 2.09 %** (30 distinct rules are named by the 32 samples; 8 of those are un-annotated named lifts).

⛔ That is **not** a claim that 97.9 % of the contract is missing. The contract was built as a
regression lock for known corruption sites, and targeted samples are right for that job. The
narrower, worse reading is the true one: **for the other ~1 033 rules nothing in the repository asks
whether the tree is right** — only whether it parses.

## What to do

1. **Touching a rule with no sample? Add one in the same leaf.** It costs one manifest entry plus
   one dispatch arm in `rust/src/ast_shape_contract.rs`, and it is the only artifact that catches a
   recurrence.
2. **Diff the shape, not the exit code.** For any fix whose defect could have a wrong-tree arm, dump
   the AST before and after and compare the discriminating field. `parse_full passed` cannot
   distinguish *right* from *parsed*.
3. **Prove the new lock is live.** Break the expected keys on purpose, watch the gate go red
   (`missing required key '…'`), then restore. A green gate never shown to go red is a claim.
4. **Beware the shape checker that agrees with you.** The first version of `.3.18`'s checker used a
   depth-first search for the discriminating `"kind"` and returned the LRM-*correct* answer on the
   *corrupted* tree: the wrapper node carrying `"kind": "concat"` also holds the operand list whose
   members each carry `"kind": "number"`, and depth-first descends into the list first. Rewritten
   breadth-first so the shallowest match wins. An instrument that confirms whatever you hoped is
   worse than no instrument.
5. **When the LRM's metasyntax and its terminals share characters, the prose settles it.** A.2.10
   uses `{ }` as literal braces on one line and as repetition two lines later; the normative
   examples in §18.5.4 (`x dist {100 := 1, 200 := 2, 300 := 5}`) are what make the reading
   adjudicable rather than a judgement call.

Related: [[a-rising-pass-rate-is-not-evidence-of-correctness]] (the pass count is the least
trustworthy signal), [[a-copied-diagnostic-covers-only-where-it-was-pasted]] (generalize the
instrument, do not fork it).
