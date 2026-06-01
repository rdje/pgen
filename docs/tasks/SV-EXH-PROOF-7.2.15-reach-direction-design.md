# SV-EXH-PROOF.7.2.15 — Next reach-steering direction (DESIGN, no code)

Status: design-first per [[feedback_no_codebase_change_without_tool_backed_facts]].
NO code authored in this leaf. Every number below is read from an artifact or the
source/grammar (citations inline). Decision requires director sign-off before any
`.7.2.16+` code.

## 1. Where we are (facts)

- Best-known state = **`.7.2.8` baseline, replay_target_count = 888** (HEAD
  `70a8270e`, generator blob `d06a560b`). This is the concentrated `.7.2.7`
  steering with NO quantifier-forcing and NO rotation.
- `.7.2.10` (quantifier-forcing) + `.7.2.11` (rotation) were a measured
  REGRESSION and were reverted (`.7.2.13`). Ablation (`.7.2.14`) attributed it:

  | run | resolved | replay_target_count | gen_successes | target_timeout | never_hit rule |
  |---|---|---|---|---|---|
  | A `.7.2.8` baseline | 1772 | **888** | 2221 | 784 | **236** |
  | B `.7.2.10`-only | 679 | 1982 | 4615 | 344 | (n/a-not-tallied) |
  | C `.7.2.12` both | 943 | 1717 | 2359 | 1837 | **750** |

- **Mechanism of the regression (fact):** forced quantifier expansion → many
  fast, valid, near-IDENTICAL samples (gen_successes doubled, timeouts dropped)
  but coverage COLLAPSED. **Diversity collapse**, not slow/deep timeouts.

## 2. The key fact that reframes the whole campaign

The `.7.2.8` residual (888) breaks down as (CHANGES `-0122`):
**417 never_selected branch + 236 never_hit rule + 235 selected_but_failed branch.**

But under the failed reach-steering the **never_hit rule count EXPLODED 236 → 750**
(`.7.2.12` artifact). i.e. **aggressive reach-steering CREATES never_hit rules** —
by forcing narrow paths that skip whole sibling rules. So the instinct "add more
steering to cover the never_hit rules" is the SAME failure mode I just reverted.
The data argues AGAINST more forcing.

## 3. Why reachable rules are never_hit (structural fact, from the grammar)

`never_hit rule` is DEFINED (`stimuli_generator.rs:1757-1759`) as: graph-reachable
from entry AND `rule_success_hits == 0`. All 236 baseline never_hit rules are
`no_deps` (not producer-gated) — they're simply never structurally generated.

Representative: `always_construct` (grammars/systemverilog.ebnf:485) is referenced
ONLY as one OR-alternative among dozens (lines 888, 3077, 3092) inside
`*`-quantified container bodies (`module_or_generate_item`, etc.). So baseline
weighted choice rarely selects that exact alternative, even though
`coverage_guidance_multiplier` already boosts uncovered branches ×24/×2 — because
(per `.7.1`) the boost operates AFTER structural depth/pruning, and these
alternatives sit deep in large OR groups under quantifiers.

This is the SAME territory `.7.2.10` tried to brute-force. The lesson from the
ablation: forcing the path destroys diversity. The lever must add coverage of the
skipped rule WITHOUT collapsing the rest of the sample.

## 4. Options (each with its risk, grounded in the ablation)

### Option A — STOP. Bank 888; close the reach campaign as "Mostly Done w/ debt."
- 888 is a verified, real **−30% vs the 1273 dead-hook baseline**, gate passes.
- Pro: no further regression risk; the hard part (a working, measured reach
  driver) is done and pushed.
- Con: per director's **literal-0 bar**, 888 ≠ 0 → SV stays Mostly Done. Leaves
  documented open debt.

### Option B — Build the honest residual REPORT (a tool, not a forcing change).
- Add a per-target `reach_target_outcome` tally (Reached / SelectedButFailed /
  NotReached) to the gate summary (extends the `.7.2.5` observability; the enum
  already exists from `.7.2.3`). NO generation-behavior change.
- Pro: turns "888 residual" into "N genuinely-unsatisfiable + M not-yet-reached",
  which is the FACT base any future literal-0 work needs, and is itself valuable
  signoff evidence. Zero regression risk (read-only).
- Con: doesn't itself reduce 888; it's the measurement that should PRECEDE any
  further code (and arguably should have preceded `.7.2.10`).

### Option C — Rule-targeted reach WITHOUT quantifier-forcing.
- Generalize `set_reach_plan` to accept a Rule target (steer the OR chain so the
  rule is ENTERED), but DO NOT force on-path quantifiers (the proven-harmful part)
  and DO NOT rotate aggressively — keep the concentrated `.7.2.7` discipline.
- Pro: directly attacks the 236 never_hit rules, the bigger lever.
- Con: HIGHER risk — it's new steering, and the ablation shows steering that
  changes structural choices can collapse diversity. Must be ONE change, measured
  against 888 before anything else, and reverted instantly if it regresses.

## 5. Recommendation

**B then (maybe) C.** Build the read-only outcome report FIRST (Option B): it is
zero-risk, it is the fact base I lacked before `.7.2.10` (which is why I guessed),
and it will tell us how much of the 888 is genuinely unsatisfiable vs reachable —
which determines whether literal-0 is even attainable by steering at all. Only if B
shows a meaningful reachable-but-not-reached population do we attempt C, as ONE
measured change against 888.

Explicitly NOT recommended: re-introducing quantifier-forcing (Option in the
reverted `.7.2.10`) — the ablation proved it collapses diversity.

## 6. Decision needed from director

1. Direction: **A (stop at 888)**, **B (build the report, then reassess)**, or **C
   (attempt rule-reach now)**?
2. If B or C: confirm the rule that each code change must be measured against the
   888 baseline and reverted on any regression (no exceptions).
