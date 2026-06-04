---
id: makefile-targets-reference
title: rust/Makefile — the targets you actually reach for (regen, gates, clippy, book)
answers:
  - "what Makefile targets exist and which do I use"
  - "how do I regenerate a parser from its grammar"
  - "how do I regenerate the SystemVerilog parser after a grammar edit"
  - "how do I run the SV stimuli quality gate or the external-corpus triage"
  - "how do I run clippy the project-sanctioned way"
  - "how do I run the mdbook docs gate"
  - "where are the gate shell scripts and how do I run one directly"
tags: [makefile, build, gates, tooling, reference]
date: 2026-06-04
status: current
evidence: rust/Makefile (phony targets); rust/scripts/*.sh (gate implementations)
reverify: `grep -oE "^[a-zA-Z_][a-zA-Z0-9_]*:" rust/Makefile | sort -u` for the live target list; `ls rust/scripts/*.sh` for gates
---

Run from the repo root as `make -C rust SHELL=/opt/homebrew/bin/bash <target>` (several gates
require bash, not sh). There are ~130 targets; these are the ones that matter day to day.

## Regenerate a parser from its grammar (after an `.ebnf` edit)
- `focus_systemverilog` — regenerate `generated/systemverilog_parser.rs` from
  `grammars/systemverilog.ebnf` (+ runs the SV book gate + manifest cross-check). `focus_regex`,
  and the per-family `regex_parser` / `semantic_parser` / `return_parser` / etc. do the same for
  other grammars. ⚠️ **Verify the regen actually happened** — `focus_*` can exit 0 while the
  generated `.rs` stays stale; assert `generated/<g>_parser.rs` mtime > `grammars/<g>.ebnf` mtime,
  or confirm a real behavioral change ([[verify-sv-parser-regen-mtime]] / the SV regen discipline).
  `generated/` is GITIGNORED → regen is local verification; commit the codegen `.rs` source change,
  not the artifact.

## The SV closed-loop / corpus gates
- `sv_external_corpus_triage_gate` — parses the real external SV corpus; the 14/14 PASS no-regression
  check (uvm_pkg, scr1, friscv, veer × profiles). Run it after any SV grammar change.
- `sv_stimuli_quality_gate` — the full closed-loop stimuli gate: regenerates IR, generates +
  replays stimuli, reports `closed_loop_replay_targets_total` (the literal-0 residual) + the
  realistic corpus. ~15–25 min; rebuilds large state under `target/sv_stimuli_quality_gate`
  (reclaim it afterward — disk discipline). See [[sv-literal-0-residual-decomposition]].
- `sv_parser_aggregate_contract_gate`, `sv_semantic_scope_contract_gate`,
  `sv_parser_family_status_gate` — contract / family-status gates.

## Clippy, tests, docs
- `clippy_on_rust_change` — the sanctioned clippy flow: source stage must be **0 errors** (strict);
  the generated-parser stage is non-strict (carries ~pre-existing debt; set
  `PGEN_CLIPPY_GENERATED_STRICT=1` to fail on it). Banner `✅ clippy_on_rust_change completed.`
- Tests: `cargo test --lib` (no-features, fast, no generated parsers) vs
  `cargo test --features generated_parsers --lib` (compiles the generated parsers; runs the
  `ast_shape_contract` shape tests). Full-workspace `cargo test` may be RED on pre-existing
  unrelated bin-test breakage — prefer `--lib`.
- `mdbook_docs_gate` — the maintained proof lane for `docs/book/` (run when the book changes).

## Other families
`regex_*` (e.g. `regex_broader_corpus_proof_gate`, `regex_pcre2_compile_oracle_gate`), `vhdl_*`,
`annotation_*`, `return_*`, `semantic_*`, `sc01..sc13_contract_gate` (semantic-runtime contracts),
`sota_exit_gate`. Every gate has a script in `rust/scripts/<name>.sh` — you can run one directly
(`bash rust/scripts/<name>.sh`) when you don't want the Make wrapper.
