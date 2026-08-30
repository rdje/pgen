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

**Corpus.** ⛔ **NOT MEASURED** — the register declares external corpus roots, but no TRACKED census turns them into counts a reader can re-derive. Owed by `DONE-BAR-VISIBILITY.3b`. The last measurement lives in `CORPUS-GRAD-ALL.2.1` and is POINTED AT rather than copied here, because a copied number rots. A raw parse census was run on 2026-08-08 -- 13 720 files, pass 4 031, fail 9 689 -- but `results.tsv` is gitignored (the rule `stimuli/*/characterization/results.tsv`), so nothing in the tree can re-derive it and it is not published here. It is also a RAW pass rate, not an adjudicated one: no answer key says which of those files SHOULD parse.

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
