# FINAL-PHASE-PREDICATE — a general `@predicate phase: final` whole-input / parse-completion gate (deferred obligations)

## Metadata

- Tree ID: `FINAL-PHASE-PREDICATE`
- Status: **`complete`** — all three children landed. **`.3` BUILD LANDED** 2026-07-11 (session #88, `PGEN-FPP-0004`): the multi-branch `view` resolver-robustness follow-up (surfaced by the FIRST consumer `REGEX-PCRE2-FIDELITY.4.11`) — the Option (b)-refined other-view fallback on a NAMED-reference miss, retiring the per-consumer `view: shaped` workaround; INERT on all 11 shipped grammars (byte-identical), semantic-suite 36/36 CLEAN. **`.1` DESIGN DECIDED** (2026-07-10, session #86, `PGEN-FPP-0001`, PURE-DOCS):
  Option B, SOTA-cited, engine BUILD SPEC frozen. **`.2` BUILD LANDED** (2026-07-10, session #86,
  `PGEN-FPP-0002`): the `phase: final` engine primitive shipped identically in the shared runtime, codegen,
  and interpreter mirror, with new semantic-suite cases + coverage, proven in isolation before any consumer.
  The director required the A-vs-B choice settled in a SOTA-cited DESIGN slice BEFORE any engine code, so
  this tree split design from build; both are now landed. The proving CONSUMERS (`REGEX-PCRE2-FIDELITY.4.11`
  then `.4.7`) are leaves of a DIFFERENT tree whose frontier now passes (the RSVC/SCP model).
- Family / slice-id prefix: `PGEN-FPP-<NNNN>` (abbreviation of the tree name; used in commit subjects)
- Roadmap lane: cross-cutting engine correctness — a **parser-agnostic** WHOLE-INPUT predicate phase that
  lets the EBNF validate a **legal forward reference** (a reference whose definition may appear LATER in the
  input than the reference), which the per-rule `pre`/`branch`/`post` phases provably cannot express. This
  is the general "reference X must resolve against a definition that may appear anywhere" capability
  (regex `\k<name>` named backrefs, `(*scs:(N))` scan-substring capture lists, subroutine calls; also
  declaration-order-free symbol use in real languages). Aligns with the EBNF-single-source-of-truth
  doctrine ([[project_ebnf_is_single_source_of_truth]]). Direct sibling of the `RULE-SPAN-VALUE-CONSTRAINT`
  (`value_compare`) and `SCOPE-CONTEXT-PREDICATE` (`in_scope_kind`) primitive trees.
- Director authorization: 2026-07-10 (session #84 close, recorded verbatim in
  `REGEX-PCRE2-FIDELITY.md` §PRIMITIVE-GAP-SCOPING and `MEMORY.md`): on seeing the SURFACED
  whole-pattern-inventory primitive the director replied *"Please do what you see fit to achieve sota,
  signoff level quality."* Binding constraints carried forward: (1) survey the literature FIRST and act on
  a citation + worked mapping ([[feedback_research_grounded_sota_no_trial_and_revert]]); (2) parser-AGNOSTIC,
  all-parsers ([[feedback_features_parser_agnostic_enable_all_parsers]]); (3) correctness is the floor
  ([[feedback_correctness_before_speed]]); (4) settle A-vs-B in a SOTA-cited DESIGN slice BEFORE any engine
  code, and state the fix-hierarchy level + why nothing lower works ([[feedback_no_workarounds_fix_hierarchy]]).
  Same engine-primitive umbrella as RSVC / SCP.
- Fix-hierarchy justification (tier-5 engine, tool-backed): the proving consumer
  (`REGEX-PCRE2-FIDELITY.4.11` named-reference UNKNOWN-name, `.3.22` oracle: 9 unknown-name spellings →
  err 115; `\k<aa>(?'aa'x)` / `(?&a)(?<a>x)` forward-ACCEPT) is a **legal-forward-reference** question.
  Tools-first exhaustion of the lower tiers (see §Design and the decision record
  [[project_final_phase_deferred_predicate_primitive]]):
  - tier 1/2 (existing annotations / store): all three phases `pre`/`branch`/`post` are per-rule at the
    rule's parse position (`semantic_runtime.rs:1001-1007`), and ALL fact-producing effects fire post-body,
    left-to-right (`ast_based_generator.rs:2100-2109`, branch-start `:3872-3890`) — there is NO on-enter
    emit and NO whole-input phase (engine-wide grep: zero terminal-phase hits). So a `has_fact`/`post` gate
    on a reference fires BEFORE a later definition is emitted and would REJECT the legal forward reference
    (`\k<aa>(?'aa'x)`) — a rejects-valid regression. This is the attribute-grammar impossibility: a forward
    reference is a right-to-left dependency ⇒ non-L-attributed ⇒ provably not single-left-to-right-evaluable
    (Dragon §5.2.3–5.2.4; Knuth 1968; Bochmann 1976). `fact_count_at_least` at `post` works only for a
    MONOTONE count; whole-input set-inclusion is outside any per-call left-context predicate.
  - tier 3/4 (new annotation / new store schema): not an annotation-shape or fact-schema gap — the facts
    and the query vocabulary already exist; what is missing is a PHASE that discharges them once the store
    is complete.
  - tier 5 (engine): a fourth predicate phase `final`, checked once at `parse_full` success against the
    completed store, realized via deferred obligations (backpatching generalized — LLVM `ForwardRefVals` →
    `validateEndOfModule`). The store already retains the whole-input inventory at completion (facts are
    global, never retracted on `@close_scope`) and `parse_full` is a single clean discharge point, so the
    engine delta is small: one phase variant + a deferred worklist (transactional) + a terminal drain.

## Goal

Expose a **general, parser-agnostic** fourth `@predicate` phase on the existing surface:

```ebnf
@predicate: { name: <primitive>, args: [<args>], phase: final }   # checked ONCE at whole-input parse completion
```

- A `final` predicate is not an inline gate on its own rule (the rule has already committed). It resolves
  its args against the carrying rule's captured content **at rule commit**, enqueues a **deferred
  obligation** `(predicate_name, resolved_args, source_position)` (transactional — rolls back with
  speculation), and is checked ONCE — after the top-level parse succeeds and consumes the full input —
  against the **now-complete** semantic store, so it can see facts emitted ANYWHERE, including later than
  the rule that carries it.
- It composes with the whole existing predicate vocabulary (`has_fact`, `fact_count_at_least`,
  `value_compare`, …): `final` is the *when*, not the *what*. It is the whole-input generalization of `post`.
- The first obligation (by ascending source position) that evaluates to `Some(false)` fails the whole parse
  with its position (validator first-error-by-position semantics); all passing ⇒ the parse stands.

## Non-Goals

- **Not** a two-pass parse / pre-scan (Option A). Rejected in the design: A is warranted only when an
  *irreversible pre-emission global aggregate* forces the count to be known before the main pass (PCRE2's
  memory-sizing/opcode case); a reference *validator* has no such dependency, and A would force re-running
  the grammar's structural recognition (divergence risk). See the decision record for the SOTA discriminator.
- **Not** a new generation-side primitive. `final` predicates do not gate branch selection during the pass,
  so the generator ignores them safely; each consumer keeps its existing conservative `@gen_predicate` draw
  (references only already-emitted names ⇒ sound). No `lacks_fact`-branch-prune duality break
  ([[project_gen_side_no_lacks_fact_branch_prune]]).
- **Not** a regex special case. The primitive is grammar-agnostic; regex is merely the proving consumer.
- **Not** arithmetic in the predicate body (unchanged). The scs RELATIVE `+N`/`-N` prior-count resolution is
  a `.4.7`-BUILD application detail (carry the parse-time prior count as an obligation attribute), not a
  primitive feature.

## Acceptance Criteria

- `.1` (this slice): A-vs-B settled (Option B) with SOTA citations + a worked PGEN mapping; the engine BUILD
  SPEC frozen; fix-hierarchy tier-5 justified tools-first; decision record + tree + regex-tree cross-refs
  landed. NO engine/grammar/codegen/generated change (PURE-DOCS).
- `.2` (BUILD): `phase: final` implemented identically in `semantic_runtime.rs` (shared),
  `ast_based_generator.rs` (codegen), `parse_harness_interpreter.rs` (mirror); transactional obligation
  worklist with checkpoint/rollback; terminal discharge at `parse_full` success + interpreter completion.
  PROVEN IN ISOLATION before any consumer (a tiny grammar whose `phase: final has_fact` accepts a forward
  reference and rejects an undefined one, byte-identical interpreter vs compile-and-run). NO-REGRESSION:
  `parse_harness_equivalence_gate` + combinator suite green; `parse_harness_semantic_suite` gains
  `phase: final` cases (hit / miss / forward-accept / rollback-under-speculation) and
  `semantic_construct_coverage_is_complete` recognizes the new phase; inert on all shipped grammars;
  `semantic_full_contract_gate` differential baseline-unchanged; clippy source-clean; mdBook +
  semantic-store book subsection lockstep.
- Consumers (`REGEX-PCRE2-FIDELITY.4.11` then `.4.7`) are leaves of a DIFFERENT tree — the FRONTIER passes
  there once `.2` lands (the RSVC/SCP model), each in its own released slice.

## Task Tree

- ID: `FINAL-PHASE-PREDICATE`  Status: `active`  Children: `.1`, `.2`, `.3`
- ID: `.1`  Status: **`done`** (`PGEN-FPP-0001`, 2026-07-10 session #86, PURE-DOCS)  Goal: settle the
  A-vs-B design (Option B, SOTA-cited) + freeze the engine BUILD SPEC + fix-hierarchy tier-5 justification.
  Acceptance: the decision record [[project_final_phase_deferred_predicate_primitive]] + this tree + the
  regex-tree cross-refs (`REGEX-PCRE2-FIDELITY.4.7`/`.4.11` notes) + resume-pointer update; no code change.
  Method: two tool-backed research streams (engine ground-truth by `file:line`; SOTA literature survey),
  then synthesis. Evidence: see §Design.
- ID: `.2`  Status: **`done`** (`PGEN-FPP-0002`, 2026-07-10 session #86)  Goal: BUILD the `phase: final`
  primitive per the frozen spec in §Design / the decision record. Acceptance: the `.2` acceptance criteria
  above, proven in isolation before any consumer, full parse-harness lockstep. Fix-hierarchy tier-5
  (engine). Landed identically in `semantic_runtime.rs` (shared core: `Final` phase, `DeferredObligation`,
  worklist, checkpoint `deferred_len` + rollback truncation, delta capture/replay, enqueue + discharge),
  `ast_based_generator.rs` (codegen: enqueue-at-commit + `parse_full`/`parse_full_from` discharge + raw
  gate + phase serializer), `parse_harness_interpreter.rs` (mirror). Semantic suite gains
  `FinalForwardGate` + `FinalRollbackSpeculation`; coverage-completeness recognizes both. Proven in
  isolation (scratch-slot generated parser: forward-ACCEPT `use a;decl a;` rc 0, undefined-REJECT
  `use a;decl b;` rc 1) BEFORE any consumer. See the Acceptance Checklist below.
- ID: `.3`  Status: **`done`** (`PGEN-FPP-0004`, BUILD LANDED 2026-07-11 session #88 on a clean repo, per
  the frozen spec routed by `PGEN-REGEX-PCRE2-0045`; decision MADE + build spec FROZEN in session #87. The
  Option (b)-refined resolver fallback shipped identically in the codegen-emitted resolver
  (`ast_based_generator.rs`) + interpreter mirror (`parse_harness_interpreter.rs`) with the new
  `semantic_reference_is_named` named-vs-positional helper; `semantic_runtime.rs:3572` needed no change
  (`content_kind_is` resolves no `$ref`). New MULTI-BRANCH default-view semantic-suite case
  `sem_final_multibranch_shaped_ref`. See the Acceptance Checklist `.3` below.)  Goal:
  CLOSE the multi-branch `view` FOOTGUN
  that the FIRST consumer (`REGEX-PCRE2-FIDELITY.4.11`) surfaced — make the shared semantic-predicate
  ARGUMENT RESOLVER resolve a NAMED reference against whichever content-view actually holds it, so a `final`
  (or any-phase) predicate on a MULTI-BRANCH rule resolves a shaped-key `$ref`/`$name` under the DEFAULT
  `view: raw` EXACTLY as a single-branch rule already does — removing the per-consumer `view: shaped`
  workaround `.4.11` had to use.

  **DECISION (director-authorized 2026-07-11, verbatim *"take whatever decision, route you need to take
  because you are the expert coder here … it needs to be sota level, signoff level quality"*): Option
  (b)-refined — a resolver FALLBACK, NOT a default-flip.** When a NAMED / object-key reference
  (`$name`/`$ref`/dotted/name-indexed) is ABSENT from the `view`-selected content, fall back to the OTHER
  view's content BEFORE raising the unresolved-attribute error. Chosen over Option (a) "default `final`
  predicates to `view: shaped`" because (b): (1) fixes the ROOT INCONSISTENCY — single-branch rules resolve
  shaped keys under the `view: raw` default only by the `semantic_raw_content=None` shaped-fallback ACCIDENT
  (see [[project_final_phase_deferred_predicate_primitive]] FIRST-CONSUMER FINDING), while a multi-branch
  tournament captures the winner's raw `Sequence` and cannot; (b) makes them IDENTICAL; (2) applies to ALL
  phases (pre/branch/post/final), not just final; (3) changes NO default (no surprise for a raw/positional
  `$N`/`$text`-intending predicate); (4) NEVER masks a real typo — a key absent from BOTH views still errors.

  **WHERE (tools-first map — VERIFY by re-reading before editing; the FPP.2 3-place mirror discipline):**
  view→content selection archetype `rust/src/ast_pipeline/semantic_runtime.rs:3572-3574`
  (`match predicate.view { Raw => raw_content, Shaped => shaped_content }`); the codegen-emitted parallel
  raises `"Semantic runtime could not resolve attribute reference '{}'"` at
  `rust/src/ast_pipeline/ast_based_generator.rs:~2558` (emitted into every generated parser via
  `resolve_semantic_predicate_spec_against_content`); the interpreter mirror is in
  `rust/src/parse_harness_interpreter.rs`. Apply the fallback at EVERY resolution site. The fallback fires
  ONLY on a NAMED-reference miss in the primary view — positional `$N` / `$text` semantics are UNCHANGED.

  **INERT-ON-SHIPPED (the no-regression proof to reproduce):** NO shipped grammar has a `view: raw`
  MULTI-BRANCH shaped-key predicate — regex's three `.4.11` named-ref gates carry EXPLICIT `view: shaped`
  (resolve shaped directly, never hitting the fallback), and every other predicate is either single-branch
  (already `None`-fallback) or explicit-shaped ⇒ byte-identical across all 11 certified grammars + the regex
  `.3.22` matrix 17/17. AFTER landing, the regex explicit `view: shaped` MAY be simplified to the default
  (a stronger proof the fixed default works) OR kept as self-documenting intent — leaf-decision (keeping
  avoids a regex regen + re-verify; either is signoff-clean).

  **COVERAGE-GAP FIX (mandatory — the exact case the `.2` proof missed):** add a MULTI-BRANCH `phase: final`
  shaped-key case to `rust/src/parse_harness_semantic_suite.rs` that uses the DEFAULT view (NO explicit
  `view: shaped`) — e.g. a 2-branch reference rule `ref := "\\k<" name ">" -> {t:"a", r:$2} | "\\g<" name
  ">" -> {t:"b", r:$2}` with `@predicate has_fact(decl, $r) phase: final` — forward-ACCEPT vs
  undefined-REJECT. This is PRECISELY the case the single-branch `FinalForwardGate`/`FinalRollbackSpeculation`
  (session #86) could not exercise; the coverage-completeness recognizer must accept it. A scratch-slot
  multi-branch `view: raw` shaped-key repro (accept/reject rc) is the isolation proof, mirroring FPP.2.

  **ACCEPTANCE (signoff):** root cause already proven (`.4.11`, `--trace-rules`); the new multi-branch
  semantic-suite case passes under the DEFAULT view (compiled parser AND interpreter byte-identical);
  `parse_harness_semantic_gate` all-CLEAN; `parse_harness_equivalence_gate` 11/11 byte-identical;
  `parse_harness_combinator_gate` intact; regex `.3.22` matrix 17/17; regex cert `fully_certified`;
  `ast_shape_contract` aligned; `duality_hunt`; clippy; full lockstep (book `semantic-store.md` `view` note
  softened from "`view: shaped` is REQUIRED" to "resolution falls back to whichever view holds the key;
  explicit `view: shaped` is now optional self-documentation" + this tree + decision record +
  CHANGES/DEVELOPMENT_NOTES/LIVE_ACHIEVEMENT_STATUS/MEMORY). NO release/contract/schema/ledger bump expected
  (a general engine-robustness improvement, INERT on every shipped parser). Fix-hierarchy tier-5 (engine) —
  the footgun is an engine-resolution inconsistency, not fixable at grammar/annotation tier except by the
  per-consumer `view: shaped` workaround this leaf retires.

## Acceptance Checklist (enforced) — `.2` BUILD

- [x] **REPRODUCE / ISSUE** — A grammar cannot validate a **legal forward reference** (definition
  appears LATER than the reference). Tool-verified (session #86): the `@predicate` phase enum is exactly
  `Pre | Branch | Post` (`semantic_runtime.rs`, `enum SemanticPredicatePhase`), all per-rule; every
  fact-producing effect fires post-body left-to-right; engine-wide grep for a whole-input/terminal phase =
  0 hits. So a `has_fact`/`post` gate on a reference fires BEFORE a later definition is emitted and would
  REJECT the legal forward reference. The proving consumer is `REGEX-PCRE2-FIDELITY.4.11` (`.3.22` oracle:
  9 unknown-name spellings → err 115; `\k<aa>(?'aa'x)` / `(?&a)(?<a>x)` forward-ACCEPT).
- [x] **ROOT CAUSE (WHY + WHERE)** — the **attribute-grammar impossibility**: a forward reference is a
  right-to-left (non-L-attributed) dependency ⇒ provably not single-left-to-right-evaluable (Dragon
  §5.2.3–5.2.4; Knuth 1968; Bochmann 1976). WHERE: the phase enumeration (`semantic_runtime.rs`
  `SemanticPredicatePhase`) has no terminal/whole-input phase; `parse_full` (`ast_based_generator.rs`) had
  no discharge hook. Not a fact-schema or annotation-shape gap — the facts + query vocabulary already
  exist; the missing piece is a PHASE that discharges once the store is complete. Fix-hierarchy tier-5
  (engine): lower tiers exhausted tools-first (see the decision record).
- [x] **FIX** — the `phase: final` deferred-obligation predicate (tier-5 engine), landed identically in
  the shared core (`semantic_runtime.rs`: `Final` phase + `DeferredObligation` + worklist + checkpoint
  `deferred_len` + rollback truncation + delta capture/replay + `enqueue_deferred_obligation` +
  `discharge_deferred_obligations`), codegen (`ast_based_generator.rs`: enqueue-at-commit +
  `parse_full`/`parse_full_from` discharge + raw gate + phase serializer), and interpreter mirror
  (`parse_harness_interpreter.rs`). Per the frozen BUILD SPEC; no lower tier can see a forward reference.
- [x] **ADDRESSED (verified)** — **isolation proof in a REAL generated parser** (scratch slot): forward
  reference `use a;decl a;` → ACCEPT (rc 0); undefined `use a;decl b;` → REJECT (rc 1: `whole-input
  predicate 'has_fact' not satisfied at parse completion`, `furthest_position=11`). A `post` gate could
  NOT accept the forward case — the exact discriminator. The `.6.2` semantic gate's two new cases —
  `sem_final_forward_gate` and `sem_final_rollback_speculation` — are **CLEAN (`diverge=0 anchor_miss=0`)**:
  the interpreter is byte-identical to the compile-and-run oracle AND the independent accept/reject anchors
  hold (forward-accept, undefined-reject, losing-branch-obligation-discarded, winning-branch-discharge).
- [x] **NO REGRESSION** — `parse_harness_semantic_gate` **35/35 CLEAN** (`2 passed; 0 failed`,
  finished 244s): the 33 pre-existing cases stay `diverge=0` ⇒ the codegen+interpreter changes are
  byte-neutral for them. `parse_harness_equivalence_gate` — the 11 shipped/certified grammars still
  byte-identical (inert on shipped grammars: no grammar uses `phase: final`, the enqueue loop iterates an
  empty `final_predicates_for_rule`, the raw-capture OR-in returns false, the discharge is a no-op with an
  empty worklist). `parse_harness_combinator_gate` — 27/27 structural intact. `cargo check`/clippy clean.
  Determinism: the suite is fixed grammars × curated inputs (no seeds).
- [x] **LOCKSTEP** — book `docs/book/src/semantic-store.md` (the `final` phase + forward-reference
  subsection + quick-refs) + `docs/book/src/parse-harness.md` (2 construct rows + case count 29→35) +
  `TOOLBOX.md` §1.8 (32→35 + the `final` construct) + decision record `BUILD LANDED` anchors + this tree +
  `MEMORY.md` + `CHANGES.md`. No downstream contract/ledger/schema bump (a new engine capability, inert on
  every shipped parser — no released-parser behavior changed).

## Acceptance Checklist (enforced) — `.3` BUILD

- [x] **REPRODUCE / ISSUE** — a MULTI-BRANCH rule carrying a `phase: final` predicate whose arg is a
  SHAPED object-key `$ref` under the DEFAULT `view: raw` REJECTS even a DEFINED name. Root-caused
  tools-first in session #87 via `PGEN_TRACE_VERBOSITY=debug … --trace-rules named_backreference` (the
  trace named the exact resolver error *"could not resolve attribute reference 'ref'"* + rule stack): a
  multi-branch tournament captures the winner's raw `Sequence` for the final-predicate view, so the shaped
  key cannot resolve against a `Sequence` (there is no `ref` element); a single-branch rule resolves it
  only by the `semantic_raw_content == None` shaped-fallback ACCIDENT — the exact case the `.2`
  single-branch isolation proof missed.
- [x] **ROOT CAUSE (WHY + WHERE)** — an engine-resolution INCONSISTENCY between single-branch and
  multi-branch carriers. WHERE (tool-verified by re-reading before editing): the codegen-emitted
  argument resolver `ast_based_generator.rs`
  `resolve_semantic_predicate_spec_against_content` / `try_…` (the *"could not resolve attribute
  reference"* error at the `RuleReference` arm of `resolve_unified_semantic_value_against_content`) +
  its interpreter mirror `parse_harness_interpreter.rs`. `semantic_runtime.rs:3572`
  (`evaluate_content_aware_predicate`) consumes `selected_content` only for `content_kind_is` (a
  content-KIND query, not a `$ref` resolution), so it has no reference to fall back on → NO change there.
  The `final`-obligation enqueue resolves against `(semantic_raw_content, node.content)` at
  `ast_based_generator.rs:~2276`.
- [x] **FIX** — Option (b)-refined resolver fallback (fix-hierarchy tier-5 engine; lower tiers can only
  offer the per-consumer `view: shaped` workaround this leaf retires). Each predicate-spec resolver
  computes `(selected_content, fallback_content)` from `spec.view` (the OTHER view is the fallback); the
  `RuleReference` arm resolves against `selected_content` then, on a miss, retries against
  `fallback_content` **iff** `Self::semantic_reference_is_named(reference)` (a new shared helper mirroring
  `resolve_semantic_reference`'s named-vs-positional split — `$`+digit is positional and NEVER retried, so
  positional `$N` / `$text` semantics are UNCHANGED; a key absent from BOTH views still errors). Threaded
  through the value/try/properties resolvers. `@emit_fact` (an effect, not a predicate) passes its own
  content as the inert fallback (byte-identical). Landed identically in codegen + interpreter mirror.
- [x] **ADDRESSED (verified)** — scratch-slot isolation in a REAL generated parser: multi-branch `ref :=
  "k<" word ">" -> {r:$2.body} | "g<" word ">" -> {r:$2.body}` with `has_fact(name_decl, $r) phase: final`
  (DEFAULT view) → `k<a>decl a;` ACCEPT (rc 0), `g<a>decl a;` ACCEPT (rc 0, the OTHER branch),
  `k<a>decl b;` REJECT (rc 1: `whole-input predicate 'has_fact' not satisfied … args [Identifier("name_decl"),
  Identifier("a")]` — the shaped key `$r` resolved to the concrete `a` VIA the fallback, then correctly
  failed as undefined). Before the fix this REJECTED even the defined name. `parse_harness_semantic_gate`
  **36/36 CLEAN** — the new `sem_final_multibranch_shaped_ref` is byte-identical (interpreter ==
  compile-and-run oracle) with all 3 anchors holding; `semantic_construct_coverage_is_complete` recognizes
  the new `FinalMultiBranchShapedRef` construct.
- [x] **NO REGRESSION** — `parse_harness_equivalence_gate`
  `certified_grammars_are_byte_identical` GREEN over all **11** shipped parsers **regenerated with the new
  codegen** (INERT-ON-SHIPPED confirmed — regex's `.4.11` gates keep explicit `view: shaped`, resolving
  in-primary and never touching the fallback); `parse_harness_combinator_gate` 27/27; the 35 prior
  semantic cases stay `diverge=0`; `duality_hunt_gate` pinned signatures unchanged (regex+svpp, seeds
  0/7/42); `regex_ast_shape_contract_gate` aligned=4 drift=0; regex cert `--entry-rule regex --count 40`
  seeds 0/7/42 **UNKNOWN=0 fully_certified=true** (witness 258, spf 0/1/1 — byte-identical to the `.4.11`
  baseline); named-ref spot-check `\k<aa>(?'aa'x)` / `(?&a)(?<a>x)` ACCEPT + `\k<zzz>` REJECT intact;
  Stage-1 source clippy exit 0.
- [x] **LOCKSTEP** — book `docs/book/src/semantic-store.md` (`view` note softened from "REQUIRED" to
  "optional self-documentation" + example drops explicit `view: shaped`) + `docs/book/src/parse-harness.md`
  (new construct row + case count 35→36) + `TOOLBOX.md` §1.8 (35→36 + the multi-branch case) + decision
  record `[[project_final_phase_deferred_predicate_primitive]]` `.3 BUILD LANDED` anchor + this tree +
  `CHANGES.md` + `DEVELOPMENT_NOTES.md` + `LIVE_ACHIEVEMENT_STATUS.md` + `MEMORY.md`. No
  release/contract/schema/ledger bump (a general engine-robustness improvement, inert on every shipped
  parser — no released-parser behavior changed).

## Design

The full SOTA survey, the A-vs-B comparison table, the discriminator, and the frozen engine BUILD SPEC
(phase enum + resolve-at-commit registration + transactional rollback + terminal discharge + generation
duality + interpreter mirror + parse-harness coverage + consumer mapping) live in the decision record
[[project_final_phase_deferred_predicate_primitive]]. Summary:

- **Chosen: Option B** (single pass + deferred obligations, terminal discharge). The SOTA discriminator:
  Option A (pre-pass) is warranted only for an irreversible pre-emission global aggregate (PCRE2 memory
  sizing); a reference validator has no such dependency, so it defers. B reuses PGEN's single authoritative
  parse + store (cannot diverge from the parse), is one general primitive, and is the pattern LLVM
  (`ForwardRefVals`→`validateEndOfModule`), classic backpatching (Dragon §6.7), and Rust/Roslyn late-binding
  converge on.
- **Engine ground-truth (tool-verified, session #86):** phase enum Pre|Branch|Post only
  (`semantic_runtime.rs:1001-1007`, reject `:4200`); no terminal/whole-input phase (grep 0 hits); all
  fact-effects post-body (`ast_based_generator.rs:2100`, branch-start `:3872`); store facts global +
  never retracted on close ⇒ complete whole-input inventory at `parse_full()` success
  (`ast_based_generator.rs:1649-1663`); `try_parse` snapshot/rollback `:6538-6658` (checkpoint
  `semantic_runtime.rs:1668-1683`); interpreter mirror `parse_harness_interpreter.rs:154/900`; gen side
  honors only `fact_count_at_least`/`has_fact`/`fact_attribute_equals`, catch-all ignores the rest
  (`stimuli_generator.rs:7759`) ⇒ gen dual = existing conservative `@gen_predicate` draw; harness gates
  `parse_harness_equivalence.rs` / `_combinator_suite.rs` / `_semantic_suite.rs`.
- **Consumers.** `.4.11` (named-ref UNKNOWN-name; the only real accepts-invalid FIX, REGEX-0098): parse-side
  `@emit_fact regex_capture_name` on definitions + `has_fact(regex_capture_name,$name) phase: final` on the
  5 reference rule families; spec = `.3.22` oracle matrix. `.4.7` (scs inventory; behavior-neutral
  migration): named → `has_fact … phase: final`, plain-numeric → `fact_count_at_least(regex_capture_group,$N)
  phase: final`, relative `+N`/`-N` → prior-count resolution in the `.4.7` BUILD; deletes
  `find_invalid_scan_substring_capture_list`.
