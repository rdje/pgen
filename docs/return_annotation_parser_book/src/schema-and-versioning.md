# Schema and Versioning

## Grammar version

The return-annotation language is versioned in its grammar header:
`grammars/return_annotation.ebnf` declares **version `2.0.0`** (the version that introduced the
extraction operators, spreading, objects, arrays, property/index access, `$text`, and `null`). This
book documents that surface.

## What is contract-stable

Per `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`, the stable surface is:

- the annotation family selector `AnnotationFamily::Return`;
- the entry points `parse_annotation` / `parse_annotation_result` / `parse_annotation_named`;
- the backend selectors `ParserBackend::Bootstrap` / `ParserBackend::Generated`;
- the diagnostic codes `E_BACKEND_UNAVAILABLE`, `E_PARSE_FAILURE`, `E_INPUT_TOO_LARGE`,
  `E_INVALID_LIMITS`, `E_INVALID_ARGUMENT`;
- the parsed-annotation node shapes documented in [AST Envelope](ast-envelope.md).

`return_annotation` is a **`Done`** family for the tracked claim — but that claim is defined by the
repository's current grammar and proof stack, not by informal future expectations.

## Proof gates

| Gate | Proves |
| --- | --- |
| `make -C rust SHELL=/bin/bash return_annotation_support_gate` | the focused aggregate `Done`-gate (now includes the auto-derived `return_annotation_exhaustiveness_gate`: grammar-driven coverage closure, stimuli-module parity, and a generated-parse-tree → typed-AST audit) |
| `make -C rust SHELL=/bin/bash return_runtime_semantics_gate` | return-annotation runtime/round-trip semantics |
| `make -C rust SHELL=/bin/bash annotation_contract_gate` | the aggregate annotation contract spine (validator coverage + built-in/shared suites + SC slices + robustness/stimuli) |
| `make -C rust SHELL=/bin/bash annotation_stimuli_quality_gate` | the closed-loop stimuli-quality proof, including the return-annotation generator/parser loop |

## Notable shape changes

These are the consumer-relevant shape changes recorded in the integration contract. They concern the
**`string_literal`** shape, which is built from a parens-grouped `Or` and therefore depends on the
engine correctly *broadcasting* a single trailing annotation to every alternative.

### 2026-05-01 — `string_literal` shape correction (task #38)

The `string_literal := ('"' … '"' | "'" … "'") -> {type:"string", value:$2}` rule had a long-standing
bug: the trailing annotation broadcast only to branch 0 of the grouped `Or`, so double-quoted strings
produced the typed `{type:"string", value:…}` node while **single-quoted** strings produced a raw
`Sequence`. The fix made both quote forms produce the typed `{type:"string", value:…}` node. Consumers
that previously special-cased the single-quoted raw `Sequence` shape should drop that workaround. This
also fixed the general `(A | B | C) -> ann` broadcast for every grammar.

### 2026-06-10 — single-quoted shape RESTORED (ledger `RETANN-0001`)

A 2026-05-14 engine refinement (an inner→outer branch-index remap for groups inside
sequences/quantifiers) accidentally re-broke the whole-body-parens broadcast: between 2026-05-14 and
2026-06-10, regenerated parsers again produced a raw `Sequence` for single-quoted strings. `BRANCH-
BROADCAST-FIX.2` restored it — the engine now keeps group-local branch indices when the rule body is
exactly one top-level parens group, so both quote forms again produce
`Json({"type":"string","value":…})`. A companion fix (`BRANCH-BROADCAST-FIX.3`) corrected a
branch-level `$text` returning the empty string inside a multi-branch rule (engine hardening; no
shipped return-annotation shape relied on branch-level `$text`).

**Takeaway for consumers:** expect the typed `{type:"string", value:…}` shape for *both* quote styles.
If you adopted a parser regenerated inside the 2026-05-14 → 2026-06-10 regression window and special-
cased the single-quoted `Sequence`, drop that workaround.
