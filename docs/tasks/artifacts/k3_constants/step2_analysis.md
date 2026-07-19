# K3 STEP-2 — the post-K3b re-profile + re-steer (`PGEN-RGX-0078-0144`)

Session #157, 2026-07-19. All measurements on the K3b floor probe
(`preserved_probes/regex_perf_probe_k3b_2620abf8`, fat-LTO+mimalloc, canonical regex
`c49c43b3`), `/usr/bin/sample` @1 ms × 10 s per cell, single-cell corpus-mode loops.
Attribution = `step2_treewalk.py` over the raw call graphs (nearest enclosing
rule/runtime frame per sampled leaf, recursion de-duplicated).

Floor custody re-validated in-run (`k3b_step2_custody_*.jsonl`, `--samples 1000 --warmup 50`):

| cell | this run (min ns) | banked `-0142` (min ns) | drift |
|---|---|---|---|
| `line_725` | 873,584 | 886,125 | −1.4% (thermal-clean) |
| `nest_80` | 153,667 | 165,541 | −7.2% (faster-than-banked; cool idle machine — floor intact, recorded honestly) |

## 1. `line_725` (the corpus MAX, 3,511 B; 8,326 samples, ~8,215 in-process) — populations

| population | cum | share | attribution (tree-walk) |
|---|---|---|---|
| parse spine (`cascade_match_*` + `parse_pattern` leaves) | ~1,900 | ~23% | atom 549 / piece 408 / pattern 369 / alternative 268 / group 240 leaf |
| **`SmallVec::clone` (the K3b `ActiveChain` snapshot)** | **720** | **8.8%** | spread across EVERY tournament site + annotated entry: atom 144 / piece 105 / txn-wrapper 70 / alternative 52 / memoized 50 / group 36 / … — `checkpoint()` is inlined; the malloc is gone (K3b) but the inline 32-B copy remains, priced by CALL FREQUENCY |
| `rollback_to_labeled` | 526 | 6.4% | atom-closure 118 / txn-wrapper 79 / pattern 63 / lookaround 47 / backref 34 / group 33 / capture_open 27 — the capture-open-class subset (group+capture_open+lookaround+backref = 141) is only **1.7% of the cell** |
| **String-keyed annotation probes (`has_rule` 224 + `directives_for_rule` 261)** | **485** | **5.9%** | txn-wrapper 240 + lookbehind/named-group/lookahead validator paths — every rule entry pays `has_rule(rule_name)` (regex is NOT `is_empty`, so the whole-grammar fast path never triggers) |
| memo `HashMap::insert` (post-K3a: no rehash) | 440 | 5.4% | alternative 107 / pattern 99 / piece 68 — the insert itself, growth churn stays dead |
| `_platform_memmove` | 452 | 5.5% | mixed: effect-directive 76 / piece 75 / pattern 66 / txn-wrapper 58 |
| build pass (arena `alloc_extend` 413 + `to_shaped_value` 215 + `cascade_build_*` ~215) | ~840 | ~10% | the `.5.j.2`-exhausted class |
| allocator (`mi_free` 193 + `mi_malloc` 115 + madvise 92) | ~400 | ~4.9% | post-K3b residual |
| `extract_delta_since` | 44 | **0.5%** | **K1b is NOT a `line_725` lever** (memoized 28 / pattern 8) |
| `apply_semantic_runtime_effect_directive` | 268 | 3.3% | the real `@emit_fact` work at capture opens |

## 2. `nest_80` (161 B, depth-80; 7,000 samples) — populations

| population | cum | share | attribution |
|---|---|---|---|
| parse spine (piece 1,007 + alternative 742 + pattern 671 leaf) | ~2,420 | ~35% | the K5 `check_cycle_id` O(d)-scan territory (inlined under fat LTO) |
| **`extract_delta_since`** | **1,340** | **19.1%** | **1,284 under recursive `parse_pattern` frames = the K1b memoized-call success-insert extraction with `memoized_call` INLINED** (raw stacks show `extract_delta_since` directly under `parse_pattern` at three call offsets; `to_vec` 1,113 + `Vec::clone` 171 + memmove 453 of it). Mechanism: each nesting level's `pattern` memo-stashes its O(d) inner fact set ⇒ O(d²) record clones ≈ 3,200 @ d=80 |
| `apply_semantic_runtime_effect_directive` | 647 | 9.2% | incl. `rule_context_path` 379 — the emit path walks the O(d) rule-context stack per emitted fact |
| chain/scope clones + drops (SmallVec 162 spill + misc Vec) | ~500 | ~7% | the K3b O(d)-spill residual (depth 80 > inline 8) |
| allocator (mi_free 325 + mi_malloc 187 + generic 39) | ~550 | ~7.9% | |
| `rollback_to_labeled` | 59 | 0.8% | the K1c kill still confirmed |

## 3. The re-steer (corpus-MAX primary, one lever per slice)

1. **K3c — rule-id-indexed annotation tables — NEXT EMISSION.** Priced at **5.9% cum on
   `line_725`** (+ the wrapper's probe-branch constant) and it fires on EVERY entry of EVERY
   pattern ⇒ a bench-geomean lever too. Design as recorded in `-0140` + sharpened here:
   `CompiledSemanticRuntimeAnnotations` gains dense per-rule-id slots (built once at
   construction from the existing String-keyed maps); `with_semantic_runtime_rule_transaction`
   takes the numeric rule id alongside the `&'static str` name (the C1 `rule_id_stack`
   precedent — every emitted call site knows its rule at codegen time); `has_rule`/
   `directives_for_rule`/`pre_predicates_for_rule` become O(1) indexed loads on the hot path.
   Predicted `line_725` −4…−7%, bench −3…−8%; falsification < −2% on 725 ⇒ stop + re-profile.
2. **K3d — checkpoint-lite `ActiveChain` snapshot (NEW lever, named + priced here, design
   AFTER K3c):** the post-K3b top single population on 725 = the inline chain copy per
   `checkpoint()` (**8.8%**) + the nest-family O(d) spill clones. Candidate mechanism: the
   chain mutates ONLY by push/truncate (scope enter/exit), so a checkpoint could record
   `(len, epoch)` and rollback truncate — zero copy on checkpoint; extraction materializes
   lazily. Audit obligations: chain mutation discipline (no in-place writes below len),
   delta-carrier semantics (`final_active_chain`). Price AFTER the K3c re-profile.
3. **K5 (`check_cycle_id` seen-set) + K1b (memo success-insert extraction) = the NEST-family
   program, ordered after the MAX levers:** nest cells sit at ~154 µs ≪ the 874 µs MAX, and
   K1b is 19.1% of `nest_80` but 0.5% of `line_725`. K1b re-priced (the `-0142` NEXT pointer's
   ask): the `:8403` success-insert extraction is the top nest population — worth ~29 µs on
   `nest_80` — but does not move the corpus MAX. Same for K5 (~35% spine share on nest).
4. **The `(`/`(?` two-byte-dispatch grammar peel — DOWN-PRICED:** its addressable population
   on `line_725` (the capture-open-class rollback subset) measures **1.7%** — below the K3c
   and K3d levers. Parked behind them.

Land-gate structure unchanged (corpus-MAX primary + bench ±2% guard + full battery;
any verdict flip = stop-and-revert).
