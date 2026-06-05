---
id: sv-corpus-gate-uvm-memory
title: The SV corpus gate's uvm parse can consume ~26 GB RAM and appear to hang — cap it
answers:
  - "why does the SV external corpus gate eat huge RAM or seem stuck"
  - "why does parsing uvm_pkg / uvm_compat_pkg use tens of GB of memory"
  - "is the SV parser slow / memory-heavy on uvm and why"
  - "how to run the SV corpus gate safely without exhausting host RAM"
tags: [systemverilog, parser, performance, memory, corpus, gate, termination]
date: 2026-06-05
status: current
evidence: docs/tasks/PARSE-TERMINATION.md leaf .3.2 (PGEN-PARSE-TERMINATION-0005); .1 (N^1.66 super-linear) + .3 (O(N^2) SemanticRuntimeState clone)
reverify: `ulimit -v 12582912; target/debug/parseability_probe --parse systemverilog <uvm_pkg.preprocessed.sv> --profile 2017` and watch RSS — slow, and memory climbs super-linearly deeper in the parse
---

**Symptom (observed 2026-06-05):** running `make -C rust sv_external_corpus_triage_gate`, the
`uvm_compat_pkg` bootstrap case (which parses **uvm_pkg.sv ≈ 2.89 MB preprocessed** via the
**debug** `parseability_probe`) climbed to **~26 GB RAM and appeared stuck**. It is **slow, not
strictly hung** — but at that size the host swaps and it looks frozen. Other corpus cases
(scr1 ≈ 0.05 MB, friscv ≈ 0.035 MB) are ~70× smaller and parse fine.

**Why (two cost components — see [[stateful-packrat-not-linear]]):**
1. **Super-linear MEMORY/TIME** — PGEN's *stateful* (semantic-store) packrat is **not linear**
   (Chida & Kawakoya, CC 2020). Measured ~**N^1.66** (PARSE-TERMINATION.1). Root cause pinned
   (PARSE-TERMINATION.3): `with_semantic_runtime_rule_transaction` does a **full
   `SemanticRuntimeState` clone per rule transaction** = O(N²). On uvm's thousands of type
   decls the cloned state balloons → the 26 GB. (Early/shallow parse is modest ~100 MB; the
   blow-up is DEEPER in the parse.) Fix designed = checkpoint/rollback swap (PARSE-TERMINATION.3.1,
   pending).
2. **Regex terminal matching** — a `sample` of the early phase is dominated by `memchr` /
   `aho-corasick` / `regex_automata` DFA (terminal/token matching over the 2.89 MB text); a
   likely second cost (check whether the generated parser recompiles terminal regexes per call,
   cf. the generator fix [[sv-witness-purdom-ordering]] / SV-EXH-PROOF.7.4.6.1).

**How to run it safely (until .3.1 / the .4 watchdog land):**
- Cap memory + time so an OOM is a *classified failure*, not host exhaustion:
  `ulimit -v 12582912` (12 GB) before the run, and/or wrap with `timeout`.
- Prefer a **release** `parseability_probe` for ad-hoc uvm parsing (debug is far heavier).
- Do NOT `timeout` the gate too short (uvm parse is legitimately minutes); but DO cap RAM.
- If a uvm parse hits the cap, that is the **expected super-linear blow-up**, not a new bug —
  it is tracked by PARSE-TERMINATION.3.2/.3.1/.4, not the grammar.

**Not caused by grammar edits** like `ANNOTATION-COMPOSITION.6.4` (those are profile no-ops on
the affected path). This is an engine/AST-pipeline performance issue, parser-agnostic in nature
but most visible on the largest SV input.
