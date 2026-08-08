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
  - "why is the raised witness entry not RULE-target-only"
  - "why must a branch-target reach plan carry a semantic prelude"
  - "why does prelude gate discovery start at the targeted alternative instead of the rule"
  - "why does the witness pass try the target's own rule before raising the entry"
  - "why did store_entry_raises drop from 16 to 1 without any coverage change"
  - "why can a coverage-equal A/B not tell you whether a reroute was necessary"
  - "why does a name-matching store gate NOT block a witness while a count gate does"
  - "which store gates may a blocking verdict count — positive ones or pruning ones"
  - "where is the generation-side store prune and what map does it read"
tags: [stimuli, coverage, semantic-store, witness, systemverilog, root-cause]
date: 2026-08-08
status: current
evidence: rust/src/ast_pipeline/stimuli_generator.rs (`witness_target_is_store_entry_blocked`, `target_forces_positive_store_gate`, `emittable_kinds_within_closure`, `StoreGateScope::GenerationPruned`, `gen_count_predicate_satisfiable`, the attempt-ordered arms inside `generate_target_witnesses` — `attempt_target_witness` / `witness_target_is_resolved`, and the branch half `set_reach_plan_forcing_quantifiers_with_prelude` / `count_gate_via_targeted_alternative` / `name_gate_via_targeted_alternative`); docs/tasks/SV-EXH-PROOF.md leaves .7.4.6.12 (WHY+WHERE_CLASS_C_2026-08-01, SIZING_CORRECTION_2026-08-01, FIX_LANDED_2026-08-02), .7.4.6.14 (the branch half) and .7.4.6.15 (the attempt order — MEASURED_PREMISE_2026-08-08, EPSILON_VECTOR_2026-08-08) and .7.4.6.17 (CENSUS_2026-08-08 / WHY_2026-08-08 / FIX_2026-08-08 — the verdict narrowed to the pruning gates); docs/book/src/grammar-wellformedness.md "The closed-loop witness pass's raised entry for store-gated targets"
reverify: "python3 docs/tasks/artifacts/sv_exh_proof/class_c_store_entry_closure.py; cargo test --features 'generated_parsers ebnf_dual_run' --lib store_entry; cargo test --features 'generated_parsers ebnf_dual_run' --lib name_gated_target_witnesses; cargo test --features 'generated_parsers ebnf_dual_run' --lib raised_branch_plan"
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
| **Gates GENERATION PRUNES ON** (`fact_count_at_least` — `StoreGateScope::GenerationPruned`) | a `lacks_fact` gate is *satisfied* by an empty store — counting it would raise the entry for targets that witness fine today. The reach-BFS edge deprioritization counts every fact query on purpose, because there over-counting only re-ranks two ways of generating the same thing; a verdict that CHANGES what is generated cannot borrow that scope. ⛔ This began as **POSITIVE gates only**, which was one criterion short — see the pruning-not-polarity section below |
| **MANDATORY descent**, not "a gate exists below here" | an ordered choice with an ungated alternative is an escape the generator simply takes. A whole-closure scan calls SV's `net_declaration_sv_2017` blocked; the mandatory walk correctly does not (its first alternative is a plain `wire a;`) |
| **CLOSURE-relative**, not absolute | the same gate on the same rule is not blocking when a producer for its kind lives inside the target's own subtree — the witness emits the fact itself |

A **BRANCH** target is judged on its *targeted alternative* first: the rule's own root `Or` almost
always offers an ungated escape, which is exactly why a whole-rule walk cannot see a gate only the
forced branch reaches.

⭐ **And the raise applies to branch targets too — the prelude is what had to be built.** The
verdict was branch-aware from day one and the branch installer already accepted an entry rule
distinct from its target rule, so the raise itself was a call-site widening. The missing piece was
the **semantic prelude**: the rule installer attaches one, the branch installer never did, so a
raised branch witness steers down correctly and still renders the gated rule against an empty
store. Attaching it required the prelude's gate discovery to be branch-scoped for a structural
reason worth remembering:

> a mandatory descent refuses an `Or` with an ungated escape — correctly, the generator would just
> take the escape. But a plan that **forces one alternative** has removed the escape, so for that
> plan the targeted alternative's mandatory render IS the whole render. Discovery starts at the
> alternative node, not at the rule.

Both prelude builders (count-gate and name-gate) gained that start, tried **last** so every gate the
existing legs find is found identically; the name-gate one keeps the `.4b.7` "only when the render
is unavoidably store-gated" guard so a prelude is never armed on an alternative that parses fine
from an empty store. The prelude-bearing installer is a separate entry point only the raised arm
calls.

⭐ **One cause, two targets.** SV's gated rule is referenced from exactly one place — the
net-declaration alternative that is *also* a residual target — so the raised-entry witness selects
and succeeds that branch on its way. The gap report had already recorded it (`depends_on`), and
closing the cause closed both effects. Measured single-variable on the gate's own closed-loop replay
stage: residual **2 → 0** on both LRM profiles (each profile's two arms agreeing exactly); list-diffed
on `sv_2017`, **2 resolved / 0 new**, `covered_rules` 1336 → 1337, `covered_branches` 1450 → 1451,
unreachable debt unchanged, and marginally FASTER (447 s vs 455 s) — an always-failing target had been
paying a full construct attempt plus a full search fallback before giving up. Exactly **one** target
in the run takes the raised entry (`store_entry_raises=1`).

⛔ **BUT THAT TRANSITIVE CLOSE IS A COINCIDENCE OF THE GRAMMAR, AND A GENERATOR IMPROVEMENT BROKE
IT.** Bounding the depth-slack retry ([[depth-slack-retry-needs-a-runaway-backstop]]) makes the
target-drive pass resolve the gated RULE early; the rule is then not a residual rule target when the
witness pass runs, the rule raise never fires (`store_entry_raises 1 → 0`), and the branch reopens
as `selected_but_failed` — `sv_2017` residual `0 → 1`. That is what forced the branch half. With it,
the same bounded configuration goes residual **1 → 0**; at the default configuration the change is
coverage-neutral (identical `covered_rules`/`covered_branches` and identical debt lists on both
profiles) and needs **14 fewer witnesses** on `sv_2017`, 13 fewer on `sv_2023`, because a raised
whole-file witness settles more targets at once. The mechanism is checked on the real parser, not
inferred: the produced witness `import\foo ::*;package\foo ;\foo \foo ;endpackage` gives
`parse_full passed`, and the same sample with only the import removed gives `furthest_position=17`.

⭐⭐ **THE ORDER OF THE TWO ATTEMPTS IS THE GUARANTEE — AND THE VERDICT WAS WRONG 15 TIMES IN 16.**
Both raises above decided from the verdict ALONE and then generated only from the raised entry, so a
target that would have witnessed fine from its own rule was rerouted whether or not it needed to be.
The pass now attempts the **own rule first** for every target and installs the raised entry only when
that attempt left the target **uncovered** — so a spurious verdict can no longer replace a working
witness, on any grammar. ⛔ The gate is *"is it covered now"*, **not** `result.is_err()`: a forced
branch whose gated content prunes lets a sibling rescue the rule, so the attempt returns `Ok` with
the branch uncredited, and an error-keyed reading would strand exactly the branch class above.

Measured single-variable on the replay stage: `store_entry_raises` **16 → 1** (`sv_2017`) and
**11 → 1** (`sv_2023`), residual `0`/`0`, every coverage figure identical, all four debt lists
`0 resolved / 0 new`. ⇒ **25 of the 27 raises were unnecessary**, and the survivors are the genuine
class-C targets. Price: `sample_errors` UNCHANGED (50/97 — every extra attempt succeeds, contra the
design's "wasted work" prediction), `+14`/`+6` witnesses, elapsed `256 → 258 s` and `444 → 436 s`.

⛔⛔ **THE DURABLE LESSON, AND IT IS NOT ABOUT THIS PASS.** The earlier verdict-first measurement
diffed HEAD against the fix and found coverage-EQUAL with byte-equal debt lists, concluding *"15 and
14 targets take the raise and not one target is lost"*. True, and the wrong question. Replacing a
working witness with another working witness moves **no** coverage number, so the metric being green
could not distinguish *"the reroute was necessary"* from *"the reroute was harmless"* — only the
ATTEMPT ORDER can, because the necessary ones are exactly the ones that still raise. **A mechanism
whose failure mode is invisible to the metric you gate on is not validated by that metric being
green** — [[feedback_instrument_needs_ground_truth]] restated for a policy rather than an instrument.

⭐⭐ **AND THE VERDICT IS NOW RIGHT 1 FOR 1 — IT HAD BEEN ASKING ABOUT POLARITY, NOT PRUNING.**
Naming the 25 (a per-target census — [[measure-a-policy-where-its-outcome-is]]) showed the population
splits **perfectly by gate CLASS**: the one genuine target per profile is a `fact_count_at_least`
**count** gate; all 15 (`sv_2017`) and 10 (`sv_2023`) spurious ones are `has_fact` /
`fact_attribute_equals` **name** gates. One rule carries both, adjacent, with identical bodies.

⛔ **The mechanism is a checkable property of the generator, not a story: there is exactly ONE
generation-side store prune and it is count-only.** `generate_rule` refuses a rule on
`!gen_count_predicate_satisfiable(rule)` (which reads `gen_count_kinds`) with
`STORE-AWARE-GEN: … fact_count_at_least predicate unsatisfiable (zero source facts)`. `gen_name_gate`
has **no analogue anywhere** — it is read only when planning a prelude, replaying a declared name
into a gated consumer, and repairing a colliding free name; never to refuse a render. So a
name-gated rule renders a fresh identifier against an empty store and its target is credited.

⇒ Since the verdict claims *"this target cannot be GENERATED from its own rule"*, and only the prune
can make that true, its scope is now exactly the map the prune reads. Measured single-variable:
blocked population **16 → 1** and **11 → 1**, spurious **15 → 0** and **10 → 0**,
`store_entry_raises` unchanged at `1`/`1`, residual `0`/`0`, and all **8** stage artifacts
**byte-identical**.

⭐ **The byte-neutrality was PREDICTED before it was measured, which is what makes it evidence.** The
raised arm is guarded `!covered && blocked` and `&&` short-circuits, so for a spurious target —
covered by its own rule by definition — the verdict was never consulted at run time even before the
fix. Ordering the attempts turned a correctness assumption into a cost knob; this turns the knob to
zero. ⚠️ What it gives up: the incidental benefit a raise might have brought a name-gated target the
own-rule attempt left uncovered for some *other* reason. None exists on either SV profile.

⛔ **A generalisable trap, worth more than this pass:** *"count every gate of the safe polarity"* felt
like the conservative choice and was not. Conservative means *"over-approximate the thing the
consumer actually acts on"* — here the generation prune — and a criterion that over-approximates a
DIFFERENT axis is just wrong in a direction nobody checks. The `epsilon` vector (`mandatory_node_gated`
answers `true` for any rule reference absent from `grammar_tree`, while `generate_rule` renders the
builtin as the empty string) is a REAL second over-approximation on the same predicate and was the
obvious suspect — it is **not** this population's cause, and not one of the 27 targets involves an
unresolvable reference. It remains a genuine defect and the basis of the ordering control's
precondition.

See also [[sv-residual-depth-budget-cause]] (the other residual class — where a budget *was* the
cause), [[branch-failure-reasons-are-the-witness-why]] (how to read which class you are in), and
[[closed-loop-residual-ratchet]] (banking the win).
