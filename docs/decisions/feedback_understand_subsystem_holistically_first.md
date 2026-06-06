---
name: feedback-understand-subsystem-holistically-first
description: STANDING, emphatic (director 2026-06-06) — NEVER build understanding of a subsystem piecemeal (grep → infer → act). It is extremely dangerous and error-prone: each fragment can be locally true while the assembled model is globally WRONG, and that error is confidently held and only caught late. Before planning or touching code in a subsystem, read its architecture docs END-TO-END and trace it in FLOW ORDER to build ONE holistic model first.
metadata:
  node_type: memory
  type: feedback
  director_directive: true
  created: 2026-06-06
---

**Director directive (2026-06-06), emphatic.** "Doing this 'building understanding piecemeal' is
extremely dangerous and error prone. Why would you do this in the first place?"

**The rule.** For any subsystem I will reason about or change, FIRST build a thorough, *holistic*
model — read the authoritative architecture docs straight through and trace the components in
data-flow order — BEFORE forming a plan or touching code. Do not substitute "enough fragments to take
the next step" for genuine understanding.

**Why piecemeal is dangerous (own it).** grep/read give fast, plausible-*feeling* partial answers, so
it is easy to mistake "I found a relevant line" for "I understand the system." Each fragment can be
individually TRUE while the assembled picture is WRONG — and that wrong model is held confidently and
surfaces only when challenged (late, expensive, and in foundational code potentially catastrophic).

**The triggering mistakes (2026-06-06, the lexical-annotations / EBNF-frontend work).**
- Inferred "regenerate the bootstrap parser" from the Makefile's seed target — WRONG; the authoritative
  parser is the hand-written `src/ebnf_frontend.rs` (the seed exists but isn't the live path). Reading
  `ebnf_frontend.rs` / the architecture docs first would have prevented it.
- Concluded too quickly "the generator can't consume position-specific annotations" before tracing how
  inline semantic annotations actually flow — needed correction.
- Let a half-formed plan steer which lines I grepped (confirmation, not comprehension).

**How to apply.**
1. Identify the authoritative *architecture* docs for the subsystem (here: `docs/AST_GENERATOR_ARCHITECTURE.md`,
   `docs/ast_transformation_pipeline.md`, `docs/reference/RUST_CODEBASE_ANALYSIS.md`, the developer-architecture
   book chapter) and read them end-to-end.
2. Trace the subsystem in *flow order* (e.g. `.ebnf` → `ebnf_frontend` → `transform_from_raw_ast` → IR
   (`ASTNode` + `Annotations`) → the two consumers: codegen vs stimuli generator), building the data
   structures and the consumption matrix.
3. Capture the model durably (KM card / book) so it is verifiable and reusable, not re-derived.
4. Only THEN plan and change code. If no architecture doc exists, build understanding by reading whole
   modules in flow order — still holistic, never fragment-by-fragment to justify a pre-chosen plan.

Reinforces [[feedback_tools_first_no_guessing]] (the tool must DIRECTLY show the truth — not feed a
guess), [[feedback_why_and_where_before_solution]] (know WHY + WHERE fully first),
[[feedback_no_codebase_change_without_tool_backed_facts]], [[feedback_research_grounded_sota_no_trial_and_revert]]
(survey first), and [[feedback_quality_over_speed_no_corners]] (momentum is never a reason to hold a
half-built model).
