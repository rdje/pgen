# Status and Distance to `Done`

**One question, answered by measurement: how far is this parser from `Done`, and how much of the
corpus passes today?**

A status label — `Mostly Done`, `In Progress` — says where a family sits on a ladder. It does not
say how far it still has to climb, and a reader who only sees the label has no way to tell whether
the remaining work is a week or a year. This page publishes the **distance** behind the label.

> This page is **DERIVED BY MEASUREMENT**. The block below is produced by
> [`scripts/report_family_status.sh`](../../../scripts/report_family_status.sh), which reads the
> done-bar audit and the corpus census rather than any hand-maintained note. Do not hand-edit it —
> re-run with `systemverilog --markdown` and republish;
> `--check docs/systemverilog_parser_book/src/status-and-distance.md` refuses when the two
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

## Where SystemVerilog stands

<!-- BEGIN DERIVED: scripts/report_family_status.sh systemverilog --markdown -->

**Claimed status:** `Provisional (corpus pending)` · **language owner:** `external-standard` · **standard:** IEEE 1800 SystemVerilog LRM

**Distance to `Done`: 3 of the 3 legs are not met.**

| Leg | State | Why, derived from the tracked tree |
|---|---|---|
| 1 — stimuli-generator proof | ⚠️ **UNPROVEN** | rests on a gate artifact, which this repository never tracks — run the family-status gate to establish it |
| 2 — gates green and invoked | ⚠️ **UNPROVEN** | 17 covering gate targets, 17 invoked, 0 invoked by nothing; rests on a gate artifact, which this repository never tracks — run the family-status gate to establish it |
| 3 — external corpus passing | ⛔ **UNMET** | the external-corpus CONFORMANCE gate's own assertion is false — `full_pass` needs zero unexplained divergences on both lanes and every row adjudicated under its declared reading (the preprocess→parse pipeline for a row the raw lane defers to it; the permanently unadjudicable classes named and bounded, never counted as a gap), and rows still carry no verdict under that reading (`corpus_conformance_compare.py --assert-tracked` prints the count) |

**Corpus.** Denominator: every row of the SystemVerilog corpus adjudication manifests over stimuli/sv/subs + stimuli/sv/uvm — **16 336 rows**, counted by `stimuli/sv/corpus_verdict_coverage.py` and held current by `bash scripts/check_sv_corpus_denominator.sh`.

| Outcome | Rows | Share of the corpus |
|---|---:|---:|
| **Passing** — the parser's verdict matches the answer key | 6 265 | 38.4 % |
| Divergence with a recorded explanation | 1 418 | 8.7 % |
| **Known defects** — divergence with no explanation | 0 | 0.0 % |
| Routed to another lane's manifest | 2 943 | 18.0 % |
| **No verdict** — nothing is known about these rows | 5 710 | 35.0 % |
| Total | 16 336 | 100.0 % |

Only **7 683 rows (47.0 %)** were asked a question they could answer, and **3 821** of the no-verdict rows are *dark* — the parse failed and nobody has looked at why. ⛔ The pass share above is a share of the WHOLE corpus, not of the adjudicated part: reporting 6265/7683 would quietly drop the 35.0 % carrying no verdict at all and the 18.0 % answered in another lane's manifest.

**Under the reading leg 3 asserts** (`DONE-BAR.3f`): the table above is the RAW lane; the bar reads a row the raw lane defers to the preprocess→parse pipeline under that pipeline's re-adjudication, and names — never demands — the permanently unadjudicable classes, each held under a recorded ceiling. Derived by the register's tracked assertion (`python3 stimuli/sv/corpus_conformance_compare.py --assert-tracked stimuli/sv/characterization/adjudication_manifest.tsv --assert-tracked-v2005 stimuli/sv/characterization/adjudication_manifest_v2005.tsv`), its sv_2017 reading cross-checked against the tuple under the pipeline that `bash scripts/check_sv_corpus_denominator.sh` prints.

| Lane | Rows | Adjudicated | Permanently unadjudicable (named, bounded) | **Still awaiting a verdict** | Unexplained | Bar rows | `full_pass` |
|---|---:|---:|---:|---:|---:|---:|---|
| `sv_2017` — the corpus, raw rows read under the pipeline where it re-read them | 16 336 | 11 456 | 479 | **1 458** | 0 | 0 | `false` |
| `verilog_2005` — the routed rows, read raw under IEEE 1364-2005 | 2 943 | 2 858 | 33 | **52** | 0 | 0 | `false` |
| **Both lanes — the corpus** | 16 336 | 14 314 | 512 | **1 510** | 0 | 0 | `false` |

The three row counts sum to the denominator (14 314 + 512 + 1 510 = 16 336). **1 510 rows still stand between this corpus and leg 3** — every one in a named class with an owning leaf; the permanently unadjudicable rows stay in the raw table above as no-verdict rows and are never relabelled. The re-adjudication table is current with the raw classes (0 stale rows). The pre-`.3f` bit (`no_verdict == 0` on the raw lane, unreachable by construction) reads `false` and is kept by name as `full_pass_raw_lane`.

**What remains between here and `Done`.** Derived from the task-tree index: the trees still OPEN whose work is attributed to this family, out of the 114 trees the index carries. **4 trees own this family's remaining work**, and **26** cross-cutting trees can gate any family.

| Open tree | Status verdict | Where the detail lives |
|---|---|---|
| `SV-CORPUS-GRAD` | open | [`docs/tasks/SV-CORPUS-GRAD.md`](../../tasks/SV-CORPUS-GRAD.md) |
| `SV-EXH-PROOF` | open | [`docs/tasks/SV-EXH-PROOF.md`](../../tasks/SV-EXH-PROOF.md) |
| `SV-PARSE-STRICT` | open | [`docs/tasks/SV-PARSE-STRICT.md`](../../tasks/SV-PARSE-STRICT.md) |
| `SV-REPLAY-DEBT` | open | [`docs/tasks/SV-REPLAY-DEBT.md`](../../tasks/SV-REPLAY-DEBT.md) |

⚠️ **The item here is a TREE, not a leaf, and that is a measured limit.** A leaf's status is free prose, and this repository has measured leaves reading `todo` for days after their fix shipped, so a per-leaf list would publish that defect as a number. The two prose arms — the index row and the tree file's own metadata — disagree on some trees, and every disagreement is counted OPEN and shown above: over-reporting is the only safe direction for a list that says how much is left.

<!-- END DERIVED -->

## How to read the corpus table honestly

The denominator is every row of the SystemVerilog corpus adjudication manifests — not the subset
that happens to have been looked at. That choice is deliberate and it is the whole point of the
`SV-CORPUS-DENOMINATOR` doctrine: a defect count published without its denominator is not a claim
about the corpus. A row carrying **no verdict** is not evidence of correctness; it is not evidence
of anything, and *unknown* and *clean* are different words.

The **dark** rows are the sharpest edge in the table: the parse failed, and the row's deferral
reason is why nobody has looked at it since. Every one of them is a question the project has not
yet asked.

**What leg 3 actually demands of these rows.** The table above is the **raw lane** — every file
parsed on its own bytes. The bar's assertion does not stop there, and since `DONE-BAR.3f`
(2026-10-08) it could not honestly: three classes of no-verdict row are not parser gaps at all.

- A **chained** row (a multi-file design, `deferred:chained_only`) is deferred by the raw lane *by
  design* and read under the preprocess→parse pipeline's re-adjudication
  (`stimuli/sv/characterization/pipeline_readjudication.tsv`, the lane
  [the pipeline chapter](preprocess-parse-pipeline.md) describes). The assertion reads it there.
- A **permanently unadjudicable** row — a `$readmemh` memory image, a Verilog-AMS file, a test whose
  own suite marks the verdict implementation-dependent, a construct the upstream runner skips — can
  never carry a verdict about SystemVerilog, and the project said so in
  `SV-CORPUS-GRAD.13f`: each class states its argument and is held under a recorded ceiling the
  instruments refuse to exceed. The assertion names and counts these rows; it does not demand them,
  and it never relabels them as a pass — they stay in the table above as no-verdict rows.
- Everything else with no verdict is **burnable**: a row the project still owes a verdict.

Before `.3f` the assertion was *zero no-verdict rows on the raw lane*, which demanded a verdict for
the first two classes too — a bar no parser could meet. The counts under this reading are the
*Under the reading leg 3 asserts* table inside the derived block above (`DONE-BAR.3g`): per lane,
adjudicated + permanently unadjudicable + **still awaiting a verdict** = the denominator, with the
unexplained and bar counts and `full_pass` itself. They are derived on every render from the
register's tracked assertion (`stimuli/sv/corpus_conformance_compare.py --assert-tracked …`), the
same command the done-bar audit evaluates, and cross-checked against the tuple *under the pipeline*
that `bash scripts/check_sv_corpus_denominator.sh` prints — so the book, the audit and the
denominator doctrine cannot disagree about this reading.

## How this page stays true

- One producer emits both renderings — the brief form reported in the engineer's completion
  messages and the thorough block above — so the two cannot tell different stories.
- `--check` re-derives the block and diffs it against what is published here, refusing on any
  difference. Republishing is the only way to change a number on this page.
- The block is derived from the **tracked tree alone**. Legs 1 and the *green-now* half of leg 2
  rest on gate artifacts under `rust/target/`, which is never tracked, so publishing this
  machine's reading of them would make the check fail on a fresh clone. Run
  `bash scripts/audit_done_bar.sh` to see what the gate artifacts in *your* tree say.

## Release distance — one number, re-derived every release

**The question the director asked (2026-09-17): how far is the SV parser from its release, as a number a reader can watch move?**

The three-leg bar above says *whether* the family is `Done`; the **release distance** counts what still stands between this tree and that bar, and its
history says how fast the count has been falling. `items = legs_unmet + bar_rows + open_leaves`:

- **legs unmet** — the `Distance to `Done`` line of the derived block above (owned by `FAMILY-STATUS-CURRENCY`).
- **bar rows** — `divergence:unexplained_*` rows in the three adjudication manifests: the rows the parser must clear on `sv_2017`, `sv_2023` and `verilog_2005`
  (the bar the director's directive names — "the SV parser shall cleanly pass all of them").
- **open leaves** — `todo` / `active` leaves of `docs/tasks/SV-CORPUS-GRAD.md` plus the `todo` leaves of `docs/tasks/ENGINE-UNIVERSAL-SERVICES.md` that SV-CORPUS-GRAD
  opened (engine work the SV parser owes).

Every figure is derived from the tracked tree by [`scripts/report_sv_release_distance.sh`](../../../scripts/report_sv_release_distance.sh); nothing here runs a gate,
so a fresh clone re-derives the same block. Each parser release appends one row to the history (`--record`) — and so does a commit outside a release that moves the distance (a leaf closed or opened), under the shipped release's label, so rows can share a label and the trailing rate divides by distinct releases (`SV-CORPUS-GRAD.23`, 2026-09-24) — and the doctrine `SV-RELEASE-DISTANCE-CURRENCY`
refuses a commit whose published block or whose history's last row disagrees with the tree. The history's `commit` column names the tree a row was DERIVED from: a row written at a release's lockstep step carries the release commit's parent (the release commit does not exist yet when the row is written), a back-filled row the release commit itself. ⛔ **What the number is not.** The ETA is a trailing-rate
ESTIMATE over the last five recorded releases; a structural leg (a corpus conformance gate, the generator proof) is one item that costs more than a bar row;
the 38 % of corpus rows with no verdict are an answer-key gap outside `items`; and an item can be opened by a release as well as closed — the history shows both.

<!-- BEGIN DERIVED: scripts/report_sv_release_distance.sh --markdown -->
**SV release distance: 10 items** (`legs_unmet + bar_rows + open_leaves`).

| Component | Count | Derived from |
|---|---:|---|
| Done-bar legs unmet | 3 of 3 | the derived block above (`report_family_status.sh`) |
| Corpus bar rows (unexplained divergences) | 0 (sv_2017 0, sv_2023 0, verilog_2005 0) | `stimuli/sv/characterization/adjudication_manifest*.tsv` |
| Open leaves owning SV work | 7 (SV-CORPUS-GRAD 1, engine leaves SV opened 6) | `docs/tasks/SV-CORPUS-GRAD.md`, `docs/tasks/ENGINE-UNIVERSAL-SERVICES.md` |
| Corpus passing (sv_2017, `match` rows) | 6265 of 16336 (38.4 %) | the sv_2017 manifest |
| Known defects (unexplained, sv_2017) | 0 | the sv_2017 manifest |

**Trailing closure rate:** -0.50 items per release over the last 5 recorded release(s); **ETA ≈ n/a (no net closure over the window) releases ≈ n/a days** — an ESTIMATE that assumes the trailing rate holds; a structural leg is one item that costs more than a bar row, and the no-verdict rows are outside the count.

History: [`release_distance/history.tsv`](../../tasks/artifacts/sv_corpus_grad/release_distance/history.tsv) (one row per release, appended by `--record` — and one more, under the shipped release's label, when a commit outside a release moves the distance; the rate above divides by distinct releases; the last row must equal this tree, checked by `--check`).
<!-- END DERIVED: scripts/report_sv_release_distance.sh -->
