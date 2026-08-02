---
id: witness-entry-policy-store-gated-targets
title: A store-gated target cannot be witnessed from its own rule — the witness-ENTRY policy, not the budget
answers:
  - "why does a coverage target report never_hit at every depth and every seed"
  - "what does STORE-AWARE-GEN fact_count_at_least predicate unsatisfiable (zero source facts) mean in a gap report"
  - "why can't a semantic prelude fix a target whose gate needs a fact from above it"
  - "what is a store-entry-blocked witness target"
  - "what does store_entry_raises= on the witness pass summary line count"
  - "why do the two witness passes root their generation at different entry rules"
  - "why must a store-gate blocking verdict count only POSITIVE gates"
  - "why is a mandatory-descent walk not the same as does a gate exist below here"
  - "how did the SystemVerilog closed-loop class-C residual close"
tags: [stimuli, coverage, semantic-store, witness, systemverilog, root-cause]
date: 2026-08-02
status: current
evidence: rust/src/ast_pipeline/stimuli_generator.rs (`witness_target_is_store_entry_blocked`, `target_forces_positive_store_gate`, `emittable_kinds_within_closure`, `StoreGateScope`, and the raised-entry arm inside `generate_target_witnesses`); docs/tasks/SV-EXH-PROOF.md leaf .7.4.6.12 (WHY+WHERE_CLASS_C_2026-08-01, SIZING_CORRECTION_2026-08-01, FIX_LANDED_2026-08-02); docs/book/src/grammar-wellformedness.md "The closed-loop witness pass's raised entry for store-gated targets"
reverify: "python3 docs/tasks/artifacts/sv_exh_proof/class_c_store_entry_closure.py; cargo test --features 'generated_parsers ebnf_dual_run' --lib witness_store_entry_blocked"
---

**Two witness passes, two entry policies — and one of them is fatal for a store-gated target.**

PGEN has two independent witness passes. The certificate-coverage *plannable-rule* pass generates
from the **grammar entry** and steers *down* to the target via reach hops. The closed-loop *stimuli*
pass generates from the **target's own rule**, so the whole depth budget lands on the target's
subtree instead of being spent descending to it. For every target whose acceptance depends only on
structure, the second policy is strictly better. For a target gated on a **positive** semantic-store
predicate whose fact only an **ancestor** emits, it can never succeed:

- the witness starts from an **empty store**;
- the generation-side store-aware prune refuses the gated rule
  (`STORE-AWARE-GEN: rule '…' fact_count_at_least predicate unsatisfiable (zero source facts)`);
- so the target reports `never_hit` **at every depth and every seed**.

⛔ **A deeper budget cannot fix this, and neither can the semantic prelude.** A prelude hosts its
producer iterations at an on-path quantifier site *earlier in the sample* — and when the entry **is**
the gated rule, there is no earlier. The blocker is the **entry policy**. (Recorded because the
owning leaf's own first hypothesis — "the branch installer never calls `compute_reach_prelude`" — was
true about the source and irrelevant as a cause.)

**The fix is prior art, one pass over:** for a *store-entry-blocked* target only, generate from the
run's entry rule with `set_reach_plan_for_rule`, which brings the hops, the quantifier forcing and
the producer prelude the other pass has always used. Every other target keeps the own-rule entry
byte-for-byte.

**Three scoping rules make the verdict safe; each has a failure it prevents.**

| rule | what it prevents |
|---|---|
| **POSITIVE gates only** (`fact_count_at_least`, `has_fact`, `fact_attribute_equals`) | a `lacks_fact` gate is *satisfied* by an empty store — counting it would raise the entry for targets that witness fine today. The reach-BFS edge deprioritization counts every fact query on purpose, because there over-counting only re-ranks two ways of generating the same thing; a verdict that CHANGES what is generated cannot borrow that scope |
| **MANDATORY descent**, not "a gate exists below here" | an ordered choice with an ungated alternative is an escape the generator simply takes. A whole-closure scan calls SV's `net_declaration_sv_2017` blocked; the mandatory walk correctly does not (its first alternative is a plain `wire a;`) |
| **CLOSURE-relative**, not absolute | the same gate on the same rule is not blocking when a producer for its kind lives inside the target's own subtree — the witness emits the fact itself |

A **BRANCH** target is judged on its *targeted alternative* first: the rule's own root `Or` almost
always offers an ungated escape, which is exactly why a whole-rule walk cannot see a gate only the
forced branch reaches.

⭐ **One cause, two targets.** SV's gated rule is referenced from exactly one place — the
net-declaration alternative that is *also* a residual target — so the raised-entry witness selects
and succeeds that branch on its way. The gap report had already recorded it (`depends_on`), and
closing the cause closed both effects. Measured single-variable on the gate's own closed-loop replay
stage: residual **2 → 0** on both LRM profiles (each profile's two arms agreeing exactly); list-diffed
on `sv_2017`, **2 resolved / 0 new**, `covered_rules` 1336 → 1337, `covered_branches` 1450 → 1451,
unreachable debt unchanged, and marginally FASTER (447 s vs 455 s) — an always-failing target had been
paying a full construct attempt plus a full search fallback before giving up. Exactly **one** target
in the run takes the raised entry (`store_entry_raises=1`).

See also [[sv-residual-depth-budget-cause]] (the other residual class — where a budget *was* the
cause), [[branch-failure-reasons-are-the-witness-why]] (how to read which class you are in), and
[[closed-loop-residual-ratchet]] (banking the win).
