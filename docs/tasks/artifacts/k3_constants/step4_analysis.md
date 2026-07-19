# K4 STEP-4 — the post-K4a re-profile + re-steer (`PGEN-RGX-0078-0151`)

Session #158, 2026-07-19. Method = STEP-3 exactly (`step3_analysis.md`): single-cell
corpus-mode loops on the K4a floor probe
(`preserved_probes/regex_perf_probe_k4a_fb7b20cf`, fat-LTO+mimalloc, canonical regex
`ee3a3cb6`), `/usr/bin/sample` @1 ms × 10 s per cell, tree-walk attribution
(`step2_treewalk.py`). Raw: `k4a_sample_{725,nest80,2880}_step4.txt` + custody
`k4a_step4_custody_*.jsonl`.

Floor custody re-validated in-run: `line_725` 714,625 ns (−2.7% vs banked 734,792),
`nest_80` 157,958 (−1.3%), `line_2880` 91,292 (−3.1%) — cool machine, floor intact.

## 0. The K4a kill — total on both target populations

- `line_725`: `rollback_to_labeled` cum **677 → 0** (the inlined fast path no longer
  even appears as a frame); `SmallVec` 43→30; `apply_delta` 0; `extract` 40 (0.5%).
  **The entire checkpoint/rollback protocol population is DEAD on the MAX cell.**
- `line_2880`: `SmallVec::clone` cum **616 → 37** (the depth≥2 checkpoint clones are
  gone; the residual 26 = the extract-side delta carriers, K4b); rollback is now only
  the genuine slow path (`rollback_to_labeled_slow` 440 cum = real fact-discard work).
- `nest_80`: chain ops ≈ 0 (18 cum, all extract-side).

## 1. `line_725` (the corpus MAX; 8,370 in-graph samples) — populations

| population | cum | share | note |
|---|---|---|---|
| parse spine (atom 755 / piece 553 / pattern 466 / alternative 357 / group 320 / lookaround 155 leaf) | ~2,800 | ~34% | |
| **build-value pass** (`to_shaped_value` 576 + `alloc_extend` 530 + `cascade_build_*` leaves ≈400) | **~1,500** | **~18%** | **concentrated: `cascade_build_value_group` = to_shaped_value 485 + arena 194 ≈ 679 = 8.1% — THE top addressable** (the shaped `-> {…}` object build per capture group; `elements[N].content.clone()` → `to_shaped_value` is the `-0123`-named pure emitter artifact) |
| memmove | 443 | 5.3% | mixed spine/build |
| memo `HashMap::insert` | ~400 | ~4.8% | the K3a-residual insert itself |
| allocator (mi_free 217 + mi_malloc_aligned 172) | ~390 | ~4.7% | |
| `apply_semantic_runtime_effect_directive` | 345 | 4.1% | real `@emit_fact` work (FactIndex 127 + SipHash 51 within) |
| checkpoint/rollback/extract/apply protocol | ~70 | **~0.8%** | **K4a's kill zone — was ~10% pre-K4a** |

## 2. `nest_80` (6,843 samples) — the nest family is now purely K1b + K5

| population | cum | share | note |
|---|---|---|---|
| **`extract_delta_since` (K1b)** | **1,452** | **21.2%** | share RISING (18.5%→21.2%) as everything else dies; 1,418 under recursive `parse_pattern` = the memo success-insert extraction, O(d²) record clones |
| parse spine (piece 925 / pattern 741 / alternative 664 / group 390 / atom 365 leaf) | ~3,100 | ~45% | K5 `check_cycle_id` territory |
| `apply_semantic_runtime_effect_directive` | 684 | 10.0% | incl. `rule_context_path` 411 = 6.0% (O(d) walk per fact) |
| allocator | ~520 | ~7.6% | |

## 3. `line_2880` (8,241 samples) — the K4b surface, now cleanly isolated

| population | cum | share | note |
|---|---|---|---|
| `extract_delta_since` | 647 | 7.9% | slow-leg extracts (memoized 217 / pattern 153 / group 121) — fact-record `to_vec` clones |
| `apply_delta` | 643 | 7.8% | winner re-insert through the FactIndex (SipHash 116 within) |
| `rollback_to_labeled_slow` | 440 | 5.3% | genuine fact-discard rollbacks (group 165 / memoized 108 / lookaround 80) |
| SipHash (fact-op hashing) | 266 | 3.2% | rollback 129 + apply_delta 116 — alive ONLY on this cell class |
| memmove | 794 | 9.6% | largely under extract/txn/apply |
| **Σ the delta slow-leg cluster (K4b)** | | **~22%** | of a 91 µs cell (7.8× below the MAX) |

## 4. The re-steer (corpus-MAX primary, one lever per slice)

1. **V1 STEP-0 — the BUILD-VALUE census on the MAX cell — NEXT.** The MAX cell's top
   addressable is now the build-value pass (~18%), concentrated in
   `cascade_build_value_group` (8.1%): the shaped `-> {…}` object construction pays
   `elements[N].content.clone()` → `to_shaped_value` (the `-0123`-named pure emitter
   artifact — the clone exists only because the builder reads a `ParseContent` it
   also owns) plus per-element arena `alloc_extend`. This population is OUTSIDE the
   `.5.j.2` exhaustion verdict (that priced the carrying-flatten lever); it needs its
   own STEP-0 census (population sizing per build class, the clone-elision candidate
   mechanism, oracle obligations) BEFORE any pricing — no number promised yet.
2. **K4b — the delta slow legs** (~22% of `line_2880`): fact-record clone traffic in
   extract, apply_delta's re-insert hashing, slow-rollback per-fact removes. Sequenced
   after the MAX lever (2880 = 91 µs ≪ 735 µs).
3. **K5 + K1b — the nest family** (extract 21.2% + spine ~45% of `nest_80`, 158 µs
   cells): unchanged order, after the MAX levers.
4. Residual MAX populations for later rounds: memo insert ~4.8%, allocator ~4.7%,
   memmove ~5.3%, effect-directive 4.1%.
