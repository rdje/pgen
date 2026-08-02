---
id: depth-slack-retry-needs-a-runaway-backstop
title: A generation retry with no runaway backstop — why `--max-depth 20` runs at depth 448, and how to price the bound
answers:
  - "why does a failure reason say max_depth=448 when I passed --max-depth 20"
  - "why is the stimuli generator's depth budget cumulative under nesting"
  - "does the generator's depth escalation ever actually produce coverage"
  - "what is the Depth-slack retry census line in a closed-loop replay log"
  - "how do I choose a cap for a generation retry without curve-fitting"
  - "how do I tell a runaway retry from a productive one"
  - "why did capping a retry make the target-drive pass better and the residual worse"
  - "what does branch_retry_max much greater than success_ordinal_max mean"
  - "what is TARGET_BRANCH_DEPTH_RETRY_CAP and what does it fix"
  - "why can't I just compute the retry slack from the configured max_depth"
  - "is a nesting cap different from a budget ceiling for the depth-slack retry"
tags: [stimuli, coverage, generator, retry, budget, instrumentation, systemverilog, root-cause]
date: 2026-08-02
status: current
evidence: rust/src/ast_pipeline/stimuli_generator.rs (`target_branch_depth_retry_slack`, `should_reach_retry_uncovered_recursive`, `MAX_UNCOVERED_REACH_RETRIES`, `DepthSlackRetryCensus`, `depth_slack_retries_by_branch`); docs/tasks/SV-EXH-PROOF.md leaf .7.4.6.13 (FIX_PRICED_AND_BLOCKED_2026-08-02, BACKSTOP_LANDED_2026-08-02); TOOLBOX.md 6.2; docs/book/src/stimuli-and-quality.md "`--max-depth` is not the depth the generator runs at"
reverify: "cargo test --features 'generated_parsers ebnf_dual_run' --lib depth_slack; grep 'Depth-slack retry census:' rust/target/sv_stimuli_quality_gate/logs/profile_2017_closed_loop_replay.log"
---

**Two retries in one `match` arm. Only one of them is bounded — and that is the defect.**

`generate_or`'s `Err` arm offers a failed branch two second chances:

- `should_reach_retry_uncovered_recursive` (constructive reach) — bounded by
  `MAX_UNCOVERED_REACH_RETRIES = 4096` since `GRAMMAR-WELLFORMED.H.4.2`, with a doc comment
  explaining why a retry needs a runaway bound.
- `target_branch_depth_retry_slack` (depth slack) — bounded by **nothing**.

The unbounded one grants `+4` depth to a targeted, never-covered, depth-blocked branch, and it
computes that `+4` from the **live** `config.max_depth` rather than the configured one. So a retry
entered inside another retry's subtree inflates an already inflated budget, and the ladder becomes
`20, 24, 28, …`. On SystemVerilog it climbs to `448` (`sv_2017`) and `676` (`sv_2023`). `ebnf` and
`semantic_annotation` climb the same ladder three rungs deep, so this is shared generator behaviour.

⛔ **These are TWO defects sharing one mechanism, and they do not share a fix.** Keep them apart:

| defect | what it is | what fixes it |
|---|---|---|
| **cost / runaway** | the retry was unbounded per branch — one `sv_2017` branch spent **726 836** retries, 37 % of the whole run's | ✅ FIXED — the per-branch backstop `TARGET_BRANCH_DEPTH_RETRY_CAP = 4096` (below) |
| **predictability** | `--max-depth` does not bound the descent, because the slack is added to the **live** budget | ⛔ STILL OPEN — and the obvious bounds are REFUTED (below). Needs an EXPLICIT per-target depth grant, not a cap |

⛔ **A per-branch backstop does NOT bound the ladder.** Measured under the cap, the ladder is
essentially invariant: `448 → 444` (`sv_2017`) and `676 → 672` (`sv_2023`). Recorded depth failures
do not even move in one direction — `sv_2017` falls `5 460 052 → 4 050 966` while `sv_2023` *rises*
`7 346 200 → 8 352 431`, because the freed budget buys more exploration (the same reason its
successes go `147 → 398`). Fixing the runaway and making `--max-depth` mean what it says are
separate changes with separate evidence.

## Reading the census

Every closed-loop replay run prints one line (only when the retry fired):

```
Depth-slack retry census: nesting_levels=107 attempts=1964056 successes=255
  deepest_paying_level=89 branches_retried=443 branch_retry_max=726836
  success_ordinal_max=103829 [successes/attempts@max_budget] L1:35/855@24 …
  [success_ordinal:count] 1:144 2:24 3:5 …
```

| read | signal |
|---|---|
| `deepest_paying_level` vs `nesting_levels` | where successes STOP vs how far the ladder climbed. `sv_2023`: `24` against `164` — **99.5 % of retry work returned zero successes** |
| `branch_retry_max` vs `success_ordinal_max` | the runaway signature: one branch far past any paying ordinal |
| the `success_ordinal` histogram | exactly what a per-branch cap costs — read off, not guessed |

## Price the cap, do not curve-fit it

The per-branch success ORDINAL histogram converts "pick a cap" into a table lookup. Measured on
SystemVerilog: `64` keeps `214/255` + `147/147`; `1024` keeps `245/255`; `4096` keeps `252/255` +
`147/147`. Choosing `4096` also matches the sibling backstop's magnitude, so a reader comparing the
two retries is not left wondering why their bounds disagree.

⛔ **A cap prices the run you measured, and capping changes the run.** Under the cap the same
SystemVerilog `sv_2023` replay found **more** successes (`147 → 398`) and nearly tripled its
target-drive resolution (`568 → 1673`), because the budget stopped being burned on one branch.
Always A/B the residual; never infer it from the histogram.

**LANDED, and here is the whole price.** `TARGET_BRANCH_DEPTH_RETRY_CAP = 4096`, consulted against
the census's own per-branch tally so the bound and its pricing cannot describe different
populations. Measured before → after on the closed-loop replay stage, both LRM profiles:

| | `sv_2017` | `sv_2023` |
|---|---|---|
| retry attempts | 1 964 056 → **302 526** (6.5x) | 2 959 658 → **408 247** (7.2x) |
| retry successes | 255 → 252 | 147 → **398** |
| `branch_retry_max` | 726 836 → **4 096** | 280 916 → **4 096** |
| target-drive resolved | 872 → 880 | 568 → **1 673** |
| stage elapsed | 458 s → **267 s** | 569 s → **437 s** |
| residual | 0 → 0 | 0 → 0 |

## The trap: making one pass better can uncover another pass's hidden dependency

Bounding the retry improved `sv_2017`'s target-drive pass (`872 → 880` resolved) — and that
improvement **raised** the residual `0 → 1`. The chain: the extra resolutions included
`wildcard_escape_nettype_identifier`, so that rule was no longer a residual RULE target when the
witness pass ran; the raised-witness-entry mechanism fires for RULE targets only
(`store_entry_raises 1 → 0`); and the store-entry-blocked BRANCH target it had been closing
*transitively* went uncovered. See [[witness-entry-policy-store-gated-targets]] — whose BRANCH half
was built for exactly this, and is what let the bound land with the residual still at `0`.

⇒ **when coverage is reached transitively, an improvement anywhere upstream can remove the
transitivity.** A residual of 0 that depends on one pass failing at exactly the right place is a
coupling worth writing down before something legitimate disturbs it.

## ⛔ The escalation is LOAD-BEARING — do not simply bound it

Computing the slack from the **configured** depth instead of the live one is the obvious fix, and it
works exactly as designed: nesting levels `106 → 8` and `163 → 8`, the ladder collapses to a flat
`configured + 4`. It is also unshippable. Measured on both LRM profiles:

| | `sv_2017` | `sv_2023` |
|---|---|---|
| retry successes | 252 → 11 | 398 → 19 |
| target-drive resolved | 880 → 205 | 1673 → 271 |
| `covered_rules` | 1337 → 1327 | 1356 → 1352 |
| `covered_branches` | 1451 → 1444 | 1490 → 1484 |
| closed-loop residual | 0 → **17** | 0 → **10** |

⇒ roughly **10 rules and 7 branches per profile are covered only because a retry nested inside
another retry inherits the inflated budget**. The escalation is the de-facto mechanism by which deep
targets get depth at all; remove it without replacing it and the coverage goes with it.

⭐ **And the two "alternative" candidates are one knob.** A nesting cap `N` and a budget ceiling `C`
are identical, because the ladder rung IS the nesting level: `C = configured + 4N`. Priced off the
census, any cap that bounds something worth bounding costs 40–87 % of retry successes; the cheapest
free cap is `N = 99` — ceiling 416, still 21x the configured depth.

⇒ the shape that can work is an **explicit** per-target depth grant (what the closed-loop witness
pass already computes: reach-prefix plus the target's own minimal derivation depth), so depth is
granted by a computed budget rather than inherited from whichever retry encloses you.

## Instrument discipline

The census is read-only and carries both controls: a positive one (the existing depth-slack unit
scenario must census exactly one level-1, ordinal-1 success) and a negative one (a grammar with no
depth-blocked targeted branch must census nothing and print nothing). A census line in a log is
therefore evidence the retry **ran**, not evidence the instrument is compiled in. Whole-run control:
with the instrument compiled in, the replay stage's `stimuli`/`gap.json`/`gap.txt` stay byte-identical
to the canonical gate's own artifacts.
