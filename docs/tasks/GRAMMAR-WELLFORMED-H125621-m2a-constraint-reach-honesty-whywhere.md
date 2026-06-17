# GRAMMAR-WELLFORMED.H.12.5.6.2.1 — M2a constraint reach-honesty WHY+WHERE + fix DESIGN

The mandated tools-first WHY+WHERE for the M2a constraint cluster (`.6.1`-adjudicated), **read the
reach path BEFORE changing it**. Pins the exact engine + grammar loci and the fix direction; the
engine change itself is the IMPLEMENT child `.6.2.2`.

> Slice `PGEN-GRAMMAR-WELLFORMED-0113` (**PURE-DOCS DESIGN** — NO code/grammar/generated/release/
> schema/ledger change; clippy not invoked). Status: `done`. Splits `H.12.5.6.2` → `.2.1` (this) +
> `.2.2` (the engine fix). Reads with [[feedback_why_and_where_before_solution]],
> [[feedback_tools_first_no_guessing]], [[feedback_no_codebase_change_without_tool_backed_facts]],
> [[feedback_ast_pipeline_parser_agnostic]], [[feedback_grammar_rules_must_consult_store]].

## WHY (decisive, tool-confirmed)

The ≈18 M2a constraint/sequence/property rules (`constraint_block_item`, `constraint_expression`,
`constraint_primary(_sv_2017)`, `constraint_set`, `extern_constraint_declaration(_sv_2017)`,
`solve_before_list`, `uniqueness_constraint(_sv_2017)`, `constant_cast`, `property_qualifier`,
`loop_variables`, `index_variable_identifier`, `hierarchical_array_identifier`,
`ps_or_hierarchical_array_identifier`, + constraint `kw_before`/`kw_foreach`/`kw_soft`/`kw_solve`)
all share the rejected witness context `constraint \foo ::\foo { … }` (`.6.1`). The grammar reading
pins which rule produces it and why the generator picks it:

- The rejected shape is **`extern_constraint_declaration_sv_2017`** (`grammars/systemverilog.ebnf:2055`):
  `( kw_static )? kw_constraint class_scope constraint_identifier constraint_block` — the **out-of-class**
  constraint definition. Its `class_scope` (`:1068` = `class_scope_type scope_resolution`, i.e. `C::`)
  is **store-gated** — `class_scope_type` (`:1065`) requires `known_unscoped_class_scope_class_identifier`
  (`:1025`) etc., a *declared class*. The minimal witness renders an undeclared `\foo`, so the gate
  fails and the parser never enters `constraint_block` → none of the body rules witness.
- The **non-gated alternative exists**: `constraint_declaration_sv_2017` (`:1395`) =
  `( kw_static )? kw_constraint constraint_identifier constraint_block` — the **in-class** form, no
  `class_scope`, reached via `class_constraint` (`:909`) ← `class_item` (`:976`/`:986`). It needs no
  store fact, and it reaches the SAME `constraint_block → constraint_block_item → constraint_expression`
  subtree. Proven to parse + witness in `.6.1` (`class C; rand int x; constraint c { x < 5; } endclass`,
  `unique {x,y};`, `solve x before y;` all PASS).

So the bug is a **reach-path selection**: a witness EXISTS (the in-class path), so by the attribution
rule this is a generator-reach deficiency, not a parser bug and not a dead branch.

## WHERE — `reach_hops` is a SHORTEST-path BFS that prefers the gated rule

`reach_hops(entry_rule, target_rule)` (`rust/src/ast_pipeline/stimuli_generator.rs:5606`) is a
**breadth-first search over the rule-reference graph**, first-discovery-wins (each rule enqueued once
via `discovered`), so it recovers the **fewest-hops** path. The two routes to the constraint body:

- **out-of-class (gated, SHORTER):** `systemverilog_file → … → description → … →
  package_or_generate_item_declaration(_sv_2017)` (refs `extern_constraint_declaration` at `:3584`/`:3601`)
  `→ extern_constraint_declaration → …_sv_2017 → constraint_block → constraint_block_item → constraint_expression`.
- **in-class (non-gated, LONGER):** `… → class_declaration → class_item(_sv_2017)` (refs `class_constraint`
  at `:976`/`:986`) `→ class_constraint → constraint_declaration → …_sv_2017 → constraint_block → …`.

The in-class route must first descend through `class_declaration → class_item` — strictly more hops
than the top-level package-item route — so the BFS discovers `constraint_block` (and every body rule
under it) through `extern_constraint_declaration` and installs a reach plan that forces the
store-gated out-of-class form. The reach probe then renders `constraint \foo ::\foo { … }`, which the
parser correctly rejects.

There is already a **precedent for biasing this BFS's site selection**:
`prefer_non_self_recursive_reference_sites` (`:5637` call, `:5714` def, RTL-FE-CLOSURE.5.3) — a stable
sort of the collected reference sites by a 0/1 structural key, applied inside `reach_hops` before
enqueue. The fix is the same shape, on a different key.

## Fix DESIGN (→ `.6.2.2`, the engine IMPLEMENT child)

**Direction:** make `reach_hops` (the plannable-reach BFS) **deprioritize reference edges that cross a
store-gated rule whose consulted fact-kind no on-path producer can establish**, so a non-gated path is
preferred even when it is a hop or two longer. Then the constraint body rules are discovered through
the in-class `constraint_declaration`, the reach probe renders `class C; constraint c { … } endclass`
(non-gated), and the parser witnesses them.

**Two candidate mechanisms (settle in `.2.2`, tools-first, ONE change):**

1. **Two-pass BFS** — run `reach_hops` once with store-gated edges *excluded*; if the target is
   reached, use that path; else fall back to the current all-edges BFS. Simplest; guarantees the
   non-gated path wins whenever one exists, and is exactly inert when none exists (the certified roster
   + every non-forked target).
2. **Weighted BFS** — give store-gated edges a higher cost (Dijkstra over the reference graph). More
   general (handles "least-gated" when no fully-non-gated path exists) but heavier; defer unless pass-1
   proves insufficient.

**"Store-gated edge" detection (must be GENERAL/parser-agnostic — keyed on annotations + the producer
graph, NEVER rule names, per [[feedback_ast_pipeline_parser_agnostic]]):** a reference whose target
rule (shallowly/transitively) carries a `@predicate has_fact`/`fact_attribute_equals`/`fact_count_at_least`
on a fact-kind for which **no `@emit_fact` producer is reachable on the path built so far** — i.e. the
exact "binding-before-use can't be satisfied by the minimal witness" condition the cert model already
names. Re-use the linter's emitter/consumer fact-kind machinery (the `F1` binding-before-use check)
rather than inventing a second predicate walker.

**Acceptance / proof obligations for `.2.2`:**
- Decisive A/B on the rebuilt DEBUG `ast_pipeline`: SV cert at seeds 0/7/42 — the ≈18 M2a rules move
  `UNKNOWN`→witnessed (target: `UNKNOWN 84 → ~66`), `spf=0`, no newly-`UNKNOWN`.
- **The 6 fully-certified grammars stay byte-identical** (`json`/`regex`/`rtl_const_expr`/`svpp`/`vhdl`/
  `rtl_frontend`) — they don't run the plannable pass once `fully_certified`, and have no gated-vs-nongated
  constraint fork; confirm via stash A/B (cert + `spf` byte-identical).
- No other SV witness regresses (the change only ever re-routes a *gated* discovery to a non-gated one;
  a target with only a gated path is byte-identical — pass-1 falls back).
- `--generate-stimuli` and the cross-family / oracle gates are byte-identical (reach behaviour is the
  witness pass only, opt-in to cert-coverage — the C2.2/H.4.2 invariant).
- GENERATOR-only (no grammar/parser change) ⇒ NO release/schema/ledger bump; lockstep = grammar-
  wellformedness book SV-arc beat + the usual continuity docs.

**Open questions handed to `.2.2`:** (a) does the directive chain need an explicit force to *avoid*
`extern_constraint_declaration` at the `package_or_generate_item_declaration` OR node, or does
re-routing the discovery edge suffice (the installed directives follow the discovered predecessors, so
re-discovery should suffice — verify with `PGEN_REACH_PATH_DUMP=1`); (b) confirm all ≈18 M2a rules
share the single in-class re-route (some — e.g. `constant_cast`, `property_qualifier` — may have a
distinct gated context needing its own check); (c) whether `loop_variables`/`index_variable_identifier`
(inside `constraint … foreach(\foo[]) …`) ride the same re-route once the enclosing constraint is
non-gated.

## Verification (read-only — PURE-DOCS)

All loci confirmed by grammar + source read and the `.6.1` `parseability_probe` A/B (in-class +
declared-class constraint forms PASS; undeclared out-of-class REJECTS). NO code/grammar/generated/
release/schema/ledger change. SV stays the only non-fully-certified shipped grammar (`UNKNOWN=84`).

## Artifacts (scratch — not tracked)

`/tmp/h1256/` (from `.6.1`) — the constraint probe samples + cert logs underpinning the WHY.
