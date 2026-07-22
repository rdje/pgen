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

- **Status (umbrella): `todo`** — SCOPED: **321 unexplained divergences** to
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

## Acceptance Criteria (tree)

1. Fresh characterization at HEAD vintage + a complete adjudication manifest:
   every vendored case classified with a metadata/spec-derived expected verdict.
2. The parser matches ALL expected verdicts; exclusions named + justified in the
   tracked manifest; **zero unexplained divergences** ("flying colors", earned).
3. The graduation gate is standing, deterministic (pinned corpus commits), and
   wired into the SV family-status `Done` computation.
4. Full lockstep (books/LIVE/contract) and every landing leaf through the
   TOOLBOX acceptance checklist.
