# Status and Distance to `Done`

**One question, answered by measurement: how far is this parser from `Done`, and how much of the
corpus passes today?**

A status label — `Mostly Done`, `In Progress` — says where a family sits on a ladder. It does not
say how far it still has to climb, and a reader who only sees the label has no way to tell whether
the remaining work is a week or a year. This page publishes the **distance** behind the label.

> This page is **DERIVED BY MEASUREMENT**. The block below is produced by
> [`scripts/report_family_status.sh`](../../../scripts/report_family_status.sh), which reads the
> done-bar audit, the task-tree index and — where one is wired — the family's corpus census, rather
> than any hand-maintained note. Do not hand-edit it: re-run with `vhdl --markdown` and
> republish; `--check docs/vhdl_parser_book/src/status-and-distance.md` refuses when the two
> disagree. Every field, and the producer that answers it, is declared in
> `rust/test_data/grammar_quality/family_status_block_v0.json`.

## The three-leg bar, and what it takes to clear it

`Done` here is not a feeling about maturity. It is a three-leg bar, defined in the main book's
[Quality and Closure Model](../../book/src/quality-and-closure-model.md) and audited by
`scripts/audit_done_bar.sh`:

1. **Stimuli-generator proof** — the family's own generated samples close the loop with zero
   residual actionable-target debt.
2. **All our gates** — every gate covering the family is green *now* and is actually invoked by
   something. A gate nothing invokes is indistinguishable from a gate that does not exist.
3. **All the external test corpus, passing** — and a *triage* gate is not a conformance gate, a
   characterization is not a pass, and the **absence** of a corpus is an unmet leg, never an
   inapplicable one.

## Where `vhdl` stands

<!-- BEGIN DERIVED: scripts/report_family_status.sh vhdl --markdown -->

**Claimed status:** `Provisional (corpus pending)` · **language owner:** `external-standard` · **standard:** IEEE 1076 VHDL LRM

**Distance to `Done`: 3 of the 3 legs are not met.**

| Leg | State | Why, derived from the tracked tree |
|---|---|---|
| 1 — stimuli-generator proof | ⚠️ **UNPROVEN** | rests on a gate artifact, which this repository never tracks — run the family-status gate to establish it |
| 2 — gates green and invoked | ⛔ **UNMET** | 9 covering gate targets, 8 invoked, 1 invoked by nothing (vhdl_combined_telemetry_contract_gate); rests on a gate artifact, which this repository never tracks — run the family-status gate to establish it |
| 3 — external corpus passing | ⛔ **UNMET** | the only gate reading the external corpus is a TRIAGE gate, and a triage gate is not a conformance gate |

**Corpus.** Denominator: every .vhd / .vhdl file under the ten vendored VHDL corpora in stimuli/vhdl/subs — **13 720 files**, counted by `stimuli/run_external_corpus.sh vhdl`, published in [`stimuli/vhdl/characterization/characterization.md`](../../../stimuli/vhdl/characterization/characterization.md) and held current by `bash scripts/check_corpus_census.sh vhdl`.

| Outcome | Files | Share of the corpus |
|---|---:|---:|
| **Parses** | 4 370 | 31.9 % |
| Rejected | 9 350 | 68.1 % |
| Timed out | 0 | 0.0 % |
| Crashed | 0 | 0.0 % |
| Total | 13 720 | 100.0 % |

⛔ **This is a RAW parse rate and it moves leg 3 not at all.** none -- these are RAW parse outcomes. Many corpus files are INTENTIONALLY invalid (GHDL gna bug regressions, non-compliant fixtures), so a parse-FAIL is frequently the CORRECT outcome. A raw pass rate is NOT an adjudicated one and this producer must never render it as though a key existed; the adjudication is owed by CORPUS-GRAD-ALL.2. So there is no *known defects* count here, and that blank is not a zero: nobody has looked. The adjudication is owed by `CORPUS-GRAD-ALL.2`. What the number IS good for is drift — it is re-derived on every commit, so a grammar or corpus change that moves it can no longer pass unnoticed.

| Sub-corpus | Files | Parses | Rejected | Share |
|---|---:|---:|---:|---:|
| `Compliance-Tests` | 78 | 28 | 50 | 35.9 % |
| `Interfaces` | 30 | 11 | 19 | 36.7 % |
| `OsvvmLibraries` | 715 | 61 | 654 | 8.5 % |
| `PoC` | 371 | 84 | 287 | 22.6 % |
| `Rudi-RV32I` | 29 | 16 | 13 | 55.2 % |
| `UVVM` | 302 | 53 | 249 | 17.5 % |
| `ghdl` | 9 758 | 3 388 | 6 370 | 34.7 % |
| `neorv32` | 69 | 51 | 18 | 73.9 % |
| `nvc` | 2 074 | 594 | 1 480 | 28.6 % |
| `vunit` | 294 | 84 | 210 | 28.6 % |

**What remains between here and `Done`.** Derived from the task-tree index: the trees still OPEN whose work is attributed to this family, out of the 113 trees the index carries. **0 trees own this family's remaining work**, and **26** cross-cutting trees can gate any family.

No open tree is attributed to this family — every tree that owned its work is closed. That is not the same as meeting the bar: the legs above are what decide `Done`, and cross-cutting work can still gate this family.

⚠️ **The item here is a TREE, not a leaf, and that is a measured limit.** A leaf's status is free prose, and this repository has measured leaves reading `todo` for days after their fix shipped, so a per-leaf list would publish that defect as a number. The two prose arms — the index row and the tree file's own metadata — disagree on some trees, and every disagreement is counted OPEN and shown above: over-reporting is the only safe direction for a list that says how much is left.

<!-- END DERIVED -->

## How to read this block

The **corpus** line does not print a pass rate, and it says why rather than leaving a blank: a blank
reads as *nothing to see* and a zero reads as *done*. The reason is one of a small set of classes
derived from the done-bar register — whether the language has a third-party corpus at all, whether
one is wired here, and whether the corpus question for a subset language has been ruled on — and
where a count is genuinely owed, the block names the leaf that owes it.

The **remaining-work** line counts the task trees still open whose work is attributed to this
family, plus the cross-cutting trees that can gate any family. The item is a *tree*, not a leaf,
because a leaf's status is free prose; the block says so in place rather than implying a precision
it does not have. Where the two prose arms — the tree index and the tree file's own metadata —
disagree, the tree is counted **open** and flagged: over-reporting is the only safe direction for a
list that says how much is left.

## How this page stays true

- One producer emits both renderings — the brief form reported in the engineer's completion
  messages and the thorough block above — so the two cannot tell different stories.
- `--check` re-derives the block and diffs it against what is published here, refusing on any
  difference. Republishing is the only way to change a number on this page.
- The block is derived from the **tracked tree alone**. Legs 1 and the *green-now* half of leg 2
  rest on gate artifacts under `rust/target/`, which is never tracked, so publishing this machine's
  reading of them would make the check fail on a fresh clone. Run
  `bash scripts/audit_done_bar.sh` to see what the gate artifacts in *your* tree say.
