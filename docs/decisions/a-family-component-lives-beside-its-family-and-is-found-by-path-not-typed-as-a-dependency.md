---
name: a-family-component-lives-beside-its-family-and-is-found-by-path-not-typed-as-a-dependency
description: "DECISION (2026-09-29, ENGINE-UNIVERSAL-SERVICES.113 / PGEN-ENGINE-UNIVERSAL-SERVICES-0169) — a language-specific component (the SystemVerilog preprocessor, its §22.14 keyword tables) lives under rust/src/families/<name>/, is DISCOVERED by build.rs and classed per-family BY ITS PATH, and reaches the engine only through a neutral interface it declares (the preprocessing stage) — never as a workspace crate the engine's manifest would have to TYPE, and never as a module lib.rs names."
id: a-family-component-lives-beside-its-family-and-is-found-by-path-not-typed-as-a-dependency
title: "A family component lives beside its family and is found by path — a typed crate dependency is one more list to keep in lockstep"
date: 2026-09-29
evidence: "docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .113 (the measured start: main.rs 1, parseability_probe.rs 13, parse_harness_equivalence.rs 20 language tokens; the end: every ENGINE-NEUTRALITY ceiling 0, 5 family-component files classed by path); rust/build.rs (`families/<name>.rs` OR `families/<name>/mod.rs` ⇒ the module and the `components()` table); scripts/check_engine_neutrality.sh (the path classifier + two self-test arms)."
reverify: "bash scripts/check_engine_neutrality.sh   # 'engine: 0 language tokens … 0 still non-zero … N family-component files classed by path'; ./rust/target/debug/ast_pipeline x --preprocess systemverilog --list-stage-options"
answers:
  - "where does a language-specific component such as a preprocessor live in PGEN"
  - "why is the SystemVerilog preprocessor not a separate crate"
  - "how does the engine reach a family's preprocessor without naming it"
  - "what does the ENGINE-NEUTRALITY register need when I add a family component"
  - "how do I add a preprocessor for a second language"
metadata:
  node_type: memory
  type: project
  created: 2026-09-29
---

**The engine may not know a language's name, so it may not TYPE a list of its components either — not in
`lib.rs` (`.112` slice 2 removed that list), and not in `Cargo.toml` as a per-family crate dependency, which
is the same list in a different file.** `.113`'s pre-registration sketched a workspace crate
`pgen-sv-preprocessor`; measured against the rule it fails on the one axis that matters (the engine's manifest
would spell the family) and costs 89 Makefile + 66 script cargo invocations. The shape that holds the rule is
the one `.112` slice 2 already established for the family hooks: **a component is a directory** —
`rust/src/families/<name>.rs` or `rust/src/families/<name>/mod.rs` with its own submodules and tests —
**discovered by `build.rs`** (the module list and the `components()` table are written into `OUT_DIR`,
unconditional on any generated artifact), **classed per-family by its path** in the `ENGINE-NEUTRALITY` guard
(no register row; it may name its own family and nothing else), and **reached through an interface the engine
owns and the family declares**: for a preprocessor, `FamilyHooks.preprocessing: Option<&PreprocessingStage>`,
found by `preprocessing_stage::stage_for(family)`.

What the engine owns of preprocessing is the STAGE — an ordered compilation unit, include search
directories, predefined macros, the profile the expanded text will be parsed under, and the family's own
options by key, in; expanded text with a source map, an event log and diagnostics, out — because every front
end knows those and no language's directive syntax is among them. The family's knobs travel as
`--stage-opt KEY=VALUE` and the family LISTS them (`--list-stage-options`); its deprecated CLI spellings are a
table the family declares and the engine rewrites (one stderr line each) — so even the deprecation of a
family's flags spells no family in the engine. A second language with a preprocessor adds one directory and
nothing to the engine, the CLIs or the guard's register.

Related: [[the-engine-is-neutral-and-the-forbidden-vocabulary-is-derived-from-the-grammars]] (the guard),
[[feedback-engine-neutrality-is-the-whole-point]] (why the boundary is a doctrine).
