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

> Re-hash these four inputs. If any differs, **this baseline no longer describes your
> tree** and the honest act is to re-measure, not to quote. The gate re-hashes them on
> every run — unlike the six oracles `SV-CORPUS-GRAD.13i` found carrying an identity
> block that nothing read, four of which were measurably stale.

⛔ **`instrument` is the fourth input, and it was added because its absence was a real
hole** (`ENGINE-UNIVERSAL-SERVICES.21`). The first block named three, on the sound
argument that the BINDING counters are an exact function of exactly those. Sound, and
insufficient: the family columns below are functions of the instrument's own classifier,
so correcting that classifier staled every published family number while the identity
tier still reported `fresh`.

| input | repo-root-relative path | sha256 |
|---|---|---|
| grammar | `grammars/systemverilog.ebnf` | `1d4564bddb0dd4467eb71c018a03ff4141ce2c05d106fba17e00df41c7c9e58c` |
| generated parser | `generated/systemverilog_parser.rs` | `46bc8a56469a0abd1f6b495608583af6cc7f2cc3b4721625bed4deed3000f9c7` |
| instrument | `stimuli/sv/corpus_parse_cost.py` | `34845c25f9836c71d8df4cfe21c7fb9b0e5f701a278c8690551cac7cb1780dba` |
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

Failed speculation is **98.3 %** of all rule entries in
this sample: the parse is overwhelmingly probing work, so a guard that probes more shows
up here long before it shows up in the raw entry count.

⚠️ **`committed` is only meaningful for an ACCEPTED parse** (TOOLBOX 3.5) — a rejected parse
commits nothing durable, so its entries are ALL speculation by construction and it drags
the whole-sample ratio up. The accepted-only sub-total is published beside it so neither is
mistaken for the other: over the 87 accepted files, entries 110,982,162 and
committed 7,069,818 — 93.6 %
failed speculation even where the parse succeeded.

## The left-recursion-elimination family

⛔ **This section counted a QUARTER of its own subject until `ENGINE-UNIVERSAL-SERVICES.21`.**
The classifier was written from the shape the prose described (`X_lr_base ( X_lr_suffix )*`)
and matched 97 of the 127 LR rule names the parser declares — no `_lr_seed`, and in a
heading that said GUARDED, not one `_lr_guard` rule. It is now derived from the two
eliminators' emission sites; see the classifier's own comment for the eight shapes.

| quantity | value |
|---|---:|
| family entries (`_lr_base`/`_lr_suffix`/`_lr_seed`/`_lr_guard`/`_lr_alt`) | 12,440,690 |
| of those, committed | 514 |
| family share of all entries in this sample | 2.985 % |

⭐ Published so `.20` acceptance (b)'s third A/B arm computes its delta straight off this
artifact instead of re-deriving it.

⭐ The family commits **514** of its 12,440,690 entries — **0.004 %**. The elimination machinery is, to three
significant figures, **pure speculation**: it is entered, it probes, it rolls back. That is
the expected shape of a structural guard and it is stated here so a later reader does not
mistake the family's entry count for productive work.

⛔⛔ **AND IT CARRIES A FINDING THAT BOUNDS THIS WHOLE INSTRUMENT.** Across the full
corpus the family takes **2.741 %** of all rule entries
(24 644 435 of 899 064 022 entries over 16 335 files, `ENGINE-UNIVERSAL-SERVICES.21`; the previous 0.681 % counted only `_lr_base`/`_lr_suffix`). The flip's entry DELTA is smaller still — the rules it
replaced were themselves entered — so the entry count moved by a few percent at most while
wall clock moved **+24.3 %**. ⇒ the binding metric is **at least ~8.9×
less sensitive** to *this* regression than the advisory one. That is not a reason to
discard it: it catches structural growth EXACTLY and cannot be fooled by a busy machine. It
is a reason to state plainly what it does **not** prove — the +24.3 % is a rise in cost PER
entry, not in the NUMBER of entries, and no counter can see that. `.20` acceptance (a)'s
profile is what attributes it; this ratchet stops it growing further unwatched meanwhile.

**Live LR-family share `2.741/8.9`** (corpus-entry share % / blind-spot factor), derived by
`python3 stimuli/sv/corpus_parse_cost.py --rederive-family-share` into
`docs/tasks/artifacts/engine_universal_services/parse_cost_ratchet/family_share.json` and re-hashed against its four recorded inputs on every run.

⚠️ The published bound was **~35×** until `.21`, computed on the 0.681 % the broken
classifier saw. The gate was under-claiming its own sensitivity by about 4×; the corrected
figure is still a large blind spot and is still the reason (a) exists.

## Per tier

| tier | files | entries | what it is for |
|---|---:|---:|---|
| `hot` | 40 | 351,463,530 | the heaviest files — where parse cost concentrates |
| `lr` | 40 | 63,034,781 | the heaviest guarded-admission files — the mechanism `.20` owns |
| `breadth` | 112 | 2,342,953 | a deterministic stride across the rest — so the ratchet is not blind elsewhere |

_Per-file rows: `entries.tsv` — 9 columns (sub-corpus, tier, path, accepted, entries,
committed, memo_hits, lr_entries, lr_committed), sorted by (sub-corpus, path) so two runs
diff directly. Failed speculation is deliberately NOT a column: it is exactly
`entries − committed` and is derived, never stored._
