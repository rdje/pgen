# VERILOG-AMS — an EBNF-driven Verilog-AMS (Accellera LRM 2023) analog/mixed-signal parser family

## Metadata

- Tree ID: `VERILOG-AMS`
- Status: `proposed` (own-now / **build-later**, director-agreed 2026-06-21). Sequenced STRICTLY AFTER
  the locked program (all existing parsers cert-coverage clean + `UNKNOWN`=0), the same bucket as
  `SVPP-EXPANSION` / `PNR-*`. Activates (frontier `.1` SCOPING, pure docs) only when prioritized —
  NOT PNT-eligible until then. No code exists for this lane.
- Family / slice-id prefix: `PGEN-VERILOG-AMS-<NNNN>`
- Roadmap lane: Phase S — HDL parser family (analog/mixed-signal), alongside SystemVerilog, VHDL,
  and Verilog-2005
- Created: `2026-06-21`
- Owner: repo-local workflow
- Disciplines: [[feedback_research_grounded_sota_no_trial_and_revert]] (ground the grammar in the
  Accellera Verilog-AMS LRM before extraction), [[project_ebnf_is_single_source_of_truth]],
  [[feedback_tools_first_no_guessing]], [[feedback_always_signoff_decisions]],
  [[feedback_ebnf_meta_grammar_lockstep]] (any new EBNF construct ports to `ebnf.ebnf` same wave).

## The frame (why this tree exists)

A user surfaced the **Accellera Verilog-AMS LRM 2023** (2026-06-21) and asked whether it is of
interest. Assessment: yes, and roadmap-aligned in spirit. Verilog-AMS is a recognized Accellera
standard and the analog/mixed-signal superset of Verilog (disciplines/natures, `analog` blocks,
`wreal`, branch contributions `V(a,b) <+ …`, `@(cross/timer/above)` events, analog operators/filters,
connect modules). It fits PGEN's north-star of best-in-class EBNF-driven HDL parsers, and a
mixed-signal front-end is directly relevant to **NEXSIM** (a primary integration target).

It is captured here so the idea survives session loss and is ready to scope on the director's word —
NOT as a pivot. Per the binding 2026-06-17 priority order, the active lanes are SV `UNKNOWN`→0
(`GRAMMAR-WELLFORMED.H.12`), then the SVA infix parse bug (`H.12.5.8`), then PARSE-COMPLETENESS;
this lane does not displace them.

## Goal

Deliver a professional-grade, EBNF-driven Verilog-AMS parser family judged against the same PGEN
closure bar as the other shipped families:
- `grammars/verilog_ams.ebnf` (LRM-grounded, return + semantic annotations),
- `generated/verilog_ams_parser.rs` + stimuli path,
- certificate-coverage `fully_certified` (UNKNOWN=0, deterministic seeds 0/7/42),
- a per-parser mdBook (`docs/verilog_ams_parser_book/`) + downstream integration contract +
  AST shape-contract manifest + a family gate,
- internal stimuli-generator AND an officially-recognized external corpus
  ([[project_external_corpus_doctrine]]).

## Non-Goals (for the stub; revisit at scoping)

- Analog simulation / SPICE-level numeric solving — PGEN parses + shapes the AST; it does not solve.
- Deciding the exact synthesizable/behavioral subset — that is the `.1` scoping leaf's job.
- Vendoring the LRM PDF now — DEFERRED until the family is scoped (the reference PDF is currently
  user-local at `~/Documents/livework/chipdoc/eda/accellera/verilog-ams/current/VAMS-LRM-2023.pdf`;
  if adopted, vendor it git-ignored like the IEEE 1800 / VHDL LRM PDFs per `-0109`).

## Reuse opportunities (to evaluate at scoping)

- The `verilog_2005` / `systemverilog` lexical model and many digital constructs are shared bases —
  Verilog-AMS extends Verilog rather than replacing it.
- The parser-agnostic AST pipeline, return/semantic annotation languages, semantic store, and the
  cert-coverage + external-corpus machinery all apply unchanged.

## Task Tree

- ID: `VERILOG-AMS`
  Status: `proposed`
  Goal: a fully-certified EBNF-driven Verilog-AMS parser family
  Children: `VERILOG-AMS.1`

- ID: `VERILOG-AMS.1`
  Status: `pending` (not activated)
  Goal: SCOPING — the analog/mixed-signal subset to target vs the Accellera Verilog-AMS LRM 2023;
  reuse-vs-fork decision against `verilog_2005`/`systemverilog`; the closure bar; and the ordered
  build leaves (`.2`+). Pure docs.
  Acceptance: a scoping doc with the subset, reuse decision, closure bar, and a leaf plan.
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `VERILOG-AMS.1` | `pending` (not activated) | Build-later; activates only when the director prioritizes it, after the locked program. |

## Decisions

- `2026-06-21`: Captured as `proposed` / own-now-build-later on director agreement after the
  Verilog-AMS LRM 2023 was surfaced. Sequenced strictly after the locked program; PDF vendoring
  deferred to scoping. Promote to a `docs/decisions/` project record when the family is scoped.

## Open Questions

- Which Verilog-AMS subset (full LRM vs a behavioral/synthesizable subset) — owner: director at
  `.1` scoping; does not block the frontier (lane is build-later).

## Blockers

- Sequencing only: build-later, after the locked program. No technical blocker.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-21` | `VERILOG-AMS` | stub creation (pure docs) | created |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `VERILOG-AMS` | `PGEN-VERILOG-AMS-0001` | proposed stub created |

## Changelog

- `2026-06-21`: Created proposed task-tree stub (`PGEN-VERILOG-AMS-0001`).
