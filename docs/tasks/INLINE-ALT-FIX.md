# INLINE-ALT-FIX: Fix the inline-alternation `$N` parser-correctness defect class

## Metadata

- Tree ID: `INLINE-ALT-FIX`
- Status: `active`
- Roadmap lane: parser-correctness (released-parser defect class)
- Created: `2026-05-16`
- Last updated: `2026-05-16`
- Owner: repo-local workflow

## Goal

Eliminate the inline-alternation-`$N` defect class from the remaining
affected released grammars. When an **inline alternation is the lead
element of a quantified iteration** (`next ((opA|opB) next)*` /
`(kw_a | kw_b) …` inside a `*`-group), the return-transform mis-builds
the positional model and emits a literal `"<invalid_sequence_access>"`
string plus a malformed nested object. The proven, gate-locked fix
(landed in `rtl_const_expr`, `PGEN-RTL-0002`, schema 1→2) is the mature
`systemverilog.ebnf` idiom: **lift the inline alternation into a named
rule** (e.g. `additive_op := plus | minus`, mirroring `binary_operator`)
so the construct becomes `next (named_op next)* -> {lhs:$1, rest:$2}`
with **bare `$2`** (never `$2*`/`$2**`/`$2::2*` — all empirically
wrong). Each grammar fix lands in lockstep with: parser regen, the
shape-contract manifest, the AST-dump schema-version bump, the
per-parser book, the integration contract, and the released-parser bug
ledger.

## Non-Goals

- Do not change the consumer-facing fold contract beyond what the fix
  structurally requires (clean `[op-envelope, operand]` `rest`).
- Do not touch grammars not in this defect class.
- Do not re-derive the fix method — it is decided (see Decisions).

## Acceptance Criteria

- For each affected grammar: the inline alternation is lifted to a
  named rule; `parseability_probe --parse-dump-ast-pretty` on a
  representative operator/keyword input shows a clean `rest`
  (`[op-envelope, operand]` entries, `[]` when none) with **no**
  `"<invalid_sequence_access>"` and **no** malformed nested object.
- The shape-contract manifest, AST-dump schema version, per-parser
  book, integration contract, and bug ledger are updated in the same
  commit; the family's `*_parser_book_gate` and
  `*_ast_shape_contract` test pass.
- `SVPP-0001` transitions to `Fixed in <version>` in the ledger.

## Task Tree

- ID: `INLINE-ALT-FIX`
  Status: `active`
  Goal: `Fix the inline-alternation $N defect class in the remaining affected grammars.`
  Children: `INLINE-ALT-FIX.1`, `INLINE-ALT-FIX.2`, `INLINE-ALT-FIX.3`

- ID: `INLINE-ALT-FIX.1`
  Status: `done`
  Goal: `sv_preprocessor SVPP-0001: lift pp_if_branch.keyword inline (kw_ifdef|kw_ifndef) alternation into a named rule; regen; manifest; schema bump; book + contract + ledger lockstep.`
  Acceptance: `parseability_probe on an \`ifdef input shows clean pp_if_branch.keyword (no <invalid_sequence_access>); systemverilog_preprocessor_parser_book_gate + systemverilog_preprocessor_ast_shape_contract green; SVPP-0001 -> Fixed in the ledger; book/contract/manifest/schema all lockstep.`
  Verification: `2026-05-16: Lifted (kw_ifdef|kw_ifndef) into named rule pp_if_keyword := kw_ifdef -> {kind:"ifdef"} | kw_ifndef -> {kind:"ifndef"} (the systemverilog.ebnf binary_operator idiom); pp_if_branch annotation unchanged ($1 now binds the named rule). BEFORE probe = malformed {items/macro/tail:"<invalid_sequence_access>", keyword:[[],"\`ifdef"]}; AFTER probe = clean {kind:"ifdef"}, zero <invalid_sequence_access> (parseability_probe, ledger repro). Parser + frontend JSON + return-annotation inventory regenerated (64->66 annotations, 27->28 distinct rules). Manifest systemverilog_preprocessor_v1.json declared_annotation_inventory rebuilt from the regenerated frontend JSON via the canonical dump_declared_annotation_inventory bin (66 entries, correct cross-check order) + extracted_at 2026-05-16 + new "conditional" sample locking the fixed path -> systemverilog_preprocessor_ast_shape_contract PASSES. Schema 1->2, release/contract 1.0.1->1.0.2. Contract + 7+ book chapters + bug ledger all lockstep (SVPP-0001 Root Caused -> Released, history kept honestly; examples-conditional rewritten to the real fixed shape with schema-2 transition note per the rtl_const_expr binary-addition template; walkers pin SVPP_AST_SCHEMA_VERSION=2; DOC-ENVELOPE-0001 4-field AstDumpPayload NOT regressed). Independently verified: contract no dup ## headers, numbers current 1.0.2/2/66/28 (1.0.1/64/27 only historical), pp_if_keyword documented; book gate independently re-run green, searchindex deterministic; clippy strict source lint passed (generated non-strict debt pre-existing/tolerated). docs/book/ checked — no SVPP-0001/defect mention there (pointer only), no edit needed.`
  Commit: `PGEN-INLINE-ALT-FIX-0001`

- ID: `INLINE-ALT-FIX.2`
  Status: `pending`
  Goal: `rtl_frontend binop_chain: lift each inline operator alternation into a named op-rule + bare $2; regen; manifest; schema bump; book + contract lockstep.`
  Acceptance: `parseability_probe on an operator expression shows clean binop_chain rest; rtl_frontend_parser_book_gate + rtl_frontend_ast_shape_contract green; book/contract/manifest/schema lockstep.`
  Verification: `pending`
  Commit: `pending`

- ID: `INLINE-ALT-FIX.3`
  Status: `pending`
  Goal: `vhdl binop_chain: FIRST empirically confirm the defect with parseability_probe; if confirmed, apply the named-op-rule + bare $2 fix + lockstep; if NOT affected, record the negative result and close the leaf.`
  Acceptance: `Empirical probe result recorded; if affected, same acceptance as .2 for vhdl; if not affected, documented negative result with the probe evidence.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `INLINE-ALT-FIX.2` | `pending` | rtl_frontend `binop_chain` inline-alternation instances — confirmed affected (memory). Apply the same proven named-op-rule + bare `$2` fix + lockstep. |

## Decisions

- `2026-05-16`: Fix method is **decided and proven** — named-op-rule
  (lift inline alternation) + bare `$2`, the `systemverilog.ebnf`
  `binary_operator` idiom; landed gate-locked in `rtl_const_expr`
  (`PGEN-RTL-0002`, schema 1→2). `$2*` single-star was empirically
  tried and rejected. No re-derivation.
- `2026-05-16`: Every `.ebnf` edit in this lane MUST be preceded by the
  mandatory annotation-doc consultation (`grammars/return_annotation.ebnf`
  + `docs/RETURN_ANNOTATIONS_REFERENCE.md` +
  `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md` + the extraction
  memory) — hook-enforced by `.claude/settings.json` PreToolUse on
  `*.ebnf`. Verify with `parseability_probe --parse-dump-ast-pretty`
  before trusting any regenerated shape.
- `2026-05-16`: Correctness-fix schema bump per grammar (shape changes),
  mirroring `rtl_const_expr` 1→2; book + contract + manifest + ledger
  updated same-commit (book-sync directive — non-negotiable).
- `2026-05-16`: Frontier order `.1` (sv_preprocessor/SVPP-0001) → `.2`
  (rtl_frontend) → `.3` (vhdl, verify-first) by downstream visibility.

## Open Questions

- `.3` vhdl: is its `binop`/`expression` chain actually affected? To be
  resolved empirically at the start of `.3` (memory says "VHDL likely
  too" but not confirmed) — not blocking `.1`/`.2`.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-16` | `INLINE-ALT-FIX` (setup) | task-tree decomposition vs workflow splitting rules; fix-method provenance (rtl_const_expr gate-locked); memory correction | `pass — lane decomposed into 3 per-grammar leaves; method decided/proven; stale Cat-B memory + index corrected to the named-op-rule + bare $2 truth` |
| `2026-05-16` | `INLINE-ALT-FIX.1` | before/after parseability_probe; regen (parser+frontend JSON+inventory 64->66); manifest rebuilt via dump_declared_annotation_inventory; systemverilog_preprocessor_ast_shape_contract; contract dup-header+numbers grep; book walker-pin + fabricated-residual grep; independent book-gate re-run; clippy; docs/book check | `pass — SVPP-0001 fixed (pp_if_keyword named rule); probe <invalid_sequence_access> -> {kind:"ifdef"}; shape-contract test PASSES; schema 1->2 / release 1.0.1->1.0.2 / 66/28; contract+7-book-chapters+ledger lockstep (SVPP-0001 -> Released, history kept); no dup ## headers; DOC-ENVELOPE not regressed; book gate green; clippy strict source ✓` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `INLINE-ALT-FIX` (setup) | `PGEN-INLINE-ALT-FIX-0000` | tree created; frontier `.1`; stale Cat-B extraction memory corrected in lockstep |
| `INLINE-ALT-FIX.1` | `PGEN-INLINE-ALT-FIX-0001` | SVPP-0001 fixed: named `pp_if_keyword` rule; schema 1→2 / 1.0.1→1.0.2 / 64→66 / 27→28; contract+book+ledger lockstep; SVPP-0001 → Released; shape-contract + book gate green |

## Changelog

- `2026-05-16`: Created task tree. Decomposed the inline-alternation
  parser-correctness lane into 3 per-grammar leaves
  (sv_preprocessor/SVPP-0001 → rtl_frontend → vhdl). Fix method decided
  (named-op-rule + bare `$2`, proven in `rtl_const_expr`). Frontier
  `.1`. Corrected the stale `feedback_quantified_group_extraction`
  memory (Cat B) + its MEMORY.md index line in lockstep.
- `2026-05-16`: `.1` done — `SVPP-0001` fixed in `systemverilog_preprocessor`
  via the named `pp_if_keyword` rule (the proven `systemverilog.ebnf`
  `binary_operator` idiom). Before/after `parseability_probe` proves
  `<invalid_sequence_access>` → `{kind:"ifdef"}`; schema `1→2`,
  release/contract `1.0.1→1.0.2`, surface `64/27→66/28`; manifest
  regression-lock rebuilt (canonical 66-entry inventory + `conditional`
  sample) → `systemverilog_preprocessor_ast_shape_contract` **passes**;
  contract + 7+ book chapters + bug ledger in lockstep (`SVPP-0001` →
  `Released`, history kept honestly; DOC-ENVELOPE-0001 not regressed);
  book gate independently re-run green; clippy strict source ✓.
  Frontier advances to `.2` (rtl_frontend `binop_chain`). Tree stays
  `active` (`.2`, `.3` remain).
