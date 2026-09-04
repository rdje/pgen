---
id: a-keyword-the-identifier-rule-does-not-refuse-is-an-identifier
title: A keyword the identifier rule does not refuse IS an identifier — derive the reserved set from Annex B, and know that no certificate, corpus lane or lint can see a word that rule admits
answers:
  - "PGEN parses `reg global;` / `var final;` / `i.super.i = 1;` — a SystemVerilog keyword is accepted as a variable or hierarchical name"
  - "`function new; input x;` parses in a class — the constructor rule is right, so why does the body accept a tf_port_declaration"
  - "which IEEE 1800-2017 keywords does the SV grammar reserve — is the list Annex B Table B.1, and how is that checked"
  - "how do I regenerate the reserved-word regex from the LRM tables (stimuli/sv/check_sv_reserved_keywords.py --write)"
  - "PGEN accepts `automatic int x;` in a package or module — Annex A footnote 10"
  - "PGEN accepts `parameter A;` in a module body — Annex A footnote 18's parameter half"
  - "how do I see which production derived a token in a file (parseability_probe --parse-dump-ast-pretty, then walk to the token's name.body)"
  - "why did adding a keyword to the reserved list need a twin rule for the randsequence code block"
  - "`x < local::x` inside randomize() with { } fails at the x — A.8.4 class_qualifier's local:: had no production"
  - "the interpreter (--interpret-parse) rejects `mailbox #(int) m = new(4);` but the generated parser accepts it — is the interpreter faithful for std / provisional class types"
  - "a Verilog-2005 test names a variable var / global / local and the sv lane now rejects it — how is the key corrected (edition_keyword_pass)"
tags: [grammar-authoring, keywords, annex-b, footnotes, sv-corpus, over-acceptance, systemverilog, doctrine]
date: 2026-09-04
status: current
evidence: "SV-CORPUS-GRAD.3.37 + .13z (PGEN-SV-CORPUS-GRAD-0366, release 1.0.211, ledger SV-0084): the SV reserved set moved from a hand-typed 173 words to IEEE 1800-2017 Table B.1's 248 (1800-2023 lists the same 248), derived by stimuli/sv/check_sv_reserved_keywords.py and enforced as doctrine SV-RESERVED-WORDS; footnote 10 (no automatic outside a procedural context) and footnote 18's parameter half closed with the same slice; 17 pins; the .13z triage read every accepts-invalid row by production"
reverify: "bash scripts/check_sv_reserved_words.sh   # self-test + gate: 248 / 248 / 124, both directions; python3 stimuli/sv/run_adjudication_repros.py | tail -1"
---

**Every downstream instrument — the certificates, the three corpus lanes, `--lint-grammar`, the pins —
consumes the same `non_keyword_identifier` rule. A word that rule does not refuse is, to all of them, a
perfectly good identifier. So a reserved-word list is never "good enough"; it is either the LRM's table,
derived and re-derived, or it is a defect nothing can see.**

## The rules

- **Derive the reserved set; do not type it.** `reserved_non_keyword_identifier_sv` held 173 of Table B.1's
  248 words for the life of the grammar: `new`, `this`, `super`, `null`, `static`, `var`, `const`, `global`,
  `final`, `local`, `protected`, `virtual`, every assertion operator (`until`, `s_eventually`, …) and every
  coverage word (`bins`, `cross`, …) were admissible identifiers. `stimuli/sv/check_sv_reserved_keywords.py`
  extracts the table from the in-repo LRM text (one identifier-shaped token per line between the heading and
  the first prose line, the PDF's page furniture skipped by pattern — the table is NOT alphabetical, it lists
  `endspecify` before `endsequence`), compares sets in both directions, and `--write` regenerates the regex.
  Doctrine `SV-RESERVED-WORDS` runs it on every commit.
- **The tell is a derivation, not a verdict.** `parseability_probe --parse-dump-ast-pretty <file> out.json
  --profile sv_2017`, then walk the JSON to the token: `global` sat under `variable_decl/assignments/
  variable/name`; `super` under `lvalue/name/scope`; `new` under `function/…/name` — the constructor was an
  ordinary function NAMED `new`, and `C c = new;` a hierarchical reference to it. The class-constructor rule
  itself was right (`{ block_item_declaration }`); it was simply never reached.
- **Reserving a word moves legal text onto the productions that name it.** `this`/`super` must reach
  `implicit_class_handle`, `new` must reach `class_new`, `null` its primary. Write the legal twin FIRST
  (`control_class_handles_and_new.sv`) and compare its kind multiset before and after — the `.3.35` rule.
- **A reserved word can also be a hole's disguise.** `local::x` (A.8.4 `class_qualifier ::= [ local :: ] [ implicit_class_handle . |
  class_scope ]`) had NO production — the nonterminal had been extracted into the grammar as a literal keyword — and
  parsed only because `local` was an admissible identifier standing in for a package name. Reserving it exposed the
  hole; the fix is the positive production, and the LRM's own §18.7.1 example is the control.
- **The interpreter has no semantic store — do not judge store-gated or provisional types with it.** `mailbox #(int) m
  = new(4);`, `semaphore s = new(1);`, `process p;` REJECT under `--interpret-parse` even with the unchanged grammar,
  while the generated parser admits Annex G's `std` classes as `provisional_class_type`. Thirty of a 77-file pre-screen
  were this artifact; the second generated build's lane moved none of them. Pre-screen with the interpreter to find
  candidates, then let the generated parser and the pins decide.
- **A key can be edition-wrong, and the proof is a parse.** A Verilog-2005 test that names a variable `var`, `global`,
  `local` or `new` is legal 1364-2005 and not SystemVerilog; the sv lanes' `must_accept` keys were a Verilog tool's
  testimony. The adjudicator's `edition_keyword_pass` re-keys such a row iff the sv parse stops ON a word 1800
  reserves and 1364 does not AND the `verilog_2005` profile accepts the bytes — measured per row, with the byte and the
  word in the basis; the second condition keeps a grammar hole (`d = null;` stops ON `null`) from hiding there.
- **Footnote 10 is about the CALLER, not the rule.** "In a data_declaration that is not within a
  procedural context, it shall be illegal to use the automatic keyword": module, package, checker and class
  sites get a `static`-only lifetime, while `rs_code_block` (A.6.12, inside a statement) keeps the full
  lifetime through an exact twin — same node templates, so no legal tree moves.
- **A footnote has two halves.** Footnote 18 forbids omitting a default outside a parameter_port_list;
  `.3.35` closed `localparam`, this slice closes `parameter` at the seven body sites ($unit, block, class,
  interface class, package/generate items) while both port-list rules keep the optional default.
- **Footnote 20 is the next one.** `reg [] x;` is legal only as the sole packed dimension of a DPI import
  (`.3.38`); that needs the DPI scope, not a lookahead.
- **A key can be wrong for four reasons at once.** Of fourteen accepts-invalid rows: five were grammar
  (this slice), one footnote 20 (`.3.38`), four were the tool elaborating (`integer x [] = 1;`,
  `P::Q = 1;`, `{a,b}.f1`, `trireg (small) t;` — all derivable, keys corrected with the production), two were
  directive arguments the parser tolerates by design (`.3.39`), one a keyword-version directive
  (`` `begin_keywords "1364-2005" `` — a profile question), one the engine (`.93`).

## The measurement

See `docs/tasks/SV-CORPUS-GRAD.md` `.3.37` and `.13z`, ledger `SV-0084`, register row `1.0.211`.
