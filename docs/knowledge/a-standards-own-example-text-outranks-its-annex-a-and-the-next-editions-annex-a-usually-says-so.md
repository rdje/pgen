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
tags: [lrm, annex-a, erratum, edition, sv-corpus, grammar-authoring, systemverilog, profiles, pattern-matching]
date: 2026-09-04
status: current
evidence: "SV-CORPUS-GRAD.3.34 (PGEN-SV-CORPUS-GRAD-0362, release 1.0.209, ledger SV-0082): IEEE 1800-2017 §12.6.1 section-12-procedural-programming-statements.md:826/:828/:840 vs Annex A section-41-data-read-api.md:2279-2286 (no `( pattern )`) vs 1800-2023 section-Annex_A-normative-formal-syntax.md:2273-2281 (`( pattern )` first); five ispras ieee-1800-2012/12/12.06.* bar rows fail -> pass on sv_2017; the .3.30 UDP-table rule's second instance"
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
