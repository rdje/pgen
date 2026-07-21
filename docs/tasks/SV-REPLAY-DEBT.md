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

- **Status: `in_progress`** (`PGEN-SV-REPLAY-DEBT-0001`, session #190, 2026-07-22;
  READ-ONLY — zero code/grammar/generated change).
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
- **Run 2 (determinism verification, IN FLIGHT):** a second canonical run under
  the memory guard (`--budget-mb 16384 --timeout-s 7200`, state dir
  `rust/target/sv_stimuli_quality_gate_rebaseline`) — the B1 determinism claim
  ("two independent runs identical") re-verified at TODAY'S vintage by
  byte-comparing the sorted target-id lists per profile. Outcomes:
  - identical ⇒ the baseline is **signal** (59/61/120) and the burn-down is
    scoped directly from the banked target list;
  - divergent ⇒ determinism REGRESSED since B1 (suspect: the gate-local 5 ms
    wall-clock `closed_loop_target_generation_timeout_ms` shell default, added
    after B1) — then re-determinizing the metric becomes the FIRST burn-down
    leaf, before any coverage push (a literal-0 claim cannot be proven against
    a noisy number).
- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — `sv_parser_family_status_gate.sh:443` requires `focused_replay_target_count == 0`; run-1 canonical gate reads 59 (2017) / 120 total: `closed_loop_replay_targets_total: 120` + `jq '(.targets // []) | length' profile_2017_replay_gap.json` → 59 (banked extracts).
  - [x] **ROOT CAUSE (WHY + WHERE)** — the debt's composition pinned from the replay-gap target records (the WHY per target is the `reason` field; the WHERE is the `id`/`node_path`): 41/59 `selected_but_failed` (replay drives the branch, witness fails), 17 `never_selected`, 1 `never_hit`; dominated by `prop_primary_sv_2017` root#N branches (24). Full lists banked in `docs/tasks/artifacts/sv_replay_debt/`.
  - [x] **FIX** — N/A (read-only re-baseline leaf; no change made).
  - [ ] **ADDRESSED (verified)** — run 2 determinism verdict (in flight): sorted target-id lists byte-compared run1 vs run2 per profile.
  - [x] **NO REGRESSION** — read-only: zero code/grammar/generated/contract change; run 2 writes to a SEPARATE state dir; git tree clean of any code diff.
  - [x] **LOCKSTEP** — MEMORY.md frontier + docs/TASK_TREE.md index row + CHANGES.md entry this commit; LIVE tracker row unchanged (`Mostly Done` stands — this leaf only re-measures).

### `.2` — Burn-down scoping from the verified baseline (design, pure-docs)

- **Status: `todo`** (opens after `.1`'s determinism verdict).
- Scope the burn-down from the banked target list, cluster by cluster, each with
  a toolbox-first WHY+WHERE leaf before any fix:
  - the `prop_primary_*` `selected_but_failed` cluster (24×2 branches — the
    dominant mass; first question: WHY does a driven branch's witness fail —
    per-branch probe via the `.7.4.6.4` timeout/unresolved sampling tooling);
  - the `never_selected` cluster (17/19 — reach-forcing question, cf.
    [[project_sv7_never_selected_rootcause]]: reachability, not weighting);
  - the `wildcard_escape_nettype_identifier` `never_hit` rule target;
  - any determinism repair if `.1` run 2 diverges (then FIRST).
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
