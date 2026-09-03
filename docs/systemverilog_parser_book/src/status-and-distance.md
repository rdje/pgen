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

**Claimed status:** `Mostly Done` · **language owner:** `external-standard` · **standard:** IEEE 1800 SystemVerilog LRM

**Distance to `Done`: 3 of the 3 legs are not met.**

| Leg | State | Why, derived from the tracked tree |
|---|---|---|
| 1 — stimuli-generator proof | ⚠️ **UNPROVEN** | rests on a gate artifact, which this repository never tracks — run the family-status gate to establish it |
| 2 — gates green and invoked | ⛔ **UNMET** | 17 covering gate targets, 16 invoked, 1 invoked by nothing (sv_combined_telemetry_contract_gate); rests on a gate artifact, which this repository never tracks — run the family-status gate to establish it |
| 3 — external corpus passing | ⛔ **UNMET** | the only gate reading the external corpus is a TRIAGE gate, and a triage gate is not a conformance gate |

**Corpus.** Denominator: every row of the SystemVerilog corpus adjudication manifests over stimuli/sv/subs + stimuli/sv/uvm — **16 336 rows**, counted by `stimuli/sv/corpus_verdict_coverage.py` and held current by `bash scripts/check_sv_corpus_denominator.sh`.

| Outcome | Rows | Share of the corpus |
|---|---:|---:|
| **Passing** — the parser's verdict matches the answer key | 5 902 | 36.1 % |
| Divergence with a recorded explanation | 1 493 | 9.1 % |
| **Known defects** — divergence with no explanation | 161 | 1.0 % |
| Routed to another lane's manifest | 2 606 | 16.0 % |
| **No verdict** — nothing is known about these rows | 6 174 | 37.8 % |
| Total | 16 336 | 100.0 % |

Only **7 556 rows (46.3 %)** were asked a question they could answer, and **4 010** of the no-verdict rows are *dark* — the parse failed and nobody has looked at why. ⛔ The pass share above is a share of the WHOLE corpus, not of the adjudicated part: reporting 5902/7556 would quietly drop the 37.8 % carrying no verdict at all and the 16.0 % answered in another lane's manifest.

**What remains between here and `Done`.** Derived from the task-tree index: the trees still OPEN whose work is attributed to this family, out of the 113 trees the index carries. **4 trees own this family's remaining work**, and **26** cross-cutting trees can gate any family.

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

## How this page stays true

- One producer emits both renderings — the brief form reported in the engineer's completion
  messages and the thorough block above — so the two cannot tell different stories.
- `--check` re-derives the block and diffs it against what is published here, refusing on any
  difference. Republishing is the only way to change a number on this page.
- The block is derived from the **tracked tree alone**. Legs 1 and the *green-now* half of leg 2
  rest on gate artifacts under `rust/target/`, which is never tracked, so publishing this
  machine's reading of them would make the check fail on a fresh clone. Run
  `bash scripts/audit_done_bar.sh` to see what the gate artifacts in *your* tree say.
