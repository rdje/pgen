# RGX-0078.5.j.4 — BATCH-1 PRICING PRE-FLIGHT

`PGEN-RGX-0078-0174` · session #170 · 2026-07-20 · **read-only measurement**

Executes the `-0173` NEXT pointer: price BATCH-1 members **2** (C2 fact-ops) and
**6** (trace-eagerness) from per-parse semantic-store event counts, **before**
committing to the BATCH-1a chain that `-0173` recommended.

**Outcome in one line: the pre-flight refutes the `-0173` recommendation. Members
2 and 6 are REFUSED on population grounds, BATCH-1a is withdrawn — and a
different, much larger member is uncovered in the same population.**

---

## 1. Custody

Two hard gates, both asserted in-run; the script refuses on either mismatch.

- artifact `generated/regex_parser.rs` sha256[0:8] = **`e4924024`** (banked floor vintage) ✅
- probe `rust/target/debug/parseability_probe` **newer than** that artifact ✅

⚠️ **Gate 2 exists because this slice caught a stale instrument.** The on-disk
debug probe predated the artifact by 28 minutes and embedded a **different regex
vintage**; every count it produced would have described the wrong parser. It was
rebuilt under the memory guard (`--budget-mb 16384`, exit 0, peak RSS 10,793 MB,
216 s) before any number was taken. *A measurement instrument needs custody just
as much as a measured artifact does.*

---

## 2. Measured — the 8-pattern bench population

| pattern | bytes | entries | facts_emitted | predicate_evaluations |
|---|---:|---:|---:|---:|
| `literal_simple` | 4 | 25 | 0 | 0 |
| `digit_sequence` | 17 | 98 | 0 | 0 |
| `character_class` | 46 | 445 | 0 | **8** |
| `alternation` | 12 | 61 | 0 | 0 |
| `capture_groups` | 23 | 124 | **3** | 0 |
| `url_simple` | 12 | 58 | 0 | 0 |
| `email_basic` | 16 | 67 | 0 | 0 |
| `anchor_complex` | 37 | 159 | **3** | 0 |
| **mean / parse** | | **129.6** | **0.75** | **1.00** |

⭐ **Seven of eight bench patterns emit ZERO facts, and seven of eight evaluate
ZERO predicates.** On the geomean steering band the semantic store is very
nearly idle.

---

## 3. Adjudication — members 2 and 6 are REFUSED

| member | population per parse | verdict |
|---|---|---|
| **2 — C2 fact-ops** | **0.75** FactIndex inserts | ⛔ **REFUSED — population does not exist** |
| **6 — trace-eagerness** | **1.75** String allocations (facts + predicates) | ⛔ **REFUSED — population does not exist** |

This is the **valid** refusal reason under the `-0170` amended rule — *"a lever
is refused ONLY when its POPULATION does not exist, never merely because it is
small."* At 0.75 and 1.75 events per parse these populations are not small,
**they are almost absent**, and no capture fraction of them is measurable
against a 28.8 ns noise floor on a 1,263.4 ns parse.

⛔ **The `-0173` BATCH-1a recommendation is WITHDRAWN.** I proposed a lib-only
chain around exactly these two members one slice ago. The pre-flight that
`-0173` itself specified has now killed it — **for the cost of one guarded
rebuild and eight parses, instead of a regen, two fat-LTO probes and a full
battery.** This is the `-0171` discipline applied one slice earlier still, and
it is the second consecutive time a chain was stopped *before* being spent.

⚠️ **The trace-eagerness DEFECT is still real and still worth fixing** — 9 of 10
`rule_context_path()` sites allocate unconditionally at trace level `none`
(`-0173` §1.4). It is simply **not a speed lever on this band**: it is a
correctness-of-instrumentation cleanup, to be landed on its own merits with **no
perf claim attached**, not a BATCH-1 member. On a fact-heavy grammar
(SystemVerilog) its population is orders of magnitude larger.

---

## 4. ⭐ What the pre-flight uncovered instead — member 5 re-scoped and PROMOTED

The counters that are near-zero are the *semantic* ones. The structure that is
**not** near-zero is the one member 5 targets:

- `push_rule_context_static` is emitted at **215 sites** against **284** rule
  methods, and it runs on **every rule entry** — bench mean **≈130 push/pop
  pairs per parse**, i.e. **~74× the combined population of members 2 and 6.**

And the decisive structural fact, verified repo-wide:

- `current_rule_context_stack` is referenced **only inside
  `semantic_runtime.rs`** (8 references, lines 2410–2599).
- Its **only** reader is `rule_context_path()`, whose callers are **all 10
  diagnostic/trace sites** and **zero callers outside that file**.
- `current_rule_context()` — the other accessor, documented "used by the
  state-store **trace events**" — has **ZERO callers anywhere in the repo**, and
  **zero occurrences in any generated parser**. It is dead API surface.

⇒ **The entire rule-context stack is a DIAGNOSTIC-ONLY structure maintained
unconditionally on the hot path, ~130 times per parse.**

**Strong precedent that this structure pays.** The comment at
`semantic_runtime.rs:2564-2569` records that `.5.i.2` (P0) already harvested
this exact structure once — removing a `to_string()` per rule entry was
**"≈−4.3% of the regex bench"**. That slice took out the *allocation* and left
the `Vec<Cow>` push/pop behind. **What remains is the residue of a lever that
already paid −4.3%.**

**Proposed mechanism — the P-env pattern (`-0099`), LIB-ONLY:** gate
`push_rule_context_static` / `pop_rule_context` on a **process-once cached
trace-enabled flag**, exactly as `-0099` hoisted a per-parse `getenv` to a
process-once cache. Trace level is process-level and cannot change mid-parse, so
the gate is sound; with fat-LTO the check collapses to a predictable branch on a
cached bool and the `Vec` traffic disappears.

⚠️ **NO BAND IS DERIVED, and none may be.** This slice counts a **population**
(≈130 push/pop pairs per parse) and establishes that the structure is
diagnostic-only. It does **not** measure the per-push nanosecond cost — an
inlined `Vec` push does no allocation and would be invisible to the
allocation-traffic decomposition that produced the `-0171` figures, and per
`-0172` a static or structural quantity is not nanoseconds. What can be said
honestly: **unlike members 2 and 6, this population is not sub-noise by
inspection**, and being LIB-ONLY it is the cheapest possible A/B that could
settle it.

⚠️ **Relation to the `-0172` refusal.** An inlined push/pop is memory traffic
inside the fused cascade functions — i.e. plausibly part of the 39.9–40.4%
memory-traffic share `-0172` named and explicitly declined to price. This slice
**names a candidate mechanism for part of that share; it does not price it.**

---

## 5. Consequences for BATCH-1

| member | status after this pre-flight |
|---|---|
| 1 — G1-B carrier (index, not box) | unchanged; emitter, unpriced upside |
| 2 — C2 fact-ops | ⛔ **REMOVED — population absent (0.75/parse)** |
| 3 — G1-C per-atom | unchanged; 26.7 ns attributed, HIGH risk |
| 4 — memo segment-copies | unchanged; 30.7 ns attributed |
| 5 — semantic-runtime | ⭐ **RE-SCOPED + PROMOTED** — diagnostic-only stack, ≈130/parse, LIB-ONLY mechanism |
| 6 — trace-eagerness | ⛔ **REMOVED from the batch** — real defect, no perf claim, land separately |

**BATCH-1a as recommended by `-0173` does not exist any more.** Its replacement
candidate is a **different** lib-only slice built on member 5.

**Campaign position unchanged:** the attributed inventory is still
BATCH-1 (members 3+4+5's allocation traffic) + G3 ⇒ ≈−10…−13% against the
−20.8% the <1 µs bar needs. This slice removed two members and re-scoped one; it
did not change the arithmetic, and it did not bank a floor number.

---

## 6. Honest bounds

1. **Counts, not nanoseconds.** No ns figure is derived anywhere in this slice.
2. **The 8-pattern bench is the geomean steering population, not the corpus.**
   Facts/predicates are far denser on other grammars (SV) and on nest/ladder
   cells; these verdicts are scoped to the geomean band and say so.
3. **Why the `-0163` instrument rule does not bite:** that rule forbids pricing
   a *fused-path frame* with a counter census, because enabling counters routes
   the parse to the protocol graph. A semantic-store **event** count is
   graph-invariant — the twin is pinned byte-identical on verdict and typed AST,
   which entails the same facts emitted and predicates evaluated on either
   graph. *This is an argument, not a measurement.* A cross-check of one cell on
   both paths remains owed.
4. **The ≈130 entries/parse mean is the bench mean of `total_entries`**, which
   counts rule-method entries; the emitted push/pop population is bounded by it,
   not necessarily equal to it (215 of 284 rule methods carry the emitted call).
5. **`current_rule_context()` being dead API is a repo-wide grep result**, sound
   for this repo; it is `pub`, so an external embedder could in principle call
   it — the emission slice should adjudicate whether to keep it as public API.

---

## 7. Reproduce

```bash
# rebuild the instrument if the artifact is newer than the probe (guarded)
scripts/run_with_memory_guard.sh --budget-mb 16384 --timeout-s 3600 -- \
  bash -c 'cd rust && cargo build --features generated_parsers --bin parseability_probe'

# the pre-flight (asserts artifact vintage AND probe freshness; refuses on either)
bash docs/tasks/artifacts/batch1_preflight/preflight_store_counters.sh
```
