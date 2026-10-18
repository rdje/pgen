---
id: a-ceremony-that-lists-its-family-gates-misses-a-gate-that-reads-the-grammar
title: A release ceremony that lists its family's gates misses a gate that reads the grammar for another reason — the EBNF dual-run caught two svpp releases only at the push (`SV-CORPUS-GRAD.13c.2v.20`)
answers:
  - "why did the CI mirror fail a grammar the family's gates all passed"
  - "which gates does a grammar release have to run"
  - "why was the svpp dual-run envelope ceiling stale since 1.0.8"
  - "what reads grammars/<family>.ebnf besides the family's own gates"
tags: [ceremony, ci-mirror, dual-run, meta-grammar, preprocessor, gates]
date: 2026-10-18
provenance: SV-CORPUS-GRAD.13c.2v.20 (PGEN-SV-CORPUS-GRAD-0465); LANG-CAPABILITY-AUDIT.10.19; rust/scripts/ebnf_frontend_dual_run_diff_gate.sh
reverify: "make -C rust SHELL=/bin/bash ebnf_frontend_dual_run_diff 2>&1 | grep -E '^systemverilog_preprocessor |passed for all'   # svpp pass, '... passed for all tracked grammars'"
---

# A ceremony that lists its family's gates misses a gate that reads the grammar

**A grammar is an input to more than its family. The EBNF dual-run parses EVERY tracked grammar with the
self-hosted meta-parser and ratchets the envelope against the Rust frontend; it reads
`grammars/systemverilog_preprocessor.ebnf` for the META-grammar's sake, so no svpp gate is it — and the svpp
release ceremony ran the svpp gates.**

## What it cost

- Release `1.0.8` raised the svpp envelope past its ceilings (26 divergences against 15, 23 unmapped against 12 —
  eleven new directive keywords, each a `[>! /\w/]` line arm 2 reads as a character class). Nothing failed: the
  dual-run runs in the push mirror, and no push fell between the two releases.
- Release `1.0.9`'s first commit then failed the mirror outright: arm 2 could not parse two shapes the release
  introduced (a bracket class inside `[>! … ]`; a `[> … ]` line after a rule that ends in a return annotation). The
  fix — a respelling, a redundant restriction removed — changed the generated parser by inlining alone, and that
  still cost a rebuild and a full SystemVerilog re-key, because every identity row names the probe's bytes.

## The rule

- **Derive a release's gate list from the files it changes, not from the family's name.** For a grammar that is: the
  family's gates, plus every gate whose inputs include `grammars/*.ebnf` — the dual-run, the reproducibility rows, the
  certificate table, the parity gates.
- **Run the push mirror's grammar gates BEFORE the identity re-key.** A re-key is the most expensive step of a
  ceremony and is invalidated by any later byte the binaries embed; a gate that can force a grammar edit belongs in
  front of it.
- **Read a refused input on both readers.** The Rust frontend is authoritative and accepted the grammar; the
  meta-grammar refused it. Each refusal was reduced to a three-line grammar (`parseability_probe --parse ebnf`)
  before anything was changed.

Related: [[a-filtered-acceptance-run-is-not-the-module]], [[a-generated-sample-the-parser-refuses-is-a-boundary-or-a-defect]].
