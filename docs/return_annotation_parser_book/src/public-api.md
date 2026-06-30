# Public API

Downstream consumers reach the return-annotation parser through PGEN's stable **embedding API** in
`rust/src/embedding_api.rs`, *not* by linking the generated parser module directly. This chapter lists
the stable surface; the integration contract
(`docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`) is the authoritative,
versioned statement, and `rust/src/embedding_api.rs` is the exact signature source.

## Entry points

| Entry point | Role |
| --- | --- |
| `parse_annotation(...)` | parse return-annotation source into an annotation AST |
| `parse_annotation_result(...)` | parse and return the richer result/diagnostic form |
| `parse_annotation_named(...)` | parse with an explicit name/context selector |

All three take the annotation family and (optionally) a backend selector; see below.

## Selectors

- **Annotation family** — `AnnotationFamily::Return` selects the return-annotation language (its
  sibling is the semantic-annotation family).
- **Backend** — `ParserBackend::Bootstrap` or `ParserBackend::Generated` pins which backend handles the
  parse. Omit/let the host choose unless you specifically need to pin one (see [Backends](backends.md)).

## Diagnostics

The stable diagnostic codes returned on failure are:

| Code | Meaning |
| --- | --- |
| `E_BACKEND_UNAVAILABLE` | the requested backend is not available in this build (e.g. the generated backend without `generated_parsers` support) |
| `E_PARSE_FAILURE` | the input is not valid return-annotation source |
| `E_INPUT_TOO_LARGE` | the input exceeds the configured input-size limit |
| `E_INVALID_LIMITS` | the supplied parse limits are invalid |
| `E_INVALID_ARGUMENT` | an argument to the API is invalid |

Match on the **code**, not on the human-readable message text — message wording can change between
releases while codes are stable.

## Build / availability

- The **Bootstrap** backend is always part of the published contract (no build-feature prerequisite).
- The **Generated** backend is part of the published contract **when `generated_parsers` support is
  present** in the build.
- Prefer the embedding API over linking `generated/return_annotation_parser.rs` directly — the
  embedding surface is the supported, version-stable seam.

## Scope / non-goals

The downstream contract is **parser acceptance, diagnostics, and the family/backend selection
surface**. The internal typed-AST conversion logic in the Rust AST pipeline (how the annotation AST is
turned into the typed parser output) is *not* itself the generic downstream parser contract; it is an
implementation detail behind the embedding API.
