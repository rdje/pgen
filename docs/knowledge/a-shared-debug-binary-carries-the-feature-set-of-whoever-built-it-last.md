---
id: a-shared-debug-binary-carries-the-feature-set-of-whoever-built-it-last
title: A shared debug binary carries the feature set of whoever built it last — rebuild the generator with the full set before any reader that needs it
answers:
  - "why does ast_pipeline say EBNF input requires building with --features ebnf_dual_run"
  - "why did four doctrines fail after running annotation_contract_gate"
  - "which gates rebuild rust/target/debug/ast_pipeline with a narrower feature set"
  - "why does the interpreter refuse a grammar file that worked an hour ago"
tags: [build, cargo, features, doctrines, gates, ast_pipeline, ebnf_dual_run, shared-binary]
date: 2026-09-30
status: current
evidence: "2026-09-30, the ENGINE-UNIVERSAL-SERVICES.118 verification chain: annotation_contract_gate (via annotation_stimuli_quality_gate.sh:556, `cargo build --features generated_parsers --bin ast_pipeline`) replaced rust/target/debug/ast_pipeline with a build lacking ebnf_dual_run; the next `bash scripts/check_doctrines.sh` failed SV-RULE-FIRE-PARTITION (`ast_pipeline --dump-rule-profiles failed`), SV-CONTRACT-CURRENCY, EBNF-ALTERNATIVE-PARITY and PIPELINE-LANE-CURRENCY at once, and the interpreter answered `EBNF input … requires building with --features ebnf_dual_run`; one `cargo build --features \"generated_parsers ebnf_dual_run\" --bin ast_pipeline` later, ALL 45 PASS — nothing in the tree had moved. The same class was named by DONE-BAR.5c's follow-up (3f943cc25, 2026-07-29: a gate broken by another gate's side effect on a shared binary)"
reverify: "cd rust && cargo build --features \"generated_parsers ebnf_dual_run\" --bin ast_pipeline >/dev/null 2>&1 && printf '@x: a' > target/probe.in && ./target/debug/ast_pipeline ../grammars/semantic_annotation.ebnf --interpret-parse target/probe.in --interpret-parse-ast-json target/probe.json | tail -1   # expect INTERPRET-PARSE: typed AST written"
---

`rust/target/debug/ast_pipeline` is ONE file, and cargo writes it from whichever `--features` set
built it last. Several gates build it with `--features generated_parsers` only
(`annotation_stimuli_quality_gate.sh`, `ast_dump_contract_gate.sh`, `annotation_robustness_gate.sh`,
the CI mirror's own `build_ast_pipeline` step); every reader that hands the binary an `.ebnf` file —
the interpreter (`--interpret-parse`), `--dump-rule-profiles`, and the four doctrines built on them —
needs `ebnf_dual_run` as well. After such a gate the doctrine enforcer goes red on four tiers at
once with messages about the SUBJECT ("could not derive the rule inventory", "differs from a fresh
render") while the tree is unchanged.

- **Read the four-at-once shape as the binary, not the tree.** SV-RULE-FIRE-PARTITION,
  SV-CONTRACT-CURRENCY, EBNF-ALTERNATIVE-PARITY and PIPELINE-LANE-CURRENCY failing together, right
  after an annotation or dump gate, is this class; the tell is the interpreter's
  `requires building with --features ebnf_dual_run`.
- **Rebuild before you read.** `cargo build --features "generated_parsers ebnf_dual_run" --bin
  ast_pipeline` (the `$(RUST_AST_PIPELINE)` rule's own set) before `check_doctrines.sh`, before any
  interpreter experiment, and before trusting a probe that a gate may have rebuilt narrower
  (`parseability_probe` too — a stale probe read 66 sentinels from a parser that no longer existed).
- **A gate that narrows a shared binary owes a widening or its own target dir.** The durable fix is
  the gates' (`--target-dir` or the full feature set); until a leaf owns it, this card is the
  discipline.

Related: [[a-build-script-panic-must-respect-the-bootstrap-posture]],
[[a-conditional-compilation-gate-is-a-claim-about-which-trees-exist]],
[[a-positive-control-built-on-a-live-defect-rots-when-the-defect-is-fixed]].
