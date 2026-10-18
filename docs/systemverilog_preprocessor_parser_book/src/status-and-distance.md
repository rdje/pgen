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
| 2 — gates green and invoked | ⚠️ **UNPROVEN** | 9 covering gate targets, 9 invoked, 0 invoked by nothing; rests on a gate artifact, which this repository never tracks — run the family-status gate to establish it |
| 3 — external corpus passing | ⛔ **UNMET** | no external corpus is wired, though the language is standardized |

**Corpus.** ⛔ **NOT MEASURED** — the language is standardized, so a corpus exists in the world, but none is wired into this repository. An UNMET leg, never an inapplicable one. Owed by `CORPUS-GRAD-ALL.1`.

**Under the reading leg 3 asserts:** ⛔ **NOT MEASURED** — the done-bar register declares no tracked leg-3 assertion for this family (`leg3_surface.tracked_assertion_command`), so there is no declared reading to count under; owed by `DONE-BAR.3`.

**What remains between here and `Done`.** Derived from the task-tree index: the trees still OPEN whose work is attributed to this family, out of the 115 trees the index carries. **1 tree own this family's remaining work**, and **26** cross-cutting trees can gate any family.

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
**expansion stage** (`rust/src/families/systemverilog/preprocessor.rs`, the SystemVerilog family's own component behind the engine's neutral preprocessing stage since `ENGINE-UNIVERSAL-SERVICES.113`), a sibling deliverable owned by the `SVPP-EXPANSION`
tree listed above, not a leg of this family's `Done` bar. Its contract is the SV integration contract's
*"Front End — the preprocess→parse pipeline"* section; its live measurement — the corpus parsed through
`parseability_probe --preprocess` under a derived context and library, read twice against the raw lane —
is the SV parser book's chapter
[*The preprocess→parse pipeline*](../../systemverilog_parser_book/src/preprocess-parse-pipeline.md).
The corpus line above stays NOT MEASURED for exactly that reason: no corpus of directive *files* is wired;
the expansion's corpus is the SystemVerilog family's, and it is measured there. The next section is the
first reading of THIS parser over that corpus; it is not yet the family's leg-3 assertion (wiring it is
`SV-CORPUS-GRAD.13c.2v.10`).

## The SystemVerilog corpus, read by this parser (2026-10-17)

Every file a SystemVerilog tool compiles is input to the preprocessor, whatever its SystemVerilog verdict.
So the corpus the SystemVerilog lanes read — 16,336 files, the path column of
`stimuli/sv/characterization/results.tsv` — asks this parser one question per file: does the directive
structure parse? The census
(`docs/tasks/artifacts/sv_corpus_grad/svpp_corpus_census/census.py`) runs the **generated** parser inside the
release `parseability_probe` (`--parse systemverilog_preprocessor`), never the interpreter, and classes every
rejection by the backtick word that opens the line where the parse stopped.

**Release `1.0.7`: 4,477 of the 16,336 files are rejected** (`census.md` beside the script):

| what opens the line where the parse stops | files |
|---|---:|
| a text macro usage (§22.5.1) — `` `uvm_object_utils(c) ``, `` `ASSERT(…) `` | 4,182 |
| a clause-22 directive the grammar does not derive — `` `begin_keywords `` 93, `` `resetall `` 69, `` `line ``, `` `pragma ``, … | 243 |
| no backtick word (a string or comment inside macro text, a one-line conditional, …) | 52 |

The dominant class is a grammar gap, not a corpus property: a passthrough line could not *begin* with a
backtick, so any line that opens with a macro usage — most UVM and OpenTitan code — was refused. Five more
defects surfaced beside it, two of them wrong trees for input the parser accepted:

- a backslash-continued `` `define `` body was cut at its first line (`"a + \"`, the continuation read as a
  separate line) — 38 corpus files;
- `` `define M (x) y `` was read as a function-like macro; §22.5.1 makes it object-like, because the
  parenthesis does not follow the name immediately;
- `` `include "a.sv" x `` was accepted, where §22.4 allows only white space or a comment on the line;
- `` `\`" ``, and a backtick inside a plain string or a `//` comment in macro text, were refused;
- `` `include `home(myfile) `` was refused, which §22.5.1 licenses in both editions.

All of them are fixed together in release `1.0.8` (task leaves `SV-CORPUS-GRAD.13c.2v.1` – `.7`). **Release `1.0.8`,
measured with the same census on the generated parser, against the `1.0.7` probe:**

| reading | files |
|---|---:|
| rejected at `1.0.7`, accepted at `1.0.8` | 4,413 |
| rejected by both | 64 |
| accepted by both | 11,859 |
| accepted at `1.0.7`, rejected at `1.0.8` | **0** |

Of the 11,859 files both releases accept, **11,790** produce a byte-identical tree; the **69** whose tree moved are each
explained — 38 by the continuation fix (the `1.0.7` define body ended in a `\`), 31 by the string-literal atom (a plain
string in macro text is no longer split at its punctuation). The 64 files `1.0.8` still rejects are each read and
written down in `residual_dispositions.tsv`, which the census refuses to run without:

| disposition | files | what it means |
|---|---:|---|
| the suite's own error test | 34 | sv2v `test/error/*`, sv-tests `*-illegal` / `*-invalid`, verilator `*_bad`, iverilog `CE` — refused at the construct the suite names |
| refused by a clause | 16 | a Verilog-AMS `begin_keywords` specifier (§22.14), a directive spread over lines (IEEE 1800-2023 §22.2: *"compiler directives shall be all on one line"*), a directive with a missing argument, a `pragma` whose expressions lack Syntax 22-8's comma |
| valid text the parser must accept — ordered fixed | 12 | IEEE 1800-2023 directive forms (`` `ifdef ( expr ) ``, a macro inside `` `line `` / `` `timescale `` — `.13c.2v.9`), an escaped-identifier macro name (`.8`), a conditional on one line (`.10`) |
| a malformed directive inside a skipped group | 2 | §22.6 asks only lexical validity there: the directive must be SKIPPED, not rejected — a gap ordered fixed by the director (2026-10-18), `.10` |

The census is not yet this family's leg-3 assertion (the derived block above still reads NOT MEASURED); wiring it is
`SV-CORPUS-GRAD.13c.2v.10`.

### Two wrong verdicts the census could not see (found 2026-10-18)

The census above reads the files this parser REFUSES. A file it wrongly ACCEPTS never reaches the disposition table, and
designing the next release found two such defects in `1.0.8`:

| defect | what `1.0.8` does | measured |
|---|---|---|
| `SVPP-0010` — a block comment spanning lines is not opaque | reads a directive keyword on a line inside the comment as a directive | 4 corpus files accepted with a wrong tree; Surelog `tests/PPComment/dut.sv` gets an `else_branch` its source comments out |
| `SVPP-0011` — a passthrough line is not lexed | accepts a string literal left open at its line end | 7 suite error tests accepted (sv2v `test/error/string_literal_eof.sv`, verilator `t_parse_eof_str_bad.v`, …) |

An example of the first — the release's tree for this input has an `else` branch:

```systemverilog
`define USE_BOOTMODE_TEST
`ifdef USE_BOOTMODE_TEST
   assign c= d;/*
`else
   assign c= 1'b0;*/
`endif
```

IEEE 1800-2023 §22.2: *"Directives are not recognized within comments or string literals"* — the `` `else `` is comment text,
so the conditional has one branch. Both defects are fixed by the next release's lexically-aware text run (task leaf
`SV-CORPUS-GRAD.13c.2v.13`), and that release's census reads the other half too: every row the SystemVerilog answer key
expects to be refused, joined to this parser's verdict.
