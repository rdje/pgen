# docs/contracts/PGEN_PNR_PARSER_INTEGRATION_CONTRACT.md

## Purpose
Record the PGEN-side view of PNR's inbound parser integration contract.

PNR authored a dedicated `PGEN_INTEGRATION.md` in the PNR repository on 2026-04-10. That file is the downstream source of truth for what PNR expects from PGEN. This document exists so PGEN sessions can discover and track that contract from inside the PGEN repo without relying on chat history.

## Contract Identity
- Downstream consumer:
  - PNR
- Contract class:
  - downstream project integration request
- PNR source document:
  - PNR repository `PGEN_INTEGRATION.md`
- Related sibling document:
  - PNR repository `RTLSYN_INTEGRATION.md`
- Inbound revision date:
  - 2026-04-10
- PGEN status:
  - accepted as downstream demand signal
  - not yet a shipped parser-family integration surface

## Source Of Truth
The authoritative request remains PNR's repo-local `PGEN_INTEGRATION.md`.

This PGEN-side document captures the current operating summary only:
- parser families PNR expects,
- API/release shape PNR wants,
- acceptance and validation model,
- cross-project constraints involving RTLSyn.

When this document and PNR's file disagree, re-read PNR's file first and update this PGEN-side mirror intentionally.

## Required Parser Families
PNR requests PGEN-generated parser crates for:
- LEF
- DEF
- Liberty
- SDC
- Verilog structural netlist subset, backed by broader SystemVerilog grammar work where appropriate
- SPEF

PNR's preferred delivery order is:
- LEF
- Liberty
- DEF
- Verilog
- SDC
- SPEF

The first minimum viable PNR-facing milestone is LEF:
- `parse_str`
- `parse_file`
- concrete `ParsedLef` AST
- structured `ParseError`
- deterministic AST/data structures
- acceptance fixture proving `sky130_fd_sc_hd.tlef` parses without diagnostics
- tagged PGEN release consumable by PNR as a git submodule

## Authoritative Source Requirements
Do not start a new PNR-facing EBNF family from tutorials, blog posts, random mirrored PDFs, or open-source parser behavior alone. Those are useful for examples and conformance checks, but not for grammar authority.

Repository inventory note (2026-04-10):
- Verilog/SystemVerilog already has tracked IEEE-derived EBNF in `grammars/`.
- The currently tracked root-level EBNF inventory does not include LEF, DEF, Liberty, SDC, or SPEF EBNF files yet.

The current source-of-authority matrix is:

| Format | Primary authority to acquire before EBNF work | Notes |
|---|---|---|
| LEF / DEF | Si2 LEF/DEF downloads: `https://si2.org/lef-def-downloads/` | Si2 states LEF/DEF are developed by Cadence and distributed by Si2. Acquire the exact API Format Specification Reference Documentation matching the PNR target, initially LEF/DEF 5.8 unless PNR revises the target. |
| Liberty | Synopsys TAP-in Liberty download: `https://www.synopsys.com/community/interoperability-programs/tap-in.html` | TAP-in is the official Synopsys access path for Liberty under the relevant open-source license. Do not use random web copies as the normative grammar source. |
| SDC | Synopsys TAP-in SDC download: `https://www.synopsys.com/community/interoperability-programs/tap-in.html` plus official Tcl syntax docs | SDC is Tcl-shaped. Use Synopsys SDC for command semantics and Tcl language docs for tokenization/quoting/substitution behavior. |
| Verilog / SystemVerilog | Already available locally from IEEE-derived workspaces: `grammars/systemverilog.ebnf`, `grammars/systemverilog_2017_lrm_extracted.ebnf`, `grammars/systemverilog_2023_lrm_extracted.ebnf`, `grammars/verilog_2005_lrm_extracted.ebnf`, `docs/systemverilog/2017`, `docs/systemverilog/2023`, and `docs/verilog/2005` | No new standards-source acquisition is needed for this family. PNR netlist work should be a constrained profile/subset and fixture/gate task over the existing standard-derived surface. If the Verilog 2005 snapshot is promoted directly, its known terminal-normalization debt still needs closure. |
| SPEF | IEEE 1481 / OLA standard lineage: `https://standards.ieee.org/ieee/1481/7651/` | PNR names IEEE 1481-2009. IEEE now lists 1481-2019 as active and superseding 1481-2009, so pin the exact SPEF/OLA revision with PNR before writing EBNF. |

Useful non-authoritative aids:
- OpenROAD / OpenDB parsers for conformance comparison and edge-case discovery,
- sky130, OpenROAD-flow, PicoRV32, Ibex, and OpenTitan fixtures for acceptance and regression,
- vendor application notes for examples.

Rule: if a source cannot be redistributed or checked into PGEN, record its acquisition path and version, then derive only the PGEN-owned EBNF and tests from it. Do not copy proprietary standard text into repository docs.

## Stable Integration Surface Target
PNR prefers one Rust crate per format, with stable crate paths suitable for submodule consumption:
- `crates/pgen-lef`
- `crates/pgen-def`
- `crates/pgen-liberty`
- `crates/pgen-sdc`
- `crates/pgen-verilog`
- `crates/pgen-spef`

This is not yet implemented in PGEN. It is a downstream target shape and should be reconciled with PGEN's actual workspace layout before any release claim.

Each parser crate should eventually expose:
- concrete public AST types, not opaque handles,
- `parse_str(input, file_label)` and `parse_file(path)` entry points,
- structured `ParseError` with file/label, line, column, byte offset, expected productions, found token, and context snippet,
- `Debug`, `Clone`, `PartialEq`, and thread-safe AST payloads,
- deterministic collection choices with no exposed `HashMap` / `HashSet` iteration-order dependence.

For writable PNR formats, PNR also requests canonical emission APIs:
- DEF
- SPEF
- optionally Verilog
- optionally SDC

The required emission invariant is AST semantic roundtrip, not byte-for-byte preservation of original source comments/formatting.

## Build / Availability Requirements
PNR consumes PGEN through a pinned git submodule, not by copying parser code into PNR:
- PNR pins PGEN to a tagged release.
- PNR does not pin to a mutable branch.
- PGEN owns parser crates and AST types.
- PNR owns the bridge from PGEN ASTs into PNR's canonical IR.
- `eda-db` is not a PGEN integration surface. Any `eda-db` coordination belongs outside PGEN, between downstream projects such as RTLSyn and PNR.
- PGEN parser crates remain independent of downstream database crates.

PNR strongly prefers committed generated Rust parser source over requiring PNR to run PGEN at build time.

## Validation / Release Gates
PNR's requested acceptance suite includes:
- sky130 LEF technology/stdcell fixtures,
- sky130 Liberty timing corners,
- OpenROAD-flow DEF fixtures,
- PicoRV32 / Ibex / OpenTitan Verilog fixtures,
- OpenROAD-flow SDC fixtures,
- PicoRV32 SPEF once extraction lands.

PGEN-side release candidates for these parser families should not claim PNR readiness until they have machine-checkable gates for:
- parse acceptance per fixture family,
- structured error behavior on malformed inputs,
- determinism,
- roundtrip AST equality for emitted formats,
- no panic on malformed input.

## RTLSYN Coordination Note
PNR's `RTLSYN_INTEGRATION.md` names PGEN as the shared parser source of truth for NexSim, RTLSyn, and PNR.

The RTLSYN-facing document adds these PGEN-relevant constraints:
- RTLSyn and PNR should consume the same PGEN-generated parser crates.
- PGEN's Verilog/SystemVerilog parser must accept the long-term PGEN-era structural netlist target documented by PNR.
- Before PGEN cutover, RTLSyn may target PNR's strict hand-written scaffold subset, but that scaffold is not PGEN's final bar.
- The `eda-db` question is not a PGEN contract surface. If RTLSyn needs to care about `eda-db`, that coordination happens outside PGEN.

## Scope / Non-Goals
PGEN does not own:
- PNR's AST-to-IR bridge,
- cross-format PDK consistency validation,
- Liberty/LEF/DEF semantic consistency checks,
- SDC object resolution against PNR design databases,
- PNR flow execution, placement, routing, timing, extraction, or signoff engines.

PGEN's job is to deliver faithful, deterministic parser and emitter surfaces with explicit contracts and release discipline.

## Current Actionable Consequences For PGEN
- Treat LEF as the first PNR-specific parser-family candidate once current closure sequencing permits.
- Keep Liberty high in the synthesis/signoff parser backlog because both PNR and RTLSyn need it.
- Add DEF and SPEF as explicit PNR-driven future parser families, not vague "EDA someday" work.
- Treat Verilog structural netlist parsing as a downstream PNR/RTLSyn handoff concern, distinct from the richer `rtl_frontend` elaboration boundary.
- Do not plan PGEN work around `eda-db`; PGEN remains responsible for parsers, ASTs, diagnostics, emitters, fixtures, and release contracts.
- Before advertising any PNR-ready release, create family-specific contract documents and gates in the same style as the existing parser-family integration contracts.
