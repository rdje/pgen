# RGX-0078.5.i.12 (PGEN-RGX-0078-0115) — terminal-literal emission land-gate evidence

The `.5.i.12` `match_lit_ascii` emission's land-gate bench: fat-LTO
`mimalloc_perf` probes, 5 alternated rounds × `--samples 2000 --warmup 200`,
analyzed with `../rep2/analyze_speed.py` (geomean-of-mins).

- **Probes:** base = fat-LTO build over the preserved `d2decb51` artifact
  (the `-0110` shipped floor) + the CURRENT lib (sha `e870a728…`); cand =
  fat-LTO build over the `4b0d8242` terminal-literal artifact (sha
  `281b34d3…`). The base binary does NOT reproduce the `-0110` probe hash
  `63c227b1…` because the lib changed (generator modules; fat-LTO layout is
  lib-global) — the honest custody proof is behavioral: base geomean-of-mins
  5298.6 ns reproduces the tracked 5224.4 ns floor within +1.4% (the `-0110`
  day-variance precedent; both sides alternated, the ratio is the verdict).
- **Verdict: GEOMEAN-OF-MINS 5298.6 ns → 5046.8 ns = −4.8%** — inside the
  `.5.i.11` census-fitted expectation band (−4…−7%, LOW −3.7%/MID −5.5%),
  falsification bar −1.8% cleared ~2.7×. The model CONFIRMED in-band on its
  first application.
- **Honest anomalies (named, absorbed by the best-mins protocol):** round 1
  cand geomean 6065.4 (ratio 1.095) is a transient outlier — its cand rounds
  2–5 sit at 5048–5219 (ratios 0.936–0.964); `anchor_complex` is the one
  pattern slower at best-min (16333 → 17167 = +5.1%, layout/I-cache class —
  every other pattern faster, `digit_sequence`/`character_class` −12.8/−12.7%
  = the attempt-dense patterns, as the model predicted).
- New tracked closure floor **≈5.22 µs → ≈5.05 µs (≈98×)** from the 496 µs
  origin.
