# `PGEN-RGX-0078-0205` — pre-registered design: the thin-memo direct-index carrier

Session #179, 2026-07-20. Pre-registered BEFORE any code, per the `-0197`
one-design-before-implementation discipline. The `-0204` re-pricing selected
this as the session's ONE fix.

## The priced target (from `-0204`, on the 1051.88 ns floor, noise 23.978 ns)

| machinery | weighted ns | attribution |
|---|---:|---|
| `HashMap::insert h6884…` self | 48.127 | bl-sites = exactly the fused thin-memo stores (`cascade_match_atom` ×14, `conditional` ×6, `piece`/`alternative`/`extended_class_content`/`code_content` ×3 each, + `try_parse_bare` wrappers) |
| map teardown (`drop_in_place<HashMap<(u16,usize),ThinTapeMemoEntry,FxBuildHasher>>` ×2) | 11.463 | per-parse bucket-walking drop |
| in-target RawTable fields (tier-1 `thin_memo_table` 0x4b0–0x4c8) | 1.384 | FxHash mix + NEON group probe + items/growth bookkeeping |
| **total identifiable** | **60.97** | **2.54× noise gross; 30/50/70% = 18.3/30.5/42.7 ns** |

Strict net banked pre-A/B: **0** (the `-0197` accounting rule). The corpus
ratchet adjudicates; unconditional reversion on a miss.

## Current representation (what changes / what must NOT change)

Artifact census (`c5e1b425`, whole-file matching): **28 thin-memoized fused
rules** (56 `insert` sites = success+failure per rule; 28 `get`; 28
`remove`). Emitted frame per rule: key `(RULE_<R>, position)` → `get` →
stamp validation (PURE / STORE-READ / never-cache-MUTATING) → hit = ONE
`extend_from_slice` splice + position jump, or cached-failure backtrack →
stale = `remove` → body → classify → `insert` (segment via ONE
`SmallVec::from_slice`, or failure).

**Carried VERBATIM (the `-0203` lane ledger — no double credit):** the
`ThinTapeMemoEntry` type and its stamp/taint doctrine; the
`from_slice`/`extend_from_slice` segment copy lanes; the epoch/deferred/
predicate counters; the recursion-guard frame; the protocol memo
(`memo`/`memo_fail`/`memo_fail_tainted` — shared with observed parses,
different soundness surface — explicitly OUT of scope).

**Replaced:** the container only — hashing, group probing, ctrl-byte
maintenance, key storage, amortized growth/rehash, and the bucket-walking
drop.

## The design fork (adjudicated in-design)

**(A) Naive per-parse direct index** — `vec![0u32; 28×(len+1)]` per parse.
REFUSED: the zeroed row array costs one alloc + memset per parse
(≈2.9 KB at len 25 ≈ 40–100 ns incl. alloc) — an ADDED fixed cost on the
sub-1 µs band (43.6% of the geomean weight) that plausibly cancels the
per-probe win on exactly the cells the bar is measured on.

**(B) TLS-recycled generation-stamped rows (SELECTED)** — the row table
lives in a thread-local scratch, recycled across parses; validity is a
generation stamp, so `begin()` is one counter bump instead of a memset:

- Lib (`mod.rs`, additive):
  - `ThinMemoScratch { slots: Vec<u64>, gen: u32 }` — slot word =
    `(gen as u64) << 32 | (idx+1)`; `0` = never-written;
    `begin(need)`: `gen = gen.wrapping_add(1); if gen == 0 { slots.fill(0);
    gen = 1; } if slots.len() < need { slots.resize(need, 0); }` (resize
    zeroes only the new tail; old slots die by generation mismatch);
    `#[inline] lookup(slot) -> Option<u32>` (load + high-word compare);
    `#[inline] store(slot, idx)`; `#[inline] clear(slot)` (stale eviction);
    u32-overflow guard at the call site: an entries index that would not
    fit u32 SKIPS caching (a memo skip is always sound).
  - `ThinMemoScratchLease` — RAII wrapper; `Lease::take()` moves the scratch
    out of a `thread_local` `Cell<Option<…>>` (cold path: fresh default);
    `Drop` returns it. The lease is a parser FIELD, so every exit path —
    success, error, panic-unwind — returns the scratch without an `impl
    Drop for RegexParser` (no E0509 partial-move hazard on other fields).
    Two parsers alive on one thread simply means the second takes a fresh
    scratch (correct, merely unshared).
- Generated parser (emitter):
  - fields (gated by `cascade_thin_memo_active`, replacing `thin_memo`):
    `thin_scratch: ThinMemoScratchLease`,
    `thin_entries: Vec<ThinTapeMemoEntry<'input>>`, `thin_stride: usize`;
  - ctor: `thin_stride = input.len() + 1`; lease take +
    `begin(THIN_RULE_COUNT * thin_stride)`; entries
    `Vec::with_capacity(((input.len()+1)*6).min(32768))` — the K3a
    formula carried over (the dense vec at ~96 B/entry reserves LESS than
    the map's bucket array did);
  - consts: `THIN_RULE_COUNT` + one `THIN_ROW_<RULE>` per thin rule,
    assigned in **sorted rule-name order** (the emitter's set is a
    `HashSet` — sorting is what keeps the artifact regen-deterministic);
  - per-rule frame: `slot = Self::THIN_ROW_<R> * parser.thin_stride +
    position`; lookup/validate/splice/evict/insert exactly as today with
    `thin_entries[idx]` as the entry place (disjoint-field borrows: entries
    shared vs tape mut — same shape the map form already used).
- Stale eviction = `clear(slot)` + append-a-new-entry on re-insert; the
  superseded entry stays as bounded garbage in the dense vec (bounded by
  the same store-epoch churn that bounded today's `remove`+`insert`).

## Why this dominates on every band (the pre-registered mechanism argument)

- per probe: one mul/add + one load + one compare, vs FxHash mix + NEON
  group probe + key compare;
- per insert: `Vec::push` + one slot store, vs hash + probe + ctrl write +
  key+value bucket write + growth amortization;
- per parse (small cells): NO map allocation, NO bucket-walk drop; ADDS one
  TLS take/put (~few ns) + a generation bump; the cold first-parse-per-
  thread pays one zeroed alloc that every later parse on the thread reuses
  (min-of-samples corpus semantics measure the warm state; real consumers
  parse many patterns per thread);
- per parse (giant cells): rows resize is one-time per thread high-water
  mark (28 × 3,512 × 8 B ≈ 787 KB for the 3,511 B MAX cell — vs the map's
  ≥3.4 MB bucket reservation at the same K3a capacity);
- teardown: entries-vec drop walks POD-`Copy`-inline entries (branch on
  spill) — strictly less than bucket-walk + per-bucket drop.

Falsification bounds (pre-registered): if the corpus geomean does not
strictly decrease vs base `unitape_6812d8e2` — or any verdict flips, or MAX
exceeds the re-settled 425,000 ns — the product change reverts
unconditionally and only the rejection record commits.

## Battery / regen / A-B plan (the `-0203` harness re-homed)

1. lib change (`mod.rs`: scratch + lease + TLS) + emitter change
   (`ast_based_generator.rs` struct/ctor/consts; `cascade.rs` frame) +
   text-pin re-anchor in `cascade.rs` unit tests;
2. all-11 regen train + ebnf fixed point + expected-delta review (growth
   bar 5%; thin-memo-active artifacts change, others byte-identical);
3. full battery: dual-feature lib tests incl. the ALL-11
   interpreter↔generated oracle; cert ×3 seeds 0/7/42; shape; duality;
   PCRE2 compile oracle; clippy source-strict;
4. A/B: floorval ×3 (base custody) + 5 alternated bench rounds + one
   corpus sweep per side vs the SHA-asserted base probe; adjudicate by the
   ratchet; preserve the candidate probe on LAND.
