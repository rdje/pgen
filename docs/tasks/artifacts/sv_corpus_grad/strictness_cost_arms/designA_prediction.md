# `designA` — the PREDICTION, written before the arm was measured

⛔ **Why this file exists.** `.13c.2k`'s price was published as a prediction, refuted by the fix by
16,547,053 entries, and the refutation is the most useful thing the leaf produced — because the
prediction was written down in a form that could be wrong. This slice adds a THIRD arm whose whole
purpose is to turn *"the call-site spelling is worse"* from an assertion into a comparison. A
prediction recorded after seeing the number is not a prediction.

**Written at 2026-08-19 14:21 CEST**, with the `designA` probe still compiling (`arm_designA.out`
contained no `ARM-COST:` line — checked, not assumed) and `arm0/t_only/designB` already measured.

## The model

Under `designA` the two identifier rules keep their HEAD definitions and the 45 unguarded
references become `non_keyword_identifier`. Two terms move, in OPPOSITE directions:

1. **An extra rule layer at the 45 sites.** Today those sites enter `identifier` directly (1 entry).
   Under `designA` they enter `non_keyword_identifier`, which enters `identifier` (2 entries).
   Measured at HEAD: `identifier` 10,349,663 calls, of which `non_keyword_identifier` accounts for
   5,504,191 ⇒ **≈ 4.85 M calls arrive through the raw sites**, so this term is **≈ +4.8 M entries**.

2. **The guard is NOT re-run per call, because every rule is memoized.** The naive reading — "the
   guard now runs on ~10 M calls instead of ~1 M" — is the one `price.py`'s own economics refute:
   `non_keyword_identifier` takes 0 memo hits TODAY only because it is asked at few distinct
   positions. Absorbing the raw sites makes it asked at the same ~1.03 M distinct positions
   `identifier` is asked at, so its MISSES — the only calls that run the guard body — should be
   ≈ 1.03 M, i.e. **about the same guard cost as `designB`**, with the difference showing up as a
   large RISE in `non_keyword_identifier`'s memo hits.

3. **The redirected speculation is unchanged.** `designA` and `designB` reject the same texts at the
   same positions (the one exception is the negation site, below), so the +17.8 M identifier
   re-entries that dominated `designB` should recur essentially unchanged.

## The numbers this predicts

| quantity | prediction |
|---|---|
| `entries`, `designA` vs `designB` | **HIGHER by ≈ +4 M to +6 M** (term 1, with terms 2-3 ≈ neutral) |
| `memo_hits`, `designA` vs `designB` | **HIGHER** — `non_keyword_identifier` starts taking hits it does not take today |
| `committed` | ≈ flat vs `designB` (same accept set apart from the negation site) |

⇒ **`designA` is predicted to be strictly worse than `designB`, and by a modest margin — not by the
2× the "one guard evaluation per call" reading would give.** If the measured gap is ≈ +11 M or more,
this model is wrong in the same way `price.py`'s was: it will have mis-modelled memoization again.

## The one place the two arms are NOT equivalent

`rooted_tf_call_sv_only`'s `!( identifier )` is the census's sole `negation` site. `designB` guards
`identifier` itself, so that lookahead starts succeeding on keyword-headed input (a WIDENING);
`designA` leaves the site untouched. Any `committed` difference between the arms should be looked
for there first.
