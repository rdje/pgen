---
id: a-standards-own-example-text-outranks-its-annex-a-and-the-next-editions-annex-a-usually-says-so
title: A standard's own example text outranks its Annex A, and the next edition's Annex A usually says so — when the LRM's example is not derivable by its formal syntax, the example is the spec and the omission is an erratum
answers:
  - "the parser rejects an example straight out of the IEEE 1800-2017 text (tagged Jmp (tagged JmpU .a), pattern matching §12.6) — is the grammar wrong or the LRM"
  - "IEEE 1800-2017 Annex A pattern has no ( pattern ) alternative but 1800-2023 does — which one does the sv_2017 profile follow"
  - "a 1800-2012 test parses under sv_2023 but not sv_2017 — edition contrast or a 2017-profile gate"
  - "how do I widen the older profile arm without changing the AST shape (give it the newer arm's spelling)"
  - "the raw-AST export refuses with 'requires building with --features ebnf_dual_run' — which cargo build do the gates use"
  - "what was the second instance of the .3.30 rule (the standard's own tables/examples are the spec)"
  - "the LRM's own example uses a construct its normative clause forbids one paragraph later — which one does the grammar follow"
  - "Annex A DERIVES the example but a footnote and the clause text restrict it, and the next edition kept the restriction — is the example the spec"
  - "import \"DPI-C\" function bit [15:0] getStimulus() is straight out of §35.5.4 and the parser now rejects it — bug or fidelity"
tags: [lrm, annex-a, erratum, edition, sv-corpus, grammar-authoring, systemverilog, profiles, pattern-matching]
date: 2026-09-04
updated: 2026-09-06
status: current
evidence: "REVERSE case SV-CORPUS-GRAD.3.43 (PGEN-SV-CORPUS-GRAD-0371, release 1.0.216, ledger SV-0089): IEEE 1800-2017 §35.5.4 example docs/systemverilog/2017/md/section-35-direct-programming-interface.md:476 `bit [15:0] getStimulus()` vs the normative IEEE 1800-2017 §35.5.5 docs/systemverilog/2017/md/section-35-direct-programming-interface.md:485-493 + Annex A footnote 21 docs/systemverilog/2017/md/section-41-data-read-api.md:3524; IEEE 1800-2023 keeps both — §35.5.4 example docs/systemverilog/2023/md/section-35-direct-programming-interface.md:489 unchanged, §35.5.5 docs/systemverilog/2023/md/section-35-direct-programming-interface.md:493-501, footnote 26 docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3490 — the example is the erratum. FIRST case SV-CORPUS-GRAD.3.34 (PGEN-SV-CORPUS-GRAD-0362, release 1.0.209, ledger SV-0082): IEEE 1800-2017 §12.6.1 section-12-procedural-programming-statements.md:826/:828/:840 vs Annex A section-41-data-read-api.md:2279-2286 (no `( pattern )`) vs 1800-2023 section-Annex_A-normative-formal-syntax.md:2273-2281 (`( pattern )` first); five ispras ieee-1800-2012/12/12.06.* bar rows fail -> pass on sv_2017; the .3.30 UDP-table rule's second instance"
reverify: "python3 stimuli/sv/run_adjudication_repros.py | tail -1   # fixed_nested_tagged_pattern_paren.sv ACCEPT on sv_2017 + sv_2023, failures=0"
---

**When a standard's own example is not derivable by its Annex A, the example is the spec and the
omission is an erratum — and the next edition's Annex A usually corrects it. A profile that transcribes
Annex A literally refuses the standard's own text; a second profile over the same corpus is what shows it.**

## The rules

- **Read the clause's examples beside its productions.** IEEE 1800-2017 §12.6.1 writes
  `tagged Jmp (tagged JmpU .a)` and `tagged Jmp (tagged JmpC '{.c, .a})`; its A.6.7.1 `pattern` has no
  `( pattern )` alternative. A nested tagged pattern needs the parentheses to bind, so the examples are not
  derivable without them. 1800-2023 spells `pattern ::= ( pattern ) | …` first — the correction is the
  evidence. Cite all three (`path:line`) in the pin, the ledger and the register.
- **Older-edition text the older profile refuses cannot be an edition contrast.** The `.14a` differential
  named five IEEE 1800-2012 tests `2023_only_pass`; a 2012 test that only the 2023 profile parses is a gate
  on the 2017 arm, not 2023-only syntax — the opposite of the `.14b` rows.
- **Give the older arm the newer arm's own spelling.** `pattern_sv_2017` gained the alternative in exactly
  the form `pattern_sv_2023` carries it (`-> {kind: "parenthesized", body: $2}`), so the `kind` sets
  coincide and a newly-accepted parse carries a value the newer profile always emitted: shape-neutral by
  construction, then confirmed by the control's byte-identical AST.
- **Batch a dead-rule deletion with a widening.** The ten terminals the UDP fix orphaned rode the same
  release; the syntax-closure allow-list returned to one by its good-news branch and both certificates'
  proof counts fell by exactly ten, expected by name before the gates confirmed them.
- **Build the debug `ast_pipeline` the way the gates do** — `cargo build --features "generated_parsers
  ebnf_dual_run" --bin ast_pipeline` — or the raw-AST export the identity checkers and the keyword
  censuses depend on refuses with *requires building with --features ebnf_dual_run*.

## The measurement

See `docs/tasks/SV-CORPUS-GRAD.md` `.3.34` and `.3.33`, ledger `SV-0082`, register row `1.0.209`; the
`.3.30` rule's first instance is `a-single-character-terminal-that-may-touch-its-neighbour-must-not-carry-a-word-boundary`.

## The reverse case — when the example is the erratum (`SV-CORPUS-GRAD.3.43`, 2026-09-06)

The rule above is about an **omission**: Annex A cannot derive text the clause's example shows, nothing normative
forbids it, and the next edition's Annex A adds the production. It does not cover a **restriction**: IEEE 1800-2017
§35.5.4's example block writes `import "DPI-C" function bit [15:0] getStimulus();` (`section-35-direct-programming-
interface.md:476`) and §35.5.5, one paragraph later, says a DPI import function's result "is restricted to small
values" — the nine keyword types and *scalar* `bit` / `logic` (`:485-493`); Annex A footnote 21 repeats the
restriction on `dpi_function_proto` itself (`section-41-data-read-api.md:3524`). Annex A DERIVES the example; the
normative text FORBIDS it; and 1800-2023 kept both the example (`:489`) and the restriction (footnote 26,
`section-Annex_A-normative-formal-syntax.md:3490`). Apply the same tie-breaker in the same direction: **the next
edition is the tell.** When it corrects Annex A toward the example, the example was the spec (the first case). When it
keeps the normative restriction beside the unchanged example, the example is the leftover — two normative statements
outrank one informative code block, and the grammar refuses the example. Verilator accepts ≤ 32-bit 2-state vector
results as an extension and sv2v transcribed the LRM's line into `test/nosim/dpi.sv:35`; both rows are keyed
`must_reject` with the production (`stimuli/sv/adjudicate_external_corpus.py`, `EXTRA_PINNED`, `.3.43`), and the
pin `invalid_dpi_result_packed_bit.sv` carries the LRM's own line as the text that must reject.
