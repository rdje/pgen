# SEM-FINDINGS — adjudicate + elegantly address the six PARSE-HARNESS.6.2 tool-established findings

- Tree ID: `SEM-FINDINGS`
- Status: `active` (created 2026-07-06, session #47, `PGEN-SEM-FINDINGS-0001` — directive capture + adjudication plan; created moments before a directed `/clear`, so this file IS the handoff)
- Director directive (2026-07-06, verbatim intent): *"You need to address the following tool-established
  findings of yours … You need to find way to elegantly address them if they need to be. Create
  task-tree for each if need be."* — i.e. (1) adjudicate each of the six `.6.2` findings (defect to fix
  vs correct-behavior to document), (2) fix the ones that need fixing, elegantly (fix-hierarchy
  discipline), (3) give each code-fix finding its own task-tree if warranted.
- Provenance: the six findings were tool-established during `PARSE-HARNESS.6.2`
  (`PGEN-PARSE-HARNESS-0015`, commit `3a323169`) and are each **differentially pinned** by the
  semantic suite (`rust/src/parse_harness_semantic_suite.rs`, gate `parse_harness_semantic_gate`) —
  full evidence in `docs/tasks/PARSE-HARNESS.md` §21 + `CHANGES.md` (session #47) + the book chapter
  *The Parse Harness → The semantic-directive orchestration suite → Grammar-author facts*.

---

## 1. The six findings + session-#47 preliminary adjudication (to be VERIFIED tools-first in `.1`)

> ⚠️ The "leaning" column is preliminary reasoning from the `.6.2` evidence. Leaf `.1` must confirm each
> with the named tool check BEFORE any fix tree is spawned (no code change without tool-backed facts).

| # | Finding (pinned by) | Preliminary adjudication | Elegant-fix candidate (fix-hierarchy tier) | Pending tool check (leaf `.1`) |
|---|---|---|---|---|
| F1 | **Split-memo FAILURE cache is store-blind** — `memo_fail` keyed `(rule, position)` only; an unannotated wrapper over a store-gated rule replays a STALE failure after a zero-width store change → valid input REJECTED (`sem_memo_wrapper`) | **NEEDS FIX — soundness** (valid input rejected is a correctness defect for a signoff-grade platform). NOTE the SUCCESS side may have the sibling gap: a memo-hit replays the cached BODY (whose nested tournament choices were made under the OLD store) — gates re-evaluate fresh, but body CONTENT is store-frozen. | **Taint-gated memo participation** (engine, `memoized_call` template): track "this body consulted the store" (predicate-evaluation counter snapshot around the body); if tainted, do NOT insert into `memo_fail` (and adjudicate whether tainted SUCCESSES may cache). Surgical: the 81% pure-structural failure majority stays cached → negligible perf cost. REJECTED alternative: a store-epoch in the key — collapses hit rates on store-heavy SV (perf regression on the biggest grammar). | Build the success-side staleness probe (isolating grammar where a nested tournament choice depends on the store + same-position re-parse); measure `PGEN_REPORT_MEMO_STATS` before/after the fix on SV. |
| F2 | **Quoted predicate args never match `$ref`-emitted fact names** — `String("x")` ≠ `Identifier("x")`; a quoted arg compiles fine and SILENTLY never matches (`sem_branch_gate` trace: `has_fact(kind=mode, name=String("special")) → false` with the fact present) | **NEEDS ADDRESSING — silent footgun** (a gate that can never pass, with zero diagnostics). Two-step: check whether ATTRIBUTE matching (`semantic_values_match`) is lenient while NAME matching is strict — an engine inconsistency would argue for unifying name matching to textual equality; otherwise a linter warning is the safe tier. | Either (a) **textual name-equality in the fact index** (engine, small, verified by the whole gate battery + the suite re-anchor) if the name/attribute asymmetry is confirmed, or (b) a **validator V-rule** (warn on quoted-String name args to `has_fact`/`lacks_fact`/`fact_attribute_equals`/`lacks_fact_attribute_equals`). | Read `semantic_values_match` (`semantic_runtime.rs`) + the `fact_index` name-matching; decide (a) vs (b) from the asymmetry evidence. |
| F3 | **Inline `phase: branch` predicate flattens RULE-WIDE** — `branch_predicates_for_rule` flat-maps every branch bucket (`semantic_runtime.rs:735-747`), so an inline branch predicate gates EVERY branch (+ double-evaluates on its own) (`sem_branch_gate`) | **NEEDS FIX — likely a registry bug with a LIVE SV consequence.** The intuitive semantics of an INLINE annotation on branch K is branch-LOCAL (that is what `branch_predicates_for_rule_branch` exists for). ⚠️ SUSPECTED LIVE SV DEFECT: `grammars/systemverilog.ebnf:4605` (`scoped_or_hierarchical_tf_identifier`, `.b.6.2.2`) puts an inline branch predicate with `$scope.name.body` on the `package_scope` branch — under flattening it ALSO evaluates on the `hierarchical` branch (branch 3), whose shaped content `{kind:"hierarchical", body:…}` has NO `scope` key → `try_resolve` fails → **branch 3 silently BLOCKED** → hierarchical tf-names may be unparseable through this rule. The SV:3452 comment documents the flattening and works AROUND it (helper-rule post gates) — 4605 appears to predate/contradict that knowledge. | **Fix the registry function** (engine, ONE function): `branch_predicates_for_rule` returns only RULE-level branch-phase predicates (from `directives_by_rule`), never the flat-mapped branch buckets — inline branch predicates become branch-local via `_for_rule_branch` (already chained at every call site). The interpreter calls the SAME registry function → parity is automatic, zero interpreter change. Update `sem_branch_gate`'s anchors (the flattening pin flips to the branch-local pin); verify SV:4605 now behaves as `.b.6.2.2` intended. | FIRST: prove/disprove the SV:4605 block — parse a hierarchical tf-call reproducer (`--parse-dump-ast-pretty` + `--trace-rules scoped_or_hierarchical_tf_identifier`, watch for `🛡️ … REJECTED branch 3/3 … unresolved args`); grep all grammars for other inline `phase: branch` users; check git log for whether flattening was ever a deliberate decision. |
| F4 | **Positional `$N` refs can NEVER resolve in directive payloads** — the annotation compiler strips `$` (frozen literal `RuleReference("2")`), the named-path lexer rejects a digit head; `resolve_positional_semantic_reference` (built deliberately in SV-EXH-PROOF.3.3.4.a.2) is DEAD from compiled directives (`sem_ref_positional_unresolvable`) | **NEEDS FIX — complete the half-wired surface** (the resolver machinery exists + is documented; the compiler makes it unreachable; a `$2` in a payload today fails HARD with no hint). Zero blast radius: no existing grammar can be using positional payload refs (they would be broken). | **Preserve the `$` sigil for digit-headed references in the payload compiler** (`semantic_runtime.rs` parse_* payload functions — ONE place): stored `RuleReference("$2.word")` flows through the EXISTING emitted resolvers' `starts_with('$')` positional dispatch unchanged — no codegen-template change, and the interpreter compiles in-process with the same function → parity automatic. Re-anchor `sem_ref_positional_unresolvable` → a working `sem_ref_positional` case. (Alternative considered: digit-head dispatch in the resolver template — rejected, it would require regenerating every parser for a template change.) | Confirm WHERE the sigil is stripped (the payload value parser in `semantic_runtime.rs`); confirm shipped parsers' frozen literals never carry positional refs (grep `generated/` + all `grammars/*.ebnf` for `$[0-9]` in directive payloads). |
| F5 | **A discarded zero-length iteration's `@emit_fact` persists** — a rule succeeding at zero length fires its effects; the quantifier guard then discards the iteration STRUCTURALLY only (`sem_zero_len_emit`) | **NO CODE FIX — defensible, relied-upon-able semantics** (a zero-width marker emission is a legitimate idiom — cf. the `en := "on"?` enabling pattern; the guard is an anti-infinite-loop mechanism, not a transaction boundary). Action = DOCUMENT as normative. | Add the effects-timing rule ("effects commit on rule success, including zero-length success; structural discard does not roll them back") to `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md` (+ the semantic_annotation parser book if it discusses effect timing). Doc-only slice. | Check the normative spec's current effects-timing wording (silent vs contradicting). |
| F6 | **Unresolved-reference stubs have no rule preamble** — a referenced-but-undefined rule compiles to a bare `Err(Backtrack)` stub (no furthest bump / memo / context); the interpreter now mirrors this (parity DONE in `.6.2`) | **Parity fixed; the RESIDUAL issue is diagnosability**: `sem_count_gate`'s missing `word` rule sailed through `--generate-parser` with NO warning and produced a parser that can never accept — a linter/codegen diagnosability gap (an undefined reference makes every referencing rule dead ⇒ squarely the GRAMMAR-WELLFORMED "unreachable = DEFECT" contract). | **A linter (or codegen-time) diagnostic**: `--lint-grammar` (and/or `--generate-parser`) reports "rule `X` references undefined rule `Y` (a never-matching stub will be generated)" as an ERROR/WARNING, excluding the intentional builtins (`builtin_any_char`, `builtin_ascii_char`, `true`, `false`, `semantic_annotation`). Belongs to the linter-completeness charter — implement as a leaf HERE (`.7`) unless `.1` finds `GRAMMAR-WELLFORMED` is the better owner (then hand it over with a cross-link). | Run `--lint-grammar` on the missing-`word` grammar — is it silent? Enumerate the intentional-builtin allowlist from `generate_unresolved_reference_method` (`ast_based_generator.rs:833-946`). |

## 2. Working rules for this tree

- **Tools-first, no guessing**: every `.1` check runs BEFORE the corresponding fix leaf is opened; a fix
  leaf carries the enforced acceptance checklist (ROOT CAUSE / ADDRESSED / NO REGRESSION).
- **Fix hierarchy** per [[feedback_no_workarounds_fix_hierarchy]]; engine changes (F1/F3/F4 candidates)
  are legitimate here because the defects ARE engine-level — each must state why no lower tier suffices.
- **Every engine fix re-runs the full battery**: `parse_harness_semantic_gate` (suite re-anchored where a
  pin legitimately flips — a pin flip is a DELIBERATE, documented semantics change, never silent),
  `parse_harness_equivalence_gate` (11 CERTIFIED byte-identical or explicitly re-baselined),
  `parse_harness_combinator_gate`, SV cert union gate seeds 0/7/42, external corpus, `ast_shape_contract`,
  clippy, `mdbook_docs_gate`; SV release/ledger bump if SV-observable (F3 likely IS — the 4605 fix
  changes accept behavior for hierarchical tf paths → ledger + release bump + conformance re-lock).
- **Interpreter parity by construction where possible**: F3 (shared registry fn) and F4 (shared compiler
  fn) need ZERO interpreter changes — the `.6.2` mirror consumes the same functions. F1 (a codegen
  template change to `memoized_call`) DOES need the interpreter's mirrored `memoized_call` updated in the
  SAME commit + the suite re-run.
- **One clean slice per leaf**; book/spec lockstep same-commit (director D5 discipline).

## 3. Leaves

- `.1` — **ADJUDICATION EVIDENCE PASS — `not-started` (the next session's first job).** Run the five
  pending tool checks in the §1 table (F3's SV:4605 reproducer FIRST — a suspected live SV defect;
  then F2's `semantic_values_match` asymmetry read; F4's sigil-strip locus + repo-wide positional-ref
  grep; F6's `--lint-grammar` silence check; F5's normative-spec wording check). Output: each
  preliminary adjudication CONFIRMED or corrected, recorded here; spawn/scope `.2`–`.7` accordingly.
  NO code.
- `.2` — **F3: branch-predicate locality (+ the SV:4605 consequence) — `not-started`.** The registry-fn
  fix + `sem_branch_gate` re-anchor + SV verification (reproducer REJECT→PASS if the defect is
  confirmed) + SV ledger/release handling. Highest priority of the fixes (live-parser impact).
- `.3` — **F1: memo × store soundness — `not-started`.** The taint-gated memo participation fix
  (codegen template + the interpreter's mirrored `memoized_call`, same commit) + the success-side
  staleness probe/adjudication + `sem_memo_wrapper` re-anchor (stale-REJECT pin flips to sound-ACCEPT)
  + `PGEN_REPORT_MEMO_STATS` perf before/after on SV.
- `.4` — **F4: positional-`$N` payload refs — `not-started`.** The compiler sigil-preservation fix +
  the `sem_ref_positional_unresolvable` → working-case re-anchor + normative-spec/book note.
- `.5` — **F2: fact-name matching — `not-started`.** Engine unification OR validator V-rule (decided by
  `.1`); suite case for whichever lands.
- `.6` — **F5: zero-length effects-timing — `not-started`.** Normative-spec (+ book) documentation
  slice. Doc-only.
- `.7` — **F6: undefined-reference diagnosability — `not-started`.** The linter/codegen diagnostic
  (with the builtin allowlist), OR hand-off to `GRAMMAR-WELLFORMED` per `.1`'s ownership call.

## 4. Current Frontier

| # | Leaf | Status | Notes |
| --- | --- | --- | --- |
| 1 | `SEM-FINDINGS.1` (adjudication evidence pass) | `not-started` (**frontier**) | Tools-first confirmation of all six adjudications; F3's SV:4605 reproducer first. NO code. |
| 2 | `SEM-FINDINGS.2` (F3 branch-predicate locality) | `not-started` | Blocked on `.1`. Suspected live SV defect — highest fix priority. |
| 3 | `SEM-FINDINGS.3` (F1 memo × store soundness) | `not-started` | Blocked on `.1`. |
| 4 | `SEM-FINDINGS.4` (F4 positional payload refs) | `not-started` | Blocked on `.1`. |
| 5 | `SEM-FINDINGS.5` (F2 fact-name matching) | `not-started` | Blocked on `.1` (engine-vs-linter tier decision). |
| 6 | `SEM-FINDINGS.6` (F5 effects-timing doc) | `not-started` | Doc-only. |
| 7 | `SEM-FINDINGS.7` (F6 undefined-ref diagnosability) | `not-started` | Blocked on `.1` (ownership call vs `GRAMMAR-WELLFORMED`). |

## 5. Acceptance criteria (tree-level)

1. Every finding carries a tool-CONFIRMED adjudication (not the preliminary leaning) recorded in §1.
2. Every needs-fix finding is fixed at the stated tier with the enforced acceptance checklist, the full
   no-regression battery, and the `.6.2` suite re-anchored deliberately (a pin flip = a documented
   semantics change with the old and new behavior both stated).
3. Every no-fix finding is documented normatively (spec/book), so the behavior is a contract, not folklore.
4. The `PARSE-HARNESS.6.2` suite remains the mechanical guard: after this tree, `parse_harness_semantic_gate`
   pins the POST-fix semantics 20/20 (or N/N with added cases).
5. SV impact (F3) handled per release policy: ledger entry, release bump, conformance re-lock if behavior
   changed.

## 6. Relationships

- Consumes: `PARSE-HARNESS.6.2` (`PGEN-PARSE-HARNESS-0015`) — the findings, the pinning suite, and the
  interpreter mirror that makes engine changes verifiable differentially.
- Touches: `GRAMMAR-WELLFORMED` (F6 linter-completeness), the SV family (F3 — ledger/release),
  `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md` (F2/F4/F5 normative wording).
- The `.6.1` findings (bounded quantifiers half-wired; direct-LR furthest divergence; LR-elim
  `rule_order` shift) are NOT in this tree's scope (director listed the six `.6.2` findings); they stay
  on the PARSE-HARNESS backlog surface.
