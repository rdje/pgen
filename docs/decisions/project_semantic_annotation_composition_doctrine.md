---
name: project-semantic-annotation-composition-doctrine
description: Doctrine — semantic-annotation tags (e.g. @profiles) have COMPOSITION semantics across rule references; the toolchain must DERIVE, CHECK, and elegantly RESOLVE them uniformly and tag-agnostically. Detection alone is insufficient; correctness must offer an elegant resolution.
metadata:
  node_type: decision
  type: project
  created: 2026-06-04
  director_directive: true
  owning_tree: ANNOTATION-COMPOSITION
---

# Doctrine: semantic-annotation composition across rule references

**Director directive (2026-06-04).** Triggered by a stimuli-coverage residual that turned out
to be a real grammar bug: `binary_module_path_operator` (an alias whose only production,
`binary_module_path_operator_sv_2023`, is tagged `@profiles: ["sv_2023"]`, while the alias
itself is untagged) is a **profile orphan** — present under `sv_2017` but unsatisfiable there,
because the profile filter removed its only production, leaving a dangling reference. The
operators are identical in IEEE 1800-2017 and -2023 (§A.6.2), so the `@profiles` tag is simply
spurious. The stimuli generator surfaced it as `rule::binary_module_path_operator → "Missing
rule"`.

The director's escalation: **this one bug reveals a more serious, general problem.** Today the
tag is `@profiles`; tomorrow it could be a different tag — *we cannot know in advance*. And the
real question is **composition**: when a rule references several other rules, each tagged
differently, what is the rule's effective tag, and what must hold for the grammar to be
consistent? Correctness work must not only *detect* these — it must provide a way to *resolve*
them **elegantly**.

## The principle (tag-agnostic)

A semantic-annotation **tag-kind** (`@profiles` is instance #1) is defined by four things, and
the toolchain (well-formedness checker, **parser generator**, **stimuli generator**) MUST treat
all tag-kinds uniformly through this same shape — never special-casing one tag:

1. **Value domain** — a lattice/set of values (for `@profiles`: a set of edition strings, e.g.
   `{sv_2017, sv_2023}`; untagged ⇒ the universal value ⊤ = "all profiles").
2. **Composition algebra** — how the tag combines bottom-up across the grammar combinators.
   For `@profiles` (a *satisfiability* set — "which values is this satisfiable under"):
   - **terminal / token** → ⊤ (profile-neutral).
   - **rule reference `r`** → the referent's *derived* set `S(r)` (this is the propagation).
   - **sequence `e1 e2 … en`** (all required) → `S(e1) ∩ S(e2) ∩ … ∩ S(en)` (**intersection** —
     satisfiable only where *every* required part is).
   - **alternation `e1 | … | en`** → `S(e1) ∪ … ∪ S(en)` (**union**), and each branch is
     **gated** to its own `S(ei)`.
   - **optional `e?`, repetition `e*`** → ⊤ (the construct can be empty, so the *whole* is
     satisfiable everywhere; when present, `e` is gated to `S(e)`). **This is why a reference
     inside `( … )*` does NOT make its container an orphan** (the `binary_module_path_operator`
     reference in `module_path_expression` is star-guarded, so `module_path_expression` stays
     satisfiable; only the standalone alias is the orphan).
   - **repetition `e+`** → `S(e)` (at least one required).
   - a rule = the alternation of its productions ⇒ `S(rule)` = union of production sets.
   - Computed as a **fixpoint** over the (mutually recursive) rule graph — same machinery as the
     existing nullability/`compute_nullable` pass in `grammar_wellformedness.rs`.
3. **Consistency invariant** — for every value `P` in the domain: **every rule PRESENT under `P`
   (kept by the filter) must be SATISFIABLE under `P`** (`P ∈ S(rule)`). A present-but-
   unsatisfiable rule is the *orphan* defect. (Equivalently: a declared tag must be ⊆ the
   derived satisfiable set.)
4. **Resolution policy (elegant — not just a diagnostic).** In priority order:
   - **(R1) Derive, don't declare (default).** A rule's effective tag should be *computed*
     bottom-up via the algebra; an explicit `@tag` becomes an **optional assertion** the checker
     *verifies* (`declared == derived`, or `declared ⊆ derived`). Derivation makes the orphan
     class *unrepresentable* — you cannot under-tag a rule into inconsistency. This is the
     elegant resolution the multi-reference case demands: a rule referencing rules of mixed tags
     simply *gets* the combinator's set-op result automatically.
   - **(R2) Auto-gate branches.** An alternation branch that references a `P`-only rule is itself
     a `P`-only branch → pruned under ¬`P` rather than dangling.
   - **(R3) Flag + propose the minimal edit.** When an explicit assertion conflicts with the
     derived value, report it AND suggest the minimal fix (drop the tag / widen it / add the
     missing-profile production) — so correctness *resolves*, not just *accuses*.

## The multi-reference answer (the director's specific question)

"What if a rule references several rules, each profiled differently?" — apply the combinator's
set operation: **sequence ⇒ intersection, alternation ⇒ union (branch-gated), optional/star ⇒
absorbing (⊤)**. E.g. `A := r1 r2` with `S(r1)={2017,2023}`, `S(r2)={2023}` ⇒ `S(A)={2023}`
(A is auto-derived sv_2023-only). `A := r1 | r2` ⇒ `S(A)={2017,2023}` with the `r2` branch gated
to `{2023}`. No human tagging required; no orphan possible.

## Grounding (not reinvented)

This is a **synthesized attribute** over the grammar (attribute grammars, Knuth 1968): each
tag-kind is an attribute whose values form a (semi)lattice and whose composition is defined per
production. The consistency invariant is an attribute-grammar well-formedness condition; the
fixpoint computation is the standard one (cf. nullability / FIRST/FOLLOW). Per
[[feedback_research_grounded_sota_no_trial_and_revert]] we adopt the framing rather than invent
an ad-hoc per-tag check. Composes with the PARSE-SOTA well-formedness lints (`.8`/`.9`).

## Scope & constraints

- **Parser-agnostic, mandatory** — the framework lives in the shared pipeline / well-formedness
  layer and benefits every grammar pgen compiles; zero grammar identifiers in the engine logic
  ([[feedback_ast_pipeline_parser_agnostic]], reaffirmed emphatically by the director
  2026-06-04). A tag-kind registers its algebra; the machinery is uniform.
- **Fix hierarchy** ([[feedback_no_workarounds_fix_hierarchy]]): the canary fix is a pure
  grammar correction (level 1 — remove the spurious tag, LRM-grounded); the lint + derivation
  framework are general pipeline features (parser-agnostic).
- **Never game the metric** ([[feedback_corpus_expected_from_spec_not_fix]]): orphan targets are
  removed from the stimuli universe ONLY as a *consequence* of fixing/deriving the grammar's
  real tags — never by trimming the target list to make a number go to zero.

## Extension recipe — adding a new tag-kind (ANNOTATION-COMPOSITION.5)

`@profiles` is instance #1, fully worked (derive + check + resolve). A second tag-kind follows the
SAME shape — no checker rewrite, no special-casing:

1. **Declare its four parts** (the principle above): value domain; composition algebra (how a
   sequence / alternation / quantifier / reference composes the tag — `seq=∩`, `alt=∪`, `opt/star=⊤`,
   `ref=referent's value`); consistency invariant (`present ⇒ …`); resolution policy (derive-by-default
   / auto-gate / explicit-as-verified-assertion).
2. **Provide two pure functions** over `ASTNode` + a `rule → tag-values` map + the value universe,
   mirroring the `@profiles` worked instance in `grammar_wellformedness.rs`:
   - a **DERIVE** function (cf. `derive_rule_profiles`) — the per-value fixpoint giving each rule's
     derived tag-set; reuse the `compute_sat_by_profile` fixpoint skeleton;
   - a **CHECK** function (cf. `detect_profile_orphans`) returning `WellformednessIssue`s, each
     carrying the **derived minimal fix** (cf. `suggested_profiles`).
3. **Wire it into `run_grammar_lint`** alongside the existing detectors (one line), and into the
   generator's profile-filter if the tag gates generation (the way `@profiles` does).

**Why no generic runtime registry yet — deferred on purpose.** There is exactly ONE real tag-kind
(`@profiles`). Building a generic dispatch engine + a contrived second tag to exercise it would be
premature abstraction — it would be shaped around `@profiles`' specifics and likely mis-fit a real
second tag, against the fix hierarchy's "concrete justification for level-3+ additions"
([[feedback_no_workarounds_fix_hierarchy]]) and the rule-of-three. The model + this recipe are
documented and the worked instance is in place; **extract the shared abstraction FROM the two
instances when a real second tag-kind arrives.** (`ANNOTATION-COMPOSITION.5`: model + recipe DONE;
generic-registry implementation deferred with this trigger.)

Owned by the **`ANNOTATION-COMPOSITION`** task tree (`docs/tasks/ANNOTATION-COMPOSITION.md`).
First instance + canary: `binary_module_path_operator` (`@profiles`). Related:
[[feedback_grammar_rules_must_consult_store]], [[reference_annotation_binds_following_rule]].
