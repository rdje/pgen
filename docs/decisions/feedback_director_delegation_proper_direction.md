# Director delegation: proceed in the proper direction without per-fork greenlight

- **Type:** feedback (standing directive — AMENDS `feedback_strategic_fork_pause_reflect`)
- **Date:** 2026-07-15 (session #121)
- **Source:** director, verbatim: *"you are the expert^2 coder, so you do not need my
  greenlight to move forward in the proper direction"* — given immediately after the
  RGX-0078 `.5.i.7` strategic-fork discussion, in the same session as the pushback
  *"We are throwing our hands in the air and walk away? :-)"* against stopping the
  speed campaign, and alongside the mdBook journey-chapter mandate.

## Context

The strategic-fork discipline (`feedback_strategic_fork_pause_reflect`, director
2026-07-12/13) had the road HARD-PAUSE at strategic forks: single recommendation +
reflection space, the director initiates the pick. At the `.5.i.7` fork (D2 scout vs
adjudicate-and-re-baseline) the director answered by delegating the pick class itself.

## Decision (how to apply)

- At a strategic fork: STILL surface it prominently with a single recommendation and
  record it durably — but the road **proceeds on the recommendation** instead of
  pausing, and the director can redirect at any time from the record.
- The delegation licenses **direction-setting**, not evidence-skipping: STEP-0 /
  pricing scouts still precede any build; the ⛔ HARD CONSTRAINT battery still governs
  any landing; correctness-floor and push-pacing directives are untouched.
- Real-world side effects stay director-owned as before (push, settings, external
  services, deletions, billed operations — per `feedback_user_is_director_not_engineer`).

## Amendment 2026-07-17 (session #144) — the HEAVY-BUILD GO is now engineer-owned too

- **Source:** director, verbatim: *"you're an elite coder, so you should be able to
  take the decision yourself … I want sota, signoff quality level decision and coding"*
  — given in response to the `.5.i.14` scout's "DECISION NEEDED — your call, director"
  request for the heavy-build GO.
- **What it changes:** the RGX-0078 campaign had carved the **heavy-build class**
  (the regen battery + the fat-LTO 5×2000 land gate) out of the delegation — every
  land-slice scout (`-0111`/`-0114`/`-0115`) recorded *"explicit DIRECTOR GO required"*
  for it, after the external kill-vector incidents. That carve-out is now **retired**:
  deciding to run the heavy-build battery + land gate for a scout-GO'd, priced,
  soundness-cleared lever is **engineer-owned**, no per-slice director GO.
- **What it does NOT change (unchanged real-world / resource constraints):**
  - the **HOST-RAM discipline** ([[feedback_host_ram_budget_all_jobs]]): one heavy job
    at a time, under `scripts/run_with_memory_guard.sh --budget-mb 16384` — a physical
    24 GB-machine constraint, not a permission gate;
  - **kill-vector watchfulness**: run guarded harness-side (the `-0115` precedent: 6
    guarded jobs, zero kills), with the DIRECTOR-RUN Terminal split as the fallback if
    the vector fires twice in a slice;
  - the ⛔ **correctness-before-speed HARD CONSTRAINT** battery still governs every
    landing (equivalence all-11 + cert 0/7/42 + PCRE2 oracle + shape/duality +
    land-iff-faster at the falsification bar);
  - **push / external / settings / deletions / billed** operations stay director-owned
    ([[feedback_user_is_director_not_engineer]] · [[feedback_push_pacing]]).

## Consequences

- RGX-0078 `.5.i.7` fork RESOLVED: the road proceeds with the D2 FULL-CASCADE-FOLDING
  STEP-0 scout (docs+instrument-only).
- New leaf `RGX-0078.8` (director mandate, same session): the speed-journey mdBook
  chapter.
- `feedback_strategic_fork_pause_reflect` remains in force for its OTHER half
  (discussion-mode file-freeze: no repo edits per exchange during a brainstorm; one
  consolidated update at explicit agreement).
- 2026-07-17: RGX-0078 `.5.i.14` GO taken engineer-owned (all-three components, priced
  −4.1% LOW / −6.4% MID, bar −2.0%); the heavy-build battery for it no longer waits on
  a per-slice director GO.
