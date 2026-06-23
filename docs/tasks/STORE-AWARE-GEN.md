# STORE-AWARE-GEN: semantic-store-aware (context-valid) stimuli generation — the generation-side dual of the parser's `@predicate`

## Metadata

- Tree ID: `STORE-AWARE-GEN`
- Status: `active` (`.1` SCOPING + `.2` DESIGN + `.3` IMPLEMENT done [`fact_count_at_least`-aware MVP closed `REGEX-PCRE2-FIDELITY.3.12`]; `.4` SCOPING done [tools-first: the composable-predicate generalization is parser-agnostic, capability-gated, current sole user SystemVerilog]; **`.4b.1` DESIGN done** [`PGEN-STORE-AWARE-GEN-0005`, 2026-06-22 — verified baseline `UNKNOWN=56`, classified the 56, pinned the name-coordinated declare-then-use prelude mechanism]; **`.4b.2` IMPLEMENT done** [`PGEN-STORE-AWARE-GEN-0006`, 2026-06-22 — landed the generator-only name-coordinated prelude; SV cert `UNKNOWN 56 → 46`, +10 store-gate cohort rules witnessed, zero newly-UNKNOWN, deterministic seeds 0/7/42]; **`.4b.3` DESIGN done** [`PGEN-STORE-AWARE-GEN-0007`, 2026-06-22 — verified the `UNKNOWN=46` baseline, classified all 46, A/B-proved the `block_type`/`data_type` asymmetry root cause (the name-prelude's gated-rule search inspects only hops+target, never the target's mandatory sub-rules, so a non-gated carrier of the inner-gated `checked_type_identifier` never gets a prelude), decomposed the deeper residual into 3 implement sub-cohorts]; **`.4b.4` IMPLEMENT done** [`PGEN-STORE-AWARE-GEN-0008`, 2026-06-22 — landed the generator-only mandatory-first gated-rule descent; SV cert `UNKNOWN 46 → 43`, +3 block-scoped carriers witnessed (`known_unscoped_block_type_identifier`, `known_unscoped_block_covergroup_identifier`, + bonus `provisional_unscoped_block_class_type`), zero newly-UNKNOWN, deterministic seeds 0/7/42]; **`.4b.5` DESIGN done** [`PGEN-STORE-AWARE-GEN-0010`, 2026-06-23 — re-verified `UNKNOWN=43` baseline; TOOLBOX 4.5 NOT-depth check (`--max-depth 32`) ruled OUT the depth hypothesis (3B set unchanged, +spf noise); the `name-prelude spec` debug trace + `furthest_position` + producer-render A/B + scoped predicate trace decomposed sub-cohort 3B into THREE mechanistically-distinct sub-blockers — **3B-ii** producer-selection (the prelude IS built but picks the alphabetically-first family producer `declared_class_alias_identifier`, a non-bootstrapping typedef-ALIAS whose own mandatory source-type slot is undeclared → the prelude declaration itself fails to parse @pos 11), **3B-i** gate-not-mandatory-first (`extern_constraint_declaration*`/`constraint_set` — gated `class_scope` buried behind leading `kw_constraint`, so `name_gate_via_mandatory_prefix` builds NO prelude), **3B-iii** target-own incompleteness (`property_qualifier` — forced `rand <type> ;` omits the variable name); pinned the self-bootstrapping-producer-selection fix as the next implement]; **`.4b.6` IMPLEMENT done** [`PGEN-STORE-AWARE-GEN-0011`, 2026-06-23 — landed the generator-only two-pass self-bootstrapping-host-branch producer selection; SV cert `UNKNOWN 43 → 41`, +2 class-scope rules witnessed (`known_unscoped_class_scope_class_identifier`, `known_unscoped_class_scoped_call_class_identifier`), zero newly-UNKNOWN, deterministic seeds 0/7/42; tools-first the `.4b.5` root cause was REFINED — the non-bootstrapping-ness lives in the producer's HOST BRANCH (the `class_type`-gated sibling of the typedef-alias in `type_declaration`), not the producer body, so the fix prefers a producer whose forced reach path renders no unsatisfiable same-store gate]; **`.4b.7` IMPLEMENT done** [`PGEN-STORE-AWARE-GEN-0012`, 2026-06-23 — landed the generator-only `.4b.7` structural gate-discovery leg in `name_gate_via_mandatory_prefix`: scan a sequence STOPPING at the first store-gated rendered position + descend an escape-free `Or`, gated on `mandatory_reach_gate` against an empty store; SV cert `UNKNOWN 41 → 38` (+3: `extern_constraint_declaration_sv_2017`, `extern_constraint_declaration`, bonus `class_scoped_tf_call`), zero newly-UNKNOWN, deterministic seeds 0/7/42; a first-cut all-element broadening regressed 6 rules (caught by the strict-subset gate, `41 → 44`) and was root-caused tools-first to a deep over-reach behind a self-satisfying producer + an ungated `data_type` escape, then refined; `constraint_set` proven a DISTINCT off-path-sibling discovery → folded into `.4b.9`]; **`.4b.8` DESIGN done** [`PGEN-STORE-AWARE-GEN-0013`, 2026-06-23, PURE-DOCS — tools-first re-root-caused `property_qualifier` from "missing var name" to a witness-generator NAME COLLISION (the reach-context class is named with the canonical `\foo`, the free `variable_identifier` reuses `\foo`, and PEG's `data_type_or_implicit` greedily parses the now-known-type `\foo` as the TYPE, dropping the mandatory var-list; A/B `class\foo ;rand\foo \bar ;endclass` PASS); fix direction = store-aware free-name DIVERSITY, a higher-blast-radius core-generator change ⇒ IMPLEMENT in a fresh focused session]. **`.4b.8` IMPLEMENT done** [`PGEN-STORE-AWARE-GEN-0014`, 2026-06-23, GENERATOR-ONLY — landed store-aware collide-aware free-name diversity (`free_name_collides_gate_kind` + `diversify_free_name_avoiding_gate_collision`, hooked at the rule-level `@sample` literal-hint override in `generate_rule`); SV cert `UNKNOWN 38 → 37`, +1 witnessed (`property_qualifier`, `class\foo ;rand\foo_0 ;endclass`), zero newly-UNKNOWN, deterministic seeds 0/7/42; the producer's own declaration is naturally exempt (fact emitted after render), so `store_name_for_gate` read-back stays consistent]. **Frontier: `.4b.9` IMPLEMENT** — off-path-sibling prelude-arming (`constraint_set`) + the 3C/misc reach-routing residual; SVA infix parse-bug + `no_path` out of scope))
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
| `2026-06-22` | `.4b.1` DESIGN | re-verified SV baseline `UNKNOWN=56` byte-identical seeds 0/7/42 (regen-lockstep build); fresh `DUMP_ALL`+`DEBUG_PROBES` classified the 56 (19 `no_path` + ~7 SVA infix-operator PARSE bug [out of scope] + ~25 store-gate declare-then-use [cohort] + misc); confirmed producer⟷consumer map + the two `$ref` shapes (`$body`/`$1.body` vs `$head.body`); pinned the name-coordinated declare-then-use prelude mechanism + no-op gate + verification matrix | DESIGN DONE (PURE-DOCS) — engine implement split to `.4b.2`; scope excludes the SVA parse-bug + no_path; expected `UNKNOWN 56 → ~31–37` earned by `.4b.2` |
| `2026-06-22` | `.4b.2` IMPLEMENT | decisive A/B on regen-lockstep build: SV cert `UNKNOWN 56 → 46`, witness `1232 → 1242` (+10 store-gate cohort witnessed), `spf=0`, `proof_reverify_failures=0`, deterministic seeds 0/7/42; new 46 a strict SUBSET of baseline 56 (script-verified ZERO newly-UNKNOWN); in-session regression `declared_forward_class_identifier` root-caused via `DEBUG_PROBES` (`typedef\foo \foo ;typedef class\foo ;` self-emit re-declaration) + fixed (exclude self-emitting producers); 6 fully-certified grammars `fully_certified=true` (regex byte-identical seeds 0/7/42); `cargo test --lib` 729/0 (+3 locks); `ast_shape_contract_gate` 18/18; `clippy_on_rust_change` source-clean (189 `eq_op` errors all in `generated/*_parser.rs`, pre-existing) | **IMPLEMENT DONE** — generator-only (no grammar/regen/release/schema); closes the directly-reachable store-gate cohort; deeper class-scope/class-member residual → `.4b.3` |
| `2026-06-22` | `.4b.4` IMPLEMENT | decisive A/B on the regen-lockstep build: SV cert `UNKNOWN 46 → 43`, witness `1242 → 1245` (+3 block-scoped carriers witnessed: `known_unscoped_block_type_identifier`, `known_unscoped_block_covergroup_identifier`, bonus `provisional_unscoped_block_class_type`), `spf=0`, `proof_reverify_failures=0`, deterministic seeds 0/7/42; new 43 a script-verified strict SUBSET of baseline 46 (ZERO newly-UNKNOWN); regex (other `store_aware`) byte-identical `198/198 fully_certified` seeds 0/7/42; json/vhdl/svpp/rtl_frontend `fully_certified=true`; `cargo test --lib` 731/0 (+2 locks: mandatory-leading-rule-ref descent, name-gate inheritance + direct short-circuit); `clippy_on_rust_change` source-clean (zero findings in the changed code; the 188 generated-`eq_op` errors pre-existing) | **IMPLEMENT DONE** — generator-only (no grammar/parser/regen/release/schema); closes block-scoped sub-cohort 3A; class-member/class-scope (3B) → `.4b.5`, reach-routing (3C) → `.4b.6` |
| `2026-06-22` | `.4b.3` DESIGN | re-verified `UNKNOWN=46` baseline (regen-lockstep build, seed 0; total=1289 proof=1 witness=1242 spf=0); `DUMP_ALL`+`DEBUG_PROBES` classified all 46 (19 `no_path` + ~7 SVA infix PARSE-bug [out of scope] + the store-gate/reach residual); **`PGEN_REACH_PATH_DUMP` A/B proved the `block_type`/`data_type` asymmetry root cause** — identical bodies (`:= checked_type_identifier packed_dimension*`), gate lives on the inner `checked_type_identifier` (`@predicate has_fact(type_name,$body)`, `:5193`), NOT the carriers; `compute_name_prelude`'s gated-rule search (`stimuli_generator.rs:2972-2976`) inspects only hops+target, never the target's mandatory sub-rules, so a non-gated carrier never gets a prelude (`data_type` only witnessed opportunistically via `checked_type_identifier`'s own top-level witness; the 2-quantifier-deep block carrier was never reached); decomposed the deeper residual into 3A (block-scoped carrier), 3B (class-member/class-scope), 3C/misc (reach-routing) | **DESIGN DONE (PURE-DOCS)** — engine implement split to `.4b.4` (3A, cleanest); `.4b.5` (3B), `.4b.6` (3C/misc) scoped; SVA parse-bug + `no_path` remain out of scope |
| `2026-06-23` | `.4b.5` DESIGN | re-verified `UNKNOWN=43` baseline (regen-lockstep build, seed 0; total=1289 proof=1 witness=1245 spf=0); **TOOLBOX 4.5 NOT-depth check** (`--max-depth 32` → `UNKNOWN=44`, spf=9, NO 3B rule leaves the set) ruled OUT the depth hypothesis; the `name-prelude spec` debug trace proved a prelude **IS** built for `known_unscoped_class_scope_class_identifier` (producer `declared_class_alias_identifier`) ⇒ not a discovery gap; `furthest_position=11` + scoped predicate trace (`checked_type_identifier` `has_fact[type_name,"\foo"]` NEGATIVE) + producer-render A/B (`typedef\foo \foo ;` REJECT vs `typedef class\foo ;` / `class\foo ;endclass` PASS) pinned the root cause = **non-bootstrapping producer chosen alphabetically**; decomposed 3B into 3B-ii (producer-selection, `.4b.6`), 3B-i (gate-not-mandatory-first, `.4b.7`), 3B-iii (target-own incompleteness, `.4b.8`); 3C → `.4b.9` | **DESIGN DONE (PURE-DOCS)** — engine implement split to `.4b.6`+ ; the self-bootstrapping-producer-selection fix (3B-ii) pinned as the cleanest next implement; SVA parse-bug + `no_path` remain out of scope |
| `2026-06-23` | `.4b.6` IMPLEMENT | decisive A/B on the regen-lockstep build: SV cert `UNKNOWN 43 → 41`, witness `1245 → 1247` (+2 class-scope rules witnessed: `known_unscoped_class_scope_class_identifier`, `known_unscoped_class_scoped_call_class_identifier`), `spf=0`, `proof_reverify_failures=0`, deterministic seeds 0/7/42; new 41 a script-verified strict SUBSET of baseline 43 (ZERO newly-UNKNOWN — the only two rules that left are the addressed pair); tools-first root-cause REFINEMENT (`.4b.5` had pinned "producer selection", but `parseability_probe` proved both family=class alternatives share the byte-identical body `type_identifier` — the non-bootstrapping-ness is in the producer's HOST BRANCH, the `class_type`-gated sibling of `declared_class_alias_identifier` in `type_declaration`, not the producer body); 6 fully-certified grammars green (regex byte-identical `fully_certified=true` seeds 0/7/42; json/vhdl/svpp/rtl_frontend `fully_certified=true`; rtl_const_expr `store_aware_gen=false` ⇒ path off ⇒ inert); `cargo test --lib --features "generated_parsers ebnf_dual_run"` **764/0** (+1 lock: `store_aware_gen_prefers_self_bootstrapping_host_branch`); `clippy_on_rust_change` source-clean (zero findings in the changed code) | **IMPLEMENT DONE** — generator-only (no grammar/parser/regen/release/schema); closes 3B-ii; gate-not-mandatory-first (3B-i) → `.4b.7`, target-own incompleteness (3B-iii) → `.4b.8`, reach-routing (3C) → `.4b.9` |
| `2026-06-23` | `.4b.7` IMPLEMENT | decisive A/B on the regen-lockstep build: SV cert `UNKNOWN 41 → 38`, witness `1247 → 1250` (+3 witnessed: `extern_constraint_declaration_sv_2017`, `extern_constraint_declaration`, bonus `class_scoped_tf_call`), `spf=0`, `proof_reverify_failures=0`, deterministic seeds 0/7/42; new 38 a script-verified strict SUBSET of baseline 41 (ZERO newly-UNKNOWN — `comm -13` empty); tools-first WHY+WHERE: `DEBUG_PROBES` forced sample `constraint\foo ::\foo {}` `parsed=false` with NO declare-then-use prefix (no prelude armed) + `--parse` A/B (`constraint\foo ::\foo {}` REJECT `furthest_position=21` vs `typedef class\foo ;constraint\foo ::\foo {}` PASS) — `name_gate_via_mandatory_prefix` could neither skip the leading `(kw_static)? kw_constraint` prefix nor descend the `class_scope_type` Or to reach `known_unscoped_class_scope_class_identifier`; first-cut broadening regressed 6 rules (`UNKNOWN 41 → 44`, caught by the strict-subset gate), root-caused tools-first to a deep over-reach (`param_assignment_sv_2017`'s `constant_param_expression` `type_name` gate behind the self-satisfying producer `declared_parameter_identifier`, armed an invalid double-`#()` prelude) + an ungated escape (`data_type` builtin); refined to STOP at the first store-gated rendered position (`node_render_store_gated` = `.4b.6` `mandatory_node_gated`) + a `mandatory_reach_gate` guard ⇒ clean `41 → 38`; 6 fully-certified grammars green (regex `fully_certified=true` seeds 0/7/42 — name-path inert since `gen_name_gate` empty; json/vhdl/svpp/rtl_frontend `fully_certified=true`; rtl_const_expr `store_aware_gen=false` ⇒ inert); `cargo test --lib --features "generated_parsers ebnf_dual_run"` **765/0** (+1 lock: `store_aware_gen_name_gate_discovers_gate_behind_leading_prefix_and_or`); `clippy_on_rust_change` source-clean (zero `stimuli_generator.rs` findings; the generated-`eq_op` errors pre-existing) | **IMPLEMENT DONE** — generator-only (no grammar/parser/regen/release/schema); closes 3B-i for the `extern_constraint_declaration*` cohort; `constraint_set` proven a DISTINCT off-path-mandatory-sibling discovery (its `class_scope` gate is a reach-path sibling, NOT on its own mandatory prefix) → folded into `.4b.9`; target-own incompleteness (3B-iii) → `.4b.8` |
| `2026-06-23` | `.4b.8` DESIGN | re-verified `UNKNOWN=38` baseline (post-`.4b.7`, seed 0; total=1289 proof=1 witness=1250 spf=0); tools-first re-root-caused `property_qualifier`: `random_qualifier` (`:4228`) + `variable_identifier` (`:5419`) are both UNGATED ⇒ not a store-gate member; the forced `class\foo ;rand\foo ;endclass` REJECTs `furthest_position=19` because the witness names the reach-context class `\foo` (canonical id, emits `type_name`) AND reuses `\foo` as the free variable, so PEG's `data_type_or_implicit := data_type | implicit_data_type` (`:1730`) greedily parses `\foo` as the TYPE, dropping the mandatory `list_of_variable_decl_assignments`; A/B `class\foo ;rand\foo \bar ;endclass` PASS pinned it. Fix direction = store-aware free-name DIVERSITY (collide-aware); BLAST-RADIUS noted (feeds many witness samples) ⇒ IMPLEMENT in a fresh focused session | **DESIGN DONE (PURE-DOCS)** — reframes 3B-iii from "missing var name" to a name-collision; IMPLEMENT split to a new `.4b.8` code leaf |
| `2026-06-23` | `.4b.8` IMPLEMENT | decisive A/B on the regen-lockstep build: SV cert `UNKNOWN 38 → 37`, witness `1250 → 1251` (+1 witnessed: `property_qualifier`, now `parsed=true witnessed_target=true`, sample `class\foo ;rand\foo_0 ;endclass`), `spf=0`, `proof_reverify_failures=0`, deterministic seeds 0/7/42; new 37 a script-verified strict SUBSET of baseline 38 (`comm -13` empty — only `property_qualifier` left); landed two generator-only helpers (`free_name_collides_gate_kind` + `diversify_free_name_avoiding_gate_collision`) hooked at the rule-level `@sample` literal-hint override in `generate_rule` — collide-aware free-name diversity keyed on `gen_name_gate` kinds, scoped to the witness pass, deterministic/RNG-neutral; producer's own declaration naturally exempt (fact emitted after render); 6 fully-certified grammars green (regex `fully_certified=true 198/198` seeds 0/7/42 — name-path inert; json/vhdl/svpp/rtl_frontend `fully_certified=true`; rtl_const_expr `store_aware_gen=false` ⇒ inert); `cargo test --lib --features "generated_parsers ebnf_dual_run"` **766/0** (+1 lock `store_aware_gen_diversifies_free_name_colliding_with_type_name_fact`); `clippy_on_rust_change` source-clean | **IMPLEMENT DONE** — generator-only (no grammar/parser/regen/release/schema); closes 3B-iii (`property_qualifier`); off-path-sibling (`constraint_set`) + 3C reach-routing → `.4b.9` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `.1` | `PGEN-STORE-AWARE-GEN-0001` | tree created + scoping + decision record + registered; trigger = `REGEX-PCRE2-FIDELITY.3.12` |
| `.2` | `PGEN-STORE-AWARE-GEN-0002` | tool-backed design ([`STORE-AWARE-GEN-design.md`](STORE-AWARE-GEN-design.md)) — generator hooks, 2 predicate strategies, checkpoint/rollback discipline, no-op gate, MVP scope, verification matrix; `.3`–`.5` turnkey |
| `.3` | `PGEN-STORE-AWARE-GEN-0003` | `fact_count_at_least`-aware generation MVP (generator-only): `gen_semantic_state` + no-op gate + emit hook + `count(K)==0` necessary-condition prune + `generate_or`/`generate_quantified` checkpoint/rollback + per-sample reset; closes `REGEX-PCRE2-FIDELITY.3.12` (cert-cov seed sweep all 0); surface-neutral, no regen/version bump |
| `.4` SCOPING | `PGEN-STORE-AWARE-GEN-0004` | tools-first re-frame: composable-predicate generalization is SV-only (41 sites, most-tuned surface → measured effort); regex tight-bound moot (multi-digit backref unreachable, 0/2537); re-scoped to `.4a` (SV prune, measured) / `.4b` (SV value-selection); recommend Phase H first as evidence gate; docs-only |
| `.4b.1` | `PGEN-STORE-AWARE-GEN-0005` | **DONE (DESIGN).** Tools-first scope + mechanism pinned on the verified `UNKNOWN=56` baseline; engine implement split to `.4b.2`. PURE-DOCS. |
| `.4b.2` | `PGEN-STORE-AWARE-GEN-0006` | **DONE (IMPLEMENT).** Generator-only name-coordinated declare-then-use witness prelude per the `.4b.1` design. SV cert `UNKNOWN 56 → 46` (+10 store-gate cohort witnessed), zero newly-UNKNOWN, deterministic seeds 0/7/42; 6 fully-certified grammars green; 729/0 tests (+3 locks); source clippy-clean. No grammar/regen/release/schema bump. |
| `.4b.3` | `PGEN-STORE-AWARE-GEN-0007` | **DONE (DESIGN).** Tools-first decomposition of the `UNKNOWN=46` residual into 3 implement sub-cohorts + the A/B-proved `block_type`/`data_type` asymmetry root cause (gated-rule search ignores the target's mandatory sub-rules). PURE-DOCS. |
| `.4b.4` | `PGEN-STORE-AWARE-GEN-0008` | **DONE (IMPLEMENT).** Generator-only mandatory-first gated-rule descent (`compute_name_prelude` now arms on a carrier's inner gated rule). SV cert `UNKNOWN 46 → 43` (+3 block-scoped carriers witnessed, incl. bonus `provisional_unscoped_block_class_type`), zero newly-UNKNOWN, deterministic seeds 0/7/42; 6 fully-certified grammars green; 731/0 tests (+2 locks); source clippy-clean. No grammar/parser/regen/release/schema bump. |
| `.4b.5` | `PGEN-STORE-AWARE-GEN-0010` | **DONE (DESIGN).** Tools-first decomposition of sub-cohort 3B into THREE mechanistically-distinct sub-blockers (3B-ii producer-selection, 3B-i gate-not-mandatory-first, 3B-iii target-own incompleteness), each with a tool-proven WHY+WHERE; NOT-depth check ruled out the depth hypothesis; pinned the self-bootstrapping-producer-selection fix (3B-ii) as the next implement. PURE-DOCS. |
| `.4b.6` | `PGEN-STORE-AWARE-GEN-0011` | **DONE (IMPLEMENT).** Generator-only two-pass self-bootstrapping-HOST-BRANCH producer selection: `compute_name_prelude` now prefers a producer whose forced reach path renders no unsatisfiable same-store gate (`reach_path_renders_unsatisfiable_gate`), falling back to the pre-4b.6 first-reachable behavior. The `.4b.5` "producer-body" framing was REFINED tools-first (both family=class alternatives share body `type_identifier`; the gate is the `class_type` mandatory SIBLING of the typedef-alias in `type_declaration`). SV cert `UNKNOWN 43 → 41` (+2 class-scope rules), zero newly-UNKNOWN, deterministic seeds 0/7/42; 6 fully-certified grammars green; 764/0 tests (+1 lock); source clippy-clean. No grammar/parser/regen/release/schema bump. |
| `.4b.7` | `PGEN-STORE-AWARE-GEN-0012` | **DONE (IMPLEMENT).** Generator-only structural gate-discovery past a leading terminal/optional prefix + into an ordered choice (`name_gate_via_mandatory_prefix` now has a `.4b.7` leg that scans a sequence, STOPS at the first store-gated rendered position, and descends an escape-free `Or`), gated on `mandatory_reach_gate` (unavoidable store-gate only). Arms a declare-then-use prelude for `extern_constraint_declaration(_sv_2017)` (gated `class_scope` behind leading `kw_constraint`). SV cert `UNKNOWN 41 → 38` (+3: the `extern_constraint_declaration*` cohort + bonus `class_scoped_tf_call`), zero newly-UNKNOWN, deterministic seeds 0/7/42; 6 fully-certified grammars green; 765/0 tests (+1 lock); source clippy-clean. No grammar/parser/regen/release/schema bump. `constraint_set` proven a DISTINCT off-path-sibling discovery → `.4b.9`. |
| `.4b.8` DESIGN | `PGEN-STORE-AWARE-GEN-0013` | **DONE (DESIGN, PURE-DOCS).** Tools-first re-root-cause of 3B-iii (`property_qualifier`): NOT "missing var name" but a witness-generator NAME COLLISION — the reach-context class is named with the canonical `\foo`, the free `variable_identifier` reuses `\foo`, and since `\foo` is a known type PEG's `data_type_or_implicit` parses it as the TYPE (first branch), dropping the mandatory var-list. A/B: `class\foo ;rand\foo ;endclass` REJECT @19 vs `class\foo ;rand\foo \bar ;endclass` PASS. Fix direction = store-aware free-name DIVERSITY (higher blast-radius core-generator change → IMPLEMENT best in a fresh focused session). |
| `.4b.8` IMPLEMENT | `PGEN-STORE-AWARE-GEN-0014` | **DONE (IMPLEMENT, GENERATOR-ONLY).** Landed store-aware collide-aware free-name diversity (`free_name_collides_gate_kind` + `diversify_free_name_avoiding_gate_collision`, hooked at the rule-level `@sample` literal-hint override in `generate_rule`). SV cert `UNKNOWN 38 → 37` (+1 witnessed: `property_qualifier`, `class\foo ;rand\foo_0 ;endclass`), witness `1250 → 1251`, `spf=0`, deterministic seeds 0/7/42, strict-subset (ZERO newly-UNKNOWN — only `property_qualifier` left). 6 fully-certified grammars green; `cargo test --lib` 766/0 (+1 lock); clippy source-clean. No grammar/parser/regen/release/schema; SV row `Mostly Done`. |
| `.4b.9` IMPLEMENT | `PGEN-STORE-AWARE-GEN-<open>` | **OPEN (frontier).** Off-path-sibling prelude-arming (`constraint_set`: its `class_scope` gate is a mandatory reach-path SIBLING, not on its own mandatory prefix) + the 3C/misc reach-routing `parsed=true witnessed=false` residual (`wildcard_escape_nettype_identifier`, `declared_class_alias_identifier`, `named_checker_port_connection*`, `repeat_range`, `with_covergroup_expression`, `context_member_method_call`, `union_modifier`). The SVA infix-operator PARSE bug (`kw_within`/`kw_until`/…) + `no_path` dead-rule candidates remain out of scope. |
| `.4b.9` | `PGEN-STORE-AWARE-GEN-<open>` | **OPEN.** IMPLEMENT sub-cohort 3C/misc (reach-routing `parsed=true witnessed=false`): `wildcard_escape_nettype_identifier`, `declared_class_alias_identifier`, `context_member_method_call`, `named_checker_port_connection(_sv_2017)`, `repeat_range`, `with_covergroup_expression`, `union_modifier`; **PLUS `constraint_set`** — off-path-mandatory-sibling discovery (proven by `.4b.7`: its `class_scope` gate is a mandatory SIBLING on the reach path, NOT on `constraint_set`'s own mandatory prefix, so neither the hop-scan nor `name_gate_via_mandatory_prefix` reaches it; needs an off-path-sibling prelude-arming pass akin to `.4b.6`'s `offpath_siblings_gated_along_path`). |

## `.4b.1` — DESIGN: store-aware (name-coordinated) witness generation — scope + mechanism (tools-first)

- **Status:** `DONE (DESIGN)` — `PGEN-STORE-AWARE-GEN-0005`, 2026-06-22, PURE-DOCS. Pins the exact mechanism and the precisely-scoped cohort on the re-verified `UNKNOWN=56` baseline; the engine implement is the new leaf **`.4b.2`** (per this tree's own `.2` DESIGN → `.3` IMPLEMENT precedent — design before implement on the most-tuned generation surface, no guessing).
- **Verified baseline (this slice, 2026-06-22).** A regen-lockstep DEBUG `ast_pipeline` (generated SV parser 09:50 + binary 09:51, both post the committed `-0111` grammar) re-measures `CERTIFICATE-COVERAGE … total=1289 proof=1 witness=1232 UNKNOWN=56 (sample_parse_failures=0, proof_reverify_failures=0)` **byte-identical at seeds 0/7/42** — matches the committed baseline. This is the `.4b.2` "before".
- **The 56-rule UNKNOWN classification (fresh `PGEN_CERT_COVERAGE_DUMP_ALL=1 PGEN_CERT_COVERAGE_DEBUG_PROBES=1`, seed 0).** The residual is NOT one cohort — `.4b` must be precisely scoped:
  - **19 `no_path`** dead-rule candidates (adjudicated NON-defects — `library_*`, `interface_class_*`, `sv_multi_entry_root`, `*parseable*`, profile keyword shells). Out of scope (linter territory, `H.12.6`).
  - **~6–7 SVA infix-operator PARSE bug** — `kw_within`/`kw_intersect`/`kw_s_until`/`kw_until`/`kw_until_with`/`kw_s_until_with` (+ likely `kw_constant`): forced samples like `sequence\foo ;0.44within 8_233.29_endsequence` `parsed=false` because the **parser rejects `a within b` at the operator** (the parked `GRAMMAR-WELLFORMED.H.12.5.8` SVA precedence-cascade). **A generator/witness change CANNOT fix a parser bug** → out of scope for `.4b`.
  - **~25 store-gate declare-then-use** (THE `.4b` COHORT): the `known_unscoped_*` type/class/covergroup/nettype/let/parameter identifier family (`known_unscoped_class_scope_class_identifier`, `…_interface_class_identifier`, `…_type_parameter_identifier`, `known_unscoped_base_class_type_parameter_identifier`, `known_unscoped_block_type_identifier`, `known_unscoped_data_type_identifier`, `known_unscoped_block_class_type`, `provisional_unscoped_block_class_type`, `known_unscoped_covergroup_type_identifier`, `known_unscoped_block_covergroup_identifier`, `known_unscoped_interface_class_type_identifier`, `known_unscoped_let_identifier`, `known_unscoped_parameter_identifier`), `checked_type_identifier`, `declared_class_alias_identifier`, `checked_nettype_identifier`, `wildcard_escape_nettype_identifier`, and the constraint cluster needing a declared class (`constraint_set`, `extern_constraint_declaration(_sv_2017)`). Forced samples (`localparam\foo \foo ;`, `class\foo ;…`, `constraint\foo ::\foo {}`, `typedef\foo \foo ;`) all `parsed=false` because `\foo` is used as a type/class WITHOUT a declaration emitting the gating fact.
  - a few **misc** (`named_checker_port_connection(_sv_2017)`, `property_qualifier`, `repeat_range`, `with_covergroup_expression`, the parked `context_member_method_call`, `class_scoped_*call*`) — adjacent; addressed opportunistically by the same mechanism where a producer exists, else deferred.
- **Producer ⟷ consumer map (static, grammar-grounded).** Producers are declaration rules `@emit_fact { kind: K, name: $body, declaration_family: V }`: `type_name` families `class` (`:965/:5113/:5117`), `interface_class` (`:969/:5121`), `covergroup` (`:1504`), `nettype` (`:3329`), `typedef` (`:5108/:5112`), `type_parameter` (`:5104`); plus `let_name` (`:2555`), `parameter_name` (`:3679`), `checker_name` (`:863`). Consumers are use-sites gated `@predicate has_fact(K,$ref)` / `fact_attribute_equals(K,$ref,declaration_family,V)`. Every cohort kind/family has ≥1 producer.
- **The `$ref` shape — the reviewable nuance (two cases).** Confirmed from rule defs: (i) **bare-identifier** (`$ref` = the rule's whole render): `checked_type_identifier := type_identifier -> {body:$1.body}`, `declared_class_alias_identifier`, `checked_nettype_identifier`, `known_unscoped_let_identifier := let_identifier`, `known_unscoped_parameter_identifier := parameter_identifier`, the bare `:= class_identifier` forms. (ii) **suffixed-head** (`$ref` = `$head.body`, the FIRST identifier of `head suffix*`): `known_unscoped_block_class_type := class_identifier (parameter_value_assignment)? (scope_resolution class_identifier …)*`. The forcing must target the `$ref`-path identifier, not always the whole render.

- **The mechanism (PINNED — ONE clean, parser-agnostic capability; the engine implement = `.4b.2`).** Extend the C2.2 semantic-prelude from `fact_count_at_least` count-gates to **name-matching gates**, as a single declare-then-use coordination:
  1. **Capability detection / no-op gate.** Add a precomputed `gen_name_gate: HashMap<String, NameGate>` (rule → `{ kind, family: Option<(attr,val)>, ref_path, negative }`), built by reusing `parse_semantic_runtime_directives` + the existing `reach_gate_kinds` walk (`stimuli_generator.rs:1454`). Turn the existing `store_aware_gen` flag on for name-gated grammars too. **EMPTY ⇒ entire path inert ⇒ byte-identical** for predicate-free grammars (json/ebnf/svpp/vhdl/rtl_*/regex) — capability-gated, NEVER grammar-name-gated (per [[feedback_features_parser_agnostic_enable_all_parsers]]).
  2. **Prelude build (sibling of `compute_reach_prelude`, `:2812`).** When a witness target (or an on-path rule) is **positively** name-gated, find a producer rule whose `@emit_fact` emits `kind=K` **with matching `declaration_family=V`** (reuse the `gen_emit_facts` filter at `:2828`, adding the attribute-match), locate an upstream quantifier site whose body reaches the producer (reuse `reach_hops`/`quantified_body_rule_name`), build a `sub_plan` steering body→producer, and set `iterations = 1` (one declaration). Reuses `ReachPrelude` + `ActiveReachPlan`.
  3. **Single-pass coordination (cleaner than the count case — NO cross-run capture).** The 1 producer iteration generates upstream FIRST and emits `(K, name=N, family=V)` into the live `gen_semantic_state` via the EXISTING emit hook (`gen_emit_facts_for_rule`, `:9872`). When the gated consumer then generates with an armed name-prelude, select a store name `N` matching `(K, family=V)` from `gen_semantic_state` (guaranteed present — the prelude just declared it) and **force the consumer's `$ref`-path identifier to render `N`**: for shape (i) reuse the `reach_prelude_replay_text` whole-render replay (`:2911`) with text `N`; for shape (ii) force only the head identifier leaf (the suffix generates freely). The producer-emitted name is reused verbatim at the use-site (the robust coordination the `.4.2.1.2.2.3` note demanded — NOT a fragile per-rule `@sample`).
  4. **Witness judge unchanged.** The certifying gate's parser re-check (`witness_check`) stays the only judge — a mis-coordinated name fails loudly (bounded attempts), never a false witness. **Negative gates** (`lacks_fact_attribute_equals`) are trivially satisfiable by an undeclared name (today's behavior) ⇒ NO prelude ⇒ not in this cohort.
- **Expected before→after (earned by `.4b.2`, NOT pre-claimed).** SV cert `UNKNOWN 56 → ~31–37` (the ~19–25 store-gate cohort witnessed); deterministic seeds 0/7/42, `spf=0`, `proof_reverify_failures=0`; **ZERO newly-UNKNOWN** (the pass only UNIONS witnesses, parser adjudicates — the M2a no-regression invariant); the 6 fully-certified grammars byte-identical + each `fully_certified=true`; `stimuli_cross_family_platform_gate` green; SV external corpus 14/14; `cargo test --lib` green (+ a no-op-for-predicate-free-grammar lock + a declare-then-use-witness-parses lock); `clippy_on_rust_change` source-clean. GENERATOR-ONLY ⇒ no grammar/regen/release/schema/ledger change; SV row stays `Mostly Done` (closure-debt retirement, not a row flip).
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

## `.4b.2` — IMPLEMENT store-aware (name-coordinated) declare-then-use witness prelude

- **Status:** `DONE` (`PGEN-STORE-AWARE-GEN-0006`, 2026-06-22). Landed the GENERATOR-ONLY engine code change
  per the `.4b.1` pinned mechanism. **SV cert `UNKNOWN 56 → 46` (witness 1232 → 1242, +10 store-gate cohort
  rules witnessed), deterministic at seeds 0/7/42, `spf=0`, ZERO newly-UNKNOWN.** This slice dogfoods the
  acceptance-checklist gate (`TOOLBOX.md` / `scripts/check_diagnosis_evidence.sh`).
- **What landed (generator-only, `stimuli_generator.rs`):** the C2.2 semantic-prelude was extended from
  `fact_count_at_least` count gates to NAME-matching store gates as a single declare-then-use coordination:
  1. `compute_name_gates` precomputes, per rule, its positive `has_fact(K,$ref)` /
     `fact_attribute_equals(K,$ref,declaration_family,V)` gate (+ `lacks_fact_attribute_equals` excluded
     families), via the SAME `parse_semantic_runtime_directives` parse — a SELF-EMITTING producer (emits the
     kind it gates on, the `declared_forward_*`/`declared_*_alias` idiom) is EXCLUDED (it is not a pure
     declare-then-use consumer). `store_aware_gen` now also activates on a name gate (capability-gated, never
     grammar-name-gated).
  2. `compute_reach_prelude` is a dispatcher: count prelude first (proven), then `compute_name_prelude` —
     finds the first name-gated rule on the reach path, picks a deterministic family-matching producer whose
     `@emit_fact` name resolves to its whole render, and hosts ONE producer iteration at an upstream
     quantifier site reaching it (`reach_hops` + `ReachPrelude { iterations: 1, name_gate: Some(...) }`).
  3. the emit hook (`gen_emit_facts_for_rule`) resolves an undotted `$ref` name (`name: $body`) to the
     producer's rendered identifier so the registered fact carries the real declared name (regex's literal
     `name: capture` and dotted refs are byte-identical/verbatim).
  4. `reach_prelude_replay_text` forces the gated consumer's whole render to the live store name the
     producer just declared (`store_name_for_gate`, most-recent `(kind, family)` match) — declare-then-use.
     The parser re-check (`witness_check`) stays the only witness judge.
- **The 10 newly-witnessed cohort rules:** `checked_type_identifier`, `checked_nettype_identifier`,
  `known_unscoped_covergroup_type_identifier`, `known_unscoped_interface_class_type_identifier`,
  `known_unscoped_let_identifier`, `known_unscoped_parameter_identifier`, `known_unscoped_data_type_identifier`,
  `known_unscoped_class_scope_interface_class_identifier`, `known_unscoped_base_class_type_parameter_identifier`,
  `known_unscoped_block_class_type`. The remaining store-gate residual (`class_scope`/class-member-context
  variants: `known_unscoped_class_scope_class_identifier`, `constraint_set`, `extern_constraint_declaration*`,
  `provisional_unscoped_block_class_type`, the `class_scoped_*call*` family, `known_unscoped_block_type_identifier`)
  needs deeper reach (a class-scope prefix or class-member host) and is a follow-up leaf (`.4b.3`); the SVA
  infix parse-bug + `no_path` remain out of scope per `.4b.1`.
- **In-session regression caught + fixed (tools-first):** the first cut regressed `declared_forward_class_identifier`
  (witnessed → UNKNOWN). `DEBUG_PROBES` named the forced sample `typedef\foo \foo ;typedef class\foo ;` — the
  prelude declared `\foo`, then forced the SELF-EMITTING `declared_forward_class_identifier` to RE-declare it →
  parse conflict. Fix: exclude self-emitting producers from the consumer gate map (a rule that emits kind `K`
  and gates `has_fact(K,…)` self-satisfies; it witnesses via normal generation). After: `56 → 46`, strict
  subset of the baseline 56 (zero newly-UNKNOWN).
- **Acceptance Checklist (enforced — EARNED, oracle-cited):**
  - [x] **REPRODUCE / ISSUE** — `UNKNOWN=56` (seeds 0/7/42; `CERTIFICATE-COVERAGE … total=1289 proof=1 witness=1232 UNKNOWN=56 (sample_parse_failures=0)`) with the store-gate cohort `parsed=false` (`DEBUG_PROBES`: forced `localparam\foo \foo ;` / `class\foo ;` use `\foo` as a type WITHOUT a declaration — declare-then-use gap).
  - [x] **ROOT CAUSE (WHY + WHERE)** — the plannable-witness pass (`stimuli_generator.rs`) forces a store-gated use-site but never generates the fact-emitting DECLARATION its `@predicate` requires; the C2.2 `compute_reach_prelude` solved this only for `fact_count_at_least` count gates, not the NAME-matching `has_fact`/`fact_attribute_equals` gates. `DEBUG_PROBES` + scoped semantic trace (`🚫 … rejected by post predicate 'fact_attribute_equals [type_name, "\foo", declaration_family, covergroup]' ↪ NEGATIVE … none matched name "\foo"`).
  - [x] **FIX** — name-coordinated declare-then-use prelude (generator-only; the `.4b.1` mechanism — Level-3+ general parser-agnostic capability reusing the existing runtime + annotation vocabulary; no grammar/regen/release/schema bump).
  - [x] **ADDRESSED (verified)** — SV cert `UNKNOWN 56 → 46` (decisive A/B on the regen-lockstep build); witness `1232 → 1242` (+10 store-gate cohort rules witnessed, listed above); `spf=0`, `proof_reverify_failures=0`; deterministic at seeds 0/7/42 (`UNKNOWN=46` byte-identical).
  - [x] **NO REGRESSION** — ZERO newly-UNKNOWN (the new 46 is a strict SUBSET of the baseline 56, script-verified); the 6 fully-certified grammars `fully_certified=true` (regex/json/vhdl/svpp/rtl_frontend re-run UNKNOWN=0; regex — the only OTHER `store_aware` grammar — byte-identical at seeds 0/7/42; rtl_const_expr `store_aware_gen=false` ⇒ path off ⇒ byte-identical, its default-depth-24 artifact pre-existing/orthogonal); `cargo test --lib --features generated_parsers` **729 passed / 0 failed** (+3 new locking tests: name whole-render resolution, producer-family gate, store-name lookup); `clippy_on_rust_change` source stage exits 0 (all 189 generated-code `eq_op` errors pre-existing in `generated/*_parser.rs`; ZERO new warnings in the changed source).
  - [x] **LOCKSTEP** — CHANGES.md, MEMORY.md, LIVE_ACHIEVEMENT_STATUS.md (SV cert 56→46), the book Grammar-Well-formedness SV-arc beat + cert number, this leaf + the tree status, decision record [[project_store_aware_generation]] updated.

## `.4b.3` — DESIGN: decompose the `UNKNOWN=46` residual + root-cause the `block_type`/`data_type` asymmetry (tools-first)

- **Status:** `DONE (DESIGN)` — `PGEN-STORE-AWARE-GEN-0007`, 2026-06-22, PURE-DOCS. Mirrors this tree's
  `.4b.1` DESIGN → `.4b.2` IMPLEMENT rhythm on the most-tuned generation surface — decompose + pin the
  mechanism before any engine change, no guessing ([[feedback_no_codebase_change_without_tool_backed_facts]],
  [[feedback_pinpoint_real_blocker_not_menu]]). The engine implements are the new leaves `.4b.4`/`.4b.5`/`.4b.6`.
- **Verified baseline (this slice, 2026-06-22).** A regen-lockstep DEBUG `ast_pipeline` (generated SV parser
  18:56 + binary 19:49, both post the committed `-0111` grammar; no Rust source newer than the binary)
  re-measures `CERTIFICATE-COVERAGE … total=1289 proof=1 witness=1242 UNKNOWN=46 (sample_parse_failures=0,
  proof_reverify_failures=0)` at seed 0 — matches the committed `.4b.2` after-state. This is the `.4b.4`+ "before".
- **The 46-rule UNKNOWN classification (fresh `PGEN_CERT_COVERAGE_DUMP_ALL=1 PGEN_CERT_COVERAGE_DEBUG_PROBES=1`, seed 0).**
  - **19 `no_path`** dead-rule candidates (the `WARNING … NO reach path from the entry` list — `sv_multi_entry_root`,
    `systemverilog_parseable_file`, `parseable_source_item`, `class_constructor_super_args`,
    `declared_interface_class_identifier`, `include_statement`, `interface_class_declaration`/`_item`/`_method`,
    the `library_*` family, and the empty-verdict `kw_include`/`kw_incdir`/`kw_library`/`kw_file_path_spec`/`kw_n_29`/`kw_n_48`
    shells). Adjudicated NON-defects (linter territory, `GRAMMAR-WELLFORMED.H.12.6`). **Out of scope.**
  - **~7 SVA infix-operator PARSE bug** — `kw_within`, `kw_until`, `kw_until_with`, `kw_intersect`, `kw_s_until`,
    `kw_s_until_with`, `kw_constant`: forced samples like `sequence\foo ;3395.0within 2.88863endsequence` are
    `parsed=false` because **the parser rejects `a within b` at the operator** (the parked
    `GRAMMAR-WELLFORMED.H.12.5.8` SVA precedence cascade). A generator/witness change CANNOT fix a parser bug.
    **Out of scope** for STORE-AWARE-GEN.
  - **The store-gate / reach residual (THE `.4b.4`+ cohort), split into 3 mechanistically-distinct sub-cohorts:**
    - **3A — block-scoped non-gated carrier** (`parsed=false`, NO declaration hosted): `known_unscoped_block_type_identifier`
      (`function new;\foo \foo ;endfunction`), `known_unscoped_block_covergroup_identifier` (same shape). The cleanest
      next implement — see the root cause below.
    - **3B — class-member / class-scope context** (needs a CLASS declared *as a class* + a `::`-scoped use render):
      `known_unscoped_class_scope_class_identifier` (`parsed=false`), `known_unscoped_class_scope_type_parameter_identifier`
      (`parsed=true witnessed=false`), `constraint_set` (`constraint\foo ::\foo {…}`), `extern_constraint_declaration`/`_sv_2017`
      (`constraint\foo ::\foo {}`), `property_qualifier` (`class\foo ;rand\foo ;endclass`), `provisional_unscoped_block_class_type`
      (got a *typedef* hosted but the gate wants family=class), the `known_unscoped_class_scoped_call_*` family, `class_scoped_tf_call`
      (the last four `parsed=true witnessed=false` — a sibling class-scope rule consumes the bytes).
    - **3C / misc — reach-routing** (`parsed=true witnessed=false`, the sample parses but a carrier sibling consumes it,
      or a self-emit idiom): `wildcard_escape_nettype_identifier` (`\foo \foo ;`), `declared_class_alias_identifier`
      (`typedef\foo \foo ;` — a SELF-EMIT alias, deliberately excluded by `.4b.2`'s self-satisfying-producer skip; it
      should witness via normal generation — a routing/weighting gap, not a declare-then-use gap), `context_member_method_call`,
      `named_checker_port_connection`/`_sv_2017`, `repeat_range`, `with_covergroup_expression`, `union_modifier`.
- **ROOT CAUSE of sub-cohort 3A (WHY + WHERE, A/B-proven, decisive).**
  `known_unscoped_block_type_identifier` and `known_unscoped_data_type_identifier` have **byte-identical bodies**
  — both `:= checked_type_identifier packed_dimension*` (`systemverilog.ebnf:1630` / `:1637`) — yet `.4b.2`
  witnessed `data_type` and left `block_type` UNKNOWN. The gate that matters lives on the INNER rule
  `checked_type_identifier := type_identifier` (`:5194`), carrying `@predicate has_fact(type_name, $body)` (`:5193`);
  the two carriers carry **no direct `@predicate`** (only a routing comment above them). `compute_name_gates`
  (`stimuli_generator.rs:6160`) registers a `NameGate` only for a rule **directly** carrying the predicate, so only
  `checked_type_identifier` is in `gen_name_gate` — not the carriers. `compute_name_prelude`'s gated-rule search
  (`stimuli_generator.rs:2972-2976`) then looks for a gated rule among **`hops` + `target_rule` ONLY**, never the
  target's own mandatory sub-rules. So when the cert target is the non-gated carrier `known_unscoped_block_type_identifier`,
  the search finds nothing gated → `compute_name_prelude` returns `None` → no declare-then-use prelude → the forced
  witness sample uses `\foo` as a type with no declaration → `checked_type_identifier`'s `has_fact` is false → `parsed=false`.
  `known_unscoped_data_type_identifier` (identical structure) was NOT witnessed by its own prelude either — it was
  witnessed **opportunistically**, as a carrier realized inside `checked_type_identifier`'s OWN single witness sample.
  `PGEN_REACH_PATH_DUMP` A/B confirms why `block_type` was never so realized: its use-site sits **two quantifiers deep**
  (`("source_text","root/q")` → … → `("class_constructor_declaration_sv_2017","root/s5/q")` block-item list → `block_data_type`),
  whereas `data_type`'s use-site is **one quantifier deep**, directly under the top-level `source_text := description*`
  quantifier — so `checked_type_identifier`'s single top-level witness realized `data_type` but never the deeper block carrier.
- **The fix for 3A (pinned, the `.4b.4` IMPLEMENT — ONE clean, parser-agnostic extension).** Extend
  `compute_name_prelude`'s gated-rule discovery so a target/on-path rule that is NOT itself name-gated but whose
  **mandatory derivation prefix** reaches a name-gated rule INHERITS that inner gate (a bounded mandatory-first descent
  through single-mandatory sub-rules — `known_unscoped_block_type_identifier`'s mandatory first sub-rule is the gated
  `checked_type_identifier`). The carrier then gets the SAME declare-then-use prelude the directly-gated rule gets; the
  flat fact store means the existing top-level typedef hosting (already proven for `data_type`) satisfies the block-context
  `has_fact` too. Capability-gated on the predicate's presence (byte-identical for predicate-free grammars), reusing the
  existing `reach_hops`/`ReachPrelude`/`reach_prelude_replay_text` machinery — no new annotation, no grammar/parser change.
  The certifying parser re-check stays the only witness judge. Expected `UNKNOWN 46 → 44` (the two 3A carriers), ZERO
  newly-UNKNOWN, deterministic seeds 0/7/42; earned by `.4b.4`, not pre-claimed.
- **3B / 3C are SEPARATE mechanisms** (their own implement leaves, each tools-first-designed before coding):
  3B needs a class declared *as a class* (family=class producer) plus a `::`-scoped use-site render (the `$ref` is a
  scoped head, not a bare identifier) and, for the `parsed=true witnessed=false` members, reach-routing so the target
  rule (not a sibling) consumes the scoped name. 3C is predominantly reach-routing/weighting (the samples already parse),
  including the deliberately-excluded self-emit `declared_class_alias_identifier`. These do NOT share 3A's one-line
  gated-rule-descent fix, so folding them into one commit would violate "fix shall always be targeted" — they are deferred.
- **Acceptance (for the `.4b.4`+ implements, not this DESIGN slice):** SV cert `UNKNOWN` drops by the addressed
  sub-cohort, deterministic seeds 0/7/42, `spf=0`; the new UNKNOWN set a strict SUBSET of the prior (ZERO newly-UNKNOWN);
  the 6 fully-certified grammars `fully_certified=true` (regex byte-identical at seeds 0/7/42); `stimuli_cross_family_platform_gate`
  green; `cargo test --lib` green; `clippy_on_rust_change` source-clean. Decisive A/B + GLOBAL cert before any commit.
- **Diagnose/verify with the toolbox** (`docs/book/src/diagnosing-unknowns.md`): `DUMP_ALL` → `DEBUG_PROBES` →
  `PGEN_REACH_PATH_DUMP`, per [[feedback_systematically_use_debug_toolbox]]. **This DESIGN slice is PURE-DOCS — no code
  change — so the `check_diagnosis_evidence.sh` code-change gate does not apply; the next leaf (`.4b.4`) carries the
  enforced acceptance checklist it will earn.**

## `.4b.4` — IMPLEMENT sub-cohort 3A: mandatory-first gated-rule descent (block-scoped non-gated carriers)

- **Status:** `DONE` (`PGEN-STORE-AWARE-GEN-0008`, 2026-06-22). Landed the GENERATOR-ONLY engine change per the
  `.4b.3` pinned mechanism. **SV cert `UNKNOWN 46 → 43` (witness 1242 → 1245, +3 block-scoped carriers witnessed),
  deterministic seeds 0/7/42, `spf=0`, ZERO newly-UNKNOWN.**
- **What landed (generator-only, `stimuli_generator.rs`):** the `.4b.2` name-prelude only fired when the witness
  TARGET (or a reach-path hop) was DIRECTLY name-gated. A block-context carrier
  (`known_unscoped_block_type_identifier := checked_type_identifier packed_dimension*`) is NOT itself gated — the
  `has_fact(type_name,$body)` gate lives on its inner `checked_type_identifier` — so it never received a prelude
  and its forced witness used `\foo` as a type with no declaration. Two new helpers close that:
  1. `mandatory_leading_rule_reference(rule)` — the rule named by `rule`'s MANDATORY-first element when that is a
     direct rule reference (`R := S` / `R := S …`, unwrapping `Atom::Node` shells, descending a leading nested
     `Sequence`); `None` for an `Or`-led / quantifier-led / terminal-led body (where the inner rule is not guaranteed
     to render). Mirrors `quantified_body_rule_name`.
  2. `name_gate_via_mandatory_prefix(rule)` — the name gate governing `rule`'s render, checking `rule` itself FIRST
     (a directly-gated rule short-circuits at depth 0, identical to pre-4b.4), else following the bounded
     mandatory-first prefix (depth ≤ 8, cycle-guarded) to the inner gated rule. Returns the INNER gated rule's name +
     gate, so the prelude arms on the rule that actually renders (`reach_prelude_replay_text` keys on it).
  The `compute_name_prelude` gated-rule search now does `hops (direct only) → or_else(target via mandatory prefix)` —
  purely ADDITIVE (a case that matched a direct gate before matches the SAME rule), so already-witnessed targets are
  unperturbed and the flat fact store means the existing top-level typedef hosting satisfies the block-context
  `has_fact`. Capability-gated on `gen_name_gate` non-empty (SV-only) ⇒ byte-identical for predicate-free grammars and
  regex.
- **The 3 newly-witnessed rules:** `known_unscoped_block_type_identifier`, `known_unscoped_block_covergroup_identifier`
  (the two scoped 3A targets), plus the bonus `provisional_unscoped_block_class_type` (a 3B-listed rule whose
  block-context carrier shape also routes its mandatory-first element through a gated inner rule). The remaining
  class-member/class-scope residual (3B: `known_unscoped_class_scope_*`, `constraint_set`,
  `extern_constraint_declaration*`, the `class_scoped_*call*` family, `property_qualifier`) → `.4b.5`; the
  reach-routing residual (3C) → `.4b.6`; the SVA infix parse-bug + `no_path` remain out of scope.
- **Acceptance Checklist (enforced — EARNED, oracle-cited):**
  - [x] **REPRODUCE / ISSUE** — `UNKNOWN=46` (seeds 0/7/42; `CERTIFICATE-COVERAGE … total=1289 proof=1 witness=1242 UNKNOWN=46 (sample_parse_failures=0)`) with the block-scoped carriers `parsed=false` (`DEBUG_PROBES`: forced `function new;\foo \foo ;endfunction` uses `\foo` as a type WITHOUT a declaration — no prelude hosted).
  - [x] **ROOT CAUSE (WHY + WHERE)** — `compute_name_prelude`'s gated-rule search (`stimuli_generator.rs:2972-2976`) inspected only the reach-path hops + the target, never the target's mandatory sub-rules; the `has_fact(type_name,$body)` gate lives on the inner `checked_type_identifier` (`systemverilog.ebnf:5193-5194`), not the byte-identical carriers (`:1630`/`:1660`). A/B-proven by `PGEN_REACH_PATH_DUMP` (block carrier sits 2 quantifiers deep; `data_type` sibling witnessed only opportunistically 1 quantifier deep).
  - [x] **FIX** — mandatory-first gated-rule descent (generator-only; the `.4b.3` mechanism — Level-3+ general parser-agnostic capability reusing the existing prelude/runtime + annotation vocabulary; no grammar/parser/regen/release/schema bump). `compute_name_prelude` arms on the carrier's inner gated rule.
  - [x] **ADDRESSED (verified)** — SV cert `UNKNOWN 46 → 43` (decisive A/B on the regen-lockstep build); witness `1242 → 1245` (+3 block-scoped carriers witnessed, listed above); `spf=0`, `proof_reverify_failures=0`; deterministic at seeds 0/7/42 (`UNKNOWN=43` byte-identical).
  - [x] **NO REGRESSION** — ZERO newly-UNKNOWN (the new 43 is a script-verified strict SUBSET of the baseline 46); regex — the only OTHER `store_aware` grammar — byte-identical `fully_certified=true 198/198` at seeds 0/7/42; json (9/9) / vhdl (216/216) / svpp (74/74) / rtl_frontend (169) all `fully_certified=true` (rtl_const_expr `store_aware_gen=false` ⇒ path off ⇒ byte-identical; its default-depth-24 artifact is pre-existing/orthogonal, covered green by the lib suite at its proper config); `cargo test --lib --features generated_parsers` **731 passed / 0 failed** (+2 locking tests: `store_aware_gen_mandatory_leading_rule_reference_descends_carrier`, `store_aware_gen_name_gate_via_mandatory_prefix_inherits_inner_gate`); `clippy_on_rust_change` source stage error-clean with ZERO findings in the changed code (the 188 generated `eq_op` errors are pre-existing in `generated/*_parser.rs`).
  - [x] **LOCKSTEP** — CHANGES.md, DEVELOPMENT_NOTES.md, MEMORY.md, LIVE_ACHIEVEMENT_STATUS.md (SV cert 46→43), docs/TASK_TREE.md, the book Grammar-Well-formedness SV-arc beat + cert number, this leaf + the tree status, decision record [[project_store_aware_generation]] updated.

## `.4b.5` — DESIGN: root-cause sub-cohort 3B + resolve the directly-gated class-rule OPEN QUESTION (tools-first)

- **Status:** `DONE (DESIGN)` — `PGEN-STORE-AWARE-GEN-0010`, 2026-06-23, PURE-DOCS. Mirrors this tree's
  `.4b.1`/`.4b.3` DESIGN → IMPLEMENT rhythm on the most-tuned generation surface — decompose + pin the
  mechanism with decisive tool evidence before any engine change, no guessing
  ([[feedback_no_codebase_change_without_tool_backed_facts]], [[feedback_pinpoint_real_blocker_not_menu]],
  [[feedback_systematically_use_debug_toolbox]]). The engine implements are the new leaves
  `.4b.6` (3B-ii, cleanest/first) / `.4b.7` (3B-i) / `.4b.8` (3B-iii); 3C/misc shifts to `.4b.9`.
- **Verified baseline (this slice, 2026-06-23).** A regen-lockstep DEBUG `ast_pipeline` (generated SV parser
  Jun-22 18:56 + binary Jun-22 23:32, both post the committed `-0111` grammar; no Rust source newer than the
  binary; `.4b.4` was generator-only, no grammar change since) re-measures
  `CERTIFICATE-COVERAGE … total=1289 proof=1 witness=1245 UNKNOWN=43 (sample_parse_failures=0,
  proof_reverify_failures=0)` at seed 0 — matches the committed `.4b.4` after-state. This is the `.4b.6`+ "before".
- **The OPEN QUESTION (from the resume pointer) — ANSWERED.** It asked why the `.4b.2`/`.4b.4` family-gated
  prelude does not witness a directly-gated bare-render class rule, hypothesizing (a) producer not reachable,
  (b) deep `class…endclass` overflowing the witness depth budget, or (c) the scoped `class_scope` render shape.
  The toolbox shows the real cause is **none of (a)/(b)/(c)** but a 4th, more precise one — **(d) the prelude
  selects the WRONG producer** (a non-bootstrapping typedef-alias). Evidence:
  - **(b) ruled OUT — TOOLBOX 4.5 NOT-depth check.** Re-running cert at `--max-depth 32` gives `UNKNOWN=44`
    with `sample_parse_failures=9`: NO 3B class-scope/constraint rule leaves the UNKNOWN set; deeper depth only
    adds spf noise and loses two unrelated rules (`inout_declaration`, `kw_inout`). The per-target budget is
    adequate; the cause is a forcing/store-gate bug, not depth.
  - **(a) ruled OUT — the `name-prelude spec` debug trace.** A bounded `PGEN_TRACE_VERBOSITY=debug` cert run
    (count 2, seed 0) grepped for `STORE-AWARE-GEN.4b name-prelude spec` shows a prelude **IS** built for the
    directly-gated `known_unscoped_class_scope_class_identifier` (×4) and `known_unscoped_class_scoped_call_class_identifier`,
    each `kind='type_name' family=Some("class") producer='declared_class_alias_identifier' site=('source_text','root') body='source_text_item'`.
    So the prelude is built and a class producer IS reachable.
  - **(d), the real cause — `furthest_position` + producer-render A/B.** The forced sample for
    `known_unscoped_class_scope_class_identifier` is `typedef\foo \foo ;localparam\foo \foo ;` and `--parse`
    rejects it at **`furthest_position=11`** — the typedef's SOURCE-TYPE slot. The scoped predicate trace names
    it exactly: `checked_type_identifier` rejected by `has_fact [type_name, "\foo"]` ↪ NEGATIVE (3 facts exist,
    none matched `\foo`). The chosen producer `declared_class_alias_identifier` (`systemverilog.ebnf:5112-5115`)
    renders a typedef-ALIAS `typedef <existing_type> <alias> ;`, whose mandatory LHS source type is itself a
    has_fact-gated `checked_type_identifier` — unsatisfiable in a bootstrap context, so the prelude declaration
    `typedef \foo \foo ;` never parses → no `type_name`/`class` fact is established → the gated consumer rejects.
    Decisive A/B over the three family=class producer renders confirms it:
    - `typedef\foo \foo ;` (`declared_class_alias_identifier`) → **REJECT** @pos 11 (LHS undeclared) — NON-bootstrapping.
    - `typedef class\foo ;` (`declared_forward_class_identifier`, `:5117-5119`) → **PASS** — self-bootstrapping.
    - `class\foo ;endclass` (`declared_class_identifier`, `:965-967`) → **PASS** — self-bootstrapping.
    `compute_name_prelude` (`stimuli_generator.rs:2991-3003`) collects family-matching producers, `producers.sort_unstable()`
    (alphabetical), and commits to the FIRST one with a reachable hop. `declared_class_alias_identifier` sorts
    before `declared_class_identifier`/`declared_forward_class_identifier`, so the one NON-bootstrapping candidate
    is always chosen and the two that would witness are never tried. A real class decl + a `::`-scoped use parses:
    `typedef class\foo ;module m;initial x=\foo ::new();endmodule` → **PASS**.
- **Sub-cohort 3B decomposes into THREE mechanistically-distinct sub-blockers (each its own targeted implement,
  per "fix shall always be targeted" [[feedback_tools_first_no_guessing]]):**
  - **3B-ii — producer selection (the cleanest + most general; `.4b.6`).** `parsed=false`, prelude built but the
    chosen producer is the non-bootstrapping typedef-alias. Affects `known_unscoped_class_scope_class_identifier`,
    `known_unscoped_class_scoped_call_class_identifier`. This is the OPEN QUESTION's "directly-gated bare-render
    class rule" case. **FIX (pinned):** make producer selection robust — `compute_name_prelude` must prefer a
    producer whose hosted declaration is *self-bootstrapping* (does NOT itself require, in a mandatory position,
    a pre-existing fact of the same gated kind). The general, parser-agnostic predicate: skip a candidate producer
    whose host derivation has a mandatory `@predicate`-gated sub-rule on the SAME `kind` other than the producer's
    own self-satisfying emit (the typedef-alias's LHS `checked_type_identifier` is exactly such a gate). Equivalent
    robust framing: try candidate producers in order and keep one whose forced sample passes the parser re-check
    (the witness judge already guards every witness; a non-bootstrapping prelude simply fails to witness). Reuses
    the existing `gen_emit_facts`/`reach_hops`/`ReachPrelude`/`reach_prelude_replay_text` machinery — no new
    annotation, no grammar/parser change; capability-gated on `gen_name_gate` non-empty ⇒ byte-identical for
    predicate-free grammars. Expected `UNKNOWN 43 → ~41` (the two class-scope rules), ZERO newly-UNKNOWN.
  - **3B-i — gate-not-mandatory-first (`.4b.7`).** `parsed=false`, NO prelude built. `extern_constraint_declaration_sv_2017
    := (kw_static)? kw_constraint class_scope constraint_identifier constraint_block` (`:2055`) and `constraint_set`
    (`:1445`) are gated only on the inner `class_scope` → `known_unscoped_class_scope_class_identifier`, which is
    NOT the mandatory-FIRST element (it sits behind the leading `kw_constraint` keyword). `.4b.4`'s
    `name_gate_via_mandatory_prefix` only follows the mandatory-first *rule reference*, so it returns `None` and
    no prelude arms. Proven: declaring the class first parses — `typedef class\foo ;constraint\foo ::\foo {}` → **PASS**.
    **FIX direction:** extend gate discovery to find a gated rule reached through the mandatory SEQUENCE (skip
    leading terminals/optionals), not only the mandatory-first rule reference. Separate mechanism from 3B-ii.
  - **3B-iii — target-own incompleteness (`.4b.8`).** `parsed=false` but the class gate is already satisfiable;
    `property_qualifier` (`:4228`)'s forced sample `class\foo ;rand\foo ;endclass` rejects @pos 19 because
    `rand <type> ;` is an incomplete `data_declaration` (no variable name). Proven: `class\foo ;rand\foo \bar ;endclass`
    → **PASS**. This is a target-own-structure/forcing-completeness gap (adjacent to 3C), NOT a store-gate gap —
    a distinct mechanism, deferred to its own leaf.
- **Acceptance (for the `.4b.6`+ implements, not this DESIGN slice):** SV cert `UNKNOWN` drops by the addressed
  sub-cohort, deterministic seeds 0/7/42, `spf=0`; the new UNKNOWN set a strict SUBSET of the prior (ZERO
  newly-UNKNOWN); the 6 fully-certified grammars `fully_certified=true` (regex byte-identical at seeds 0/7/42);
  `stimuli_cross_family_platform_gate` green; `cargo test --lib` green; `clippy_on_rust_change` source-clean.
  Decisive A/B + GLOBAL cert before any commit.
- **Diagnose/verify with the toolbox** (`docs/book/src/diagnosing-unknowns.md`): `DUMP_ALL` → `DEBUG_PROBES` →
  `name-prelude spec` debug trace → `furthest_position` + scoped `--trace-rules` predicate trace +
  the TOOLBOX 4.5 `--max-depth` NOT-depth check, per [[feedback_systematically_use_debug_toolbox]]. **This DESIGN
  slice is PURE-DOCS — no code change — so the `check_diagnosis_evidence.sh` code-change gate does not apply; the
  next leaf (`.4b.6`) carries the enforced acceptance checklist it will earn.**

## `.4b.6` — IMPLEMENT sub-cohort 3B-ii: self-bootstrapping HOST-BRANCH producer selection

- **Status:** `DONE` (`PGEN-STORE-AWARE-GEN-0011`, 2026-06-23). Landed the GENERATOR-ONLY engine change per the
  `.4b.5` pinned mechanism — **with a tools-first refinement of its root cause** (the discipline at work:
  [[feedback_no_codebase_change_without_tool_backed_facts]], [[feedback_systematically_use_debug_toolbox]]).
  **SV cert `UNKNOWN 43 → 41` (witness 1245 → 1247, +2 class-scope rules witnessed), deterministic seeds 0/7/42,
  `spf=0`, ZERO newly-UNKNOWN.**
- **Tools-first ROOT-CAUSE REFINEMENT (why the first cut was a no-op, and the corrected mechanism).** `.4b.5`
  pinned the fix as "producer selection: prefer a producer whose own derivation is self-bootstrapping", and the
  first implementation checked the producer RULE's mandatory body for a same-kind gate. The A/B was decisive
  that it did **nothing** — SV cert stayed `UNKNOWN=43`, the forced sample for
  `known_unscoped_class_scope_class_identifier` byte-identical (`typedef\foo \foo ;localparam\foo \foo ;`). Reading
  the grammar with the toolbox showed why: BOTH family=class producers reachable from the top-level site share the
  **byte-identical body** `type_identifier` (`declared_class_alias_identifier := type_identifier` `:5115`,
  `declared_forward_class_identifier := type_identifier` `:5119`) — so a producer-BODY check cannot distinguish
  them. The non-bootstrapping-ness lives one level up, in the producer's **HOST BRANCH** of `type_declaration`:
  the alias sits in `kw_typedef class_type declared_class_alias_identifier …` (`:5146`), whose `class_type`
  **mandatory sibling** descends to a `has_fact(type_name,·)` gate; the forward sits in
  `kw_typedef kw_class declared_forward_class_identifier semi` (`:5152`), a clean `kw_class` literal.
  `parseability_probe` proved it: `typedef\foo \foo ;localparam\foo \foo ;` → REJECT `furthest_position=11`
  (the undeclared `class_type` source); `typedef class\foo ;localparam\foo \foo ;` → **PASS**;
  `class\foo ;endclass⏎localparam\foo \foo ;` → **PASS**.
- **What landed (generator-only, `stimuli_generator.rs`):** the producer selection in `compute_name_prelude` is now
  a **TWO-PASS** scan keyed on the producer's reach PATH, not its body:
  1. `reach_path_renders_unsatisfiable_gate(hops)` + `offpath_siblings_gated_along_path(...)` — walk the producer's
     reach-path hops (the `collect_rule_reference_sites` `node_path` encoding, identical to
     `offpath_sibling_depth_along_path`) and, at every Sequence crossed, test each MANDATORY off-path sibling with
     the existing `mandatory_node_gated` store-gate walk against an EMPTY available set (the prelude is the first
     declaration). The typedef-alias path is flagged (its `class_type` sibling is gated); the forward path is clean.
  2. `compute_name_prelude` wraps the site/producer loops in `for prefer_clean in [true, false]`: PASS 1 skips any
     producer whose forced reach path is gated; PASS 2 restores the exact pre-4b.6 "first reachable producer at the
     innermost site" fallback, so a candidate is NEVER dropped (byte-identical when no clean producer exists). For an
     already-witnessed target the chosen producer's path is necessarily clean, so PASS 1 selects the SAME producer —
     the change only PROMOTES a clean producer where the pre-4b.6 path committed a gated one (and so never witnessed).
  Capability-gated on `gen_name_gate` non-empty (SV-only) ⇒ byte-identical for predicate-free grammars and for the
  count path (`compute_count_prelude` is untouched ⇒ regex byte-identical). The parser re-check (`witness_check`)
  stays the sole witness judge.
- **The 2 newly-witnessed rules:** `known_unscoped_class_scope_class_identifier`,
  `known_unscoped_class_scoped_call_class_identifier` (the 3B-ii pair). The remaining 3B residual —
  gate-not-mandatory-first (`extern_constraint_declaration(_sv_2017/_sv_2023)`, `constraint_set`) → `.4b.7`;
  target-own incompleteness (`property_qualifier`) → `.4b.8`; reach-routing 3C → `.4b.9`; the SVA infix parse-bug +
  `no_path` remain out of scope per `.4b.1`.
- **Acceptance Checklist (enforced — EARNED, oracle-cited):**
  - [x] **REPRODUCE / ISSUE** — `UNKNOWN=43` (seeds 0/7/42; `CERTIFICATE-COVERAGE … total=1289 proof=1 witness=1245 UNKNOWN=43 (sample_parse_failures=0)`) with the 3B-ii class-scope rules `parsed=false` (`DEBUG_PROBES`: forced `typedef\foo \foo ;localparam\foo \foo ;` — the prelude's typedef-ALIAS declaration rejects on its undeclared `class_type` source type).
  - [x] **ROOT CAUSE (WHY + WHERE)** — `compute_name_prelude`'s producer loop (`stimuli_generator.rs`) commits the alphabetically-first reachable family=class producer `declared_class_alias_identifier`, whose HOST BRANCH `type_declaration := kw_typedef class_type declared_class_alias_identifier …` (`systemverilog.ebnf:5146`) carries the mandatory `class_type` sibling → `has_fact(type_name,·)` gate, unsatisfiable in the empty-store prelude. Both class producers share body `type_identifier` (`:5115`/`:5119`), so the blocker is the host-branch sibling, NOT the producer body. `parseability_probe` A/B: alias prelude REJECT `furthest_position=11`; forward prelude (`typedef class\foo ;…`) PASS; real-class prelude (`class\foo ;endclass…`) PASS.
  - [x] **FIX** — two-pass HOST-PATH-aware producer selection (generator-only; Level-3+ general parser-agnostic capability reusing the existing reach-path + `mandatory_node_gated` store-gate walk + annotation vocabulary; no grammar/parser/regen/release/schema bump). `compute_name_prelude` PASS 1 prefers a producer whose forced reach path renders no unsatisfiable same-store gate; PASS 2 = the byte-identical pre-4b.6 fallback.
  - [x] **ADDRESSED (verified)** — SV cert `UNKNOWN 43 → 41` (decisive A/B on the regen-lockstep build); witness `1245 → 1247` (+2 class-scope rules witnessed, listed above); `spf=0`, `proof_reverify_failures=0`; deterministic at seeds 0/7/42 (`UNKNOWN=41` byte-identical).
  - [x] **NO REGRESSION** — ZERO newly-UNKNOWN (the new 41 is a script-verified strict SUBSET of the baseline 43 — `comm -13` empty; the only two rules that left are the addressed pair); regex `fully_certified=true 198/198` byte-identical at seeds 0/7/42; json (9/9) / vhdl (216/216) / svpp (74/74) / rtl_frontend (169) all `fully_certified=true` (rtl_const_expr `store_aware_gen=false` ⇒ path off ⇒ inert, covered green by the lib suite at its proper config); structural inertness proven (only SystemVerilog declares positive name-gate predicates — 33; all 6 fully-certified grammars 0 ⇒ `gen_name_gate` empty ⇒ `compute_name_prelude` early-returns); `cargo test --lib --features "generated_parsers ebnf_dual_run"` **764 passed / 0 failed** (+1 locking test `store_aware_gen_prefers_self_bootstrapping_host_branch`); `clippy_on_rust_change` source stage clean (zero findings in the changed `stimuli_generator.rs` ranges; the generated-code `eq_op` errors are pre-existing in `generated/*_parser.rs`).
  - [x] **LOCKSTEP** — CHANGES.md, DEVELOPMENT_NOTES.md, MEMORY.md, LIVE_ACHIEVEMENT_STATUS.md (SV cert 43→41), docs/TASK_TREE.md (frontier), the book Grammar-Well-formedness SV-arc beat + cert number, this leaf + the tree status, decision record [[project_store_aware_generation]] updated.

## `.4b.7` — IMPLEMENT sub-cohort 3B-i: gate-discovery past a leading prefix + into an ordered choice

- **Status:** `DONE` (`PGEN-STORE-AWARE-GEN-0012`, 2026-06-23). GENERATOR-ONLY engine change. **SV cert
  `UNKNOWN 41 → 38` (witness 1247 → 1250, +3 witnessed), deterministic seeds 0/7/42, `spf=0`, ZERO
  newly-UNKNOWN.** The 3 newly-witnessed rules: `extern_constraint_declaration_sv_2017`,
  `extern_constraint_declaration`, and the bonus `class_scoped_tf_call`.
- **Tools-first WHY+WHERE (the discovery gap).** The `.4b.2`/`.4b.4` discovery
  (`name_gate_via_mandatory_prefix`) only follows the MANDATORY-FIRST *rule reference*. For
  `extern_constraint_declaration_sv_2017 := ( kw_static )? kw_constraint class_scope constraint_identifier
  constraint_block` (`:2055`) the gated `known_unscoped_class_scope_class_identifier` sits behind a leading
  `( kw_static )?` optional + a `kw_constraint` keyword rule, AND inside `class_scope` →
  `class_scope_type := ( … | known_unscoped_class_scope_class_identifier | … ) …`'s ordered choice — so
  the mandatory-FIRST walk returns `None` and NO prelude arms. `DEBUG_PROBES` (seed 0) showed the forced
  sample `constraint\foo ::\foo {}` `parsed=false` with NO declare-then-use prefix (vs the `.4b.6` cohort
  which HAD a prefix), confirming "no prelude". `parseability_probe` A/B: `constraint\foo ::\foo {}` →
  REJECT `furthest_position=21` (class `\foo` undeclared); `typedef class\foo ;constraint\foo ::\foo {}` →
  **PASS**. So a declare-then-use prelude is exactly the fix; the only gap is DISCOVERY.
- **Tools-first ROOT-CAUSE REFINEMENT (why the first cut regressed, and the corrected mechanism).** A
  first-cut broadening that scanned ALL mandatory sequence elements + descended every `Or` (first gated
  alternative) regressed SV cert `41 → 44` (+3 wins, −6 regressions), CAUGHT by the strict-subset gate
  (`associative_dimension`, `if_generate_else_clause`, `mixed_parameter_port_list`,
  `mixed_string_parameter_port_list`, `parameter_port_declaration(_sv_2017)`). `DEBUG_PROBES` named the
  mechanism: the broadened scan STEPPED PAST a self-satisfying producer (`param_assignment_sv_2017 :=
  declared_parameter_identifier … assign constant_param_expression`) into a DEEP `type_name` gate inside
  `constant_param_expression`, arming a structurally-invalid double-`#()` prelude
  (`module\foo #(type\foo )#(parameter\foo )`); and `data_type`'s `Or` has an ungated builtin ESCAPE the
  generator takes, so a prelude there breaks an otherwise-fine witness. FIX: STOP the sequence scan at the
  FIRST store-gated rendered position (`node_render_store_gated` = the `.4b.6` `mandatory_node_gated` walk
  against an empty store), descend an `Or` ONLY when it has no ungated escape, and gate the whole broadened
  leg on `mandatory_reach_gate` (the render is UNAVOIDABLY store-gated). This drops all 6 regressions and
  keeps all 3 wins (`41 → 38`).
- **What landed (generator-only, `stimuli_generator.rs`).** `name_gate_via_mandatory_prefix` is now a
  TWO-LEG discovery: leg 1 = the `.4b.4` mandatory-FIRST chain (`name_gate_via_mandatory_first_chain`,
  preserved byte-identical); leg 2 (`.4b.7`, fires only when leg 1 is `None`) = a structural walk
  (`gate_in_mandatory_prefix_rule`/`gate_in_mandatory_prefix_node`) that scans a sequence stopping at the
  first `node_render_store_gated` element, descends an escape-free `Or` to its first POSITIVE name gate,
  and is guarded by `mandatory_reach_gate(rule, ∅)`. Reuses the existing `.4b.6` `mandatory_node_gated`
  store-gate machinery + the `gen_name_gate`/prelude/`reach_prelude_replay_text` vocabulary — no new
  annotation, no grammar/parser change. Capability-gated on `gen_name_gate` non-empty (SV-only) ⇒
  byte-identical for predicate-free grammars and for the count path.
- **`constraint_set` is a DISTINCT problem (deferred to `.4b.9`).** Its `class_scope` gate is a mandatory
  OFF-PATH SIBLING on the reach path (the witness reaches `constraint_set` inside an
  `extern_constraint_declaration` `constraint_block`), NOT on `constraint_set`'s own mandatory prefix
  (`constraint_expression`), so neither the hop-scan nor `name_gate_via_mandatory_prefix` reaches it. It
  needs an off-path-sibling prelude-arming pass (akin to `.4b.6`'s `offpath_siblings_gated_along_path`).
- **Acceptance Checklist (enforced — EARNED, oracle-cited):**
  - [x] **REPRODUCE / ISSUE** — `UNKNOWN=41` (seeds 0/7/42; `CERTIFICATE-COVERAGE … total=1289 proof=1 witness=1247 UNKNOWN=41 (sample_parse_failures=0)`) with `extern_constraint_declaration_sv_2017` / `extern_constraint_declaration` / `constraint_set` residual; `DEBUG_PROBES` forced sample `constraint\foo ::\foo {}` `parsed=false`, NO declare-then-use prefix ⇒ no prelude armed.
  - [x] **ROOT CAUSE (WHY + WHERE)** — `name_gate_via_mandatory_prefix` (`stimuli_generator.rs`) only follows the mandatory-FIRST rule reference, so it cannot skip `extern_constraint_declaration_sv_2017`'s leading `( kw_static )? kw_constraint` (`systemverilog.ebnf:2055`) nor descend `class_scope` → `class_scope_type`'s `Or` (`:1065`) to reach the positive gate `known_unscoped_class_scope_class_identifier` (`:1025`, `fact_attribute_equals[type_name,·,declaration_family,class]`). `parseability_probe` A/B: no-class REJECT `furthest_position=21`; class-first PASS.
  - [x] **FIX** — generator-only two-leg discovery (Level-3+ general parser-agnostic capability reusing the `.4b.6` `mandatory_node_gated` walk + the existing prelude/name-gate vocabulary; no grammar/parser/regen/release/schema bump). Leg 2 scans a sequence STOPPING at the first store-gated rendered position, descends an escape-free `Or`, and is guarded by `mandatory_reach_gate` against an empty store (unavoidable gate only).
  - [x] **ADDRESSED (verified)** — SV cert `UNKNOWN 41 → 38` (decisive A/B on the regen-lockstep build); witness `1247 → 1250` (+3: `extern_constraint_declaration_sv_2017`, `extern_constraint_declaration`, `class_scoped_tf_call`); `spf=0`, `proof_reverify_failures=0`; deterministic at seeds 0/7/42 (`UNKNOWN=38` byte-identical).
  - [x] **NO REGRESSION** — ZERO newly-UNKNOWN (the new 38 is a script-verified strict SUBSET of the baseline 41 — `comm -13` empty across seeds 0/7/42; the only rules that left are the 3 wins); regex `fully_certified=true 198/198` at seeds 0/7/42 (name-path inert — regex declares no positive name gates ⇒ `gen_name_gate` empty ⇒ `compute_name_prelude` early-returns); json (9/9) / vhdl (216/216) / svpp (74/74) / rtl_frontend (169) all `fully_certified=true` (rtl_const_expr `store_aware_gen=false` ⇒ inert, covered by the lib suite); `cargo test --lib --features "generated_parsers ebnf_dual_run"` **765 passed / 0 failed** (+1 locking test `store_aware_gen_name_gate_discovers_gate_behind_leading_prefix_and_or`); `clippy_on_rust_change` source-clean (zero `stimuli_generator.rs` findings; the generated-code `eq_op` errors pre-existing in `generated/*_parser.rs`).
  - [x] **LOCKSTEP** — CHANGES.md, DEVELOPMENT_NOTES.md, MEMORY.md, LIVE_ACHIEVEMENT_STATUS.md (SV cert 41→38), this leaf + the tree status (frontier → `.4b.8`), the book Grammar-Well-formedness SV-arc beat + cert number, decision record [[project_store_aware_generation]] note.

## `.4b.8` — DESIGN: root-cause sub-cohort 3B-iii (`property_qualifier`) — NOT a missing var name; a witness-generator NAME COLLISION (tools-first)

- **Status:** `DONE (DESIGN)` — `PGEN-STORE-AWARE-GEN-0013`, 2026-06-23, PURE-DOCS. The `.4b.5` framing
  ("target-own incompleteness — `rand <type> ;` omits the variable name") was a SYMPTOM; the toolbox
  re-root-caused it to a deeper **witness-generator name-collision**, which reframes the IMPLEMENT and
  raises its blast-radius. The IMPLEMENT (`.4b.8` proper) is pinned as the next code slice and is best done
  in a fresh, focused session (a core witness-generator name-selection change, not a contained prelude
  extension like `.4b.7`).
- **Verified baseline (this slice, 2026-06-23, post-`.4b.7`).** `CERTIFICATE-COVERAGE … total=1289 proof=1
  witness=1250 UNKNOWN=38 (sample_parse_failures=0)` at seed 0 — the `.4b.7` after-state. `property_qualifier`
  `DEBUG_PROBES`: forced sample `class\foo ;rand\foo ;endclass` `parsed=false`.
- **Tools-first ROOT CAUSE (WHY + WHERE).** `property_qualifier := random_qualifier` (`:4228`) has NO store
  gate, and `variable_identifier := declaration_identifier` (`:5419`) is NOT gated either — so this is not a
  store-gate cohort member. The reach context is `class_property := property_qualifier* data_declaration`
  inside `class_declaration`, so the witness wraps the target in `class\foo ;…endclass` — naming the class
  with the **canonical witness identifier `\foo`**, which emits a `type_name` fact. Inside, the minimal
  `data_declaration_sv_2017 := (kw_const)? (kw_var)? (lifetime)? data_type_or_implicit
  list_of_variable_decl_assignments semi` (`:1577`) picks `data_type_or_implicit` = `implicit_data_type`
  (empty, `:2300`) and a free `variable_identifier` = **`\foo`** (the same canonical name) — intending
  `rand` + implicit type + variable `\foo`. But `\foo` is now a KNOWN TYPE, so when the parser re-checks,
  PEG's `data_type_or_implicit := data_type | implicit_data_type` (`:1730`) tries `data_type` FIRST and
  greedily consumes `\foo` as the TYPE, leaving the mandatory `list_of_variable_decl_assignments` empty →
  REJECT `furthest_position=19`. `parseability_probe` A/B is decisive: `class\foo ;rand\foo ;endclass` →
  REJECT @19; `class\foo ;rand\foo \bar ;endclass` (distinct variable name `\bar`) → **PASS**. So the gate
  is satisfied; the failure is a generator⟷parser disagreement caused by REUSING the class name as the
  variable name under a type-first ordered choice.
- **FIX DIRECTION (pinned; the IMPLEMENT).** Give the witness generator **name diversity / store-aware free
  name selection**: when it renders a FREE declaring identifier (a producer like `variable_identifier`)
  whose canonical name already exists in the store as a `type_name` (or any fact a sibling ordered choice
  would greedily consume), pick a DISTINCT fresh name not in that store. This is the generation-side dual of
  the parser's disambiguation and stays parser-agnostic (keyed on the live store, not a rule name). ⚠️
  **BLAST RADIUS:** free-name selection feeds MANY witness samples, so this is higher-risk than the contained
  `.4b.x` prelude-discovery work — it needs a careful before→after GLOBAL cert across all grammars × seeds
  0/7/42 and the strict-subset gate, plus determinism confirmation. Candidate narrow scoping to de-risk:
  only diversify a free name when it collides with a same-named `type_name` fact AND sits after an
  empty-capable `data_type_or_implicit`-shaped optional (the exact `property_qualifier` shape), rather than
  diversifying all free names.
- **Acceptance (for the `.4b.8` IMPLEMENT, not this DESIGN slice):** SV cert `UNKNOWN` drops by
  `property_qualifier` (and any sibling the name-diversity unblocks), deterministic seeds 0/7/42, `spf=0`,
  strict SUBSET (ZERO newly-UNKNOWN); the 6 fully-certified grammars green; `cargo test --lib` green;
  `clippy_on_rust_change` source-clean. **This DESIGN slice is PURE-DOCS — no code — so the
  `check_diagnosis_evidence.sh` code-change gate does not apply; the `.4b.8` IMPLEMENT carries the enforced
  acceptance checklist it will earn.**

## `.4b.8` — IMPLEMENT: store-aware collide-aware free-name diversity (GENERATOR-ONLY)

- **Status:** `DONE (IMPLEMENT)` — `PGEN-STORE-AWARE-GEN-0014`, 2026-06-23, GENERATOR-ONLY. Landed exactly
  the `.4b.8` DESIGN. SV cert `UNKNOWN 38 → 37` (+1 witnessed: `property_qualifier`), deterministic seeds
  0/7/42, strict-subset (ZERO newly-UNKNOWN), 6 fully-certified grammars green. No grammar/parser/regen/
  release/schema/inventory/ledger change; SV row stays `Mostly Done` (closure-debt retirement).
- **What landed (generator-only, `rust/src/ast_pipeline/stimuli_generator.rs`).** Two helpers — the
  generation-side dual of the parser's type-vs-identifier disambiguation: `free_name_collides_gate_kind`
  (does a token already exist in the generation store under a fact kind some positive name-gate consumes? —
  keyed on the grammar's OWN `gen_name_gate` kinds, never a rule name) and
  `diversify_free_name_avoiding_gate_collision` (on collision, append the smallest `_<n>` clearing both the
  gate-kind collision and the active keyword exclusions, preserving leading/trailing layout so the
  escaped-identifier whitespace terminator survives). Hooked at the rule-level `@sample` literal-hint
  override in `generate_rule` (the path that renders the canonical `\foo `). DETERMINISTIC and RNG-NEUTRAL
  (no re-roll ⇒ the rest of the seed's stream is byte-identical), scoped to the witness pass
  (`reach_plan.is_some()`) and capability-gated on `gen_name_gate` non-empty. The producer's own
  declaration is naturally exempt — it emits its `type_name` fact only AFTER its render completes, so the
  store is empty at the class-name render (no collision) and populated at the later variable render
  (collision → diversified) — so `store_name_for_gate` read-back stays consistent without special-casing.
- **Acceptance Checklist (enforced — EARNED, oracle-cited):**
  - [x] **REPRODUCE / ISSUE** — `UNKNOWN=38` (seeds 0/7/42; `CERTIFICATE-COVERAGE … total=1289 proof=1 witness=1250 UNKNOWN=38 (sample_parse_failures=0, proof_reverify_failures=0)`); `property_qualifier` `DEBUG_PROBES` forced sample `class\foo ;rand\foo ;endclass` `parsed=false witnessed_target=false`.
  - [x] **ROOT CAUSE (WHY + WHERE)** — a witness-generator NAME COLLISION: `property_qualifier := random_qualifier` (`systemverilog.ebnf:4228`) + `variable_identifier := declaration_identifier` (`:5419`) are both UNGATED, so the witness names the reach-context class with the canonical `\foo` (emits `type_name` via `class_identifier` `@emit_fact` `:965`) AND re-renders the SAME `\foo` (the `@sample: "\\foo "` on `escaped_identifier` `:356`) as the free variable; since `\foo` is now a known type, PEG's `data_type_or_implicit := data_type | implicit_data_type` (`:1730`) parses `\foo` as the TYPE (first branch) and drops the mandatory `list_of_variable_decl_assignments`. `parseability_probe` A/B: `class\foo ;rand\foo ;endclass` REJECT `furthest_position=19` vs `class\foo ;rand\foo \bar ;endclass` PASS.
  - [x] **FIX** — generator-only collide-aware free-name diversity (Level-3+ general parser-agnostic capability; no grammar/parser/regen/release/schema bump). When a free declaring identifier re-renders a name already in the store under a name-gate-consumed fact kind, render a distinct `_<n>` variant (`\foo` → `\foo_0`). Keyed on `gen_name_gate` kinds; scoped to the witness pass; deterministic/RNG-neutral.
  - [x] **ADDRESSED (verified)** — SV cert `UNKNOWN 38 → 37` (decisive A/B on the regen-lockstep build); witness `1250 → 1251` (+1: `property_qualifier`, now `parsed=true witnessed_target=true`, sample `class\foo ;rand\foo_0 ;endclass`); `spf=0`, `proof_reverify_failures=0`; deterministic at seeds 0/7/42 (`UNKNOWN=37` byte-identical).
  - [x] **NO REGRESSION** — ZERO newly-UNKNOWN (the new 37 is a script-verified strict SUBSET of the baseline 38 — `comm -13` empty; the only rule that left is `property_qualifier`); regex `fully_certified=true 198/198` at seeds 0/7/42 (name-path inert — `gen_name_gate` empty); json (9/9) / vhdl (216/216) / svpp (74/74) / rtl_frontend (169) all `fully_certified=true` (rtl_const_expr `store_aware_gen=false` ⇒ inert, covered by the lib suite); `cargo test --lib --features "generated_parsers ebnf_dual_run"` **766 passed / 0 failed** (+1 locking test `store_aware_gen_diversifies_free_name_colliding_with_type_name_fact`); `clippy_on_rust_change` source-clean (zero findings in the changed code; the 188 generated `eq_op` errors pre-existing in `generated/*_parser.rs`).
  - [x] **LOCKSTEP** — CHANGES.md, DEVELOPMENT_NOTES.md, MEMORY.md, LIVE_ACHIEVEMENT_STATUS.md (SV cert 38→37), this leaf + the tree status (frontier → `.4b.9`), the book Grammar-Well-formedness SV-arc beat + cert number, decision record [[project_store_aware_generation]] note.
