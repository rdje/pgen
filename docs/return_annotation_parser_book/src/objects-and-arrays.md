# Objects and Arrays

Objects and arrays are how a return annotation builds a *labeled, structured* value out of the
captured elements. They are the most common shapes in real grammars.

## Object literals

```ebnf
object_literal := '{' object_properties? '}'
-> {type: "object", properties: $2}

object_properties := object_property (',' object_property)*
-> [$1, $2::2*]

object_property := property_key ':' expression
-> {key: $1, value: $3}

property_key := identifier | string_literal
```

An object is brace-delimited, comma-separated `key: value` pairs. A key is an **identifier** or a
**string literal**; a value is any expression.

| You write | Builds |
| --- | --- |
| `{}` | empty object (`properties` is absent/empty) |
| `{type: "node"}` | one pair: key `type`, value the string `node` |
| `{key: $1, value: $3}` | two pairs referencing captures `$1` and `$3` |
| `{"a-b": $2}` | a string key (use a string literal when the key isn't a bare identifier) |

Each pair parses to `{key: …, value: …}`, and `properties` is the **flat, source-order list** of those
pair nodes (see *List shape* below). A worked end-to-end example (`object_property` from
`grammars/json.ebnf`):

```ebnf
object_property := property_key ':' expression
-> {key: $1, value: $3}
```

parses to the annotation node:

```json
{ "type": "object", "properties": [ { "key": <key-node>, "value": <value-node> } ] }
```

## Array literals

```ebnf
array_literal := '[' array_elements? ']'
-> {type: "array", elements: $2}

array_elements := array_element (',' array_element)*
-> [$1, $2::2*]

array_element := expression
```

An array is bracket-delimited, comma-separated expressions. Any element may itself be a spread or
extraction (see [Operators](operators.md)), which is how `[$1, $2*]` and `[$1, $3::2*]` are built.

| You write | Builds |
| --- | --- |
| `[]` | empty array |
| `[$1]` | one element |
| `[$1, $2]` | two elements |
| `[$1, $2*]` | head `$1` then the spread of `$2` |
| `[$1, $3::2*]` | head `$1` then the extraction-spread of `$3` (the canonical list idiom) |

## List shape: flat vs. cons

How `properties` / `elements` are laid out depends on which spread idiom built the list — this is the
single most important thing to know when walking the AST:

- **`[$1, $2::2*]` — extraction-spread → a FLAT list.** This is what `object_properties` and
  `array_elements` use, and it is the recommended idiom for a `X (sep X)*` pattern: `$2` is the list of
  `(sep, X)` tuples, `::2` pulls the `X` out of each tuple (index 2, 1-based: 1 = separator, 2 = item),
  and `*` spreads them. The result is a flat `[item1, item2, item3, …]` in source order. Walk it as an
  ordinary array.
- **`[$1, $3*]` — plain spread → a CONS list.** Some grammars (e.g. `json.ebnf`'s `members`/`elements`)
  use a plain `*` spread, which produces the nested `[head, [next, [next, …]]]` cons-shape. There the
  first element is the head and the second is the *tail list* of the rest; walk it recursively.

The return-annotation grammar itself uses the **flat** `::2*` idiom for its own `properties`/`elements`,
so when you parse an annotation and read those fields you get flat, source-order lists. Choose `::2*`
in your own grammars when you want a flat list and `*` when a cons-shape is acceptable; see
[Operators](operators.md) for the precise mechanics, and the
[Operators](operators.md#known-limitations-of-spread-forms) limitations on mixed spread forms.
