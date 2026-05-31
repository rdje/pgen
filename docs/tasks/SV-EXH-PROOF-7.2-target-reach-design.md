# SV-EXH-PROOF.7.2 — Deterministic Target-Reach / Path-Forcing (DESIGN)

> Companion design doc for task-tree leaf `SV-EXH-PROOF.7.2`
> (see `docs/tasks/SV-EXH-PROOF.md`). **Pure design — no code in this
> slice.** Implementation is decomposed into separate code-leaves
> `.7.2.1+`, opened only after this design is accepted.
>
> Status: DESIGN drafted 2026-05-31. This is pure design (docs only), so
> nothing gates writing it down. The `.7.2.1+` *implementation*
> code-leaves remain BLOCKED until BOTH (a) the `.6` clean-run
> confirmation lands — a full `sv_parser_family_status_gate` run green
> end-to-end, **still OUTSTANDING as of this writing** (run it via
> `bash rust/scripts/sv_parser_family_status_gate.sh`, not a root `make`
> target — there is no root Makefile) — AND (b) this design is accepted.

---

## 1. The problem this solves

The single remaining short Done-criterion for the `systemverilog` main
parser is **`focused_replay_target_debt_zero`**: after closed-loop
focused replay, `focused_replay_target_count > 0` (≈1482 residual
coverage targets, ~645 of them *never_selected*). Until that count
reaches 0 the family stays **Mostly Done** instead of **Done**.

A coverage *target* is a `(rule_name, node_path, branch_index)` OR-branch
that the generator should exercise but hasn't. `never_selected` means the
weighted choice at that OR-node never even picked the branch — not that
it picked it and the branch failed.

## 2. Root cause (from `.7.1`, confirmed by source read)

The OR-decision in `stimuli_generator.rs` (around line 3620) runs in two
phases:

1. **Candidate filtering** (`@~3623`): unless `mutation_active`, each
   alternative is kept only if `branch_within_depth_budget(alt,
   remaining_depth)` holds (depth-floor pruning). If everything is over
   budget, only the *shallowest* branch(es) survive (depth-floor
   fallback `@~3629`).
2. **Coverage-biased weighted choice** (`@~3648`): over the *surviving*
   candidates, `branch_selection_weight × coverage_guidance_multiplier`
   — the existing ×24 (success_hits==0) / ×2 (selected_hits==0) boost.

The boost in phase 2 **cannot rescue a branch that phase 1 already
dropped**. A `never_selected` target is therefore (almost always) a
*reachability* failure, not a weighting failure:

- the target branch is **deeper than the depth budget** at its OR-node,
  so it is pruned out of `candidate_indices` before weighting; and/or
- one of its **ancestor OR-nodes** never selects the alternative that
  descends toward the target rule, so the target rule is never entered;
  and/or
- a **recursion-pressure penalty** / parent-OR-never-entered condition
  fires upstream.

Adding *more* weight (the instinctive fix) is therefore the wrong lever
— it operates in phase 2, downstream of the phase-1 gate that is doing
the blocking.

## 3. What already exists (build on it — do not reinvent)

The engine already contains every primitive this design needs; they are
just not composed for coverage-driven reach:

| Existing primitive | Where | What it does |
| --- | --- | --- |
| `mutation_active` bypass | `@3620-3621` | when set, `candidate_indices = (0..prepared.len())` — i.e. **depth-floor pruning is skipped entirely**. Proof that a "consider all branches at this site" mode is already supported. |
| `forced_or_branch_for_site(site_key)` | `@~3699` (via `build_attempt_order @3662`) | during mutation replay, **forces a chosen branch to the front of `attempt_order`** at a specific OR site. Proof that per-site branch forcing already works. |
| `compute_reachable_rules(entry)` | `@2927` | BFS over the rule-reference graph (`collect_rule_references @2952`) producing the reachable rule *set*. The basis for a reach *path* (predecessor-tracked BFS). |
| `StimuliCoverageTarget` | `@505` | already carries `rule_name`, `node_path`, `branch_index`, `deficit`, and **`depends_on: Vec<TargetDependency>`** — "the prerequisite chain from the reachability analysis". |

So the mechanism is: **generalize the two mutation-replay primitives
(all-candidates bypass + per-site branch forcing) so a *coverage target*
can drive them, using the prerequisite chain the planner already
computes.**

## 4. The mechanism — `TargetReachPlan`

For one active `StimuliCoverageTarget` `T = (R, P, B)`:

### 4.1 Build the reach plan (static, from the grammar)
A `TargetReachPlan` is a sequence of **forced OR-site directives**
`[(rule_i, node_path_i, branch_index_i), …]` from the entry rule down to
`(R, P, B)`:

- Derive it from `T.depends_on` (the prerequisite chain) plus a
  predecessor-tracked variant of `compute_reachable_rules` that records,
  for each step, **which OR-branch at the parent references the next rule
  on the path**. (Extend `collect_rule_references` to also report the
  `(node_path, branch_index)` at which each reference occurs, so a parent
  → child edge knows the branch that realises it.)
- The final directive is the target itself: force branch `B` at `(R, P)`.
- This is pure graph computation over the already-parsed grammar tree —
  **no hardcoded rule names, no SV-specific logic.**

### 4.2 Activate reach mode during generation
While a `TargetReachPlan` is active, at each OR-decision:

1. **Suppress structural pruning *only on the path*.** If the current
   `(current_rule, node_path)` matches a directive in the active plan,
   take the `mutation_active`-style branch: `candidate_indices =
   (0..prepared.len())` — i.e. grant a *reach budget exception* so the
   deep target branch is not filtered out at phase 1 (`@3625`). Off-path
   OR-nodes keep normal depth-floor pruning, so the generator stays
   terminating everywhere else.
2. **Force the descending branch.** Feed the plan's directive into
   `build_attempt_order` as a *second forcing source* alongside
   `forced_or_branch_for_site` (same shape: prepend the forced branch to
   `attempt_order`). At the target's own site, force `B`.
3. **Honor a per-target depth/fuel budget.** The reach budget exception
   is bounded: a `TargetReachPlan` carries a max-extra-depth and an
   attempt cap. If the target's branch is gated by a semantic predicate
   that legitimately rejects (e.g. a `@predicate has_fact(...)` that the
   reach context can't satisfy), the plan **fails gracefully and is
   logged** — never an infinite descent. A genuinely unsatisfiable target
   is reported (so the residual count is honest), not silently retried.

### 4.3 Why this converges where weighting didn't
It moves the intervention from phase 2 (weighting, downstream) to phase 1
(candidate set, upstream) *for the targeted path only*, and it forces the
ancestor descent decisions instead of leaving them to chance. The target
branch becomes a guaranteed candidate **and** is reached, so its
`selected_counts` goes from 0 → ≥1 and it leaves the `never_selected`
set.

## 5. Parser-agnostic / NO-WORKAROUNDS positioning

- **GENERAL**: the plan is computed from the grammar's rule-reference
  graph + OR-branch structure + the existing target/`depends_on` data.
  It contains zero hardcoded rule names or SV sigils, per
  `[[feedback_ast_pipeline_parser_agnostic]]`. Every grammar pgen
  compiles (RGX, VHDL, RTL, SVPP) gets the same capability for free.
- **NO-WORKAROUNDS level**: this is a **level-5 parser-agnostic engine
  enhancement** (a generator-level capability), but it is the *minimal*
  such enhancement — it generalizes two primitives that already exist
  (`mutation_active` all-candidates + `forced_or_branch_for_site`) rather
  than inventing new machinery. Levels 1–4 are insufficient because the
  blocker is the engine's phase-1 candidate filter itself; no grammar
  annotation or store fact can make a depth-pruned branch a candidate.
  This must be documented in each `.7.2.1+` commit per
  `[[feedback_no_workarounds_fix_hierarchy]]`.
- It is the same class of change as the `.3.3.3` IIFE fix: parser-agnostic,
  generator-level, zero `unsafe`, benefits every parser — NOT an
  SV-specific kludge (`[[feedback_prefer_grammar_leave_engine_alone]]`).

## 6. Implementation decomposition (future code-leaves)

Each is its own task-tree leaf with empirical verification; opened only
after this design is accepted. **Do NOT bundle.**

- **`.7.2.1`** — predecessor-tracked reach-path computation: extend
  `collect_rule_references` to report `(node_path, branch_index)` per
  reference; add `compute_reach_path(entry, target)` returning the forced
  directive sequence. Unit-tested on a tiny synthetic grammar (no SV
  dependency).
- **`.7.2.2`** — `TargetReachPlan` struct + activation plumbing: thread an
  optional active plan through the OR-decision; implement
  `reach_plan_active_at(rule, node_path)` and the candidate-set bypass at
  `@3625`; extend `build_attempt_order` with the reach-forcing source.
- **`.7.2.3`** — per-target budget + graceful-failure + honest reporting
  (unsatisfiable targets counted, not retried).
- **`.7.2.4`** — wire the replay planner to emit `TargetReachPlan`s for
  `never_selected` targets and drive them; regenerate; re-measure
  `focused_replay_target_count`.

## 7. Verification plan (for the code-leaves)

- **Per-target micro-check**: a chosen `never_selected` target's
  `selected_counts` goes 0 → ≥1 after its plan runs.
- **Aggregate**: `focused_replay_target_count` strictly decreases across
  `.7.2.4`; the end goal is **0**.
- **No-regression (hard gates)**: SV external corpus **14/14**; lib
  `--features generated_parsers --lib` **609/609**; RGX **44/0**; SV
  shape-contract GREEN; `sv_syntax_closure_gate` no NEW unreachable
  branches; full `sv_parser_family_status_gate`
  (`bash rust/scripts/sv_parser_family_status_gate.sh`) still
  `family_status_overall: pass` end-to-end.
- **Determinism**: the same target → the same plan → the same forced
  decisions (no `Math.random`/time dependence), so replay is reproducible.

## 8. Open questions to resolve during `.7.2.1`

1. When multiple ancestor OR-nodes can reach the target rule (diamond in
   the rule graph), which path does `compute_reach_path` choose? Proposal:
   shortest path (fewest forced directives) to minimise the reach-budget
   exception surface; tie-break deterministically by branch index.
2. Mutual recursion on the path (e.g. expression grammars): the budget in
   §4.3 must cap *forced* recursion depth so a self-referential path
   terminates. Confirm against the known mutual-recursion cases
   (`module_path_conditional_expression`, the blessed unreachable set
   from `.5.1`).
3. Interaction with semantic predicates on forced branches: forcing a
   branch does **not** bypass its `@predicate` gate (correctly — that
   would generate invalid stimuli). Targets whose branch is predicate-
   gated may need the reach plan to *also* establish the required facts
   (e.g. emit a declaration first). Scope this in `.7.2.3`; it may reveal
   a second, distinct class of residual targets.
