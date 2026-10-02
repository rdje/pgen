---
name: project-preprocess-parse-pipeline-is-the-sv-deliverable
description: DIRECTOR STEER (2026-09-09) — the SystemVerilog deliverable is the PREPROCESS→PARSE PIPELINE (preprocessor → expanded text → SV parser, under a real compilation context), developed TOGETHER with the parser; the raw-parse corpus benchmark is grammar-development scaffolding, not the deliverable. Activates tree SVPP-EXPANSION now, superseding its 2026-06-08 build-later sequencing. The expansion ENGINE already exists (hand-written, `rust/src/sv_preprocessor.rs`, since 2026-02-27) and composes 14/14 on the curated real-design cases; the driver being built is engine-agnostic, and the 2026-06-09 "transformation over the svpp AST" ruling is decided by the pipeline's measured defect counts, not re-litigated up front.
id: project-preprocess-parse-pipeline-is-the-sv-deliverable
title: "The SV deliverable is the preprocess→parse pipeline; a tree's frame is re-measured before its plan is executed"
date: 2026-09-09
evidence: "docs/tasks/SVPP-EXPANSION.md leaf .1 (the preprocess-only census rust/target/svpp_e1/pp_census.sh: files=4157 rc0=4079 rc_nonzero=78; the triage gate summary.txt of 2026-08-24: 14/14; git log --follow -- rust/src/sv_preprocessor.rs | tail -1 = 636f35c0, 2026-02-27)"
reverify: "git log --oneline --follow -- rust/src/sv_preprocessor.rs | tail -1   # 636f35c0 — the engine predates the 2026-06-08 tree; and: make -C rust SHELL=/bin/bash sv_external_corpus_triage_gate   # 14/14 through preprocess_then_parse_full"
answers:
  - "what is the SystemVerilog deliverable for Nexsim, the parser or the pipeline"
  - "does PGEN already have a SystemVerilog preprocessor or macro expansion stage"
  - "why is SVPP-EXPANSION active when it was sequenced after the locked program"
  - "should the preprocessor be rebuilt as a transformation over the svpp AST"
  - "what is missing for a real compilation context, +define, include dirs, unit file list"
  - "is the raw-parse corpus benchmark the release number"
  - "why re-measure a task tree's frame before executing its plan"
metadata:
  node_type: memory
  type: project
  director_directive: true
  created: 2026-09-09
  owning_tree: SVPP-EXPANSION
  supersedes_sequencing_of: project_svpp_expansion_stage_for_nexsim
---

**CONTEXT.** Three director statements bear on the SV front end, and a session reading only the newest
would contradict the older two without knowing it:

1. **2026-06-08** ([[project_svpp_expansion_stage_for_nexsim]]): PGEN must deliver the SV expansion stage;
   the SV + SVPP parsers are not release-complete for NEXSIM without it; **build-later**, sequenced after
   the locked program (every parser cert-coverage clean, `UNKNOWN` = 0).
2. **2026-06-09** (tree `SVPP-EXPANSION`, *Decisions*): the expansion stage is a **parse-tree transformation
   over the enhanced `svpp.ebnf` AST**, not a separate token-stream engine — EBNF as the single source of
   truth for the directive grammar.
3. **2026-09-09** (`MEMORY.md` at `feb45c4b`, recorded by the previous session from the director): *"BUILD
   THE PREPROCESS→PARSE PIPELINE — develop the preprocessor and the SV parser TOGETHER, not in isolation.
   Real SV/Verilog files are littered with macro directives, so the correct flow for EVERY file is
   preprocessor → expanded (directive-free) text → SV parser; the raw-parse corpus benchmark is
   grammar-development SCAFFOLDING … NOT the deliverable. The pieces EXIST: a full preprocessor in
   `rust/src/sv_preprocessor.rs` … WORK: wire a driver that runs the preprocessor FIRST with a real
   compilation context (include dirs, `+define`s, the unit's file list), then parses the expanded output,
   and measure end-to-end recognition on the macro-heavy corpus the raw lane defers (UVM … OpenTitan). If
   anything is missing, say so."*

**THE MEASURED STATE (2026-09-09, `SVPP-EXPANSION.1`).** The 2026-06-08 record's *"no PGEN stage feeds
svpp's output into sv"* was false when written: a hand-written expansion engine has existed since
2026-02-27 (`636f35c0`) with macros, conditionals, includes, a byte-indexed source map and diagnostics; the
shell composition `rust/scripts/sv_external_corpus_triage_gate.sh` has run it before the SV parser on seven
curated real-design cases (UVM's two packages included) and last read **14/14** (2026-08-24). A
preprocess-only census over all 4 157 UVM + OpenTitan corpus rows reads **4 079 of 4 157 preprocess** under a
plausible include path; 71 of the 78 refusals are an include-PATH gap and **7** are the engine's own, in
three defect classes. What is missing is the CONTEXT and the CHAIN, not the engine: no predefined macros, no
compilation unit (IEEE 1800-2017 §22.5.1 `:272-273` makes a macro visible for the rest of the unit), no
single command, a source map nothing reads, no corpus-scale measurement, and nothing reading the suites' own
`.core` / `.f` context.

**THE DECISION.**
- `SVPP-EXPANSION` is **active now**; statement 3 supersedes statement 1's sequencing. The pipeline sits on
  the SV release path inside the lane lock ([[project_nexsim_sv_signoff_delivery_focus]]) — it is SV work.
- The deliverable is the pipeline as the director described it: a compilation context (predefines, include
  dirs, an ordered unit file list with one macro table), one command that preprocesses in-process and parses
  the expanded text, rejections mapped back to origin `file:line:column` through the source map, and a corpus
  lane that measures end-to-end recognition on the population the raw lane defers.
- Statement 2 is **not overturned and not applied up front**. The driver is engine-agnostic (it composes any
  preprocessor exposing `PreprocessedOutput` with the parser), so the measurement is built on the engine
  that exists. The re-platforming onto the svpp AST is decided on the pipeline's numbers: a residual in the
  single digits argues for fixing the scanner; a class of defects the scanner's shape cannot express argues
  for the 2026-06-09 ruling. That reading goes back to the director with the numbers, as a finding.
- The raw-parse corpus lane (`stimuli/run_external_corpus.sh sv`) stays as grammar-development scaffolding:
  it isolates grammar coverage and its adjudication keeps deferring macro-blocked rows. Its published bar
  is not the release number; the pipeline lane's is.

**CONSEQUENCES.**
- `SV-CORPUS-GRAD.4`'s chaining half and `.13d`'s 1 527 expansion-owned rows are owned by
  `SVPP-EXPANSION.4` / `.6`; the axis-2 denominator moves only through `SV-CORPUS-DENOMINATOR`'s gate.
- The svpp *parser* (the directive-structure grammar) is unchanged by this; its book keeps describing the
  parser, and the pipeline is documented as a sibling deliverable in the SV parser book and the contract.
- A future session that finds the three statements in tension reads this record first: the tension is
  known, recorded, and resolved by measurement.

**Cross-links.** Tree [SVPP-EXPANSION](../tasks/SVPP-EXPANSION.md) (the `.1` record holds every command
and number above). [[project_svpp_expansion_stage_for_nexsim]] (the 2026-06-08 directive, whose deliverable
this keeps and whose sequencing this supersedes). [[feedback_sv_strict_lrm_compliance_default]] (why
`` `include `MACRO `` stays refused by default — ⛔ CORRECTED 2026-10-02 by `SVPP-EXPANSION.5`(m): that
refusal was a misreading. IEEE 1800-2017 §22.5.1 licenses the form in a sentence; the strict default never
required refusing it).
