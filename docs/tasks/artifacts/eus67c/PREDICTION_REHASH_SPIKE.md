# PRE-REGISTRATION — `ENGINE-UNIVERSAL-SERVICES.67`(c)(iii): the memo REHASH SPIKE, and the fix pricing

Written **before** the lifetime tape existed and before any number below was read.
Scored by [`score_rehash_predictions.py`](score_rehash_predictions.py) over a fixed roster, so
the census cannot lose a row (`-0127`).

## ⛔ THE QUANTITY, AND THE ESTIMATOR, BOTH NAMED BEFORE THE NUMBER

Five carry-forwards say a pre-registration inherits the frame of the question it was written in.
So, explicitly:

- **QUANTITY.** Every number below is **requested live bytes** — Σ `Layout::size()` over unfreed
  allocations — read by the counting `#[global_allocator]` in `parse_alloc_census_probe`. It is an
  **ALLOCATION** quantity, the same one `peak_live` is, so numerator and denominator are made of the
  same stuff. ⛔ It is **NOT** peak RSS. Where residency is claimed (the fix pricing, P9) the
  instrument changes to `/usr/bin/time -l` and the row says so.
- **ESTIMATOR.** `-0129`'s prediction 7 failed because *"worst delta < 0.5 %"* named no statistic and
  two instruments answered differently for the same run. Every row below therefore names min / median
  / max / exact-count explicitly, and a row that names none is a defect in the row.
- **INSTANT.** `peak_live` is set at ONE instant, identified by the ordinal pair
  (`peak_at_alloc_calls`, `peak_at_free_calls`) the probe already records. The size-class snapshot
  `-0128` reasoned from is taken at the HYSTERESIS instant, which is within one hysteresis step
  (≤ 64 KiB) of the peak and **is not the same instant**. Predictions that compare the two say which.

## ⛔ WHAT IS ALREADY RE-DERIVED, AND SO IS NOT A PREDICTION

Recorded here so it cannot be presented later as a blind call: before writing this file I re-counted
`-0128`'s published *"two success tables are live at once on **4 of 8** inputs"* against its own
tracked artifact `alloc_attribution.txt`, and got **5 of 8**. P3 below is the direct instrument's
verdict on the same question, which is a different measurement, not a repetition of that count.

## ⛔ THE POPULATION, NAMED BEFORE THE RUN

`-0128` attributed **eight** inputs and REFUSED two by name (`syn_case_items_10000.sv` and
`syn_concat_items_10000.sv`: their arena chunk multiset is not derivable from the reported
capacity). The lifetime tape has no arena dependency, so it covers **ten**. Every row below that
quotes `-0128`'s count is therefore scored against **`-0128`'s eight, listed by name in the
scorer**, and the ten-input figure is reported beside it — never substituted for it. Letting the
denominator widen silently would turn a correct prediction into a falsification and an incorrect one
into a pass, which is the `-0127` grouping defect in its arithmetic form.

## THE ROSTER

| # | prediction | estimator |
|---|---|---|
| 1 | the tape never overflows: `tape_overflow == 0` on all 11 ladder inputs at threshold 8 KiB / capacity 65 536 | exact count of overflowing inputs, predict **0** |
| 2 | **CONTROL** — for every size class at or above the tape threshold, the tape's reconstruction of live bytes at the SNAPSHOT instant equals the probe's independently-maintained `SNAP_CLASS_BYTES` | exact count of mismatching (input, class) pairs, predict **0** |
| 3 | two or more memo-family (`T=296`) tables are live at the PEAK instant on exactly **5** of the 8 attributed inputs | exact count |
| 4 | wherever two are live, the process peak lies INSIDE the rehash window — the superseded table's free ordinal is strictly greater than `peak_at_free_calls` | exact count, predict **5 of 5** |
| 5 | rehash overlap (bytes of superseded memo tables live at the peak) as a share of `peak_live`: every overlapping input in **[5 %, 20 %]**, and the **median over the overlapping inputs in [12 %, 17 %]** | per-input min/max, and median over overlapping inputs only |
| 6 | the memo's own peak/steady ratio is **exactly 1.5000** on every overlapping input and **exactly 1.0000** on every non-overlapping one | ratio to 4 dp, exact count of rows matching |
| 7 | `-0128`'s *"the memo's contribution to PEAK is 1.5× its steady-state size"* is TRUE where it was measured and FALSE as a ladder-wide sentence — it holds on 5 of 8, not 8 of 8 | exact count of inputs where the ratio is 1.5 |
| 8 | pre-sizing the memo from input length CANNOT pay for itself: final buckets per input byte spans **more than 3×** across the 8 attributed inputs, so any single length-derived factor overshoots some input by more than the 50 % the spike costs | max/min ratio of (final buckets ÷ input bytes) |
| 9 | **RESIDENCY, different instrument (`/usr/bin/time -l` peak RSS).** Replacing the thin table's `Vec::resize(need, 0)` with a zeroed allocation the kernel backs lazily drops peak RSS on the isolating comment-only shape by **≥ 85 %**, while `peak_live` (allocation) moves by **< 0.1 %** | peak RSS ratio, median of 3 repeats; `peak_live` exact |
| 10 | the thin table's slot index fits 24 bits on the whole ladder: `max(thin_distinct_slots) < 2^24`, so a narrowed `(u8 generation, u24 index)` word is arithmetically possible on this corpus — and is NOT a bound, because nothing caps the index | exact max over 11 inputs |

## ⛔ WHAT THIS PRE-REGISTRATION DELIBERATELY DOES NOT CLAIM

- **The tape cannot compute the counterfactual whole-parse peak.** It records only allocations at or
  above its threshold, so between two recorded events the live curve moves by small allocations it
  does not see. Removing the overlap therefore lowers the peak by **at most** the overlap bytes —
  the peak may relocate to a moment the tape cannot bound. P5 is an **upper bound on the saving**
  and is reported as one, not as the saving.
- **P9 is a mechanism experiment, not an engine change.** It measures whether `alloc_zeroed` pages
  stay non-resident on this host at this size. Whether the engine can adopt it — the recycling
  `ThinMemoScratchLease` performs is the reason the table survives the parse — is priced separately
  and is not settled by P9 alone.
