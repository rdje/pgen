# Answer-key PROVENANCE census — CORPUS-KEY-AUDIT.2

> DERIVED. Re-run: `python3 docs/tasks/artifacts/corpus_key_audit/key_provenance_census.py`
>
> ⛔ **THIS IS A TRUST BOUND, NOT A DEFECT COUNT.** A `tool-testimony` or
> `suite-convention` expectation is not wrong; it is *capable of being wrong in the way
> `SV-CORPUS-GRAD.13e.3` measured twice*, which a `clause-cited` one is not. The number
> below says how much of the published bar rests on evidence outside the standard.

## `sv_2017` — 7556 keyed rows

| provenance | rows | share | `must_accept` | `must_reject` |
|---|---|---|---|---|
| `clause-cited` | **426** | 5.6% | 291 | 135 |
| `tool-testimony` | **1802** | 23.8% | 1748 | 54 |
| `suite-convention` | **5326** | 70.5% | 5323 | 3 |
| `UNCLASSIFIED` | **2** | 0.0% | 1 | 1 |

⇒ **trust bound: 7130 of 7556 (94.4%)** of this lane's expectations rest on evidence OUTSIDE the standard.

## `verilog_2005` — 2534 keyed rows

| provenance | rows | share | `must_accept` | `must_reject` |
|---|---|---|---|---|
| `clause-cited` | **201** | 7.9% | 157 | 44 |
| `tool-testimony` | **485** | 19.1% | 474 | 11 |
| `suite-convention` | **1846** | 72.8% | 1846 | 0 |
| `UNCLASSIFIED` | **2** | 0.1% | 2 | 0 |

⇒ **trust bound: 2333 of 2534 (92.1%)** of this lane's expectations rest on evidence OUTSIDE the standard.

## ⛔ The BAR's own rows (274, `sv_2017`) — a different population, a different number

The published defect bar is the `sv_2017` `divergence:unexplained_*` subset, **not** the whole
key. A divergence tends to get INVESTIGATED, and investigation is exactly the event
that produces a clause cite — so the bar is measurably **better**-evidenced than the
key as a whole. Attaching the key-wide share to the bar overstates the claim.

| provenance | bar rows | share |
|---|---|---|
| `clause-cited` | 26 | 9.5% |
| `tool-testimony` | 121 | 44.2% |
| `suite-convention` | 126 | 46.0% |
| `UNCLASSIFIED` | 1 | 0.4% |

⇒ **248 of 274 (90.5%)** of the rows *behind the published bar* rest on evidence outside the standard, against **93.8%** key-wide. ⭐ Use **this** number when speaking about the BAR and the key-wide one when speaking about the ANSWER KEY: they are different populations, and the bar is the better-evidenced of the two.

## By suite — where the non-clause evidence actually lives

| suite | rows | `clause-cited` | `tool-testimony` | `suite-convention` | `UNCLASSIFIED` |
|---|---|---|---|---|---|
| `iverilog` | 3134 | 362 | 537 | 2235 | 0 |
| `verilator` | 3134 | 21 | 771 | 2342 | 0 |
| `ispras-sv-tests` | 1147 | 37 | 0 | 1107 | ⛔ 3 |
| `sv-tests` | 927 | 5 | 0 | 921 | ⛔ 1 |
| `sv2v` | 920 | 195 | 356 | 369 | 0 |
| `Surelog` | 628 | 5 | 623 | 0 | 0 |
| `verible` | 130 | 1 | 0 | 129 | 0 |
| `slang` | 70 | 1 | 0 | 69 | 0 |

## ⛔ What the classifier got wrong before it got this right

Both corrections are pinned as `--self-test` arms, because a classifier's own history is
the only evidence its current rule is not the next mistake.

| # | the rule | what it did | rows |
|---|---|---|---|
| 1 | *any mention of `IEEE 1800-2017` is a clause cite* | swept in every Surelog row, whose basis reads *"parses under **Surelog's** IEEE 1800-2017 grammar"* — an edition naming the upstream TOOL's grammar, i.e. the purest tool testimony in the corpus, classified as its opposite | **623 over-counted** |
| 2 | *a clause cite needs an `A.n.n` production* | missed rows citing a numbered clause directly — *"IEEE 1800-2017 22.8 / IEEE 1364-2005 19.2 permit it only OUTSIDE…"* | **72 under-counted** |

⇒ the rule is not *does it mention the standard* but ***does it point at a PLACE in it***.
Settled by ENUMERATING what follows every edition mention rather than guessing a third
time — 623 `grammar`, 72 a dotted clause number, 63 `clauses`/`Annex`/`production`, and
nothing else. That the partition is EXHAUSTIVE is why the split is trustworthy.

## ⛔ The residual, VERBATIM — a census that hides its leftovers is a census you cannot check

**4 of 10090 rows** (0.04%) match no rule. Every one is printed here rather than folded into a class, because a
residual absorbed into the nearest bucket is exactly how a classifier stops being
checkable. The first cut left **4 961** unclassified (49 %) and the vocabulary was
extended by ENUMERATING those shapes, not by widening a rule until the number fell.

- `ispras-sv-tests` / `must_reject` — pinned .8b.2: the committed text is syntactically broken - the a1 assert property's opening parenthesis is never closed (upstream typo); parse-level regardless of the intended 16.9.4 nested-gclk-function semantic rule
- `sv-tests` / `must_accept` — pinned: bit-select on a vectored net is a semantic (elaboration) restriction; the syntax itself parses
- `ispras-sv-tests` / `must_accept` — pinned .8c.1: 12.8.2 early defparam hierarchical-name resolution ambiguity - pure elaboration semantics; defparam and the generate block are ordinary 1364-2005 BNF
- `ispras-sv-tests` / `must_accept` — ispras-1364 POSITIVE pinned .8c.2 (KNOWN_TEXT_BUGS re-adjudicated, no errata flip): `begin_keywords "1364-2001" - 19.11 selects the 1364-2001 keyword set so 'uwire' is an ordinary identifier in scope; section 19: directives may appear anywhere in the source description (the sv-lane 22.14 begin_keywords mirror - directive-aware keyword selection is parser duty)

⚠️ **TWO of these ARE clause cites, so `clause-cited` is a LOWER bound, short by exactly 2.** They spell the clause as a bare dotted number (*"…pinned .8c.1: 12.8.2 early defparam…"*, *"…- 19.11 selects the 1364-2001 keyword set…"*), and a general bare-dotted-number rule is **deliberately not added**: `.8b.3` and `.13e.2` are task-leaf ids in the very same strings, so such a rule would classify every pinned row as clause-cited on its own bookkeeping. Two rows misfiled in the conservative direction is the cheaper error. The other two cite nothing external at all — they are the adjudicator's own reading of the file — which is a third trust profile, too small a population (2) to justify naming a class for.

## Samples, one per class — so the classification is auditable, not asserted

**`clause-cited`**

- `Surelog` / `must_reject` — pinned .3.15: bare begin/end generate_block at line 5 - IEEE 1800-2017 A.4.2 admits a generate_block only under loop_generate_construct / if_generate_
- `Surelog` / `must_reject` — pinned .3.15: bare begin/end generate_block at line 4 - IEEE 1800-2017 A.4.2 admits a generate_block only under loop_generate_construct / if_generate_
- `Surelog` / `must_reject` — pinned .3.15: bare begin/end generate_block at line 49 - IEEE 1800-2017 A.4.2 admits a generate_block only under loop_generate_construct / if_generate

**`tool-testimony`**

- `Surelog` / `must_accept` — Surelog: golden log 1364_2005.log completes with no [SNT:]/[FTL:] - upstream testimony that the unit's single source parses under Surelog's IEEE 1800-
- `Surelog` / `must_accept` — Surelog: golden log 3SigsSensList.log completes with no [SNT:]/[FTL:] - upstream testimony that the unit's single source parses under Surelog's IEEE 1
- `Surelog` / `must_accept` — Surelog: golden log AaFirstTest.log completes with no [SNT:]/[FTL:] - upstream testimony that the unit's single source parses under Surelog's IEEE 180

**`suite-convention`**

- `ispras-sv-tests` / `must_accept` — ispras: '// ! TYPE: POSITIVE' clause-keyed valid example
- `ispras-sv-tests` / `must_accept` — ispras: '// ! TYPE: POSITIVE' clause-keyed valid example
- `ispras-sv-tests` / `must_accept` — ispras: '// ! TYPE: POSITIVE' clause-keyed valid example

**`UNCLASSIFIED`**

- `ispras-sv-tests` / `must_reject` — pinned .8b.2: the committed text is syntactically broken - the a1 assert property's opening parenthesis is never closed (upstream typo); parse-level r
- `sv-tests` / `must_accept` — pinned: bit-select on a vectored net is a semantic (elaboration) restriction; the syntax itself parses
- `ispras-sv-tests` / `must_accept` — pinned .8c.1: 12.8.2 early defparam hierarchical-name resolution ambiguity - pure elaboration semantics; defparam and the generate block are ordinary 

