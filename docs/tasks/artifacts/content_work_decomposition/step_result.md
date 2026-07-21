# PGEN-RGX-0078-0215 — content-work decomposition: result + the exhaustion adjudication

Session #186, 2026-07-21. Read-only research slice (no product change).
Executes the `-0214` NEXT pointer's option (a): decompose the last ≥noise
un-decomposed populations on the 1,004.4 ns floor before any further fix
selection. Floor probe `preserved_probes/regex_perf_probe_carrier48_8d392176`
(SHA re-asserted in-run).

## Custody

- NO new capture: the inputs are the SHA-banked `-0213` capture (probe SHA +
  the three raw-PC SHAs from `raw_custody.json` + the band manifest all
  REFUSE-gated by `decompose_content.py` before any number is emitted).
- Every view re-sums to its `-0213` bucket price EXACTLY (tolerance 0.01 ns,
  REFUSE-gated): build_value 72.6611 / arena_alloc 69.4154 /
  semantic_runtime 49.3605 / vec_growth 28.6025 — combined 220.04 ns ≈ 9.6×
  noise (22.871 ns). Weighting byte-for-byte the `-0186`/`-0213` convention.
- Analysis offline and re-runnable: `python3 decompose_content.py` →
  `decomposition_content.txt` (2,700 lines; per-symbol tables + annotated
  within-symbol clusters + per-bucket transport tallies + static caller
  census).

## The decomposition (what each population actually is)

### build_value 72.66 ns — legitimate conversion at its constant floor

- `cascade_build_piece` 22.14: tape `decode_event` dispatch + staging the
  shaped-object bodies (constant `&'static` key strings interleaved with
  dynamic span words) into stack blocks, then arena `alloc_shaped_pairs`;
  0x340-byte frames (transport: the bucket carries 33.2 ns sp/fp spill).
- `ParseContent::to_shaped_value` 21.71: **8.5 ns is ONE instruction** — the
  first-touch discriminant load `ldr x8, [x1]` (1,212 samples), a
  data-dependent pointer-chase over arena-scattered nodes; the rest is the
  conversion walk + the exact-sized transient materialization.
- `cascade_build_alternative` 10.83 / `_quantifier` 4.87 / `_value_literal`
  4.20; 56 further symbols all ≤1.5 ns.
- **Adjudication: NO designed mechanism.** The V1 (build-value) family is
  CLOSED measured-exhausted (`-0154` M1 falsified ≈1 clone/parse; M2 folded
  into K4b; `-0156` M3 REFUSED at +0.30% vs a −2…−6% band). The frame
  ceremony is the refused compiler-tuning family; the discriminant
  pointer-chase is layout-owned (the carrier already slimmed `-0212`,
  arena-ref ABI refused `-0211`).

### arena_alloc 69.42 ns — the re-entrancy-forced materialize-then-copy

- Three `typed_arena::alloc_extend` monomorphs carry 60.26 ns:
  24-B `PgenValue` slice bodies 26.44 (element-by-element tag-checked copy
  loop reading the exact-sized transient, then `mi_free` of it — the
  annotated loop cluster alone is 17.03 ns), 40-B `(&str, PgenValue)` pair
  bodies 23.91 (whole-object q-copy staging through sp), plus a
  string/callout monomorph 9.91.
- The transient exists to satisfy the documented `alloc_extend`
  no-reentrant-allocation constraint (`ast_pipeline/mod.rs:1155`, verified
  in-source `-0155`).
- **Adjudication: NO licensable mechanism.** The family is adjudicated:
  round-trips are ~free (M3 `-0156` refused; the `-0214` law), the heap→
  stack swap refused, and the only remaining shape — reserve-uninit +
  fill-in-place inside `typed_arena` (eliding the read-half of the copy
  loop) — prices at ≈8–12 ns (SUB-noise) against an unsafe redesign of the
  shared representation's drop-correctness. Refused as a solo lever by the
  campaign's own rules; the in-place protocol-zone value-ization road stays
  the recorded out-of-scope `.5.j.1` item.

### semantic_runtime 49.36 ns — pay-per-use store machinery, fragmented

- 45 symbols, none ≥ noise/3: `FactIndex::insert` 7.16 (REQUIRED fact
  content, band-rising — real emissions on corpus cells),
  `with_semantic_runtime_rule_transaction` ≈13.7 across 17 monomorphs
  (contains the `-0206` protocol-memo-inserts fork surface, 12.26 ns,
  sub-noise), `SemanticRuntimeState::new` 6.16 (per-parse init of USED
  state — the recycling lane CLOSED ×2, `-0209`/`-0210`),
  `extract_delta_since_slow` 3.63 (K1-owned deferred-extract),
  `directives_for_rule` 3.25, rollback_slow 2.82, rule_context_path 2.39.
- **Adjudication: NO ≥noise mechanism** — the largest coherent sub-surface
  is the already-banked protocol-memo-inserts fork (sub-noise).

### vec_growth 28.60 ns — amortized doubling of parse-time children vecs

- `finish_grow` 16.51 (its own frame + the entry overflow-check carry
  ≈15.6; the grown-buffer memcpy lives in external memops, not here) +
  `grow_one` family ≈12.0; callers = the `cascade_match_*`/`parse_*`
  closures' children accumulation.
- **Adjudication: NO designed mechanism.** A pre-size design is an
  allocation-size/count census = NOT a price under mimalloc (the `-0214`
  design law, its third face); the K3a input-proportional memo pre-size
  already landed; the copy content is externally-owned memops.

## ⭐ THE EXHAUSTION ADJUDICATION (the campaign-level finding)

With this slice, EVERY population on the 1,004.4 ns floor is decomposed to
symbol/cluster level, and the inventory is:

| mass | ns | status |
|---|---|---|
| spine_other | 186.6 | mechanism-less (ABI/frame; compiler-tuning + arena-ref refused) |
| allocator | 122.6 | fast-path ceremony, proven un-ownable (`-0209/-0210/-0214`) |
| external/kernel | 113.4 | libsystem memcpy/syscalls/sampler — not parser-owned |
| other_text | 83.0 | outlined helpers/clones/trim — no coherent mechanism |
| build_value | 72.7 | THIS SLICE: V1 closed measured-exhausted confirmed |
| arena_alloc | 69.4 | THIS SLICE: materialize-then-copy family closed; residual sub-noise |
| teardown_drop | 50.9 | un-ownable by pooling AND elision (`-0214`) |
| harness | 50.4 | probe timing shell — not parser-ownable |
| semantic_runtime | 49.4 | THIS SLICE: fragmented; largest sub-surface 12.3 sub-noise |
| vec_growth | 28.6 | THIS SLICE: pre-size = refuted-by-law; copies external |
| tape_helpers | 20.3 | decode_event 18.7 — the build pipeline's tape replay; sub-noise single |
| thin_entry_push | 16.4 | 96-B thin-memo entry pushes — memo machinery; sub-noise |
| hashbrown_map | 14.5 | fat-memo insert monomorphs; sub-noise |
| in-target lanes | ~39.3 | ALL sub-noise singles (`-0206/-0208/-0213` ×3 confirmed) |

(The ≥14 ns masses; the remainder to the 1,003.3 ns floor is tier-2 derived
role credit, unattributed in-target instruction classes, and <14 ns buckets —
all itemized in the `-0213` `role_reprice.txt`.)

**No priced ≥noise designed mechanism exists anywhere on the floor.** The
entire remaining sub-noise banked inventory — the `-0170` quartet ≈22.8 ns +
the protocol-memo-inserts fork ≈12.3 ns + assorted singles ≈3–5 ns — sums to
≈38–40 ns at IMPOSSIBLE 100% delivery, against a −5.4% ≈ 54 ns gap to the
margined ≤950.0 ns bar; at the campaign's honest 30–70% in-band delivery
precedent it is ≈ −1.3…−2.7%. **The margined bar is NOT reachable on the
current evidence.** (The `-0168` foresight — "with the corrected
coefficient, NO SINGLE IDENTIFIED LEVER CLOSES THE <1µs BAR" — is now
measured-final rather than model-predicted.)

This satisfies the layer-A option (c) trigger verbatim: the exhaustion
question goes to the director. Achieved state: corpus geomean **1,004.4 ns**
(raw sub-1µs readings banked: 994.4–1005.4), ≈494× cumulative from the
496 µs start, correctness untouched throughout (flips 0/2,189 on every
accepted fix).

**The engineer's ONE recommendation:** call the campaign at the honest
floor — run the #162 closing wave (the preserved-probe confirmation sweep +
book/handoff/contract re-baseline), recording that raw sub-1 µs was achieved
while the margined (+50 ns drift-proof) claim is out of designed reach; the
alternative (a final sub-noise BATCH worth ≈ −1.3…−2.7% that cannot reach
the bar either, with the `-0209/-0210/-0214` triple-refusal pattern as the
risk precedent) is NOT recommended. The call-off is the director's per the
#162 directive — this session STOPS at the surfaced question.

## Outputs banked in this dir

`decompose_content.py` → `decomposition_content.txt` + this file.
