---
name: project-svpp-expansion-is-a-service-over-the-svpp-ast
description: DIRECTOR DIRECTION (2026-10-19) — svpp's EXPANSION is a service over the svpp directive AST, offered to downstream projects in-process (in-memory file provider) and as a tool, with three consumption modes (AST only / PGEN expands / intercept), and the SystemVerilog parser's own front end runs on the same engine; concretizes the 2026-06-09 ruling and supersedes the 2026-09-09 steer that built the driver on the hand-written scanner.
id: project-svpp-expansion-is-a-service-over-the-svpp-ast
title: "svpp's expansion is a service over the svpp AST — offered downstream in-process and as a tool, and the SV front end runs on the same engine"
date: 2026-10-19
evidence: "measured 2026-10-19 before the direction: systemverilog_preprocessor 1.0.9 yields the directive AST only (its contract's Scope); the expansion stage rust/src/families/systemverilog/preprocessor.rs is a hand-written scanner reachable only through parseability_probe --preprocess, which parses the expanded text and never emits it; the embedding API's SV embed module publishes parse / AST-dump entry points only (grep: no preprocess_* function); the two readers of the directive layer diverge at SVPP-EXPANSION.5(w)–(ac)"
reverify: "grep -n 'pub fn' rust/src/families/systemverilog/mod.rs | grep -c preprocess   # 0 until SVPP-EXPANSION.9.2 publishes the API; and: grep -n 'Scope / Non-Goals' -A3 docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md   # 'This parser does not decide LIVENESS'"
answers:
  - "how does a downstream project get preprocessed SystemVerilog from svpp"
  - "does the SystemVerilog parser use svpp or its own preprocessor"
  - "is the expansion stage a transformation over the svpp AST or a separate engine"
  - "what did the director decide about the svpp expansion wrapper tool"
  - "what are the three ways to consume svpp"
---

# svpp's expansion is a service over the svpp AST, offered downstream and used by the SV front end

- **Date:** 2026-10-19 · **By:** the director (verbatim below) · **Recorded by:** `PGEN-SV-CORPUS-GRAD-0468`
- **Tree:** `SVPP-EXPANSION.9` (design). Concretizes the 2026-06-09 ruling
  ([[project_preprocess_parse_pipeline_is_the_sv_deliverable]] records the tree's two statements); supersedes the
  2026-09-09 steer that built the driver on the hand-written scanner.

## The direction (verbatim)

> "I have a question, how a downstream consumer would use svpp if the goal is to have it preprocess SystemVerilog files
> so that such project can operate on the preprocess files ? I am asking because right now the SV parser doesn't use
> svpp, but its own builtin preprocessor."

> "I think you would need to define the contract the downstream project on how to intregrate svpp efficiently,
> elegantly to their projects"

> "I think we should make the preprocessing as easy as possible to use by the downsream project. I know we are
> providing parsers, but what do you think about providing another tool which can be use also in-memory or in-process
> that will consume the AST from svpp and apply expansion that the downstream process could use either svpp alone, so
> that downstream project will have to do the expansion itself, or it doesn't want to do the expansion and will
> happily use PGEN svpp expansion wrapper tool to the job for it, your take."

## What was true when it was given (measured)

`systemverilog_preprocessor` `1.0.9` yields the directive AST only. The expansion stage
(`rust/src/families/systemverilog/preprocessor.rs`) is a hand-written scanner that does not read that AST, reachable
only through `parseability_probe --preprocess`, which does not emit the expanded text; the embedding API publishes no
preprocessing entry point. A downstream project could obtain neither the expanded text nor a source map. The two
readers of the directive layer diverge in measured places (`SVPP-EXPANSION.5`(w) – (ac)).

## The decision

1. Two layers with one boundary, the svpp AST: svpp (text → AST) and an EXPANDER (AST + context → expanded text,
   source map, events, diagnostics, included files).
2. Three consumption modes, each first-class: AST only; PGEN expands; intercept (edit the AST, then expand), with an
   in-memory file provider in the context.
3. One engine: the SystemVerilog parser's own front end calls the same expander. The hand-written directive recognizer
   is retired in favour of the grammar's AST; the stage's substantive expansion logic is kept.
4. A downstream contract and a book chapter publish the surfaces, the context, the outputs and their stability axes.

## Why

- A downstream project integrating "svpp" must get preprocessing from it, or the name promises what it cannot give.
- One reader of the directive layer is the only design in which "integrate svpp" and "get what PGEN's parser saw" are
  the same thing; with two readers every divergence is a defect ticket forever.
- The EBNF is the single source of truth (north star): the directive layer's structure belongs to the grammar.

## How to apply

Design before code (`SVPP-EXPANSION.9`): the contract draft; then the Rust API over the existing stage, so a consumer
has text early; then the expander over the AST, measured by the pipeline re-key with the (w) – (ac) rows as the
expected movers; then the SV front end on it. Never a narrowed promise: the expander meets the full clause-22
requirement under every edition profile.
