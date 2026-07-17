# RGX-0078.5.i.9 (PGEN-RGX-0078-0110) — D3 boundary-scanner land-gate evidence

The `-0110` D3 emission slice's land-gate bench + first-contact tripwire + the
emitted plan report. Reproducible from a checkout at this commit:
`make -C rust focus_regex` (artifact `d2decb51…`), then build both probes and run
`--samples 2000 --warmup 200` alternated (see `analyze_speed.py` in `../rep2/`).

- **Probes:** base = the PRESERVED shipped probe `7310ed73…` (the `-0105`
  candidate binary = the ≈6.11µs floor); cand = fat-LTO `mimalloc_perf` build
  over the D3 artifacts, sha `63c227b1…`. The candidate build is REPRODUCIBLE:
  byte-identical binaries from two independent builds (the hooks-emit interlude's
  extra fns are dead code under fat-LTO — see DEVELOPMENT_NOTES).
- **Rounds (this directory):** the FINAL canonical-vintage run — per-round ratios
  `0.849/0.856/0.849/0.848/0.845`, per-pattern best-mins `0.744–0.904`, EVERY
  round and EVERY pattern faster. **GEOMEAN-OF-MINS: 6181.0ns → 5224.4ns =
  −15.5%.** An earlier same-binaries run measured −14.3% (ratios 0.846–0.861) —
  the two independent 5×2000 runs bracket the honest band; the recorded landing
  value is the final canonical-vintage run.
- **Model verdict:** inside the `.5.i.8` §5.c profile-consistent expectation
  −9…−15% (falsification bar −4% cleared ~4×; the CENTRAL −25.1% regression
  upper half was optimistic, exactly as the honest reconciliation predicted).
- **Base cross-check:** 6181.0ns reproduces the tracked 6110.6ns baseline within
  +1.2% (day variance; both sides alternated, the ratio is the honest number).
- `tripwire_16of16.txt` — the first-contact tripwire: 8/8 bench-pattern ASTs
  (bare path = the scan graph) + 8/8 outcome dumps (the protocol twin, entry
  pins Σ1331) byte-identical to the shipped-vintage references.
- `boundary_scanner_plan.txt` — the emitted `BOUNDARY-SCANNER-PLAN` (19 rules:
  13 text / 5 span_transform / 1 shaped_object; 40 candidates dropped, reasons
  named) from `PGEN_FUSIBILITY_DUMP_ALL=1 --report-fusibility-census`.
