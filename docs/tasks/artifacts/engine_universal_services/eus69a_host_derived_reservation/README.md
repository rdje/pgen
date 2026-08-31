# `ENGINE-UNIVERSAL-SERVICES.69`(a) SLICE 2 — the host-derived reservation default

The last literal `.65` left standing was `DEDICATED_PARSE_STACK_BYTES = 256 MiB` — the number the
recursion ceiling is derived FROM, so while it was compiled in the ceiling could not be a function
of the host. This directory holds the producers and the recorded runs for making it one.

## The law

```text
reservation = min( 256 MiB , max( 96 MiB , physical_memory / 64 ) )
```

⭐ **The share is chosen so `256 MiB × 64 = 16 GiB`** — the law returns the historic literal
EXACTLY on a 16 GiB host, the developer-machine class it was typed for. It *explains* the constant
rather than replacing it, which is also the strongest available no-regression argument for a change
to a default: every host at or above the design point is provably accept-set-neutral.

⛔ **The share is a POLICY, not a proof.** Deriving it needs the total process footprint at full
stack spend, and `.84` records that quantity as NOT ATTRIBUTED. A bound with an unmeasured term is
not a bound, so none is claimed. `.84` is the leaf that would let it be derived.

## Files

| file | what it is |
|---|---|
| `preregistration.md` | written BEFORE any deciding measurement and before any source edit. Names four frames and seven falsifiable predictions. **P1–P7 all confirmed.** |
| `derivation_check.sh` | producer. Five arms: the resolved reservation + provenance from the shipped binary; the same host quantity from `sysctl`/`/proc/meminfo` (a SECOND instrument); and the three override arms. |
| `derivation_check_debug.txt` | its recorded run. |
| `red_controls.txt` | the mutation battery: one targeted mutation per new lock, each required to make that lock FAIL. **6 of 7 RED.** |
| `generated_digests_after_regeneration.txt` | the zero-generated-bytes proof — 11/11 byte-identical across a full regeneration, and why the gate could not have supplied it. |

## Why every number here is load-independent

The host carried an unrelated `rustc` at ~27 % CPU and a load average of 4.28 throughout this
slice. **Nothing measured here is a wall clock or a peak RSS** — every column is an integer the
process holds at startup, or a `sysctl` reading. Layer A's rule (*never measure peak RSS on a busy
machine*) is why the two cost figures this slice needed — the `.78` ladder's crossing rung and its
runtime — were left as ARITHMETIC over already-measured per-frame costs and routed to `.87`(a) to
be timed on an idle host, rather than estimated here and published as measurements.

## ⛔⛔⛔ The finding that matters most is about the CONTROLS, not the code

The battery required each new lock to fail under a targeted mutation. Six did. The seventh — for
**the most important lock in the slice**, *the first-sample anchor must not track the derived
default*, whose failure would restore `.79`'s measured `rc 134` — came back **GREEN**.

Root cause, chased rather than shrugged at: `.69`(a)'s own upper clamp makes
`dedicated_parse_stack_bytes()` **numerically identical** to `DEDICATED_PARSE_STACK_BYTES` on any
host with ≥ 16 GiB. The mutation is a no-op *here*; the defect is only observable in the 6–16 GiB
derived band.

⇒ **a control that cannot fail on your host passes for that reason, and a green battery hides it.**

The fix is the move this module had already made once: `scale_sample_depth_to_reservation` was
split into a pure kernel *"because the resolved reservation is a process-wide `OnceLock`: a test
that read it could pin the law at exactly one value, which is the least interesting way to check a
law that exists to vary with it."* The ANCHOR needed exactly the same treatment, and now has it —
so both the split form and the collapsed form execute in one process on any host.

⚠️ The wrapper half stays unfalsifiable from a ≥ 16 GiB host and says so in the test body.
`.87`(e) owns removing the host-dependence.
