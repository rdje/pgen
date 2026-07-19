# G STEP-2 — nanosecond-denominated pricing of the surviving levers (`PGEN-RGX-0078-0168`)

Session #168, 2026-07-20. **Measurement + docs only — zero code, grammar, or
artifact change; no build, no regen, no gate perturbed.** This slice executes the
`-0167` NEXT pointer verbatim: a *cheap* pricing step that denominates **G1-C**
and **entry-count fusion** in nanoseconds, side by side, against the measured
noise floor — before either earns a multi-hour regen+A/B chain.

It satisfies the `-0167` STANDING RULE by construction (every lever below gets a
written per-parse ns estimate **and** the noise floor it must clear), and the
`-0163` STANDING INSTRUMENT RULE by construction (all inputs are already-banked
release-path sampling + a static structural census; nothing is re-observed, so
the observability twin cannot flip).

## Method + custody

- **No new measurement was taken.** Inputs are the banked `-0162` artifacts
  (`../geomean_reprofile/strat60_census.jsonl`, `selfcum_tables.txt`) and the
  banked `-0166` noise-floor measurement. Floor and custody untouched: bench
  ≈1,937.4 ns / corpus MAX 483,583 ns / corpus geomean 1,263.4 ns; regex
  artifact `e4924024`, floor probe `1d3fa0ee`.
- Reproducible in one command: `python3 price_levers.py` (banked here, stdlib
  only, no deps) → `pricing_output.txt`. It re-derives the `-0162` banked fit
  exactly as a self-check before doing anything new.
- **Noise floor (the bar every estimate below is measured against):** the SAME
  binary re-run 8× spans **2.28% peak-to-peak** (`-0166`) ⇒ on a 1,263.4 ns
  geomean that is **≈28.8 ns/parse**. A lever whose honest estimate lands under
  this is unrunnable by construction.

## 1. ⛔ The banked `21.3 ns/entry` coefficient is CONFOUNDED — it is ~2× too large

`-0162` banked `min_ns ≈ 306 + 21.3 × entries` (R²=0.75) and derived from it the
campaign's headline: *entries carry ≈85% of marginal cell time ⇒ −25% entries ≈
−21% geomean ⇒ the bar is reachable via ENTRY COUNT ALONE.* That fit reproduces
exactly here (`306.0 + 21.28 × entries`), so the arithmetic was never in doubt.

**What was never checked is whether `entries` is an independent regressor.** It
is not:

| diagnostic | value | reading |
|---|---|---|
| `min_ns ~ entries` alone | `306.0 + 21.28·e`, R²=0.7477 | the banked fit |
| `min_ns ~ bytes` alone | `474.6 + 99.72·b`, R²=**0.7615** | **bytes fits BETTER** |
| `corr(bytes, entries)` | **0.900** (VIF 5.3) | severe collinearity |
| aggregate entries/byte | 5.207 | — |

The two "models" are arithmetically the same fit: the `-0162` per-byte marginal
cost (§1: ≈111 ns/B) divided by the census's own 5.2 entries/byte is
111/5.2 ≈ **21.3** — the banked coefficient recovered from the byte model. So
`entries` was standing in for input size.

**Joint fit — holding input size constant, an entry is worth half as much:**

```
min_ns = 313.9 + 56.79·bytes + 10.28·entries        R² = 0.7946
                               ^^^^^  ± 3.39 (t=3.03, 95% CI [3.5, 17.1])
```

The entries coefficient **collapses 21.28 → 10.28 ns (−52%)** once bytes is in
the model. It stays significant (t=3.03) — entries are *real* work, not an
artifact — but roughly half of what `-0162` attributed to "per entry" is
per-byte work that **survives any wrapper fusion**: the byte still has to be
scanned, matched, and its value built whether or not a wrapper rule wraps it.

⇒ **entries' marginal share of cell time is ≈41%, not the banked ≈85%**
(95% CI [14%, 68%] — wide, because collinearity is what wide error bars look
like; the point estimate is the honest one to steer on, and the *direction* is
not in doubt since the solo fit is biased upward by construction).

This is the `-0166` lesson in a new costume, caught **before** the chain instead
of after it: a paper count (there, removed instructions; here, removed entries)
that was never converted into executed-work nanoseconds.

## 2. Re-priced: entry-count fusion

Both models are reported because the campaign metric is a **geomean**, for which
the log-space fit is the formally correct one; the linear-ns fit is the
conservative end of the band.

| entry cut | `-0162` banked claim | RE-PRICED (linear / log) | ns/parse | vs 28.8 ns noise |
|---|---|---|---|---|
| −10% | −8.5% | −4.1% / −6.1% | 52–77 | 1.8–2.7× |
| **−25%** | **−21.2%** | **−10.2% / −14.6%** | **129–184** | **4.5–6.4×** |
| −40% | −33.8% | −16.3% / −22.3% | 206–281 | 7.2–9.8× |
| −50% | −42.3% | −20.4% / −27.0% | 258–341 | 9.0–11.8× |

⛔ **`-0162`'s "the bar is reachable via ENTRY COUNT ALONE" is REFUTED.**
Crossing −20.9% (geomean 1,263.4 → <1,000 ns) needs **37% (log) to 51% (linear)
of ALL entries removed**, while `-0162`'s own population table puts wrapper
chains at **30–50% of entries**. So even a *perfect* wrapper fusion — every
pass-through wrapper entry in the grammar eliminated, with zero residual cost —
lands the geomean **at** the bar at best, not past it.

✅ **But the lever is emphatically RUNNABLE.** At a realistic −25% entry cut it
prices at **129–184 ns = 4.5–6.4× the noise floor**. It is the largest single
lever the campaign has left, and it has never been attempted.

## 3. Re-priced: G1-C (per-atom `ParseNode` materialization + arena copy + tape push)

The broad `-0162` "allocator+memory traffic" cluster (27–31% self) is **not**
G1-C's addressable mass — it also contains per-parse setup memset, teardown
drops, and memo-table allocation, none of which a per-atom node change touches.
The directly attributable populations are the two the 72-byte copy and the tape
push actually drive:

| population (self%) | sub-1µs | 1–2.5µs | 2.5–20µs |
|---|---|---|---|
| `typed_arena::ArenaT::alloc_extend` | 8.6% | 6.7% | 5.3% |
| `_platform_memmove` | 5.3% | 6.5% | 7.4% |
| **sum** | **13.9%** | **13.2%** | **12.7%** |

Log-share-weighted across the three geomean bands (38.0 / 35.3 / 26.0 per
`-0162` §1) = **13.3% of parse time = 168 ns/parse** of addressable mass.

| capture fraction | saving | vs noise |
|---|---|---|
| 30% (conservative) | −50 ns = −4.0% | 1.8× |
| 50% (mid) | −84 ns = −6.7% | 2.9× |
| 70% (optimistic) | −118 ns = −9.3% | 4.1× |

✅ Runnable — but **smaller than entry-count fusion at every comparable
assumption**, and it carries the higher risk of the two (a build-pass carrier
change, per `-0164`).

## 4. ⚠️ The two live levers OVERLAP — their savings must NOT be summed

`-0162` already recorded this and it survives the correction: *"every fused
entry also deletes its checkpoint+arena+memo-insert constants; the alloc cluster
scales with entries."* Fusing an entry deletes that entry's arena alloc and tape
push — i.e. it captures part of G1-C's 168 ns as a side effect. They attack the
same mass from opposite ends. Any future slice that reports "fusion + G1-C =
X%" by addition is wrong by construction; the second lever must be re-priced on
the floor the first one leaves (the `-0162` §5 "re-price after" discipline).

## 5. ▶️ ADJUDICATION — what earns a chain, what does not

| lever | honest ns estimate | vs 28.8 ns noise | verdict |
|---|---|---|---|
| **entry-count fusion** (−25% entries) | **−129…−184 ns** | **4.5–6.4×** | ✅ **RUN IT — the largest lever left, never attempted** |
| **G1-C** per-atom node/arena/tape | −50…−118 ns | 1.8–4.1× | ✅ runnable, but smaller + higher risk ⇒ sequence AFTER, re-priced on the new floor |
| **G1-B** box `ParseError`'s cold payload | one `drop_in_place` call vs a 1,265 ns geomean | ≪ 1× | ⛔ **UNRUNNABLE BY CONSTRUCTION — do not spend a chain** (confirms the `-0167` recommendation, now with a number) |
| **G1-A** literal-return (landed `-0166`) | measured **+0.11%** | inside noise | ⛔ closed: panic scaffolding is never executed |

**Recommendation (engineer, proceeding on it):** the next heavy chain goes to
**entry-count fusion**, with the **re-priced** acceptance band **−10…−15%**
corpus geomean at a −25% entry cut — explicitly **not** the `-0162` −21% claim,
which this slice refutes. Falsification: better than −4% ⇒ below bar.

⛔ **A campaign-level consequence the director should see (§ surfaced):** with
the corrected coefficient, **no single identified lever closes the <1µs bar.**
The honest ceiling is entry fusion (−10…−27% depending on how much of the
30–50% wrapper population is genuinely fusible) *plus* a re-priced G1-C *plus*
G3 fixed-cost trim (≈171–306 ns of setup/teardown). The bar is reachable only if
entry fusion lands near the top of its band **and** the residual levers stack —
which they only partially do, per §4.

## 6. What this slice did NOT do (scope honesty)

- No new profiling run, no counter census, no build, no regen. Every number is
  derived from already-banked artifacts.
- The 95% CI on the entries coefficient is wide ([3.5, 17.1] ns). Collinearity
  bounds what a 60-cell observational census can resolve. Narrowing it needs an
  **interventional** measurement (fuse one wrapper chain, measure) — which is
  precisely the chain being recommended, so the CI narrows as a by-product
  rather than justifying a separate slice.
- No floor number is banked and no tracker row changes: this slice measures, it
  does not move the floor.
