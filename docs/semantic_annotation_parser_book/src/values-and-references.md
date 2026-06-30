# Annotation Values and References

This chapter details the value language — everything that can appear after the colon in
`@name: value`. All shapes are derived from `grammars/semantic_annotation.ebnf`.

## Primitive values

| Kind | Forms | Example | Parsed node (abridged) |
| --- | --- | --- | --- |
| string | double `"…"`, single `'…'`, raw `r"…"`, multiline `"""…"""`, template `` `…` `` | `"pure"` | `{type: "string", value: …, quote_style: "double"}` |
| integer | decimal, hex `0x…`, binary `0b…`, octal `0o…` | `42`, `0xFF`, `0b1010`, `0o17` | `{type: "integer", value: …, base: 10\|16\|2\|8}` |
| decimal | `[+-]?\d*.\d+` | `-2.5` | `{type: "decimal", value: …}` |
| scientific | `…[eE]±…` | `1.5e9` | `{type: "scientific", value: …}` |
| boolean | `true`/`false`/`yes`/`no`/`on`/`off`/`enabled`/`disabled`/`active`/`inactive` | `true` | `{type: "boolean", value: …}` |
| null | `null`/`nil`/`none`/`undefined`/`void` | `null` | `{type: "null", value: …}` |
| identifier | `[a-zA-Z_][a-zA-Z0-9_]*` | `pure` | `{type: "identifier", name: …}` |

## Structured values

| Kind | Form | Example | Parsed node |
| --- | --- | --- | --- |
| array | `[ e, … ]` | `[A, B, C]` | `{type: "array", elements: […]}` |
| object | `{ k: v, … }` | `{level: 5, assoc: "left"}` | `{type: "object", properties: […]}` |
| tuple | `( e, … )` | `(a, b)` | `{type: "tuple", elements: […]}` |
| set | `#{ e, … }` | `#{x, y}` | `{type: "set", elements: […]}` |
| map | `{ k => v, … }` | `{input => output}` | `{type: "map", entries: […]}` |

Object properties may be a plain `key: value`, a computed key `[expr]: value`, a shorthand `name`, or a
spread `...value`. An array/argument element may also be a spread `...value`. (Object and map share the
`{ … }` delimiters; a `=>` between key and value makes it a map entry.)

## Expression values

The value language includes a full expression sublanguage — useful for `@predicate`, `@constraint`,
and computed metadata:

- **arithmetic** — `+ - * / % // **` with unary `+ - ! ~`, e.g. `base * 2 + offset`;
- **logical** — `||`, `&&`, `!`, and implication `=>`, e.g. `enabled && (debug || test)`;
- **comparison** — `== != < > <= >= === !== <=> ~= =~ !~`, e.g. `x > 0`;
- **conditional** (ternary) — `cond ? a : b`, e.g. `enabled ? "on" : "off"`;
- **function call** — `name(args…)` with positional / named / spread arguments, e.g.
  `has_fact(type_name, $head)` or `calculate(base * 2 + offset)`;
- **qualified name** — dotted `a.b.c`;
- **lambda** — `(params) => body` or `p => body`.

## Reference values

References point at things outside the literal value:

| Kind | Form | Example | Parsed node |
| --- | --- | --- | --- |
| rule reference | `$head` + dotted/indexed chain | `$head`, `$1.body`, `$matrix[0][1]` | `{type: "rule_reference", name: …}` |
| symbol reference | `%name` | `%TOKEN` | `{type: "symbol_reference", name: …}` |
| type reference | primitive `T`, generic `T<…>`, union `A\|B`, intersection `A&B`, function `(…)=>T`, array `T[]`, optional `T?` | `Map<String, List<Integer>>` | `{type: "generic_type", …}` |
| path reference | absolute `/…`, relative `./…`/`../…`, home `~/…` | `./src/p.sv` | `{type: "relative_path", path: …}` |
| URL reference | `(https?\|ftp\|file)://…` | `https://example.com` | `{type: "url_reference", url: …}` |

### Rule references — `$…` (dotted + indexed, depth-unbounded)

The `$<ref>` reference is the one you will use most (it is how a directive payload names a captured
sub-result). It accepts named heads (`$head`), positional heads (`$1`, 1-indexed), dotted property
segments (`.field`), and non-negative integer index segments (`[i]`), chained to **unbounded depth**:

```text
$head
$1.body
$1.body.subkey
$items[0]
$matrix[0][1]
$a.b[0].c[1].d.e[2].r.z
```

This is a deliberate **subset** — dotted property + non-negative integer indexing only — **not** full
JSONPath: no filters (`[?(@.x)]`), wildcards (`*`), recursive descent (`..`), negative indices, or
range slices. Each excluded feature would need its own normative leaf. Malformed forms (a bare trailing
`.`, or `[` with no `digits]`) roll back to before the offending segment rather than being silently
swallowed.

> **Two runtime surfaces, one accepted set.** Authors writing directives *inside* a grammar `.ebnf` hit
> the directive-payload runtime (`unified_semantic_ast.rs::StructuredSemanticValueParser`); freestanding
> annotation strings through the embedding API hit this EBNF surface. Both accept the same `$ref` set.
> See [Backends](backends.md).

### Shaped vs. raw reference resolution

When a `$name` / `$a.b` reference resolves against a rule X (SEMREF-SHAPED, 2026-05-18):

- if X has a `->` return annotation that produces an object (`ParseContent::Json`), `$name` is a
  key/path lookup into the **shaped** structure down to a scalar leaf (String/Number/Bool as-is; an
  absent key, a non-object intermediate, `Null`, or a non-scalar leaf → unresolved);
- if X has no `->`, resolution is the raw sub-rule-name descendant search.
