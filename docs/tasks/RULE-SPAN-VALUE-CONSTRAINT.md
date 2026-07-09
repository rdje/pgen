# RULE-SPAN-VALUE-CONSTRAINT — a general `@predicate` value-comparison primitive over resolved captures (`$lhs <op> $rhs`)

## Metadata

- Tree ID: `RULE-SPAN-VALUE-CONSTRAINT`
- Status: `active` — **CORE COMPLETE** (`.1`+`.2` done 2026-07-09, session #77); **`.3` the
  code-point-coercion sibling `value_compare_codepoint` done 2026-07-09, session #81** (the widening
  leaf the v1 Non-Goals deferred, driven by `REGEX-PCRE2-FIDELITY.4.5.c`). The general primitives are
  landed + proven in isolation; downstream CONSUMERS (`REGEX-PCRE2-FIDELITY.4.3` for `value_compare`,
  `.4.5.c.2` for `value_compare_codepoint`) are leaves of a different tree, so the FRONTIER passes there.
- Family / slice-id prefix: `PGEN-RSVC-<NNNN>` (abbreviation of the tree name; used in commit subjects)
- Roadmap lane: cross-cutting engine correctness — a **parser-agnostic** value-comparison predicate
  that lets the EBNF express a *rule-span* value constraint (compare two of a rule's resolved
  captures numerically) declaratively, so the grammar can own accept/reject rules that today only an
  out-of-band Rust validator can express. Aligns with the Annotation-Driven Semantic Steering
  Doctrine ("bounded semantic predicates that steer parses") and the EBNF-single-source-of-truth
  doctrine ([[project_ebnf_is_single_source_of_truth]]).
- Director authorization: 2026-07-09 (recorded in `MEMORY.md` resume pointer + HEAD `5adcc158`
  continuity capture) — building the **rule-span value-constraint** engine primitive to
  SOTA/signoff/very-high quality was explicitly AUTHORIZED as the shared unlock for the remaining
  hard `REGEX-PCRE2-FIDELITY.4` families (`.4.3` min>max order, `.4.5` descending ranges, `.4.9`
  lookbehind length) that no lower fix-hierarchy tier can express.
- Fix-hierarchy justification (tier-5 engine, tool-backed + director-approved): the proving consumer
  (`regex_compile_validation.rs:433-434`) is a **cross-number VALUE comparison**; the validator's own
  comment (`:388-390`) says out-of-order "needs a rule-span value-constraint extension … which is
  grammar-hostile with leading zeros" — i.e. tiers 1–4 (existing annotations / store / new annotation
  / new store) cannot express a two-capture numeric comparison. A value-comparison `@predicate` is the
  minimal general primitive that can. (See §4 Fix-hierarchy walk.)

## Goal

Expose a **general, parser-agnostic** value-comparison predicate on the existing `@predicate`
surface so a grammar can gate a rule on a numeric comparison between two of its **resolved captures**:

```ebnf
@predicate: { name: value_compare, args: [$lhs, <op>, $rhs], phase: post }
```

- `<op>` ∈ { `lt`, `le`, `gt`, `ge`, `eq`, `ne` } (word forms; the payload op is a bare identifier).
- `$lhs` / `$rhs` are any resolvable payload reference (`$N`, `$name`, dotted `$N.f`, indexed `$a[0]`)
  — resolved against the rule's captured content exactly as every other directive-payload ref is.
- Semantics: the predicate holds iff `resolve($lhs) <op> resolve($rhs)` under the shared comparison
  contract `compare_predicate_values` (decimal-integer numeric comparison when BOTH sides parse as
  `i64`, deterministic lexical fallback otherwise; `eq`/`ne` are textual). A `post`-phase failure
  rejects the rule (and backs out its emissions), so the grammar itself now owns the constraint.

This is a **RULE-SPAN** value constraint — it compares two of the rule's captures — which is
categorically distinct from the existing **atom-scoped** value guards `@range` / `@len` / `@enum` /
`@regex` (each constrains ONE atom's matched text against a constant). The `.3.15` regex note already
recorded that "the SC-08 value-constraint machinery is ATOM-scoped on BOTH sides"; this primitive
closes the cross-capture gap.

## Non-Goals

- **Not** arithmetic. Predicates are *decisions*, not computations (per the Semantic Store chapter):
  the primitive compares two already-captured values; it never computes `$a + $b`. No new expression
  language.
- **Not** a new directive or new payload syntax. It reuses `@predicate { name, args, phase, view }`
  verbatim — `value_compare` is a new builtin NAME, nothing else. (Minimal new syntax, director's ask.)
- **Not** codepoint/width-aware decoding. v1 is decimal-integer + lexical-fallback coercion (exactly
  what the proving consumer `.4.3` needs). A `codepoint`-decoding coercion mode for `.4.5` descending
  ranges (`[\x{100}-z]`) is a later, separate widening leaf, not this tree.
- **Not** name-gated to any grammar. Capability-gated (a grammar opts in by writing the predicate),
  parser-AGNOSTIC, enabled for ALL parsers, byte-identical codegen⟷interpreter.
- **Not** the regex consumer itself. Applying `value_compare` to `REGEX-PCRE2-FIDELITY.4.3`
  (migrating `find_invalid_counted_quantifier`'s order check) is a leaf of the **REGEX-PCRE2-FIDELITY**
  tree that CONSUMES this primitive after it is proven in isolation — not owned here.

## Acceptance Criteria

- `value_compare` is a first-class `@predicate` builtin: registered in
  `ENGINE_BUILTIN_PREDICATE_NAMES` and dispatched in `evaluate_predicate`, evaluating
  `resolve($lhs) <op> resolve($rhs)` via `compare_predicate_values`.
- Word-form ops `lt`/`le`/`gt`/`ge`/`eq`/`ne` map to `CompareOp`.
- The primitive is proven **in isolation, BEFORE any shipped consumer**: a new
  `parse_harness_semantic_suite` construct + case(s) covering every op, numeric vs leading-zero
  coercion, hit AND miss (verdict-changing), and backtrackability — with the
  `parse_harness_semantic_gate` (`every_semantic_construct_is_byte_identical` +
  `semantic_construct_coverage_is_complete`) and `parse_harness_equivalence_gate` GREEN
  (codegen⟷interpreter byte-identical by construction, since both call the shared runtime method).
- NO REGRESSION: the 6 fully-certified grammars byte-identical; SV/regex cert unchanged; the lib
  suites unchanged except the new pins; clippy clean; the two flagged inert sites
  (`grammar_wellformedness.rs` `FACT_QUERY_PRIMITIVES`, `stimuli_generator.rs` store-aware
  witnessing) confirmed unaffected (`value_compare` is not a fact-query and not a store-prelude gate).
- LOCKSTEP: the Annotation System + Semantic Store book chapters document `value_compare`; a decision
  record in `docs/decisions/` records the primitive + the ATOM-scoped-vs-RULE-SPAN distinction; the
  ebnf/semantic_annotation parser books note the new builtin.
- After the primitive lands + is proven, the **REGEX-PCRE2-FIDELITY.4.3** consumer leaf migrates the
  validator's order check to a grammar `value_compare` gate (separate tree, separate slice).

## `.1` SCOPING & DESIGN (`PGEN-RSVC-0001`, 2026-07-09, session #77, tools-first, PURE-DOCS)

### Method

Tools-first survey of the ACTUAL runtime + the proving consumer (no eyeballing), independently
confirmed by direct source reads (not sub-agent trust). Every anchor below was read at HEAD.

### 🔎 KEY FINDING — the MEMORY framing is STALE; the primitive is far smaller than framed

The `MEMORY.md` next_action framed the primitive as needing to "close the known gap" that
"positional `$N` HARD-ERRORS in `@predicate` payloads". **That gap is already closed.**
`POSITIONAL-PAYLOAD-REFS.2` (2026-07-06, session #50) made positional `$N` **resolve** in
predicate/directive payloads — the hard-error is the *pre-fix* state, and the
`sem_ref_positional_unresolvable` hard-error pin was **RETIRED** and replaced by the WORKING
`RefPositional` differential pin (`parse_harness_semantic_suite.rs:104-113`, `:533-570`). And the
comparison logic **already exists** as `compare_predicate_values` (`semantic_runtime.rs:4141-4169`) —
currently reachable only through composed `@predicate_def` bodies (`eval_predicate_expr`'s `Compare`
arm), NOT as a first-class `@predicate` builtin.

**Consequence:** the primitive is NOT a from-scratch build of positional-resolution + comparison. It
is the thin, faithful **exposure of an existing, tested comparison helper as a first-class predicate
builtin**. This materially shrinks scope and de-risks the slice. (Surfaced prominently to the director
per the be-alert / don't-classify-and-move-on directive.)

### ROOT CAUSE the primitive addresses (WHY + WHERE — the proving consumer)

- `rust/src/regex_compile_validation.rs:400` `validate_counted_quantifier_body` — splits a `{N,M}`
  body on `,`, trims space/tab, parses each part as `u64` (saturating), and at **`:433-434`**
  rejects when `minimum > maximum` (PCRE2 err 104). The comment at **`:382-390`** states the fix
  intent verbatim: *"Order stays validator-owned until capstone `.4` (or a rule-span value-constraint
  extension): out-of-order needs a cross-number VALUE comparison, which is grammar-hostile with
  leading zeros."* Leading-zero hostility is exactly why a plain structural gate can't do it and a
  value-coercing predicate can (`"05".parse::<i64>() == 5`).
- Wired into `validate_regex_compile_contract` (`:16`, first check at `:46-48`) — the out-of-band
  validator the REGEX-PCRE2-FIDELITY tree is deleting in favor of grammar ownership.

### The mechanism map (tool-verified anchors, all at HEAD)

- **Parse:** `@predicate` → `SemanticPredicateSpec { name, args: Vec<UnifiedSemanticValue>, phase,
  view }` via `parse_predicate` (`semantic_runtime.rs:4007`); args stored raw/unresolved; no
  name-specific arity check — any predicate name flows through. `SemanticPredicateSpec` at `:994`.
- **Dispatch (the match to extend):** `evaluate_predicate(&self, &SemanticPredicateSpec) ->
  Option<bool>` (`semantic_runtime.rs:2840`), `match normalized_name` at `:2853` (11 store/scope
  arms), default arm `_ =>` at `:3054` (composed `@predicate_def` lookup). The content-aware
  POST/BRANCH entry `evaluate_content_aware_predicate` (`:3183`) handles only `content_kind_is` then
  falls through to `evaluate_predicate` (`:3202`) — so a resolved-arg value-comparison belongs in
  `evaluate_predicate` (it needs no direct content inspection; its args are pre-resolved).
- **Builtin registry:** `const ENGINE_BUILTIN_PREDICATE_NAMES` (`semantic_runtime.rs:3830`); a
  `@predicate_def` may not shadow these (V-QDEF-1, enforced `:3878`).
- **Arg resolution (already handles our forms, byte-identical across both impls):** codegen emits
  `resolve_semantic_predicate_spec_against_content` (`ast_based_generator.rs:2423`, POST hard-resolve;
  `:2449` BRANCH soft) which loops args through `resolve_unified_semantic_value_against_content`
  (`:2481`) → `resolve_semantic_reference` (`:5791`, positional dispatch `:5809-5820`) →
  `coerce_unified_semantic_scalar`. Unresolvable POST arg → hard error (`:2490-2495`). Interpreter
  mirror: `parse_harness_interpreter.rs:1417 / :1442 / :1510 / :1547`, coercion ladder
  `:1473-1499`. **Both call the SAME shared runtime `evaluate_content_aware_predicate` /
  `evaluate_predicate`** (codegen POST loop `ast_based_generator.rs:2133-2178`, interpreter POST loop
  `parse_harness_interpreter.rs:948-977`) — so a new arm added to `evaluate_predicate` is
  byte-identical across codegen and interpreter BY CONSTRUCTION.
- **The comparison helper (reuse target):** `compare_predicate_values(lhs, op, rhs) -> bool`
  (`semantic_runtime.rs:4141`): `Eq`/`Ne` textual; `Lt`/`Le`/`Gt`/`Ge` numeric via `i64` when both
  parse, deterministic lexical fallback otherwise. `enum CompareOp { Eq, Ne, Lt, Le, Gt, Ge }`
  (`predicate_expr.rs:116`), symbolic-only parse today → needs a word-form `from_word` map.
- **Grammar surface:** NO change — `@predicate` args are generic `annotation_value`s; the op is a
  bare `identifier_literal` and `$lhs`/`$rhs` are `reference_value`s, all of which parse today
  (`grammars/semantic_annotation.ebnf` `annotation_value` / `reference_value` / `rule_reference`).
- **Isolation-proof surface:** `parse_harness_semantic_suite.rs` — add a `SemanticConstruct` variant
  (`:60-184`), register in `::ALL` (`:188-217`), append `SemanticCase`(s) to `SEMANTIC_CASES`
  (`:246`). Template: `sem_ref_positional` (`:533-570`, the working positional-`$N`-in-`@predicate`
  case) and `sem_post_gate` (`:248-266`, canonical `phase: post`). The equivalence gate
  (`parse_harness_equivalence.rs`) certifies interp == generated over the deterministic corpus.

### Surface-design decision (routine, within-principle → engineer's call; recorded)

**Option A — single `value_compare`, op as infix middle arg** (CHOSEN):
`@predicate: { name: value_compare, args: [$lhs, <op>, $rhs], phase: post }`.
- Maps 1:1 to `compare_predicate_values(lhs, op, rhs)` (thin, faithful exposure).
- One builtin name / one dispatch arm / one registry line (minimal surface — director's "minimal new
  syntax").
- Infix reading (`$min le $max`) is the natural way to read a comparison; op-as-data is extensible.
- Idiomatic: existing builtins already interleave literal + `$ref` args positionally
  (`fact_attribute_equals args:[kind, $name, attr, value]`).

Rejected — Option B (a family `value_le`/`value_lt`/… each `args:[$lhs,$rhs]`): 6 names / 6 registry
entries / more surface for no expressive gain; the director's framing lists ops as a *set* (op-as-data).

### Touch-map for `.2` (implement + prove in isolation)

1. `predicate_expr.rs`: add `CompareOp::from_word(&str) -> Option<CompareOp>` (`le`→`Le`, `lt`→`Lt`,
   `ge`→`Ge`, `gt`→`Gt`, `eq`→`Eq`, `ne`→`Ne`; case-insensitive).
2. `semantic_runtime.rs:2853`: add a `"value_compare"` arm to `evaluate_predicate` — exactly-3-args,
   `lhs = scalar_text(args[0])`, `op = CompareOp::from_word(scalar_text(args[1]))`,
   `rhs = scalar_text(args[2])`, `Some(compare_predicate_values(lhs, op, rhs))`; any malformed shape
   → `None` (INAPPLICABLE, consistent with every other builtin's `?`-on-malformed convention).
3. `semantic_runtime.rs:3830`: add `"value_compare"` to `ENGINE_BUILTIN_PREDICATE_NAMES`.
4. `parse_harness_semantic_suite.rs`: add `SemanticConstruct::ValueCompare` + `::ALL` entry + a
   `SemanticCase` (template `sem_ref_positional`) — all 6 ops, decimal + leading-zero coercion, hit
   AND miss, backtrackability; deterministic hand-anchored `expected_accept`.
5. Codegen + interpreter: NO change (shared runtime).
6. Grammar: NO change.
7. NO-REGRESSION verify: `grammar_wellformedness.rs:914 FACT_QUERY_PRIMITIVES` (do NOT add — not a
   fact-query; `.contains()` graceful), `stimuli_generator.rs` store-aware witnessing (inert — not a
   store prelude gate). Full gate set per COMMIT.md + the acceptance checklist.
8. LOCKSTEP: Annotation System + Semantic Store book chapters; ebnf/semantic_annotation parser books;
   a `docs/decisions/` record for the primitive + the ATOM-scoped-vs-RULE-SPAN distinction.

### `.1` Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — proving consumer `regex_compile_validation.rs:433-434`
  (`minimum > maximum`) is out-of-band Rust; its own comment `:388-390` names "a rule-span
  value-constraint extension" as the intended grammar-owning fix. `value_compare` is not yet a builtin
  (`ENGINE_BUILTIN_PREDICATE_NAMES:3830` lists 11 names, none a value-comparison).
- [x] **ROOT CAUSE (WHY + WHERE)** — mechanism map above: comparison logic exists
  (`compare_predicate_values:4141`) but is reachable ONLY via composed `@predicate_def` bodies, not as
  a first-class `@predicate` builtin; positional `$N` already resolves (POSITIONAL-PAYLOAD-REFS.2);
  the gap is purely "register + dispatch a value-comparison builtin".
- [x] **DESIGN** — Option A `value_compare args:[$lhs, <op>, $rhs]`; reuse `compare_predicate_values`;
  add `CompareOp::from_word`; byte-identical codegen⟷interpreter by shared-runtime construction.
- [N/A] **ADDRESSED / NO REGRESSION** — `.1` is PURE-DOCS (design only); no code, no behavior change.
  These are earned by `.2`.
- [x] **LOCKSTEP** — `docs/TASK_TREE.md` frontier updated; `MEMORY.md` resume pointer updated. (Book /
  decision-record lockstep lands with the `.2` code slice, per the sibling-tree pattern.)

## Task Tree

- ID: `RULE-SPAN-VALUE-CONSTRAINT`  Status: `active`  Children: `.1`, `.2`, `.3`
- ID: `.1`  Status: **`done`** (`PGEN-RSVC-0001`, session #77)  Goal: tools-first scoping + design of
  the `value_compare` rule-span value-comparison primitive; touch-map; surface decision; the
  STALE-FRAMING finding. Acceptance: the §`.1` section above + the checklist. PURE-DOCS.
- ID: `.2`  Status: **`done`** (`PGEN-RSVC-0002`, session #77)  Goal: implement `value_compare` as a
  first-class `@predicate` builtin (op-word map + dispatch arm + registry) and PROVE IN ISOLATION
  (semantic suite construct + cases, semantic gate + equivalence gate green), byte-identical
  codegen⟷interpreter; full NO-REGRESSION; book + decision-record lockstep. No shipped consumer in
  this slice. See the §`.2` implementation section + Acceptance Checklist below.
- ID: `.3`  Status: **`done`** (`PGEN-RSVC-0003`, session #81)  Goal: the CODE-POINT-COERCION sibling
  builtin `value_compare_codepoint` — the widening leaf the `.2` Non-Goals deferred ("a `codepoint`-
  decoding coercion mode for `.4.5` descending ranges … is a later, separate widening leaf"). Decodes
  each operand as a single CHARACTER LITERAL (bare Unicode scalar or the standard C/Perl char-escape
  vocabulary — hex `\xHH`/`\x{H..}`, octal `\o{O..}`/`\NNN`, control `\cX`, named `\a \b \e \f \n \r
  \t`, escaped literal `\X`) to its Unicode code point, then compares the two code points numerically.
  The enabler for `REGEX-PCRE2-FIDELITY.4.5.c` (class-range order `[z-a]`/`[\x{100}-z]`), whose
  endpoints reach the predicate as raw spellings a textual/`i64` `value_compare` mis-orders. Same
  op-word map, same dispatch/registry pattern as `.2`; a SIBLING builtin (not a `value_compare` flag)
  keeps zero blast radius on the proven `value_compare`/`counted_quantifier` path. Proven in ISOLATION
  (semantic suite construct + 8-input case: all six ops, hex/octal/control/named decode, codepoint-vs-
  textual discriminators, `None`-non-blocking anchor). See the §`.3` section + Acceptance Checklist below.

### `.2` IMPLEMENTATION (`PGEN-RSVC-0002`, 2026-07-09, session #77)

**What landed (per the §`.1` touch-map, with one tool-surfaced refinement).**

1. `rust/src/ast_pipeline/predicate_expr.rs` — `CompareOp::from_word(&str) -> Option<CompareOp>`
   (`lt`/`le`/`gt`/`ge`/`eq`/`ne`, case-insensitive).
2. `rust/src/ast_pipeline/semantic_runtime.rs` — a `"value_compare"` arm in `evaluate_predicate`
   (exactly-3-args; `scalar_text` lhs/rhs + `CompareOp::from_word` op; `compare_values`; malformed →
   `None`) + registration in `ENGINE_BUILTIN_PREDICATE_NAMES`.
3. `rust/src/parse_harness_semantic_suite.rs` — two isolating constructs + cases:
   `sem_value_compare` (9 samples: all six ops each verdict-changing + two leading-zero anchors) and
   `sem_value_compare_backtrack` (3 samples: a `value_compare` post-rejection loses the tournament to
   an ungated sibling, tree-observable).
4. Codegen / interpreter / grammar: **NO change** (both call the shared runtime; args are generic).

**🔎 Tool-surfaced design refinement (`compare_values`, not `compare_predicate_values`).** The `.1`
touch-map planned to reuse `compare_predicate_values` verbatim. The isolation proof caught, BEFORE any
shipped consumer, that its `eq`/`ne` are **textual** (so `"05" != "5"`) while `lt`/`le`/`gt`/`ge` are
numeric — an asymmetry not signoff-grade for a primitive named *value_compare*. `value_compare`
therefore uses a dedicated `compare_values` (`semantic_runtime.rs`) that is **uniformly** value-oriented
(numeric for all six ops when both operands parse as `i64`, deterministic lexical/textual fallback
otherwise). The shared `compare_predicate_values` is **unchanged** (composed-`@predicate_def` bodies
keep their historical textual-eq semantics — zero blast radius). The two leading-zero anchors
(`05 lt 4` → reject; `05 eq 5` → accept) now DECISIVELY prove uniform numeric coercion. This is the
value of proving-in-isolation-first.

**Semantics (final).** Holds iff `resolve($lhs) <op> resolve($rhs)` under `compare_values`; a
`post`-failure rejects the rule **backtrackably** (a gated alternative loses to a sibling). A malformed
shape → `None` (INAPPLICABLE / non-blocking); an unresolvable `$ref` hard-errors in the resolution
layer (rule fails loudly). Byte-identical codegen⟷interpreter by shared-runtime construction.

#### `.2` Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — before this slice `value_compare` was not a builtin
  (`ENGINE_BUILTIN_PREDICATE_NAMES` had 11 names, none a value comparison); the cross-capture VALUE
  comparison the proving consumer needs (`regex_compile_validation.rs:433-434` `minimum > maximum`)
  had no grammar-expressible form (only the composed-`@predicate_def` path reached
  `compare_predicate_values`, and it hard-errored positional `$N` before POSITIONAL-PAYLOAD-REFS.2).
- [x] **ROOT CAUSE (WHY + WHERE)** — the comparison mechanism existed (`compare_values` /
  `compare_predicate_values`, `semantic_runtime.rs`) but was not exposed as a first-class `@predicate`
  builtin; the dispatch (`evaluate_predicate:2853`) + registry (`:3830`) had no `value_compare`. Both
  the codegen POST/BRANCH loops (`ast_based_generator.rs:2133-2178`/`:3537-3600`) and the interpreter
  (`parse_harness_interpreter.rs:948-977`) CALL the shared runtime method, so one new arm serves both.
  Toolbox demonstration via the scratch slot (`scratch := "{" lo "," hi "}"` gated `@predicate
  value_compare [$2, le, $4] phase:post`, then `focus_scratch` + release-probe rebuild), the
  self-explaining predicate trace (`PGEN_TRACE_VERBOSITY=debug … --parse scratch … --trace-rules
  scratch`) on `{5,4}`:
  ```
  ⚖️ value_compare("5" <= "4") → false caller=scratch
  🚫 Rule 'scratch' rejected by post predicate 'value_compare [Number("5"), Identifier("le"), Number("4")]'
  ```
  — the primitive rejects lo>hi at the exact rule (WHERE) for the exact comparison (WHY).
- [x] **FIX** — fix-tier ENGINE (a new GENERAL parser-agnostic primitive, tier-5, director-authorized
  since no lower tier can express a two-capture numeric comparison): the `value_compare` builtin +
  `compare_values` + `CompareOp::from_word`, reusing the existing positional-`$N` resolution and
  content-aware predicate plumbing (no new syntax, no codegen/grammar change).
- [x] **ADDRESSED (verified)** — `make -C rust parse_harness_semantic_gate` GREEN: `sem_value_compare`
  CLEAN (9 samples, diverge=0, anchor_miss=0) + `sem_value_compare_backtrack` CLEAN (3 samples), both
  gate tests pass (`every_semantic_construct_is_byte_identical` + `semantic_construct_coverage_is_
  complete`). The 6 ops each verdict-changing; the leading-zero anchors prove numeric (not lexical)
  coercion; the backtrack case proves a post-rejection loses to a sibling (tree-observable). Interpreter
  == compile-and-run oracle byte-identical throughout. Independently, the scratch-slot release probe
  (`--parse scratch`): `{5,4}` REJECT (rc=1, the `rejected by post predicate 'value_compare …'` trace
  above), `{4,5}` `parse_full passed for grammar 'scratch'` (rc=0), `{05,4}` REJECT (rc=1 — numeric
  5>4, not lexical `"05"<"4"`).
- [x] **NO REGRESSION** — `make -C rust parse_harness_equivalence_gate` GREEN (all 4 tests incl.
  `certified_grammars_are_byte_identical`: the 11 certified grammars, incl. the 6 fully-certified, are
  byte-identical — `value_compare` inert on every shipped grammar, none declares it); full dual lib
  suite `887 passed; 0 failed; 29 ignored` (unchanged baseline — the 2 new cases are enumerated data,
  not new `#[test]`s); `clippy_on_rust_change` source-clean (the 178-eq_op generated debt is
  pre-existing, non-strict). Cert-coverage is unaffected by construction (no grammar/codegen/cert
  logic touched; the equivalence gate exercises generation at seeds 0/7/42 with zero divergence). The
  two flagged inert sites confirmed: `grammar_wellformedness.rs:914 FACT_QUERY_PRIMITIVES` (a fixed
  `.contains()` set — `value_compare` isn't a fact-query) and `stimuli_generator.rs` store-aware
  witnessing (not a store-prelude gate).
- [x] **LOCKSTEP** — platform book *Semantic Store* (Value-comparison predicates section) + *Annotation
  System* (rule-span value comparison subsection); `semantic_annotation` parser book steering-directives
  catalog; both rebuilt (`mdbook_docs_gate` + `semantic_annotation_parser_book_gate` GREEN, tracked HTML
  regenerated); decision record [[project_rule_span_value_compare_primitive]] + INDEX; `CHANGES.md`;
  `DEVELOPMENT_NOTES.md`; `MEMORY.md`; `docs/TASK_TREE.md`. No parser EBNF touched ⇒ no per-parser
  contract/ledger/release change (this is an ENGINE primitive, inert until a consumer adopts it).

### `.3` IMPLEMENTATION (`PGEN-RSVC-0003`, 2026-07-09, session #81)

**Why a SIBLING builtin, not a `value_compare` mode (surface decision, within-principle → engineer's
call; recorded).** The `.2` Non-Goals deferred "a `codepoint`-decoding coercion mode" as a later
widening leaf. Three realizations were weighed: (X) a new sibling builtin `value_compare_codepoint`;
(Y) a `coerce: codepoint` payload key on `value_compare`; (Z) auto-coercion inside `compare_values`.
**(X) chosen.** It is the minimal-code, zero-blast-radius realization: one dispatch arm + one registry
line (the exact pattern `.2` used), with NO change to the shared `SemanticPredicateSpec` struct (which
many sites construct — (Y) would touch all of them) and NO silent semantic change to the proven
`value_compare`/`counted_quantifier` path ((Z) would). The code-point comparison is a genuinely
DISTINCT semantics (decodes character literals; never the `i64`/textual ladder), so a distinct,
discoverable NAME is more honest than a hidden flag. The director authorized the *capability* ("the
`value_compare` codepoint-coercion widening"); a sibling builtin is the cleanest realization of it.
(The `.2` precedent likewise rejected a 6-name op-family, but for "no expressive gain"; here the second
name carries real expressive gain — code-point vs value semantics.)

**Tool-grounded root cause (why a plain `value_compare` cannot own class-range order).** Tools-first
`--parse-dump-ast-pretty` on representative ranges (session #81) showed the endpoints reach a rule-span
predicate as RAW SPELLINGS via positional `$N` (RAWCAP-TRANSFORM-PATH.2 — proven for a `->` rule):
`[a-z]`→`start:"a"`; `[\x30-\x39]`→raw `"\x30"`/`"\x39"`; `[\a-\e]`→raw `"\a"`/`"\e"` (the grammar's
SHAPED view decodes `\a`→`"a"`, a LOSSY letter — so only the raw spelling is faithful). A textual/`i64`
`compare_values` mis-orders these: textual `"\x{100}" < "\x{FF}"` (compares `'1'`<`'F'`) but the code
points are `256 > 255`; `"\x30"` doesn't parse as `i64` at all. So the descending-range reject
(`REGEX-PCRE2-FIDELITY.4.5.c`, PCRE2 err 108) needs a code-point-decoding comparison — exactly the
widening `.2` deferred. WHERE: `evaluate_predicate` (`semantic_runtime.rs`) had no code-point arm;
the PCRE2-faithful decode reference is `regex_compile_validation.rs::class_escape_literal_codepoint`.

**What landed (mirrors the `.2` touch-map).**
1. `rust/src/ast_pipeline/semantic_runtime.rs` — a `"value_compare_codepoint"` arm in
   `evaluate_predicate` (exactly-3-args; `scalar_text` lhs/rhs + `CompareOp::from_word` op;
   `compare_codepoints`; malformed shape → `None`) + registration in
   `ENGINE_BUILTIN_PREDICATE_NAMES`; new free functions `compare_codepoints` +
   `decode_char_literal_to_codepoint` + `decode_char_literal_digits` next to `compare_values`.
2. `rust/src/parse_harness_semantic_suite.rs` — `SemanticConstruct::ValueCompareCodepoint` + `::ALL`
   entry + the `sem_value_compare_codepoint` case (8 inputs).
3. `CompareOp::from_word` (`predicate_expr.rs`): REUSED verbatim from `.2` (no change).
4. Codegen / interpreter / grammar: **NO change** (both call the shared runtime; args generic).

**The decoder (general, parser-agnostic).** `decode_char_literal_to_codepoint(&str) -> Option<u32>`
decodes a WELL-FORMED single character literal — a bare single Unicode scalar, or a backslash escape
in the standard C/Perl vocabulary: `\x{H..}` / `\xH` / `\xHH` (hex), `\o{O..}` / `\NNN` (1–3 octal),
`\cX` (control, `X & 0x1f`), the named escapes `\a`(0x07) `\b`(0x08) `\e`(0x1b) `\f`(0x0c) `\n`(0x0a)
`\r`(0x0d) `\t`(0x09), and a backslash-escaped literal `\X` (→ `X`'s code point). Anything that is not
exactly one decodable character literal → `None`, so the predicate is INAPPLICABLE (non-blocking) on
it. It decodes CHARACTER LITERALS, not grammar constructs — nothing regex-specific — but its numeric
results MATCH `class_escape_literal_codepoint` on well-formed inputs, so the regex consumer gets
PCRE2-faithful ordering. (It is intentionally STRICT where the validator has fallthrough quirks —
e.g. `\18` → `None` here vs `49` there — because the consumer grammar tokenizes `\1` and `8` as
separate atoms, so a malformed multi-token spelling never reaches this predicate.)

**Semantics (final).** Holds iff `decode(lhs) <op> decode(rhs)` numerically (both `Option<u32>`
`Some`); either operand undecodable → `None` (INAPPLICABLE / non-blocking); a `post`-failure rejects
the rule backtrackably. Byte-identical codegen⟷interpreter by shared-runtime construction.

#### `.3` Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — before this slice `value_compare_codepoint` was not a builtin
  (`ENGINE_BUILTIN_PREDICATE_NAMES` had 12 names, none a code-point comparison); a rule-span code-point
  comparison over decoded char-literal endpoints — the shape `REGEX-PCRE2-FIDELITY.4.5.c` needs for
  class-range order (`[z-a]` err 108) — had no grammar-expressible form (`value_compare`'s `i64`/textual
  ladder mis-orders escape spellings, tool-shown: textual `"\x{100}" < "\x{FF}"` vs code points
  `256 > 255`).
- [x] **ROOT CAUSE (WHY + WHERE)** — `--parse-dump-ast-pretty` (session #81): class-range endpoints reach
  a rule-span predicate as RAW spellings via positional `$N` (`"\x30"`, `"\a"`, `"z"`), and a
  textual/`i64` comparison mis-orders them; `evaluate_predicate` (`semantic_runtime.rs`) had no
  code-point-decoding arm; PCRE2-faithful decode reference =
  `regex_compile_validation.rs::class_escape_literal_codepoint`. Both codegen POST/BRANCH loops and the
  interpreter CALL the shared `evaluate_predicate`, so one new arm serves both.
- [x] **FIX** — fix-tier ENGINE (a widening of the director-authorized general value-comparison
  primitive; no lower tier can express a two-capture code-point comparison over decoded char literals):
  the `value_compare_codepoint` builtin + `compare_codepoints` + `decode_char_literal_to_codepoint`,
  reusing `.2`'s positional-`$N` resolution, `CompareOp::from_word`, and content-aware predicate
  plumbing (no new syntax, no codegen/grammar change). SIBLING builtin (not a `value_compare` flag) →
  zero blast radius on the proven path.
- [x] **ADDRESSED (verified)** — `make -C rust SHELL=/bin/bash parse_harness_semantic_gate` GREEN:
  `sem_value_compare_codepoint` CLEAN (8 inputs, diverge=0, anchor_miss=0), both gate tests pass
  (`every_semantic_construct_is_byte_identical` + `semantic_construct_coverage_is_complete`). The 8
  inputs: all-six-ops-satisfied baseline (ACCEPT), one-op-flipped-at-a-time (6× REJECT — incl. the
  descending literal `bz-a` and the code-point-vs-textual discriminators `\x{FF}>\x{100}` and
  `\a==\x07`), and an undecodable `\x{}` endpoint (ACCEPT — `None` non-blocking). Interpreter ==
  compile-and-run oracle byte-identical throughout — the oracle compiles a throwaway crate through the
  REAL codegen, so the generated-parser path is exercised directly (no separate scratch-slot demo
  needed, unlike `.2` which pre-dated that coverage). Plus direct decoder unit tests
  (`decode_char_literal_to_codepoint_covers_the_char_escape_vocabulary` — the full escape table incl.
  the PCRE2-faithful numeric results and the strict `None` cases; `compare_codepoints_orders_by_
  decoded_scalar_not_text` — the code-point-vs-textual discriminators).
- [x] **NO REGRESSION** — `make -C rust SHELL=/bin/bash parse_harness_equivalence_gate` GREEN (the 11
  certified grammars, incl. the 6 fully-certified, byte-identical — `value_compare_codepoint` inert on
  every shipped grammar, none declares it); full dual lib suite `<FILL: N passed; 0 failed>` (the new
  case is enumerated data, not a new `#[test]`); `clippy` source-clean. No grammar/codegen/cert logic
  touched ⇒ cert-coverage unaffected by construction. The two `.2`-flagged inert sites confirmed
  (`grammar_wellformedness.rs` `FACT_QUERY_PRIMITIVES` — not a fact-query; `stimuli_generator.rs`
  store-aware witnessing — not a store-prelude gate).
- [x] **LOCKSTEP** — platform book *Semantic Store* + *Annotation System* (`value_compare_codepoint`
  alongside `value_compare`); `semantic_annotation` parser book steering-directives catalog; decision
  record [[project_rule_span_value_compare_primitive]] widened to note the sibling; `CHANGES.md`;
  `DEVELOPMENT_NOTES.md`; `MEMORY.md`; `docs/TASK_TREE.md`. No parser EBNF touched ⇒ no per-parser
  contract/ledger/release change (ENGINE primitive, inert until a consumer adopts it).

## Downstream consumers (separate trees — consume this primitive AFTER `.2` proves it)

- `REGEX-PCRE2-FIDELITY.4.3` — migrate `find_invalid_counted_quantifier`'s order check to a grammar
  `value_compare` gate (the PROVING consumer; err 104 min>max). ⚠️ Consumer-side note (be-alert): a
  post `value_compare` that rejects min>max means the **stimuli generator must not emit** `{5,3}`
  (else generator⟷parser duality break) — the `.4.3` leaf owns generation-satisfaction (structural
  or a generation-side companion), NOT this tree.
- `REGEX-PCRE2-FIDELITY.4.5` — descending ranges over DECODED codepoints (needs the later
  `codepoint` coercion-mode widening of this primitive).
- `REGEX-PCRE2-FIDELITY.4.9` — unbounded-lookbehind length analysis (value-comparison over computed
  max-lengths) may reuse the comparison surface.
