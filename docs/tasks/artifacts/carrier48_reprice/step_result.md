# PGEN-RGX-0078-0213 — carrier48 re-price: result + adjudication

Session #185, 2026-07-21. Read-only measurement slice (no product change in
this leaf). Floor probe `preserved_probes/regex_perf_probe_carrier48_8d392176`
(sha256 `8d392176b8456ac66ddaa62b989ad78204029a2c360caef19c4d811a40c1f75c`).

## Custody

- Probe + sampler-source SHAs asserted by `run_capture.sh` AND re-asserted by
  every analysis script (REFUSE on mismatch).
- Band inputs byte-derived from the LAST custody-green `-0212` interleaved
  candidate sweep (`result_carrier_slimming/rerun_cand2.jsonl`, sha256
  `580ab972…`): geomean reproduces **1003.3049233222944 ns** exactly
  (REFUSE-gated), bands 1127/701/350, 11 cells ≥20 µs excluded by design;
  `rerun_cand1` agrees at −0.21%.
- Weighting = band log-shares × the floor geomean (the `-0186` convention);
  noise span carried: 28.8/1263.4 × floor = **22.871 ns**.
- Load custody per the `-0212` hardened protocol: `load_snapshots.txt` around
  every band — no foreign compute job (max non-system CPU ≈ 22% UI/system
  churn; the `-0212` contaminator class idle ≤ 4.2%). First capture attempt
  aborted by an instrument bug (SIGPIPE from `ps|sort|head` under
  `set -o pipefail` in the new snapshot_load; fixed to `awk 'NR<=6'`,
  re-run clean — recorded honestly, no numbers were taken from the aborted
  run).
- Capture: zero drops, stored 65,488/61,746/45,499, target
  7,346/7,177/6,140, verdict identity vs the banked floor sweep on all
  2,178 band rows (`capture_analysis.txt`).
- Role map RE-DERIVED from scratch per the `-0171` standing rule
  (`classify_and_reprice.py`): the 28-offset census over the two target
  symbols closes EXACTLY onto the 9 roles — every offset ≥0x230 in the old
  `-0208` map shifted DOWN uniformly by 0x20 (the `-0212` parser-field
  shrink), the low block (guard 0x8–0x60, checkpoint-six 0xF0–0x198)
  unchanged; machine qualification PASS, REFUSE-on-unknown. The thin-memo
  dense entry stride is asserted STILL 96 B (`#0x60` umaddl) — the
  `ThinTapeMemoEntry` (stamp + 6-word inline SmallVec outcome) never
  contained the carrier, so the `-0212` slimming correctly did not shrink
  it (source-verified, `mod.rs` `ThinTapeMemoEntry`).

## The fresh price list (floor 1003.3049233222944 ns; noise 22.871 ns)

In-target tier-1 role lanes (ALL sub-noise, refused as solo fixes —
the `-0206`/`-0208` conclusion reproduces on the new floor):
input_view 13.00, checkpoint 9.60, position 5.74, guard 4.38, tape 3.10,
rows 1.62, entries 0.94, arena 0.87, taint 0.01.

Out-of-target populations (weighted ns):

| population | ns | note |
|---|---|---|
| spine_other | 186.60 | was 203.75 pre-slim; still the largest parser-owned mass |
| allocator | 122.58 | mi_free 35.62 + malloc_generic 21.99 + malloc_aligned 18.73 + zero_aligned 13.03 + page_retire 11.62 + overalloc 10.75 + … |
| external_or_injected | 113.37 | libsystem/kernel (memcpy family, syscalls, sampler) |
| other_text | 83.04 | outlined helpers, clones, trim |
| build_value | 72.66 | to_shaped_value 21.71 + cascade_build_piece 22.14 + … |
| arena_alloc | 69.42 | alloc_extend monomorphs |
| teardown_drop | 50.93 | drop RegexParser 13.78 + drop NodeArena 10.49 + state vec drops |
| harness | 50.35 | parse_once_timed 44.29 (probe timing shell — NOT parser-ownable) |
| semantic_runtime | 49.36 | FactIndex::insert 7.16 + transaction shells |
| vec_growth | 28.60 | finish_grow 16.51 + grow_one family |
| tape_helpers | 20.30 | decode_event 18.74 |
| thin_entry_push | 16.35 | the two 96-B push_mut monomorphs |
| hashbrown_map | 14.49 | insert monomorphs |

## Spine decomposition (fresh, `decompose_spine.py` — re-sums 186.5951 EXACTLY)

- 170 symbols; top: cascade_regex 25.97 / parse_pattern 16.02 /
  parse_regex::{closure} 12.27 / cascade_match_alternative 10.32 /
  scan_letter 10.12 — **no symbol reaches noise except cascade_regex
  (1.14×), whose clusters are the once-per-parse ROOT ceremony** (epilogue
  ldp stalls 146+35 samples; final-value staging into the arena).
- Transport tally (mechanical, re-sums): x_scalar_state_or_elem 84.64 +
  x_frame_spill(sp/fp) 65.51 + q_payload 15.72 + other_alu 13.59 +
  control 6.94 + call 0.20. vs `-0211` on the fat carrier: sp-staging
  83.68 → 72.15 (−11.5), q-copies 21.35 → 15.72 (−5.6) — the slimming
  delivered exactly where priced; the residual is irreducible-shaped
  (48-B carrier ABI + frame ceremony; arena-ref ABI REFUSED `-0211`,
  compiler tuning REFUSED family).

## ⭐ THE FINDING — the fat-memo tiny-input pre-size hazard, machine-pinned

The emitted constructor (`ast_based_generator.rs:1535`, "Optim #7") pre-sizes
the PROTOCOL (fat) memo at a FIXED 256 capacity:

```
memo: rustc_hash::FxHashMap::with_capacity_and_hasher(256, Default::default())
```

In THIS probe's disassembly the inlined constructor allocates it as

```
mov  w0, #0x7208
movk w0, #0x2, lsl #16        ; 0x27208 = 160,264 bytes
bl   _mi_malloc_aligned
add  x0, x0, #0x27, lsl #12   ; + 0x27000 = ctrl offset 159,744
mov  w1, #0xff                ; hashbrown ctrl memset (520 B)
```

i.e. **a 160,264-byte allocation + free EVERY parse** (512 buckets × 312 B
per `((RuleId,usize), MemoEntry)` bucket + 520 ctrl) — feeding a path whose
DYNAMIC price is ≈2.5 ns (`memoized_call`): post-`-0118`/`-0205` the 28 hot
thin rules bypass the fat memo entirely, so the "Optim #7" premise ("even
small regex patterns produce ~50-200 memo entries") is STALE on the thin-memo
architecture. The allocator population's band gradient (13.7% → 11.9% → 9.9%
of stored samples, falling with input size) carries the fixed-per-parse
signature; the mi large-object lanes (malloc_generic / overalloc /
page_retire) are exactly the 160 KB size-class path.

Paper trail: K3a (`-0141`) deliberately left this pre-size OUT of scope
("SV-hot, no SV measurement here — recorded K3a-residual"); the C2 design
text (`RGX-0078.md` §C2) pre-named the exact fix shape: "adaptive
`.min(input.len())` so a 5-char pattern never eats a 256-bucket upfront
alloc — the tiny-input hazard".

Corpus exposure (from the SHA-pinned `rerun_cand2` rows): **2,182/2,189
cells (99.7%) have `pattern_bytes` < 255** (sub1us median 6 B, max 73 B) —
all shrink; the 7 cells ≥255 B (incl. the 3,511-B corpus-MAX holder) keep
the 256 cap ⇒ byte-identical behavior where the fat memo is actually hot.

## Adjudication

1. **SELECTED — the `-0214` ONE fix: adaptive fat-memo pre-size**
   `with_capacity_and_hasher((input.len() + 1).min(256), …)` at the single
   emitter site (`ast_based_generator.rs:1535`; all 11 artifacts regen).
   This is the banked construct-LESS fork CONCRETIZED as the proven
   C2/K3a capacity-hint idiom — construct LESS for tiny inputs, identical
   for len ≥ 255 (the SV-hot case), no feature/grammar gating, no new
   branches, no state, correctness-neutral by construction (capacity hint
   only). Designed mechanism ≥ noise: eliding a 160 KB alloc+free pair +
   ctrl memset per parse against the 122.6 ns allocator + 113.4 ns
   external populations; prediction (falsifiable): **−1.0…−5.0% corpus
   geomean, strongest in the sub-1 µs band; MAX guarded** (the MAX-holder
   cells keep the 256 cap or are ≥20 µs where the fat memo is busy).
   Growth-risk bound: fat-memo inserts on regex are dynamically ≈0
   (memoized_call 2.5 ns); a sub-255-B parse inserting past `len+1`
   entries re-grows by doubling — bounded, measured by the ratchet.
   MUST-SURFACE obligation (process law #175): this fork is surfaced +
   durably recorded HERE and in the tree leaf before any code; the
   2026-07-15 amended fork directive (surface + ONE recommendation +
   durable record, then PROCEED) licenses proceeding this session.
2. **REFUSED this session — spine_other**: still the largest mass but
   mechanism-less after the slimming (no symbol ≥ noise except the root
   ceremony, whose only levers are the refused compiler-tuning family /
   arena-ref ABI); stays research-class.
3. **REFUSED this session — the `-0170` sub-noise batch** (checkpoint 9.60 +
   position 5.74 + guard 4.38 + tape 3.10 ≈ 22.8 ns ≈ 1.0× noise gross,
   no coherent single mechanism).
4. **BANKED follow-ups** (sub-noise singles observed this capture, priced
   honestly, NOT selected): the per-parse `grammar_profile`
   `Some(DEFAULT.to_string())` heap alloc of a constant (`Cow<'static>`
   shape); the per-parse `Arc<Vec<AtomicU64>>` rule-call-counter slice
   construction (275 × 8 B + Arc ceremony; the INCREMENTS are
   contract-blocked telemetry per `-0197`, the CONSTRUCTION is not);
   both are mi small-path items well under noise alone.

## Outputs banked in this dir

`run_capture.sh` + `capture_provenance.txt` + `load_snapshots.txt` +
`input_derivation.txt` + `band_manifest.json` + the three
`capture_band_*_raw_pcs.txt`/`_times.jsonl`/`_stdout.txt` +
`teardown_qualification_*` + `raw_custody.json` + `analyze_capture.py` →
`capture_analysis.txt` + `classify_and_reprice.py` → `role_reprice.txt` +
`decompose_spine.py` → `decomposition.txt` + this file.
