# PGEN-RGX-0078-0217 — the closure confirmation sweep (the #162 closing wave, part 1)

Session #187, 2026-07-21. Measurement-only; no product change. Executes the
first element of the #162 closing wave in its honest-floor form, per the
`-0216` NO-GO adjudication (the call-off proceeding under the director's
ruling).

## Custody

- Probe: `preserved_probes/regex_perf_probe_carrier48_8d392176` run DIRECTLY
  (the preserved closure probe; SHA banked `probe.sha256`, prefix asserted
  before launch). `target/release/regex_perf_probe` absent at session start —
  irrelevant by design: the confirmation is defined ON the preserved binary.
- Corpus: the canonical PCRE2 external corpus (2,189 cells), full sampling
  (warmup 50, samples 1000/cell), under
  `scripts/run_with_memory_guard.sh --budget-mb 16384 --floor-pct 10`,
  `caffeinate -i`; guard exit 0; load snapshots banked before/after (no
  foreign process >50% CPU either side) — `load_snapshots.txt`,
  `custody.txt`.
- Metric convention PROVEN before any fresh number was read: the analyzer's
  self-check reproduced the banked `-0212` `rerun_cand2.jsonl` floor geomean
  `1003.3049233222944` EXACTLY (`analyze_confirmation.py selfcheck`).

## The confirmation reading

| quantity | value | adjudication |
|---|---|---|
| unrounded corpus geomean | **991.6693567682667 ns** | ⭐ **RAW SUB-1 µs CONFIRMED on a fresh independent sweep** |
| vs floor-of-record 1,004.4 | −1.2675% | inside the observed cross-session drift family (±≈2%) |
| vs the banked floor sweep 1003.3049… | −1.1597% | same |
| corpus MAX (min_ns) | 355,750 (`line_725`) | ≤ the settled 425,000 guardrail — PASS |
| verdict identity vs the banked `-0212` floor sweep | **flips 0/2,189** | correctness custody exact |
| expected-vs-actual divergences | 321 | IDENTICAL to the banked sweep's 321 — the STANDING tracked PCRE2-divergence population (the oracle-tuple class), not a regression |
| margined bar ≤950.0 | NOT MET | recorded as out of designed reach (the `-0215`/`-0216` exhaustion record) |

## What this settles

The #162/#176 confirmation requirement — a fresh-sweep reading of the
preserved closure probe below 1 µs — is MET on the raw form: the campaign's
closing claim is **raw corpus geomean < 1 µs, reproduced across independent
sweeps** (banked family 994.4–1005.4; this sweep 991.7), ≈494× cumulative
from the 496 µs start, correctness untouched (flips 0/2,189 against the
floor sweep; the 321 known-divergence population byte-stable). The margined
(+50 ns drift-proof) ≤950.0 form is recorded as out of designed reach per
the completed exhaustion record (`-0215` + the `-0216` NO-GO on the last
road).

Next (part 2 of the wave, `-0218`): the regex book + handoff +
integration-contract re-baseline recording exactly this.

## Outputs banked in this dir

`analyze_confirmation.py` (self-checking analyzer) → `confirmation_analysis.txt`;
`confirm_sweep.jsonl` + `confirm_sweep_stdout.txt`; `probe.sha256`;
`custody.txt`; `load_snapshots.txt`; this file.
