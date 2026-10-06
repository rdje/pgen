---
id: a-stage-that-passes-through-what-it-did-not-resolve-lets-the-next-stage-accept-it
title: A stage that passes through what it did not resolve lets the next stage accept it — a pipeline's pass is only as strong as the weakest stage's post-condition, so each stage refuses its own leftovers
answers:
  - "why does the preprocess-then-parse pipeline report pass for a file with an undefined macro"
  - "two stages are each correct on their own — how can their composition be wrong"
  - "where should an unresolved macro usage be refused: the preprocessor or the parser"
  - "why does the SystemVerilog grammar accept a bare backtick line at the top level"
  - "what is the undefined_macro_policy stage option for"
  - "how do I find rows a pipeline passes over text it never resolved"
tags: [pipeline, preprocessor, post-condition, composition, silent-acceptance, corpus-lane, systemverilog]
date: 2026-10-06
status: current
evidence: "SVPP-EXPANSION.6f (`PGEN-SVPP-EXPANSION-0044`). The SystemVerilog preprocessor kept an undefined macro usage as written (a design the macro-string renderer and the paste walker rely on), and the SV grammar's compiler_directive rule carries the RAW lane's residue — an opaque backtick line at the top level is accepted so an unexpanded file parses around its macro uses. Composed in the pipeline, a top-level `not_defined reached the parser and the row read pass: verilator t_preproc_nodef_bad.v, sv2v error/undefined_macro.sv, and 25 riscv-dv instruction files whose macros live in a package the lane refuses — 38 passing rows over unresolved text, found by re-expanding every passing row and scanning the output for a backtick outside comments and strings; 318 failing rows carried one too. The fix is the stage's own post-condition: after expansion no backtick survives outside a comment or string unless it is a directive the parser reads — an undefined name, a function-like macro without its argument list, a stray macro-string token and an unmatched `end_keywords are refused with their clause (undefined_macro_policy=error, the default; keep restores the passthrough)"
reverify: "rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/invalid_undefined_macro_usage.sv --profile sv_2017 --preprocess 2>&1 | tail -1 | grep -c 'neither a defined macro nor a compiler directive'  # expect 1: the stage refuses before the parser can accept the opaque line"
---

**A pipeline's `pass` is a conjunction of stage claims, and the weakest one wins.** The preprocessor
said "I expanded what I could and left the rest as written"; the parser said "a top-level backtick
line I do not recognise is an opaque directive, accepted". Each is a defensible design for the lane
it was built for — the expander's leniency serves the renderers that need a usage's bytes, the
grammar's residue serves the raw lane that parses unexpanded files. Composed, they produced a
`pass` for a file whose first line was an undefined macro.

- **Each stage refuses its own leftovers.** The stage that knows what it did not resolve is the one
  that must say so: after expansion, a backtick outside a comment or string literal is either a
  directive the parser reads or a gap — an undefined name (§22.5.1 gives a usage its meaning by
  substitution; no definition, no derivation), a function-like macro used without its parentheses,
  a macro-string token outside macro text, an `` `end_keywords `` closing nothing. The scan is the
  stage's post-condition, not the parser's business, and a consumer that wants the old passthrough
  asks for it (`undefined_macro_policy=keep`).
- **Measure the hole by re-deriving every pass, not by reading failures.** The failing rows had been
  censused twice and the hole never showed, because a leftover usage that the parser accepted left
  no failure to read. Re-expanding the 12 749 passing rows and scanning the output took minutes and
  named 38; the same scan over the failing rows named 318 whose stop was elsewhere.
- **The false passes were in the design suite's convention.** 25 of the 38 were riscv-dv instruction
  files, members of a package the lane refuses, counted ADJUDICATED as "a shipping design's file
  accepted as part of its unit" — the convention is right, the acceptance was not. A correction that
  lowers the adjudicated count is the honest direction here.
- **A lenient residue needs an owner that says where it applies.** The grammar's opaque directive
  line is the raw lane's and stays; the pipeline's guard is the stage's scan. Writing down which
  lane a leniency serves is what keeps the next composition from inheriting it silently.

Related: [[a-search-path-entry-grants-every-name-in-it-not-the-one-it-was-derived-for]] (the same
slice family: a grant wider than its justification), [[a-zero-in-a-census-can-mean-shadowed-not-absent-and-an-overlap-is-broken-by-a-guard-not-by-reordering]].
