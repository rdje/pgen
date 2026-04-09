# Annotation System

Annotations are one of the defining differences between PGEN and a simpler grammar-to-parser tool.

## Two Annotation Families

### Return annotations

Return annotations shape the AST that generated parsers return. They are the normative way to control parse-result structure instead of treating the generated tree as fixed.

### Semantic annotations

Semantic annotations steer parser-generation behavior and related transformation/runtime choices in the Rust AST pipeline.

Together, these two annotation families make PGEN a parser platform rather than only a parser emitter.

## Why They Matter

Without annotations, grammar-driven generation can still produce parsers. With annotations, PGEN can also control:

- AST shape,
- transformation behavior,
- steering metadata,
- downstream usability of generated parsers.

That is why annotation grammars are core platform surfaces, not optional extras.

## Bootstrap Reality

Annotations also sit at the center of one of PGEN's historic bootstrapping constraints.

Because annotation parsers are needed by the generation pipeline itself, PGEN carries bootstrap-safe annotation grammar contracts so those parsers can be generated without circular dependency on themselves.

This is why the docs distinguish between:

- bootstrap-safe built-in annotation grammars,
- full main annotation grammars,
- generated parser steady-state behavior.

## Proof Expectations

Annotation support is not considered real just because syntax exists. It is expected to have:

- validator coverage,
- shared/built-in suite coverage,
- round-trip or comparable contract evidence,
- maintained aggregate gates.

## Primary Source Docs

- `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`
- `docs/RETURN_ANNOTATIONS_REFERENCE.md`
- `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`
- `docs/contracts/PGEN_SEMANTIC_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`
- `README.md`
