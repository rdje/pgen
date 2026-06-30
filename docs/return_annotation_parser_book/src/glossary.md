# Glossary

- **Return annotation** — the `-> …` payload on a grammar rule that says how the rule's match is
  lowered into the value it returns. The return-annotation parser is the parser for this mini-language.
- **AST shaping** — reshaping and labeling the parse tree into the returned value, using
  objects/arrays/references/extraction/spread. Structural, not computational (no arithmetic or function
  calls).
- **Passthrough** — the default "return the value unchanged" behavior, equivalent to `$1`. A bare `->`
  or an empty annotation means passthrough; a single-element rule body with no `-> …` is given a
  synthetic `$1` by the code generator.
- **Positional reference (`$N`)** — a reference to the Nth captured element of the rule body (1-based).
- **`$text` / `$0`** — the rule's whole matched source text as a single string; lowers to
  `UnifiedReturnAST::MatchedText`. `$0` is an alias for `$text`.
- **Extraction (`$N::T`)** — pull one element (by 1-based index, or `first`/`last`) out of a quantified
  capture; an optional trailing `*` spreads it.
- **Spread (`*`)** — splice a capture's children into the surrounding array/object list. Builds a
  cons-shape list when applied to a repetition.
- **Flatten-spread (`**`)** — like spread, but additionally unwraps a pushed child whose content is
  itself a `Sequence`/`Quantified`, pushing its children inline.
- **Property access (`$N.field`)** / **index access (`$N[i]`)** — reach into a shaped value by name or
  by index; both chain at unbounded depth (a dotted-property + non-negative-integer subset, *not* full
  JSONPath).
- **Cons-list shape** — `[head, [next, [next, …]]]`: the nested list a plain `*` spread produces for a
  repetition. Walk it recursively.
- **Flat-list idiom (`[$1, $2::2*]`)** — the recommended `X (sep X)*` form that produces a flat,
  source-order list (Category A). The return-annotation grammar uses it for its own
  `properties`/`elements`.
- **Annotation AST** — the structured `{type: …}` representation the return-annotation parser produces
  from annotation source (see [AST Envelope](ast-envelope.md)).
- **Bootstrap backend** — the hand-written parser (`rust/src/ast_pipeline/unified_return_ast.rs`,
  documented by `grammars/builtin_return_annotation.ebnf`) that breaks the chicken-and-egg cycle;
  always part of the contract.
- **Generated backend** — the generated parser (`generated/return_annotation_parser.rs`, from
  `grammars/return_annotation.ebnf`); part of the contract when `generated_parsers` support is present.
- **Annotation family (`AnnotationFamily::Return`)** — the embedding-API selector for the
  return-annotation language; its sibling is the semantic-annotation family.
- **`UnifiedReturnAST`** — the internal runtime model the annotation AST lowers into; an implementation
  detail behind the embedding API, not part of the downstream contract.
