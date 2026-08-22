---
id: an-instrument-can-be-green-in-both-arms-of-the-defect-it-describes
title: If your contract's assertion vocabulary cannot express the property, its verdict is invariant under the defect — run the fix and check the gate MOVED, not that it passed
answers:
  - "my shape contract is green but the AST is wrong — how is that possible"
  - "how do I tell whether a passing gate is actually checking the thing it names"
  - "a gate passed before and after my fix, is that reassuring or alarming"
  - "how do I test a contract rather than trust it"
  - "what does a key-presence assertion actually prove"
  - "why did a whole payload go missing with every gate green"
  - "does a positional $N count whitespace regexes as elements"
  - "my return annotation resolves to the empty string and the rule still parses"
  - "one field of my annotation is right and its neighbour is empty"
  - "a two-arm control produced no output for one arm, is that agreement"
tags: [instruments, contracts, gates, evidence, controls, return-annotations, typed-ast, claim-verification]
date: 2026-08-23
status: current
evidence: |
  GRAMMAR-WELLFORMED.H.16.7 (`PGEN-GRAMMAR-WELLFORMED-0172`), 2026-08-23.
  `grammars/semantic_annotation.ebnf:33` declared `-> {type: "semantic_annotation", name: $3,
  value: $6}`. A positional `$N` counts EVERY top-level element of the rule body, the `/\s*/` layout
  regexes included — `1="@" 2=/\s*/ 3=annotation_name 4=/\s*/ 5=":" 6=/\s*/ 7=annotation_value` — so
  `$6` named the third SEPARATOR, which always matches the empty string. Measured on the SHIPPED
  release parser, 12 of 12 annotation value shapes published `value: ""`: the entire payload
  discarded at the grammar's declared entry rule.
  `rust/test_data/ast_shape_contract/semantic_annotation_v1.json` exists to assert that the runtime
  AST matches the declared shape. It asserts `expected_json_object_keys_present: [type, name, value]`
  and pins only `type`. `value: ""` satisfies key-presence; `value` cannot be pinned, because
  `rust/src/ast_shape_contract.rs:673`/`:679` implement exactly two assertion kinds — key-PRESENT and
  exact-STRING — and after the fix `value` is an OBJECT. The gate therefore reads 18/18 PASS
  identically before and after a two-character change that restored the whole payload. Routed as
  GRAMMAR-WELLFORMED.H.16.7b. The lint was also clean and certificate coverage also did not move
  (correctly — a return annotation moves AST SHAPING, not acceptance).
reverify: "make -C rust SHELL=/bin/bash ast_shape_contract_gate   # 18/18 PASS. Now break it on purpose: set `value: $6` back in grammars/semantic_annotation.ebnf:33 and :37, `make -C rust SHELL=/bin/bash semantic_annotation_parser`, re-run the gate — still 18/18 PASS with every annotation's payload gone. `git checkout grammars/semantic_annotation.ebnf && make -C rust SHELL=/bin/bash semantic_annotation_parser` to restore."
---

**A gate that passes tells you one of two things, and it does not tell you which:** that the property
holds, or that the gate cannot express the property. Those are indistinguishable from the outside, and
the second one is silent forever.

A contract's real coverage is bounded by its **assertion vocabulary**, not by its intent. The
AST-shape contract here is named, wired, run in CI, and green — and it can say exactly two things
about a key: *it exists*, and *it equals this exact string*. Neither can express *"this key carries a
payload"*. So an entry rule that published `value: ""` for **every annotation it ever parsed** sat
inside a green contract whose whole purpose was to assert the declared shape.

## The test that distinguishes the two cases

Do not ask *did the gate pass?* Ask **did the gate MOVE?**

> Land the fix. Re-run the gate. If its verdict is **identical** before and after a change that
> demonstrably fixed the thing the gate is named for, the gate is not measuring it.

That is a one-command check and it is the only one that separates "property holds" from "property
inexpressible." Here it reads 18/18 in both arms — so the green was never evidence, and saying so in
the manifest's own `doctrine` field is worth more than the fix.

It generalises past contracts: any assertion whose vocabulary is weaker than the property is a gate
that fails open, permanently and quietly. Presence checks are the classic case — `has_key`,
`is_not_null`, `len() > 0`, `exit code == 0`. Each is satisfied by a degenerate value.

## The mechanism worth knowing on its own: `$N` counts the separators

```ebnf
semantic_annotation := "@" /\s*/ annotation_name /\s*/ ":" /\s*/ annotation_value
#                      $1    $2       $3          $4   $5    $6         $7
    -> {type: "semantic_annotation", name: $3, value: $6}   # $3 ✅   $6 ✗ — the third /\s*/
```

A positional reference indexes **every top-level element**, layout regexes included, so off-by-one is
the default failure rather than an exotic one. It resolves, it is in range, it always matches empty,
and nothing anywhere reports it: the grammar lints clean, the rule is witnessed, the parse ACCEPTS,
and the certificate tuple does not move.

⭐ **The tell was inside the same annotation.** `name: $3` worked while `value: $6` was empty — one
field right and its neighbour empty in one rule is an index error and almost nothing else. When a
shaping bug hits exactly one field, suspect the index before the shaping.

Related but distinct: an **out-of-range** `$N` publishes the literal string `<invalid_sequence_access>`
into the AST instead (a separately tracked class — see `SV-AST-SHAPE-FIDELITY`, `POST-SV-AUDIT`,
`INLINE-ALT-FIX`). The separator-valued variant leaves nothing at all, which is why it is harder to see.

## And a control in the same slice printed nothing, which reads as agreement

The pre-fix/post-fix certificate control's first arm emitted **zero** lines while the second emitted
three. Grepped for a difference, that is "no divergence". It was an error:
`--report-certificate-coverage` resolves the grammar NAME from the FILENAME, so a copy at a scratch
path is an unregistered grammar and the tool refuses. ⭐⭐ **Empty output and matching output are the
same shape at a glance and opposite in meaning** — assert that both arms produced output before
comparing them, and add a RED arm (here: the same grammar plus one rule ⇒ `116/0/84/32` instead of
`115/0/84/31`) so the control is *shown* able to move.

See also [[a-control-that-cannot-fail-is-not-a-control]] (the arm never read your change) and
[[a-control-that-clears-your-hypothesis-has-not-cleared-the-symptom]] (the control was sound and the
conclusion overran it). This card is the third member of that family: **the control ran, the arms were
real, and the instrument had nothing to say.**
