# Annotation System

Annotations are one of the defining differences between PGEN and a simpler grammar-to-parser tool.

## Two Annotation Families

### Return annotations

Return annotations shape the AST that generated parsers return. They are the normative way to control parse-result structure instead of treating the generated tree as fixed.

### Semantic annotations

Semantic annotations steer parser-generation behavior and related transformation/runtime choices in the Rust AST pipeline.

They also now have a stricter same-line scanner contract. Inline rule-body annotations consume only their own payload:

- quoted payloads,
- balanced structured payloads such as `{...}`, `[...]`, or `(...)`,
- or a scalar token payload.

They do not get to swallow the rest of the rule body. That matters because branch-local hints like `@sample: "..." alpha | beta` are only useful if `alpha | beta` still survives as real branch syntax after tokenization.

That steering now includes more than regex-target tweaks. Literalish directives such as `@sample`, `@literal`, `@example`, and legacy `@stimulus` can now be used as parser-proven stimuli seeds for:

- regex atoms,
- non-regex non-OR rule expansions,
- and inline branch-local OR alternatives.

PGEN also now has a narrower replay-only variant: `@probe_sample`.

- `@sample` is the ordinary always-on literalish steering tool.
- `@probe_sample` is for target-drive replay.
- `@probe_sample` only short-circuits when that rule is the active generation entry, so it can help probe broad dependency rules without collapsing ordinary top-level generation transitively.

That widened the annotation system from "token-shape nudges" into a real narrow branch-steering surface for coverage-guided replay, while still keeping the project rule that sample hints must be justified by parser-backed evidence rather than sprayed across a grammar blindly.

Together, these two annotation families make PGEN a parser platform rather than only a parser emitter.

## Semantic Seeds, Linters, And Front-End Workbenches

The next major widening for semantic annotations is not "more random annotation flexibility." It is a disciplined semantic-seed layer that downstream tools can trust.

The intended model is:

- the grammar emits local semantic seeds,
- the parser preserves source fidelity and provenance,
- later attribution passes compute broader meaning such as binding, typing, and flow,
- and downstream rule engines consume that attributed model rather than guessing from raw parse trees.

That matters first for HDL signoff-style consumers, but it is not an HDL-only idea. If PGEN lands the right semantic-seed, provenance, and export infrastructure, the same platform work should help:

- linters,
- compiler front-ends,
- elaborators,
- and other downstream semantic tools built on PGEN-backed grammars.

The detailed planning surfaces for those adjacent lanes now live in:

- `docs/reference/PGEN_LINTER_ENABLEMENT_ROADMAP.md`
- `docs/reference/PGEN_COMPILER_ELABORATOR_ENABLEMENT_ROADMAP.md`

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
- `docs/reference/PGEN_LINTER_ENABLEMENT_ROADMAP.md`
- `docs/reference/PGEN_COMPILER_ELABORATOR_ENABLEMENT_ROADMAP.md`
- `docs/RETURN_ANNOTATIONS_REFERENCE.md`
- `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`
- `docs/contracts/PGEN_SEMANTIC_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`
- `README.md`
