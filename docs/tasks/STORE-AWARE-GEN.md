# STORE-AWARE-GEN: semantic-store-aware (context-valid) stimuli generation — the generation-side dual of the parser's `@predicate`

## Metadata

- Tree ID: `STORE-AWARE-GEN`
- Status: `active` (`.1` SCOPING + `.2` DESIGN + `.3` IMPLEMENT done — the `fact_count_at_least`-aware MVP landed and closed `REGEX-PCRE2-FIDELITY.3.12`; frontier `.4` generalize to the other composable predicates, `.5` per-grammar verify)
- Family / slice-id prefix: `PGEN-STORE-AWARE-GEN-<NNNN>`
- Roadmap lane: stimuli-generator quality / parser sign-off pillars **C** (fidelity) + **D** (coverage) —
  the generator must emit only samples that satisfy the SAME semantic predicates the parser enforces
- Created: `2026-06-08`
- Owner: repo-local workflow
- Director directive: 2026-06-08 — "plan this new generator capability (semantic-store-aware generation
  honoring `fact_count_at_least`) for the stimuli generation at some point … task-tree track it so that
  we do not forget."
- Decision record: [[project_store_aware_generation]]

## The frame (why this tree exists)

PGEN's EBNF (+ its `@predicate` / `@emit_fact` / `@fact_kind` semantic annotations) is the **single
source of truth** for the accepted language ([[project_ebnf_is_single_source_of_truth]]). The PARSER
honours those predicates: a rule gated by `@predicate: { name: fact_count_at_least, args:
[regex_capture_group, $index], phase: post }` is accepted only when the semantic store proves the
constraint. But the **stimuli generator is predicate-BLIND** — it has *no* fact/predicate machinery
(tool-checked: `fact_count_at_least` is evaluated only in `semantic_runtime.rs` and the linter, never in
`stimuli_generator.rs`). So the generator over-generates strings that are structurally valid but
**semantically invalid**, which the parser then correctly rejects — a generator⟷parser duality break at
the SEMANTIC level (the same shape of defect EBNF-SOURCE-OF-TRUTH names at the structural level).

The capability this tree owns: a **generation-time semantic store** that the generator maintains as it
emits — emitting the same facts the parser would emit (`@emit_fact`), and **evaluating the same
`@predicate` annotations to gate which branches/values it generates**, so every generated sample
satisfies the grammar's semantic predicates by construction. The *same* annotations then steer BOTH
parsing and generation (the bidirectional ideal already stated for the lexical pillar) — closing the
duality at the semantic level.

### The trigger (tool-backed, 2026-06-08)

`REGEX-PCRE2-FIDELITY.3.12`: after the `.3.7` (b)+(c) spacing fix (`PGEN-LEXICAL-ANNOTATIONS-0024`), the
regex cert-coverage residual at seed 1 is `\98495*` — the generator emits a multi-digit numeric
backreference (`numeric_backreference = "\\" backreference_digits`, gated in the grammar by
`@predicate fact_count_at_least(regex_capture_group, $index)`) to a group that does not exist. `\9`
passes; `\98`/`\984`/`\98495` reject (PCRE2-faithful — `\98…` with no group 98 is error 115). The
PARSER is correct; the GENERATOR is predicate-blind. A fixed numeric cap on `backreference_digits` would
be a guess (the valid bound is the context-dependent capture-group count) — i.e. a workaround. The
principled fix is for the generator to honour `fact_count_at_least` against a generation-time store.

## SOTA grounding (research-grounded; cite before acting — [[feedback_research_grounded_sota_no_trial_and_revert]])

- **ISLa — "Input Invariants" (Steinhöfel & Zeller, ESEC/FSE 2022).** The canonical SOTA: a declarative
  *input specification language* layering **semantic constraints** (e.g. "a `def` must reference an
  existing element", "a length field equals the body length") on top of a context-free grammar, then
  generating inputs that satisfy grammar **and** constraints (SMT-backed solving + grammar fuzzing).
  PGEN's `@predicate fact_count_at_least(regex_capture_group, $index)` is exactly an ISLa-style semantic
  constraint ("`\N` references an existing capture group"); this tree is PGEN's native realization of
  the ISLa idea, reusing the EXISTING `@predicate` vocabulary instead of a separate constraint language.
- **Data-dependent / two-level grammars** (van Wijngaarden; Jim et al. POPL 2010, the basis of PGEN's
  store-gated rules per [[project_vision_and_discipline]]): the parse-side mechanism already used; this
  tree mirrors it on the generation side.
- **The Fuzzing Book (Zeller et al.)** — grammar-based generation + coverage (already cited by
  STIMULI-SIGNOFF); semantic/constraint generation is its `ISLa`/constraint-fuzzing chapter.
- **Purdom (1972)** minimal-sentence generation — the witness/min-length lineage PGEN already uses
  (`SV-EXH-PROOF.7.4.2`, STIMULI-SIGNOFF.2).
- **PGEN-native angle:** the generator should reuse `semantic_runtime.rs`'s predicate evaluator and fact
  store on the generation side (one evaluator, two drivers) — the most idiomatic, lowest-duplication
  design, and the one that keeps the EBNF the single source of truth.

## Goal

The stimuli generator, in faithful mode, emits only samples that satisfy the grammar's `@predicate`
constraints — starting with `fact_count_at_least` (the regex `.3.12` driver) and generalizing to the
composable primitive set (`has_fact`, `lacks_fact`, `fact_attribute_equals`, `resolve_path`). The SAME
`@predicate`/`@emit_fact` annotations steer both parse and generation; no new annotation vocabulary is
introduced (the generator becomes a second consumer of the existing one).

## Non-Goals

- A separate constraint language (ISLa-style standalone): PGEN already has `@predicate`; reuse it.
- Full SMT solving in the first cut: start with the directly-decidable predicates (fact-count, has/lacks
  fact) via a generation-time store + bounded retry/biasing; escalate to constraint-solving only if a
  predicate genuinely needs it (and only with tool evidence).
- Guaranteeing a globally-uniform distribution under constraints (a Boltzmann/quantitative concern owned
  by STIMULI-SIGNOFF.5); this tree is about *validity*, not distribution.
- Touching the parser or the grammar acceptance semantics (generation-only; parser stays the oracle).

## Acceptance Criteria

- A generation-time semantic store + predicate evaluator (reusing `semantic_runtime.rs`), driven by the
  generator, parser-AGNOSTIC.
- `numeric_backreference` (and any `fact_count_at_least`-gated rule) generates only indices the
  generation-time store can satisfy → regex DEFAULT cert-coverage `sample_parse_failures` = 0 at seed 1
  (and across a seed sweep), closing `REGEX-PCRE2-FIDELITY.3.12`.
- No regression: `stimuli_cross_family_platform_gate` green; every grammar's stimuli gate green; the
  mechanism is a no-op (byte-identical) for grammars with no `@predicate`-gated generative construct.
- Per the no-workarounds hierarchy: a Level-3+ general parser-agnostic generator capability, justified
  because the generator structurally cannot otherwise honour a context-dependent predicate; documented as
  such. No fixed-bound guesses.
- Lockstep docs: this tree, the decision record, the book (a "context-valid generation" section), CHANGES.

## Leaves

- `.1` — **SCOPING + SOTA SURVEY (this slice, `PGEN-STORE-AWARE-GEN-0001`, pure docs). DONE.** Named the
  capability; tool-backed root cause (generator has zero predicate/fact machinery; `fact_count_at_least`
  is parse-only); cited the SOTA (ISLa input invariants + data-dependent grammars + Fuzzing Book);
  recorded the decision; registered the tree; pointed `REGEX-PCRE2-FIDELITY.3.12` at it. Establishes the
  cited ground per the research-grounded-SOTA discipline. NO code.
- `.2` — **DESIGN (pure docs). DONE (`PGEN-STORE-AWARE-GEN-0002`, 2026-06-08) —
  [`STORE-AWARE-GEN-design.md`](STORE-AWARE-GEN-design.md).** Pinned the exact mechanism, tool-backed (all
  file:line refs verified): (a) a `gen_semantic_state: SemanticRuntimeState` on the generator + an EMIT
  hook in `generate_rule` mirroring the codegen effect phase (`ast_based_generator.rs:1566`); (b) two
  predicate strategies — (A) constraint-directed (preferred/invertible: for `fact_count_at_least(K,
  $index)` constrain the generated index to ≤ `gen_semantic_state.count(K)` and PRUNE the branch when the
  count is 0, mirroring branch-predicate pruning) and (B) generate-check-backtrack (general fallback,
  mirrors the parser's post-predicate); (c) the checkpoint/rollback discipline reusing the parser's EXACT
  API (`checkpoint`/`rollback_to_named`/`extract_delta_since`/`apply_delta`) at every speculative
  generation site (`generate_or`, `generate_quantified`, the relational retry loop, target/witness
  retries); (d) the no-op guarantee via a one-time `grammar_has_generative_predicates` gate (byte-identical
  for predicate-free grammars). Reuses `parse_semantic_runtime_directives` + the resolve helpers — no new
  runtime code. Includes the `.3` MVP scope, the verification matrix, risks/mitigations, and the
  no-workarounds-hierarchy placement (Level-3+, justified). The `.3`–`.5` implementation is now turnkey.
- `.3` — **IMPLEMENT `fact_count_at_least`-aware generation (the regex `.3.12` driver). DONE
  (`PGEN-STORE-AWARE-GEN-0003`, 2026-06-08).** Landed exactly the `.2` design MVP, generator-only
  (`stimuli_generator.rs`): `gen_semantic_state: SemanticRuntimeState` + `store_aware_gen` no-op gate +
  precomputed `gen_emit_facts`/`gen_count_kinds` (via `compute_store_aware_gen_directives`, reusing
  `parse_semantic_runtime_directives`); EMIT hook in `generate_rule` (on success, mirrors the parser's
  effect phase — a generated capture group emits `regex_capture_group`); the sound NECESSARY-CONDITION
  prune (`gen_count_predicate_satisfiable`: a `fact_count_at_least(K,$ref)` rule is unsatisfiable when
  `count(K)==0` since no positive `$ref` can match an empty set — evaluated via the SAME
  `evaluate_predicate`) → `generate_rule` fails fast → `generate_or` (which already retries the next
  branch) backtracks to a satisfiable alternative; checkpoint/rollback of `gen_semantic_state` at
  `generate_or` AND `generate_quantified` (mirrors `try_parse`); per-sample store reset in
  `generate_from_entry`. NO grammar change, NO regen, NO AST change, NO version bump (surface-neutral;
  the regex PARSER is unchanged — only the generator emits more semantically-valid samples). **VERIFIED:**
  regex DEFAULT cert-coverage `sample_parse_failures` = **0 across the seed sweep** (seed 0/1/7/13 + count
  500/seed 0,1 — seed 1 was 1 [`\98495`], now 0); determinism byte-identical; backrefs still generated
  (construct not destroyed); json/VHDL/SV byte-identical (`stimuli_cross_family_platform_gate` ✅,
  no-op gate); `regex_pcre2_compile_oracle_gate` byte-identical; self-hosting OK; `cargo test --lib`
  **620/0** (+2 locks: `store_aware_gen_count_predicate_necessary_condition`,
  `store_aware_gen_off_for_predicate_free_grammar`); strict source clippy clean. **Closes
  `REGEX-PCRE2-FIDELITY.3.12` — regex DEFAULT cert-coverage is now clean across all measured seeds.**
  HONEST scope: the MVP prunes the `count(K)==0` category (the systematic one). The tight bound
  (`$index ≤ count` when `count ≥ 1`) — a generated multi-digit backref whose value exceeds a non-zero
  group count — is not yet enforced; it did not arise in the seed sweep (it needs ≥1 capture group AND a
  backref value in `(count, generated]`), and is the natural `.4` value-constraint refinement.
- `.4` — **GENERALIZE to the composable primitive set** (`has_fact`, `lacks_fact`,
  `fact_attribute_equals`, `resolve_path`): gate branch/value selection generally so any `@predicate`-
  gated rule generates only satisfiable samples. Tools-first, one primitive at a time, measured.
- `.5` — **VERIFY + per-grammar cert-coverage closure.** Re-run cert-coverage per grammar (ties into
  `GRAMMAR-WELLFORMED` Phase H): generated samples honour every `@predicate` → no semantic
  round-trip failures. Round-trip / golden tests; book + decision lockstep.

## Cross-links

- **Closes:** `REGEX-PCRE2-FIDELITY.3.12` (numeric-backreference over-generation) — the concrete trigger.
- **Composes with:** [[project_ebnf_is_single_source_of_truth]] (extends the duality from structure to
  semantics), `GRAMMAR-WELLFORMED` (the certifying duality — a witness must round-trip), `STIMULI-SIGNOFF`
  (generator signoff vision; this is the *validity* axis vs its coverage/distribution axes),
  `PARSE-FIDELITY` (pillar C — correct AST / no silent mis-generate).
- **Disciplines:** [[feedback_research_grounded_sota_no_trial_and_revert]] (survey first — done in `.1`),
  [[feedback_no_workarounds_fix_hierarchy]] (Level-3+, justified; no fixed-bound guess),
  [[feedback_ast_pipeline_parser_agnostic]] (absolute — the store/evaluator are reused, parser-agnostic),
  [[feedback_tools_first_no_guessing]].

## Blockers

- None. Implementation is a deliberate effort (a new generator capability); sequenced after the current
  regex cert-coverage work per the director ("at some point"). `.1` (this scoping) is complete now so the
  capability is tracked and not forgotten.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-08` | `.1` | tool-backed root cause (generator has no fact/predicate machinery — grep of `stimuli_generator.rs`; `fact_count_at_least` parse-only in `semantic_runtime.rs`/linter); SOTA cited (ISLa, data-dependent grammars, Fuzzing Book) | SCOPING DONE (docs) |
| `2026-06-08` | `.2` | tool-backed design — verified the parser's emit/predicate runtime (`SemanticFactSpec`/`SemanticPredicateSpec`/`evaluate_predicate`/`emit_fact`/`checkpoint`/`rollback_to_named`/`extract_delta_since`/`apply_delta` in `semantic_runtime.rs`; codegen emit/predicate/try_parse sites in `ast_based_generator.rs`); pinned the generator hooks, the 2 predicate strategies, the checkpoint/rollback discipline, the no-op gate, the MVP scope + verification matrix | DESIGN DONE (docs) — `.3`–`.5` turnkey |
| `2026-06-08` | `.3` | regex DEFAULT cert-coverage seed sweep (seed 0/1/7/13 + count 500/seed 0,1) all `sample_parse_failures=0` (seed 1 was 1); determinism byte-identical; backrefs still generated; json/VHDL/SV byte-identical (`stimuli_cross_family_platform_gate` ✅); `regex_pcre2_compile_oracle_gate` byte-identical; self-hosting OK; `cargo test --lib` 620/0 (+2 locks); strict source clippy clean | **IMPLEMENT DONE** — closes `REGEX-PCRE2-FIDELITY.3.12`; generator-only, surface-neutral (no regen/version bump); MVP prunes the `count(K)==0` category, tight `$index≤count` bound deferred to `.4` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `.1` | `PGEN-STORE-AWARE-GEN-0001` | tree created + scoping + decision record + registered; trigger = `REGEX-PCRE2-FIDELITY.3.12` |
| `.2` | `PGEN-STORE-AWARE-GEN-0002` | tool-backed design ([`STORE-AWARE-GEN-design.md`](STORE-AWARE-GEN-design.md)) — generator hooks, 2 predicate strategies, checkpoint/rollback discipline, no-op gate, MVP scope, verification matrix; `.3`–`.5` turnkey |
| `.3` | `PGEN-STORE-AWARE-GEN-0003` | `fact_count_at_least`-aware generation MVP (generator-only): `gen_semantic_state` + no-op gate + emit hook + `count(K)==0` necessary-condition prune + `generate_or`/`generate_quantified` checkpoint/rollback + per-sample reset; closes `REGEX-PCRE2-FIDELITY.3.12` (cert-cov seed sweep all 0); surface-neutral, no regen/version bump |
