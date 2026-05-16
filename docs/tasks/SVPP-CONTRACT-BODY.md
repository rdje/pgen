# SVPP-CONTRACT-BODY: Bring sv_preprocessor contract to SV parity

## Metadata

- Tree ID: `SVPP-CONTRACT-BODY`
- Status: `active`
- Roadmap lane: sv_preprocessor deliverables
- Created: `2026-05-14`
- Last updated: `2026-05-16`
- Owner: repo-local workflow

## Goal

Body out `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md`
(currently a thin scaffolding) to SV-equivalent depth: typed AST shape,
public API, schema versioning, release identity, gate recipe, and the
64-annotation baseline.

## Non-Goals

- Do not duplicate the future sv_preprocessor mdBook.
- Do not document SV LRM preprocessor semantics outside the current grammar
  scope.

## Acceptance Criteria

- The contract documents the AST envelope, pp_item dispatch (10 kinds),
  directive shapes (define/undef/include/timescale/default_nettype/celldefine),
  conditional-compilation tree, macro_body fragment, and the 64-annotation
  baseline.
- Cross-references to `systemverilog_preprocessor_v1.json` and (future) book.

## Task Tree

- ID: `SVPP-CONTRACT-BODY`
  Status: `active`
  Goal: `Body out the sv_preprocessor integration contract.`
  Children: `SVPP-CONTRACT-BODY.1`, `SVPP-CONTRACT-BODY.2`,
  `SVPP-CONTRACT-BODY.3`, `SVPP-CONTRACT-BODY.4`

- ID: `SVPP-CONTRACT-BODY.1`
  Status: `done`
  Goal: `Audit current contract content; populate Contract Identity + Schema Versioning + Release 1.0.1.`
  Acceptance: `Contract carries explicit version numbers, annotation count, sample input (single_define).`
  Verification: `2026-05-15: Identity (1.0.1), Schema Versioning (1.0.0/0.1.0 rows), Release 1.0.1 Highlights (full 64-annotation surface including pp_item dispatch + 7 directives + conditional tree + macro fragment dispatch) inserted at the top of the existing scope-limited contract.`
  Commit: `SVPP-CONTRACT-BODY-Slice-1`

- ID: `SVPP-CONTRACT-BODY.2`
  Status: `done`
  Goal: `Document AST envelope + pp_item dispatch + per-directive shapes.`
  Acceptance: `Section enumerates 10 pp_item kinds + 7 directive shapes with field lists.`
  Verification: `2026-05-16: Added "## AST Envelope and pp_item Dispatch" (real 4-field AstDumpPayload + truncation envelope + accuracy note; systemverilog_preprocessor_file root; 10-branch pp_item dispatch define/undef/include/timescale/default_nettype/celldefine/endcelldefine/conditional/non_directive_line/blank_line; 7 directive rule shapes pp_define/pp_undef/pp_include/pp_timescale/pp_default_nettype/pp_celldefine/pp_endcelldefine w/ field lists + grammar line refs; Known Defects SVPP-0001 honest, NOT fixed, schema stays 1). Independently verified: section ×1, no dup ## headers, no fabricated-struct reintroduction; inventory cross-check EXACT (generated/systemverilog_preprocessor_return_annotations.json: annotation_count 64, 64-elem array, 27 distinct rules, pp_item 10 branches); release 1.0.1 / schema 1 / 64 / 27 consistent. Lockstep DOC-ENVELOPE-0001: sv_preprocessor book comprehensively closed — 8 src chapters (ast-envelope/changelog-index/examples-conditional/examples-single-define/glossary/json-carrier/schema-versioning/walking-the-ast) reconciled to real AstDumpPayload + canonical pinned-constant walker (const SVPP_AST_SCHEMA_VERSION:u32=1); broad-audit targeted fabricated-residual 0 (5 remaining matches all legitimate disclaimers/truncation); SVPP-0001 framing not regressed. Independently re-ran systemverilog_preprocessor_parser_book_gate → green (mdbook_build + tracked_html_check), HTML deterministic.`
  Commit: `SVPP-CONTRACT-BODY-Slice-2`

- ID: `SVPP-CONTRACT-BODY.3`
  Status: `pending`
  Goal: `Document conditional-compilation tree + macro_body fragment (9 kinds) + macro_default_atom (8 kinds).`
  Acceptance: `Section enumerates all fragment kinds with consumer guidance.`
  Verification: `pending`
  Commit: `pending`

- ID: `SVPP-CONTRACT-BODY.4`
  Status: `pending`
  Goal: `Add gate recipe + manifest cross-reference + README/LIVE_ACHIEVEMENT_STATUS links.`
  Acceptance: `Contract references manifest + future book + lib API entry points.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `SVPP-CONTRACT-BODY.3` | `pending` | Conditional-compilation tree + macro_body fragment (9 kinds) + macro_default_atom (8 kinds) build on the now-existing AST envelope + pp_item dispatch section. |

## Decisions

- `2026-05-14`: Model after the SV contract structure for cross-grammar
  consistency.

## Open Questions

- None blocking.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-15` | `SVPP-CONTRACT-BODY.1` | manual review of inserted sections | `pass — Identity through Release 1.0.1 surface populated at the top of the existing scope-limited contract` |
| `2026-05-16` | `SVPP-CONTRACT-BODY.2` | section-count + dup-`## `-header + no-fabricated-reintroduction grep on contract; inventory cross-check (annotation_count/array-len/distinct-rules/pp_item-branches); broad-audit targeted fabricated residual on book src; SVPP-0001 framing check; independent `systemverilog_preprocessor_parser_book_gate` re-run | `pass — AST Envelope + pp_item Dispatch section (real AstDumpPayload, 10 pp_item kinds, 7 directive shapes, SVPP-0001 honest/not-fixed/schema-1); inventory EXACT 64/27/10; sv_preprocessor book DOC-ENVELOPE-0001 comprehensively closed (8 chapters, targeted fabricated-residual 0, 5 legitimate disclaimers/truncation); gate green; HTML deterministic` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `SVPP-CONTRACT-BODY.1` | `SVPP-CONTRACT-BODY-Slice-1` | Contract Identity + Schema Versioning + Release 1.0.1 Highlights (full 64-annotation surface) inserted at the top |
| `SVPP-CONTRACT-BODY.2` | `SVPP-CONTRACT-BODY-Slice-2` | AST Envelope + pp_item Dispatch (10 kinds + 7 directive shapes, SVPP-0001 honest); sv_preprocessor book DOC-ENVELOPE-0001 comprehensively closed in lockstep (8 chapters, fabricated-residual 0, gate green) |

## Changelog

- `2026-05-14`: Created task tree.
- `2026-05-16`: `.2` done (AST Envelope + pp_item Dispatch section — 10 pp_item kinds + 7 directive shapes + SVPP-0001 honest/not-fixed/schema-1; inventory cross-check EXACT 64/27/10; sv_preprocessor book `DOC-ENVELOPE-0001` comprehensively closed in lockstep — 8 chapters, targeted fabricated-residual 0, independent gate re-run green). Frontier advances to `.3` (conditional-compilation tree + macro_body fragment + macro_default_atom). Tree stays `active` (`.3`, `.4` remain).
