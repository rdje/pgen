# ParseContent Variants

`ParseContent<'input>` has six variants. This chapter describes each one, when it appears, and how to handle it.

```rust
pub enum ParseContent<'input> {
    Terminal(&'input str),
    TransformedTerminal(String),
    Json(serde_json::Value),
    Sequence(Vec<ParseNode<'input>>),
    Alternative(Box<ParseNode<'input>>),
    Quantified(Vec<ParseNode<'input>>, &'static str),
}
```

The variants are externally tagged in JSON dumps — the JSON object key names match the Rust variant name.

## Variant: `Terminal(&'input str)`

A literal matched-text terminal. The string slice borrows from the input.

**JSON form:**

```json
{ "Terminal": "matched_text" }
```

**When it appears:**

- Single-character grammar atoms emitted as terminals: `"["`, `"]"`, `"^"`, `"\\Q"`, `"\\E"`, etc.
- Regex-literal rules (rules whose body is a `/.../` pattern), where the captured group becomes the terminal value: `letter`, `digit`, `whitespace`, `literal_char`, `class_literal`, `class_safe_special`, `any_char`, `special_char`, `unicode_char`.

Example — a regex literal `a` parsed:

```json
{
  "rule_name": "letter",
  "span": { "start": 0, "end": 1 },
  "content": { "Terminal": "a" }
}
```

For consumers walking, `Terminal` is the terminal case of recursion — there are no children to descend into.

## Variant: `TransformedTerminal(String)`

An owned String typically produced by `@transform` semantic annotations that coerce a regex match through a Rust expression. Unlike `Terminal(&str)`, this variant owns its string (because the transform may have produced a value not borrowable from the input).

**JSON form:**

```json
{ "TransformedTerminal": "transformed_value" }
```

**When it appears:**

In the regex parser today, `digits` uses `@transform: str::parse::<usize>().unwrap_or(0)` to coerce its match to an integer. The integer becomes a typed number carried via the shaped carrier, NOT a `TransformedTerminal`. So in **current** regex output, `TransformedTerminal` rarely appears at the top of the AST.

It may still appear in deeper subtrees for legacy / fallback cases. Consumers should handle it analogously to `Terminal` — it's a leaf scalar.

## Variant: `Shaped(PgenValue<'input>)` — serialized as `"Json"`

The typed structured carrier (since the REPRESENTATION landing, 2026-07-16; earlier releases carried the same logical value as `Json(serde_json::Value)`, retired 2026-07-17). `PgenValue` is an arena-backed `Copy` value mirroring the six JSON value types variant-for-variant — `Null` / `Bool` / `Int`+`UInt`+`Float` / `Str` / `Array` / `Object` (objects key-sorted exactly like `serde_json::Map`) — so it holds any JSON shape, including **null** (the `null` literal landed in the slice that introduced typed `counted_quantifier_body` to mark the unbounded `{n,}` form). It serializes under the `"Json"` wire tag with byte-identical output, so dumps and downstream JSON consumers see no difference from the retired owned-`Value` carrier.

**JSON form:**

```json
{ "Json": <any-json-value> }
```

**When it appears:**

Whenever a grammar rule carries an explicit return annotation that produces a typed shape:

- Object literal: `-> {type: "regex", pattern: $1}` produces `Shaped(Object(...))`.
- Array literal: `-> [$1, $2*]` produces `Shaped(Array(...))`.
- String literal: `-> "lazy"` produces `Shaped(Str("lazy"))`.
- Number literal: `-> 0` produces `Shaped(Int(0))` (integer-preserving).
- Boolean literal: `-> true` produces `Shaped(Bool(true))`.

For the regex parser today, `Shaped` is the dominant top-level variant because the entry rule (`regex`) is annotated.

**Walking a Shaped variant:**

Either convert once at your boundary — `to_serde_value()` yields the byte-identical owned `serde_json::Value` — or pattern-match `PgenValue` natively to skip the copy (objects are key-sorted pair slices):

```rust
use pgen::ast_pipeline::PgenValue;

match &node.content {
    ParseContent::Shaped(value) => {
        match value {
            PgenValue::Object(pairs) => {
                // key-sorted — binary_search is the Map::get equivalent
                // (the same idiom the generated parser emits internally)
                let kind = pairs
                    .binary_search_by(|(key, _)| key.as_bytes().cmp("type".as_bytes()))
                    .ok()
                    .map(|i| pairs[i].1);
                // ... handle each "type" discriminator
            }
            PgenValue::Array(items) => {
                // ... iterate items
            }
            // ...
        }
    }
    other => { /* handle other variants */ }
}
```

## Variant: `Sequence(Vec<ParseNode<'input>>)`

A flat sequence of child nodes. Used when a grammar rule's body is a multi-element Sequence or when an annotation explicitly produces a flat array.

**JSON form:**

```json
{ "Sequence": [ <node>, <node>, ... ] }
```

**When it appears:**

- Multi-element sequence rule bodies WITHOUT a return annotation: e.g. `(?:` `pattern?` `)` becomes a 3-element Sequence at the `noncapturing_group` level today.
- Annotation-produced flat arrays: `concatenation = piece+ -> [$1**]` produces a flat `Sequence` of piece nodes (the `**` flatten-spread unwraps any nested Sequences/Quantifieds one level).

Example — `[a-z]`'s class_body is a Sequence of class_item nodes, not yet annotated.

## Variant: `Alternative(Box<ParseNode<'input>>)`

A boxed inner node, used when wrapping a single chosen alternative from an Or rule. Mostly an internal codegen artifact.

**JSON form:**

```json
{ "Alternative": <node> }
```

**When it appears:**

- Or rules whose body is `A | B | C ...` — the matched alternative is wrapped in an `Alternative` envelope.
- Some Sequence elements (synthetic `element_N` wrappers) where the codegen emits Alternative-wrapped child nodes.

Most consumers walking annotated rules don't pattern-match `Alternative` directly — they rely on the rule's annotation to unwrap. For unannotated Or rules (like `atom`), consumers walking the AST DO need to handle this variant.

```rust
ParseContent::Alternative(boxed_inner) => {
    // recurse on the inner node
    walk(&*boxed_inner)
}
```

## Variant: `Quantified(Vec<ParseNode<'input>>, &'static str)`

A repetition group from a `*` / `+` / `?` grammar quantifier.

**JSON form:**

```json
{ "Quantified": [ [ <node>, <node>, ... ], "*" ] }
```

The `&'static str` is the quantifier marker — `"*"`, `"+"`, or `"?"` — exactly as it appears in the grammar source.

**When it appears:**

- `?`-quantified rule bodies: zero or one matches. Vec is empty (no match) or 1-element (matched).
- `*`-quantified rule bodies: zero or more matches. Vec contains all repetitions.
- `+`-quantified rule bodies: one or more matches. Vec has at least one element.

Example — for input `a` (no trailing quantifier), `piece`'s `quantifier?` slot emits `Quantified([], "?")`, which serializes as `"quantifier": []` in the typed shape.

For `a*`, the typed `quantifier` shape carries the fully-typed `{"type":"quantifier","min":0,"max":null,"greediness":[]}` (annotated as of slice 6, post-1.1.34). The whole quantifier subtree is annotated — see [Quantifier Subtree](rules-quantifier.md).

## Special interactions

Consumers should be aware of three interactions between variants:

### 1. `to_json_value()` collapses everything to `serde_json::Value`

`ParseContent::to_json_value()` walks any variant and produces a `serde_json::Value`. The mapping:

| Variant | `to_json_value()` output |
|---|---|
| `Terminal(s)` | `Value::String(s.to_owned())` |
| `TransformedTerminal(s)` | parsed-as-JSON if valid, else `Value::String(s)` |
| `Shaped(v)` | `v.to_serde_value()` (the byte-identical owned `Value`) |
| `Sequence(nodes)` | `Value::Array(<each node's content.to_json_value()>)` |
| `Alternative(node)` | `node.content.to_json_value()` (transparent unwrap) |
| `Quantified(nodes, _)` | `Value::Array(<each node's content.to_json_value()>)` |

This is what the byte-equivalence guarantee for `parse_full_regex().content.to_json_value()` output is built on (the removed `parse_regex_typed()` entry returned this same value).

### 2. Alternative is transparent in JSON output

Because `Alternative.to_json_value()` unwraps, the JSON output never has `{"Alternative": ...}` keys at the JSON-serialisation level for `to_json_value()` paths. The `Alternative` variant is visible only when consumers walk the `ParseNode` tree directly, not when they `to_json_value()` it.

In `parse-dump-ast-pretty` output (the raw envelope-level dump), `Alternative` variants ARE visible because that path uses `serde::Serialize` on the `ParseContent` enum.

### 3. The Json carrier "swallows" inner structure

When a rule emits the typed carrier (`Shaped(typed_value)`), the typed value is a fully-flattened value tree. If a consumer wants the source spans of inner sub-rules that contributed to that typed value, they're not directly accessible from inside the carrier — only the values are. The spans would have to come from walking the un-`to_json_value()`d `ParseNode` tree, which means traversing the legacy recursive variants for unannotated subrules.

This is a known tradeoff: typed-Json output is consumer-convenient but loses span fidelity at the layer where the annotation flattened. Fully-annotated grammars (the eventual goal of task #40) will need either:

- Source-span preservation via a different annotation primitive (future feature); or
- Consumer logic that walks the legacy ParseNode tree for span access while consuming the typed JSON for structure.

For RGX's current adapter refactor, the typed-Json shape is sufficient for matching/compilation; if span reporting is needed it can come from the un-walked envelope.
