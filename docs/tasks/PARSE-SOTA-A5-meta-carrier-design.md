# PARSE-SOTA.11 (adoption A5) — `_meta` carrier: grounded design + phased plan

> Owner leaf: `PARSE-SOTA.11`. Design/scoping slice (pure docs). Commit: `PGEN-PARSE-SOTA-0011`.
> The `_meta` carrier was director-APPROVED ([[feedback_meta_carrier_design]], Option A) and
> its implementation **explicitly deferred to a fresh-context session** because it is
> disruptive. This doc pins the real implementation surface (tools-first) + a phased,
> de-risked rollout so the codegen change can land cleanly rather than wholesale.

## What A5 is
Emit an **additive `_meta` sibling key** on every typed AST object, carrying
`span` / `line_col` / `rule` / `branch_index` / `source_text` / `trivia`. Roslyn / rowan /
SwiftSyntax full-fidelity model (PARSE-SOTA research §E); enables round-trip / IDE / linter
uses and unlocks A4's deferred `parse(node._meta.source_text)` re-parse oracle. **Schema
stays 1** (additive sibling key; older consumers ignore it).

## Implementation surface — TWO surfaces (tools-first, file:line)
PGEN builds typed AST objects in **two places**, and `_meta` must attach in BOTH (PGEN's
two-surface architecture):
1. **Runtime interpreter** — `rust/src/ast_pipeline/unified_return_ast.rs` (object build at
   `:636`–`:706`, `let mut map = serde_json::Map::new(); … Ok(Value::Object(map))`). Used by
   the bootstrap path. A change here is runtime (shared lib), no regen, but affects every
   object the interpreter builds.
2. **Codegen** — `rust/src/ast_pipeline/return_annotation_handler.rs` (emits the object-
   construction Rust into the generated parsers; `:355` shows `ParseNode { … span: 0..0 }`).
   A change here requires **regenerating all 10 `generated/*_parser.rs`**.
A consistent `_meta` requires editing BOTH surfaces in lockstep (else the bootstrap and the
generated parsers disagree) — this is the core reason A5 is a coordinated, multi-slice
effort, not a single edit.

## Original single-site note (superseded by the two-surface finding above)
- **Object construction:** `rust/src/ast_pipeline/ast_based_generator.rs:6774` — the
  `key: "kind"` insertion is where each typed object's fields are built; `_meta` attaches
  here (one more key per object).
- **Span source:** `ParseNode` carries a `span` field, BUT it is frequently `0..0`
  (placeholder, e.g. `return_annotation_handler.rs:355` emits `span: 0..0`). So
  `_meta.span` / `line_col` / `source_text` need **real span population** to be meaningful
  — today's spans are not reliably populated.
- **No existing `_meta` scaffolding** anywhere (grep-confirmed).
- **Blast radius:** adding `_meta` to every object changes `expected_json_object_keys_present`
  for every rule in **all 8 `test_data/ast_shape_contract/*_v1.json` manifests** + the
  `ast_shape_contract.rs` running-parser test, and requires regenerating **all 10
  `generated/*_parser.rs`** parsers. (This is the disruption the approval note flagged.)

## Why NOT wholesale-now
A single commit that (a) changes codegen to emit `_meta` on every object, (b) populates
real spans, (c) regenerates 10 parsers, and (d) migrates 8 shape contracts + the test is a
large, coupled change with high regression risk — the opposite of the one-thing-at-a-time,
measure-each-step discipline. The approved design deferred it for exactly this reason.

## Phased plan (de-risked: additive + opt-in first)
- **`.11.1` — `_meta` emission, OPT-IN (default OFF).** Thread a codegen/generation flag
  (e.g. `emit_meta: bool` in the generation config, default false). When ON, the object
  builder at `:6774` adds a `_meta` sibling (`rule`, `branch_index`, `span` from
  `ParseNode.span`). When OFF (default), output is **byte-identical** to today → ZERO blast
  radius on existing behavior, shape contracts, and the 10 parsers. Verify: a focused test
  that the flag-on output carries `_meta` and flag-off is unchanged; `make focus_regex`
  green (default path untouched). LOW RISK, completable as one slice.
- **`.11.2` — real span population.** Ensure `ParseNode.span` is populated with true byte
  ranges where it is currently `0..0`, so `_meta.span` / `line_col` / `source_text` are
  accurate. Verify against known inputs. (Engine-adjacent; its own slice.)
- **`.11.3` — default-ON + contract migration (the disruptive coordination).** Flip the
  default, regenerate all 10 parsers, update all 8 shape-contract manifests' object-key
  expectations + the shape-contract test, bump nothing (schema stays 1, additive). Verify
  EACH grammar via `make focus_<grammar>` + the shape-contract test + the round-trip gates.
  This is the coordinated slice the approval note meant by "fresh-context session".
- **`.11.4` — unlock A4's oracle.** Add the per-node `parse(node._meta.source_text)`
  re-parse round-trip test (deferred from A4).

## Recommendation
Implement `.11.1` (opt-in, safe, additive) as the next focused slice; schedule `.11.2`/
`.11.3` as a dedicated, coordinated effort (the heavy, contract-migrating part) with the
full `make focus_*` + shape-contract + round-trip verification battery — NOT bundled at the
tail of an unrelated session. Until then the carrier is available opt-in for consumers, and
the default output (and every shape contract) stays exactly as today.

## 2026-06-30 re-grounding (tools-first; line refs above were stale, and `.11.1` is heavier than "one slice")
A fresh tools-first pass against HEAD updates the surface map (the `:6774` and
`return_annotation_handler.rs:355` line refs above are from 2026-06-02 and have drifted):

- **The full-codegen typed-object FUNNEL is a single site:**
  `ast_pipeline/ast_return_transform.rs::generate_object_transform` (≈`:364`), which builds
  `{ let mut __pgen_obj = serde_json::Map::new(); …; ParseContent::Json(serde_json::Value::Object(__pgen_obj)) }`.
  It is reached from ONE codegen entry — `ast_based_generator.rs:4165`
  (`generate_return_transform` → `AstReturnTransformer::generate_transform(ast, captured_vars, rule_name)`).
  The real `rule_name` is supplied only at that top-level entry; the file-internal recursive
  `generate_transform` calls (≈`:219/:265/:336/:484/:495/:520/:544`) pass `""`, so nested
  object literals would carry an empty `_meta.rule` unless the name is threaded down.
- **`return_annotation_handler.rs` is the BOOTSTRAP-mode handler** (limited subset; emits
  `ParseContent::Terminal(r#"…"#)` strings, NOT the typed `{kind,…}` carrier) — a distinct
  surface from the full codegen.
- **The runtime interpreter `unified_return_ast.rs` builds objects at ≈`:661–753`** (~6
  `serde_json::Map::new()` → `Value::Object` sites) — the third surface.
- **Why `.11.1` is NOT a clean "one slice / zero blast radius" change:** the `emit_meta`
  flag's natural source is the `AstBasedGenerator` struct, which has **51 explicit
  `AstBasedGenerator { … }` construction sites** (no `..Default::default()`), so a field add
  touches all 51; and a coherent `_meta`-on-every-typed-object must move all THREE surfaces
  in lockstep. The "zero blast radius" is true of the default-OFF OUTPUT only, not the CODE
  change. ⇒ `.11.1` is the leading edge of the disruptive A5 coordination this design (and
  the [[feedback_meta_carrier_design]] approval) deferred to a dedicated session — it is not
  a safe autonomous-loop slice. Schedule it deliberately. ([[feedback_no_codebase_change_without_tool_backed_facts]].)
