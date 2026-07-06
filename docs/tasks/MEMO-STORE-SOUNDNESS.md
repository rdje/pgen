# MEMO-STORE-SOUNDNESS — close the memoization × semantic-store composition gap on the FAILURE side (F1)

- Tree ID: `MEMO-STORE-SOUNDNESS`
- Status: `active` (created 2026-07-06, session #47, spawned by `SEM-FINDINGS` — director directive
  2026-07-06; this tree owns finding **F1**)

## 1. The finding (tool-established, PARSE-HARNESS.6.2 session #47)

The split packrat memo's **failure cache is store-blind**: `memo_fail` is keyed `(rule_id, position)`
only (`memoized_call`, codegen template @`ast_based_generator.rs:6440`, PARSE-TERMINATION.6). A rule
whose body failure was CAUSED by the semantic-store state gets that failure cached; a retry at the
same position after the store changed replays the stale failure:

- MEASURED (`sem_memo_wrapper`, the differential pin): `program := wrap "?" | en wrap "!"`,
  `wrap := gated` (wrap UNANNOTATED), `gated` post-gated on `has_fact(g, "on")`, `en := "on"?`
  (zero-width emitter). On `"go!"`: branch 1 — `gated`'s gate rejects → `wrap`'s BODY fails →
  `memo_fail[(wrap, 0)]`; branch 2 — `en` emits the fact zero-width, `wrap` retries at position 0 →
  **stale failure replayed → REJECT**, where fresh evaluation would ACCEPT. Valid input rejected =
  a soundness defect.
- The design is otherwise CORRECT where it was aimed: the rule transaction WRAPS the memo, so an
  ANNOTATED rule's own gates/effects are never cached (`sem_memo_gate_retry` pins the sound side —
  a memo hit re-evaluates the rule's own gates fresh; the success side replays semantic deltas,
  `.b.6.2.36.4`). The gap is the COMPOSED failure cached one level up, in an unannotated ancestor.
- ⚠️ SUSPECTED SIBLING GAP (success side, adjudicate in `.1`): a memo-HIT replays the cached BODY,
  whose **nested tournament choices were made under the OLD store** — gates re-evaluate fresh, but
  body CONTENT is store-frozen. Same key-blindness, manifesting as a stale AST instead of a stale
  verdict. Needs its own isolating probe before deciding scope.

## 2. The elegant fix (design; confirm scope in `.1`, implement in `.2`)

**Taint-gated memo participation** (engine tier — the defect lives in the `memoized_call` template;
no lower tier can see it):

- Track "this body consulted the store": a monotonically-increasing **predicate-evaluation counter**
  on `SemanticRuntimeState` (bumped in `evaluate_predicate`/`evaluate_content_aware_predicate`);
  `memoized_call` snapshots it before the body and compares after. If the body (transitively)
  evaluated ≥1 predicate, the attempt is **store-tainted**:
  - tainted FAILURE → do NOT insert into `memo_fail` (re-parse on retry — honest, sound);
  - tainted SUCCESS → per `.1`'s success-side adjudication: either keep caching (delta replay +
    fresh gates already cover the pinned behavior) or exclude likewise.
- WHY this tier/shape: PARSE-TERMINATION.6's own numbers say failures are the ~81% majority and
  overwhelmingly pure-structural — excluding the store-tainted minority restores soundness at
  negligible perf cost. **REJECTED alternative:** a store-epoch in the memo key — SV emits facts
  constantly, so the epoch churns and the memo collapses on the biggest grammar (a perf regression
  where the memo matters most).
- **The interpreter's mirrored `memoized_call` gets the IDENTICAL fix in the same commit** (this is
  the one SEM-FINDINGS fix where parity is NOT automatic — the memo is a mirrored template, not a
  shared function); `parse_harness_semantic_gate` re-run proves the two stay byte-identical.
- Suite re-anchor (deliberate, documented): `sem_memo_wrapper`'s stale-REJECT pin (`"go!"` false)
  flips to the sound ACCEPT; `sem_memo_gate_retry` must stay CLEAN unchanged.

## 3. Verification battery (leaf `.2`)

Full engine-change battery: `parse_harness_semantic_gate` (re-anchored) + `parse_harness_equivalence_gate`
(11 CERTIFIED byte-identical — expected UNCHANGED: no shipped grammar exhibits the wrapper-staleness
shape on the gate corpus; any change is investigated, not waved through) + `parse_harness_combinator_gate`
+ SV cert union gate seeds 0/7/42 + external corpus 14/14 + `ast_shape_contract` + clippy +
`mdbook_docs_gate`. **Perf evidence required:** `PGEN_REPORT_MEMO_STATS=1` before/after on the SV
external corpus (hit-rate + wall-clock) — the fix must NOT materially regress SV parse performance
(the taint-exclusion thesis, measured). SV release bump only if SV-observable behavior changes
(expected NO — verify).

## 4. Leaves

- `.1` — **EVIDENCE: success-side staleness probe + taint-counter design check — `not-started`
  (frontier).** (a) Author the success-side isolating grammar (nested tournament choice depends on
  the store; same-position re-parse after a zero-width store change; does the oracle replay a stale
  AST?) and measure via the `.6.2` suite driver (`--ignored` probe). (b) Confirm the
  predicate-counter hook points (`evaluate_predicate` / `evaluate_content_aware_predicate` — both
  sides of the dispatch incl. composed `@predicate_def`s). (c) `PGEN_REPORT_MEMO_STATS` BASELINE on
  the SV external corpus (the before-number `.2` must match against). NO code (the probe is
  test-only scouting, allowed as measurement).
- `.2` — **FIX: taint-gated memo participation (codegen template + interpreter mirror, same commit)
  — `not-started`.** Blocked on `.1`. The §2 fix + regen + the §3 battery + perf before/after + the
  enforced acceptance checklist + book/spec lockstep (the memo × store contract becomes documented
  behavior).

## 5. Current Frontier

| # | Leaf | Status | Notes |
| --- | --- | --- | --- |
| 1 | `.1` (success-side probe + design check + perf baseline) | `not-started` (**frontier**) | Tools-first; test-only scouting. |
| 2 | `.2` (taint-gated memo fix, both implementations) | `not-started` | Blocked on `.1`. Engine tier; perf-gated. |

## 6. Relationships

- Spawned by [`SEM-FINDINGS`](SEM-FINDINGS.md) (F1). Evidence base: `PARSE-HARNESS.6.2` —
  `sem_memo_wrapper` / `sem_memo_gate_retry` pins; the `.b.6.2.36.4` success-side delta-replay work
  (this tree is its failure-side completion).
- The interpreter mirror (`parse_harness_interpreter.rs::memoized_call`) co-changes with the codegen
  template — the `.6.2` differential suite is the mechanical guard that they stay byte-identical.
