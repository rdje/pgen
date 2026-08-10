# SV-CORPUS-GRAD: graduate the SV parser from the external official / recognized SV test corpora (verible, slang, sv-tests, verilator, …) — the SECOND mandatory axis of the SV `Done` bar

## Metadata

- Tree ID: `SV-CORPUS-GRAD`
- Status: **`active`** (created 2026-07-22, session #190, on the director's
  bar-amending directive — verbatim: "AS part of th SV Done campaign we should
  include the external official and recognized SV test corpus (verible, slang,
  ...). The SV parser shall cleanly pass all of them with flying colors. No Done
  WITHOUT the SV parser graduating from those external official and recognized SV
  corpus." — wording clarified by the director same-day: "Before claiming SV is
  Done it should have passed all the external and official test corpus";
  [[project_sv_done_requires_external_corpus_graduation]])
- Roadmap lane: the **Nexsim delivery directive**
  ([[project_nexsim_sv_signoff_delivery_focus]]) + the locked program. Sibling of
  `SV-REPLAY-DEBT` (axis 1: the family-status gate's last criterion); this tree
  owns axis 2. BOTH must be green before the SV family row flips `Done`.
- Created: `2026-07-22`
- Owner: repo-local workflow

## ⭐ The corpora ALREADY EXIST — this tree builds on them, it does NOT re-acquire

Director-prompted extensive sweep (2026-07-22) re-surfaced the full prior
capture. The acquisition + first characterization were **done 2026-06-17** by
`EXTERNAL-CORPUS.3.1` (`PGEN-EXTERNAL-CORPUS-0007`):

- **Vendored suites** (git submodules, pinned, sparse/shallow to test dirs,
  licenses flagged in `stimuli/sv/subs/PROVENANCE.md`; director 2026-06-17:
  GPL-OK for test-input use, copyleft flagged):
  - `stimuli/sv/subs/sv-tests` — `25e4d275`, ISC, `tests/`, 1028 files, per-file
    `:should_fail_because:`/`:tags:` metadata — THE canonical conformance corpus;
  - `stimuli/sv/subs/verible` — `a0a8d8eb`, Apache-2.0, `verible/verilog/`, 152;
  - `stimuli/sv/subs/slang` — `4106501b`, MIT, `tests/`, 92;
  - `stimuli/sv/subs/verilator` — `a534a1d1`, ⚠️ LGPL-3.0/Artistic-2.0,
    `test_regress/`, ~3263 (the largest practical SV corpus);
  - plus the pre-existing real-design corpora `Cores-VeeR-EL2` / `scr1` /
    `friscv` (multi-file designs needing include/lib chaining; they feed the
    curated `sv_external_corpus_triage_gate`, 14/14 green) and `stimuli/sv/uvm`.
- **Runner**: `stimuli/run_external_corpus.sh sv` (per-file
  `parseability_probe --parse systemverilog --profile sv_2017`, parallel,
  timeout-bounded; re-acquire recipe in PROVENANCE).
- **Baseline characterization** (2026-06-17 vintage, ~25 SV releases stale):
  `stimuli/sv/characterization/characterization.md` + `results.tsv` —
  **5128 files: 2975 pass (58.0%) / 2151 fail / 2 timeout**; per suite:
  sv-tests 76.9%, verible 78.9%, slang 77.2%, verilator 59.4%; the full-design
  corpora read low (friscv 7% / scr1 14% / VeeR 16.7%) because the bulk runner
  parses files in ISOLATION without the preprocessing/include chaining the
  curated 14/14 triage gate does perform.
### ⛔ AXIS-2 FRESHNESS AUDIT — measured 2026-08-08 (session #213), READ BEFORE QUOTING ANY NUMBER

The `5128 / 58.0%` figure above is **pre-ADD-v1 narrative** and must not be cited as current;
`.8`'s vendoring grew the universe. Measured on HEAD this session:

| fact | value | how it was obtained |
|---|---|---|
| corpus universe | **16 336** files | `find stimuli/sv/subs stimuli/sv/uvm -type f \( -name '*.sv' -o -name '*.svh' -o -name '*.v' \) \| wc -l` |
| tracked report says | `9 693 pass / 6 634 fail / 9 timeout / **59.3 %**` | `stimuli/sv/characterization/characterization.md` |
| that report was committed | **2026-07-25** (`350b96de`) | `git log -1 --date=short -- …/characterization.md` |
| `grammars/systemverilog.ebnf` last changed | **2026-07-26** (`7219547c`) | `git log -1 --date=short -- grammars/systemverilog.ebnf` |

⛔ **The measurement PREDATES a grammar commit, so it provably cannot describe HEAD.** This is not
a suspicion — it is a date comparison. Per
[[project_all_parsers_fully_pass_stimuli_and_external_corpora]] ("the first honest act is to
re-measure them rather than to quote them"), **the campaign's first act is a re-measure**, and the
`403` unexplained-divergence adjudication inherits the same defect.

> ✅ **ANSWERED by leaf `.10` (same day): the re-measure was run and the number SURVIVED it.** All
> 16 336 files were re-parsed on HEAD — **zero pass→fail and zero fail→pass** against this report,
> so `59.3 %` and `unexplained = 403` are correct at HEAD; the grammar commit that made this report
> stale-by-provenance changed no SV verdict. ⛔ **That does not retire the audit's rule** — the
> report was unquotable *when written*, and only a measurement could distinguish "stale and wrong"
> from "stale and right". The reports are now **self-dating** (an Instrument-identity hash table),
> so the next reader answers this in one command instead of a date archaeology. See `.10`.

Two further defects in the artifact itself, both already solved for VHDL by
`CORPUS-GRAD-ALL.2.1` and inherited here for free:
1. `results.tsv` is **untracked** (`.gitignore:412`) and **overwritten in place** by the runner —
   ⚠️ preserve it BEFORE re-running or the comparison baseline is destroyed (the trap is recorded
   in `DEVELOPMENT_NOTES.md`; this session preserved it to `rust/target/sv_axis2_baseline/`).
2. Its column 3 carries **absolute paths**, so it is unreproducible from another checkout.
   `stimuli/run_external_corpus.sh` already emits repo-root-relative paths since
   `CORPUS-GRAD-ALL.2.1`, so the next run fixes this automatically — no code change needed here.

Per-file counts by sub-corpus on HEAD (the burn-down denominators): opentitan 3 983, iverilog
3 799, verilator 3 263, ispras-sv-tests 1 266, sv-tests 1 028, sv2v 953, Surelog 828, friscv 441,
black-parrot 205, uvm-core 174, verible 152, Cores-VeeR-EL2 102, slang 92, scr1 50.

- **Canonical repo/roster reference**:
  `docs/decisions/reference_sv_external_corpus_and_oracle_repos.md`
  (+ KM card) — the recognized corpus/oracle list (director 2026-06-09).
- **The binding 2026-06-17 director cross-lane sequencing**
  (`GRAMMAR-WELLFORMED.md` history): **(1)** SV `UNKNOWN→0` — ✅ **DONE
  2026-07-22** (`STRUCTURED-WITNESS-SYNTH`, SV recognized `fully_certified`);
  **(2)** the `H.12.5.8` infix property/sequence binary-operator parse bug —
  **OPEN** (`.8.1` WHY+WHERE done, `.8.2` fix-design pending); **(3)** THEN the
  full external-corpus gap-drive. This directive ACTIVATES lane 3 as an
  in-campaign mandate; lane 2 is a known top burn-down item inside it.
- Boundary kept: **PARSE-COMPLETENESS** retains the differential-ORACLE lane
  (building slang/verible as reference binaries — still director-gated); this
  tree needs only the corpora + their own answer keys, no tool builds. The
  parked `REJECTS-VALID-DEFENSE` SV clause is superseded-by-GO here.

## Goal (the tree's single deliverable)

The SV parser **graduates** from the vendored recognized corpora: every case
carries a **spec/metadata-derived expected verdict** (accept-valid /
reject-invalid / out-of-scope-with-named-cause), the parser matches every
expected verdict — **zero unexplained divergences** — proven by a standing
deterministic **graduation gate** wired into the SV family-status `Done`
computation as its 8th criterion. Exclusions named, justified, tracked; never
gamed ([[feedback_corpus_expected_from_spec_not_fix]]).

**⛔ AMENDED (director 2026-07-22 session #191, ×2 escalating —
[[project_sv_corpus_100pct_lrm_coverage_mandate]]): the bar gains a COVERAGE
axis.** The corpus itself must be proven **top of class**: it must exercise
**100% of the SV LRMs' parseable surface** (1800-2017/2023 + 1364-2005 for
`verilog_2005`), MEASURED — per-profile grammar-rule participation union over
the whole corpus (uncovered rule = corpus gap) + the clause matrix from the
keyed suites + keyed NEGATIVES where the LRM defines illegality;
N/A-with-cause only for clauses with no parse surface (the ratified
principle). Acquisition of the required corpora is a FIRST-CLASS immediate
leaf (the director's order overrides the earlier burn-down-first sequencing).
Graduation (`.5`) = zero unexplained divergences **AND** 100% measured
coverage.

## Ground rules

- **Characterize FIRST, fix SECOND**: measurement passes never masquerade as
  fixes; each defect class then gets its own tool-diagnosed burn-down leaf (fix
  hierarchy: annotations > store > grammar > engine; TOOLBOX checklist on every
  landing leaf; releases bumped per policy).
- A corpus fail is NOT automatically a parser bug: sv-tests
  `:should_fail_because:` negatives, verilator expected-fail regressions, and
  intentionally-invalid fixtures make parse-FAIL the CORRECT verdict — the
  adjudication manifest is what turns raw pass-rates into a defect signal.
- **Preprocessing**: full-design corpora (and many suite cases) require
  `\`define`/`\`include` chaining — route through the triage-manifest
  `bootstrap_files` convention (the 14/14 uvm precedent); where true macro
  EXPANSION is required, adjudicate against the `SVPP-EXPANSION` dependency
  with a named cause, never a silent exclusion.
- **Heavy runs** under `scripts/run_with_memory_guard.sh --budget-mb 16384`;
  known-pathological inputs excluded up front and tracked
  ([[feedback_dont_run_jobs_that_hit_known_pathological_inputs]]).

## Leaves

### `.1` — Re-characterize at TODAY's vintage + adjudication design (read-only + docs)

- **Status: `done`** (`PGEN-SV-CORPUS-GRAD-0003`, session #190, 2026-07-22).
- **Fresh baseline at HEAD** (guarded `stimuli/run_external_corpus.sh sv`,
  exit 0, 61 s, probe vintage-asserted newer than the artifact): **5,128
  files — 3,049 pass (59.5%) / 2,078 fail / 1 timeout** vs the 2026-06-17
  baseline 2,975 (58.0%) / 2,151 / 2. Per-suite: sv-tests 76.9→**78.3**,
  verible 78.9→**79.6**, verilator 59.4→**61.2**, VeeR 16.7→17.6;
  slang/friscv/scr1 unchanged — the gains sit exactly where the ~25
  intervening releases fixed (SVA cascades, LRM restorations, bind).
  Report committed (`stimuli/sv/characterization/characterization.md`).
- ⭐ **The "known burn-down member" is ALREADY FIXED:** the `H.12.5.8` infix
  property/sequence class was closed by releases **1.0.148** (sequence
  cascade, `SV-0010`) + **1.0.149** (property cascade, `SV-0011`), both
  2026-06-25 — independently re-verified at HEAD by a 12/12 REJECT→PASS
  matrix (`docs/tasks/artifacts/sv_replay_debt/h1258_matrix_at_head.txt`).
  Better: those cascades CREATED the `prop_primary_*`/`seq_*` rule layers —
  the `SV-REPLAY-DEBT` residual cluster IS the cascades' new branch universe
  (a generation-coverage target, not a parse bug), which also mechanistically
  explains the 84→120 debt growth.
- **Adjudication design (feeds `.2`):** per-suite expected-verdict derivation —
  sv-tests: `:should_fail_because:`/`:tags:` headers (must-fail vs must-pass);
  verilator: `test_regress` expected-fail naming/driver conventions (`*_bad*`,
  per-test `.out`/`.pl` drivers); slang/verible: fixture semantics are
  positive-dominant — classify per-dir; full designs (VeeR/scr1/friscv):
  chained-unit adjudication via the triage-manifest `bootstrap_files`
  convention (isolation fails ≠ defects). Verdict taxonomy:
  `must_accept` / `must_reject` / `chained_only` / `out_of_scope_with_cause`
  (preprocessor-expansion-required → `SVPP-EXPANSION`; tool-specific
  extension; non-LRM). Manifest = per-file rows (suite, path, expected,
  observed, divergence-class), deterministic and diffable.
- Roster additions: adjudicated + FROZEN v1 by `CORPUS-GRAD-ALL.1`
  (ispras/sv-tests, ivtest, sv2v, Surelog, OpenTitan, black-parrot — see
  `docs/tasks/artifacts/corpus_grad_all/frozen_rosters_v1.md`); vendoring =
  its own leaf when the campaign reaches them.
- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — the stale 2026-06-17 baseline (58.0%) vs the amended Done bar.
  - [x] **ROOT CAUSE (WHY + WHERE)** — N/A (measurement leaf); the H.12.5.8 attribution documented (1.0.148/1.0.149 changelog + the 12/12 matrix).
  - [x] **FIX** — N/A (read-only).
  - [x] **ADDRESSED (verified)** — the fresh guarded run (exit 0, marker banked) with per-suite deltas coherent with the intervening fix history.
  - [x] **NO REGRESSION** — no code change; every per-suite pass-rate ≥ the June baseline (none regressed).
  - [x] **LOCKSTEP** — characterization report committed; tree/MEMORY/CHANGES this commit.

### `.2` — The adjudication manifest + the honest divergence baseline

- **Status: `done`** (`PGEN-SV-CORPUS-GRAD-0004`, session #190, 2026-07-22).
- **Deliverables** (all tracked): `stimuli/sv/adjudicate_external_corpus.py`
  (deterministic stdlib-only generator; byte-identical across re-runs, proven
  by cmp ×3), `stimuli/sv/characterization/adjudication_manifest.tsv` (5,128
  per-file rows: suite / relpath / observed / expected / adjudication / basis),
  `stimuli/sv/characterization/adjudication_summary.md`.
- **⭐ THE HONEST BURN-DOWN BASELINE: 330 unexplained divergences** —
  **324 rejects-valid** (sv-tests 66, verilator 235, verible 20, slang 3) +
  **6 accepts-invalid** (all verilator, named in the manifest:
  `t_class_super_bad3` / `t_concat_link_bad` / `t_flag_wpedantic_bad` /
  `t_timescale_parse_bad` / `t_unconnected_bad` / `t_wire_trireg_unsup`).
  Full picture: 2,842 match / 330 unexplained / 1,102 explained-with-cause
  (svpp macro_use 786, conditional 191, include 124, timeout 1) / 854 deferred
  (chained_only 704 — the design corpora + fragments, leaf `.4`; svpp_owned 150).
  *(Baseline REFINED 330 → 321 same-day by `.3.0`'s answer-key triage — 12
  rows re-adjudicated on upstream in-file grounds; see `.3.0`.)*
- **Expected-verdict derivation (per `.1`'s design, spec/metadata-only — never
  fix-adjacent):** sv-tests `:type:`-stage × `:should_fail_because:` logic
  (post-parse should-fails = parse-level `must_accept`; only `parsing`-stage
  should-fails = `must_reject`; `preprocessing`-typed → svpp-owned) + 7 pinned
  LRM-grounded rulings for header-less should-fails (6 lexical/grammar-level
  `must_reject`, 1 semantic `must_accept` — recorded in-script with reasons);
  verilator driver conventions (`fails=True|test.vlt_all` × golden-`.out`
  "syntax error" split; `t_pp_*`/`t_preproc_*` = preprocessor-target → svpp;
  driverless files = fragments → chained; `include-dependent rejects demoted to
  chain-level); slang/verible per-dir fixture semantics (strict-mode errors are
  semantic; kythe include/multi-file dirs chained).
- **Hardening iterations (all landed before banking):** comment/string
  stripping before the svpp-dependency scan (a macro named in a comment must
  not explain away a real defect — moved 2 rows to unexplained); timeout
  precedence over deferral (the 1 pathological row surfaces regardless of
  lane); preprocessor-target and include-demotion refinements (collapsed the
  accepts-invalid population 11 → 6 by metadata-grounded rules, mirrored from
  the sv-tests preprocessing rule).
- **Tool verification:** representative rejects-valid row
  `tests/chapter-5/5.7.1--integers-underscores.sv` re-verified live —
  unambiguously valid LRM 5.7.1 SV (underscore literals, spaced based literal
  `32 'h 12ab_f001`), `parseability_probe --parse systemverilog --profile
  sv_2017` exit 1 at HEAD, matching the manifest row. Cluster shape is
  coherent with known history: chapter-16 property rows sit in the same
  grammar region as the `SV-REPLAY-DEBT` prop_primary cluster.
- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — 2,078 raw fails carried no defect signal until
    expected-vs-actual adjudication separated intended-fails and harness
    dependencies from parser-owned divergences.
  - [x] **ROOT CAUSE (WHY + WHERE)** — N/A (measurement leaf); every expected
    verdict carries its per-row `basis` provenance in the manifest.
  - [x] **FIX** — N/A (no parser change; additive analysis tooling only).
  - [x] **ADDRESSED (verified)** — manifest built over all 5,128 rows;
    determinism proven byte-identical across re-runs; probe spot-verification
    of the observed column; accepts-invalid rows individually named.
  - [x] **NO REGRESSION** — no parser/grammar/codegen surface touched.
  - [x] **LOCKSTEP** — characterization report pointer, tree, TASK_TREE index,
    MEMORY/CHANGES/DEVELOPMENT_NOTES/LIVE tracker this commit.

### `.3` — Defect burn-down (umbrella; one leaf per defect class)

- **Status (umbrella): `in_progress`** — worklist after `.3.1`: **279
  unexplained divergences** — **RE-BASED to 445 by `.8a`'s ADD-v1
  acquisition, 550 by `.8b.1`, 557 by `.8b.2`, and 564 by `.8b.3`** (543
  rejects-valid + 21 accepts-invalid; the sv_2017-lane answer keys are now
  COMPLETE — every remaining number is measured defect signal, not
  deferral; new families join the leaf-cutting map as burn-down proceeds). Pre-`.8a` detail (273
  rejects-valid + 6 accepts-invalid; the manifest is the per-row worklist,
  the `.3.0` family table the leaf-cutting map — F1 CLEARED by `.3.1`).
  Suite split: verilator 220, sv-tests 39
  (generic 11, chapter-5/lexical 9 — incl. the probe-verified
  underscore/spaced-literal gap — chapter-7 4, chapter-6 4, chapter-16/SVA 4,
  chapter-18 3, chapter-11 2, chapter-8 1, chapter-12 1), verible 13,
  slang 1, + the 6 named accepts-invalid rows (verilator-strictness
  adjudication candidates).
- ⭐ **REFRESHED leaf-cutting map (`.3.2`, session #198):** the 543-row
  population is now re-clustered at the ADD-v1 vintage into 13 construct
  families (`rejects_valid_families_v2.md`). Ranked #1 = **SVA
  implication/property (ch16), 101 rows** (ispras 53 / verilator 31 / Surelog 8
  / sv-tests 4) — root cause tool-pinned: the `|->`/`|=>` operators are ABSENT
  from the grammar (see `.3.2`). Burn-down cut from this map: **`.3.3`**
  (`|->`/`|=>`, SVA #1, done) → **`.3.4`** (modport shared-direction list, #3,
  done) → **`.3.5`** (spaced-based number literals §5.7.1, #6, done) →
  **`.3.6`** (`unique0` if/case qualifier §12.4.2/§12.5.3, in progress);
  remaining: constraint/randomize (23), directives (22), drive strength (21),
  named block (20), size-cast (13), enum-base-range (9), … cut from the same
  map. Burn-down continued: **`.3.6`** (`unique0` if/case §12.4.2, #11, done,
  432→426) → **`.3.7`** (drive/charge strength keywords §28.11/A.8.6, in
  progress; the family map REFRESHED to v3 over the current 426-row population
  that leaf — see `.3.7`). Burn-down continued: **`.3.7`** (drive/charge strength
  keywords §28.11/A.8.6, done, 426→406, CROSS-PROFILE + the largest v2005 heal
  116→62). Rejects-valid baseline after `.3.7`: 406 (accepts-invalid 21; v2005 62),
  then **406 → 383 → 382** across `.3.8`/`.3.9`. ⛔ **CURRENT, RE-MEASURED ON HEAD by
  `.10` (2026-08-08): rejects-valid 382 + accepts-invalid 21 = `unexplained` 403**,
  reproduced identically under three different instrument settings. This line had
  read `406` since `.3.7` while the later leaves moved it — corrected by `.10`.

#### `.3.0` — Stuck-point clustering over the rejects-valid population (read-only diagnosis)

- **Status: `done`** (`PGEN-SV-CORPUS-GRAD-0005`, session #191, 2026-07-22).
- Every `divergence:unexplained_rejects_valid` row probed;
  `furthest_position` extracted; clustered by a normalized 3-token stuck
  signature (`stimuli/sv/cluster_rejects_valid.py`, 1.7 s / 324 probes).
  Artifacts: `docs/tasks/artifacts/sv_corpus_grad/rejects_valid_clusters.tsv`
  (per-row: positions, signature, stuck line) + `rejects_valid_clusters.md`
  (ranked table).
- **⭐ Triage feedback into the answer key (baseline 330 → 321):** the
  clustering surfaced 12 mis-adjudicated rows, fixed in the adjudicator on
  upstream in-file/metadata grounds (never fix-adjacent): (a) verible
  `// verilog_syntax:` excerpt-mode fixtures = tool-mode fragments →
  out-of-scope-with-cause (5 rows); (b) three pinned intentionally-invalid
  fixtures → `must_reject` (verible `bad-id-lex.sv` "lexer should reject",
  verible `module_begin_block.sv` "LRM-invalid syntax", slang
  `cross-ident-in-binsof.sv` "LRM disallows … bins_expression"); (c) the
  sv-tests `.svh`-include-payload rule promoted generic (slang `local.svh` is
  a bare string literal). Refined manifest re-proven deterministic (cmp ×2).
  **Refined baseline: 321 unexplained = 315 rejects-valid (verilator 235 /
  sv-tests 66 / verible 13 / slang 1) + 6 accepts-invalid.**
- **The consolidated defect-family worklist (analyst merge of the 171 raw
  signatures; counts ≈ from signature groups, exact rows in the TSV):**
  | family | ≈rows | representative |
  |---|---|---|
  | F1 `interface class` (LRM 8.26 — construct absent) | ~47 | `interface class Bar; endclass` stuck at `class` |
  | F2 SVA property/sequence tails (LRM 16: `disable iff`, `\|->`/`\|=>` RHS, `[*N]`/`[->N]` reps, match items) | ~23 | `disable iff (a) b \|-> c` |
  | F3 constraint/randomize (LRM 18: `dist {[a:b] :/ w}`, `randomize() with {…}`, `rand_mode`) | ~23 | `dist { [0:1], [2:5] :/ 2 }` |
  | F5 compiler directives in-scope (LRM 22: `` `begin_keywords`` semantics, `` `pragma``, `` `__FILE__`` ) | ~17 | `` `begin_keywords "1364-2001" `` then `reg logic;` |
  | F4 modport direction-lists (LRM 25.5) | ~15 | `modport modp(input clk, rst);` |
  | F6 number-literal lexicals (LRM 5.7: spaced based literals `32 'd 1`, `-8'd 6`, size-cast `32'(…)`) | ~13 | probe-verified in `.2` |
  | F8 interface member type refs (`if0.rq_t` as type, virtual-interface members) | ~10 | `localparam type p0_t = if0.rq_t;` |
  | F9 drive-strength/charge/`scalared`-`vectored` decl forms + UDP | ~9 | `assign (supply0, weak1) #(1:0:1,…)` |
  | F7 enum base/range forms (`enum [15:0] {…}`) | ~8 | `typedef enum [15:0] {` |
  | F11 legacy generate/label forms | ~7 | `begin : topgen` under `generate` |
  | F10 `unique0` on if/case (LRM 12.4.2) | 4 | `unique0 if (a == 0)` — smallest well-defined class |
  | long tail (singleton signatures, per-row triage as burn-down proceeds) | ~150 | — |
- **Recommended first burn-down leaf: F1 `interface class`** — the largest
  single well-defined construct family; one grammar-addition wave probably
  clears ~15% of the entire baseline. Each `.3.x` fix leaf owes its own
  TOOLBOX WHY+WHERE + the full heavy battery per the ground rules.
- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — 324 rejects-valid rows had no defect-class
    structure; burn-down leaves cannot be cut from a flat list.
  - [x] **ROOT CAUSE (WHY + WHERE)** — per-row stuck positions + signatures
    banked in the TSV (tool: parseability_probe furthest_position, 324 runs).
  - [x] **FIX** — N/A code-wise; the 12-row answer-key refinement is the
    leaf's corrective output, grounds cited per row in-script.
  - [x] **ADDRESSED (verified)** — refined manifest deterministic (cmp ×2);
    cluster artifacts banked; family worklist + first-leaf recommendation.
  - [x] **NO REGRESSION** — no parser surface touched; adjudicator refinement
    strictly metadata-grounded.
  - [x] **LOCKSTEP** — tree/TASK_TREE/MEMORY/CHANGES/LIVE updated to 321 this
    commit.

#### `.3.1` — F1: `interface class` unreachable under `sv_2017` (LRM 8.26 profile-gating defect)

- **Status: `done`** (`PGEN-SV-CORPUS-GRAD-0006`, session #191, 2026-07-22;
  release `1.0.167` → **`1.0.168`**, schema `16` unchanged, ledger
  **`SV-0038`**).
- **REPRODUCE:** minimal `interface class Bar;\nendclass` rejects at
  `--profile sv_2017` (furthest_position 9 = at `class`); the same input
  parses **exit 0** at `--profile sv_2023`.
- **ROOT CAUSE (WHY+WHERE, tool-backed):** the three
  `interface_class_declaration` consumer wirings live ONLY in the sv_2023
  variant rules — `class_item_sv_2023`, `anonymous_program_item_sv_2023`,
  `package_or_generate_item_declaration_sv_2023` — while the `_sv_2017`
  twins omit the branch (`grammars/systemverilog.ebnf` ~:1049 / ~:552 /
  :3841). Scoped `--trace-rules interface_class_declaration` shows the rule
  is NEVER ENTERED under sv_2017 (no call-tree lines). But interface classes
  are IEEE **1800-2017** §8.26 (`interface_class_declaration ::=` present in
  `docs/systemverilog/2017/md/section-8-classes.md`); the declaration rule
  itself (:2634) and the `implements` clause in `class_declaration_sv_2017`
  (:1027) were already profile-correct — only the consumer wirings were
  mis-gated 2023-only (the 2017 `grammar_clean.ebnf` extraction lacks the
  production, the likely mis-gating origin).
- **FIX (hierarchy level 1 — pure grammar, LANDED):** the
  `interface_class_declaration` branch mirrored from each sv_2023 twin into
  the three sv_2017 variants (`grammars/systemverilog.ebnf` :554/:1054/:3853);
  canonical regen `make -C rust focus_systemverilog` (65 s, guarded, parser
  mtime > grammar mtime asserted). Zero new rules — census 1,466 unchanged.
- **VERIFIED (measured GLOBALLY, full battery green):**
  - Repro matrix: minimal + sv-tests `class_test_28` REJECT→ACCEPT under
    `sv_2017`; both still ACCEPT under `sv_2023`; sanity module unchanged.
  - **Full external corpus 5,128: pass 3,049 → 3,091 (+42, 59.5% → 60.3%),
    ZERO per-suite regressions** (sv-tests 805→832, verilator 1,996→2,011;
    VeeR/friscv/scr1/slang/verible byte-identical). Adjudication baseline
    **321 → 279 unexplained** (rejects-valid 315→273; sv-tests chapter-8
    18→1). Residual adjacent rows deliberately NOT claimed (named in `.3.0`
    artifacts): illegal interface-class contents `_bad` tests + the
    store-gated `implements <type_parameter>` head.
  - `sv_stimuli_quality_gate` PASS (exit 0, peak 11,967 MB / 1,760 s guarded;
    `closed_loop_replay_targets_total` **120 UNCHANGED** — no replay-debt
    growth from the new branches).
  - `sv_cert_recognized_union_gate` GREEN on an evidence-grounded
    re-baseline: the fix converts **4 unreachability proofs → genuine
    witnesses** (canonical `1343/10/1322/11` → `1343/6/1326/11`; union
    witness `1333→1337`; UNKNOWN unchanged 11/0; residual `[]`;
    count-conserving ±4; deterministic seeds 0/7/42; spf=0; solo-canonical
    agrees `1343/17/1326/0`). Contract re-baselined same-slice with a full
    rebaseline_note (`systemverilog_recognized_cert_union_contract.json`).
  - `verilog_2005_conformance_gate` GREEN byte-inert (orphans 0, matrix
    240/0, cert `1115/328/773/14` byte-identical seeds 0/7/42) — the
    `@profiles`-gated edit provably does not touch the v2005 profile.
  - `ast_shape_contract_gate` PASS; `sv_external_corpus_triage_gate` PASS;
    `--lint-grammar` clean (1,466 rules, all error classes 0,
    profile_orphans 0).
  - `clippy_on_rust_change` completed (grammar-only change — no Rust source
    delta; the regenerated parser is the untracked emit); dual-feature lib
    tests **1013 passed / 0 failed / 29 ignored** — byte-equal to the banked
    #189 baseline count.
- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — minimal 2-line repro + the F1 cluster (~47
    corpus rows) rejected under `sv_2017`, furthest_position at `class`.
  - [x] **ROOT CAUSE (WHY + WHERE)** — tool-backed (scoped `--trace-rules`:
    rule never entered; sv_2023 exit-0 control; LRM §8.26 grounding): the
    three consumer wirings were profile-gated `sv_2023`-only; WHERE =
    `grammars/systemverilog.ebnf` sv_2017 variant rules.
  - [x] **FIX** — hierarchy level 1 (pure grammar), three mirrored branches.
  - [x] **ADDRESSED (verified)** — before→after measured globally: corpus
    +42 / baseline 321→279 / repro matrix flips; full gate battery green.
  - [x] **NO REGRESSION** — zero per-suite corpus regressions; sv_2023 and
    verilog_2005 byte-inert; replay-debt total unchanged; UNKNOWN counts
    unchanged; quality/shape/triage/conformance gates green.
  - [x] **LOCKSTEP** — ledger `SV-0038` + contract `1.0.168` highlights +
    identity + SV book changelog-index + tree/TASK_TREE/MEMORY/CHANGES/LIVE
    this commit.
  burn down (315 rejects-valid + 6 accepts-invalid; the manifest is the
  per-row worklist, the `.3.0` family table above is the leaf-cutting map:
  F1 interface-class ~47 first). Suite split: verilator 235, sv-tests 66
  (by chapter: generic 21, chapter-8/classes 18, chapter-5/lexical 9 — incl.
  the probe-verified underscore/spaced-literal gap — chapter-7 4, chapter-6 4,
  chapter-16/SVA 4, chapter-18 3, chapter-11 2, chapter-12 1), verible 13,
  slang 1, + the 6 named accepts-invalid rows (verilator-strictness
  adjudication candidates).
  ~~First known member `GRAMMAR-WELLFORMED.H.12.5.8`~~ — RESOLVED before this
  tree reached it: fixed by releases 1.0.148/1.0.149 (re-verified 12/12 at
  HEAD, see `.1`). NOTE the standing convergence: the chapter-16 property rows
  live in the same grammar region as the `SV-REPLAY-DEBT` `prop_primary_*`
  residual cluster — one lane may move both axes.

#### `.3.2` — Refreshed family map over the current 543-row population + the foundational SVA finding (read-only diagnosis)

- **Status: `done`** (`PGEN-SV-CORPUS-GRAD-0017`, session #198, 2026-07-23;
  READ-ONLY — no parser/grammar/generated/codegen change).
- **Why:** `.3.0`'s family map was built over the pre-ADD-v1 315-row
  population. The current `.3`-worklist is **543 rejects-valid** (post-`.8b.3`
  / `.8c.*`), of which only the clause-keyed ispras rows carried a structural
  key. This leaf refreshes the leaf-cutting map over the whole current
  population and pins the sharpest family's root cause.
- **The instruments (both deterministic, both re-runnable at any vintage):**
  1. Re-ran the tracked `stimuli/sv/cluster_rejects_valid.py` over the current
     manifest → `docs/tasks/artifacts/sv_corpus_grad/rejects_valid_clusters_v2.tsv`
     + `.md` (543 rows probed, 247 distinct 3-token stuck signatures; guarded,
     10 s, peak 210 MB). The `.3.0` artifacts (`rejects_valid_clusters.{tsv,md}`)
     are PRESERVED unmodified. (One-line hardening to the tool: captured probe
     text is now stripped to a repo-relative path so an unreadable-file error —
     `sv2v/test/lex/latin1.sv`, latin1-encoded — cannot leak an absolute
     checkout path into a tracked artifact; the DOCPATH doctrine flagged the
     first draft, which is the gate working.)
  2. NEW `stimuli/sv/classify_rejects_valid_families.py` (priority-ordered
     structural bucketer over the stuck source line) →
     `rejects_valid_families_v2.{tsv,md}`. Determinism byte-proven (cmp ×2 on
     both outputs); `py_compile` clean.
- **⭐ THE REFRESHED FAMILY MAP (543 rows → 13 families, ranked by cross-suite
  yield):**
  | # | family | rows | dominant suites |
  |---|---|---|---|
  | 1 | **SVA implication/property (ch16)** | **101** | ispras 53, verilator 31, Surelog 8, sv-tests 4 |
  | 2 | interface/modport (ch25) | 50 | verilator 24, ispras 18 |
  | 3 | constraint/randomize (ch18) | 23 | verilator 19 |
  | 4 | compiler directives (ch22) | 22 | iverilog 18 |
  | 5 | number literal spaced-based (ch5) | 22 | sv-tests 13, Surelog 5 |
  | 6 | drive/charge strength (ch28) | 21 | verilator 10, ispras 5 |
  | 7 | named block/label (ch9/27) | 20 | verilator 8, Surelog 6 |
  | 8 | size/type cast `N'(...)` (ch6/11) | 13 | Surelog 5, sv2v 4 |
  | 9 | enum base range (ch6) | 9 | verilator 8 |
  | 10 | foreach/array (ch7) | 9 | verilator 5 |
  | 11 | unique0 (ch12) | 6 | verilator 3 |
  | 12 | coverage bins/cross (ch19) | 4 | ispras |
  | — | OTHER (per-row triage tail) | 243 | broad |
- **⭐⭐ THE FOUNDATIONAL SVA FINDING (tool-pinned WHY+WHERE — the #1 family's
  root cause):** the SV grammar is **missing the two core SVA sequence-
  implication operators `|->` (overlapped) and `|=>` (non-overlapped)** — no
  tokens, no `prop_primary` branches, and (git `-S`) they have **never** been
  present. Confirmed four independent ways:
  1. A **basic** concurrent assertion is rejected: `assert property
     (@(posedge clk) a |-> b)` fails (furthest at the `|->`); `assert property
     (@(posedge clk) p3)` (no implication) **passes** — the gap is the
     implication operator itself, not the RHS.
  2. Token scan: the only `>`/`|` operator tokens are `implies` (`->`),
     `or_assign` (`|=`), `sequence_implies` (`=>`), `nonblocking_implies`
     (`->>`) — **no `|->`, no `|=>`** (grammar-wide grep + `--dump`).
  3. Scoped `--trace-rules prop_primary_sv_2017`: after the antecedent
     `sequence_expr` returns, no branch consumes `|->`; `bitwise_or` matches
     `|` then fails needing an RHS expression → backtrack.
  4. IEEE 1800-2017 defines them pervasively (§17/§23 examples;
     `vpiOverlapImplyOp`/`vpiNonOverlapImplyOp`, §83). The mangled
     `prop_primary` branches `implies property_expr` (`->`), `sequence_expr
     or_assign property_expr` (`|=`), and `property_expr implies property_expr`
     are the extraction's damaged remnants — the LRM-markdown→EBNF extractor
     split the `|`-prefixed operators on `|` (the EBNF alternation metachar),
     which is why the LRM-*extracted* grammar also lacks them.
  - **Reconciliation with H.12.5.8** ("infix property/sequence bug fixed",
    1.0.148/1.0.149): its 12/12 matrix tested only the *word* operators
    (`until`/`or`/`and`/`intersect`/`within`/`##`) — **never `|->`/`|=>`**. This
    gap is orthogonal and open.
- **Impact / recommendation:** the `|->`/`|=>` gap explains the bulk of the
  101-row SVA family (every `s |-> p` / `s |=> p`), spans ispras (clause-keyed),
  verilator, and Surelog, and **converges with `SV-REPLAY-DEBT`** (the
  `prop_primary_*` cluster) — one fix moves both SV `Done` axes. **Recommended
  first burn-down leaf `.3.3`:** add the two operators (tokens + `prop_primary`
  branches, both profiles), faithful to A.2.10, additive/low-risk. Remaining
  families (interface/modport 50, constraint/randomize 23, …) are subsequent
  `.3.x` leaves cut from this map.
- **Artifacts:** `docs/tasks/artifacts/sv_corpus_grad/rejects_valid_clusters_v2.{tsv,md}`,
  `rejects_valid_families_v2.{tsv,md}`; tools
  `stimuli/sv/cluster_rejects_valid.py` (re-run) +
  `stimuli/sv/classify_rejects_valid_families.py` (new).
- **Acceptance Checklist (enforced — diagnosis leaf)**
  - [x] **REPRODUCE / ISSUE** — the `.3` worklist grew 315 → 543 across the
    ADD-v1 acquisition with no refreshed structural map; the sharpest family
    (SVA) had no pinned root cause.
  - [x] **ROOT CAUSE (WHY + WHERE)** — the `|->`/`|=>` absence tool-pinned four
    ways (minimal repro, token scan, `--trace-rules`, LRM grounding); WHERE =
    `grammars/systemverilog.ebnf` token section + `prop_primary_sv_2017/2023`.
  - [x] **FIX** — N/A (read-only; additive diagnosis tooling only — no
    parser/grammar/codegen surface touched).
  - [x] **ADDRESSED (verified)** — the refreshed 543-row family map over the
    whole population; determinism byte-proven (cluster re-run + classifier ×2).
  - [x] **NO REGRESSION** — no parser/grammar/generated surface touched; the
    `.3.0` artifacts preserved unmodified; `results.tsv` untouched.
  - [x] **LOCKSTEP** — tree + TASK_TREE index + MEMORY/CHANGES/
    DEVELOPMENT_NOTES this commit.

#### `.3.3` — SVA implication operators `|->`/`|=>` absent (IEEE 1800-2017 A.2.10 — the #1 rejects-valid family)

- **Status: `done`** (`PGEN-SV-CORPUS-GRAD-0018`, session #198,
  2026-07-23; release `1.0.168` → **`1.0.169`**, schema `16` → **`17`**, ledger
  **`SV-0039`**).
- **REPRODUCE:** `assert property (@(posedge clk) a |-> b)` and `… a |=> b`
  reject at HEAD (`--profile sv_2017`, furthest at the `|->`/`|=>`); the
  implication-free `assert property (@(posedge clk) p3)` **passes**. The #1
  refreshed rejects-valid family (`.3.2`: SVA implication/property, 101 rows,
  ispras 53 / verilator 31 / Surelog 8).
- **ROOT CAUSE (WHY + WHERE, tool-pinned in `.3.2`, four ways):** the two core
  SVA sequence-implication operators are entirely absent from
  `grammars/systemverilog.ebnf` — no `|->` (overlapped) / `|=>` (non-overlapped)
  token, and no `prop_primary` branch consumes them (`git -S` proves they were
  never present). The only `>`/`|` operator tokens are `implies` (`->`),
  `or_assign` (`|=`), `sequence_implies` (`=>`), `nonblocking_implies` (`->>`).
  IEEE 1800-2017 A.2.10 defines `property_expr ::= … | sequence_expr |->
  property_expr | sequence_expr |=> property_expr …` (used pervasively — §17,
  §23; `vpiOverlapImplyOp`/`vpiNonOverlapImplyOp` §83). The mangled
  `prop_primary` branches `implies property_expr` / `sequence_expr or_assign
  property_expr` / `property_expr implies property_expr` are the LRM-markdown→
  EBNF extractor's damaged remnants (the `|`-prefixed operators were split on
  `|`, the EBNF alternation metachar; the LRM-extracted grammar lacks them too).
- **FIX (hierarchy level 1 — pure grammar, additive):** add two tokens
  (`overlapped_implication := "|->"`, `non_overlapped_implication := "|=>"`,
  **profile-gated `["sv_2017","sv_2023"]`** — SVA is SystemVerilog-only, absent
  from IEEE 1364-2005, so the operators are gated exactly like
  `nonblocking_implies`/`SV-0023`) and mirror the two A.2.10 branches
  `sequence_expr <op> property_expr` into BOTH `prop_primary_sv_2017` and
  `prop_primary_sv_2023`. The pre-existing mangled branches are LEFT in place
  (removing them = a separate accepts-invalid leaf; under the longest-match
  tournament the correct operator wins for real `|->`/`|=>` input). Scope kept
  tight per one-commit/one-defect.
  - ⭐ **Gating decision (recorded):** the first regen left the tokens UNGATED,
    and `verilog_2005_conformance_gate` correctly went RED — not a parse
    regression (`profile_orphans=0`, corpus matrix 0 mismatches) but a census
    shift (ungated tokens count in EVERY profile's inventory: v2005 cert
    total 1115→1117 as +2 unreachability proofs). The LRM-faithful fix is to
    GATE the tokens `sv_2017/sv_2023` (they are not 1364-2005 constructs) — the
    gate telling us "you added something visible to v2005" is the gate working.
    Gating keeps v2005 GENUINELY byte-inert (cert stays 1115, no re-baseline)
    while the sv_2017/sv_2023 union cert still witnesses them (1345). Chose
    faithfulness over the cheaper re-baseline (quality > speed).
- **VERIFIED (measured GLOBALLY):**
  - **Repro matrix (regen'd parser, guarded 307 s / 11 GB):** `a |-> b`,
    `a |=> b`, `a |-> p3` (named prop), `a |=> ##1 b` (delayed seq), and the
    exact ispras `16.12.01_01` all REJECT→ACCEPT under `sv_2017`; `sv_2023`
    accepts too; a sanity module unchanged. AST shapes correct:
    `{kind:"overlapped_implication"}` / `{kind:"non_overlapped_implication"}`.
  - **Full external corpus (16,336 files, guarded re-char): pass
    9,361 → 9,433 (+72), timeout 9 → 9 (unchanged), ZERO per-suite
    regressions** — ispras 996→1,030 (+34), verilator 2,011→2,032 (+21,
    all `t_assert_*` SVA files), Surelog 669→675 (+6), opentitan 691→695 (+4),
    sv-tests 832→836 (+4), iverilog 3,162→3,165 (+3), all others byte-flat;
    0 crash; overall 57.7%. (The gained files are all SVA-assertion cases;
    the tracked ~20 s-boundary verilator `t_math_synmul_mul.v` is a timeout in
    both the pre-fix and post-fix runs — its `observed` jitters pass↔timeout
    with machine load, unrelated to the fix and outside the baseline as a
    `chained_only` deferral.)
  - **Adjudication baseline: rejects-valid 543 → 481 (−62), accepts-invalid
    21 → 21 (IDENTICAL set — no over-acceptance introduced), unexplained
    564 → 502.** The 62 moved rows all provably contain `|->`/`|=>` (spot-
    verified across chapters 6/11/14/16); zero new rejects-valid (no valid file
    newly rejected). Manifest deterministic (cmp ×2), path-clean.
  - **`sv_cert_recognized_union_gate` GREEN on an evidence-grounded re-baseline:**
    the 2 new operator tokens are each POSITIVELY WITNESSED — total 1,343→1,345
    (+2), canonical witness 1,326→1,328 (+2), union witness 1,337→1,339 (+2);
    proof 6, UNKNOWN 11/0, residual `[]` all unchanged; **still
    `fully_certified_via_union=true`**, deterministic seeds 0/7/42, spf=0.
    Contract JSON re-baselined same-slice with a full rebaseline_note.
  - **v2005 adjudication manifest BYTE-IDENTICAL** (SVA is not in
    `verilog_2005`; the gated tokens are referenced only by
    `prop_primary_sv_2017`/`_sv_2023`) — 149 unexplained unchanged; git-diff
    empty. **`verilog_2005_conformance_gate` GREEN byte-inert** on the GATED
    build: cert `1115/328/773/14` byte-identical, corpus matrix 240/0,
    profile-orphans 0 (no re-baseline of the conformance contract needed).
  - **`sv_stimuli_quality_gate` PASS** — `closed_loop_replay_targets_total`
    120 → **126** (⚠️ **+6**: the 4 new `prop_primary` branches + 2 token rules
    are new closed-loop GENERATION-coverage targets — the same generation-target
    growth pattern the H.12.5.8 property cascades produced; **this feeds
    `SV-REPLAY-DEBT`** (the sibling axis) rather than reducing it; NOT a
    regression — the gate is green, the targets are new surface to witness).
    `ast_shape_contract_gate` 18/0; `sv_external_corpus_triage_gate` 14/14.
  - **Gating re-verification (the second regen, gated tokens):** the repro
    matrix re-passes under `sv_2017`/`sv_2023` and `verilog_2005` correctly
    REJECTS `|->`; the full corpus re-char + re-adjudication produces a manifest
    **BYTE-IDENTICAL** to the pre-gating one (proving the gating tweak changed
    no parse outcome), and `sv_cert_recognized_union_gate` re-confirms
    `1345/1339` green + `verilog_2005_conformance_gate` re-confirms `1115`.
  - **`--lint-grammar` GREEN: 1,468 rules** (census 1,466 → 1,468, +2 token
    rules), **profile_orphans 0**, all error classes 0 (non_terminating /
    unreachable_rules / undefined_references / unbound_fact_kinds /
    ordered_choice_shadowing = 0). **`clippy_on_rust_change` PASS** ("no
    Rust/generated Rust changes detected" — grammar-only, the generated parser
    is the untracked emit). **Dual-feature lib tests 1,020 passed / 0 failed /
    29 ignored** (byte-equal to the post-`.8c.3` baseline — no lib test added).
- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — minimal `a |-> b` / `a |=> b` reject; the F-SVA
    cluster (101 rows) is the #1 refreshed family (`.3.2`).
  - [x] **ROOT CAUSE (WHY + WHERE)** — tool-pinned four ways (`.3.2`); WHERE =
    token section + `prop_primary_sv_2017`/`_sv_2023`.
  - [x] **FIX** — hierarchy level 1 (pure grammar): 2 profile-gated tokens +
    4 branches; LRM-faithful gating (v2005 byte-inert).
  - [x] **ADDRESSED (verified)** — before→after measured globally: corpus +72
    (zero regressions), baseline 564→502, repro matrix flips, correct AST kinds.
  - [x] **NO REGRESSION** — zero per-suite corpus regressions; accepts-invalid
    identical; v2005 byte-inert (manifest + cert 1115); cert green (re-baselined,
    still fully-certified); quality/shape/triage green; gating manifest
    byte-identical.
  - [x] **LOCKSTEP** — ledger `SV-0039` + contract `1.0.169`/schema 17 + SV book
    changelog-index + tree/TASK_TREE/MEMORY/CHANGES/DEVELOPMENT_NOTES/LIVE this
    commit.

#### `.3.4` — modport shared-direction port list absent (IEEE 1800-2017 A.2.9 — the #3 rejects-valid family, interface/modport)

- **Status: `done`** (`PGEN-SV-CORPUS-GRAD-0019`, session #199, 2026-07-23;
  release `1.0.169` → **`1.0.170`**, schema `17` → **`18`**, ledger **`SV-0040`**;
  the `in_progress` header was a lockstep-drift never flipped at landing — the
  body's acceptance checklist, the release/ledger bump, and the commit subject
  all record DONE; corrected in `.3.7`, `PGEN-SV-CORPUS-GRAD-0022`).
  The second fix cut from the `.3.2`/v2 refreshed family map:
  after `.3.3` drained the #1 SVA family, **interface/modport (ch25), 50 rows**
  is the next single-construct family (verilator 24 / ispras 18 / sv-tests 3 /
  sv2v 3 / Surelog 1 / verible 1). Filtered to the current (post-`.3.3`, 481)
  rejects-valid manifest: all 50 still stuck. The dominant coherent sub-defect
  (~22 rows) is the **comma-shared-direction modport port list**
  (`modport master(input a, b, output c, d)`); the remainder of the family is
  the modport-expression `.P(expr)` form (helped by the same fix where a
  shared-direction list follows) plus ~15 method-call/queue-slice rows the
  coarse `\.\w+\s*\(` bucketer mis-filed into this family (they belong to OTHER,
  routed on their own).
- **ROOT CAUSE (WHY + WHERE), tool-pinned three ways:**
  - **WHERE:** `grammars/systemverilog.ebnf:526`
    `modport_simple_ports_declaration := port_direction modport_simple_port`.
  - **WHY:** the rule is **missing the LRM A.2.9
    `{ , modport_simple_port }` repetition** — it admits exactly ONE port per
    direction group. IEEE 1800-2017 §25 / A.2.9 (verified verbatim from the
    in-repo LRM md `docs/systemverilog/2017/md/section-25-interfaces.md:139`):
    `modport_simple_ports_declaration ::= port_direction modport_simple_port { , modport_simple_port }`.
  - **Probe (before):** control `modport master(input a, output c)` ACCEPTS;
    defect `modport master(input a, b, output c, d)` REJECTS at
    `furthest_position=58` (the shared-direction port `b`). Scoped
    `--trace-rules modport_item,modport_ports_declaration,modport_simple_ports_declaration`
    rule_stack at the furthest position: after `input a`, the outer
    `modport_item ( comma modport_ports_declaration )*` cannot start a new
    `modport_ports_declaration` at `b` (not a `port_direction`/attribute/
    import-export/clocking), so `rparen` is expected but `,` is found. Evidence
    `docs/tasks/artifacts/sv_corpus_grad/modport_diag/before.txt`.
- **FIX (grammar-only, hierarchy level 1) — TWO coupled parts:**
  1. Add the LRM repetition to the one rule —
     `modport_simple_ports_declaration := port_direction modport_simple_port ( comma modport_simple_port )*`
     with the family's proven `[$first, $rep::2*]` extraction-spread idiom
     (`-> {direction: $1, ports: [$2, $3::2*]}`). The emitted shape changes
     `port: $2` → `ports: [$2, $3::2*]` (a wire-format change).
  2. **Complete the SV reserved-keyword list with `ref`
     (`reserved_non_keyword_identifier_sv:402`).** A mid-measurement
     regression surfaced this REQUIRED, coupled second part: two verilator
     files that were PASSING (`t_interface_virtual.v` / `t_interface_virtual_bad.v`,
     `modport phy(input addr, ref data)`) started FAILING because the new
     greedy `( comma modport_simple_port )*` gobbled `ref` as a
     `port_identifier`. Root cause (tool-pinned): `ref` is a genuine SV
     port_direction (`port_direction_sv_only := kw_ref`) but was
     **erroneously omitted** from `reserved_non_keyword_identifier_sv` (which
     already lists `input`/`output`/`inout`); the engine's `*` does not
     backtrack across the rule boundary, so `ref` must fail `port_identifier`
     for the repetition to stop at a new direction group. Adding `ref` to the
     SV reserved list (`!reserved_non_keyword_identifier` negative guard) is
     the LRM-faithful completion (IEEE 1800 Table B.1 reserves `ref`) and makes
     part 1 sound — with `ref` reserved, both `input a, b, output c, d` (stops
     at `output`) and `input addr, ref data` (stops at `ref`) parse correctly.
     v2005 keeps its own list (`_v2005:405`, no `ref`), so verilog_2005 stays
     byte-inert.
  - No new rule/token (census unchanged 1468); modport is SV-only
    (`modport_declaration` is `@profiles:["sv_2017","sv_2023"]`). Wire-format
    change ⇒ schema `17`→`18`, release `1.0.169`→`1.0.170`, ledger `SV-0040`.
- **MEASURED GLOBALLY (main sv_2017 lane, guarded re-characterization + adjudication):**
  - **Full external corpus 16,336: pass 9,433 → 9,459 (+26 net raw);**
    29 gains (verilator 13 / ispras-sv-tests 11 / Surelog 2 / sv2v 2 /
    opentitan 1) − 3 raw pass→fail (three `deferred:v2005_profile_lane` ivtest
    `.v` files — `andnot1.v` / `pr1745005.v` / `tern7.v` — that declare
    `reg ref;`: under `sv_2017` `ref` is now correctly reserved, so they reject
    there; they stay v2005-lane-deferred and still ACCEPT under `verilog_2005`
    where `ref` is a legal identifier). timeout 9 → 10 (one opentitan file at
    the 20 s boundary; already failing at baseline — a perf blip, not a
    correctness regression).
  - **Adjudication (graduation) baseline: rejects-valid 481 → 454 (−27)**
    — verilator −13 / ispras-sv-tests −11 / sv2v −2 / Surelog −1;
    **accepts-invalid 21 → 21 (IDENTICAL set — no over-acceptance)**;
    **0 new rejects-valid** (no graduation regression, `comm` verified);
    unexplained 502 → 475.
  - **v2005 byte-inert:** `adjudication_manifest_v2005.tsv` BYTE-IDENTICAL to
    HEAD, `results_v2005.tsv` sorted-identical, `verilog_2005_conformance_gate`
    GREEN (lint orphans 0, corpus 240/0, cert deterministic seeds 0/7/42).
  - **Regression root-caused mid-measurement (the `ref` coupled fix):** the
    modport-only regen first broke `t_interface_virtual.v` /
    `t_interface_virtual_bad.v` (`modport phy(input addr, ref data)`,
    pass→fail); tool-pinned to the missing `ref` reservation; part 2 recovered
    both AND fixed additional `modport(...,ref x)` files (why the drop is −27,
    beyond modport-only's −18).
  - Evidence `docs/tasks/artifacts/sv_corpus_grad/modport_diag/`.
- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — control accepts / defect rejects
    (`furthest_position=58`); 50-row family, ~22 the coherent shared-direction
    sub-defect.
  - [x] **ROOT CAUSE (WHY + WHERE)** — tool-pinned three ways (grammar `:526` +
    probe control/defect + scoped trace) and LRM-verified verbatim; the coupled
    `ref` root cause tool-pinned from the mid-measurement regression.
  - [x] **FIX** — the one-rule LRM repetition + the coupled `ref` reserved-word
    completion (`:402`); both grammar-only, hierarchy level 1.
  - [x] **ADDRESSED (verified)** — before→after measured globally: corpus +26,
    rejects-valid 481→454 (−27), repro matrix + `ref`-modport + the 2 formerly
    regressed files all ACCEPT, correct `ports` array AST.
  - [x] **NO REGRESSION** — 0 new rejects-valid; accepts-invalid set identical;
    v2005 byte-inert (manifest + conformance gate); shape/book gate GREEN; the
    3 raw pass→fail are v2005-lane-deferred + parse under v2005;
    `sv_stimuli_quality_gate` + `sv_cert_recognized_union_gate` confirmed GREEN
    before commit (see the leaf's measured-globally note for the numbers).
  - [x] **LOCKSTEP** — ledger `SV-0040` + contract `1.0.170`/schema 18 + SV book
    changelog-index + tree/TASK_TREE/MEMORY/CHANGES/DEVELOPMENT_NOTES/LIVE this
    commit.

#### `.3.5` — spaced-based number literals absent (IEEE 1800-2017 §5.7.1 — the #6 rejects-valid family, chapter-5 lexical; CROSS-PROFILE)

- **Status: `done`** (`PGEN-SV-CORPUS-GRAD-0020`, session #200,
  2026-07-23; release `1.0.170` → **`1.0.171`**, schema `18` UNCHANGED, ledger
  **`SV-0041`**; committed `d332a7f8`). The third fix cut from the `.3.2`/v2 refreshed family map: after
  `.3.3` (SVA #1) and `.3.4` (modport #3), **number literal spaced-based (ch5),
  22 rows** is the next single-construct family (sv-tests 13 / Surelog 5 /
  ispras-sv-tests 2 / verilator 2). ⭐ Unlike `.3.3`/`.3.4` (SV-only), the fix is
  **cross-profile** — `integral_number` is NOT profile-split, so it heals the
  same lexical gap in the `verilog_2005` lane too (IEEE 1364-2005 §3.5.1 has the
  identical rule; the v2005 rejects-valid baseline includes `always3.1.2I` `5'h 0`).
- **ROOT CAUSE (WHY + WHERE), tool-pinned:**
  - **WHERE:** `grammars/systemverilog.ebnf:439`
    `integral_number := /([0-9][0-9_]*)?'[sS]?[dDhHoObB][0-9a-fA-FxXzZ?_]+/`.
  - **WHY:** `integral_number` is a **single regex terminal**. The layout skipper
    auto-consumes whitespace *before* each terminal but NOT *inside* one match,
    so the terminal cannot span the whitespace the LRM permits **between the size
    and the `'`** and **between the base format and the value**. IEEE 1800-2017
    §5.7.1 (verified verbatim, in-repo LRM md
    `section-1024-…-error-shall-be-reported.md:207-209`): *"The apostrophe
    character and the base format character shall not be separated by any white
    space. … The unsigned number token shall immediately follow the base format,
    optionally preceded by white space."* So whitespace is legal size↔`'` (the
    `.2`-verified valid example `32 'h 12ab_f001`) and base↔value, but NOT within
    the base specifier (`' h` stays illegal).
  - **Probe (before):** control `32'h0000_0001` ACCEPTS; defects `32'h 0000_0001`
    and `32 'h 0000_0001` REJECT at `furthest_position=41` (the base↔value
    space); all 22 family rows + a v2005 `5'h 0` repro REJECT at HEAD. Evidence
    `docs/tasks/artifacts/sv_corpus_grad/spaced_literal_diag/before.txt`.
- **FIX (grammar-only, hierarchy level 1) — one terminal, LRM-faithful:**
  - `integral_number := /([0-9][0-9_]*[ \t]*)?'[sS]?[dDhHoObB][ \t]*[0-9a-fA-FxXzZ?_]+/`
    — add horizontal-whitespace `[ \t]*` at exactly the two LRM-legal seams
    (inside the optional size group after the digits, and before the value); the
    `'[sS]?[dDhHoObB]` base specifier stays contiguous (LRM forbids interior
    white space); the value charset is UNCHANGED so `-` between base and value
    (LRM: illegal) still rejects, and interior whitespace inside the value is
    still rejected (value is one token).
  - No new rule/token (**census UNCHANGED**); the terminal's `body: $1` shape is
    unchanged (whole matched string, now optionally including the interior
    spaces — additive: spaced literals were 100% unparseable before, so no
    retained-text lock or wire-format contract is broken). ⇒ **AST-dump schema
    UNCHANGED** (no structural change); release `1.0.170`→`1.0.171` (parse-behavior
    change), ledger `SV-0041`.
  - The Rust `regex` engine matches terminals linearly (no backtracking), so
    `[ \t]*` introduces **zero** catastrophic-backtracking risk; precedent
    `timeunit_separator_trivia:5538` already embeds `[ \t\r\n]` in a terminal.
  - **Deliberate scope (conservative, documented):** `[ \t]` (horizontal
    whitespace) covers 100% of the observed corpus (all single-line). A literal
    spanning a newline/comment between base and value (LRM §5.3 white space
    includes newlines) is out of this leaf's scope — handling it cleanly needs
    the composite-rule restructure (size/base/value as sub-terminals with
    `trivia` between), a schema-affecting change deferred to avoid a terminal
    that spans lines (which could mask real end-of-line errors).
- **MEASURED GLOBALLY (guarded re-characterization + adjudication, both lanes; 257 s peak 6,054 MB):**
  - **Repro + family:** control `32'h0000_0001` still ACCEPTS; `32'h 0000_0001`
    / `32 'h 0000_0001` / a v2005 `5'h 0` REJECT→ACCEPT; the LRM-illegal
    `8'd -6` (minus between base and value) still correctly REJECTS; **all 22
    family rows flipped REJECT→PASS** (`after_family_rows.txt`).
  - **MAIN sv_2017 lane — full external corpus 16,336: pass 9,459 → 9,585
    (+126** opentitan 83 / iverilog 14 / sv-tests 13 / ispras-sv-tests 8 /
    Surelog 6 / verilator 2**), timeout 10 → 9.** Adjudication: **rejects-valid
    454 → 432 (−22 = the whole #6 family, `comm`-verified as exactly the 22
    keyed spaced-literal rows, ZERO new)**; **accepts-invalid 21 → 21 (BYTE-
    IDENTICAL set — no over-acceptance)**; unexplained 475 → 453; match
    5,655 → 5,677.
  - **V2005 lane (CROSS-PROFILE bonus) — 2,459: pass 2,107 → 2,126 (+19),**
    rejects-valid **135 → 116 (−19, 0 new, `comm`-verified)**, accepts-invalid
    **14 → 14 (IDENTICAL set)**, unexplained 149 → 130.
  - **NO-REGRESSION (the `.3.4` LAW — per-FILE pass-set `comm` diff, not net):
    0 pass→fail in BOTH lanes** (126 + 19 gains, strictly additive — a
    more-permissive terminal cannot break a prior successful parse).
  - Gates: `ast_shape_contract_gate` GREEN (shape inert — `body:$1` unchanged);
    `verilog_2005_conformance_gate` GREEN (curated matrix 240/0 has no
    spaced-literal case; cert byte-inert); `sv_external_corpus_triage_gate`
    14/14; `sv_stimuli_quality_gate` PASS (`closed_loop_replay_targets_total`
    126 → **125**, −1: one `sv_2017` closed-loop replay target is newly
    WITNESSED — a replay-debt gap CLOSED by the fix, feeding `SV-REPLAY-DEBT`; a
    decrease is an improvement, and no target can regress under a strictly-
    more-permissive change; the exact target not pinned — the net −1 is
    conservative);
    `sv_cert_recognized_union_gate` GREEN BYTE-INERT (no rule/token added ⇒
    census 1345 unchanged, canonical `1343/…`, union residual `[]`,
    `fully_certified_via_union`); `systemverilog_parser_book_gate` GREEN.
  - Evidence `docs/tasks/artifacts/sv_corpus_grad/spaced_literal_diag/`.
- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — control accepts; all 22 family rows + v2005 repro
    REJECT at HEAD (`before.txt`); `furthest_position=41`.
  - [x] **ROOT CAUSE (WHY + WHERE)** — grammar `:439` single terminal cannot span
    LRM-legal interior whitespace; LRM §5.7.1 verified verbatim; probe + trace.
  - [x] **FIX** — the one-terminal `[ \t]*` seams (size↔`'`, base↔value; `'`+base
    contiguous; value charset unchanged), grammar-only, hierarchy level 1.
  - [x] **ADDRESSED (verified)** — before→after measured globally both lanes:
    main corpus +126 / rejects-valid −22; v2005 +19 / rejects-valid −19.
  - [x] **NO REGRESSION** — 0 pass→fail per-file both lanes; 0 new rejects-valid;
    accepts-invalid sets byte-identical (21 / 14); cert-union / quality / v2005-
    conformance / shape / book GREEN.
  - [x] **LOCKSTEP** — ledger `SV-0041` + contract `1.0.171` + SV book +
    tree/TASK_TREE/MEMORY/CHANGES/DEVELOPMENT_NOTES/LIVE this commit.

#### `.3.6` — `unique0` if/case qualifier absent (IEEE 1800-2017 §12.4.2 / §12.5.3, A.6.6 `unique_priority` — the #11 rejects-valid family, ch12)

- **Status: `done`** (`PGEN-SV-CORPUS-GRAD-0021`, session #201,
  2026-07-24; release `1.0.171` → **`1.0.172`**, schema `18` UNCHANGED, ledger
  **`SV-0042`**). The fourth fix cut from the `.3.2`/v2 refreshed family map: after
  `.3.3` (SVA #1), `.3.4` (modport #3), `.3.5` (spaced literals #6), the
  **`unique0` (ch12)** family is the cleanest remaining single-construct gap —
  6 keyed rejects-valid rows (ispras 1 / sv-tests 1 / sv2v 1 / verilator 3),
  100% LRM-legal with **zero adjudication ambiguity** (`unique0` is a defined
  keyword, not a tool extension). SV-only (added in IEEE 1800-2009), absent from
  IEEE 1364-2005 — so `verilog_2005` correctly rejects it.
- **REPRODUCE (tool-pinned):** `unique0 if (a == 0) b = 1;` REJECTs at
  `furthest_position=36` and `unique0 case (i) …` REJECTs at
  `furthest_position=49` under `--profile sv_2017`, while the sibling
  qualifiers **`unique if`**, **`priority if`**, and **`unique case`** all
  PASS — the grammar handles `unique`/`priority` but not `unique0`. All 6
  keyed family rows reject at HEAD. Evidence
  `docs/tasks/artifacts/sv_corpus_grad/unique0_diag/before.txt`. (`t_lint_*_bad.v`
  are `_bad` **lint** cases — syntactically valid, correctly rejects-valid.)
- **ROOT CAUSE (WHY + WHERE):**
  - **WHERE:** `grammars/systemverilog.ebnf:5861`
    `unique_priority := kw_unique_58037c00 -> {kind:"unique"} | kw_priority_3345867e -> {kind:"priority"}`
    (already `@profiles: ["sv_2017","sv_2023"]`, :5860).
  - **WHY:** the rule has **only** the `unique` and `priority` branches — **no
    `unique0`** — and no `unique0` token exists (the only qualifier tokens are
    `kw_unique_58037c00 := /unique\b/` :6647 and `kw_priority_3345867e :=
    /priority\b/` :6430). Since `/unique\b/` cannot match `unique0` (no word
    boundary between `e` and `0`), `unique0` is entirely unrecognized. IEEE
    1800-2017 A.6.6 / §12 (verified verbatim, in-repo LRM md
    `section-12-procedural-programming-statements.md:123`):
    `unique_priority ::= unique | unique0 | priority`; §12.4.2 (":227") "The
    keywords unique, unique0, and priority can be used before an if"; §12.5.3
    (":559") the case/casez/casex qualifiers; examples `unique0 if (…)` (:240)
    and `unique0 case(a)` (:595). Line :607's `kw_unique` is the `.unique()`
    **array method** (`array_method_name`) — correctly untouched (`unique0` is
    not an array method). Evidence
    `docs/tasks/artifacts/sv_corpus_grad/unique0_diag/`.
- **FIX (hierarchy level 1 — pure grammar, additive; mirrors `.3.3`'s idiom):**
  add one profile-gated token `@profiles: ["sv_2017","sv_2023"]`
  `kw_unique0 := trivia /unique0\b/` and one branch to `unique_priority`
  (`| kw_unique0 -> {kind:"unique0"}`). Gated exactly like the SVA operators
  (`.3.3`) so `verilog_2005` stays byte-inert (the token is unreachable there;
  gating avoids the census-shift RED that an ungated token triggers). The
  `unique_priority` node shape is unchanged (still `{kind:<string>}`, a new
  enum value only) and `unique0 if`/`case` were previously 100% unparseable ⇒
  **schema `18` UNCHANGED** (additive; the `.3.5` reasoning). Census
  `1468 → 1469` (+1 token), release `1.0.171 → 1.0.172`, ledger `SV-0042`.
- **VERIFIED (measured GLOBALLY, guarded re-characterization + adjudication, both lanes):**
  - **Repro matrix (regen'd parser, fresh probe):** `unique0 if` / `unique0 case`
    REJECT→ACCEPT under `sv_2017` (and `sv_2023`); `verilog_2005` correctly
    REJECTS (`unique0` SV-only); the sibling `unique if` / `priority if` /
    `unique case` all still ACCEPT; AST carries `unique_priority: {kind:"unique0"}`.
    All 6 keyed family rows flip REJECT→PASS. Evidence
    `docs/tasks/artifacts/sv_corpus_grad/unique0_diag/{before.txt,after_repro.txt}`.
  - **MAIN sv_2017 lane — full external corpus 16,336: pass 9,585 → 9,591 (+6).**
    Adjudication: **rejects-valid 432 → 426 (−6** = the whole #11 family,
    `comm`-verified as EXACTLY the 6 keyed unique0 rows, ZERO new**),
    accepts-invalid 21 → 21 (BYTE-IDENTICAL set** — no over-acceptance**)**,
    match 5,677 → 5,683.
  - **NO REGRESSION (the `.3.4` LAW — per-FILE pass-set `comm` diff, not net):
    0 pass→fail** (strictly additive — an added qualifier alternative cannot
    break a prior successful parse); the 6 `fail→pass` are EXACTLY the 6 keyed
    rows (no timeout jitter, debug probe matched the baseline probe type).
    Evidence `docs/tasks/artifacts/sv_corpus_grad/unique0_diag/global_measurement.txt`.
  - **V2005 lane BYTE-INERT:** `adjudication_manifest_v2005.tsv` BYTE-IDENTICAL to
    baseline (the gated token is unreachable under `verilog_2005`); rejects-valid
    116 unchanged.
  - **`sv_cert_recognized_union_gate` GREEN on an evidence-grounded re-baseline:**
    the new `kw_unique0` token is POSITIVELY WITNESSED — total 1,345→1,346 (+1),
    canonical witness 1,328→1,329 (+1), union witness 1,339→1,340 (+1); proof 6,
    canonical UNKNOWN 11, union UNKNOWN 0, residual `[]` all unchanged; **still
    `fully_certified_via_union=true`**, deterministic seeds 0/7/42,
    `unmet_criteria_count=0`. Contract JSON re-baselined same-slice.
  - **`verilog_2005_conformance_gate` GREEN byte-inert** (lint orphans 0, corpus
    matrix 240/0, cert `1115/328/773/14` deterministic seeds 0/7/42).
  - **`ast_shape_contract_gate` 18/0**; **`sv_external_corpus_triage_gate`
    PASS** (no parse failures); **`--lint-grammar` GREEN 1,469 rules, orphans 0**.
  - **`sv_stimuli_quality_gate` PASS** (`closed_loop_replay_targets_total` 125 UNCHANGED (no new closed-loop generation target — the new qualifier is immediately witnessed)).
- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — `unique0 if`/`case` reject (furthest 36/49);
    controls `unique`/`priority` if + `unique` case pass; 6 keyed family rows
    (`before.txt`).
  - [x] **ROOT CAUSE (WHY + WHERE)** — `unique_priority:5861` lacks the
    `unique0` branch/token (`/unique\b/` can't match `unique0`); LRM
    A.6.6/§12.4.2/§12.5.3 verified verbatim.
  - [x] **FIX** — hierarchy level 1 (pure grammar): 1 profile-gated token
    (`kw_unique0`) + 1 branch; LRM-faithful gating (v2005 byte-inert).
  - [x] **ADDRESSED (verified)** — before→after measured globally: corpus +6,
    rejects-valid 432→426, repro matrix flips, correct `{kind:"unique0"}` AST.
  - [x] **NO REGRESSION** — 0 pass→fail per-FILE; 0 new rejects-valid;
    accepts-invalid set BYTE-IDENTICAL; v2005 manifest byte-identical;
    cert-union re-baselined GREEN (still `fully_certified_via_union`);
    v2005-conformance / shape / triage / quality GREEN.
  - [x] **LOCKSTEP** — ledger `SV-0042` + contract `1.0.172` + SV book +
    tree/TASK_TREE/MEMORY/CHANGES/DEVELOPMENT_NOTES/LIVE this commit.

#### `.3.7` — drive/charge strength keywords absent (IEEE 1800-2017 §28.11 / A.8.6 — the drive/charge strength family, ch28; CROSS-PROFILE, same word-boundary bug class as `.3.6`/`.3.3`)

- **Status: `done`** (`PGEN-SV-CORPUS-GRAD-0022`, session #202,
  2026-07-24; release `1.0.172` → **`1.0.173`**, schema `18` UNCHANGED, ledger
  **`SV-0043`**). The fifth fix cut from the refreshed family map. **Map refreshed
  to the v3 vintage this leaf** (the `.3.2`/v2 map was over the pre-burn-down
  543-row population; the current baseline is **426**): re-ran the tracked
  `stimuli/sv/cluster_rejects_valid.py` + `classify_rejects_valid_families.py`
  over the HEAD manifest → `rejects_valid_clusters_v3.{tsv,md}` +
  `rejects_valid_families_v3.{tsv,md}` (426 rows, 222 signatures, 11 families;
  determinism byte-proven cmp ×2; guarded, 5 s, peak 0 MB; the `.3.2`/v2
  artifacts PRESERVED unmodified). The refreshed ranking: OTHER 245, SVA
  (ch16) 35 (drained 101→35 by `.3.3`), interface/modport 25 (drained 50→25 by
  `.3.4`), constraint/randomize 23, compiler directives 22, **drive/charge
  strength 21**, named block/label 20, size/type cast 13, enum base range 9,
  foreach/array 9, coverage bins/cross 4. Of the untouched families,
  **drive/charge strength (21 rows)** is the pick: the only large family that is
  BOTH homogeneous (nearly all rows are `(strengthN, strengthM)` on net decls /
  continuous assigns / gate instances / pullup-pulldown) AND 100% LRM-legal with
  ZERO adjudication ambiguity — the constraint family is heterogeneous (only 6
  of 23 are `dist`; the rest are `randomize()...with{}` on method chains, `union
  soft`, `x inside{}||`), compiler directives are `` `__FILE__``/`` `__LINE__``
  preprocessor macros (route to `SVPP-EXPANSION`, not a grammar fix), and both
  named-block-in-`generate` and bare-range `enum [N:0]` are LRM-QUESTIONABLE
  (adjudication candidates, not clean grammar gaps — probed and set aside).
- **REPRODUCE (tool-pinned, `drive_strength_diag/before.txt`):** all three
  drive-strength sites REJECT under `--profile sv_2017` while a strength-free
  control accepts —
  `wire (strong1, pull0) n = 1;` (furthest 23),
  `assign (highz0, weak1) n = 1;` (furthest 32),
  `nor (highz1, strong0) g(o,a,b);` (furthest 33),
  `pullup (supply1) pu1(a);` (furthest 33),
  and the coupled supply-net forms `supply0 gnd;` (furthest 21) /
  `wire (supply0, supply1) x;` (furthest 23). All 21 keyed family rows reject at
  HEAD.
- **ROOT CAUSE (WHY + WHERE) — the SAME word-boundary/metachar bug class as
  `unique0` (`.3.6`) and `|->`/`|=>` (`.3.3`):**
  - **WHERE:** `grammars/systemverilog.ebnf` `strength:5267`
    (`strength := kw_supply | kw_strong | kw_pull | kw_weak`, tokens
    `/supply\b/`, `/strong\b/`, `/pull\b/`, `/weak\b/`), `drive_strength:2045`
    (`kw_highz`, `/highz\b/`), and `net_type:3549` (`kw_supply -> {kind:"supply"}`).
  - **WHY:** every real SV/Verilog strength keyword is **digit-suffixed** —
    `supply0`/`supply1`, `strong0`/`strong1`, `pull0`/`pull1`, `weak0`/`weak1`,
    `highz0`/`highz1` (IEEE 1800-2017 A.8.6: `strength0 ::= supply0 | strong0 |
    pull0 | weak0`; `strength1 ::= supply1 | strong1 | pull1 | weak1`) — but
    the grammar's strength tokens are **bare** (`/supply\b/` etc.), and there is
    NO word boundary between a letter and a trailing digit, so `/supply\b/`
    cannot match `supply0`. Grammar-wide grep confirms **no digit-suffixed
    strength token exists** (`supply0|supply1|strong0|…` appear ONLY inside the
    `reserved_non_keyword_identifier` regexes :402/:405, never as usable
    tokens). ⇒ the leaf `strength` rule can never match a real strength keyword,
    so the `drive_strength` / `net_strength` / `pulldown_strength` /
    `pullup_strength` wiring (all present: `continuous_assign:1577`,
    gate-inst `:2372-2399`, `net_declaration:3479`) is **entirely DEAD**.
  - **Probe/trace:** `--trace-rules strength,drive_strength,continuous_assign`
    shows `drive_strength` IS entered but its `strength`/`lparen strength …`
    branch fails to consume `strong1`/`highz0`; the bare `kw_strong`/`kw_weak`
    tokens still legitimately serve the SVA `strong(seq)`/`weak(seq)` property
    operators (`:4414`/`:4416`) — so the fix must ADD digit-suffixed tokens,
    NOT repurpose the bare ones. Evidence
    `docs/tasks/artifacts/sv_corpus_grad/drive_strength_diag/`.
- **FIX (hierarchy level 1 — pure grammar, additive; UNGATED = cross-profile,
  the `.3.5` cross-profile precedent — drive strength & supply nets are IEEE
  1364-2005 constructs too):**
  1. Add 10 digit-suffixed strength tokens (convention per `.3.6`'s
     `kw_unique0`): `kw_supply0 := trivia /supply0\b/`, `kw_supply1`,
     `kw_strong0`, `kw_strong1`, `kw_pull0`, `kw_pull1`, `kw_weak0`, `kw_weak1`,
     `kw_highz0`, `kw_highz1`. Ungated (available in every profile — Verilog-2005
     included), because the consumers (`drive_strength`/`net_type`/…) are ungated.
  2. Model `strength0` / `strength1` as SEPARATE rules per IEEE 1800-2017 A.8.6
     (`strength0 ::= supply0 | strong0 | pull0 | weak0`;
     `strength1 ::= supply1 | strong1 | pull1 | weak1`) and rewrite
     `drive_strength` to the LRM's exact SIX opposite-digit combos
     (`(strength0,strength1)` / `(strength1,strength0)` / `(strength0,highz1)` /
     `(strength1,highz0)` / `(highz0,strength1)` / `(highz1,strength0)`,
     `kw_highz0`/`kw_highz1` referenced directly). Likewise
     `pulldown_strength` (single = `strength0`) and `pullup_strength`
     (single = `strength1`) per A.8.6.
     - ⭐ **Why STRICT, not a single permissive `strength`** (measured, not
       assumed — the BE-ALERT gate working): a first pass used one combined
       `strength` accepting any two strength keywords symmetrically. The guarded
       re-adjudication then showed **accepts-invalid 21 → 22 (+1)**: verilator's
       intentional negative `t_strength_strong1_strong1_bad.v`
       (`wire (strong1, strong1) a = 1;`, a same-digit pair that IEEE 1800-2017
       A.8.6 does NOT permit — verilator emits `syntax error, unexpected
       strong1`) was wrongly accepted. Per
       [[feedback_corpus_expected_from_spec_not_fix]] the spec says reject, so the
       model was tightened to the LRM's opposite-digit `strength0`/`strength1`
       grammar. The strict accept-set is a strict SUBSET of the permissive one, so
       it keeps 0 pass→fail vs baseline while restoring accepts-invalid to 21.
  3. `net_type`: replace the dead `kw_supply -> {kind:"supply"}` branch with
     `kw_supply0 -> {kind:"supply0"} | kw_supply1 -> {kind:"supply1"}` (supply
     nets).
  4. Remove the now-orphaned bare tokens `kw_supply` / `kw_pull` / `kw_highz`
     (no remaining reference after 1–3; keeps `--lint-grammar` orphans 0). The
     bare `kw_strong` / `kw_weak` stay (SVA). Census `1469 → 1477` (net +8 =
     +10 tokens +1 `highz` rule −3 orphaned tokens).
  - Additive: drive strength / supply nets were previously 100% unparseable and
    no shape-contract sample exercises `strength`/`drive_strength`/`net_type`-supply
    (the only `net_type` sample is the unrelated `nettype` construct) ⇒ no
    witnessed wire shape changes ⇒ **schema `18` UNCHANGED** (the `.3.5`/`.3.6`
    reasoning). Release `1.0.172 → 1.0.173`, ledger `SV-0043`.
- **VERIFIED (measured GLOBALLY, guarded re-characterization + adjudication, both lanes):**
  - **Strict-model repro matrix (regen'd parser):** same-digit `(strong1, strong1)`
    / `(strong0, strong0)` / mixed-same-digit `(highz0, weak0)` correctly REJECT;
    opposite-digit `(weak0, weak1)` / `(strong0, strong1)` / reversed
    `(supply1, supply0)` / `(highz0, weak1)` / gate `nor (highz1, strong0)` /
    single `pullup (supply1)` / net `supply0 gnd;` all ACCEPT; SVA
    `strong(a ##1 b)` / `weak(...)` still parse (bare `kw_strong`/`kw_weak`
    retained). 10/10 as expected.
  - **MAIN sv_2017 lane — full external corpus 16,336: pass 9,591 → 9,669
    (+78).** Adjudication: **rejects-valid 426 → 406 (−20** = verilator 10 /
    ispras-sv-tests 5 / sv2v 2 / iverilog 2 / Surelog 1, `comm`-verified ZERO
    new**), accepts-invalid 21 → 21 (BYTE-IDENTICAL set** — the
    `t_strength_strong1_strong1_bad` negative correctly rejects under the strict
    model**)**, match 5,683 → 5,703. timeout 9 UNCHANGED, crash 0.
  - **NO REGRESSION (the `.3.4` LAW — per-FILE pass-set `comm`): 0 pass→fail**
    (strict accepts are a subset of the permissive first pass; strictly additive
    vs baseline); the 78 `fail→pass` include the 20 keyed drive-strength family
    rows plus larger files whose sole blocker was a drive-strength / supply-net
    form.
  - **V2005 lane — CROSS-PROFILE HEAL (drive strength & supply nets are IEEE
    1364-2005 constructs): corpus 2,459: pass 2,126 → 2,180 (+54);
    rejects-valid 116 → 62 (−54** = iverilog 50 / ispras-sv-tests 4, ZERO new**),
    accepts-invalid 14 → 14 (BYTE-IDENTICAL), 0 pass→fail.** The single largest
    v2005-lane burn-down to date (the `.3.5` cross-profile precedent, ×3 the
    magnitude).
  - Evidence `docs/tasks/artifacts/sv_corpus_grad/drive_strength_diag/`.
  - **Gates (all GREEN, seeds 0/7/42):** `sv_syntax_closure_gate`
    (defined_rule_count 1469→**1477**, unreachable_rules 0), `ast_shape_contract_gate`
    18/0 (no strength/drive_strength/net_type sample), `sv_external_corpus_triage_gate`
    PASS, `systemverilog_parser_book_gate` PASS; `sv_cert_recognized_union_gate`
    re-baselined GREEN (+8 positively witnessed — total 1346→1354, canonical witness
    1329→1337, union witness 1340→1348; UNKNOWN 11/0, residual `[]`, still
    `fully_certified_via_union`); `verilog_2005_conformance_gate` re-baselined GREEN
    (NOT byte-inert — CROSS-PROFILE; cert `1115/328/773/14` → **`1123/330/779/14`**,
    +8 positively accounted, UNKNOWN 14 identical, corpus matrix 240/0, profile_orphans
    0 — per the drift policy, contract `verilog_2005_conformance_contract_v0.json`
    re-baselined + note appended same-slice); `sv_stimuli_quality_gate` PASS (`closed_loop_replay_targets_total` 125→126, +1: the new `strength0`/`strength1` + `drive_strength` branches are a new closed-loop generation-coverage target — feeds `SV-REPLAY-DEBT`, not a regression).
- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — 6 representative forms reject (furthest
    23/32/33/33/21/23); 21-row homogeneous family (`before.txt`).
  - [x] **ROOT CAUSE (WHY + WHERE)** — `strength:5267`/`net_type:3549`/
    `drive_strength:2045` reference bare tokens that can't match the digit-suffixed
    strength keywords; no digit-suffixed token exists; LRM A.8.6 verified.
  - [x] **FIX** — hierarchy level 1 (pure grammar): 10 digit-suffixed tokens +
    LRM-faithful `strength0`/`strength1` + 6-combo `drive_strength` + `net_type`
    supply0/supply1 + pull*_strength − 3 orphaned tokens; ungated cross-profile.
  - [x] **ADDRESSED (verified)** — before→after measured globally: main +78 /
    rejects-valid −20; v2005 +54 / rejects-valid −54; strict repro matrix 10/10;
    correct `strength0`/`strength1`/`net_type` AST.
  - [x] **NO REGRESSION** — per-FILE pass-set `comm` 0 pass→fail BOTH lanes; 0
    new rejects-valid both lanes; accepts-invalid sets BYTE-IDENTICAL (21/14);
    v2005 CROSS-PROFILE heal measured; gates green (appended at landing).
  - [x] **LOCKSTEP** — ledger `SV-0043` + contract `1.0.173` + SV book +
    tree/TASK_TREE/MEMORY/CHANGES/DEVELOPMENT_NOTES/LIVE this commit.

#### `.3.8` — the SVA cycle-delay RANGE `##[m:n]` / `##[m:$]` is ABSENT (IEEE 1800-2017 A.2.10 — the ch16 family's largest remaining cluster; ⭐ the SIXTH+ instance of the ledger's ALREADY-DOCUMENTED "dropped-delimiter class": LITERAL `[ ]` read as EBNF optional-grouping)

- **Status: `done`** (`PGEN-SV-CORPUS-GRAD-0023`, session #204, 2026-07-25;
  release `1.0.173` → **`1.0.174`**, schema `18` UNCHANGED, ledger **`SV-0044`**).
  The sixth fix cut from the `.3` family map.
- **PICK (measured, not guessed).** The map was re-derived at the current
  **406**-row baseline by filtering the tracked `.3.7`/v3 cluster TSV to the
  HEAD manifest's `divergence:unexplained_rejects_valid` set (406/406 rows
  carried over; the 20 drive-strength rows `.3.7` drained are gone, leaving the
  family at 1) and re-running the tracked
  `classify_rejects_valid_families.py` over it. Refreshed ranking: OTHER 245,
  **SVA implication/property (ch16) 35**, interface/modport 25,
  constraint/randomize 23, compiler directives 22, named block/label 20,
  size/type cast 13, enum base range 9, foreach/array 9, coverage bins/cross 4,
  drive/charge strength 1. Ranking by *normalized stuck signature* (the
  homogeneity lens, sharper than the coarse family bucket) puts **`[ NUM :` at
  the top with 28 rows across 3 suites** — and **24 of those 28 have `##[` in
  the stuck line** (ispras-sv-tests 16 / verilator 6 / Surelog 2). That is the
  largest homogeneous, 100%-LRM-legal, zero-adjudication-ambiguity, pure-grammar
  gap left — the `.3.6`/`.3.7` selection criteria applied unchanged. Rejected
  alternatives, with cause: compiler directives 22 are `` `__FILE__ ``/`` `__LINE__ ``
  preprocessor macros (route to `SVPP-EXPANSION`, not a grammar fix); named
  block/label 20 and enum base range 9 are LRM-QUESTIONABLE (adjudication
  candidates — `.3.7` probed and set them aside); constraint/randomize 23 is
  heterogeneous (`dist` value-ranges ≈4 rows, the rest `randomize()...with{}`
  method chains); the residual 4 rows of the `[ NUM :` cluster are covergroup
  `binsof … intersect {[100:200]}` / `dist { [0:1], [2:5] :/ 2 }` value-range
  forms whose `value_range` rule ALREADY carries the literal-bracket
  alternatives (`value_range_sv_2017:5946`) ⇒ a DIFFERENT root cause, correctly
  out of this leaf's scope.
- **LRM GROUND TRUTH (verified verbatim in the in-repo LRM text BEFORE any
  edit — `docs/systemverilog/2017/txt/section-15-interprocess-synchronization-and-communication.txt:1263`):**

  ```
  cycle_delay_range ::=
  ## constant_primary
  | ## [ cycle_delay_const_range_expression ]
  | ##[*]
  | ##[+]
  ```

  ⭐ **The `[ ]` in alternative 2 are LITERAL SystemVerilog brackets, not BNF
  optional-markers** — and the LRM proves it in its own normative prose, which
  is why this is adjudicable rather than a judgement call:
  - `:1360` — "`##[*]` is used as an equivalent representation of `##[0:$]`."
  - `:1362` — "`##[+]` is used as an equivalent representation of `##[1:$]`."

  Alternatives 3/4 (`##[*]`, `##[+]`) are *unambiguously* literal-bracket
  tokens, and the LRM equates them to `##[0:$]` / `##[1:$]` — forms that are
  only writable if alternative 2's brackets are literal too. Corroborated by
  ~15 source examples across the LRM: `@(negedge clk) d ##[2:5] e;` (§9:896),
  `req ##[4:32] gnt` / `req ##[4:$] gnt` (:1435/:1439), `w ##1 x ##[2:10] y;`
  (:1673), `(te1 ##[1:5] te2) and (te3 ##2 te4 ##2 te5)` (:2524).
- **⭐ WHY THE EXTRACTOR GOT IT WRONG (the NEW metachar class — motivating
  evidence for `LRM-GRAMMAR-FIDELITY`).** The LRM's BNF meta-notation uses
  `[ ]` for BOTH literal brackets AND optional items (compare
  `sequence_instance ::= ps_or_hierarchical_sequence_identifier [ ( [
  sequence_list_of_arguments ] ) ]` at :1273, where every bracket IS an
  optional-marker). In the published PDF the two are distinguished
  typographically (literal terminals are set in a different face); the
  PDF→text conversion **destroys that signal**, leaving the two uses
  character-identical. The extractor resolved alternative 2 the wrong way.
  ⛔ **CORRECTION OF RECORD (made during this leaf, before landing): this is NOT
  a new class.** The bug ledger already NAMES it — "the documented
  dropped-delimiter class" — with at least FIVE prior instances:
  `stream_concatenation`'s literal `{ }` + `[ ]` (`SV-0002`), the covergroup
  `trans_range_list` repeat forms, the `boolean_abbrev` sequence-repetition
  family, the six covergroup-adjacent SVA bounded-property operators, and
  `value_range` (whose in-grammar comment reads "Same fix for sv_2023 across
  all 4 bracketed range variants"). `cycle_delay_range` is the **sixth+**.
  ⭐⭐ **The damning detail: `boolean_abbrev` lives in A.2.10 — the SAME Annex A
  subclause as `cycle_delay_range`.** A fix landed in that very subclause and
  did not sweep its immediate neighbours, leaving this one broken for another
  month until a corpus row happened to hit it. Every instance to date has been
  found REACTIVELY, one construct at a time, by whatever the corpus tripped
  over. That is the actionable finding: the class does not need more
  motivating evidence, it needs an **exhaustive Annex-A bracket/brace sweep**
  — which is precisely what the director-gated `LRM-GRAMMAR-FIDELITY` tree
  should own. The PDF→text point below explains WHY the class exists; it does
  not make the class new.
- **ROOT CAUSE (WHY + WHERE) — grammar source, `grammars/systemverilog.ebnf`:**
  - **WHERE:** `cycle_delay_range:1703`.

    ```
    cycle_delay_range := kw_token_93ac8946 constant_primary
                                -> {kind: "primary",      body: $2}
                      | kw_token_93ac8946 ( cycle_delay_const_range_expression )?
                                -> {kind: "paren_range",  body: $2}
                      | kw_token_71b8cf7e   -> {kind: "token_71b8cf7e"}   # "##[*]"
                      | kw_token_9768502a   -> {kind: "token_9768502a"}   # "##[+]"
    ```

  - **WHY:** alternative 2 renders the LRM's literal `[ … ]` as PGEN's
    `( … )?` **optional-group metasyntax**, so the rule matches `##` followed by
    an *optional, bracket-less* `constant_expression : constant_expression`.
    No alternative can consume the `[` of a real `##[1:3]`: alt 1 needs a
    `constant_primary` (cannot start with `[`), alt 2 takes the empty option and
    leaves `[1:3]` to `seq_unary`, alts 3/4 require the exact literal `##[*]` /
    `##[+]`. ⇒ **`##[m:n]` and `##[m:$]` are entirely unparseable in every
    profile** — the bracketed cycle-delay range is DEAD, while unbracketed
    `##1` / `##2` (`cycle_delay:1696`) parse fine, which is exactly the observed
    corpus split (`te3 ##2 te4` passes inside files that fail at `##[1:5]`).
    The `{kind: "paren_range"}` annotation name is itself a tell that the
    synthesis read the brackets as a grouping. Note
    `cycle_delay_const_range_expression:1700-1701` is INTACT and already carries
    both the `expr : expr` and `expr : $` forms — only the bracket wrapper is
    wrong, so the fix is confined to one rule.
- **REPRODUCE (tool-pinned, `cycle_delay_range_diag/before.txt`)** — 7 minimal
  LRM-sourced cases under `--profile sv_2017`, `rc` = the probe's own exit code.
  **All 4 bracketed forms REJECT; all 3 controls ACCEPT:**

  | case | construct | LRM source | rc | furthest |
  |---|---|---|---|---|
  | `r1_range_basic.sv` | `data ##[1:3] gnt` | §16 / :1673 | 1 | 121 |
  | `r2_range_seq.sv` | `d ##[2:5] e` | §9 :896 | 1 | 122 |
  | `r3_range_dollar_hi.sv` | `req ##[4:$] gnt` | :1439 | 1 | 113 |
  | `r4_range_spaced.sv` | `te1 ## [2:5] te2` | ispras 16.09.08_01 | 1 | 133 |
  | `c1_control_unbracketed.sv` | `te3 ##2 te4` (alt 1) | — | **0** | — |
  | `c2_control_star.sv` | `a ##[*] b` (alt 3) | :1360 | **0** | — |
  | `c3_control_plus.sv` | `a ##[+] b` (alt 4) | :1362 | **0** | — |

  The controls are the regression tripwire: `##[*]`/`##[+]` are literal tokens
  reachable ONLY by backtracking past alternative 2, so they must still accept
  after the edit.
- **TRACE (the WHY, `cycle_delay_range_diag/trace_r1_before.txt`)** —
  `PGEN_TRACE_VERBOSITY=debug … --trace-rules cycle_delay_range,cycle_delay_const_range_expression`
  on `r1`. Position **121 is exactly the `[`** (the `##` spans 119..121), which
  is precisely the reported `furthest_position=121`. Branch by branch:

  ```
  🚪 Entering branch 1/4 for rule 'cycle_delay_range' at position 118
  ✅ Rule 'kw_token_93ac8946' successfully parsed from 118 to 121 (consumed 3 bytes: ' ##')
  🔙 Speculative parse failed … rule_stack: [… "seq_delay_expr", "cycle_delay_range",
       "constant_primary", "constant_primary_sv_2017", "primary_literal"]
       input_context: "posedge clk) data ##[1:3] gnt; endproper"
  🚪 Entering branch 2/4 for rule 'cycle_delay_range' at position 118
  ✅ Leaving branch 2/4 for rule 'cycle_delay_range' at position 121 (success)
  ```

  ⭐ **That `✅ Leaving branch 2/4 … at position 121 (success)` line is the
  misreading caught in the act** — and it is EMPIRICAL, not inferred: branch 2
  *succeeds* having consumed only `##` (118→121) **with the optional group
  matched EMPTY**, leaving the literal `[1:3]` to `seq_unary`, which cannot
  start on `[`. Branch 1 consumes `##` then dies in `constant_primary` at 121
  (a `[` is not a primary); branches 3/4 are terminal mismatches (`##[*]` /
  `##[+]` vs the input `##[1`). ⇒ **no branch can consume a literal `[` after
  `##`**, so `##[m:n]` / `##[m:$]` are unparseable.
- **FIX (planned — hierarchy level 1, pure grammar, additive; UNGATED):**
  replace alternative 2's optional-group with the LRM's literal brackets —
  `kw_token_93ac8946 lbrack cycle_delay_const_range_expression rbrack`
  (annotation `{kind: "range", body: $3}`; the `paren_range` name retires with
  the misreading). `lbrack`/`rbrack` are existing tokens (used by
  `value_range_sv_2017:5946`, the same-class precedent). No new rule, no new
  token ⇒ census UNCHANGED. Ungated because `cycle_delay_range`'s consumers
  (`seq_delay_expr:5055`) are SVA-side and already profile-scoped, and the LRM
  form is identical in 1800-2017 and 1800-2023. Expected: additive (the
  bracketed form is currently 100% unparseable ⇒ no witnessed wire shape can
  change ⇒ **schema `18` expected UNCHANGED**, the `.3.5`/`.3.6`/`.3.7`
  reasoning). Release `1.0.173 → 1.0.174`, ledger `SV-0044` — both to be
  confirmed at landing.
- **VERIFIED (measured GLOBALLY, guarded, both lanes; evidence
  `cycle_delay_range_diag/{before,after,global_measurement}.txt`):**
  - **Repro matrix flips (`after.txt`):** all 4 bracketed forms REJECT→ACCEPT;
    **all 3 controls still ACCEPT** — including `##[*]`/`##[+]`, which are
    reachable ONLY by backtracking PAST the edited alternative and were
    therefore the real tripwire. Under `--profile verilog_2005` the SVA forms
    correctly still reject. AST is LRM-faithful: `##[1:3]` →
    `{kind:"range", lo:1, hi:3}`, `##[4:$]` → `{kind:"dollar_hi", lo:4}`.
  - **MAIN sv_2017 lane — full external corpus 16,336: pass 9,669 → 9,692
    (+23)**, fail 6,658 → 6,634. Adjudication: **rejects-valid 406 → 383
    (−23, ZERO new — verified by set difference, not by net count)**,
    **accepts-invalid 21 → 21 BYTE-IDENTICAL set** (no over-acceptance),
    unexplained 427 → 404, match 5,703 → 5,726.
  - **NO REGRESSION (the `.3.4` LAW — per-FILE pass-set diff): 0 pass→fail and
    0 pass→timeout**, key sets identical (16,336/16,336). Only two adjudication
    transitions exist in the whole manifest: 23 ×
    `unexplained_rejects_valid → match` and 1 × `deferred:chained_only →
    divergence:explained_timeout` (the jitter row below — an EXPLAINED bucket,
    so the honest baseline is untouched).
  - **TARGETING — strictly surgical:** the 23 flips are EXACTLY keyed `##[`
    rows; **flipped-but-not-keyed = 0** (no unrelated file changed verdict).
  - ⚠️ **The 1 fail→timeout row is PROVEN JITTER, not a regression**
    (`alert_handler_reg_top.sv`, 671 KB / 22,322 lines, opentitan autogen):
    it contains **ZERO** occurrences of `##`, so `cycle_delay_range` is never
    reached for it; re-run ALONE without 8-way contention it completes in
    **15 s with verdict FAIL** — its TRUE verdict is unchanged, it merely
    crossed the 20 s wall under parallel load. `jobs=8` was deliberately NOT
    tuned down, so the run stays methodologically identical to the baseline.
  - **V2005 lane BYTE-INERT:** 2,459 files, pass 2,180 / fail 279 / timeout 0 —
    ZERO per-file transitions, and `adjudication_manifest_v2005.tsv` is
    **BYTE-IDENTICAL** to baseline (`cmp` clean). SVA is unreachable under
    `verilog_2005`.
  - ⚠️ **Path-normalization requirement (new, banked):** the baseline
    `results.tsv` rows carry absolute paths under
    the pre-move home-directory checkout location because they predate the
    director's move of all GitHub projects to a 4 TB SSD
    (`/Volumes/SSD/…`, confirmed 2026-07-25). The per-file diff normalizes to
    the `/pgen/`-relative form; **a naive diff reports all 16,336 rows as
    changed**, which would read as total regression or total heal. Since the
    `.3.4` LAW makes this diff the binding no-regression proof for every `.3.x`
    leaf, the requirement is recorded in `DEVELOPMENT_NOTES.md` too.
- **⭐ SIBLING DEFECT SURFACED (routed to `.3.9`, deliberately NOT folded in):**
  1 of the 24 keyed rows did not flip —
  `verilator/test_regress/t/t_sequence_sexpr_unsup.v`. The fix DID work there:
  the file now parses PAST its old stuck point and dies later at
  `furthest_position=1112`, which is exactly `## [*] b;` — a **SPACED**
  `## [*]`. Minimal repro on the post-fix parser: `##[*]` ACCEPT / `## [*]`
  REJECT; `##[+]` ACCEPT / `## [+]` REJECT; `##[1:2]` and `## [1:2]` BOTH
  ACCEPT. Root cause is a DIFFERENT mechanism in the same rule: alternatives
  3/4 are FUSED literal tokens (`kw_token_71b8cf7e := trivia "##[*]"`,
  `kw_token_9768502a := trivia "##[+]"`) which cannot admit the whitespace
  SystemVerilog's free-form lexing permits between `##` and `[`. ⭐ That this
  leaf's STRUCTURED alternative accepts BOTH spacings is independent evidence
  the structured model is correct. Deferred because nothing regresses by
  deferring, and retiring the fused tokens changes AST kinds
  (`{kind:"token_71b8cf7e"}` → `{kind:"star"}`) — a shape/schema decision that
  deserves its own leaf.
- **GATES (all GREEN, seeds 0/7/42):**
  - `sv_syntax_closure_gate` — `defined_rule_count` **1477 UNCHANGED**
    (zero new rules/tokens, as designed), `unreachable_rules: 0`.
  - `ast_shape_contract_gate` — PASS, **schema `18` holds** (no locked sample
    referenced the retired `paren_range` kind).
  - `sv_cert_recognized_union_gate` — GREEN **with NO re-baseline needed**:
    `union_witness 1348` == `expected_union_witness`, `union_residual_rules []`
    == expected, canonical UNKNOWN=11 / union UNKNOWN=0, still
    `fully_certified_via_union: true`, `unmet_criteria_count: 0`, deterministic
    across seeds. ⇒ `systemverilog_recognized_cert_union_contract.json`
    UNTOUCHED.
  - `verilog_2005_conformance_gate` — GREEN **BYTE-INERT**: cert
    `1123/330/779/14` IDENTICAL to baseline, matrix 240/0, `profile_orphans 0`
    ⇒ `verilog_2005_conformance_contract_v0.json` UNTOUCHED. (Contrast `.3.7`,
    which was cross-profile and had to re-baseline BOTH contract JSONs; this
    fix is SVA-only so neither moves.)
  - `sv_external_corpus_triage_gate` — PASS.
  - `systemverilog_parser_book_gate` — PASS.
  - `sv_stimuli_quality_gate` — PASS (peak 12,425 MB / 1,878 s).
    ⚠️ `closed_loop_replay_targets_total` **126 → 127 (+1)** — honest and
    expected: the bracketed alternative is a NEW closed-loop generation target
    (the generator must now emit `##[m:n]` to witness it, whereas the old
    optional-group branch was satisfied trivially by a bare `##`). Same
    accounting as `.3.7`'s +1; it feeds `SV-REPLAY-DEBT`, and a new
    generation-coverage target is not a regression.
- **Acceptance Checklist (enforced — to be completed at landing)**
  - [x] **REPRODUCE / ISSUE** — 4 LRM-sourced bracketed forms reject
    (furthest 121/122/113/133) while all 3 controls (`##2`, `##[*]`, `##[+]`)
    accept; 24 keyed corpus rows (`before.txt`).
  - [x] **ROOT CAUSE (WHY + WHERE)** — `cycle_delay_range:1703` alt 2 renders
    the LRM's LITERAL `[ … ]` as PGEN optional-group `( … )?`; LRM A.2.10 +
    the :1360/:1362 `##[*]`≡`##[0:$]` prose + ~15 source examples verified
    verbatim; trace shows branch 2 SUCCEEDING on `##` alone with the group
    matched empty at the exact `furthest_position=121` (`trace_r1_before.txt`).
  - [x] **FIX** — alt 2 → `kw_token_93ac8946 lbrack
    cycle_delay_const_range_expression rbrack -> {kind: "range", body: $3}`
    (`grammars/systemverilog.ebnf:1705`); existing tokens only, zero new
    rules/tokens, census UNCHANGED; LRM branch order preserved.
  - [x] **ADDRESSED (verified)** — before→after measured globally, both lanes:
    repro matrix 4 REJECT→ACCEPT + 3 controls held; main corpus pass
    9,669→9,692 (+23); rejects-valid 406→383 (−23); correct `{kind:"range"}` /
    `{kind:"dollar_hi"}` AST.
  - [x] **NO REGRESSION** — per-FILE pass-set diff: **0 pass→fail, 0
    pass→timeout**; **0 new** rejects-valid (set difference, not net count);
    accepts-invalid set BYTE-IDENTICAL (21); v2005 manifest BYTE-IDENTICAL;
    the single fail→timeout row proven jitter (zero `##` in the file; 15 s solo,
    verdict unchanged); all 7 gates green with BOTH contract JSONs untouched.
  - [x] **LOCKSTEP** — ledger `SV-0044` + contract `1.0.174` (identity + the
    schema-18 KEEP note) + SV book changelog-index +
    tree/TASK_TREE/MEMORY/CHANGES/DEVELOPMENT_NOTES/LIVE this commit.

#### `.3.9` — the SPACED `## [*]` / `## [+]` cycle-delay abbreviations reject (IEEE 1800-2017 A.2.10 alts 3/4 — fused literal tokens vs SV's free-form lexing; the sibling `.3.8` surfaced)

- **Status: `active`** (session #205, 2026-07-25 — `PGEN-SV-CORPUS-GRAD-0024`).
  Opened by `.3.8` (session #204, 2026-07-25) from a MEASURED non-flip, not from
  speculation. Deliberately NOT folded into `.3.8`: different mechanism, nothing
  regresses by deferring, and the fix changes AST kinds (a shape/schema decision
  of its own).
- **EVIDENCE ALREADY BANKED (`cycle_delay_range_diag/global_measurement.txt`).**
  `.3.8` healed 23 of its 24 keyed rows; the 24th,
  `verilator/test_regress/t/t_sequence_sexpr_unsup.v`, now parses PAST its old
  stuck point and dies at `furthest_position=1112` = `## [*] b;`. Minimal repro
  on the post-`.3.8` parser (`--profile sv_2017`):

  | form | verdict |
  |---|---|
  | `##[*]` | ACCEPT |
  | `## [*]` | **REJECT** |
  | `##[+]` | ACCEPT |
  | `## [+]` | **REJECT** |
  | `##[1:2]` | ACCEPT |
  | `## [1:2]` | ACCEPT ← `.3.8`'s structured alternative admits BOTH spacings |

- **ROOT CAUSE (WHY + WHERE, already pinned):** `cycle_delay_range:1707/:1709`
  alternatives 3/4 reference FUSED literal tokens
  `kw_token_71b8cf7e := trivia "##[*]"` (`:6655`) and
  `kw_token_9768502a := trivia "##[+]"` (`:6659`). A fused multi-character
  literal cannot admit interior whitespace, but SystemVerilog is free-form and
  `##`, `[`, `*`, `]` are separate lexical tokens — so any spacing the author
  chooses is legal. ⭐ Same *family* as `.3.8` (the extractor mishandling
  A.2.10's literal brackets) but the opposite failure mode: `.3.8` was brackets
  wrongly treated as METASYNTAX, this is brackets wrongly FUSED into an atom.
- **FIX (planned — pure grammar, mirrors `.3.8`):** replace both fused tokens
  with structured alternatives using the existing `star` / `plus` tokens, which
  `consecutive_repetition:~5050` already uses in exactly this
  `lbrack star rbrack` / `lbrack plus rbrack` shape:

  ```
  | kw_token_93ac8946 lbrack star rbrack  -> {kind: "star"}
  | kw_token_93ac8946 lbrack plus rbrack  -> {kind: "plus"}
  ```

  Then retire the two now-orphaned fused tokens (keeps `--lint-grammar`
  orphans 0; census 1477 → 1475).
- **⚠️ OPEN DECISION the leaf must adjudicate first (why it is not a drive-by):**
  this RENAMES the emitted AST kinds `{kind:"token_71b8cf7e"}` →
  `{kind:"star"}` and `{kind:"token_9768502a"}` → `{kind:"plus"}`. Unlike
  `.3.8`'s `paren_range` (which was unreachable in practice, hence additive),
  **these two kinds ARE currently produced** — `##[*]` / `##[+]` parse today, so
  any consumer keyed on them would break. That is a genuine wire-shape change ⇒
  expect a **schema bump 18 → 19** and a contract/book entry, unless the leaf
  measures that no shape-contract sample and no downstream contract references
  them. Resolve on measurement, per `.3.7`'s strict-vs-permissive precedent.
  (Retiring the opaque `token_<hash>` kind names is itself a readability win —
  they are extractor artifacts, not LRM vocabulary.)
- **SCOPE:** ≥1 corpus row (`t_sequence_sexpr_unsup.v`); a grammar-wide sweep
  for OTHER fused `kw_token_<hash> := trivia "<multi-char containing brackets>"`
  literals MUST run in the same leaf — a one-site fix here would repeat exactly
  the mistake `.3.8` documented. ⛔ Per `.3.8`'s correction of record, the
  delimiter class is ALREADY DOCUMENTED in the bug ledger and has recurred
  SIX+ times, always found reactively one construct at a time — and the
  `boolean_abbrev` fix landed in **A.2.10, the same subclause as
  `cycle_delay_range`**, without sweeping its neighbours. This leaf is the
  fused-literal FACE of that class; the bracket-as-metasyntax face was `.3.8`.
  Both argue the same conclusion: `LRM-GRAMMAR-FIDELITY` needs an EXHAUSTIVE
  Annex-A delimiter sweep, not another isolated repair.

##### `.3.9` EXECUTION LOG (session #205, 2026-07-25, `PGEN-SV-CORPUS-GRAD-0024`)

- **REPRODUCED at HEAD `d0d2152f`** (release probe, `--profile sv_2017`;
  `artifacts/sv_corpus_grad/fused_bracket_literal_diag/before.txt`) — the charter's
  6-row matrix confirmed EXACTLY as predicted:
  `##[*]` ACCEPT · `## [*]` **REJECT** (`furthest_position=214`) ·
  `##[+]` ACCEPT · `## [+]` **REJECT** (`furthest_position=146`) ·
  `##[1:2]` ACCEPT · `## [1:2]` ACCEPT.
  (The two spaced-range ACCEPTs also re-confirm the probe is post-`.3.8` vintage.)
- **ROOT CAUSE (WHY + WHERE) tool-pinned with a DIFFERENTIAL** — not just "the
  branch fails", but *the same branch succeeding on the tight spelling*
  (`trace_r1_before.txt`, `--trace-rules cycle_delay_range` at `debug`):
  - on the SPACED input, ALL FOUR branches fail at the `##` position:
    `❌ Branch 3/4 for rule 'cycle_delay_range' failed at position 209`
    (bytes 209.. = `' ## [*] '`; branch 2 reaches the `*` = the reported
    `furthest_position=214`);
  - on the TIGHT input, the SAME branch WINS:
    `🏁 Rule 'cycle_delay_range' selected branch 3/4 consuming 6 chars`.
  ⇒ the mechanism is the fused literal, isolated to whitespace alone.
  WHERE: `kw_token_71b8cf7e := trivia "##[*]"` (`:6655`) and
  `kw_token_9768502a := trivia "##[+]"` (`:6659`) — each fusing FOUR distinct
  lexical tokens. The layout skipper (`trivia`) runs only BEFORE a terminal,
  never inside one match — the SAME engine property `.3.5` root-caused for
  spaced-based number literals.
- **⭐ THE MANDATED SWEEP RAN, and was WIDENED past the charter's bracket filter**
  (`sweep_fused_delimiter_literals.txt`). The governing law is not about brackets;
  it is IEEE 1800-2017 **§5.3** verbatim: white space is "ignored except when
  [it serves] to separate other lexical tokens" ⇒ the only question per fused
  literal is *one lexical token, or several?* So ALL **52** multi-character
  literal token definitions were adjudicated (plus 48 inline-literal hits):
  - **46 correct as fused** — single operator glyphs (`!=`, `<<<=`, `|->`, …).
  - **2 adjudicated correct as fused WITH a named LRM cause** — `attr_open "(*"`
    (`:6063`) / `attr_close "*)"` (`:6062`): two-character delimiters on the
    §5.4 `/*`-block-comment precedent, **36/36 LRM attribute examples written
    tight, zero spaced**, and un-fusing would collide with the LRM's own separate
    `event_control ::= … | @ (*)` production. **NO CHANGE** — a speculative
    un-fusing here would have changed acceptance with no LRM support.
  - **2 = this leaf's defect** (`##[*]`, `##[+]`) — fixed.
  - **0 fused grammar literals hiding in rule bodies** — all 48 inline hits are
    `@sample:`/`@probe_sample:` payloads, `@fact_kind:` descriptions, comments,
    or regex-terminal bodies.
  ⇒ the bracket/brace face of the class is now **EXHAUSTIVELY closed** for this
  grammar — the charter's "do not repeat `.3.8`'s one-site mistake" requirement
  is discharged by measurement, not by assertion.
- **⭐⭐ THE SWEEP SURFACED A NEW MEASURED DEFECT UNDER A DIFFERENT LAW → new leaf
  `.3.10`** (see below). Found *only* because the filter was widened.
- **OPEN DECISION RESOLVED ON MEASUREMENT — schema `18` → `19`.** The charter
  required this be adjudicated first, not assumed:
  - **Locked shape-contract samples: ZERO references.** Machine-checked all 31
    `samples` in `rust/test_data/ast_shape_contract/systemverilog_v1.json` for
    `token_71b8cf7e` / `token_9768502a` / `##[` — no hit. The two grep hits in
    that file are both in `calibration_history` **prose**, not assertions.
  - **Downstream contract: no LIVE normative table references them.** The two
    hits (`PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md:1373`, book
    `changelog-index.md:22`) sit inside **era-dated release-highlights blocks**
    (Release 1.0.77 / 1.0.54). ⭐ Those correctly keep saying `paren_range` and
    `token_*`: at 1.0.77 the kinds genuinely WERE those, and the project's
    convention (and `PGEN-RGX-SCHEMA-DOCSYNC-0001`'s ruling) is *era-dated rows
    untouched*. **This is NOT a `.3.8` lockstep gap** — verified against
    `git show d0d2152f`.
  - **BUT the kinds ARE reachable and CURRENTLY EMITTED** — proven by AST dump,
    not inferred: `##[*]` → `{"kind": "token_71b8cf7e"}`, `##[+]` →
    `{"kind": "token_9768502a"}` (1 occurrence each, under
    `sequence_expr…delay`). This is the decisive difference from `.3.8`, whose
    retired `paren_range` was *unreachable in practice* (it needed a bare `##`).
  ⇒ renaming a **reachable, emitted** kind is a real wire-shape change for a
  downstream consumer (Nexsim), which is exactly what the schema version exists
  to signal. Under-signalling a breaking rename to save a version number would be
  the wrong trade. **BUMP 18 → 19.**
- **FIX (grammar-only, fix-hierarchy tier 2 = grammar, mirrors an in-grammar
  precedent):** `cycle_delay_range:1707/:1709` alternatives 3/4 rewritten
  structurally, and the two fused tokens RETIRED (`:6655`/`:6659`, replaced by an
  explanatory comment block):

  ```
  | kw_token_93ac8946 lbrack star rbrack  -> {kind: "star"}
  | kw_token_93ac8946 lbrack plus rbrack  -> {kind: "plus"}
  ```

  ⭐ This is not a novel shape: `consecutive_repetition:1377` already spells
  `lbrack star rbrack -> {kind: "star"}` / `lbrack plus rbrack -> {kind: "plus"}`
  verbatim — so the fix reuses existing tokens AND an existing house idiom, and
  the resulting kind names (`star`/`plus`) are already this grammar's vocabulary
  rather than extractor hash artifacts. LRM branch order preserved. No new rules
  or tokens; census **1477 → 1475** (the 2 retired tokens).
  Tournament safety checked: on `##[1:2]` alt 2 still wins (alt 3/4 need a bare
  `*`/`+`); on `##[*]` alt 2 fails inside `cycle_delay_const_range_expression`
  and alt 3 wins — no new ambiguity.
- **`--lint-grammar` after the edit: CLEAN** — `1475 rules`, `non_terminating=0`,
  `ordered_choice_shadowing=0`, `unreachable_rules=0`, `undefined_references=0`,
  `unbound_fact_kinds=0`, **`profile_orphans=0`** (so retiring the tokens stranded
  nothing). Regen green (`focus_systemverilog`, guard exit 0, peak 2,288 MB / 71 s;
  return-annotation inventory 2,280 entries).
- **BASELINE PINNED for the before→after** (post-`.3.8`, measured from the tracked
  manifests): corpus 16,336 files — pass **9,692** / fail 6,634 / timeout 10 /
  crash 0 (59.3%); adjudication match **5,726**, `unexplained_rejects_valid`
  **383**, `unexplained_accepts_invalid` **21** (total unexplained 404).
- **KEYED TARGET POPULATION = exactly 1 row** (mechanically derived: the
  rejects-valid rows whose file contains `##`+whitespace+`[`) —
  `verilator/test_regress/t/t_sequence_sexpr_unsup.v`, the very file `.3.8`
  surfaced. ⭐ It is an ideal targeted oracle: it exercises all ten spaced
  cycle-delay forms in one file (`## DELAY`, `## ( DELAY )`, `## [1:2]`,
  `## [*]`, `## [+]`, each also with a leading `a`), and today dies at
  `furthest_position=1112` = the `## [*]` on line 59. **Honest scope statement,
  stated up front rather than discovered later: the corpus delta from this leaf is
  at most +1 pass.** Its value is the CLASS it removes (every legal spaced
  spelling), not the row count — and the exhaustive sweep above is the larger
  deliverable.
- **VERIFIED — repro matrix AFTER** (`after.txt`, release probe relinked 15:46:15,
  mtime-asserted NEWER than the 15:24:20 parser): all 6 rows ACCEPT (`## [*]` and
  `## [+]` REJECT→ACCEPT, the 4 controls held); `verilog_2005` still REJECTS both.
- **⭐ THE CLASS IS CLOSED, not just the two charted forms** — 9 spacing variants
  ALL ACCEPT: `##[*]`, `## [*]`, `##[ *]`, `##[* ]`, `##  [  *  ]`, `##<TAB>[*]`,
  `##[+]`, `## [+]`, `##[ + ]`.
- **⭐ NOT over-permissive** — 5 near-miss negatives ALL still REJECT: `##[]`,
  `##[*`, `##[**]`, `##[+*]`, `##[1:2:3]`. (A more-permissive terminal change
  must be shown not to have opened the door too far; measured, not argued.)
- **AST shape** — `{kind:"star"}` / `{kind:"plus"}` emitted for both spacings,
  with **zero** residual `token_71b8cf7e` / `token_9768502a` occurrences.
- **GLOBAL MEASUREMENT** (`global_measurement.txt`; DEBUG probe, matching the
  baseline probe type per the `.3.6` requirement; guard exit 0, 5,700 MB / 217 s):
  main corpus 16,336 — pass 9,692 → **9,694**, fail 6,634 → 6,634, timeout 10 → 8.
  ⭐ **HONEST ATTRIBUTION: the yield is +1, not +2**, exactly the ceiling stated
  above before measuring. The per-FILE diff shows only THREE transitions:
  `fail→pass` on the keyed row (mine), plus `timeout→pass` and `timeout→fail` on
  two files containing **zero** `##` occurrences — so `cycle_delay_range` is never
  reached and the change provably cannot affect them. Both re-run SOLO (15 s /
  17 s against the 20 s wall) and each moved TOWARD its true verdict; one is the
  SAME opentitan file `.3.8` proved was jitter in the OPPOSITE direction, so the
  two leaves are consistent, not contradictory. The artifact records "do not cite
  +2 as this fix's yield".
- **ADJUDICATION**: match 5,726 → 5,727; `unexplained_rejects_valid` 383 → **382**
  (SET difference: 1 healed = the keyed row, **0 NEW**);
  `unexplained_accepts_invalid` 21 → 21 with the **SET BYTE-IDENTICAL**;
  `explained_timeout` 10 → 8 and `deferred:chained_only` 5,266 → 5,268 (the two
  jitter rows re-classifying); **all 9 other classes byte-identical**.
  ⇒ **rejects-valid graduation baseline is now 382.**
- **NO REGRESSION (the `.3.4` LAW — per-FILE pass-set diff, not net counts;
  paths normalized per `.3.8`): 0 pass→fail, 0 pass→timeout, 0 pass→crash.**
- **`verilog_2005` lane RE-RUN, not inferred** (it had to be: the retired tokens
  WERE in the v2005 rule universe): 2,459 files 2,180/279/0;
  `results_v2005.tsv` **BYTE-IDENTICAL** (path-normalized) and
  `adjudication_manifest_v2005.tsv` **BYTE-IDENTICAL**; v2005 unexplained stays 76
  (62 rejects-valid + 14 accepts-invalid). Correct by construction — the rule is
  reachable only through SV-gated SVA, so the census moves but parsing does not.
- **GATES**
  - `sv_syntax_closure_gate` — PASS, `defined_rule_count` **1475** (the −2, as
    designed), `unreachable_rules: 0`. (`unreachable_branches: 2` is at its
    tracked cap of 2 — verified PRE-EXISTING: the contract
    `systemverilog_syntax_closure_contract.json` is untouched by this leaf.)
  - `ast_shape_contract_gate` — PASS **18/18**.
  - `sv_cert_recognized_union_gate` — first run RED on contract drift **exactly as
    predicted**, then GREEN on an evidence-grounded re-baseline. Measured
    identically at seeds 0/7/42: canonical total **1352** / proof **6** / witness
    **1335** / UNKNOWN **11** / `sample_parse_failures 0`; union total 1352 /
    proof 6 / witness **1346** / UNKNOWN **0** / `union_residual_rules []`;
    `fully_certified_via_union: true`. Both retired tokens were WITNESSED, so the
    −2 lands ENTIRELY in the witness columns — **proof, both UNKNOWN counts and
    the residual set are ALL UNCHANGED**. The grammar lost two symbols and kept
    every certificate. Contract re-baselined to `expected_total 1352` /
    `expected_canonical_witness 1335` / `expected_union_witness 1346` + a new
    `rebaseline_note`; verified by JSON diff that ONLY those 4 fields changed and
    `done_rule` is byte-identical.
  - `verilog_2005_conformance_gate` — first run RED on the predicted −2 census
    drift, then GREEN on re-baseline. Measured `1121/328/779/14` (was
    `1123/330/779/14`) identically at seeds 0/7/42, `sample_parse_failures 0`,
    `proof_reverify_failures 0`; **behaviour unaffected and verified, not
    inferred** — corpus matrix **240 checks / 0 mismatches**, alias checks 2,
    `profile_orphans 0`, lint exit 0. Contract re-baselined to
    `expected_total 1121` / `expected_proof 328` (witness 779 and UNKNOWN 14
    untouched); JSON-diff-verified that ONLY those 2 pins changed and the
    `baseline_note` was strictly APPENDED, preserving the full provenance chain.
    ⭐ `.3.8`'s "v2005 byte-inert" precedent did **NOT** carry over, and assuming
    it would have made this gate look like a regression: a TOKEN carries no
    `@profiles` gate of its own — only its consumers do — so
    `--dump-rule-profiles` on the pre-fix grammar (`git show HEAD:`) measured both
    retired tokens satisfiable under ALL THREE profiles
    (`census_effect.txt`: sv_2017 1354→1352, sv_2023 1373→1371,
    verilog_2005 1123→1121). **LAW BANKED: profile-inertness of BEHAVIOUR does
    not imply profile-inertness of the CENSUS.**
  - ⭐⭐ **THE SHARPEST FINDING OF THE GATE PASS — the same two retired tokens were
    classified DIFFERENTLY per profile, and both classifications are correct:**
    under `sv_2017` they were **WITNESSES** (reachable and positively exercised),
    so the sibling union contract re-baselined *witness* 1337→1335 / 1348→1346
    with **proof 6 unchanged**; under `verilog_2005` they were **PROOFS** (the
    `VERILOG-2005-PROFILE.6.7` `ProfileEntryUnreachable` class — SVA is
    profile-unreachable there), so this contract re-baselined *proof* 330→328 with
    **witness 779 byte-identical**. In BOTH profiles the −2 is fully positively
    accounted and **UNKNOWN is untouched** (11/0 and 14) — the grammar lost two
    symbols and surrendered no certificate anywhere. This is not a contradiction
    to reconcile but the profile machinery working exactly as designed, and it is
    the reason a single "expected census delta" cannot be applied blindly to both
    contracts: WHICH column moves is profile-dependent.
  - `sv_external_corpus_triage_gate` — PASS (no preprocess / parse / blocked
    failure cases).
  - `systemverilog_parser_book_gate` — PASS (mdbook build + tracked-HTML check);
    the tracked `docs/systemverilog_parser_book-html/` was regenerated and the
    rendered `changelog-index.html` VERIFIED to carry the new `1.0.175` entry
    rather than trusting the gate's "HTML present" check (18 HTML files touched,
    incl. the usual `searchindex-*.js` hash rename).
  - `sv_stimuli_quality_gate` — PASS (guard exit 0, peak 11,188 MB / 2,471 s;
    `closed_loop_initial_replay_determinism_passes 2/2`,
    `closed_loop_replay_preprocess_warnings_total 0` / `errors_total 0`).
    ⭐ `closed_loop_replay_targets_total` **127 → 124 (−3)** — a DECREASE, i.e.
    three closed-loop replay-debt gaps CLOSED. A decrease cannot be a regression
    here: the criterion `focused_replay_target_debt_zero` (the LAST unmet SV
    family-status criterion, owned by `SV-REPLAY-DEBT`) is satisfied at **0**
    targets, so this moves TOWARD the `Done` bar. Note the direction contrast with
    the immediately preceding leaves, and that it is coherent: `.3.8` went 126→127
    (+1) because its new bracketed alternative became a NEW generation target the
    generator had yet to witness, whereas this leaf RETIRES two symbols and its
    two new branches are witnessed immediately (the same "immediately witnessed"
    pattern `.3.6` recorded when its count held). Same accounting family as
    `.3.5`'s 126→125. Feeds `SV-REPLAY-DEBT`, which owns the burn-down.
  - **clippy** — source-strict PASS; generated stage reports **291** errors, which
    is EXACTLY the tracked baseline (session #192 `.7a`) ⇒ **zero new generated
    clippy debt** from this leaf.
    ⚠️ **BUT `make clippy_on_rust_change` FIRST REPORTED SUCCESS BY SKIPPING** —
    "No Rust/generated Rust changes detected; skipping clippy flow." Its detection
    (`clippy_on_rust_change.sh:46`) matches git-reported changed paths against
    `rust/*.rs` / `generated/*.rs`, but `generated/` is **gitignored by repo
    policy**, so a regenerated parser is INVISIBLE to it. A grammar-only leaf
    therefore changes only the tracked `.ebnf`, and the step that exists to lint
    the regenerated parser cannot see the regenerated parser — it passes green
    while doing nothing. The real lint above was obtained with
    `PGEN_CLIPPY_FORCE=1`. ⇒ **`COMMIT.md` step 2 has been silently
    self-exempting for every grammar-only leaf, which is most of the `.3.x`
    series.** Same shape as the `BIN-BUILD-INTEGRITY` findings (a check whose
    covered set is narrower than it looks); NOT fixed here — routed as a
    surfaced finding + candidate leaf for that tree (see `DEVELOPMENT_NOTES.md`
    item 7 for this session).
  - `--lint-grammar` — clean: 1475 rules, `non_terminating`/
    `ordered_choice_shadowing`/`unreachable_rules`/`undefined_references`/
    `unbound_fact_kinds`/**`profile_orphans`** all 0.

- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — `## [*]` / `## [+]` REJECT at
    `furthest_position=214`/`146` while the tight `##[*]`/`##[+]` and both
    `##[1:2]` spacings ACCEPT (`before.txt`); 1 keyed corpus row
    (`t_sequence_sexpr_unsup.v`) rejecting at `furthest_position=1112`.
  - [x] **ROOT CAUSE (WHY + WHERE)** — `cycle_delay_range:1707/:1709` reference
    FUSED literals `kw_token_71b8cf7e := trivia "##[*]"` (`:6655`) /
    `kw_token_9768502a := trivia "##[+]"` (`:6659`), each fusing FOUR lexical
    tokens; `trivia` skips only BEFORE a terminal, never inside one match.
    Trace-proven DIFFERENTIALLY (`trace_r1_before.txt`): spaced input →
    `❌ Branch 3/4 for rule 'cycle_delay_range' failed at position 209`
    (bytes 209.. = `' ## [*] '`); tight input → the SAME branch wins,
    `🏁 Rule 'cycle_delay_range' selected branch 3/4 consuming 6 chars`.
    Governing law: IEEE 1800-2017 §5.3, quoted verbatim.
  - [x] **FIX** — fix-hierarchy tier 2 (grammar): two structural alternatives from
    existing tokens + retire the two fused tokens; reuses the in-grammar
    `consecutive_repetition:1377` idiom; census 1477 → 1475.
  - [x] **ADDRESSED (verified)** — 6/6 repro rows REJECT→ACCEPT or held; 9 spacing
    variants ACCEPT; AST `{kind:"star"}`/`{kind:"plus"}`; main corpus pass
    9,692→9,694 (+1 attributable, +1 proven jitter); rejects-valid 383→382.
  - [x] **NO REGRESSION** — per-FILE pass-set diff **0 pass→fail / 0 pass→timeout /
    0 pass→crash**; **0 new** rejects-valid (set difference); accepts-invalid set
    BYTE-IDENTICAL (21); 5 near-miss negatives still REJECT; v2005 lane
    byte-identical (results + manifest); cert union GREEN at seeds 0/7/42 with
    `spf=0`, UNKNOWN 11/0 and residual `[]` unchanged,
    `fully_certified_via_union: true`; shape contract 18/18; lint
    `profile_orphans=0`.
  - [x] **LOCKSTEP** — ledger `SV-0045` + contract identity `1.0.175` / schema
    `18`→`19` (with the breaking-rename rationale for downstream) + SV book
    changelog-index + BOTH cert contract JSONs re-baselined +
    tree/TASK_TREE/MEMORY/CHANGES/DEVELOPMENT_NOTES/LIVE this commit.

#### `.3.10` — the spaced UDP/timing-check NUMBER literals (`1 'b 1`) reject (IEEE 1800-2017 §5.7.1 — fused number literals in `init_val` / `scalar_constant`; the sibling `.3.9`'s widened sweep surfaced)

- **Status: `done`** (session #206, 2026-07-25) — opened by `.3.9` (session #205)
  from a MEASURED reject, not from speculation. Same discipline `.3.8` applied
  when it opened `.3.9`: a different-law/different-mechanism finding gets its own
  leaf instead of being folded in silently. The charter below is the leaf as
  OPENED; the closure record follows it.
- **EVIDENCE ALREADY BANKED**
  (`artifacts/sv_corpus_grad/fused_bracket_literal_diag/sweep_fused_delimiter_literals.txt`,
  CLASS C). Minimal repro on the post-`.3.8` parser (`--profile sv_2017`), a UDP
  with a §5.7.1-spaced initial value:

  | form | verdict |
  |---|---|
  | `initial q = 1'b1;` | ACCEPT |
  | `initial q = 1 'b 1;` | **REJECT** (`furthest_position=81`) |

- **ROOT CAUSE (WHY + WHERE, already pinned):** `init_val:2554` (IEEE 1800-2017
  A.5.2 / §29, `init_val ::= 1'b0 | 1'b1 | 1'bx | 1'bX | 1'B0 | 1'B1 | 1'Bx |
  1'BX | 1 | 0`) and `scalar_constant:4935` enumerate a CLOSED literal set, so
  they do not route through `integral_number` — they reference 12 of their own
  fused number-literal tokens (`:6382`–`:6396` `1'b0`/`1'b1`/`1'bx`/`1'bX`/
  `1'B0`/`1'B1`/`1'Bx`/`1'BX`, half as `trivia "…"` and half as `/…\b/`, plus the
  unsized `'b0`/`'b1`/`'B0`/`'B1` twins). IEEE 1800-2017 **§5.7.1** permits white
  space between the SIZE and the `'` and between the base format and the VALUE
  (not between `'` and the base char) — the EXACT law leaf `.3.5` applied to
  `integral_number:439` (ledger `SV-0041`). These copies never got that fix.
- **⛔ CLASS EVIDENCE (why this is not a one-off):** `SV-0030`
  (`SV-DOLLAR-LRM-FIDELITY.3`) already repaired a DIFFERENT bug in these very
  tokens — digit-less `1'b`/`1'B` prefix-merging — see the comment block at
  `:6376`. So this family has now been patched reactively TWICE without either
  pass noticing the remaining lexical-fidelity hole. Together with `.3.8`'s
  bracket-as-metasyntax face and `.3.9`'s bracket-as-fused-atom face, that is a
  THIRD independent argument for the exhaustive `LRM-GRAMMAR-FIDELITY` Annex-A
  sweep rather than a fourth isolated repair.
- **FIX (planned):** adjudicate reuse-`integral_number`-seams vs per-token
  `[ \t]*` seams (12 tokens, 2 consumer rules); prefer whichever keeps the LRM's
  closed set enforced (a naive `integral_number` swap would wrongly admit
  `2'b11` as an `init_val`).
- **SHAPE QUESTION — ALREADY MEASURED AND ANSWERED (session #205, so the leaf does
  not have to rediscover it): NO schema bump is expected, and the kind names are
  LOCKED and must NOT change.** ⭐ Unlike `.3.9`, BOTH consumer rules carry locked
  `ast_shape_contract` samples — `rust/test_data/ast_shape_contract/systemverilog_v1.json`
  has `scalar_constant_lrm_digits` and `init_val_lrm_digits`, each
  `input: "1'b0"`, `rule_under_test` the respective rule, and
  `expected_json_object_string_values: {"kind": "1'b0"}` (both `drift_status:
  aligned`, both landed by `SV-0030` / `SV-DOLLAR-LRM-FIDELITY.3`). ⇒ the correct
  fix admits white space at the two §5.7.1 seams while KEEPING every
  `{kind:"1'b0"}` … `{kind:"1'BX"}` name, making the change **purely additive**
  (only previously-REJECTED spellings gain parses, no emitted kind changes) —
  so schema `19` should hold. A fix that renamed these kinds would break two
  locked samples AND a published contract row; that is the design constraint,
  established by measurement up front rather than discovered by a red gate.
- **SCOPE:** measure the corpus population first (`init_val` is UDP-only, so it
  may be 0 rows — in which case this is a correctness/LRM-fidelity fix with no
  graduation delta, and should be sequenced accordingly rather than oversold).

---

**⭐ CLOSED — session #206, 2026-07-25, `PGEN-SV-CORPUS-GRAD-0025`, release
`1.0.175`→`1.0.176`, schema `19` UNCHANGED, ledger `SV-0046`.**
Evidence bundle: `docs/tasks/artifacts/sv_corpus_grad/fused_number_literal_diag/`
(`before.txt`, `after.txt`, `transitions.txt`, `trace_before_differential.txt`,
`sweep_fused_number_literals.txt`, `census_effect.txt`, `global_measurement.txt`,
plus the re-runnable driver `run_matrix.sh` and the analyser
`analyze_corpus_delta.py`).

- **REPRODUCED (`before.txt`, 26-row matrix over BOTH consumer rules).** The leaf
  inherited a 2-row repro; it was widened to a full class matrix before any edit.
  8 LRM-legal `init_val` spellings REJECTED (`1 'b 1`, `1 'b1`, `1'b 1`, tab
  forms, `1 'B 1`, `1 'b x`, `1 'b X`, `1  'b  0`), all 9 near-miss negatives
  already rejected, and the same 8 rejected under `verilog_2005` too.
- **⭐ A SECOND FACE THE LEAF DID NOT KNOW ABOUT — `scalar_constant` does not
  REJECT, it SILENTLY DEGRADES THE AST.** `scalar_timing_check_condition:4967`
  carries a fallback `expression` branch, so a failing `scalar_constant` never
  fails the file — it just loses the typed shape. AST-dump differential:

  | input (in a `specify` timing check) | BEFORE | AFTER |
  |---|---|---|
  | `cond == 1'b1` | `{kind:"eq", rhs:{kind:"1'b1"}}` | unchanged (control) |
  | `cond == 1 'b 1` | **`{kind:"expression"}`** | `{kind:"eq", rhs:{kind:"1'b1"}}` |
  | `cond == 'b 1` | **`{kind:"expression"}`** | `{kind:"eq", rhs:{kind:"'b1"}}` |

  ⇒ this face is invisible to the corpus pass/fail columns entirely. It would
  never have been found by another rejects-valid sweep, only by asking what the
  AST actually contains. Banked as a general lesson: **a rule with an
  expression-shaped fallback branch converts an acceptance defect into a shape
  defect, so "the corpus still passes" is not evidence that a construct works.**
- **ROOT CAUSE (WHY + WHERE), trace-proven DIFFERENTIALLY**
  (`trace_before_differential.txt`; `--trace-rules init_val`, TOOLBOX step 3):
  on the SPACED input, branches 1/2/5/6 fail on their contiguous regexes at the
  space before the `'` and branches 3/4/7/8 (string literals) fail likewise —
  then **branch 9, the LRM's BARE `1` alternative, MATCHES**, and `longest_match`
  selects `9/10 consuming 2 chars`. On the TIGHT input the SAME branch 2 matches
  and wins with 5 chars. Only the whitespace differs.
  ⭐ **The sharp part, which the leaf's original framing did not have: this is not
  a clean rejection at `init_val`. The rule SUCCEEDS having eaten only the `1`,
  and the parse dies downstream in `sequential_body` (surface 189, furthest
  266).** A mis-parse that reports its failure somewhere else is exactly why
  eyeballing the error position would have sent an investigator to the wrong
  rule — the case for TOOLBOX-first, not a formality.
  WHERE: `init_val:2554` alts 1-8, `scalar_constant:4935` alts 1-8; tokens
  `:6382`-`:6396` (8 sized) and `:6635`-`:6641` (4 unsized). Engine property:
  `trivia` skips only BEFORE a terminal, never inside one match — the same
  property `.3.5`/`SV-0041` and `.3.9`/`SV-0045` each root-caused.
- **GOVERNING LAW — IEEE 1800-2017 §5.7.1, read from the LRM workspace, verbatim
  (not paraphrased from the sibling leaf):** a based literal "shall be composed of
  up to three tokens" (size / `'`+base / value); "The apostrophe character and the
  base format character shall **not** be separated by any white space" (the ONE
  closed seam); "The unsigned number token shall immediately follow the base
  format, **optionally preceded by white space**" (explicitly open). The
  size↔apostrophe seam is open by §5.3 (separate lexical tokens) with no carve-out.
- **FIX (fix-hierarchy tier 2 = grammar; 12 token bodies, nothing else):**
  `[ \t]*` at exactly the two open seams —
  `/1[ \t]*'b[ \t]*0\b/` … `/1[ \t]*'B[ \t]*X/` for the 8 sized tokens and
  `/'b[ \t]*0\b/` … `/'B[ \t]*1\b/` for the 4 unsized ones (unsized have only ONE
  interior seam — no size token exists). The 4 string-literal tokens
  (`"1'bx"`/`"1'bX"`/`"1'Bx"`/`"1'BX"`) became regexes **without** adding a `\b`
  they never had, so their prefix-matching semantics are preserved exactly.
  ⭐ **Keeping them as their own tokens — rather than swapping in
  `integral_number` — is what keeps the LRM's CLOSED set closed**: the leading `1`
  stays hard-coded, so `2'b11` still cannot be an `init_val` (the naive-swap trap
  the leaf charter warned about, discharged by construction and proven by R1).
  `[ \t]` (horizontal only) matches `.3.5` deliberately; a literal spanning a
  NEWLINE is the same deferred composite-rule case there and here (row D1).
  **No new rules or tokens — census `1475` UNCHANGED.**
- **VERIFIED — `after.txt` / `transitions.txt`** (release probe relinked 19:03,
  mtime-asserted NEWER than the 18:44 parser; same `run_matrix.sh` driver re-run,
  not re-typed):
  - **8 rows REJECT→ACCEPT, and identically under `verilog_2005`** — a
    CROSS-PROFILE heal (IEEE 1364-2005 §3.5.1 carries the same allowance; these
    tokens carry no `@profiles` gate). Same shape as `.3.5`; contrast the
    SVA-gated `.3.8`/`.3.9`.
  - **must-ACCEPT 16/16 under both profiles; must-REJECT 9/9 still reject under
    both.** The over-permissiveness guard is measured, not argued: `2'b1`,
    `1'b2`, `1'bz`, `1'b 11`, bare `'b1`-as-`init_val`, the `SV-0030`
    digit-less regression guards `1'b`/`1 'b`, and — the LRM-critical pair —
    `1' b 1` and `1 ' b 1`, which §5.7.1 PROHIBITS and which still reject.
  - **⭐ AST kinds are BYTE-IDENTICAL tight vs spaced** (`--entry-rule init_val`):
    4 tight/spaced pairs, 4/4 emitting the same `kind` with only `span_end`
    differing. That is the direct proof the correct LRM alternative now wins
    (not the bare-`1` fallback) AND that no kind is renamed.
  - Regex-level pre-check (before any rebuild): all legal spacings match, all
    prohibited/out-of-set spellings reject, and "size must be exactly 1" holds
    for all 12 patterns.
- **⭐ THE MANDATED SWEEP RAN AND IS EXHAUSTIVE**
  (`sweep_fused_number_literals.txt`). `.3.9` set the precedent — do not fix the
  one site the corpus tripped over. All **14** token definitions embedding an
  apostrophe were adjudicated (grammar-wide grep, no sampling): 12 = this defect;
  `tick := "'"` is one character so it has no interior; and
  **`unbased_unsized_literal:493` (`'0`/`'1`/`'x`/`'z`) ADJUDICATED CORRECT AS
  FUSED with a named LRM cause** — Annex A footnote **48**: *"The apostrophe ( ' )
  in unbased_unsized_literal shall not be followed by white_space."* ⭐ **This is
  the sweep's most valuable outcome: that token looks IDENTICAL in shape to the
  defective unsized `'b0` twins, and a mechanical "add seams wherever there is an
  apostrophe" pass would have widened acceptance against the standard.** Same
  adjudication shape as `.3.9`'s `attr_open`/`attr_close`. The 7 neighbouring
  closed-literal-set rules (`edge_descriptor`, `level_symbol`, `output_symbol`,
  `zero_or_one`, `z_or_x`, `finish_number`, `1step`) were swept too: all are
  one-character tokens or already-structural multi-token sequences ⇒ no interior
  seam. **0 further members. The fused NUMBER-literal face is CLOSED.**
- **⚠️ THE SWEEP SURFACED A NEW MEASURED DEFECT IN THE OPPOSITE DIRECTION → new
  leaf `.3.11`** (see below): Annex A footnote **44** forbids white space between
  a `time_literal`'s number and its unit, but `time_literal:495` is spelled
  `number time_unit` (two rules), so `timeunit 10 ns;` **ACCEPTS** and should not.
  Over-acceptance, not under-acceptance — deliberately NOT folded in.
- **CENSUS — BYTE-INERT, measured per profile** (`census_effect.txt`):
  rule_count `1475`→`1475`; satisfiable_under `sv_2017` 1352→1352, `sv_2023`
  1371→1371, `verilog_2005` 1121→1121; and the strongest form — **the per-rule
  profile map is identical across all 1475 rules (0 differences)**, with the
  sorted rule-name lists byte-identical. Those three counts are exactly the two
  contract pins, which is why both cert gates came back byte-inert.
  ⚠️ **Tool-usage correction recorded in the artifact:** the first
  `--dump-rule-profiles` pass passed profile NAMES where the flag wants an OUTPUT
  PATH, silently writing three stray JSON dumps into the repo root and yielding a
  meaningless reading. Strays deleted, measurement redone. The conclusion never
  depended on the bad reading (the rule-set set-diff is independent and stronger),
  but a wrong measurement does not get to sit in the record unremarked.
- **`--lint-grammar`: CLEAN** — 1475 rules, `non_terminating`/
  `ordered_choice_shadowing`/`unreachable_rules`/`undefined_references`/
  `unbound_fact_kinds`/`nullable_repetition`/**`profile_orphans`** all 0.
  Regen green (`focus_systemverilog`, guard exit 0, peak 2,255 MB / 71 s;
  return-annotation inventory 2,280 entries).
- **⭐ KEYED CORPUS POPULATION = 0 ROWS, DERIVED AND STATED BEFORE MEASURING.**
  The 12 tokens have exactly two consumers, both context-restricted (`init_val`
  is UDP-only, `scalar_constant` timing-check-only). Of 16,336 files, 5 carry a
  spaced size seam and 145 a spaced value seam, but only 3 also contain
  `primitive`/`specify` — and probing all 3 shows each dies elsewhere
  (`` `ifdef `` ×2, UDP-table edge symbols ×1). Every other hit is an
  expression-position number routed through `integral_number`, healed since
  `.3.5`. ⇒ **ceiling +0 pass, stated up front; measured outcome +0 pass.**
  This is an LRM-fidelity/correctness leaf and must not be sold as a graduation
  leaf.
- **GLOBAL MEASUREMENT** (`global_measurement.txt`; DEBUG probe per the `.3.6`
  requirement, mtime-asserted newer than the parser; guard exit 0):
  - **Main `sv_2017` lane (16,336 files, 232 s, peak 5,697 MB):** pass 9,694 →
    9,693, fail 6,634 → 6,634, timeout 8 → 9. **Exactly ONE per-FILE transition:**
    `pass→timeout` on `verilator/test_regress/t/t_math_synmul_mul.v`.
  - **⛔ THAT ROW IS PROVEN CONTENTION JITTER, AND PROVEN — NOT ARGUED.**
    `.3.8`/`.3.9` each dismissed their jitter rows with a TEXT argument ("the file
    contains no `##`, so the rule is never reached"), which is an *inference*
    about reachability. This leaf proves it directly with the instrument, plus a
    positive control so the zero is shown to be a real zero:
    1. **Structural** — the file contains `primitive` 0× and `specify` 0×, so
       both consumer rules are unreachable in it (its 7 `1'b` hits are ordinary
       expression-position numbers).
    2. ⭐ **Instrumented** — `--trace-rules init_val,scalar_constant` on that exact
       file emits **0** lines naming either rule; the SAME probe with the SAME
       flags emits **10** on the UDP repro. ⇒ the edited regexes are never
       EXECUTED there, so the change can affect neither its verdict nor its time.
    3. **Empirical** — run solo it takes 17.94 / 17.97 / 17.89 s against the 20 s
       wall and exits 0 = **pass**; its true verdict is unchanged.
    4. **Historical** — `.3.9` recorded this SAME file flipping the OPPOSITE way
       (`timeout→pass`). A row that oscillates both ways across consecutive leaves
       is a wall-clock artifact.
    ⚠️ **The 15 s (`.3.9`) vs 17.9 s (here) solo reading was NOT waved away** — that
    is precisely the shape a silent speed regression would take, and the north
    star makes speed co-equal and never-regressed. Point 2 settles it: a rule
    entered zero times cannot cost time; the delta is cross-session variance
    (different debug build, different ambient load).
  - **ADJUDICATION — set-level, not just counts:** `match` 5,727 → 5,727 SET
    IDENTICAL; **`unexplained_rejects_valid` 382 → 382 SET IDENTICAL (zero new,
    zero healed)**; `unexplained_accepts_invalid` 21 → 21 SET IDENTICAL; all 5
    `explained_svpp_*` and all 6 `deferred:*` classes SET IDENTICAL except the one
    jitter row moving `deferred:chained_only` → `divergence:explained_timeout`.
    The entire 16,336-row manifest differs by **exactly one line**.
  - **NO REGRESSION (the `.3.4` LAW — per-FILE pass-set diff, not net counts):
    0 pass→fail, 0 pass→timeout, 0 pass→crash attributable to this leaf.**
  - **`verilog_2005` lane RE-RUN, not inferred** — and here it HAD to be, because
    unlike `.3.8`/`.3.9` this fix genuinely reaches `verilog_2005`: 2,459 files,
    2,180/279/0, **ZERO per-FILE transitions**; `results_v2005.tsv`
    CONTENT-IDENTICAL (sorted diff 0 lines; the raw byte diff is parallel-job
    emission ORDER only) and `adjudication_manifest_v2005.tsv` BYTE-IDENTICAL;
    v2005 unexplained stays 76 (62 rejects-valid + 14 accepts-invalid).
    **Capability widened cross-profile, behaviour on this corpus unchanged.**
- **GATES — all 7 GREEN, seeds 0/7/42, and BOTH cert contracts UNTOUCHED**
  (the `.3.8` posture, predicted in writing in `census_effect.txt` BEFORE the
  gates ran):
  - `sv_syntax_closure_gate` — PASS, `defined_rule_count` **1475** (unchanged, as
    designed), `unreachable_rules: 0`, `unresolved_rule_reference_count: 0`.
    (`unreachable_branches: 2` is the pre-existing tracked cap, contract untouched.)
  - `ast_shape_contract_gate` — PASS **18/18**. ⭐ This is the decisive
    no-schema-bump proof: the three locked samples `init_val_lrm_digits`,
    `scalar_constant_lrm_digits` and `scalar_timing_check_condition_eq` pin
    exactly the kinds this leaf touches, and pass unchanged.
  - `sv_cert_recognized_union_gate` — PASS **with NO re-baseline**: union_witness
    **1346** == expected, canonical UNKNOWN **11**, union UNKNOWN **0**,
    `union_residual_rules []`, `fully_certified_via_union: true`,
    `sample_parse_failures 0`, `unmet_criteria_count 0`, deterministic at seeds
    0/7/42. ⭐ The recorded risk that a widened REGEX terminal could still move
    the witness column (the generator synthesizes witnesses FROM the regex) is
    exactly why this was run rather than argued away; measured outcome: it did not.
  - `verilog_2005_conformance_gate` — PASS **byte-inert**: cert
    `1121/328/779/14` exactly as pinned, corpus matrix **240 checks / 0
    mismatches**, alias checks 2, `profile_orphans 0`, lint exit 0, deterministic
    at seeds 0/7/42, `unmet_criteria_count 0`.
  - `sv_external_corpus_triage_gate` — PASS (no preprocess / parse / blocked
    failure cases).
  - `systemverilog_parser_book_gate` — PASS (mdbook build + tracked-HTML check);
    the rendered `docs/systemverilog_parser_book-html/changelog-index.html` was
    VERIFIED to carry the new `1.0.176` entry rather than trusting the gate's
    "HTML present" check.
  - `sv_stimuli_quality_gate` — PASS (guard exit 0, peak 11,967 MB / 1,993 s):
    `closed_loop_profiles_passed 2/2`,
    `closed_loop_initial_replay_determinism_passes 2/2`, preprocess warnings 0 /
    errors 0, total_warnings 0 / total_errors 0, all 16 sample rows `pass` at
    100% parseability acceptance.
  - **clippy** — source-strict PASS; generated stage **291** errors = EXACTLY the
    tracked baseline (session #192 `.7a`) ⇒ zero new generated debt.
    ⚠️ Required `PGEN_CLIPPY_FORCE=1`: `.3.9`'s finding that
    `make clippy_on_rust_change` silently self-exempts on grammar-only leaves
    (`generated/` is gitignored, so the regenerated parser is invisible to its
    git-diff trigger) is RECONFIRMED by direct observation — now two consecutive
    slices. Still routed to `BIN-BUILD-INTEGRITY`, still unopened.
  - `--lint-grammar` — clean: 1475 rules, `non_terminating` /
    `ordered_choice_shadowing` / `unreachable_rules` / `undefined_references` /
    `unbound_fact_kinds` / `nullable_repetition` / **`profile_orphans`** all 0.

- **⚠️⚠️ `closed_loop_replay_targets_total` 124 → 127 (+3) — NOT SEMANTICALLY
  ATTRIBUTABLE TO THIS LEAF, and running that check down surfaced a
  MEASUREMENT-INTEGRITY problem in how the whole `.3.x` series has been reading
  this metric.** The gate PASSES either way; this is about what the number means.
  - **MEASURED (hard fact, not inference):** the 127 targets were ENUMERATED
    (62 under `sv_2017` + 65 under `sv_2023`) and **ZERO of them relate to this
    leaf's edit** — not by `rule_name`, not by `node_path`, not by `branch_id`,
    not anywhere in the target records. Checked against both the 12 edited tokens
    and the full consumer chain (`init_val`, `scalar_constant`,
    `scalar_timing_check_condition`, `sequential_body`, `udp_declaration_*`,
    `specify_block`): **intersection EMPTY in both profiles.** The 23/24 distinct
    target rules are all constraint / assertion / class / property /
    net-declaration surfaces (`prop_primary_*`, `constraint_primary_*`,
    `class_declaration_sv_2023`, `randomize_call`, …).
  - **NOT run-to-run noise, and I checked before claiming it was:** the first
    hypothesis was sampling wobble over the 5,000-attempt budget. `SV-REPLAY-DEBT.1`
    REFUTES that — it verified determinism directly, with two independent canonical
    runs at one vintage producing **byte-identical** gap artifacts (sha256-verified,
    all four JSONs). Also measured here: `closed_loop_parseability_shadow_target_timeout_errors_total 0`
    and `helper_timeout_errors_total 0`, so the 5 ms per-target budget never fired.
  - **INFERENCE (labelled as such — the mechanism, consistent with BOTH facts):**
    the closed-loop generator is seeded and deterministic *for a fixed grammar*,
    but widening 12 regex bodies ENLARGES their generatable language and therefore
    shifts the generator's consumption of the random stream. Which unrelated
    branches happen to be witnessed within the attempt budget reshuffles. So the
    +3 is *caused by* the edit and *not semantically about* it — the debt sits in
    rules the edit provably cannot reach.
  - ⛔ **THE FINDING: prior leaves attributed this metric SEMANTICALLY without
    performing the relatedness check.** `.3.9` recorded "127→124 (−3) = three
    replay-debt gaps CLOSED … moves TOWARD `focused_replay_target_debt_zero`";
    `.3.8` recorded "126→127 (+1) — the bracketed alternative is a NEW closed-loop
    generation target". Neither enumerated the targets to see whether they were in
    or downstream of the changed rules. Note the series shape: `.3.9` REMOVED two
    symbols and went −3; this leaf removes nothing and goes +3, landing back
    exactly on 127. ⭐ **Because `focused_replay_target_debt_zero` is the LAST
    unmet SV family-status criterion, a Done-gate is being read off a number that
    moves for reasons unrelated to the work.** The check that settles it is cheap —
    enumerate `targets[]` and intersect with the changed rules, which is what this
    leaf did and what no prior leaf did.
  - ⇒ **Routed to `SV-REPLAY-DEBT.1c`** (opened this session), which owns the
    burn-down and the criterion. **This leaf claims NO replay-debt movement.**

- **Rejects-valid graduation baseline REMAINS 382; accepts-invalid REMAINS 21.**

- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — `before.txt`: 8 LRM-legal `init_val` spellings
    REJECT under BOTH `sv_2017` and `verilog_2005` (`1 'b 1`, `1 'b1`, `1'b 1`,
    tab forms, `1 'B 1`, `1 'b x`, `1 'b X`, `1  'b  0`) while the tight `1'b1`
    accepts; and — the face the charter did not know about — `scalar_constant`
    never rejects but silently degrades its AST from
    `{kind:"eq", rhs:{kind:"1'b1"}}` to a flat `{kind:"expression"}`.
  - [x] **ROOT CAUSE (WHY + WHERE)** — `init_val:2554` alts 1-8 and
    `scalar_constant:4935` alts 1-8 reference 12 FUSED number-literal tokens
    (`:6382`-`:6396` sized, `:6635`-`:6641` unsized) that bypass `integral_number`
    and so never inherited `.3.5`/`SV-0041`'s §5.7.1 seam fix; `trivia` skips only
    BEFORE a terminal, never inside one match. Trace-proven DIFFERENTIALLY
    (`trace_before_differential.txt`): on the spaced input branches 1-8 fail and
    the bare-`1` branch 9 MATCHES (`selected branch 9/10 consuming 2 chars`), so
    `init_val` SUCCEEDS on one character and the parse dies downstream in
    `sequential_body`; on the tight input the SAME branch 2 wins with 5 chars.
    Governing law IEEE 1800-2017 §5.7.1, quoted verbatim from the LRM workspace.
  - [x] **FIX** — fix-hierarchy tier 2 (grammar): `[ \t]*` at exactly the two
    §5.7.1-open seams in 12 token BODIES; the closed set stays closed because the
    leading `1` remains hard-coded. No new rules or tokens; census **1475
    UNCHANGED**; per-rule profile map identical across all 1475 rules.
  - [x] **ADDRESSED (verified)** — `after.txt` / `transitions.txt`: 8 rows
    REJECT→ACCEPT identically under both profiles; must-ACCEPT **16/16**;
    the shape face repaired (`1 'b 1` and `'b 1` both now emit the typed
    `eq`/`rhs` shape); AST kinds BYTE-IDENTICAL tight vs spaced (4/4 pairs),
    proving the correct LRM alternative wins and no kind is renamed.
  - [x] **NO REGRESSION** — must-REJECT **9/9** still reject under both profiles
    (incl. the §5.7.1-PROHIBITED `1' b 1` / `1 ' b 1`, `2'b1`, `1'b2`, `1'bz`,
    `1'b 11`, digit-less `1'b`); per-FILE pass-set diff **0 pass→fail, 0
    pass→timeout, 0 pass→crash attributable** (the single `pass→timeout` proven
    unreachable by `--trace-rules`, 0 hits vs 10 on a positive control);
    rejects-valid 382 SET-IDENTICAL, accepts-invalid 21 SET-IDENTICAL; v2005 lane
    **0 transitions** with a byte-identical manifest; both cert gates green with
    NO re-baseline; `ast_shape_contract_gate` 18/18; lint `profile_orphans=0`;
    clippy generated 291 = tracked baseline.
  - [x] **LOCKSTEP** — ledger `SV-0046` + contract identity `1.0.176` (with the
    schema-`19`-stays rationale and the shape-repair note for downstream) + SV
    book changelog-index (rendered HTML verified to carry `1.0.176`) +
    tree/TASK_TREE/MEMORY/CHANGES/DEVELOPMENT_NOTES/LIVE this commit. Both cert
    contract JSONs deliberately UNTOUCHED (measured byte-inert, not assumed).

#### `.3.11` — `time_literal` admits white space the LRM forbids (`timeunit 10 ns;` wrongly ACCEPTS — IEEE 1800-2017 Annex A footnote 44; surfaced by `.3.10`'s sweep)

- **Status: `todo`** — opened by `.3.10` (session #206, 2026-07-25) from a
  MEASURED over-acceptance, not speculation. Same discipline `.3.8`→`.3.9`→`.3.10`
  applied: a different-direction/different-mechanism finding gets its own leaf.
- **EVIDENCE ALREADY BANKED**
  (`artifacts/sv_corpus_grad/fused_number_literal_diag/sweep_fused_number_literals.txt`,
  SWEEP 3). Measured on the pre-`.3.10` parser (`--profile sv_2017`), `timeunit <lit>;`:

  | form | verdict | LRM |
  |---|---|---|
  | `10ns` | ACCEPT | legal |
  | `10 ns` | **ACCEPT** | **ILLEGAL** (footnote 44) |
  | `10<TAB>ns` | **ACCEPT** | **ILLEGAL** (footnote 44) |

- **ROOT CAUSE (WHY + WHERE, already pinned):** IEEE 1800-2017 Annex A footnote
  44, verbatim — *"The unsigned number or fixed-point number in time_literal shall
  not be followed by a white_space."* But `time_literal:495` is spelled
  `number time_unit` — TWO rules in sequence — and PGEN's `trivia` skipper runs
  before every terminal, so the seam the LRM CLOSES is wide open. This is the
  exact INVERSE of `.3.10`: there a lexical allowance was missing from a fused
  token; here a lexical PROHIBITION is rendered structurally.
- **⛔ WHY IT IS NOT A `.3.10` FOLD-IN:** (1) opposite direction —
  accepts-invalid, not rejects-valid, so it is measured by a different corpus
  column and, unlike `.3.10`, a fix that TIGHTENS acceptance **can break
  currently-passing corpus files**; (2) different mechanism (structural-vs-lexical
  seam, no fused literal anywhere); (3) different law (footnote 44, not §5.7.1's
  three-token decomposition).
- **⛔ CORRECTION OF RECORD (director review, same session #206): the "dialect
  tension" this leaf was opened with DOES NOT EXIST — it was my unverified claim,
  and measuring it dissolved the dilemma.** The leaf originally said "real-world SV
  writes `10 ns` constantly and mainstream simulators accept it, so enforcing
  footnote 44 makes PGEN reject text the ecosystem treats as fine." **That was
  asserted from general knowledge, not measured, and it is wrong.** Measured over
  all 16,336 corpus files (verilator, opentitan, black-parrot, iverilog, ispras,
  Surelog, sv2v, sv-tests, uvm-core — i.e. real designs, not toys):

  | pattern | files |
  |---|---|
  | `timeunit`/`timeprecision` + **spaced** unit | **0** |
  | `#<num>` + **spaced** unit | 4 → **all 4 FALSE POSITIVES** (`#1 ps[idx]`, `#2 s = ~s`, `##1 s ##1` — delays followed by *signals* named `ps`/`s`) |
  | `timeunit`/`timeprecision` + tight unit (control) | 54 |
  | `` `timescale `` + tight unit (control) | 219 |

  ⇒ **the ecosystem writes time literals TIGHT, universally: 273 control hits, 0
  genuine spaced ones.** ⭐ And footnote 44 has a real lexical reason rather than
  being pedantry: if white space were legal there, `#10 ns` would be ambiguous
  with delay `10` followed by an identifier `ns` — which is exactly the shape of
  all four false positives above. That is almost certainly why simulators lex a
  time literal as one token, i.e. why they reject the spaced form too.
- **⇒ ADJUDICATED: fix STRICTLY, and NO switch is needed for this leaf.** The
  strict fix costs nothing measurable — keyed corpus population is **0 rows**, so
  it cannot break a currently-passing file. Enforcing footnote 44 is a pure
  accepts-invalid repair with no dialect trade-off to make. **This leaf must NOT
  be used as the motivating case for a strictness switch** (see the note below):
  designing a general mechanism around a case with zero measured conflict is
  designing against a hypothesis.
- **FIX (candidate, now unblocked):** make `time_literal` lexical (a single
  terminal spanning number+unit with no interior white space), or add a
  no-white-space guard between the two rules. Check `time_unit:5555`'s other
  consumers before choosing. Expected: accepts-invalid population drops by however
  many of the tracked 21+14 rows are this defect; verify with a keyed set-diff.
  ⚠️ It is still an accepts-invalid (TIGHTENING) fix, so the per-FILE pass-set diff
  matters more here than in any `.3.x` leaf so far — a tightening change is the one
  shape that CAN turn passing files into failing ones.
- **FIX (candidate, pending that adjudication):** make `time_literal` lexical (a
  single terminal spanning number+unit with no interior white space), or add a
  no-white-space guard between the two rules. Check `time_unit:5555`'s other
  consumers before choosing.

---

**⏳ SESSION #207 (2026-07-26) — DIAGNOSIS COMPLETE AND FIX WRITTEN AND VERIFIED,
BUT DELIBERATELY NOT LANDED. Status stays `todo`; it is now BLOCKED on a missing
PGEN primitive, and the block is MEASURED, not asserted.** Evidence bundle:
`docs/tasks/artifacts/sv_corpus_grad/time_literal_ws_diag/` (`before.txt`,
`after.txt`, `transitions.txt`, `trace_before_differential.txt`,
`sweep_lexical_adjacency.txt`, `keyed_population.txt`, `census_effect.txt`,
`global_measurement.txt`, `design_adjudication.txt`, `generator_shape_probe.txt`,
plus the re-runnable drivers `run_matrix.sh`, `run_generator_probes.sh`,
`analyze_transitions.py`, `analyze_corpus_delta.py`, `analyze_manifest_delta.py`,
`scan_population_raw.py`, `scan_population_stripped.py`).

- **⭐⭐ THE CHARTER UNDERSTATED THE DEFECT BY A WHOLE DIRECTION. It is not only an
  accepts-invalid nicety with "zero measured conflict" — it is ALSO a
  rejects-valid defect that BREAKS LEGAL REAL-WORLD CODE, and it owns 2 of the
  382 tracked `unexplained_rejects_valid` rows.** Because `s`/`ms`/`us`/`ns`/`ps`/
  `fs` are ordinary identifier spellings, the over-permissive `time_literal`
  STEALS the ubiquitous `<number> <white space> <signal>` pair:

  | real corpus shape | verdict today |
  |---|---|
  | `#1 ps[idx] = 1'b1;` (iverilog `ivltests/pr2785294.v:22`, `ps` = a reg array) | **REJECT** |
  | `#2 s = ~s;` (Surelog `tests/FSMBsp13/top.v:63`, `s` = a reg) | **REJECT** |
  | `trans ##1 start_trans ##1 s ##1 end_trans;` (ispras `16.08_04.sv:19`, ×2 files) | **REJECT** |
  | the SAME shapes with a non-unit identifier (`qs`, `t`, `zz`) — control | ACCEPT |

  Trace-proven: `Rule 'time_literal' successfully parsed from 48 to 51 (consumed
  3 bytes: '2 s')`. ⭐ That is exactly the lexical ambiguity footnote 44 exists to
  prevent — which is also why simulators lex a time literal as one token. The
  `.3.10` sweep had dismissed all four of these as "FALSE POSITIVES"; they are
  false positives *as time literals* and that is precisely the point — PGEN
  parses them as time literals anyway.
- **A SECOND, INDEPENDENT FACE THE CHARTER DID NOT HAVE — the NUMBER CLASS.**
  Annex A A.8.4 is `time_literal ::= unsigned_number time_unit |
  fixed_point_number time_unit`, but `time_literal:495` referenced the full A.8.7
  `number`, so `timeunit 1e3ns;`, `1.5e3ns`, `4'd10ns`, `4'b10ns` and `'d10ns`
  all wrongly ACCEPT. Different law from footnote 44, same rule, same direction.
- **ROOT CAUSE (WHY + WHERE), trace-proven DIFFERENTIALLY**
  (`trace_before_differential.txt`): on `timeunit 10 ns;` vs `timeunit 10ns;` the
  SAME branch wins — `Rule 'kw_ns_7320d5b7' successfully parsed from 23 to 26
  (consumed 3 bytes: ' ns')` versus `from 23 to 25 (consumed 2 bytes: 'ns')`. The
  token's own regex matches one byte later, but the RULE still spans the space:
  the terminal's layout skip swallowed it. WHERE: `time_literal:495`, spelled
  `number time_unit` — two rules, so PGEN's unconditional pre-terminal layout
  skip opens the seam the LRM closes. Same engine property as `.3.5`/`SV-0041`,
  `.3.9`/`SV-0045`, `.3.10`/`SV-0046`.
- **⭐ THE FIX WAS WRITTEN AND IS FULLY PARSER-VERIFIED** — a positive lookahead
  on the fused lexeme, `time_literal := &/[0-9][0-9_]*(\.[0-9][0-9_]*)?(s|ms|us|ns|ps|fs)\b/
  number time_unit -> {value: $2, unit: $3}`. Measured on a regenerated parser:
  all **38** matrix rows exactly as designed (13/13 legal spellings still ACCEPT,
  7/7 footnote-44 rows ACCEPT→REJECT, 7/7 A.8.4 rows ACCEPT→REJECT, 5/5
  over-tightening guards still REJECT, 4/4 real-world shapes REJECT→ACCEPT,
  `1step` untouched); `{value, unit}` preserved BYTE-IDENTICALLY; census
  BYTE-INERT (1475→1475, per-rule profile map identical across all 1475 rules);
  **main corpus pass 9,693→9,698 with 0 pass→fail, rejects-valid 382→380 (both
  healed rows the keyed files, ZERO new), accepts-invalid 21 SET-IDENTICAL**;
  v2005 lane 0 transitions, manifest BYTE-IDENTICAL; `sv_syntax_closure_gate`
  PASS; `ast_shape_contract_gate` PASS 18/18.
- **⛔ AND THEN IT WAS REVERTED, because `sv_cert_recognized_union_gate` went RED
  on a GEN↔PARSE DUALITY BREAK: `sample_parse_failures` 22 / 16 / 17 at seeds
  0 / 7 / 42 (expected 0).** The certificate accounting itself stayed perfect
  (union UNKNOWN 0, witness 1346, residual `[]`); the generator simply emits
  samples the strict parser rejects — verbatim `timeprecision 0//x\n//x\nps;`
  and `timeunit 8 'O   z//x\n//x\ns`.
- **⭐ THE DECIDING MEASUREMENT (`generator_shape_probe.txt`) — three grammar
  shapes, one generator run:** `num unit` with `trivia`-prefixed unit tokens
  emits `A 7904 ns`; the SAME structure with the `trivia` prefix REMOVED still
  emits `B 8918 s` — **the generator inserts a separator between EVERY pair of
  sequence elements, independently of `trivia`** — while a single fused terminal
  emits `C 6358s` / `C 19.20808ns`, tight every time. ⇒ **PGEN cannot express "no
  layout between these two elements"; an LRM lexical-adjacency constraint must be
  ONE terminal.** That is exactly what the other three Annex-A adjacency
  footnotes already are (fn 33 / 48 / 50 — all single regexes, all 15/15 rows
  measured correct in `sweep_lexical_adjacency.txt`). `time_literal` was the only
  one written structurally, and the structural form is the one that cannot be
  made strict.
- **ALL FOUR ROUTES MEASURED AND EACH BLOCKED** (`design_adjudication.txt`):
  1. structural + lookahead — parser-perfect, **generator breaks** (the red gate).
  2. fused terminal — duality-complete, but orphans `time_unit:5555` **and its 6
     `kw_*` tokens** (grammar-wide grep: `time_literal` is its ONLY consumer).
     `time_unit ::= s | ms | us | ns | ps | fs` is a genuine Annex-A production ⇒
     `feedback_no_rule_deletion_without_lrm_proof` forbids removing it and the
     lint/closure gates forbid leaving it unreferenced. It would ALSO flatten
     `{value, unit}` (a regex terminal binds exactly one value — no capture-group
     → `$N` mapping exists in codegen), i.e. a schema break on a rule NEXSIM reads.
  3. `@sample: "10ns"` generator pin (annotation tier, the highest fix tier) —
     MEASURED to pin the text correctly, but when the pinned rule is the SOLE
     path to `time_unit` (the SV situation) generator coverage COLLAPSES to
     **rules 3/13, branches 0/6**, stranding `time_unit` + 6 tokens toward cert
     UNKNOWN and breaking `fully_certified_via_union`.
  4. a no-layout boundary primitive — **does not exist**, on either side:
     all 1,798 `match_regex` call sites pass `skip_leading_whitespace = true`
     with no opt-out, and the generator's separator insertion is unconditional.
- **⛔ NOTHING WAS DEGRADED TO MAKE A GATE PASS.** No LRM rule deleted, no cert
  contract re-baselined to absorb a duality break, no schema broken, no gate
  re-specified. Grammar, `ast_shape_contract.rs` and the corpus characterization
  outputs are all restored to HEAD behaviour and the parser regenerated from the
  restored grammar.
- **⇒ BLOCKED ON `LEX-ADJACENCY`** (new tree, opened this session): a NO-LAYOUT
  LEXICAL BOUNDARY that BOTH the parser and the stimuli generator honour. Once it
  lands, route 1 applies unchanged — the grammar edit is written verbatim above
  and this leaf's matrix + corpus lanes are the ready-to-re-run acceptance
  evidence.
- ⭐ **UPDATE (session #208, `LEX-ADJACENCY.1` — `PGEN-LEX-ADJACENCY-0001`): the
  blocker is SMALLER than this leaf concluded, and one of its findings is
  CORRECTED.** `LEX-ADJACENCY.1` measured that the no-layout capability is **not
  missing** — it ships on both halves, and is merely **un-declarable**: the parse
  half is LIVE in `generated/return_annotation_parser.rs` (**10 of 20**
  `match_regex` sites pass `false`) behind a hard-coded rule-NAME `matches!` arm,
  and the generate half is LIVE as `atomic_token_depth` but INFERRED from the
  return shape. ⇒ **this leaf's "ROUTE 4 … DOES NOT EXIST" is superseded.** The
  four routes it priced were each genuinely blocked and that work stands; what it
  missed is a FIFTH trigger (`-> $text` / `@transform`) that was never probed, and
  a parse-half claim generalized from SV alone (`0 of 1,798`) without checking
  another grammar. The corrected design (rule-level `@lexical_token`, deep +
  interior-only, statically emitted) is
  [`LEX-ADJACENCY-design.md`](LEX-ADJACENCY-design.md); this leaf stays `todo`,
  still blocked, but now on a **scoped, unblocked `.2`** rather than on an
  unpriced capability. ⚠️ It also inherits a NAMED trap: `@lexical_token` is deep,
  and §5.7.1 (the `.3.10` law) leaves some based-literal seams deliberately OPEN —
  `time_literal` is safe only because A.8.4 restricts its number to
  `unsigned_number`/`fixed_point_number`, which must be re-verified when the fix
  is re-applied, not assumed.
- **BY-PRODUCT ALREADY LANDED (independent of the block):** `TOOLBOX.md` §2.2 now
  documents that `--trace-rules R` traces R's **dynamic extent**, so naming a
  suspect leaf rule alone can print NOTHING while that rule succeeds — measured
  here (`time_literal` alone → 0 lines; its caller `cycle_delay_range` → the line
  that is the whole root cause). ⚠️ This also retro-weakens one leg of `.3.10`'s
  jitter argument, which used trace-emptiness as proof of non-entry; recorded in
  `global_measurement.txt`, where `--dump-rule-entry-counts-json` shows
  `time_literal` entered **28,643** times on the very file `.3.10` reasoned about.

#### `.3.12` — a RECURSION-GUARD rejection is cached in a recursion-BLIND memo, so a legal parse is refused (the packrat × cycle-guard composition gap; ⭐ the exact sibling of `MEMO-STORE-SOUNDNESS` F1, on the OTHER context axis)

- **Status: `in progress`** (session #215, 2026-08-08). Cut from the HEAD-vintage
  382-row cluster map (`.10`) — entered as the *size/type cast* family (12 rows,
  clusters `' ( ID` 7 + `' ( '` 2 + …) and root-caused to an **engine-tier
  soundness defect that is not cast-specific and not SV-specific**.
- **Why this leaf and not a grammar leaf:** the fix hierarchy (declarative >
  grammar > engine) is a preference for the LOWEST tier that reaches the root
  cause. Measured below, the grammar is already LRM-faithful on the construct
  that fails — `casting_type ::= … | constant_primary` is present, and the rule
  parses the input CORRECTLY in isolation. No grammar edit can reach the cause.

**REPRODUCE — the minimal input, and the pair that isolates it.**

```
module m;
  localparam int P = 8;
  logic [7:0] x;
  initial x = $clog2(P)'(P);     # REJECT  (furthest_position=74)
endmodule
```

| input (same skeleton) | verdict |
|---|---|
| `4'(P);` — literal casting_type | **PASS** |
| `(P)'(P);` — parenthesised `constant_primary` | **PASS** |
| `(P+1)'(P);` — parenthesised expression | **PASS** |
| `P'(P);` — `ps_parameter_identifier` | **PASS** |
| `$clog2(P)'(P);` — `system_tf_call` casting_type | **REJECT** |
| `$bits(P)'(P);` | **REJECT** |
| `f(P)'(P);` — plain `tf_call` casting_type | **REJECT** |

⇒ the failing class is exactly *casting_type = a CALL* (IEEE 1800-2017 A.8.4
`casting_type ::= … | constant_primary`, A.8.4 `constant_primary ::= … |
constant_function_call`, A.8.2 `constant_function_call ::= function_subroutine_call`
⊇ `system_tf_call` / `tf_call`). All three spellings are LRM-legal.

⭐ **AND THE RULE ITSELF IS INNOCENT — proven, not argued.** Driving the SAME
input through the SAME parser with `--entry-rule` (TOOLBOX 1.2, entry-relative
AST dump) parses it:

```
parseability_probe --parse-dump-ast-pretty systemverilog cast_only.txt out.json \
    --profile sv_2017 --entry-rule cast          # "$clog2(P)'(P)"  -> parse_full passed
parseability_probe … --entry-rule casting_type   # "$clog2(P)"      -> parse_full passed
parseability_probe … --entry-rule primary        # "$clog2(P)'(P)"  -> REJECT, stops at 9
```

`primary` stops at **9** — the length of `$clog2(P)` — i.e. it selected the
`call_primary` branch and never took the 13-char `cast` branch, under a
`branch_policy=longest_match` tournament that must prefer the longer match.

**ROOT CAUSE (WHY + WHERE) — the trace names the mechanism.**

`PGEN_TRACE_VERBOSITY=debug … --entry-rule primary --trace-rules primary_sv_2017,cast`
(⚠️ per TOOLBOX 2.2 the PARENT is traced, not only the suspect):

```
🚪 Entering branch 8/14 for rule 'method_call_receiver_sv_2017' at position 0
💾 Memo miss for rule 110 at position 0 - computing fresh result        # 110 = cast
💾 Memo miss for rule 111 at position 0 - computing fresh result        # 111 = casting_type
💥 Infinite recursion detected in rule 'call_primary' at position 0
❌ Exiting rule 'constant_function_call' with error: Backtrack { position: 0 }
💥 Infinite recursion detected in rule 'casting_type' at position 0
❌ Exiting rule 'constant_cast' with error: InvalidSyntax { message: "Infinite recursion detected" }
❌ Exiting rule 'constant_primary_sv_2017' with error: Backtrack { position: 0 }
💾 Memoized failed result for rule 111 at position 0
💾 Memoized failed result for rule 110 at position 0                    # ← the poisoning
…
🚪 Entering branch 9/15 for rule 'primary_sv_2017' at position 0        # 9/15 = the `cast` branch
💾 Memo hit for rule 110 at position 0 - cached failure                 # ← replayed out of context
❌ Exiting rule 'cast' with error: Backtrack { position: 0 }
🏁 Rule 'primary_sv_2017' selected branch 2/15 consuming 9 chars (branch_policy=longest_match)
```

- **WHY.** The FIRST attempt of `cast` at position 0 happens *inside*
  `call_primary`'s own frame (`call_primary → method_call → method_call_receiver
  → cast`). From there the chain `cast → casting_type → constant_primary →
  constant_function_call → call_primary` re-enters `call_primary` at the SAME
  position, so `RecursionGuard::check_cycle_id` correctly returns
  `CycleType::Infinite` and the attempt fails. **That failure is a fact about the
  PARSE STACK, not about `(rule, position)`** — yet `memoized_call` files it in
  `memo_fail`, whose key is `(rule_id, position)` and nothing else. When
  `primary_sv_2017` later reaches its own `cast` branch at position 0 — with
  `call_primary` no longer on the stack, where the guard would NOT fire — the
  store-blind cache replays the stale failure and the legal 13-char cast is never
  attempted.
- **WHERE.** `rust/src/ast_pipeline/ast_based_generator.rs` — the cycle guard
  emitted at `:3869` (`check_cycle_id` → three blocking arms) and the split memo
  at `:8891` (`memoized_call`: `memo_fail` / `memo_fail_tainted` / `memo`).
  Mirrored on the fused bare path in
  `rust/src/ast_pipeline/ast_based_generator/cascade.rs:629` (guard) + its thin
  memo below it, and in the interpreter
  `rust/src/parse_harness_interpreter.rs:1088` (`memoized_call`) whose
  `parse_rule` depth ceiling (`:854`) is the same context-dependent shape.
  ⇒ measured: **both graphs reject identically** (bare `--parse`, and the
  protocol graph forced via `--dump-rule-entry-counts-json`), so the
  observability twin is intact and BOTH memos carry the defect.
- ⭐ **THIS IS `MEMO-STORE-SOUNDNESS` F1, ON A SECOND CONTEXT AXIS.** That tree
  closed *"the failure cache is store-blind"* with taint + epoch validation. The
  memo is also **recursion-blind**: an outcome produced under a cycle-guard
  rejection is not a function of `(rule, position)` either. The two defects have
  the same shape, the same blast radius (every generated parser), and the same
  cure family — which is why the fix below is deliberately modelled on it rather
  than invented.

**FIX — tier: ENGINE (codegen template + the shared `RecursionGuard`). Two
candidate scopes were implemented; the first was REJECTED BY MEASUREMENT.**

⛔ **CANDIDATE 1 — "any cycle-guard rejection anywhere in the body taints it"
(a monotone counter). Implemented, regenerated, measured, and REJECTED.** It is
sound and it fixed the defect (all 8 reproducer rows PASS), but it is not
shippable: on `verilator/test_regress/t/t_math_synmul_mul.v` — a file the `.10`
baseline PASSES inside a 60 s budget on the release probe — the parse ran
**past 300 s** (`timeout 300` → `rc=143`, wall 3 m 51 s). Cause: in a deeply
mutually-recursive grammar almost every expression body has SOME block
somewhere in its subtree, so a subtree-wide taint deletes most of the packrat
protection. ⭐ **This is the same trap `MEMO-STORE-SOUNDNESS.2` hit on the store
axis** — its exclusion thesis measured **117×** on SV and was replaced by
epoch *validation*. Recorded here because the identical mistake was available
again and only a measurement caught it: the leaf that closed the store axis
says so in as many words, and it still had to be re-learned by running it.

✅ **CANDIDATE 2 — FRAME-SCOPED taint (landed).** The insight the counter form
misses: a guard rejection is context-dependent only with respect to frames
**outside the memoized rule's own subtree**. A block whose blocking frame is
that rule itself, or one of its own descendants, is re-created identically by
every replay of that body and is therefore harmless.

- `RecursionGuard` gains `last_block_frame` — the stack index of the frame that
  caused the verdict, written only on the two already-failing return paths of
  `check_cycle` / `check_cycle_id`. Both scans return on the FIRST (oldest,
  shallowest) match, so the index is exactly the floor a caller needs. The hot
  `CycleType::None` fall-through writes nothing.
- The parser gains `recursion_block_floor: usize` (`usize::MAX` = none). Each
  guard arm lowers it: `Infinite` / `LeftRecursive` to the blocking frame's
  index, the over-depth `MutualRecursive` arm to **0** — that ceiling is a fact
  about the WHOLE stack, so it taints unconditionally.
- `memoized_call` and the fused thin memo open a scope around the body (save the
  caller's floor, reset to `usize::MAX`, note `entry_depth`), and on exit hand
  the floor up min'd with the caller's. The entry is tainted — and a tainted
  **FAILURE** is therefore not cached (see `2a` below for why successes are) —
  iff `floor < entry_depth - 1`, i.e. the block came
  from a strict ancestor. Because each level re-applies the same test against its
  own depth, taint propagates up **exactly as far as the ancestor that owns the
  blocking frame, and stops there**.
- ⛔ Deliberately NOT epoch-validated like the store taint: there is no monotone
  "recursion epoch" to compare against — the guard's verdict is a function of the
  live stack, which no scalar summarizes.

⛔⛔ **CANDIDATE 2a — "refuse BOTH polarities" — WAS ALSO MEASURED AND REJECTED, and
this is the leaf's most valuable finding.** Symmetry with the store axis was assumed,
implemented, and the corpus refuted it: **4 files regressed pass→fail** —
`ispras-sv-tests/ieee-1800-2012/16/16.14.06.01_03.sv` and `_05.sv`,
`opentitan/hw/vendor/pulp_riscv_dbg/src/dm_sba.sv`,
`verilator/test_regress/t/t_reloop_local.v` — every one of them a CAST used inside an
INDEX in a cyclic expression context:

```
a5: assert property (foo[const'(i)] && bar[i]);          # ispras  (minimal: e3.sv, furthest=76)
assert property (@(posedge clk) shuffle[ctr+Word'(i)] == i);   # verilator
be_mask[int'({be_idx[$high(be_idx):1], 1'b0}) +: 2] = '1;      # opentitan
```

Trace on the minimal case names the mechanism exactly as before —
`💥 Infinite recursion detected in rule 'call_primary' at position 62` and
`… 'casting_type' at position 73` — but with the OPPOSITE consequence: here the cached
**success** was what made the parse work. Fresh re-derivation at that position, with
`call_primary` on the stack, is blocked by the guard, so the parse that the cache used
to supply is simply lost.

⭐ **THE ASYMMETRY IS PRINCIPLED, NOT A PATCH — and it is the opposite of the store
axis's.** The semantic store changes what the CORRECT ANSWER IS, so a stale success
there is genuinely wrong (`MEMO-STORE-SOUNDNESS.1` measured one flipping both the verdict
AND the tree). A cycle guard changes only what the SEARCH CAN REACH — the language is
untouched. So:

| | cached FAILURE | cached SUCCESS |
|---|---|---|
| what it asserts | "no derivation from here" | "this derivation exists, and was found" |
| what the guard actually established | only that *the search stopped* | nothing — the derivation is real |
| replay in a different stack | **unsound** → refuses legal input (the bug) | over-permissive w.r.t. the GUARD, exactly right w.r.t. the GRAMMAR |
| verdict | **refuse to cache** | **keep caching** |

⇒ **the shipped gate is FAILURES ONLY.** And the success-side replay is not merely
tolerable, it is **LOAD-BEARING**: it is how this engine parses indirect
left-recursive constructs at all. That is worth stating plainly because it is a
property of the engine nobody had written down — *the packrat memo is not only a cache
here; on cyclic rules it is part of the acceptance semantics.* Whether such a construct
parses depends on which context evaluated it first, which is also the deeper reason the
original defect existed. Routed as a standing finding, not silently absorbed → `.11c`.

**Mirrored at every memo site, so the two graphs and the oracle cannot drift:**
`ast_based_generator.rs` (protocol memo) · `ast_based_generator/cascade.rs`
(the fused bare graph's thin memo — the DEFAULT path for a plain `--parse`, and
measured to carry the defect identically) · `parse_harness_interpreter.rs` (the
differential oracle). ⚠️ The interpreter keeps a **counter** rather than a floor,
and that is exact rather than sloppy: it has no cycle guard at all — its depth
ceiling is its whole runtime-cycle-breaking path — and a depth ceiling is a
whole-stack fact, so its floor would be 0 at every site, which is what "any hit
taints" already means.

**Regression pin (durable, in a gate that RUNS):** a new isolating case
`recursion_guarded_memo_isolation` in the `.6.1` structural combinator suite
(`make -C rust parse_harness_combinator_gate`), on a 6-rule synthetic grammar
that reaches `cast` at position 0 once from inside the cycle and once from
outside it. It is a differential AND an anchor case, so it fails on both a
divergence and a wrong absolute verdict. ⛔ Deliberately NOT a bare `#[test]`:
`cargo test --lib` is RED on HEAD and no gate reads it
(`CI-PARITY-GATE-ROT.21`), so a pin placed there would be unreachable — the
`GATE-REACHABILITY` principle applied before the fact instead of after.

**MEASUREMENT (candidate 3, the shipped gate).**

*The construct matrix — same skeleton, `initial x = <expr>;`, `--profile sv_2017`:*

| casting_type | before | after |
|---|---|---|
| `4'(P)` literal · `(P)'(P)` paren · `(P+1)'(P)` expr · `P'(P)` param | PASS | PASS (unchanged) |
| `$clog2(P)'(P)` · `$clog2(8)'(3)` · `$bits(P)'(P)` — `system_tf_call` | **REJECT** | **PASS** |
| `f(P)'(P)` — a plain user `tf_call` | REJECT | REJECT ⚠️ see the honest bound |

*The corpus, 16 336 files, release probe at 60 s (`analyze_transitions.py`, the `.10` instrument
with its positive + negative controls):*

| | before | after |
|---|---|---|
| pass | 9 694 | **9 712 (+18)** |
| **pass → fail** | — | **0** |
| **pass → timeout / crash** | — | **0** |
| timeout | 4 | 4 (the same `.11a` xbar files) |
| unexplained divergences | 403 | **393** |
| — rejects-valid | 382 | **372 (−10)** |
| — accepts-invalid | 21 | **21 (unchanged)** |

Every changed row is `fail → pass`, spread across **six independent sub-corpora** — opentitan 7,
Surelog 4, sv2v 3, verilator 2, black-parrot 1, ispras 1 — which is what distinguishes an engine fix
from a fixture accident.

*The `verilog_2005` lane, re-run rather than inferred (the engine change is profile-blind, so it had
to be):* 2 459 files → **2 180 pass / 279 fail / 0 timeout**, per-file transitions **ZERO**.

*Performance, the axis that killed candidate 1:* `t_math_synmul_mul.v` — **2.59 s, accepted**
(candidate 1: killed at `timeout 300`). The corpus wall-clock stays in the same band as the `.10`
baseline's 116 s.

⚠️⚠️ **HONEST BOUND, stated because it is a real limit and not a rounding error: `f(P)'(P)` does NOT
heal, and it healed under candidate 2.** A cast whose `casting_type` is a plain user `tf_call` still
rejects; the `system_tf_call` spellings (`$clog2`, `$bits`) — the ones the corpus actually contains —
do heal. The mechanism is the flip side of the success-caching decision: a recursion-tainted SUCCESS
is now replayed, and on that input the replayed derivation wins a tournament the fresh one would
have lost. ⛔ It is **not a regression** — `f(P)'(P)` rejects on the pre-`.3.12` baseline too — it is
an un-healed member of the same class, and the corpus contains zero instances of it. The design that
would close it *and* keep the 4 files is the **validated** form (store each tainted entry's blocked
queries and re-run `check_cycle_id` at replay, instead of the current refuse-failures/keep-successes
approximation). That is strictly more machinery for a case with no corpus population, so it is
routed to `.11c` with the seeded-left-recursion work rather than built here.

⭐ **What the three candidates cost, kept as the leaf's real product.** Candidate 1 was refuted on
PERFORMANCE, candidate 2 on CORRECTNESS, and both refutations came from the same 16 336-file
measurement; neither was reachable by reasoning, and candidate 2's error was specifically *reusing a
neighbouring axis's soundness argument without asking what its condition establishes*. The two
generalisations are promoted to
`docs/knowledge/a-memo-key-must-name-every-context-the-outcome-depends-on.md`.

⚠️ **Two honest scope bounds on the regeneration, neither introduced here.**
`generated/ebnf.rs` is a BOOTSTRAP seed that `regenerate_generated_parsers` only
produces when it is ABSENT (it covers "the annotation pair + 7 grammar families"),
so the `ebnf` registered parser still carries the pre-fix memo; that is the
repository's standing bootstrap posture — the seed was already 9 days behind other
codegen work before this leaf — not a regression created by it. Same for
`generated/scratch_parser.rs`, which `focus_scratch` owns. Both inherit the fix the
next time they are produced; the fix itself is in the shared codegen, not in any
artifact.

- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — `parseability_probe --parse systemverilog cast_b.sv
    --profile sv_2017` → `Parser did not consume full input at position 0
    [furthest_position=96]` on `initial x = $clog2(P)'(P);`, an IEEE 1800-2017
    A.8.4-legal size cast; `$bits(P)'(P)` and `f(P)'(P)` reject identically while
    `4'(P)` / `(P)'(P)` / `(P+1)'(P)` / `P'(P)` all pass.
  - [x] **ROOT CAUSE (WHY + WHERE)** — `PGEN_TRACE_VERBOSITY=debug …
    --entry-rule primary --trace-rules primary_sv_2017,cast` prints, in order:
    `💥 Infinite recursion detected in rule 'call_primary' at position 0` →
    `💾 Memoized failed result for rule 110 at position 0` →
    `🚪 Entering branch 9/15 for rule 'primary_sv_2017' at position 0` →
    `💾 Memo hit for rule 110 at position 0 - cached failure` →
    `🏁 Rule 'primary_sv_2017' selected branch 2/15 consuming 9 chars
    (branch_policy=longest_match)`. WHERE: the guard at
    `ast_based_generator.rs:3869` (mirrored `cascade.rs:629`) and the split memo at
    `ast_based_generator.rs:8891` — a stack-dependent verdict filed under the
    stack-blind key `(rule_id, position)`. The rule is exonerated by the SAME parser
    on the SAME bytes: `--entry-rule cast` → `parse_full passed`.
  - [x] **FIX** — tier: ENGINE (no lower tier can see it — the grammar is already
    LRM-faithful and `cast` parses the input standalone). `RecursionGuard::last_block_frame`
    + a per-body `recursion_block_floor` scope; a recursion-tainted FAILURE caused by a
    STRICT ANCESTOR frame is not cached. Two earlier scopes measured and rejected (above).
  - [x] **ADDRESSED (verified)** — oracle: `stimuli/run_external_corpus.sh sv 60 8 0`
    under the memory guard with `PGEN_PARSE_PROBE_BIN=rust/target/release/parseability_probe`,
    joined by `analyze_transitions.py` (which carries a positive identity-self-join control
    and a planted-flip negative control). **corpus pass 9 694 → 9 712 (+18)**, unexplained
    `403 → 393`, rejects-valid `382 → 372`; the 3 `system_tf_call` matrix rows flip
    REJECT→PASS; `t_math_synmul_mul.v` 300 s+ → **2.59 s**.
  - [x] **NO REGRESSION** — per-FILE census over all 16 336 rows: **0 pass→fail,
    0 pass→timeout, 0 pass→crash**; accepts-invalid **21 identical**; `verilog_2005`
    lane re-run 2 180/279/0 with **ZERO** transitions of 2 459. ⭐ **CROSS-FAMILY, because the fix is
    in SHARED codegen and SV being clean does not prove VHDL is:** the VHDL corpus re-run —
    **13 720 files, 4 335 pass / 9 385 fail / 0 timeout, ZERO per-file transitions.**
    Re-runnable oracles, all
    green: `CERTIFICATE-COVERAGE: … total=1352 proof=17 witness=1335 UNKNOWN=0
    fully_certified=true (sample_parse_failures=0)` **identical at seeds 0, 7 and 42**;
    `ast_shape_contract_gate` **18/18**; `parse_harness_combinator_gate` **28/28 CLEAN**
    (incl. the new pin); `parse_harness_equivalence_gate` **4/4** (the interpreter stays
    byte-identical to all 11 certified generated parsers, which is what proves the
    interpreter mirror did not drift); `parse_harness_semantic_gate` **36/36 CLEAN**.
  - [x] **LOCKSTEP** — release `1.0.176 → 1.0.177` (schema **19 UNCHANGED**, proven by
    the shape gate rather than argued) + contract Current-state note; ledger row
    `SV-0047`; SV book changelog; `TOOLBOX.md` 27→28 cases; the parse-harness book
    chapter + `PARSE-HARNESS.md` §20 LIVE-SPEC note; *Inside Parser Performance* gains the
    fourth taint class AND has its "replay and re-execution agree" premise CORRECTED —
    that claim was false on this axis, and the P1b conclusion it supported survives for a
    different reason (a cache miss re-executes), which the chapter now states;
    `KNOWLEDGE_MAP` + the new card; `CHANGES.md`; `DEVELOPMENT_NOTES.md`; `MEMORY.md`;
    `docs/TASK_TREE.md`; `.11c` opened for the routed finding.

#### `.3.13` — the ch22 "compiler directives" family is an ADJUDICATOR defect, not a parser one: `__FILE__`/`__LINE__` are predefined text MACROS sitting in the known-DIRECTIVES allowlist (⭐ and the family took THREE verdicts: 27 macro rows corrected, 8 routed to `.3.14` as a real grammar gap, 2 found to be §34 protected envelopes)

- **Status: `done`** (2026-08-08, session #216 — see the EXECUTION block below; the
  diagnosis banked below is session #215's and is preserved verbatim, including the
  **34** it recorded, which the execution re-measured to **37**).
- Diagnosis complete and tool-pinned (session #215, 2026-08-08),
  cut from the post-`.3.12` **372-row** map where this is the largest NAMED family
  (**34 rows**: iverilog 22 / verilator 7 / sv-tests 3 / ispras 1 / sv2v 1; clusters
  `` ` ID ) `` 15, `` ` ID , `` 11, `` ` ID ID `` 6).
- **REPRODUCE.** `iverilog/ivtest/ivltests/fileline.v` is nine lines whose whole point
  is `$display(`__FILE__);`, and its own header quotes the clause: *"P1800/D8 22.13 —
  `__FILE__ expands to the name of the current input file, in the form of a string
  literal."* The parser rejects it; the manifest row reads
  `must_accept / divergence:unexplained_rejects_valid`, basis *"compiles under the
  iverilog SV dialect, parse-level valid"* — which is true **after preprocessing**.
- **ROOT CAUSE (WHY + WHERE).** `stimuli/sv/adjudicate_external_corpus.py:106` —
  `KNOWN_DIRECTIVES` contains `"__FILE__", "__LINE__"`. `preproc_dependency()`
  (`:149`) walks every `` ` ``-prefixed name and returns `macro_use` **only for names
  NOT in that set**, so a file whose sole preprocessing dependency is a predefined
  text macro is judged to have none and falls through to `unexplained`. ⭐ The set
  conflates two things the LRM separates: a **compiler directive**
  (`` `timescale ``, `` `default_nettype ``…) steers the preprocessor and leaves the
  surrounding text parseable, whereas a **predefined text macro** (IEEE 1800-2017
  §22.13) *expands to a value* and the expression around it is not parseable until it
  does. Evidence that the mechanism is otherwise sound and this is a hole in it, not a
  missing feature: the manifest already carries **1 074**
  `divergence:explained_svpp_macro_use` rows — the detector works, these two names
  are simply on the wrong side of its allowlist.
- ⛔ **THE TRAP THIS LEAF MUST NOT FALL INTO, stated before any work starts.**
  Reclassifying rows lowers `unexplained` **without fixing a parser defect**, which is
  the precise shape of gaming the tree forbids
  ([[feedback_corpus_expected_from_spec_not_fix]]). It is legitimate here ONLY because
  the criterion is spec-derived and mechanical — §22.13 defines these as macros that
  expand — and it must be reported as an **adjudication correction**, never as
  burn-down yield. The `.5` graduation bar counts unexplained divergences, so a
  reclassification that is not spec-derived would corrupt the bar itself.
- ⚠️ **AND THE FAMILY IS NOT HOMOGENEOUS — do not reclassify it wholesale.** The
  `` ` ID ID `` cluster is `` `pragma protect encoding=(enctype="raw") ``
  (ispras `34.03.01_01.sv`). `pragma` IS a genuine compiler directive with a NORMATIVE
  parse surface in IEEE 1800-2017 §22.11 (`pragma_expression`, `pragma_value` — Annex
  A.1.2 `pragma_directive`), so those rows are a candidate **real grammar gap**, the
  opposite verdict from the `__FILE__` rows. The leaf's first act is therefore to SPLIT
  the 34 by directive name with an LRM cite each, not to move them.
- **Sequencing when cut:** split by directive name → per-name LRM adjudication →
  fix the allowlist for the macro subset (spec-derived, one constant) → measure the
  reclassification separately from any grammar work → re-baseline with the split
  stated in both directions.

##### `.3.13` EXECUTION — `done` (2026-08-08, session #216, `PGEN-SV-CORPUS-GRAD-0031`)

Full record + every measurement:
`docs/tasks/artifacts/sv_corpus_grad/ch22_directive_split/README.md`.

- ⭐ **The family took THREE verdicts, not two — the "do not reclassify wholesale"
  warning was right and then some.** Split by directive name over the re-measured
  **37** backtick-stuck rows (the banked 34 summed four of the six clusters; the
  correction is recorded, not absorbed):
  - **27 rows = §22.13 predefined text MACROS** (`` `__LINE__ `` 15, `` `__FILE__ ``
    12) — the adjudicator hole. **Fixed here.**
  - **8 rows = IN-SCOPE COMPILER DIRECTIVES** (`` `default_nettype `` 3,
    `` `pragma `` 2, `` `line `` 1, `` `undef `` 1, `` `timescale `` 1) — a REAL
    grammar gap, left as defect signal and **routed to `.3.14`**.
  - **2 rows = §34 PROTECTED ENVELOPES** — a third class nobody predicted, found by
    measurement rather than by reading.
- ⛔ **The trap was avoided by MEASURING the "is this a directive gap?" question
  instead of adjudicating it from the file text.** `strip_probe/` blanks every
  backtick-led line (exactly the text a `compiler_directive` alternative swallows)
  and re-parses: **8 of 10** then PASS ⇒ bounded real gap; **2 still REJECT**, both
  stopping at their base64 `key_block` payload, not at a directive. Had those 2 been
  reclassified with the other 8 the bar would have absorbed a §34 dependency as a
  grammar defect — or, with a blanket `` `pragma protect `` rule, 2 genuine defect
  rows would have been silently explained away (ispras `34.03.01_01.sv` is
  `enctype="raw"` — plain text — and correctly stays in the gap half).
- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — `parseability_probe --parse systemverilog
    …/repro/C_predefined_macro.sv --profile sv_2017` → `Error: parse_full rejected …
    [furthest_position=27]` on `initial $display(`__FILE__);`, while the manifest row
    for `iverilog/ivtest/ivltests/fileline.v` read
    `must_accept / divergence:unexplained_rejects_valid` — i.e. counted against the
    graduation bar as a parser defect.
  - [x] **ROOT CAUSE (WHY + WHERE)** — the family is TWO defects, and the toolbox is what
    separated them. **Half A (this leaf's):** WHY —
    `stimuli/sv/adjudicate_external_corpus.py` `KNOWN_DIRECTIVES` held
    `"__FILE__", "__LINE__"`, so `preproc_dependency()` — which returns `macro_use` only
    for `` ` ``-names *outside* that set — reported NO dependency for a file whose sole
    dependency is a §22.13 macro, and the row fell through to `unexplained`. WHERE — that
    constant, consumed by the `TICK_RE` walk in `preproc_dependency()`; the LRM separates
    exactly what it conflated, §22.13 verbatim: *"`__FILE__ expands to the name of the
    current input file, in the form of a string literal"*
    (`docs/systemverilog/2017/md/section-22-compiler-directives.md:823`). Evidence the
    mechanism is otherwise sound rather than missing: 1 074 `explained_svpp_macro_use`
    rows already existed. **Half B (routed to `.3.14`):** the discriminator is PLACEMENT,
    proven with two probe runs on the same directive text —
    `parseability_probe --parse-dump-ast-pretty systemverilog …/repro/A_directive_top_level.sv`
    ACCEPTS and its AST carries `{"kind":"compiler_directive"}`, while the identical
    directive one line lower (`…/repro/B_directive_in_scope.sv`, inside the module)
    rejects with `furthest_position=9` (`D_timescale_in_scope.sv` 9,
    `E_undef_in_class.sv` 8). Grammar text does not change with placement, so the WHERE is
    the reachability of the rule, not its body: `grammars/systemverilog.ebnf:257` defines
    `compiler_directive` and `:241` offers it as an alternative of `source_text_item` only.
  - [x] **FIX** — declarative tier, two spec-derived edits in one script:
    (1) `__FILE__`/`__LINE__` removed from `KNOWN_DIRECTIVES` (§22.13); (2) a new
    strongest-first `protected_envelope` dependency keyed on §34.5's
    `key_block`/`data_block`/`digest_block` pragma expressions, yielding
    `divergence:explained_svpp_protected_envelope`. No parser, grammar, generator or
    generated artifact touched.
  - [x] **ADDRESSED (verified)** — ground-truth control FIRST: the pre-edit rebuild
    reproduced the tracked manifest **byte-identically** (`cmp`), so every moved row is
    attributable to the edit. After: `sv_2017` unexplained **393 → 360** (rejects-valid
    372 → 339, accepts-invalid 21 unchanged), explained 1 430 → 1 463, **match and
    deferred unchanged**; `verilog_2005` unexplained **76 → 75**. 42 rows changed class,
    all enumerated in `delta.txt`. Re-cut cluster map: 339 rows / 200 signatures, and
    the backtick population is now **exactly the 8 routed rows** — the split is visible
    in the worklist itself.
  - [x] **NO REGRESSION** — manifest deterministic (`cmp` ×2 byte-identical, both
    lanes); `bash scripts/check_doctrines.sh` green; no parser surface exists to
    regress (zero Rust/EBNF/generated bytes changed, `git diff --stat` confirms), so
    the cert/AST/corpus batteries are inert by construction rather than by assertion.
  - [x] **LOCKSTEP** — book (*Grammar Well-formedness* → the adjudication-manifest
    section, new), `CHANGES.md`, `MEMORY.md`, `docs/TASK_TREE.md`, the tracked
    manifests/summaries, the re-cut cluster artifacts, and `.3.14` cut.
- ⚠️ **THE HONEST COST, stated because an explained label is not a clean bill of
  health.** Of the 33 rows that left `unexplained`, **29 were stuck exactly at the
  macro/envelope token**; the other **4** use `` `__FILE__ ``/`` `__LINE__ ``
  elsewhere in the file and were stuck at an unrelated construct
  (`br_gh782b.v` comment/number · `sv_type_identifier_package_name.v` `T::VALUE !==` ·
  `t_randomize_within_func.v` `randomize(…) with {…}` · `t_vams_basic.v` `wreal`).
  The label is correct — those files genuinely need expansion — but it hides a real
  stuck point, so the four are ROUTED to `.9` as **crafted minimal cases** (the
  construct is the unit, not the vendored file), never dropped.

#### `.3.14` — in-scope compiler directives REJECT: `compiler_directive` is an alternative of `source_text_item` and of nothing else (IEEE 1800-2017 clause 22 — the F5 family's parser half, routed by `.3.13`)

- **Status: `done`** (2026-08-09, session #217, `PGEN-SV-CORPUS-GRAD-0033`) — cut 2026-08-08
  (session #216) from `.3.13`'s split, with the root cause already tool-pinned and the fix
  scope already measured; split into **`.3.14a`** (the adjudication ruling — `done`) and
  **`.3.14b`** (the grammar change — `done`). ⭐ The delivered split is exactly the ruling:
  **5 rows flip REJECT→PASS, 3 stay REJECT and are pinned `must_reject` on §22.8**;
  `sv_2017` unexplained `360 → 352` (−3 adjudication correction, **−5 real yield**),
  `verilog_2005` `75 → 74`, **0 pass→fail in either lane**, accepts-invalid unchanged in
  both. Two findings routed OUT: **`.3.14c`** (a column-0 `#` comment silently drops
  alternation arms) and **`.3.14d`** (the remaining item-list hosts). **8 rows**:
  verilator `t_lint_implicit_{def,func,type}_bad.v` (`` `default_nettype ``),
  sv-tests `5.6.4--compiler-directives-pragma.sv` + ispras
  `ieee-1800-2012/34/34.03.01_01.sv` (`` `pragma ``), sv-tests
  `5.6.4--compiler-directives-debug-line.sv` (`` `line ``), sv-tests
  `class_test_48.sv` (`` `undef ``), sv2v `test/core/time.sv` (`` `timescale ``).
- **ROOT CAUSE (WHY + WHERE), already measured — do not re-derive it.** The
  discriminator is placement, not spelling: `` `default_nettype none `` **before**
  `module m;` PASSES and its AST carries `{"kind":"compiler_directive"}`; the *same
  text* inside the module REJECTS at `furthest_position=9`
  (`artifacts/sv_corpus_grad/ch22_directive_split/repro/`). WHERE:
  `grammars/systemverilog.ebnf:257` defines `compiler_directive := trivia
  /`[^\r\n]*/` and `:241` offers it as an alternative of **`source_text_item` only**;
  no in-scope item list can reach it, and `:229` carries the same alternative
  commented out in `parseable_source_item`.
- **SCOPE IS MEASURED, not estimated:** blanking every backtick-led line makes **8 of
  the 8** parse (`strip_probe/probe.txt`) ⇒ directive tolerance at the in-scope item
  lists is *sufficient* for the whole routed population, and nothing else is needed.
- ⛔ **THE ADJUDICATION QUESTION THIS LEAF MUST SETTLE FIRST, because it decides
  whether the fix is legal at all.** Clause 22's syntax boxes are each marked *"not in
  Annex A"* — directives are not part of the parser's BNF surface — yet §22.8 says
  `` `default_nettype `` *"can be used only outside design elements"* and §22.3 makes
  an in-design-element `` `resetall `` *"illegal"*. So there are two coherent readings,
  and the repo has already banked one: the `.8c.2` pin for ivtest `no_timescale_in_module`
  rules an in-module-body `` `timescale `` **must_accept** — *"directive placement is
  unrestricted … the parser must tolerate it"* — and the `.8b.2` pin for ispras
  `22.14.01_04.sv` reasons *"under the directive-aware reading (the F5
  in-scope-directives family)"*. ⇒ the standing law is TOLERATE, and a placement
  "shall" in a not-in-Annex-A clause is preprocessor-stage, exactly as the
  `svpp_owned_v2005` pins already treat directive-grammar and directive-value errors.
  Confirm that reading holds against `feedback_sv_strict_lrm_compliance_default`
  (over-acceptance is a defect) **before** widening acceptance, and record the ruling —
  it governs every future directive row.
- **Sequencing:** confirm the adjudication ruling above → decide the fix tier
  (declarative reuse of the existing `compiler_directive` rule at the in-scope item
  lists vs. a narrower carrier; ⛔ prove no existing surface already covers it per
  `DESIGN-PRIOR-ART`) → measure the profile-byte-invariance of the SV ASTs → the full
  heavy battery per the tree ground rules → re-baseline the corpus.

##### `.3.14a` — the ADJUDICATION RULING (read-only; the leaf's mandated first act) — `done` (2026-08-08, session #216, `PGEN-SV-CORPUS-GRAD-0032`)

⭐⭐ **THE ROUTED 8 ARE NOT UNIFORM EITHER — the ruling this leaf was cut to confirm is
REFUTED IN PART BY THE CLAUSE TEXT.** `.3.13` handed over "the standing law is TOLERATE";
reading clause 22 directive by directive says **tolerate 5, and keep rejecting 3**. Each
row's clause was read in full (`docs/systemverilog/2017/md/section-22-compiler-directives.md`),
not sampled:

| directive | § | what the clause says about PLACEMENT | rows | ruling |
|---|---|---|---|---|
| `` `line `` | 22.12 | **"The directive can be specified anywhere within the SystemVerilog source description"** (verbatim) | 1 | **TOLERATE** — in-scope use is expressly legal |
| `` `undef `` | 22.5.2 | no placement restriction stated | 1 | **TOLERATE** |
| `` `timescale `` | 22.7 | no restriction; it governs "the design elements that follow this directive" | 1 | **TOLERATE** |
| `` `pragma `` | 22.11 | no restriction; "alters interpretation of the SystemVerilog source" | 2 | **TOLERATE** |
| `` `default_nettype `` | 22.8 | **"It can be used only outside design elements"** (verbatim) | 3 | **KEEP REJECTING** |
| (`` `resetall ``, not in this population) | 22.3 | **"It shall be illegal for the `resetall directive to be specified within a design element"** | 0 | same class as `` `default_nettype `` |

- ⛔ **The convenient reading was the wrong one, and it would have been a REGRESSION dressed
  as a burn-down.** A blanket "directives are the preprocessor's business, tolerate them
  everywhere" clears all 8 rows in one edit — and *widens* acceptance into two constructs the
  LRM explicitly forbids, against the standing doctrine that over-acceptance is a defect and
  tolerance is additive ([[feedback_sv_strict_lrm_compliance_default]]). Today's parser
  already refuses in-scope `` `default_nettype `` and is **RIGHT** to; the fix must not
  un-fix that.
- **Why the not-in-Annex-A objection does not carry here.** Every clause-22 syntax box is
  marked *"not in Annex A"*, and the repo's `svpp_owned_v2005` pins do route directive
  *grammar* and directive *value* errors to the preprocessor lane. But a PLACEMENT rule is
  not checkable by any stage that has already consumed the directive: deciding "was this
  inside a design element" requires design-element boundaries, i.e. a parse. PGEN's SV parser
  is the only stage that sees raw text *and* structure, so it is the only stage that can
  enforce §22.8/§22.3 at all — and it already does. The banked `.8c.2`
  (`no_timescale_in_module` → "the parser must tolerate it") and `.8b.2`
  (`22.14.01_04.sv` → "the directive-aware reading") pins are consistent with this: both
  concern directives their clauses place NO restriction on.
- ⇒ **THE STANDING RULE for every future directive row, in one line:** *tolerate a compiler
  directive wherever the LRM does not restrict its placement, and keep rejecting it exactly
  where a clause-22 restriction says it may not appear.* Not "directives are trivia", and not
  "directives are Annex A".
- **Consequence for the 3 `` `default_nettype `` rows** (verilator `t_lint_implicit_*_bad.v`):
  their expected verdict is re-adjudicated to **`must_reject`, pinned on §22.8 verbatim**, so
  the observed reject becomes a `match` — the parser is claimed CORRECT, with a cite, rather
  than the rows being quietly deferred. ⚠️ This overrides the `VerilatorIndex` heuristic
  (driver `fails=True` + no "syntax error" in the golden ⇒ `must_accept`), which is upstream
  *tool* testimony; the corpus doctrine ranks the SPEC above the tool, and the pin tables
  (`ISPRAS_NEGATIVE_PINNED`, `IVTEST_CE_STAGE_PINNED`, …) are the existing mechanism for
  exactly that. Verilator tolerating it is a real-world datum, not a conformance argument.
- **DESIGN-PRIOR-ART — the two surfaces that could already cover this, both probed:**
  (1) **the `trivia` rule** (`grammars/systemverilog.ebnf:533`,
  `trivia := (line_comment | block_comment)*`) — adding a directive arm would tolerate
  directives *everywhere* at zero grammar cost, and is **REFUSED**: the generated layout
  skipper consumes trivia before any branch is tried, so the existing
  `source_text_item` → `compiler_directive` alternative would go **ENGINE-SHADOWED-DEAD** and
  its `{kind:"compiler_directive"}` AST node would silently disappear from every currently
  passing file — the exact `comment_only_source_region` mechanism already documented in this
  grammar at `:260`-`:271`, and an AST break for downstream. (2) **`source_text_item:241`
  itself** — the tolerance exists but is reachable only at file top level, which is the
  defect. ⇒ no existing surface covers it; the carrier is a bounded set of in-scope item-list
  alternatives reusing the existing `compiler_directive` rule.
- **Honest bound to carry into `.3.14b`:** item-list alternatives tolerate a directive
  *between items*, not between two tokens of one statement. That is what the measured
  population needs (all 8 rows parse once directive LINES are removed at item positions —
  `strip_probe/probe.txt`), and the residue is a stated bound, not a silent cap.

##### `.3.14b` — the grammar change (in-scope directive tolerance, minus the placement-restricted names) — `done` (2026-08-09, session #217, `PGEN-SV-CORPUS-GRAD-0033`)

⭐⭐ **`.3.14a`'s RULING TABLE WAS ALSO INCOMPLETE, AND READING THE CLAUSE END TO END IS WHAT
CAUGHT IT.** `.3.14a` enumerated only the six directive names the routed population happened
to contain. IEEE 1800-2017 clause 22 and IEEE 1364-2005 clause 19, read in full, carry **two
further placement-restricted families that table never mentions**:

| directive | 1800-2017 | 1364-2005 | verbatim |
|---|---|---|---|
| `` `unconnected_drive `` / `` `nounconnected_drive `` | §22.9 | §19.9 | "These directives **shall be specified outside the design element declarations**." |
| `` `begin_keywords `` / `` `end_keywords `` | §22.14 | §19.11 | "can **only be specified outside a design element**" |

Deriving the whitelist from `.3.14a`'s table instead of from the clause would have tolerated
those four in-scope — an over-acceptance regression against
[[feedback_sv_strict_lrm_compliance_default]], inside the very leaf cut to prevent that
mistake. **Sampling a clause is not reading it.** (`` `celldefine ``/`` `endcelldefine `` read
the other way on the same evidence — §22.10/§19.1 "may appear anywhere in the source
description" — and stayed TOLERATED.) The two LRMs agree name for name; `` `undefineall ``
is the sole SV-only name (0 hits in 1364-2005 clause 19).

- **THE SHIPPED RULE, one line:** *tolerate a compiler directive wherever the LRM does not
  restrict its placement AND the directive neither hides nor rewrites the source text that
  follows it.*
  - **TOLERATED** — `` `celldefine `` `` `endcelldefine `` `` `undef `` `` `timescale ``
    `` `pragma `` `` `line `` (+ `` `undefineall ``, `sv_2017`/`sv_2023` only).
  - **KEPT REJECTING (placement)** — `` `resetall `` `` `default_nettype ``
    `` `unconnected_drive `` `` `nounconnected_drive `` `` `begin_keywords `` `` `end_keywords ``.
  - **KEPT REJECTING (text-hiding)** — `` `include `` `` `define `` `` `ifdef `` `` `ifndef ``
    `` `else `` `` `elsif `` `` `endif ``; the file is not honest parser input and is already
    routed to `explained_svpp_*`. `` `define `` also continues across lines with a trailing
    `\`, so a single-line carrier would MIS-CONSUME it.
  - **NOT DIRECTIVES** — `` `__FILE__ `` `` `__LINE__ `` (§22.13 MACROS, the `.3.13` ruling).
- ⛔ **Carrier is a NAME WHITELIST, not the blanket `compiler_directive` rule**, for two
  independent reasons: (1) placement is not uniform across clause 22 (above); (2) a backtick
  line is not necessarily a directive — `` `MY_MACRO(x) `` at an item position is an
  unexpanded §22.13-class MACRO use, and a blanket rule would swallow the line, drop the items
  the macro expands to, and turn a correct reject into a silent pass. New rules
  `in_scope_compiler_directive` + `in_scope_compiler_directive_sv_only`
  (`grammars/systemverilog.ebnf`), hosted as the LAST alternative of `non_port_module_item`
  (module bodies) and `class_item` (class bodies). `DESIGN-PRIOR-ART` discharged by `.3.14a`.
- ⛔⛔ **THE FIRST LANDING SILENTLY DID NOTHING, AND THE TOOLBOX — NOT INSPECTION — FOUND IT.**
  After a full regenerate + rebuild, all 8 rows still rejected. The new rule was present in
  `generated/systemverilog_parser.rs` (65 references) but had **NO CALLER**, and
  `cascade_match_non_port_module_item` carried **8 alternatives, not 9**. **Root cause: a `#`
  comment at COLUMN 0 inside an alternation list TERMINATES the rule, and every `|` arm below
  it is dropped from the generated parser with no diagnostic.** Discriminating pair, measured
  both ways: column-0 comment ⇒ arm DROPPED (the first attempt); comment INDENTED to the
  continuation column ⇒ arm LIVE (`net_declaration_sv_2017:3585`'s `checked_nettype_identifier`,
  `wildcard_escape_nettype_identifier`, `interconnect_net_declaration_sv_only` all verified
  present). **Repo-wide audit, 17 grammars: 8 comment-above-arm sites, all 8 INDENTED and
  verified live; COLUMN-0 occurrences 0** ⇒ nothing shipped is damaged. The hazard is
  unguarded and fails in the ACCEPTING direction (a dropped arm only narrows the language),
  which no pass-rate reveals → routed to **`.3.14c`**.
- **Verification:** artifacts + every number in
  `docs/tasks/artifacts/sv_corpus_grad/ch22_directive_fix/` (instrument sha256 triple,
  `matrix.sh`/`matrix_result.txt`, `ast_invariance_check.py`, `repro/`, `before/`, `after/`,
  `delta.txt`). Re-runnable oracles, all green at `HEAD`:
  `check_doctrines.sh` **17/17**; `parse_harness_equivalence_gate` **4/4** ⭐ (the interpreter
  stays byte-identical to all 11 certified generated parsers — the independent proof that the
  MODIFIED SV parser did not drift from its oracle, since a grammar edit is exactly what could
  break that mirror); `ast_shape_contract_gate` **18/18** (the schema-19 claim held by a gate,
  not argued); `parse_harness_combinator_gate` **31/31 CLEAN**;
  `parse_harness_semantic_gate` **36/36 CLEAN**; `sv_semantic_scope_contract_gate` PASS;
  `systemverilog_parser_book_gate` PASS; `mdbook_docs_gate` PASS (10/10 per-parser books).
  Corpus runs guarded (`--budget-mb 12288`, peaks 7 852 MB / 116 s and 127 MB / 10 s); battery
  guarded (peak 11 709 MB / 1 007 s).
- ⛔ **A THIRD finding, routed OUT — `clippy_on_rust_change` CANNOT FIRE on a grammar-only
  commit.** Running it here printed *"No Rust/generated Rust changes detected; skipping clippy
  flow"* **after** this leaf had regenerated a 131 MB `generated/systemverilog_parser.rs` full of
  new emitted code. Measured cause: the script triggers on a `generated/*.rs` path in
  `git diff` ∪ `git diff --cached` ∪ `git ls-files --others --exclude-standard`, but `generated/`
  is gitignored (`.gitignore:24`) so `--exclude-standard` filters it out — the union's
  `generated/*.rs` count is **0**, by construction, forever. The skipped stage is the one holding
  the generated-parser correctness floor at 0, and the commit class it cannot see is exactly the
  class that emits new generated code. Family-agnostic, so it is NOT an SV defect → routed to
  **`GENERATED-LINT-CORRECTNESS.11`** (parked: governance, does not block the SV release lane).
  This leaf's own posture is the honest one until it is fixed: the flow was invoked with
  `--force` and is green.
- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — re-measured at `HEAD` before any edit: all 8 routed rows
    REJECT (`before_8rows.txt`); `` `timescale `` in a module body rejects at
    `furthest_position=9` while the same text above the module PASSES.
  - [x] **ROOT CAUSE (WHY + WHERE)** — `compiler_directive`
    (`grammars/systemverilog.ebnf`) was an alternative of `source_text_item` and of NOTHING
    ELSE, so no in-scope item list could reach it (tool-pinned by `.3.13`, re-confirmed here).
    Plus the second, self-inflicted root cause above (column-0 comment ⇒ silently dropped
    arm), pinned by generated-code inspection after the toolbox showed the rule had no caller.
  - [x] **FIX** — the whitelist rules + their two host alternatives; `VERILATOR_PINNED` in
    `stimuli/sv/adjudicate_external_corpus.py` pinning the 3 §22.8 rows `must_reject` on the
    LRM cite (spec outranks the verilator driver heuristic, which is tool testimony).
  - [x] **ADDRESSED (verified)** — the designed split lands EXACTLY: the 5 TOLERATE rows flip
    REJECT→PASS, the 3 §22.8 rows stay REJECT. Control matrix **33 rows / 0 misses**, covering
    every tolerated name in module AND class bodies, all six placement-restricted names, all
    seven text-hiding names, user/UVM/`__FILE__`/`__LINE__` macro uses, unchanged top-level
    behaviour, the `\b` name-prefix guard, and profile gating (`` `undefineall `` accepted
    under `sv_2017`/`sv_2023`, REJECTED under `verilog_2005`). ⚠️ The FIRST version of that
    harness was VACUOUS — `printf '%s'` wrote a literal `\n`, collapsing every case to one
    line so the whole `expect=REJECT` half "passed" measuring nothing; the shipped harness
    pins a positive AND a negative control and REFUSES to report on a miss
    ([[feedback_instrument_needs_ground_truth]]). Corpus `sv_2017` pass **9 712 → 9 720 (+8)**;
    `verilog_2005` pass **2 180 → 2 181 (+1)**.
  - [x] **NO REGRESSION** — per-FILE census, BOTH lanes: the ONLY transition anywhere is
    `fail → pass` (**0 pass→fail, 0 pass→timeout, 0 pass→crash** over 16 336 + 2 459 rows).
    ⛔ **The over-acceptance control is `accepts-invalid`, and it is UNCHANGED in both lanes:
    21 → 21 (`sv_2017`) and 14 → 14 (`verilog_2005`)** — that is the number that would have
    moved had the whitelist been wrong. **AST invariance measured EXHAUSTIVELY without the
    pre-change binary**: the delta adds one production emitting one node shape, so an AST can
    only differ by containing it; over **all 287** previously-passing files that contain a
    whitelisted directive token at line start (the only files whose AST could move), 287/287
    dumped, 595 top-level directive nodes on the pre-existing path, **0 in-scope (new-arm)
    nodes**, controls pinned. Adjudicator determinism `cmp`-proven byte-identical.
  - [x] **ATTRIBUTION (⛔ the two halves are NOT both yield)** — measured separately by running
    the adjudicator alone against the UNCHANGED baseline `results.tsv`: the `VERILATOR_PINNED`
    pins move `sv_2017` unexplained **360 → 357 (−3)** with the parse verdict `fail → fail`
    — an **ADJUDICATION CORRECTION**, never burn-down; the grammar change then moves
    **357 → 352 (−5)** — **REAL YIELD**. The pin-only run touched exactly those 3 rows and
    left the `verilog_2005` arm byte-identical. `verilog_2005` unexplained **75 → 74**.
    ⭐ Three further `fail → pass` heals were NOT predicted by the routing (Surelog
    `PragmaProtect/pp.top.sv`, `PragmaProtect/svpp_all/top.sv`, iverilog
    `no_timescale_in_module.v`); all three sit in DEFERRED lanes, so they are pass-rate, not
    bar movement. `no_timescale_in_module.v` is the file `.8c.2` pinned `must_accept` on the
    reading *"the parser must tolerate it"* — that banked pin is now **vindicated by the
    parser**, and it is the +1 in the v2005 lane.
  - [x] **LOCKSTEP** — release `1.0.177 → 1.0.178` (schema **19 UNCHANGED** — the in-scope node
    is byte-identical to the top-level one, `{"body": …, "kind": "compiler_directive"}`, so no
    emitted kind is added, renamed or removed) + contract Current-state note; ledger row
    `SV-0048`; SV book changelog; `CHANGES.md`; `DEVELOPMENT_NOTES.md`; `MEMORY.md`;
    `docs/TASK_TREE.md`; `.3.14c` (the column-0-comment arm-drop hazard) and `.3.14d` (the
    remaining item-list hosts) opened for the routed findings.
- **⚠️ HONEST BOUNDS (stated, not silently capped)**
  1. **Item positions only** — an item-list alternative tolerates a directive *between items*,
     never between two tokens of one statement. That is what the measured population needs
     (`ch22_directive_split/strip_probe/probe.txt`); a directive mid-statement still rejects.
  2. **Two hosts, not all** — module bodies and class bodies. Generate / interface / program /
     package / checker bodies are the stated residue → **`.3.14d`**.
  3. **The 2 §34 protected-envelope rows still reject** and correctly so — they stop at their
     base64 payload, not at a directive (`.3.13` reclassified them already).
  4. **`` `pragma `` is tolerated as a LINE**, not parsed as a structured §22.11 pragma; its
     effect is not modelled, only its presence stops being a parse error.

##### `.3.14c` — ⛔ a COLUMN-0 `#` comment inside an alternation list silently DROPS every arm below it (routed by `.3.14b`, 2026-08-09)

- **Status: `todo`** — no lint, no gate, no diagnostic. Measured discriminating pair in
  `.3.14b`: a comment at column 0 between two `|` arms terminates the rule and the arms below
  vanish from the generated parser; the same comment INDENTED to the continuation column is
  absorbed and the arms survive. **Repo-wide audit at the time of routing: 17 grammars, 8
  comment-above-arm sites, all 8 indented and verified LIVE, 0 column-0 occurrences** — so this
  is a latent hazard, NOT current damage.
- **Why it must be fixed rather than remembered:** it fails in the ACCEPTING direction — a
  dropped arm only narrows the accepted language, so no pass-rate, corpus delta or AST-shape
  gate can reveal it; it cost this leaf one full regenerate + release-rebuild cycle before the
  toolbox pinned it. Owed: either a frontend diagnostic (a `|` continuation after a rule the
  parser considers ended) or a `check_doctrines.sh` lint on the textual pattern, plus a
  decision on whether column-0 termination is the INTENDED frontend semantics at all.

##### `.3.14d` — the remaining in-scope item-list hosts (routed by `.3.14b`, 2026-08-09)

- **Status: `todo`** — `.3.14b` hosts `in_scope_compiler_directive` at `non_port_module_item`
  and `class_item`, the two hosts the measured population needs. Generate, interface, program,
  package and checker bodies still reject a legally-placed directive. Zero corpus rows demand
  it today (measured: the routed 8 are fully covered by the two hosts), so this is
  under-acceptance with no current witness — worked as **crafted minimal cases** in the `.9`
  gap loop, where the construct, not a vendored file, is the unit.

#### `.3.15` — the bare `generate_block` family is an ADJUDICATOR hole, not a parser defect: the repo had ALREADY pinned this exact construct `must_reject` from the same suite, and the sibling fixture carried the opposite verdict because no in-file comment happened to say so

- **Status: `done`** (2026-08-09, session #218, `PGEN-SV-CORPUS-GRAD-0034`) —
  **adjudicator + docs, ZERO parser bytes.** Cut from the HEAD re-cut of the worklist
  (below), where `named block/label (ch9/27)` was the largest COHERENT construct family
  (20 rows; `OTHER` is larger but is by definition per-row triage).
- ⭐⭐ **THE TRANSFERABLE LESSON, and it is `.3.14b`'s one level deeper.** `.3.14b` taught
  *the corpus says which rows EXIST, only the clause says which are LEGAL*. This leaf adds:
  **the repo's OWN prior rulings are part of the clause record, and an inconsistency among
  them is a defect in its own right.** `EXTRA_PINNED` already held
  `verible/…/testdata/module_begin_block.sv` as `must_reject`, because that fixture's in-file
  comment calls a bare begin block *"LRM-invalid syntax"*. Its sibling
  `generate_begin_module.sv` — the SAME construct, the SAME suite, one nesting level in —
  was `must_accept` and counted against the graduation bar as a parser defect. The two
  verdicts differ by **whether an upstream author wrote a comment**, and nothing else.
  ⇒ before adjudicating a family, GREP THE PIN TABLES FOR THE CONSTRUCT: a contradiction
  already in the table is stronger evidence than any fresh reading of the clause.
- **REPRODUCE (before any edit).** `Surelog/tests/GenerateBlock/dut.sv` is 19 lines whose
  whole point is `generate begin : A … end endgenerate`; the parser rejects at
  `furthest_position=86`, exactly at the `:` of the label, and the manifest row read
  `must_accept / divergence:unexplained_rejects_valid` on the basis *"Surelog golden log …
  completes with no [SNT:]/[FTL:]"* — upstream TOOL testimony.
- **ROOT CAUSE (WHY + WHERE) — and the WHY is that the parser is RIGHT.** IEEE 1800-2017
  A.4.2: `generate_region ::= generate { generate_item } endgenerate` and
  `generate_item ::= module_or_generate_item | interface_or_generate_item |
  checker_or_generate_item`. **`generate_block` is not a `generate_item`** — it is reachable
  only from `loop_generate_construct`, `if_generate_construct` and `case_generate_item`
  (`docs/systemverilog/2017/md/section-27-generate-constructs.md:68-127`, mirrored verbatim
  at `grammars/systemverilog.ebnf:2517-2521`; footnote 30 restricts only the interface case).
  So a `begin … end` directly in a generate region or a module body has **no production at
  all**. WHERE the defect actually is: `stimuli/sv/adjudicate_external_corpus.py`, whose
  per-suite `expect()` heuristics are tool testimony with no pin for this construct.
  ⭐ The discriminating pair proves it is placement and not the block:
  `repro/C_labelled_begin_in_for.sv` and `repro/D_labelled_begin_in_if.sv` **PASS**, while
  `repro/A_*`/`repro/B_*` reject — the parser handles a labelled `generate_block` perfectly
  wherever the LRM actually puts one.
- **FIX** — declarative, spec-derived, two pin tables and no parser surface: 19 rows added to
  the existing cross-suite `EXTRA_PINNED` (prior art — it is already keyed `(suite, relpath)`
  and already consulted FIRST, so no new surface was designed:
  [[feedback_read_prior_art_before_designing]]), and a new `V2005_LRM_PINNED` consulted first
  inside `expect_v2005()` for the 6 rows in the other lane, cited to IEEE 1364-2005 A.4.2
  (`grammars/verilog_2005_lrm_extracted.ebnf:452-462, :749` — `module_or_generate_item` has
  no bare-block alternative there either, so the verdict does not change with the edition).
- **ADDRESSED (verified).** Ground-truth control FIRST: the unchanged adjudicator reproduced
  the tracked manifest **byte-identically** (`cmp`), so every moved row is attributable to
  the edit. `sv_2017` unexplained rejects-valid **331 → 312 (−19)**; `verilog_2005`
  **60 → 54 (−6)**; `match` +19 / +6. Row-level diff = exactly the 25 pinned files, each
  `must_accept / unexplained_rejects_valid` → `must_reject / match`; every other class
  byte-identical in both lanes; both manifests reproduce byte-identically across two runs.
  The re-cut family map then shows `named block/label` at **20 → 1** — the family is burned
  down, and the 1 is the row that never belonged to it.
- **NO REGRESSION.** ⛔ **`unexplained_accepts_invalid` is the control that matters and it is
  UNCHANGED in both lanes (21 → 21, 14 → 14)** — the number that would have moved had a pin
  been wrong. Zero Rust, EBNF, generator or generated bytes changed (`git diff --stat`), so
  the cert/AST/corpus batteries are inert **by construction** rather than by assertion, and
  no parse verdict moved anywhere: the construct still rejects, which is the point.
- ⛔⛔ **THE HALF-FIX THIS LEAF NEARLY SHIPPED, and what caught it.** The leaf was cut from
  the sv_2017 worklist and `MEMORY.md`'s `next_action` names that lane. The sweep was written
  **parameterized by lane** (`--manifest`/`--profile`) rather than hard-coded — a deliberate
  refusal to fork the diagnostic
  (`docs/knowledge/a-copied-diagnostic-covers-only-where-it-was-pasted.md`) — and running it
  against the `verilog_2005` manifest surfaced **6 more rows, all iverilog, none of them one
  of the 19**. Stopping at the lane the task named would have shipped a fix that looked
  complete. ⇒ **the lane a task names is not the lane a construct lives in.**
- ⛔ **TWO INSTRUMENT DEFECTS, found and stated rather than quietly fixed.**
  1. The first spelling of "strip the trailing whitespace/comment run" was
     `re.sub(rb"(?:\s+|//[^\n]*|/\*.*?\*/)+\Z", …)` — two quantifiers nested under an
     anchored `+`, which **backtracks catastrophically**. Measured spinning at **100 % CPU
     for over three minutes with an EMPTY probe process table**: the instrument looked
     *busy*, not *broken*, and the natural (wrong) report would have been "the corpus is
     slow". The tell is CPU-time ≈ wall-time with nothing forked. Replaced by a linear
     backwards scan, unit-checked on six cases including the known limit it does not
     handle (`//` inside a string); the sweep now runs in **5 s**, matching
     `cluster_rejects_valid.py` on the same file set.
  2. `--manifest` resolved against the caller's cwd rather than the repository root
     (directive 12). Fixed, so one command line means one thing from any directory.
- **ROUTED OUT (never dropped — routing decides WHEN, not WHETHER,
  [[feedback_every_finding_must_be_fixed_not_logged]]):** `.3.16` (the `inside` set in a
  generate condition) and `.3.17` (a non-UTF-8 source file the probe cannot position at all).
- ⛔ **A THIRD finding, routed OUT — `TASK-ACCEPTANCE` could not SEE this commit.** With the
  whole change staged, `bash scripts/check_diagnosis_evidence.sh` printed *"no code change
  staged; task-acceptance checklist not required"* and exited 0, so its PASS in this commit's
  17/17 is **vacuous**. Measured cause: `scripts/check_diagnosis_evidence.sh:69-78` sets
  `code_changed=1` only for `grammars/*.ebnf`, `rust/src/*`, `generated/*`, the ast-shape
  contracts, `scripts/check_*.sh`, `rust/scripts/*.sh`, `.githooks/*`, the workflows,
  `rust/build.rs` and the two Makefiles — `stimuli/**` matches nothing, by a narrowing that is
  deliberate and documented at `:64-68`. ⛔ But the adjudicator assigns the EXPECTED VERDICT
  for every corpus row, so it *defines what counts as a defect* and the sum of its two
  `unexplained` classes IS the `.5` bar — the enforcer's own comment at `:62` says *"a change
  to a gate is a change to what 'verified' MEANS."* The blind spot is the recent norm, not a
  hypothetical: `.3.13`, `.3.14a` and this leaf all moved the bar with zero parser bytes.
  Family-agnostic (a path list with no family term), so it is not an SV defect → routed to
  **`DOCTRINE-GAP-OWNERSHIP.6`** (parked: governance, does not block the SV release lane).
  This leaf's own posture is the honest one until it is fixed: the checklist above is complete
  and evidence-backed **by author discipline, not because a gate required it.**
- **Verification:** the evidence bundle is
  `docs/tasks/artifacts/sv_corpus_grad/gen_block_family/` (README, both instruments with
  their controls, `repro/`, `sweep_result.txt`, `sweep_result_v2005.txt`). Re-runnable
  oracles green at `HEAD`: `bash scripts/check_doctrines.sh`.
- **LOCKSTEP** — tracked manifests + summaries (both lanes), the re-cut live worklist
  (`rejects_valid_clusters.*`, and `rejects_valid_families.*` created as the live surface the
  base name was missing), the book's adjudication-manifest section, `CHANGES.md`,
  `MEMORY.md`, `docs/TASK_TREE.md`, and `.3.16`/`.3.17` opened.
- ⚠️ **HONEST BOUNDS (stated, not silently capped)**
  1. **A `must_reject` row proves the FILE is invalid, not that the rest of it parses.** All
     25 stop *at* the illegal construct, so a further defect later in the same file is
     invisible to that row by construction — the same shape as `.3.13`'s four
     stuck-elsewhere rows, and given the same disposition: crafted minimal cases in `.9`.
  2. **Tolerance stays FUTURE and ADDITIVE.** Real tools accept the legacy generate region —
     a real-world datum, not a conformance argument
     ([[feedback_sv_strict_lrm_compliance_default]]). Making it parse today would convert a
     correct reject into an over-acceptance defect.
  3. **This is an ADJUDICATION CORRECTION, not burn-down yield.** `.5` counts unexplained
     divergences, so it must never be reported as parser progress. Parser yield this leaf: **0**.

##### `.3.16` — `inside` where a CONSTANT expression is required: legal in an `expression`, illegal in a `constant_expression` (routed by `.3.15`, closed same day)

- **Status: `done`** (2026-08-09, session #218, `PGEN-SV-CORPUS-GRAD-0035`) —
  **adjudicator + docs, ZERO parser bytes**, parser yield **0**. A second adjudication
  correction, and the routed description was too NARROW: it named "a generate condition",
  and the sweep found the construct also in a **localparam initializer**.
- **REPRODUCE.** `Surelog/tests/InsideOp/dut.sv` stops at `furthest_position=286`, on the
  ` inside {Get0, GetDefault}` of `if (GetWhat inside {Get0, GetDefault}) begin : gen_zero`
  — the `begin : gen_zero` after it is never reached, which is why it was `.3.15`'s NEGATIVE
  control. Manifest row: `must_accept` on the Surelog golden log.
- **ROOT CAUSE (WHY + WHERE) — the parser is RIGHT, and the clause was READ, not skimmed**
  (the discipline `.3.14a` learned the hard way). IEEE 1800-2017 **A.8.3** verbatim:
  `constant_expression ::= constant_primary | unary_operator { attribute_instance }
  constant_primary | constant_expression binary_operator { attribute_instance }
  constant_expression | constant_expression ? { attribute_instance } constant_expression :
  constant_expression` — **no `inside` alternative**. And `inside` is not a fallback via
  `binary_operator` either: **A.8.6** lists them exhaustively
  (`+ - * / % == != === !== ==? !=? && || ** < <= > >= & | ^ ^~ ~^ >> << >>> <<< -> <->`)
  and `inside` is absent. The operator has its own production, an alternative of
  `expression` ONLY — `inside_expression ::= expression inside { open_range_list }` (A.8.3,
  re-quoted as Syntax 11-3 in §11.4.13, whose prose likewise speaks only of expressions).
  `grammars/systemverilog.ebnf:430` mirrors A.8.3 exactly. WHERE: the adjudicator's per-suite
  tool-testimony heuristics, same as `.3.15`.
- ⭐ **The discriminating pair, tracked** — same operator, same file shape, different required
  nonterminal: `repro/F_inside_in_expression.sv` (`initial b = a inside {1, 2};`) **PASSES**,
  `repro/E_inside_in_constant_expression.sv` (`localparam int B = A inside {1, 2};`)
  **rejects AT the keyword** (byte 56).
- **FIX** — 2 rows added to `EXTRA_PINNED` with per-shape cites (A.4.2
  `if ( constant_expression )` for the generate condition; A.2.3 `param_assignment` →
  `constant_param_expression` → `constant_mintypmax_expression` → `constant_expression` for
  the localparam initializer).
- **ADDRESSED (verified).** `sv_2017` unexplained rejects-valid **312 → 310 (−2)**, `match`
  +2, exactly the 2 intended rows, every other class byte-identical; the `verilog_2005`
  manifest is **byte-identical** (`cmp`) — the construct does not occur in that lane, and
  that is MEASURED, not assumed (see below). Manifest deterministic across two runs. The
  re-cut family map drops from 9 families to **8**: `named block/label` is gone entirely.
- **NO REGRESSION.** `unexplained_accepts_invalid` **21 → 21**. Zero Rust/EBNF/generated
  bytes; `results.tsv` untouched, so no parse verdict moved anywhere.
- ⭐⭐ **THE INSTRUMENT WAS GENERALIZED, NOT COPIED — and generalizing it re-proved `.3.15`.**
  The cheap way to sweep a second construct is to copy `sweep_begin_family.py` and edit one
  string, which is the defect
  `docs/knowledge/a-copied-diagnostic-covers-only-where-it-was-pasted.md` names. Instead the
  probe, trivia scanner, manifest walk and controls stayed shared and only the **predicate**
  became selectable (`--family begin|inside`). Two dividends fell out immediately:
  1. `--family begin` now measures **0 / 310** (and **0 / 54** in v2005) — an INDEPENDENT
     re-proof that `.3.15` closed that family, from an instrument that was not written to
     confirm it.
  2. `--family inside` found a **second row the coarse bucketer never surfaced**
     (`verilator/t_inside_unpacked_param.v`, a localparam initializer, nowhere near a
     generate block) — so the routed one-row description was wrong about the construct's
     extent, and only a whole-population sweep could have said so.
- ⛔ **THE CONTROL WAS PINNED TO A CORPUS ROW, AND THE INSTRUMENT REFUSED — correctly.** The
  first cut used `Surelog/tests/InsideOp/dut.sv` as the `inside` control. Under
  `verilog_2005` that file is SV-only source and stops at `package`, not at `inside`, so the
  v2005 sweep printed *"REFUSE: positive control not classified"* and produced no numbers.
  ⭐ The fix is **not** to relax the assertion but to CONSTRUCT the state being observed
  ([[feedback_ground_truth_control_must_not_pin_untracked_state]]): `repro/G_inside_v2005_pure.sv`
  is pure 1364 text plus the one keyword, rejects AT `inside` under `verilog_2005` (byte 43)
  and **PASSES** under `sv_2017` — which is itself the edition evidence that `inside` is
  SV-only. With it, the v2005 lane answers **0 / 54 measured**, not "never asked". A control
  table keyed `(family, profile)` now REFUSES outright when no constructed control exists for
  a combination, rather than sweeping uncontrolled.
- **Verification:** `docs/tasks/artifacts/sv_corpus_grad/gen_block_family/` — `repro/E`,
  `repro/F`, `repro/G`, and the four banked sweeps (`sweep_result.txt`,
  `sweep_result_v2005.txt`, `sweep_result_inside.txt`, `sweep_result_inside_v2005.txt`).
  `bash scripts/check_doctrines.sh` 17/17.
- ⚠️ **HONEST BOUND.** Same as `.3.15`: a `must_reject` row proves the FILE is invalid, not
  that the rest of it parses — both rows stop at the illegal construct, so anything later in
  those files is invisible to them and belongs to `.9` as crafted cases. And as with `.3.15`,
  this is an **ADJUDICATION CORRECTION**, never burn-down yield.

##### `.3.17` — a non-UTF-8 corpus file cannot be positioned at all (routed by `.3.15`, 2026-08-09)

- **Status: `todo`** — `sv2v/test/lex/latin1.sv` is ISO-8859-1 (`file(1)`: *ISO-8859 text*),
  and `parseability_probe` fails it with *"stream did not contain valid UTF-8"* — **no
  `furthest_position` is produced at all**, so the row is invisible to every stuck-point
  instrument in the tree (the `.3.15` sweep reports it as the single `non-integer probe
  result` over 331 rows). It is a statement about the **input decoder**, not about a
  construct.
- **Owed:** the IEEE 1800-2017 §5.1 ruling on the legal source character set, then either a
  decoder that admits it or an adjudication class that names the encoding as the cause —
  never silent membership in `unexplained_rejects_valid`, where it prices as a grammar
  defect. ⚠️ Cross-family by construction: any grammar's file reader has this property, so
  check `vhdl`/`regex` before assuming it is SV's.

##### `.3.18` — `expression_or_dist` renders the LRM's LITERAL `{ dist_list }` braces as EBNF repetition, so the whole `dist` constraint operator is wrong THREE ways at once (IEEE 1800-2017 A.2.10 / §18.5.4 — ⭐ the SEVENTH instance of the documented dropped-delimiter class, and the THIRD in Annex A subclause A.2.10 alone)

- **Status: `done`** (2026-08-09, session #219, `PGEN-SV-CORPUS-GRAD-0036`; release
  `1.0.178` → **`1.0.179`**, schema **`19` → `20`**, ledger **`SV-0049`**) — a REAL parser
  defect with parser bytes and **parser yield 6**, cut after two consecutive zero-yield
  adjudication leaves (`.3.15`, `.3.16`).
- **PICK (measured, from the STUCK POSITION over the LIVE worklist — not from the family
  bucketer, per `MEMORY.md`'s standing warning that `OTHER` at 226/310 is burned past
  usefulness).** Cluster **#14 `: = NUM` (4 rows)** in the HEAD
  `rejects_valid_clusters.md` is a *token-level tell*: the 3-token stuck window shows `:`
  and `=` as SEPARATE tokens, i.e. the parse died on the `:=` of a `dist` weight. Widening
  from the cluster to the construct with the tracked positional scan
  (`keyed_dist_rows.py`, `keyed_rows_before.txt`) gives the real extent: **6 / 310 rows are
  blocked INSIDE a `dist { … }`**, across 3 suites (verilator 3, sv-tests 2,
  ispras-sv-tests 1). ⛔ The coarse bucketer files these under
  `constraint/randomize (ch18)` mixed with `randomize()…with{}` method chains — a
  heterogeneous 22-row bucket `.3.8` explicitly declined for that reason; the positional
  scan is what separates the two mechanisms.
- **REPRODUCE (tool-pinned, `dist_list_braces/before.txt`)** — 8 cases under
  `--profile sv_2017`, every `d*` copied VERBATIM from the LRM's own normative text.
  ⭐ **The defect is THREE-sided, which a pass/fail probe alone cannot show:**

  | case | construct | LRM source | verdict | furthest |
  |---|---|---|---|---|
  | `d1_dist_weighted` | `x dist {100 := 1, 200 := 2, 300 := 5}` | §18.5.4 :503 | **REJECT** | 52 |
  | `d2_dist_range_eq` | `x dist { [100:102] := 1, … }` | §18.5.4 :518 | **REJECT** | 49 |
  | `d3_dist_range_prop` | `x dist { [100:102] :/ 1, … }` | §18.5.4 :520 | **REJECT** | 49 |
  | `d4_dist_unweighted_soft` | `soft x dist {5, 8};` | §18.5.11 :1406 | ⚠️ **ACCEPT — MIS-PARSED** | — |
  | `d5_dist_in_property` | `(a dist {1 := 1, 0 := 3}) \|-> b` | A.2.10 | **REJECT** | 72 |
  | `c1_constraint_no_dist` | `x > 100; x < 300;` | control | ACCEPT | — |
  | `c2_concat_expression` | `q = {4'd5, 4'd8};` | control | ACCEPT | — |
  | `n1_dist_no_braces_ILLEGAL` | `x dist 100 := 1;` | A.2.10 — **no such production** | ⚠️ **ACCEPT** | — |

  1. **Under-acceptance** — `d1`/`d2`/`d3`/`d5`: every weighted or ranged distribution the
     LRM writes is unparseable, in constraint AND assertion contexts.
  2. ⭐ **Silent MIS-PARSE** — `d4` *parses*, so no pass-rate anywhere reports it, but the
     emitted AST gives the dist item's value `{"kind": "concat"}`: the LRM's **two** items
     `5` and `8` are handed to the consumer as **one** item whose value is the
     concatenation expression `{5, 8}`. A wrong tree is worse than a rejection — the
     downstream (Nexsim) has no way to notice. `matrix.py --check-d4-shape` names the kind.
  3. **Over-acceptance** — `n1`: the brace-less `x dist 100 := 1;` has no production in
     A.2.10 at all and is accepted, which is a defect under
     [[feedback_sv_strict_lrm_compliance_default]].
- **LRM GROUND TRUTH (verified verbatim in the in-repo LRM text BEFORE any edit).** A.2.10,
  quoted identically at three places
  (`docs/systemverilog/2017/txt/section-18-constrained-random-value-generation.txt:372`,
  `:487`, and `section-15-…:1297`):

  ```
  expression_or_dist ::= expression [ dist { dist_list } ]
  dist_list ::= dist_item { , dist_item }
  dist_item ::= value_range [ dist_weight ]
  dist_weight ::= := expression | :/ expression
  ```

  ⭐ **The two `{ }` uses sit two lines apart and mean opposite things** — in `dist_list`
  they ARE repetition metasyntax; in `expression_or_dist` they are LITERAL SystemVerilog
  braces. The LRM settles it in its own normative prose and examples, so this is
  adjudicable rather than a judgement call: `:503` `x dist {100 := 1, 200 := 2, 300 := 5}`,
  `:518` `x dist { [100:102] := 1, … }`, `:520` `x dist { [100:102] :/ 1, … }`, `:1393`
  `constraint B2 { disable soft x; soft x dist {5, 8};}`, `:1406`
  `constraint B3 { soft x dist {5, 8}; }` — and the prose at `:497` calls the operand
  *"a comma-separated list of integral expressions and ranges"*, which is the braced list,
  not a repetition of lists.
- **ROOT CAUSE (WHY + WHERE) — grammar source, `grammars/systemverilog.ebnf`:**
  - **WHERE:** `expression_or_dist:2344`.

    ```
    expression_or_dist := @probe_sample: "1" expression ( kw_dist_02450072 dist_list* )?
                       -> {expr: $1, dist: $2}
    ```

  - **WHY:** the LRM's literal `{ dist_list }` was rendered as PGEN's `dist_list*`
    **zero-or-more metasyntax**, so the rule matches `dist` followed by a *brace-less*
    repetition of dist lists. Nothing in the rule can consume a literal `{`. What the
    corpus then sees is not a clean rejection but the three verdicts above, because
    `dist_item → value_range → expression` can itself start with `{` as a
    **concatenation**: for `{5, 8}` that speculative path SUCCEEDS (one item, wrong tree —
    `d4`), for `{3 := 1}` it dies at the `:=` (`d1`, and this is precisely the `: = NUM`
    cluster signature), and for `{[100:102] …}` it dies at the `[` (`d2`/`d3`). One dropped
    delimiter, three symptoms — which is why the cluster map scattered them.
  - **TRACE (the WHY, `dist_list_braces/trace_d1_before.txt`)** —
    `PGEN_TRACE_VERBOSITY=debug … --trace-rules expression_or_dist,dist_list,dist_item,dist_item_sv_2017,dist_weight`
    on the 4-line minimal repro (`repro/min_dist.sv`). The whole dist-family trace is seven
    lines and they all name the same byte:

    ```
    🚪 Entering branch 1/2 for rule 'dist_item' at position 46
    ❌ Branch 1/2 for rule 'dist_item' failed at position 46
    🚪 Entering branch 2/2 for rule 'dist_item' at position 46
    ❌ Branch 2/2 for rule 'dist_item' failed at position 46
    ❌ Exiting rule 'dist_item_sv_2017' with error: Backtrack { position: 46 }
    ❌ Exiting rule 'dist_item'        with error: Backtrack { position: 46 }
    ❌ Exiting rule 'dist_list'        with error: Backtrack { position: 46 }
    ```

    Position 46 **is the `{`** — the parser attempts a `dist_item` ON the brace and both of
    its branches fail there, so no production consumes it. `dist_weight` is never entered at
    all. `furthest_position=49` is the `:=` three bytes later, reached only by the
    speculative concatenation path described above.
- ⭐⭐ **THE CLASS FINDING, and it is the actionable part.** This is the **seventh** logged
  instance of the ledger's "dropped-delimiter class" (literal LRM delimiters read as BNF
  metasyntax) — after `stream_concatenation` (`SV-0002`), `trans_range_list`,
  `boolean_abbrev`, the six bounded-property operators, `value_range`, and
  `cycle_delay_range` (`.3.8`, `SV-0044`). ⛔ **Three of those seven — `boolean_abbrev`,
  `cycle_delay_range` and now `expression_or_dist` — live in Annex A subclause A.2.10.**
  `.3.8` already recorded the identical complaint ("a fix landed in that very subclause and
  did not sweep its immediate neighbours") and its own PICK notes even name
  `dist { [0:1], [2:5] :/ 2 }` as a residual it was setting aside. It was set aside for two
  weeks and then found again, reactively, by a different cluster. ⇒ this leaf is direct
  evidence that the reactive one-construct-at-a-time posture does not converge, and that
  the exhaustive Annex-A bracket/brace sweep owned by the director-gated
  `LRM-GRAMMAR-FIDELITY` tree is the only thing that closes the class. Routed, not worked
  ([[feedback_flow_findings_are_routed_not_worked]]) — see the routing note at landing.
- **FIX (hierarchy level 1 — pure grammar, existing tokens only):** restore the LRM's
  literal braces at `expression_or_dist:2344` —
  `( kw_dist_02450072 lbrace dist_list rbrace )?`. `lbrace`/`rbrace` already exist
  (`:6884`/`:6920`) and `dist_list` is unchanged, so **zero new rules and zero new tokens**
  ⇒ `defined_rule_count` expected UNCHANGED. The `*` also retires: `dist_list` already
  carries its own `( comma dist_item )*`, so the repetition was doubly wrong.
- **ADDRESSED (verified) — the repro matrix flips on all three axes at once
  (`dist_list_braces/after.txt`):**

  | case | before | after |
  |---|---|---|
  | `d1` / `d2` / `d3` / `d5` | REJECT (52 / 49 / 49 / 72) | **ACCEPT** |
  | `d4` (AST kind of the dist item's value) | ACCEPT, `"kind": "concat"` ⚠️ | ACCEPT, **`"kind": "number"`** |
  | `n1` (LRM-illegal, brace-less) | ACCEPT ⚠️ | **REJECT** (furthest 47) |
  | `c1` / `c2` (controls) | ACCEPT | ACCEPT |

- ⭐ **SCHEMA `19` → `20` — MEASURED, not assumed, and it is the one thing about this leaf
  that reaches consumers.** `.3.8`'s "the construct was 100 % unparseable ⇒ no witnessed
  wire shape can move ⇒ schema unchanged" reasoning does **not** transfer here, because the
  `d4` shape *did* parse. Dumping the same file on both parsers:

  ```
  BEFORE  dist: [ [trivia,"dist"], [ [ <one dist_item: {5,8} as a concatenation> ] ] ]   # 2 elements
  AFTER   dist: [ [trivia,"dist"], {kind:"lbrace"}, [ <item 5>, <item 8> ], {kind:"rbrace"} ]  # 4 elements
  ```

  Two consumer-visible changes: the `dist` array grows from **2 to 4** elements (the literal
  braces are now nodes), and the item list flattens from one mis-parsed item to the LRM's
  **two**. Per `docs/systemverilog_parser_book/src/schema-versioning.md` ("an existing return
  annotation is restructured" / "a grammar rule changes shape in a way that's user-visible"),
  that is a bump. ⛔ The `ast_shape_contract_gate` could NOT have caught this — **0 of its 31
  locked samples contain a `dist`** (checked before the edit), so the bump is an author
  ruling backed by a dump, and a `dist` sample is added to the contract by this leaf so the
  next change to this rule is gate-visible.
- **ADDRESSED — measured GLOBALLY, both lanes** (evidence
  `dist_list_braces/{transitions_sv2017.txt,transitions_v2005.txt}`; baselines preserved to
  `rust/target/sv_axis2_baseline/*.pre_3_18.*` **before** the run, per the tree's standing
  overwrite trap):
  - **MAIN `sv_2017` lane, 16 336 files — pass 9 720 → 9 726 (+6)**, fail 6 612 → 6 606,
    timeout 4 (unchanged), crash 0.
  - **Adjudication: `unexplained_rejects_valid` 310 → 304 (−6)**, `match` 5 766 → 5 772.
  - **V2005 lane BYTE-INERT:** 2 459 files, pass 2 181 / fail 278 / timeout 0 — **zero
    per-file transitions**, and `adjudication_manifest_v2005.tsv` is **BYTE-IDENTICAL**
    (`cmp` clean). Constraints and SVA are unreachable under IEEE 1364-2005, and this is
    now MEASURED rather than assumed.
- **NO REGRESSION — the `.3.4` LAW, per-FILE, plus set differences rather than net counts:**
  - **Transition matrix: `pass→pass` 9 720, `fail→pass` 6, `timeout→timeout` 4.
    ZERO `pass→fail`, ZERO `pass→timeout`, ZERO `pass→crash`** — and no jitter row to
    explain away this time.
  - **ZERO NEW `unexplained_rejects_valid`** — by set difference over the 16 336-row
    manifest, not by net count (310 → 304 = 6 healed, 0 added).
  - ⛔ **`unexplained_accepts_invalid` is the control that matters, and its SET is
    BYTE-IDENTICAL (21 → 21).** This leaf deliberately *tightens* the language (`n1`), so a
    silent widening elsewhere is the failure mode; the set comparison is what rules it out.
  - **The ONLY adjudication transition anywhere in the manifest is 6 ×
    `unexplained_rejects_valid → match`.** Nothing else moved class in either lane.
  - **TARGETING — proven, not asserted:** the 6 flipped files are **set-equal** to the 6
    rows `keyed_dist_rows.py` keyed BEFORE the edit. **flipped-but-not-keyed = 0** and
    **keyed-but-not-flipped = 0** (the check is a set comparison in the leaf's own evidence
    bundle). Contrast `.3.8`, where 1 keyed row of 24 did not flip and had to be
    root-caused; here the fix is exactly as wide as the diagnosis said.
  - ⭐ **`verilator/t_constraint_dist_randc_bad.v` flipping to PASS is CORRECT, not
    over-acceptance** — the `_bad` in its name is a SEMANTIC error, and §18.5.4's own
    "Limitations" say so: *"A dist operation shall not be applied to randc variables."*
    That is an elaboration rule, not a syntax rule, so a conforming parser must parse the
    file and let elaboration reject it. Checked rather than assumed, because a `_bad`
    fixture flipping to pass is exactly the shape a real over-acceptance would have.
- ⭐ **INDEPENDENT RE-PROOF FROM THE RE-CUT WORKLIST** (the `.3.16` discipline — a second
  instrument, not written to confirm this fix, agreeing with it). Re-running the tracked
  `cluster_rejects_valid.py` + `classify_rejects_valid_families.py` against the new manifest:
  **304 rows / 184 signatures** (was 310 / 185), the `: = NUM` cluster that *surfaced* this
  leaf is **gone entirely** (4 → 0, the signature no longer exists), and
  `constraint/randomize (ch18)` drops **22 → 16**, with the sv-tests contribution to that
  family going to **0**. The signature count falling by exactly 1 is the tell that a whole
  stuck-point shape was retired rather than a few rows shuffled.
- ⛔⛔ **TWO PROOF GATES WERE ALREADY RED ON HEAD, AND THIS LEAF IS NOT THE CAUSE — measured.**
  `sv_cert_recognized_union_gate` failed `canonical total=1354 (expected 1352)` and
  `verilog_2005_conformance_gate` failed `cert total=1122 (expected 1121)`. Attribution was
  established with the toolbox before either contract was touched:
  `ast_pipeline --dump-rule-profiles` on the **pre-fix** and **post-fix** grammars gives an
  **identical** per-profile census (`sv_2017` 1354 → 1354, `sv_2023` 1373 → 1373,
  `verilog_2005` 1122 → 1122) with **zero** rules changing their satisfiable-profile set ⇒ this
  leaf's grammar edit contributes exactly **0**. The arrears belong to **`.3.14b`**
  (commit `3e316e3c`, release `1.0.178`), which added `in_scope_compiler_directive` +
  `in_scope_compiler_directive_sv_only` and re-baselined neither contract;
  `git show 6a2c088a:grammars/systemverilog.ebnf` (the `.3.9` commit that last set 1352/1121)
  contains neither rule. ⭐ **The asymmetry is the fingerprint:** `+2` in the sv_2017 union
  contract but `+1` in the v2005 one, because only `in_scope_compiler_directive` is satisfiable
  under `verilog_2005` — its twin is `@profiles`-gated. Both re-baselined here with attributing
  notes; the accounting is **fully positive** in both (proof/UNKNOWN/residual-set unchanged, the
  deltas land in `witness` and `proof` respectively), and the v2005 gate's **behavioural** half
  was green throughout (corpus matrix 240 × 0 mismatches, `profile_orphans 0`). Root cause —
  these gates are **operator-invoked, not in the automatic per-push tier**, so a census-moving
  grammar change leaves them red with no signal until someone runs them — routed to
  **`CI-PARITY-GATE-ROT.22`**. ⚠️ Recorded plainly because the tempting move was to fold a silent
  re-baseline into this commit and let the leaf read as if both gates had always been green.
- **GATES (green at landing):** `ast_shape_contract_gate` PASS — and **proven non-vacuous**:
  the new sample is listed in the run (`expression_or_dist_braced_list (rule=expression_or_dist)
  … structural_ok=true`), and breaking its expected-keys list on purpose made the gate FAIL with
  `missing required key 'THIS_KEY_DOES_NOT_EXIST'` before it was restored. A green gate never
  shown to go red is a claim, not a proof. `sv_syntax_closure_gate` PASS with
  **`defined_rule_count` 1477 UNCHANGED** and `unreachable_rules: 0` — as designed, since the
  grammar diff replaces one rule line with one rule line and adds no rules or tokens.
  `sv_cert_recognized_union_gate` PASS **after the re-baseline** — canonical UNKNOWN **11**,
  union UNKNOWN **0**, `union_residual_rules []`, `unmet_criteria_count 0`, deterministic across
  seeds 0/7/42. `verilog_2005_conformance_gate` PASS **after the re-baseline** — lint
  `orphans=0`, corpus **240 checks / 0 mismatches**, 2 alias checks, cert deterministic across
  seeds 0/7/42. `sv_external_corpus_triage_gate` PASS. `systemverilog_parser_book_gate` PASS
  (the new `json-carrier.md` section and the rebuilt `schema-versioning.md` table both render).
  `clippy_on_rust_change` PASS including the STRICT generated-parser stage.
  `check_published_version_currency.sh` OK. `bash scripts/check_doctrines.sh` **17/17**.
- **ROUTED OUT (never dropped — routing decides WHEN, not WHETHER,
  [[feedback_every_finding_must_be_fixed_not_logged]]):**
  1. **`LRM-GRAMMAR-FIDELITY`** — the exhaustive Annex-A bracket/brace sweep. This leaf is the
     seventh instance of the class and the third in A.2.10; `.3.8` raised exactly this and its
     own PICK notes named `dist { … }` as a residual, which was then rediscovered two weeks
     later by an unrelated cluster. The reactive posture is measurably not converging.
  2. **`SV-AST-SHAPE-FIDELITY.4`** — shape-locked share of the annotated SV surface measured at
     **2.09 %** (30 of 1 055 annotated rules). The `d4` mis-parse is the existence proof that
     nothing else in the repo asks whether the emitted tree is *right*.
  3. **`SV-AST-SHAPE-FIDELITY.5`** — the SV book's schema timeline had rotted for a **second**
     time (prose said `16`, contract said `19`, rows 17/18/19 absent for three releases). Fixed
     here; routed there because a surface backfilled twice by hand needs a currency gate.
- ⚠️ **HONEST BOUNDS (stated, not silently capped)**
  1. **`dist_weight` still admits white space the LRM's operator spelling arguably forbids** —
     it is modelled as the two tokens `colon assign` / `colon slash` (`:6164`/`:6174`), so
     `x dist {100 : = 1}` is tolerated. Same family as `.3.11`'s `time_literal` item, which is
     parked on a director scope call; **not introduced here**, and no corpus row depends on it.
     Deliberately out of scope rather than quietly folded in.
  2. **`.9` still owns the crafted cases.** The six healed rows stop being *blocked at the
     `dist`*; whether the rest of each file is LRM-clean is a different question their verdicts
     cannot answer.
  3. **The `dist` annotation is left as `{expr: $1, dist: $2}`** — faithful and minimal. A
     cleaner carrier that exposed the `dist_list` directly (dropping the brace tokens) would be
     a second schema change on the same field in the same release, so it is not folded in.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `furthest_position=49` on the 4-line
  `dist_list_braces/repro/min_dist.sv`; the full 8-case matrix in `before.txt` shows 4 LRM-legal
  forms REJECT (52/49/49/72), the LRM-illegal `n1` wrongly ACCEPT, and `d4` ACCEPT with the
  wrong AST kind; 6 corpus rows keyed by `keyed_dist_rows.py`.
- [x] **ROOT CAUSE (WHY + WHERE)** — `expression_or_dist:2344` renders A.2.10's LITERAL
  `{ dist_list }` as the EBNF repetition `dist_list*`. `--trace-rules
  expression_or_dist,dist_list,dist_item,dist_item_sv_2017,dist_weight` at
  `PGEN_TRACE_VERBOSITY=debug` shows both `dist_item` branches entered and failed **at position
  46, which is the `{`**, `dist_list` backtracking from the same byte, and `dist_weight` never
  entered (`trace_d1_before.txt`). LRM verified verbatim in-repo at
  `section-18-…:372/:487` + `section-15-…:1297`, with the literal-brace reading settled by the
  normative examples at `:503/:518/:520/:1393/:1406`.
- [x] **FIX** — tier 1, pure grammar: `( kw_dist_02450072 lbrace dist_list rbrace )?`
  (`grammars/systemverilog.ebnf:2344`). Existing tokens only; zero new rules or tokens.
- [x] **ADDRESSED (verified)** — repro matrix 4 REJECT→ACCEPT, `d4` AST kind `concat`→`number`,
  `n1` ACCEPT→REJECT, controls held; corpus pass **9 720 → 9 726 (+6)**, unexplained
  rejects-valid **310 → 304**. Oracles: `stimuli/run_external_corpus.sh sv 60 8 0` +
  `stimuli/sv/adjudicate_external_corpus.py` + `matrix.py`.
- [x] **NO REGRESSION** — per-file census **0 pass→fail / 0 pass→timeout / 0 pass→crash** over
  16 336 files; **ZERO new** rejects-valid by set difference; `unexplained_accepts_invalid` set
  **BYTE-IDENTICAL** (21); v2005 manifest `cmp`-clean with zero transitions; flipped set
  **set-equal** to the keyed set; `ast_shape_contract_gate` GREEN (negative-control-proven);
  `sv_syntax_closure_gate` GREEN with census 1477 unchanged.
- [x] **LOCKSTEP** — ledger `SV-0049`; contract `1.0.179` + schema `19`→`20`; SV book
  `changelog-index.md`, `json-carrier.md` (new "The `dist` Constraint Operator" section) and
  `schema-versioning.md` (row 20 **plus the reconstructed 17/18/19**); the new shape sample;
  `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `docs/TASK_TREE.md`; `SV-AST-SHAPE-FIDELITY`
  `.4`/`.5` opened.

##### `.3.19` — the config `use` clause cannot consume a `#`: IEEE 1800 Annex A's `use_clause` and clause 33's own normative EXAMPLES disagree, and the parser implements only Annex A (⭐ a NEW defect class — the contradiction is INSIDE the standard, not in PGEN's transcription of it)

- **Status: `done`** (2026-08-09, session #219, `PGEN-SV-CORPUS-GRAD-0037`; release
  `1.0.179` → **`1.0.180`**, schema **`20` UNCHANGED**, ledger **`SV-0050`**) — a REAL parser
  defect with parser bytes and **parser yield 8**, cut from the STUCK POSITION over the LIVE
  worklist per `MEMORY.md`'s standing warning against the family bucketer — a warning this
  leaf then turned into a measured instance (see the re-cut section).
- **PICK (measured).** Clusters **#6 `# ( .` (6 rows)** and **#24 `# ( )` (2 rows)** in the
  HEAD `rejects_valid_clusters.md` are one construct split by the clusterer purely on
  whether the override list is empty. The token-level tell is the same in both: the 3-token
  stuck window opens on `#` `(`. Widening from the cluster to the construct with the tracked
  positional scan (`keyed_config_use_rows.py`, `keyed_rows_before.txt`) gives the real
  extent: **8 / 304 rows are blocked at a config `use #( … )`**, across 2 suites
  (ispras-sv-tests 6, verilator 2) — i.e. the two clusters are exactly the whole construct,
  with nothing else mixed in.
- **REPRODUCE (tool-pinned, `config_use_param_override/before.txt`)** — 17 cases under
  `--profile sv_2017`. Every case is the SAME design + `config` block with only the single
  config-rule line varying, so a verdict difference can come from nothing but `use_clause`.

  | group | cases | verdict BEFORE |
  |---|---|---|
  | `a1`–`a7` — the four Annex A `use_clause` alternatives (incl. `:config`, `[lib.]`, the named-only and named-with-cell forms) | 7 | **ACCEPT** (all) |
  | `h1`–`h5` — `use #( … )`, copied VERBATIM from the LRM's OWN clause-33 examples | 5 | **REJECT** (246/249/249/249/249) |
  | `c1`–`c3` — controls: `liblist`, `cell … use`, ordinary module `#()` instantiation | 3 | ACCEPT |
  | `n1`/`n2` — POSITIONAL override inside a config, which LRM 33.4.3 forbids in so many words | 2 | REJECT (must stay) |

- **LRM GROUND TRUTH (verified verbatim in the in-repo LRM text of BOTH revisions before any
  edit).** ⭐ **The standard contradicts itself, identically, in 1800-2017 and 1800-2023.**
  - **Annex A / Syntax 33-4** — no `#` anywhere. Present four times in-repo and identical in
    all four: `2017/txt/section-33-…:227`, `2023/txt/section-33-…:227`,
    `2023/txt/section-Annex_A-…:271`, and `2017/txt/section-41-data-read-api.txt:290`
    (⚠️ that last path is an **extraction artifact**, not a citation error — the 2017 LRM
    split filed Annex A's body under a section name taken from a stray page header; the
    Annex A preamble sits in the same file at `:35`. The 2023 extraction is clean and carries
    identical text, so nothing here rests on the mis-titled file alone):

    ```
    use_clause ::= use [ library_identifier . ] cell_identifier [ : config ]
                 | use named_parameter_assignment { , named_parameter_assignment } [ : config ]
                 | use [ library_identifier . ] cell_identifier named_parameter_assignment
                   { , named_parameter_assignment } [ : config ]
    ```

  - **Clause 33.4.3's normative examples** — every parameter override is written `use #( … )`,
    **14 occurrences, 7 in each revision and at the same 7 places**:
    2017 `:365` `instance top use #(.WIDTH(32));`, `:366`, `:382`, `:383`,
    `:412` `use #(.W());`, `:423` `use #();`, `:462`; 2023 `:366`, `:367`, `:383`, `:384`,
    `:413`, `:424`, `:463`. A spelling repeated 7 times and then re-published unchanged six
    years later is not a typo.
- **THE ADJUDICATION, and it is decidable rather than a preference — three independent grounds:**
  1. ⭐ **Annex A subordinates itself, in its own preamble** (2023 `section-Annex_A-…:24`,
     2017 `section-41-data-read-api.txt:35`): *"The full syntax
     and semantics of SystemVerilog are not described solely using BNF. The normative text
     description contained within the clauses and annexes of this standard provide
     **additional details on the syntax** and semantics described in this BNF."* This is the
     same principle already ruled on in this repo for Annex A **footnotes** 44/48 —
     [[feedback_sv_strict_lrm_compliance_default]] §2, *"Annex A footnotes are normative and
     in scope, not just the BNF"*.
  2. ⭐⭐ **The BNF-only reading makes an explicit normative prohibition VACUOUS.** LRM 33.4.3
     states *"Configurations may not use positional parameter notation to override
     parameters."* (2017 `:333`, 2023 `:334`). Under Annex A's brace-less alternatives positional notation is not even
     **expressible** — there is no production that could carry it. The sentence is only
     meaningful if the `#( … )` form is intended, because that is the one place an
     `ordered_parameter_assignment` could otherwise appear. A standard does not forbid what
     its own grammar cannot write.
  3. **MEASURED, not recalled** (the trap this repo already burned itself on —
     [[feedback_sv_strict_lrm_compliance_default]], the provenance correction): the tracked
     census `census_use_clause_spellings.sh` (output `use_clause_census.txt`) enumerates every
     config `use` clause in the whole vendored corpus — **27 clauses: 12 parameter overrides,
     all 12 spelled `#( … )`; ZERO brace-less; 15 plain `use [lib.]cell [:config]`.** The
     Annex-A override alternatives are not merely unused by the parser, they are unused by the
     world. (Corroborating only — grounds 1 and 2 are internal to the standard and stand
     alone.)
  ⇒ the parser must accept the **union** of what the two normative surfaces print. Adding
  `use #( … )` is COMPLIANCE, not dialect tolerance: the standard prints those exact lines as
  legal configurations, so accepting them cannot be "accepting text the standard forbids".
- **ROOT CAUSE (WHY + WHERE) — grammar source, `grammars/systemverilog.ebnf`:**
  - **WHERE:** `use_clause:6039`–`6043` — four alternatives, faithfully transcribing Annex A
    A.1.5 and **only** Annex A. No alternative can begin with `hash`.
  - **WHY (trace, `config_use_param_override/trace_h1_before.txt`)** —
    `PGEN_TRACE_VERBOSITY=debug … --trace-rules config_rule_statement,use_clause` on
    `repro/h1_hash_named.sv` (the `.3.11` trap avoided: the parent is traced alongside the
    suspect, since the leaf rule alone prints nothing usable):

    ```
    ✅ Exiting rule 'inst_clause' successfully - advanced from 227 to 242
    🚪 Entering branch 1/4 for rule 'use_clause' at position 242
    ❌ Branch 1/4 for rule 'use_clause' failed at position 242
    …
    🚪 Entering branch 3/4 for rule 'use_clause' at position 242
    ❌ Exiting rule 'named_parameter_assignment' with error: Backtrack { position: 247
    ❌ Branch 3/4 for rule 'use_clause' failed at position 242
    🚪 Entering branch 4/4 for rule 'use_clause' at position 242
    ❌ Branch 4/4 for rule 'use_clause' failed at position 242
    ❌ Exiting rule 'use_clause' with error: Backtrack { position: 242
    ```

    Byte map: **227** = the `instance` starting the rule, **242** = where `use_clause` is
    entered, **247** = **the `#` itself**. `inst_clause` succeeds; all four `use_clause`
    branches then fail, and the one that gets furthest (branch 3, `use named_parameter_assignment …`)
    dies **on the `#`** because `named_parameter_assignment:3549` begins with `dot`. Nothing
    in the rule can consume a `#`.
- ⭐⭐ **THE CLASS FINDING — this is NOT the dropped-delimiter class, and that distinction is
  the actionable part.** The seven logged instances of that class (`SV-0002`
  `stream_concatenation`, `trans_range_list`, `boolean_abbrev`, the six bounded-property
  operators, `value_range`, `cycle_delay_range` `SV-0044`, `expression_or_dist` `SV-0049`)
  are all **PGEN transcription errors**: the LRM wrote a literal delimiter and PGEN read it as
  BNF metasyntax. Here PGEN's transcription of Annex A is **exactly right** — the delimiters
  are missing **in the standard**, and the standard's own clause body contradicts it. That is
  a different defect class with a different detector: no amount of care transcribing Annex A
  finds it, because it is only visible when the BNF is diffed against the clause's normative
  examples. ⇒ the `LRM-GRAMMAR-FIDELITY` Annex-A sweep, as currently chartered, would **not**
  have caught this one. Routed there as a distinct sub-item, not folded into the existing one
  ([[feedback_every_finding_must_be_fixed_not_logged]]).
- **FIX (hierarchy level 1 — pure grammar, existing tokens only):** add the clause-33 spelling
  as a fifth `use_clause` alternative, sited immediately after its brace-less Annex-A sibling:

  ```
  | @probe_sample: "use #(.P(1))" kw_use_04489a12 hash lparen
      ( named_parameter_assignment ( comma named_parameter_assignment )* )? rparen
      ( colon kw_config_dfba7aad )?
  ```

  `hash`/`lparen`/`rparen`/`comma`/`named_parameter_assignment` all already exist
  (`:6190`/`:6909`/`:6934`/`:3549`) ⇒ **zero new rules and zero new tokens**, so
  `defined_rule_count` is expected UNCHANGED.
  ⛔ **The list is `named_parameter_assignment`, NOT `list_of_parameter_assignments`.** The
  obvious-looking reuse (`parameter_value_assignment:4169` is literally
  `hash lparen ( list_of_parameter_assignments )? rparen`) would also admit
  `ordered_parameter_assignment` and therefore accept `use #(32)` — the one spelling LRM
  33.4.3 `:333` explicitly forbids. `n1`/`n2` are the controls that keep that honest.
  The empty `#()` form is first-class (LRM 2023 `:424`, and two corpus rows), hence the
  optional list rather than a `+`.
- ⭐⭐⭐ **THE FIRST ATTEMPT AT THIS FIX SILENTLY DELETED TWO ALTERNATIVES OF THE RULE, AND
  ONLY THE REPRO MATRIX CAUGHT IT.** This is the most important thing this leaf found, and
  it is recorded in full rather than quietly corrected — it is a defect in the EBNF
  frontend, i.e. in the single source of truth itself.
  - **What happened.** The new alternative was landed with its explanatory comment block at
    **column 0**, above the `|` line, which is how a comment before a *rule* is written.
    Regeneration and the release rebuild both succeeded. Then `after.txt` came back with
    **9** deviations instead of the expected 0: the five `h*` cases still REJECTED (the fix
    appeared not to exist) **and `a1`/`a2`/`a3`/`c2` — four cases that had ACCEPTED before
    the edit — had flipped to REJECT.**
  - **Why the obvious reading was WRONG, and why it was measured instead.** The tempting
    conclusion was "`#` is mishandled" — and the new alternative contains a `#` twice, once
    in the `@probe_sample: "use #(.P(1))"` STRING and once per comment line. Either guess
    would have produced a plausible, publishable, wrong root cause. The discriminator
    (`frontend_truncation_probe.sh`, output `frontend_truncation.txt`) is five one-rule
    synthetic grammars that each declare exactly three alternatives and differ in one thing:

    | case | shape | IR alternatives (want 3) |
    |---|---|---|
    | `C_baseline` | no comment | **3** |
    | `D_indented_comment` | comment INDENTED between alts | **3** |
    | `B_hash_in_string` | `#` inside a `@probe_sample` STRING | **3** |
    | `A_col0_comment_after_alt1` | comment at **column 0** after alt 1 | **1** — node degrades `Or`→`Sequence` |
    | `E_col0_comment_after_alt2` | comment at **column 0** after alt 2 | **2** |

    ⇒ strings are handled correctly and `#` is not the problem. **A column-0 comment line
    inside a rule body ENDS the rule**, and every alternative after it is discarded. `D` vs
    `A` differ only in the leading whitespace of a comment.
  - **Confirmed on the real rule at the IR level**, not inferred from the symptom:
    `ast_pipeline grammars/systemverilog.ebnf --generate-stimuli --dump-gen-ast` gave
    `use_clause` **3** alternatives (the three that preceded the comment) — so the new
    `#( … )` arm *and* the pre-existing simple `use [lib.]cell [:config]` arm *and* its
    `-> {library,name,config}` annotation had all been dropped. That explains both halves
    of the symptom exactly: `h*` still rejected because the new arm was never there, and
    `a1`/`a2`/`a3`/`c2` regressed because the SIMPLE arm they depend on was deleted too.
  - ⛔⛔ **EVERY INSTRUMENT THIS REPO OWNS WAS BLIND TO IT.** With two alternatives gone:
    `--lint-grammar` clean; `defined_rule_count` **1477, unchanged**; `--dump-rule-profiles`
    **byte-identical** across all 1 477 rules and all three profiles
    (`sv_2017` 1354 / `sv_2023` 1373 / `verilog_2005` 1122) — the very census this leaf had
    *already run and banked* as `rule_profile_census.txt` before the symptom appeared. A
    rule-count gate cannot see this because no rule is added or removed; only the *inside*
    of one rule shrinks.
  - **FIX in this leaf:** indent the comment block to match the six pre-existing interior
    comments in `systemverilog.ebnf`, plus a `⛔⛔` warning in the grammar at the site. IR
    re-verified: **5 alternatives, alt 4 is the new `#( … )` arm, and the annotation is back
    on the (now-5th) simple arm.** A whole-IR diff pre→post shows **exactly one rule
    changed — `use_clause`** — with `rule_order` identical and every other rule
    byte-identical.
  - **Is any live language missing today? NO — measured, both directions**
    (`truncation_census.py`, output `truncation_census.txt`, exits non-zero on any hit):
    census 1 (textual, ALL tracked grammars) finds **0** truncating sites; census 2
    (structural, SV) compares source `|`-count against IR alternative count for all **10**
    rules carrying an interior comment and finds **0 alternatives lost** — every
    pre-existing interior comment is indented and intact. So nothing shipped is wrong
    because of this; it is a live trap, not a live defect.
  - **ROUTED** to the new tree **`EBNF-FRONTEND-SILENT-TRUNCATION`**
    ([[feedback_every_finding_must_be_fixed_not_logged]]) — `.1` a mechanism-agnostic
    source⟷IR alternative-count gate, `.2` the frontend repair (and a written decision on
    what actually continues a rule body), `.3` the EBNF-dialect documentation. Not worked
    here: it does not block this leaf, and the lane lock is SV release
    ([[feedback_flow_findings_are_routed_not_worked]]).
  - ⚠️ **The direction of this failure was luck.** It removed alternatives, so the parser
    UNDER-accepted and a control row went red. The same truncation applied to a rule whose
    later alternatives carry `@predicate` gates or negative lookaheads would make the parser
    silently OVER-accept — and nothing in this repo would have gone red at all.
- ⚠️⚠️ **HONEST BOUND — THIS LEAF LEAVES A REAL `verilog_2005` OVER-ACCEPTANCE STANDING, AND
  IT IS FIXED BY `.3.20`, NOT MERELY LOGGED.** Surfaced by running the repro matrix under the
  v2005 profile as well, which is the only reason it was noticed at all.
  - **The LRM fact, verified in-repo:** IEEE 1364-2005 has **ONE** `use_clause` alternative —
    `use [library_identifier.]cell_identifier[:config]`
    (`docs/verilog/2005/txt/section-Annex_A-…:119`, repeated at
    `section-13-…:271`). There is **no parameter override in a 1364-2005 config at all**, and
    `use #(` appears **zero** times anywhere in the 2005 text. The named-override alternatives
    are a SystemVerilog addition.
  - **The grammar fact, measured:** `use_clause` carries **no `@profiles` gate** — pre AND
    post, `--dump-rule-profiles` reports it `satisfiable_under [sv_2017, sv_2023,
    verilog_2005]`. So all four pre-existing override alternatives (`a4`–`a7`) ALREADY
    accept under `verilog_2005`, and `after_v2005.txt` shows this leaf's `h1`–`h5` joining
    them. **The over-acceptance is pre-existing for 4 forms and extended to 5 by this leaf**
    — stated plainly rather than filed under "not introduced here".
  - **Blast radius, measured before deciding:** **0 of the 2 459 files in the v2005 lane
    contain a config `use` clause of any spelling** (scan over `v2005_lane_files.tsv`), so
    the gap costs zero corpus rows and the v2005 lane cannot move either way. Same shape as
    `.3.11`'s time-literal measurement — and per
    [[feedback_sv_strict_lrm_compliance_default]] that governs how much CARE the tightening
    needs, **never whether to do it**.
  - **Why it is not folded in here.** `@profiles` is a RULE-level directive, so gating four
    of five alternatives means splitting `use_clause` into a profiled sub-rule ⇒
    `defined_rule_count` MOVES ⇒ **both** cert contracts must be re-baselined in the same
    commit (the `CI-PARITY-GATE-ROT.22` tripwire, which exists because `.3.14b` did exactly
    this and shipped two RED gates for a whole release). That is a second, tightening change
    with its own per-file pass-set proof, bundled into a leaf whose subject is a different
    defect — the same reasoning `.3.18` used when it declined to fold a second schema change
    into one release.
  - ⇒ **`.3.20` opened and worked NEXT, in this same lane** — not parked. Routing decides
    WHEN, never WHETHER ([[feedback_every_finding_must_be_fixed_not_logged]]), and every
    piece of LRM evidence it needs is banked above.
- **ADDRESSED (verified) — the repro matrix flips exactly where the LRM says it should
  (`after.txt`, `after_v2005.txt`):**

  | group | before | after |
  |---|---|---|
  | `h1`–`h5` (clause-33 `use #( … )`) | REJECT (246/249/249/249/249) | **ACCEPT** ×5 |
  | `a1`–`a7` (the four Annex-A alternatives) | ACCEPT ×7 | ACCEPT ×7 |
  | `c1`–`c3` (controls) | ACCEPT ×3 | ACCEPT ×3 |
  | `n1`/`n2` (LRM-forbidden positional) | REJECT | **REJECT** — strictness held |
  | cases differing from the post-fix expectation | 5 | **0** |

  ⭐ `n1`/`n2` staying REJECT is the load-bearing row: it is what proves the fix reused
  `named_parameter_assignment` and not `list_of_parameter_assignments`.
- ⭐ **SCHEMA UNCHANGED AT `20` — MEASURED, not assumed.** `.3.18` bumped because one of its
  cases *parsed* before the fix with a wrong tree; here every `#( … )` form was a hard
  REJECT, so no witnessed wire shape can move. Proven rather than argued: the AST of all
  **10** already-parsing cases (`a1`–`a7`, `c1`–`c3`) was dumped on both parsers and is
  **byte-identical 10/10** (`cmp`). The new arm's own shape is
  `{kind: "inst_use", clause: …, body: [ ["use"], {kind:"hash"}, {kind:"lparen"},
  [<named assignments>], {kind:"rparen"} ]}`.
- **ADDRESSED — measured GLOBALLY, both lanes** (baselines preserved to
  `rust/target/sv_axis2_baseline/*.pre_3_19.*` **before** the run, per this tree's standing
  overwrite trap):
  - **MAIN `sv_2017` lane, 16 336 files — pass 9 726 → 9 734 (+8)**, fail 6 606 → 6 598,
    timeout 4 (unchanged), crash 0.
  - **Adjudication: `unexplained_rejects_valid` 304 → 296 (−8)**, `match` 5 772 → 5 780.
  - **V2005 lane BYTE-INERT:** 2 459 files, pass 2 181 / fail 278 / timeout 0 — **zero
    per-file transitions**, and `adjudication_manifest_v2005.tsv` is **BYTE-IDENTICAL**
    (`cmp` clean). Predicted from the lane scan (0 config `use` clauses) and then confirmed.
- **NO REGRESSION — the `.3.4` LAW, per-FILE, by set difference rather than net counts
  (`transitions_sv2017.txt`, `transitions_v2005.txt`, `adjudication_setdiff.txt`):**
  - **Transition matrix: `pass→pass` 9 726, `fail→fail` 6 598, `fail→pass` 8,
    `timeout→timeout` 4. ZERO `pass→fail`, ZERO `pass→timeout`, ZERO `pass→crash`**, and the
    file key-set is identical on both sides.
  - **ZERO NEW `unexplained_rejects_valid`** by set difference over the 16 336-row manifest
    (304 → 296 = 8 healed, 0 added).
  - ⛔ **`unexplained_accepts_invalid` is the control that matters and its SET is unchanged
    (21 → 21, 0 new, 0 healed).** This leaf WIDENS the language, so a silent over-acceptance
    elsewhere is the failure mode; the set comparison is what rules it out.
  - **The ONLY adjudication transition anywhere in either lane is 8 ×
    `unexplained_rejects_valid → match`.** The v2005 manifest has **0** rows changing class.
  - **TARGETING — proven, not asserted:** the 8 flipped files are **set-equal** to the 8 rows
    `keyed_config_use_rows.py` keyed BEFORE the edit. **flipped-but-not-keyed = 0** and
    **keyed-but-not-flipped = 0**; re-running the keyer after the fix returns **0 / 296**.
- ⭐ **INDEPENDENT RE-PROOF FROM THE RE-CUT WORKLIST** (the `.3.16`/`.3.18` discipline — a
  second instrument, not written to confirm this fix, agreeing with it): **296 rows / 182
  signatures** (was 304 / 184), and **BOTH** signatures that surfaced this leaf are gone
  entirely — `grep` for `# (` in the new cluster table returns **nothing**. The signature
  count falling by exactly 2 is the tell that two whole stuck-point shapes retired rather
  than rows shuffling.
- ⭐⭐ **AND THE RE-CUT CONVICTED THE COARSE FAMILY BUCKETER, CONCRETELY.** `MEMORY.md`'s
  standing warning is "⛔ NOT from the family bucketer"; this leaf turns that from advice into
  a measured instance. Of the 8 healed rows, the bucketer had filed **6 under
  `interface/modport (ch25)`** and 2 under `OTHER` — **not one under anything resembling
  config/ch33, because no such family exists in it.** A leaf-cutter trusting that view would
  have gone to IEEE 1800 clause 25 to fix a clause-33 defect. The `interface/modport` family
  correspondingly drops 24 → 18 and `OTHER` 226 → 224 — i.e. **the bucketer's second-largest
  family shrank by 25 % from a fix that touched no interface code at all.**
- ⚠️ **INSTRUMENT TRAP FOUND AND WORKED AROUND (worth its own note):
  `classify_rejects_valid_families.py` DEFAULTS TO THE RETIRED `_v2` ARTIFACTS.** Its
  `--clusters`/`--out-tsv`/`--out-md` defaults are `rejects_valid_clusters_v2.tsv` /
  `rejects_valid_families_v2.{tsv,md}`, so the obvious no-argument invocation (a) reads the
  **2026-07-23** cluster table and (b) writes the **`_v2`** outputs, leaving the LIVE
  `rejects_valid_families.{md,tsv}` untouched and stale. Caught because the regenerated
  families report said **304 rows** while the freshly-cut clusters said **296** — a
  one-number inconsistency that would otherwise have shipped as a stale tracked artifact. No
  damage: the `_v2` rewrite is **byte-identical to tracked** (`git status` clean on those two
  paths), which incidentally re-proves the classifier deterministic over an unchanged input.
  The correct invocation passes all three paths explicitly; routed to `.3.21`.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `furthest_position=246` on
  `config_use_param_override/repro/h1_hash_named.sv`; the 17-case matrix in `before.txt` shows
  all **5** clause-33 `use #( … )` forms REJECT (246/249/249/249/249) while all 7 Annex-A forms
  and 3 controls ACCEPT; **8 corpus rows** keyed by `keyed_config_use_rows.py`
  (`keyed_rows_before.txt`), across ispras-sv-tests (6) and verilator (2).
- [x] **ROOT CAUSE (WHY + WHERE)** — `use_clause:6039` implements IEEE 1800 Annex A A.1.5
  faithfully and completely, and Annex A has no `#`; the LRM's own clause 33.4.3 prints
  `use #( … )` 7× per revision. `--trace-rules config_rule_statement,use_clause` at
  `PGEN_TRACE_VERBOSITY=debug` shows `inst_clause` succeeding 227→242 and **all four**
  `use_clause` branches failing at 242, the deepest (branch 3) exiting
  `named_parameter_assignment` with `Backtrack { position: 247 }` — **byte 247 IS the `#`**
  (`trace_h1_before.txt`, byte map included). Nothing in the rule can consume it.
- [x] **FIX** — tier 1, pure grammar: a fifth alternative
  `kw_use hash lparen ( named_parameter_assignment ( comma named_parameter_assignment )* )? rparen ( colon kw_config )?`
  (`grammars/systemverilog.ebnf:6042`). Existing tokens only; **zero new rules or tokens**.
  ⛔ `named_parameter_assignment`, NOT `list_of_parameter_assignments` — the latter would accept
  `use #(32)`, which LRM 33.4.3 forbids.
- [x] **ADDRESSED (verified)** — repro matrix **5 REJECT→ACCEPT**, 7 Annex-A forms and 3 controls
  held, both LRM-forbidden positional cases still REJECT; **0 of 17** cases differ from the
  post-fix expectation, under `sv_2017` **and** `verilog_2005`. Corpus pass **9 726 → 9 734**,
  unexplained rejects-valid **304 → 296**; re-keying returns **0 / 296**. Oracles:
  `stimuli/run_external_corpus.sh sv 60 8 0` (release probe, memory guard) +
  `stimuli/sv/adjudicate_external_corpus.py` + `matrix.py` + `keyed_config_use_rows.py`.
- [x] **NO REGRESSION** — per-file census **0 pass→fail / 0 pass→timeout / 0 pass→crash** over
  16 336 files with an identical key set; **ZERO new** rejects-valid by set difference;
  `unexplained_accepts_invalid` set unchanged (21 → 21, 0 new); v2005 lane **0 transitions** and
  manifest `cmp`-clean; flipped set **set-equal** to the keyed set (both directions 0);
  `sv_syntax_closure_gate` GREEN with `defined_rule_count` **1477 unchanged** and
  `unreachable_rules: 0`; `sv_cert_recognized_union_gate` GREEN **with no re-baseline**
  (canonical UNKNOWN 11, union UNKNOWN 0, witness 1348, residual `[]`, seeds 0/7/42) —
  attributable in advance because `--dump-rule-profiles` is byte-identical pre→post across all
  1 477 rules and all three profiles (`rule_profile_census.txt`); `ast_shape_contract_gate` GREEN
  and **proven non-vacuous** (`ast_shape_negative_control.txt`);
  `verilog_2005_conformance_gate` GREEN; `clippy_on_rust_change` GREEN incl. the STRICT generated
  stage; `bash scripts/check_doctrines.sh` 17/17.
- [x] **LOCKSTEP** — ledger `SV-0050`; contract `1.0.180` (schema **20, deliberately unchanged**,
  with a measured justification); SV book `changelog-index.md`, `json-carrier.md` (new
  "The Config `use` Clause and Parameter Overrides" section) and `schema-versioning.md` (the
  non-bump recorded so a reader cannot mistake a missing row for a missing release); the new
  `use_clause_hash_named_override` shape sample + its dispatch arm; `CHANGES.md`,
  `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `docs/TASK_TREE.md`; new tree
  `EBNF-FRONTEND-SILENT-TRUNCATION`; new leaves `.3.20`, `.3.21`, `LRM-GRAMMAR-FIDELITY.1b`.

##### `.3.20` — the config `use_clause` parameter-override alternatives are not profile-gated, so `verilog_2005` accepts four (now five) forms IEEE 1364-2005 has no production for (routed by `.3.19`, 2026-08-09)

- **Status: `done` (2026-08-09, `PGEN-SV-CORPUS-GRAD-0039`, release `1.0.181`, schema **20
  UNCHANGED**, ledger `SV-0051`, parser yield 0 corpus rows BY CONSTRUCTION and that is the
  point).** It is a STRICTNESS (tightening) fix under
  [[feedback_sv_strict_lrm_compliance_default]], so it carries a regression risk the widening
  leaves do not: the per-FILE pass-set diff (the `.3.4` LAW) is the primary safety instrument,
  and any `pass → fail` halts the leaf. **Measured: 0 transitions of any kind, in either lane.**
- **ISSUE (already measured by `.3.19`, evidence banked at
  `docs/tasks/artifacts/sv_corpus_grad/config_use_param_override/after_v2005.txt`):** under
  `--profile verilog_2005` the parser accepts `use .W(8)`, `use .W(8), .D(16)`,
  `use adder .W(8)`, `use rtlLib.adder .W(8)` and (since `.3.19`) `use #( … )`. IEEE
  1364-2005 declares exactly one alternative — `use [library_identifier.]cell_identifier[:config]`
  (`docs/verilog/2005/txt/section-Annex_A-…:119`, `section-13-…:271`) — and contains zero
  occurrences of `use #(`.
- **ROOT CAUSE (already located):** `use_clause` (`grammars/systemverilog.ebnf:6039`) carries
  no `@profiles` directive, and `--dump-rule-profiles` confirms `satisfiable_under
  [sv_2017, sv_2023, verilog_2005]`. The override alternatives are SystemVerilog-only
  language sitting in a profile-universal rule.
- **PLANNED FIX (shape settled 2026-08-09 by the director's "make the decision" ruling — this
  leaf is SCHEDULED NEXT, not deferred):** split the **four** override alternatives into a
  `@profiles: ["sv_2017", "sv_2023"]` sibling named `use_clause_param_override_sv_only`, leaving
  `use_clause` with a reference to it plus the single 1364-2005 form. House pattern verified in
  place: `always_keyword:651` is exactly this shape (`kw_always … | always_keyword_sv_only`,
  with `@profiles` on the sibling at `:646`).

  ```
  @profiles: ["sv_2017", "sv_2023"]
  use_clause_param_override_sv_only := <alternatives 1-4, verbatim and IN ORDER>

  use_clause := use_clause_param_override_sv_only
              | @probe_sample: "use top" kw_use ( library_identifier dot )? cell_identifier ( colon kw_config )?
             -> {library: $2, name: $3, config: $4}
  ```

- ⛔ **TWO TRAPS TO CARRY INTO THE EDIT — both already paid for once in this tree:**
  1. **The sibling MUST be referenced FIRST.** `use_clause`'s own comment block (`:6032`-`:6038`,
     `GRAMMAR-WELLFORMED.G.4.8`) records that the override alternatives must precede the simple
     `use [lib.]cell [:config]` form, else PEG ordered choice commits the simple alt to
     `use lib.cell` and strands a trailing `.param()` — it was a cert-coverage
     witness-parseability residual. Referencing the sibling second would silently re-introduce
     that defect, and it is the kind that shows up as a coverage residual rather than a parse
     failure.
  2. **The `-> {library: $2, name: $3, config: $4}` annotation stays on the simple alternative
     and its `$N` indices DO NOT shift** — positional refs are per-ALTERNATIVE, which
     `.3.19` confirmed at IR level when it inserted a new alternative above this one
     (`branch_return_annotations` for `use_clause` went `[null,null,null,{…}]` →
     `[null,null,null,null,{…}]`, the object simply moving with its own arm).
  3. ⛔⛔ And the standing one: **any comment block added inside the rule body must be INDENTED**
     → `EBNF-FRONTEND-SILENT-TRUNCATION`.
- **EXPECTED CENSUS MOVE — predict it, then verify, because the prediction is falsifiable:**
  `defined_rule_count` **1477 → 1478**; per-profile `sv_2017` **1354 → 1355**, `sv_2023`
  **1373 → 1374**, `verilog_2005` **1122 → 1122 (UNCHANGED**, since the new rule is
  `@profiles`-gated out of it). ⇒ the `sv_cert_recognized_union_gate` contract needs a
  re-baseline and `verilog_2005_conformance_gate` should **not** — the same `+1 / +0` asymmetry
  `.3.18` used as the attributing fingerprint for `.3.14b`'s arrears. Run
  `ast_pipeline --dump-rule-profiles` on both grammars and diff BEFORE touching either contract.
- ⛔⛔ **THE CENSUS WILL MOVE, AND BOTH CERT CONTRACTS MUST BE RE-BASELINED IN THE SAME
  COMMIT** — `defined_rule_count` 1477 → 1478 (or more), `sv_2017`/`sv_2023` +1 and
  `verilog_2005` +0, so `sv_cert_recognized_union_gate` and `verilog_2005_conformance_gate`
  both need new baselines with attributing notes. This is the exact `CI-PARITY-GATE-ROT.22`
  tripwire that `.3.14b` tripped and shipped RED for a whole release; it is written here in
  advance so the leaf cannot forget.
- **EXPECTED BLAST RADIUS: zero corpus rows.** `.3.19` scanned all 2 459 v2005-lane files and
  **none contains a config `use` clause of any spelling**, so the v2005 manifest should be
  `cmp`-clean and the sv_2017 lane untouched. ⚠️ That prediction is the thing to VERIFY, not
  to assume — a tightening that moves nothing is also what a fix that silently did nothing
  looks like, so the leaf must show the repro matrix flipping `a4`–`a7`/`h1`–`h5` to REJECT
  under `verilog_2005` while holding them ACCEPT under `sv_2017`.

**WORKED 2026-08-09 — what the plan above got right, and the one thing it got wrong:**

- ⭐⭐ **THE REAL FINDING IS NOT THE FIX, IT IS THAT THE V2005 ARM OF THE MATRIX HAD NO ORACLE.**
  `.3.19` ran its 17-case matrix under `verilog_2005` and banked the output as
  `after_v2005.txt` — 17 rows, every `want` column reading `ACCEPT`, and a footer saying
  **"cases differing from the post-fix expectation: 0"**. That file records nine
  over-acceptances and calls them green. The cause is in `matrix.py`: it carried ONE
  expectation column, `sv_2017`'s, and its own code said so —
  *"`want` is the post-fix expectation under sv_2017 only; other profiles print it for
  reference without judging"*. An instrument that prints a column it does not judge is a
  confident guess with a header ([[feedback_instrument_needs_ground_truth]]).
  ⛔ The over-acceptance was found by a HUMAN reading a column the tool declined to check —
  which is exactly the failure mode the tool exists to remove. `.3.20` therefore fixes the
  instrument first: `CASES` now carries a per-profile expectation `(sv, v2005)`, **every**
  profile is judged, and an unrecognized profile is a **REFUSAL** (proven: `--profile pcre2`
  exits 1 with `REFUSE: no per-case expectation declared`) rather than an unjudged run.
  Re-running the FIXED matrix against the UNCHANGED pre-fix parser is what produced
  `before_3_20_v2005.txt`: **9 cases differing**, from the same binary that had reported 0.
- **ROOT CAUSE (WHY + WHERE), re-measured on this leaf rather than inherited:**
  `./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --dump-rule-profiles` reports
  `use_clause -> {'declared_profiles': None, 'satisfiable_under': ['sv_2017', 'sv_2023',
  'verilog_2005']}`. The rule carries no `@profiles` directive at all, so four
  SystemVerilog-only alternatives sit in a profile-universal rule
  (`rule_profile_census_3_20.txt`).
- **FIX — tier 1, pure grammar, exactly the shape settled in advance.** `@profiles` is a
  RULE-level directive, so the four override alternatives moved verbatim and IN ORDER into
  `@profiles: ["sv_2017", "sv_2023"] use_clause_param_override_sv_only`, and `use_clause`
  became `use_clause_param_override_sv_only | <the single 1364-2005 form>`. House pattern:
  `always_keyword` / `always_keyword_sv_only` (`:646`). Zero new tokens.
- **ALL THREE ADVANCE-DECLARED TRAPS CLEARED, VERIFIED AT IR LEVEL, NOT BY READING THE FILE**
  (`ir_split_verification_3_20.txt`, from `--dump-gen-ast` on both grammars):
  1. **Sibling referenced FIRST** — `use_clause` alternative 0 is the bare
     `rule_reference use_clause_param_override_sv_only`, so `GRAMMAR-WELLFORMED.G.4.8`'s
     specific-before-general ordering is *relocated*, not relaxed.
  2. **`$N` indices unshifted** — `branch_return_annotations['use_clause']` goes
     `[null,null,null,null,{library: $2, name: $3, config: $4}]` →
     `[null,{library: $2, name: $3, config: $4}]`: the object moved with its own arm.
  3. **The moved comment block did not truncate the new rule** —
     `use_clause_param_override_sv_only` has **4** alternatives, and the whole-grammar IR diff
     shows **exactly one rule added, one rule's body changed, `rule_order` otherwise
     identical**, with annotation deltas confined to those two rules. This leaf moves a comment
     block into a new rule body, i.e. straight into the `EBNF-FRONTEND-SILENT-TRUNCATION` trap
     `.3.19` paid for; the IR is the only instrument that can see it.
- ⭐ **THE CENSUS PREDICTION WAS FALSIFIABLE AND IT HELD, DIGIT FOR DIGIT.** Predicted before
  the edit: `defined_rule_count` 1477 → 1478, `sv_2017` 1354 → 1355, `sv_2023` 1373 → 1374,
  `verilog_2005` 1122 → **1122 unchanged**. Measured: exactly that, with **0** rules changing
  their satisfiable-profile set. ⇒ `sv_cert_recognized_union_gate` needs a re-baseline and
  `verilog_2005_conformance_gate` does not.
- ⚠️ **THIS LEAF'S OWN PLAN CONTRADICTED ITSELF ABOUT THAT, AND THE MEASUREMENT SETTLED IT.**
  One bullet above says *"BOTH CERT CONTRACTS MUST BE RE-BASELINED"*; the bullet before it says
  the union contract *"needs a re-baseline and `verilog_2005_conformance_gate` should **not**"*.
  Both were written the same day. The `+1 / +0` census is the arbiter: the union contract moved
  and was re-baselined **in the same commit**; the v2005 contract was left alone and its gate
  came back GREEN with the cert census `1122/329/779/14` **unchanged**. Recording this rather
  than quietly picking one: a plan that disagrees with itself is a plan whose numbers were
  never measured, and the only safe response is to run the instrument.
- **The re-baseline is the gate's OWN output, not a hand-computed prediction.** The gate was run
  BEFORE the contract was touched and named all 12 unmet criteria itself — 4 per seed × 3 seeds,
  identical deltas: `canonical total 1355 (expected 1354)`, `canonical witness 1338 (expected
  1337)`, `union total 1355 (expected 1354)`, `union witness 1349 (expected 1348)`. Accounting
  fully positive: `expected_proof` 6, `expected_canonical_unknown` 11, `expected_union_unknown`
  0 and `expected_union_residual_rules` `[]` all UNCHANGED — the new rule is WITNESSED.
- ⭐ **SCHEMA STAYS AT 20 — and this non-bump is the one that had to be measured.** `.3.19`
  could argue its non-bump from first principles (the construct was 100 % unparseable, so no
  witnessed shape could move). This leaf cannot: it RESTRUCTURES a rule that parses today. Two
  independent instruments say the AST does not move: **15/15** accepting repro cases `cmp`-identical
  under `sv_2017` (and the 6 still-legal ones identical under `verilog_2005`), and the
  `use_clause_hash_named_override` shape lock — whose observed `content_kind` is compared against
  a LIVE parse on every gate run (`ast_shape_contract.rs:639`) — unmoved. ⇒ **a bare
  rule-reference alternative carrying no return annotation is AST-transparent.** That is a
  reusable fact about PGEN's emission, not a fact about `use_clause`
  (`ast_identity_3_20.txt`). ⛔ `.3.19` added that sample writing *"`.3.20` is about to SPLIT
  this rule … this sample is what makes that restructuring gate-visible instead of silent"* —
  the lock was placed one leaf in advance and it did its job.
- **ADDRESSED (verified) — the repro matrix moves in exactly one profile:**

  | profile | before | after |
  |---|---|---|
  | `verilog_2005` | **9** cases differ from the LRM expectation | **0** |
  | `sv_2017` | 0 | 0 |
  | `sv_2023` | 0 | 0 |

  The nine are `a4`–`a7` (the four named-override forms, over-accepted since the parser existed)
  and `h1`–`h5` (the `#( … )` forms, over-accepted since `.3.19`). ⭐ The load-bearing rows are the
  ones that did **not** move: `a1`–`a3` (the legal 1364-2005 spellings), `c2` (a `cell_clause`
  `use`), and above all **`c3` — an ordinary module instantiation `adder #(8, 16) a1();`**, which
  is legal Verilog-2001/2005 and would have been collateral damage from any fix that reached for
  the shared `parameter_value_assignment` surface instead of the config-local one.
- **NO REGRESSION — the `.3.4` LAW, per-FILE, in BOTH lanes** (`transitions_3_20_sv2017.txt`,
  `transitions_3_20_v2005.txt`, `adjudication_setdiff_3_20.txt`):
  - **sv_2017: 16 336 files, `pass→pass` 9 734, `fail→fail` 6 598, `timeout→timeout` 4 —
    ZERO transitions of any kind.** v2005: **2 459 files, `pass→pass` 2 181, `fail→fail` 278 —
    ZERO transitions.** Key sets identical on both sides of both lanes.
  - **Both adjudication manifests `cmp`-CLEAN** ⇒ **0 rows changed adjudication class anywhere**,
    in either lane. `unexplained_rejects_valid` — ⛔ **the control that matters for a TIGHTENING**,
    since taking language away is what turns a correctly-accepted file into a wrongly-rejected
    one — is 296 (sv_2017) / 54 (v2005) with **0 new** by set difference. `unexplained_accepts_invalid`
    likewise 21 / 14, 0 new.
  - The raw `results*.tsv` differ from their baselines only in parallel-job ROW ORDER; sorted, they
    are byte-identical. Stated rather than hidden, because "the file changed" would otherwise read
    as a finding.
- ⭐ **AND THE ZERO WAS PREDICTED FROM THE GRAMMAR, NOT OBSERVED AFTERWARDS** — which is the only
  way to tell a tightening that correctly moves nothing from a fix that silently did nothing.
  `v2005_blast_radius.py` re-derives the reachability premise from the grammar text itself
  (`use_clause` ← `config_rule_statement` ← `config_declaration`, which opens with the `config`
  keyword) and then scans the lane: **0 of 2 459 files contain the token `config` at all** — so
  the maximum reachable blast radius is literally zero. ⛔ It **REFUSES** rather than reporting a
  number if that premise stops holding, and carries three ground-truth controls (positive,
  negative, and a planted `use_clause` reference the derivation must catch). Its refusal path
  is not hypothetical: it fired during authoring, because the grammar spells the keyword rule
  `kw_config_dfba7aad` and the check looked for `kw_config`. A guard that fires for a
  bookkeeping reason is one edit away from a guard that never fires, so the controls are now
  permanent.
- ⚠️ **INSTRUMENT GAP FOUND AND ROUTED — `clippy_on_rust_change` is structurally blind to a
  pure-grammar change.** Its trigger set is derived from `git diff` / `git ls-files --others`
  and includes `generated/*.rs` — but `generated/` is `.gitignore`d, so a regenerated parser is
  invisible to it. A leaf that edits only `grammars/*.ebnf` therefore gets *"No Rust/generated
  Rust changes detected; skipping clippy flow"* even though it just rewrote 131 MB of the very
  artifact the flow exists to lint. Worked around here with `PGEN_CLIPPY_FORCE=1` (GREEN: source
  strict + the STRICT generated stage + the correctness-roster policy check); ⛔ **it did not
  block this leaf and the lane lock is the SV release, so it is ROUTED, not worked** →
  `CI-PARITY-GATE-ROT.23` ([[feedback_flow_findings_are_routed_not_worked]],
  [[feedback_every_finding_must_be_fixed_not_logged]]).

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — the FIXED matrix run against the UNCHANGED pre-fix parser
  (`before_3_20_v2005.txt`): under `--profile verilog_2005`, `a4`–`a7` and `h1`–`h5` all report
  `ACCEPT` against `want REJECT`, footer **"cases differing from the post-fix expectation: 9"**,
  while `before_3_20_sv2017.txt` / `before_3_20_sv2023.txt` report **0**. Minimal reproducer:
  `./rust/target/release/parseability_probe --parse systemverilog repro/a4_use_named_only.sv
  --profile verilog_2005` → `parse_full passed` (wrong; IEEE 1364-2005 Annex A A.1.5 `:119` has
  one `use_clause` alternative and zero occurrences of `use #(`).
- [x] **ROOT CAUSE (WHY + WHERE)** — `./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf
  --dump-rule-profiles` reports `use_clause -> {'declared_profiles': None, 'satisfiable_under':
  ['sv_2017', 'sv_2023', 'verilog_2005']}` (`rule_profile_census_3_20.txt`): the rule at
  `grammars/systemverilog.ebnf:6039` carries no `@profiles` directive, so four SystemVerilog-only
  alternatives are satisfiable under the strict profile. `--lint-grammar` is clean before and
  after (`profile_orphans=0`), which is exactly why nothing was reporting it.
- [x] **FIX** — tier 1, pure grammar, zero new tokens: the four override alternatives move verbatim
  and in order into `@profiles: ["sv_2017", "sv_2023"] use_clause_param_override_sv_only`
  (`grammars/systemverilog.ebnf:6049`), referenced FIRST by `use_clause`. Why no lower tier: the
  `@profiles` directive is rule-level, so a per-alternative gate is not expressible declaratively;
  the sibling split IS the declarative idiom (`always_keyword_sv_only`, `:646`).
- [x] **ADDRESSED (verified)** — repro matrix under `verilog_2005` **9 cases differing → 0**, with
  `sv_2017` and `sv_2023` holding at **0 → 0**; nine spellings flip ACCEPT→REJECT and the six legal
  ones plus all three controls hold. Oracles, each re-runnable and deterministic:
  `matrix.py --profile {sv_2017,sv_2023,verilog_2005}` (release probe);
  `ast_pipeline --dump-rule-profiles` (census `1477→1478`, `sv_2017 1354→1355`, `sv_2023
  1373→1374`, `verilog_2005 1122→1122`, exactly as predicted);
  `ast_pipeline --generate-parser --dump-gen-ast` (IR: one rule added, one body changed,
  `rule_order` otherwise identical, sibling has its 4 alternatives).
- [x] **NO REGRESSION** — per-file census **0 transitions of ANY kind** across 16 336 sv_2017 files
  and 2 459 v2005 files (identical key sets); both adjudication manifests **`cmp`-clean**, so 0 rows
  changed adjudication class and `unexplained_rejects_valid` (the tightening control) is 296/54 with
  **0 new** by set difference; ASTs **byte-identical 15/15** under `sv_2017`;
  `sv_syntax_closure_gate` GREEN (`defined_rule_count: 1478`, `unreachable_rules: 0`,
  `unresolved_rule_reference_count: 0`); `sv_cert_recognized_union_gate` GREEN after an in-commit
  re-baseline (canonical UNKNOWN 11, union UNKNOWN 0, witness 1349, residual `[]`, seeds 0/7/42,
  `unmet_criteria_count: 0`); `verilog_2005_conformance_gate` GREEN with **NO** re-baseline
  (`CERTIFICATE-COVERAGE: … total=1122 proof=329 witness=779 UNKNOWN=14 … sample_parse_failures=0,
  proof_reverify_failures=0`, corpus 240 checks / 0 mismatches, `lint_profile_orphans: 0`, seeds
  0/7/42); `ast_shape_contract_gate` GREEN (18/18, `use_clause_hash_named_override` observed
  `content_kind` unmoved); `clippy_on_rust_change` GREEN under `PGEN_CLIPPY_FORCE=1` incl. the
  STRICT generated stage and `GENERATED-CLIPPY-CORRECTNESS: ✅ POLICY-ONLY PASS`;
  `bash scripts/check_doctrines.sh` 17/17.
- [x] **LOCKSTEP** — ledger `SV-0051`; contract `1.0.181` (schema **20, deliberately unchanged**,
  with a measured justification and a current-state note closing `.3.19`'s flagged residual);
  `sv_cert_recognized_union_gate` contract re-baselined with an attributing note in the SAME commit
  (`CI-PARITY-GATE-ROT.22`); SV book `changelog-index.md`, `json-carrier.md` (the section's
  "Bound worth knowing" replaced by a per-profile acceptance table) and `schema-versioning.md`;
  `matrix.py` upgraded to a per-profile oracle; new artifacts
  `analyze_adjudication_setdiff.py`, `v2005_blast_radius.py`; `CHANGES.md`,
  `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `docs/TASK_TREE.md`; new leaf `CI-PARITY-GATE-ROT.23`.

##### `.3.21` — `classify_rejects_valid_families.py` defaults to the RETIRED `_v2` artifacts, so the obvious invocation silently reads a July input and leaves the live report stale (routed by `.3.19`, 2026-08-09)

- **Status: `todo`** — instrument hygiene on the axis-2 worklist chain. Small, but it sits
  directly under the burn-down's PICK step, so a stale read here mis-aims a whole leaf.
- **ISSUE (measured in `.3.19`):** run with no arguments, the script's defaults are
  `--clusters …/rejects_valid_clusters_v2.tsv`, `--out-tsv …/rejects_valid_families_v2.tsv`,
  `--out-md …/rejects_valid_families_v2.md` (`stimuli/sv/classify_rejects_valid_families.py`
  `:79`/`:81`/`:83`). So it (a) classifies the **2026-07-23** cluster table and (b) writes the
  `_v2` outputs, while the LIVE `rejects_valid_families.{md,tsv}` — the tracked artifact the
  burn-down actually reads — is left untouched and stale.
- **HOW IT SURFACED, and why that matters:** the regenerated families report claimed **304
  rows** while the clusters cut minutes earlier said **296**. A one-number inconsistency. It
  was noticed only because both numbers happened to be on screen together; nothing in the
  script, and no gate, compares them.
- **The sibling `cluster_rejects_valid.py` does NOT share the defect** — its defaults already
  point at the live `rejects_valid_clusters.{tsv,md}`. So the two halves of one pipeline
  disagree about which vintage is current, which is the worst possible arrangement.
- **PLANNED FIX:** point the defaults at the live artifacts (matching the clusterer), and add
  a REFUSAL when the input cluster table's row count disagrees with the manifest's current
  `unexplained_rejects_valid` count — an instrument that cannot notice it is reading a stale
  input is a confident guess ([[feedback_instrument_needs_ground_truth]]).
- **No damage this time, and it was checked rather than assumed:** the `_v2` rewrite is
  byte-identical to the tracked July version (`git status` clean on both paths), which
  incidentally re-proves the classifier deterministic over an unchanged input.

**WORKED 2026-08-09 (`PGEN-SV-CORPUS-GRAD-0040`) — and it was TWO defects, not one:**

- **Status: `done`.** Instrument-only; **zero parser bytes**, no release, no schema, no ledger
  row. Evidence: `docs/tasks/artifacts/sv_corpus_grad/families_classifier_vintage/vintage_and_corruption.txt`.
- **REPRODUCE, run rather than quoted.** The HEAD script was copied back into `stimuli/sv/`
  (so its `parent.parent.parent` root resolution still lands on the repo root) and invoked
  with only its two OUTPUT paths redirected, exercising the default `--clusters` exactly as an
  operator hits it: it classified **543 rows** of the 2026-07-23 table against a live
  population of **296**, and wrote the `_v2` outputs.
- ⚠️ **HONEST SCOPE — the live report was NOT stale at HEAD, and saying otherwise would have
  been the easy overclaim.** `.3.19` was burned by this once and thereafter passed all three
  paths explicitly, so the tracked `rejects_valid_families.{tsv,md}` were current at 296. The
  defect is a **latent trap in the defaults** — fired once, armed for the next operator — not
  a stale artifact today. What made it dangerous is that the sibling `cluster_rejects_valid.py`
  already defaulted to the LIVE paths: two halves of one pipeline, each individually
  self-consistent, disagreeing about which vintage is current.
- ⭐⭐ **AND BUILDING THE ROW-COUNT REFUSAL EXPOSED A SECOND, OLDER DEFECT: the input TSV was
  CORRUPT, in every tracked vintage.** A field-count histogram of the tracked cluster table:
  **295 lines with 6 fields, 1 with 7, 2 with 1, 1 empty** — 299 physical data lines over a
  296-row population.
  - **ROOT CAUSE (WHY + WHERE):** `cluster_rejects_valid.py::probe_one` `:219` returns
    `(suite, rel, -3, -3, "<NO-POSITION>", out.strip()[:120])` — the probe's raw stdout+stderr.
    A probe *read* error is multi-line (`Error: failed to read input file …` / blank /
    `Caused by:` / `stream did not contain valid UTF-8`), so one record became **four physical
    lines**. Separately one `stuck_line` contained a literal TAB, giving a 7-field row.
  - **WHY IT WAS INVISIBLE:** the classifier's `if len(cols) < 6: continue` dropped the three
    orphan fragments **in silence**, and 295 + 1 = 296 still matched the manifest. Every
    published total was right by accident.
  - ⛔ **This is why the refusal could not be a `wc -l` comparison.** On a healthy pipeline it
    would have read 299 against 296 and fired — a false positive, which is precisely what
    teaches an operator to disable a check. The corruption had to be fixed at the SOURCE
    before the count could mean anything.
- **FIX 1 — `cluster_rejects_valid.py::tsv_cell`,** applied at the write site so every column
  is safe by construction. ⚠️ **The first cut was too wide and the diff said so:**
  `" ".join(value.split())` also collapsed whitespace RUNS and rewrote **79 lines** of a
  296-row artifact that has one real defect. Narrowed to a one-for-one replacement of
  `[\r\n\t]`; the re-cut then differs from the tracked table in **exactly the two defective
  records**, with the `.md` summary byte-identical (0 diff lines) and the regenerated
  `families.tsv` differing in the same two rows. A repair whose diff is forty times the size
  of the bug is a second change smuggled in beside the first.
- **FIX 2 — two refusals plus three always-on controls**, replacing the reader's vigilance:
  defaults now name the LIVE artifacts; a malformed line **REFUSES** (naming file, line
  number and content) instead of being skipped; and the row count must equal the live
  manifest's `unexplained_rejects_valid` count, with `--expect-rows` as a deliberate
  off-manifest escape hatch. `ground_truth_controls()` pins a positive, a negative and a
  detector-live check before any number is published — because the reconciliation is only as
  good as the predicates beneath it, and *a bucketer that returned one family for everything
  would reconcile perfectly on row count*.
- ⛔⛔ **MY OWN FIRST EVIDENCE RUN FOR THIS LEAF WAS WRONG TWICE, IN THE SAME CLASS THE LEAF
  REPAIRS — recorded, not quietly re-run.** (a) It read `$?` through a `| sed` pipeline, so
  every refusal printed `exit=0`: an evidence harness that cannot see a failure. (b) It tried
  to demonstrate the ROW-COUNT refusal using the retired `_v2` table — but that table is
  *also* corrupted, so the MALFORMED refusal fired first and the row-count path was never
  exercised at all. **A refusal that fires for the wrong reason is not evidence for the reason
  you wanted.** The corrected proof uses a well-formed 295-row table (the live one minus a
  row) so the path under test is the only one that can fire.
- **ADDRESSED (verified) — all four paths proven with REAL exit codes:** malformed input
  `exit=1` naming line 147; wrong vintage `exit=1` (295 vs 296); missing manifest with no
  `--expect-rows` `exit=1`; the deliberate `--expect-rows 295` escape hatch `exit=0`; and the
  repaired live pipeline with **no arguments** printing
  `reconciled: 296 rows == …adjudication_manifest.tsv (divergence:unexplained_rejects_valid)`
  then `classified 296 rows into 8 families`, `exit=0`.
- ⚠️ **ROUTED, not worked → `.3.22`:** the one `<NO-POSITION>` row is
  `sv2v/test/lex/latin1.sv`, a deliberately Latin-1-encoded lexer fixture the probe cannot
  READ as UTF-8. It sits in `unexplained_rejects_valid` — the defect-signal class — while
  being a fact about the instrument's input handling, not about the grammar. Same shape as the
  `SV-CORPUS-GRAD.11a` tripwire (*a corpus `timeout` is a fact about the INSTRUMENT, not the
  parser*). One row of 296; it does not block the release lane
  ([[feedback_flow_findings_are_routed_not_worked]]).

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — the HEAD script run in place with only its output paths
  redirected classified **543 rows** from its default `--clusters`
  (`rejects_valid_clusters_v2.tsv`, 2026-07-23) against a live population of **296**, and
  wrote the `_v2` outputs; `git show HEAD:stimuli/sv/classify_rejects_valid_families.py |
  grep -n 'default=base'` names all three stale defaults at `:79`/`:81`/`:83`.
- [x] **ROOT CAUSE (WHY + WHERE)** — two, both located by running tools over the artifacts
  rather than by reading code: (1) the three `default=base / "…_v2…"` argparse defaults, while
  the sibling clusterer's defaults at `cluster_rejects_valid.py:243`/`:246` already point at
  the live paths; (2) `git ls-files`-tracked `rejects_valid_clusters.tsv` yields the field
  histogram `295×6, 1×7, 2×1, 1×0` — 299 physical lines for 296 rows — traced to
  `cluster_rejects_valid.py:219`, where the `<NO-POSITION>` branch writes the probe's raw
  multi-line stdout+stderr into the last TSV column unescaped, and to the classifier's
  `if len(cols) < 6: continue`, which discarded the resulting fragments silently.
- [x] **FIX** — tier 1, ops/instrument, no parser bytes: `tsv_cell()` at the clusterer's write
  site replacing `[\r\n\t]` one-for-one (deliberately NOT collapsing whitespace runs — that
  first cut rewrote 79 lines for a 2-line defect); live defaults, a malformed-line REFUSAL, a
  manifest row-count REFUSAL with an `--expect-rows` escape hatch, and three always-on
  ground-truth controls in the classifier.
- [x] **ADDRESSED (verified)** — re-cut cluster table differs from tracked in **exactly the two
  defective records** (`diff` shown in full), `.md` summary byte-identical, field histogram now
  `296×6` with zero malformed lines; all four refusal/acceptance paths re-run with real exit
  codes (`exit=1`,`exit=1`,`exit=1`,`exit=0`) plus the no-argument green path printing
  `reconciled: 296 rows == …(divergence:unexplained_rejects_valid)`. Oracles, each re-runnable:
  `python3 stimuli/sv/classify_rejects_valid_families.py` (no args) and
  `awk -F'\t' 'NR>1{print NF}' … | sort -n | uniq -c` over the cluster table.
- [x] **NO REGRESSION** — zero parser bytes, so no parse behaviour can move: `git diff` touches
  only the two `stimuli/sv/*.py` instruments and their three output artifacts. The regenerated
  `rejects_valid_families.tsv` differs from tracked in **exactly the same two rows** as the
  cluster table and nowhere else; the family ranking is unchanged (`OTHER` 224,
  `interface/modport` 18, `constraint/randomize` 16, `SVA implication/property` 11, …, 8
  families over 296 rows), which is the burn-down's PICK input and therefore the thing that
  must not silently shift. `bash scripts/check_doctrines.sh` 17/17.
- [x] **LOCKSTEP** — new evidence artifact
  `docs/tasks/artifacts/sv_corpus_grad/families_classifier_vintage/vintage_and_corruption.txt`;
  both instrument docstrings rewritten to carry the why; new routed leaf `.3.22`;
  `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `docs/TASK_TREE.md`. No book/contract/
  ledger/schema change — N/A, the change is internal tooling with no user-visible parser
  surface.

##### `.3.22` — one `unexplained_rejects_valid` row is an ENCODING failure, not a parser defect: the probe cannot READ `sv2v/test/lex/latin1.sv` (routed by `.3.21`, 2026-08-09)

- **Status: `todo`.** Routed, not worked: one row of 296, and it does not block the SV release
  lane ([[feedback_flow_findings_are_routed_not_worked]]).
- **ISSUE (measured):** the live cluster table carries exactly one `<NO-POSITION>` row —
  `sv2v test/lex/latin1.sv`, whose stuck "line" is the probe's own error, *"failed to read
  input file … Caused by: stream did not contain valid UTF-8"*. The file is a deliberately
  Latin-1-encoded **lexer** fixture; the parser never ran on it.
- **WHY IT MATTERS beyond one row:** it is adjudicated
  `divergence:unexplained_rejects_valid` — the class that means *"the parser wrongly rejected
  valid input"*, i.e. the defect signal the whole `.3` burn-down is cut from. It will never
  yield to a grammar fix, so it is a permanent unit of noise in the number the campaign is
  driving to zero. Same shape as the standing `SV-CORPUS-GRAD.11a` tripwire: **a corpus
  `timeout` is a fact about the INSTRUMENT, not the parser** — and so is an unreadable file.
- **THE ADJUDICATION QUESTION (decide before coding):** does PGEN owe IEEE 1800 anything on
  non-UTF-8 source? 1800-2017 §5.1 admits an implementation-defined character set beyond
  ASCII, so a Latin-1 file is arguably legal input a signoff-grade tool should read. Two
  defensible outcomes, and the leaf must pick one on evidence, not convenience:
  (a) the expected verdict is wrong ⇒ reclassify to an `out_of_scope_with_cause:encoding` /
  `impl_varying` deferral in `adjudicate_external_corpus.py`, which removes it from the defect
  signal honestly; or (b) it is a genuine gap ⇒ the probe should decode with a declared
  fallback rather than failing the read.
- ⛔ **Do not simply drop the row.** Whichever way it goes, the manifest must record the cause;
  a silently removed row is the `.3.19`-class defect (a number that improves because the
  instrument stopped looking).
- **First step:** count the population, not the instance — sweep the whole corpus for files
  that fail to decode as UTF-8, so the leaf is sized before it is scoped. One visible row is
  the witness, not the construct (`.3.16`'s lesson).

##### `.3.23` — the `enum [N:M] { … }` family is an ADJUDICATOR hole, not a parser gap: PGEN is RIGHT to reject all 9 rows (diagnosed 2026-08-09, **DONE** 2026-08-09)

- **Status: `done`** (`PGEN-SV-CORPUS-GRAD-0044`) — all 9 pinned `must_reject`, LRM-grounded
  and per-file justified; **ZERO parser bytes**, and the construct is still REJECTED.
  ⛔ Accepting it would have been an over-acceptance defect
  ([[feedback_sv_strict_lrm_compliance_default]]). Same class as `.3.15`/`.3.16`. Evidence in
  `docs/tasks/artifacts/sv_corpus_grad/enum_base_range/`: `sweep.py` (re-runnable) with the
  before/after pair `sweep.txt` → `sweep_after.txt`, the six-case reproducer, and
  `verify_pins.py` → `verify_pins.txt` (the per-file justification, with controls).
- **HOW IT WAS PICKED — and why the frontier's "NOT the bucketer, use a token-level tell"
  rule earned its keep twice over.** The coarse family bucketer files this under
  *"enum base range (ch6)", 9 rows*. The stuck-signature clusters split the SAME construct
  across **two** signatures — `{ ID =` (7 rows) and `{ ID ,` (2 rows) — differing only in
  whether the enum's first member carries an `= value`. ⇒ **a leaf cut from the top cluster
  alone would have seen 7 and silently missed 2**, and `sweep.py` therefore keys on the
  CONSTRUCT, not on either view. Sized before scoped, per `.3.16`.
- **THE CONSTRUCT:** `typedef enum [2:0] { A, B } e_t;` — a packed dimension with **no base
  type**.
- **ROOT CAUSE (WHY + WHERE) — the parser is correct; the EXPECTED VERDICT is wrong.**
  The six-case reproducer isolates it to exactly one shape (`--profile sv_2017`, release
  probe): `enum [2:0] {…}` **REJECT** at `furthest_position=30`, while `enum {…}`,
  `enum int {…}`, `enum logic [2:0] {…}`, `enum bit signed [2:0] {…}` and
  `enum my_t [2:0] {…}` all **ACCEPT**. So every LRM-expressible base parses and the single
  rejection is the one Annex A cannot derive — a precise strictness boundary, not a hole in
  enum support.
- **THE LRM SAYS REJECT, on two independent surfaces that AGREE** (unlike `.3.19`, where the
  standard contradicted itself — the discriminator is worth stating, because it is what
  decides whether the parser or the adjudicator moves):
  1. **Annex A A.2.2.1** —
     `enum_base_type ::= integer_atom_type [signing] | integer_vector_type [signing]
     [packed_dimension] | type_identifier [packed_dimension]`. A `packed_dimension` is
     reachable ONLY behind a type; a bare `[2:0]` has no derivation.
  2. **Clause 6.19 prose** — *"In the absence of a data type declaration, the default data
     type shall be **int**. **Any other data type used with enumerated types shall require an
     explicit data type declaration.**"* A `[2:0]` dimension IS another data type (a 3-bit
     vector, not `int`), so it *requires* the explicit declaration it is missing. Every
     dimension-bearing example in 6.19 writes one: `enum bit [1:0] {IDLE, …}`.
  3. Measured, not recalled: **zero** occurrences of a literal base-less `enum [` anywhere in
     the 1800-2017 or 1800-2023 text (`grep -rE "enum[[:space:]]*\[[0-9A-Za-z_$]"` → empty;
     the many `enum [ enum_base_type ]` hits are BNF optionality brackets, not dimensions).
- **THE POPULATION IS EXACTLY 9, AND THERE IS NOTHING HIDDEN** (`sweep.txt`): scanning all
  **16 336** manifest rows for the construct finds **9** `unexplained_rejects_valid` — the
  entire actionable set, every one expected `must_accept` — plus 2 already-deferred
  `explained_svpp_macro_use`. All 9 are stuck at the `enum [` itself, confirmed per row
  against the cluster table's `furthest_position`, so none is being reclassified on a
  coincidence of containing the token elsewhere. Eight are `verilator/test_regress`, one is
  `sv2v/test/core` — a vendor-extension cluster, which is what the evidence predicts.
- **FIX AS LANDED — adjudicator only:** all 9 pinned `must_reject` in `EXTRA_PINNED`
  (`stimuli/sv/adjudicate_external_corpus.py`), each basis naming the construct, its line, and
  the A.2.2.1 + 6.19 ground.
- ⭐ **THE OPEN CHOICE, DECIDED: `must_reject`, NOT an `out_of_scope_with_cause:vendor_extension`
  deferral — and decided on the taxonomy's definitions, as the diagnosis demanded.**
  `out_of_scope_with_cause` means *owned by ANOTHER LANE*, and its rows adjudicate to
  `deferred:` — no verdict, no claim. There is no vendor-dialect lane to own these, so the
  deferral would have removed the rows from the proof surface while asserting nothing.
  `must_reject` turns today's REJECT into a `match`: the parser is claimed **correct with a
  cite**, and CI re-verifies that claim on every run. It is the strictly stronger of the two,
  and it is the ruling `.3.14a`/`.3.15`/`.3.16` already made on the same spec-outranks-tool
  shape.
  - ⛔ **The taxonomy's own docstring was the obstacle, and it was WRONG, so it was fixed in
    the same edit.** It read `must_reject = parse/lexical-level intentional invalidity` —
    but ~30 existing pins are text a *vendor tolerates* and the standard cannot derive, which
    is not "intentional" by any reading. The operative test has always been **the missing
    derivation**, never the upstream author's intent; the docstring now says so. A definition
    that contradicts 30 uses of itself is a hole waiting to be argued from.
- ⛔ **PER-FILE JUSTIFICATION WAS MANDATORY, and this is the trap.** A file can reject for more
  than one reason. Reclassifying its expected verdict on the enum ground would then MASK a
  second, real defect behind a correct-looking pin. **`verify_pins.py` is the instrument
  built for it** (`verify_pins.txt`): per row it locates every base-less `enum […] {` header
  in BYTES with comments/strings blanked, runs the real release probe, and requires
  `furthest_position` to land **inside the first such header** — REFUSING the pin on ACCEPT,
  on stuck-EARLIER, or on stuck-LATER. Measured: **9/9 AT-ENUM**, every stuck point exactly
  one byte before the `{`.
  - ⭐ **It refuses rather than guesses** ([[feedback_instrument_needs_ground_truth]]): a
    POSITIVE control (the tracked reproducer must classify AT-ENUM), a NEGATIVE control (the
    same reproducer with a syntax error *planted before* the enum must classify EARLIER — the
    leg that proves the differ can actually see the masking case, rather than rubber-stamping
    every row), and a second NEGATIVE (a legal `enum logic [2:0]` must yield ZERO construct
    matches). All three pass; a miss aborts before any row verdict is printed.
- ⚠️ **HONEST BOUND, carried in the instrument itself:** a pinned row can no longer testify
  about anything AFTER its enum header. That is inherent — the text is LRM-underivable, so the
  file can never be `must_accept` — but it is a real loss of corpus reach, not a free win.
- **MEASURED EFFECT (matches the prediction exactly):** `unexplained_rejects_valid`
  **296 → 287**, `match` **5780 → 5789**, and a full manifest diff shows **18 changed lines =
  the 9 rows and nothing else** — no other row moved, not even a basis string.
  `unexplained_accepts_invalid` stays **21**, so no over-acceptance was introduced. The v2005
  lane manifest is byte-identical (none of the 9 is in it).
  ⚠️ That is a *drop in the defect signal achieved by correcting an expectation* — legitimate
  here because the LRM is the oracle, but it is exactly the shape of
  [[a-rising-pass-rate-is-not-evidence-of-correctness]] and is presented as an adjudication
  correction, never as parser progress. **Parser yield: 0, by construction.**
- **SIDE-EFFECT, ROUTED AND RE-MEASURED, NOT LEFT TO ROT:** 4 of the 9 sat inside `.3.24`'s
  71-row population, so that tracked number went stale the instant the pins landed. It is
  re-derived to **67 of 287** (verilator 55, sv2v 11, sv-tests 1) by
  `advertised_invalidity_recount.py` → `.txt`, whose ground-truth leg **reproduces `.3.24`'s
  recorded 71/59/11/1 on the pre-pin vintage** before publishing anything. Two candidate
  readings of `_bad` were tried and rejected first (a stem *suffix* yields 64, not 71) — so
  the predicate is now pinned in code rather than re-guessed. ⛔ The baseline revision is
  **searched**, never defaulted to `HEAD`: a fixed default would silently invert the moment
  these pins committed — the `.3.21` stale-vintage trap, refused by construction.
- **INSTRUMENT DEFECT FOUND AND FIXED WHILE RE-RUNNING IT:** `sweep.py` printed its
  *"every one of them is expected `must_accept`, which is the mis-adjudication"* conclusion
  **unconditionally**, so the post-fix re-run asserted a finding that no longer existed. It
  now reports the state it measured. A diagnosis instrument that cannot say "resolved" is a
  standing source of false live findings.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `sweep.py` → `sweep.txt`: 9 `divergence:unexplained_rejects_valid`
      rows over the whole 16 336-row manifest carry a base-less `enum [`, every one expected
      `must_accept`; the coarse bucketer filed them as *"enum base range (ch6)", 9 rows* while
      the stuck-signature clusters split the SAME construct across `{ ID =` (7) and `{ ID ,` (2).
- [x] **ROOT CAUSE (WHY + WHERE)** — the EXPECTED VERDICT is wrong, not the parser.
      `verify_pins.py` → `verify_pins.txt`, real release probe, `--profile sv_2017`: all 9 reject
      with `furthest_position=` inside their first base-less enum header (e.g.
      `t_enum.v` `furthest_position=648`, header span 638..649, line 39, `enum [2:0] {`) — WHERE =
      the `enum […]` header itself, nowhere earlier. WHY = IEEE 1800-2017 A.2.2.1 reaches
      `packed_dimension` only behind a type, and 6.19 requires an explicit data type declaration
      for any non-`int` enum base; the six-case matrix in `sweep.txt` shows the other five base
      shapes all ACCEPT, so the boundary is exact.
- [x] **FIX** — declarative tier, adjudicator only: 9 `EXTRA_PINNED` entries + the taxonomy
      docstring correction. ⛔ No grammar/engine tier is admissible — a grammar "fix" here would
      be over-acceptance ([[feedback_sv_strict_lrm_compliance_default]]).
- [x] **ADDRESSED (verified)** — re-runnable oracle `python3 stimuli/sv/adjudicate_external_corpus.py`:
      `unexplained_rejects_valid` **296 → 287**, `match` **5780 → 5789**; the same instrument
      re-run (`sweep_after.txt`) reports **9 match / 0 actionable**; `verify_pins.py` exits 0 with
      **9/9 AT-ENUM** and all three ground-truth controls PASS.
- [x] **NO REGRESSION** — ZERO parser bytes staged (no `grammars/`, no `rust/src/`, no
      `generated/`), so no parser behaviour can move; the construct is still REJECTED. Full
      manifest diff = **18 lines, i.e. exactly the 9 rows** and nothing else;
      `unexplained_accepts_invalid` unchanged at **21** (no over-acceptance);
      `adjudication_manifest_v2005.tsv` byte-identical; `bash scripts/check_doctrines.sh` →
      **ALL 17 doctrines PASS**.
- [x] **LOCKSTEP** — the burn-down worklist regenerated from the new manifest
      (`rejects_valid_clusters.{tsv,md}` 296→287 rows, `rejects_valid_families.{tsv,md}` 8→7
      families, the `.3.21` reconciliation guard confirming `287 rows ==` the manifest — since
      re-run to 284 by `.3.24`); `.3.24`
      re-sized 71→67 with its own instrument; `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`,
      `docs/TASK_TREE.md` updated. Book/contract/ledger/schema/release: **N/A — no user-visible
      parser behaviour, no released-parser surface and no schema field changed** (the parser is
      byte-identical); the DONE-BAR register is unchanged for the same reason.

##### `.3.24` — SIZE THE ADJUDICATOR-HOLE SHARE OF THE REMAINING WORKLIST — **ANSWERED: essentially ZERO, and the answer says keep cutting construct leaves** (routed by `.3.23`; **DONE** 2026-08-09)

- **Status: `done`** (`PGEN-SV-CORPUS-GRAD-0045`). ⛔ **ZERO parser bytes.** Deliverable was a
  NUMBER, and the number is decision-changing in the direction nobody predicted. ⚠️ **Potentially scope-changing for the SV release
  lane, which is why it is a leaf and not a note:** if a material share of the remaining 296
  `unexplained_rejects_valid` rows are adjudicator work rather than parser work, then the
  distance to axis-2 green is shorter than the raw number says — and, more importantly,
  burning them down as *parser* defects would inject over-acceptance
  ([[feedback_sv_strict_lrm_compliance_default]]).
- **MEASURED (2026-08-09):** of the **296** rows, **71** lived in files whose path advertises
  intentional invalidity — a `_bad` basename, a `test/error/` directory, or an `_ILLEGAL`
  suffix. Split: **verilator 59, sv2v 11, sv-tests 1**. Every one was expected `must_accept`.
- **RE-MEASURED AT HEAD: `64` of `284`** (verilator 52, sv2v 11, sv-tests 1) — 71/296 before
  `.3.23`, 67/287 after it, 64/284 after `.3.24`'s own three pins. `.3.23` pinned 4 of the 71 (`t_enum_bad_value.v`, `t_enum_bad_wrap.v`,
  `t_enum_type_methods_bad.v`, `t_enum_type_nomethod_bad.v`). ⭐ The number is now backed by a
  re-runnable instrument instead of being a bare figure —
  `docs/tasks/artifacts/sv_corpus_grad/enum_base_range/advertised_invalidity_recount.py`,
  which pins the predicate in code and REFUSES unless it still reproduces the 71/59/11/1
  baseline on a pre-`.3.23` tracked vintage. **Re-run it before quoting this number**; two
  plausible readings of the `_bad` rule disagree with the recorded one (a stem *suffix* yields
  64), which is precisely why it is no longer left to be re-derived by hand.
- ⛔⛔ **DO NOT TREAT THE FILENAME AS AN ORACLE. That is the whole difficulty of this leaf, and
  the naive reading is wrong.** Verilator's `_bad` suffix means *this test expects an error*,
  and the overwhelming majority of those errors are **elaboration/semantic** — width
  mismatches, unresolved names, type errors — all of which are perfectly **parseable**. So
  `must_accept` at PARSE level is the correct default for that population, and the adjudicator
  is not obviously wrong. The finding is that the class has never been AUDITED, not that it is
  mis-classified.
- ⭐ **`.3.23` is the worked example that proves the subtlety, in both directions at once.**
  `t_enum_bad_value.v` and `t_enum_bad_wrap.v` were in this 71 (and are two of the 4 `.3.23`
  removed from it — so this example is now *worked*, not hypothetical). Their `_bad` label refers to an
  enum **value** being out of range — an elaboration error, so `must_accept` at parse level is
  right for the reason the name gives. But they nonetheless fail to PARSE, for a completely
  unrelated reason (`enum [N:M]`, a construct IEEE 1800 cannot derive), and *that* reason does
  make the expected verdict wrong. ⇒ **the name explains a different defect from the one the
  parser hits.** Any audit that reasons from the filename alone will get both of these
  backwards.
- **THE ONLY SOUND METHOD, therefore:** adjudicate from the suites' own driver metadata (which
  error STAGE each test expects — the machinery `.8b.1`/`.8b.2`/`.8b.3` already built for
  Surelog golden logs, sv2v stage classification and ivtest descriptors), cross-checked against
  the row's actual `furthest_position` locus. A row is reclassifiable only when the metadata
  says the expected error is LEXICAL/SYNTACTIC **and** the parser's stuck point is the
  construct that metadata names.
- **DELIVERABLE:** a number, not a fix — *how many of the 296 are adjudicator work?* — plus the
  per-row evidence for whichever subset is reclassified. Sizing this changes how the rest of
  the `.3` burn-down should be planned, so it is worth doing before the next several
  construct leaves rather than after.
- ⚠️ **And the reverse risk is the one to hold on to:** every row wrongly moved OUT of
  `unexplained_rejects_valid` is a real parser defect made invisible. The class must shrink by
  evidence, never by plausibility ([[a-rising-pass-rate-is-not-evidence-of-correctness]]).

#### THE ANSWER (measured 2026-08-09) — `stage_claim_census.py` → `stage_claim_census.txt`

- ⭐ **ADJUDICATOR WORK SIZEABLE FROM METADATA: 0 of 284 (0.0 %). PARSER WORK BY UPSTREAM'S OWN
  TESTIMONY: 270 of 284 (95.1 %).** The remaining 14 (4.9 %) are undecidable from metadata —
  verilator `%Error-UNSUPPORTED` / internal-error rows, which are testimony about the TOOL and
  support neither verdict, so they are broken out rather than quietly folded into "post-parse".
  Split of the 270: **199 UPSTREAM_SAYS_VALID** (a driver expecting success, a clean Surelog
  golden, an ispras `TYPE: POSITIVE`, an sv-tests positive, an sv2v conversion input, an ivtest
  `normal`) and **71 POST_PARSE_CLAIM** (upstream names elaboration / lint / type / name
  resolution). ⇒ **the burn-down is parser work; keep cutting construct leaves.**
- ⛔ **THE FILENAME HYPOTHESIS IS REFUTED, which is the point of having measured it.** The leaf
  opened on 71 rows whose path advertises invalidity; the census says none of them is
  reclassifiable *from that signal*, and the `_bad` population is now **64 of 284** by its own
  re-runnable instrument. Had the burn-down trusted the name, it would have injected
  over-acceptance on ~64 files ([[feedback_sv_strict_lrm_compliance_default]]).

#### ⭐⭐ BUT THE LEG THAT FOUND THE DEFECT IS THE OTHER ONE — and 3 rows moved

- **`SYNTAX_ERR_RE` was the whole stage test for verilator, and verilator's LEXER never says
  "syntax error".** `VerilatorIndex` decided *"does this `fails=True` test intend a parse-level
  failure?"* by grepping the golden for that one literal. Enumerating verilator's real message
  set across **all 1 427 tracked goldens** split ONE construct family down the middle:

  | file | construct | golden | verdict |
  |---|---|---|---|
  | `t_parse_eof_str_bad.v` | unterminated `"` string | ALSO says `syntax error` | `must_reject` ✔ |
  | `t_parse_eof_qqq_bad.v` | unterminated triple-quoted string | only `EOF in unterminated ... string` | `must_accept` ✘ |
  | `t_parse_eof_attr_bad.v` | unterminated `(*` | only `EOF in (*` | `must_accept` ✘ |
  | `t_fuzz_eof_bad.v` | both at once | ALSO says `syntax error` | `must_reject` ✔ |

  Plus `t_lint_vcmarker_bad.v` — a file containing literal `<<<<<<< HEAD` merge-conflict markers,
  filed as *"a post-parse tool error (lint/elab/unsupported) — syntax valid"*. **Four files, one
  lexical class, two verdicts, separated by nothing but whether the upstream message happened to
  contain two particular words.** That is `.3.15`'s *"the pin table already contradicts itself"*
  one level down: in the HEURISTIC rather than in the pins — and a heuristic hole re-opens on
  every re-vendoring, which a pin does not.
- ⛔⛔ **A PREFIX RULE WOULD HAVE BEEN WRONG, and the enumeration is what showed it.** The
  `EOF in …` family is NOT uniformly parse-stage: `EOF in define argument list`,
  `Unterminated ( in define formal arguments.` and `EOF in unterminated preprocessor expression`
  belong to the **preprocessor (svpp lane)**, and `Unterminated /* comment inside -f file.` is
  about a **`-f` command file**, not source text at all. Matching `EOF in ` would have swept all
  four into the parse verdict. So the fix is an ENUMERATED allowlist — the same shape `.3.14b`
  was forced into for compiler directives, for the same reason.
- **PER-ROW JUSTIFICATION, to `.3.23`'s standard:** each of the 3 was probed and its stuck point
  matches the construct its golden names — `t_lint_vcmarker_bad` `furthest_position=212`, the
  parser having consumed `module t;` and stopped at the `<<<<<<<` (verilator: `:9:1`);
  `t_parse_eof_qqq_bad` `=202`, stopped at the triple-quoted opener (verilator `:7:1`);
  `t_parse_eof_attr_bad` `=210`, having consumed `(* attr` and hit EOF needing `*)` (verilator
  `:7:1`). LRM grounds: clause 5.9 (a string literal sits on ONE line and must close), A.9.1
  (`attribute_instance` requires its `*)`), and — for the conflict markers — no lexical
  derivation of `<<<<<<<` exists at all.
- **MEASURED:** `unexplained_rejects_valid` **287 → 284**, `match` **5789 → 5792**; exactly those
  3 rows moved and **no row moved in the reverse direction**, so no new
  `unexplained_accepts_invalid` was created (still **21**); the v2005 lane manifest is
  byte-identical. ⚠️ An ADJUDICATION CORRECTION, not burn-down yield.

#### ⛔ THE HONEST BOUND ON THE NUMBER — stated because it is load-bearing

- **A basis census inherits the adjudicator's blind spot.** It classifies the RECORDED derivation,
  so a row derived WRONG is classified confidently *as what the adjudicator believed*. The 3 rows
  above were invisible to it and were found by the vocabulary leg. ⇒ **"0 of 284" means "nothing
  left that the recorded reasoning can surface", never "nothing left".**
- **And no metadata census can ever see the `.3.23` class**, because upstream is a TOOL and the
  oracle is the STANDARD: a row can carry impeccable `UPSTREAM_SAYS_VALID` testimony and still be
  LRM-underivable. Those are found only by reading the construct at the stuck point.
- ⭐ **Both bounds are now GUARDED, not just documented.** The census refuses on an unrecognized
  `basis` spelling instead of bucketing it as "other", and LEG 1 refuses if verilator emits a
  lexical spelling nobody has ruled on — so the next re-vendoring that grows a message stops the
  instrument rather than silently shrinking the number. Both refusal paths are exercised: LEG 1
  runs a planted unruled spelling on every invocation, and dropping a real spelling from the
  ruled set was verified to abort ([[a-check-whose-inputs-all-pass-has-not-been-tested]]).

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — the leaf's own premise: 71 of 296 rows sat in paths advertising
      invalidity with no audit ever run, and a wrong call in either direction is expensive.
      Re-derived at HEAD by `advertised_invalidity_recount.py` (64 of 284 now).
- [x] **ROOT CAUSE (WHY + WHERE)** — WHERE = `VerilatorIndex.__init__`, which built
      `syntax_out_stems` from `SYNTAX_ERR_RE = re.compile(r"syntax error")` alone. WHY = verilator's
      LEXER errors never contain that string, so an entire lexical class was invisible to the stage
      test; enumerating all 1 427 goldens shows 10 distinct lexical spellings of which 6 are
      parse-stage and only 2 co-occur with `syntax error`. Evidence: `stage_claim_census.txt`
      LEG 1 table + the four-file contradiction above + per-row `furthest_position=212/202/210`
      probe output matching each golden's own line:col.
- [x] **FIX** — declarative tier, adjudicator only: `VERILATOR_PARSE_STAGE_RE`, an ENUMERATED
      allowlist (6 spellings in, 4 explicitly out with their owning lane named), consumed by
      `VerilatorIndex`. No grammar or engine change is admissible — the parser is right to reject
      all three files.
- [x] **ADDRESSED (verified)** — re-runnable oracle `python3 stimuli/sv/adjudicate_external_corpus.py`:
      `unexplained_rejects_valid` **287 → 284**, `match` **5789 → 5792**;
      `stage_claim_census.py` exits 0 with LEG 1 fully adjudicated and 0 unrecognized bases.
- [x] **NO REGRESSION** — ZERO parser bytes staged (no `grammars/`, `rust/src/`, `generated/`).
      Manifest diff = exactly the 3 rows, **no row moved in the reverse direction**,
      `unexplained_accepts_invalid` unchanged at **21**, `adjudication_manifest_v2005.tsv`
      byte-identical, `bash scripts/check_doctrines.sh` → ALL 17 PASS.
- [x] **LOCKSTEP** — worklist regenerated (`rejects_valid_clusters` 287→284 rows / 180→177
      signatures; `rejects_valid_families` 284 rows / 7 families; `.3.21` reconciliation guard
      green); `advertised_invalidity_recount` re-run (67→64); `CHANGES.md`,
      `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `docs/TASK_TREE.md`, the book's adjudication chapter.
      Book/contract/ledger/schema/release/register: **N/A — the parser is byte-identical**, so no
      user-visible parser surface moved.

##### `.3.25` — the queue-slice `q[a:$]` bound REJECTS: `part_select_range` routes through `constant_range`, and `constant_expression` correctly has no `$` primary (IEEE 1800-2017 A.8.4 **footnote 42** + §7.10.1 + §7.10.4 — ⭐ the OTHER half of the footnote whose `q[$]` half `SV-EXH-PROOF.3.3.4.b.6.2.13` already fixed)

- **Status: `done`** (`PGEN-SV-CORPUS-GRAD-0046`). Cut from the live 284-row worklist by a
  **token-level tell**, per the `.3.23`/`.3.24` instruction to trust neither the family bucketer
  nor a single signature: the tell is `$` **inside a bracketed range**, which the structural
  bucketer scatters across the `OTHER`, `interface/modport (ch25)` and `SVA (ch16)` families and
  which no one cluster signature collects (the 11 rows carrying it hold **6 distinct**
  signatures — `] ; /` ×4, `@ ( ID` ×2, `ID [ ID` ×2, `] } ;`, `] ; ID`).

#### THE TELL, AND WHAT IT IS *NOT* (the tell is not the family — measured)

- 11 of 284 rows carry `$` inside `[...]`, across **3 suites** (Surelog 3, ispras-sv-tests 5,
  sv-tests 3). ⛔ **They are NOT one defect, and the minimal-repro matrix is what proved it** —
  4 of the 11 parse fine in isolation today and are stuck on something else entirely:

  | shape | minimal repro | verdict today |
  |---|---|---|
  | queue slice, `$` hi bound | `q = q[1:$]` | **REJECT** ← this leaf, 7 rows |
  | queue back-index | `q[$]`, `q[$-1]` | PASS (`.6.2.13`) |
  | queue dimension | `int q[$];`, `protected int q[$];` | PASS |
  | SVA consecutive repetition | `a[*1:$] ##1 b` | PASS |
  | SVA cycle-delay range | `a ##[1:$] b` | PASS |
  | bare `$` primary | `x = $;` | PASS |

  ⇒ the leaf owns exactly the **7** `expr[lo:$]` rows; the other 4 stay on the worklist under
  their own causes. Sizing the tell before cutting is what `.3.23` cost the tree by skipping.

#### ROOT CAUSE — the WHERE is one production, and the WHY is that Annex A contradicts itself

- **WHERE.** `grammars/systemverilog.ebnf:4179` `part_select_range := constant_range |
  indexed_range`, and `:1590` `constant_range := constant_expression colon constant_expression`.
  `constant_expression` bottoms out in `constant_primary` (`:1516`), which — faithfully to
  IEEE 1800-2017 A.8.4 — has **no `$` alternative**. The bare-`$` primary lives only on the
  non-constant side (`primary_dollar_sv_only`, `:4315`, `@profiles: ["sv_2017","sv_2023"]`).
- **WHY the parser is not simply right to reject.** Three normative sources say `$` is legal here
  and only the production tree says otherwise:
  - **A.8.4 footnote 42** (`docs/systemverilog/2017/md/section-41-data-read-api.md:3564`) —
    *"The `$` primary shall be legal only in **a select for a queue variable**, in an
    open_value_range, covergroup_value_range, integer_covergroup_expression, or as an entire
    sequence_actual_arg or property_actual_arg."* A select's range is `part_select_range`, so the
    footnote **presupposes** a derivation the production tree does not provide.
  - **§7.10.1** — *"In a queue slice expression such as Q[a:b], the slice bounds may be arbitrary
    integral expressions and, in particular, **are not required to be constant expressions**."*
  - **§7.10.4** — the normative example block is written in this syntax: `q = q[1:$];`,
    `q = q[0:$-1];`, `q = { q[0:pos-1], e, q[pos:$] };`, `q = q[2:$];`, `q = q[1:$-1];`.
- ⭐ **AND ANNEX A ITSELF SHOWS THE INTENT, one line down.** The sibling alternative is
  `indexed_range ::= **expression** +: constant_expression` while
  `constant_indexed_range ::= constant_expression +: constant_expression` — i.e. Annex A *does*
  distinguish the select form from the constant-select form by exactly the `expression` vs
  `constant_expression` axis, and then `part_select_range` re-uses `constant_range` anyway. The
  contradiction is a transcription wart in Annex A, not a rule. ⇒ this is the **`.3.19` class**
  (*the contradiction is INSIDE the standard*), **not** the `.3.23` class (*nothing in the LRM
  licenses it*), and the two are separated here by a named footnote, not by plausibility.
- ⭐⭐ **THE SIBLING HALF OF THE SAME FOOTNOTE IS ALREADY FIXED IN THIS GRAMMAR, WITH THIS
  REASONING.** `bit_select_expression` (`:848`) carries a dedicated `kw_dollar` alternative added
  by `SV-EXH-PROOF.3.3.4.b.6.2.13` for `q[$]` — the *bit-select* half of footnote 42. This leaf is
  the *part-select* half. The house idiom for "`$` as a range bound" is likewise already in the
  file three times (`:1744`/`:1745` covergroup `dollar_lo`/`dollar_hi`, `:1794`
  `cycle_delay_const_range_expression`), so the fix adds no new vocabulary.

#### ⛔ WHY NOT THE WIDER FIX — the over-acceptance was priced, not assumed

- The tempting fix is `part_select_range := expression colon expression` (Annex A's evident
  intent). **Refused**: measured, `expression` admits bound forms `constant_expression` does not,
  and none is LRM-legal in a part-select — `v[a++ : b]` and `v[a inside {1,2} : 0]` both go
  REJECT→PASS under it. That is textbook over-acceptance
  ([[feedback_sv_strict_lrm_compliance_default]] — SV is strict-LRM **by default**).
- **And the constness axis is already, unavoidably, not syntactic** — measured: `v[a:b]`,
  `v[a+1:b-1]` and `v[f(a):b]` all PASS **today**, because a parser cannot tell a parameter from a
  variable. So "keep `constant_expression`" buys no strictness it does not already have, and the
  honest minimum is to add **`$` and nothing else**.
- **Fix-hierarchy tier: grammar (tier 2).** Tier 1 (declarative annotation) does not apply — the
  defect is a missing *production*, not a policy knob. No engine change.

#### THE FIX (strictly additive, profile-gated)

```ebnf
part_select_range := constant_range           -> {kind: "range",         body: $1}
                   | queue_slice_range_sv_only -> {kind: "queue_slice",  body: $1}   # NEW
                   | indexed_range            -> {kind: "indexed_range", body: $1}
```
- `queue_slice_range_sv_only` admits `$` on either bound (`lo:$` / `$:hi` / `$:$`), with the
  `$` bound itself `kw_dollar ( ( plus | minus ) constant_expression )?` so the LRM's own
  `q[0:$-1]` / `q[1:$-1]` examples derive. Bounds without a `$` are untouched — the
  `constant_range` alternative still owns them, so **no existing match changes**.
- ⛔ **`@profiles: ["sv_2017", "sv_2023"]`, via the documented `_sv_only` lift idiom** (the same
  one `primary_dollar_sv_only` uses): IEEE 1364-2005 has neither queues nor a `$` primary, so an
  ungated rule would hand the `verilog_2005` lane a new accepts-invalid row. The v2005 manifest
  being byte-identical afterwards is the check that this held.

#### ⭐ MANIFEST RECONCILIATION — exactly 7 rows moved, and every one is this leaf's

⛔ **Every moved row was diffed** (`git show HEAD:…/adjudication_manifest.tsv` vs the regenerated
one) rather than inferred from the totals. Final: `match` **5792 → 5799**,
`unexplained_rejects_valid` **284 → 277**, `unexplained_accepts_invalid` **21 → 21**,
`explained` **1463 → 1463** (unchanged), `deferred` **8776 → 8776** (unchanged),
corpus `timeout` **4 → 4** (unchanged). **7 rows changed class; nothing else moved.**

**7 rows `unexplained_rejects_valid → match`, all `fail → pass`** — the burn-down yield:

| suite | file |
|---|---|
| ispras-sv-tests | `ieee-1800-2012/06/06.24.03_03.sv`, `07/07.10.04_02.sv`, `11/11.04.14.04_01.sv` |
| sv-tests | `chapter-7/queues/insert_assign.sv`, `pop_front_assign.sv`, **`pop_back_assing.sv`** |
| verilator | **`test_regress/t_queue_back.v`** |

⭐⭐ **The two in bold were NOT in the leaf's 11-row candidate set — the token-level tell
under-counted, exactly as `.3.23` warned it would.** The tell regex required `$` immediately before
the `]`, so `q = q[0:$-1];` (`pop_back_assing.sv:25`, the LRM's own `pop_back` example) did not
match it. ⇒ **the tell is a cut heuristic, never a census** — the same defect shape `.3.23`
recorded for the bucketer and for single signatures, now recorded for the token tell as well. The
authoritative count is always the manifest diff.

#### ⛔⛔ AND THE FIRST RE-RUN WAS WRONG — the leaf ran the corpus on DIFFERENT PARAMETERS than the artifact it was diffing against

**This is recorded, not quietly re-run, because the mistake is the finding** (routed to `.3.27`).
The first characterization re-run invoked `stimuli/run_external_corpus.sh sv` on its **defaults**
and produced **6 extra moved rows** (`deferred:chained_only → divergence:explained_timeout`,
timeouts `4 → 10`, one of them `pass → timeout`). Those 6 were investigated as a suspected
performance regression: the grammar was reverted to `HEAD`, the parser regenerated, the debug
probe rebuilt, and the six files timed on BOTH parsers serially — **20/33/32/30/127/20 s at
baseline vs 21/35/32/31/128/20 s with the fix**, i.e. +0…+2 s, and `mm_ram.sv` contains no
`[…:$]` site at all, so the new alternative cannot even execute in it. **No regression.**

⭐ **The actual cause was configuration drift, and the artifact's own provenance block said so.**
`characterization.md` records the parameters it was generated with:

```
Generated by `stimuli/run_external_corpus.sh sv 60 8 0` against `parseability_probe`.
| parse binary | `rust/target/release/parseability_probe` | …
```

— **60 s and the RELEASE probe**. The script's *defaults* are **20 s and the DEBUG probe**, and
`mm_ram.sv` is **12 s release vs 127 s debug**. So the bare invocation silently re-measured the
whole corpus under a ~10× slower binary at a 3× tighter deadline. Re-run at the recorded
parameters (`PGEN_PARSE_PROBE_BIN=…/release/parseability_probe … sv 60 8 0`), the timeout
population returns to **exactly 4**, `explained`/`deferred` return to their baseline values, and
the diff is the clean 7 rows above.

⚠️ **The lesson is not "read the header more carefully."** The provenance block existed and was
correct — it is the `FLOW-INTEGRITY` hand-off-provenance work paying off — but **nothing compares
it against the run that overwrites it**, so a mismatched re-run publishes over a tracked oracle
with no complaint. Had the 6 rows landed on `must_accept` files instead of `deferred` ones, they
would have left `unexplained_rejects_valid` and the burn-down number would have improved for
reasons having nothing to do with the parser
([[a-rising-pass-rate-is-not-evidence-of-correctness]]). Routed to `.3.27`.

#### ROUTING EVIDENCE — three findings sent out, each measured OUTSIDE the family first

- **→ `EBNF-FRONTEND-SILENT-TRUNCATION.4`** (frontend, not SV). Writing the offset as an inline
  `( plus -> {…} | minus -> {…} )` group made the EBNF frontend re-read the annotation payload as
  the element's **quantifier** (`--dump-gen-ast`:
  `Quantified{element: plus, quantifier: "kind: \"plus\""}`). **Reproduces outside SV** — a
  5-rule synthetic (`top := a ( b -> {kind: "bee"} | c ) d`) yields the identical IR, so it is
  the shared frontend, not this grammar. **Fails CLOSED**: codegen aborts `Unknown quantifier`
  on both the synthetic and the full SV grammar (exit 1), so no wrong parser can ship; the
  defect is that `--lint-grammar` passes it and the message names a quantifier nobody wrote.
  Worked around here by lifting the group to the named rule `queue_slice_dollar_offset`.
  ⭐ **UPGRADED 2026-08-10 on a director challenge — the construct is LEGAL by the project's own
  spec**, so this is a frontend ⟷ meta-grammar divergence rather than an author error:
  `grouped_expression → rule_expression → alternation`, and `alternation` admits a per-branch
  `->` before each `|` (`grammars/ebnf.ebnf:316/:124/:134`). Confirmed against the real
  meta-parser with the 1.9 envelope driver — arm 2 builds a proper `return_annotation` inside a
  `grouped` node with **0** `quantified` nodes, while arm 1 builds the bogus quantifier. Full
  evidence + the honest last-branch bound in that leaf.
- **→ `.3.26`** (SV, same family, so the reproduces-outside question does not arise). The
  `{}` / `'{}` empty-concat inversion; measured evidence in that leaf.
- **→ `.3.27`** (the corpus RUNNER, not SV). The runner's defaults (20 s / debug probe) differ
  from the parameters the tracked artifact records (60 s / release probe), and nothing compares
  them. **Reproduces outside SV by construction** — `stimuli/run_external_corpus.sh` is the shared
  family-parameterized runner the VHDL lane (`CORPUS-GRAD-ALL.2`) uses unchanged, so the remedy
  belongs in the runner. Evidence = this leaf's own two runs (defaults → 6 spurious rows,
  timeouts 4→10; recorded parameters → 0 spurious rows, timeouts 4→4) plus the reverted-grammar
  timing A/B proving the parser was never implicated.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `./rust/target/release/parseability_probe --parse systemverilog
      <f> --profile sv_2017` on the 7 corpus rows: all `REJECT`, e.g. sv-tests
      `queues/delete_assign.sv` `furthest_position=538`, ispras `07.10.04_01.sv`
      `furthest_position=559`. Minimal repro `module m; int q[$]; initial q = q[1:$]; endmodule`
      → `Parser did not consume full input at position 0 [furthest_position=51]`, position 51
      being the `]` immediately after the `$`.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHERE `grammars/systemverilog.ebnf:4179`
      `part_select_range := constant_range | indexed_range` → `:1590` `constant_range :=
      constant_expression colon constant_expression` → `constant_primary` (`:1516`) has no `$`.
      WHY = A.8.4 footnote 42 licenses the `$` primary *in a select for a queue variable* while
      A.8.4's own production tree cannot derive it. Isolated by the 12-case
      `--parse` matrix above, which exonerates every neighbouring `$` construct
      (`q[$]`, `q[$-1]`, `a[*1:$]`, `a ##[1:$]`, `x = $`, `int q[$]` all PASS) and convicts
      exactly `q[lo:$]`; `q[$:1]` REJECT independently proves `constant_expression` derives no `$`.
- [x] **FIX** — grammar tier: one new profile-gated alternative `queue_slice_range_sv_only` on
      `part_select_range`. No engine change, no new annotation. The wider
      `expression colon expression` form was measured and REFUSED (it takes `v[a++ : b]` and
      `v[a inside {1,2} : 0]` REJECT→PASS).
- [x] **ADDRESSED (verified)** — re-runnable oracle
      `./rust/target/release/parseability_probe --parse systemverilog <f> --profile sv_2017`:
      **5 of the 7** rows go **REJECT→PASS** (`insert_assign.sv`, `pop_front_assign.sv`,
      `07.10.04_02.sv`, `06.24.03_03.sv`, `11.04.14.04_01.sv`). ⚠️ **The other 2 are NOT fixed and
      are NOT claimed** — `delete_assign.sv` and `07.10.04_01.sv` still REJECT, but their
      `furthest_position` moved **DEEPER** (538→603 and 559→873), i.e. the `$` bound now parses
      and they stop at the *next* construct, `q = {};` — routed to `.3.26`, not folded in here
      ([[a-rising-pass-rate-is-not-evidence-of-correctness]] cuts both ways: a partial fix is
      reported partial). All 8 §7.10.4 normative forms parse: `q[1:$]`, `q[0:$-1]`, `q[pos:$]`,
      `q[pos+1:$]`, `q[1:$-1]`, `{ q[0:pos-1], 1, q[pos:$] }`, `q[$:1]`, `q[$:$]`.
      Adjudicator oracle `python3 stimuli/sv/adjudicate_external_corpus.py` (over a
      characterization re-run at the artifact's OWN recorded parameters — `sv 60 8 0`, release
      probe): `unexplained_rejects_valid` **284 → 277**, `match` **5792 → 5799**; the manifest
      diff is **exactly 7 rows**, all `fail → pass` (2 of them outside the leaf's candidate set —
      see the reconciliation above).
- [x] **NO REGRESSION** — `unexplained_accepts_invalid` **21 → 21** (no row moved the wrong way);
      `explained` **1463 → 1463** and `deferred` **8776 → 8776** UNCHANGED, corpus `timeout`
      **4 → 4**; `adjudication_manifest_v2005.tsv` and `results_v2005.tsv` **byte-identical** —
      the `@profiles` gate held, confirmed independently by `--dump-rule-profiles`
      (`verilog_2005` **1122 → 1122**, and **0** rules changed their satisfiable-profile set).
      Re-runnable oracle `make -C rust SHELL=/bin/bash sv_cert_recognized_union_gate`: ✅ GREEN at
      seeds 0/7/42 — canonical UNKNOWN **11** UNCHANGED, union UNKNOWN **0** UNCHANGED,
      residual `[]` UNCHANGED, `expected_proof` **6** UNCHANGED, `sample_parse_failures=0`, all
      three new rules WITNESSED. `make ast_shape_contract_gate` ✅ (18/18).
      `bash scripts/check_doctrines.sh` → ALL 17 PASS.
      ⛔ Performance A/B (grammar reverted to `HEAD`, parser regenerated, probe rebuilt, six
      heaviest corpus files timed serially on both): **+0…+2 s**, ≤6 % — noise.
- [x] **LOCKSTEP** — ⛔ **the cert contract is re-baselined IN THIS COMMIT**
      (`systemverilog_recognized_cert_union_contract.json`: `expected_total` 1355→1358,
      `expected_canonical_witness` 1338→1341, `expected_union_witness` 1349→1352 — the
      `CI-PARITY-GATE-ROT.22` tripwire; the gate was run BEFORE the re-baseline and named all 12
      unmet criteria itself). `verilog_2005_conformance_contract_v0.json` correctly does NOT move.
      Worklist regenerated; `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`,
      `docs/TASK_TREE.md`; the book's `grammar-wellformedness.md` gains *"When the standard's own
      productions contradict its own footnote"*; Knowledge-Map card
      `annex-a-footnotes-license-derivations-the-productions-cannot-derive` (74→78 facts).
      Release/schema/ledger: **N/A** — no published parser surface or AST shape changed
      (`ast_shape_contract` byte-identical), the change is purely additive acceptance.

##### `.3.26` — ⛔⛔ the empty unpacked array concatenation is wrong in BOTH directions at once: PGEN accepts `'{}` (which Annex A cannot derive) and rejects `{}` (which Annex A *is*) — routed by `.3.25`, 2026-08-10

> ⛔⛔ **RETRACTED IN PART by `.3.26d` (`PGEN-SV-CORPUS-GRAD-0192`) — read before quoting anything
> in this leaf, including its heading.** The `{}` half is correct and stands: PGEN did reject the
> A.8.1 form, and fixing that was pure under-acceptance repair worth 5 corpus rows. **The `'{}`
> half is FALSE.** This leaf's heading, its "wrong in BOTH directions" framing, its
> over-acceptance verdict, and the whole `THE ADJUDICATION` section below rest on the inference
> *"not derivable from Annex A ⇒ not legal SV"*, which is **invalid** and which this tree had
> already refuted twice (`.3.25` `q[a:$]` via A.8.4 footnote 42; `.3.19` `use #(...)` via clause
> 33.4.3). `'{ }` is **legal SystemVerilog**: §11.4.12 writes the construct's own delimiter pair as
> `'{ }`, Annex M/VPI names the operator `vpiAssignmentPatternOp 75 /* '{} assignment pattern */`,
> and **no LRM text declares it illegal** — measured exhaustively over both revisions in `.3.26b`
> and `.3.26c`. ⇒ the rule was wrong in **ONE** direction. `'{}` is not tolerance, not bucket-(a),
> and not the deferred strictness switch's first customer; it is simply correct, and since
> `.3.26c` it is modelled as an `assignment_pattern` with an empty element list. Director ruling +
> the standing rule (**"non-LRM" is a CITATION, never an inference**):
> [[feedback_sv_strict_lrm_compliance_default]] § BOUNDING RULING.

- **Status: `done` 2026-08-10 (`PGEN-SV-CORPUS-GRAD-0187`), grammar-only, ZERO Rust bytes.**
  ⚠️ **Superseded in part** — see the retraction banner above and leaves `.3.26b`/`.3.26c`/`.3.26d`.
  ⛔ **NOT fixed in `.3.25`** — it is a distinct defect with its own root
  cause, its own LRM citation, and a real dialect-tolerance question that must not be settled by
  a one-line literal flip. Surfaced because `.3.25`'s fix moved two rows' `furthest_position`
  DEEPER onto this construct.
- **THE FINDING.** IEEE 1800-2017 **A.8.1**:
  `empty_unpacked_array_concatenation35 ::= { }` — **bare braces, no apostrophe**
  (`docs/systemverilog/2017/md/section-41-data-read-api.md:2907`), with **footnote 35**: *"`{ }`
  shall denote an empty unpacked array concatenation, as described in 10.10, and shall not be
  used in any other [context]"* (`:3551`). §7.10 says the same in prose: *"The empty queue can be
  denoted by an empty unpacked array concatenation `{}`"*. And **no `assignment_pattern`
  alternative is empty** — all four require at least one `expression` (`:2292`–`:2295`) — so
  **`'{}` has no derivation anywhere in Annex A**.
- **PGEN today** (`grammars/systemverilog.ebnf:2229`):
  `empty_unpacked_array_concatenation := tick lbrace rbrace`. Measured with
  `parseability_probe --parse systemverilog … --profile sv_2017`:

  | form | Annex A | PGEN | verdict (as recorded 2026-08-10) | verdict (CORRECTED, `.3.26d`) |
  |---|---|---|---|---|
  | `q = {};` | **derivable** (A.8.1) | **REJECT** | under-acceptance | ✅ unchanged — under-acceptance |
  | `q = '{};` | **not derivable** | **PASS** | ~~over-acceptance~~ | ⛔ **NOT a defect** — legal SV per §11.4.12 + Annex M; Annex A is incomplete here, and accepting it is CORRECT |

- ⭐ **ROOT CAUSE IS A CITATION ERROR IN A PRIOR FIX, NOT A MISSING FIX.**
  `SV-EXH-PROOF.3.3.4.b.6.2.37.8` replaced a genuinely broken rule (`lbrace epsilon rbrace`,
  where `epsilon` was an undefined symbol, so the rule could never match) — that half was right.
  But it wrote the replacement literal as `'{ }` and cited *"§A.6.7"*; the production is at
  **A.8.1** and reads `{ }`. The fix traded a never-matching rule for a wrong-literal rule, and
  because its motivating input (`uvm_cache::get`'s `return '{};`) genuinely uses the apostrophe
  form, the error reproduced as a *success* and was never questioned.
- ⛔ **WHY THIS IS NOT A ONE-LINE FLIP — both populations are real, measured over
  `stimuli/sv/subs/`:** **42** files write `= '{}` and **56** write `= {};`. Deleting the `tick`
  arm would regress the 42 (including uvm_pkg, the `.37.8` motivator).
  ⛔ **[RETRACTED from here to the end of this bullet — `.3.26d`.** There was no choice to make:
  `'{}` is legal SV, so "accept both" was never a tolerance decision and there is no bucket-(a)
  row. The `42`/`56` counts are also superseded — re-measured unanchored as **49 tick / 91 bare**
  below.] So the leaf must decide
  between (a) LRM-only `{ }` — strict, regresses real-world code that every major tool accepts;
  (b) accept both, recording `'{}` as **deliberate dialect tolerance** and therefore a row the
  accepts-invalid triage must carry. ⇒ this is exactly the bucket-(a) evidence the **director's
  strictness-axis directive** (2026-07-25, this tree, §"STRICTNESS AXIS") asked to be collected
  before designing the switch — the first instance found where strict-LRM and ecosystem reality
  genuinely conflict. **Do not resolve it by plausibility.**
  ⭐ **The last sentence was right and was not followed** — the plausibility that resolved it was
  the unread premise, not the adjudication built on top.
- **First act when opened:** add `{}` (the LRM form) — that half is unambiguous and pure
  under-acceptance repair; ⛔ **[RETRACTED — `.3.26d`]** ~~then adjudicate the `'{}` arm against the
  strictness directive rather than silently keeping it.~~

##### `.3.26` — THE ADJUDICATION (2026-08-10, session #232): ⛔ NOT a director call — two recorded rulings already decide it, and they decide it in *opposite* directions that compose

> ⛔⛔ **THIS WHOLE SECTION IS RETRACTED (`.3.26d`).** It adjudicates a strictness question that
> **does not exist**, because its premise — *"`'{}` is over-acceptance"* — is false (banner at the
> top of `.3.26`). The two rulings it invokes are real and correctly quoted; they simply do not
> apply, since nothing needed tolerating. Kept verbatim as the record of the reasoning, and because
> its own failure mode is the lesson: **a well-sourced adjudication built on an unsourced premise
> is still unsourced.** The composition argument reads convincingly precisely because both cited
> rulings are genuine — which is what made the missing citation easy to miss.

The leaf was opened expecting a strictness-axis escalation. It is not one. **Both arms are settled
by decision records already on disk**, and the interesting part is that they pull opposite ways and
still compose into one answer.

| ruling | what it says here |
|---|---|
| `feedback_sv_strict_lrm_compliance_default` — *"So, we will stick to strict-LRM compliance then, good I prefer that"* (director, 2026-07-25) | strict-LRM **by default**; over-acceptance is a defect; the tolerance switch is **DEFERRED, not rejected** ⇒ `'{}` is over-acceptance and cannot simply be shrugged at |
| `feedback_uvm_is_valid_sv` — *"A pgen parser failure on UVM means **our parser has a defect**. The input is correct."* | uvm-core writes `return '{};` at `uvm_lru_cache.svh:206` and `:273` ⇒ **rejecting `'{}` would BE a defect** |

⭐ **They compose via the strictness directive's own vocabulary.** This tree's directive
(§ *STRICTNESS AXIS*, constraint 2) splits over-acceptance into *(a) genuine dialect tolerance the
ecosystem relies on* and *(b) plain over-acceptance bugs*, and rules that **"bucket (a) is what a
switch is FOR, bucket (b) should simply be fixed."** `'{}` is bucket (a) with the strongest possible
witness — the Accellera reference library itself. ⇒ **KEEP the arm, NAME it, and route it as the
switch's first customer**; do not delete it, and do not leave it undocumented either. Deleting it
would regress uvm-core and 48 other corpus files to satisfy a switch that does not exist yet.

⇒ the fix is **two arms, for two different reasons**, and the grammar comment says which is which so
a later reader cannot mistake the tolerated arm for a second LRM production.

- **`lbrace rbrace`** — the A.8.1 production, verbatim. Pure under-acceptance repair.
- **`tick lbrace rbrace`** — no derivation in Annex A; retained as recorded bucket-(a) tolerance.

⚠️ **AND A FINDING ABOUT THE EVIDENCE BASE ITSELF, routed to `LRM-GRAMMAR-FIDELITY`.** The
strictness directive's constraint 2 names the **accepts-invalid population (21 + 14 = 35 rows)** as
*"by construction, every place PGEN currently accepts what the standard forbids."* The claim is that
this population cannot see over-acceptance no suite probes negatively — because it is derived from
corpus rows whose *answer key* says `must_reject`, and a tolerance the whole ecosystem shares is
exactly the one nobody writes a negative test for. ⇒ the census would measure *over-acceptance the
corpus happens to probe negatively*, not over-acceptance. Same shape as the standing *"no cut
heuristic is a census"* lesson.

> ⛔ **ITS WITNESS IS RETRACTED (`.3.26d`).** This paragraph originally offered `'{}` as the worked
> counter-example. `'{}` is legal SV, so it is **not** over-acceptance and proves nothing here. The
> structural argument above may well be true, but as of `.3.26d` it has **zero witnesses** and is
> carried in `LRM-GRAMMAR-FIDELITY.1c` as an **unwitnessed hypothesis**, not a finding. ⭐ The
> instrument `.1c` asks for — an Annex-A-derived over-acceptance audit that never consults a corpus
> — is what would produce a real witness, and is now the only thing that can promote it.

⚠️ **The leaf's own population counts were a cut heuristic too, and under-counted — re-measured
here.** `.3.26`'s recorded `42` / `56` came from an `=`-anchored regex, which cannot see
`return '{};` — the very call site that motivated the original defective fix. Unanchored over
`stimuli/sv/subs` + `stimuli/sv/uvm`:

```
$ grep -rlE "'\{ *\}"        …   # tick form, any context  -> 49 files
$ grep -rlE "(^|[^'])\{ *\}" …   # bare form, any context  -> 91 files
```

**49 tick / 91 bare** (regex FILE counts, not a parse census — comments and string literals are not
excluded; stated so the number is not later quoted as exact). The direction is what matters and it
is unambiguous: the LRM form is the *more* used of the two, so the repair serves the larger
population as well as the standard.

##### `.3.26` — ⚠️ A SURPRISING CERT NUMBER, RUN DOWN RATHER THAN ACCEPTED (and it was benign)

Banked because the number arrived in the **flattering** direction, which is the direction a session
is least likely to question. The ad-hoc canonical cert command printed:

```
CERTIFICATE-COVERAGE: … total=1358 proof=17 witness=1341 UNKNOWN=0 fully_certified=true
```

against a contract pinning `expected_proof=6` / `expected_canonical_unknown=11`. **`UNKNOWN 11 → 0`
and `fully_certified=true` on a leaf that merely added an alternative is not plausible** — adding an
alternative makes more of a grammar reachable, and cannot prove eleven rules *unreachable*.

Two measurements settled it, in order:

1. **Is it mine?** Run the identical command against the **HEAD** grammar (`git show
   HEAD:grammars/systemverilog.ebnf`, placed at a path whose basename still resolves the registered
   grammar name) with the same generated parser:

   ```
   HEAD grammar : total=1358 proof=17 witness=1341 UNKNOWN=0 fully_certified=true
   working copy : total=1358 proof=17 witness=1341 UNKNOWN=0 fully_certified=true
   ```

   **Byte-identical.** ⇒ the leaf is cert-NEUTRAL, and the 17/0 predates it. That is also this
   leaf's NO-REGRESSION evidence on the certification axis — an A/B against HEAD is stronger than
   agreement with a stored constant, because it cannot be satisfied by a stale contract.
2. **Then why does the contract say 6/11?** Because the *gate* runs a different configuration, and
   `make sv_cert_recognized_union_gate` reproduces the contract exactly:
   `canonical_unknown: 11 == expected_canonical_unknown: 11`, `union_unknown: 0`,
   `union_witness: 1352`, `union_residual_rules: []`, deterministic across seeds `[0,7,42]`, **PASS**.

⭐ **The explanation, and it is worth keeping** (TOOLBOX 4.6): the entry universe of an unreachability
proof is *the cert entry plus every `--cert-union-config` entry present in the tree*. The bare
command declares ONE entry, so 11 rules reachable only from `sv_multi_entry_root` / `library_text` /
`systemverilog_parseable_file` are provably unreachable **from that entry** and get certified by
proof. The gate declares all four entries, correctly REFUSES those eleven proofs, and witnesses the
rules through the union instead. ⇒ `6 + 11 = 17` is an identity, not a coincidence, and
`fully_certified=true` from the bare command is a claim about a **narrower** grammar than the one SV
actually ships.
⛔ **So the bare invocation is the wrong instrument for an SV headline** — it reports a stronger
result than the shipped configuration supports. Quote `sv_cert_recognized_union_gate`, never
`--report-certificate-coverage` alone, when the question is "is SV fully certified".

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — the inversion, both directions, minimal repros under
  `rust/target/sv_3_26/`:
  `./rust/target/release/parseability_probe --parse systemverilog …/lrm_bare.sv --profile sv_2017`
  → `Parser did not consume full input at position 0 [furthest_position=37, +37 bytes deeper]`
  on `initial q = {};` (the LRM form), while `initial q = '{};` PASSES. `furthest_position=37`
  lands **exactly on the `}`** — the parser consumed `{` as a concatenation opener and died where
  `concatenation` demands its first `expression`.
- [x] **ROOT CAUSE (WHY + WHERE)** — correctness family. **WHERE:** `grammars/systemverilog.ebnf`,
  rule `empty_unpacked_array_concatenation`, whose whole body was `tick lbrace rbrace`. **WHY:** a
  CITATION error in a prior fix, not a missing fix. `SV-EXH-PROOF.3.3.4.b.6.2.37.8` correctly
  replaced a never-matching body (`lbrace epsilon rbrace`; `epsilon` is undefined in this grammar,
  which `--lint-grammar`'s undefined-reference check would now catch) but wrote the replacement as
  `'{ }` citing *"§A.6.7"*. The production is **A.8.1** and reads `{ }`
  (`docs/systemverilog/2017/md/section-41-data-read-api.md:2907`), with **footnote 35**: *"{ } shall
  denote an empty unpacked array concatenation … and shall not be used in any other form of
  concatenation"* (`:3551`); **no `assignment_pattern` alternative is empty** — all four require at
  least one `expression` (`:2292`–`:2295`) — so `'{}` has no derivation in Annex A at all. The error
  reproduced as a SUCCESS (its motivating input genuinely used the apostrophe) and was never
  questioned. ⇒ the rule was wrong in BOTH directions simultaneously.
- [x] **FIX** — declarative tier (grammar only; ZERO Rust bytes). Two annotated alternatives,
  `lbrace rbrace | tick lbrace rbrace`, each carrying the rule's existing
  `-> {kind: "empty_unpacked_array_concat"}` so the emitted node is identical on both arms. Arm 2 is
  retained deliberately as recorded bucket-(a) dialect tolerance (see the adjudication above); the
  grammar comment says which arm is which and why, and the stale "§A.6.7 / `'{ }`" citation is
  corrected in place so the next reader is not misled the same way.
- [x] **ADDRESSED (verified)** — `REJECT → PASS` on the LRM form, `PASS → PASS` on the tolerated
  form, on **both** build modes (debug and release probes agree), with
  `--parse-dump-ast` confirming both arms emit the same `empty_unpacked_array_concat` node.
  Corpus, at the parameters `.3.27` now binds: **pass `9741 → 9746`, fail `6591 → 6586`**, timeout
  `4 → 4`, crash `0 → 0`. Adjudicated burn-down: **`unexplained` `298 → 293`**
  (rejects-valid `277 → 272`; accepts-invalid `21` unchanged); live worklist `277 → 272` rows /
  `175 → 171` clusters, still 7 families.
  ⭐ **EVERY MOVED ROW IS EXPLAINED BY THE CONSTRUCT — exactly 5, all
  `divergence:unexplained_rejects_valid → match`, from FOUR independent suites**, and none moved the
  other way:
  `ispras-sv-tests ieee-1800-2012/07/07.10.04_01.sv` (the standard's OWN clause-keyed example for
  §7.10.4), `iverilog ivtest/ivltests/sv_queue3.v`, `sv-tests tests/chapter-7/queues/delete_assign.sv`,
  `verilator test_regress/t/t_queue_empty_bad.v`, `verilator test_regress/t/t_queue_empty_pin.v`.
  A fix whose corpus delta is entirely its own construct is the shape a construct leaf should have.
- [x] **NO REGRESSION** — `--lint-grammar` exit 0. **Certificate coverage A/B against HEAD, not
  against a stored constant:** the HEAD grammar and the working copy produce a **byte-identical**
  `CERTIFICATE-COVERAGE: … total=1358 proof=17 witness=1341 UNKNOWN=0` line, and
  `make sv_cert_recognized_union_gate` **PASSED** reproducing its pinned contract exactly
  (`canonical_unknown: 11 == expected 11`, `union_unknown: 0`, `union_witness: 1352`,
  `union_residual_rules: []`, deterministic across seeds `[0,7,42]`, `unmet_criteria_count: 0`) ⇒ the
  cert contract needs **no re-baseline**, `expected_total=1358` is unmoved because no rule was added.
  `make ast_shape_contract_gate` **18/18 passed**.
  `make generated_clippy_correctness_gate` → `✅ PASS — 0 clippy::correctness findings across 10
  required + 1 optional generated artifacts` (⛔ run explicitly: `clippy_on_rust_change` printed
  `No Rust/generated Rust changes detected; skipping` — the known standing tripwire, since
  `generated/` is untracked so git sees no change). Corpus timeouts unchanged at 4, all four
  serially re-confirmed, `0` reclassified.
- [x] **LOCKSTEP** — `grammars/systemverilog.ebnf` comment rewritten with the correct A.8.1 citation
  and the two-arms rationale; `LRM-GRAMMAR-FIDELITY.1c` opened for the biased-evidence-base finding;
  `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `docs/TASK_TREE.md`, and the regenerated
  adjudication/worklist artifacts. No contract/ledger/schema change: the AST node is unchanged on
  both arms, so no published integration surface moves, and this is an under-acceptance repair found
  in-house rather than a downstream-reported released-parser bug.

⚠️ **OPERATIONAL NOTE FOR THE NEXT GRAMMAR LEAF — and the remedy is a rule this repo ALREADY has,
not a new one.** The release `parseability_probe` build takes **~20 minutes** and was killed twice at
10–11 minutes, both times reporting `signal: 15, SIGTERM` with **zero** rustc errors — which reads
exactly like a compile failure until you check the log for an `error[EXXXX]` that is not there.

⛔ **First diagnosis was that a new detach wrapper was needed. That was WRONG, and checking before
building it is what caught it.** `scripts/run_with_memory_guard.sh:262` already does `set -m`, giving
its child **its own process group** — which is precisely the isolation the dead builds lacked. The
evidence separates cleanly along that line and nothing else:

| job | under the guard? | outcome |
|---|---|---|
| corpus promote run (~13 min) | **yes** | completed `exit=0` |
| corpus re-measure (347 s) | **yes** | completed `exit=0` |
| `cargo build --release` (~11 min) | no | `signal: 15`, 0 rustc errors |
| `cargo build` debug (~11 min) | no | `signal: 15`, 0 rustc errors |
| `cargo build --release`, ad-hoc `setsid` (19 m 55 s) | no (hand-detached) | completed `exit=0` |

⇒ the 13-minute corpus job survived while an 11-minute build died **because one was guarded and the
other was not**. `README.md` already states *"heavy or background jobs must run under the memory
guard"*; the defect was that this session did not read a `cargo build` as a "job". So:

```bash
scripts/run_with_memory_guard.sh --budget-mb 16384 --timeout-s 7200 -- \
  cargo build --release --features generated_parsers --bin parseability_probe
```

⭐ **A second detach mechanism was deliberately NOT added.** Shipping `scripts/run_detached.sh`
alongside a guard that already isolates the process group would have been two mechanisms for one
job, with the usual consequence — they drift, and the next reader cannot tell which is
authoritative. **The fix for "I did not apply the existing rule" is never a second rule.**

⛔ **DIAGNOSTIC SIGNATURE worth keeping:** `signal: 15, SIGTERM` in a cargo log with **no
`error[EXXXX]` anywhere** is an INFRASTRUCTURE kill — an unguarded long job — never a broken grammar.
Grep the log for a real rustc error before re-diagnosing the change that "broke the build".
Measured build costs, so the next leaf can budget: release `parseability_probe` **19 m 55 s**, debug
`parseability_probe` + `ast_pipeline` **2 m 42 s**, full SV corpus **347 s** + serial
re-confirmation.

##### `.3.27` — ⛔ the corpus runner's DEFAULTS do not match the parameters the tracked artifact was produced with, and nothing compares them — so the obvious invocation silently re-baselines a graduation ORACLE (routed by `.3.25`, 2026-08-10)

- **Status: `done` 2026-08-10 (`PGEN-SV-CORPUS-GRAD-0186`).** The bare, documented invocation
  `stimuli/run_external_corpus.sh sv` overwrote `characterization.md` + `results.tsv` — and
  therefore the adjudication manifest and the burn-down number — **under different measurement
  conditions than the artifact it replaces**, with no warning. It now **adopts** the recorded
  parameters and **refuses** (exit 5) a run that would deviate.
- **MEASURED in `.3.25` (do not re-derive):**

  | | committed artifact's provenance | script defaults |
  |---|---|---|
  | per-file timeout | **60 s** | **20 s** |
  | parse binary | `rust/target/release/parseability_probe` | `rust/target/debug/parseability_probe` |

  The two are not close: `mm_ram.sv` parses in **12 s release / 127 s debug**. Running the
  defaults moved **6 rows** into `divergence:explained_timeout` (timeouts **4 → 10**, one row
  `pass → timeout`) on a parser change that provably could not touch them. Re-run at the recorded
  parameters, the timeout population returns to exactly **4** and those 6 rows do not move.
- ⭐ **The provenance block is RIGHT — that is what makes this worth fixing.** `characterization.md`
  faithfully records binary path, sha256 and invocation (the `FLOW-INTEGRITY` hand-off-provenance
  work). The gap is that the record is **write-only**: no check reads the existing artifact's
  parameters and compares them with the run about to replace it.
- ⚠️ **Why it is a bar defect and not a nuisance.** Here the 6 rows were `deferred → explained`,
  so `unexplained` (298) was untouched. The same drift landing on a `must_accept` row records it
  `explained_timeout` and quietly removes it from `unexplained_rejects_valid` — **the burn-down
  number improves because the machine was busy** ([[a-rising-pass-rate-is-not-evidence-of-correctness]]).
  Nothing currently guards that direction.
- **Candidate remedies (decide with measurement, do not assume):**
  1. **Make the recorded parameters the defaults** — the runner reads the existing
     `characterization.md` provenance block and reuses timeout + binary unless explicitly
     overridden. Cheapest, and it makes the correct run the easy run.
  2. **REFUSE on drift** — if an artifact exists and the pending run's parameters differ, abort
     with both sets printed and require an explicit `--rebaseline`. The
     `DOCTRINE_ENFORCEMENT.md` §3 *structural* archetype; strongest, and it cannot rot.
  3. **Record per-file duration** in `results.tsv` and report the population within 2× of the
     timeout, so the boundary set is a known number rather than a surprise.
  4. **Confirm a timeout serially before recording it** — a timeout is real only if it reproduces
     without contention; ~4–10 files, so the cost is trivial.
- **Cross-family:** `stimuli/run_external_corpus.sh` is the shared family-parameterized runner the
  VHDL lane (`CORPUS-GRAD-ALL.2`) uses unchanged — fix it in the runner, never in an SV wrapper.

##### `.3.27` — WHAT WAS BUILT (2026-08-10, session #232)

⭐ **THE DRIFT IS SYSTEMATIC, NOT AN SV ACCIDENT — measured before designing anything.** All three
tracked characterization artifacts record `60 8 0` against `rust/target/release/parseability_probe`:

```
$ awk -F'`' '/^Generated by `stimuli/{print FILENAME": "$2}' \
    stimuli/{sv,vhdl}/characterization/characterization.md \
    stimuli/sv/characterization/characterization_v2005.md
stimuli/sv/characterization/characterization.md:        sv 60 8 0
stimuli/vhdl/characterization/characterization.md:      vhdl 60 8 0
stimuli/sv/characterization/characterization_v2005.md:  sv2005 60 8 0
```

against script defaults of `20` and `rust/target/debug/parseability_probe`
(`run_external_corpus.sh:26`/`:31` pre-fix). ⇒ **3 of 3 tracked oracles could not be reproduced by
the invocation their own header documents.** `.3.25` found it on SV; it was never SV-specific.

⛔ **AND THE ROOT CAUSE IS ONE LINE OF BASH, NOT A MISSING FEATURE.** `TIMEOUT_S="${2:-20}"`
collapses *"the caller asked for 20 s"* and *"nobody said, so the default is 20 s"* into the same
value. Every later comparison is then impossible **by construction** — the information needed to
detect drift is destroyed on line 26, before any check could run. The fix captures `"${2-}"`
first and defaults afterwards; everything else follows from having that distinction back.

**FOUR REMEDIES WERE PRICED; ALL FOUR LANDED, TWO IN A DIFFERENT SHAPE THAN PROPOSED** — the
`decide with measurement, do not assume` instruction changed two of them:

1. **Recorded parameters become the defaults** — the runner reads the artifact's own provenance
   block (the `Generated by …` invocation line + the `| parse binary |` row) and adopts timeout /
   jobs / max-files / parse-binary for any argument the caller omits. The correct run is now the
   *easy* run: `stimuli/run_external_corpus.sh sv` measures at 60 s against the release probe.
2. **REFUSE on drift** (`DOCTRINE_ENFORCEMENT.md` §3 *structural* archetype) — a caller-supplied
   value that differs from the recorded one aborts with **exit 5**, printing both sets, before any
   file is parsed and without touching the artifact. Escape hatches are explicit and loud:
   `PGEN_CORPUS_REBASELINE=1` (deliberate new baseline) or the new `PGEN_CORPUS_OUT_DIR=<dir>`
   (measure elsewhere and **diff before promoting** — the non-destructive path that did not exist).
   ⭐ Safe to make hard-refusing because **nothing automated invokes this runner**: `grep -rn
   run_external_corpus --include=Makefile --include=*.mk --include=*.sh --include=*.yml` returns
   only prose references, so no gate can be broken by the refusal.
3. **Per-file durations — to a SIDECAR, ⛔ NOT a 4th column of `results.tsv` as proposed.** The
   proposal was measured against the consumers first and would have broken all four:
   `adjudicate_external_corpus.py:2384` and `:2547` and `corpus_rule_coverage.py:155` each do
   `suite, observed, path = line.split("\t")` — a hard 3-way unpack raising `ValueError` — and
   `cluster_rejects_valid.py:306` does `if len(cols) != 3: continue`, which would have **silently
   dropped every row and reported an empty worklist**. One loud break and one silent one, the
   silent one being the generator of the burn-down worklist itself. So durations go to
   `durations.tsv` (the same rows plus a wall-seconds column) and `results.tsv` keeps its
   three-column contract untouched. The report publishes the deadline-proximity population
   (files finishing within 2× of the timeout) and the slowest completing file.
4. **Serial re-confirmation of every timeout** — each `timeout` row is re-run alone at the same
   deadline before it is recorded, because a timeout under `-P 8` is a claim about the machine as
   much as about the parser. Bounded by `PGEN_CORPUS_TIMEOUT_RECONFIRM_MAX` (default 64 ≈ 16× the
   measured maximum population of 4) and **any truncation is reported in the artifact**, never
   silent.

⭐ **A FIFTH DEFECT, FOUND WHILE FIXING THE FOURTH AND IN THE SAME CLASS: the oracle was not
diffable.** `xargs -P` appends in *completion* order, so `results.tsv` row order was
nondeterministic — `sort -c` fails on the pre-fix file at line 3. A re-run in which every single
verdict was identical still produced a diff of thousands of moved lines, which is a complete
explanation of why nobody ever diffed this artifact against the run replacing it. Both artifacts
are now sorted by `(sub-corpus, path)` under `LC_ALL=C`. **This is the same defect as the headline
one** — an oracle that cannot be compared with its replacement — and closing the parameter half
while leaving the ordering half would have left the comparison just as impractical.

⚠️ **HONEST BOUNDS, stated rather than discovered later.** (a) The sha256 of the parse binary is
deliberately **NOT** part of the refusal: a grammar change *must* be followed by a re-measure with
a rebuilt probe, and refusing on the hash would block the correct workflow — the instrument-identity
table already flags hash drift for the reader, which is the right tool for that question. Only the
binary **path** (release vs debug) and the three run parameters bind. (b) Durations are recorded via
`EPOCHREALTIME` (µs) on bash ≥ 5 and fall back to whole seconds on bash 4; the script already
required bash ≥ 4 for `mapfile`, so this adds no new floor. (c) The refusal reads only the
**canonical** report, so it binds even when output is redirected — deliberate, so a scratch run
still has to say out loud that it is deviating.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — the documented bare invocation could not reproduce the artifact it
  overwrites. `bash -n stimuli/run_external_corpus.sh` clean; the drift is visible directly in the
  tracked headers vs the pre-fix source: `awk -F'`' '/^Generated by `stimuli/{print $2}'` yields
  `sv 60 8 0` / `vhdl 60 8 0` / `sv2005 60 8 0` while `git show HEAD:stimuli/run_external_corpus.sh`
  carries `TIMEOUT_S="${2:-20}"` and `PROBE="${PGEN_PARSE_PROBE_BIN:-$ROOT/rust/target/debug/parseability_probe}"`.
  `.3.25` measured the consequence: 6 rows moved to `divergence:explained_timeout`, timeouts
  `4 → 10`, on a parser change that provably could not touch them.
- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow family. **WHERE:**
  `stimuli/run_external_corpus.sh:26`/`:31` (pre-fix). **WHY:** `${2:-20}` and
  `${PGEN_PARSE_PROBE_BIN:-…/debug/…}` erase the distinction between *supplied* and *defaulted*, so
  no downstream check can exist; and the provenance block that records the truth was **write-only**
  — `git log -S 'Generated by `stimuli/run_external_corpus.sh' --oneline -- stimuli/run_external_corpus.sh`
  shows the line only ever being WRITTEN, never read back. Confirmed by `bash -n` (syntax clean, so
  this is semantics not a typo) and by `grep -rn run_external_corpus --include=Makefile
  --include=*.mk --include=*.sh --include=*.yml .` returning **no invoker** — nothing existed that
  could have compared the two.
- [x] **ADDRESSED (verified)** — measured before→after, five controls, each re-runnable:
  - **REFUSE arm (timeout drift):** `stimuli/run_external_corpus.sh sv 20 8 0` → `EXIT=5`, prints
    `pending run: 20 / tracked artifact: 60`; `shasum -a 256` of `results.tsv` +
    `characterization.md` **byte-identical before and after** (nothing parsed, nothing written).
  - **REFUSE arm (binary drift):** `PGEN_PARSE_PROBE_BIN=$PWD/rust/target/debug/parseability_probe
    … sv 60 8 0` → `EXIT=5`, `pending run: rust/target/debug/… / tracked artifact:
    rust/target/release/…` (the absolute caller path normalized to repo-root-relative to compare).
  - **ADOPT arm (the headline fix):** bare `stimuli/run_external_corpus.sh sv` now reports
    `per-file timeout 60s (provenance) · parallel jobs 8 (provenance) · max files 0 (provenance) ·
    parse binary rust/target/release/parseability_probe (provenance)` — **before the fix this same
    command ran 20 s against the debug probe.**
  - **FRESH-TREE arm:** with the canonical report absent, the run reports all four as `(default)`
    plus `(no tracked artifact for this family yet)`; the report was restored **byte-identical**
    (`shasum` compared) after the control.
  - **RE-CONFIRMATION ground truth (positive AND negative):**
    `bash docs/tasks/artifacts/sv_corpus_grad/timeout_reconfirm_control/run_control.sh` → exit 0.
    Positive arm (stub slow once, then fast — the contention shape): `parallel timeouts=3
    re-confirmed=3 reclassified=3 final timeouts=0`. Negative arm (stub always slow — a genuinely
    slow parse): `parallel timeouts=3 re-confirmed=3 reclassified=0 final timeouts=3`. ⇒ the pass
    fires, reclassifies **only** on disagreement, and never erases a reproducible timeout. ⭐ The
    control **failed its first run and refused to publish a number** (its own repo-root arithmetic
    was off by one) rather than reporting `0` — which is the failure direction an instrument must
    have.
- [x] **NO REGRESSION** — `bash -n stimuli/run_external_corpus.sh` clean. The consumer contract is
  proven, not assumed: `results.tsv` still carries exactly 3 fields
  (`awk -F'\t' '{print NF}' | sort -u` → `3`; `durations.tsv` → `4`), and the **silent** consumer
  was re-run against the new artifact — `python3 stimuli/sv/cluster_rejects_valid.py --results …
  --grammar systemverilog --profile sv_2017` → `rows: 124  clusters: 42` against exactly `124`
  `fail` rows in the input, i.e. **zero silently dropped**. ⭐ **FULL-CORPUS RECONCILIATION — the
  decisive oracle re-run.** The bare `stimuli/run_external_corpus.sh sv`, which before this leaf
  ran 20 s/debug, now adopts 60 s/release and was re-run over all 16 336 files
  (`--budget-mb 16384`, `exit=0 peak_tree_rss=12279MB elapsed=358s`):

  ```
  $ diff <(LC_ALL=C sort stimuli/sv/characterization/results.tsv) \
         <(LC_ALL=C sort rust/target/corpus_3_27_full/results.tsv) | grep -c '^[<>]'
  0
  tracked : rows=16336 pass=9741 fail=6591 timeout=4 crash=0
  re-run  : rows=16336 pass=9741 fail=6591 timeout=4 crash=0
  ```

  **Zero differing rows out of 16 336** — the same command that moved 6 rows and took timeouts
  `4 → 10` before the fix now reproduces the tracked oracle exactly. No `.ebnf`, Rust, codegen or
  generated artifact is touched by this leaf, so the cert/AST/clippy surfaces are unreachable from
  it and the grammar-side gates are unaffected by construction.

⭐⭐ **AND THE DURATION CENSUS ANSWERED THE QUESTION IT WAS BUILT TO ASK, BETTER THAN EXPECTED.**
The flip-risk population at the recorded parameters is **empty**, and the corpus turns out to be
sharply bimodal with a 45-second gap:

| band | files |
|---|---|
| `< 1 s` | 16 316 |
| `1–10 s` | 15 |
| `10–30 s` | **1** (`opentitan/hw/vendor/pulp_riscv_dbg/tb/mm_ram.sv`, 15.15 s) |
| `30–60 s` (within 2× of the deadline) | **0** |
| `> 60 s` (timeout) | 4 — all four **re-confirmed serially, 0 reclassified** ⇒ real, not contention |

⇒ two conclusions the pass/fail/timeout triple could never have supported. **(1) At 60 s/release the
verdict set is structurally stable** — there is no boundary population for a busy machine to flip,
so the 4 timeouts are a property of the parser and nothing else. **(2) The old defaults sat exactly
on the cliff:** at a 20 s deadline `mm_ram.sv` (15.15 s release) is *inside* 2× the deadline, and on
the debug binary it takes **127 s** and times out outright — which is precisely the row `.3.25`
watched move. The census turns *"the defaults were different"* into *"and here is the one file that
made the difference, and how far it sat from the edge."*
- [x] **LOCKSTEP** — book *The Gate Flow* (the corpus-runner recipe + the binding-parameters rule),
  `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`, and this leaf. No contract/ledger/schema
  surface changes: the runner is operator-invoked and publishes no downstream contract.

##### `.3.26b` — ⛔ DIRECTOR RULING 2026-08-10: `'{}` is LEGAL SV; the `.3.26a` deletion is REVERTED and its premise was FABRICATED (comment-only, generated parser BYTE-IDENTICAL)

- **Status: `done` 2026-08-10 (`PGEN-SV-CORPUS-GRAD-0189`).** `.3.26a` (never released — the deletion
  was reverted before any build shipped) removed the `'{ }` arm claiming it was "non-LRM
  over-acceptance". **The claim was never read in the LRM. It was inferred**, and the director
  stopped it: *"removing `'{}` does not belong to those types of decision because this causes PGEN SV
  parser to reject inputs using `'{}`"* … *"It wasn't sota, signoff and sure not professional-grade,
  please refrain from doing such thing again."* Full ruling + the standing rule it establishes:
  [[feedback_sv_strict_lrm_compliance_default]] § **BOUNDING RULING**.
- **What the LRM actually says** (searched exhaustively across `docs/systemverilog/{2017,2023}/md/`):
  **no text anywhere** declares `'{}` illegal or unsupported; **§11.4.12** writes the construct's own
  delimiter pair as `'{ }` (*"…enclosed in braces that begin with an apostrophe ( '{ } )"*); **Annex
  M/VPI** names the operator `vpiAssignmentPatternOp 75 /* '{} assignment pattern */`. ⇒ `'{}` is a
  **third instance of the Annex-A-incompleteness class**, alongside `q[a:$]` (footnote 42, `.3.25`)
  and `use #(...)` (clause 33.4.3, `.3.19`) — **not** over-acceptance.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — with the arm deleted, the release probe REJECTED the uvm call-site
  shape `return '{};` and `q = '{};`, i.e. the parser rejected input uvm-core and 49 corpus files
  write. Restored: `./rust/target/release/parseability_probe --parse systemverilog … --profile
  sv_2017` → `{}` **PASS**, `'{}` **PASS**, uvm `return '{};` shape **PASS**.
- [x] **ROOT CAUSE (WHY + WHERE)** — correctness family, and the defect was in the REASONING, not the
  grammar. **WHERE:** the `.3.26`/`-0187` prose + the `grammars/systemverilog.ebnf` comment on
  `empty_unpacked_array_concatenation`. **WHY:** *"not derivable from Annex A"* was equated with
  *"illegal SV"*. `./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --lint-grammar` →
  **exit 0** on the restored two-arm rule, and `--parse-dump-ast` shows both arms emitting the same
  `empty_unpacked_array_concat` node — so the arm was never a grammar defect to begin with. The
  inference had already been refuted twice in-repo (`.3.25` footnote 42; `.3.19` clause 33.4.3) and
  is carded as [[annex-a-footnotes-license-derivations-the-productions-cannot-derive]].
- [x] **ADDRESSED (verified)** — both arms restored and verified REJECT→PASS on the release probe for
  all three shapes above; the grammar comment now states the arm is CORRECT and cites §11.4.12 +
  Annex M, replacing the "over-acceptance / bucket-(a) tolerance" framing.
- [x] **NO REGRESSION** — **the change is comment-only and that is PROVEN, not asserted**:
  re-running `make -C rust focus_systemverilog` leaves `generated/systemverilog_parser.rs`
  **byte-identical** (`sha256 10ad6361d1a3db9d8343d46cf957e98f99a3248f68fc2bff3e5336a2a9f80fb7`
  before and after) ⇒ zero codegen effect, so every parser-side oracle (cert-coverage,
  `ast_shape_contract`, generated-clippy, the corpus) is unreachable from this change by
  construction. `--lint-grammar` exit 0.
- [x] **LOCKSTEP** — [[feedback_sv_strict_lrm_compliance_default]] gains the BOUNDING RULING above
  its absolutist wording; new standing directive
  [[feedback_every_finding_is_owned_and_scheduled_never_just_logged]] + `docs/decisions/INDEX.md`
  row; `MEMORY.md` carries the correct-forward debt list. ⚠️ **STILL CARRYING THE FALSE FRAMING —
  correct forward next session:** `CHANGES.md` `-0187`, leaf `.3.26`, the card
  `a-mis-cited-production-reproduces-as-a-success.md`, and `LRM-GRAMMAR-FIDELITY.1c` (whose worked
  example was `'{}` and must be re-based on a genuine over-acceptance row).

##### `.3.26c` — ⭐ MODEL IT WHERE IT BELONGS: `'{ }` is an `assignment_pattern` with an EMPTY element list, not an "empty concatenation with an apostrophe" (director 2026-08-10)

- **Status: `done` 2026-08-10 (`PGEN-SV-CORPUS-GRAD-0191`), grammar-only, ZERO Rust bytes.**
- **DIRECTOR:** *"which means that the inside of `'{...}` can be empty. this need to be properly
  capture in the EBNF."* ⇒ `.3.26`'s two-arm `empty_unpacked_array_concatenation` was expedient and
  **structurally wrong**. `'{ }` is not a concatenation: §11.4.12 calls `'{ }` the delimiter of a
  structure/array **literal**, and Annex M names the operator `vpiAssignmentPatternOp`. The empty
  case is the degenerate `assignment_pattern`, so it belongs in `assignment_pattern`.
- **THE CHANGE** — two rules, net acceptance identical, model corrected:

  | rule | before | after |
  |---|---|---|
  | `assignment_pattern` | 4 alternatives, each requiring ≥ 1 element | **5** — new `@sample: "'{}" tick lbrace rbrace -> {exprs: []}` |
  | `empty_unpacked_array_concatenation` | `lbrace rbrace` **\|** `tick lbrace rbrace` | **pure A.8.1**: `lbrace rbrace` only |

- ⭐ **THE AST IS THE REAL DELIVERABLE HERE, and it is now the natural degenerate case.** `'{}`
  previously emitted `{kind: "empty_unpacked_array_concat"}` — a node no assignment-pattern consumer
  would look for. It now emits the **same shape as every other assignment pattern**, differing only
  in the element list. Measured with `--parse-dump-ast`:

  ```
  '{}       … "kind":"primary" → "kind":"sv_2017" → {"pattern":{"exprs":[]},   "type":[]}, "kind":"assign_pattern"
  '{0, 1}   … "kind":"primary" → "kind":"sv_2017" → {"pattern":{"exprs":[…]},  "type":[]}, "kind":"assign_pattern"
  {}        … "kind":"empty_unpacked_array_concat"     (unchanged — this one IS A.8.1)
  ```

  ⇒ a Nexsim-side consumer walking assignment patterns gets `exprs: []` instead of a special case.
- **REACHABILITY CHECKED BEFORE MOVING, not after.** `'{}` acceptance now flows through
  `assignment_pattern → assignment_pattern_expression`, which is referenced at every site
  `empty_unpacked_array_concatenation` is: `primary` (`:3237`/`:3254`), `constant_primary` via
  `constant_assignment_pattern_expression` (`:1479`→`:1542`/`:1580`), and the two lvalue sites
  (`:4381`/`:4443`). No use site loses the form.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — pre-change, `'{}` parsed only via a `tick lbrace rbrace` arm grafted
  onto the A.8.1 bare-brace rule, so `--parse-dump-ast` on `q = '{};` emitted
  `{kind: "empty_unpacked_array_concat"}` — a *concatenation* node for a construct the LRM calls a
  literal, and a shape no `assignment_pattern` consumer handles.
- [x] **ROOT CAUSE (WHY + WHERE)** — correctness family. **WHERE:** `grammars/systemverilog.ebnf`,
  `assignment_pattern` (all four alternatives required ≥ 1 element, so the empty case had nowhere
  legitimate to live) and `empty_unpacked_array_concatenation` (carried the graft). **WHY:** the
  empty case was modelled by its *delimiters* rather than by *what it is*. `--dump-gen-ast` on the
  corrected grammar confirms the intent landed and nothing was silently truncated:
  `assignment_pattern` = **Or with 5 alternatives / 5 branch return annotations** (alt 4 the new
  empty one), `empty_unpacked_array_concatenation` = **single Sequence / 1 annotation**.
  `--lint-grammar` **exit 0** (`undefined_references=0`, `ordered_choice_shadowing=0`,
  `unreachable_rules=0`, `profile_orphans=0`, 1481 rules).
- [x] **ADDRESSED (verified)** — all four shapes PASS on the rebuilt probe: `q = {};` (A.8.1),
  `q = '{};` (empty pattern), uvm's `return '{};`, and the non-empty regression `a = '{0, 1};`.
  AST parity between empty and non-empty patterns shown above.
- [x] **NO REGRESSION** — cert coverage deterministic and **unmoved** at seeds 0/7/42
  (`total=1358 proof=17 witness=1341 UNKNOWN=0`); `make sv_cert_recognized_union_gate` **PASSED**
  reproducing its pinned contract exactly (`canonical_unknown: 11 == expected 11`, `union_unknown: 0`,
  `union_witness: 1352`, `union_residual_rules: []`, `unmet_criteria_count: 0`) ⇒ **no cert
  re-baseline needed**, `expected_total=1358` unmoved (no rule added or removed — one alternative
  moved between two existing rules). `make ast_shape_contract_gate` **18/18 passed** — ⚠️ honest
  reading: that means the construct is not pinned by the manifest, **not** that the shape is
  unchanged; the `'{}` shape change is real and was measured directly with `--parse-dump-ast` above.
  `make generated_clippy_correctness_gate` → **`✅ PASS — 0 clippy::correctness findings`** across
  10 required + 1 optional generated artifacts.
  ⭐⭐ **FULL-CORPUS PROOF OF ACCEPTANCE-NEUTRALITY — row level, not just totals.** Re-measured all
  **16 336** files at the provenance-bound parameters (`.3.27` adopted 60 s / release / 8 jobs
  automatically; guard `exit=0`, `elapsed=347 s`): **pass 9746 · fail 6586 · timeout 4 · crash 0 —
  every count identical to the pre-change baseline**, all 4 timeouts serially re-confirmed with 0
  reclassified. Re-adjudicated: `match=5804 unexplained=293 explained=1463 deferred=8776`
  (v2005 `match=2186 unexplained=68`) — **unchanged**, and
  **`adjudication_manifest.tsv` has ZERO diff**, i.e. all 16 336 rows adjudicate identically.
  `characterization.md` moves only 5 lines: the three instrument hashes (expected — new binary,
  grammar and generated parser) and one file's wall time `15.37 → 15.24 s` (machine noise).
  ⇒ the restructure is acceptance-neutral **by measurement**, not merely by argument.
- [x] **LOCKSTEP** — grammar comments on both rules rewritten to say which production each is and
  ⛔ not to re-graft the arm; `CHANGES.md`; `DEVELOPMENT_NOTES.md`; `MEMORY.md`; `docs/TASK_TREE.md`.

##### `.3.26d` — ⛔ PAY THE CORRECT-FORWARD DEBT: four durable surfaces still called `'{}` "non-LRM over-acceptance" three commits after the ruling that refuted it (docs only, ZERO code bytes)

- **Status: `done` 2026-08-10 (`PGEN-SV-CORPUS-GRAD-0192`), docs only, ZERO code bytes.**
- **WHY THIS IS ITS OWN LEAF.** `.3.26b` obtained the director ruling and fixed the *grammar comment*
  and the *decision record*, then listed four surfaces it had **not** fixed and deferred them to
  "next session". `.3.26c` then re-modelled the construct without touching that list either. So for
  three commits the repository's changelog, its own task tree, its Knowledge-Map card and a routed
  leaf in another tree all still asserted a claim the project had formally retracted. ⭐ **A
  retraction that lands in one layer and not the others is not a retraction — it is a
  contradiction**, and the layers that kept the false version are exactly the ones a future session
  reads first (`CHANGES.md`, the KM card via retrieval, the routed leaf when `LRM-GRAMMAR-FIDELITY`
  opens).
- **THE FOUR SURFACES, and what each one now says:**

  | surface | carried | now |
  |---|---|---|
  | `CHANGES.md` `-0187` | *"`'{}` … which Annex A cannot derive at all"*; a bullet adjudicating it as **bucket-(a) dialect tolerance** | ⛔ RETRACTED-IN-PART banner naming the false clauses + inline markers; the `{}` half explicitly preserved as correct |
  | leaf `.3.26` (this tree) | heading *"wrong in BOTH directions"*; an over-acceptance verdict table row; a whole ADJUDICATION section | banner at the leaf head; the verdict table gains a CORRECTED column; the ADJUDICATION section marked retracted **verbatim, not deleted** |
  | KM card `a-mis-cited-production-reproduces-as-a-success` | *"wrong in both directions at once"*; a closing section adjudicating the tolerated arm | the "both directions" claim removed; a new **sequel section** — the same fix made a SECOND citation claim and that one was never opened either |
  | `LRM-GRAMMAR-FIDELITY.1c` | `'{}` as its worked counter-example + as its SEED ROW | re-stated as an **UNWITNESSED hypothesis**; the three known constructs re-cast as **negative controls** |

- ⭐ **THE CARD GOT STRONGER, NOT WEAKER — which is why it was corrected rather than retired.** Its
  thesis is *"a mis-cited production reproduces as a SUCCESS, so the test that should catch it
  passes."* The very fix that occasioned it made **two** citation claims: the transcription (`'{ }`
  vs `{ }`, opened and verified) and a negative claim (*"`'{}` is not legal"*, never opened). The
  card's own habit — *quote the clause number, and open the clause* — would have caught both. So the
  correction adds a second worked instance to the same card instead of contradicting it, and sharpens
  the rule to: **a claim of the form "X is not legal" is a citation with the quote left out.**
- ⚠️ **`.1c` LOST ITS ONLY WITNESS AND IS SAID SO OUT LOUD.** The structural argument (a census keyed
  on suite `must_reject` answers cannot see a tolerance the whole ecosystem shares) is untouched by
  the retraction — but it now has **zero** confirmed instances, and a `todo` leaf that reads like a
  measured finding when it is an unwitnessed hypothesis is the same failure one layer down. It also
  gained a real design constraint from the retraction: **Annex-A non-derivability is a KNOWN-BAD
  classifier** for the audit `.1c` proposes — it would flag `'{ }`, `q[a:$]` and `use #(...)`, all
  three legal and all three already measured — so those three become the instrument's ground-truth
  controls.
- ⭐ **INDEPENDENT RE-VERIFICATION, not inherited trust.** The LRM claims were re-run from scratch
  rather than copied from `.3.26b`, and the re-run **found a tooling trap the prose does not name**:
  an exact-phrase grep for *"begin with an apostrophe"* over `docs/systemverilog/2017/md/` returns
  **0 hits**, because the markdown wraps the sentence mid-phrase. Searching the short token finds it
  immediately at `section-0-defined-as-false-…-is-greater.md:520-522`, under the `#### 11.4.12
  Concatenation operators` heading at `:498`. ⇒ recorded on the card: **search the distinctive token,
  not the sentence** — the second way this construct has now produced a false negative from a search
  instrument (the first was `pdftotext`, `.3.26b`/`-0190`).
- ⚠️ **AND A SEPARATE DEFECT FOUND WHILE PAYING THE DEBT — routed, not worked** (SV lane lock; and
  per [[feedback_every_finding_is_owned_and_scheduled_never_just_logged]] it is OWNED and SCHEDULED,
  not merely logged). `CHANGES.md` was missing entries for `-0188`, `-0189` and `-0190` entirely —
  including the director ruling itself. Censused over the last 120 commits:
  **41 (34 %) have no `CHANGES.md` entry**. The three SV ones are written here (in-lane); the general
  gap is routed to **`LIVE-DOC-CONTAINMENT.5`**.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — ops family. The debt list is `.3.26b`'s own LOCKSTEP box (*"STILL
  CARRYING THE FALSE FRAMING — correct forward next session"*). Reproduced mechanically:
  `git log --format='%h%x09%s' -120` joined against `grep -q -- "$id" CHANGES.md` returns
  **`MISSING 41` / `HAS 79`**, with `PGEN-SV-CORPUS-GRAD-0188/-0189/-0190` among the missing — i.e.
  the changelog's last word on `'{}` was the retracted `-0187`, and the retraction was absent from
  that surface altogether.
- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow family. **WHERE:** `CHANGES.md` `-0187`,
  `docs/tasks/SV-CORPUS-GRAD.md` `.3.26`, `docs/knowledge/a-mis-cited-production-reproduces-as-a-success.md`,
  `docs/tasks/LRM-GRAMMAR-FIDELITY.md` `.1c`. **WHY:** a correction was applied to the layers the
  fixing session was editing (grammar comment, decision record) and *listed* for the layers it was
  not, and nothing mechanically holds a retraction to its own debt list — the same class as the
  missing changelog entries, which `git log -S` shows landing silently 41 times in 120 commits.
  Re-verified the underlying LRM facts independently: `grep -rn "'{ *}" docs/systemverilog/2017/md/`
  → exactly **2** hits, both supporting (`section-0-…-is-greater.md:522` §11.4.12 notation;
  `section-83-accept-on-operator.md:89` `vpiAssignmentPatternOp`); `grep -rn "shall not.*'{"` → **0**.
- [x] **ADDRESSED (verified)** — all four surfaces now carry the corrected claim; re-grepped after
  the edits, the strings `over-acceptance`, `not derivable` and `bucket (a)` survive in this tree
  **only** inside explicitly-marked retraction context, never as an assertion.
- [x] **NO REGRESSION** — **ZERO code bytes**: `git diff --cached --name-only` contains no
  `grammars/`, no `rust/`, no `generated/` path, so no parser oracle is reachable from this change by
  construction. `bash scripts/check_doctrines.sh` PASS (all registered doctrines, incl. `MEMORY-ARCH`,
  `KNOWLEDGE-MAP` regen-and-diff, `LIVE-DOC-CURRENCY`, `TASK-ACCEPTANCE`).
- [x] **LOCKSTEP** — `CHANGES.md` (the `-0187` banner **plus** the three backfilled entries
  `-0188`/`-0189`/`-0190` and this one); `DEVELOPMENT_NOTES.md`; `MEMORY.md` (debt cleared);
  `docs/TASK_TREE.md`; `LRM-GRAMMAR-FIDELITY.1c`; new leaf `LIVE-DOC-CONTAINMENT.5` owning the
  routed changelog-coverage gap. No book chapter asserts anything about `'{}` (checked:
  `grep -rn "'{}" docs/book/src/` → 0 hits), so no book edit is due.

##### `.3.26e` — ⛔ THE GUARD RAIL WAS LEFT BEHIND WHEN THE ARM MOVED: `'{}` now lives in a rule with no comment, and its old home still says "TWO ARMS" (comment-only, generated parser BYTE-IDENTICAL)

- **Status: `done` 2026-08-10 (`PGEN-SV-CORPUS-GRAD-0193`), grammar comment-only, generated parser
  BYTE-IDENTICAL, ZERO Rust bytes.** Opened by `.3.26d` while re-reading the grammar to verify the
  correct-forward was complete. **Owns two defects in `grammars/systemverilog.ebnf`, both introduced
  by `.3.26c`'s otherwise-correct re-modelling.**
- ⛔⛔ **DEFECT 1 — THE PROTECTIVE COMMENT DID NOT MOVE WITH THE ARM, AND IT IS THE ONLY THING THAT
  EVER STOPPED THE DELETION.** `.3.26a` deleted the `'{}` arm outright on the "non-LRM" inference and
  was reverted only by a director ruling. `.3.26b` then wrote the guard — the §11.4.12 + Annex M
  citations, *"the apostrophe form is LEGAL and must keep parsing — ⛔ do not 'fix' it away"* — into
  the comment on `empty_unpacked_array_concatenation`, which is where the arm was **then**. `.3.26c`
  moved the arm to `assignment_pattern` and **left the guard behind.** Today
  `grammars/systemverilog.ebnf:733` reads, in full:

  ```ebnf
                      | @sample: "'{}" tick lbrace rbrace
                     -> {exprs: []}
  ```

  — no citation, no rationale, no ⛔. ⇒ **a reader who arrives at `assignment_pattern` applying the
  strict-LRM policy sees an alternative that Annex A's four productions do not derive, with nothing
  to stop them.** That is the *exact* starting state of `.3.26a`, restored at a new address, and the
  protection now sits on a rule whose comment (correctly) tells you the arm is **not** there.
  ⚠️ **This is not hypothetical risk-writing: the deletion has already happened once in this file.**
- ⛔ **DEFECT 2 — the old home's comment now contradicts itself, 10 lines apart.** Line 2229 says
  *"TWO ARMS, and they are there for DIFFERENT reasons — do not collapse them"*; line 2239 says
  *"⭐ THIS RULE IS NOW PURE A.8.1 — the bare-brace form ONLY."* Both are in the same comment block
  over a rule that has had **one** arm since `.3.26c`. `.3.26c` rewrote the body of the block and
  missed its header.
- **THE FIX (declarative tier — comments only, no production changes):** move the guard to where the
  arm actually is, and repair the stale header. Net acceptance and net AST **unchanged by
  construction**; the proof obligation is byte-identity of the generated parser, not a corpus run.
- ⭐ **THE GENERAL LESSON, and it is the same shape as `.3.26d`'s:** `.3.26c` verified **reachability**
  before moving the arm (it checked every use site, and that check was right and is recorded). It did
  not verify that the arm's **documented protection** moved with it. A construct's guard rail is part
  of the construct; relocating one without the other silently reverts a director ruling to an
  unguarded state while every parse-level oracle stays green. ⇒ **when a construct moves, the comment
  that defends it is part of the move, and only a reader can check that — no gate can.**

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — read the shipped grammar at HEAD. `assignment_pattern`'s empty
  alternative (`grammars/systemverilog.ebnf:733`) carried **zero** comment bytes: no citation, no
  rationale, no ⛔ — while the guard written to protect it (*"The apostrophe form is LEGAL and must
  keep parsing — ⛔ do not 'fix' it away"*) sat on `empty_unpacked_array_concatenation`, a rule whose
  own comment correctly states the arm is **not** there. Independently, that same block's header read
  *"TWO ARMS … do not collapse them"* 10 lines above *"⭐ THIS RULE IS NOW PURE A.8.1 — the bare-brace
  form ONLY"*, over a one-arm rule.
- [x] **ROOT CAUSE (WHY + WHERE)** — correctness family; the defect is in the durable comment layer,
  not in a production. **WHERE:** `grammars/systemverilog.ebnf` — `assignment_pattern` (the arm's new
  home, undocumented) and the comment block over `empty_unpacked_array_concatenation` (stale header).
  **WHY:** `.3.26c` verified **reachability** before moving the arm — every use site checked, and that
  check was correct — but a construct's *documented protection* is not reachable from any use site, so
  nothing prompted moving it. `./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf
  --lint-grammar` → **exit 0** both before and after (`1481 rules`, `non_terminating=0`,
  `ordered_choice_shadowing=0`, `unreachable_rules=0`, `undefined_references=0`,
  `unbound_fact_kinds=0`, `profile_orphans=0`), i.e. **no lint can see this class** — which is the
  point: the deletion it guards against passes lint too.
- [x] **ADDRESSED (verified)** — guard moved to the arm's actual home and the stale header repaired.
  `--dump-gen-ast` confirms the inserted column-0 comment truncated nothing (the standing tripwire):
  `assignment_pattern` = **Or / 5 alternatives / 5 `branch_return_annotations`**,
  `assignment_pattern_entry` = single Sequence (it sits immediately above the insertion point and was
  the rule at risk), `empty_unpacked_array_concatenation` = single Sequence / 1 annotation,
  `rule_order` = **1481**, entry `systemverilog_file` — reproducing `.3.26c`'s recorded values
  exactly. All four shapes PASS on the release probe: `q = {};`, `q = '{};`, the real uvm
  `return '{};` shape, and the non-empty regression `a = '{0, 1};`.
- [x] **NO REGRESSION** — **comment-only, PROVEN not asserted.** `make -C rust SHELL=/bin/bash
  focus_systemverilog` re-run under `scripts/run_with_memory_guard.sh --budget-mb 16384`
  (`completed exit=0 peak_tree_rss=1962MB elapsed=101s`) leaves `generated/systemverilog_parser.rs`
  **byte-identical** — `cmp` clean against the pre-change copy, `sha256`
  `959e457800dfcbc046ab1b7fa790af65947fadbf5dfd554bae7cb5e5bfb79bad` **before and after**. ⇒ zero
  codegen effect, so every parser-side oracle (cert-coverage at seeds 0/7/42, `ast_shape_contract`,
  `generated_clippy_correctness`, the 16 336-file corpus) is unreachable from this change **by
  construction**, and `clippy_on_rust_change` has nothing to lint because no Rust byte moved.
- [x] **LOCKSTEP** — `CHANGES.md`; `DEVELOPMENT_NOTES.md`; `MEMORY.md`; `docs/TASK_TREE.md`. Book:
  N/A — no user-visible parser behaviour changes (acceptance and AST are byte-identical), and no book
  chapter mentions the construct (`grep -rn "'{}" docs/book/src/` → 0).
  **`promotion:` PROMOTED** — the lesson is carded as
  [[a-constructs-guard-comment-is-part-of-the-construct]] (a cross-family grammar-authoring rule, not
  an SV fact), linked from [[a-mis-cited-production-reproduces-as-a-success]]. Declining was
  considered and rejected: the class was measured live in 4 of 17 grammars, so it is retrieval-worthy
  rather than incident-local.

⚠️ **HONEST BOUND — this leaf fixes the instance, not the class.** Nothing mechanical connects an arm
to the comment that defends it, so the next relocation can strand the next guard exactly the same way.
A cheap candidate check exists (an alternative carrying an `@sample` that no comment within N lines
mentions), but it would be a heuristic over prose and this repo's standing lesson is that a heuristic
is not a census. ⇒ **deliberately NOT mechanized here**, recorded as a known gap rather than papered
over with a check that would fail open. Routed to `LRM-GRAMMAR-FIDELITY.1c`'s instrument discussion,
which already owns the "what does the grammar accept that Annex A cannot derive" enumeration — the
same list is exactly the set of arms that need a defending comment.

⚠️ **ROUTING EVIDENCE — does the class reproduce OUTSIDE SystemVerilog? MEASURED: yes, but SV carries
most of it.** Counting protective grammar comments (`^#` lines matching `⛔` or
*do not delete/remove/re-add/collapse/fix*) across all **17** tracked grammars:
`systemverilog` **14**, `vhdl` **5**, `ebnf` **3**, `semantic_annotation` **1**, the other 13 zero.
⇒ the mechanism is **not SV-specific** — any grammar with a deliberately-kept arm can strand its
guard by relocating the arm — but the exposure is concentrated where the LRM-fidelity pressure is,
which is why the instrument belongs in `LRM-GRAMMAR-FIDELITY` (cross-family) rather than in this
SV corpus tree. ⛔ Honest bound on this number itself: it counts *comments that look protective*, not
*arms that need protection*, so it sizes the population's upper edge and cannot say how many are
currently stranded — exactly the census-vs-heuristic distinction above, applied to my own number.

### `.4` — Full-design corpora chaining

- **Status: `todo`** — extend the curated chaining (bootstrap_files) so
  friscv/scr1/VeeR adjudicate file-by-file honestly or as chained units;
  expansion-dependent cases classified with cause against `SVPP-EXPANSION`.

### `.5` — The standing graduation gate + family-status wiring (code)

- **Status: `todo`** — a deterministic `make` gate asserting zero unexplained
  divergences over the vendored suites at pinned commits; wired as the 8th
  `sv_parser_family_status_gate` criterion (`external_corpus_graduation_green`)
  so `Done` is machine-computed over BOTH axes.

### `.6` — Done-flip lockstep

- **Status: `todo`** — LIVE tracker / SV parser book / SV integration contract /
  top-level book corpus chapter; the SV row flips `Done` ONLY when axis 1
  (`SV-REPLAY-DEBT`) and axis 2 (this tree) are both green.

### `.7` — The LRM-coverage instrument (measured corpus sufficiency)

- **Status: `done`** (mandated by
  [[project_sv_corpus_100pct_lrm_coverage_mandate]]) — `.7a` (the rule-coverage
  instrument + the FIRST measured number, 91.1% / 120 gaps) **done**; ⛔ **both
  instruments RE-MEASURED at HEAD by `.7c` (2026-08-08) — quote `.7c`'s
  92.3 % / 104 gaps, never `.7a`'s original figure**; `.7b`
  (clause matrix from the keyed suites + negatives-density report) **done**
  (`PGEN-SV-CORPUS-GRAD-0016`, session #197). Both coverage lenses now stand:
  the authoritative rule-participation % (`.7a`) and the LRM-structure clause
  matrix + negatives density (`.7b`). The measured worklists feed `.9`.

#### `.7a` — The rule-coverage instrument: measured coverage = 91.1% (120 gaps) ⛔ SUPERSEDED by `.7c` (HEAD: **92.3 % / 104 gaps**)

- **Status: `done`** (`PGEN-SV-CORPUS-GRAD-0008`, session #192, 2026-07-22).
- **The instrument (two parser-agnostic surfaces, both deterministic):**
  1. `ast_pipeline <g>.ebnf --dump-rule-profiles OUT.json` (NEW flag,
     `rust/src/main.rs` — ~55-line read-only handler reusing the lint's own
     `derive_rule_profiles`/`extract_profile_context`, zero new analysis
     code, no codegen change): the DENOMINATOR — full rule inventory with
     declared `@profiles` + DERIVED per-profile satisfiability.
     ⭐ Cross-instrument confirmation: satisfiable_under sv_2017 = **1,343**
     and verilog_2005 = **1,115** EQUAL the cert-coverage canonical totals;
     sv_2023 = 1,362; orphans 0; `interface_class_declaration` reads
     dual-profile post-`.3.1`. TOOLBOX §5.4 + book section added.
  2. `stimuli/sv/corpus_rule_coverage.py` (NEW driver/reporter): the
     NUMERATOR — per accepted corpus file, the generated parser's
     transactional coverage testimony (`--dump-rule-outcome-counts-json`
     `rule_committed_counts` — the existing `RGX-0078.5.h.1b` flag; sound
     under PEG backtracking; failed parses contribute NOTHING), unioned and
     diffed against the inventory → per-rule status
     covered / **GAP** / na_profile, thin-coverage watchlist, per-suite
     contribution, named exclusions.
- **⭐ THE FIRST MEASURED COVERAGE NUMBER (profile sv_2017, the full 16,336-file
  vendored universe): 1,223/1,343 = 91.1% — 120 UNCOVERED rules = the
  measured `.9` worklist** (`rule_coverage_sv_2017.md` + per-rule `.tsv`,
  tracked). Gap shape (coherent with the banked corpus-sufficiency
  assessment): 55 `kw_*` terminal-cohort rules (PATHPULSE$, specify edge
  specifiers B/F/N/P/R/Z, `binsof`, …) + covergroup bins/cross machinery +
  assignment-pattern net-lvalues + dist weights + drive strengths + library
  map constructs. Testimony soundness spot-proven: 253/308 `kw_*` rules DO
  commit (the 55 are real gaps, not a testimony blind spot); na_profile 123;
  fired∩na_profile = ∅ (no inventory-vs-reality contradiction);
  9,360/9,361 accepted files contributed.
- **Named exclusion + engine finding (excluded-with-cause, loud):** Surelog
  `ExponTimeIfElseGen/dut.sv` — plain parse 0.28 s exit 0, but with the
  transactional coverage stack enabled the SAME parse blows up >100×
  (measured >30 s) and does not contribute testimony (the driver's timeout
  lane names such files in the report; this run: the file surfaced via the
  disagreement lane instead). WHY (first-order): the coverage bookkeeping
  defeats the memo's protection on pathological-backtracking shapes.
  Deep root-cause + any engine-side fix = a follow-up owned by this tree
  (instrument tolerates named exclusions; graduation math unaffected).
- **Verification:** driver determinism proven (two 300-file runs, TSV + md
  BYTE-IDENTICAL); full run guarded (exit 0, peak 14,486 MB / 137 s);
  `clippy_on_rust_change` source-strict PASS (generated-stage non-strict
  tracked count now 291 — all in `generated/*` parsers, 0 in `src/`, the
  +1 vs 290 pre-dates this slice at the `.3.1` regen);
  `mdbook_docs_gate` PASS; dual-feature `ast_pipeline` rebuilt (guarded,
  peak 10,086 MB).
- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — the mandate's coverage axis had NO
    instrument: corpus sufficiency was unmeasurable (the `.7` charter).
  - [x] **ROOT CAUSE (WHY + WHERE)** — instrument leaf; the one anomalous
    row (ExponTimeIfElseGen) tool-diagnosed to the coverage-stack blowup
    (plain-vs-instrumented A/B, measured >100×), lane-named in the report.
  - [x] **FIX** — the two surfaces above (flag + driver); no parser/codegen
    behavior change (flag is opt-in read-only; probe untouched).
  - [x] **ADDRESSED (verified)** — the measured 91.1%/120-gap report over
    the full universe; denominator cross-confirmed against the cert totals
    (1,343/1,115 equality); testimony soundness spot-proven.
  - [x] **NO REGRESSION** — clippy source-strict pass; generated parsers
    untouched; mdbook gate pass; determinism byte-proven; no tracked
    surface behavior changed.
  - [x] **LOCKSTEP** — TOOLBOX §5.4 + routing row, grammar-wellformedness
    book section, tree/TASK_TREE/MEMORY/CHANGES/LIVE this commit.

#### `.7b` — Clause matrix + negatives density (done)

- **Status: `done`** (`PGEN-SV-CORPUS-GRAD-0016`, session #197, 2026-07-23;
  READ-ONLY measurement — zero parser/grammar/generated/codegen change).
- **The instrument (one parser-agnostic surface, deterministic, no parser
  run):** `stimuli/sv/corpus_clause_coverage.py` reads the committed
  adjudication manifests + static suite metadata and maps the KEYED corpus
  onto the LRM's own chapter/clause structure — the structural companion to
  `.7a`'s rule lens. Three keyed inputs carry clause metadata: sv-tests
  `:tags:` (edition 1800-2017), ispras `ieee-1800-2012/` dotted filenames,
  ispras `ieee-1364-2005/` `test_*` filenames (Verilog). ivtest is NOT
  clause-keyed (its `.list` `CE` rows are negatives but carry no clause) — so
  it feeds the negatives axis only, honestly noted. Editions are kept
  distinct (clause NUMBERS are not comparable across the 1364/1800 boundary);
  the 1364-2005 lane's verdicts are taken from the `verilog_2005` manifest
  (`adjudication_manifest_v2005.tsv`) where those files are adjudicated (in
  the main manifest they are deferred to that lane).
- **⭐ THE MEASURED CLAUSE PICTURE** (`clause_coverage.md` + per-clause
  `.tsv`, tracked): **2,529 keyed cases** over **852 1800-family + 184
  1364-2005 distinct clauses**; **0 parse-bearing chapter gaps** — every LRM
  chapter 5–35 has ≥1 keyed case (so there is NO chapter-level positive gap;
  the positive gaps are the rule-level 120 from `.7a`). Non-parse-bearing
  chapters (1–4, 36–41) are N/A-with-cause.
- **⭐ THE NEGATIVE AXIS IS MEASURED-THIN (the sufficiency assessment's
  prediction, now quantified):** only **5 of 31** parse-bearing chapters
  carry ANY clause-keyed `must_reject` (5 Lexical=4, 11 Operators=1, 13
  Tasks/functions=1, 16 Assertions=1, 22 Compiler-directives=2 → **9 keyed
  1800-family negatives total**); the 1364-2005 negative axis is **entirely
  UNKEYED** (0 clause-keyed). The bulk of the corpus's negatives are unkeyed
  and un-attributable to a clause without per-file adjudication: sv_2017 lane
  **149** (sv2v 72 / iverilog 41 / verilator 32 / verible 2 / sv-tests 1 /
  slang 1), verilog_2005 lane **33** (iverilog CE). → the negative axis per
  chapter is the sharpest structural gap for `.9`.
- **Join with `.7a`:** `.7a` is the authoritative parseable-surface % (rule
  participation, 91.1% / 120 gaps for sv_2017); `.7b` is the LRM-structure
  lens surfacing two gaps `.7a` cannot express (a parse-bearing chapter with
  no keyed case; a chapter with no keyed negative). `.9` closes gaps from
  both — the report's final section states exactly how.
- **Acceptance Checklist (enforced — instrument leaf, mirrors `.7a`)**
  - [x] **REPRODUCE / ISSUE** — the mandate's coverage axis had only the
    rule lens (`.7a`); the LRM-structure/clause view and the per-chapter
    negatives-density were unmeasured (the `.7b` charter).
  - [x] **ROOT CAUSE (WHY + WHERE)** — instrument leaf; the keyed-vs-unkeyed
    split is tool-derived from the manifests (only 3 suites carry clause
    metadata → ~14% keyed; ivtest CE rows are unkeyed negatives), and the
    edition boundary (1364 vs 1800 clause numbering) is respected, not merged.
  - [x] **FIX** — one Python surface (`corpus_clause_coverage.py`); no
    parser/codegen/grammar behavior change (reads committed metadata only).
  - [x] **ADDRESSED (verified)** — the measured clause matrix + negatives
    density over the full keyed universe; internal-consistency self-checks
    reproduce: keyed(9)+unkeyed(149)=158 = the manifest's total sv_2017
    `must_reject`; the 1364-2005 lane positives sum to 327 = the v2005
    manifest's ispras `must_accept`.
  - [x] **NO REGRESSION** — read-only: zero code/grammar/generated/codegen
    change; the two report artifacts are BYTE-IDENTICAL across two independent
    runs (determinism proven, `cmp` clean on both `.tsv` and `.md`); no
    tracked parser surface touched; `python3 -m py_compile` clean.
  - [x] **LOCKSTEP** — TOOLBOX §5.4 companion + index row, grammar-
    wellformedness book companion paragraph, tree + TASK_TREE index +
    MEMORY/CHANGES/DEVELOPMENT_NOTES/LIVE this commit.

#### `.7c` — the COVERAGE axis re-measured at HEAD (the other half of the graduation bar, stale on both sides of its fraction)

- **Status: `done`** (`PGEN-SV-CORPUS-GRAD-0028`, session #214, 2026-08-08, immediately after
  `.10`). Same defect class `.10` closed for the divergence axis, now applied to
  the coverage axis — because graduation needs BOTH
  ([[project_sv_corpus_100pct_lrm_coverage_mandate]]: zero unexplained divergences
  **AND** 100 % measured coverage).
- ⭐ **This leaf was found by APPLYING `.10`'s new instrument-identity check to the
  neighbouring artifacts, and it is stale on BOTH sides of the fraction** — not a
  suspicion, a measurement:

  | quantity | `rule_coverage_sv_2017.md` (2026-07-22) | HEAD today | how re-measured |
  |---|---|---|---|
  | rules in the grammar | 1 466 | **1 475** | `ast_pipeline grammars/systemverilog.ebnf --dump-rule-profiles` |
  | satisfiable under `sv_2017` (the DENOMINATOR) | 1 343 | **1 352** | same |
  | accepted corpus files (the NUMERATOR base) | 9 360 | **9 694** | `.10`'s tracked run |

  ⇒ the published **`1 223/1 343 = 91.1 %` / 120 gaps cannot be quoted**, and the
  9 rules added since (the `.3.8`/`.3.9`/`.3.10` cascades) are by construction absent
  from the gap list.
- ⛔ **Why this is load-bearing rather than tidy-up:** leaf `.9` — the loop that
  drives coverage to 100 % — burns its worklist directly from `.7a`'s **120 uncovered
  rules**. A stale gap list means working rules that may already be covered while
  missing rules that are not. No gate reads either artifact, so the staleness was
  entirely silent — the same class as the divergence axis, found the same way.

**⛔ A THIRD CONSUMER OF THE SAME PATH CONVENTION, and this one killed the run.**
`corpus_rule_coverage.py:266` called `Path(p).relative_to(ROOT)` on a results.tsv
column-3 value, which raises `ValueError: 'stimuli/sv/subs/Surelog/…/dut.sv' is not in
the subpath of '/Volumes/SSD/…/pgen'` now that the column is repo-root-relative
(`CORPUS-GRAD-ALL.2.1`). ⭐ It failed **at the report stage, after all 9 694 files had
been probed** — 167 s of work discarded at the last step. That makes three consumers of
one convention change (`adjudicate_external_corpus.py` in `.10`, this, and the routed
`.11b`), so the fix here is a **single `repo_relative()` normalizer** accepting both
spellings forever, rather than a third per-call-site patch. Unit-checked on all three
shapes (relative in / absolute-in-this-checkout / absolute-elsewhere).

**THE RE-MEASURED COVERAGE PICTURE (guarded, exit 0, 171 s, peak RSS 6 753 MB):**

| | `.7a` (2026-07-22) | HEAD | |
|---|---|---|---|
| covered / satisfiable under `sv_2017` | 1 223 / 1 343 | **1 248 / 1 352** | |
| **measured coverage** | 91.1 % | **92.3 %** | +1.2 pt |
| **uncovered rules = the `.9` worklist** | 120 | **104** | **−16** |
| files contributing testimony | 9 360 | 9 693 | |
| instrumentation-timeout exclusions | 1 (`ExponTimeIfElseGen`) | **0** | the named exclusion cleared itself |

⭐ **16 gaps CLOSED, and ZERO new gaps** — and the closed set is not a mystery, it is
this campaign's own burn-down showing up in the coverage lens: `drive_strength`,
`strength`, `pullup_strength`, `pulldown_strength`, `kw_supply_*`, `kw_highz_*`,
`kw_pull_*` (that is `.3.7` exactly), `seq_or_tail`, `cover_sequence_statement`,
`kw_first_match_*`, `kw_sync_reject_on_*`, `property_lvar_port_direction` (the
`.3.3`/`.3.8` SVA cascades), plus `extern_tf_declaration`, `kw_forkjoin_*` and two
`kw_token_*`. **0 new gaps** also settles the denominator question: all 9 rules added to
the grammar since are already exercised, so the worklist shrank without hidden growth.

**Clause axis (`.7b` instrument, re-run):** the structural headline is UNCHANGED — 2 529
keyed cases over 852 1800-family + 184 1364-2005 clauses, **0 parse-bearing chapter
gaps**, 9 keyed negatives, unkeyed negatives `sv_2017` 149 / `verilog_2005` 33 — and
`clause_coverage.md` is byte-identical. Only the per-clause TSV moved (66 rows), all in
the **1364-2005** lane's observed pass/fail split: the tracked report is dated
2026-07-23 and reads its v2005 verdicts from a lane that was itself measured
2026-07-26. ⇒ a report was consuming verdicts older than its own source — the exact
staleness class, one level down, now current.

- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — `python3 stimuli/sv/corpus_rule_coverage.py` aborts with
    `ValueError: 'stimuli/sv/subs/Surelog/tests/ExponTimeIfElseGen/dut.sv' is not in the
    subpath of '/Volumes/SSD/Documents/github/pgen'` (guard marker `guard.90274.marker`,
    exit 1 at 167 s), so the coverage axis could not be re-measured at all; and the
    tracked `91.1 %` was measurably stale on both sides (denominator 1 343 → 1 352 via
    `--dump-rule-profiles`; numerator base 9 360 → 9 694 accepted files).
  - [x] **ROOT CAUSE (WHY + WHERE)** — `stimuli/sv/corpus_rule_coverage.py:266` (and
    `:261`) apply `Path(p).relative_to(ROOT)` to results.tsv column 3, which is
    repo-root-relative since `CORPUS-GRAD-ALL.2.1` (`db5a640e`, `git log -S`) and
    therefore not under ROOT as an absolute path. Third consumer of that one convention;
    the failure lands at report-emit time, after the whole measurement has been paid for.
  - [x] **FIX** — one `repo_relative()` helper normalizing both spellings at a single
    site, replacing 2 call sites; rationale in-script naming all three consumers. Tier:
    repo-script. `ast.parse` clean; helper unit-checked on relative / absolute-in-repo /
    absolute-foreign inputs.
  - [x] **ADDRESSED (verified)** — the run completes (guard `guard.3788.marker`, exit 0,
    171 s, peak 6 753 MB): `coverage[sv_2017]: 1248/1352 = 92.3% (104 gaps, 123
    na_profile, 9693 files, 0 timeout-excluded, 1 disagreements)`; the clause instrument
    completes with its headline reproduced exactly (2 529 / 852 / 184 / 0 chapter gaps /
    9 keyed negatives).
  - [x] **NO REGRESSION** — the gap set is a strict SUBSET of the previous one: **16
    closed, 0 new** (set difference on the per-rule TSVs), so nothing regressed into the
    worklist; `na_profile` 123 unchanged; `clause_coverage.md` byte-identical; no
    grammar, codegen, generated artifact or parser surface touched — the only code change
    is a report-path formatter, and the 1 `disagreements` row is the pre-existing
    `ExponTimeIfElseGen` lane, now surfacing there instead of as a timeout.
  - [x] **LOCKSTEP** — both coverage reports + per-rule/per-clause TSVs regenerated;
    `.9`'s worklist re-cut to 104; `.7`/`.7a` headline annotated as superseded;
    tree/TASK_TREE/MEMORY/CHANGES/DEVELOPMENT_NOTES this commit.

### `.8` — ADD-v1 corpus vendoring (the director-ordered acquisition)

- **Status: `done`** — `.8a` (vendoring + runner fold + answer keys +
  fresh baseline) **done**; `.8b` (deep answer-key extraction: `.8b.1`
  mechanical lanes + `.8b.2` 42 named-residue pins + `.8b.3` 283 clustered
  CE stage pins) **done** — the sv_2017-lane answer keys are COMPLETE (all
  `.8a` deferred key populations drained); `.8c.1` (v2005 lane runner +
  mechanical keys + the first measured v2005 baseline: 135 unexplained)
  **done**; `.8c.2` (176 vlg-CE stage pins + the 17 KNOWN_TEXT_BUGS
  re-adjudications ⇒ v2005 baseline honestly re-based 135 → 150, triage
  DRAINED 176 → 0 — the v2005-lane answer keys are COMPLETE) **done**;
  `.8c.3` (the recursion-ceiling-must-bound-the-real-stack engine fix:
  the dedicated 256 MiB parse stack at the three integration/instrument
  boundaries; the v2005 crash row → match, 150 → 149) **done**.
  Roster-v2 candidate logged: **slang embedded-unittest
  extraction** (thousands of SV snippets inside slang's C++ unit tests — the
  sharpest open conformance oracle; extraction tool + fragment entry-point
  mapping required).

#### `.8a` — Vendor the six ADD-v1 corpora + uvm-core fold + adjudicated re-baseline

- **Status: `done`** (`PGEN-SV-CORPUS-GRAD-0007`, session #192, 2026-07-22).
- **Vendored (pinned sparse shallow submodules, blob-filtered partial clones —
  `git clone --depth 1 --filter=blob:none --sparse` + `sparse-checkout set` +
  `submodule add` + `absorbgitdirs`; licenses + roles in PROVENANCE.md):**
  | submodule | pin | sparse | SV files | key |
  |---|---|---|---|---|
  | `ispras-sv-tests` | `f9062e68` | `ieee-1364-2005/`+`ieee-1800-2012/` | 1,266 | `// ! TYPE: POSITIVE\|NEGATIVE\|VARYING` (1,124/23/117) |
  | `iverilog` (ivtest) | `a4989d02` | `ivtest/` | 3,799 | `regress-sv.list` 992 entries (normal/CE/CO × gold) |
  | `sv2v` | `6662fa5d` | `test/` | 953 | dir semantics (`error/` negatives, `.v` goldens) |
  | `Surelog` | `d21c1c70` | `tests/` | 828 | dir-level units + golden logs (extraction = `.8b`) |
  | `black-parrot` | `f91010f6` | `bp_*` | 205 | design corpus (macro-heavy; `external/` NOT initialized — dedupe-by-true-upstream + basejump license) |
  | `opentitan` | `720d7242` | `hw/` | 3,983 | design corpus (UVM-scale; 237 MB via blob-filter) |
  - **uvm-core** (`stimuli/sv/uvm/`, plain tracked files, pre-submodule
    vintage) folded into the bulk universe as sub-corpus `uvm-core` (runner
    `EXTRA_DIRS` + label mapping — 174 files), per the director's order.
- **Fresh guarded characterization at HEAD (16,336 files, exit 0):**
  9,361 pass / 6,966 fail / 9 timeout (57.3%). The four June suites +
  designs are **byte-coherent with the `.3.1` numbers** (sv-tests 832 /
  verilator 2,011 / verible 121 / slang 71 / VeeR 18 / scr1 7 / friscv 31 —
  zero regression, same probe vintage). New keyed suites: ispras 78.7% /
  ivtest 83.2% / Surelog 80.8% / sv2v 75.1%; design corpora read low pending
  chaining (opentitan 17.3% / black-parrot 7.8% / uvm-core 11.5%). The 9
  timeouts are named in the manifest (8 opentitan autogen xbar/pinmux giants
  + the tracked verilator `t_math_synmul_mul.v`).
- **Adjudicator extended (answer keys spec/metadata-only, per doctrine):**
  ispras `// ! TYPE:` header keys (POSITIVE → must_accept; NEGATIVE → stage
  triage `.8b` — the sampled NEGATIVE is SEMANTIC invalidity, so no blanket
  must_reject; VARYING → impl-varying lane; `ieee-1364-2005/` → the
  verilog_2005 profile lane); ivtest `regress-sv.list`/`regress-vlg.list`
  logical-entry parser (backslash-continuations joined; CE stage split
  mirrors the verilator convention — golden `syntax error` = must_reject,
  golden post-parse-only = must_accept, no golden = stage triage `.8b`);
  sv2v dir semantics (`.sv` inputs must_accept by suite contract, `.v`
  goldens → v2005 lane, `error/` → conversion-vs-parse pre-triage `.8b`);
  Surelog → chained dir-level units (key extraction `.8b`); opentitan /
  black-parrot / uvm-core → DESIGN_SUITES chained lane. Deferral-slug
  mechanism added (`out_of_scope_with_cause:<slug>` → `deferred:<slug>`);
  historical labels byte-stable.
- **⭐ THE RE-BASED HONEST BASELINE: 445 unexplained divergences = 439
  rejects-valid + the same 6 named accepts-invalid.** Old-suite continuity
  EXACT: sv-tests 39 / verilator 226 / verible 13 / slang 1 = **279 — the
  `.3.1` baseline reproduced byte-for-byte** by the extended adjudicator.
  ADD-v1 keyed suites contribute **+166 newly measured defect signal**:
  ispras-1800 **114** (clause-keyed POSITIVE rejects — the sharpest new
  worklist, clauses 4/5/6/8/… per the manifest), sv2v **31**, ivtest **21**.
  Full picture: 4,419 match / 1,308 explained-with-cause (svpp macro 952 /
  conditional 207 / include 140 / timeout 9) / 10,164 deferred
  (chained_only 5,896; v2005_profile_lane 2,458; no_sv_key 1,130;
  negative_stage_triage 201; error_pretriage 234; impl_varying 90;
  svpp_owned 155). Manifest determinism proven (two runs byte-identical);
  representative new rows probe-verified live (ispras `04.05_01.sv` /
  `05.07.01_04.sv` — TYPE POSITIVE, exit 1 at HEAD).
- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — the director's acquisition order
    ([[project_sv_corpus_100pct_lrm_coverage_mandate]]): the corpus lacked
    the frozen-roster ADD-v1 tier; the negative/clause-keyed axes were the
    thinnest (banked corpus-sufficiency assessment).
  - [x] **ROOT CAUSE (WHY + WHERE)** — N/A (acquisition/measurement leaf);
    every new expected verdict carries its per-row `basis` provenance.
  - [x] **FIX** — N/A (no parser change; corpus + tooling only).
  - [x] **ADDRESSED (verified)** — six pinned submodules on disk + fresh
    16,336-file guarded characterization (exit 0) + extended adjudicator
    manifest (determinism cmp ×2; probe spot-verification of new rows).
  - [x] **NO REGRESSION** — zero parser surface touched; the four June
    suites' pass counts byte-identical to `.3.1`; the old-suite unexplained
    population reproduces EXACTLY (279); historical adjudication labels
    byte-stable.
  - [x] **LOCKSTEP** — PROVENANCE + tree + TASK_TREE index +
    MEMORY/CHANGES/DEVELOPMENT_NOTES/LIVE this commit.

#### `.8b` — Deep answer-key extraction (umbrella; COMPLETE)

- **Status: `done`** — the five populations from `.8a`, split:
  `.8b.1` (mechanical metadata lanes 1/2/5) **done**; `.8b.2` (per-file
  pinned stage adjudication: ispras NEGATIVE 21 + the sv2v named-ambiguous
  residue 21) **done**; `.8b.3` (the CE-without-gold stage-triage
  population, 283 ivtest rows, clustered stage pins) **done** — every
  deferred answer-key population from `.8a` is now drained
  (`error_pretriage` 0, `negative_stage_triage` 0).

##### `.8b.1` — Mechanical deep keys: Surelog goldens + sv2v patterns + ivtest vvp_tests (done)

- **Status: `done`** (`PGEN-SV-CORPUS-GRAD-0009`, session #193, 2026-07-22).
- **What landed** (all in `stimuli/sv/adjudicate_external_corpus.py`,
  spec/metadata-only per doctrine; no parser surface touched):
  - **`SurelogIndex`** — unit = a `tests/` dir directly holding `.sl`
    drivers; keyable iff single-source (recursive), no out-of-unit refs
    (`-y/-v/-f/-map/-cfg/-batch`, `..` paths), and committed golden log(s)
    present: `[SNT:]` → must_reject intent, clean completed log →
    must_accept (upstream 1800-2017 parse testimony), `[FTL:]`/no-log/multi
    → chained with named sub-cause. Yield: 628 keyable units of 700.
  - **sv2v `test/error/` stage classification** — the `// pattern:` upstream
    keys stage-classified per the IEEE 1800-2017 LRM (in-repo md, BNF vs
    prose): 64 `must_reject` (BNF violations — cites per group: A.6.10 `#0`,
    A.6.3 seq_block ordering, A.6.7 case-inside literal `case`, A.2.2.1
    data_type modifiers, A.2.2.2 strength pairs, A.4.1.1 ordered/named
    mixing, EOF truncations, lexical), 123 `must_accept` (prose-"shall"
    semantics — end-labels 9.3.4/23.2.1, resolution/bindings, value rules,
    jump placement 12.8, multiple case defaults 12.5; incl. the two
    1364→1800 BNF relaxations `charge_strength_non_trireg` +
    `drive_strength_uninit`), 25 preproc-stage → svpp lane; 21 named
    residue (no `// pattern:` key or stage-ambiguous: `lhs_*`,
    `export_outside_package_*`, `localparam*_no_default`,
    `decl_const_var_uninit`, `parameter_list_not_type`, `severity_task_*`,
    `line_*`, `interface_excess_ports`/`_non_lhs`/`_missing_direction`,
    `dangling_stmt`, `include_apos`) → `.8b.2`. Table exhaustiveness
    machine-audited (212 classified + 22→21 residue = 234, zero stale).
  - **ivtest vvp_tests JSON secondary keys** — descriptors (the upstream
    `vvp_reg.py` system: `type`/`source` under `ivltests/`/`iverilog-args`/
    `gold` as `gold/<g>-iverilog-<chan>.gold`) key ONLY rows with an
    explicit SV generation flag; multi-descriptor sources keyed only on
    verdict agreement; AMS runs → `verilog_ams_lane`; explicit plain-Verilog
    generations → v2005 lane; no-generation descriptors stay `no_sv_key`
    honestly (upstream default generation not encoded).
  - **Generic demotion** (mirror of the `include rule): a `must_reject` on a
    macro/conditional-dependent file demotes to svpp-owned — a raw-text
    reject would testify for the wrong reason (fired exactly once:
    `Surelog/tests/PreProcMacro`, a macro-torture SNT unit).
- **Measured globally (before → after, committed manifest vs regenerated):**
  16,336 rows both; **baseline 445 → 550 unexplained (+105 newly measured:
  Surelog 70 / ivtest +17 / sv2v +18 = 537 rejects-valid + 13
  accepts-invalid)**; match 4,419 → 5,260 (+841); deferred 10,164 → 9,091.
  **Full row-by-row transition audit: every class change ∈ the intended
  transition set** (Surelog chained→{536 match, 70 rejects-valid, 22
  svpp-explained, 1 svpp-owned}; ivtest no_sv_key→{135 match, 17
  rejects-valid, 104+1 svpp-explained, 103 CE-triage, 20 AMS, 6 NI, 1
  v2005}; sv2v error_pretriage→{170 match, 11 rejects-valid, 7
  accepts-invalid, 25 svpp}); ZERO collateral movement in any other suite
  (old-suite 279-baseline populations byte-stable). ⭐ The 7 sv2v
  accepts-invalid rows are a NEW over-acceptance worklist (parser accepts
  BNF-invalid text: `decl_after_stmt`, `decl_bare`, `decl_ranged_implicit`,
  `decl_signed_implicit`, `block_start_3`, `auto_dim_int`,
  `block_comment_eof` — the last = unterminated block comment accepted,
  a lexer-tolerance candidate for `.3.x`).
- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — 828 Surelog + 234 sv2v-error + 1,130 ivtest
    no_sv_key rows carried no expected verdicts (`.8a` deferral slugs).
  - [x] **ROOT CAUSE (WHY + WHERE)** — N/A defect-wise (key-extraction
    leaf); every new verdict carries per-row `basis` provenance naming its
    upstream key (golden log name, LRM cite, descriptor JSON).
  - [x] **FIX** — N/A (no parser change; adjudicator tooling only).
  - [x] **ADDRESSED (verified)** — determinism cmp ×2 byte-identical;
    table-exhaustiveness audit; full transition audit (above); live probe
    spot-verification of representative new rows (`Surelog/tests/1364_2005/
    dut.v` exit 1, `ivtest/ivltests/br_gh1321.v` exit 1,
    `sv2v test/error/auto_dim_int.sv` exit 0 = accepts-invalid,
    `drive_strength_uninit.sv` exit 1 = rejects-valid — all four match the
    manifest).
  - [x] **NO REGRESSION** — zero parser surface touched; `results.tsv`
    unchanged (same probe vintage); old-suite unexplained populations
    reproduce EXACTLY (verilator 226 / sv-tests 39 / verible 13 / slang 1 /
    ispras 114); historical labels byte-stable outside the three intended
    populations.
  - [x] **LOCKSTEP** — tree + TASK_TREE index + MEMORY/CHANGES/
    DEVELOPMENT_NOTES/LIVE this commit.

##### `.8b.2` — Per-file pinned stage adjudication (done)

- **Status: `done`** (`PGEN-SV-CORPUS-GRAD-0010`, session #193, 2026-07-22).
- **What landed**: all 42 rows read and pinned with LRM-grounded rulings
  (`ISPRAS_NEGATIVE_PINNED` 21 + `SV2V_PINNED` 21 in the adjudicator; every
  basis cites its Annex-A production or prose clause):
  - **ispras NEGATIVE (21)**: 17 semantic-stage → must_accept (reg-on-net
    6.7.1 prose, 6.21 lifetime-keyword prose, casts/traversal/alias/config/
    checker/defparam/covergroup/name-conflict semantics, duplicate named
    connections 23.3.2.2, return-in-fork placement prose); 4 parse-level →
    must_reject: `13.05.02_02` (`ref input` — A.2.7 single
    tf_port_direction), `16.09.04_01` (⭐ the committed text itself has an
    unclosed parenthesis — upstream typo; committed-text-over-intent),
    `22.14.01_02` (`logic` reserved under BOTH the `begin_keywords
    1800-2005 set and bare sv_2017), `22.14.01_04` (under 1364-2005
    keywords the items match no production — the F5 directive family's
    reject side).
  - **sv2v residue (21)**: 7 must_reject (naked module-level statement
    A.1.4/A.6.1; `(1 = x)` — operator_assignment needs a variable_lvalue;
    type_assignment `= 1` A.2.4; `localparam X;`/`localparam type X;` —
    **A.10 footnote 18**: omission legal only within a parameter_port_list
    and never for localparam; `$fatal x;`; module-scope `export` —
    package_export_declaration is a package_item ONLY per A.1.11), 9
    must_accept (const-without-init — 6.20.6 is prose-only; streaming/
    pattern LHS — A.8.5 variable_lvalue admits streaming_concatenation via
    A.6.1's variable alternative, inner lvalue-ness 11.4.14 prose; iface
    arity/direction/lvalue semantics; `$fatal(.x(...))` — parseable as a
    generic A.8.2 system_tf_call, the 20.10 shape is semantic-only;
    top-level `export` — a legal A.1.2 $unit package_item, prose
    restriction), 5 preproc → svpp (`include quoting 22.5, `line argument
    validity 22.12).
- **Measured (before → after)**: baseline **550 → 557 unexplained (540
  rejects-valid + 17 accepts-invalid)**; match 5,260 → 5,290 (+30);
  `error_pretriage` drained 21 → **0**; `negative_stage_triage` 304 → 283
  (pure ivtest now). The +7 new signal: ispras `22.14.01_04`
  accepts-invalid (directive-unaware nested-interface acceptance — the F5
  family's first measured accepts-invalid side) + `06.07.01_02`/
  `08.26.04_01` rejects-valid (parser stricter than BNF on reg-as-net-type
  and forward-typedef implements) + sv2v `dangling_stmt` +
  `localparam_no_default` ×2 accepts-invalid (⭐ naked module-level
  statement and defaultless localparam ACCEPTED — over-acceptance
  worklist grows to 17) + `severity_task_arg` rejects-valid.
- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — 42 rows deferred with named ambiguity by
    `.8b.1`/`.8a` (no upstream stage encoding).
  - [x] **ROOT CAUSE (WHY + WHERE)** — N/A defect-wise; every pin's basis
    names its LRM production/clause (BNF-vs-prose split re-derived from
    `docs/systemverilog/2017/md`, incl. A.10 fn-18 read verbatim).
  - [x] **FIX** — N/A (adjudicator pin tables only).
  - [x] **ADDRESSED (verified)** — determinism cmp ×2; pin-table
    staleness audit (all pinned files exist; sv2v table exhaustive
    234/234); transition audit: exactly the 42 rows moved, zero
    collateral, zero basis-only drift; probes ×3 confirm
    (`dangling_stmt` exit 0, `06.07.01_02` exit 1, `22.14.01_04` exit 0).
  - [x] **NO REGRESSION** — zero parser surface touched; `results.tsv`
    unchanged; all other populations byte-stable.
  - [x] **LOCKSTEP** — tree + TASK_TREE index + MEMORY/CHANGES/
    DEVELOPMENT_NOTES/LIVE this commit.

##### `.8b.3` — CE-without-gold stage triage (done)

- **Status: `done`** (`PGEN-SV-CORPUS-GRAD-0011`, session #194, 2026-07-22).
- **Method (clustered spec-side, per the leaf charter):** all 283 files
  read in full (avg 15 lines; full-text digest banked in-session), clustered
  by the construct under test, and every cluster adjudicated against the
  IEEE 1800-2017 Annex A BNF + its normative footnotes (in-repo
  `docs/systemverilog/2017/md/section-41-data-read-api.md` — the Annex A
  dump; ~14 load-bearing productions/footnotes re-verified verbatim before
  pinning: A.2.5 dimensions, A.2.2.1 data_type/struct_union/enum_base_type,
  A.8.2 list_of_arguments, A.1.9 class_constructor_declaration, A.1.7
  program items, A.4.2/A.1.11 generate chain, A.6.8
  for_variable_declaration, A.2.1.2 inout_declaration, A.1.3
  parameter_port_declaration, A.2.4 defparam/net_decl_assignment,
  constant_primary's missing `$` alternative, footnotes 10/15/18/20).
- **What landed:** `IVTEST_CE_STAGE_CLUSTERS` in the adjudicator — 46
  clusters (8 reject / 38 accept) flattened to the per-file
  `IVTEST_CE_STAGE_PINNED` table (import-time audits: exactly 283 unique
  keys, duplicate refusal), consulted at BOTH triage sites (regress-list
  CE-without-gold and vvp_tests CE-without-usable-golden); residual
  fall-through basis re-worded for upstream-added rows.
  **Verdict split: 243 must_accept / 40 must_reject.** Key rulings:
  - **fn 20 is normative syntax** (the `.8b.2` footnote law extended):
    unsized `[]` packed dimension is legal ONLY as a DPI import's sole
    packed dimension ⇒ `reg [] x;` parse-level reject.
  - **fn 10 both halves**: `automatic` in a non-procedural
    data_declaration + implicit-type declarations without `var` ⇒
    package `automatic int x;` / `x;` / `[3:0] x;` parse-level reject.
  - **A.8.2 order law**: positional-then-named argument order IS BNF-legal
    (only the 5 named-then-positional `*_fail4` rows reject) — the tf-call
    mirror of the `.8b.1` A.4.1.1 module-connection mixing ruling.
  - **fn 15 ruled denotation-dependent** ⇒ semantic stage: type_identifier
    enum bases (9 rows) stay parse-accepts even where fn 15 makes them
    illegal, because legality turns on what the identifier denotes.
  - **Edition law (opposite face of `.8b.1`'s 1364→1800 rule):** 2023-only
    syntax judged in the sv_2017 lane ⇒ `union soft` + the 8
    `parameter type class/enum/struct/union` rows reject.
  - **Escape-hatch law reaffirmed:** module instantiation inside a program
    parses as A.6.10 checker_instantiation (program5b must_accept).
- **Measured (before → after, committed manifest vs regenerated):**
  16,336 rows both; **baseline 557 → 564 unexplained (543 rejects-valid +
  21 accepts-invalid)**; match 5,290 → 5,566 (+276);
  `negative_stage_triage` **283 → 0 — every `.8a` deferred answer-key
  population is now drained**. Row-by-row transition audit: **exactly the
  283 triage rows moved, zero collateral** (276 → match, 4 → new
  accepts-invalid, 3 → new rejects-valid). The +7 new signal, all named:
  accepts-invalid `br_ml20181012b` (`reg [];` accepted — fn 20),
  `parameter_no_default_fail2` (defaultless body `parameter` accepted —
  fn 18, joins the `.8b.2` defaultless-localparam over-acceptance family),
  `sv_class_constructor_fail` (non-ANSI `input x;` inside `function new`
  accepted), `sv_package_lifetime_fail` (package `automatic` accepted —
  fn 10; over-acceptance worklist 17 → 21); rejects-valid `program5b`
  (checker-instantiation form rejected), `sv_class_new_typed_fail4`
  (`T::new` scope rejected), `sv_void_cast_fail3` (`void'(1+2)` rejected).
- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — 283 CE rows carried intent but no stage
    (the `.8a`/`.8b.1` deferral slugs); the graduation bar cannot count
    them without spec-side verdicts.
  - [x] **ROOT CAUSE (WHY + WHERE)** — N/A defect-wise (key-extraction
    leaf); every pin's basis names its Annex A production or normative
    footnote, re-verified verbatim from the in-repo LRM md before pinning.
  - [x] **FIX** — N/A (adjudicator pin tables only; no parser surface).
  - [x] **ADDRESSED (verified)** — import-time table audits (283 keys,
    duplicate refusal) + on-disk staleness audit (all 283 pinned files
    exist) + key-set equality against the manifest population; determinism
    cmp ×2 byte-identical; full transition audit (exactly-283, zero
    collateral); probes ×6 confirm representative rows live
    (`br_ml20181012b` exit 0, `program5b` exit 1, `sv_void_cast_fail3`
    exit 1, `generate_module` exit 1, `sv_const_fail1` exit 0,
    `sv_named_arg_task_fail4` exit 1 — all matching the manifest).
  - [x] **NO REGRESSION** — zero parser surface touched; `results.tsv`
    unchanged (same probe vintage); all non-triage populations byte-stable
    (audited row-by-row).
  - [x] **LOCKSTEP** — tree + TASK_TREE index + MEMORY/CHANGES/
    DEVELOPMENT_NOTES/LIVE this commit.

#### `.8c` — The verilog_2005 profile lane (`.8c.1` done; `.8c.2` todo)

- **Status: `in_progress`** — the 2,459 `deferred:v2005_profile_lane` rows
  (ispras `ieee-1364-2005/` 356 + ivtest `regress-vlg.list`/plain-Verilog
  vvp descriptors 1,762 + sv2v goldens 341) are the frozen-roster v2005
  corpus, feeding the v2005 arm of `.5`/`.7`.

##### `.8c.1` — Lane runner + mechanical v2005 keys + the first measured v2005 baseline (done)

- **Status: `done`** (`PGEN-SV-CORPUS-GRAD-0012`, session #194, 2026-07-22).
- **What landed:**
  - Runner `sv2005` mode (`stimuli/run_external_corpus.sh`): re-parses ONLY
    the adjudicator-emitted lane list (`v2005_lane_files.tsv` — single
    source of lane membership, no duplicated derivation) under
    `--profile verilog_2005` → `results_v2005.tsv` +
    `characterization_v2005.md`.
  - **Instrument fix (found live): probe signal-death is now its own
    `crash` status** (rc ≥ 128; previously silently folded into `fail`) and
    `divergence:unexplained_crash` — a defect class that is NEVER explained
    away regardless of the expected verdict.
  - Adjudicator v2005 arm: lane-list emission + `--results-v2005` intake
    (subset/completeness audits against the lane derivation, stale-run
    refusal) + IEEE 1364-2005 answer keys: ispras TYPE headers (POSITIVE →
    must_accept with the KNOWN_TEXT_BUGS residue reserved to `.8c.2`; the
    only 2 NEGATIVE files read + pinned — both semantic-stage 12.3.3 port
    signedness / 12.8.2 defparam-resolution → must_accept; VARYING →
    impl-varying lane), ivtest `regress-vlg.list` types with the CE golden
    syntax-error split + plain-Verilog vvp descriptors (agreement-gated),
    sv2v `.v` conversion-golden contract → separate
    `adjudication_manifest_v2005.tsv` + `adjudication_summary_v2005.md`
    (the main sv_2017 manifest stays BYTE-IDENTICAL — proven).
- **⭐ THE FIRST MEASURED V2005 BASELINE: 2,459 rows — 1,947 match /
  135 UNEXPLAINED (133 rejects-valid: ivtest 106 / ispras-1364 17 / sv2v
  11 (per-suite: ivtest 107 incl. the crash) + 1 accepts-invalid + 1
  crash) / 172 svpp-explained / 205 deferred (176 vlg-CE-without-gold →
  `.8c.2`; 27 impl-varying; 2 chained).** 85.6% raw pass under the strict
  verilog_2005 profile.
- **⭐ NAMED ENGINE FINDING (tool-pinned, tracked follow-up):**
  `ivtest/ivltests/br_gh330.v` (a ~600-line chained-ternary torture file,
  vlg-type `normal`) KILLS the debug-build probe with a REAL stack
  overflow (`thread 'main' has overflowed its stack`, abort rc 134) under
  BOTH profiles, while the release probe parses it exit-0 — the
  [[feedback_recursion_ceiling_must_bound_real_stack]] class: the
  recursion ceiling is not bounding the real stack in debug builds. The
  committed sv_2017 `results.tsv` recorded this same crash as a plain
  `fail` (lane-deferred there, so the 564 baseline is uncontaminated); the
  runner now separates the classes going forward. Engine-side graceful
  refusal = a follow-up leaf owned by this tree.
- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — 2,459 lane rows had no v2005-profile
    observations or verdicts; the `.5` gate's v2005 arm was unmeasured.
  - [x] **ROOT CAUSE (WHY + WHERE)** — the crash row tool-pinned (debug
    stack overflow, both profiles, release exit-0; stderr banked in-tree);
    key rows carry per-row basis provenance.
  - [x] **FIX** — instrument-level only (runner crash status + adjudicator
    crash class + v2005 arm); zero parser surface.
  - [x] **ADDRESSED (verified)** — guarded run (exit 0, 20 s, peak 550 MB);
    determinism cmp ×2 on manifest + summary; main-manifest byte-identity
    proven (git diff empty); lane-list/results subset+completeness audits;
    probes ×3 under `--profile verilog_2005` match the manifest
    (`test_03_05_01_2.v` exit 1, `test/lex/line.v` exit 1, `br_gh1174a.v`
    exit 1) + the crash matrix (debug 134 / release 0).
  - [x] **NO REGRESSION** — the sv_2017 manifest/summary byte-identical;
    `results.tsv` untouched; runner sv/vhdl paths unchanged apart from the
    additive crash status (no existing row class changes until the next
    full re-characterization).
  - [x] **LOCKSTEP** — tree + TASK_TREE index + MEMORY/CHANGES/
    DEVELOPMENT_NOTES/LIVE this commit.

##### `.8c.2` — v2005 CE-without-gold stage pins + KNOWN_TEXT_BUGS re-adjudication (done)

- **Status: `done`** (`PGEN-SV-CORPUS-GRAD-0013`, session #195, 2026-07-22).
  The engine follow-up (c) is split out as leaf `.8c.3` (one commit = one
  defect); this leaf = the answer-key work (a)+(b).
- **What landed (adjudicator keys only — zero parser surface):**
  - **(a) `IVTEST_VLG_CE_STAGE_CLUSTERS`** — all 176
    `negative_stage_triage_v2005` rows (175 vlg-list CE-without-gold + 1
    vvp-descriptor triage row) read per-file and stage-pinned against the
    IEEE 1364-2005 Annex A BNF, re-verified VERBATIM from the in-repo
    `docs/verilog/2005/md` full Annex A dump before pinning (+ the
    section 19 directive rules for the svpp pair). 30 clusters:
    **146 must_accept / 28 must_reject / 2 svpp-owned (directive-stage)**;
    flatten audits count (exactly 176) + duplicates; consulted at BOTH
    triage points (vlg CE-without-gold + vvp `triage` descriptors), never
    overriding a golden-backed key.
  - **(b) `ISPRAS_1364_POSITIVE_PINNED`** — the 17 failing ispras-1364
    POSITIVE rows re-adjudicated against the suite's `KNOWN_TEXT_BUGS`
    errata: **ZERO errata flips** — no committed text embodies an LRM typo
    that renders it BNF-invalid. ⭐ The one errata candidate
    (`PATHPULSE$ = 3;`, 14.6.1) is ruled STILL PARSEABLE: A.2.4
    `specparam_assignment` keeps the plain
    `specparam_identifier = constant_mintypmax_expression` alternative,
    A.9.3 `simple_identifier` admits `$`, and **Annex B does NOT reserve
    `PATHPULSE$`** — the identifier escape-hatch law (the A.8.2
    system_tf_call mirror). All 17 stay must_accept = measured v2005
    defect signal (per-file pinned bases: 3.5.1 spaced/signed literals ×5,
    1364 strength grammar ×4, UDP bodies ×3, scalared/vectored net,
    mintypmax parameter, PATHPULSE$, $width/edge-control,
    `begin_keywords).
  - **Key 1364-vs-1800 laws pinned (the edition law's v2005 face):**
    empty tf-call parens illegal (A.8.2/A.6.9 require >= 1 expression);
    unnamed-block declarations illegal (A.6.3); a task body holds exactly
    ONE statement_or_null (A.2.7); ANSI input/inout ports take neither
    defaults (A.2.3) nor variable types (A.2.1.2); port_declaration
    illegal in a portless module (A.1.2 second form); parameter/specparam/
    specify illegal in generate scope (A.1.4/A.4.2); gate terminals are
    never empty and buf/not need >= 2 (A.3.1/A.3.3); `size ::=
    non_zero_unsigned_number` (A.8.7); constant contexts admit NO
    hierarchical names (A.8.4); mixed ordered/named connections illegal
    (A.4.1). Accept-side mirrors: trailing null port legal (A.1.3 `port`
    may be empty); `#(...)` after an identifier is denotation-blind
    module-shaped instantiation (UDP-vs-module = link stage); `$clog2` in
    parameters = A.8.4 constant_system_function_call (1364-2005-added);
    directive placement unrestricted (section 19 "may appear anywhere" —
    `no_timescale_in_module` is iverilog strictness, NOT a 1364 error).
- **⭐ MEASURED RESULT — the v2005 baseline is HONESTLY RE-BASED 135 → 150
  unexplained (135 rejects-valid / 14 accepts-invalid / 1 crash);
  `negative_stage_triage_v2005` DRAINED 176 → 0 — the v2005-lane answer
  keys are COMPLETE** (match 1,947 → 2,105; deferred 205 → 32 = 27
  impl-varying + 2 chained + 3 svpp-owned incl. the generic macro
  demotion on `module_input_port_list_def`). The +15 signal is fully
  named: **2 rejects-valid** (`always3.1.2I` — the spaced `5'h 0` 3.5.1
  literal, the ispras chapter-3.5.1 cluster's ivtest face;
  `no_timescale_in_module` — in-module-body directive tolerance, the F5
  directive family's v2005 face) + **13 accepts-invalid — the FIRST
  measured v2005 over-acceptance worklist** (`^=` compound assignment
  accepted (br1015a), `0'b0` zero-size literal (br_gh60a), empty tf-call
  parens ×3 (function4, task_nonansi_fail5/8), ANSI port defaults
  (module_inout_port_list_def), variable-typed input/inout ports ×2,
  parameter-in-generate, `dut.WIDTH` in a constant (pr2792883),
  two-statement task body (task_port_range_mismatch), unnamed-block/fork
  declarations ×2).
- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — 176 rows deferred
    (`negative_stage_triage_v2005`) + 17 ispras POSITIVE rejects carried
    only the generic re-adjudication note; the v2005 answer keys were
    incomplete.
  - [x] **ROOT CAUSE (WHY + WHERE)** — every pin carries its Annex A
    production / section-19 rule / errata citation, re-verified verbatim
    from the in-repo 1364-2005 md before pinning (the full Annex A dump
    read end-to-end this session).
  - [x] **FIX** — answer-key tables + two guarded lookup sites in
    `stimuli/sv/adjudicate_external_corpus.py`; zero parser surface.
  - [x] **ADDRESSED (verified)** — mechanical set-equality: pin table ==
    the 176-row population EXACTLY (missing=[], extra=[]); row-by-row
    transition audit: changed == triage ∪ ispras-rejects == 193 rows,
    ZERO collateral (158→match / 13→accepts-invalid / 2→rejects-valid /
    3→svpp-deferred / 17 basis-only); determinism cmp ×2 on manifest +
    summary; probes ×6 under `--profile verilog_2005` confirm both sides
    (always3.1.2I exit 1, no_timescale_in_module exit 1, br1015a exit 0,
    unnamed_block_var_decl exit 0, task_nonansi_fail5 exit 0, br_gh60a
    exit 0).
  - [x] **NO REGRESSION** — the main sv_2017 manifest + summary
    BYTE-IDENTICAL (cmp + empty git diff; totals 5566/564/1435/8771
    unchanged); v2005 svpp-explained 172 and impl-varying 27 byte-stable;
    the 308 passing ispras POSITIVE rows keep their basis byte-stable.
  - [x] **LOCKSTEP** — tree + TASK_TREE index + MEMORY/CHANGES/
    DEVELOPMENT_NOTES/LIVE this commit.

##### `.8c.3` — the recursion ceiling must bound the REAL stack (DONE)

- **Status: `done`** — measurement + design (`PGEN-SV-CORPUS-GRAD-0014`,
  session #195, read-only; evidence
  `docs/tasks/artifacts/sv_corpus_grad/8c3_stack_ceiling_measurement.md`);
  **implementation LANDED (`PGEN-SV-CORPUS-GRAD-0015`, session #196;
  evidence `docs/tasks/artifacts/sv_corpus_grad/8c3_implementation_verification.md`
  + `8c3_oracle_run.txt`)**.
- **What landed (the banked design, executed exactly):** new shared module
  `rust/src/dedicated_parse_stack.rs` (`DEDICATED_PARSE_STACK_BYTES =
  256 MiB`; `run_on_dedicated_parse_stack` spawn-per-call + panic-capture +
  re-entrant-inline; `run_cli_main_on_dedicated_parse_stack` CLI wrapper;
  5 unit tests incl. a 32 MiB-deep recursion proof). Routed at exactly the
  three designed boundaries: (1) embedding-API SV/VHDL family entries
  (`embedding_api.rs` — worker panic → `E_PARSE_FAILURE`; spawn-per-call
  preserves host parallelism, ~50–100 µs noise vs ms-scale HDL parses;
  `EMBEDDING_API_VERSION` `1.3.0` → `1.3.1`); (2) `parseability_probe`
  whole-main wrap; (3) `ast_pipeline` whole-main wrap (both binary
  configs). **The regex path is byte-untouched** (keeps its RGX-0085
  worker + nesting pre-check — zero perf-floor risk; no parse-loop code
  changed anywhere; the shared 4096 ceiling constant untouched). +2
  embedding regression-lock tests (deep-parens SV ×2 profiles, deep VHDL)
  that run on 2 MiB libtest threads — they pass only if the routing is
  real.
- **ACCEPTANCE CHECKLIST (task-acceptance procedure):**
  - [x] **ROOT CAUSE (WHY + WHERE)** — tool-pinned in the `-0014`
    measurement slice (ulimit × build-mode × depth bisect matrix,
    `8c3_stack_ceiling_measurement.md`): the 4096-frame ceiling
    (`GENERATED_RECURSION_GUARD_MAX_DEPTH`,
    `ast_based_generator.rs:31`) needs ≈8 MB (release, ≈2 KB/frame) /
    ≈70 MB (debug, ≈17 KB/frame) of REAL stack for SV — more than the
    8 MB default main stack — so the OS guard page fired first (rc 134
    SIGABRT): deep-parens N=2000 crashed BOTH build modes; br_gh330.v
    (real corpus) crashed debug = the v2005 lane's 1 `crash` row.
  - [x] **ADDRESSED (verified, before → after on the symptom)** — the
    16-cell oracle (`8c3_oracle_run.txt`, both modes × both profiles ×
    {N=380, N=500, N=2000, br_gh330}): **ALL PASS — zero signal deaths**
    (pre-fix rc 134 across the matrix); deep-parens N=2000 → clean rc 1
    ceiling rejection in all 4 mode×profile cells (the rejection's
    `furthest_position=702` ≈ 681 parens × ~6 frames/paren ≈ 4096 = the
    ceiling's arithmetic signature; sv_2017 ≈11 frames/paren so its
    boundary sits lower — profile-dependent BY DESIGN); **br_gh330.v
    ACCEPTS rc 0 in all 4 cells** (pre-fix: debug crashed, so debug and
    release now also AGREE cell-by-cell); embedding locks pass on 2 MiB
    libtest threads. **The v2005 crash row re-adjudicated on measurement:
    `crash`/`divergence:unexplained_crash` → `pass`/`match` — v2005
    baseline 150 → 149 unexplained (135 rejects-valid / 14
    accepts-invalid / 0 crash); characterization 2107 pass / 352 fail /
    0 crash.**
  - [x] **NO REGRESSION** — (a) the v2005 lane re-run ×2 + adjudication
    ×2: `results_v2005` verdict-sets sorted-identical across runs, both
    manifests BYTE-IDENTICAL across runs; exactly ONE row changed vs HEAD
    (zero collateral). (b) The MAIN sv_2017 manifest + summary + lane
    list re-derived BYTE-IDENTICAL (totals `5566/564/1435/8771`
    unchanged; the gitignored `results.tsv` raw dump was reconstructed
    losslessly from the tracked manifest, and the adjudicator re-running
    the FULL key derivation over it reproducing the manifest byte-exact
    is the soundness self-check). (c) Full guarded lib battery green
    (1,020 passed / 0 failed / 29 ignored — the prior 1,013 + the 7 new
    locks) + clippy source-strict PASS. (d) Zero hot-path/parse-loop code
    touched; the regex family's code paths unmodified byte-for-byte
    (routing helper is SV/VHDL-cfg-gated).
  - [x] **LOCKSTEP** — embedding contract (`1.3.1` history + new
    Stack-Robustness Contract section), SV + VHDL integration contracts,
    platform book (embedding chapter §Stack robustness), SV + VHDL parser
    books (public-api §Stack robustness), `RUST_CODEBASE_ANALYSIS.md`
    architecture note, tree + TASK_TREE index + MEMORY/CHANGES/
    DEVELOPMENT_NOTES this commit.
- **Honest note (recorded, not hidden):** the over-deep rejection surfaces
  as the generic `Parser did not consume full input …
  furthest_position=…` message — `RecursionDepthExceeded` participates in
  backtracking like any branch failure, so the FINAL surfaced error is
  positional. The class fix is complete (bounded graceful rejection, never
  a process abort); making the ceiling error surface preferentially is
  engine error-priority work — possible future polish, deliberately out of
  this boundary-locus slice.
- **⭐⭐ MEASURED ESCALATION — the `.8c.1` "debug-build robustness" framing
  UNDERSTATED the class: the RELEASE build ALSO hard-aborts.** A ~400-deep
  parenthesized expression (≈4 KB of text) stack-overflows the release
  probe at the default 8 MB main stack (rc 134, uncatchable SIGABRT); the
  `GENERATED_RECURSION_GUARD_MAX_DEPTH = 4096` clean ceiling NEVER fires
  for SV in EITHER build mode (release accepted N=380 parens without
  firing; guard page kills at N≈390 — crash-before-ceiling, the exact
  [[feedback_recursion_ceiling_must_bound_real_stack]] scenario, proven
  for the flagship family in the shipping build mode; a process-abort
  class at the Nexsim embedding boundary).
- **Measured constants (ulimit-controlled, both modes):** per paren level
  release ≈21 KB / debug ≈180 KB (≈10–11 logical frames/level ⇒ ≈2 KB vs
  ≈17 KB per frame); ceiling-real stack need ≈8 MB release / ≈70 MB
  debug. br_gh330 (600 ternaries): debug CRASH at 8 MB → **ACCEPT rc 0 at
  16 MB** (matches release) ⇒ the crash row converts to a match on
  measurement once the fix lands (v2005 150 → 149).
- **Design (banked):** generalize the PGEN-RGX-0085 `GeneratedRegexWorker`
  model — run generated-parser parses at the integration/instrument
  boundaries (`parseability_probe`, `ast_pipeline` CLI drivers,
  embedding-API family entries) on a dedicated **256 MiB-stack** worker
  thread so the EXISTING 4096 ceiling provably fires before the guard
  page in BOTH modes with ≥2× margin. Zero hot-path cost (nothing inside
  the parse loop changes ⇒ zero regex-floor risk); parser-agnostic
  boundary locus; the shared ceiling constant untouched. Rejected: SV
  O(n) nesting pre-check (no sound bracket proxy for ternary chains);
  in-loop remaining-stack check (hot-path cost every family); lowering
  the ceiling (br_gh330 is REAL corpus needing ~1000+ frames).
  Regression oracle: the deep-parens synthetic must yield a graceful
  recursion diagnostic, never rc 134, at every boundary in both modes.

### `.9` — Gap-driven acquisition/crafting loop to 100%

- **Status: `todo`** — ⭐ **worklist RE-CUT at HEAD by `.7c`: 104 uncovered rules**
  (was 120; 16 closed by this campaign's own `.3.x` burn-down, 0 new). The
  per-rule list is `stimuli/sv/characterization/rule_coverage_sv_2017.tsv`
  (`status == GAP`); the clause-axis gaps are the per-chapter NEGATIVES density in
  `clause_coverage.md`, which remains the sharpest structural gap. Every
  `.7`-reported uncovered rule/clause gets a
  corpus case: sourced from the ADD tiers, or crafted directly from the
  in-repo LRM markdown (`docs/systemverilog/2017`/`2023`) with the clause
  cited (externally-grounded, never generator-derived — external means
  externally authored). Loop until 100% of the parseable surface is
  exercised or N/A-with-cause. Feeds `.5`'s widened criterion.

### `.10` — the axis-2 RE-MEASURE at HEAD (the FRESHNESS AUDIT's mandated first act) + the self-dating report

- **Status: `done`** (`PGEN-SV-CORPUS-GRAD-0027`, session #214, 2026-08-08) — the leaf the
  **AXIS-2 FRESHNESS AUDIT** (tree preamble, session #213) ordered: the tracked
  `59.3 %` over 16 336 files was committed **2026-07-25** and
  `grammars/systemverilog.ebnf` last changed **2026-07-26**, so the number
  provably cannot describe HEAD. Nothing in this campaign may be quoted until it
  is re-measured ([[project_all_parsers_fully_pass_stimuli_and_external_corpora]]:
  *"the first honest act is to re-measure them rather than to quote them"*).
- **Scope (deliberately narrow — this is a MEASUREMENT leaf, not a fix leaf):**
  (1) prove the INSTRUMENT is HEAD-vintage before trusting any number it emits;
  (2) re-run `stimuli/run_external_corpus.sh sv` and bank the per-file
  transitions against the preserved baseline; (3) re-adjudicate and re-cluster so
  `.3`'s burn-down worklist is defect signal at HEAD rather than at a July
  vintage; (4) close the staleness defect CLASS at its source so the audit never
  has to be reconstructed from unrelated commit dates again.
- ⛔ **Ground rule inherited from the tree:** characterize FIRST, fix SECOND. Any
  defect this leaf surfaces is ROUTED to a `.3.x` burn-down leaf, never worked
  here — and a rising pass count is NOT by itself evidence of correctness
  ([[a-rising-pass-rate-is-not-evidence-of-correctness]]).

**STEP 1 — INSTRUMENT FRESHNESS (done; the number's precondition).**

| question | method | verdict |
|---|---|---|
| is `generated/systemverilog_parser.rs` what HEAD's grammar + HEAD's codegen emit? | regenerate under the guard (`focus_systemverilog`, exit 0, 71 s, peak RSS 1 949 MB) and `shasum -a 256` both artifacts against the pre-run copies | **BYTE-IDENTICAL** — `786e49aa…` parser, `ab9acf62…` annotations. The artifact was already fresh; the STALE thing is the report, exactly as the audit said |
| was the tracked report's binary the debug or the release probe? | the report records only the basename `parseability_probe` | ⛔ **UNANSWERABLE from the artifact** — which is itself the defect this leaf closes (see the report change below) |

**THE DURABLE FIX FOR THE DEFECT CLASS (code, `stimuli/run_external_corpus.sh`):**
the report now carries an **Instrument identity** table — the sha256 of the parse
binary, of the grammar, and of the generated parser, plus the measuring `HEAD`
(with a `+dirty` marker). Re-hashing three files now answers *"is this number
still mine?"* in one command; before, staleness could only be inferred by
comparing the git commit dates of two OTHER files. Family-generic (paths derived
from `$GRAMMAR`), so VHDL and the `sv2005` lane inherit it for free.

**STEP 2 — ⛔ THE BLOCKER: THE RE-ADJUDICATION DIED ON ITS FIRST `uvm-core` ROW.**

```
$ python3 stimuli/sv/adjudicate_external_corpus.py
unrecognized results path shape: 'stimuli/sv/uvm/uvm-core-2020.3.1/compat/uvm_compat_macros.svh'
```

- **ROOT CAUSE (WHY + WHERE):** `adjudicate_external_corpus.py:1947` splits column 3
  on the literal `"/stimuli/sv/uvm/"` — **with a leading slash**, which a
  repo-root-relative row (`stimuli/sv/uvm/…`) cannot contain. Both sides dated by
  `git log -S`: the marker was written **2026-07-22** (`95368e5c`, leaf `.8a`), when
  the runner still emitted ABSOLUTE paths; the runner went repo-root-relative
  **2026-08-08** (`db5a640e`, `CORPUS-GRAD-ALL.2.1`) — and this leaf is the first
  SV re-run since, so the break had never been executed.
- ⭐ **The break sits directly under a comment claiming it cannot happen.**
  `run_external_corpus.sh:83` states the consumers key on *"the `/subs/<suite>/`
  (or `/stimuli/sv/uvm/`) INFIX … which a relative path still contains — verified
  before changing this."* That is true of the FIRST arm and false of the SECOND:
  `stimuli/sv/subs/verilator/x` does still contain `/subs/verilator/`, but nothing
  relative contains `/stimuli/…`. **The claim was verified on one arm and
  generalized to both** — a shared infix is compatible only with a spelling that
  has it. The comment is corrected in place.
- **FIX (minimal, one tier — the marker, not the caller):** the infix becomes the
  module constant `UVM_FOLD_MARKER = "stimuli/sv/uvm/"`, leading-slash-free, so it
  matches an absolute row AND a relative one. The `/subs/{suite}/` arm is left
  untouched: it is demonstrably correct for both spellings, and dropping its
  leading slash would widen it for no gain.

**STEP 3 — THE RE-MEASURE. Three runs, because each one exposed the next variable.**

| run | probe | budget | files | pass | fail | timeout | rate |
|---|---|---|---|---|---|---|---|
| baseline (tracked, 2026-07-25) | unrecorded | 20 s | 16 336 | 9 693 | 6 634 | 9 | 59.3 % |
| A — like-for-like at HEAD | `target/debug` | 20 s | 16 336 | 9 693 | 6 632 | 11 | 59.3 % |
| B — honest budget | `target/debug` | 60 s | 16 336 | **9 694** | 6 636 | 6 | **59.3 %** |
| **C — TRACKED** | `target/release` | 60 s | 16 336 | **9 694** | 6 638 | 4 | **59.3 %** |

⭐ **THE HEADLINE, and it is a per-FILE claim rather than a count claim:** across
**16 336** files the HEAD verdict set is **IDENTICAL** to the 2026-07-25 baseline —
**9 693 pass→pass, 6 632 fail→fail, ZERO pass→fail, ZERO fail→pass.** Every
difference in both runs is confined to the timeout column. ⇒ the grammar commit
that made the tracked report *provably stale by provenance* (`7219547c`,
`@entry: true` mandatory) changed **no SV corpus verdict at all**. The report was
stale in the sense that mattered procedurally and sound in the sense that mattered
numerically — and only a re-measure could tell those two apart, which is exactly
why the doctrine says re-measure rather than reason.

- **Run A's 2 `fail → timeout` rows are an INSTRUMENT ARTIFACT, individually
  re-verified rather than assumed:** both opentitan
  `alert_handler_reg_top.sv` (22 322 / 19 466 lines) complete **rc=1 (a genuine
  `fail`) in 17.56 s and 15.69 s** when run alone — they sit just under a 20 s
  budget and crossed it under 8-way load. Their true HEAD verdict is `fail`, as in
  the baseline.
- ⭐ **Run B ROOT-CAUSES A LOOSE END `.3.9` COULD ONLY LABEL.** `.3.9` recorded its
  corpus move as *"9,692→9,694 (+1 attributable, +1 proven jitter)"*. The jitter row
  is `verilator/test_regress/t/t_math_synmul_mul.v`, and it is not jitter: it is a
  parse that straddles a 20 s budget, so it PASSES deterministically at 60 s. That
  is why the tracked report reads 9 693 while `.3.9` measured 9 694 — the two
  numbers were never in conflict, they were taken at different budgets. Run B
  reproduces `.3.9` exactly.
- ⭐ **RUN C SETTLES THE QUESTION THE BASELINE COULD NOT EVEN BE ASKED: does the
  BUILD change the verdict? Measured, no.** Debug vs release over the same 16 336
  files at the same budget: **9 694 pass identical, 6 636 fail identical, ZERO
  pass↔fail divergence.** The only column that moves is `timeout` (6 → 4: the
  42 192-line `pinmux_reg_top.sv` and the 571-line `mm_ram.sv` resolve to `fail`
  once the binary is fast enough to finish). ⇒ **a `timeout` is a statement about
  the instrument, never about the parser** — which is precisely why the tracked
  artifact should be produced by the fastest honest instrument, and why the report
  now names its binary by hash.
- **⇒ run C is the tracked artifact** (release probe, 60 s: the fewest
  instrument-induced rows of the three). Runs A and B are banked alongside it at
  `rust/target/sv_axis2_baseline/` so the comparison chain is reproducible. Note
  this makes the tracked artifact NOT reproducible by the runner's bare default,
  which takes `rust/target/debug/parseability_probe` — hence
  `PGEN_PARSE_PROBE_BIN=rust/target/release/parseability_probe` is the recipe, and
  the binary's sha256 is in the report so the difference can never be silent again.

**STEP 4 — RE-ADJUDICATION AND RE-CLUSTERING AT HEAD.**

- **The burn-down baseline is CONFIRMED, not merely re-stated: `unexplained = 403`
  (rejects-valid **382** + accepts-invalid **21**)**, match 5 727, explained 1 430,
  deferred 8 776 — and it read **403 in all three runs**, so it is stable against
  both the budget and the build. The only manifest rows that differ from the tracked
  copy are the timeout resolutions leaving `divergence:explained_timeout` for their
  real classes (3 rows at run B, 5 at run C). Determinism of BOTH generators
  re-proven byte-for-byte (`cmp` ×2 on the manifest, `cmp` ×2 on the clusters).
- **The `verilog_2005` lane was stale too and is now re-measured** (`results_v2005.tsv`
  dated 2026-07-26): 2 459 files → **2 180 pass / 279 fail / 0 timeout (88.7 %)**,
  per-file transitions **ZERO changed**, and `adjudication_manifest_v2005.tsv` +
  `adjudication_summary_v2005.md` come back **BYTE-IDENTICAL** to the tracked
  copies (unexplained 76). Two lanes re-measured, neither moved.
- ⚠️ **The tracked cluster artifact was 109 rows behind and nobody could see it.**
  `rejects_valid_clusters.{tsv,md}` was still at its **273-row** pre-ADD-v1 vintage
  while the population has been 382 since `.3.9`; regenerated here to **382 rows /
  211 signatures**. (The `_v2`/`_v3` snapshots `.3.2`/`.3.7` cut were refreshed; the
  unsuffixed artifact the tooling actually writes was not.)

**ROUTED — found here, worked elsewhere (characterize first, fix second):**

1. ⚠️ **Four opentitan files exceed a 60 s budget even on the RELEASE binary, and
   they are one coherent family: the autogen crossbars** — `xbar_main.sv` (2 332 /
   1 340 lines) and `xbar_peri.sv` (**261** / **321** lines). A 261-line file that
   cannot be parsed in 60 s is super-linear backtracking, not input size. TOOLBOX
   §3.1 names the locus rather than guessing it: on `xbar_peri.sv`,
   `streaming_concatenation` **407 352** calls, `attribute_instance` **394 824**,
   `system_tf_call` **394 128**, `hierarchical_identifier` **329 017** — the
   `primary` alternative cascade. Routed to `SV-CORPUS-GRAD.11` (opened below).
2. ⚠️ **`cluster_rejects_valid.py` carries the SAME latent path defect**, in a
   different shape: its `--manifest` mode rebuilds each file as
   `subs_root / suite / relpath`, which is wrong for `uvm-core` (it lives at
   `stimuli/sv/uvm/`, not under `subs/`). Dormant only because all 174 uvm rows
   adjudicate `deferred:*` and never enter the cluster population. Routed to
   `SV-CORPUS-GRAD.11`.

- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — the tracked `59.3 %` report was committed 2026-07-25
    and `grammars/systemverilog.ebnf` last changed 2026-07-26 (`git log -1
    --date=short` on each), so it provably could not describe HEAD; and
    `adjudicate_external_corpus.py` then aborted with `unrecognized results path
    shape: 'stimuli/sv/uvm/…'`, blocking the re-adjudication outright.
  - [x] **ROOT CAUSE (WHY + WHERE)** — `adjudicate_external_corpus.py:1947` splits on
    `"/stimuli/sv/uvm/"`, a marker whose leading slash no repo-root-relative row can
    contain. Both sides dated with `git log -S`: marker written `95368e5c`
    (2026-07-22, absolute-path era), runner made relative `db5a640e` (2026-08-08) —
    this leaf is the first SV re-run across that boundary. The
    "verified before changing this" comment held for the `/subs/<suite>/` arm only.
  - [x] **FIX** — tier: repo-script, minimal. `UVM_FOLD_MARKER = "stimuli/sv/uvm/"`
    (leading-slash-free, matches both spellings); the over-general comment in
    `run_external_corpus.sh` corrected in place; `bash -n stimuli/run_external_corpus.sh`
    clean and `python3 -c "import ast; ast.parse(...)"` clean on the adjudicator.
  - [x] **ADDRESSED (verified)** — the adjudicator completes both lanes
    (`match=5727 unexplained=403 explained=1432 deferred=8774`; v2005
    `match=2179 unexplained=76`), was byte-identical across two consecutive runs
    (`cmp` ×2), and all four corpus runs completed under the guard at exit 0 with a
    banked marker (`guard.73455.marker` 299 s / `guard.76052.marker` 344 s /
    `guard.98561.marker` 116 s / `guard.23950.marker` 36 s; peak tree RSS 2 730 /
    4 809 / 7 341 / 851 MB, all inside the 16 384 MB budget).
  - [x] **NO REGRESSION** — per-FILE transition census over all 16 336 rows:
    **0 pass→fail, 0 pass→timeout, 0 pass→crash, 0 fail→pass** — held in every
    comparison run (A and B and C vs the baseline, and C vs B, i.e. across the
    budget AND across the build); v2005 lane **0** changed rows of 2 459 and its
    manifest+summary byte-identical to the tracked copies; `unexplained` 403
    unchanged in both count and composition (382 + 21) in all three runs;
    the census instrument itself carries a **positive control** (identity self-join
    → 0 changes, pass count reproducing the tracked 9 693) and a **negative
    control** (one planted flip → caught exactly once, at exactly that file). No
    grammar, codegen or generated artifact touched — `focus_systemverilog`
    regenerates `generated/systemverilog_parser.rs` **byte-identically**
    (`786e49aa…`).
  - [x] **LOCKSTEP** — characterization reports (both lanes) + adjudication manifest
    and summary + cluster artifacts regenerated; tree `.3` umbrella population
    corrected 406 → 382; `.11` opened for the two routed findings;
    TASK_TREE/MEMORY/CHANGES/DEVELOPMENT_NOTES + the book's corpus chapter this
    commit.

### `.11` — the two defects `.10` ROUTED (opened 2026-08-08, session #214)

- **Status: `todo`.** Both were found by `.10` and both are outside a measurement
  leaf's mandate; neither blocks the axis-2 campaign.
- **`.11a` — the four opentitan autogen-crossbar parses that exceed 60 s on the
  release binary** (`top_{darjeeling,earlgrey}/ip/xbar_{main,peri}`). Two are 261
  and 321 lines, so this is super-linear backtracking, not input size. Tool-pinned
  locus already banked by `.10` (`--dump-rule-call-counts` on `xbar_peri.sv`:
  `streaming_concatenation` 407 352 / `attribute_instance` 394 824 /
  `system_tf_call` 394 128 calls). Fix hierarchy applies as usual; the SPEED
  signature family (TOOLBOX group 2) governs its acceptance evidence.
  ⭐⭐ **ROOT-CAUSED 2026-08-10 (session #232) — the leaf now has a 5-SECOND REPRODUCER instead of a
  60-second 12 GB corpus file. Work from this, not from the corpus.**
  - **TRIGGER = `if / else if` CHAIN DEPTH, not input size.** The decode is an `always_comb` with a
    **19-deep** `if / else if` chain, each condition `(a & ~(ID)) == ID`. Decisive differential:
    `top_englishbreakfast/.../xbar_peri.sv` (**239 lines**) **PASSES**, while
    `top_darjeeling/.../xbar_peri.sv` (**261 lines**) consumes 12 GB — a ~9 % size difference with
    opposite outcomes. `diff` of the two shows only comment/port-list size, no new construct.
  - **GROWTH IS ~2× PEAK RSS PER ADDED BRANCH ⇒ O(2ⁿ)**, measured on a synthetic chain
    (`/usr/bin/time -l`, release probe):

    | branches | 4 | 6 | 8 | 10 | 12 | 13 | 14 | 15 | 16 |
    |---|---|---|---|---|---|---|---|---|---|
    | peak RSS (MB) | 26 | 28 | 32 | 46 | 104 | 177 | 307 | 583 | 1138 |

    Extrapolating the measured doubling to the real file's 19 branches gives ≈ 9 GB, consistent with
    the 12 362 MB measured on `xbar_main.sv`. ⇒ the corpus file is not special; **chain depth is the
    whole variable.**
  - **THE COST IS THE `primary` ALTERNATIVE CASCADE.** `--dump-rule-entry-counts-json` diffed
    n=12 → n=14 (+2 branches) shows **every** hot rule at ≈ **4×** (= 2× per branch), led by
    `system_tf_call` 49 326 → 196 810, `attribute_instance` 49 338 → 196 810,
    `tf_call_with_args`/`class_scoped_tf_call_with_args` ≈ 28 900 → 114 934 — the same cascade `.10`
    pinned via `--dump-rule-call-counts`.
  - ⛔ **HYPOTHESIS RAISED AND REFUTED BY MEASUREMENT — do not re-run this dead end.** The store
    looked guilty: `--dump-rule-outcome-counts-json` shows `facts_emitted` **45 181 → 180 365** and
    `rollbacks` **2 188 992 → 8 640 174** across n=12 → 14, and `MEMO-STORE-SOUNDNESS.2` evicts
    taint-gated memo entries on every store write. **Refuted:** a chain whose conditions are PURE
    LITERALS (no identifiers ⇒ no fact lookups) still blows up — **58 / 145 / 497 MB at n = 12 / 14 /
    16**. Same exponential base, ~2.3× lower constant. ⇒ the driver is **structural ambiguity in the
    chain**, not the fact/store machinery.
  - ⛔ **AND IT IS NOT A MEMO MISS.** `total_memo_hits` scales with everything else and the **hit
    rate is a constant 63.4 % → 63.9 %** across n=12 → 14. The memo is serving proportionally; the
    work itself is exponential.
  - **REPRODUCER (regenerate in seconds):** an `always_comb` containing `if ((a & ~(32'hK)) == 32'hK)
    begin s = 5'dK; end else if …` × N, N = 12…16. Both the identifier and literal-only variants are
    needed — the pair is what refutes the store hypothesis.
  - ⭐⭐ **THE CHOICE SITE IS NAMED (2026-08-10, `PGEN-SV-CORPUS-GRAD-0194`, Protocol D + entry/outcome
    counters). It is `conditional_else_branch`, `grammars/systemverilog.ebnf:1491-1492`:**

    ```ebnf
    conditional_else_branch := conditional_statement -> {kind: "elseif", body: $1}
                             | statement_or_null     -> {kind: "else",   body: $1}
    ```

    **The two alternatives are LANGUAGE-OVERLAPPING on exactly the input this chain produces.** Alt 1
    is `conditional_statement`; alt 2 reaches the *same* rule via
    `statement_or_null → statement → statement_item → statement_item_sv_2017 → conditional_statement`.
    So on an `else if …`, **both alternatives parse the identical text**, and the rule's policy is the
    default `longest_match`, which explores every alternative before picking. Each level therefore
    parses the entire remaining chain **twice** ⇒ `T(k) = 2·T(k−1)`.
  - **MEASURED, not argued — the entry count at that site is EXACTLY `2ⁿ − 1`**
    (`--dump-rule-entry-counts-json`, identifier variant):

    | branches n | 4 | 5 | 6 |
    |---|---|---|---|
    | `conditional_else_branch` entries | **15** | **31** | **63** |
    | `2ⁿ − 1` | 15 | 31 | 63 |
    | `conditional_statement` entries | 71 | 143 | 287 |
    | `statement_or_null` entries | 55 | 111 | 223 |
    | total entries | 38 820 | 56 001 | 85 566 |

    Protocol D's own line confirms both arms are live at every level:
    `🏁 Rule 'conditional_else_branch' selected branch 1/2 consuming 109 / 187 / 265 chars` at the
    outer levels and `branch 2/2 consuming 31 chars` at the terminal `else`.
  - ⭐⭐ **AND THE MEMO IS NOT THE MISSING PIECE THE WAY IT LOOKS — IT ALREADY COLLAPSES THE RECURSIVE
    RULE AND FAILS ON THE WRAPPERS.** `--dump-rule-outcome-counts-json` at n=6:

    | rule | entries | memo hits | body executions |
    |---|---|---|---|
    | `conditional_statement` | 287 | **208** | 79 |
    | `conditional_else_branch` | 63 | **0** | 63 |
    | `statement_or_null` | 223 | **0** | 223 |
    | `statement` / `statement_item` / `statement_item_sv_2017` | 224 each | **0** | 224 each |

    ⇒ the packrat memo *is* serving the duplicated `conditional_statement`, so the naive reading
    ("add memoization") is already true and already insufficient. The exponential lives in the
    **wrapper chain**, which re-executes in full 223 times — and `statement_item_sv_2017` is a
    **20-alternative** choice (`🏁 … selected branch 14/20`), which is precisely where `.10`'s pinned
    hot rules come from (`system_tf_call` 318 → 588 → 1114 across n=4/5/6).
  - ⛔⛔ **THE STORE IS NOW REFUTED AT PER-RULE RESOLUTION, not just in aggregate.** The earlier
    refutation showed the literal-only variant still blows up. Re-run with outcome counters, the
    identifier and literal-only chains at n=6 are **identical on every number that matters** —
    `conditional_else_branch` 63/0, `conditional_statement` 287/208, `statement_or_null` 223/0,
    `statement_item_sv_2017` 224/0, `facts_emitted` 1128 in **both** — differing only in total
    entries (85 566 vs 75 803, i.e. the smaller literal text). **The exponential is purely
    structural.** Do not revisit the store.
  - ⛔ **THE ONE OPEN QUESTION, named precisely so the next slice does not re-derive it: WHY does
    `statement_or_null` take 0 memo hits across 223 entries at ~13 distinct byte positions?** It is
    memoized — every one of the 1481 rules is wrapped in `memoized_call` (1482 sites; the call takes
    a `RULE_*` constant, **not** a string, so `grep 'memoized_call("rule")'` returns 0 and is a false
    negative — that cost a step here). And it SUCCEEDS, so the `.3.12` recursion-taint rule does not
    apply: that rule refuses to cache **failures only** (`ast_code_generator.rs:623`,
    `if result.is_ok() || recursion_block_events == snapshot { insert }`). ⇒ the remaining candidate
    is **store-taint eviction** (`MEMO-STORE-SOUNDNESS.2`: an entry whose body consulted the store is
    replayable only while the store is unchanged; n=6 shows `predicate_evaluations=1467`,
    `rollbacks=66 491`), which would evict the wrappers' entries before they can ever be replayed
    while `conditional_statement`'s survive inside a quieter window. **MEASURE IT, do not assume it** —
    the deciding instrument is a per-rule insert-vs-evict census, which does not exist yet and is
    likely the `--build-a-tool` step.
  - ⭐⭐ **ANSWERED AND MEASURED 2026-08-10 (`PGEN-SV-CORPUS-GRAD-0195`). Store-taint eviction is
    CONFIRMED, and the answer also REFUTES this leaf's own reading of `conditional_statement`.**
    The instrument is `docs/tasks/artifacts/sv_corpus_grad/memo_insert_evict_census.py` — and it
    needed **no engine change**: the generated `memoized_call` ALREADY logs every memo transition
    under `PGEN_TRACE_VERBOSITY=debug`, keyed by numeric rule id. The census joins those lines
    against the parser's own `RULE_NAMES` table. Measured, both variants, all three depths:

    | rule (n=4 / n=5 / n=6) | success inserts | STALE-tainted evictions | **SUCCESS replays** | failure hits |
    |---|---|---|---|---|
    | `conditional_else_branch` | 15 / 31 / 63 | 11 / 26 / 57 | **0 / 0 / 0** | 0 / 0 / 0 |
    | `conditional_statement` | 15 / 31 / 63 | 11 / 26 / 57 | **0 / 0 / 0** | 44 / 98 / **208** |
    | `statement_or_null` | 31 / 63 / 127 | 22 / 52 / 114 | **0 / 0 / 0** | 0 / 0 / 0 |
    | `statement_item_sv_2017` | 32 / 64 / 128 | 22 / 52 / 114 | **0 / 0 / 0** | 0 / 0 / 0 |

    ⇒ **the packrat success replay NEVER fires on the chain — 0 replays against 381 inserts at n=6,
    at every depth, in BOTH the `ident` and `lit` variants (every per-rule number identical).**
    Inserts go as `2ⁿ − 1`; evictions as `2ⁿ − 1 − n`, i.e. **every entry that is ever looked up
    again is found stale and thrown away**; the `n` survivors are simply never revisited.
  - ⛔⛔ **CORRECTION TO `-0194` (this leaf's previous bullet, and that commit's subject line).** The
    claim *"the memo ALREADY collapses `conditional_statement` (287 entries/208 hits)"* is **WRONG**,
    and the error was reading a FUSED counter. `rule_memo_hit_counts` sums three different memo
    paths — replayed success, cached clean failure, cached tainted failure (`parser_registry.rs:136`
    says so). Split apart, **all 208 of `conditional_statement`'s n=6 "hits" are cached FAILURES and
    exactly 0 are success replays.** The memo is collapsing the cheap dead-end probes, not the
    expensive re-parse of the remaining chain. ⇒ *"add memoization" is not already-true-and-
    insufficient; the memo is simply not serving this construct at all.*
  - **ROOT CAUSE (WHY + WHERE), now mechanical.** `memoized_call` in the generated parser:
    (a) stamps a successful entry as tainted iff its body transitively moved the **global cumulative**
    `predicate_evaluations()` counter (`generated/systemverilog_parser.rs:1419458` snapshot →
    `:1419463` compare → `:1419491` stamp), and (b) on every later lookup evicts that entry if the
    **global** `write_epoch` has moved since (`:1419399-1419416`). On the SV statement surface both
    conditions hold essentially always — any rule spanning the statement surface evaluates *some*
    predicate, and the store is written constantly during exploration — so the memo is **effectively
    disabled for the whole upper statement surface**. Without packrat's collapse, the
    language-overlap at `conditional_else_branch` (named by `-0194`) compounds unchecked:
    `T(k) = 2·T(k−1)` ⇒ O(2ⁿ). ⭐ Note the eviction path is itself the proof of taint: it can only
    run on an entry carrying `tainted_at_epoch = Some(_)`.
  - ⭐ **NEW, PREVIOUSLY UNWRITTEN, AND IT INVALIDATES A CLAIM IN THE BULLET ABOVE: 663 of the 1481
    SV rules (2871 call sites) are NOT memoized at all.** They are reached through the generated
    `inlined_frame_call` helper, which keeps the full observable frame — entry counter, coverage
    push, enter/exit trace — but contains **no `memoized_call`**. So *"every one of the 1481 rules is
    wrapped in `memoized_call`"* is true of the rule METHOD and false of the executed PATH. This is
    exactly what made the census's first ground-truth control fail on ~305 live rules, and it is why
    that control is now **partitioned** (STRICT rules must balance exactly; inlined-reachable rules
    may only fall short, and the shortfall is reported, never dropped). None of the four chain rules
    above is inlined — they all balance exactly, which is what makes their zeros trustworthy.
  - ⛔ **The instrument REFUSES rather than guesses.** Every run cross-checks the trace census against
    the independent atomic counters (`--dump-rule-outcome-counts-json`): for every STRICT rule,
    `miss + hits` must equal `rule_entry_counts` exactly, and the id→name join is itself verified
    (all 1481 `RULE_*` consts must index their own name). Measured: **310 STRICT rules balance
    exactly** on all six runs. The control fired for real on the first attempt — that is how the
    inlined population was found at all, rather than being averaged into a published number.
  - **REPRODUCE (seconds, no rebuild):**
    ```bash
    python3 docs/tasks/artifacts/sv_corpus_grad/gen_if_else_chain.py 6 ident tmp/c6.sv
    PGEN_TRACE_VERBOSITY=debug ./rust/target/release/parseability_probe --parse systemverilog \
        tmp/c6.sv --profile sv_2017 --trace --trace-log-file tmp/t6.log
    ./rust/target/release/parseability_probe --parse systemverilog tmp/c6.sv \
        --profile sv_2017 --dump-rule-outcome-counts-json tmp/oc6.json
    python3 docs/tasks/artifacts/sv_corpus_grad/memo_insert_evict_census.py tmp/t6.log \
        --verify tmp/oc6.json --rules conditional_else_branch,conditional_statement,statement_or_null
    ```
  - ⇒ **THE FIX HIERARCHY IS NOW RESOLVABLE, and the memo question did change the answer — it
    ELIMINATED tier 3 from this leaf.** An engine fix to the taint over-approximation would help, but
    it is cross-family, sits on a SOUNDNESS mechanism (`MEMO-STORE-SOUNDNESS.2`), and is not SV-corpus
    work ⇒ **ROUTED OUT to `.11d`** (evidence below). For `.11a` itself the tier-1 *declarative* fix
    stands and is now known to be sufficient on its own: `@branch_policy: ordered` on
    `conditional_else_branch` removes the 2× at the choice site whether or not the memo ever serves
    it. ⛔ Still owed by the FIX slice, unchanged: acceptance-neutrality proof (Protocol D —
    `longest_match` and `ordered` differ exactly where two arms tie, and here they tie **by
    construction**, which is the whole defect) + the MEMORY before→after at n=16 and on
    `xbar_main.sv`.
  - ⛔⛔ **TRAP FOR THE FIX SLICE, verified 2026-08-10 so it is not re-derived — IT MUST BE `ordered`,
    AND SV'S OWN IDIOM IS THE WRONG ONE.** `@branch_policy` is an EXISTING declarative surface (no new
    annotation ⇒ `DESIGN-PRIOR-ART` is satisfied by construction) taking **three** values —
    `longest_match` (the default), `ordered`, `priority_first` (`annotation_validator.rs:669`). But
    **`priority_first` STILL RUNS THE FULL TOURNAMENT** — it changes only how a tie is resolved, not
    whether the losing arms are explored (`grammar_wellformedness.rs:1255`: *"under the DEFAULT
    `longest_match` policy (and `priority_first`) the engine runs the FULL tournament"*). Only
    `ordered` short-circuits: the keep-first-winner emission *"belongs to the `ordered` policy alone"*
    (`ast_based_generator.rs:4715-4720`). ⚠️ **All 29 existing `@branch_policy` uses in
    `systemverilog.ebnf` are `priority_first`**, as are every one in `rtl_frontend.ebnf` — so copying
    the surrounding idiom yields a change that is a no-op against this defect while looking like the
    fix. Confirm the emitted parser short-circuits before trusting any before→after number.
  - **FIX HIERARCHY, in order, with what each would cost — none chosen yet, because the memo question
    above changes the answer:** (1) *declarative* — give `conditional_else_branch` an
    `@branch_policy: ordered` so alt 1 commits on an `else if` and alt 2 is never explored; this is a
    **selection-semantics change** and per Protocol D must be proven not to change acceptance, since
    `longest_match` vs `ordered` differ exactly where two arms tie. (2) *grammar* — restructure so the
    `else if` case is not reachable through two paths (the LRM writes `[ else statement_or_null ]`,
    and `conditional_else_branch` is PGEN's own encoding, so this is in-scope and does not touch
    LRM fidelity). (3) *engine* — whatever the memo census turns up.
  - **ACCEPTANCE (unchanged):** a **MEMORY** before→after (`/usr/bin/time -l` at n=16 and on
    `xbar_main.sv`) alongside the TOOLBOX group-2 SPEED signature, plus a full-corpus acceptance-
    neutrality proof, since options (1) and (2) both touch selection.
  - **REPRODUCER, now scripted:** `docs/tasks/artifacts/sv_corpus_grad/gen_if_else_chain.py N {ident|lit} OUT.sv`.
    Confirmed on the release probe: n=12/14/16 → **114 / 349 / 1314 MB**, `~1.94×` per added branch.

  ### `.11a` — ✅ **FIXED 2026-08-10 (`PGEN-SV-CORPUS-GRAD-0197`), tier-1 declarative, ONE annotation**

  The fix is `@branch_policy: ordered` on `conditional_else_branch` (`grammars/systemverilog.ebnf`),
  plus the comment explaining why it is load-bearing. **No Rust, no engine change, no other rule.**
  The exponential is not merely reduced — it is **gone**: peak RSS is now FLAT in chain depth.

  | | n=12 | n=14 | n=16 | `xbar_main.sv` (earlgrey) |
  |---|---|---|---|---|
  | **before** | 114 MB / 0.41 s | 348 MB / 1.61 s | 1313 MB / 6.59 s | **12 362 MB / TIMEOUT at 60 s** |
  | **after** | **27 MB / 0.01 s** | **29 MB / 0.01 s** | **29 MB / 0.01 s** | **118 MB / 0.13 s, PASSES** |

  All **4** tracked `timeout` rows now PASS (≤ 203 MB, ≤ 0.21 s). Choice-site entries collapse from
  `2ⁿ − 1` (15/31/63) to exactly **n** (4/5/6), and total parse entries — 38 820 / 56 001 / 85 566
  before — become **28 284 / 33 181 / 38 078**, i.e. **exactly linear** (+4 897 per added branch).

  #### Acceptance Checklist (enforced)
  - [x] **REPRODUCE / ISSUE** — `/usr/bin/time -l` on the scripted reproducer: n=16 → `1313 MB`,
    `6.59s`; and the four corpus rows adjudicated `divergence:explained_timeout`, of which
    `xbar_main.sv` alone peaked at `12362514432 maximum resident set size` before the 60 s deadline.
  - [x] **ROOT CAUSE (WHY + WHERE)** — two measured halves, both banked above.
    **(i) The site:** `conditional_else_branch` (`systemverilog.ebnf:1491`) has LANGUAGE-OVERLAPPING
    arms — `--trace-rules conditional_else_branch` at `PGEN_TRACE_VERBOSITY=debug` shows both arms
    live and winning at different levels (`🏁 … selected branch 1/2 consuming 109/187/265 chars`,
    `branch 2/2 consuming 31 chars`), and the entry counters put it at exactly `2ⁿ − 1` (`-0194`).
    **(ii) Why packrat does not absorb it:** `--dump-rule-outcome-counts-json`, split per memo path
    by the `-0195` census, gives `0` success replays against 381 inserts / 342 stale-tainted
    evictions at n=6. ⇒ each level re-parses the whole remaining chain: `T(k) = 2·T(k−1)`.
    Cross-checked with `--lint-grammar` (`ordered_choice_shadowing=0` — the arms overlap in the
    LANGUAGE, which no static shadowing check can see, and that is exactly why it needed measuring).
  - [x] **FIX** — tier **1 (declarative)** of the fix hierarchy: one `@branch_policy: ordered`
    annotation. Tier 2 (grammar restructuring) is unnecessary and tier 3 (engine) was ELIMINATED from
    this leaf and routed to `.11d`. ⛔ `priority_first` — this file's other 29 uses — would NOT work:
    it still runs the full tournament.
  - [x] **ADDRESSED (verified)** — `/usr/bin/time -l`, release probe: n=16 **1313 MB → 29 MB (45×)**
    and **6.59 s → 0.01 s**; `xbar_main.sv` **12 362 MB / TIMEOUT → 118 MB / 0.13 s**; all 4 timeout
    rows PASS. `--dump-rule-entry-counts-json`: choice site `2ⁿ − 1 → n`, totals now exactly linear.
    Corpus-wide, `timeout=4 → timeout=0` and the whole 16 336-file run drops to **71 s at 4 756 MB
    peak** (was 12 468 MB).
  - [x] **NO REGRESSION** — every named oracle re-run and green:
    **(a)** external corpus `sv_2017`, 16 336 files: the ONLY transitions are **4 × timeout→pass**,
    **0 pass→non-pass**, fail set byte-identical `6586 → 6586`;
    **(b)** external corpus `verilog_2005`, 2 459 files: **byte-identical verdicts, 0 transitions**;
    **(c)** adjudication `unexplained` held at **293** (no false improvement) while
    `divergence:explained_timeout` retired 4 → **0**;
    **(d)** `sv_cert_recognized_union_gate`: `union UNKNOWN=0`, `canonical UNKNOWN=11`,
    `residual=[]`, `sample_parse_failures=0`, deterministic across **seeds 0/7/42**, all equal to the
    pinned expected values;
    **(e)** `ast_shape_contract_gate` **18/18 GREEN**; `systemverilog_parser_book_gate` GREEN;
    **(f)** `GENERATED-CLIPPY-CORRECTNESS: ✅ PASS — 0 clippy::correctness findings` across 10+1
    generated artifacts under 68 pinned lints;
    **(g)** `--lint-grammar`: `ordered_choice_shadowing=0`, `unreachable_rules=0`,
    `non_terminating=0`, `undefined_references=0` — and the shadowing verdict is **newly meaningful
    here**, since it fires ONLY under `@branch_policy: ordered`.
  - [x] **LOCKSTEP** — grammar comment; this leaf; `.11`'s timeout tripwire + the `.3.27` memory-guard
    guidance corrected (below); the promoted corpus oracle + re-adjudicated manifest;
    `TOOLBOX.md`/book/`CHANGES.md`/`DEVELOPMENT_NOTES.md`/`MEMORY.md`.

  ⭐⭐ **ACCEPTANCE NEUTRALITY IS PROVEN AT THE CHOICE SITE ITSELF, not only corpus-wide.** Protocol D
  under `ordered` reproduces `-0194`'s `longest_match` trace **byte-for-byte**:
  `🏁 Rule 'conditional_else_branch' selected branch 1/2 consuming 109 / 187 / 265 chars` at the outer
  levels and `branch 2/2 consuming 31 chars` at the terminal `else`. ⇒ `ordered` selects **exactly
  what the tournament selected**; it just stops exploring the loser. The AST is unchanged (n=4 →
  3 × `elseif` + 1 × `else`). That is the strongest form of the neutrality argument the leaf owed:
  the two policies differ only where arms TIE, and here the tie is won by the same arm either way.

  ⛔ **ROUTED FINDING (hit while landing this leaf, NOT worked — lane lock): the TASK-ACCEPTANCE
  signature family is missing a documented instrument.** `DIAGNOSIS_SIG` in
  `scripts/check_diagnosis_evidence.sh:323` accepts `--dump-rule-call-counts` and
  `--dump-rule-outcome-counts` but **not `--dump-rule-entry-counts`** — even though that flag is a
  first-class TOOLBOX entry (3.4), is deterministic and re-runnable, and is the instrument that
  produced this leaf's `2ⁿ − 1` and its `→ n` refutation. A ROOT CAUSE box backed ONLY by it is
  rejected as unbacked, which is the exact "gate reports a missing capability" shape
  ([[project_waiver_is_a_gate_bug_report]]). ⇒ **no waiver was needed here** (the box legitimately
  cites `--trace-rules` and `--dump-rule-outcome-counts-json`, the instruments that actually decided
  each half), so this is filed, not worked. **Owner: `GENERATED-LINT-CORRECTNESS.12`**, a leaf that
  now EXISTS and carries the evidence — ⛔ an earlier revision of this bullet named `.8` (the
  nearest-looking sibling) without writing anything into that tree, which is a **dangling routing
  promise**: the owning tree had no idea, so the finding was logged, not tracked. Naming an owner is
  not routing; creating the owning leaf is. ⚠️ Price the token against the whole corpus before adding
  it — `.4`'s chartered hypothesis would have admitted 2 of 304 boxes.

  ⛔ **TWO PRIOR STATEMENTS IN THIS TREE ARE NOW OBSOLETE — correcting forward rather than editing
  them away.** (1) `.11`'s standing tripwire says these four rows' adjudication *"depends on the
  runner's timeout argument and on which binary ran"*. It no longer does: they pass in 0.03–0.21 s,
  which no plausible budget cuts. (2) `.3.27` raised the SV memory-guard guidance to **≥ 16384 MB**
  because one file needed 12 GB, and `stimuli/run_external_corpus.sh`'s header still carries that
  reasoning. The measured tree peak is now **4 756 MB**, so the README's example `--budget-mb 12288`
  is sufficient again — the guidance is corrected in the runner header in this commit, with the
  history kept.

  ⛔ These four are also a standing tripwire for this campaign: they are the only
  rows whose adjudication class depends on the runner's timeout argument and on
  which binary ran, so a future slowdown would read as a corpus regression that is
  really a budget effect. `.10` measured that dependence explicitly — 9 timeouts at
  debug/20 s, 6 at debug/60 s, 4 at release/60 s, with the pass/fail sets identical
  throughout.
  ⭐⭐ **NEW DIMENSION, measured 2026-08-10 by `.3.27` and recorded here because `.11a` owns
  these four files: the backtracking is not only slow, it is a MEMORY divergence, and it is
  large enough to take the whole machine's budget.** `/usr/bin/time -l` on ONE file, alone,
  release binary, at the 60 s deadline:

  ```
  $ /usr/bin/time -l timeout 60 ./rust/target/release/parseability_probe --parse systemverilog \
      stimuli/sv/subs/opentitan/hw/top_earlgrey/ip/xbar_main/rtl/autogen/xbar_main.sv --profile sv_2017
        60.10 real        58.72 user         1.29 sys
        12362514432  maximum resident set size
  ```

  **12 362 MB in one process** — i.e. this single file exceeds the README's example
  `--budget-mb 12288` on its own, and the full corpus run's measured tree peak is
  **12 468 MB**. ⇒ two consequences, neither previously written down: (1) the documented
  memory-guard budget cannot run the SV corpus at all — `.3.27` raised the runner's guidance
  to **≥ 16384 MB** for `sv` after a run was killed `reason=rss-budget` at
  `peak_rss_mb=12468`; (2) ⛔ **the fix-hierarchy framing in this leaf is incomplete.** A
  divergence that allocates ~12 GB is not merely a SPEED defect that a faster machine hides —
  it is unbounded memory growth on **valid, machine-generated, industry-standard RTL**, which
  a downstream embedder experiences as an OOM, not as a slow parse. So `.11a`'s acceptance
  evidence needs a MEMORY before→after alongside the TOOLBOX group-2 speed signature, and its
  severity is higher than "four slow files" reads.
  ⛔ **And this is the sharpest instance of the class `.3.27` exists to stop:** these four are
  currently adjudicated `divergence:explained_timeout` — i.e. a 12 GB unbounded-allocation
  parser defect is sitting inside the population the burn-down treats as EXPLAINED. The word
  "timeout" is doing work it has not earned.
- **`.11b` — `cluster_rejects_valid.py` rebuilds `uvm-core` paths under `subs/`.**
  Same root cause family as `.10`'s blocker (a path convention encoded in two
  places), dormant because all 174 uvm rows adjudicate `deferred:*`. It will wake up
  the moment `.4` chaining promotes any uvm row into the defect population.
- **`.11c` — ⭐ the packrat memo is part of the ACCEPTANCE SEMANTICS on cyclic rules,
  and nothing says so** (routed out of `.3.12`, 2026-08-08). Measured there: whether an
  indirect left-recursive construct parses depends on the memo replaying a success the
  cycle guard would refuse to re-derive — remove that replay and 4 corpus files stop
  parsing. Two consequences nobody has priced. **(a)** Acceptance is
  EVALUATION-ORDER-DEPENDENT on such rules: the same construct parses or not according to
  which context reached that `(rule, position)` first, which is the deeper reason `.3.12`'s
  defect existed at all rather than a separate curiosity. **(b)** Every "dropping a cache
  cannot change a correct parse" argument in this repository is therefore true only of
  ACYCLIC rules — including the one `RGX-0078.5.i.4` (P1b memo elision) leaned on, which
  is why the performance chapter's wording was corrected in the same commit. ⛔ Not a
  defect report: the current behaviour is the useful one and `.3.12` preserves it
  deliberately. It is a **missing invariant** — the engine should state, and gate, what
  its memo guarantees on a cyclic rule. Natural home for the fix is a Warth-style seeded
  left-recursion treatment (grow-the-seed), which would make the acceptance
  order-INdependent by construction rather than by cache luck; that is a design leaf, and
  it belongs to the engine, not to this corpus tree.
- **`.11d` — ⭐ the memo TAINT test is a global over-approximation, so packrat silently
  stops working wherever the store is live** (routed out of `.11a`, 2026-08-10,
  `PGEN-SV-CORPUS-GRAD-0195`). **Status: `todo`.** `memoized_call` taints a successful entry
  iff the **global cumulative** `predicate_evaluations()` counter moved across the body, and
  then invalidates it as soon as the **global** `write_epoch` moves. Both signals are
  whole-parse scalars, so the test cannot distinguish *"this body's outcome depends on the
  store"* from *"some unrelated rule anywhere in this body's subtree consulted the store"*,
  nor *"the facts this entry actually read changed"* from *"any fact anywhere changed"*.
  ⇒ on a grammar where predicates are common and the store is written during exploration,
  **every non-leaf rule is tainted and every tainted entry is stale by its next lookup**, so
  the packrat guarantee silently degrades to none. Measured on the `.11a` chain: 0 success
  replays against 381 inserts, 342 evictions (n=6). ⛔ **NOT a soundness complaint —
  `MEMO-STORE-SOUNDNESS.2` is conservative in the SAFE direction and the current behaviour is
  correct.** It is a **precision** defect with an unbounded performance cost, and the fix is a
  narrower dependency record (which facts/scopes a body actually read) rather than a global
  counter pair. Natural home is the engine + `MEMO-STORE-SOUNDNESS`, not this corpus tree.
  ⚠️ Sizing it needs a per-rule taint census across families first — do NOT assume the `.11a`
  chain's 100 % eviction rate is representative.

### `.12` — ⭐⭐ AUDIT THE `explained` POPULATION: the one class ever audited hid a 12 GB parser defect (`todo`, opened 2026-08-10 by `.11a`)

- **Status: `todo`. IN-LANE and it bears directly on the release claim** — this is not a governance
  finding. It asks whether *"axis 2 = 293 unexplained is the whole remaining distance"* is TRUE.
- **THE PRECEDENT, and it is 1-for-1.** `divergence:explained_timeout` held **4** rows that the
  burn-down treated as understood. `.11a` audited them and every one was an **O(2ⁿ) parser defect
  allocating 12 GB on valid, machine-generated, industry-standard RTL** — a downstream OOM, not a
  slow parse. One annotation retired the entire class (4 → 0) and all four now PASS in ≤ 0.21 s. ⇒
  **the only `explained` class this campaign has ever opened turned out to be a defect wearing a
  resource-limit mask.** That is a sample of one, and it is the only sample there is.
- **THE REMAINING POPULATION — 1 459 rows, and after `.11a` they are ALL one family:**

  | class | rows |
  |---|---|
  | `divergence:explained_svpp_macro_use` | 1111 |
  | `divergence:explained_svpp_conditional` | 203 |
  | `divergence:explained_svpp_include` | 141 |
  | `divergence:explained_svpp_protected_envelope` | 4 |

  Every one asserts *"this file fails because it needs the preprocessor lane, not because the parser
  is wrong."* Plausible — `svpp` is a real, separate, unfinished lane — and **exactly as plausible as
  "this file fails because it timed out" was.**
- ⛔ **THE ASYMMETRY THAT MAKES THIS WORTH DOING.** A wrong `unexplained` verdict costs a wasted
  investigation. A wrong `explained` verdict costs a **shipped parser defect**, because the row is
  removed from the burn-down by construction and nothing ever looks at it again. The two errors are
  not equally priced, and only one of them is being checked.
- **HOW TO AUDIT IT CHEAPLY — a spot-check, not a re-adjudication.** The claim is falsifiable per
  row: if a row is genuinely svpp-blocked, the parse must fail *at or before* the first
  preprocessor-requiring construct. So sample each class, take the `furthest_position=` from the
  parse error, and check what sits at that offset. A row whose furthest position is **past** its last
  macro/include/`ifdef` is not svpp-blocked — it is a parser gap with an svpp label. ⚠️ Sample per
  CLASS, not uniformly: 1 111 of the rows are one class, so a uniform sample tells you about
  `macro_use` and nothing else.
- ⛔ **DO NOT re-derive expected verdicts from the parser** — the manifest's doctrine is
  expected-from-SPEC ([[feedback_corpus_expected_from_spec_not_fix]]). This leaf audits whether the
  OBSERVED→`explained` mapping is sound, never what the expected verdict should be.
- **Owed:** a per-class spot-check with its sample size and method stated, the misclassified rows
  named and re-routed, and — whatever the result — the honest sentence about what `explained` means
  written into the axis-2 accounting. A clean audit is a real outcome and worth the same commit as a
  dirty one; what is not acceptable is the current state, where the number is quoted as settled and
  has been checked exactly once.

## ROUTING EVIDENCE (`.3.12` → `.11c`, `.11a` → `.11d`, and the `.11a`/`.11b` pair from `.10`)

⛔ Required by the `ROUTING-EVIDENCE` doctrine: *a routing is a claim about WHERE a defect lives, and
the deciding evidence is usually already on disk.*

### `.11c` — the memo is part of the acceptance semantics on cyclic rules

1. **Does it reproduce OUTSIDE the family it is routed to? — YES, MEASURED, and that is precisely
   why it is routed to the ENGINE rather than to any grammar family.** Three independent
   measurements, none of them SystemVerilog:
   - the isolating case `recursion_guarded_memo_isolation` reproduces the whole mechanism on a
     **6-rule SYNTHETIC grammar** with no SV in it, through the compile-and-run oracle AND the
     interpreter (`parse_harness_combinator_gate`, 28/28);
   - the code changed is `ast_based_generator.rs` + `ast_based_generator/cascade.rs` +
     `RecursionGuard` in `mod.rs` — **shared codegen and shared runtime**, compiled into every
     generated parser, so no grammar can opt out;
   - the **VHDL** corpus was re-run for exactly this reason (13 720 files, **0 transitions**), and
     the equivalence gate re-certified **all 11** grammars, confirming the behaviour is uniform
     rather than SV-shaped.
2. **What was MEASURED to place it there, not what makes it plausible?** That refusing to replay a
   recursion-tainted SUCCESS costs **4 corpus files pass→fail** — i.e. the replay is load-bearing for
   acceptance, not merely a cache hit. That is an observation about the ENGINE's acceptance
   semantics on cyclic rules; it was obtained by A/B-ing two engine builds over the same 16 336-file
   corpus, not inferred from the SV grammar.
3. **What would have to be true for the routing to be WRONG, and was it checked?** It would be wrong
   if the behaviour were an artifact of `systemverilog.ebnf`'s particular cycle (`cast → casting_type
   → constant_primary → constant_function_call → call_primary`) rather than of the engine. Checked
   and refuted: the synthetic grammar has none of those rules and exhibits the identical
   order-dependence, and the interpreter — a second implementation with **no cycle guard at all**,
   only a depth ceiling — shows the same class through its own mechanism. ⚠️ **Honest limit:** the
   *quantitative* claim ("how much acceptance depends on evaluation order") is measured only on SV
   and on the synthetic case. No other family's cyclic surface has been swept for order-dependent
   acceptance, and `.11c` should start by doing that rather than by assuming SV is representative.

### `.11d` — the memo taint test is a global over-approximation

1. **Does it reproduce OUTSIDE the family it is routed to? — the MECHANISM yes, by construction;
   the PATHOLOGY not yet, and the negative result is recorded rather than smoothed over.** The taint
   snapshot/compare/stamp and the epoch-eviction are emitted from the **shared codegen**
   `memoized_call` template, so they are compiled identically into all ten generated parsers — no
   grammar can opt out. But the pathology needs BOTH legs, and only SV currently has them.
   Measured, same instrument, one parse each:

   | grammar | `predicate_evaluations` | `facts_emitted` / `rolled_back` | stale-tainted evictions |
   |---|---|---|---|
   | `systemverilog` (`.11a` chain, n=6) | **1467** | 1128 / 1119 | **1860** |
   | `regex` (named groups + backref + bounded quants) | **3** | 5 / 0 | **0** |

   ⇒ regex is CLEAN, and that is informative rather than disappointing: it shows the degradation is
   not "predicates are present" but "predicates are evaluated **and** the store is written between
   an insert and its next lookup". `regex` is the ONLY other grammar in the repo declaring
   `@predicate` (18; vhdl/json/rtl_frontend/svpp declare none), so SV vs regex is the whole
   available cross-family surface today — which is exactly why `.11d` must start by BUILDING the
   cross-family taint census rather than by generalising from one chain.
2. **What was MEASURED to place it in the ENGINE, not in `systemverilog.ebnf`?** That the eviction
   fires on rules whose taint is **inherited, not intrinsic**: `statement_or_null`,
   `statement`, `statement_item` and `statement_item_sv_2017` declare no predicate of their own, yet
   all four are stamped tainted and evicted at the same rate (114 each at n=6), because
   `predicate_evaluations()` is a whole-parse counter and any predicate anywhere in their subtree
   moves it. A grammar edit cannot fix a test that never looked at the grammar. The identical
   per-rule numbers under the `ident` and `lit` variants — chains that differ in whether their
   conditions reference an identifier at all — is the second half of that: the taint does not track
   what the rule actually read.
3. **What would have to be true for the routing to be WRONG, and was it checked?** It would be wrong
   if the evictions were driven by `systemverilog.ebnf`'s own predicate placement — i.e. if moving or
   deleting SV's predicates removed the degradation, making it a grammar-shaped defect. Partially
   checked and NOT fully closed: the `lit` variant removes every identifier-driven fact LOOKUP from
   the chain's conditions and changes nothing (same inserts, same 114 evictions, `facts_emitted`
   1128 in both), which rules out the chain's OWN predicates as the driver. ⚠️ **Honest limit:** it
   does not rule out SV's predicate surface as a whole, because the SV stdlib preload and the
   statement surface still emit facts throughout. A grammar with predicates but no mid-parse store
   writes has not been constructed, and that synthetic control — not another corpus run — is the
   cheapest thing `.11d` can do to make this routing airtight. The `parse_harness_semantic_suite`
   (TOOLBOX 1.8) already owns the isolating-grammar machinery for exactly that.

### `.11a` / `.11b` (routed by `.10`, evidence recorded here for completeness)

- **`.11a`** (four opentitan crossbars over a 60 s budget) is routed INSIDE this tree, not out — it
  is an SV-corpus performance row with a tool-pinned locus (`--dump-rule-call-counts` on
  `xbar_peri.sv`). It was **re-measured under `.3.12`**: still exactly 4 timeout rows, the same
  files, unchanged by the memo fix — so it is not a memo-caching defect wearing a performance mask,
  which is the reading that would have made this routing wrong.
- **`.11b`** (`cluster_rejects_valid.py` rebuilding `uvm-core` paths under `subs/`) is a
  repo-script path-convention defect, also routed inside this tree. It reproduces outside SV **by
  construction** — the same normalizer defect class was already measured in
  `adjudicate_external_corpus.py` (`.10`) and `corpus_rule_coverage.py` (`.7c`) — which is why it is
  filed as a convention defect with three known consumers rather than as an SV-corpus bug.

## Corpus-sufficiency assessment (banked 2026-07-22, session #191 — the no-BS baseline behind the mandate)

- Stressed well today: LRM-clause breadth (sv-tests), 30 years of real-world
  regression mess (verilator).
- Measured gaps: (1) no verification-class (UVM-scale) code in the bulk run
  (uvm-core vendored but outside `subs/`); (2) parse-level `must_reject`
  population ≈50 rows, accepts-invalid candidates just 6 — the negative axis
  is the thinnest; (3) ~38% of the vendored corpus (1,102 explained + 866
  deferred rows) exercises nothing until `.4` chaining + svpp expansion land
  — the biggest untapped stress reserve is already on disk; (4) slang/verible
  vendored slices are crumbs of their real (embedded) test surfaces; (5) no
  very-large-file / fuzzer-shaped population (1 timeout case total).

## Acceptance Criteria (tree)

1. Fresh characterization at HEAD vintage + a complete adjudication manifest:
   every vendored case classified with a metadata/spec-derived expected verdict.
2. The parser matches ALL expected verdicts; exclusions named + justified in the
   tracked manifest; **zero unexplained divergences** ("flying colors", earned).
3. The graduation gate is standing, deterministic (pinned corpus commits), and
   wired into the SV family-status `Done` computation.
4. Full lockstep (books/LIVE/contract) and every landing leaf through the
   TOOLBOX acceptance checklist.
5. **(Amended 2026-07-22)** The corpus itself is proven **top of class**:
   the `.7` instrument reports **100% measured coverage of the parseable
   grammar surface per profile** (uncovered rules/clauses = 0 or
   N/A-with-cause), with keyed negatives present wherever the LRM defines
   parse-level illegality — the ADD-v1 suites vendored and adjudicated.

---

## ⭐ DIRECTOR DIRECTIVE (2026-07-25, session #206) — a STRICTNESS AXIS for the SV parser

Raised by the director while reviewing `.3.11`: *"I want the SV parser to be
flexible, strict-LRM compliance would mean hardcode, let's give us the possibility
to accept strictness and dialect-tolerance, so I would go for a switch."*

**⭐ SETTLED SAME SESSION, after the `.3.11` measurement was shown to the director:**
*"So, we will stick to strict-LRM compliance then, good I prefer that."* ⇒ the SV
parser is **STRICT-LRM BY DEFAULT** and the switch is **DEFERRED, not rejected**.
Banked as decision record
`docs/decisions/feedback_sv_strict_lrm_compliance_default.md` (layer C), which
carries the policy, the ⚠️ scope limit, the re-open trigger and the mechanism
constraints. The three constraints below stand as the design brief IF it re-opens.

**ACCEPTED as a direction, with three engineering constraints recorded so the
design does not start on the wrong foot.** No leaf opened yet — the design leaf
belongs to `LRM-GRAMMAR-FIDELITY` (which already owns cross-family fidelity
infrastructure) rather than to this corpus-graduation tree.

1. **⛔ Do NOT bootstrap the design from `.3.11`.** That case was measured to have
   **zero** real-world conflict (0 of 16,336 files write a spaced time literal;
   273 write it tight), so it needs no switch — it is a plain accepts-invalid bug.
   A general mechanism designed around a single non-contentious instance will be
   the wrong mechanism.
2. **⭐ The evidence base already exists and is measured: the accepts-invalid
   population — 21 rows (`sv_2017` lane) + 14 rows (`verilog_2005` lane) = 35.**
   Those are, by construction, every place PGEN currently accepts what the standard
   forbids. Sorting those 35 into *(a) genuine dialect tolerance the ecosystem
   relies on* vs *(b) plain over-acceptance bugs* is the real input to a strictness
   policy: bucket (a) is what a switch is FOR, bucket (b) should simply be fixed.
   That triage is a read-only leaf and should come first.
3. **⛔ TWO HARD CONSTRAINTS ON THE MECHANISM:**
   - **It must be EBNF-NATIVE.** `EBNF-SOURCE-OF-TRUTH` is a *mechanically
     enforced* doctrine (`scripts/check_doctrines.sh`: "no new out-of-band
     acceptance validator wired outside the EBNF"). A strictness switch bolted on
     as a runtime parser flag would fail that gate on the pre-commit hook. The
     existing `@profiles` annotation is the proof that an EBNF-native switch is
     achievable.
   - **It must be ORTHOGONAL to `@profiles`, not folded into it.** Profiles answer
     *which standard* (`sv_2017` / `sv_2023` / `verilog_2005`); strictness answers
     *how pedantically to enforce it*. Folding the second into the first gives a
     combinatorial explosion (`sv_2017_strict`, `sv_2017_lax`, … ×3) and would
     muddle a mechanism that is currently clean, gate-verified
     (`profile_orphans=0`) and load-bearing for two cert contracts.

**Recommended sequencing:** triage the 35 accepts-invalid rows into
dialect-tolerance vs bug (read-only) → design the orthogonal EBNF-native strictness
annotation against bucket (a) → implement → wire a gate that proves both settings
behave as declared. Fix `.3.11` strictly in the meantime; it is independent of all
of the above.
