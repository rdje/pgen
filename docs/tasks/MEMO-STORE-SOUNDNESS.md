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
- ✅ SIBLING GAP **CONFIRMED** (success side, `.1` 2026-07-06 session #49 — probe
  `measure_memo_success_side_staleness`, `parse_harness_semantic_suite.rs::measurement`): a memo-HIT
  replays the cached BODY, whose **nested tournament choices were made under the OLD store** — gates
  re-evaluate fresh, but body CONTENT is store-frozen. Measured on BOTH observables, BOTH
  implementations (interpreter = oracle, i.e. the mirror is faithful to the defect):
  - **verdict-observable** (`probe_memo_success_verdict`: gated `wide:="gox"` vs `narrow:="go"` under
    unannotated `pick`): `"gox!"` → `accepted=false` on oracle AND interpreter — the stale narrow win
    (end 2) misaligns `"!"`, where fresh evaluation picks `wide` (gate passes post-emission) and
    ACCEPTs. Controls: `"go?"` accepts store-free; `"ongo!"` accepts at a fresh position; `"gox?"`
    both-rejects.
  - **AST-observable** (`probe_memo_success_ast`: equal-length `special/normal` with shaped `kind`
    markers): `"go!"` → ACCEPT with stale `normal_pick` on both sides, where fresh evaluation shapes
    `special_pick`; control `"ongo!"` (fresh position) correctly shapes `special_pick`, proving the
    gate steers the tournament when no stale entry interferes.
  - **`.1` ADJUDICATION: tainted SUCCESSES must be excluded from the memo too** — the success-side
    delta-replay + fresh-gates design provably does NOT cover nested tournament content (it can flip
    the verdict via a stale end-position AND the tree at equal length).

## 2. The elegant fix (design; confirm scope in `.1`, implement in `.2`)

**Taint-gated memo participation** (engine tier — the defect lives in the `memoized_call` template;
no lower tier can see it):

- Track "this body consulted the store": a monotonically-increasing **predicate-evaluation counter**
  on `SemanticRuntimeState` (bumped in `evaluate_predicate`/`evaluate_content_aware_predicate`);
  `memoized_call` snapshots it before the body and compares after. If the body (transitively)
  evaluated ≥1 predicate, the attempt is **store-tainted**:
  - tainted FAILURE → do NOT insert into `memo_fail` (re-parse on retry — honest, sound);
  - tainted SUCCESS → **exclude likewise** (`.1` adjudication, tool-established: delta replay +
    fresh gates do NOT cover nested tournament content — stale verdict AND stale AST measured).
- **`.1` hook-point + mechanics audit (tool-established, session #49):**
  - ALL predicate evaluation funnels through `evaluate_predicate` (`semantic_runtime.rs:2032`):
    PRE via `evaluate_directive_predicate` (`:2379` → `:2387`; codegen `:1768`/interp `:618`);
    POST + BRANCH via `evaluate_content_aware_predicate` (`:2356`, delegating at `:2377`; codegen
    `:1890`+`:3328`/interp `:661`+`:1858`); composed `@predicate_def`s via the default arm
    (`:2227` → `evaluate_composed_predicate` → `eval_primitive_call_as_bool` `:2337` → back into
    `evaluate_predicate` `:2353` — within the outer call's dynamic extent either way).
  - `content_kind_is` is the ONE content-only arm that short-circuits in
    `evaluate_content_aware_predicate` without reaching `evaluate_predicate` — it reads parse
    content, not the store, and content is memo-key-covered ⇒ **exempting it from taint is sound**
    (decide in `.2`: single hook in `evaluate_predicate` + documented exemption, vs conservative
    dual-hook).
  - Both evaluators take **`&self`** ⇒ the counter must be interior-mutable (`Cell<u64>` beside
    `counters: SemanticStoreCounters`, `semantic_runtime.rs:1232` — `Cell<u64>` keeps the state's
    `Clone`/`PartialEq`/`Debug` derives).
  - **Monotonicity across speculation holds by construction**: `rollback_to_named` (`:1721`) is
    TRUNCATION-based (facts/scope-arena lengths) and never touches `counters`; the rule
    transaction's `mem::take` window MOVES the state (no clone — PARSE-TERMINATION.3.1 removed the
    clone-restore; pinned by the `:9072` no-`original_semantic_runtime_state.clone()` codegen test)
    and only starts AFTER the body returns, so `memoized_call`'s before/after counter reads always
    see the live state.
  - A rule's OWN gates evaluate OUTSIDE its `memoized_call` (pre before `f`, post/branch in the
    take-window after `f`) ⇒ they never taint the rule's own memo entry — exactly right, since the
    transaction re-evaluates them fresh on every hit. Nested rules' whole transactions run inside
    the outer body ⇒ they taint the outer entry — exactly the `sem_memo_wrapper` shape.
  - `stimuli_generator.rs:11536` also calls `evaluate_predicate` (generation side) — bumps are
    harmless there (no memo participates).
- WHY this tier/shape: PARSE-TERMINATION.6's own numbers say failures are the ~81% majority and
  overwhelmingly pure-structural — excluding the store-tainted minority restores soundness at
  negligible perf cost. **REJECTED alternative:** a store-epoch in the memo key — SV emits facts
  constantly, so the epoch churns and the memo collapses on the biggest grammar (a perf regression
  where the memo matters most).
- **The interpreter's mirrored `memoized_call` gets the IDENTICAL fix in the same commit** (this is
  the one SEM-FINDINGS fix where parity is NOT automatic — the memo is a mirrored template, not a
  shared function); `parse_harness_semantic_gate` re-run proves the two stay byte-identical.
- Suite re-anchor (deliberate, documented): `sem_memo_wrapper`'s stale-REJECT pin (`"go!"` false)
  flips to the sound ACCEPT; `sem_memo_gate_retry` must stay CLEAN unchanged. Additionally (per
  `.1`): promote the two success-side probe grammars (`probe_memo_success_verdict` /
  `probe_memo_success_ast`) into `SEMANTIC_CASES` with SOUND anchors (`"gox!"` ACCEPT;
  `"go!"` → `special_pick`) — the success-side staleness gets a permanent differential pin, not
  just a scouting probe.

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

- `.1` — **EVIDENCE: success-side staleness probe + taint-counter design check — `done`
  (2026-07-06, session #49, `PGEN-MEMO-STORE-SOUNDNESS-0001`).** (a) Author the success-side isolating grammar (nested tournament choice depends on
  the store; same-position re-parse after a zero-width store change; does the oracle replay a stale
  AST?) and measure via the `.6.2` suite driver (`--ignored` probe). (b) Confirm the
  predicate-counter hook points (`evaluate_predicate` / `evaluate_content_aware_predicate` — both
  sides of the dispatch incl. composed `@predicate_def`s). (c) `PGEN_REPORT_MEMO_STATS` BASELINE on
  the SV external corpus (the before-number `.2` must match against). NO code (the probe is
  test-only scouting, allowed as measurement).

  **Outcome (session #49, 2026-07-06):** (a) DONE — success-side staleness CONFIRMED on both
  observables × both implementations (§1); adjudication: tainted successes excluded too. (b) DONE —
  full hook-point + monotonicity audit recorded in §2. (c) DONE — baseline below.

  ### (c) `PGEN_REPORT_MEMO_STATS=1` SV external-corpus BASELINE (the before-numbers `.2` must match)
  Protocol (re-runnable oracle): `PGEN_REPORT_MEMO_STATS=1 make -C rust SHELL=/bin/bash
  sv_external_corpus_triage_gate` — 14/14 parse-pass; aggregates `parse_total_ms=398455`,
  `parse_max_ms=190958` (same-day plain run: `388418`/`186825` ⇒ stats overhead ≈2.6%, so `.2`
  before/after comparisons should use stats-on runs on both sides). Main-parse per case
  (`parse_full_ms`; memo `ok+fail` end-state entry counts from the `=== MEMO STATS` block —
  NOTE: the stats surface reports END-STATE entry counts, not hit rates; wall-clock is the perf
  signal, entry counts measure memo participation):

  | case (profile) | parse_full_ms | memo ok | memo fail | fail share |
  |---|---|---|---|---|
  | uvm_pkg (2017) | 190958 | 4,376,930 | 21,836,980 | 83.3% |
  | uvm_pkg (2023) | 190723 | 4,389,865 | 21,905,854 | 83.3% |
  | uvm_compat_pkg (2017) | 3307 | 78,032 | 400,384 | 83.7% |
  | uvm_compat_pkg (2023) | 3293 | 78,094 | 401,169 | 83.7% |
  | scr1_core_top (2017) | 1484 | 47,376 | 274,594 | 85.3% |
  | scr1_core_top (2023) | 1502 | 47,417 | 276,372 | 85.4% |
  | scr1_top_ahb (2017) | 735 | 24,509 | 124,009 | 83.5% |
  | scr1_top_ahb (2023) | 742 | 24,517 | 124,398 | 83.5% |
  | friscv_rv32i_core (2017) | 1770 | 54,633 | 281,050 | 83.7% |
  | friscv_rv32i_core (2023) | 1761 | 54,633 | 281,563 | 83.8% |
  | friscv_pipeline (2017) | 235 | 4,868 | 26,090 | 84.3% |
  | friscv_pipeline (2023) | 235 | 4,870 | 26,191 | 84.3% |
  | veer_el2_lsu (2017) | 847 | 25,405 | 138,263 | 84.5% |
  | veer_el2_lsu (2023) | 863 | 25,405 | 138,373 | 84.5% |

  (uvm_compat's bootstrap parse re-parses uvm_pkg — its stats block matches the uvm_pkg row.)
  Failures are 83–85% of memo entries on EVERY case — the PARSE-TERMINATION.6 "failures are the
  ~81% majority" thesis re-measured at corpus scale, which is the perf premise of the `.2`
  taint-exclusion design (excluding the store-tainted minority must not materially move these
  wall-clocks).

  ### Acceptance Checklist (enforced)
  - [x] **REPRODUCE / ISSUE** — probe `measure_memo_success_side_staleness` (`cargo test --features
    "generated_parsers ebnf_dual_run" --lib
    parse_harness_semantic_suite::measurement::measure_memo_success_side_staleness -- --ignored
    --nocapture`): `verdict-observable staleness: oracle=CONFIRMED (stale REJECT of "gox!")
    interp=CONFIRMED` + `AST-observable staleness: oracle=CONFIRMED (stale normal_pick on "go!")
    interp=CONFIRMED` — the §1 suspected success-side gap is real.
  - [x] **ROOT CAUSE (WHY + WHERE)** — the memo success map is keyed `(rule_id, position)` only and
    a hit replays the cached node + delta (`ast_based_generator.rs:6462-6499` codegen template;
    interpreter mirror `parse_harness_interpreter.rs:716-726`): body content frozen under the
    first-evaluation store. Verbatim probe output (the WHY+WHERE lines):
    `"gox!" interp[accepted=false furthest_position=0 kind=-] oracle[accepted=false
    furthest_position=0 kind=-]` (the stale narrow replay — `furthest_position=0` because the memo
    backtrack reports the stale key position, another staleness tell) and `"go!"
    interp[accepted=true furthest_position=0 kind=normal_pick] oracle[accepted=true
    furthest_position=0 kind=normal_pick]` (stale tree). Probe controls isolate the mechanism:
    fresh-position `"ongo!"` (`accepted=true furthest_position=2 kind=special_pick`) and store-free
    `"go?"` behave correctly; ONLY the same-position post-store-change retry replays stale content.
  - [x] **FIX** — evidence-only leaf: the minimal change is the `--ignored` scouting probe itself
    (tooling/measurement tier — no parser behavior touched). The engine fix is `.2` (design §2,
    now scope-complete: tainted failures AND successes excluded).
  - [x] **ADDRESSED (verified)** — the §1 SUSPECTED sibling gap is now MEASURED: before = suspected
    / unpinned; after = CONFIRMED on 2 observables × 2 implementations with fresh-position +
    store-free controls green (probe output above, deterministic curated inputs).
  - [x] **NO REGRESSION** — probe is `#[ignore]` test-only; `make -C rust SHELL=/bin/bash
    parse_harness_semantic_gate` re-run GREEN (20/20 CLEAN, byte-identical) after the edit; the SV
    external-corpus triage gate re-run 14/14 parse-pass (the (c) baseline run below IS the oracle
    re-run).
  - [x] **LOCKSTEP** — parser behavior unchanged (measurement only) ⇒ book/contract/ledger N/A this
    leaf; the memo × store contract documentation lands with the `.2` fix. Tree §1/§2 updated
    same-commit (this file).
- `.2` — **FIX: taint-gated memo participation (codegen template + interpreter mirror, same commit)
  — `not-started`.** Blocked on `.1`. The §2 fix + regen + the §3 battery + perf before/after + the
  enforced acceptance checklist + book/spec lockstep (the memo × store contract becomes documented
  behavior).

## 5. Current Frontier

| # | Leaf | Status | Notes |
| --- | --- | --- | --- |
| 1 | `.1` (success-side probe + design check + perf baseline) | `done` | Success-side staleness CONFIRMED (both observables × both implementations); hook points + monotonicity audited; corpus baseline locked. |
| 2 | `.2` (taint-gated memo fix, both implementations) | `not-started` (**frontier**) | Unblocked. Engine tier; perf-gated; tainted FAILURES **and** SUCCESSES excluded (`.1` adjudication). |

## 6. Relationships

- Spawned by [`SEM-FINDINGS`](SEM-FINDINGS.md) (F1). Evidence base: `PARSE-HARNESS.6.2` —
  `sem_memo_wrapper` / `sem_memo_gate_retry` pins; the `.b.6.2.36.4` success-side delta-replay work
  (this tree is its failure-side completion).
- The interpreter mirror (`parse_harness_interpreter.rs::memoized_call`) co-changes with the codegen
  template — the `.6.2` differential suite is the mechanical guard that they stay byte-identical.
