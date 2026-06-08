# AST Envelope Structure

The json parser returns PGEN's standard typed parse node. The top node carries `rule_name`, a `span`, and
a `content` whose `Json` variant holds the shaped object produced by the rule's return annotation. For

```json
{"a": [1, "x", true, null], "b": -2.5}
```

`parseability_probe --parse-dump-ast-pretty json` returns (abridged):

```json
{
  "content": {
    "Json": {
      "type": "json",
      "value": {
        "type": "object",
        "members": [
          { "type": "pair", "key": "\"a\"", "value": {
              "type": "array",
              "elements": [
                { "type": "number", "value": "1" },
                [ { "type": "string", "value": "\"x\"" },
                  [ { "type": "boolean", "value": true },
                    [ { "type": "null" } ] ] ]
              ] } },
          [ { "type": "pair", "key": "\"b\"", "value": { "type": "number", "value": "-2.5" } } ]
        ]
      }
    }
  },
  "rule_name": "json",
  "span": { "start": 0, "end": 38 }
}
```

## Per-node shapes

| Form | Shape |
| --- | --- |
| document | `{ "type": "json", "value": <value> }` |
| object | `{ "type": "object", "members": [ <pair>, [ <pair>, [ … ] ] ] }` (or `[]` when empty) |
| pair | `{ "type": "pair", "key": <string>, "value": <value> }` |
| array | `{ "type": "array", "elements": [ <value>, [ <value>, [ … ] ] ] }` (or `[]` when empty) |
| string | `{ "type": "string", "value": "\"…\"" }` |
| number | `{ "type": "number", "value": "<digits>" }` |
| boolean | `{ "type": "boolean", "value": true \| false }` |
| null | `{ "type": "null" }` |

## Three things to know when consuming the AST

1. **Lists are a cons-shape, not a flat array.** `members` and `elements` come from the `-> [$1, $3*]`
   return annotation, which produces `[head, [next, [next, …]]]` — the first element is the head and the
   second element is the *tail list* of the rest. Walk it recursively (head, then recurse on element `[1]`)
   rather than assuming a flat list. A single-element list is `[ <item> ]`.
2. **String values retain their surrounding quotes.** `string` returns its whole matched text (`-> $1`),
   so the value is `"\"x\""` (the quotes are part of the string), and **no escape decoding is performed**
   (see [Grammar and Scope](grammar-and-scope.md)). Consumers that need the decoded contents must strip the
   quotes and decode themselves.
3. **Numbers are strings.** `number` returns the matched text as a string (`"1"`, `"-2.5"`); convert to a
   numeric type downstream. Booleans (`true`/`false`) and `null` are the only values lowered to native
   JSON scalars.
