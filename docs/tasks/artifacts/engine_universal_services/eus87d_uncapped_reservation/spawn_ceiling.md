# `ENGINE-UNIVERSAL-SERVICES.87`(d) — CAN A HOST-SIZED RESERVATION ACTUALLY BE MAPPED?

Removing `.69`(a)'s upper clamp makes the reservation unbounded above in a HOST-SUPPLIED
quantity. Prediction **P6** said the mapping cannot plausibly fail, because `host/64` is
1.56 % of physical memory as a *virtual* reservation with lazily committed pages. That is
an argument, so it was measured instead.

⛔⛔ **THIS FILE WAS REWRITTEN UNDER A DIRECTOR CHALLENGE AND THREE OF ITS FIRST NUMBERS WERE
WRONG.** The first cut sampled a 12-point grid, reported a grid point as a boundary, and
timed each size ONCE. What follows is the corrected record; the corrections are kept visible
because two of them change what the leaf may claim.

Producers, both tracked and both standalone (they depend on nothing in this repository):
[`spawn_ceiling_probe.rs`](spawn_ceiling_probe.rs) (the grid),
[`spawn_boundary_probe.rs`](spawn_boundary_probe.rs) (the bisection, the repeats and the
state-dependence arms). Recorded 2026-08-31, macOS/aarch64, 24 GiB, 12 cores.

## 1. The single-spawn ceiling is 64 TiB — bisected, not sampled

```
largest spawnable stack : 70368744177664 B = 64.000 TiB   (= 2^46 exactly)
smallest refused stack  : 70369817919488 B = 64.001 TiB
```

⛔ **CORRECTION.** The grid's first report was *"the first refusal is at 128 TiB — the
reservation an 8 PiB host would derive."* Both halves are wrong: 128 TiB was simply the next
sample after 64 TiB, and a grid cannot locate a boundary. The refusal begins at **64 TiB**,
which is `2^46` — an address-space limit, not a memory one — and the smallest host that would
DERIVE a refused reservation is therefore **4 PiB**, not 8 PiB.

## 2. At every realistic reservation the spawn is free, and now with repeats

`n = 9` per row, on a host under load (a release LTO build was running), so read the **min**:

```
     100663296 B  (96 MiB)   min 27 667 ns   median 30 083 ns   max 40 500 ns
     268435456 B  (256 MiB)  min 28 042 ns   median 30 083 ns   max 33 333 ns
     402653184 B  (384 MiB)  min 27 000 ns   median 28 709 ns   max 40 834 ns
```

⇒ **`.87`(d) adds no per-parse spawn cost**: 384 MiB is indistinguishable from the 256 MiB
reference at n=9, and both sit an order of magnitude under the *"~50–100 µs of thread setup is
noise against ms-scale HDL file parses"* the module's own header already prices.
⚠️ The first cut read 96 MiB at **96 µs** and 384 MiB at **36.9 µs** from one sample each and
called the difference warm-up. The repeats CONFIRM that reading — every size is ~28–30 µs —
but it was a guess when it was written.

## 3. ⛔⛔ THE SPAWN LIMIT IS NOT A FUNCTION OF THE REQUESTED SIZE ALONE

Measured, and it refuted the explanation offered for it before the explanation was published:

```
ARM A  fresh process, 16 TiB x8:              OK OK OK OK OK OK OK FAIL
       then 384 MiB x8:                       OK OK OK OK OK OK OK OK
ARM B  one 64 TiB spawn first, then 16 TiB x8: OK OK OK FAIL FAIL FAIL FAIL FAIL
       then 384 MiB x8:                       OK OK OK OK OK OK OK OK
```

The first story was *"16 TiB failed because the process had already mapped near 64 TiB."*
ARM A **refutes it**: 16 TiB fails on the 8th attempt in a FRESH process with no large prior
map. The mechanism is cumulative — `8 × 16 TiB = 128 TiB` and `64 + 3×16 = 112 TiB` both run
into the same `2^46` budget — so **address space is not returned promptly on join**, and
successive large reservations contend for one budget.

⇒ the honest bound is not `reservation < 64 TiB` but **`Σ live reservations < 64 TiB`**.

## 4. And the concurrent form of that bound, measured directly

```
           256 MiB (reference)  concurrent live:  128 (probe cap)  = 0.03 TiB VA
   384 MiB (this host, .87(d))  concurrent live:  128 (probe cap)  = 0.05 TiB VA
         16 GiB (a 1 TiB host)  concurrent live:  128 (probe cap)  = 2.00 TiB VA
```

No refusal at any realistic size — the probe cap was reached every time. At 384 MiB the VA
budget would admit ≈175 000 concurrent parses. ⇒ §3 is a statement about absurd reservations,
not an operating limit anyone will meet.

## 5. What this settles for the plausibility window (P8)

With no upper clamp, an ABSURD physical-memory reading becomes an absurd reservation, so the
bound moves to the READING. §1–§4 set where it goes, and it is **not** where the first cut put
it. The first cut chose `[512 MiB, 1 PiB)` because *"the largest admitted reading derives a
16 TiB reservation, measured OK"* — but §3 shows 16 TiB is exactly the size that fails once a
handful are live, so that justification is void.

The window is **`[512 MiB, 64 TiB)`**, derived from a property instead: the largest admitted
reading derives a **1 TiB** reservation, which leaves the `2^46` budget room for **64
concurrent** maximally-deep parses. 64 TiB is also above every single-image machine that
exists (the largest shipping today are ≈64 TiB), so no real host is refused its share.

⚠️ **STATED AS A POLICY, NOT A PROOF**, like `PARSE_STACK_HOST_MEMORY_SHARE_DIVISOR` and for
the same reason: it encodes a host-population judgement that will move. If single-image
machines grow past 64 TiB, this constant rots — and the thing that re-derives it is
[`spawn_boundary_probe.rs`](spawn_boundary_probe.rs), which is tracked so the re-derivation
costs one command.
⛔ It is a **TIGHTENING** of the `[512 MiB, 1 PiB)` bounds that
`the_host_memory_query_answers_on_every_target_it_claims_to_support` has asserted since
`.69`(a), not the pure hoist the amendment first called it.
