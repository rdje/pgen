# `ENGINE-UNIVERSAL-SERVICES.69`(a) slice 1 — pre-registration: does the overshoot scale with the reservation?

⛔ **Written before the third reservation point ran.** Two points define a line, so reading a law off
them is the `-0129` ESTIMATOR failure with a different name.

## What I predicted, and why it matters that I was wrong

Before measuring I reasoned: the probe's stride is a fraction of what is still affordable, affordable
is a fraction of the budget, and the budget is a fixed share of the reservation — **so the overshoot
should be PROPORTIONAL to the reservation and the margin should be INVARIANT**, exactly as `.79`(b)(c)
made the first-sample margin invariant.

Measured on the `pattern` shape (`'{'{'{ 1 }}}`):

| reservation | budget | reserve | used | overshoot | margin |
|---|---|---|---|---|---|
| 96 MiB (the documented floor) | 94 371 840 | 6 291 456 | 100 073 408 | **5 701 568** | **1.10×** |
| 256 MiB (the shipped default) | 251 658 240 | 16 777 216 | 256 727 776 | **5 069 536** | **3.31×** |

⇒ the overshoot is **roughly CONSTANT** (5.07–5.70 MB, 1.12× apart) across a **2.67×** change in
reservation, while the reserve scales with the reservation. **My prediction was wrong in the
dangerous direction**: the margin is not invariant, it SHRINKS as the reservation shrinks, so the
documented floor is the binding end rather than an equally-safe one.

## The discriminating third point

Run the same shape at **192 MiB** (`201326592`), which is 2× the floor and 0.75× the default.

| hypothesis | predicted overshoot | predicted margin |
|---|---|---|
| **H-CONST — overshoot is reservation-independent** (what the two points suggest) | ≈ 5.0–5.7 MB | ≈ **2.2–2.5×** |
| **H-PROP — overshoot is proportional to the reservation** (what I reasoned before measuring) | ≈ 11.4 MB | ≈ **1.10×** |

The two predictions differ by more than a factor of two, so the point discriminates. ⚠️ A reading
between them means the overshoot has BOTH a fixed and a proportional term, which is the outcome that
most needs saying rather than rounding to the nearer hypothesis.

## Why this decides something rather than being trivia

`.69`(a) owes a reservation default DERIVED FROM A REAL HOST QUANTITY. If H-CONST holds, then a
derived default is **smaller on a smaller host**, and a smaller reservation has a **smaller margin** —
so deriving the default without also changing the reserve would move small hosts toward the edge.
Under H-PROP the reserve's divisor form would be safe at every size and (a) could proceed alone.
⇒ this is a constraint ON THE FIX `.69`(a) is for, which is why it is measured before the fix is
designed rather than after.
