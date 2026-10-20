# docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md

## Purpose
Define the version-controlled downstream integration-contract surface for parser families published by PGEN.

These documents exist so a downstream project can be pointed at one stable `.md` file for the parser family it wants to consume, instead of reverse-engineering readiness from chat history, scattered gate names, or internal Rust modules.

Issue-reporting protocol for any integrated parser:
- `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`
- `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`

## Rule
- Every current and future parser family that PGEN publishes for downstream consumption must have a tracked integration-contract document.
- If a parser family does not yet have a stable downstream host API, its family document must say that explicitly instead of implying readiness.
- A parser family should not claim painless downstream integration unless its family document, its stable API surface, and at least one executable gate agree.
- If a parser family is actively published for downstream consumption, its family document must publish:
  - a contract version
  - a parser release version
  - a last-updated stamp
- Parser release versions are not just git provenance aliases:
  - downstream bug reports should cite them
  - fixes should land in them
  - contract-changing or capability-widening releases should bump them intentionally

## Current Family Documents

| Family | Integration Contract | Primary Stable Surface | Notes |
|---|---|---|---|
| `systemverilog` | `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` | `pgen::embedding_api` | Nexsim-facing host profile contract. |
| `systemverilog_preprocessor` (the preprocessing SERVICE — the expander over the svpp AST) | `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_EXPANSION_CONTRACT.md` | today: `pgen::preprocessing_stage::stage_for("systemverilog")` in-process and `ast_pipeline --preprocess systemverilog` as a tool; the typed API (`pgen::families::systemverilog::preprocessing_service::{preprocess, preprocess_str}` over a `FileProvider`), `parseability_probe … --preprocess --emit-preprocessed [--emit-source-map]` and the typed refusal since `SVPP-EXPANSION.9.2` | `1.0` (2026-10-20; held to the code by `SVPP-EXPANSION-CONTRACT-CURRENCY`); the expander over the svpp AST itself is `.9.3` |
| `systemverilog_preprocessor` | `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` | `rust/src/families/systemverilog/preprocessor.rs` runtime stage (the SystemVerilog family's component since `ENGINE-UNIVERSAL-SERVICES.113`) | Explicitly documents that a generic public embedding API is not published yet. |
| `vhdl` | `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md` | `pgen::embedding_api` | Nexsim-facing host profile contract. |
| `regex` | `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md` | `pgen::embedding_api` | Downstream-ready regex contract for RGX and other regex consumers; the current release/contract versions are declared AUTHORITATIVELY in that document's Contract Identity block (kept in machine lockstep with `parser_embedding_api_contract()` by the `PGEN-RGX-0091` drift gate) — as of 2026-07-21: release `1.1.106`, integration contract `1.1.109`. |
| `return_annotation` | `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md` | `pgen::embedding_api` annotation parse API | Family-specific contract layered on top of the aggregate annotation proof spine. |
| `semantic_annotation` | `docs/contracts/PGEN_SEMANTIC_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md` | `pgen::embedding_api` annotation parse API | Family-specific contract layered on top of the aggregate annotation proof spine. |

## Downstream Project Contract Documents

| Downstream | Contract | Status | Notes |
|---|---|---|---|
| `PNR` | `docs/contracts/PGEN_PNR_PARSER_INTEGRATION_CONTRACT.md` | Demand captured; parser-family releases pending | Captures PNR's 2026-04-10 inbound request for LEF, DEF, Liberty, SDC, Verilog structural netlist, and SPEF parser crates plus API/release expectations. |

## Required Shape For Each Family Document
- `Purpose`
- `Contract Identity`
- `Source Of Truth`
- `Stable Integration Surface`
- `Build / Availability Requirements`
- `Validation / Release Gates`
- `Scope / Non-Goals`

Recommended additions when a family is actively consumed by another project:
- a downstream-specific checklist
- representative pass/fail examples
- explicit startup checks
- a short “what this does not promise” section
- a pointer to `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`

## Engine-Wide Behaviour Changes

Some behaviour a consumer sees belongs to the ENGINE, not to any one grammar: the generated parsers
share a built-in layout skipper, so a change there moves the accept set of every family whose grammar
does not claim the introducer as a token of its own. Such a change has no grammar revision and no
per-family release to hang itself on, so it is recorded here once, per-family ledger rows link to it,
and each family's own contract keeps its own version.

### 2026-09-15 — an unterminated block comment is no longer layout (`ENGINE-UNIVERSAL-SERVICES.107`)

Through the artifacts shipped before this date, the built-in comment arm advanced past a `/*`, scanned
for `*/`, and — on reaching the end of input without one — KEPT the bytes consumed. An unterminated
block comment was therefore layout: a file whose entire text is `/*` parsed, and so did any file that
ended inside a comment. IEEE 1800-2017 §5.4 states the rule in one sentence (*"A block comment shall
start with /\* and end with \*/"*), and every grammar in the tree that spells its own comment token
agrees by requiring the closer; the bytes simply never reached it. The arm now REWINDS to the opener
and stops skipping, so the grammar's own terminal refuses the text where it stands.

**What a consumer sees.** Input that ends inside a block comment is now REJECTED, at the `/*` rather
than at end of input. Nothing else moves: the skipper consumes strictly less, so the accept set can
only narrow, and every input whose verdict is unchanged takes the identical path. Measured on the
SystemVerilog corpus (16 336 files, three profiles) against a parser built from the previous commit:
`ast_moved=0` on all three profiles and exactly three files' verdicts moved, each one a file ending
inside a comment.

**Which families moved, measured per family rather than assumed** (a file ending in `/*` versus the
same file with the comment closed, on the previous artifact and on this one):

| Family | Artifact re-emitted | Accept set moved | Evidence |
|---|---|---|---|
| `systemverilog` | yes | yes | corpus lanes `sv_2017` 9 847 → 9 844, `sv_2023` 9 872 → 9 869, `verilog_2005` unchanged; ledger `SV-0107` |
| `vhdl` | yes | yes | `entity e is end entity; /*` PASS → REJECT, the closed form unchanged; ledger `VHDL-0007` |
| `return_annotation` | yes | yes | `{kind: "x"} /*` PASS → REJECT, the closed form unchanged; ledger `RETANN-0002` |
| `rtl_frontend` | yes | yes | `module m; endmodule /*` PASS → REJECT, the closed form unchanged |
| `rtl_const_expr` | yes | yes | `1 + 2 /*` PASS → REJECT, the closed form unchanged |
| `json` | yes | yes | `{"a": 1} /*` PASS → REJECT, the closed form unchanged |
| `systemverilog_preprocessor` | yes | **not observed** | its own `inline_trivia` claims the bytes before the engine's skipper is reached; four shapes tried (`` `define A 1 /* ``, `` `timescale 1 ns / 1 ps /* ``, `` `include "x.svh" /* ``, a bare `/*` line) and all four parse before and after |
| `regex` | no | no | `@whitespace_sensitive`: it emits no layout skipper at all |
| `semantic_annotation` | no | no | declares `"/*"` as its own literal token, which suppresses the engine's arm |

(`ebnf` and `scratch` are not published families; both declare `"/*"` themselves and re-derive
byte-identically.)

## Operational Rule
- When a downstream project asks “how do I integrate parser X?”, point it first to the family document listed here.
- When a downstream project reports a released-parser bug, require the repro bundle from `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md` and log the accepted report in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` under the matching parser family/profile.
- GitHub is not required for that loop; local git-tracked records in PGEN plus zero-or-more downstream consumer repos are sufficient.
- When the family document and the code disagree, fix the upstream source of truth first, then bring the family document back into parity.
- When a new parser family becomes a real downstream target, add its family document here in the same change that exposes or stabilizes the host surface.
