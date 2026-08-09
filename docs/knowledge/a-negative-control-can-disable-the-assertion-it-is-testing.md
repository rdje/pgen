---
id: a-negative-control-can-disable-the-assertion-it-is-testing
title: A negative control that stays GREEN has not proven the gate is dead — it may have switched off the very assertion it was meant to trip; break the field the gate actually reads
answers:
  - "I broke my new shape sample on purpose and the gate still passed — is the gate vacuous"
  - "which field does the ast_shape_contract gate actually assert on"
  - "why are my expected_json_object_keys_present checks never running"
  - "how do I prove a newly added gate sample is live rather than decorative"
  - "what does drift_status aligned actually control in the shape contract"
  - "is a passing negative control evidence of anything"
tags: [gates, instrument-honesty, ast-shape-contract, negative-control, proof-discipline]
date: 2026-08-09
status: current
evidence: rust/src/ast_shape_contract.rs (the `let aligned = sample.current_content_kind == sample.expected_content_kind` guard, and the `if aligned` block that owns every structural assertion); docs/tasks/artifacts/sv_corpus_grad/config_use_param_override/ast_shape_negative_control.txt (both attempts, with the gate output for each); docs/tasks/SV-CORPUS-GRAD.md leaf .3.19
reverify: "grep -q 'let aligned = sample.current_content_kind == sample.expected_content_kind' rust/src/ast_shape_contract.rs && echo ALIGNED-GUARD-STILL-OWNS-THE-ASSERTIONS"
---

**A control that passes is not evidence the thing works. It is evidence you have not yet found the
assertion.**

The `ast_shape_contract` gate classifies every sample by

```rust
let aligned = sample.current_content_kind == sample.expected_content_kind;
if aligned { /* ← every structural assertion lives in here */ }
```

so a sample whose `expected_` and `current_` kinds **differ** is treated as *drifted*, and its
structural checks — including `expected_json_object_keys_present` — are **skipped entirely**.

That makes the intuitive negative control useless:

| attempt | what was broken | gate result | why |
|---|---|---|---|
| 1 | `expected_content_kind`, plus a demand for a key that does not exist | **GREEN** ❌ | the edit made the sample drifted, so the assertion it was testing never ran |
| 2 | `current_content_kind` (expected moved with it, keeping the sample aligned) | **FAILED, naming the sample and rule** ✅ | the regression lock compares the *observed* kind against `current_content_kind` |

**The rule to carry forward:** the live lock is `current_content_kind`. `expected_content_kind` is a
*classifier* — it decides whether a sample is asserted at all — not an assertion. To prove a sample
is live, break the field the gate reads, not the field that describes intent.

**The general shape of the trap**, worth checking whenever a gate has a status/enabled/expected
field alongside its data: **if your tampering can move a sample out of the checked population, a
green result means "not checked", not "checked and fine".** Prefer a control that keeps the item
inside the population and corrupts the value under test.

**Honest bound worth knowing at the same time:** for a sample whose content kind is not
`json_object` — e.g. an un-annotated alternative, which classifies as `sequence` — the regression
lock is the *only* live assertion. Such a sample pins the **kind** of the emitted node, not its
interior. That is the strongest lock available for an un-annotated rule, and it is materially weaker
than a `json_object` sample; say so rather than implying parity.

See also [[a-rule-with-no-shape-sample-is-checked-for-parseability-only]],
[[a-rising-pass-rate-is-not-evidence-of-correctness]].
