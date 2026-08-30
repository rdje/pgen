# `ENGINE-UNIVERSAL-SERVICES.69`(d) — measured readings (PGEN-ENGINE-UNIVERSAL-SERVICES-0138)

Release `parseability_probe`, `--profile sv_2017`, macOS/aarch64.
Deterministic columns (cause, depth, bytes) are load-independent; wall clock was taken with
other jobs on the host and is INDICATIVE ONLY.

```
input       reservation  verdict  secs     stack_probe
deep_2400   256 MiB      accept     6.11   no_refusal
deep_2400    96 MiB      accept     6.31   no_refusal
deep_2600   256 MiB      accept     7.40   no_refusal
deep_2600    96 MiB      accept    23.61   BudgetExhausted depth=31568 used=94557920 budget=94371840 b/frame=2995 pct=100.197
deep_3000   256 MiB      accept     9.67   no_refusal
deep_3000    96 MiB      accept   162.96   BudgetExhausted depth=31568 used=94969120 budget=94371840 b/frame=3008 pct=100.633
cast_4814   256 MiB      REJECT    44.68   BudgetExhausted depth=81624 used=251668848 budget=251658240 b/frame=3083 pct=100.004
cast_4814    96 MiB      REJECT   829.23   BudgetExhausted depth=30732 used=94378640 budget=94371840 b/frame=3071 pct=100.007
```

## Proportionality (`.65`'s falsifiability control), SAME input, two reservations

`cast_4814`: depth 81 624 -> 30 732. Depth ratio **0.37651** vs reservation ratio **0.37500**
— agreeing to **0.40 %**, with per-frame cost essentially unmoved (3 083 vs 3 071 B).

## Overshoot past budget is SHAPE-driven, not reservation-driven

| input | reservation | overshoot | frames | reserve margin |
|---|---|---|---|---|
| `cast_4814` | 256 MiB | 10 608 B | 3.4 | 1 582x |
| `cast_4814` | 96 MiB | 6 800 B | 2.2 | 925x |
| `deep_2600` | 96 MiB | 186 080 B | 62.1 | 34x |
| `deep_3000` | 96 MiB | 597 280 B | 198.6 | 11x |

88x spread at ONE reservation. This refuted a claim the leaf had already written down.
