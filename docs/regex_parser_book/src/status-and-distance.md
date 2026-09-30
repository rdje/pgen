# Status and Distance to `Done`

**One question, answered by measurement: how far is this parser from `Done`, and how much of the
corpus passes today?**

A status label — `Mostly Done`, `In Progress` — says where a family sits on a ladder. It does not
say how far it still has to climb, and a reader who only sees the label has no way to tell whether
the remaining work is a week or a year. This page publishes the **distance** behind the label.

> This page is **DERIVED BY MEASUREMENT**. The block below is produced by
> [`scripts/report_family_status.sh`](../../../scripts/report_family_status.sh), which reads the
> done-bar audit, the task-tree index and — where one is wired — the family's corpus census, rather
> than any hand-maintained note. Do not hand-edit it: re-run with `regex --markdown` and
> republish; `--check docs/regex_parser_book/src/status-and-distance.md` refuses when the two
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

## Where `regex` stands

<!-- BEGIN DERIVED: scripts/report_family_status.sh regex --markdown -->

**Claimed status:** `Provisional (corpus pending)` · **language owner:** `external-standard` · **standard:** PCRE2 (recognized reference implementation, vendored under regex_corpus_bundle/third_party/upstream/pcre2)

**Distance to `Done`: 3 of the 3 legs are not met.**

| Leg | State | Why, derived from the tracked tree |
|---|---|---|
| 1 — stimuli-generator proof | ⚠️ **UNPROVEN** | rests on a gate artifact, which this repository never tracks — run the family-status gate to establish it |
| 2 — gates green and invoked | ⛔ **UNMET** | 14 covering gate targets, 7 invoked, 7 invoked by nothing (regex_ast_shape_contract_gate, regex_combined_telemetry_contract_gate, regex_corpus_bundle_contract_gate, regex_embedded_code_block_contract_gate, regex_pcre2_compile_oracle_gate, regex_pcre2_compile_perf_gate, regex_pcre2_textsafe_corpus_gate); rests on a gate artifact, which this repository never tracks — run the family-status gate to establish it |
| 3 — external corpus passing | ⛔ **UNMET** | the corpus gate reads no declared external corpus root |

**Corpus.** Denominator: every pattern spec pcre2test 10.47 declares in testdata/testinput2 that the normalizer can pair soundly with its recorded verdict — **2 189 cells** out of 2 239 pattern specs detected, counted by `regex_corpus_bundle/scripts/normalize_pcre2_compile_oracle.py` and held current by `bash scripts/check_corpus_census.sh regex`.

| Outcome | Cells | Share of the corpus |
|---|---:|---:|
| **Matches the oracle** — PGEN's compile verdict equals PCRE2's | 1 879 | 85.8 % |
| **False accepts** — PGEN accepts a pattern PCRE2 rejects | 262 | 12.0 % |
| False rejects, every one an explained divergence | 48 | 2.2 % |
| Total | 2 189 | 100.0 % |

⭐ **Unlike the VHDL census, this pass share IS a conformance share**: PCRE2 10.47's own recorded compile verdict for each pattern -- a REAL answer key, so unlike the VHDL census this pass share IS a conformance share. 50 of the 2 239 detected pattern specs are DECLINED rather than answered (48 carry a suffix modifier under which the raw pattern text is not what pcre2_compile sees, 2 fail UTF-8 decoding); they are counted in the denominator and named, never dropped.

**The declined 50 are named, not dropped** — 48 `unsupported_suffix_token`, 2 `utf8_decode_failure`. They are counted in the 2 239-spec denominator above, because a case the census refused to ask is not a case that passed.

⚠️ **The verdict half is anchor-held, not re-executed on every commit.** Producing it needs a cargo build of the corpus probe and a run over every case — measured 80 s and 5.5 GB peak, via `rust/scripts/regex_pcre2_compile_oracle_gate.sh` — so `bash scripts/check_corpus_census.sh regex` re-derives the DENOMINATOR from the normalizer on every commit and holds the published tuple's cell count equal to it, while `REGEX-ORACLE-ANCHOR-SYNC` holds every live anchor equal to each other and to the tracked ratchet bounds.

**What remains between here and `Done`.** Derived from the task-tree index: the trees still OPEN whose work is attributed to this family, out of the 114 trees the index carries. **5 trees own this family's remaining work**, and **26** cross-cutting trees can gate any family.

| Open tree | Status verdict | Where the detail lives |
|---|---|---|
| `REGEX-CODE-HOOKS` | open | [`docs/tasks/REGEX-CODE-HOOKS.md`](../../tasks/REGEX-CODE-HOOKS.md) |
| `REGEX-PCRE2-FIDELITY` | open | [`docs/tasks/REGEX-PCRE2-FIDELITY.md`](../../tasks/REGEX-PCRE2-FIDELITY.md) |
| `RGX-0078` | open | [`docs/tasks/RGX-0078.md`](../../tasks/RGX-0078.md) |
| `RGX-0090` | ⚠️ DISPUTED — the index reads CLOSED, the tree file reads OPEN; counted OPEN | [`docs/tasks/RGX-0090.md`](../../tasks/RGX-0090.md) |
| `RGX-0091` | ⚠️ DISPUTED — the index reads CLOSED, the tree file reads OPEN; counted OPEN | [`docs/tasks/RGX-0091.md`](../../tasks/RGX-0091.md) |

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
