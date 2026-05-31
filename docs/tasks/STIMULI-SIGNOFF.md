# STIMULI-SIGNOFF: Signoff-Grade EBNF Stimuli Generator (capability-gap closure)

## Metadata

- Tree ID: `STIMULI-SIGNOFF`
- Status: `proposed`
- Roadmap lane: `Stimuli generator → best-in-class / signoff-grade (user vision 2026-05-31)`
- Created: `2026-05-31`
- Last updated: `2026-05-31`
- Owner: repo-local workflow

## Goal

Make PGEN's EBNF-based stimuli generator (`rust/src/ast_pipeline/stimuli_generator.rs`)
signoff-grade / best-in-class / "breathtaking" per the user vision
([[project_stimuli_generator_signoff_vision]]): a deliberate capability-gap
audit of what a signoff EBNF stimuli generator SHOULD do that ours doesn't yet,
then close those gaps as GENERAL, parser-agnostic grammar-structure capabilities.

## Non-Goals

- No hardcoded rule-names/sigils — every capability is a GENERAL grammar-structure
  property ([[feedback_ast_pipeline_parser_agnostic]]).
- Not a rewrite for its own sake — gaps must be evidenced (e.g. by the
  `SV-EXH-PROOF.7` `focused_replay_target_debt` data).
- Correctness before speed ([[feedback_correctness_before_speed]]).

## Acceptance Criteria

- A capability-gap audit (what a signoff generator should do vs. ours) recorded.
- Each accepted gap → a leaf; each landed as a parser-agnostic, signoff-verified
  capability (corpus/gate-proven, no regression across families).
- Candidate axes: guaranteed/exhaustive coverage closure (not best-effort) with
  machine-checkable proof; depth strategy reaching deep-but-rare branches;
  deterministic target-reach/path-forcing; semantic-store-aware valid-by-
  construction generation; counterexample shrinking; negative/invalid stimuli;
  scale/perf; targeted generation; in-memory↔generated-module parity.
- Each completed leaf committed through `COMMIT.md`; any code change owns a leaf.

## Task Tree

- ID: `STIMULI-SIGNOFF`
  Status: `proposed`
  Goal: `Signoff-grade, parser-agnostic EBNF stimuli generator via capability-gap closure.`
  Children: `STIMULI-SIGNOFF.1`

- ID: `STIMULI-SIGNOFF.1`
  Status: `pending`
  Goal: `CAPABILITY-GAP AUDIT (pure docs): enumerate, out-of-the-box, what a signoff EBNF stimuli generator SHOULD do that ours doesn't yet; seed it with the SV-EXH-PROOF.7 reachability finding ([[project_sv7_never_selected_rootcause]] — 358 never_selected branches are a reachability, not weighting, problem). Produce a prioritized gap list → ordered leaf plan. Tools-first; no code until a code leaf owns it.`
  Acceptance: `Prioritized capability-gap list + ordered leaf plan recorded; each gap tagged parser-agnostic-by-design.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `STIMULI-SIGNOFF.1` | `pending` (proposed) | The audit defines the lane; `SV-EXH-PROOF.7` is already surfacing concrete gaps to seed it. |

## Decisions

- `2026-05-31`: Created as a thin owning skeleton by `TASKTREE-GOV.2` to own the user's stimuli-generator-signoff vision. `SV-EXH-PROOF.7` (close `focused_replay_target_debt_zero`) feeds this tree: its generator limitations are audit data points, but `.7` itself stays owned by SV-EXH-PROOF (SV closure) — STIMULI-SIGNOFF owns the GENERAL capability-gap closure.

## Open Questions

- Does the first concrete capability (deterministic target-reach/path-forcing, from `.7`) land under SV-EXH-PROOF.7 or graduate here? (resolve when `.7` resumes)

## Blockers

- None (vision lane; `SV-EXH-PROOF.7` is the active feeder).

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-31` | `STIMULI-SIGNOFF.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `STIMULI-SIGNOFF.1` | `pending` | `pending` |

## Changelog

- `2026-05-31`: Created thin skeleton (TASKTREE-GOV.2 roadmap-coverage).
