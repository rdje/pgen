---
id: a-file-no-list-names-is-judged-by-running-the-tool
title: A file no list names is judged by running the tool — the vendored compiler's exit status and message, recorded verbatim, are the key; the clause overrides the tool where the tool is lenient; a companion reached by a channel nobody reads is deferred by name, not left dark
answers:
  - "48 ivtest rows had no answer key — how were they adjudicated"
  - "an unlisted test file has no upstream expectation — is that permanent"
  - "iverilog accepts a construct the Annex A production refuses — which one decides"
  - "a library companion reached through -y: how is it classed when no list names it"
  - "a continuation source of a CE entry: how do you know which file carries the compile error"
  - "the tool refuses a file at elaboration only — is the parse valid"
  - "a Verilog-2005 file uses `new` as an identifier — what does the sv_2017 lane expect"
  - "what is the deferred:no_sv_key class and why is it zero"
tags: [systemverilog, answer-key, ivtest, iverilog, corpus, adjudication, tool-testimony, clause-pin]
date: 2026-10-09
provenance: SV-CORPUS-GRAD.52 (PGEN-SV-CORPUS-GRAD-0439); SV-CORPUS-GRAD.13e.13 (the residue), .13e.14 (the method), .13c.2k (the pull-gate pin reused)
reverify: "python3 stimuli/sv/run_adjudication_repros.py   # ADJUDICATION-REPROS: ... failures=0; grep -c 'pinned .52\\|companion (.52)' stimuli/sv/adjudicate_external_corpus.py  # 48"
---

# A file no list names is judged by running the tool

**The population.** `.13e.13` read every list the vendored iverilog runners read and 48 rows were left
with no key: 40 files no list, descriptor, continuation source or `` `include `` names, and 8 continuation
sources of two `CE` entries whose list cannot say which source carries the error. The class
`deferred:no_sv_key` is burnable — the absence of an upstream key is a fact about the suite, not about the
file — so each row was judged the `.13e.14` way.

**The method, in order of authority.**
1. **Run the tool on the file alone**, under the compiler's default generation (`-g2005`) and its
   SystemVerilog generation (`-g2012`); record the exit status and the first message verbatim in the pin.
   A clean compile is `must_accept`. An elaboration- or semantic-stage message (`Unable to bind`, `cannot be
   driven`, `Invalid enum base type`, `Repeat timeunit does not match`) is `must_accept` too: the text was
   derived by the BNF and refused by prose the parser does not implement. A `syntax error` under one
   generation and a clean compile under the other is an edition reading, not a parser verdict.
2. **The clause overrides the tool where the tool is lenient.** Three files the tool compiles are refused
   by Annex A — `pulldown (pd1, pd2)` (one terminal per pull-gate instance), `reg real vo;` (two data types
   on one declaration), `{a,,b}` (an empty concatenation element) — and one is an edition contrast
   (`reg [7:0] new;`, legal 2005 text, `new` reserved by 1800-2017 Annex B). Each got a reproducer the
   oracle re-runs on every commit and, for the contrast, a byte-identical control that must ACCEPT under
   `verilog_2005`.
3. **A companion reached by a channel nobody reads is deferred by name.** Two files are `-y` library
   modules whose macros are defined by the unit's main file; alone they cannot be judged. They are
   `deferred:chained_only` with the driver and the channel named in the basis, so the pipeline reads them
   under the unit's context and the next reader knows the index does not walk `-y`.

**Measured.** 48 rows moved on the sv_2017 and sv_2023 lanes and nothing else: 44 `match`, 2 explained by
the macro position, 2 `chained_only`; the bar stayed 0 on every lane; the raw tuple's adjudicated count
rose by 46 and the no-verdict count fell by 46.

**The rule.** A row with no key is not a row with no verdict — it is a row whose key must be MADE, from the
tool's own output and the standard's own text, and written down so the next reader can re-run both.
