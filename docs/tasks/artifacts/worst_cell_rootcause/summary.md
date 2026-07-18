# RGX-0078.5.j.4 STEP-0 — the worst-cell superlinearity ROOT CAUSE (session #153)

Measurement vehicle: the `-0132` floor-validated release `regex_perf_probe` (`7fd0a31b`,
fat-LTO+mimalloc, corpus mode — the closure-bench unit, BARE parse path) for all timings;
the HEAD-coherent debug `parseability_probe` (rebuilt this session against the `-0129`
artifacts, guard peak 10329 MB) for `--dump-rule-entry-counts-json` /
`--dump-rule-outcome-counts-json`; `/usr/bin/sample` @1ms×10s leaf categorization
(the re-profile method) on `nest_80`.

## 1. The depth ladder (release bare path — `ladder_times.jsonl`)

- `flat_N` (`"x"*N`): LINEAR, ~165 ns/B — the length control.
- `nest_D` (`"("*D+"x"+")"*D`): QUADRATIC — ns/B grows linearly with depth
  (1,097 ns/B @ d=1 → 13,218 ns/B @ d=80); doubling depth ≈ 3.7× time at large d.
- `nest_80` (161 B, NO backref) = 2,128,042 ns ≈ `orig_4674` (164 B, `\80`) = 2,120,833 ns
  ⇒ **the backreference is irrelevant; pure capturing-group nesting is the whole 4674 mechanism**
  (`nestbr_D` ≈ `nest_D` at every rung).
- `orig_6538` 92,042 ns vs `6538_noquant` 91,375 ns ⇒ **the `{26}` counted quantifier is
  irrelevant** to the 6538 cost.
- `look_D`/`lookstar_D`: same depth-linear per-byte growth, much smaller coefficient.

## 2. The discriminator: rule entries are LINEAR (protocol-graph dumps)

`total_entries` at nest d=10/20/40/80 = 132/252/492/972 (clean 2× per doubling; ~12
entries/level; top-rule table flat at 1–2 entries/depth-unit). **No structural re-parse,
no backtracking blowup.** Quadratic time × linear entries ⇒ per-entry cost grows with depth.

## 3. The mechanism: C3-B tournament delta traffic is O(inner fact-store) per boundary

`store_counters` at nest d=10/20/40/80:

| d | facts_emitted | facts_rolled_back | rollbacks (slow) | tournament (slow) |
|---|---|---|---|---|
| 10 | 240 | 230 | 97 (49) | 33 (31) |
| 20 | 880 | 860 | 187 (94) | 63 (61) |
| 40 | 3,360 | 3,320 | 367 (161) | 123 (121) |
| 80 | 13,120 | 13,040 | 727 (321) | 243 (241) |

`facts_emitted ≈ 2d²` with only d net facts surviving. Each nesting level's ~3 Or-tournament
sites (of the spine chain pattern→alternation→…→group→capturing_group) run the full C3-B
cycle over the ENTIRE inner fact set (k facts at level k): per successful branch
`extract_delta_since` (clones the k `SemanticFactRecord`s + the FULL `active_chain` +
`scopes` — semantic_runtime.rs:2765, the unconditional `final_*` clones at :2824-2825),
`rollback_to_labeled` slow path (k× `FactIndex::remove`, each `to_ascii_lowercase` alloc +
`FactNameKey` String clone + 2 SipHash string hashes — :1980), then winner `apply_delta`
(k× `FactIndex::insert` through the same allocating/hashing path, counted as re-emissions —
:2834/2865). Σ over levels = O(d²) fact-ops ≈ 39k heavy ops at d=80 ≈ ~53 ns/op.

The 1 µs-bar relevance: every `(` in ANY pattern emits `regex_capture_group` facts
(grammars/regex.ebnf:1459/1491/1493 — the backreference/conditional validation surface),
so the mechanism prices EVERY group-bearing pattern, superlinearly with nesting depth.

## 4. Profile confirmation (sample_nest80.txt, leaf top-of-stack)

Semantic-store cluster ≈ 70%+ of leaf samples: SipHash `Hasher::write` 1,128 +
`hash_one` 474 (the FactIndex maps still use std's DEFAULT SipHash — the FxHash slice
never reached `semantic_runtime`), `memmove` 892 (fact-record/chain clones),
`FactIndex::insert` 660, `rollback_to_labeled` 372, `memcmp` 291, mi_malloc/mi_free 518,
fact/property Vec clones+drops ~340, `apply_delta` 92. Actual parse work
(`cascade_match_*` + `parse_pattern`) ≈ 350 samples ≈ 5%.

## 5. The four-class anchor table (why each worst cell is slow)

| class | entries | ns/entry | fact re-emissions | slow rollbacks | driver |
|---|---|---|---|---|---|
| `flat_160` | 1,125 (7.0/B) | 23.0 | 0 | 0 | fused-path baseline |
| `lookstar_8` | 452 (9.2/B) | 94.7 | 0 | 33 | lookaround scope opens (8) → epoch-bumped transactions |
| `line_6538` | 1,020 (24.3/B) | 90.2 | 134 | 95 | entry-DENSE (24/B) × transactional constant |
| `nest_80` | 972 (6.0/B) | 2,189 | 13,040 | 321 | the quadratic fact-delta protocol |

Two worst-cell SUB-CLASSES: **(a) depth-superlinear** (4674 + the nest family — killed by
the K1 protocol fix below); **(b) entry-dense transactional-constant** (6538, the 73–91 µs
cluster, the 725 giant at 457 ns/B — killed by the per-entry constant program, incl. K3).

## 6. The kill design (named candidates, priced by population)

- **K1 — WINNER-IN-PLACE TOURNAMENT COMMIT (lazy C3-B branch cleanup; the structural kill).**
  Defer a successful branch's extract+rollback until the NEXT branch attempt actually starts;
  at selection, if the winner is the branch whose effects are live, COMMIT IN PLACE (drop the
  checkpoint — no extract, no rollback, no re-apply). C3-B semantics preserved: a losing live
  branch is rolled back at next-attempt/selection exactly as today; multi-successful-branch
  tournaments keep the stored-delta path. With first-byte dispatch pruning most sites to ONE
  attempted branch, the nest chain's 2d² re-emission traffic → ~0. Predicted on nest_80:
  −85…−95% (2.13 ms → ~100–250 µs); the SAME mechanism kills line_4674. Obligations:
  write-epoch semantics audit (fewer bumps = more replayable memo entries — sound iff state
  identical; needs the MEMO-STORE-SOUNDNESS.2 review), interpreter mirror (the equivalence
  gate + semantic suite are the oracles), emitted-code restructure in the tournament protocol.
- **K3 — fact-op constants (additive, general):** FxHash for `FactIndex` maps; pre-normalized
  lowercase kind at emit (kill per-op `to_ascii_lowercase` allocs); borrowed lookup keys
  (kill the per-op `FactNameKey` String clone). ~30% of the nest_80 profile is hashing alone.
- **K5 — the NEXT superlinear term (post-K1 re-measure):** `check_cycle_id`
  (ast_pipeline/mod.rs:1193) linearly scans the recursion stack per rule entry (O(d) frames ×
  O(d) entries with ~8 frames/level). Expect it to surface once K1 lands; fix class = seen-set
  keyed (rule_id, pos). Thin-memo tape-segment store sizes are the other candidate term.

Slice order (one lever at a time, measured): K1 → re-measure ladder+corpus → K3 → K5 as
re-steered. Land gate unchanged (fat-LTO alternated 5×2000, bar −2.0%, byte-identical floor).
