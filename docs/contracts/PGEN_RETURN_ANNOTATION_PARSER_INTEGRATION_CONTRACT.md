# docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md

## Purpose
Define the current downstream integration contract for PGEN's `return_annotation` parser family.

## Source Of Truth
- Main grammar source:
  - `grammars/return_annotation.ebnf`
- Bootstrap-safe grammar source:
  - `grammars/builtin_return_annotation.ebnf`
- Tracked generated artifacts:
  - `generated/return_annotation.json`
  - `generated/return_annotation_parser.rs`
- Public host API:
  - `rust/src/embedding_api.rs`
- Normative semantic/contract doc:
  - `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`

## Stable Integration Surface
- Annotation family:
  - `return`
- Stable host entry points:
  - `parse_annotation(...)`
  - `parse_annotation_result(...)`
  - `parse_annotation_named(...)`
- Stable family selector:
  - `AnnotationFamily::Return`
- Stable backend selectors:
  - `ParserBackend::Bootstrap`
  - `ParserBackend::Generated`
- Stable diagnostics:
  - `E_BACKEND_UNAVAILABLE`
  - `E_PARSE_FAILURE`
  - `E_INPUT_TOO_LARGE`
  - `E_INVALID_LIMITS`
  - `E_INVALID_ARGUMENT`

## Build / Availability Requirements
- Bootstrap backend is part of the published contract.
- Generated backend is also part of the published contract when `generated_parsers` support is present.
- Downstream consumers should use the embedding API surface rather than linking directly to generated parser modules.

## Validation / Release Gates
- `make -C rust SHELL=/bin/bash annotation_contract_gate`
- `make -C rust SHELL=/bin/bash return_runtime_semantics_gate`
- `make -C rust SHELL=/bin/bash return_annotation_support_gate`

## Scope / Non-Goals
- The downstream contract is parser acceptance, diagnostics, and the annotation family/backend selection surface.
- Internal typed AST conversion logic in the Rust AST pipeline is not itself the generic downstream parser contract.
- `return_annotation` is currently a `Done` family for the tracked claim, but that claim is still defined by the repo’s current grammar and proof stack, not by informal future expectations.
- When reporting downstream bugs, follow `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`; accepted released-parser bugs should then be logged in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`.

## Notable Recent Shape Changes

### 2026-06-10 — `string_literal` single-quoted shape RESTORED (regression window 2026-05-14 → 2026-06-10; ledger `RETANN-0001`)
- A 2026-05-14 engine refinement (the inner→outer branch-index remap for groups inside sequences/quantifiers) accidentally re-broke the task #38 broadcast below for WHOLE-BODY parens groups: between 2026-05-14 and 2026-06-10, regenerated `return_annotation` parsers again produced a raw `Sequence` for single-quoted strings (`'x'`) while double-quoted strings stayed typed.
- Fixed by `BRANCH-BROADCAST-FIX.2` (`PGEN-BRANCH-BROADCAST-FIX-0002`): the engine now keeps group-local branch indices when the rule body is exactly one top-level parens group, so the trailing annotation broadcasts to every runtime branch again. Both quote forms produce `Json({"type":"string","value":"..."})` — the #38 contract shape.
- Consumers that adopted a parser regenerated inside the regression window and special-cased the single-quoted `Sequence` shape must drop that workaround. Consumers on pre-2026-05-14 artifacts are unaffected.
- Regression locks: the AST shape-contract manifest's `single_quoted_string_literal_typed` runtime sample (`'x'` → `json_object` with `value:"x"`), the declared-annotation inventory (19→20 rows, both artifact and raw-IR crosscheck comparisons), and focused engine unit tests for the whole-body/mixed/trailing-group/patterns-(A)–(D) shapes.
- Companion fix in the same wave (`BRANCH-BROADCAST-FIX.3`): a branch-level `$text` (MatchedText) inside a multi-branch rule returned the EMPTY string (the tournament arm rolled the position back before evaluating the transform); it now returns the exact matched span. No shipped `return_annotation` shape used branch-level `$text`, so this is engine hardening for this family.

### 2026-05-01 — `string_literal` shape correction (task #38)
- The `string_literal := ('"' string_content_double '"' | "'" string_content_single "'") -> {type:"string", value:$2}` rule had a long-standing internal bug: the trailing return annotation broadcasted only to branch 0 of the parens-grouped Or, leaving branch 1 (single-quoted strings) with raw passthrough. Empirically: double-quoted strings produced `Json({"type":"string", "value":"..."})` while single-quoted strings produced raw `Sequence([Terminal("'"), Alternative(Terminal("...")), Terminal("'")])`.
- The fix lands in two extractors that need to agree: `extract_rule_annotations` in `rust/src/ast_pipeline/mod.rs` and the cross-checker `extract_declared_annotations_from_json` in `rust/src/ast_shape_contract.rs`. Both now broadcast the trailing annotation to every branch that was inside the just-closed group.
- Post-fix: both quote forms produce `Json({"type":"string", "value":"..."})`. This is the shape consumers should expect from the published return-annotation parser going forward.
- This is a buggy→correct fix, not a versioned-consumer-impacting evolution. Downstream consumers of the inventory artifact (the PGEN-internal ast_pipeline) are unaffected. Downstream consumers of the raw parse output that previously special-cased the single-quoted Sequence shape need to update their walking code.
- Cross-grammar effect: any grammar using `(A | B | C) -> ann` with a single trailing annotation now correctly broadcasts to every alternative. The same fix is therefore relevant to any future return_annotation grammar use of the parens-grouped-Or pattern.
