# SV-REPLAY-DEBT: drive `focused_replay_target_debt_zero` — the LAST unmet SV family-status `Done` criterion — to literal 0 (or an honest, named irreducible remainder)

## Metadata

- Tree ID: `SV-REPLAY-DEBT`
- Status: **`active`** (created 2026-07-22, session #190 — the director-corrected PNT
  frontier: "But SV is not Done yet"; the `STRUCTURED-WITNESS-SYNTH` tree closed the
  locked program's CERTIFICATE axis, but SV's family-status `Done` bar has ONE unmet
  criterion left — `focused_replay_target_debt_zero`)
- Roadmap lane: the **locked program** (director 2026-06-08: all parsers → `Done`)
  + ⭐ the **Nexsim delivery directive** (director 2026-07-22, verbatim: "NEXSIM
  needs a sota, signoff SV parser, so we need to focus on that and deliver" —
  [[project_nexsim_sv_signoff_delivery_focus]]): the SV parser is the delivery
  focus; this tree is step 1 of that path. SystemVerilog's machine-computed
  family-status (`sv_parser_family_status_gate`) is `Mostly Done` with 6/7
  `Done`-criteria met; this tree owns the 7th.
- Created: `2026-07-22`
- Owner: repo-local workflow
- Cross-links: lineage = `SV-EXH-PROOF.7` (the 2026-06 closed-loop coverage
  campaign: ~2660 → 97 via steering → Purdom → budget → derivation-directed
  CONSTRUCTION, `-0148`/`-0149`) + `.7.4.6.x` (the metric-noise diagnosis) +
  `GRAMMAR-WELLFORMED.B1` (`PGEN-GRAMMAR-WELLFORMED-0005`, 2026-06-05: the
  wall-clock generation deadline replaced by a DETERMINISTIC step-counter budget —
  the canonical residual then read **84 IDENTICAL across two independent runs**).
  The handoff pointer that opened this tree: commit `f48a60b2`
  (`PGEN-STRUCTURED-WITNESS-SYNTH-HANDOFF-0002`).

## Goal (the tree's single deliverable)

`sv_parser_family_status_gate` reports `focused_replay_target_debt_zero: true`
(`focused_replay_target_count = 0`, read from the canonical closed-loop replay's
`profile_2017_replay_gap.json` target list) — **or** an honest, tool-proven
classification of a named irreducible remainder (rigorously proven out of the
coverable universe, never gamed, per
[[feedback_corpus_expected_from_spec_not_fix]]).

⭐ BAR AMENDED same-day (director 2026-07-22,
[[project_sv_done_requires_external_corpus_graduation]]): this tree is **axis 1
of TWO** — the SV family row flips `Done` only when BOTH this criterion AND the
external-corpus graduation (sibling tree `SV-CORPUS-GRAD`) are green. Note the
convergence: this tree's dominant residual cluster (`prop_primary_*` /
assertion / sequence rules) and the corpus campaign's known
`GRAMMAR-WELLFORMED.H.12.5.8` infix property/sequence parse bug live in the
SAME grammar region.

## The mechanical criterion chain (pinned 2026-07-22)

- `rust/scripts/sv_parser_family_status_gate.sh:443` — `Done` requires
  `focused_replay_target_count == 0` (one of 7 criteria; 6 already green).
- The count = `((.targets // []) | length)` of
  `<quality-state>/work/profile_2017_replay_gap.json`
  (`rust/scripts/sv_parser_aggregate_contract_gate.sh:275`), produced by the
  canonical closed-loop replay inside `make -C rust sv_stimuli_quality_gate`.
- NOTE the criterion reads **profile 2017 only**; the quality gate's own
  `closed_loop_replay_targets_total` is the BOTH-profiles (2017+2023) sum — the
  historical arc numbers (97/89/105/120, then 84 deterministic) are that total.

## Historical arc (context for honest comparison)

| Date | Reading (both-profiles total) | Mechanism |
|---|---|---|
| ~2026-05 | ~2660 initial | pre-campaign |
| 2026-06-04 | 888 → 753 → 273 | `.7.2` steering → `.7.4.4` Purdom → `.7.4.5` budget |
| 2026-06-04 | **97** (`-0149`) | `.7.4.6.3` derivation-directed construction |
| 2026-06-05 | 97/89/105/120 | `.7.4.6.6`: metric NOISE-DOMINATED (±~25, wall-clock budget) |
| 2026-06-05 | **84 deterministic** (two identical runs) | `GRAMMAR-WELLFORMED.B1` step-counter budget |
| 2026-07-22 | **120** (2017: **59** + 2023: 61) | this tree's `.1` re-baseline (run 1; ~20 SV releases later, grammar grown to 1343/1362 rules) |

The 84 → 120 delta spans ~20 SV releases of grammar change (LRM-bracket /
`$`-anchor / covergroup restorations, `verilog_2005` profile gating, dead-branch
de-dup −25, …) — the target UNIVERSE moved, so the delta is not a regression
signal by itself. The June-era non-covering cluster (`sequence_expr #0..#11`) is
GONE from today's list; the population has genuinely shifted.

## Leaves

### `.1` — Re-baseline the debt at today's vintage (tools-first, read-only) + determinism verification

- **Status: `done`** (`PGEN-SV-REPLAY-DEBT-0001` tree+run-1 / `-0002` determinism
  close, session #190, 2026-07-22; READ-ONLY — zero code/grammar/generated
  change). **VERDICT: the baseline is SIGNAL — `focused_replay_target_count = 59`
  (profile 2017, the criterion) / 120 both-profiles, DETERMINISTIC: all four
  gap artifacts BYTE-IDENTICAL (sha256-verified) across two independent
  canonical runs at HEAD vintage; the `GRAMMAR-WELLFORMED.B1` step-counter
  determinism holds end-to-end at today's vintage (the post-B1 gate-local 5 ms
  drive budget did NOT re-introduce wobble). Evidence
  `docs/tasks/artifacts/sv_replay_debt/determinism_verdict_run1_vs_run2.txt`.**
- **Goal:** replace the stale ~97–105 (pre-determinization, 2026-06-05-era) anchor
  with a measured, determinism-verified number at today's vintage, and bank the
  full target list the burn-down will be scoped from.
- **Run 1 (existing canonical evidence, 2026-07-22 00:06):** the session-#189 `.3`
  battery ran the canonical `make -C rust sv_stimuli_quality_gate` (state dir
  `rust/target/sv_stimuli_quality_gate`, run span 23:39–00:06) at HEAD's exact
  code vintage (commit `c847254c` code; every later commit docs-only). Contract
  config confirmed from `summary.txt`: `closed_loop_target_max_attempts: 5000`
  (`source: contract`), `closed_loop_target_generation_timeout_ms: 5` (gate-local
  default), profiles 2017+2023 both PASS. Readings:
  - `closed_loop_replay_targets_total = 120` (initial 5394);
  - **`focused_replay_target_count` (profile 2017, the criterion) = 59** — 58
    branch targets + 1 rule target;
  - profile 2023 = 61 (60 branch + 1 rule);
  - reasons (2017): `selected_but_failed=41`, `never_selected=17`, `never_hit=1`;
  - concentration (2017): `prop_primary_sv_2017` **24**,
    `concurrent_assertion_statement` 4, `method_call_receiver_sv_2017` 4,
    `ps_or_hierarchical_array_identifier` 3, … (2023 mirrors: `prop_primary_sv_2023`
    24, near-identical family split);
  - the ONE rule-type target (both profiles): `rule::wildcard_escape_nettype_identifier`
    (`reason=never_hit`, `reachable=true`);
  - end-of-replay coverage (2017): `covered_reachable_rules 1323/1324`,
    remaining reachable-branch gap 58 (`reachable_branches` is the DYNAMIC
    deficit>0 set per `.5.2.3`).
  - Evidence: `docs/tasks/artifacts/sv_replay_debt/canonical_run_20260722_summary_extract.txt`
    + `profile_{2017,2023}_replay_gap_breakdown_run1.txt`
    + `profile_{2017,2023}_replay_gap_target_ids_run1.txt` (the full sorted target-id lists).
- **Run 2 (determinism verification — the closed-loop verdict is EARNED; the
  gate itself hit an INFRA defect at its summary stage, see `.1b`):** guarded
  canonical re-run (budget 16384 MB, separate state dir
  `rust/target/sv_stimuli_quality_gate_rebaseline`, 1,845 s, peak 12,149 MB).
  Every closed-loop stage completed `ok` and ALL FOUR gap artifacts are
  BYTE-IDENTICAL run1↔run2 (`profile_2017_initial_replay_gap` 2,659 =
  `d4ec5e225af2`; `profile_2017_replay_gap` 59 = `77b9de8419ae`;
  `profile_2023_initial_replay_gap` 2,735 = `3949d27068dc`;
  `profile_2023_replay_gap` 61 = `66a920b2ed32`). The gate then FAILED (exit 2)
  at `sv_stimuli_quality_gate.sh:2764` — `jq: Argument list too long` — a
  gate-SCRIPT robustness defect (root-caused + owned by `.1b` below), NOT a
  parser/coverage/determinism failure: the failure is downstream of the
  finalized gap artifacts. The identical-runs outcome holds ⇒ the burn-down is
  scoped directly from the banked target list; no re-determinization needed.
  `.1b`'s fixed-gate re-run supplies the end-to-end gate-green confirmation.
- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — `sv_parser_family_status_gate.sh:443` requires `focused_replay_target_count == 0`; run-1 canonical gate reads 59 (2017) / 120 total: `closed_loop_replay_targets_total: 120` + `jq '(.targets // []) | length' profile_2017_replay_gap.json` → 59 (banked extracts).
  - [x] **ROOT CAUSE (WHY + WHERE)** — the debt's composition pinned from the replay-gap target records (the WHY per target is the `reason` field; the WHERE is the `id`/`node_path`): 41/59 `selected_but_failed` (replay drives the branch, witness fails), 17 `never_selected`, 1 `never_hit`; dominated by `prop_primary_sv_2017` root#N branches (24). Full lists banked in `docs/tasks/artifacts/sv_replay_debt/`.
  - [x] **FIX** — N/A (read-only re-baseline leaf; no change made).
  - [x] **ADDRESSED (verified)** — run-2 determinism verdict: all four gap JSONs BYTE-IDENTICAL run1↔run2 (sha256 pairs in `determinism_verdict_run1_vs_run2.txt`; run-1 gate exit 0; run-2 closed-loop stages all `ok`, its summary-stage infra failure root-caused + owned by `.1b`) ⇒ the stale ~97–105 anchor is REPLACED by the verified deterministic baseline 59 (2017 criterion) / 120 (total).
  - [x] **NO REGRESSION** — read-only: zero code/grammar/generated/contract change; run 2 writes to a SEPARATE state dir; git tree clean of any code diff.
  - [x] **LOCKSTEP** — MEMORY.md frontier + docs/TASK_TREE.md index row + CHANGES.md entry this commit; LIVE tracker row unchanged (`Mostly Done` stands — this leaf only re-measures).

### `.1b` — Gate-script ARG_MAX robustness fix (code; found by `.1` run 2)

- **Status: `done`** (`PGEN-SV-REPLAY-DEBT-0002` fix / `-0003` end-to-end close,
  session #190). **END-TO-END CONFIRMED: the fixed gate ran GREEN (exit 0,
  1,922 s, peak 12,647 MB) at a state dir LONGER than the failing run's; all
  four gap artifacts BYTE-IDENTICAL to the deterministic baseline (a THIRD
  independent reproduction — determinism triple-confirmed); summary totals
  identical to run-1 (5394 / 120, profiles 2/2).** Corrected observation
  banked: the ~35 GB/run shadow trace logs are written UNCONDITIONALLY (the
  success path keeps them too) — log gating/rotation joins the
  sibling-hardening follow-up alongside the 10-site `--argjson` census.
- **The defect:** `rust/scripts/sv_stimuli_quality_gate.sh` passes the ENTIRE
  realistic-corpus per-case JSON array as ONE `jq --argjson` command-line
  argument (`:2793`, program at `:2764`). Measured: macOS `ARG_MAX=1,048,576`;
  run-1's cases JSON = 1,002,686 B (the DEFAULT state dir passes with <46 KB
  headroom — by luck); run-2's = 1,034,802 B (the 11-chars-longer custom state
  dir inflates the 730 embedded paths) ⇒ argv+env > ARG_MAX ⇒ E2BIG ⇒
  `jq: Argument list too long` ⇒ make Error 126 (guard exit 2, 1,845 s in).
  The canonical gate runs within ~4% of the kernel argv limit at DEFAULT
  config — any growth (more cases, longer checkout path, custom state dir)
  breaks it.
- **The fix (minimal):** pass the cases via a FILE, not argv — materialize
  `jq -s` into `<jsonl>.aggregate.json` and hand it to the summary `jq -n` via
  `--slurpfile cases_slurp` (+ `cases: $cases_slurp[0]`); the `[]` empty-case
  guard keeps its semantics by materializing `[]` into the same file.
- **Sibling census (recorded, NOT changed here — one commit = one defect):**
  the same slurp-var-then-`--argjson` shape exists at 10 sites / 9 gate scripts
  (`regex_broader_corpus_proof_gate:285`, `sv_declared_shadow_promotion_gate:559`,
  `sv_parse_full_ratio_promotion_gate:551`, `sv_external_corpus_triage_gate:575`,
  `sv_preprocessor_template_differential_gate:542`,
  `vhdl_external_corpus_triage_gate:240`,
  `sv_preprocessor_curated_differential_gate:302`,
  `sv_preprocessor_quality_gate:1011`, `sv_stimuli_quality_gate:2539` +
  the fixed `:2761`) — all sub-threshold payloads today (14–44 cases); the
  hardening sweep is a follow-up slice when any grows.
- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — the run-2 production failure (`jq: Argument list too long`, make Error 126, guard exit=2 at 1,845 s) + standalone repro on run-2's exact jsonl: the old `--argjson` shape exits 126 with the identical message (`argmax_fix_standalone_verification.txt`).
  - [x] **ROOT CAUSE (WHY + WHERE)** — `--argjson cases "$realistic_cases_json"` (`sv_stimuli_quality_gate.sh:2793`, program `:2764`) puts the ~1 MB cases array on execve argv: measured 1,034,802 B (run 2) vs `ARG_MAX=1,048,576` with env ⇒ E2BIG; run 1's 1,002,686 B passed with <46 KB headroom by luck.
  - [x] **FIX** — script-only, minimal: materialize `jq -s` into `<jsonl>.aggregate.json` (with the `[]` empty-guard) and pass via `--slurpfile cases_slurp` + `cases: $cases_slurp[0]`; `bash -n` clean.
  - [x] **ADDRESSED (verified)** — before→after on the symptom: old shape exit 126 on run-2's jsonl → fixed shape exit 0 on the SAME input (1,096,872 B emitted). END-TO-END EARNED: the full guarded gate re-run at the LONGER state dir exited 0 (marker `status=completed exit=0`, 1,922 s).
  - [x] **NO REGRESSION** — byte-identity oracle: on run-1's data the old and new mechanisms emit BYTE-IDENTICAL JSON (1,064,756 B, `cmp` clean); AND the green re-run's four gap artifacts are BYTE-IDENTICAL to the deterministic baseline with summary totals matching run-1 (5394/120, 2/2).
  - [x] **LOCKSTEP** — CHANGES.md entry; no user-facing surface/book change (internal gate-script robustness).

### `.2` — Burn-down scoping from the VERIFIED baseline (design, pure-docs)

- **Status: `todo`** — NEXT. The deterministic target list (banked, 59+61) burns
  down cluster by cluster, each with a toolbox-first WHY+WHERE leaf before any
  fix:
  - the `prop_primary_*` `selected_but_failed` cluster (24×2 root-level
    branches — the dominant mass). ⭐ CONVERGENCE RESOLVED (session #190,
    `h1258_matrix_at_head.txt`): the `H.12.5.8` infix parse bug is ALREADY
    FIXED (releases 1.0.148/1.0.149, 2026-06-25; 12/12 REJECT→PASS re-verified
    at HEAD) — and those cascade releases CREATED `prop_primary_*`/`seq_*`
    (+16 branches/profile), so this cluster IS the cascades' new branch
    universe not yet covered by replay: a GENERATION-COVERAGE target (why does
    the driven branch's witness fail to parse — per-branch probe via the
    `.7.4.6.4` timeout/unresolved sampling tooling), which also explains the
    84→120 debt growth mechanistically;
  - the assertion/statement cluster (`concurrent_assertion_statement` 4,
    `method_call_receiver_*` 4, `ps_or_hierarchical_array_identifier` 3, …);
  - the `never_selected` cluster (17/19 — reach-forcing question, cf.
    [[project_sv7_never_selected_rootcause]]: reachability, not weighting);
  - the `wildcard_escape_nettype_identifier` `never_hit` rule target (both
    profiles — the store-gated wildcard-import rule from `SV-PARSE-STRICT.2`).
- Fix hierarchy per [[feedback_no_workarounds_fix_hierarchy]]; generator-side
  changes must stay parser-agnostic + capability-gated (the PASS-3f precedent).

## Acceptance Criteria (tree)

1. `sv_parser_family_status_gate` green with `focused_replay_target_debt_zero: true`
   AND the SV family row honestly flipped to `Done` — or the residual is a named,
   tool-proven irreducible set with the proof recorded and the criterion/universe
   corrected rigorously (never gamed).
2. Every landing leaf passes the TOOLBOX.md acceptance checklist (tool-backed
   WHY+WHERE + measured before→after + no regression at seeds 0/7/42, SV external
   corpus 14/14, realistic corpus green).
3. Books/LIVE/contract lockstep on any user-visible change.
