# `ENGINE-UNIVERSAL-SERVICES.61`(a) — ceiling-label evidence

Producers and results for the false-positive census behind `.61`(a)'s NO REGRESSION arm, and
for the `.62` finding that census turned up.

| file | what it is |
|---|---|
| `census_ceiling_label.sh` | the census producer, unnamed profile — runs the SHIPPED fused graph with no env var |
| `census_ceiling_label_2017.sh` | the same census under `--profile sv_2017`, the profile the tracked characterization used and the one that ships |
| `corpus_label_census_unnamed_profile.tsv` | 16 336 rows: `verdict`, `LABELLED`/`plain`, path |
| `corpus_label_census_sv_2017.tsv` | 16 336 rows, same shape, `sv_2017` |
| `es62_profile_flipped_files.txt` | the 28 files that ACCEPT unnamed and REJECT under `sv_2017` — `.62`'s evidence |

## The numbers

| census | profile | files | ACCEPT | REJECT | TIMEOUT | LABELLED |
|---|---|---:|---:|---:|---:|---:|
| A | unnamed (the union — `.62`) | 16 336 | 9 818 | 6 512 | 6 | **0** |
| B | `sv_2017` | 16 336 | 9 794 | 6 542 | 0 | **0** |

Census B reproduces `stimuli/sv/characterization/results.tsv` with **0 differences over all
16 336 files**, so the label added zero false positives on the population an embedder meets.

⛔ **Why the instrument does not use `PGEN_REPORT_MEMO_STATS`.** Reading the guard's counter used
to require it, and `.56`(b) established that every observation channel is a disjunct of the
`bare_parse` selector — observing the parse ROUTES IT OFF the fused graph. A pre-change attempt at
this census timed out on 762 of the first 2 179 files for that reason. The label is emitted by the
shipped parse itself, so these runs need no env var and measure the graph that ships.

⛔ **The probe exits 1 on a rejected parse**, so a nonzero status is not a timeout — only
`gtimeout`'s own 124/137 is. Conflating them reported 2 381 phantom "timeouts" on a population
that parses in milliseconds.

## `.61`(b) — is 4096 frames a defensible LOCATION?

| file | what it is |
|---|---|
| `contact_census.sh` / `contact_census_vhdl.sh` | ceiling-CONTACT censuses — read the guard's own refusal counter, so they cover the ACCEPT path the `(a)` label cannot |
| `contact_census_sv_2017.tsv` / `contact_census_vhdl.tsv` | `verdict`, refusal count, path — 16 336 + 13 720 rows |
| `nesting_depth_census.py` | the INDEPENDENT oracle: `()[]{}` depth from source text, no parser involved |
| `nesting_depth_sv_top400.tsv` / `nesting_depth_vhdl_top400.tsv` | the 400 deepest files per family |
| `shape_cliff.py` | bisects the accept cliff for ten SV construct shapes, deriving frames-per-level for each |

### The ruling's numbers

| family | files | ACCEPT | REJECT | contacting the ceiling |
|---|---:|---:|---:|---:|
| systemverilog (`sv_2017`) | 16 336 | 9 794 | 6 542 | **0** |
| vhdl | 13 720 | 4 369 | 9 351 | **0** |
| **total** | **30 056** | 14 163 | 15 893 | **0** |

Worst measured shape capacity **249** levels (`cast_paren`, 16.4 frames/level); best **816**
(`begin_end`, 5.0); `index_sel` is iterative and has no cliff. Deepest real bracket nesting:
SystemVerilog **22** (p99.9 = 8), VHDL **16** (p99.9 = 6) — an **11.3×** margin.

⚠️ A keyword-block nesting oracle was written and REFUTED before publication: it reported 1 427
on a UVM package, an artifact of prototypes (`extern`/`pure virtual`/`import "DPI-C" function`)
and forward typedefs (`typedef class foo;`) opening nothing — that file reads `function` 4 316
against `endfunction` 3 225. The keyword axis is covered by the guard's counter instead, which
is shape-blind by construction.
