# K3 STEP-0 — the post-K1c re-profile + re-priced constant program (`PGEN-RGX-0078-0140`)

Session #155, 2026-07-18. All measurements on the `-0139` release probe (`d9d3d611`,
fat-LTO+mimalloc, canonical regex `e10855b6`), `/usr/bin/sample` @1 ms × 10 s per cell,
single-cell corpus-mode loops. Floor custody re-validated in-run:

| cell | this run (min ns) | banked `-0139` (min ns) | drift |
|---|---|---|---|
| `line_725` | 1,094,959 | 1,101,875 | −0.6% (thermal-clean) |
| `nest_80` | 168,292 | 170,958 | −1.6% (thermal-clean) |

## 1. The profile SHIFTED post-K1c — the `-0135` K3 premise is REFUTED as priority

The `-0135` K3 pricing ("FxHash for `FactIndex` + pre-normalized kinds + borrowed keys —
hashing alone ≈30% of the nest profile") was measured on the PRE-K1/K1c artifacts, when the
tournament delta protocol pushed O(d²) fact-ops through `FactIndex`. K1 (`-0137`) + K1c
(`-0139`) killed that traffic at the source. On the current floor:

- **SipHash: 0 samples in BOTH profiles** (`grep -c SipHash sample_725.txt sample_nest80.txt`
  → 0/0; it was 1,602 leaf samples on nest_80 in `-0135`). The hash-function lever is dead.
- `rollback_to_labeled` on nest_80: **51 samples** (was the dominant cluster) — the fast path
  + the K1c kill did their job.

## 2. `line_725` (the corpus MAX, 3,511 B, ~3,885 in-process samples) — the NEW populations

Top-of-stack leaves (full table in `sample_725.txt`):

| population | samples | share | attribution (tree-walk) |
|---|---|---|---|
| `_platform_memmove` | 692 leaf / 890 cum | 22.9% cum | mixed: table growth + Vec growth + arena |
| `mi_malloc_aligned` + `mi_free` | 326+417 leaf, 931 cum malloc | ~24% cum | mixed: checkpoint chain clones, memo growth, Vec |
| `hashbrown reserve_rehash` (2 monos) | 415 leaf / **520 cum** | **13.4% cum** | via `HashMap::insert` from `cascade_match_piece`/`alternative`/`parse_pattern` + `memoized_call` + failure-cache inserts (`parse_python_named_backreference`, `parse_lookbehind_neg_open` closures) = the memo containers growing 256→32K buckets |
| `HashMap::insert` | 262 leaf / 649 cum | 16.7% cum | memo success/failure inserts from the cascade frames |
| `rollback_to_labeled` | 412 leaf / 520 cum | 13.4% cum | `with_semantic_runtime_rule_transaction` 128 + `cascade_match_atom` closure 87 + `lookaround` 61 + `group` 37 + `backreference` 22 — slow-path rollbacks discarding real speculative facts (capture-open attempts at `(?:` sites) + fast-path constant |
| `directives_for_rule` | 188 leaf / 309 cum | 8.0% cum | `with_semantic_runtime_rule_transaction` per-entry String-keyed FxHashMap probes (`has_rule` + `directives_for_rule` + `pre_predicates_for_rule`) |
| parse work (`cascade_match_*`, `parse_pattern`) | ~1,335 cum | ~34% | the spine itself |

## 3. `nest_80` residual (168 µs) — for the record (K5 territory, not K3)

Top leaves: `cascade_match_piece` 951, `_platform_memmove` 956, `parse_pattern` 659,
`cascade_match_alternative` 619, `Vec::clone` 203 + `to_vec` 177 +
`drop_in_place<Vec<UnifiedSemanticProperty>>` 147, `rule_context_path` 111,
`rollback_to_labeled` 51. The spine recursion dominates — consistent with the named K5 term
(`check_cycle_id` `mod.rs:1193` O(d) stack scan per entry, inlined into the cascade frames
under fat LTO) + O(d) `active_chain` clones per checkpoint (K3b kills the alloc pair; the
O(d) copy remains until/unless an undo-log design is priced).

## 4. The memo-entry census (the K3a sizing fact) — `memo_census.txt`

`PGEN_REPORT_MEMO_STATS=1` on the release probe (the stats emitter is generated-parser code,
so the floor probe honors it directly — no debug-probe rebuild needed):

- `line_725`: **19,427 total entries** (8,382 success thin-memo + 11,045 cached failures)
  vs today's pre-size `min(len+1, 256)` = 256 ⇒ ~7 growth doublings, each a full rehash
  (hashbrown), total rehash work ≈ 2×final ⇒ exactly the 13.4% `reserve_rehash` population.
- Bounds across the worst-cell class (7 exemplars): success ≤ 4.51 entries/B (nest_80),
  failures ≤ 3.15 entries/B (line_725/6538). Tainted populations are tiny everywhere
  (≤ 54 entries) — the tainted maps do NOT need scaling.

## 5. The re-priced K3 program (design; one lever per slice, corpus-MAX primary)

- **K3a — input-proportional memo container pre-size** (emitter constructor,
  `ast_based_generator.rs:1435/:1537/:1541` + the deriv-tape hints `:1451-1452`):
  `thin_memo` and `memo_fail` pre-size `min(6·(len+1), 32768)` elements (K=6 covers every
  observed cell per-map; the 32K-element cap bounds construction memory for giant inputs —
  at ~72 B/entry ≈ 2.3 MB worst-case pre-allocation); `memo_fail_tainted` UNCHANGED
  (`min(len+1,256)`, tainted ≤54 observed); deriv tape `max(64, min(4·(len+1), 32768))`
  events / same-shape boundary. The protocol `memo` map's fixed 256 (`:1530`) is
  DELIBERATELY out of scope — it is the SV-hot map and this slice carries no SV
  measurement (recorded as K3a-residual for the per-family campaigns).
  Predicted: `line_725` −10…−18% (rehash dies + insert cheapens); the 73–91 µs cluster
  −3…−8%; bench-neutral expected (small patterns: 6·(len+1) stays ≤ today's 256-bucket
  class; the ±2% band + corpus A/B guard it).
- **K3b — `active_chain` SmallVec** (`semantic_runtime.rs:2172/:1839` + checkpoint
  `:2737`): `Vec<ScopeId>` → `SmallVec<[ScopeId; 8]>` (dep already present from `-0118` C3).
  The chain starts `vec![ScopeId::ROOT]` — NEVER empty — so today EVERY `checkpoint()`
  (every annotated-rule entry + every tournament site) pays a real malloc+memcpy+free.
  Inline-8 kills the allocator round-trip for scope depth ≤8 (typical corpus; deep nests
  spill but keep correctness). Predicted: a few % on entry-dense cells + a small geomean
  move.
- **K3c — rule-id-indexed annotation lookup** (kill the per-entry String probes):
  dense per-rule-id slot table in `CompiledSemanticRuntimeAnnotations` + the wrapper takes
  the rule id (the C1 `rule_id_stack` precedent). Bigger change (emitter + runtime + regen);
  **price AFTER the K3a/K3b re-profile**.
- **K5 unchanged** (the `check_cycle_id` seen-set/monotone design) — the nest-family
  residual; next after K3a/b per the recorded order.

Falsification bounds (stated before building): K3a < −4% on `line_725` ⇒ the rehash
attribution was wrong — stop and re-profile; any verdict flip anywhere ⇒ stop-and-revert.
