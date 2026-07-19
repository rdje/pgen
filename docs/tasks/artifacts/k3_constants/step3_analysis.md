# K3 STEP-3 — the post-K3d re-profile + re-steer (`PGEN-RGX-0078-0148`)

Session #158, 2026-07-19. All measurements on the K3d floor probe
(`preserved_probes/regex_perf_probe_k3d_5f954220`, fat-LTO+mimalloc, canonical regex
`ee3a3cb6`), `/usr/bin/sample` @1 ms × 10 s per cell, single-cell corpus-mode loops
(the STEP-2 method exactly). Attribution = `step2_treewalk.py` over the raw call
graphs. NEW this step: the sample populations are cross-pinned by the
**store-counter dumps** (`--dump-rule-outcome-counts-json` on a fresh HEAD-coherent
debug `parseability_probe`, guard-built exit 0 — counters run the PROTOCOL graph,
recorded caveat below).

Raw evidence: `k3d_sample_725_step3.txt` / `k3d_sample_nest80_step3.txt` /
`k3d_sample_2880_step3.txt` + custody `k3d_step3_custody_{725,nest80,2880}.jsonl`.

Floor custody re-validated in-run (`--samples 1000 --warmup 50`):

| cell | this run (min ns) | banked `-0147` (min ns) | drift |
|---|---|---|---|
| `line_725` | 795,292 | 806,042 | −1.3% (thermal-clean) |
| `nest_80` | 156,750 | 165,708 | −5.4% (faster-than-banked, cool idle machine — floor intact, recorded honestly) |
| `line_2880` | 107,125 | 109,250 | −1.9% (thermal-clean) |

## 0. The prior-lever kill confirmations (in-profile, this floor)

- **K3d CONFIRMED on the MAX cell:** `SmallVec::clone` cum on `line_725` **720 → 43**
  (8.8% → 0.5%); on `nest_80` **→ 12** (the nest family opens no scopes, so the
  depth-1 sentinel fires at every checkpoint — the STEP-2 ~7% spill population is gone).
- **K3c CONFIRMED everywhere:** `has_rule`/`directives_for_rule` String-probe samples =
  **0 / 0** on `line_725` and `nest_80` (10 residual frames on `line_2880`, the
  named-group validator path — trivia).
- **K3a holds:** zero `reserve_rehash` populations; memo cost is the insert itself.
- **K1c holds:** `nest_80` rollbacks all `unchanged` (322 of 322), `rollback_to_labeled`
  cum 73 = 1.0%.

## 1. `line_725` (the corpus MAX, 3,511 B; 8,384 in-graph samples) — populations

| population | cum | share | attribution (tree-walk) |
|---|---|---|---|
| parse spine (`cascade_match_*` + `parse_*` leaves) | ~2,660 | ~32% | atom 649 / piece 578 / pattern 383 / alternative 336 / group 246 / backref 147 / capture_open 123 / escape_unit 98 / anchor 96 leaf |
| **`rollback_to_labeled` — now the #1 runtime population** | **677** | **8.1%** | atom-closure 183 / lookaround 85 / txn-wrapper 71 / group 60 / pattern 53 / alternative 40 / piece 30 / backref 29 / memoized 26 / capture_open 25 / lookbehinds 28. **The capture-open class subset (group+capture_open+lookaround+backref+lookbehinds) = 227 = 2.7%** |
| build pass (arena `alloc_extend` 519 + `to_shaped_value` 515 + `cascade_build_*` leaves) | ~1,250 | ~15% | **concentrated: `cascade_build_value_group` owns to_shaped_value 444 + alloc_extend 197 ≈ 641 = 7.7%** (the ~50 shaped `-> {…}` capture-group values per parse) |
| `_platform_memmove` | 439 | 5.2% | piece 103 / effect-directive 77 / pattern 62 / txn-wrapper 45 / alternative 44 |
| memo `HashMap::insert` (leaf) | 370 | 4.4% | the K3a-residual insert itself |
| allocator (mi_free 210 + mi_malloc_aligned 142) | ~352 | ~4.2% | |
| `apply_semantic_runtime_effect_directive` | 325 | 3.9% | the real `@emit_fact` work (FactIndex 122 + SipHash 46 of it) |
| `SmallVec::clone` (chain snapshots, depth ≥ 2 only) | 43 | 0.5% | **the K3d kill confirmed** |
| `extract_delta_since` | 46 | 0.5% | **K1b is NOT a MAX lever (3rd consecutive confirmation)** |

### 1.1 ⭐ The rollback population DECODED by counters — the `-0140` reading CORRECTED

Store counters for the `line_725` parse (protocol graph):

```
total_entries=26447  committed=10819  facts_emitted=271  facts_rolled_back=0
rollbacks=20490  rollbacks_unchanged=20490  (100%)
rollbacks_tournament=272  rollbacks_tournament_unchanged=272  scopes_opened=0
```

**Every single one of the 20,490 rollbacks is the UNCHANGED case; zero facts are ever
discarded on this accepting parse.** The `-0140` narrative ("slow-path speculative-fact
discards") is REFUTED for this cell: the O(1) epoch fast path (P3c-i, `.5.i.5`,
`semantic_runtime.rs:3120`) already short-circuits every one of them. The 8.1% is
therefore **pure per-call transaction-protocol constant × ~20k calls/parse**:

- the non-inlined call + the **by-value `SemanticRuntimeCheckpoint` argument** —
  5 usizes + `write_epoch` + the 40-B `ActiveChain` SmallVec ≈ **88 B constructed,
  copied, and dropped per attempt** (the struct is deliberately NOT `Copy` — the
  comment at `semantic_runtime.rs:1937` says `Copy` was removed exactly to
  accommodate the chain snapshot);
- the fast-path compares + 2–4 counter RMWs + the SmallVec drop glue.

Arithmetic pin: 8.1% of 795 µs ≈ 64 µs / ~20.5k calls ≈ **~3 ns/call** — exactly a
call+copy+compare constant, corroborating the classification. The checkpoint-SIDE
construction (≥ rollback frequency; successes never roll back) is inlined into the
callers and hides inside the spine/memmove shares — the same lever covers it.

*Caveat (recorded):* the counter dump runs the PROTOCOL graph; the sampled profile is
the BARE cascade graph. Fact traffic (emitted/rolled-back) is graph-invariant for the
same accepted parse; absolute rollback counts differ (cascade plain-restores
non-effect sites), so ~20k is the protocol-graph figure and the bare-graph frequency
is the same order (the 677-sample share is the bare-graph ground truth).

## 2. `nest_80` (161 B, depth-80; 7,004 in-graph samples) — populations

| population | cum | share | attribution |
|---|---|---|---|
| parse spine (piece 991 + pattern 744 + alternative 733 + group 422 + atom 368 leaf) | ~3,260 | ~46% | the K5 `check_cycle_id` O(d)-scan territory (inlined under fat LTO); share RISES as other populations die |
| **`extract_delta_since`** | **1,298** | **18.5%** | **K1b UNCHANGED (STEP-2: 19.1%)** — 1,263 under recursive `parse_pattern` = the memoized-call success-insert extraction (`to_vec` 1,090 + memmove 476 within); O(d²) record clones |
| `apply_semantic_runtime_effect_directive` | 750 | 10.7% | incl. `rule_context_path` 461 = 6.6% — the emit path still walks the O(d) rule-context stack per fact |
| allocator (mi_free 310 + mi_malloc 139) | ~449 | ~6.4% | |
| `FactIndex` | 151 | 2.2% | under the effect directive |
| `rollback_to_labeled` | 73 | 1.0% | K1c kill holds (322/322 unchanged) |
| `SmallVec` (chain ops) | 12 | 0.2% | **K3d killed the nest chain-snapshot population entirely** |

Counters: `total_entries=810, facts_emitted=80 (net-only), facts_rolled_back=0,
rollbacks=322 all unchanged` — the `-0139` counter pins hold exactly.

## 3. `line_2880` (450 B; 8,241 in-graph samples) — the DELTA-PROTOCOL cell

Counters first — this cell is structurally DIFFERENT:

```
total_entries=2683  committed=950  memo_hits=117  facts_emitted=97  facts_rolled_back=83
rollbacks=2296  rollbacks_unchanged=2253  rollbacks_nonempty_chain=2234  (97%!)
rollbacks_tournament=227  rollbacks_tournament_unchanged=184  scopes_opened=1
```

**One long-lived scope spans essentially the whole parse** (`scopes_opened=1` yet
97% of checkpoints see a non-root chain) ⇒ the K3d depth-1 sentinel almost never
fires here ⇒ every checkpoint still clones the depth-2 chain. AND the slow legs are
real on this cell: 83 of 97 emitted facts get rolled back (speculative fact traffic),
43 slow-path rollbacks, 117 memo hits.

| population | cum | share | attribution |
|---|---|---|---|
| **`rollback_to_labeled`** | **745** | **9.0%** | atom 161 / group 157 / memoized 116 / lookaround 79 / backref 40 / txn 38 — leaf risen 347→401→477 across the K3c→K3d→now floors |
| **`SmallVec::clone` (depth-2 chain snapshots + delta carriers)** | **616** | **7.5%** | spread over EVERY site: atom 108 / piece 63 / txn 60 / class_atom 45 / memoized 38 / backref 32 / group 27 / … — the K3d-recorded residual, now the top single leaf after memmove |
| `extract_delta_since` | 654 | 7.9% | memoized 213 / pattern 134 / group 131 / atom 83 / backref 42 / lookaround 39 (`to_vec` 529 + memmove 183 within) |
| `apply_delta` (winner re-apply) | 507 | 6.2% | group 186 / memoized 106 / atom 94 / lookaround 75 / backref 45 (FactIndex 313 + memmove 112 within) |
| `FactIndex` (insert/remove hashing) | 436 | 5.3% | apply_delta 313 / effect 67 |
| memmove | 684 | 8.3% | extract 183 / txn 146 / apply_delta 112 |
| SipHash | 244 | 3.0% | rollback 112 / apply_delta 104 — fact-op hashing is ALIVE on this cell (0 on 725/nest) |
| allocator (mi_free 343 + mi_malloc 206) | ~549 | ~6.7% | |

**Σ the C3-B delta-protocol cluster (rollback + clones + extract + apply_delta +
FactIndex/SipHash under them) ≈ 33% of the cell** — `line_2880` (107 µs, #4 worst)
is THE scope-open transactional exemplar: the cost is the depth≥2 checkpoint clone ×
frequency PLUS genuine per-fact delta traffic at tournament sites.

## 4. The re-steer (corpus-MAX primary, one lever per slice)

1. **K4a — SLIM `Copy` CHECKPOINT (chain undo-log / epoch snapshot) — NEXT (design
   slice first, the house discipline).** The named populations it kills: `line_725`
   rollback constant **8.1%** + a checkpoint-construction share hidden in the spine;
   `line_2880` chain-clone **7.5%** + its rollback constant share of 9.0%; the last
   nest chain ops. Candidate mechanism (design slice to fix): give the chain a
   monotone **mutation epoch** (or a WAM-style undo trail); `checkpoint()` records
   `(chain_len, chain_epoch)` instead of cloning content ⇒ `SemanticRuntimeCheckpoint`
   loses its only non-`Copy` field, becomes `Copy` (7 words, register-class, no Drop
   glue); the rollback fast path becomes inline-able at call sites (epoch compare +
   return). Restore correctness rests on the push/truncate mutation discipline
   (`-0146` tool-verified: exactly 4 mutation sites) — the two wholesale-restore
   sites (`apply_delta`, labeled restore) are the audit obligations (trail them or
   epoch-fallback). **Predicted: `line_725` −5…−9% (falsification < −2.5%),
   `line_2880` −8…−14%, bench −3…−7% (every entry pays the pair), ladder
   non-regression.** Any verdict flip = stop-and-revert.
2. **K4b — the delta-carrier + slow-leg program on the scope-open class** (extract
   `final_*` clones / apply_delta re-insert / per-fact hashing — the remaining ~20%
   of `line_2880`), priced AFTER the K4a re-profile.
3. **K5 (`check_cycle_id` seen-set) + K1b (memo success-insert extraction) = the
   NEST-family program, unchanged** (18.5% + ~46% spine on nest_80; nest cells
   ~157 µs ≪ the 795 µs MAX — still after the MAX levers).
4. **NEW frontier population NAMED (post-K4): the `line_725` build-value pass ~15%,
   concentrated in `cascade_build_value_group` ≈ 7.7%** (shaped `-> {…}` object
   construction: `to_shaped_value` 444 + arena 197). The `.5.j.2` "build-pass
   exhausted at ≈1%" verdict priced the carrying-flatten lever, NOT this shaped-value
   population — it needs its own census before any pricing (recorded honestly; no
   number promised).
5. The `(`/`(?` two-byte peel stays PARKED (the capture-open rollback subset is 2.7%
   of 725, and those rollbacks are fast-path constants K4a already covers).

## 5. Method notes

- Sample totals: 8,384 / 7,004 / 8,241 main-thread in-graph samples for
  725 / nest80 / 2880; shares are against the in-graph total (loop-harness frames
  included, ≤2% of samples — consistent with STEP-2's convention).
- The debug `parseability_probe` was rebuilt this session (post-incident first
  rebuild; guard `--budget-mb 16384`, exit 0) against the on-disk canonical
  `ee3a3cb6` artifacts at HEAD `32f051f9` — counter semantics are K3d-vintage
  (`rollbacks_nonempty_chain` = "non-root chain at checkpoint", exactly what §3 uses).
- The `sample`-attach trap hit once and was fixed in-chain: backgrounding a
  `VAR=… && probe … &` compound backgrounds the WHOLE compound as a bash subshell —
  `$!` then names bash, not the probe (the first 725 sample file captured bash and
  was re-recorded). Rule: background the probe command directly.
