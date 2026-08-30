# `ENGINE-UNIVERSAL-SERVICES.79`(a) — the per-frame stack-cost census

`.79` established that a stack floor cannot be a byte constant, because the per-frame cost it would
have to encode is a per-(family, profile, build-mode) measurement. This directory holds the
instrument that measures it, so the claim is a TABLE rather than one family's number generalised to
eleven.

## Files

| file | what it is |
|---|---|
| `gen_deep_input.py` | one deep-nesting shape per family — the cheapest recursive construct that family accepts |
| `per_frame_census.sh` | the ladder: climbs by doubling until the depth probe refuses on STACK grounds, then reports the probe's own `bytes_per_frame` |
| `census_debug_96MiB.txt` | the recorded DEBUG run at the 96 MiB floor |

## Running it

```bash
bash docs/tasks/artifacts/engine_universal_services/eus79a_per_frame_census/per_frame_census.sh \
     ./rust/target/debug/parseability_probe
# one cell only:
PGEN_CENSUS_CELLS="regex:" bash .../per_frame_census.sh ./rust/target/debug/parseability_probe
```

⭐ **It runs at a REDUCED reservation on purpose.** Since `.79`(b)(c) the ceiling is proportional to
the reservation and crossing cost is quadratic in source depth (`.68`), so the 96 MiB floor reaches
each family's ceiling far more cheaply than the 256 MiB default. The measured quantity is
reservation-INDEPENDENT — it is `used_bytes / depth` at the refusal — and `-0141` measured the same
reading at 2 048 / 4 096 / 8 192 levels of one shape, which this run reproduces for `json` (2 882 at
16 384 levels here, 2 882 at 50 000 by hand).

## Reading the CAUSE column

Only two causes are facts about the STACK (`.69`(d)); the census stops on either:

| cause | meaning |
|---|---|
| `BudgetExhausted` / `NoFrameAffordable` | a real measurement — `BYTES_PER_FRAME` is the answer |
| `REFUSED_UNOBSERVABLE` | the parse WAS refused (its own diagnostic says so and gives the frame count) but no `stack_probe` reading reached the caller — `ENGINE-UNIVERSAL-SERVICES.82` |
| `RUNG_TIMEOUT_<n>s` | the crossing rung exceeded the wall-clock cap; nothing is claimed |
| `CEILING_NOT_REACHED` | climbed past the ladder's maximum with no refusal AND no rejection |

⛔⛔ **`REFUSED_UNOBSERVABLE` EXISTS BECAUSE THE FIRST CUT OF THIS SCRIPT GOT REGEX WRONG.** It had no
such arm, so a refused regex parse fell through to `CEILING_NOT_REACHED` — i.e. the census would have
published *"this family has no measurable ceiling"* about a family that demonstrably refused, at
262 144 levels of climbing to get there. Caught by checking the row rather than trusting it: the
rejection text reads `ran out at 4096 parser frames` while `stack_probe:` reads `no_refusal`. With
the arm, regex resolves at **512** levels instead of 262 144.

⚠️ **Verdicts, causes and frame/byte counts are deterministic and load-independent. Wall clocks are
not** — check `uptime` before quoting any timing from this (`-0143`).
