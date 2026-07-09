---
name: project-scope-context-predicate-primitive
description: The `in_scope_kind` / `not_in_scope_kind` @predicate builtins — a GENERAL, parser-agnostic SCOPE-ANCESTRY / lexical-containment primitive that gates a rule on whether the parser is currently inside an open scope of a given kind, at ANY nesting depth (the whole-active-chain generalization of the innermost-only `current_scope_is`). Reads the live scope chain, so it auto-unwinds on `@close_scope` — unlike the global+monotonic fact predicates. Lets the EBNF own "reject X anywhere inside enclosing Y" rules (the `\K`-in-lookaround unlock). Landed SCOPE-CONTEXT-PREDICATE.1 (2026-07-10) under the same director-authorized engine-primitive umbrella as `value_compare`.
metadata:
  node_type: memory
  type: project
  director_directive: true
  created: 2026-07-10
  owning_tree: SCOPE-CONTEXT-PREDICATE
---

**THE PRIMITIVE.** `in_scope_kind` / `not_in_scope_kind` are first-class `@predicate` built-ins that
gate a rule on **lexical containment** — is the parser currently inside an open scope of a given
kind, at any nesting depth?

```ebnf
# Reject \K anywhere inside a lookaround body (PCRE2 err 199), accept it elsewhere.
@predicate: { name: not_in_scope_kind, args: [lookaround], phase: pre }
keep_out_anchor := "\\K" -> {type: "anchor", kind: "keep_out"}
```

- `in_scope_kind(kind)` returns `Some(true)` iff **any** currently-open scope — the innermost frame
  OR any enclosing ancestor up to and including the global scope — has that kind; `Some(false)`
  otherwise; `None` if the kind arg is missing/empty (the `?`-on-malformed → non-blocking convention
  every built-in follows).
- `not_in_scope_kind(kind)` is the boolean complement (the `has_fact`/`lacks_fact` pairing), so
  "reject X inside Y" needs no `@predicate_def` negation wrapper.
- Both walk the live `active_chain` via the shared `active_scope_chain_has_kind` helper, so they
  **auto-unwind** the instant an `@close_scope` pops the scope.
- Kind-only (a built-in kind or any `Custom(String)` label such as `lookaround`); a name-filtered
  `(kind, name)` variant is a documented, non-breaking future extension.

It is the **whole-active-chain generalization** of the innermost-only `current_scope_is`.

**WHY (fix-hierarchy tier-5, tools-first).** A grammar could not express "reject construct X anywhere
inside enclosing construct Y" — a pervasive context-sensitive need (`\K` in a lookaround, `return`
outside a function, `break` outside a loop, a nested-`atomic` ban). Lower tiers were exhausted against
the proving consumer, PCRE2's `\K`-in-lookaround (err 199; oracle: `(?=a(b\Kc))` and `((?=x\Ky))`
REJECT for nesting depth, `(?=ab)\K` and `(?>a\Kb)` ACCEPT):

- `current_scope_is` reads **only the innermost scope frame**, so it misses a `\K` nested inside a
  plain group inside the lookaround. It would pass the current regex grammar *only* under the fragile,
  unenforced, non-local invariant "lookarounds are the sole scope-openers" — a hidden global coupling
  that a future scope-opener would silently break. Using it here is the `NO WORKAROUNDS` violation
  ([[feedback_no_workarounds_fix_hierarchy]]).
- `has_fact` / `lacks_fact` are **global and monotonic**: `close_scope` pops the frame but never
  retracts a fact (`semantic_runtime.rs:2706-2748` never truncates `self.facts`), so a fact emitted at
  construct-OPEN leaks to later siblings — `(?=ab)\K` would be wrongly rejected.
- `has_fact_in_current_scope` is depth-exact — collides across unrelated subtrees and grows with
  nested groups.

The infrastructure was already present (`active_chain` is the live stack of open `ScopeId`s;
`ScopeNode` carries `kind`), so the honest, robust, general answer is a built-in that walks the chain.
This is the "contextual gate primitive" the `.4.10` leaf itself anticipated. Same director-authorized
engine-primitive umbrella (2026-07-09) as [[project_rule_span_value_compare_primitive]].

**DESIGN.** Two arms in `evaluate_predicate` (`semantic_runtime.rs`) directly after `current_scope_is`
+ the shared `active_scope_chain_has_kind` helper; both registered in
`ENGINE_BUILTIN_PREDICATE_NAMES` (so no `@predicate_def` may shadow them). Both the parse-harness
interpreter and the generated parser dispatch store-consulting predicates through the **same**
`SemanticRuntimeState::evaluate_predicate`, so the primitive is byte-identical across the two lanes by
construction — **no codegen change**. Each arm parses the kind via `SemanticScopeKind::parse`
(arbitrary strings → `Custom(String)`), `?`-on-malformed → `None`, and emits a self-explaining
`pgen_trace_high!` verdict (the "annotations explain themselves" doctrine).

**PROVEN IN ISOLATION, BEFORE ANY CONSUMER** (the RSVC model). Test
`in_scope_kind_predicates_walk_the_active_scope_chain`: nested-depth TRUE even under an inner `block`
scope where `current_scope_is` is FALSE (the key contrast), post-close AUTO-UNWIND FALSE, root-`global`
TRUE, malformed None. NO-REGRESSION: `semantic_full_contract_gate` differential 80/80 baseline-unchanged;
inert on all shipped grammars. The regex consumer (`\K`-in-lookaround: `@open_scope: {kind: lookaround}`
on the lookaround open-markers + this gate on the keep-out anchor) is `REGEX-PCRE2-FIDELITY.4.10`, a
leaf of a different tree, landed in its own commit.

**GENERAL RULE FOR GRAMMAR AUTHORS.** For a *lexical-containment* gate ("X is legal only inside / only
outside construct Y, at any depth"), use `@open_scope`/`@close_scope` + `in_scope_kind` /
`not_in_scope_kind` — **not** facts. Facts are global and monotonic (never retracted on close), so they
model "seen X earlier in the parse" (declaration-before-use), the opposite of "currently inside X".
Relates to [[feedback_grammar_rules_must_consult_store]] and [[feedback_features_parser_agnostic_enable_all_parsers]].
