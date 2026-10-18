---
id: the-preprocessor-parser-is-measured-over-the-systemverilog-corpus
title: The preprocessor parser is measured over the SystemVerilog corpus — every SV corpus file is a preprocessor input (`SV-CORPUS-GRAD.13c.2v.2`)
answers:
  - "how well does the systemverilog_preprocessor parser read real code"
  - "why did the svpp parser reject 4,477 corpus files at release 1.0.7"
  - "what are the 64 files the svpp parser still rejects at release 1.0.8"
  - "how is an svpp grammar change prototyped before generating the parser"
  - "what does SVPP-CORPUS-CENSUS measure"
tags: [systemverilog_preprocessor, corpus, census, release, clause-22]
date: 2026-10-17
provenance: SV-CORPUS-GRAD.13c.2v.2 (PGEN-SV-CORPUS-GRAD-0457 opened it, -0458 the release 1.0.8); .13c.2v.1 / .3 – .7 (the defects it found); .13c.2v.8 – .10 (what it routed)
reverify: "python3 docs/tasks/artifacts/sv_corpus_grad/svpp_corpus_census/census.py --bank --probe rust/target/release/parseability_probe   # SVPP-CORPUS-CENSUS-BANK: cases=66 … failures=0; the full census needs the SV raw lane's results.tsv"
---

# The preprocessor parser is measured over the SystemVerilog corpus

**The question.** The `systemverilog_preprocessor` family published its corpus leg as NOT MEASURED: no corpus of
"directive files" is wired. But every file a SystemVerilog tool compiles is a preprocessor input, whatever its
SystemVerilog verdict, so the 16,336 files the SV lanes read ask the directive-structure parser one question each:
does the structure parse?

**The instrument.** `docs/tasks/artifacts/sv_corpus_grad/svpp_corpus_census/census.py` runs the GENERATED parser in
the release `parseability_probe` (never the interpreter), classes every rejection by the backtick word that opens the
line where `furthest_position=` lands, and joins each to a written reason in `residual_dispositions.tsv`;
`--before <probe>` adds transitions and AST identity; `--bank` runs 66 clause-22 reproducers with no corpus.

**What it found.** Release `1.0.7` refused 4,477 files: 4,182 at a line that OPENS with a macro usage (the grammar's
passthrough text could not begin with a backtick), 243 at seven clause-22 directives with no production, 52 at
neither (a string or comment holding a backtick in macro text, a one-line conditional). Beside them it exposed three
wrong trees for accepted input: a continued define body cut at its first line, `` `define M (x) `` read as
function-like, punctuation inside a string split out of macro text.

**Release `1.0.8`.** Rejections 4,477 → 64, zero `pass → fail`; 11,790 of the 11,859 previously accepted trees
byte-identical and the 69 that moved each attributed. The 64: 34 suites' own error tests, 16 refused by a clause, 12
valid IEEE 1800-2023 / escaped-name / one-line-conditional text and 2 malformed directives inside a skipped group —
all 14 gaps the director ruled FIXED, never documented limits (2026-10-18; `.13c.2v.8` – `.10`).

**How to change the grammar safely.** Prototype the edit in a scratch copy and run the same population through
`ast_pipeline <grammar> --interpret-parse` with both grammars (≈ 7 min, no build), attributing every moved tree to one
delta; then generate once and let the census confirm on the generated parser — on `1.0.8` the two named identical sets.
