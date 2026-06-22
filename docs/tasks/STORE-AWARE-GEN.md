# STORE-AWARE-GEN: semantic-store-aware (context-valid) stimuli generation — the generation-side dual of the parser's `@predicate`

## Metadata

- Tree ID: `STORE-AWARE-GEN`
- Status: `active` (`.1` SCOPING + `.2` DESIGN + `.3` IMPLEMENT done [`fact_count_at_least`-aware MVP closed `REGEX-PCRE2-FIDELITY.3.12`]; `.4` SCOPING done [tools-first: the composable-predicate generalization is SystemVerilog-only — the most-tuned generation surface — and the regex tight-bound is moot/unreachable]. Frontier: `.4a`/`.4b` are MEASURED SV efforts, recommended to be evidence-gated by `GRAMMAR-WELLFORMED` Phase H first)
- Family / slice-id prefix: `PGEN-STORE-AWARE-GEN-<NNNN>`
- Roadmap lane: stimuli-generator quality / parser sign-off pillars **C** (fidelity) + **D** (coverage) —
  the generator must emit only samples that satisfy the SAME semantic predicates the parser enforces
- Created: `2026-06-08`
- Owner: repo-local workflow
- Director directive: 2026-06-08 — "plan this new generator capability (semantic-store-aware generation
  honoring `fact_count_at_least`) for the stimuli generation at some point … task-tree track it so that
  we do not forget."
- **SHARPENED directive: 2026-06-08 (same session, emphatic)** — triggered by the director observing the
  svpp generator "is outputting garbage" (~48% comment chars/sample) and "is not properly steered by the
  EBNF": **"the generator needs FULL support for the semantic fact store … it needs to output text based
  on CONTEXT == semantic fact store."** This ELEVATES the tree from the single-predicate
  `fact_count_at_least` MVP to FULL store-aware, **context-aware** generation across every grammar (all
  predicate primitives + complete `@emit_fact` emission + scope-awareness) — the store IS the context that
  stops garbage. See the [[project_store_aware_generation]] AMENDMENT (incl. the honest scope: the
  comment-density knob + the svpp `condition_text` lexical-context residual are ADJACENT facets, not store
  facts).
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
  - **`.4` SCOPING (`PGEN-STORE-AWARE-GEN-0004`, 2026-06-08, tools-first).** Two decisive findings
    re-frame `.4`:
    - **Blast radius:** `resolve_path` is used by **no** grammar; `has_fact` / `lacks_fact` /
      `fact_attribute_equals` are used **only by `systemverilog.ebnf`** (41 `@predicate` sites — regex's
      one `has_fact` mention is a COMMENT; its only generative predicate is `fact_count_at_least`, done in
      `.3`). So generalizing these predicates touches the **most heavily-tuned generation surface in the
      repo** (the entire `SV-EXH-PROOF` campaign): reach plans, witness/diverse passes, weighting,
      target replay. It is NOT a small or low-risk slice — it MUST be measured against the SV machinery
      (`sv_stimuli_quality_gate`, the `SV-EXH-PROOF` gates, external corpus 14/14, `stimuli_cross_family_platform_gate`).
    - **Two SV honoring tiers:** (i) the SOUND PRUNE generalizes the `.3` mechanism directly —
      `has_fact(K, $ref)` is unsatisfiable when `count(K) == 0` (the same necessary condition), so a rule
      requiring a declared type/class is pruned when none is declared; `lacks_fact(K, $ref)` is trivially
      satisfiable when `count(K) == 0` (rarely prunes). The prune needs NO new mechanism. (ii) the
      COMPLETE honoring needs **value selection** — generate a `$ref` that MATCHES an existing fact (e.g.
      a reference to a declared type), a genuinely new generator mechanism (select from the store), not
      just a satisfiability prune. Tier (i) is the measured first step; tier (ii) is the larger build.
    - **The `.3` tight `$index ≤ count` bound is NOT applicable** (closed-as-moot, tools-backed): the
      multi-digit `numeric_backreference` construct (value ≥ 10, needing ≥ 10 capture groups) is
      effectively **never generated** in random regex — 0 of 2537 generated samples contained any
      `\NN` (N ≥ 10), and the cert-cov re-parse is 0 across a wide seed sweep (11 seeds × count 300).
      So the count-1-9-over-value case has NO reachable instance; per the no-speculative-fix discipline
      ([[feedback_no_codebase_change_without_tool_backed_facts]]) the tight bound is NOT implemented.
    - **Re-scope:** `.4a` = SV predicate-honoring **prune** (tier i) — a measured slice gated behind the
      SV machinery (verify no-op-or-improvement on `sv_stimuli_quality_gate` + corpus 14/14 BEFORE
      keeping). `.4b` = SV value-selection (tier ii) — the larger build. Both are deliberate, measured SV
      efforts, not session-end rushes. **Recommendation:** run `GRAMMAR-WELLFORMED` Phase H FIRST to wire
      cert-coverage for SV and SEE whether SV generation actually has `@predicate`-violating residuals —
      that evidence decides whether `.4a`/`.4b` are needed and bounds their scope (don't change the tuned
      SV generator speculatively).
  - **⚠️ FRAMING CORRECTION (director 2026-06-08, `PGEN-STORE-AWARE-GEN-0004` follow-up).** The above
    `.4` scoping's "SystemVerilog-only / high-risk" framing was WRONG per the standing doctrine
    [[feedback_features_parser_agnostic_enable_all_parsers]]: the semantic-annotation predicates
    (`has_fact`/`lacks_fact`/`fact_attribute_equals`/`fact_count_at_least`) are **parser-AGNOSTIC engine
    features**; SV being their only current USER does NOT make them SV-specific, and the generation-side
    honouring must likewise be **general and enabled for ALL parsers** (capability-gated on the predicate's
    presence — exactly as `.3` already does — never grammar-name-gated). So `.4a`/`.4b` are the CORRECT
    parser-agnostic direction (the right thing to do), NOT an "SV-specific risk"; the only real caveat is
    normal regression verification (does ANY grammar regress?), with the SV gates used because SV is the
    current user — not because SV is special. **Proven-for-SV ⇒ safe-for-all.** Phase H still runs first,
    as the evidence gate (what residuals exist per grammar), not as a risk-avoidance excuse.
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
| `2026-06-08` | `.4` SCOPING | predicate blast-radius grep (`has_fact`/`lacks_fact`/`fact_attribute_equals` = SV-only, 41 sites; `resolve_path` = none; regex `has_fact` = a comment); over-value reachability (0 of 2537 generated samples have a `\NN` N≥10; cert-cov 0 across 11 seeds × count 300) | SCOPING DONE (docs) — generalization is SV-only/high-risk (measured effort); regex tight-bound moot (unreachable); recommend Phase H first to gate `.4a`/`.4b` with evidence |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `.1` | `PGEN-STORE-AWARE-GEN-0001` | tree created + scoping + decision record + registered; trigger = `REGEX-PCRE2-FIDELITY.3.12` |
| `.2` | `PGEN-STORE-AWARE-GEN-0002` | tool-backed design ([`STORE-AWARE-GEN-design.md`](STORE-AWARE-GEN-design.md)) — generator hooks, 2 predicate strategies, checkpoint/rollback discipline, no-op gate, MVP scope, verification matrix; `.3`–`.5` turnkey |
| `.3` | `PGEN-STORE-AWARE-GEN-0003` | `fact_count_at_least`-aware generation MVP (generator-only): `gen_semantic_state` + no-op gate + emit hook + `count(K)==0` necessary-condition prune + `generate_or`/`generate_quantified` checkpoint/rollback + per-sample reset; closes `REGEX-PCRE2-FIDELITY.3.12` (cert-cov seed sweep all 0); surface-neutral, no regen/version bump |
| `.4` SCOPING | `PGEN-STORE-AWARE-GEN-0004` | tools-first re-frame: composable-predicate generalization is SV-only (41 sites, most-tuned surface → measured effort); regex tight-bound moot (multi-digit backref unreachable, 0/2537); re-scoped to `.4a` (SV prune, measured) / `.4b` (SV value-selection); recommend Phase H first as evidence gate; docs-only |
| `.4b.1` | `PGEN-STORE-AWARE-GEN-<open>` | **OPEN — frontier. Phase-H evidence gate now SATISFIED (see leaf below).** |

## `.4b.1` — IMPLEMENT store-aware (name-coordinated) witness generation — the cert-coverage `UNKNOWN`-tail fix

- **Status:** `IN PROGRESS` (frontier). Owns the engine code change; nothing committed yet.
- **Phase-H evidence gate — SATISFIED (tools-first, 2026-06-22).** The `.4` scoping recommended running
  `GRAMMAR-WELLFORMED` Phase H first to SEE whether SV generation has real `@predicate`-violating
  residuals before building tier (ii). It does, and the debug toolbox proves it decisively:
  - `PGEN_CERT_COVERAGE_DEBUG_PROBES=1` shows the plannable-witness pass forces store-gated rules with
    samples like `localparam \foo \foo ;` — using `\foo` as a type WITHOUT a declaration that emits the
    required fact (**640 of the forced samples `parsed=false`**, the dominant failure mode).
  - The scoped semantic trace names the exact rejection:
    `🚫 Rule 'known_unscoped_covergroup_type_identifier' rejected by post predicate
    'fact_attribute_equals [type_name, "\foo", declaration_family, covergroup]' ↪ NEGATIVE: ... none
    matched name "\foo"`. Same for `checked_type_identifier` (`has_fact(type_name,…)`) and the whole
    `known_unscoped_*` / `class_scoped_*` / `constraint`/`extern` store-gated family (~26 of the SV
    residual 55). So the SV `UNKNOWN` tail is dominated by tier-(ii) value-selection, exactly `.4b`.
- **Root cause (WHY+WHERE, proven):** the plannable witness pass (`stimuli_generator.rs`) forces a
  store-gated rule's USE-SITE but never generates the fact-emitting DECLARATION its `@predicate` requires,
  so `has_fact`/`fact_attribute_equals` is false → the gated branch rejects → the forced sample does not
  re-parse → no witness. The existing **C2.2 semantic-prelude** (`compute_reach_prelude`, ~`:2812`) already
  solves the analogous problem for `fact_count_at_least` (regex) by hosting a fact PRODUCER in a quantifier
  site; it does NOT handle the NAME-matching gates (`has_fact`/`fact_attribute_equals`), where the prelude
  must declare the *same name* the use-site emits.
- **The fix (ONE clean, SOTA, parser-agnostic mechanism — NOT a per-rule `@sample` hack):** extend the
  C2.2 prelude from count-gates to **name-matching gates**: when a witness target is gated by
  `has_fact(K,$ref)` / `fact_attribute_equals(K,$ref,attr,val)`, find a producer rule whose `@emit_fact`
  emits kind `K` (matching `attr=val`), synthesize a declaration prelude, and **coordinate the name** so
  the use-site `$ref` equals the producer's emitted name (declare-then-use). Reuses the existing
  `gen_emit_facts` / `reach_gate_kinds` / `ReachPrelude` machinery and the `semantic_runtime` evaluator
  (one evaluator, two drivers); capability-gated on the predicate's presence (no-op / byte-identical for
  grammars without these gates), per [[feedback_features_parser_agnostic_enable_all_parsers]]. Director
  pre-authorized general semantic-annotation additions if the minimal vehicle needs one
  ([[feedback_pinpoint_real_blocker_not_menu]]); first cut targets the engine prelude with no new
  annotation. ⚠️ Avoid the fragile name-coupling hack the earlier `.4.2.1.2.2.3` note warned against —
  the coordination must be robust (captured producer name reused at the use-site, verified by the
  parser re-check that already guards every witness).
- **Acceptance:** SV cert `UNKNOWN` drops by the store-gated cohort, deterministic seeds 0/7/42, `spf=0`;
  the 6 fully-certified grammars byte-identical (`stimuli_cross_family_platform_gate` + each stimuli gate
  green); SV external corpus 14/14; clippy source-clean. Decisive A/B + GLOBAL cert before any commit.
- **Diagnose/verify with the toolbox** (`docs/book/src/diagnosing-unknowns.md`): `DUMP_ALL` →
  `DEBUG_PROBES` → scoped semantic trace, per [[feedback_systematically_use_debug_toolbox]].
