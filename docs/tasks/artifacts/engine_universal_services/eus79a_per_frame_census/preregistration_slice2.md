# `ENGINE-UNIVERSAL-SERVICES.79`(a) slice 2 — pre-registration

⛔ **Written BEFORE the deciding run finished**, and tracked for the same reason `.69`(d)'s
`preregistration.md` is: that record is the only reason a 0.004 % overshoot was not published as a
refutation of a bound. The running series of pre-registration failures in layer A is a series of
FRAMES a prediction inherited instead of checking, so the frame is named here first.

## The observation that forced it

The RELEASE census read `systemverilog` / `verilog_2005` at **3 709 B/frame** on the nested-paren
shape at the 96 MiB floor. `.69`(d) published **3 071–3 083 B/frame** for release SystemVerilog on
the `cast_4814` shape (two reservations, 0.39 % apart). The two disagree by **20 %**, which is four
times the ±5 % this leaf established for a per-frame reading.

⚠️ **The comparison is CONFOUNDED and I must not attribute it before measuring.** Two axes differ
between the readings at once:

- **PROFILE** — mine is `verilog_2005`, `.69`(d)'s `cast_4814` is not a `verilog_2005` input.
  Measured in debug on ONE shape, this axis is worth 18.3 %: `sv_2017` 24 559 vs `verilog_2005`
  29 046.
- **SHAPE** — mine is `assign w = ((( 1 )))`, `.69`(d)'s is a sign-coerced cast chain. `.69`(d)
  states in its own words that *"per-frame cost is a property of the SHAPE, not of the stack it runs
  on"*, and `.69`(a) leaves an UNMET acceptance criterion saying the reserve must be sized against
  the WORST shape, with an 88× spread already measured in overshoot at ONE reservation.

## The deciding run, and what each outcome means

The in-flight retry measures release `systemverilog` / **`sv_2017`** on the SAME nested-paren shape.
That holds SHAPE fixed and moves PROFILE, which is exactly the axis `.69`(d)'s reading also moves.

| prediction | release `sv_2017`, paren shape | what it would mean |
|---|---|---|
| **P1 — PROFILE-driven** | ≈ **3 135** B/frame (= 3 709 ÷ the debug profile ratio 29 046/24 559 = 1.183), i.e. within ~2 % of `.69`(d)'s 3 071–3 083 | the disagreement is the profile axis this census already controls; `.69`(d)'s number and mine are the same measurement of different profiles, and nothing is wrong |
| **P2 — SHAPE-driven** | ≈ **3 700** B/frame, i.e. ~20 % above `.69`(d)'s cast reading | the census's axis list — family × profile × build mode — is INCOMPLETE, because a fourth axis moves the quantity by more than the spread the leaf publishes as its precision |

⛔ **Neither outcome retracts a published number.** `.69`(d)'s 3 083.27 was measured on its own shape
and stands; this census's numbers are measured on theirs and stand. What is at stake is whether the
census's own STATED axis list describes the quantity it publishes.

⚠️ **P1 and P2 are not exhaustive.** A reading between them (say 3 300–3 500) splits the difference
and means BOTH axes are material at once, which is the outcome that most needs saying out loud
rather than rounding to whichever hypothesis is nearer.

## The second thing this slice must not explain away

The first release run was killed by the memory guard at **12 561 MB** of process-tree RSS while
measuring `return_annotation` at 16 384 nesting levels — an input of roughly **33 KB**. That is
≈380 000× the input size, against `.67`'s CLOSED ingestion bound of p50 **2 595×** / max **8 158×**.
⛔ `.67`'s bound was derived over a 487-file corpus census of REAL SystemVerilog, so a pathological
deep-nesting shape is outside the population it was fitted on — the honest reading is that the bound
is not contradicted, its POPULATION is. Whether the growth law on this shape is owned by any existing
leaf (`.74` thin-table zero-fill, `.75` transient traffic) is a question for a census, not for a
guess, and it is recorded here so the finding cannot quietly become a footnote about a killed job.
