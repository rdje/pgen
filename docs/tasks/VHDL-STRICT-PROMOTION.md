# VHDL-STRICT-PROMOTION: VHDL strict-promotion floor ratchet

## Metadata

- Tree ID: `VHDL-STRICT-PROMOTION`
- Status: `done` (single leaf `.1` complete `PGEN-VHDL-STRICT-PROMOTION-0001`)
- Roadmap lane: VHDL family — no-regression-baseline hardening (`LIVE_ACHIEVEMENT_STATUS.md` `vhdl` row = `Done`; roadmap: "Preserve the closed VHDL family/status/aggregate proof stack as a no-regression baseline")
- Created: `2026-07-01`
- Owner: repo-local workflow

## Goal

Convert the VHDL strict-promotion gate's parse-full ratio check from a no-op
(`TARGET_MIN_RATIO=0`, which accepts any ratio ≥ 0%) into an enforced,
evidence-backed no-regression floor, so a future regression that degrades VHDL
closed-loop stimuli parse-full quality below the demonstrated level trips the
gate instead of passing silently. Keep the gate green + its recommendation
(`enable_required_strict_mode`) unchanged, and document the strict-promotion
gate in the live book (currently undocumented there).

## Non-Goals

- Do NOT change the VHDL grammar, generated parser, AST schema, or release
  version (this is a gate/policy + docs change, not a parser change).
- Do NOT actually flip `PGEN_VHDL_STRICT_PROMOTION_MODE` to required-enforce or
  wire downstream required-strict-mode enforcement — that is a separate,
  larger lane if the director prioritizes it.
- Do NOT rewrite dated historical tracker notes in `LIVE_ACHIEVEMENT_STATUS.md`
  (e.g. the 2026-03-17 `observed_ratio_min=12` note is correct history for its
  date, not drift).

## Acceptance Criteria

- The strict-promotion floor is raised from `0` to an evidence-backed
  conservative value in every path that runs the gate (direct gate default +
  the `sota_exit_gate` policy default), keeping the env override intact.
- The floor is set below the demonstrated sustained minimum with margin, from a
  measured multi-seed sweep (not guessed).
- `make -C rust SHELL=/bin/bash vhdl_strict_promotion_gate` stays green with
  `recommendation=enable_required_strict_mode`, `primary_blocker=none`,
  `trial_passed=3`, deterministic across the fixed gate seeds.
- `make -C rust SHELL=/bin/bash vhdl_parser_family_contract_gate` stays green
  (VHDL `Done` no-regression baseline preserved).
- The live book documents the strict-promotion gate + the ratcheted floor;
  `mdbook_docs_gate` passes.
- Live docs updated; committed through `COMMIT.md`.

## Task Tree

- ID: `VHDL-STRICT-PROMOTION`
  Status: `active`
  Goal: ratchet the VHDL strict-promotion parse-full floor with evidence + book lockstep
  Children: `VHDL-STRICT-PROMOTION.1`

- ID: `VHDL-STRICT-PROMOTION.1`
  Status: `done`
  Goal: raise `TARGET_MIN_RATIO` `0 -> 75` (gate default + sota policy default), re-verify green, document the gate in the book
  Acceptance: gate + family-contract gate green with the new floor; recommendation/eligibility unchanged; book section added + mdbook gate green
  Verification: `done` — see Verification Log
  Commit: `PGEN-VHDL-STRICT-PROMOTION-0001`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | — | — | Frontier empty — tree `done` (`.1` complete) |

## Decisions

- `2026-07-01`: Floor value = `75`. Evidence: an 8-seed sweep
  (`SEED_STRIDE=41000`, seeds 22001..309001) plus the canonical 3-seed gate run
  (seeds 22001/272001/522001) — **all 11 distinct seeds observed 100%
  parse-full (8/8 each)**. `75` = 2 sample-steps (25%) of margin below the
  demonstrated minimum: a strong ratchet (far above the no-op `0` and the
  2026-03-17 historical low of `12`) that still tolerates a benign 2-sample
  RNG-stream perturbation on the fixed gate seeds without a false trip.
- `2026-07-01`: Bump BOTH `vhdl_strict_promotion_gate.sh` (gate default) and
  `sota_exit_gate.sh` (`POLICY_VHDL_STRICT_PROMOTION_TARGET_MIN_RATIO` default),
  because `sota_exit_gate` passes its own explicit `TARGET_MIN_RATIO` env to the
  gate — bumping only the gate default would leave the floor active in the
  family-contract path but inactive under sota (inconsistent). The env override
  (`PGEN_VHDL_STRICT_PROMOTION_TARGET_MIN_RATIO`, `PGEN_SOTA_POLICY_...`) is
  preserved so `0` can be restored deliberately.
- `2026-07-01`: This lane is owned by a task tree even though the mechanical
  code-change classifier (`scripts/check_diagnosis_evidence.sh`) treats only
  `grammars|rust/src|generated|ast_shape_contract` as code — the resume pointer
  and director directive require task-tree ownership for this change, and it
  gives the VHDL strict-promotion roadmap lane an explicit home.

## Open Questions

- Whether to later actually enable required-strict-mode (the gate's standing
  recommendation) — deferred; own its own leaf/lane if the director prioritizes.
  Does not block this frontier.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-01` | `VHDL-STRICT-PROMOTION.1` | 11-seed sweep (evidence) | all 100% parse-full (8/8 each) — 8-seed sweep (22001..309001) + canonical 3-seed (22001/272001/522001) |
| `2026-07-01` | `VHDL-STRICT-PROMOTION.1` | `make vhdl_strict_promotion_gate` (ADDRESSED, after 0→75) | GREEN — `target_min_ratio=75`, `recommendation=enable_required_strict_mode`, `eligible=1`, `trial_passed=3`, `observed_ratio 100/100/100`, `primary_blocker=none` |
| `2026-07-01` | `VHDL-STRICT-PROMOTION.1` | `make vhdl_parser_family_contract_gate` (NO REGRESSION) | GREEN — strict-promotion sub-report `target_min_ratio=75`, VHDL `Done` baseline intact |
| `2026-07-01` | `VHDL-STRICT-PROMOTION.1` | `make mdbook_docs_gate` (book lockstep) | GREEN — `pass: mdbook_build` |

Notes: not a code change per the mechanical classifier (`grammars|rust/src|generated|ast_shape_contract`) — only `rust/scripts/*.sh` + docs — so clippy N/A and no parser/grammar/schema/release change (6 fully-certified grammars + SV inert by construction). Full aggregate `sota_exit_gate` not re-run (heavy); its strict-promotion sub-stage uses the same gate at the same lockstepped floor 75, proven green above.

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `VHDL-STRICT-PROMOTION.1` | `PGEN-VHDL-STRICT-PROMOTION-0001` (leaf `VHDL-STRICT-PROMOTION.1`) | ratchet `0->75` (gate + sota policy) + book lockstep |

## Changelog

- `2026-07-01`: Created task tree; gathered 11-seed sweep evidence (all 100%);
  chose floor `75`; implemented ratchet (gate + sota policy defaults), verified
  green (strict-promotion + family-contract + mdbook gates), added the book
  section; leaf `.1` and the tree `done`.
