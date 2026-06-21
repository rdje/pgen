# GRAMMAR-WELLFORMED.H.12.5.6.2.2.2 — M2a reach-honesty ENGINE IMPLEMENT (store-free reach pass)

The engine implement of the M2a constraint reach-honesty fix designed in `.2.2.1` (`-0114`) and
`.2.1` (`-0113`): make the certificate-coverage reach machinery witness the SystemVerilog
constraint-body cluster by routing it through the **non-gated in-class** `constraint_declaration`
instead of the **store-gated out-of-class** `extern_constraint_declaration`.

> Slice `PGEN-GRAMMAR-WELLFORMED-0115` (GENERATOR-ONLY — no grammar/release/schema/ledger change).
> Status: `done`. Reads with [[feedback_no_codebase_change_without_tool_backed_facts]],
> [[feedback_tools_first_no_guessing]], [[feedback_ast_pipeline_parser_agnostic]],
> [[feedback_always_signoff_decisions]], [[project_cert_coverage_tournament_loser_leak]],
> [[project_store_aware_generation]], [[feedback_quality_over_speed_no_corners]].
> Parent design: `GRAMMAR-WELLFORMED-H1256221-m2a-reach-honesty-empirical-confirm.md`.

## WHY (confirmed by `-0114` reach-path dump)

`reach_hops` (`stimuli_generator.rs`) is a fewest-hops BFS with first-discovery-wins. The
`constraint_block` subtree is reachable BOTH via:
- the **store-gated** out-of-class route `package_or_generate_item_declaration_sv_2017 (o8) →
  extern_constraint_declaration → … → constraint_block`, which mandatorily renders `class_scope`
  (`class_scope_type`'s head `Or` — all four identifier alternatives carry a fact-query `@predicate`
  on `type_name`, so it needs a DECLARED class a minimal witness cannot provide); and
- the **non-gated** in-class route `… → class_declaration → … → constraint_declaration_sv_2017 →
  constraint_block`, which parses store-free.

The extern route is FEWER hops, so first-discovery-wins assigned the whole subtree the gated
predecessor; the reach plan then forced the extern branch and the parser correctly rejected the
undeclared-class `\foo ::` form → the body rules never witnessed.

## WHAT (the implement — two design iterations, tools-first)

### Iteration A (two-pass `reach_hops`) — rejected after measurement

First cut made `reach_hops` two-pass: pass 1 excludes store-gated edges, pass 2 (fallback) is the
original all-edges BFS. Decisive A/B (SV cert seed 0): `UNKNOWN 84 → 67`, `spf=0`, but **one
newly-UNKNOWN**: `known_unscoped_checker_identifier` (a store-gated rule, `@predicate
has_fact(checker_name, $body)`). Tools-first root cause: that rule was only ever **incidentally**
(bystander) witnessed by a *different* target's all-edges probe; rerouting ALL targets changed that
probe's sample and dropped the bystander coverage (its own probe can never witness it — it is
store-gated and needs STORE-AWARE-GEN, like its persistent-UNKNOWN siblings). Deterministic across
seeds 0/7/42. Per the director's strict no-regression doctrine
([[project_cert_coverage_tournament_loser_leak]] — a net-positive fix that de-witnessed rules was
REVERTED), a single newly-UNKNOWN is unacceptable → iteration A rejected.

### Iteration B (strictly-additive store-free pass) — LANDED

`reach_hops` reverted to the ORIGINAL all-edges BFS, so every existing pass (diverse, recursive-reach,
plannable, target-own) keeps its EXACT RNG stream and witness landscape — all bystander coverage
preserved, **no newly-UNKNOWN by construction**. The store-gated-edge deprioritization became a
SEPARATE, strictly-additive **store-free reach pass** run LAST, over ONLY the rules still UNKNOWN
after every other pass. It re-routes each residual target through the non-gated carrier and can only
UNION new witnesses (the parser adjudicates every probe ⇒ never a false witness). This mirrors the
existing additive passes (`generate_target_own_structure_witnesses`) and the project-wide "the reach
pass only ever unions witnesses" guarantee.

Implementation (all `rust/src/ast_pipeline/stimuli_generator.rs` + the `rust/src/main.rs` driver):
- `reach_hops_pass(entry, target, exclude_store_gated_edges)` — the BFS body, with an
  `emitted_available` set per `Discovery` (ancestors' `@emit_fact` kinds) and, in pass-1 mode, an
  edge skip when the target is mandatorily forced through a store-gate consulting a kind not on the
  path. `reach_hops` = `reach_hops_pass(.., false)` (original behaviour, used by `compute_reach_path`).
- `compute_reach_gate_kinds(annotations)` — precomputes per-rule fact-query-predicate consulted
  kinds (the 5 kind-first primitives `has_fact`/`lacks_fact`/`fact_attribute_equals`/
  `lacks_fact_attribute_equals`/`fact_count_at_least`), mirroring `compute_store_aware_gen_directives`'
  per-rule flatten + the same `parse_semantic_runtime_directives` parse. Stored in the new
  `reach_gate_kinds` field. (F1's `consulted_kinds_in_predicate` was NOT reused: its
  `FACT_QUERY_PRIMITIVES` set omits `lacks_fact_attribute_equals`, which the scoped class-scope head
  alternative uses — it would have missed the gate.)
- `edge_is_store_gated` / `mandatory_reach_gate` / `mandatory_node_gated` — the mandatory-subtree
  gate detector: Or gated iff EVERY alternative is gated, Sequence gated iff ANY element is, `?`/`*`
  non-propagating, lookahead-skipping, cycle-guarded, transitive, profile-aware (a reference to a
  profile-pruned rule is treated as gated — it cannot be rendered, so it is no escape).
- `set_reach_plan_for_rule_mode(.., store_free)` — the reach-plan setter selects the BFS variant; the
  pub `set_reach_plan_for_rule` keeps its signature (delegates with `false`).
- `run_plannable_witness_pass(.., store_free, ..)` — the shared plannable body; `generate_plannable_
  rule_witnesses` (all existing/test callers unchanged) and the new
  `generate_plannable_store_free_witnesses` are thin wrappers. **Guarded truly inert**: when
  `store_free && reach_gate_kinds.is_empty()` it returns immediately (a store-free probe can differ
  from all-edges only when a fact-query predicate exists), so predicate-free grammars pay ZERO extra
  cost.
- `main.rs` cert driver: after the target-own pass, runs the store-free pass over the post-target-own
  residual UNKNOWN (only when non-empty).

GENERAL/parser-agnostic — keyed purely on the producer graph + annotations, never rule names. Bias-hook
family of `prefer_non_self_recursive_reference_sites`.

## VERIFICATION (decisive A/B, deterministic)

DEBUG `ast_pipeline` (`--features "generated_parsers ebnf_dual_run"`), cert `--count 40`.

- **SystemVerilog** (`--grammar-profile sv_2017 --entry-rule systemverilog_file`):
  - A (baseline): `total=1289 proof=1 witness=1204 UNKNOWN=84 spf=0`.
  - B (after): `total=1289 proof=1 witness=1221 UNKNOWN=67 spf=0` — **deterministic at seeds 0/7/42**.
  - `UNKNOWN 84 → 67` (−17, witness +17); **newly-UNKNOWN = ∅** (no regression); `spf=0`.
  - Store-free pass: "84 residual UNKNOWN rules targeted; 17 witnessed by re-routing through a
    non-gated carrier".
  - 17 closed rules (the constraint-body cluster + adjacents): `constraint_block_item`,
    `constraint_expression`, `constraint_primary`, `constraint_primary_sv_2017`, `constraint_set` had
    been the optimistic extra; the landed set is `constraint_block_item`, `constraint_expression`,
    `constraint_primary(_sv_2017)`, `loop_variables`, `index_variable_identifier`, `solve_before_list`,
    `uniqueness_constraint(_sv_2017)`, `kw_before`/`kw_foreach`/`kw_or`/`kw_soft`/`kw_solve`,
    `hierarchical_array_identifier`, `ps_or_hierarchical_array_identifier`, `property_actual_arg`.
  - Per R1 (`-0114`): `extern_constraint_declaration(_sv_2017)` + the `known_unscoped_class_scope_*`
    store-gated identifiers correctly REMAIN UNKNOWN (extern-only / store-gated ⇒ STORE-AWARE-GEN, not
    reach-honesty). `constraint_set` also remains (its store-free probe did not witness it minimally) —
    a residual, not a regression.
- **The six fully-certified grammars BYTE-IDENTICAL** (seed 0): `json` 9/9, `regex` 198/198
  (the predicate-bearing grammar — exercises `compute_reach_gate_kinds`, stays fully certified),
  `rtl_const_expr` 48/48, `systemverilog_preprocessor` 74/74, `vhdl` 216/216, `rtl_frontend` 169
  (1 proof + 168 witness) — all `fully_certified=true`, `spf=0`. The store-free pass never runs for
  them (empty residual and/or empty `reach_gate_kinds`).
- **Ordinary generation byte-identical by construction**: the new field is read only on the
  cert-only store-free path; `reach_hops` is the original all-edges BFS; no change touches
  `generate_from_entry`. So `--generate-stimuli`, cross-family, and oracle surfaces are unaffected.
- **Clippy**: `make -C rust SHELL=/opt/homebrew/bin/bash clippy_on_rust_change` ✅ — strict source
  lint clean (the 188 generated-parser errors are the known deferred generated-clippy debt, non-strict
  stage; none in the edited source).

## OUTCOME / FRONTIER

SystemVerilog `UNKNOWN 84 → 67`; still the only non-fully-certified shipped grammar. M2a constraint
reach-honesty is CLOSED for the re-routable `constraint_block`-subtree subset. Remaining M2/M3:
`H.12.5.6.3` (M2b store-gated, parked → STORE-AWARE-GEN), `H.12.5.7` (M3 plannable property/sequence
temporal operators), and lane 2 `H.12.5.8` (the SVA sequence/property binary-operator parse bug).

## Artifacts (scratch — not tracked)

`/tmp/h1256222_*` — the A/B cert logs + reach dumps underpinning the numbers above.
