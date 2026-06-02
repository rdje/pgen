# SV-EXH-PROOF.7.4.1 — what the stimuli residual measures (criterion classification)

> Pure-docs INVESTIGATION (tools-first, no code). Owner leaf: `SV-EXH-PROOF.7.4.1`.
> Source-cited from `rust/src/ast_pipeline/stimuli_generator.rs` +
> `rust/scripts/sv_parser_aggregate_contract_gate.sh` (read 2026-06-02).
> Purpose: pin the coverage criterion so `.7.4.3`'s literal-0 witnessing is exact.
> Discipline: [[feedback_no_codebase_change_without_tool_backed_facts]],
> [[feedback_why_and_where_before_solution]].

---

## 0. TL;DR

The residual (`replay_target_count` = `(.targets // []) | length` of
`profile_<profile>_replay_gap.json`, read by the gate at
`sv_parser_aggregate_contract_gate.sh:144,275`) is a **finite set of coverage
obligations, already filtered to reachable-only, each needing exactly ONE success**
(`required_successes` defaults to 1). Each obligation is either:

- a **Rule** target — "production R was reachable but never generated" = **rule
  coverage (RC)**; or
- a **Branch** target — "ordered-choice alternative at `(rule, node_path,
  branch_index)` was reachable but never selected/succeeded" = **context-dependent
  branch coverage (CDBC-like)**.

**Therefore literal-0 is bounded-finite and is exactly Purdom/Hennessy-&-Power
set-cover completion** (`.7.3`): emit ONE minimal witness per residual obligation.
There is no depth-unboundedness to fear — the universe is `reachable_rules` +
`reachable_branches-with-deficit`, both finite, both already enumerated by the gap
report.

---

## 1. The target struct (Q1) — `stimuli_generator.rs:479-482, 516-546`

```rust
pub enum StimuliCoverageTargetType { Rule, Branch }

pub struct StimuliCoverageTarget {
    pub id: String,
    pub target_type: StimuliCoverageTargetType,
    pub rule_name: String,
    pub node_path: Option<String>,       // Some(..) for Branch, None for Rule
    pub branch_index: Option<usize>,     // Some(..) for Branch, None for Rule
    pub reachable: bool,
    pub required_successes: u64,
    pub current_successes: u64,
    pub deficit: u64,
    pub priority_score: u64,
    pub reason: String,
    pub depends_on: Vec<String>,
    pub reach_classification: Option<String>,
}
```

## 2. What a target represents (Q2) — Rule @ ~1801-1816, Branch @ ~1971-1986

- **Rule target** = a grammar production that was never generated → **rule coverage**.
- **Branch target** = a specific ordered-choice alternative inside a rule, addressed
  by `node_path` (the `o{i}/s{i}/q/l/a` path into the rule body) + `branch_index` →
  **context-dependent branch coverage** ("take alternative B at this OR-node in rule R").

The universe is built in `generate_gap_report` by iterating all rules and all branch
groups with `deficit > 0`.

## 3. Covered vs residual (Q3) — `evaluate_target_statuses` ~2246-2279

```rust
let required_successes = target.required_successes.max(1);
let remaining = required_successes.saturating_sub(current_successes);
if remaining == 0 { continue; }   // COVERED → not in residual
// else → stays in the residual
```

`required_successes_per_target = 1` (seen in the gap-report summary). So **each
obligation needs exactly one generated success** — the textbook "exercise it at least
once" coverage bar. One witness per obligation closes it.

## 4. Reasons taxonomy (Q4) — Rule ~1773-1780, Branch ~1894-1906

- Rule: `never_hit` (reachable, 0 successes) · `below_threshold` · `unreachable_from_entry`.
- Branch: `never_selected` (reachable, never chosen) · `selected_but_failed` ·
  `below_threshold` · `references_rule_missing_from_active_grammar` · `unreachable_from_entry`.

The dominant residual reasons are `never_selected` (branches) and `never_hit` (rules)
— i.e. obligations the diverse pass simply never reached, which is precisely what a
targeted minimal witness fixes.

## 5. Reachability filtering (Q5) — `.filter(|t| t.reachable)` @ 2085, 2251, 2288-2291

The residual is **already reachable-only**: targets are added to the `.targets` array
only when `reachable == true` (`~1802, ~1971`); unreachable ones go to separate
`unreachable_rule_debt` / `unreachable_branch_debt` lists and never enter the residual.
`reach_classification` further labels each: **`reachable_by_plan`** (a deterministic
`compute_reach_path` steering plan exists → directly witnessable by the existing
`forced_or_branch_for_site` machinery), `reachable_rule_not_generated` (rule needs a
witness that enters it), or `no_reach_path` (reachable in the graph but no current
steering plan → needs the Purdom min-length construction of `.7.4.2` to find a
concrete derivation).

## 6. Universe (Q6) — `CoverageDebtSummary` ~2055-2071

`total_rules` / `reachable_rules` / `unreachable_rules`; `total_branches` /
`reachable_branches` (dynamic remaining-deficit) / `unreachable_branches`. In the
shadow_state JSON observed on disk during this investigation: 1362 rules (1271
reachable), 1607 branches (486 reachable-with-deficit), **525 residual targets (39
Rule + 486 Branch)**.

## 7. The JSON (Q7) — gate `sv_parser_aggregate_contract_gate.sh:144,275`

```bash
closed_loop_replay_gap_json=".../profile_2017_replay_gap.json"
replay_target_count="$(extract_json_number "$f" '((.targets // []) | length)')"
```
`.targets[]` each = `{id, target_type, rule_name, node_path?, branch_index?,
reachable, current_successes, required_successes, reason, reach_classification}`.

---

## 8. HONEST number note (signoff)

The deterministic best-known residual logged across the `.7.2` campaign is **888**
(seed `seed_base(12001) + profile_idx*1_000_000 + 700_000`). The on-disk shadow_state
read during *this* investigation showed **525** (39 Rule + 486 Branch) — a *different*
prior run's artifact, not the deterministic-seed figure. `.7.4.1` does not re-run the
gate (criterion classification doesn't depend on the exact count); `.7.4.3` will
measure the deterministic count fresh, before/after, and act on that number per the
instant-revert rule. Reporting both honestly per
[[feedback_always_signoff_decisions]]; do not treat 525 as "the new baseline."

---

## 9. Conclusion → what `.7.4.3` must witness

- **Criterion = rule coverage (Rule targets) + context-dependent branch coverage
  (Branch targets), required_successes = 1, reachable-only.** Literal-0 = every
  reachable obligation gets ≥1 generating success. **Bounded-finite.**
- **Target set for witnessing** = the `.targets[]` array. Partition by
  `reach_classification`:
  - `reachable_by_plan` (most Branch targets) → witness via the EXISTING
    `compute_reach_path` + `forced_or_branch_for_site` (already proven by the 888
    runs); the new work is to emit these as a SEPARATE appended segment, not steer
    the shared pass.
  - `reachable_rule_not_generated` (Rule targets) + `no_reach_path` Branch targets →
    need `.7.4.2`'s Purdom shortest-derivation to construct a minimal entering
    witness.
- This confirms `.7.3`'s plan applies verbatim: **diverse background untouched +
  one minimal witness per obligation, appended.** Proceed to `.7.4.2` (Purdom
  min-length table, pure analysis).
