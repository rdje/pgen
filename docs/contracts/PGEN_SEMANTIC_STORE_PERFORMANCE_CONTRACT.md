# PGEN Semantic Store — Performance Contract

**Status:** DRAFT (sketch — first cut for review).
**Companion to:** `docs/contracts/PGEN_SEMANTIC_STORE_API_CONTRACT.md` (§6 complexity table) and `docs/contracts/PGEN_SEMANTIC_STORE_TEST_PLAN.md` (§6 stress tests).
**Audience:** Phase 1 implementor; CI gate authors; reviewers.

This document binds the complexity classes from the API contract and the stress-test categories from the test plan to **concrete latency, memory, and scalability numbers** measured on a documented baseline machine. All numbers are SKETCH defaults; we review and adjust before they become binding.

---

## 1. Why this document exists

User mandate, 2026-05-21:

> "The semantic-store shall by no means be the bottleneck of the parser. The semantic-store shall be top-notch, sign-off level quality."

A "bottleneck" is something that shows up in a profile. Without numeric budgets bound to a baseline machine, every implementor optimises to a different mental model. This contract eliminates that ambiguity: every primitive has a budget; CI regresses on the budget; deviations require explicit re-budgeting (with rationale).

## 2. Baseline machine

All numbers in this document are measured on the following reference configuration. **Bench runs on other machines are informational only**; CI gates use a runner matching the baseline as closely as possible.

```
CPU:        Apple M3 Pro (12-core, p-cores at 4.05 GHz)
RAM:        36 GB LPDDR5
OS:         macOS 14.x (Darwin 24.x kernel)
Rust:       stable 1.95+ (matches rust/Cargo.toml rust-version)
Profile:    --release with default profile (debug-assertions=false, opt-level=3)
Allocator:  system (no jemalloc / mimalloc); revisit if profile shows allocator-bound
```

**`[TBC]`** — is M3 Pro the right baseline, or should we pick a more commonly-deployed CI machine (GitHub Actions ubuntu-22.04 x86_64, for example)? Sketch picks M3 Pro because it's the development machine; the gating numbers would migrate ~1.2x slower on x86 CI.

**Re-measure cadence:** quarterly, or whenever the baseline machine spec changes. The contract version (top of file) bumps minor when re-measured.

## 3. Per-primitive latency budgets

All numbers are p99 unless otherwise stated. p50 is informational. Maximum is a hard ceiling: any single call exceeding it is a failed test, regardless of average.

### 3.1 DECLARE — `declare_fact_kind`

| Metric | Budget | Test ID |
|---|---|---|
| p99 latency | ≤ 10 µs | (single-shot at startup; not in hot path) |
| Max latency | ≤ 100 µs | U-DECL-* |
| Memory cost | ≤ 4 KB per declared kind (one index hash-map + metadata) | S-MEMORY-1 |

Rationale: declaration is one-shot per kind at grammar-compile time. Not on the parser hot path. Budget is loose.

### 3.2 EMIT — `emit_fact`

| Metric | Budget | Test ID |
|---|---|---|
| p50 latency | ≤ 200 ns | S-PERF-EMIT-1 |
| p99 latency | ≤ 500 ns | S-PERF-EMIT-1 |
| Max latency | ≤ 5 µs | S-PERF-EMIT-1 |
| Throughput (sustained) | ≥ 2M emits/sec | S-PERF-EMIT-1 |

Rationale: emit is on the parser hot path. uvm-pkg emits ~100k facts during its parse; at 500ns each, total emit time is ~50ms — comfortably below the parser's overall budget.

Memory cost of a single fact (excluding attribute payload, which varies by kind):
- ≤ 256 bytes overhead per fact (FactId + per-index pointers + scope reference + source tag)
- Attribute payload: variable; capped at sum of declared `attributes` * average value size.

### 3.3 QUERY — four primitives

| Primitive | p99 latency | Max latency | Test ID |
|---|---|---|---|
| `has_fact` | ≤ 200 ns | ≤ 2 µs | S-PERF-HAS-FACT-1 |
| `fact_attribute` | ≤ 250 ns | ≤ 2 µs | (extension of HAS test) |
| `fact_count_at_least` | ≤ 50 ns | ≤ 500 ns | (counter-backed; cheapest primitive) |
| `resolve_path` (depth 1) | ≤ 250 ns | ≤ 2 µs | S-PERF-RESOLVE-PATH-1 |
| `resolve_path` (depth 5) | ≤ 1 µs | ≤ 10 µs | S-PERF-RESOLVE-PATH-1 |
| `resolve_path` (depth N) | scales linearly: ≤ 200 ns * N | ≤ 2 µs * N | (covered by P-SCALABILITY-1) |

Rationale: queries are on the hottest parser path (every method-call disambiguation invokes at least one). At 200ns p99, a uvm-pkg parse making ~500k queries spends ~100ms total on querying — well within budget.

### 3.4 SCOPE — open / close

| Metric | Budget | Test ID |
|---|---|---|
| `open_scope` p99 | ≤ 100 ns | (in U-SCOPE-* warm-up) |
| `close_scope` p99 (no exportable facts) | ≤ 100 ns | U-SCOPE-2 |
| `close_scope` p99 (with N exportable facts) | ≤ 100 ns + (cost of `export_scope(N)`) | S-PERF-EXPORT-1 |

Rationale: scope ops are bookkeeping; trivial unless they trigger export.

### 3.5 EXPORT — `export_scope`

| Metric | Budget | Test ID |
|---|---|---|
| Time per exportable fact | ≤ 1 µs (serialisation + index walk) | S-PERF-EXPORT-1 |
| Atomic-write overhead | ≤ 5 ms (one fsync + rename) | (within EXPORT-1) |
| Export of 100k-fact scope | ≤ 200 ms total | S-PERF-EXPORT-1 |

Rationale: export is once-per-scope-close and is dominated by I/O for large scopes. The 5ms fsync floor is set by the OS, not us.

### 3.6 IMPORT — `import_from_library`

| Metric | Budget | Test ID |
|---|---|---|
| Cold load (lazy mode) | ≤ 10 ms for any artefact size | S-PERF-IMPORT-LAZY-1 |
| Per-attribute lazy fetch | ≤ 5 µs | S-PERF-IMPORT-LAZY-1 |
| Cold load (eager mode, off by default) | ≤ 1 µs per index entry + ≤ 5 µs per attribute payload | (informational) |

Rationale: lazy mode is the parser hot-path default — only index entries load eagerly; attribute values lazy-load on first query. A 50MB artefact has ~500k facts; lazy cold load is bounded by the index-entry size only (≤ 100 bytes each = 50MB indexed in <10ms).

### 3.7 TRANSACTIONS — begin / commit / rollback

| Metric | Budget | Test ID |
|---|---|---|
| `begin_transaction` p99 | ≤ 100 ns | (warm-up of U-TX-*) |
| `commit_transaction` (merge to parent) | ≤ 50 ns per undo-log entry | U-TX-7 |
| `commit_transaction` (top-level discard) | ≤ 100 ns | (within U-TX-2) |
| `rollback_transaction` p99 | ≤ 200 ns per undo-log entry | S-PERF-ROLLBACK-1 |
| Rollback of 1M-emit transaction | ≤ 200 ms total | S-PERF-ROLLBACK-1 |

Rationale: rollback latency × emit count = total backtracking cost. SV PEG can backtrack heavily (the `.3.3.3` SEMTRACE showed 110 RESTOREs on a small repro). 200ns per undo means uvm-scale backtracking is sub-second total.

## 4. Memory budgets

### 4.1 Per-entity overhead (excluding payload)

| Entity | Overhead | Notes |
|---|---|---|
| Fact | ≤ 256 bytes | FactId, per-index back-pointers, scope ref, source tag |
| Scope node | ≤ 200 bytes | Kind label, name, parent ptr, child list, fact-list head |
| Transaction | ≤ 128 bytes | Tx header (parent ref, undo-log root) |
| Undo-log entry | ≤ 64 bytes | Op kind, FactId or ScopeId or ImportHandle |
| Index entry | ≤ 64 bytes | Composite-key hash + FactId pointer |
| `KindHandle` | 4 bytes | u32 index into kind registry |
| `ScopeId` | 4 bytes | u32 index into scope arena |
| `FactId` | 8 bytes | u64 (32-bit scope id + 32-bit per-scope counter) |

### 4.2 Total budget at scale

| Scale | Memory budget | Test ID |
|---|---|---|
| 100k facts + 10k scopes | ≤ 60 MB | S-MEMORY-1 |
| 1M facts + 100k scopes | ≤ 500 MB | S-MEMORY-1 |
| 10M facts + 1M scopes | ≤ 5 GB | (informational; rare in practice) |

Rationale: 1M facts × 256B overhead = 256MB; + 1M × ~3 index entries × 64B = 192MB; + 100k scopes × 200B = 20MB; total ~470MB. Budget gives ~6% headroom.

### 4.3 Library artefact size on disk

| Metric | Budget |
|---|---|
| Per-fact bytes in JSON artefact | ≤ 256 bytes (excluding attribute payload) |
| Compressibility | ≥ 5x with gzip (artefact format is JSON, repetitive keys) |

## 5. Scalability requirements

The defining property: **primitive latency must not grow with store size** (in O() terms). The test plan's S-tests verify this by measuring p99 at 10k / 100k / 1M / 10M facts:

| Primitive | At 10k | At 100k | At 1M | At 10M | Required scaling |
|---|---|---|---|---|---|
| `emit_fact` p99 | 500 ns | 500 ns | 500 ns | 500 ns | O(1) |
| `has_fact` p99 | 200 ns | 200 ns | 200 ns | 200 ns | O(1) |
| `fact_attribute` p99 | 250 ns | 250 ns | 250 ns | 250 ns | O(1) |
| `resolve_path` (depth 5) p99 | 1 µs | 1 µs | 1 µs | 1 µs | O(1) in N (linear in depth only) |
| `rollback_transaction` (1k undo-log entries) | 200 µs | 200 µs | 200 µs | 200 µs | O(undo entries) — not O(N) |

A 10%+ regression at any scale point fails the S-SCALABILITY-1 test.

## 6. Configuration — opt-in profiles

The performance contract assumes the **default config**. Two opt-in profiles loosen budgets in exchange for added diagnostics:

- **`debug` profile** (`SemanticStoreConfig::debug()`): all budgets relaxed 10x; full per-primitive timing recorded; useful for development. Off by default.
- **`tracing` profile** (`SemanticStoreConfig::with_tracing(level)`): per-operation events emitted to `PGEN_TRACE_VERBOSITY` stream; budgets relaxed 2x.

Production parsers use the default config. CI bench runs use default config.

## 7. CI gating

The bench harness in `rust/benches/semantic_store.rs` runs the S-tests and produces a JSON report. The CI gate is implemented as `scripts/check_perf_contract.sh`:

1. Run benches; emit `target/criterion/semantic_store/report.json`.
2. Parse the JSON; extract p50, p99, max for every primitive.
3. Compare each measured number to its budget in this document.
4. If any number exceeds budget by > 10%, fail the gate with a precise message:
   `"perf-contract: rollback p99 = 250ns/entry; budget = 200ns/entry; +25% (exceeds 10% tolerance)"`.
5. If any number exceeds budget by 1–10%, warn but pass (slow degradation tracking).
6. Improvements are silently accepted; the baseline auto-updates on confirmed improvement (with audit log).

Re-budgeting (raising a budget) requires:
- Justification in a commit message + a note in `feedback_*` durable memory.
- A separate commit dedicated to the budget change; never folded into a behaviour-change commit.

## 8. Where the numbers come from (rationale by primitive)

- **EMIT 500 ns p99** — rough back-of-envelope: 1 hash-map insertion (≈ 50 ns) × 3 indexes = 150 ns; + undo-log append (10 ns); + scope-list append (20 ns); + allocator (≈ 50 ns); + Rust overhead = ~250 ns mean, ~500 ns p99 with cache misses. Within reach of a tuned hash map (FxHash, robin-hood, etc.).
- **HAS_FACT 200 ns p99** — hash-map lookup with one indirection. Tuned implementations (`hashbrown::HashMap`) measure 50-100ns for u64-keyed lookups; with composite keys + scope-walk, 200ns p99 is achievable.
- **RESOLVE_PATH per-segment 200 ns** — each segment is one `has_fact` + one attribute load. Linear in path depth, not store size.
- **ROLLBACK 200 ns / entry** — each undo entry: one hash-map removal (≈ 100 ns) + per-index removals (≈ 50 ns each). One emit's rollback is roughly the cost of its emit (symmetric).
- **EXPORT 1 µs / fact** — serialisation cost. JSON `serde_json` measures ~200-500 ns per small object on modern hardware; 1 µs leaves margin for I/O batching.

All numbers are RECONSIDER-IF-WRONG. Phase 1 implementor measures and feeds back any budget that's unachievable or wildly conservative.

## 9. Open questions

- **`[TBC]`** Is the M3 Pro baseline appropriate, or should we use a more reproducible CI machine?
- **`[TBC]`** Should the contract require allocator-independence (numbers should hold under system / jemalloc / mimalloc)? Sketch: don't require; pick best for default config.
- **`[TBC]`** The 10% tolerance for CI gating — too tight, too loose? Sketch: 10% catches real regressions while tolerating measurement noise.
- **`[TBC]`** Re-budgeting cadence (quarterly): sufficient, or per-release?

## 10. References

- `docs/contracts/PGEN_SEMANTIC_STORE_API_CONTRACT.md` §6 — complexity classes this contract binds to numbers.
- `docs/contracts/PGEN_SEMANTIC_STORE_TEST_PLAN.md` §6 — the stress tests asserting these numbers.
- `docs/proposals/CONTEXT_AWARE_PARSING_DESIGN.md` §11 — quality bar mandating this contract.
- `feedback_universal_semantic_store` (memory) — the principle being enforced.

---

**Amend freely.** Every number above is SKETCH; review and adjust before any number becomes binding. Once approved, this contract is the final Phase 0.5 artefact and unblocks Phase 1.
