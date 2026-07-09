# SCOPE-CONTEXT-PREDICATE — a general `@predicate` scope-ancestry gate (`in_scope_kind` / `not_in_scope_kind`)

## Metadata

- Tree ID: `SCOPE-CONTEXT-PREDICATE`
- Status: `active` — **`.1` the primitive DONE** (2026-07-10, session #84, `PGEN-SCP-0001`); landed +
  proven in isolation. The downstream CONSUMER (`REGEX-PCRE2-FIDELITY.4.10` — reject `\K` inside a
  lookaround) is a leaf of a different tree, so the FRONTIER passes there (the RSVC model).
- Family / slice-id prefix: `PGEN-SCP-<NNNN>` (abbreviation of the tree name; used in commit subjects)
- Roadmap lane: cross-cutting engine correctness — a **parser-agnostic** scope-context predicate that
  lets the EBNF gate a rule on **lexical containment** ("am I currently inside an open scope of kind
  K, at any nesting depth?"), so a grammar can own accept/reject rules that today only an out-of-band
  Rust validator can express. This is the general "reject construct X anywhere inside enclosing
  construct Y" capability (`\K` in a lookaround, `return` outside a function, `break` outside a loop,
  a nested-`atomic` ban, …). Aligns with the Annotation-Driven Semantic Steering Doctrine and the
  EBNF-single-source-of-truth doctrine ([[project_ebnf_is_single_source_of_truth]]). Direct sibling
  of the `RULE-SPAN-VALUE-CONSTRAINT` primitive tree.
- Director authorization: 2026-07-09 (recorded in the RSVC tree + `MEMORY.md`) — building the engine
  primitives the remaining hard `REGEX-PCRE2-FIDELITY.4` families need, to SOTA/signoff quality, was
  explicitly AUTHORIZED as the shared unlock; the `.4.10` leaf's stated need is "a contextual gate
  primitive", which is exactly this. Same authorization umbrella as RSVC (which names `.4.9` as a
  consumer); this tree is the contextual-gate analogue for `.4.10`.
- Fix-hierarchy justification (tier-5 engine, tool-backed): the proving consumer (`\K`-in-lookaround)
  is a **lexical-containment** question — "is this `\K` anywhere inside an open lookaround body?".
  Tools-first exhaustion of the lower tiers (see §Design):
  - tier 1/2 (existing annotations / store): the only scope predicate today is `current_scope_is`,
    which sees ONLY the innermost scope frame — it misses a `\K` nested inside a plain group inside
    the lookaround (`(?=a(b\Kc))`, oracle-REJECT). It would work only under the fragile, non-local,
    unenforced invariant "lookarounds are the sole scope-openers in this grammar" — a hidden global
    coupling (a `NO WORKAROUNDS` violation, [[feedback_no_workarounds_fix_hierarchy]]).
  - The fact predicates (`has_fact`/`lacks_fact`) are **global and monotonic** — `close_scope` does
    NOT retract facts (`semantic_runtime.rs:2706-2748` pops the frame but never truncates `self.facts`)
    — so a fact emitted at lookaround-OPEN leaks to later siblings and `\K` AFTER the lookaround
    (`(?=ab)\K`, oracle-ACCEPT) would be wrongly rejected. `has_fact_in_current_scope` keys off an
    exact integer depth, so it collides with nested groups.
  - tier 3/4 (new annotation / new store schema): not an annotation-shape or fact-schema gap.
  - tier 5 (engine): a built-in predicate that walks the live scope chain (`active_chain`) and tests
    each node's `kind` is the minimal general primitive that answers the containment question
    correctly AND auto-unwinds on `close_scope` (the chain shrinks). The infrastructure already
    exists (`active_chain()` `:2235`, `scope_node()` `:2220`, `ScopeNode.kind` `:2689`); the arm is
    a few lines.

## Goal

Expose two **general, parser-agnostic** scope-context predicates on the existing `@predicate` surface:

```ebnf
@predicate: { name: in_scope_kind,     args: [<kind>], phase: pre }   # gate PASSES when inside a <kind> scope
@predicate: { name: not_in_scope_kind, args: [<kind>], phase: pre }   # gate PASSES when NOT inside a <kind> scope
```

- `<kind>` is a scope-kind identifier (`SemanticScopeKind`: `global`/`file`/`package`/`class`/…, or
  any `Custom(String)` such as `lookaround`).
- `in_scope_kind(kind)` returns `Some(true)` iff ANY currently-open (active) scope — the innermost
  frame OR any enclosing ancestor up to and including root — has that kind; `Some(false)` otherwise;
  `None` if the kind arg is missing/empty (mirrors `current_scope_is`'s `?`-on-malformed convention).
- `not_in_scope_kind(kind)` is the boolean complement (the `has_fact`/`lacks_fact` pairing precedent),
  so a grammar can express the common "reject X inside Y" gate without a `@predicate_def` wrapper.

It is the whole-active-chain generalization of the existing innermost-only `current_scope_is`.

## Non-Goals

- **Not** a name-filtered ancestry query (`in_scope_kind(kind, name)`). `current_scope_is` supports an
  optional name arg; the ancestry primitive is kind-only for now (YAGNI — no consumer needs a
  name-filtered containment check). A `(kind, name)` extension can layer on later if a consumer
  appears; the kind-only shape is documented so that addition is non-breaking.
- **Not** a fact retraction / scope-scoped fact model. Facts stay global+monotonic; this primitive
  reads the SCOPE chain, which already unwinds on `close_scope`. (The "scoped fact that auto-retracts
  on close" idea is out of scope — the scope-chain read is the correct, simpler mechanism here.)
- **Not** the regex consumer itself. Wiring `@open_scope: {kind: lookaround}` onto the lookaround
  open-markers and gating `\K` with `not_in_scope_kind(lookaround)` is `REGEX-PCRE2-FIDELITY.4.10`, a
  leaf of a different tree that CONSUMES this primitive after it is proven in isolation.

## Acceptance Criteria

- The primitive is proven **in isolation, BEFORE any shipped consumer**: new `semantic_runtime.rs`
  unit tests exercise `in_scope_kind`/`not_in_scope_kind` directly on a `SemanticRuntimeState` with
  hand-built nested scopes, asserting the exact `\K`-in-lookaround semantics:
  - true at any nesting depth inside the target-kind scope (incl. inside a deeper non-target scope);
  - false at root / outside;
  - false again AFTER the target scope is closed (auto-unwind — the key property facts lack);
  - `in_scope_kind(global)` always true (root is always open);
  - malformed/empty kind → `None`.
- Both names are registered in `ENGINE_BUILTIN_PREDICATE_NAMES` (so a `@predicate_def` may not shadow
  them, V-QDEF-1) and dispatched by the shared `evaluate_predicate` (so BOTH the interpreter and the
  generated parser get them with no codegen change — the dispatch is a single shared method).
- NO regression: the semantic/return/annotation contract gates stay green; the primitive is inert on
  every shipped grammar (no grammar uses it yet); clippy source-clean.
- Live-docs + the semantic-annotation mdBook + the steering control matrix are updated same-commit
  (the predicate is a new user-facing steering surface).

## `.1` DESIGN — tools-first WHY + WHERE (2026-07-10, session #84, `PGEN-SCP-0001`)

The design was driven entirely by the debug toolbox / source read, not by guessing:

1. **Oracle-frozen acceptance spec** (`pcre2test` 10.47) for the proving consumer `\K`-in-lookaround
   (err 199 "\K is not allowed in lookarounds"):
   - REJECT: `(?=a\Kb)` `(?!a\Kb)` `(?<=\K.)` `(?<!\K.)` `(?*a\Kb)` `(?<*a\Kb)` `(*pla:a\Kb)`
     `(*nla:a\Kb)` `(*plb:\K.)` `(*positive_lookahead:a\Kb)` `(*napla:a\Kb)` — every lookaround form;
     **`(?=a(b\Kc))` and `((?=x\Ky))` also REJECT** — `\K` in a NESTED group inside a lookaround, and
     a lookaround nested in a capture group, both still reject (⇒ the gate must see the enclosing
     lookaround at ANY nesting depth, not just the innermost frame).
   - ACCEPT: `\Kword` `a\Kb` `(?:a\Kb)` `(a\Kb)` `(?>a\Kb)` (atomic group is NOT a lookaround)
     `(?=ab)\K` `\K(?=ab)` (`\K` OUTSIDE the lookaround — the fact-leak counter-case). `(?=a[\K]b)`
     rejects for a different reason (err 107, already owned by `.4.4`).
2. **The gate has no fallback to defeat it** — tool-confirmed, not eyeballed (`parseability_probe
   --parse-dump-ast-pretty regex`): `\K` alone dumps `{ "kind": "keep_out", "type": "anchor" }`, a
   SINGLE parse path — because `simple_escape_letter_strict` (`grammars/regex.ebnf:1189`, comment
   `:1169` "52 minus the 7 escape-anchors `A B G K Z b z`") EXCLUDES `K`, so `\K` cannot re-route as a
   generic escape (`grammars/regex.ebnf:416` is its only production). And the same probe on `(?=a\Kb)`
   shows the reject is **currently VALIDATOR-owned, not grammar-owned**: it parses, then
   `Error: parse_full rejected … \K is not accepted inside a lookaround by the regex compile contract`
   (the `find_invalid_keep_out_escape_in_lookaround` compile-contract message). So `.4.10` is a
   validator→grammar migration, and gating the one `keep_out` production is sufficient. (Consumer
   detail for `.4.10`.)
3. **The store capability gap** (WHERE = `semantic_runtime.rs::evaluate_predicate` `:2868` + the
   built-in set `:3933`): `current_scope_is` is innermost-only (`:2882`); `has_fact`/`lacks_fact` are
   global+monotonic with no retract-on-close (`:2903`/`:2936`, `close_scope` `:2706`);
   `has_fact_in_current_scope` is depth-exact (`:2944`). None expresses "any ancestor active scope of
   kind K." The live stack `active_chain` (`:2235`) + `ScopeNode.kind` (`:2689`) already carry exactly
   what a containment query needs. ⇒ add `in_scope_kind` / `not_in_scope_kind` walking `active_chain`.

Both the parse-harness interpreter (`evaluate_directive_predicate` → `evaluate_predicate`) and the
generated parser dispatch store-consulting predicates through the SAME `evaluate_predicate` method, so
the primitive is byte-identical across the two lanes by construction (no codegen change).

## Task Tree

- ID: `SCOPE-CONTEXT-PREDICATE`  Status: `active`  Children: `.1`
- ID: `.1`  Status: **`done`** (`PGEN-SCP-0001`, session #84)  Goal: the `in_scope_kind` /
  `not_in_scope_kind` built-in predicates (whole-active-chain scope-kind containment), added to
  `evaluate_predicate` + `ENGINE_BUILTIN_PREDICATE_NAMES`, proven in isolation with new
  `semantic_runtime.rs` unit tests, inert on all shipped grammars, docs/book/matrix lockstep. See the
  §`.1` Implementation + Acceptance Checklist below.

## `.1` IMPLEMENTATION (`PGEN-SCP-0001`, 2026-07-10, session #84)

Files:
1. `rust/src/ast_pipeline/semantic_runtime.rs` — two new arms in `evaluate_predicate` (placed directly
   after `current_scope_is`, its innermost-only cousin): each parses the kind arg via
   `SemanticScopeKind::parse` (`?` on malformed) and returns `Some(self.active_chain.iter().any(node
   kind == expected))` (and its complement), with a self-explaining `pgen_trace_high!` verdict line
   (the "annotations explain themselves" doctrine); both names appended to
   `ENGINE_BUILTIN_PREDICATE_NAMES` with a `SCOPE-CONTEXT-PREDICATE.1` comment; a new unit test
   `in_scope_kind_predicates_walk_the_active_scope_chain` covering the acceptance-criteria matrix.
2. `docs/semantic_annotation_parser_book/src/steering-directives.md` (+ rendered html) — document the
   two predicates alongside `current_scope_is`, with the containment/auto-unwind semantics.
3. `docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md` — note the SC-13 built-in predicate set
   now includes the scope-ancestry pair.
4. Live-docs: `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`, this tree.

#### `.1` Acceptance Checklist (enforced)

- [x] **ROOT CAUSE / ISSUE** — before this slice no built-in predicate answers "am I inside an open
  scope of kind K at any nesting depth"; a gate using such a name got `None` from `evaluate_predicate`
  (non-blocking) — proven by its absence from `ENGINE_BUILTIN_PREDICATE_NAMES` and the match arms.
  Tool-backed WHY+WHERE in §Design: the `pcre2test` 10.47 oracle matrix (err 199; the nested
  `(?=a(b\Kc))`/`((?=x\Ky))` REJECT + the post-close `(?=ab)\K` ACCEPT cases); a `parseability_probe
  --parse-dump-ast-pretty regex` diagnosis confirming `\K`→`{kind:keep_out,type:anchor}` (single parse
  path) and that `(?=a\Kb)`'s reject is currently VALIDATOR-owned ("`\K` is not accepted inside a
  lookaround by the regex compile contract"), so `.4.10` is a validator→grammar migration; and the
  source read proving the three existing scope/fact predicates are each insufficient (`current_scope_is`
  innermost-only `:2882`; `has_fact`/`lacks_fact` global+monotonic, `close_scope` never retracts `:2706`;
  `has_fact_in_current_scope` depth-exact `:2944`).
- [x] **ADDRESSED** — `in_scope_kind`/`not_in_scope_kind` walk `active_chain` via the shared
  `active_scope_chain_has_kind` helper; new unit test `in_scope_kind_predicates_walk_the_active_scope_chain`
  proves the exact `\K`-in-lookaround semantics (nested-depth TRUE even under an inner `block` scope
  where `current_scope_is` is FALSE — the KEY contrast; post-close FALSE auto-unwind; root-global TRUE;
  malformed None). before→after: the names go from unrecognized (`None`, non-blocking) to a correct
  `Some(bool)` verdict. Both registered in `ENGINE_BUILTIN_PREDICATE_NAMES`.
- [x] **NO REGRESSION** — `cargo test --lib semantic_runtime` 135/135 + predicate/scope suite 59/59
  green; `semantic_full_contract_gate` ✅ (differential 80/80 matched, baseline allowed=0 new=0
  resolved=0); inert on all shipped grammars (no consumer references the names); clippy SOURCE strict
  lint clean (`clippy_on_rust_change` ✅ — the 177× generated `eq_op` errors are pre-existing generated
  debt, non-strict, 0 mention my symbols); mdBook docs gate ✅.

## Decisions

- **The primitive over the `current_scope_is` trick.** `current_scope_is(lookaround)` would pass the
  current regex grammar ONLY because lookarounds happen to be the sole scope-openers — a fragile,
  unenforced, non-local invariant. The active-chain walk is the honest, robust, general expression and
  matches the `.4.10` leaf's own "contextual gate primitive" language. (See Fix-hierarchy justification.)
- **Both `in_scope_kind` and `not_in_scope_kind` as builtins**, mirroring `has_fact`/`lacks_fact` — so
  the common "reject X inside Y" gate needs no `@predicate_def` negation wrapper.
- **Kind-only, not `(kind, name)`.** No consumer needs name-filtered containment; documented as a
  non-breaking future extension.
- **Primitive first, consumer separate.** The regex `.4.10` consumer is a leaf of
  `REGEX-PCRE2-FIDELITY`, landed in its own commit after this primitive is proven in isolation — the
  exact RSVC model.

## Verification Log

- `.1` (2026-07-10, session #84, `PGEN-SCP-0001`):
  - oracle spec: `pcre2test` 10.47 matrix for `\K`-in-lookaround (err 199) frozen in §Design — every
    lookaround form REJECTs incl. nested `(?=a(b\Kc))` / `((?=x\Ky))`; `(?=ab)\K` / `\K(?=ab)` /
    `(?>a\Kb)` ACCEPT.
  - isolation: `in_scope_kind_predicates_walk_the_active_scope_chain` — PASS (proves nested-depth true,
    the `current_scope_is`-false / `in_scope_kind`-true contrast, post-close auto-unwind, root-global,
    malformed None).
  - suites: `cargo test --lib semantic_runtime` 135/135; predicate/scope suite 59/59.
  - no-regression gate: `semantic_full_contract_gate` ✅ (semantic differential 80/80 matched, baseline
    unchanged); `mdbook_docs_gate` ✅.
  - lint: `clippy_on_rust_change` ✅ source strict clean; generated `eq_op` debt pre-existing (0 mention
    the new symbols).

## Commit Log

- `.1` — `PGEN-SCP-0001` (this commit): `in_scope_kind` / `not_in_scope_kind` engine primitives +
  isolation test + `semantic-store.md` book subsection + tree/live-docs. Files listed in the commit
  report.
