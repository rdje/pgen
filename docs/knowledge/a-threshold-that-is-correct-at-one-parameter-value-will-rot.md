---
id: a-threshold-that-is-correct-at-one-parameter-value-will-rot
title: A safety threshold pinned as a constant encodes a measurement — make the SAFETY MARGIN invariant in the parameter instead of correct at one value of it
answers:
  - "how do I stop a safety threshold or floor from going stale"
  - "two compiled-in constants have to stay consistent with each other — which one should stop being a constant"
  - "my guard has a minimum-resource floor computed from a per-unit cost measurement; is that sound"
  - "should I re-derive a stale literal from a fresh measurement"
  - "how do I make a resource guard correct for families or workloads I have never measured"
  - "what does a census of measured costs actually license me to claim"
  - "why did lowering a documented, supported knob crash the process"
  - "I made a threshold depend on a runtime parameter — where can that go wrong"
  - "my fix is invisible at the default setting, so what did my tests actually cover"
  - "my census reported not-found for one member of the population — can I publish that"
  - "how do I know a per-unit cost measured on one subject generalises to the others"
tags: [resource-limits, invariants, claim-verification, engine, safety, derived-bounds]
date: 2026-08-30
status: current
evidence: docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .79 (PGEN-ENGINE-UNIVERSAL-SERVICES-0142, its same-day correction -0144, the .79(a) debug census in -0145, and .79's founding measurement in -0138); docs/tasks/artifacts/engine_universal_services/eus79a_per_frame_census/ (the census producer and its recorded table); rust/src/dedicated_parse_stack.rs (first_stack_sample_depth, MIN_DEDICATED_PARSE_STACK_BYTES); rust/src/ast_pipeline/mod.rs (RecursionGuard::new); rust/docs/EMBEDDING_API_CONTRACT.md (Stack-Robustness Contract, 1.3.4)
reverify: "python3 -c \"n=300; open('tmp/p300.sv','w').write('module m; wire w; assign w = ' + '('*n + '1' + ')'*n + '; endmodule\\n')\" && PGEN_REPORT_STACK_PROBE=1 PGEN_PARSE_STACK_BYTES=100663296 ./rust/target/debug/parseability_probe --parse systemverilog --profile sv_2017 tmp/p300.sv   # rc=1 with cause=BudgetExhausted; the pre-.79 binary gave rc=134, 'fatal runtime error: stack overflow, aborting'"
---

Two constants had to stay consistent with each other, nothing held them together but a comment,
and the process died in the gap.

## The shape

A guard asks the host stack how much room is left, but only once the parse is already **4 096
frames** deep — sampling on every frame would cost the hot path. Reaching that first question costs
real stack. So the module also carried a **minimum reservation** (96 MiB), documented as the
smallest stack from which the question is still reachable, and computed as

```
4096 frames × ≈17 KB per frame ≈ 70 MB   (measured once, 2026-07, one family, debug)
```

Both constants entered in the same commit. `git log -S` on each returns exactly one commit and no
later one. Meanwhile the per-frame cost was re-measured at **25 335–29 031 B** — 4 096 frames now
cost **99–113 MiB** against the **90 MiB** the floor makes spendable. At the documented floor the
stack is gone *before the guard asks*, so a 642-byte source file killed the process with an
uncatchable `SIGABRT` rather than being rejected. The floor's whole purpose was to be the value that
is still safe, and it was documented as such, so an operator who trusted it got the abort.

## Why re-deriving the literal is the same defect

The obvious fix is to recompute 96 MiB from today's cost. Do not:

1. **A per-unit cost is a per-(family, profile, build-mode, shape) measurement.** Here two *profiles
   of one grammar family* differ by 14.6 %. A single number cannot be right for all of them.
2. **A census reports the worst cost it happened to RUN, never the worst that exists.** Ten families
   ship; two are measured. A literal derived from that census is a claim about the eight nobody ran.
3. **A constant has no way to notice it has aged.** The stale one passed every gate for two days and
   read as an argument, not as a measurement, because it was written in prose.

## The move: make the MARGIN invariant, not the value correct

Ask what actually has to hold: *the guard must sample before the resource runs out*. Both sides of
that are linear in the reservation —

- what the reservation can carry is `budget ÷ cost_per_frame`, and `budget` is a fixed share of it;
- so make the first sample **the requested depth scaled by the reservation's share of the default**.

Their ratio — the safety margin — is then **invariant in the reservation**, and the per-unit cost
*cancels out of the derivation entirely*. One measurement at the default now covers every smaller
value, for every family and both build modes, without measuring any of them. Measured: at the floor,
**7 of 9 ladder rungs went from `rc 134` to a clean `rc 1` refusal**; at an intermediate reservation
the refusal depth is **byte-identical before and after** (4 862 / 4 957 / 5 051 frames), because the
self-measuring loop converges to the same ceiling whatever depth it starts sampling at; and at the
default the scale factor is exactly 1, so the shipped path is unchanged by construction.

## Guard-rails that came out of it

- **Scale DOWN only.** Scaling the sample depth *up* on a large reservation would also raise the hard
  ceiling kept by callers whose stack extent the process cannot know — threads the reservation does
  not describe. Clamp at the requested value.
- **Say what the invariance does not buy.** It makes the margin the same everywhere; it does not
  measure it — invariance moves a many-valued risk to a single-valued one, which is the win, and is
  not the same as closing it. ⭐ Here the census that followed put a number on it: nine cells over seven subjects, a
  **≈15×** spread in per-unit cost, a tightest margin of **2.12×**, and exactly one subject under
  11×. ⛔ And note what the census had to CORRECT in this very record: the sentence above once read
  *"the margin is still one number from one family (2.37×)"*, which stopped being true the moment the
  census landed — **a claim scoped by "so far" needs re-reading the moment the "so far" moves**, and
  four published surfaces carried that stale sentence for a commit because only the tracker was
  updated. ⭐⭐ **The second half of the census then tested the invariance claim on the axis it was
  really about**: measured in the OTHER build mode, per-unit cost drops 7.83×–10.33× and the tightest
  margin moves 2.12× → 16.57×, so the binding mode is the expensive one — but the RATIO between the
  two modes is **not one constant** (three profiles of one subject agree to 1.042×; a second subject
  is 27 % away), so a value in one mode still cannot be derived from the value in the other. ⇒ **the
  invariance argument survives being measured, and the temptation it removes — carrying one number
  across a parameter — is refuted a second time on a second axis.**
- ⛔⛔ **APPLY THE LAW ONLY WHERE THE PARAMETER ACTUALLY DESCRIBES THE SUBJECT — this is the second
  half of the lesson and it cost a separate defect one commit later.** Once a threshold is a function
  of a parameter, every subject that threshold governs inherits the parameter whether or not it is
  described by it. Here the parameter was a stack reservation belonging to threads one module
  creates; the first cut read it as a process-global, so it also scaled the ceiling of a shipping
  parser family that runs on its **own** long-lived worker with an unrelated stack — measurably
  narrowing that family's accept set at any reduced setting (400 nested groups: accept at the
  default, refused at exactly the scaled depth below it). The fix is to read the subject's OWN value
  of the parameter and to leave subjects that have none exactly as they were.
- ⛔ **AND NOTICE WHY THE TESTS MISSED IT: at the DEFAULT setting the scaling factor is 1, so a
  parameterised change is INVISIBLE on the default path — which is where every gate runs.** Verifying
  three settings on one subject and one setting on every other subject leaves the whole
  (other subject × non-default setting) quadrant untested, and that is exactly where the defect was.
  ⇒ **vary the parameter across the whole population, not across one member of it.**
- ⭐⭐ **THEN MEASURE THE POPULATION, because "one subject's number does not generalise" is itself a
  claim.** Deriving instead of pinning is the right move whether or not the spread is large — but
  until the spread is measured it is an argument, not a fact. Measured here across nine measurement CELLS — **seven** distinct subjects,
  one of which contributes three because it has three dialects — in the expensive build mode: the per-unit cost spans **14.81×** (1 961 → 29 046 bytes per frame), so a
  constant derived from any one of them is wrong for another by more than an order of magnitude. The
  same census also answers the question the derivation could not: the tightest safety margin at the
  shipped default is **2.12×**, and exactly one subject sits under 11×. ⇒ **the census is not
  decoration on the fix; it is what tests the fix** — and in this case it is what FOUND a regression
  in it, on the first new subject it touched.
- ⛔⛔ **A CENSUS'S "NOT FOUND" IS A CLAIM, AND IT NEEDS THE SAME SCRUTINY AS ITS NUMBERS.** One row
  came back *ceiling not reached* after climbing to 262 144 levels — and the subject demonstrably
  refuses: its own rejection text named the frame count while the instrument reported nothing, because
  that subject runs on a worker whose reading never reaches the caller. Publishing the row as written
  would have asserted *"this member has no measurable limit"* about a member with a plainly visible
  one. The fix is a distinct verdict — *refused, but not observable by this route* — carrying the
  count the other surface does provide. ⇒ **when an instrument reports absence, check a second surface
  before you write it down**; and note the failure direction, which is silent and flattering.
- **A test that pins the other side of the relationship is the defect wearing a test.** Check the
  *property* (the ratio is the same at four reservations) and read the engine's own constant rather
  than re-typing it. Prove the check goes RED: restoring the pre-fix law must fail it, and here it
  failed exactly the two new locks and none of the nine existing ones.
