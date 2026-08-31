# `ENGINE-UNIVERSAL-SERVICES.87`(d) — PRE-REGISTRATION

Written BEFORE the clamp was removed and before any measurement was taken, so the
predictions below can refute the design rather than describe it.

Host: `sysctl hw.memsize` = **25 769 803 776 B** (24 GiB), macOS/aarch64, 12 cores.
Binary: release `parseability_probe`, `--profile sv_2017`, `PGEN_REPORT_STACK_PROBE=1`.
Shape: nested `int'(…)` casts — `.69`(d)'s shape, so the two leaves share an input.

## What the change is

Delete the upper clamp from `derive_parse_stack_reservation`:

```text
before:  reservation = min( REFERENCE , max( FLOOR , host_memory / 64 ) )
after:   reservation =                  max( FLOOR , host_memory / 64 )
```

`DEDICATED_PARSE_STACK_BYTES` keeps its other two roles unchanged — the fixed SCALING
ANCHOR of `scale_sample_depth_to_reservation`, and the fallback when host memory cannot
be read. Only the clamp goes.

## Predictions

| # | prediction | refuted if |
|---|---|---|
| **P1** | At HEAD the resolved reservation on this host is **268 435 456** (256 MiB), provenance `host_memory_derived` — the clamp binding, not a literal. | any other number or provenance |
| **P2** | After the lift, `derive_parse_stack_reservation(25 769 803 776)` = **402 653 184** (384 MiB) = exactly `host_memory / 64`; provenance still `host_memory_derived`. | any other number |
| **P3** | **THE CAPABILITY.** `cast_4814.sv` is **REJECTED** at HEAD's default and **ACCEPTED** after, with **no environment variable set**. | it still rejects |
| **P4** | **PROPORTIONALITY, UPWARD — never measured above the reference before.** `.69`(d) recorded `cast_4814` refusing at frame depth **81 624** on 256 MiB. At 384 MiB the refusal depth should be **≈122 436** (×1.500). Measured on a rung deep enough to refuse at 384 MiB. | the depth ratio departs from 1.500 by more than **2 %** |
| **P5** | The accept boundary moves from **4 813** to **≈7 220** source levels (18.82 levels/MiB × 384). So `cast_7000` accepts and `cast_7600` refuses. | either rung lands on the wrong side |
| **P6** | **RISK — the spawn.** A 384 MiB thread stack maps without failure. More generally a reservation of `host_memory / 64` cannot plausibly be refused: it is 1.56 % of physical memory as a *virtual* reservation with lazily committed pages. | any `DedicatedParseStackError::Spawn` |
| **P7** | **COST.** Accepting `cast_4814` at 384 MiB costs **≈32 s** — `22.24 s` at depth 4 000 (the SV contract's own figure) scaled by `(4814/4000)² = 1.45` — i.e. at or below the **44.68 s** the *refusal* cost at 256 MiB. The direction is genuinely uncertain: the accept does less work than a budget-exhausting refusal, but over a deeper input. | outside 20–70 s |

## Why P4 is the one that matters

The SV contract publishes *"vary the reservation and the boundary moves with it, linearly —
18.82 levels per MiB, flat to 0.14 %"*. Every point behind that number was taken at **or
below** the 256 MiB reference (96 / 128 / 192 / 256). `.87`(d) is the first change that puts
a shipping host **above** it, and nothing has ever measured the law there. If P4 fails, the
contract's central claim is one-sided and (d) has to say so before it ships.

## AMENDMENT 1 (same session, before the code change, after P6 was measured)

P6 was measured first because it is the one prediction that could have stopped the design
(see [`spawn_ceiling.md`](spawn_ceiling.md)). It passed — and in passing it exposed a
consequence the original list MISSED, so the amendment is recorded rather than folded
silently into the predictions above:

> **P8.** With no upper clamp the reservation is unbounded above in a **host-supplied**
> quantity, so an ABSURD physical-memory reading becomes an absurd reservation. The clamp was
> silently providing that protection. The bound therefore moves to the READING: a reading
> outside a plausibility window is UNKNOWN and takes `ReferenceFallback`, which is the contract
> `query_host_physical_memory` already states for a zero or short read (*"Both are 'unknown',
> never a number"*).

## AMENDMENT 2 (under a director challenge, still before the code change)

⛔ **AMENDMENT 1's window and its justification are BOTH WITHDRAWN**, and so is the sentence
that called the move a hoist. The details are in [`spawn_ceiling.md`](spawn_ceiling.md) §1, §3
and §5; the corrections that bind this pre-registration:

- The single-spawn ceiling is **64 TiB** (`2^46`), bisected — not the 128 TiB the first grid
  reported. So the smallest host that could derive a REFUSED reservation is **4 PiB**, not 8.
- **The spawn limit is not a function of the requested size alone.** Address space is not
  returned promptly on join, so `Σ live reservations < 64 TiB`. Measured: 16 TiB succeeds
  seven times in a fresh process and fails on the eighth. ⇒ *"the largest admitted reading
  derives 16 TiB, measured OK"* was never a valid reason for a 1 PiB bound.
- ⚠️ And the first EXPLANATION offered for that failure — *"the process had already mapped near
  64 TiB"* — was refuted by a fresh-process arm before it was published
  ([[a mutation is only a control where the defect it names can exist]], `-0158`'s rule applied
  in time for once).

> **P8′ (replaces P8).** The plausibility window is **`[512 MiB, 64 TiB)`**, derived from a
> property rather than from a single spawn: the largest admitted reading derives a **1 TiB**
> reservation, leaving the `2^46` budget room for **64 concurrent** maximally-deep parses
> (measured: 128 concurrent live threads at 16 GiB each, 2 TiB of VA, no refusal). Refuted if
> the largest admitted reading cannot sustain 64 concurrent spawns.
> ⛔ This is a **TIGHTENING** of the `[512 MiB, 1 PiB)` literals
> `the_host_memory_query_answers_on_every_target_it_claims_to_support` has asserted since
> `.69`(a), not a hoist of them — and it is a POLICY, not a proof, that rots if single-image
> machines grow past 64 TiB. Its re-deriver is tracked: `spawn_boundary_probe.rs`.

## MEASURED — the BEFORE arm (release binary built from HEAD `622c8b5b`)

```
$ ./rust/target/release/parseability_probe --stack-reservation
reservation_bytes=268435456       reservation_mib=256
source=host_memory_derived        host_memory_bytes=25769803776
unclamped_share_bytes=402653184   share_divisor=64
floor_bytes=100663296             reference_bytes=268435456

$ RES=default measure.sh 4814
casts=4814   res=default     rc=1       45.35s
    cause=BudgetExhausted depth=81670 used_bytes=251660912
    reservation_bytes=268435456 budget_bytes=251658240 bytes_per_frame=3081
```

✅ **P1 holds exactly** — the cap is binding and `unclamped_share_bytes` shows what it is
withholding. ✅ **P3's before-half holds**: `cast_4814` is REJECTED at the default.
⭐ **And it reproduces an independent earlier measurement**: `.69`(d) recorded this same input
refusing at depth **81 624** on a different day and a different binary; today's **81 670** is
**0.056 %** away. The AFTER arm's prediction is therefore restated against today's controlled
reading: **81 670 × 1.5 = 122 505**, refuted beyond 2 %.
