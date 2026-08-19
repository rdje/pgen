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
| grammar | `grammars/systemverilog.ebnf` | `63a0dd49cdf5d5f365c998430c114fd47f3384fd10f9b774ba6d2925f5a7dde7` |
| generated parser | `generated/systemverilog_parser.rs` | `5a1dfa3620b2d387551499f642e363c6996f02f6c2d8245a472949d287f621a0` |
| instrument | `stimuli/sv/corpus_parse_cost.py` | `f106e3613df034e7552b0694a32e67738826d09a01b3fcb9e1643cee36e75051` |
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
| **rule entries** | **413,108,276** | more rule-method entries: structural work grew |
| **committed entries** | **6,759,475** | more surviving work |
| **failed speculation** (`entries − committed`) | **406,348,801** | more probing waste — the mechanism a GUARD spends through |
| **memo hits** | **183,279,718** | memo behaviour moved |

Failed speculation is **98.4 %** of all rule entries in
this sample: the parse is overwhelmingly probing work, so a guard that probes more shows
up here long before it shows up in the raw entry count.

⚠️ **`committed` is only meaningful for an ACCEPTED parse** (TOOLBOX 3.5) — a rejected parse
commits nothing durable, so its entries are ALL speculation by construction and it drags
the whole-sample ratio up. The accepted-only sub-total is published beside it so neither is
mistaken for the other: over the 87 accepted files, entries 109,980,463 and
committed 6,708,591 — 93.9 %
failed speculation even where the parse succeeded.

## The left-recursion-elimination family

⛔ **This section counted a QUARTER of its own subject until `ENGINE-UNIVERSAL-SERVICES.21`.**
The classifier was written from the shape the prose described (`X_lr_base ( X_lr_suffix )*`)
and matched 97 of the 127 LR rule names the parser declares — no `_lr_seed`, and in a
heading that said GUARDED, not one `_lr_guard` rule. It is now derived from the two
eliminators' emission sites; see the classifier's own comment for the eight shapes.

| quantity | value |
|---|---:|
| family entries (`_lr_base`/`_lr_suffix`/`_lr_seed`/`_lr_guard`/`_lr_alt`) | 12,440,410 |
| of those, committed | 514 |
| family share of all entries in this sample | 3.011 % |

⭐ Published so `.20` acceptance (b)'s third A/B arm computes its delta straight off this
artifact instead of re-deriving it.

⭐ The family commits **514** of its 12,440,410 entries — **0.004 %**. The elimination machinery is, to three
significant figures, **pure speculation**: it is entered, it probes, it rolls back. That is
the expected shape of a structural guard and it is stated here so a later reader does not
mistake the family's entry count for productive work.

⛔⛔ **AND IT CARRIES A FINDING THAT BOUNDS THIS WHOLE INSTRUMENT.** Across the full
corpus the family takes **2.761 %** of all rule entries
(24 633 438 of 892 171 789 entries over 16 336 files, `ENGINE-UNIVERSAL-SERVICES.21`; the previous 0.681 % counted only `_lr_base`/`_lr_suffix`).

⛔ **What this metric cannot see, stated as a property rather than as a number.** These
counters tick only in the PROTOCOL graph, and a counter counts EVENTS — a rise in the cost
PER event is invisible to it on any graph. So the binding numbers guard **structural work**
exactly and price nothing. That limit is a property of the metric, so it cannot go stale.

⭐ **On the one change this ratchet was built for, the counters were not the blind half.**
The guarded admission moved them **+10.59 %** — **3.49× larger** than the whole
family's own entry count, and far outside any band a ratchet could hide. The wall clock,
meanwhile, produced no admissible figure for the same change at all.
(ARM 1 812 963 769 -> ARM 2 899 064 022 rule entries over 16 335 corpus files, `docs/tasks/artifacts/engine_universal_services/guard_ab_entries.txt` (`ENGINE-UNIVERSAL-SERVICES.20` (b), three-arm A/B).)

⛔⛔ **This section published a sensitivity bound of `~8.9×`, and BOTH terms were wrong.**
Retired by `ENGINE-UNIVERSAL-SERVICES.26`. It was `24.3 / 2.761`. The
numerator, a `+24.3 %` wall-clock regression, is REFUTED — not reproducible from the raw
data of the runs that produced it (`.20` slice 5). The denominator was the wrong quantity
independently of that: sensitivity is how much the counter MOVED, not how large the family
is, and the bound's *"the flip's entry delta is strictly smaller"* was an INFERENCE that a
tracked artifact in its own leaf had already refuted. ⇒ **the factor is retired, not
re-computed**: it fails under every available reading (0.41× on the point estimate, 1.82× on
the most adversarial pairing), and no admissible wall-clock figure exists to rebuild it
from. `.20` acceptance (a)'s profile is what attributes fused-graph cost; this ratchet stops
structural work growing unwatched meanwhile.

**Live LR-family share `2.761`** (corpus-entry share %), derived by
`python3 stimuli/sv/corpus_parse_cost.py --rederive-family-share` into
`docs/tasks/artifacts/engine_universal_services/parse_cost_ratchet/family_share.json` and re-hashed against its four recorded inputs on every run.

⚠️ This anchor was the PAIR `2.741/8.9` until `.26` retired the second element. The share
itself is unaffected — it is a correctly measured quantity, and the two eras of it are on
record: it read **0.681 %** until `.21`, computed by a classifier that saw 97 of the
parser's 127 LR rule names. What `.26` removed is the ratio built on top of it.

## Per tier

| tier | files | entries | what it is for |
|---|---:|---:|---|
| `hot` | 40 | 348,459,214 | the heaviest files — where parse cost concentrates |
| `lr` | 40 | 62,316,155 | the heaviest guarded-admission files — the mechanism `.20` owns |
| `breadth` | 112 | 2,332,907 | a deterministic stride across the rest — so the ratchet is not blind elsewhere |

_Per-file rows: `entries.tsv` — 9 columns (sub-corpus, tier, path, accepted, entries,
committed, memo_hits, lr_entries, lr_committed), sorted by (sub-corpus, path) so two runs
diff directly. Failed speculation is deliberately NOT a column: it is exactly
`entries − committed` and is derived, never stored._
