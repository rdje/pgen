# AST Envelope Structure

When you parse return-annotation source through the embedding API, you get back a structured
**annotation AST** — the parser's representation of the `-> …` payload. This chapter is the canonical
table of node shapes and a worked example. Every shape here is exactly what the corresponding rule's
`-> {…}` annotation in `grammars/return_annotation.ebnf` produces.

## Node shapes

| You write | Parsed annotation node |
| --- | --- |
| `$N` | `{ "type": "positional", "index": <N> }` |
| `$text` / `$0` | `{ "type": "matched_text" }` |
| `"…"` / `'…'` | `{ "type": "string", "value": "<inner text>" }` |
| `42` / `-2.5` | a native JSON number (`i64` for integers, `f64` for floats) |
| `true` / `false` | a native JSON boolean |
| `null` | `{ "type": "null" }` |
| `{ k: v, … }` | `{ "type": "object", "properties": [ { "key": <k>, "value": <v> }, … ] }` |
| `[ e, … ]` | `{ "type": "array", "elements": [ <e>, … ] }` |
| `$N::T` (`*`) | `{ "type": "extraction", "base": <ref>, "target": <T>, "spread": "true"? }` |
| `$N*` | `{ "type": "spread", "base": <expr> }` |
| `$N**` | `{ "type": "flat_spread", "base": <expr> }` |
| `$N.field` | `{ "type": "property_access", "base": <expr>, "property": "field" }` |
| `$N[i]` | `{ "type": "array_access", "base": <expr>, "index": <expr> }` |
| `->` / *(empty)* | passthrough (equivalent to `$1`) |

Notes:

- `index` (of `positional`) and number literals are real numbers, not strings.
- `target` (of `extraction`) is a number for an index target, or the string `"first"` / `"last"`.
- `spread` is present (and equal to the string `"true"`) only when the extraction carried a trailing
  `*`; otherwise the key is absent.
- `properties` and `elements` are **flat, source-order lists** because the grammar builds them with the
  `[$1, $2::2*]` extraction-spread idiom (see [Objects and Arrays](objects-and-arrays.md)).
- nested nodes (`base`, `value`, `index`, …) are themselves nodes of these same shapes — the AST is
  recursive.

## Worked example

The annotation

```text
-> {
     type: "function",
     name: $1,
     params: [$3, $4::2*],
     body: $6
   }
```

parses to (abridged):

```json
{
  "type": "object",
  "properties": [
    { "key": "type",   "value": { "type": "string", "value": "function" } },
    { "key": "name",   "value": { "type": "positional", "index": 1 } },
    { "key": "params", "value": {
        "type": "array",
        "elements": [
          { "type": "positional", "index": 3 },
          { "type": "extraction", "base": { "type": "positional", "index": 4 },
            "target": 2, "spread": "true" }
        ] } },
    { "key": "body",   "value": { "type": "positional", "index": 6 } }
  ]
}
```

## From annotation AST to runtime lowering

The annotation AST above is what the PGEN code generator consumes to emit the rule's lowering code. At
runtime the host (`rust/src/ast_pipeline/unified_return_ast.rs`) lowers these nodes into the
`UnifiedReturnAST` model — e.g. a `{type: "matched_text"}` node becomes `UnifiedReturnAST::MatchedText`
(the whole matched span as a string), positional references resolve to the captured child at that
index, and `object`/`array` nodes build the corresponding JSON object/array. This book documents the
parsed-annotation shapes; the `UnifiedReturnAST` lowering and the typed-AST conversion are internal to
the AST pipeline and are not part of the downstream parser contract (see
[Public API](public-api.md) and the integration contract).
