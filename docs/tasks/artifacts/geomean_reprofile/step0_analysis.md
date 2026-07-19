# Post-C1 geomean-weighted re-profile + re-steer (`PGEN-RGX-0078-0162`)

Session #163, 2026-07-19. Toolbox-first, measurement + docs only — the release floor
is byte-untouched. Steering per the `-0161` director directive: the campaign
CALL-OFF trigger is **corpus GEOMEAN < 1 µs** (now 1,263.4 ns ⇒ ≈ −21% needed);
corpus MAX is SETTLED at 483,583 ns. This re-profile therefore weights the
**geomean population** (the p50-class cells), not the MAX cell.

## Method + custody (all proven in-run)

- Release floor probe: `rust/target/release/regex_perf_probe` **byte-identical to
  `preserved_probes/regex_perf_probe_c1_1d3fa0ee`** (sha256 prefix `1d3fa0ee` both).
  Artifact custody: `generated/regex_parser.rs` = canonical **`cb322b75`**.
- Floorval custody best-of-3 (`--samples 2000 --warmup 200`, `caffeinate -ims`):
  geomeans 1,914.9 / 1,900.0 / 1,810.5 ns = **−1.16% / −1.93% / −6.55%** vs the
  banked 1,937.4 — machine slightly fast today, no slow-side breach; floor intact
  (`floorval_r{1,2,3}.txt`).
- Geomean-band workloads (built from the banked `../k4b_delta/c1_ab/corpus_cand.jsonl`
  per-cell distribution; the 11 cells ≥ 20 µs = 0.8% log-share excluded by design):
  `band_sub1us` 939 cells / `band_1to2p5` 751 / `band_2p5to20` 488 = 2,178 of 2,189.
  Fresh corpus-mode sweeps: **ZERO verdict flips across all 2,178 cells** vs the
  banked floor run (per-cell `expected_parse`/`actual_parse` identity).
- Attribution: `/usr/bin/sample` @1 ms × 14 s per band on the release probe
  (`sample_band_*.txt`), self/cum tree aggregation (`selfcum_tables.txt`).
- Counters: debug `parseability_probe` **guard-rebuilt this session** (the prior
  binary's 19:43 mtime predated the artifact's 20:18 write = the `-0157`
  ambiguous-vintage class ⇒ re-derived, not trusted; guard exit 0, peak 10,012 MB,
  new mtime 21:09 > artifact) — `--dump-rule-outcome-counts-json` on 8 quantile
  exemplars (`exemplars.json`, `outcome_*.json`) + a stratified 60-cell census
  (20/band, evenly spaced in-band; `strat60_cells.jsonl`, `strat60_census.jsonl`,
  `strat60_full_census.json`).

## 1. The geomean population's structure (banked floor data)

Log-share by per-cell min_ns band (2,189 cells, geomean 1,263.4 ns):

| band | n | % cells | log-share |
|---|---|---|---|
| < 600 ns | 571 | 26.1% | 22.4% |
| 600–1,000 | 368 | 16.8% | 15.6% |
| 1,000–2,500 | 751 | 34.3% | 35.3% |
| 2,500–5,000 | 348 | 15.9% | 18.1% |
| 5,000–20,000 | 140 | 6.4% | 7.9% |
| ≥ 20,000 | 11 | 0.5% | 0.8% |

Two structural facts that steer everything:

1. **Per-parse fixed overhead is ONLY ≈ 171 ns** — the fastest 0–2-byte cells run
   166–167 ns; the ≤25 B/<5 µs least-squares fit gives `min_ns ≈ 171 + 111·bytes`.
   Uniform-cut arithmetic on the real distribution: killing ALL fixed cost lands
   the geomean ≈ 1,050 ns ⇒ **a fixed-cost program alone cannot cross the bar**
   (a uniform −200 ns/cell would; only ~171 exist).
2. **The marginal per-byte cost ≈ 111–125 ns/B even for plain literal chars**
   (`abc` 458 ns → `abcd` 583 ns = +125 ns/char). The geomean program must cut
   per-byte parse work, i.e. entries/byte × per-entry constant.

## 2. Attribution: the three band profiles agree (self-time, top populations)

| population (self%) | sub-1µs | 1–2.5µs | 2.5–20µs |
|---|---|---|---|
| allocator+memory traffic (mi_* malloc/free, memmove, memset, Vec `finish_grow`, arena `alloc_extend`) | ≈ 31% | ≈ 27% | ≈ 27% |
| spine dispatch self (`piece` + `atom` + `parse_pattern` + `alternative` + `memoized_call` + `entry_concatenation`) | ≈ 25% | ≈ 22% | ≈ 24% |
| memo insert (`hashbrown::insert`) | 3.8% | 3.8% | 3.6% |
| build-value (`to_shaped_value` + `cascade_build_*`) | ≈ 5% | ≈ 6–10% cum | ≈ 7–10% cum |
| per-parse teardown (parser/arena/memo-map drops) | ≈ 3% | ≈ 3.4% | ≈ 3% |
| per-parse setup (`SemanticRuntimeState::new` 1.1% + `CompiledSemanticRuntimeAnnotations::clone` 0.8% + memset share) | ≈ 3% | in mix | in mix |
| semantic-runtime fact chain (`apply_semantic_runtime_effect_directive` cum) | **0.08%** | **5.29%** | **7.25%** |
| timer + harness (`mach_absolute_time`, `Timespec::now`, `parse_once_timed` self) | ≈ 7% | ≈ 3% | ≈ 1% |

(`sample_band_*.txt` + `selfcum_tables.txt`; the sub-1µs timer share is the
`-0159` measurement caveat showing up in-metric — honest floor for sub-µs claims.)

## 3. Counter censuses: the entry-count model

Quantile exemplars (`outcome_*.json`): entries/cell 20–363; **entries/byte ≈ 5–6.7
on typical accepted cells**; `facts_emitted` = **0** at p10/p25/p50, 3–5 at p75/p90;
**memo hits ≈ 0 everywhere**; rollbacks are all the unchanged fast path. The
literal chain is exact: each plain char costs 5 entries
(`piece → atom → literal → literal_char → letter`) + one trailing failed `piece`.

Stratified 60-cell census (4,744 entries): **`min_ns ≈ 306 + 21.3 × entries`**
(the intercept is the fixed cost plus entry-correlated setup; 79.1 entries/cell
avg). Per-rule entry population:

| rule cluster | share of entries |
|---|---|
| `piece` + `atom` spine pair | 20.0% |
| class chain (`class_literal`, `class_atom`, `class_range*`, …) | ≈ 14% |
| literal chain (`letter`+`literal`+`literal_char`) | 12.4% |
| per-parse scaffolding (`regex`, `entry_*`, `pattern`, `alternation`, `alternative`, `concatenation`) | ≈ 10% |
| group/capture chain | 5.4% |
| quantifier chain | 3.6% |

Entries' marginal share of cell time ≈ (21.3 × 79.1)/(306 + 21.3 × 79.1) ≈ **85%**.
⇒ an entry-count reduction of ~25% ≈ −21% geomean = **the bar is reachable via
entry count alone**; wrapper chains are 30–50% of entries by the table above.
(An early plain-char heuristic priced fusion at −30…−47% — recorded as an
OVER-count; the census's 11% literal-chain share is ground truth. Honest.)

## 4. ⛔ The queued post-C1 program is REFUTED for the geomean bar (each population measured)

- **C2 fact-op constants — DEAD for the bar.** `facts_emitted` 0–5/cell on the
  geomean population; the whole semantic-runtime cluster prices 0.08%/5.3%/7.3%
  by band ⇒ log-weighted ≈ 3.7% ceiling for a TOTAL kill. `to_ascii*`/`FactNameKey`
  frames: 0 in all three profiles.
- **C3 memo-hit delta cheapening — DEAD for the bar.** Memo hits ≈ 0–1 per
  typical cell (the 6526 re-apply leg is a single cell ≈ 0.05% log-share).
- **K5 `check_cycle_id` seen-set — DEAD for the bar.** 0 frames in all bands
  (typical depth is small; nest-family only).
- **K1b — not a geomean population** (ladder/nest cells; `4674` is one cell).
- **6526 residual — not a geomean population** (one cell).

All five stay ADJUDICATED-OUT for the campaign bar (the MAX cell is SETTLED per
`-0161`); they remain on the books only as post-campaign constant trims.

## 5. ▶️ RE-STEER: the G-program (geomean bar, −21% needed)

- **G1 — WRAPPER-CHAIN FUSION (entry-count kill; the primary lever).**
  Emitter-level, parser-agnostic, license-gated chain-collapse: inline
  single-alternative pass-through rule bodies (no annotations, no facts, no memo
  need, value shape = wrapper) into their callers — the literal chain, the class
  chain, the quantifier/group wrappers. STEP-1 = a fusibility-census lane driven
  by the SAME license as the emission (the C1 census↔emission discipline verbatim)
  across ALL 11 grammars, banked BEFORE any regen + a recorded design slice with
  acceptance bands and a pre-adjudicated revert rule. Every entry eliminated also
  deletes its checkpoint + arena + memo-insert constant (the 27–31% alloc cluster
  and 4% memo insert scale with entries).
- **G2 — per-entry constant trim (21.3 ns/entry)** — re-priced AFTER G1 on the
  new floor (populations shift when entries fuse).
- **G3 — per-parse fixed-cost trim (≈171–306 ns: setup + teardown ≈ 6–8%)** —
  `SemanticRuntimeState::new`, the annotations clone, memo-table alloc/memset,
  parser/arena/map drops. Helps the sub-1µs band's log-share most; re-priced
  post-G1.

NEXT = **`-0163` G1 STEP-1**: the chain-collapse license + fusibility census +
design record (one lever per slice; census surprise ⇒ stop).

## 6. OPS notes (banked for the record)

- ⚠️ `sample $!` under a `caffeinate -ims probe &` compound profiles the WRONG
  process (bash/caffeinate wrapper — the first capture round produced bash
  `wait4` trees). The house method is **`sample regex_perf_probe <dur> 1 -wait
  -mayDie -f <out>`** by process NAME; a `-wait` sampler left behind after its
  probe exits parks forever — unstick it by re-running the workload so the name
  reappears.
- §8 cleanup executed mid-session (director directive): **+23 GB reclaimed**
  (33→56 GB free) — `rust/target/debug/incremental` (26 GB cache) deleted;
  `cargo sweep --time 2` confirmed `deps` all live-vintage; `generated_logs`
  already empty; preserved probes + release binaries untouched.
- The debug-probe rebuild (needed for counters) ran guarded AFTER all `sample`
  captures' timing runs (quiet-machine discipline); band timing JSONLs were
  captured before any build started.
