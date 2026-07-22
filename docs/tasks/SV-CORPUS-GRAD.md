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
  acquisition** (the old-suite 279 reproduced EXACTLY + 166 newly measured:
  ispras-1800 114 / sv2v 31 / ivtest 21 — see `.8a`; new families join the
  leaf-cutting map as burn-down proceeds). Pre-`.8a` detail (273
  rejects-valid + 6 accepts-invalid; the manifest is the per-row worklist,
  the `.3.0` family table the leaf-cutting map — F1 CLEARED by `.3.1`).
  Suite split: verilator 220, sv-tests 39
  (generic 11, chapter-5/lexical 9 — incl. the probe-verified
  underscore/spaced-literal gap — chapter-7 4, chapter-6 4, chapter-16/SVA 4,
  chapter-18 3, chapter-11 2, chapter-8 1, chapter-12 1), verible 13,
  slang 1, + the 6 named accepts-invalid rows (verilator-strictness
  adjudication candidates). Next family candidates: F2 SVA tails / F3
  constraint-randomize / F5 in-scope directives — but corpus ACQUISITION
  (`.8`) outranks further burn-down per the director's 2026-07-22 order
  ([[project_sv_corpus_100pct_lrm_coverage_mandate]]).

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

- **Status: `in_progress`** (mandated by
  [[project_sv_corpus_100pct_lrm_coverage_mandate]]) — `.7a` (the rule-coverage
  instrument + the FIRST measured number) **done**; `.7b` (clause matrix from
  the keyed suites + negatives-density report) todo.

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

#### `.7b` — Clause matrix + negatives density (todo)

- **Status: `todo`** — the clause matrix from the keyed suites (sv-tests
  `:tags:`, ispras clause-encoded filenames, ivtest keys) + a
  negatives-density report (keyed `must_reject` per LRM chapter — the
  thinnest axis per the sufficiency assessment), joined with `.7a`'s
  rule-level view.

### `.8` — ADD-v1 corpus vendoring (the director-ordered acquisition)

- **Status: `in_progress`** — `.8a` (vendoring + runner fold + answer keys +
  fresh baseline) **done**; `.8b` (deep key extraction) + `.8c` (v2005-profile
  lane run) remain. Roster-v2 candidate logged: **slang embedded-unittest
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

#### `.8b` — Deep answer-key extraction (todo)

- **Status: `todo`** — (1) Surelog per-test accept/error key extraction from
  drivers/golden logs (828 rows now chained-deferred); (2) sv2v `error/`
  conversion-error vs parse-error pre-triage (234 rows); (3) ispras NEGATIVE
  per-file stage triage (23 files — sampled member is semantic-stage);
  (4) ivtest CE-without-gold stage triage (~180 rows); (5) the ivtest
  `no_sv_key` population sweep (vvp_tests JSON descriptors as a secondary
  key source where SV-dialect).

#### `.8c` — The verilog_2005 profile lane (todo)

- **Status: `todo`** — the 2,458 `deferred:v2005_profile_lane` rows (ispras
  `ieee-1364-2005/` 360 + ivtest `regress-vlg.list` + sv2v goldens) are the
  frozen-roster v2005 corpus: a `--profile verilog_2005` bulk-runner mode +
  profile-aware adjudication (mind ispras `KNOWN_TEXT_BUGS`), feeding the
  v2005 arm of `.5`/`.7`.

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
