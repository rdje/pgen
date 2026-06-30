# The Bootstrap Path

PGEN's two **annotation grammars** present a chicken-and-egg problem: `grammars/return_annotation.ebnf` and
`grammars/semantic_annotation.ebnf` define the very `-> …` and `@…` languages that a normal grammar uses
in its own annotations. You cannot parse the annotations of the annotation grammar with a parser that does
not exist yet. **Bootstrap mode** breaks that cycle. Most grammar authors never invoke it directly, but
understanding it explains a few rules you will hit. The authoritative reference is
`docs/BOOTSTRAP_MODE_SPECIFICATION.md`.

## What bootstrap mode is

Bootstrap mode provides built-in, hand-rolled annotation parsing inside the AST pipeline, used when the
generated annotation parsers are unavailable (a clean build from scratch) or when explicitly forced:

- the AST pipeline first tries the external generated annotation parsers;
- if they are unavailable or fail, it **falls back to bootstrap mode automatically** — no flags needed;
- `--bootstrap-mode` forces the bootstrap path for regeneration workflows and proof gates.

The annotation parsers themselves are generated in bootstrap mode **only**, so an annotation parser is
never asked to parse the annotations that define itself. The bootstrap-safe grammar contracts used for
that path are `grammars/builtin_return_annotation.ebnf` and `grammars/builtin_semantic_annotation.ebnf`.

## Why an author cares: the flat-structure limit

The bootstrap annotation parser is deliberately simple. It supports **unlimited flat** structures but
**zero nesting**. This matters whenever your annotation is parsed on the bootstrap path:

**Return annotations — supported (flat only):**

- scalars: `$1`, `$2`, …
- arrays: `[ $1, $2, … ]`, quantified `[ $1* ]`, mixed `[ $1, $2* ]`
- objects: `{ k1: $1, k2: $2, … }` with identifier keys and scalar / simple-quantified values

**Return annotations — NOT supported on the bootstrap path (fall back to raw):**

- nested objects/arrays: `{ outer: { inner: $1 } }`, `[ { name: $1 } ]`, `{ items: [ $1, $2 ] }`
- dotted values inside a structure: `[ $1.name ]`
- dynamic keys / function-call values: `{ $1: $2 }`, `{ r: func($1) }`

**Semantic annotations — supported:** simple `name: value`, identifiers, function calls up to 4 arguments,
and `$ref` references including unbounded-depth dotted (`$a.b.c…`) and integer-indexed (`$items[0][1]`)
chains. **Not supported:** functions with > 4 arguments, nested function calls, and JSONPath features
beyond the dotted/indexed subset.

When a pattern exceeds bootstrap capabilities, it is stored as a raw string (with a warning) rather than
failing the build — bootstrap mode never blocks the build.

## The two semantic-annotation surfaces

There are two parser surfaces for semantic-annotation source, kept in lockstep:

1. **the EBNF-language surface** — `grammars/semantic_annotation.ebnf` →
   `generated/semantic_annotation_parser.rs`, the formal language definition;
2. **the grammar directive-payload runtime** —
   `rust/src/ast_pipeline/unified_semantic_ast.rs::StructuredSemanticValueParser` (hand-rolled), the
   surface a real grammar author actually hits when the AST pipeline reads a grammar's `@directive: { … }`
   while generating a parser.

When the two diverge, the runtime is what actually parses your grammar's directive payloads. A change to
"what `$<ref>` accepts" must touch **both** surfaces — a fact discovered the hard way (the
`SV-EXH-PROOF.3.3.4.a.1` slice). See `docs/BOOTSTRAP_MODE_SPECIFICATION.md` for the full account.

## Practical takeaway

For an ordinary grammar (not an annotation grammar), you rarely think about bootstrap. The one rule that
reaches you: if a richly **nested** return annotation silently comes back as a raw string, you are on the
bootstrap path — flatten the annotation, or ensure the generated annotation parsers are built. See
[The Build Recipe](build-recipe.md) for regenerating the annotation parsers
(`make -C rust return_semantic_parsers`).
