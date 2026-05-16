# SVPP-CONTRACT-BODY: Bring sv_preprocessor contract to SV parity

## Metadata

- Tree ID: `SVPP-CONTRACT-BODY`
- Status: `done`
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
  Status: `done`
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
  Status: `done`
  Goal: `Document conditional-compilation tree + macro_body fragment (9 kinds) + macro_default_atom (8 kinds).`
  Acceptance: `Section enumerates all fragment kinds with consumer guidance.`
  Verification: `2026-05-16: Added "## Conditional Compilation and Macro Body Fragments" (after AST-Envelope, before Source Of Truth): the 5-rule conditional tree (pp_conditional/pp_if_branch/pp_elsif_branch/pp_else_branch/pp_endif, field lists + grammar line refs + reconstruction walk), macro_body_fragment 9 kinds + macro_default_atom 8 kinds (per-kind detail + grammar refs + the verified no-"comma"-kind/macro_default_text nuance), and a pp_define-body consumer-walk paragraph. SVPP-0001 framed in fragment context (pp_if_branch.keyword inline-alternation-$N, malformed nested object w/ three <invalid_sequence_access>; NOT fixed, schema stays 1; referenced by anchor + bug ledger, not duplicated). Independently verified: section ×1, no dup ## headers, no fabricated-struct reintroduction; inventory cross-check EXACT (generated/systemverilog_preprocessor_return_annotations.json: macro_body_fragment 9, macro_default_atom 8, the 5 conditional rules each 1-branch); numbers uniform (1.0.1 only — the 2 "1.0.0" hits are the pre-existing Schema-Versioning-table rows; schema 1 / 64 / 27 consistent). Scope clean (contract file only).`
  Commit: `SVPP-CONTRACT-BODY-Slice-3`

- ID: `SVPP-CONTRACT-BODY.4`
  Status: `done`
  Goal: `Add gate recipe + manifest cross-reference + README/LIVE_ACHIEVEMENT_STATUS links.`
  Acceptance: `Contract references manifest + future book + lib API entry points.`
  Verification: `2026-05-16: Added "## Companion Documentation" (6-row surface/authority table: this contract / book + -html / systemverilog_preprocessor_v1.json manifest / return-annotation inventory / EMBEDDING_API_CONTRACT / bug ledger w/ SVPP-0001 note; precedence EMBEDDING_API_CONTRACT > inventory/manifest > contract > book), "### Gate Recipe" (book gate / AST-shape test / on-demand regen — each source-verified: rust/Makefile:750-753, rust/src/ast_shape_contract.rs:817 unique fn, build-recipe.md flags; Validation/Release Gates by anchor; uses SHELL=/opt/homebrew/bin/bash + a DOC-README-SHELL-0001 note vs README's /bin/bash), "## Glossary" (15 contract-scoped terms incl. SVPP-0001). Independently verified: Companion Documentation/Glossary/Gate Recipe ×1 each; no dup ## headers; no fabricated-struct reintroduction; numbers clean (1.0.1 / schema 1 / 64 / 27, ZERO 1.0.2 or schema-2 leak); SVPP-0001 honest (NOT claimed fixed, 18 refs); README:237 lists the book (read-only confirm; not edited — DOC-README-SHELL-0001 divergence noted). Scope = contract file only.`
  Commit: `SVPP-CONTRACT-BODY-Slice-4`

## Current Frontier

_Empty — tree complete. All leaves `.1`–`.4` `done` (`2026-05-16`)._

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
| `2026-05-16` | `SVPP-CONTRACT-BODY.3` | section-count + dup-`## `-header + no-fabricated-reintroduction grep; conditional-tree rule presence; inventory fragment/atom-count cross-check; numbers/SVPP-0001/scope check | `pass — Conditional Compilation and Macro Body Fragments section (5-rule conditional tree, macro_body_fragment 9, macro_default_atom 8, pp_define-body consumer walk, SVPP-0001 fragment-context honest/not-fixed/schema-1); inventory EXACT (9/8/5×1-branch); 1.0.1 uniform (2 "1.0.0" = pre-existing schema-table rows); no dup ## headers; scope = contract only` |
| `2026-05-16` | `SVPP-CONTRACT-BODY.4` | section-count (Companion Documentation/Glossary/Gate Recipe ×1) + dup-`## `-header + no-fabricated-reintroduction grep; gate/test/regen command source-verification; numbers + SVPP-0001-not-claimed-fixed + README-listing + scope check | `pass — Companion Documentation (6-row authority table) + Gate Recipe (3 commands source-verified: Makefile:750-753, ast_shape_contract.rs:817 unique fn, build-recipe.md; SHELL=/opt/homebrew/bin/bash + DOC-README-SHELL-0001 note) + 15-term Glossary; no dup ## headers; numbers clean (1.0.1/schema 1/64/27, ZERO 1.0.2 or schema-2 leak); SVPP-0001 honest (18 refs, not claimed fixed); README:237 lists book; scope = contract only. Tree closed.` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `SVPP-CONTRACT-BODY.1` | `SVPP-CONTRACT-BODY-Slice-1` | Contract Identity + Schema Versioning + Release 1.0.1 Highlights (full 64-annotation surface) inserted at the top |
| `SVPP-CONTRACT-BODY.2` | `SVPP-CONTRACT-BODY-Slice-2` | AST Envelope + pp_item Dispatch (10 kinds + 7 directive shapes, SVPP-0001 honest); sv_preprocessor book DOC-ENVELOPE-0001 comprehensively closed in lockstep (8 chapters, fabricated-residual 0, gate green) |
| `SVPP-CONTRACT-BODY.3` | `SVPP-CONTRACT-BODY-Slice-3` | Conditional Compilation and Macro Body Fragments (5-rule conditional tree + macro_body_fragment 9 + macro_default_atom 8 + pp_define-body consumer walk; SVPP-0001 fragment-context honest); inventory cross-check EXACT; contract-only |
| `SVPP-CONTRACT-BODY.4` | `SVPP-CONTRACT-BODY-Slice-4` | Companion Documentation 6-row authority table + source-verified Gate Recipe + 15-term Glossary; closes the tree (8th completed) |

## Changelog

- `2026-05-14`: Created task tree.
- `2026-05-16`: `.2` done (AST Envelope + pp_item Dispatch section — 10 pp_item kinds + 7 directive shapes + SVPP-0001 honest/not-fixed/schema-1; inventory cross-check EXACT 64/27/10; sv_preprocessor book `DOC-ENVELOPE-0001` comprehensively closed in lockstep — 8 chapters, targeted fabricated-residual 0, independent gate re-run green). Frontier advances to `.3` (conditional-compilation tree + macro_body fragment + macro_default_atom). Tree stays `active` (`.3`, `.4` remain).
- `2026-05-16`: `.3` done (Conditional Compilation and Macro Body Fragments section — 5-rule conditional tree + macro_body_fragment 9 kinds + macro_default_atom 8 kinds + pp_define-body consumer walk; SVPP-0001 framed in fragment context, honest/not-fixed/schema-1; inventory cross-check EXACT 9/8/5×1-branch; contract-only — sv_preprocessor book DOC-ENVELOPE already closed in `.2`). Frontier advances to `.4` (gate recipe + manifest cross-ref + README/LIVE links — final leaf, closes the tree). Tree stays `active` (`.4` remains).
- `2026-05-16`: `.4` done (Companion Documentation 6-row authority table + source-verified Gate Recipe + 15-term Glossary; independently verified — sections ×1, no dup `## ` headers, no fabricated reintroduction, numbers clean, SVPP-0001 honest, commands source-verified, README lists the book). **Tree `SVPP-CONTRACT-BODY` complete** — root + all leaves `.1`–`.4` `done`; frontier emptied; promoted to Completed in `docs/TASK_TREE.md` (**8th completed tree — all task trees now complete**). DOC-README-SHELL-0001 noted in the Gate Recipe (contract uses the correct `/opt/homebrew/bin/bash`).
