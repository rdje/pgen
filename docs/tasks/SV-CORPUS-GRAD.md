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
