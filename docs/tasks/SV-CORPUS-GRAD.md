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
  116→62). **Current rejects-valid baseline after `.3.7`: 406** (accepts-invalid
  21; v2005 62).

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
  instrument + the FIRST measured number, 91.1% / 120 gaps) **done**; `.7b`
  (clause matrix from the keyed suites + negatives-density report) **done**
  (`PGEN-SV-CORPUS-GRAD-0016`, session #197). Both coverage lenses now stand:
  the authoritative rule-participation % (`.7a`) and the LRM-structure clause
  matrix + negatives density (`.7b`). The measured worklists feed `.9`.

#### `.7a` — The rule-coverage instrument: measured coverage = 91.1% (120 gaps)

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

- **Status: `todo`** — every `.7`-reported uncovered rule/clause gets a
  corpus case: sourced from the ADD tiers, or crafted directly from the
  in-repo LRM markdown (`docs/systemverilog/2017`/`2023`) with the clause
  cited (externally-grounded, never generator-derived — external means
  externally authored). Loop until 100% of the parseable surface is
  exercised or N/A-with-cause. Feeds `.5`'s widened criterion.

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
