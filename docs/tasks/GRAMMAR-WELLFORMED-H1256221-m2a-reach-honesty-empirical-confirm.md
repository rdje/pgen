# GRAMMAR-WELLFORMED.H.12.5.6.2.2.1 — M2a reach-honesty empirical confirmation + refined design (pre-implement)

The first sub-step of the M2a engine fix (`H.12.5.6.2.2`): with the rebuilt-from-source DEBUG
`ast_pipeline`, reproduce the decisive baseline, **empirically confirm the `-0113` WHY via the
reach-path dump** (the mandated "read the reach path before changing it"), and lock the implement
design against what the tools actually show. Tools-first investigation here **refined and partially
corrected** the `-0113` design, so the engine IMPLEMENT is split out as `.2.2.2`.

> Slice `PGEN-GRAMMAR-WELLFORMED-0114` (**PURE-DOCS** — NO code/grammar/generated/release/schema/
> ledger change; clippy not invoked). Status: `done`. Splits `H.12.5.6.2.2` → `.2.2.1` (this) +
> `.2.2.2` (the engine implement). Reads with [[feedback_why_and_where_before_solution]],
> [[feedback_tools_first_no_guessing]], [[feedback_no_codebase_change_without_tool_backed_facts]],
> [[feedback_ast_pipeline_parser_agnostic]], [[feedback_quality_over_speed_no_corners]],
> [[project_store_aware_generation]]. Parent design: `GRAMMAR-WELLFORMED-H125621-m2a-constraint-reach-honesty-whywhere.md`.

## Decisive baseline (A side — reproduced, deterministic)

`PGEN_REACH_PATH_DUMP=1 PGEN_CERT_COVERAGE_DUMP_ALL=1 ast_pipeline grammars/systemverilog.ebnf
--report-certificate-coverage --grammar-profile sv_2017 --entry-rule systemverilog_file --count 40
--seed 0` (features `generated_parsers,ebnf_dual_run`):

```
CERTIFICATE-COVERAGE: grammar='systemverilog' entry='systemverilog_file' samples=40
  total=1289 proof=1 witness=1204 UNKNOWN=84 fully_certified=false
  (sample_parse_failures=0, proof_reverify_failures=0)
```

Matches the layer-A pointer exactly. This is the A baseline any `.2.2.2` change is measured against.

## WHY — empirically confirmed by the reach-path dump (resolves `-0113` open-Q (a))

The `PGEN_REACH_PATH_DUMP=1` hop chains show the constraint cluster is split into two routes:

- **Gated, SHORTER (the bug):** `… → package_or_generate_item_declaration_sv_2017` **branch `o8`**
  `→ extern_constraint_declaration → extern_constraint_declaration_sv_2017 (root/s4) → constraint_block
  → constraint_block_item → constraint_expression → constraint_primary → …`. Every body rule under
  `constraint_block` inherits the `extern_constraint_declaration_sv_2017` predecessor.
- **Non-gated, LONGER (the witness path):** `… → package_or_generate_item_declaration_sv_2017`
  **branch `o9`** `→ class_declaration → class_declaration_sv_2017 → class_item → class_item_sv_2017
  → class_constraint → constraint_declaration (→ constraint_block)`.

`constraint_block` is the SHARED subtree. Both `extern_constraint_declaration_sv_2017` (o8) and the
in-class path reach it, but the extern route is **fewer hops**, so first-discovery-wins assigns
`constraint_block` (and its whole subtree) the extern predecessor. The installed reach directives then
force the o8 branch and render `extern_constraint_declaration_sv_2017`'s mandatory `class_scope`
sibling (`\foo ::`), which the parser correctly rejects (undeclared class) → the body rules never
witness. **Re-routing the discovery edge suffices** (the installed directives follow the discovered
predecessors), but the fix must overcome a route that is genuinely **shorter** — a same-hop-distance
tie-break (the `prefer_non_self_recursive_reference_sites` shape) is **insufficient**.

## Three findings that refine / partially correct the `-0113` design

### R1 — the re-routable subset is the `constraint_block` subtree only (NOT all ≈18; target ≠ ~66)

Only rules reachable via BOTH the extern and in-class routes can be re-routed to witness: the
`constraint_block` subtree (`constraint_block`, `constraint_block_item`, `constraint_expression`,
`constraint_primary(_sv_2017)`, `constraint_set`, `solve_before_list`, `uniqueness_constraint(_sv_2017)`,
`loop_variables`, `index_variable_identifier`, and the constraint-body `kw_*` — `kw_before`/`kw_soft`/
`kw_solve`/`kw_constant`/`kw_foreach`/`kw_within`/`kw_intersect`), all of which the in-class
`constraint_declaration_sv_2017` ALSO reaches (and which is already WITNESSED). **But
`extern_constraint_declaration(_sv_2017)` itself is extern-ONLY** — its witness REQUIRES rendering
`class_scope`, i.e. a declared class — so re-routing cannot witness it; it (and the
`known_unscoped_class_scope_*` store-gated identifiers) belong to **STORE-AWARE-GEN**, not reach-honesty.
⇒ the `-0113` "UNKNOWN 84 → ~66" target is OPTIMISTIC; the realistic re-route delta is the
`constraint_block`-subtree subset, with the extern-only + store-gated-identifier rules remaining for
STORE-AWARE-GEN.

### R2 — the detector's all-alternatives-gated rule DOES fire here (verified)

`extern_constraint_declaration_sv_2017` mandatorily renders `class_scope` (`grammars/systemverilog.ebnf:1068`)
→ `class_scope_type` (`:1065`) whose head is an `Or` of four identifier alternatives. **All four are
store-gated** (verified): `scoped_class_scope_identifier` (`:1047`, `@predicate lacks_fact_attribute_equals
… class`), `known_unscoped_class_scope_class_identifier` (`:1025`, `fact_attribute_equals … class`),
`…interface_class_identifier` (`:1028`, `… interface_class`), `…type_parameter_identifier` (`:1031`,
`… type_parameter`). So the conservative rule "an `Or` is gated iff EVERY alternative is gated" fires →
the **edge into `extern_constraint_declaration` is correctly classified store-gated** (the design's
"target rule transitively carries a predicate"), and excluding it in pass-1 leaves `constraint_block`
reachable only via the non-gated in-class route. ✓

### R3 — a SECOND mechanism: satisfiable-alternative selection (negative `lacks_fact`)

`scoped_class_scope_identifier` (`:1047` = `non_typedef_package_scope class_identifier`) gates on the
**negative** `lacks_fact_attribute_equals [type_name, $scope.body.name.body, declaration_family, class]`
— satisfied when the name is NOT a declared class, which an undeclared identifier IS. So a
`class_scope` rendering that PASSES its predicate plausibly exists (modulo `non_typedef_package_scope`'s
own gate, to be checked in `.2.2.2`), and the cluster's non-witnessing is influenced not only by
edge-routing but by **which `class_scope_type` head alternative the reach pass / generator selects**
(the `-0113` probe rendered the gated bare-class form `\foo ::`, not the negatively-gated scoped form).
This is a DISTINCT lever the `-0113` single-mechanism design did not account for; `.2.2.2` must analyze
both (edge-routing AND satisfiable-alternative selection) tools-first before coding — steering
`class_scope_type` to the satisfiable alternative might witness `extern_constraint_declaration` ITSELF
(which re-routing cannot).

## Safety property (de-risks `.2.2.2`)

`reach_hops` only SELECTS a reach path; the parser still adjudicates every probe. A wrong path choice
therefore **cannot create a false witness** — its worst case is "no improvement." The real risks are
(a) **de-witnessing** a rule that previously witnessed via a now-deprioritized edge, and (b) perturbing
the 6 fully-certified grammars — both caught by the required decisive A/B (SV cert seeds 0/7/42) +
byte-identical checks. Combined with the design's pass-2 fallback (all-edges BFS preserves
reachability), the scariest failure mode (false certification) is off the table by construction.

## Refined detector design (the `.2.2.2` implement spec)

GENERAL/parser-agnostic, keyed on annotations + the producer graph, NEVER rule names
([[feedback_ast_pipeline_parser_agnostic]]). Same bias-hook family as
`prefer_non_self_recursive_reference_sites` (`stimuli_generator.rs:5714`).

1. **Two-pass `reach_hops`:** pass-1 runs the BFS with store-gated edges EXCLUDED; if the target is
   reached, use that path; else fall back to pass-2 (the current all-edges BFS). Guarantees the
   non-gated path wins whenever one exists and is exactly inert when none does (the certified roster +
   every non-forked target).
2. **"Store-gated edge `R --site--> C`" detection:** `C` mandatorily (profile-aware, `Or`-aware =
   gated iff all alternatives gated, optional/star = non-propagating, cycle-guarded, transitively)
   renders a rule carrying a fact-query `@predicate` (`has_fact`/`fact_attribute_equals`/
   `fact_count_at_least`/their `lacks_*` duals) whose consulted fact-kind is NOT in the
   **emitted-on-path** set.
3. **emitted-on-path tracking:** extend the BFS `Discovery` struct with the cumulative set of fact-kinds
   emitted by rules on the discovery path to each node (predecessor's set ∪ this hop's emitted kinds).
   Reuse `grammar_wellformedness::collect_emitted_fact_kinds` (emitter enumeration) and
   `consulted_kinds_in_predicate(name, args)` (consulted-kind extraction) — the F1 machinery, NOT a
   second predicate walker.

**Corrected acceptance for `.2.2.2`:** rebuild DEBUG `ast_pipeline`; decisive A/B SV cert seeds 0/7/42
— the `constraint_block`-subtree subset moves `UNKNOWN`→witnessed (delta is the subtree count, NOT all
≈18; `extern_constraint_declaration` + `known_unscoped_class_scope_*` stay `UNKNOWN` for
STORE-AWARE-GEN), `spf=0`, no newly-`UNKNOWN`; the 6 fully-certified grammars byte-identical (stash
A/B); `--generate-stimuli` + cross-family + oracle byte-identical; GENERATOR-only ⇒ NO release/schema/
ledger bump; lockstep = grammar-wellformedness book SV-arc beat + continuity docs.

## Open questions handed to `.2.2.2`

- (a) **CONFIRMED** by the reach dump: re-routing the discovery edge suffices (directives follow the
  discovered predecessors) — verify the installed plan with `PGEN_REACH_PATH_DUMP=1` post-change.
- (b) per-rule gated context for `constant_cast` / `property_qualifier` (may differ from the constraint
  context — confirm each shares the single in-class re-route or needs its own check).
- (c) whether `loop_variables` / `index_variable_identifier` (inside `constraint … foreach(\foo[]) …`)
  ride the same re-route once the enclosing constraint is non-gated.
- (d) **NEW (R3):** does steering `class_scope_type` to the satisfiable `scoped_class_scope_identifier`
  alternative witness `extern_constraint_declaration` itself — a lever re-routing cannot reach? Settle
  whether `.2.2.2` is edge-routing only, alternative-selection only, or both; change ONE thing first,
  measure GLOBAL.

## Verification (read-only — PURE-DOCS)

Baseline reproduced deterministically (the summary above). Reach-path dump captured and read
(`/tmp/h125622_baseline_seed0.log`). All grammar loci (`extern_constraint_declaration_sv_2017` `:2055`,
`constraint_declaration_sv_2017` `:1395`, `class_scope` `:1068`, `class_scope_type` `:1065`, the four
gated identifiers `:1025`/`:1028`/`:1031`/`:1047`) and engine loci (`reach_hops` `:5606`,
`prefer_non_self_recursive_reference_sites` `:5714`, `set_reach_plan_for_rule` `:2710`,
`generate_plannable_rule_witnesses` `:2947`; F1 `grammar_wellformedness.rs:927`/`:958`) confirmed by
read. **NO code/grammar/generated/release/schema/ledger change.** SV stays the only non-fully-certified
shipped grammar (`UNKNOWN=84`).

## Artifacts (scratch — not tracked)

`/tmp/h125622_baseline_seed0.log` — the seed-0 cert + reach-path dump underpinning the WHY confirmation.
