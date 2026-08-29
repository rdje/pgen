# PRE-REGISTRATION — `ENGINE-UNIVERSAL-SERVICES.67`(b) LEG 4, the allocator-side census

Written after `parse_alloc_census_probe` compiled and **before any number it prints was read**.
Producer: `rust/src/bin/parse_alloc_census_probe.rs`; ladder: the same 11 inputs as
`-0121`/`-0124`/`-0126` (`docs/tasks/artifacts/eus67/attribution_chunks.tsv`).

## ⛔ THE QUANTITY, NAMED BEFORE THE NUMBER

`-0126`/`-0127`'s carry-forward is that a pre-registration inherits the frame of the question it
was written in, so it cannot catch an error in that frame — predictions 3 and 5 of leg 3 were both
wrong because I pre-registered against *allocation* while the leaf asks about *residency*. So each
row below names the quantity, the instrument that reads it, and whether that instrument reads the
same thing its own denominator is made of.

| symbol | quantity | instrument | kind |
|---|---|---|---|
| `peak_live` | max over the process lifetime of Σ `Layout::size()` over unfreed allocations | this probe's counting `GlobalAlloc` | **ALLOCATION** |
| `arena_alloc` | `typed_arena` chunk bytes handed out | `ArenaChunkModel` (leg 3, oracle-proved 230/230) | **ALLOCATION** |
| `owned` | AST-owned child-`Vec` + `String` heap | `OwnedHeapFootprint` (leg 2) | **ALLOCATION** |
| `peak_rss` | max resident set size | `/usr/bin/time -l` | **RESIDENCY** |
| `floor` | process floor, 14.72 MB (band to 18.86) | `.67`(a) | RESIDENCY |

⇒ **The attribution `peak_live = arena_alloc + owned + residual` is allocation-side throughout: one
kind of quantity, so the numerators and the denominator are made of the same stuff.** Any ratio
mixing a live figure with an RSS figure is a **CROSS-QUANTITY** ratio and is labelled as one below;
leg 3 measured that gap at 1 008 MB of allocation costing 32 768 B of RSS, so it is not assumed
small.

## Predictions

| # | prediction | kind |
|---|---|---|
| 1 | `peak_live / (peak_rss − floor)` lands in **[0.50, 1.10]** on every ladder input, with the **median in [0.65, 0.90]** — i.e. floor-subtracted peak RSS usually EXCEEDS peak live requested bytes | CROSS-QUANTITY |
| 2 | the allocation-side residual `1 − (arena_alloc + owned)/peak_live` is a **MINORITY** of `peak_live` on every ladder input, in **[0.20, 0.60]**, median ≈ **0.35** | allocation |
| 3 | the residual share is **larger on the two real OpenTitan/verilator files than on `syn_netlist_instances`**, because the netlist shape is the most arena-dominated in the leg-3 ladder (2.5 % chunk slack, 78–89 % nodes arena) | allocation |
| 4 | at the near-peak snapshot there is **at least one size class holding ≥ 5 % of `peak_live` in ≤ 4 live allocations**, on every input ≥ 30 KB — i.e. the memo table and/or the tape are single large allocations and are individually visible, not smeared | allocation |
| 5 | **`alloc_bytes / peak_live ≥ 2.0`** on every ladder input — allocation traffic is at least twice the live high-water, so transient churn is real and is NOT what peak is made of | allocation |
| 6 | **`free_bytes / alloc_bytes ≥ 0.50`** on every ladder input | allocation |
| 7 | `end_live` (after the parse returns and the arena is dropped) is **< 10 MB** on every ladder input | allocation |
| 8 | the counting-allocator binary's `peak_rss` is within **±2 %** of the canonical `parseability_probe`'s on every ladder input | residency |
| 9 | the **sum of live bytes in classes ≥ 2^20** at the snapshot is **≥ `arena_alloc`** on every input ≥ 30 KB (arena chunks are large by construction, so they must show up there) | allocation |

⚠️ **CONTROL, NOT A PREDICTION** — `peak_live − snapshot_live ≤ 1 MiB` is true by construction of
the hysteresis, so it verifies the instrument rather than the parser. It is listed so the scorecard
is a complete census of this file (`-0127`(2): a count over an incomplete table is an unfinished
census) and it is scored as a control.

## What would falsify the leaf's framing rather than a number

If prediction 1 comes back **> 1.10** — peak live requested bytes exceeding floor-subtracted peak
RSS — then arena slack that leg 3 proved non-resident is a large part of `peak_live`, and
`peak_live` is then the wrong denominator for a residency question. In that case the residual share
must be quoted against the **touched** arena figure and re-derived, not rescaled.

## ADDENDUM — prediction 10, added before any number was read, derived from SOURCE

⭐ **PROVENANCE, stated exactly rather than left to be assumed** (`-0126` labelled two rows POST-HOC
because the reasoning preceded the run but the file did not): this row was written **after** the
rows above and **before** `parse_alloc_census_probe` had produced a single figure — the build was
still running. It comes from reading the generated parser's constructor, not from data.

`generated/systemverilog_parser.rs:4390` builds the thin-memo scratch as
`ThinMemoScratchLease::take(); lease.begin(THIN_RULE_COUNT * (input.len() + 1))` with
`THIN_RULE_COUNT = 109` (`:3291`), and `ThinMemoScratch::begin` (`rust/src/ast_pipeline/mod.rs:2426`)
does `self.slots.resize(need, 0)` on a `Vec<u64>`. ⇒ **a dense table of `109 × (len + 1)` eight-byte
slots — 872 bytes per input byte — allocated AND written (`resize` zero-fills), so unlike arena chunk
slack it is resident by construction.** It is also linear in input size with a constant that has
nothing to do with what the file contains.

**PREDICTION 10.** The near-peak size-class table contains, on every ladder input ≥ 30 KB, a single
live allocation of exactly `109 × (bytes + 1) × 8` bytes:

| input | file bytes | predicted thin-scratch bytes | MB | expected class |
|---|---|---|---|---|
| `floor.sv` | 20 | 18 312 | 0.02 | 15 |
| `spi_device_reg_pkg.sv` | 30 419 | 26 526 240 | 25.30 | 25 |
| `br_gh330.v` | 36 673 | 31 979 728 | 30.50 | 25 |
| `syn_concat_items` 10 000 | 68 923 | 60 101 728 | 57.32 | 26 |
| `comp1000.v` | 69 795 | 60 862 112 | 58.04 | 26 |
| `syn_port_list` 10 000 | 128 912 | 112 412 136 | 107.20 | 27 |
| `syn_netlist_instances` 2 500 | 130 637 | 113 916 336 | 108.64 | 27 |
| `syn_case_items` 10 000 | 197 863 | 172 537 408 | 164.54 | 28 |
| `t_math_synmul_mul.v` | 330 162 | 287 902 136 | 274.56 | 29 |
| `syn_netlist_instances` 10 000 | 535 637 | 467 076 336 | 445.44 | 29 |
| `pinmux_reg_top.sv` | 1 237 143 | 1 078 789 568 | 1 028.81 | 31 |

**PREDICTION 11.** Against `-0121`'s published floor-subtracted peaks that makes the thin scratch
**10 %–42 % of the parse footprint**, and — the sharp part — **it is largest exactly where the AST
share is smallest**: `spi_device_reg_pkg.sv` (AST 15.6 %, thin scratch ≈ 41 %) and
`syn_port_list` 10 000 (AST 18.3 %, thin scratch ≈ 37 %) are the two lowest AST rows in leg 3's
ladder. I predict the rank correlation between AST share and thin-scratch share across the ten
non-floor inputs is **negative**.

⚠️ **TWO WAYS THIS CAN BE WRONG, NAMED NOW.** (i) `input.len()` is the length of the string the
PARSER receives, which for SystemVerilog may be preprocessed text rather than the file — if so every
predicted byte count is wrong by the preprocessing ratio while the FORM `109 × (n + 1) × 8` still
holds, and the measured allocation then tells me `n`. (ii) `ThinMemoScratch` is a recycled
`thread_local`, so in a process that parses once it is a fresh `Vec` and `resize` allocates exactly
`need`; a harness that parsed twice would see the second parse allocate nothing. This probe parses
once per process.

⛔ **AND THE QUANTITY, AGAIN.** `resize(need, 0)` WRITES every slot, so this is one of the few
allocation figures that is also a residency figure. That is a claim about `Vec::resize`, and the
allocator census cannot check it — the class table proves the bytes were REQUESTED. If it matters to
the ruling it must be measured the way leg 3 measured chunk residency, not assumed from the word
`resize`.
