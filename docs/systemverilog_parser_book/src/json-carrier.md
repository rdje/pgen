# The Json Carrier

This chapter is a flat reference table of every `systemverilog.ebnf` rule that carries a `-> ...` return annotation, and the JSON shape that annotation produces.

> **Note:** The systemverilog return-annotation campaign is **in early phase**. Most rules are currently un-annotated and produce the recursive-envelope shape described in [AST Envelope Structure](ast-envelope.md). This table grows as the campaign progresses.

## Currently annotated rules

| Rule | Annotation | JSON shape produced |
|---|---|---|
| _(none yet)_ | _(none yet)_ | _(none yet)_ |

The first slice of the campaign will land in a follow-up commit and add its row here.

## Sub-rules with implicit defaults

Rules that have no explicit annotation default to their grammar-shape envelope (see [Parse Content Variants](parse-content-variants.md)). The default is documented at the rule level in `grammars/systemverilog.ebnf` comments where the default is non-obvious.

## Unannotated-on-purpose rules

Some rules will remain un-annotated by design — typically utility / helper rules whose envelope shape is the most useful representation, or rules whose typed shape would be redundant with their parent rule's shape.

| Rule | Reason |
|---|---|
| _(none yet)_ | _(none yet)_ |

Each row added here will cite the slice that decided the rule should remain un-annotated.

## How to read the annotation column

The annotation column shows the EBNF `-> ...` clause from `grammars/systemverilog.ebnf`. The reference grammar for the annotation language is:

- `$N` — positional reference to the Nth body element (1-indexed).
- `$N.field` — member access on a typed sub-rule shape.
- `{field: value, ...}` — typed object literal.
- `[v1, v2, ...]` — array literal.
- `[$N**]` — flatten-spread an array-shaped reference.
- `true` / `false` / `null` — boolean / null scalars.
- `@transform` — typed numeric value via `str::parse::<TYPE>`-style transform.
- `"text"` — string literal.

See `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md` for the full annotation-language grammar.
