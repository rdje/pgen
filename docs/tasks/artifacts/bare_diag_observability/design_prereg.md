# `PGEN-RGX-0078-0201` — bare diagnostic observability: the public-boundary design + the one licensed fix (pre-registration)

Written BEFORE any code change (the `-0197` execution contract: the diagnostics
member may not be implemented without an observer/public-boundary design). This
record is the design; §6 pre-registers the A/B and its falsification bounds.

## 1. The owned member and its held pricing

`PGEN-RGX-0078-0185` priced the complete bare-path diagnostic mechanism at the
C1 vintage (preserved probe `1d3fa0ee`, floor 1,263.4 ns):

| population (C1 vintage) | static | dynamic (sub/mid/high) |
|---|---|---|
| cached logger gates | 170 gates = 419 PCs | 104 / 95 / 120 |
| coverage snapshot/truncate | 39 checkpoints = 219 PCs | 30 / 29 / 38 |
| `rollbacks_nonempty_chain` sites | 63 sites = 315 PCs | 0 / 0 / 1 |
| **whole mechanism** | **953 PCs** | **149 / 143 / 193 = 8.042632 ns** |

`-0197` held it: "diagnostics … need observer/public-boundary designs". The
`-0200` NEXT pointer scoped this session: design first; the trace-off fast path
may NOT silently remove public diagnostics.

## 2. Current-vintage re-enumeration (the C1 pricing is partially stale)

Verified against HEAD `074d8bb9` (post `-0198`/`-0199`/`-0200`, post
PARSER-NEUTRALITY; canonical DEFAULT regex emit `1a5f7018`):

- **The logger-gate population is largely ALREADY HARVESTED.** At C1 the bare
  path's speculation sites called the generic `try_parse`, whose body carries
  3 `trace_enabled()` gates (`generated/regex_parser.rs:413354/413364/413392`);
  fat-LTO inlined those gates at every speculation site — the bulk of the 170.
  `-0200` introduced `try_parse_bare` (`generated/regex_parser.rs:413407-413436`)
  which carries ZERO trace gates, and every fused speculation site calls it
  (193 call sites). The generated cascade region carries ZERO
  `trace_enabled`/`logger_enabled` occurrences (last occurrence line 413770;
  first `fn cascade_*` line 413834). Residual bare-reachable gates live only in
  (a) the once-per-parse entry protocol frame and (b) the protocol frames of
  cascade-INELIGIBLE (effect-carrying) residual rules — both are SHARED code
  that serves traced parses and is NOT removable. **No implementable
  logger-gate subset remains; recorded as absorbed by `-0200` (unpriced there,
  honest here).**
- **The coverage snapshot/truncate SURVIVES.** `try_parse_bare` still takes
  `saved_coverage_len = self.coverage_stack.len()` and
  `self.coverage_stack.truncate(saved_coverage_len)` on failure
  (`generated/regex_parser.rs:413413/413419`; emitted at
  `rust/src/ast_pipeline/ast_based_generator.rs:6680/6687`). 193 static call
  sites on the current bare path.
- **The `rollbacks_nonempty_chain` maintenance SURVIVES.** The hot rollback
  fast path executes `if checkpoint.chain_len > 1 { …nonempty_chain += 1 }`
  (`rust/src/ast_pipeline/semantic_runtime.rs:3299-3301`; slow path
  `:3358-3359`) on every bare failed speculation and C3-B island cleanup
  (`cascade.rs:994/:1032` emissions).

## 3. WHY the surviving work is dead on the bare path (the licenses)

- **Routing invariant (emitted at `ast_based_generator.rs:1797-1800`; generated
  `regex_parser.rs:1214-1216`):** `bare_parse = !coverage_enabled &&
  !logger_enabled && !counters_observed && !report_memo_stats_enabled()`, and
  `parse_from` forces `bare_parse = false`. The fused graph (and therefore
  `try_parse_bare` and the island cleanups) executes ONLY under `bare_parse`.
- **Coverage:** every `coverage_stack.push` in the generated parser is gated on
  `coverage_enabled` (e.g. `regex_parser.rs:2533-2535`); the cascade emitter
  emits NO coverage pushes (zero `coverage_stack` occurrences in `cascade.rs`).
  Bare ⇒ `!coverage_enabled` ⇒ the stack length is INVARIANT across any bare
  region ⇒ the snapshot/truncate is a provable no-op. Removing it changes no
  reachable behavior (coverage consumers — outcome dumps, cert coverage — opt
  in pre-parse and route to the protocol graph; TOOLBOX 3.5 routing note).
- **Chain counter:** `rollbacks_nonempty_chain` steers nothing (source doc
  `semantic_runtime.rs:2296-2321`; `-0185` source_semantics.md). Its only
  repo-wide readers serialize outcome dumps (`parser_registry.rs`), which are
  coverage consumers ⇒ protocol graph ⇒ exact values preserved there.

## 4. The public boundary (`-0187`) and the compatibility design

`-0187` (verbatim constraint): generated parsers publicly expose
`semantic_runtime_state()`, and `SemanticStoreCounters` values are readable
post-parse without a pre-parse observer latch — "bare-path elision needs an
explicit opt-in or compatibility design; diagnostic-only does not license
silent removal."

**Adjudicated design — the OBSERVED-PARSE boundary (the shipped D2-A
precedent, no new latch):**

- The per-rule entry counters already have exactly this shape: a bare parse
  ticks no per-rule counters, and observation is a pre-parse opt-in
  (`rule_call_counts()` marks `counters_observed`; TOOLBOX 3.4). We extend the
  same documented boundary to the store's diagnostic CLASSIFICATION counter:
  **`rollbacks_nonempty_chain` is exact on OBSERVED parses** (any parse with a
  diagnostic consumer active pre-parse: transactional coverage / outcome dumps,
  trace, rule-counter observation, memo-stats — each of which already forces
  the protocol graph). A bare parse may skip its maintenance. **Required
  semantic counters are exempt and always exact** — `predicate_evaluations` is
  memo-taint soundness state (`-0185` foundational correction) and is untouched.
- The explicit opt-in is the EXISTING consumer surface (no new API): an
  embedder that wants exact diagnostic counters enables any diagnostic
  consumer before parsing. The boundary is DOCUMENTED (not silent) in: the
  `counters()` / field doc comments (the API surface itself), TOOLBOX 3.5, the
  book's observability-twin note, and CHANGES.
- A state-side latch (`Cell<bool>` set by `counters()`) was considered and
  REJECTED: `SemanticRuntimeState` derives `Clone + PartialEq`
  (`semantic_runtime.rs:2361`) so the latch would leak into equality/clone
  semantics and give a read-only accessor a side effect; and it still cannot
  repair the parse-first-then-read-first-time case, so it buys no additional
  soundness over the documented boundary.
- **Exposure audit (current):** `rollbacks_nonempty_chain` appears in NO
  versioned integration contract, NOT in `EMBEDDING_API_CONTRACT.md`, NOT in
  any book chapter (repo-wide grep); its only in-repo readers are the
  outcome-dump serializers (protocol-routed). The rollback telemetry QUARTET
  (`rollbacks`, `rollbacks_unchanged`, `rollbacks_tournament`,
  `rollbacks_tournament_unchanged`) stays FULLY MAINTAINED on the bare path —
  the `-0187` member (0.227347146 ns) is NOT implemented here (one-fix
  discipline); this design unblocks it for a future owned leaf.

## 5. The ONE fix (scope-exact change list)

**LIB (`rust/src/ast_pipeline/semantic_runtime.rs`), additive only:**
1. `rollback_to_labeled_bare(checkpoint, label)` — the fast path minus the
   `chain_len > 1` diagnostic test/increment; the outlined slow path gains a
   private `maintain_chain_diag: bool` parameter (existing entry points pass
   `true` — byte-identical behavior; the bare entry passes `false`).
   All other counters (incl. the telemetry quartet) update identically.
2. Doc-comment amendments: `counters()` + the `rollbacks_nonempty_chain` field
   document the observed-parse boundary.
3. Focused tests: bare variant skips the chain counter on both paths while the
   quartet and required counters are identical; the documented-boundary test.

**EMITTER (`rust/src/ast_pipeline/ast_based_generator.rs` + `cascade.rs`):**
4. `try_parse_bare` emission (`ast_based_generator.rs:6672ff`): drop
   `saved_coverage_len` + the truncate; call `rollback_to_labeled_bare`.
5. Island C3-B cleanup emissions (`cascade.rs:994/:1032`): call
   `rollback_to_labeled_bare` (bare-graph-only sites).
6. Emitter self-check tests updated to pin the NEW shape
   (`ast_based_generator.rs:11108/11129`, `cascade.rs:2918`).
7. NO routing-predicate change (no latch — §4).

**Out of scope (recorded, not owned here):** protocol `try_parse` and every
protocol frame (shared with traced/covered parses); the telemetry quartet
(`-0187`); the entry-frame once-per-parse trace-scope check; `-0202` cascade
error design; `-0203` carrier core.

## 6. Pre-registered adjudication (the binding `-0197` ratchet)

- **Base:** `preserved_probes/regex_perf_probe_neutrality_948cbd63` (the
  designated immediate-parent baseline, floorval custody-green; byte-compare
  against `rust/target/release/regex_perf_probe` before any change).
- **Candidate:** the post-change all-11 regen + fat-LTO rebuilt
  `regex_perf_probe` (mimalloc_perf, the standard closure-bench config).
- **Protocol:** the `-0200` harness re-used (3 base floor-validation rounds; 5
  alternating bench rounds; canonical PCRE2 external-corpus sweeps both sides;
  every run under the memory guard at 16384 MB, serialized).
- **LAND iff ALL of:** unrounded candidate corpus geomean STRICTLY below the
  same-session base geomean; verdict flips 0/2,189; candidate MAX ≤ 483,583 ns
  settled bound. Equal-or-higher geomean, any flip, or a MAX breach ⇒
  **unconditional product reversion** (lib + emitter + regenerated artifacts);
  only the design + rejection evidence commit.
- **Honest magnitude expectation:** the implementable residual is the coverage
  snapshot/truncate + the chain compare + the (never-taken on regex,
  `chain_len`=1) increment — a sub-noise single-digit-ns model. Direction is
  adjudicated by the strict same-session ratchet exactly as `-0198`/`-0199`.
- **Correctness oracles at the changed vintage (all must be green):**
  dual-feature lib suite (incl. the ALL-11 interpreter↔generated byte-identical
  oracle), regex cert coverage seeds 0/7/42 (`268/9/259/0 fully_certified`,
  `spf=0`), `ast_shape_contract_gate`, `duality_hunt_gate`,
  `regex_pcre2_compile_oracle_gate`, clippy source-strict (generated-stage debt
  at the tracked 290), all-11 regen train with the ebnf fixed point + an
  expected-delta review over every artifact (deltas ONLY at the `-0201`
  surfaces: `try_parse_bare` body + the two island cleanup calls).

## 7. Behavior-identity argument (the correctness boundary)

- Protocol parses: byte-identical — every change is bare-graph-only emission or
  an additive lib API; existing lib entry points delegate with
  `maintain_chain_diag = true`.
- Bare parses: verdict / AST / `furthest_position` / error payloads identical —
  the coverage stack is length-invariant on the bare path (§3), and the chain
  counter feeds no parse decision. The ONLY observable delta is the value of
  `rollbacks_nonempty_chain` read through the public accessor AFTER a bare
  parse — the documented observed-parse boundary (§4).
- The interpreter twin needs no mirror: it has no bare graph (protocol
  semantics throughout), and the equivalence oracles compare verdict +
  `furthest_position` + typed AST, never store counters.
