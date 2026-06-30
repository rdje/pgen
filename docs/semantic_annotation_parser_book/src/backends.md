# Backends: Bootstrap vs Generated

Like its sibling `return_annotation`, the semantic-annotation parser has **two backends**, exposed as
`ParserBackend::Bootstrap` and `ParserBackend::Generated`. It also has **two runtime surfaces** for the
`$ref` payload language — be aware of both.

## The chicken-and-egg problem

PGEN generates a parser *from* a `.ebnf`, reading the grammar's annotations to do so. Generating the
parser *for the semantic-annotation language itself* would therefore require an already-existing
semantic-annotation parser — circular. PGEN breaks the cycle with a **bootstrap** path:
`grammars/builtin_semantic_annotation.ebnf` describes a small, hand-written, intentionally permissive
parser that does not depend on a generated annotation parser, so it can be used while generating
everything else. This is why annotation parsers are *"generated with bootstrap mode only."*

## The two backends

### Bootstrap backend (`ParserBackend::Bootstrap`)

- The hand-written parser, documented by `grammars/builtin_semantic_annotation.ebnf`.
- **Always part of the published contract** — no build-feature prerequisite.
- Behavior contract: input is **always outer-trimmed** before classification; classification is strict
  `TransformExpr → Structured → Raw` (see [AST Envelope](ast-envelope.md)); it **never hard-fails** —
  an unknown/malformed payload falls back to `Raw`. For same-line inline rule-body annotations the
  payload capture is bounded (quoted strings, balanced `{…}`/`[…]`/`(…)`, or a scalar non-whitespace
  run) so it cannot swallow the following rule-body syntax.

### Generated backend (`ParserBackend::Generated`)

- The generated parser at `generated/semantic_annotation_parser.rs`, produced from the full
  `grammars/semantic_annotation.ebnf` via the bootstrap generator (see [Build Recipe](build-recipe.md)).
- Part of the published contract **when the generated annotation parser is available**.
- The full, stricter/more-complete value language this book describes.

### Bootstrap ↔ generated parity is a contract

Per the binding policy (2026-02-20), `builtin_semantic_annotation.ebnf` and `semantic_annotation.ebnf`
must remain exact executable compatibility specs for their respective parsers; **drift between them is
a contract failure**. Where their surfaces overlap (rule references, for example) they must accept the
same set. Parity is verified by construct-level parse-acceptance/rejection coverage, typed-AST mapping
coverage, runtime-intent conformance, round-trip coverage, deterministic replay, and bootstrap/generated
differential parity (the SC-01…SC-13 Tier-4 gates).

## The two runtime surfaces for `$ref`

Independently of the backend split, the `$<ref>` reference language has two entry points that must stay
in lockstep:

- **EBNF surface** (`grammars/semantic_annotation.ebnf::rule_reference_name`) — hit by **freestanding
  annotation strings** parsed through the embedding API.
- **Directive-payload runtime** (`unified_semantic_ast.rs::StructuredSemanticValueParser::parse_rule_reference`)
  — hit by authors writing directives **inside** a grammar `.ebnf` file.

Both accept the same dotted + indexed, depth-unbounded `$ref` set (see [Annotation Values and
References](values-and-references.md)).

## Which backend should I use?

Use the stable embedding API (`parse_annotation` / `parse_annotation_result` / `parse_annotation_named`)
with `AnnotationFamily::Semantic` and let the host route to a backend; pass an explicit `ParserBackend`
only when you need to pin one (e.g. to compare bootstrap vs. generated in a test). See
[Public API](public-api.md).
