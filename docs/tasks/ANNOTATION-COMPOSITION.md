# ANNOTATION-COMPOSITION — semantic-annotation tag composition across rule references

> Task tree. Owns the general framework for how **semantic-annotation tags** (e.g. `@profiles`)
> compose across rule references, and how the toolchain (well-formedness checker, parser
> generator, stimuli generator) must DERIVE, CHECK, and elegantly RESOLVE them — tag-agnostically.
>
> **Metadata**
> - Status: `active`
> - Created: 2026-06-04 (`PGEN-ANNOTATION-COMPOSITION-0001`)
> - Origin: a stimuli-coverage residual (`SV-EXH-PROOF.7.4.6.6`) that proved to be a real grammar
>   bug — `binary_module_path_operator` is a `@profiles` orphan (present under sv_2017,
>   unsatisfiable there). Director escalated: one bug reveals a general problem (tag composition).
> - Doctrine: [[project_semantic_annotation_composition_doctrine]] (the four-part tag-kind model;
>   composition algebra; derive-by-default; consistency invariant; elegant-resolution policy).
> - Disciplines: [[feedback_ast_pipeline_parser_agnostic]] (HARD — zero grammar identifiers in
>   engine logic; reaffirmed emphatically 2026-06-04), [[feedback_research_grounded_sota_no_trial_and_revert]]
>   (attribute-grammar grounding), [[feedback_no_codebase_change_without_tool_backed_facts]],
>   [[feedback_no_workarounds_fix_hierarchy]], [[feedback_corpus_expected_from_spec_not_fix]]
>   (never game the stimuli metric by trimming targets).

---

## Root

- ID: `ANNOTATION-COMPOSITION`
  Status: `active` (canary fix `.3` first; lint `.2` enforces; derivation `.4` resolves; `.5` generalizes)
  Goal: `Make semantic-annotation tags COMPOSE correctly and consistently across rule references, detected at grammar-compile time and resolved elegantly, for ANY tag-kind (not just @profiles).`

## Leaves

- ID: `ANNOTATION-COMPOSITION.1`
  Status: `done` (design — PGEN-ANNOTATION-COMPOSITION-0001, 2026-06-04)
  Goal: `Characterize the doctrine: the four-part tag-kind model (value domain, composition algebra, consistency invariant, resolution policy); the @profiles algebra (terminal=⊤, ref=S(r), seq=∩, alt=∪ branch-gated, opt/star=⊤, plus=S(e)); derive-by-default; the multi-reference answer (combinator set-ops); attribute-grammar grounding.`
  Verification: `done — decision record docs/decisions/project_semantic_annotation_composition_doctrine.md authored + indexed; registered in docs/TASK_TREE.md.`
  Commit: `PGEN-ANNOTATION-COMPOSITION-0001`

- ID: `ANNOTATION-COMPOSITION.3`
  Status: `pending` (canary GRAMMAR fix — do FIRST; it also closes a latent sv_2017 parser gap)
  Goal: `Fix binary_module_path_operator per IEEE 1800 §A.6.2: remove the spurious '@profiles: ["sv_2023"]' tag on binary_module_path_operator_sv_2023 (line 607) — the module-path binary operators (== != && || & | ^ ^~ ~^) are IDENTICAL in 2017 and 2023, so the tag is wrong. This makes the rule profile-neutral → the base alias is satisfiable under both profiles → no dangling reference, no phantom stimuli target, AND the generated sv_2017 parser correctly accepts module-path binary chains (latent gap closed). Minimal + AST-shape-preserving (no production/emit change).`
  Acceptance: `regen (parser mtime > grammar mtime); binary_module_path_operator generates under BOTH sv_2017 and sv_2023; SV external corpus stays 14/14; lib + shape-contract green; the stimuli phantom target rule::binary_module_path_operator no longer "Missing rule" under sv_2017.`
  Verification: `pending`
  Commit: `pending`

- ID: `ANNOTATION-COMPOSITION.2`
  Status: `pending` (profile-consistency LINT — the first instance of the doctrine's checker)
  Goal: `Implement the consistency invariant for @profiles in grammar_wellformedness.rs: compute S(rule) per profile bottom-up via the composition algebra (fixpoint, reusing the compute_nullable pattern); flag every rule PRESENT under profile P but NOT satisfiable under P (the orphan defect). Must NOT false-positive star/optional-guarded references (the algebra's ⊤ for opt/star). Wire into --lint-grammar + the well-formedness gate. SWEEP the SV grammar for siblings of the binary_module_path_operator defect and report them.`
  Acceptance: `the lint flags binary_module_path_operator under sv_2017 BEFORE .3's fix and passes AFTER; no false positive on module_path_expression (star-guarded ref); sweep output enumerates any other orphans; 2+ unit tests (orphan + star-guarded-not-orphan); parser-agnostic (zero grammar identifiers); lib + clippy green.`
  Verification: `pending`
  Commit: `pending`

- ID: `ANNOTATION-COMPOSITION.4`
  Status: `pending` (elegant RESOLUTION — derive-by-default)
  Goal: `Make a rule's effective @profiles set DERIVED from its productions (bottom-up via the algebra) rather than hand-declared; treat an explicit @profiles as a VERIFIED ASSERTION (declared ⊆ derived) + auto-gate alternation branches to their referents' profiles. Eliminates the orphan class structurally (you cannot under-tag into inconsistency). Parser-gen + stimuli-gen consume the derived value. Measure: the orphan class becomes unrepresentable; no regression in corpus/shape; determinism.`
  Acceptance: `derivation matches hand-tags where they were correct; conflicts (if any) reported with the minimal-edit suggestion; corpus 14/14; shape contract green; parser-agnostic; lib+clippy green.`
  Verification: `pending`
  Commit: `pending`

- ID: `ANNOTATION-COMPOSITION.5`
  Status: `pending` (tag-AGNOSTIC generalization)
  Goal: `Generalize the .2/.4 machinery into a TagKind registry: each tag-kind declares its value domain + composition algebra + consistency rule + resolution default; the well-formedness checker, parser generator, and stimuli generator all consult the registry uniformly. @profiles becomes the first registered tag-kind; a future tag plugs in its algebra with no checker rewrite. Document the extension point in the book.`
  Acceptance: `@profiles re-expressed as a registered TagKind with identical behavior to .2/.4; a second (test/example) tag-kind validated through the same path; book chapter on the extension point; parser-agnostic; lib+clippy green.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `ANNOTATION-COMPOSITION.3` | `pending` | Canary grammar fix — small, LRM-grounded, also closes a latent sv_2017 parser gap; unblocks the stimuli residual. |
| 2 | `ANNOTATION-COMPOSITION.2` | `pending` | Lint — enforces the invariant + sweeps for siblings so the fix doesn't regress and others surface. |
| 3 | `ANNOTATION-COMPOSITION.4` | `pending` | Derive-by-default — the elegant resolution (orphan class becomes unrepresentable). |
| 4 | `ANNOTATION-COMPOSITION.5` | `pending` | Tag-agnostic registry — the next tag plugs in; the general framework the director asked for. |
