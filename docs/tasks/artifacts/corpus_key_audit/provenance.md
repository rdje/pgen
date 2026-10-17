# Answer-key PROVENANCE census — CORPUS-KEY-AUDIT.2

> DERIVED. Re-run: `python3 docs/tasks/artifacts/corpus_key_audit/key_provenance_census.py`
>
> ⛔ **THIS IS A TRUST BOUND, NOT A DEFECT COUNT.** A `tool-testimony` or
> `suite-convention` expectation is not wrong; it is *capable of being wrong in the way
> `SV-CORPUS-GRAD.13e.3` measured twice*, which a `clause-cited` one is not. The number
> below says how much of the published bar rests on evidence outside the standard.

## `sv_2017` — 7733 keyed rows

| provenance | rows | share | `must_accept` | `must_reject` |
|---|---|---|---|---|
| `clause-cited` | **639** | 8.3% | 310 | 329 |
| `tool-testimony` | **1741** | 22.5% | 1692 | 49 |
| `suite-convention` | **5352** | 69.2% | 5349 | 3 |
| `UNCLASSIFIED` | **1** | 0.0% | 0 | 1 |

⇒ **trust bound: 7094 of 7733 (91.7%)** of this lane's expectations rest on evidence OUTSIDE the standard.

## `verilog_2005` — 2896 keyed rows

| provenance | rows | share | `must_accept` | `must_reject` |
|---|---|---|---|---|
| `clause-cited` | **309** | 10.7% | 158 | 151 |
| `tool-testimony` | **479** | 16.5% | 468 | 11 |
| `suite-convention` | **2107** | 72.8% | 2107 | 0 |
| `UNCLASSIFIED` | **1** | 0.0% | 1 | 0 |

⇒ **trust bound: 2587 of 2896 (89.3%)** of this lane's expectations rest on evidence OUTSIDE the standard.

## By suite — where the non-clause evidence actually lives

| suite | rows | `clause-cited` | `tool-testimony` | `suite-convention` | `UNCLASSIFIED` |
|---|---|---|---|---|---|
| `iverilog` | 3742 | 498 | 559 | 2685 | 0 |
| `verilator` | 3068 | 125 | 717 | 2226 | 0 |
| `ispras-sv-tests` | 1146 | 38 | 0 | 1106 | ⛔ 2 |
| `sv-tests` | 933 | 18 | 0 | 915 | 0 |
| `sv2v` | 920 | 238 | 335 | 347 | 0 |
| `Surelog` | 629 | 20 | 609 | 0 | 0 |
| `verible` | 128 | 9 | 0 | 119 | 0 |
| `slang` | 63 | 2 | 0 | 61 | 0 |

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

**2 of 10629 rows** (0.02%) match no rule. Every one is printed here rather than folded into a class, because a
residual absorbed into the nearest bucket is exactly how a classifier stops being
checkable. The first cut left **4 961** unclassified (49 %) and the vocabulary was
extended by ENUMERATING those shapes, not by widening a rule until the number fell.

- `ispras-sv-tests` / `must_reject` — pinned .8b.2: the committed text is syntactically broken - the a1 assert property's opening parenthesis is never closed (upstream typo); parse-level regardless of the intended 16.9.4 nested-gclk-function semantic rule
- `ispras-sv-tests` / `must_accept` — pinned .8c.1: 12.8.2 early defparam hierarchical-name resolution ambiguity - pure elaboration semantics; defparam and the generate block are ordinary 1364-2005 BNF

⚠️ **TWO of these ARE clause cites, so `clause-cited` is a LOWER bound, short by exactly 2.** They spell the clause as a bare dotted number (*"…pinned .8c.1: 12.8.2 early defparam…"*, *"…- 19.11 selects the 1364-2001 keyword set…"*), and a general bare-dotted-number rule is **deliberately not added**: `.8b.3` and `.13e.2` are task-leaf ids in the very same strings, so such a rule would classify every pinned row as clause-cited on its own bookkeeping. Two rows misfiled in the conservative direction is the cheaper error. The other two cite nothing external at all — they are the adjudicator's own reading of the file — which is a third trust profile, too small a population (2) to justify naming a class for.

## Samples, one per class — so the classification is auditable, not asserted

**`clause-cited`**

- `Surelog` / `must_accept` — Surelog: golden log 1364_2005.log completes with no [SNT:]/[FTL:] - upstream testimony that the unit's single source parses under Surelog's IEEE 1800-
- `Surelog` / `must_reject` — pinned .3.52: line 4 `parameter type AT;` - a class-body TYPE parameter without a default. IEEE 1800-2017 Annex A footnote 18: "It shall be legal to o
- `Surelog` / `must_reject` — pinned .3.15: bare begin/end generate_block at line 5 - IEEE 1800-2017 A.4.2 admits a generate_block only under loop_generate_construct / if_generate_

**`tool-testimony`**

- `Surelog` / `must_accept` — Surelog: golden log 3SigsSensList.log completes with no [SNT:]/[FTL:] - upstream testimony that the unit's single source parses under Surelog's IEEE 1
- `Surelog` / `must_accept` — Surelog: golden log AaFirstTest.log completes with no [SNT:]/[FTL:] - upstream testimony that the unit's single source parses under Surelog's IEEE 180
- `Surelog` / `must_accept` — Surelog: golden log AllPackageSignal.log completes with no [SNT:]/[FTL:] - upstream testimony that the unit's single source parses under Surelog's IEE

**`suite-convention`**

- `ispras-sv-tests` / `must_accept` — ispras: '// ! TYPE: POSITIVE' clause-keyed valid example
- `ispras-sv-tests` / `must_accept` — ispras: '// ! TYPE: POSITIVE' clause-keyed valid example
- `ispras-sv-tests` / `must_accept` — ispras: '// ! TYPE: POSITIVE' clause-keyed valid example

**`UNCLASSIFIED`**

- `ispras-sv-tests` / `must_reject` — pinned .8b.2: the committed text is syntactically broken - the a1 assert property's opening parenthesis is never closed (upstream typo); parse-level r
- `ispras-sv-tests` / `must_accept` — pinned .8c.1: 12.8.2 early defparam hierarchical-name resolution ambiguity - pure elaboration semantics; defparam and the generate block are ordinary 

