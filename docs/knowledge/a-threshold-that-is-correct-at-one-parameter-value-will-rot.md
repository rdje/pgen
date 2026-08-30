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
tags: [resource-limits, invariants, claim-verification, engine, safety, derived-bounds]
date: 2026-08-30
status: current
evidence: docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .79 (PGEN-ENGINE-UNIVERSAL-SERVICES-0142, and .79's founding measurement in -0138); rust/src/dedicated_parse_stack.rs (first_stack_sample_depth, MIN_DEDICATED_PARSE_STACK_BYTES); rust/src/ast_pipeline/mod.rs (RecursionGuard::new); rust/docs/EMBEDDING_API_CONTRACT.md (Stack-Robustness Contract, 1.3.4)
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
  measure it. The margin is still one number from one family (here 2.37×), and a workload costing
  more than 60 KiB per frame (61 440 B, exactly `budget ÷ 4 096`) would be unsafe at *every* reservation equally. Invariance moves a
  many-valued risk to a single-valued one — that is the win, and it is not the same as closing it.
- **A test that pins the other side of the relationship is the defect wearing a test.** Check the
  *property* (the ratio is the same at four reservations) and read the engine's own constant rather
  than re-typing it. Prove the check goes RED: restoring the pre-fix law must fail it, and here it
  failed exactly the two new locks and none of the nine existing ones.
