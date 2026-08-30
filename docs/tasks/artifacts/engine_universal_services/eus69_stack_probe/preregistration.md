# `.69`(d) — PRE-REGISTRATION, written BEFORE the instrumented binary ran

Recorded so the measurement can REFUTE these, not confirm them. Ten prior
pre-registration failures in this tree are catalogued in layer A; each was a
FRAME inherited instead of checked. Naming the frame here is the point.

## Predictions

- **P1 (cause).** On a deep input that contacts the ceiling, the cause that
  fires is `NoFrameAffordable`, NOT `BudgetExhausted`. Reason: the stride is
  `affordable/4`, so each sample advances a quarter of the REMAINING headroom
  and `used` approaches `budget` asymptotically without crossing it. If this is
  right, `BudgetExhausted` is near-unreachable by construction and the enum's
  "one refusal that is really about the stack" is the OTHER arm.
- **P2 (per-frame).** The MEASURED `bytes_per_frame` lands at or below `.71`'s
  arithmetic upper bound of **≤ 3.008 KB** (cast_paren shape). A measurement
  above it refutes `.71`'s bound, not this instrument.
- **P3 (tightness).** `used_bytes / budget_bytes` at the terminal refusal is
  HIGH (> 90 %), i.e. the derived ceiling really does spend the reservation.
  A low ratio would mean the ceiling refuses with the stack mostly free —
  which is the claim `.71`(W2) WITHDREW, so a low ratio here would REINSTATE it.
- **P4 (cost cliff).** The 16.9× cost step measured at 96 MiB (deep_3000:
  9.61 s -> 162.11 s, same ACCEPT verdict) is accompanied by a refusal record.
  If the instrument reports NO refusal on that input, the cliff has a different
  cause and P4 is refuted.

## What would make each of these WRONG rather than unlucky

- P1 wrong => the stride argument is wrong, and `.65`'s headroom reserve is
  doing something other than what its comment says.
- P2 wrong => `.71`'s published bound is refuted; that is a finding about `.71`,
  and it must be reported as such rather than folded into this leaf silently.
- P3 wrong => `.71`(W2)'s withdrawal was itself wrong.
- P4 wrong => the 96 MiB cliff is NOT ceiling contact; root-cause it separately
  before publishing anything about reservations and cost.

## Frames I am deliberately naming (the thing the ten failures had in common)

- **The QUANTITY**: `used_bytes` is measured from a PADDED base
  (`STACK_BASE_ANCHOR_PAD_BYTES = 64 KiB`), so it OVER-reads spend by up to
  64 KiB. Immaterial at 100+ MiB, decisive at small depths. Do not quote it as
  exact.
- **The ESTIMATOR**: the record holds the LAST refusal, not the deepest or the
  first. On an input that contacts the ceiling many times these differ; say
  which one every published number is.
- **The SET**: one input shape (nested casts) is not the population. Any claim
  about "the ceiling" needs at least a second shape.
