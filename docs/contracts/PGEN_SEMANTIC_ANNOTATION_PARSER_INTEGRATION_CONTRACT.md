# docs/contracts/PGEN_SEMANTIC_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md

## Purpose
Define the current downstream integration contract for PGEN's `semantic_annotation` parser family.

## Source Of Truth
- Main grammar source:
  - `grammars/semantic_annotation.ebnf`
- Bootstrap-safe grammar source:
  - `grammars/builtin_semantic_annotation.ebnf`
- Tracked generated artifacts:
  - `generated/semantic_annotation.json`
  - `generated/semantic_annotation_parser.rs`
- Public host API:
  - `rust/src/embedding_api.rs`
- Normative semantic/contract docs:
  - `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`
  - `docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`

## Stable Integration Surface
- Annotation family:
  - `semantic`
- Stable host entry points:
  - `parse_annotation(...)`
  - `parse_annotation_result(...)`
  - `parse_annotation_named(...)`
- Stable family selector:
  - `AnnotationFamily::Semantic`
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
- Generated backend is part of the published contract when the generated annotation parser is available.
- Downstream consumers should use the embedding API surface rather than directly depending on internal generated parser types.

## Validation / Release Gates
- `make -C rust SHELL=/bin/bash annotation_contract_gate`
- `make -C rust SHELL=/bin/bash semantic_usage_gate`
- `make -C rust SHELL=/bin/bash semantic_runtime_contract_gate`
- `make -C rust SHELL=/bin/bash semantic_full_contract_gate`

## Scope / Non-Goals
- This contract covers parser-family selection, acceptance/rejection, diagnostics, and the current bootstrap/generated host surface.
- Semantic-runtime meaning, steering leverage, and aggregate proof obligations are governed by `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md` and `docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`, not by this file alone.
- `semantic_annotation` does not currently have a separate top-level live-status row; track its maturity through the annotation proof spine and the docs above.
- When reporting downstream bugs, follow `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`; accepted released-parser bugs should then be logged in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`.

## Recent Additions

### 2026-05-20 — `SV-EXH-PROOF.3.3.4.a.1` / `.a.2` (`PGEN-SV-EXH-PROOF-0026` / `0027`): rule-reference syntax — dotted + indexed, depth-unbounded

The `$<ref>` reference shape accepted in semantic-annotation directive payloads is extended (strictly additive — every prior `$name` / `$1` reference parses byte-identically):

```
rule_reference   ::= "$" head segment*
head             ::= /[a-zA-Z_][a-zA-Z0-9_]*/        # named
                  |  /[0-9]+/                          # positional, 1-indexed
segment          ::= "." /[a-zA-Z_][a-zA-Z0-9_]*/     # dotted property
                  |  "[" /[0-9]+/ "]"                  # non-negative integer index
```

Examples now accepted: `$name.body`, `$1.body.subkey`, `$items[0]`, `$matrix[0][1]`, `$a.b[0].c[1].d.e[2].r.z`.

Subset boundary fixed: dotted property + non-negative integer indexing only. NOT full JSONPath (no filters / wildcards / recursive descent / negative indices / range slices). Each excluded feature would require its own normative leaf.

Two-surface lockstep. Both the EBNF-language surface (`grammars/semantic_annotation.ebnf::rule_reference_name`) and the grammar-directive-payload runtime (`unified_semantic_ast.rs::StructuredSemanticValueParser::parse_rule_reference`) are extended in lockstep. Authors writing directives inside grammar `.ebnf` files hit the runtime surface; freestanding annotation strings through the embedding API hit the EBNF surface. Both accept the same set after these slices.

Durable no-depth-limit guarantee. The reference depth is structurally unbounded at every layer (EBNF `*`, hand-rolled `loop`, lexer, resolver iterator). Locked by two regression tests exercising 64 segments each — see `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md` "Rule Reference Syntax (Normative)" for the normative pin and the failure-direction.

Strict trailing-dot / strict-bracket policy. Malformed forms (bare `.`, `[` with no `<digits>]`) roll back to before the offending segment; the surrounding payload parser then handles the leftover or falls back to `Raw`.
