# SV-EXH-PROOF.7.4.3a — WHY the stimuli residual fails to be covered (full step-by-step root cause)

> Pure-docs INVESTIGATION (tools-first, head-on debug; no code change). Owner leaf:
> `SV-EXH-PROOF.7.4.3a`. Every claim below is backed by a source citation OR a
> toolbox measurement run this session. Director directive (2026-06-02): *"fully
> explain, step-by-step, what is going on, hence why the fail — without a clear
> explanation there is no way to come up with a fix … use the toolbox … debug head
> on, do not give up."* Discipline: [[feedback_why_and_where_before_solution]],
> [[feedback_no_codebase_change_without_tool_backed_facts]].

---

## TL;DR — one sentence

The residual targets are **not ungeneratable and not a counting bug**; they fail
because **the generation depth budget (`max_depth=24`, counting rule-expansion
recursion) is exhausted descending the deeply-factored SV grammar from the top entry,
so deeply-nested rules either hit the hard depth limit and error out
(`selected_but_failed` / `never_hit`) or are steered away by the near-limit
terminating-branch guard (`never_selected`)** — and steering (`.7.2.x`) could not fix
it because forcing the path to a deep target still leaves its subtree to generate
within the same exhausted budget.

---

## The facts (clues gathered from the toolbox this session)

### F1 — The residual taxonomy (from the actual `profile_2017_replay_gap.json`)
Two on-disk artifacts (the gate runs two modes). Mechanism is identical in both; the
counts differ by run:

| | shadow_state (525) | generation_state (1883, stale `.7.2.20` run) |
|---|---|---|
| reachable_rules / covered | 1271 / 1232 | 1271 / 457 |
| residual rules (`never_hit`) | 39 | 814 |
| residual branches `never_selected` | 291 | 826 |
| residual branches `selected_but_failed` | 195 | 243 |
| reach_classification | all `reachable_by_plan` (branches) / `reachable_rule_not_generated` (rules) | same |

So every residual target is classified **reachable** — the generator *could* reach it;
it just doesn't. Three failure reasons: `never_hit` (rule, 0 successes),
`never_selected` (branch never chosen), `selected_but_failed` (branch chosen but its
subtree never completed a successful sample).

### F2 — `covered_reachable_branches: 0` is a DEAD-COUNTER ARTIFACT, not a bug
`stimuli_generator.rs:1849` does `if deficit == 0 { continue; }` (skips any branch with
`success_hits >= 1`), so the increment at `:1886` guarded by `success_hits > 0` is
**unreachable**. The metric is always 0 by construction. Real branch coverage is
counted by `covered_branches` (`:1847`, before the `continue`). → This clue is cleared;
branch coverage *is* recorded. (Worth a separate metric-cleanup leaf, not the cause.)

### F3 — The rules/branches generate FINE standalone (so they are not ungeneratable)
Direct generation via `ast_pipeline --generate-stimuli --entry-rule R` on the compiled
SV grammar (`systemverilog_gen_ast.json`), seed 712001, `max_depth=24`, `max_repeat=4`:
- `always_keyword` → **all 4 branches** (`always` / `always_comb` / `always_ff` /
  `always_latch`) appear across 20 samples (the closed loop marks `#0`/`#2`
  `never_selected`).
- `ansi_port_declaration` → **30/30 successful** samples.

### F4 — Depth sensitivity is THE mechanism (decisive measurement)
Same standalone generation, varying only `--max-depth`:

| rule | d=2 | d=4 | d=8 | **d=9** | **d=10** | d=11 | d=12 | d=24 |
|---|---|---|---|---|---|---|---|---|
| `ansi_port_declaration` | FAIL | FAIL | FAIL | **FAIL** | **20/20** | 20/20 | 20/20 | 30/30 |
| `property_expr_sv_2017` | FAIL | FAIL | FAIL | — | — | — | — | so slow it didn't finish in 100 s |

`ansi_port_declaration`'s **minimum generation-recursion depth is exactly 10** — it
cannot produce a single successful sample with fewer than 10 levels, even standalone
(full fresh budget). `property_expr_sv_2017` (the #1 residual cluster, 24 branches) is
deeper still and pathologically expensive at high depth.

### F5 — WHERE the depth limit bites (source)
- `stimuli_generator.rs:4377` — hard limit: `if depth > self.config.max_depth { … "Stimuli
  generation depth exceeded max_depth={} while expanding rule '{}'" }` → the generation
  attempt **errors out** (no sample) when recursion passes 24.
- `:4509` / `:5413` — near the limit (`depth >= max_depth - 1`) the OR-decision is forced
  toward a **terminating (shortest) branch** (the "recursion guard", unit-tested at
  `:9847 recursion_guard_prefers_terminating_branch_at_depth_limit`).
- `:4869` — an existing **depth-slack retry** already grants a *targeted* OR branch
  temporary extra depth — evidence the engine already knows depth is the constraint, but
  the slack is insufficient (888 residual persists).

---

## The step-by-step causal chain (why each reason happens)

Setup: the closed loop generates from entry `systemverilog_file` with `max_depth=24`,
where depth = rule-expansion recursion depth (incremented per rule layer, enforced at
`:4377`).

1. **SV's grammar is deeply factored.** Reaching a leaf-ish rule like
   `ansi_port_declaration` from `systemverilog_file` traverses many intermediate rule
   layers (`source_text_item → … → module_item → … → ansi_port_declaration → net_port_header
   → data_type → … → identifier`), each consuming one depth level.
2. **The target's own subtree also needs many levels** — F4: `ansi_port_declaration`
   needs ≥ 10 levels for its *shortest* successful derivation; `property_expr` needs far
   more.
3. **Budget exhaustion → `selected_but_failed` / `never_hit`.** From the top entry, by
   the time the generator descends to the target, much of the 24 budget is spent. Let the
   descent cost `D` levels and the target need `M` levels; whenever `D + M > 24` the
   subtree generation trips `:4377` "depth exceeded" → the **whole sample fails** → the
   branch's `success_hits` stays 0 (it was selected → `selected_but_failed`) and the rule
   never completes (`never_hit`). F4 gives `M = 10` for `ansi_port_declaration`, so any
   descent costing more than 14 levels dooms it — and SV's factoring easily exceeds that.
4. **Near-limit termination guard → `never_selected`.** For branches that sit even deeper,
   once `depth >= 23` the OR-decision is *forced* toward the shortest terminating
   alternative (`:4509`/`:5413`), so the deep alternative is **never chosen** → its
   `selected_hits` stays 0 → `never_selected`.
5. **Recursion-pressure / depth-floor steering compounds it.** Per
   [[project_sv7_never_selected_rootcause]], the unforced weighted choice is additionally
   penalized for deep recursion, so even before the hard limit the generator
   statistically avoids the deep paths.

Net: the residual ≈ the **deep-factored region of the SV grammar that cannot be
reached *and* completed within a 24-level budget measured from the top entry.**

---

## Why `.7.2.x` steering could not fix it (and collapsed diversity)

A reach plan forces the OR-decisions *along the path to* a deep target, but **the subtree
*below* the target is still generated by the normal depth-bounded recursion** — which is
exactly what trips `:4377` (F3/F4). So forcing a deep target yields `selected_but_failed`
(branch selected, subtree still exceeds depth): **no coverage gain**. Worse, those forced
attempts consume the shared 5000-attempt target-drive budget on doomed generations,
leaving fewer ordinary diverse samples → net resolved coverage *fell* (the measured
888 → 1717 / 1883 / 1982 regressions). The lever was never "steer harder"; it was always
the **depth budget**.

---

## What this means for the fix (`.7.4.3`, refined)

The explanation tells us precisely what a witness must do: **give each residual target a
derivation that fits the depth budget.** Concretely (all monotone-additive, no change to
the diverse pass):

1. **Generate each residual rule/branch as a SEPARATE witness with a FRESH / adequate
   depth budget** — e.g. rooted at (or forced via a *short* path to) the target so the
   target's subtree gets the full ~24 levels (F3: standalone it succeeds), instead of the
   leftovers after a top-entry descent. A standalone-entry (or short-path) witness's
   coverage counters still increment globally → the target is covered.
2. **Pick the SHORTEST subtree below the target** using the Purdom min-length table
   (`.7.4.2`) so even deep rules complete within budget and `property_expr`-style blowup
   is avoided.
3. **Generalize the existing depth-slack retry (`:4869`)** if a witness still needs more
   headroom than 24.

This is exactly the decoupled Purdom/Hennessy-&-Power completion of `.7.3`, now with the
**depth-budget root cause** that explains *why* it will work where steering failed: a
witness resets/extends the depth budget for the target — something a from-the-top steered
attempt structurally cannot do.

### Honest open question for `.7.4.3`
Some `selected_but_failed` targets may need their subtree minimized *and* extra depth; a
few deep recursive rules (`property_expr`) may be expensive enough that a per-target
witness needs a generation timeout. `.7.4.3` will MEASURE the residual after the witness
pass and report the number; any irreducible tail gets its own analysis rather than a
guess.
