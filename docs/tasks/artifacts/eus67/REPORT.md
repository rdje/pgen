# `ENGINE-UNIVERSAL-SERVICES.67` (a) — the parse-memory ratio, re-derived over REAL corpus files

Producers (both tracked, both re-runnable):

```bash
docs/tasks/artifacts/eus67/measure_parse_rss.sh <filelist> <out.tsv> [timeout_s] [profile]
python3 docs/tasks/artifacts/eus67/parse_rss_distribution.py \
    --floor-lo 15433728 --floor-hi 19775488 [--dominance 2.0] <out.tsv>...
```

Probe: `rust/target/release/parseability_probe`, profile `sv_2017`, one `fork` per file,
peak RSS from `/usr/bin/time -l`. Host: 24 GB Darwin 25.5.0.

⛔ The profile is a measurement parameter, not a default. SystemVerilog declares no
`@default_profile`, so an unnamed profile parses the union of every dialect
(`ENGINE-UNIVERSAL-SERVICES.62`) and its numbers are not comparable with these.

## Populations measured

| dataset | what | file |
|---|---|---|
| `rss_ge20k.tsv` | **CENSUS** — every one of the 487 SV corpus files ≥ 20 KB | 487 rows |
| `rss_body.tsv` | stratified sample of the body (< 20 KB), every `n/300`-th file size-sorted | 300 rows |
| `rss_syn.tsv` | the six `.63`/`.66` synthetic shapes at n = 2 500 / 5 000 / 10 000 | 18 rows |
| `rss_repro.tsv` | 4 files × 3 repeats, 300 MB – 3 GB | 12 rows |
| `rss_floor.tsv` | `module m; endmodule` × 8 | 8 rows |

**Reproducibility: 0.03 % – 0.48 %** spread over 3 repeats on inputs costing 300 MB to 3 GB.
The percentiles below are stable to well under their own resolution.

## The process floor is a BAND, not a constant

Measured 14.72 MB (lowest peak RSS over any real corpus file) to 18.86 MB (highest of eight
repeats on a 20-byte module). A ~19 % band. Every figure below is reported at **both** ends
so a conclusion that depends on the choice is visible rather than hidden.

## The ratio over real input — files ≥ 20 KB, floor 14.72 MB

| population | n | min | p25 | p50 | p75 | p90 | max |
|---|---|---|---|---|---|---|---|
| all verdicts | 487 | 874× | 940× | 1 022× | 2 237× | 3 352× | 8 158× |
| **ACCEPT** | 59 | 1 042× | 2 181× | **2 595×** | 4 334× | 5 889× | **8 158×** |
| REJECT | 428 | 874× | 935× | 982× | 1 431× | 3 140× | 4 535× |

At the other end of the floor band (18.86 MB) the ACCEPT row reads
min 1 657× · p25 2 094× · p50 **2 407×** · p75 4 183× · p90 5 850× · max **8 040×**.
The conclusion does not move.

Worst real files observed:

| file | bytes | verdict | peak RSS |
|---|---|---|---|
| `verilator/…/t_math_synmul_mul.v` | 330 162 | ACCEPT | **1 961 MB** |
| `opentitan/top_darjeeling/…/pinmux_reg_top.sv` | 1 328 188 | REJECT | **3 178 MB** |
| `iverilog/ivtest/ivltests/br_gh330.v` | 36 673 | ACCEPT | 300 MB (**8 158×**) |

## `.67`'s ⚠️ is DISCHARGED — but the synthetic figures are the tail, not the middle

The six synthetic shapes were re-measured on **this** instrument, which reproduces the
independently produced figures `.66`/`.67` published:

| shape @ 10 000 items | KB | `.67` published | re-measured | delta | marginal ratio |
|---|---|---|---|---|---|
| netlist instances | 523.1 | 4 064.0 MB | 4 062.1 MB | −0.05 % | 7 923× |
| `case` items | 193.2 | 1 143.2 MB | 1 142.3 MB | −0.07 % | 5 976× |
| concat elements | 67.3 | 498.8 MB | 499.8 MB | +0.20 % | 7 380× |
| wire declarations | 135.7 | 572.2 MB | 573.2 MB | +0.18 % | 4 216× |
| module ports | 125.9 | 304.6 MB | 305.0 MB | +0.13 % | 2 361× |
| procedural statements | 154.1 | (not published) | 649.3 MB | — | 4 216× |

Worst agreement 0.20 %. ⇒ the two instruments agree, so the comparison below is between
**populations**, not between measurement methods.

- Synthetic range **2 361× – 7 923×** vs real ACCEPT range **1 042× – 8 158×**.
- The real ACCEPT **max** (8 158×) matches the synthetic max (7 923×) to 3 %.
- The real ACCEPT **p50** (2 595×) sits just above the synthetic **minimum** (2 361×).

⇒ **The synthetic band brackets the real range and is not an artifact — but it
over-represents the expensive end. `.67`'s headline 7 950× is the real p99+, not the real
median.** The honest real-input figure is ~2 600× for an accepted file and ~1 000× across
all verdicts.

## Linearity — confirmed by fit

Least squares over n = 2 500 / 5 000 / 10 000, `peak RSS = per-item × N + intercept`:

| shape | KB/item | intercept | max residual | intercept as % of RSS @10k |
|---|---|---|---|---|
| netlist instances | 412.3 | 36.1 MB | 1.92 MB | 0.9 % |
| `case` items | 115.2 | 16.8 MB | 0.29 MB | 1.5 % |
| procedural statements | 64.2 | 20.0 MB | 6.37 MB | 3.1 % |
| wire declarations | 56.7 | 19.4 MB | 1.40 MB | 3.4 % |
| concat elements | 48.9 | 22.9 MB | 0.56 MB | 4.6 % |
| module ports | 29.5 | 17.4 MB | 0.54 MB | 5.7 % |

Every intercept is the process floor itself (14.7 – 18.9 MB) plus a few MB, and the largest
residual is 6.4 MB against an RSS of 4 GB. **Linear in item count, with no fixed parse
component.** The per-item cost spans **14×** across shapes — 29.5 KB for a module port,
412.3 KB for a netlist instance.

## ⚠️ The one real-input linearity probe disagrees, and it is the most consequential number here

The three `pinmux_reg_top.sv` files (one generator, three chip tops) are the only real
near-identical size ladder in the corpus:

| file | bytes | peak RSS |
|---|---|---|
| `top_englishbreakfast/…/pinmux_reg_top.sv` | 1 161 034 | 2 937 MB |
| `top_earlgrey/…/pinmux_reg_top.sv` | 1 237 143 | 3 044 MB |
| `top_darjeeling/…/pinmux_reg_top.sv` | 1 328 188 | 3 178 MB |

Fit: **marginal slope 1 513×, intercept 1 261 MB**, max residual 1.8 MB against 3 GB. Their
*average* ratio is 2 509×, so on this ladder the marginal byte costs **40 % less** than the
average byte — the opposite of the synthetic shapes, whose intercept is the process floor.

⛔ **This is three points from three DIFFERENT files — one degree of freedom — and the
instrument cannot distinguish a 1 261 MB fixed PARSE cost from 1 261 MB of shared file
CONTENT that all three chip tops emit.** No ruling is drawn from it. It is handed to `.67`(b)
as the concrete question attribution has to answer, because the two readings extrapolate very
differently at netlist scale.

## What "bounded only by host RAM" actually buys, on real input

At the measured real-input ratios, a 16 GB host (≈14 GB usable) tops out at:

| ratio used | source | largest ingestible SV source |
|---|---|---|
| 8 158× | real ACCEPT worst | **~1.7 MB** |
| 2 595× | real ACCEPT median | **~5.4 MB** |
| 874× | real minimum | **~16 MB** |

`.67`'s own "~40 000 netlist instances on a 16 GB machine" re-derives here as **40 551**
(14 GB ÷ 412.3 KB per instance) — **confirmed**. Real ASIC gate-level netlists run 10⁵–10⁷
instances and 100 MB–10 GB of text, so the ingestion bound stands and is short of the target
population by **20× to 6 000×**.

## NOT MEASURABLE, named rather than fudged

**97 % of the corpus is < 20 KB** (median file 826 bytes) and its parse footprint does not
dominate the 14.7–18.9 MB floor: 290 of 299 body rows peak below 2× the floor, and 32 of them
peak *below the floor file's own* RSS.

⛔ Worse, the admissibility rule that removes those rows **selects on the measured quantity**.
At the body's median size the 2×-floor bar is itself ≈ 9 500×, so the "admissible body" p50 of
11 123× is an artifact of the rule. **No ratio is published for the corpus body.**

⭐ The lower bound survives that, because the two *uncensored* size bands clear their own bar
by a wide margin and bottom out at the same place:

| band | n | admissibility bar | observed min | censored? |
|---|---|---|---|---|
| 20–50 KB | 335 | 1 505× | 907× | yes — lower tail cut |
| 50–100 KB | 103 | 599× | 893× | yes — lower tail cut |
| **100–256 KB** | 27 | **301×** | **886×** | no |
| **256 KB–1.9 MB** | 22 | **102×** | **874×** | no |

The ~875× minimum is a property of the parser, not of the selection rule.

## The instrument defect this analysis caught in itself

The first pass computed `(rss − floor) / bytes` and dropped non-positive values with
`if x > 0`. On the body that silently discarded **10.7 %** of rows — every one of them a row
whose peak came in below the floor — and biased the survivors upward. A filter that removes
the inconvenient tail of a distribution and reports nothing is not a cleanup; it is the
defect. `parse_rss_distribution.py` now excludes by a **named rule** and **counts** the
exclusion on every run.
