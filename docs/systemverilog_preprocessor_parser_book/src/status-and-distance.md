# Status and Distance to `Done`

**One question, answered by measurement: how far is this parser from `Done`, and how much of the
corpus passes today?**

A status label — `Mostly Done`, `In Progress` — says where a family sits on a ladder. It does not
say how far it still has to climb, and a reader who only sees the label has no way to tell whether
the remaining work is a week or a year. This page publishes the **distance** behind the label.

> This page is **DERIVED BY MEASUREMENT**. The block below is produced by
> [`scripts/report_family_status.sh`](../../../scripts/report_family_status.sh), which reads the
> done-bar audit, the task-tree index and — where one is wired — the family's corpus census, rather
> than any hand-maintained note. Do not hand-edit it: re-run with `systemverilog_preprocessor --markdown` and
> republish; `--check docs/systemverilog_preprocessor_parser_book/src/status-and-distance.md` refuses when the two
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

## Where `systemverilog_preprocessor` stands

<!-- BEGIN DERIVED: scripts/report_family_status.sh systemverilog_preprocessor --markdown -->

**Claimed status:** `Provisional (corpus pending)` · **language owner:** `external-standard` · **standard:** IEEE 1800 SystemVerilog LRM, clause 22 (compiler directives)

**Distance to `Done`: 3 of the 3 legs are not met.**

| Leg | State | Why, derived from the tracked tree |
|---|---|---|
| 1 — stimuli-generator proof | ⚠️ **UNPROVEN** | rests on a gate artifact, which this repository never tracks — run the family-status gate to establish it |
| 2 — gates green and invoked | ⛔ **UNMET** | 9 covering gate targets, 7 invoked, 2 invoked by nothing (sv_preprocessor_curated_differential_gate, sv_preprocessor_template_differential_gate); rests on a gate artifact, which this repository never tracks — run the family-status gate to establish it |
| 3 — external corpus passing | ⛔ **UNMET** | no external corpus is wired, though the language is standardized |

**Corpus.** ⛔ **NOT MEASURED** — the language is standardized, so a corpus exists in the world, but none is wired into this repository. An UNMET leg, never an inapplicable one. Owed by `CORPUS-GRAD-ALL.1`.

**What remains between here and `Done`.** Derived from the task-tree index: the trees still OPEN whose work is attributed to this family, out of the 114 trees the index carries. **1 tree own this family's remaining work**, and **26** cross-cutting trees can gate any family.

| Open tree | Status verdict | Where the detail lives |
|---|---|---|
| `SVPP-EXPANSION` | open | [`docs/tasks/SVPP-EXPANSION.md`](../../tasks/SVPP-EXPANSION.md) |

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

## The expansion stage is a sibling deliverable

The parser this book documents reads compiler directives into an AST. Executing them — expanding a file
so the SystemVerilog parser reads text with IEEE 1800-2017 clause 22 already applied — is the
**expansion stage** (`rust/src/sv_preprocessor.rs`), a sibling deliverable owned by the `SVPP-EXPANSION`
tree listed above, not a leg of this family's `Done` bar. Its contract is the SV integration contract's
*"Front End — the preprocess→parse pipeline"* section; its live measurement — the corpus parsed through
`parseability_probe --preprocess` under a derived context and library, read twice against the raw lane —
is the SV parser book's chapter
[*The preprocess→parse pipeline*](../../systemverilog_parser_book/src/preprocess-parse-pipeline.md).
The corpus line above stays NOT MEASURED for exactly that reason: no corpus of directive *files* is wired;
the expansion's corpus is the SystemVerilog family's, and it is measured there.
