---
name: project-rule-span-value-compare-primitive
description: The `value_compare` @predicate builtin — a GENERAL, parser-agnostic RULE-SPAN value-comparison primitive that gates a rule on a comparison between two of its own resolved captures (`args:[$lhs, <op>, $rhs]`, ops lt/le/gt/ge/eq/ne), value-oriented (numeric when both operands parse as i64, lexical/textual fallback). Distinct from the ATOM-scoped @range/@len/@enum/@regex guards. Lets the EBNF own cross-capture accept/reject rules an out-of-band Rust validator used to express (the shared unlock for REGEX-PCRE2-FIDELITY .4.3 min>max / .4.5 descending ranges / .4.9 lookbehind length). Director-authorized 2026-07-09; landed RULE-SPAN-VALUE-CONSTRAINT.2.
metadata:
  node_type: memory
  type: project
  director_directive: true
  created: 2026-07-09
  owning_tree: RULE-SPAN-VALUE-CONSTRAINT
---

**THE PRIMITIVE.** `value_compare` is a first-class `@predicate` built-in that gates a rule on a
**comparison between two of the rule's own resolved captures** — a *rule-span value constraint*:

```ebnf
@predicate: { name: value_compare, args: [$min, le, $max], phase: post }
counted_quantifier := "{" min:number "," max:number "}"
```

- `<op>` ∈ { `lt`, `le`, `gt`, `ge`, `eq`, `ne` } — a **word-form** operator (the payload arg is a
  bare identifier, not a symbol).
- `$lhs` / `$rhs` are any resolvable payload reference (`$N`, `$name`, dotted, indexed), resolved
  against the rule's captured content exactly as every other directive-payload reference.
- The comparison is **value-oriented** (`compare_values`): numeric for **all six** ops when both
  operands parse as `i64` (so `05` equals `5` and `05 < 4` is false), deterministic lexical/textual
  fallback otherwise. Holds iff `resolve($lhs) <op> resolve($rhs)`; a `post`-phase failure rejects the
  rule (backtrackably — a gated alternative loses to a sibling rather than killing the parse).

**WHY (director-authorized 2026-07-09).** A grammar could not express a cross-number VALUE comparison
(e.g. PCRE2's counted-quantifier `{5,4}`-is-out-of-order reject, err 104) — only an out-of-band Rust
validator could (`regex_compile_validation.rs` `validate_counted_quantifier_body`, whose own comment
named "a rule-span value-constraint extension" as the intended fix; leading-zeros make it grammar-
hostile). Out-of-band acceptance gates are DEFECTS ([[project_ebnf_is_single_source_of_truth]]). This
primitive is the minimal GENERAL engine capability that lets the EBNF own such rules. It is the shared
unlock for the hard `REGEX-PCRE2-FIDELITY.4` families: `.4.3` (min>max order), `.4.5` (descending
ranges, via a later `codepoint`-coercion widening), `.4.9` (lookbehind length).

**HOW IT'S GENERAL (parser-agnostic, per [[feedback_ast_pipeline_parser_agnostic]] /
[[feedback_features_parser_agnostic_enable_all_parsers]]).** Capability-gated, never name-gated: a
grammar opts in by writing the predicate; enabled for ALL parsers. Byte-identical in the generated
parser and the parse-harness interpreter **by construction** — both CALL the shared runtime method
`evaluate_content_aware_predicate` → `evaluate_predicate` (`semantic_runtime.rs`), so a single new
dispatch arm serves both (proven by `parse_harness_semantic_gate` + `parse_harness_equivalence_gate`).

**DISTINCT FROM its neighbours (the key design boundary).**
- **Not a store query.** It reads no facts — no `@fact_kind`/`@emit_fact`, nothing store-related to
  roll back. It is a `@predicate` that gates on captured VALUES.
- **Rule-span, not atom-scoped.** It compares two *different* captures. The existing value guards
  `@range`/`@len`/`@enum`/`@regex` are **atom-scoped** — each constrains a *single* atom's matched
  text against a *constant* (recorded at `REGEX-PCRE2-FIDELITY.3.15`: "the SC-08 value-constraint
  machinery is ATOM-scoped on BOTH sides"). That atom-scoped machinery is inert for a cross-capture
  comparison; `value_compare` closes exactly that gap.

**DESIGN NOTES (tool-surfaced).**
- The framing that positional `$N` "HARD-ERRORS in `@predicate` payloads" was **stale** —
  `POSITIONAL-PAYLOAD-REFS.2` (2026-07-06) already made `$N` resolve (the `sem_ref_positional_
  unresolvable` pin was retired). So the primitive is the thin exposure of an existing capability, not
  a from-scratch build.
- **`compare_values` vs `compare_predicate_values`.** The comparison logic already existed as
  `compare_predicate_values` (used by composed `@predicate_def` bodies) — but there `eq`/`ne` are
  *textual*, so `"05" != "5"`. The isolation proof caught this asymmetry BEFORE any shipped consumer
  (an anchor mismatch on a leading-zero equality case). For a primitive literally named
  *value_compare* that asymmetry is not signoff-grade, so `value_compare` uses a dedicated
  `compare_values` that is uniformly value-oriented (numeric for `eq`/`ne` too when both parse). The
  shared `compare_predicate_values` is **unchanged** (zero `@predicate_def` blast radius). This is the
  value of proving in isolation first ([[feedback_correctness_before_speed]]).
- **Malformed vs unresolvable.** A malformed shape (wrong arity / unknown op word) → `None`
  (INAPPLICABLE, non-blocking) — the same `?`-on-malformed convention every builtin follows. An
  **unresolvable** `$ref` argument hard-errors in the resolution layer (the rule fails loudly), never
  a silent pass.

**v1 SCOPE.** Decimal-integer numeric + lexical/textual fallback coercion (exactly what `.4.3` needs).
A `codepoint`-decoding coercion mode for `.4.5` descending ranges (`[\x{100}-z]`) is a later, separate
widening leaf, not v1.

**LANDED.** `RULE-SPAN-VALUE-CONSTRAINT.2` (`PGEN-RSVC-0002`, 2026-07-09). Touch: `predicate_expr.rs`
(`CompareOp::from_word`), `semantic_runtime.rs` (`value_compare` arm + `compare_values` +
`ENGINE_BUILTIN_PREDICATE_NAMES`), `parse_harness_semantic_suite.rs` (2 isolating constructs/cases).
Codegen/interpreter/grammar unchanged (shared runtime; args generic). Proven:
`parse_harness_semantic_gate` (2 cases CLEAN, 9+3 samples byte-identical) + `parse_harness_equivalence_
gate` (6 fully-certified byte-identical, `value_compare` inert on all shipped grammars) + dual lib
suite 887/0 + clippy source-clean. First consumer (separate tree): `REGEX-PCRE2-FIDELITY.4.3`.
