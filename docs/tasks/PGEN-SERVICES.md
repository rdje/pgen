# PGEN-SERVICES: PGEN as a service provider — the register of shared, contracted services

## Metadata

- Tree ID: `PGEN-SERVICES`
- Status: `proposed` — opened 2026-10-19 by DIRECTOR DIRECTION
  ([[project_pgen_is_a_service_provider_shared_contracted_services]]); its first built block is `SVPP-EXPANSION.9`
  (the SV lane's own work, under the lane lock [[project_nexsim_sv_signoff_delivery_focus]]); the register's
  inventory leaf (`.1`) is docs-only and runs beside it.
- Roadmap lane: engine services (cross-family) — the published, contracted half of what `ENGINE-UNIVERSAL-SERVICES`
  made neutral
- Created: `2026-10-19`
- Owner: repo-local workflow
- Priority: **DIRECTOR DIRECTION (2026-10-19, verbatim in the record)** — *"I would like PGEN become a truly a very
  high quality parsing, or compilation and/or elaboration service provider for downstream consumers, projects. If
  PGEN parsers can share services other PGEN parsers, and become like lego blocks where some of this shared services
  can themselve be leveraged by downstream consumers too, that's a win on many account."*

## Goal

Every common part of two PGEN parsers is ONE service behind ONE contracted boundary, and every service a downstream
project could want is reachable through a published, versioned surface — the parsers' own contracts being the model.

## The register (the structure this tree builds)

One row per service, five columns, each MEASURED before it is written: **contract** (the document), **stability
axes** (service release version + output schema version), **neutrality tier** (engine-neutral, or a family component
implementing a neutral trait — [[feedback_neutrality_ladder_before_a_family_capability]]), **consumers** (the PGEN
parsers that use it; the downstream surface that exposes it: Rust typed, by-name, CLI), **currency gate** (what holds
the contract to the code). A service with all five is a BLOCK; one lacking any is a COMPONENT and is listed as such.

## Leaves

### `.1` — the inventory: what PGEN's engine already shares, and which of it is a block today (`todo`, docs only)

Candidates and their state as measured 2026-10-19 (each row to be re-measured by the leaf before it is published):
the preprocessing stage socket (`rust/src/preprocessing_stage.rs`, one implementation — the SystemVerilog family's;
internal; becomes the first block through `SVPP-EXPANSION.9`); the semantic store (four contract documents under
`docs/contracts/PGEN_SEMANTIC_STORE_*`; no downstream surface; currency gate to be measured); source maps and
diagnostics (`StageOutput`'s types, internal); the embedding API (contracted, `rust/docs/EMBEDDING_API_CONTRACT.md`,
versioned — the one block that exists); the parse limits and the dedicated parse stack (contracted inside the API);
the stimuli generator and the certificate (internal; a downstream project certifying its own grammar is a service
PGEN performs on itself today); the AST/JSON carrier (contracted per parser book); the library/stdlib bundle
resolution (SV-only). Output: the register table with the five columns filled from measurement, the components
named as components.

### `.2` — the register as a doctrine (`todo`, after `.1`)

A service row that claims a contract names a currency gate that runs; a published surface without a row is a
finding. The shape is `PARSER-BOOK-CURRENCY` / `PUBLISHED-VERSION-CURRENCY` applied to services.

### The first block

`SVPP-EXPANSION.9` — the svpp expander over the svpp AST, in-process and as a tool, the SV front end on the same
engine ([[project_svpp_expansion_is_a_service_over_the_svpp_ast]]). Its neutral parts (the file provider, include
resolution, source maps with the expansion chain, the event stream) belong to the engine's socket; its SystemVerilog
macro semantics to the family.
