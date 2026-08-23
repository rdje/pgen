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

### What a map KEY may be

A map **key** is any value **except one that could itself consume a `=>`**. The value language spells
`=>` in three other roles — the implication operator (`a => b`), the lambda arrow (`x => body`) and the
function-type arrow (`(Foo) => Bar`) — and under PEG's ordered choice a key allowed to be any of those
swallows the map's own arrow and the entry stops parsing. So the key grammar excludes exactly those
three reaches and nothing else:

| may be a key | may **not** be a key |
| --- | --- |
| any primitive — `"s"`, `1`, `true`, `null`, `name` | an implication — `a => b` |
| any structured value — array, object, tuple, set, nested map | a lambda — `x => body`, `(x, y) => body`, `[x] => body` |
| arithmetic / logical-or / comparison / conditional / call | a function type — `(Foo) => Bar` |
| a rule / symbol / path / URL reference, or a non-function type | |

Values are unrestricted, so a lambda or implication is perfectly legal on the right of the arrow —
`{a => (x) => y}` parses, with key `a` and a lambda value.

### What may sit LEFT of a `=>` inside a value

"Unrestricted" is a statement about the value **rule**, not about the arrow. A `=>` *inside* a value
is never the map arrow — the entry already consumed that one — so it has to be one of the value
language's three other arrow constructs, and each declares its own left-hand operand language:

| construct | what may sit left of its `=>` | what may sit right |
| --- | --- | --- |
| lambda | an identifier, a `[a, …]` / `{a, …}` destructuring pattern, or a parenthesised parameter list `(a, b)` | any value |
| implication | a logical-or operand — number, identifier, rule reference, call, qualified name, parenthesised arithmetic | the same |
| function type | a parenthesised type list, `(Foo<Bar>)` | a type reference |

So the key position and the arrow-left position inside a value do **not** admit the same things, and
that is deliberate: a key is followed by the map's own arrow, so it is an operand of nothing, while
an arrow's left operand has to mean something to one of the three constructs above. Measured over 23
operand shapes, all 23 are legal keys, legal plain values and legal arrow *right* operands; 13 of the
23 may also sit left of an arrow inside a value. The other ten — `"s"`, `'s'`, `[1]`, `{a: 1}`,
`#{1}`, a nested map, `%S`, `./p`, `https://h/p`, `Foo<Bar>` — may not, because none of them is a
lambda parameter, a logical operand or a parenthesised type list:

```text
@ x : { %S => 1 }                   # fine — a symbol reference is a perfectly good KEY
@ x : { k => %S => 1 }              # rejected — `%S => 1` is no lambda, implication or function type
@ x : { k => (Foo<Bar>) => Baz }    # fine — that one IS a function type
@ x : { k => a => b => c }          # fine — identifier arrows chain as nested lambdas
```

Write the nested value with its own braces when you want a map inside a map — `{k => {%S => 1}}` —
which is what the ten shapes above are asking for whenever they turn up in that position.

> ⚠️ **Before 2026-08-23 this was broken and the failure was silent.** The key was declared as a full
> value, so a map entry parsed *if and only if* its `key => value` was **not** itself a valid value.
> Measured over nine key shapes, only the string-keyed one worked: `{"a" => "b"}` parsed while
> `{1 => 2}`, `{a => b}`, `{[a] => b}`, `{(a) => b}`, `{(a, b) => c}`, `{true => false}`,
> `{(Foo) => Bar}` and `{Foo => Bar}` were all rejected. `GRAMMAR-WELLFORMED.H.16.6`/`.6a`/`.6b`.
> The repair only **widens** — measured over 1 211 inputs, 25 newly accepted, 0 newly rejected, and 0
> typed ASTs moved — because a key that had swallowed the arrow could never finish its entry anyway.

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

#### Where a path or URL ends — and how to put a delimiter inside one

A path or a URL is written **unquoted**, so the parser needs a rule for where it stops. It stops at
whitespace, at any of this value language's structural delimiters — `,` `]` `}` `)` — and at `>`
(RFC 3986 §2 excludes `<` and `>` from a URI outright). It also may not *end* on `=`, which is what
lets an unquoted URL sit immediately left of the map arrow:

```text
@ x : [ https://example.com/p ]          # the `]` closes the array
@ x : { https://example.com/p=>1 }       # the `=>` is the map arrow, not part of the URL
@ x : https://example.com/p?a=1&b=2      # `=` is fine INSIDE a URL — query strings parse
```

To put one of those characters **into** a path or URL, escape it with a backslash. This is the same
convention the string literals use for their own quote character, and it works in every position:

```text
@ x : https://example.com/a\,b           # a comma inside the URL
@ x : [ https://example.com/a\,b , 1 ]   # …and the bare comma still separates the array
@ x : { k => /opt/a\,b }
@ x : https://example.com/p?a\=          # a trailing `=`
```

Quoting the whole value works too, and is the better choice when a value has several such
characters:

```text
@ x : "https://example.com/a,b(c)"
```

> ⚠️ **Changed in `GRAMMAR-WELLFORMED.H.16.6d`.** These four terminals were previously *"any run of
> non-whitespace"*, which meant a path or URL abutting a closing delimiter **swallowed it** —
> `[ftp://server]` failed to parse while `ftp://server]` parsed as one complete value, and adding a
> space (`[ftp://server ]`) was the only workaround. If you were relying on a bare `,` `]` `}` `)`
> or `>` inside an unquoted path or URL, escape it or quote the value.

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

- if X has a `->` return annotation that produces an object (`ParseContent::Shaped`, serialized as `"Json"`), `$name` is a
  key/path lookup into the **shaped** structure down to a scalar leaf (String/Number/Bool as-is; an
  absent key, a non-object intermediate, `Null`, or a non-scalar leaf → unresolved);
- if X has no `->`, resolution is the raw sub-rule-name descendant search.

### Positional references — `$N` (working since POSITIONAL-PAYLOAD-REFS.2)

A **positional** head (`$1`, `$2`, … — 1-indexed over the rule's parsed elements, literals included)
resolves against the annotated rule's own content, with optional dotted / indexed segments chained
behind it. Since `POSITIONAL-PAYLOAD-REFS.2` (2026-07-06) positional references **resolve** from
compiled grammar directives; before that fix the payload compiler stripped the `$` sigil and a
positional payload reference always hard-failed. All three forms are differentially pinned
(generated parser ≡ interpreter) by the `sem_ref_positional` suite case:

```text
@emit_fact: { kind: pf, name: $2, family: p }                     # plain: position 2's text
mk := "(" word ")"                                                 # $2 = the word element → "a"

@predicate: { name: has_fact, args: [pf, $2.word], phase: post }   # dotted from a position
use := "[" pair "]"                                                # $2 = pair → first word below it

@predicate: { name: has_fact, args: [pf, $2[0][2]], phase: post }  # chained-indexed
idx := "{" pair "}"                                                # [0] unwraps, [2] = second word
```

Raw-tree walk semantics an author must know (AST-dump-established, pinned differentially):

- **positions count every element** — literals occupy positions (`mk := "(" word ")"` puts `word`
  at `$2`);
- **a position that binds a rule wraps that rule's node** — a dotted segment may **self-match** the
  wrapped rule's name (`$2.word` above resolves even though position 2 *is* the `word` element);
- **indexing into a bound rule's children needs the `[0]` unwrap first** — `$2[0][2]`, not `$2[2]`
  (`[0]` unwraps the position's rule-node wrapper; the next index walks the rule's real children,
  0-based, literals included);
- **an unresolvable reference is a hard error** — a segment walking into a literal/terminal element
  (e.g. `$3.word` where position 3 is `"]"`) fails the whole rule, identically on the generated
  parser and the interpreter (pinned by `sem_ref_positional_deep_unresolvable`).
