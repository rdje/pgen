---
name: project-json-full-standard-proof
description: STANDING GOAL (director 2026-06-08) — grammars/json.ebnf SHALL, at some point, FULLY match the official JSON standard (RFC 8259 / ECMA-404), not just its lexical/syntactic core. JSON is the deliberate proof vehicle: a small, complete, well-corpus'd standard PGEN must handle "with no sweat" — full conformance demonstrates PGEN can implement a complete language standard end-to-end. Acceptance = JSONTestSuite fully green (every y_ accepted, every n_ rejected incl. the trailing-content + deep-nesting cases, i_ documented), no crashes. Owned by EXTERNAL-CORPUS (umbrella .2F; sub-leaves .2a done, .2b/.2c pending).
metadata:
  node_type: memory
  type: project
  director_directive: true
  created: 2026-06-08
  owning_tree: EXTERNAL-CORPUS
---

**THE GOAL (binding, director 2026-06-08).** `grammars/json.ebnf` shall, **at some point, FULLY match the
official JSON standard** (RFC 8259 / ECMA-404) — not merely its lexical/syntactic core. The point is a
**proof of capability**: "we need to prove that PGEN can handle [a] full standard with no sweat." JSON is
the deliberate demonstrator — a *small, complete, independently-corpus'd* standard — so reaching **full**
conformance on it is a clean, objective demonstration that PGEN (EBNF + the AST pipeline + the stimuli
generator) can implement a complete language standard end-to-end and prove it.

**Why JSON is the right proof vehicle.** It is the smallest fully-specified mainstream standard with a
recognized, adversarial external corpus (JSONTestSuite — *"Parsing JSON is a Minefield"*). If PGEN can drive
that corpus fully green from an EBNF grammar, it is concrete evidence the platform handles complete
standards — a credibility anchor for the larger standards (SystemVerilog/IEEE 1800, VHDL, PCRE2) where full
conformance is far harder to demonstrate so cleanly.

**Acceptance (objective, the corpus is the oracle).** Re-running `json_corpus_bundle/scripts/run_json_corpus.sh`:
- **`y_` 95/95** accepted — **DONE** (`EXTERNAL-CORPUS.2a`, the RFC-8259 lexical upgrade).
- **`n_` 188/188** rejected — every must-reject file rejected, including:
  - the 5 trailing-content / comment cases (`{"a":"b"}//`, `…#`, `/*comment*/`) → strict end-of-input
    (`EXTERNAL-CORPUS.2c`);
  - the 2 deep-nesting `n_` files that currently **crash** → rejected, not crashed (`EXTERNAL-CORPUS.2b`).
- **no crashes** on any file (incl. the 1 `i_` deep-nesting crash) — `EXTERNAL-CORPUS.2b`.
- `i_` (implementation-defined) outcomes documented (either verdict is acceptable; record PGEN's choice).
- *(stretch)* the cert-coverage RAW-witness `sample_parse_failures` driven back toward 0 (the generator
  faithfully samples the stricter terminals — the G.4.7/F2 raw-generation-fidelity item).

**Status (2026-06-08).** `y_` 95/95, `n_` 181/188 (5 trailing + 2 crash remain), 3 crashes. The lexical
standard is met; the residual is **structural robustness** (end-of-input + recursion guard), tracked by
`EXTERNAL-CORPUS.2c` + `.2b`, rolled up under the umbrella `EXTERNAL-CORPUS.2F`.

**How to apply.** Treat json as a held-to-the-full-standard parser (not a "simplified built-in" anymore):
when the residual leaves land, re-run the corpus and only then describe json as *fully* standard-conformant
in the live tracker / book. Composes with [[project_external_corpus_doctrine]] (the corpus is the external
oracle) and [[feedback_corpus_expected_from_spec_not_fix]] (never derive expecteds from the parser).
