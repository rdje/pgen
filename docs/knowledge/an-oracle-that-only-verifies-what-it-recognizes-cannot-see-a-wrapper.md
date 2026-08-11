---
id: an-oracle-that-only-verifies-what-it-recognizes-cannot-see-a-wrapper
title: An oracle that only verifies what it RECOGNIZES cannot see an engine that WRAPS the thing it recognizes — skipping the unrecognized is a silent pass, and finding the declared value nested inside the wrapper also marks it COVERED
answers:
  - "my shape gate is green but the AST is wrong"
  - "why did the annotation-vs-AST gate not catch a wrong emitted shape"
  - "is a discriminator-driven AST checker sound"
  - "why is coverage green when the declared shape is never actually produced"
  - "how do I gate that a generated parser returns the AST the grammar declared"
  - "what does the reserved _pgen_ type prefix mean in an emitted AST"
  - "how do I test that an engine is not leaking an internal representation"
tags: [instrument-soundness, oracle-design, return-annotations, ast-shape, negative-space, engine-boundary]
date: 2026-08-11
status: current
evidence: docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .8; rust/src/auto_return_annotation_shape_gate.rs `walk_and_verify_against_discriminator_map` + test `inventory_wide_gate_fails_on_an_engine_internal_type_discriminator`; rust/src/ast_pipeline/lr_chain_fold.rs `ENGINE_INTERNAL_TYPE_PREFIX`; rust/src/parse_harness_combinator_suite.rs case `left_recursion_folded_ast`
reverify: "cd rust && cargo test --features generated_parsers --lib auto_return_annotation_shape_gate::tests::inventory_wide_gate_fails_on_an_engine_internal_type_discriminator -- --nocapture"
---

PGEN's annotation-shape gate walks a parser's typed AST, finds every object whose `type:` field
matches a **declared** discriminator from the grammar's return-annotation inventory, and verifies it
against that annotation's shape. It is a good instrument. It was also, for the whole life of the
left-recursion eliminator, **structurally unable to see** that every LR-eliminated rule in every
grammar was publishing the eliminator's internal record instead of the author's shape.

## The two blindnesses, and why they compound

An LR-eliminated rule emitted this for the input `$1.a.b` — the *declared* shape is
`{type: "property_access", base: $1, property: $3}`:

```jsonc
{ "initial":  {"type": "property_access", "base": {…}, "property": "a"},
  "suffixes": [{"alt_index": 0, "captures": [".", "b"], "type": "_pgen_lr_chain_alt"}],
  "type": "_pgen_lr_chain",
  "wrapper_specs": "<439 bytes of serialized templates>" }
```

1. **Verification skipped it.** `_pgen_lr_chain` is not a declared discriminator — synthetic
   annotations are deliberately excluded from the inventory — so the walker had no descriptor to
   check it against and moved on. *An unrecognized value produced no verdict, and no verdict reads
   exactly like a pass.*
2. **Coverage was satisfied by the wrapper's own contents.** The walker recurses into every nested
   value, so it found the real `{type: "property_access", …}` object sitting inside the record's
   `initial` field, verified *that*, and marked `property_access` **covered**. The
   "declared-but-never-produced" report — the one leg that could plausibly have hinted at the
   problem — stayed empty.

⇒ Both legs green, on an AST that was not the declared one and had silently dropped the `.b`
continuation entirely.

## The general rule

> A checker whose domain is *"values I recognize"* is blind to any transformation that **preserves
> the recognized value and changes its position**. Wrapping is exactly such a transformation.

This is a sibling of *two implementations agreeing is not evidence either one is right*: PGEN's
differential gates compared the interpreter against the generated parser byte-for-byte and were
green throughout, because both folded — i.e. both leaked — identically.

## The fix: assert the NEGATIVE SPACE

Positive checks ("this value matches its declaration") cannot bound what else may appear. Add a
complementary assertion over what may **not**:

- **Reserve a namespace for the engine.** Every engine-synthesized discriminator carries the
  `_pgen_` prefix (`lr_chain_fold::ENGINE_INTERNAL_TYPE_PREFIX`). No grammar author can declare one.
- **Fail on its presence anywhere in a published AST**, rather than skipping it. A generated parser
  must return the AST its grammar declared; an engine intermediate reaching a consumer is a contract
  break regardless of which engine feature produced it — so this catches the *next* one too, not
  just left recursion.
- **Prove the check fires.** The gate's RED probe feeds the exact pre-fix value and asserts the
  failure names the leak; the same test asserts `discriminators_not_covered()` is still **empty** on
  that value, so the record itself states why the coverage leg could never have caught it.

## Designing the positive case so it can discriminate

A test case only measures what its inputs can distinguish. The combinator suite already had a left
recursion case — and it was **annotation-free**, so its typed AST was purely structural and the
record never appeared in it. The replacement declares object annotations on **two distinct
operators**, so a fold that ignores `alt_index` or cross-wires two templates cannot pass, and the
gate asserts the **exact** left-nested value against the declaration rather than against a second
implementation of the same engine.

Related: [[a-negative-control-can-disable-the-assertion-it-is-testing]],
[[a-check-whose-inputs-all-pass-has-not-been-tested]],
[[a-catch-all-alternative-makes-an-accept-meaningless]],
[[a-directly-left-recursive-alternative-inside-a-choice-is-dead-code]],
[[a-gate-must-be-able-to-fail-and-able-to-run]].
