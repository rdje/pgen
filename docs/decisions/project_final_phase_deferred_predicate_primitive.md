---
name: project-final-phase-deferred-predicate-primitive
description: The `phase: final` @predicate phase — a GENERAL, parser-agnostic WHOLE-INPUT / parse-completion predicate that is checked ONCE, after the top-level parse succeeds, against the now-complete semantic store. Realized via DEFERRED OBLIGATIONS (each `phase: final` predicate resolves its args at rule-commit and enqueues a check; a terminal discharge pass runs them all at `parse_full` success) — backpatching generalized to a semantic check (LLVM `ForwardRefVals`→`validateEndOfModule`). Lets the EBNF own LEGAL-FORWARD-REFERENCE validation (a reference whose definition appears LATER in the input) that Pre/Branch/Post cannot express. A-vs-B DESIGN DECIDED (Option B, SOTA-cited) 2026-07-10; BUILD pending FINAL-PHASE-PREDICATE.2. Unlocks REGEX-PCRE2-FIDELITY .4.11 (named-ref UNKNOWN-name, the real accepts-invalid fix, REGEX-0098) + .4.7 (scs capture inventory).
metadata:
  node_type: memory
  type: project
  director_directive: true
  created: 2026-07-10
  owning_tree: FINAL-PHASE-PREDICATE
---

**THE PRIMITIVE.** `phase: final` is a fourth `@predicate` phase (after `pre` / `branch` / `post`). A
`final` predicate is **not** an inline gate on its own rule — the rule has already committed. It is a
**whole-input assertion**: it is checked ONCE, after the top-level parse succeeds and consumes the full
input, against the **now-complete** semantic store — so it can see facts emitted **anywhere** in the
input, including LATER than the rule that carries it. This is the missing capability for validating a
**legal forward reference** (a reference whose definition appears later than the reference).

```ebnf
# Reject a named backreference to a name never defined ANYWHERE in the pattern
# (PCRE2 err 115), while ACCEPTING a forward reference `\k<aa>(?'aa'x)`.
@emit_fact: { kind: regex_capture_name, name: $name }        # parse-side inventory (definition)
named_group := named_group_open_angle capture_name ">" pattern ")"

@predicate: { name: has_fact, args: [regex_capture_name, $name], phase: final }   # deferred check
backreference := "\\k<" capture_name ">"
```

- A `final` predicate resolves its args against the carrying rule's captured content **at rule commit**
  (so `$name` → the concrete name string is captured while the rule's context still exists), and enqueues
  a **deferred obligation** `(predicate_name, resolved_args, source_position)` onto the store's worklist.
- The obligation is transactional — it rides the same speculation rollback as `@emit_fact` (a speculative
  parse that fails discards its obligations), so only obligations on the FINAL successful tree survive.
- At top-level `parse_full` success, the engine **discharges** every enqueued obligation, in ascending
  source-position order, by evaluating it through the existing `evaluate_predicate` against the completed
  store. The FIRST failing obligation fails the whole parse with its position (matching a validator's
  first-error-by-position semantics). All obligations passing ⇒ the parse stands.
- `final` composes with the ENTIRE existing predicate vocabulary — `has_fact` (set membership: "name
  defined somewhere"), `fact_count_at_least` (whole-input count: "at least N groups exist"), `value_compare`,
  etc. It is the *when*, not the *what*.

It is the **whole-input generalization of `post`**: where `post` fires at the end of one rule and sees only
what has parsed so far, `final` fires at the end of the whole parse and sees the complete inventory.

**WHY (fix-hierarchy tier-5, tools-first).** A grammar cannot express "reference X must resolve against a
definition that may appear later in the input" — the classic legal-forward-reference need (regex
`\k<name>` / `(*scs:(N))` / subroutine calls; also declaration-order-free symbol use in real languages).
Every lower tier was exhausted, tool-backed, against the proving consumer (PCRE2 named references, `.3.22`
frozen oracle: 9 unknown-name spellings → err 115; `\k<aa>(?'aa'x)` / `(?&a)(?<a>x)` forward-ACCEPT):

- **Pre / Branch / Post are all per-rule, at the rule's parse position** (`semantic_runtime.rs:1001-1007`;
  fire order `ast_based_generator.rs:2030`/`:2078`/`:2100`/`:2133`). All fact-producing effects fire
  **post-body, left-to-right** (`:2100-2109`, branch-start `:3872-3890`); there is **no on-enter emit and
  no whole-input phase** (engine-wide grep: zero `two_pass`/`end_of_parse`/`second_pass`/terminal hits). So
  a `has_fact`/`post` gate on the reference fires **before** a later definition is emitted and would
  **reject the legal forward reference** (`\k<aa>(?'aa'x)`) — a rejects-valid regression. This is the
  ATTRIBUTE-GRAMMAR impossibility, not a coding gap: a forward reference is an attribute depending on a
  not-yet-visited (right) subtree ⇒ a right-to-left dependency ⇒ **non-L-attributed** ⇒ provably NOT
  single-left-to-right-evaluable (Dragon §5.2.3–5.2.4; Knuth 1968; Bochmann 1976 gives the Algol-60
  declaration/use scope example as the canonical multi-pass case).
- **`fact_count_at_least(K, N)` at `post` works only for a MONOTONE count** ("at least N groups so far") —
  it is why `numeric_backreference` is gated today (`regex.ebnf:415`). "Does name X exist ANYWHERE" is a
  for-all set-inclusion across the whole input, outside any per-call left-context predicate.
- **The only status-quo alternative is an out-of-band host validator** (`validate_regex_compile_contract`,
  run at `parser_registry.rs:1223 post_parse_semantic_contract`) — which is invisible to the stimuli
  generator and violates EBNF-single-source-of-truth ([[project_ebnf_is_single_source_of_truth]]). Deleting
  it is precisely the goal REGEX-PCRE2-FIDELITY is converging on; `.4.7` migrates `find_invalid_scan_substring_capture_list`
  INTO the grammar via this primitive, and `.4.11` closes a divergence the validator never even checked.

The store already retains the whole-input inventory at parse completion (facts are global and **never
retracted on `@close_scope`** — `semantic_runtime.rs` never truncates `self.facts` on close; the SCP
decision documents the same fact), and `parse_full` is a single clean point where the top-level tree has
committed and no speculation can roll it back. So the honest, general answer is a phase that **discharges
the check there**, not a second parse and not a host validator.

**A-vs-B DECISION (Option B, SOTA-cited).** Two theoretically-sound ways to make the whole-input
inventory available to a forward-reference check:

- **(A) explicit two-pass / pre-scan** — a dedicated pass builds the complete inventory first, then the
  main pass checks references against it. Users: two-pass assembler (Beck §2.1); **PCRE2** `parse_regex`
  pre-pass + sizing run (`HACKING`: the compile needs "full knowledge of group names and numbers
  throughout"); .NET `CountCaptures`→`ScanRegex`; ordered/multi-visit attribute grammars (Kastens 1980).
- **(B) single pass + deferred obligations, terminal discharge** — one authoritative pass; each unresolved
  reference registers an obligation; a terminal end-of-parse phase discharges all obligations against the
  now-complete store, erroring on any left open. Users: Dragon §6.7 **backpatching** (`makelist`/`merge`/
  `backpatch`); **LLVM** `LLParser` — `ForwardRefVals` recorded via `createGlobalFwdRef`, discharged by
  `validateEndOfModule()`, hard-error `"use of undefined value"`; Rust/Roslyn late name-binding over the
  built tree; ANTLR/tree-sitter/scope-graphs resolve over the completed structure.

**Chosen: B.** The single most useful finding from the survey is the **discriminator**: PCRE2 pays for a
pre-pass **not because forward references exist**, but because an *irreversible global aggregate* — how much
memory to allocate, which opcodes to emit — depends on the whole-pattern group count and MUST be known
*before* emission. A pure reference **validator** has no such up-front dependency: nothing irreversible
happens at the reference site that needs the answer now, so it can defer. And in a grammar-driven generator,
Design A would force **re-running the grammar's structural recognition** just to find the definitions —
exactly the price PCRE2 pays (a *separate hand-written* `parse_regex` distinct from its compiler), duplicated
logic that can drift from the real grammar. Design B **reuses the one authoritative parse PGEN already owns**
and its store, so the terminal check cannot diverge from the parse that produced it — the same reason Rust/
Roslyn bind over the built AST rather than re-scanning source. B is one general parser-agnostic primitive; A
is per-consumer bespoke. (Reserve A only if a future *structural/layout* engine decision genuinely needs a
global count fixed before the main pass — the PCRE2 case — and keep it out of the reference-validation path.)

**FROZEN BUILD SPEC (FINAL-PHASE-PREDICATE.2 — engine slice, tier-5, mirrored + gated).**
1. **Phase enum** `semantic_runtime.rs:1001-1007` — add `Final`; parse it `:1009-1018` (accept `final`,
   aliases `parse_complete` / `whole_input`); widen the reject message `:4200-4206` to list it.
2. **Registration (at commit, resolve-now).** In codegen classify predicates: `{pre, branch, post}` stay
   inline gates; `final` predicates are routed into the post-body effect path (`ast_based_generator.rs:2100-2109`,
   interpreter `parse_harness_interpreter.rs:940`). There, resolve the predicate's args against the rule's
   captured content and push `DeferredObligation { predicate_name, resolved_args, source_position }` onto a
   new `SemanticRuntimeState.deferred_obligations: Vec<…>`.
3. **Rollback.** Extend `SemanticRuntimeCheckpoint` (`semantic_runtime.rs:1668-1683`) with `deferred_len`;
   `checkpoint()` records it, `rollback_to_named()` truncates `deferred_obligations` to it (mirrors `fact_len`).
4. **Terminal discharge.** New `SemanticRuntimeState::discharge_deferred_obligations(&self) -> Result<(), (pos,msg)>`
   — sort obligations by `source_position` ascending, evaluate each via `evaluate_predicate(predicate_name,
   resolved_args)` against the completed store, return the first that is `Some(false)`. Called at
   `parse_full()` success (`ast_based_generator.rs:1649-1663`, after full-consume confirmed) and at the
   interpreter's top-level completion (`interpret_parse_gen_ast_core` return); on `Err` return
   `ParseError::InvalidSyntax { message, position }`.
5. **Generation side — NO new gen primitive.** `final` predicates are gen-neutral for pruning (they do not
   gate a branch during the pass), so the generator's existing catch-all safely ignores them
   (`stimuli_generator.rs:7759`). Each consumer keeps its existing conservative `@gen_predicate` draw (only
   references already-emitted names ⇒ sound, never emits an invalid forward ref), so there is **no
   `lacks_fact`-branch-prune duality break** — the trap that killed `.4.8`'s fact-based A′
   ([[project_gen_side_no_lacks_fact_branch_prune]]). The gen dual for `.4.11` is the same `@gen_predicate
   has_fact(regex_capture_name, …)` idiom `.4.7`/scs already ships.
6. **Interpreter mirror + gates (lockstep).** Implement identically in `semantic_runtime.rs` (shared),
   `ast_based_generator.rs` (codegen), `parse_harness_interpreter.rs` (mirror). Add `phase: final` cases to
   `parse_harness_semantic_suite.rs` (hit / miss / FORWARD-ref accept / rollback-under-speculation) and teach
   `semantic_construct_coverage_is_complete` the new phase; `parse_harness_equivalence.rs` +
   `parse_harness_combinator_suite.rs` stay green. Prove the primitive **in isolation before any consumer**
   (the RSVC/SCP model): a tiny grammar whose `phase: final has_fact` accepts a forward reference and rejects
   an undefined one, byte-identical interpreter vs compile-and-run.

**CONSUMER MAPPING.**
- **`.4.11` (named-reference UNKNOWN-name; the ONLY real accepts-invalid FIX, ledger REGEX-0098).** Add
  parse-side `@emit_fact regex_capture_name` on every group-name definition (currently `@gen_emit_fact`-only,
  `regex.ebnf:1045`), and `@predicate has_fact(regex_capture_name, $name) phase: final` on the 5 reference
  rule families (`backreference`, `subroutine_named`, `named_braced`, `subroutine_call`,
  `python_named_backreference`). Frozen acceptance spec = the `.3.22` oracle matrix.
- **`.4.7` (scs capture inventory; behavior-NEUTRAL validator→grammar migration).** scs NAMED item →
  `has_fact(regex_capture_name, $name) phase: final`; scs PLAIN-NUMERIC item → `fact_count_at_least(regex_capture_group,
  $N) phase: final` (whole-input "≥ N groups exist"). scs RELATIVE `+N`/`-N` needs prior-count arithmetic to
  resolve the absolute ref before the final availability check — the one wrinkle, resolved in `.4.7`'s BUILD
  slice (record the parse-time prior count as an obligation attribute), not a blocker for the primitive.
  Deletes `find_invalid_scan_substring_capture_list`.

**GENERAL RULE FOR GRAMMAR AUTHORS.** For a **legal-forward-reference** check ("reference must resolve
against a definition that may appear anywhere, including later"), emit a `regex_capture_name`-style
definition fact and gate the reference with `phase: final` — **not** `post` (which fires before a later
definition exists and rejects the legal forward reference) and **not** a host validator. Reserve a real
pre-pass only for an irreversible pre-emission global aggregate. Same director-authorized engine-primitive
umbrella (2026-07-09/2026-07-10) as [[project_rule_span_value_compare_primitive]] and
[[project_scope_context_predicate_primitive]]; grounded in [[feedback_research_grounded_sota_no_trial_and_revert]],
[[feedback_no_workarounds_fix_hierarchy]], [[feedback_features_parser_agnostic_enable_all_parsers]],
[[feedback_correctness_before_speed]].
