# PGEN-RGX-0078-0214 — design pre-registration: the adaptive fat-memo pre-size

Banked BEFORE any code change (session #185, 2026-07-21). Selected by the
`-0213` re-price adjudication (`docs/tasks/artifacts/carrier48_reprice/
step_result.md`) as this session's ONE fix under the `-0197` ratchet.

## The mechanism (tool-backed, `-0213`)

The emitted constructor (`ast_based_generator.rs:1535`, comment "Optim #7")
pre-sizes the PROTOCOL (fat) memo at a FIXED 256 capacity. In the floor
probe's inlined constructor this is a machine-pinned

- `_mi_malloc_aligned(0x27208 /* 160,264 B */, 8)` — 512 hashbrown buckets
  × 312-B `((RuleId, usize), MemoEntry)` bucket,
- plus the 520-B ctrl memset at `+0x27000`,
- plus the matching `mi_free` at teardown —

**every parse**, feeding a path whose dynamic price on the thin-memo
architecture is ≈2.5 ns (`memoized_call`; the 28 hot thin rules bypass the
fat memo since `-0118`/`-0205`, so the "Optim #7" premise — "~50-200 memo
entries even for small patterns" — is stale). The allocator population's
band gradient (13.7% → 11.9% → 9.9% of stored samples) carries the
fixed-per-parse signature; the mi large-object lanes (`_mi_malloc_generic`,
`_mi_theap_malloc_zero_aligned_at_overalloc`, `__mi_page_retire`) are the
160 KB size-class path.

Paper trail: K3a (`-0141`) deliberately left this pre-size out of scope
("SV-hot, no SV measurement here — recorded K3a-residual"); the C2 design
text pre-named the exact fix shape ("adaptive `.min(input.len())` so a
5-char pattern never eats a 256-bucket upfront alloc — the tiny-input
hazard"). This unit is also the `-0210`-banked construct-LESS fork
CONCRETIZED (constructing less per-parse state), satisfying its
MUST-SURFACE obligation via this durable record + the `-0213` step result
BEFORE code (the 2026-07-15 amended fork directive: surface + ONE
recommendation + durable record, then proceed).

## The change (exact)

`rust/src/ast_pipeline/ast_based_generator.rs:1535` (the single live
emitter site; the annotation pair regenerates through the same template in
bootstrap mode; `ast_code_generator.rs` — the legacy bootstrap lane — emits
a plain `HashMap` and is not in the 11-artifact train):

```rust
// before
memo: rustc_hash::FxHashMap::with_capacity_and_hasher(256, Default::default()),
// after
memo: rustc_hash::FxHashMap::with_capacity_and_hasher(
    (input.len() + 1).min(256),
    Default::default(),
),
```

with the stale "Optim #7" comment rewritten to record the adaptive form and
its `-0213` evidence (the tiny-input hazard; SV-hot ceiling kept at 256).

Fix-hierarchy tier: emitter capacity hint (the established C2/K3a tier).
Correctness-neutral BY CONSTRUCTION: `with_capacity` is a hint — no parse
decision, memo semantics, wire byte, or public surface changes; the map
still grows on demand. No gating (applies to all parsers uniformly), no
new branches on any hot path, no new state.

## Priced properties

- **Who shrinks:** 2,182/2,189 corpus cells (99.7%) have `pattern_bytes`
  < 255 (sub-1 µs band median 6 B → capacity 7 → 8 buckets ≈ 2.6 KB alloc
  instead of 160,264 B). **Who is untouched:** the 7 cells ≥255 B — incl.
  the 3,511-B corpus-MAX holder — keep capacity 256 byte-identically, and
  every SV-class input (≫255 B) keeps today's exact behavior (the K3a
  SV-hot note honored).
- **Growth risk (bounded, measured by the ratchet):** a <255-B parse that
  inserts more than `len+1` fat-memo entries re-grows by doubling. On
  regex the fat-memo insert population is dynamically ≈0; pathological
  cells are guarded by the 425,000 ns MAX bound and the 0-flip identity
  requirement.
- **Prediction (falsifiable):** −1.0…−5.0% corpus geomean, strongest in
  the sub-1 µs band; bands should improve non-uniformly (the fixed-cost
  signature inverted). Any non-decrease of the unrounded pooled geomean ⇒
  unconditional product reversion per the ratchet.

## Adjudication protocol (pre-registered)

The `-0212` hardened land gate verbatim, re-targeted:

1. Base = `preserved_probes/regex_perf_probe_carrier48_8d392176`
   (sha `8d392176b8456ac66ddaa62b989ad78204029a2c360caef19c4d811a40c1f75c`),
   copied to scratch with its SHA banked BEFORE regeneration.
2. Pre-regen artifact SHA bank; all-11 regen train (tool rebuilt first;
   ebnf canonical re-derivation + FIXED-POINT byte-identity check;
   8 `focus_*` targets); post-regen SHA bank; **expected-delta review:
   every artifact's diff must be the constructor pre-size lines ONLY** —
   any other delta REFUSES the land.
3. Candidate probe = `cargo build --release --features
   "generated_parsers mimalloc_perf" --bin regex_perf_probe` (fat-LTO,
   the standard closure-bench config), mtime-verified newer than every
   touched source.
4. Bench floorval ×3 on the base (custody ±6% vs the banked 1,579.7 ns)
   + 5 alternating bench rounds (steering only).
5. Corpus: the interleaved base₁→cand₁→base₂→cand₂ sweeps with per-sweep
   load snapshots; custody gates: each base sweep within ±3% of the
   1,004.4 ns floor-of-record AND base₁↔base₂ within ±2.5%; adjudication
   on pooled per-cell minima (fair to both sides); verdict flips must be
   0/2,189; candidate MAX ≤ 425,000 ns.
6. Full battery (the `-0210` shape): dual-feature `ast_pipeline` +
   `parseability_probe` rebuilds, dual-feature lib suite (incl. the
   ALL-11 interpreter↔generated oracle), cert ×3 seeds 0/7/42
   (banked `268/9/259/0 fully_certified`, spf 0), `ast_shape_contract_gate`,
   `duality_hunt_gate`, `regex_pcre2_compile_oracle_gate`,
   `clippy_on_rust_change` — all memory-guarded at 16384 MB, one heavy
   job at a time.
7. LAND ⇒ preserve the candidate probe as
   `preserved_probes/regex_perf_probe_adamemo_<sha8>`, re-baseline the
   floor of record + books/LIVE/MEMORY, accept commit, prove clean, STOP.
   REVERT ⇒ byte-restore, bank the refused diff + numbers, STOP (the
   `-0209`/`-0210` refusal shape).

## Deviations

None at banking time. Any implementation-time deviation gets a dated
addendum here BEFORE proceeding (the `-0210` §3 precedent).

### Addendum 2026-07-21 (execution-time custody correction, recorded honestly)

The prereg's step 2 banked pre-regen artifact **SHAs** but the
expected-delta review (the `-0210` instrument) diffs against pre-regen
**file copies** (`$S/artifacts_pre_regen/`). The omission was caught while
the regen train's FIRST step (the tool build, which writes no artifacts)
was still running; the copies were taken immediately and verified
byte-genuine against the already-banked SHA list (`diff` of the copies'
`shasum` output vs `artifacts_pre_regen.sha256` — clean). No artifact had
been regenerated at copy time, so the review baseline is exact. Process
note for the next unit: bank COPIES + SHAs together, before launching the
train.
