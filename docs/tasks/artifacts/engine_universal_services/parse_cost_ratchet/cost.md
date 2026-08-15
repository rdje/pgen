# SV Parse-Cost Ratchet — the BINDING baseline

`ENGINE-UNIVERSAL-SERVICES.20` acceptance (d). Produced by
`stimuli/sv/corpus_parse_cost.py`; held by `scripts/check_parse_cost_ratchet.sh`.

> ⛔ **This file is byte-compared by the gate.** It carries only values that are exact
> functions of the tree — no wall clock, no timestamp, no `HEAD`. That is not tidiness:
> a baseline whose bytes move on their own cannot be diffed, and a baseline nobody
> diffs is the staleness defect this leaf's own `.13i` is a record of.

## What binds, and what it cannot see

| metric | graph observed | machine-dependent | role |
|---|---|---|---|
| total rule entries | PROTOCOL (`parse_*`) | no | **BINDING** |
| bare-parse wall clock | FUSED (`cascade_*`) | yes | advisory, wide band |

⛔ **The binding metric's blind spot, declared rather than discovered.** Requesting the
entry dump routes the parse to the PROTOCOL graph (TOOLBOX 3.4 ROUTING). A production
parse with no diagnostic consumer runs the FUSED `cascade_*` graph, which ticks no
per-rule counters — including the fused twins of the very rules this leaf is about
(`cascade_match_casting_type_lr_suffix`, `cascade_build_select_expression_lr_base`, …).
The +24.3 % that opened `.20` was measured on the fused graph. So this number guards
**structural work** and cannot, on its own, describe that cost. The wall-clock advisory
in `advisory.json` is the only view of the other graph, and it is machine-dependent —
which is why it advises and does not bind. **Neither metric alone is sufficient.**

## Instrument identity (what produced this number)

> Re-hash these three inputs. If any differs, **this baseline no longer describes your
> tree** and the honest act is to re-measure, not to quote. The gate re-hashes them on
> every run — unlike the six oracles `SV-CORPUS-GRAD.13i` found carrying an identity
> block that nothing read, four of which were measurably stale.

| input | repo-root-relative path | sha256 |
|---|---|---|
| grammar | `grammars/systemverilog.ebnf` | `1d4564bddb0dd4467eb71c018a03ff4141ce2c05d106fba17e00df41c7c9e58c` |
| generated parser | `generated/systemverilog_parser.rs` | `463c647603e83af1ed3e41fffb16ac2041f466fe1ac3b0df507161d3fcfc839e` |
| sample inputs | `stimuli/sv/parse_cost_sample.tsv` | `c3e01f2d29714af9bb15e977e3ca52c48045616d1cf157a8c13e5bd645016205` |

`sample inputs` digests the manifest ORDER plus every sampled file's bytes: the corpora
are git submodules, so a bump can move this measurement without touching one byte of
PGEN, and a parser hash alone would not notice.

## The binding numbers

Four exact integers. Each was verified deterministic across repeated release runs AND
byte-identical between the debug and release probes before being made binding.

| quantity | value | what a rise means |
|---|---:|---|
| sample files measured | 192 | — |
| accepted / rejected | 87 / 105 | a correctness move, not a cost move |
| **rule entries** | **416,841,264** | more rule-method entries: structural work grew |
| **committed entries** | **7,124,616** | more surviving work |
| **failed speculation** (`entries − committed`) | **409,716,648** | more probing waste — the mechanism a GUARD spends through |
| **memo hits** | **186,981,263** | memo behaviour moved |

Failed speculation is **98.3 %** of all rule entries
in this sample: the parse is overwhelmingly probing work, so a guard that probes more
shows up here long before it shows up in the raw entry count.

⚠️ **`committed` is only meaningful for an ACCEPTED parse** (TOOLBOX 3.5) — a rejected
parse commits nothing durable. The accepted-only sub-total is published beside the
whole-sample one so neither is mistaken for the other: over the 87 accepted
files, entries 110,982,162 and committed 7,069,818.

## The guarded admission's own family

| quantity | value |
|---|---:|
| `*_lr_base` / `*_lr_suffix*` entries | 3,092,966 |
| of those, committed | 127 |
| family share of all entries | 0.742 % |

⭐ Published so `.20` acceptance (b)'s third A/B arm computes its delta straight off this
artifact instead of re-deriving it.

⛔⛔ **AND IT CARRIES A FINDING THAT BOUNDS THIS WHOLE INSTRUMENT.** Across the full
16 336-file corpus the family takes **0.681 %** of all rule entries. The flip's entry
DELTA is smaller still — the rules it replaced were themselves entered — so the entry
count moved **well under 1 %** while wall clock moved **+24.3 %**. ⇒ the binding metric
is roughly **35× less sensitive** to *this* regression than the advisory one. That is not
a reason to discard it: it catches structural growth EXACTLY and cannot be fooled by a
busy machine. It is a reason to state plainly what it does **not** prove — the +24.3 %
is a rise in cost PER entry, not in the NUMBER of entries, and no counter can see that.
`.20` acceptance (a)'s profile is what attributes it; this ratchet stops it growing
further unwatched in the meantime.

## Per tier

| tier | files | entries | what it is for |
|---|---:|---:|---|
| `hot` | 40 | 351,463,530 | the heaviest files — where parse cost concentrates |
| `lr` | 40 | 63,034,781 | the heaviest guarded-admission files — the mechanism `.20` owns |
| `breadth` | 112 | 2,342,953 | a deterministic stride across the rest — so the ratchet is not blind elsewhere |

_Per-file rows: `entries.tsv` (6 columns: sub-corpus, tier, path, accepted, entries,
lr-family entries), sorted by (sub-corpus, path) so two runs diff directly._
