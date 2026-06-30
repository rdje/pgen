# Public API

Downstream consumers reach the semantic-annotation parser through PGEN's stable **embedding API** in
`rust/src/embedding_api.rs`, not by depending on the generated parser types directly. This chapter
lists the stable surface; the integration contract
(`docs/contracts/PGEN_SEMANTIC_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`) is the authoritative,
versioned statement, and `rust/src/embedding_api.rs` is the exact signature source.

## Entry points

| Entry point | Role |
| --- | --- |
| `parse_annotation(...)` | parse semantic-annotation source into an annotation AST |
| `parse_annotation_result(...)` | parse and return the richer result/diagnostic form |
| `parse_annotation_named(...)` | parse with an explicit name/context selector |

All three take the annotation family and (optionally) a backend selector.

## Selectors

- **Annotation family** — `AnnotationFamily::Semantic` selects the semantic-annotation language (its
  sibling is `AnnotationFamily::Return`).
- **Backend** — `ParserBackend::Bootstrap` or `ParserBackend::Generated` pins which backend handles the
  parse. Omit/let the host choose unless you specifically need to pin one (see [Backends](backends.md)).

## Diagnostics

The stable diagnostic codes returned on failure are:

| Code | Meaning |
| --- | --- |
| `E_BACKEND_UNAVAILABLE` | the requested backend is not available in this build |
| `E_PARSE_FAILURE` | the input is not valid semantic-annotation source |
| `E_INPUT_TOO_LARGE` | the input exceeds the configured input-size limit |
| `E_INVALID_LIMITS` | the supplied parse limits are invalid |
| `E_INVALID_ARGUMENT` | an argument to the API is invalid |

Match on the **code**, not the human-readable message text — wording can change between releases while
codes are stable.

Beyond parse-level errors, the steering directives can raise **semantic warnings** (e.g.
`W_SEM_UNSATISFIABLE_VALUE_DOMAIN`, `W_SEM_TOKEN_STEERING_WITHOUT_REGEX_ATOM`,
`W_SEM_CRITICAL_PATH_WITHOUT_COVERAGE_TARGET`, …). The unknown-directive policy (`ignore` / `warn` /
`strict`) and strict-warning promotion are selector-controlled (`PGEN_STRICT_SEMANTIC_WARNING_CODES`);
these are part of the steering semantics, governed by the normative spec, not the parse contract.

## Build / availability

- The **Bootstrap** backend is always part of the published contract (no build-feature prerequisite).
- The **Generated** backend is part of the published contract **when the generated annotation parser is
  available** in the build.
- Prefer the embedding API over depending on `generated/semantic_annotation_parser.rs` types directly.

## Scope / non-goals

The downstream **parse contract** is family selection, acceptance/rejection, diagnostics, and the
bootstrap/generated host surface. The semantic-runtime *meaning*, steering leverage, and aggregate proof
obligations are governed by `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md` and
`docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`, not by the integration contract alone.
