# PGEN-SERVICES: PGEN as a service provider — the register of shared, contracted services

## Metadata

- Tree ID: `PGEN-SERVICES`
- Status: `active` — `.1` DONE 2026-10-19 (`PGEN-SV-CORPUS-GRAD-0471`, the first measured register: 4 blocks, 7 components, 1 specification, 1 finding routed to `.3`); opened 2026-10-19 by DIRECTOR DIRECTION
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

### `.1` — the inventory: what PGEN's engine already shares, and which of it is a block today (✅ `done` 2026-10-19, `PGEN-SV-CORPUS-GRAD-0471`; docs only)

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

#### The register, first measurement (2026-10-19; every cell re-derivable by the command in its note)

The candidate list is DERIVED from the engine-neutrality register's file classes
(`rust/test_data/grammar_quality/engine_neutrality_register_v0.json`: `api`, `harness`, `registry`, the engine
modules under `rust/src`, and the per-family components under `rust/src/families/`), not recalled. **Verdict**: a
BLOCK has all five columns; a COMPONENT lacks at least one; a SPEC has a contract and no implementation.

| # | Service | Contract | Stability axes | Neutrality tier | Consumers | Currency gate | Verdict |
|---|---|---|---|---|---|---|---|
| 1 | **Embedding API** — parse, AST dump, typed outcomes, by-name forms (`rust/src/embedding_api.rs`) | `rust/docs/EMBEDDING_API_CONTRACT.md` | API `2.2.0`; per family: parser release + AST-dump schema | engine (register class `api`; names no language since `2.0.0`) | the six published families' books; downstream hosts (Nexsim) | `embedding_api_gate` (two `cargo test --lib embedding_api` runs) + the `PUBLISHED-VERSION-CURRENCY` doctrine | **BLOCK** |
| 2 | **AST / JSON carrier** — `AstDumpPayload`, the typed shapes, the declared-annotation inventories | the embedding contract + each family's integration contract + each book's `json-carrier.md` (6 books) | per-family AST-dump schema version | engine carrier, per-family shapes | the six families; downstream | `PARSER-BOOK-CURRENCY`; the ast-shape-contract manifests (regression tests) | **BLOCK** |
| 3 | **Parse limits + the dedicated parse stack** — `ParseLimits`, `rust/src/dedicated_parse_stack.rs` (`derive_parse_stack_reservation`, `resolved_parse_stack_reservation`) | the embedding contract (`1.3.x`, `1.3.6`: the reservation derived in both directions) | the API version | engine (every family's hooks take the host-derived default) | every family; 4 modules read `ParseLimits`; downstream hosts | `embedding_api_gate` | **BLOCK** (inside row 1's contract) |
| 4 | **SystemVerilog standard-library bundle** — embedded in the binary, `PGEN_PARSER_LIBS_DIR` the one override | the embedding contract (`2.1.0`) + the SV integration contract | the API version; the SV parser release | family component (`families/systemverilog/mod.rs`, `stdlib_bundle_dir`) | the SV parser (`--lib-in`); the pipeline lanes | `embedding_api_gate`; `SV-CONTRACT-CURRENCY` | **BLOCK**, family-scoped |
| 5 | **Preprocessing** — the neutral stage socket (`rust/src/preprocessing_stage.rs`) + the SystemVerilog expander (`families/systemverilog/preprocessor.rs`) | `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_EXPANSION_CONTRACT.md` — DRAFT `0.1` (`SVPP-EXPANSION.9.1`) | contracted for `.9.2`: a service release version; output schema `1` → `2` at `.9.3` | socket: engine; implementation: SV family component | the SV parser's front end; downstream via the socket and `ast_pipeline --preprocess` | none yet (`.9.2` wires it) | **COMPONENT** → block at `SVPP-EXPANSION.9.2` |
| 6 | **Source text reading** — `pgen::source_text` (`decode_source_bytes`, `read_source_file`, `DecodedSource`: the reader PGEN uses itself; vendor RTL with non-UTF-8 bytes in comments) | NONE — published in the main book's embedding chapter only; absent from `rust/docs/EMBEDDING_API_CONTRACT.md` (`grep -c source_text` → 0) | none | engine | the probe and the SV lanes; downstream hosts, whom the book tells to use it | none | **COMPONENT** — ⚠️ FINDING: a published surface without a contract row; routed to `.3` |
| 7 | **Stimuli generation + the certificate** — `rust/src/ast_pipeline/stimuli_generator.rs`; `--generate-stimuli`, `--generate-stimuli-module`, `--report-certificate-coverage` | none as a service (the published certification table is held by `GRAMMAR-CERT-CURRENCY`) | none | engine (grammar-driven) | every tracked grammar's gates (`ebnf_stimuli_quality_gate`, `stimuli_module_parity_gate`, 3 scripts read the certificate) — PGEN on itself | `GRAMMAR-CERT-CURRENCY` for the table | **COMPONENT** — candidate: a downstream project certifying ITS grammar is this service's external use |
| 8 | **Semantic store** | `docs/contracts/PGEN_SEMANTIC_STORE_{API_CONTRACT,PERFORMANCE_CONTRACT,SCHEMA_LANGUAGE_SPEC,TEST_PLAN}.md` (spec `0.1.0`) | the spec version | engine by design | NONE — no implementation under `rust/src` (`grep -rln 'SemanticStore\|semantic_store' rust/src` → nothing) | the push mirror checks only that `README.md` references the four files | **SPEC** (contract without implementation) |
| 9 | **The family socket** — `FamilyHooks` (`pre_check`, `dedicated_stack`, `pre_parse`, `post_parse`, `contract`, `preprocessing`) | the `ENGINE-NEUTRALITY` register's component rule | none | engine | `regex`, `systemverilog` (hooks); `vhdl` (`HOOKS = NONE`) | the `ENGINE-NEUTRALITY` doctrine | **COMPONENT** — an internal boundary, not a downstream service |
| 10 | **The parse harness** — `parse_harness.rs`, combinator suite, equivalence, interpreter, semantic suite | none | none | engine (register class `harness`) | every family's gates | the gates themselves | **COMPONENT** — internal |
| 11 | **Keyword version tables** — `families/systemverilog/keyword_versions.rs` (IEEE 1800 / 1364 reserved words per `version_specifier`) | the SV integration contract's §22.14 text | the SV parser release | family component | the expander (`` `begin_keywords ``) — its only reader (`grep -rln keyword_versions:: rust/src`) | `SV-KEYWORD-VERSIONS` (equal to the in-repo LRM tables) | **COMPONENT** with a currency gate — candidate: *"is X reserved under version V"* as a downstream query |
| 12 | **Parser generation from EBNF** — `ebnf_frontend.rs`, the self-hosted meta-grammar (`grammars/ebnf.ebnf`), `ast_pipeline INPUT.ebnf --generate-parser` | partial: the two annotation contracts (`PGEN_RETURN_ANNOTATION_…`, `PGEN_SEMANTIC_ANNOTATION_…`); the EBNF dialect in the main book; no service version | none for the generator | engine | every family; a downstream project authoring a grammar for PGEN | the self-hosting dual run; `EBNF-ALTERNATIVE-PARITY` | **COMPONENT** (contract partial) — the most fundamental service PGEN offers, unversioned as one |

**What the measurement says.** PGEN has exactly one contracted service family today — the embedding API and the three
things contracted inside it (rows 1 – 4) — and it is the model: a versioned document, a gate that holds it, a surface
every parser shares. Everything else is engine-internal and uncontracted (rows 7, 9, 10, 12), a draft (row 5), a
specification without code (row 8), or published without a contract (row 6). The director's direction is served, in
order, by: row 5 (`SVPP-EXPANSION.9.2`, in flight); row 6 (`.3`, small, additive); then the two candidates (rows 7
and 12) once a downstream project names the need.

### `.3` — `pgen::source_text` into the embedding contract (`todo`, opened 2026-10-19 by `.1`; additive, `2.3.0`)

`decode_source_bytes` / `read_source_file` / `DecodedSource` are a published downstream surface (the main book's
embedding chapter instructs hosts to use them) with no contract entry, no version axis and no currency gate. Owed: an
additive `2.3.0` entry in `rust/docs/EMBEDDING_API_CONTRACT.md` naming the three items and their guarantees (UTF-8 with
the documented non-UTF-8 handling), a test under `embedding_api_gate`, and the row 6 verdict re-measured to BLOCK.

### `.2` — the register as a doctrine (`todo`, after `.1`)

A service row that claims a contract names a currency gate that runs; a published surface without a row is a
finding. The shape is `PARSER-BOOK-CURRENCY` / `PUBLISHED-VERSION-CURRENCY` applied to services.

### The first block

`SVPP-EXPANSION.9` — the svpp expander over the svpp AST, in-process and as a tool, the SV front end on the same
engine ([[project_svpp_expansion_is_a_service_over_the_svpp_ast]]). Its neutral parts (the file provider, include
resolution, source maps with the expansion chain, the event stream) belong to the engine's socket; its SystemVerilog
macro semantics to the family.
