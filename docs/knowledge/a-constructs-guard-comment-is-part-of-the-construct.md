---
id: a-constructs-guard-comment-is-part-of-the-construct
title: A construct's guard comment is part of the construct — relocate one without the other and you silently revert the ruling that saved it
answers:
  - "I am moving a grammar rule or alternative — what besides reachability do I have to check"
  - "why did a deliberately kept grammar arm get deleted again"
  - "is verifying reachability enough when relocating a grammar alternative"
  - "where should the comment defending a non-obvious grammar arm live"
  - "can a lint or gate catch a stranded grammar comment"
  - "how do I stop a future session from deleting an arm on purpose-built reasoning"
tags: [grammar-authoring, lrm-fidelity, refactoring, comments, verification, systemverilog]
date: 2026-08-10
status: current
evidence: docs/tasks/SV-CORPUS-GRAD.md leaf .3.26e (the two defects, the byte-identity proof, the refused heuristic, the 17-grammar routing census); grammars/systemverilog.ebnf rules assignment_pattern (the guard, now above the arm it defends) and empty_unpacked_array_concatenation (the repaired header); the precedent chain .3.26a (deletion) -> .3.26b (director ruling + guard written) -> .3.26c (arm moved, guard stranded)
reverify: "grep -c 'DO NOT DELETE' grammars/systemverilog.ebnf && ./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --lint-grammar >/dev/null && echo 'lint exit 0 on BOTH sides — which is the point'"
---

When you move a rule or an alternative, you check that nothing loses reach. That check is
mechanical, it is the one everybody remembers, and it is not sufficient. **The comment explaining
why a non-obvious arm exists is not reachable from any use site**, so nothing prompts you to move it
— and it is frequently the only thing standing between that arm and the next well-intentioned
deletion.

## The worked instance

`grammars/systemverilog.ebnf` carries an alternative for the empty assignment pattern `'{ }`. Annex A
of IEEE 1800-2017 does not derive it — all four `assignment_pattern` productions require at least one
element — so it *looks* like over-acceptance to anyone applying a strict-LRM policy. It is not: the
standard writes the construct's own delimiter pair as `'{ }` in §11.4.12 and names the operator
`vpiAssignmentPatternOp` in Annex M, and no text forbids it (see
[[a-mis-cited-production-reproduces-as-a-success]]).

The sequence:

1. **`.3.26a` deleted the arm** on exactly that inference. A director ruling reverted it the same day
   — deleting it makes the parser reject `return '{};`, which uvm-core writes and 49 corpus files use.
2. **`.3.26b` wrote the guard**: the citations, the deletion history, and *"⛔ do not 'fix' it away"*,
   in the comment above the rule that then held the arm.
3. **`.3.26c` moved the arm** to a better-modelled home — correctly, and after verifying every use
   site — **and left the guard behind.**

The result at HEAD was an alternative with no citation, no rationale and no stop-sign, while the
paragraph defending it sat on a *different* rule, under a comment that correctly told you the arm was
no longer there. That is not a hypothetical exposure; it is the precise state that produced step 1,
reconstituted at a new address.

## Why no gate catches it

`--lint-grammar` returns exit 0 both before and after — **necessarily**, because the deletion the
comment guards against also lints clean, generates clean and parses clean. Only a corpus run notices,
and only if someone runs one. Reachability is checkable and was checked. *Defensibility* is a property
of prose, and prose has no edges for a graph to walk.

> **The rule: when a construct moves, the comment that defends it is part of the move.** Only a reader
> can verify that, so make it a step in the move rather than hoping a gate objects.

## Practical form

- **Put the guard where the arm is, not where the arm was.** If the same rationale is genuinely needed
  in two places, the second copy should say *"the arm lives at X"* and nothing else — one authority.
- **A guard is worth writing when the arm's own text argues against it** — anything a strict reading
  of the source standard would flag. It needs: the citation that makes it legal, one line of *why it
  is here*, and an explicit ⛔ naming the deletion that already happened.
- **Repair the neighbours in the same edit.** The same relocation left the old rule's comment header
  saying *"TWO ARMS … do not collapse them"* ten lines above *"this rule is now pure A.8.1, the
  bare-brace form only"*, over a one-arm rule. A block that contradicts itself teaches the reader to
  trust none of it.
- ⛔ **Comments stay OUTSIDE the rule body** in this grammar: a column-0 comment between alternatives
  deletes the alternatives after it. Verify with `--dump-gen-ast` after inserting one — check the
  alternative count and the branch-annotation count, not just that it lints.

## The bound, stated rather than mechanized

A check is imaginable — flag an alternative carrying an `@sample` that no nearby comment mentions —
and it was **deliberately refused**: it is a heuristic over prose, its failure direction is open, and
this project's standing lesson is that no cut heuristic is a census. The honest place for it is the
Annex-A-vs-shipped-grammar enumeration (`LRM-GRAMMAR-FIDELITY.1c`), because *"what does the grammar
accept that the annex cannot derive"* and *"which arms need a defending comment"* are **the same
list, read twice**.

⚠️ The class is not language-specific. Protective comments across the 17 tracked grammars:
`systemverilog` 14, `vhdl` 5, `ebnf` 3, `semantic_annotation` 1, the rest zero — so any grammar with
a deliberately-kept arm can strand its guard the same way. (And the honest bound on that number: it
counts comments that *look* protective, not arms that *need* protection.)
