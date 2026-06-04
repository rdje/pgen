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
  Status: `done` (PGEN-ANNOTATION-COMPOSITION-0002, 2026-06-04 — canary grammar fix; collapsed to canonical style; orphan resolved + latent sv_2017 parser gap closed)
  Goal: `Fix binary_module_path_operator per IEEE 1800 §A.6.2: remove the spurious '@profiles: ["sv_2023"]' tag on binary_module_path_operator_sv_2023 (line 607) — the module-path binary operators (== != && || & | ^ ^~ ~^) are IDENTICAL in 2017 and 2023, so the tag is wrong. This makes the rule profile-neutral → the base alias is satisfiable under both profiles → no dangling reference, no phantom stimuli target, AND the generated sv_2017 parser correctly accepts module-path binary chains (latent gap closed). Minimal + AST-shape-preserving (no production/emit change).`
  Acceptance: `regen (parser mtime > grammar mtime); binary_module_path_operator generates under BOTH sv_2017 and sv_2023; SV external corpus stays 14/14; lib + shape-contract green; the stimuli phantom target rule::binary_module_path_operator no longer "Missing rule" under sv_2017.`
  Verification: `done — COLLAPSED binary_module_path_operator to match its sibling unary_module_path_operator (canonical operator-rule style): one profile-neutral rule, 9 bare {kind} branches, removed the spurious @profiles:["sv_2023"] tag + the _sv_2023 variant + the {body} wrapper (per director Q: it is NOT 2023-only — IEEE 1800 §A.6.2 operators are edition-invariant, and the sibling unary_module_path_operator is untagged/bare-{kind}). REGEN verified (generated parser mtime > grammar; binary_module_path_operator_sv_2023 count=0 in the parser, rule present). ORPHAN RESOLVED: generates under BOTH sv_2017 AND sv_2023 (was "Missing rule" under sv_2017). Shape contract 13/13 (rule corpus-unexercised → no live drift; calibration_history note added superseding the 2026-05-12 {body} note). lib --features generated_parsers --lib 650/0. SV external corpus triage 14/14 (parse_pass_total=14, fail=0, skip=0). Also closes a LATENT sv_2017 PARSER gap (module-path == chains were unparseable under sv_2017). generated/ is GITIGNORED → grammar source committed, regen is local.`
  Commit: `PGEN-ANNOTATION-COMPOSITION-0002`

- ID: `ANNOTATION-COMPOSITION.2`
  Status: `done` (PGEN-ANNOTATION-COMPOSITION-0003, 2026-06-04 — lint landed; SWEEP found 36 sibling orphans)
  Goal: `Implement the consistency invariant for @profiles in grammar_wellformedness.rs: compute S(rule) per profile bottom-up via the composition algebra (fixpoint, reusing the compute_nullable pattern); flag every rule PRESENT under profile P but NOT satisfiable under P (the orphan defect). Must NOT false-positive star/optional-guarded references (the algebra's ⊤ for opt/star). Wire into --lint-grammar + the well-formedness gate. SWEEP the SV grammar for siblings of the binary_module_path_operator defect and report them.`
  Acceptance: `the lint flags binary_module_path_operator under sv_2017 BEFORE .3's fix and passes AFTER; no false positive on module_path_expression (star-guarded ref); sweep output enumerates any other orphans; 2+ unit tests (orphan + star-guarded-not-orphan); parser-agnostic (zero grammar identifiers); lib + clippy green.`
  ImplGrounding: `(read-only survey 2026-06-04) grammar_wellformedness.rs operates on the rule-body HashMap<String, ASTNode> + rule_order; mirror its compute_nullable/node_nullable FIXPOINT but compute per-profile SATISFIABILITY sat[P][rule] (terminal=true, ref r = present(r,P) && sat[P][r], sequence=AND of required, alternation=OR, optional/star=true=⊤, plus=inner). @profiles is NOT in the ASTNode body — it is a DIRECTIVE: plumb a rule_profiles: HashMap<String,Vec<String>> built from annotations.semantic_annotations[rule] via the same extraction as ast_based_generator.rs::rule_profiles (line 6015; name=="profiles", parse_semantic_string_list, lowercased). all_profiles universe = union of declared profiles. INVARIANT: for each profile P, each rule PRESENT under P (profiles empty=universal OR contains P) must have sat[P][rule]==true; else WellformednessIssue::ProfileOrphan{rule, profile}. New enum variant + new pub fn detect_profile_orphans(grammar, rule_order, rule_profiles, all_profiles).`
  Verification: `done — added WellformednessIssue::ProfileOrphan + node_satisfiable (composition algebra: terminal=⊤, ref=defined&&present-under-P&&sat, seq=AND, alt=OR, opt/star=⊤, plus/+=inner, lookahead=⊤, EXTERNAL/undefined ref=⊤ for soundness) + detect_profile_orphans (per-profile satisfiability fixpoint mirroring compute_nullable; flags PRESENT-but-unsatisfiable rules, but ONLY when satisfiable under another profile so genuine non-termination is not conflated). Wired into --lint-grammar (main.rs): builds rule_profiles from annotations.semantic_annotations (@profiles directive, parse_semantic_string_list, lowercased) + the profile universe; runs only when >=2 profiles. 2 unit tests (orphan dangling + star-guarded-NOT-orphan) pass. lib (no-features) 590/0; clippy source 0. SWEEP on the real SV grammar: profile_orphans=36 (22 sv_2017 + 14 sv_2023) — binary_module_path_operator was NOT isolated; 36 siblings confirmed (many edition-rename pairs: nettype_declaration/net_type_declaration, rs_production/production, nonconsecutive_repetition/non_consecutive_repetition, severity_system_task/elaboration_severity_system_task, ...). Reported as WARNINGS (lint exit still tied to non_terminating=0). The 36 are a remediation backlog → .6.`
  Commit: `PGEN-ANNOTATION-COMPOSITION-0003`

- ID: `ANNOTATION-COMPOSITION.6.1`
  Status: `done` (characterization — PGEN-ANNOTATION-COMPOSITION-0004, 2026-06-04; tool-backed taxonomy of the 36)
  Goal: `WHY+WHERE characterize the 36 orphans before any fix; batch by defect-shape; identify binary_module_path_operator-style invariant TRAPS so they are NOT mis-fixed.`
  Verification: `done — ALL 36 share the structure 'base := base_sv_<EDITION>' (an untagged base aliasing a SINGLE-edition variant; the OTHER edition's variant never exists). Tool-verified taxonomy (THREE classes; the correct fix DIFFERS per class, so neither a blanket tag NOR a blanket derive-by-default is safe): (A) sv_2023-NEW features (~9 sv_2017-orphans: class_constructor_arg, class_constructor_arg_list, dynamic_override_specifiers, final_specifier, forward_type, incomplete_class_scoped_type, data_type_or_incomplete_class_scoped_type, initial_or_extends_specifier, type_parameter_declaration, type_identifier_or_class_type) — genuinely 2023-only → tag the base @profiles sv_2023 (or derive); SAFE (feature absent in 2017). (B) edition-RENAME pairs — construct served in BOTH editions under different names: net_type_declaration[17]↔nettype_declaration[23], net_type_identifier↔nettype_identifier, production(_identifier/_item)[17]↔rs_production(...)[23], weight_specification[17]↔rs_weight_specification[23], list_of_parameter_assignments[17]↔list_of_parameter_value_assignments[23], non_consecutive_repetition[17]↔nonconsecutive_repetition[23], elaboration_system_task[17]↔elaboration_severity_system_task[23] (+ severity_system_task), interface_instance_identifier[17]↔interface_port_identifier[23] → tag each base to its variant's edition; SAFE (counterpart serves the other edition). (C) invariant TRAPS — edition-INVARIANT construct with only ONE variant (tagging would create a REAL parser gap, the binary_module_path_operator class): CONFIRMED non_zero_decimal_digit (variant = literally digit "1"; decimal_digit itself is profile-neutral) + non_zero_unsigned_number (transitive). NEEDS LRM CHECK (rename vs trap): range_list[23]/open_range_list[17], open_value_range[17]. → these must be NEUTRALIZED (the .3 collapse), NEVER tagged. CONCLUSION: per-rule LRM §A grounding required ([[feedback_no_codebase_change_without_tool_backed_facts]]); the lint (.2) is the regression backstop meanwhile. Fix batches: .6.2 = Class A, .6.3 = Class B, .6.4 = Class C (+ resolve the range/value-range ambiguity), then lock the lint at 0.`
  Commit: `PGEN-ANNOTATION-COMPOSITION-0004`

- ID: `ANNOTATION-COMPOSITION.6`
  Status: `in_progress` (remediation umbrella — `.6.1` characterization done; `.6.2`/`.6.3`/`.6.4` fix batches pending)
  Goal: `Remediate the 36 @profiles orphans the .2 lint found (22 sv_2017 + 14 sv_2023). Each needs LRM-grounded per-rule analysis: is the construct genuinely edition-specific (→ tag the base/alias correctly so it is FILTERED OUT under the wrong profile, not present-but-dangling), or edition-invariant (→ remove the spurious tag / collapse like .3's binary_module_path_operator)? Many appear to be edition-RENAME pairs (e.g. nettype_declaration[2017] vs net_type_declaration[2023]) where a base rule references the wrong-edition variant. CAUTION: some orphans may be LATENT PARSER GAPS (like binary_module_path_operator's sv_2017 module-path gap) — check reachability. Batch by defect-shape; each batch: LRM check + fix + regen + corpus 14/14 + shape contract. Then ESCALATE the lint to a gate locked at profile_orphans=0 (so new orphans can't be introduced). NEVER game by mass-tagging to silence the lint — each tag must be LRM-correct ([[feedback_corpus_expected_from_spec_not_fix]]).`
  Acceptance: `profile_orphans -> 0 (each fix LRM-grounded, not silenced); any latent parser gaps found are noted; corpus 14/14 throughout; shape contract green; lint wired into a gate at 0; parser-agnostic.`
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
| — | `ANNOTATION-COMPOSITION.3` | `done` (`-0002`) | Canary grammar fix LANDED — collapsed to canonical style; orphan resolved both profiles; corpus 14/14; latent sv_2017 parser gap closed. |
| — | `ANNOTATION-COMPOSITION.2` | `done` (`-0003`) | Profile-consistency LINT landed; SWEEP found 36 sibling orphans (22 sv_2017 + 14 sv_2023). |
| — | `ANNOTATION-COMPOSITION.6.1` | `done` (`-0004`) | Characterized the 36 → 3 classes (A sv_2023-new, B rename-pairs, C invariant-traps); blanket fixes UNSAFE (traps). |
| 1 | `ANNOTATION-COMPOSITION.6.2` | `pending` (FRONTIER) | Fix Class A (sv_2023-new features → tag @profiles sv_2023), LRM-grounded + corpus 14/14. |
| 2 | `ANNOTATION-COMPOSITION.6.3` | `pending` | Fix Class B (edition-rename pairs → tag each base to its variant's edition). |
| 3 | `ANNOTATION-COMPOSITION.6.4` | `pending` | Fix Class C (invariant traps → NEUTRALIZE like .3; resolve range/value-range ambiguity); then lock the lint at 0. |
| 3 | `ANNOTATION-COMPOSITION.4` | `pending` | Derive-by-default — the elegant resolution (orphan class becomes unrepresentable). |
| 4 | `ANNOTATION-COMPOSITION.5` | `pending` | Tag-agnostic registry — the next tag plugs in; the general framework the director asked for. |
