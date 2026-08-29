# `ENGINE-UNIVERSAL-SERVICES.67`(b) leg 3 — PRE-REGISTERED PREDICTION

Written **before any leg-3 number was read**, and committed in the same slice as the
result so it cannot be edited to fit. `-0124` did this and had 2 of 4 legs falsified,
including its headline; that is the only reason its small numbers are believable rather
than convenient.

The instrument (`ArenaChunkModel` + the report's second line) and its oracle
(`arena_chunk_oracle_probe`, 230/230 exact, RED control 230/230 rejecting a 48-byte
perturbation) were built and proven first. What follows are predictions about the REAL
PARSES, none of which had been run when this file was written.

## What the previous legs left open

`-0121`/`-0124` could bound resident arena bytes only by `[items, ~2 × items]`, so the
arena's share of the floor-subtracted peak was **"anywhere in 28–71 %"** and the
withdrawn conclusion — *"a substantial majority is provably NOT the AST"* — stayed
withdrawn. The lower-bound share (`arena item bytes + owned heap`) measured
**14.8–36.2 %** over the admitted ladder rows.

## Predictions

| # | claim | prediction |
|---|---|---|
| 1 | chunk overhead `chunk bytes / item bytes`, aggregate, per input | **1.25–1.65**, median ≈ **1.40** |
| 2 | the overhead is dominated by the `nodes` arena | **yes** — `nodes` is ~80 % of item bytes |
| 3 | RESIDENT arena+AST share of the floor-subtracted peak, admitted rows | **20–55 %**, median **35–45 %** |
| 4 | the measured points sit in the LOWER half of the old 28–71 % band | **yes** — a doubling arena is rarely caught at its worst |
| 5 | the withdrawn conclusion *"a substantial majority is provably NOT the AST"* | **NOT restored** — the honest statement will be that the AST is a large MINORITY |
| 6 | `items_agree` (model item count vs `Arena::len()`) | **true on 11 of 11** |
| 7 | `extend_regrow` (an `alloc_extend` outrunning its own `size_hint`) | **0 on 11 of 11** |
| 8 | no-regression: peak RSS and wall clock, instrument OFF | **within ±1 %** on every input |
| 9 | the pre-leg-3 columns of the producer's TSV | **re-derive UNCHANGED** against `attribution_owned.tsv` |

## Why 1 and 3 are the ones that can embarrass me

Prediction 1 is arithmetic on a doubling allocator under a log-uniform stopping point
(`2 ln 2 ≈ 1.386`), so it is the closest thing here to a derivation — which is exactly
why a miss would be informative: it would mean the extend arenas behave nothing like the
`alloc` arenas, and `-0125`'s qualification (2) would be even more binding than it read.

Prediction 3 is the one the leaf's headline depends on. If it lands ABOVE 55 % the
withdrawn conclusion is not merely unrestored, it is refuted in the opposite direction —
the footprint would be mostly AST after all, and `-0121`'s falsified pre-registration
would itself have been falsified.

## The named risk in this prediction

Every figure is over the ELEVEN-input ladder, which is 5 real files and 6 synthetic
shapes. It is not the corpus. `.67`(a) already showed the ladder's shapes bracket the
real ACCEPT range rather than sit in its middle, so a ladder median is not a corpus
median and will not be published as one.

---

## ADDENDUM — a SECOND pre-registration, written after the first measurement exposed a defect in the QUESTION

The ladder was measured against the predictions above and two of them were falsified. Before that
result could be published, running the numbers exposed something the predictions had not asked
about at all: `ArenaChunkModel` reports what `typed_arena` took from the **allocator**, and this
leaf is attributing **peak RSS**. Those are the same quantity only if allocated-but-never-written
chunk pages are resident — and on a lazy-commit OS they are not.

⛔ **That is a defect in the question, not in the model.** The oracle proved the model reproduces
the allocator exactly, 230/230; it can say nothing about residency, because it measures the
allocator too. Publishing the allocated figure against an RSS denominator would have attributed
the arena's unused chunk slack to the arena — silently, in the flattering direction, with an
oracle standing behind it.

⛔⛔ **AND THE FIRST THING THIS ADDENDUM HAS TO SAY IS THAT ROWS 10 AND 11 ARE NOT
PRE-REGISTERED, BECAUSE I RAN THE EXPERIMENT BEFORE I WROTE THEM DOWN.** The reasoning
(lazy commit ⇒ RSS should track written bytes) is what made me design the `N` vs `N + 1`
arms to be decisive in the first place, and it did precede the run — but a prediction that
was not written down before the measurement is not a pre-registration, it is a recollection,
and this file is exactly the place where that distinction is the whole point. They are
recorded below as **POST-HOC**, labelled, rather than quietly folded in with the rest.

| # | claim | prediction | status |
|---|---|---|---|
| 10 | peak RSS tracks TOUCHED bytes, not allocated | **yes** — fresh anonymous pages are not resident until written | ⛔ POST-HOC, not pre-registered |
| 11 | the `N` vs `N + 1` arms differ in peak RSS by | **< 5 %**, against a 2.00× difference in allocated bytes | ⛔ POST-HOC, not pre-registered |
| 12 | written/item ratio (`touched chunk bytes / arena item bytes`) | **1.00–1.15** — chunks are retired FULL, so nearly every written slot holds an item | ✅ pre-registered (the ladder has not been re-run) |
| 13 | the corrected TOUCHED share vs the first (allocated) reading | **strictly lower on every input**, and materially so where the arena had just grown | ✅ pre-registered |

⭐ Prediction 12 is the one that can still embarrass me: if written bytes turn out far above item
bytes, the `alloc_extend` arenas must be retiring partly filled chunks at scale, and the
"chunks are retired full" reasoning behind `touched` would be wrong in a way the `alloc`-only
intuition hid.
