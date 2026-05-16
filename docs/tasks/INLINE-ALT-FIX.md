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
  Status: `pending`
  Goal: `sv_preprocessor SVPP-0001: lift pp_if_branch.keyword inline (kw_ifdef|kw_ifndef) alternation into a named rule; regen; manifest; schema bump; book + contract + ledger lockstep.`
  Acceptance: `parseability_probe on an \`ifdef input shows clean pp_if_branch.keyword (no <invalid_sequence_access>); systemverilog_preprocessor_parser_book_gate + systemverilog_preprocessor_ast_shape_contract green; SVPP-0001 -> Fixed in the ledger; book/contract/manifest/schema all lockstep.`
  Verification: `pending`
  Commit: `pending`

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
| 1 | `INLINE-ALT-FIX.1` | `pending` | `SVPP-0001` is the only one with a filed released-parser bug-ledger entry (highest downstream visibility) and a confirmed repro; fix it first. |

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

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `INLINE-ALT-FIX` (setup) | `PGEN-INLINE-ALT-FIX-0000` | tree created; frontier `.1`; stale Cat-B extraction memory corrected in lockstep |

## Changelog

- `2026-05-16`: Created task tree. Decomposed the inline-alternation
  parser-correctness lane into 3 per-grammar leaves
  (sv_preprocessor/SVPP-0001 → rtl_frontend → vhdl). Fix method decided
  (named-op-rule + bare `$2`, proven in `rtl_const_expr`). Frontier
  `.1`. Corrected the stale `feedback_quantified_group_extraction`
  memory (Cat B) + its MEMORY.md index line in lockstep.
