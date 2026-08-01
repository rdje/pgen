---
name: project_acceptance_box_must_be_written_by_the_change
description: "DOCTRINE FIX (2026-08-01, `GENERATED-LINT-CORRECTNESS.7`) — `.3` scoped the SIGNATURE to its own box but never scoped the BOX to the change, so ANY ticked box in ANY staged task file satisfied a requirement: for every tree already holding one compliant leaf, box-scoping was VACUOUS. A satisfying box must now sit in a LEAF SECTION the change touches, and all three must live in ONE file. Also settles that the chartered SIXTH signature family is REFUSED at 0–3 of 307 (0%)."
metadata:
  node_type: memory
  type: project
id: project-acceptance-box-must-be-written-by-the-change
title: An acceptance box only counts if THIS change wrote it — box-scoping without leaf-scoping is vacuous
date: 2026-08-01
answers:
  - "why did a code change pass the acceptance checklist without writing one"
  - "can an old completed leaf supply the checklist for a new leaf in the same task file"
  - "is a sixth diagnosis-signature family needed for the generator/coverage instruments"
  - "what does box-scoped evidence actually guarantee, and what does it not"
reverify: bash docs/tasks/artifacts/generated_lint_correctness/run_diag_evidence_leaf_scope_probes.sh
---

**What this record settles.** Two questions `GENERATED-LINT-CORRECTNESS.7` was chartered to
answer, under a director approval that was explicitly **conditional** — *"Fix the enforcer
carefully … **if you find a misbehavior in the enforcer**"*. One answer is a refusal; the other is
a fix.

## 1. The SIXTH signature family is REFUSED — priced at 0 % of the corpus

`SV-EXH-PROOF.7.4.6.9` root-caused 79 targets with the per-branch `failure_reasons` record, an
instrument `TOOLBOX.md` §6.1 calls *"the FIRST thing to read"* on a residual-coverage question —
and no `DIAGNOSIS_SIG` family names it. That is a real expressiveness gap, and it is still not a
family, because it is not a **population**. Priced against every ticked ROOT CAUSE box:

| candidate sixth family | newly admits |
|---|---|
| `failure_reasons` / `top_failure_reasons` / `reachable_branch_debt` / `branch_groups` | **0 of 307 (0 %)** |
| + `--dump-gen-ast`, `--gap-report-*`, `--report-k-path-coverage`, `depth_exceeded` … | 1 of 307 (0 %) |
| widest plausible generator/coverage vocabulary | 3 of 307 (0 %) |

`.4` refused its own chartered family at **2 of 304 (0.7 %)**. This one prices *below* that. ⇒ a
family added here would be a gate widened to let content through — the same move as raising a cap
to land content. **Refused, and the refusal is the result.**

The corpus shape that `.4` measured is unchanged and sharper: of 307 unbacked boxes, **299 (97 %)
are OUT-OF-BOX** (the leaf holds tool evidence, outside the ticked bullet) and only **8 (2 %)** are
NO-EVIDENCE. It is a placement gap, and no token set closes a placement gap.

## 2. ⭐⭐ The misbehaviour: box-scoping was VACUOUS for any tree with one finished leaf

`.3` required the signature to sit inside the ticked box's own bullet. It never required the *box*
to belong to the change. `box_matches` returned true when **any** ticked box in **any** staged
`docs/tasks/*.md` satisfied a requirement — so a task tree that already holds one compliant leaf
supplies the checklist for every later leaf, forever, at no cost.

Reproduced, not argued (`run_diag_evidence_leaf_scope_probes.sh`):

| probe | pre-`.7` |
|---|---|
| **H1** a new leaf with NO checklist, in the same file as one old completed leaf | **PASSED** |
| **H2** the owning leaf in file B, backed boxes only in an unrelated co-staged file A | **PASSED** |
| **H3** the three boxes satisfied only across two different files | **PASSED** |

⇒ `.3`'s record claims cross-FILE leakage was closed. It was closed only for a signature leaking
*into* a box; a whole borrowed **box** still crossed files freely. **33 of the tracked task files
carry ≥1 backed ROOT CAUSE box**, i.e. a standing free pass — and, in the sharpest available
statement of the problem, **this leaf's own commit would have satisfied box 1 on four historical
boxes (`.2`/`.3`/`.4`/`.5`) without writing a single line of checklist.**

Replayed over the last 400 commits: **138 code-change commits pass today, and 7 of them pass ONLY
by borrowing** — 5 whose own NO REGRESSION box carries no gate signature, and 2 that wrote no
qualifying box at all. All 7 were read individually; every one is a genuine instance of the
defect, so the strengthening has **0 false positives** on the measured corpus.

## The rule that shipped

A box may satisfy a requirement only if it sits inside a **leaf section** (bounded by markdown
headings of level ≤ 3, so a `#### Acceptance Checklist` block belongs to its `###` leaf) that the
staged change **touches**, and all three requirements must be met within **one** file.

Deliberately permissive *inside* a leaf: a follow-up commit editing any part of the same leaf
keeps its checklist (GREEN-2), and a deletion-only edit still counts as touching it (GREEN-3).
What it stops is the cross-leaf and cross-file borrow.

⚠️ **Trailing blank lines are trimmed off a section, and that is load-bearing.** A new leaf is
APPENDED, and an append begins with the blank separator line that syntactically still belongs to
the previous section — so without the trim, writing a brand-new leaf with no checklist "touches"
the finished leaf above it and inherits its boxes. The first implementation had exactly that hole
and **RED-H1 caught the rule being vacuous against the most common edit in the repository.**

## ⚠️ Honest bounds (stated, not discovered later)

1. **The enforcer cannot know which leaf OWNS a change.** *"Written by this commit"* is a proxy,
   not a proof: editing inside an unrelated leaf's own checklist section still satisfies it.
2. **`unchecked` stays file-wide and section-blind** — an unticked box says the step is
   unfinished, and that verdict must not depend on whether the commit happened to edit that part
   of the file. Its cross-leaf false-block hazard was measured at **population 0** (no tracked
   task file carries an unticked ROOT-CAUSE box) and deliberately left alone.
3. One further candidate misbehaviour was **disproven on evidence, not assumed away**: the
   `box_body` `^#` terminator hiding a signature inside a fenced block (**0 boxes** affected
   repo-wide). A second — `ROOT_KW`'s bare `\bwhy\b` over-match (**4** headers, all `**FIX**` /
   `**LOCKSTEP**` / `**REPRODUCE**` boxes, **none carrying a signature**) — was recorded here as a
   latent fails-open widening and routed rather than fixed. ⭐ **It has since been FIXED**
   (`GENERATED-LINT-CORRECTNESS.9`, `-0011`: `ROOT_KW='root cause|why ?[-+/&] ?where'`, priced at
   4 boxes dropped / **0 backed** / 0 files losing their last backed box). ⚠️ **And the reason it
   should not have been left latent is worth keeping**: *"none carries a signature today"* is a
   fact about the corpus at one instant, not about the rule — the reach is one `**FIX**` box
   quoting one command away. ⭐ A probe then found the **same alternative failing CLOSED too**: an
   *unticked* `**FIX**` box writing the word "why" tripped bound 2's `unchecked` and blocked a leaf
   whose real boxes were all ticked and backed. Bound 2's "population 0" measurement was true and
   did not cover that shape.
4. **Routed, not fixed → `GENERATED-LINT-CORRECTNESS.8`:** `NOREGRESS_SIG` has the same
   wrong-vocabulary defect group 2 had. It names only parser-side global gates, so an
   ops/build-flow change has no natural no-regression token and **120 of 416 (29 %)** NO
   REGRESSION boxes are unbacked. That gap is *why* the borrow hole went unnoticed — it was
   silently absorbing it. An honest idiom exists today (*"no grammar/rust/generated touched ⇒ all
   11 parsers byte-identical BY CONSTRUCTION"*, which `.4` itself used), so closing the borrow
   does not force a waiver; the vocabulary still deserves its own priced leaf per
   `DOCTRINE-GAP-OWNERSHIP`.

## Consequences

- `scripts/check_diagnosis_evidence.sh` gains `added_ranges` + `section_touched`, and
  `box_matches` becomes `box_matches_in_file` with separable present / signature-backed /
  change-owned stages, so a breach is reported at the stage it actually failed.
- The census instrument `run_root_cause_box_census.sh` gains **ground-truth controls that REFUSE
  (exit 2) on a miss** — it had published `.4`'s headline numbers with nothing pinning its box
  arithmetic. The controls pin the four rules that were each, at some point, actually wrong; both
  were proven to fire by mutation before the numbers here were trusted
  ([[feedback_instrument_needs_ground_truth]]).
- Probes: **9/9 after, 6/9 before** — the three RED arms pass the gate pre-`.7` (the hole) while
  every GREEN and CONTROL arm is byte-identical on both sides (the non-weakening proof). `.3`'s
  6-arm and `.4`'s 13-arm drivers re-run **6/6** and **13/13**.

Related: [[project_codegen_emission_root_cause_signature]],
[[project_ops_build_flow_root_cause_signature]], [[feedback_instrument_needs_ground_truth]],
[[project_waiver_is_a_gate_bug_report]], [[feedback_read_prior_art_before_designing]].
