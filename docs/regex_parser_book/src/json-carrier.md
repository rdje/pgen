# The Json Carrier

`ParseContent::Json(serde_json::Value)` is the variant downstream consumers will spend most of their time inside. This chapter explains where it comes from, when it appears, and how consumers should treat it.

## What it is

A `Json(serde_json::Value)` is the runtime representation of the typed shape that a grammar rule's return annotation produces. It carries any of the six JSON value types:

- `Value::Object(Map<String, Value>)` — for `-> {...}` annotations.
- `Value::Array(Vec<Value>)` — for `-> [...]` annotations.
- `Value::String(String)` — for `-> "..."` annotations.
- `Value::Number(Number)` — for numeric literal annotations and integer-coerced `@transform` matches.
- `Value::Bool(bool)` — for `-> true`/`-> false` annotations.
- `Value::Null` — for `-> null` annotations (added in the slice that introduced typed `counted_quantifier_body` `{min, max:null}` for the unbounded `{n,}` form).

## When it appears

The codegen emits `ParseContent::Json(...)` whenever a rule has an explicit return annotation that lifts the rule's output into a typed shape. The current set of annotated rules in `grammars/regex.ebnf`:

| Rule | Annotation | Json shape produced |
|---|---|---|
| `regex` | `-> {type: "regex", pattern: $1}` | Object `{type, pattern}` |
| `pattern` | `-> $1` | Whatever `alternation` produced (transparent passthrough) |
| `concatenation` | `-> [$1**]` | Array (flat, via `**` flatten-spread) |
| `piece` (branch 0) | `-> $1` | Whatever `piece_quoted_run_quantified` produced |
| `piece` (branch 1) | `-> {type: "piece", atom: $1, quantifier: $2}` | Object `{type, atom, quantifier}` |
| `piece_quoted_run_quantified` | `-> [$2**, {type: "piece", atom: $3, quantifier: $5}]` | Array of piece-objects |
| `quoted_run_inner_piece` | `-> {type: "piece", atom: $1, quantifier: []}` | Object `{type, atom, quantifier:[]}` |
| `counted_quantifier` | `-> $3` | Whatever `counted_quantifier_body` produced (transparent passthrough) |
| `counted_quantifier_body` (branch 0) | `-> {min: $1, max: $3}` | Object `{min, max}` (`{n,m}` form) |
| `counted_quantifier_body` (branch 1) | `-> {min: $1, max: null}` | Object `{min, max:null}` (`{n,}` unbounded form) |
| `counted_quantifier_body` (branch 2) | `-> {min: $1, max: $1}` | Object `{min, max}` (`{n}` form, max == min) |
| `counted_quantifier_body` (branch 3) | `-> {min: 0, max: $3}` | Object `{min:0, max}` (`{,m}` form) |
| `anchor` (branch 0..8) | `-> {type: "anchor", kind: "<name>"}` per branch | Object `{type:"anchor", kind:<name>}` — `kind` ∈ `start_of_line` / `end_of_line` / `start_of_input` / `end_of_input_or_before_last_newline` / `end_of_input` / `word_boundary` / `non_word_boundary` / `match_start` / `keep_out` |
| `posix_word_boundary_alias` (branch 0) | `-> {type: "anchor", kind: "posix_word_start"}` | Same anchor family as the `anchor` rule (kind = `posix_word_start`) |
| `posix_word_boundary_alias` (branch 1) | `-> {type: "anchor", kind: "posix_word_end"}` | Same anchor family as the `anchor` rule (kind = `posix_word_end`) |
| `backreference` (branch 0) | `-> {type: "backreference", kind: "numeric", index: $2}` | Object `{type, kind:"numeric", index:<int>}` |
| `backreference` (branch 1) | `-> {type: "backreference", kind: "named", ref: $2}` | Object `{type, kind:"named", ref:<raw name_ref shape>}` |
| `backreference` (branch 2) | `-> {type: "backreference", kind: "named_braced", ref: $2}` | Object `{type, kind:"named_braced", ref:<raw braced_name_ref shape>}` |
| `backreference` (branch 3) | `-> {type: "backreference", kind: "subroutine", ref: $2}` | Object `{type, kind:"subroutine", ref:<raw subroutine_ref shape>}` |
| `backreference_digits` | `@transform: str::parse::<usize>().unwrap_or(0)` | Number (integer) |
| `name_ref` (branch 0, angle) | `-> $2` | Whatever `name` produced (typed name string) |
| `name_ref` (branch 1, quote) | `-> $2` | Whatever `name` produced (typed name string) |
| `braced_name_ref` | `-> $3` | Whatever `name` produced (typed name string) |
| `name` | regex literal `/(...)/` | Terminal of the matched name string (clean, no chain) |
| `subroutine_ref` (branch 0, braced) | `-> $1` | Whatever `braced_subroutine_ref` produced |
| `subroutine_ref` (branch 1, angle) | `-> $2` | Whatever `signed_digits_or_name` produced (string for name, or `[<sign?>, <int>]` for digits) |
| `subroutine_ref` (branch 2, quote) | `-> $2` | Same as branch 1 |
| `subroutine_ref` (branch 3, signed_digits) | `-> $1` | Whatever `signed_digits` produced (typed `{sign, value}` object) |
| `braced_subroutine_ref` | `-> $3` | Whatever `signed_digits_or_name` produced |
| `signed_digits` | `-> {sign: $1, value: $2}` | Object `{sign:<"+"|"-"|[]>, value:<int>}` |
| `posix_class` | `-> {type: "posix_class", name: $3, negated: $2}` | Object `{type:"posix_class", name:<str>, negated:<true \| []>}` |
| `posix_negation` | `-> true` | Boolean `true` (matched), or `[]` from the un-matched `posix_negation?` slot |
| `quant_base` (branch 0 `*`) | `-> {min: 0, max: null}` | Object `{min:0, max:null}` (unbounded zero-or-more) |
| `quant_base` (branch 1 `+`) | `-> {min: 1, max: null}` | Object `{min:1, max:null}` (unbounded one-or-more) |
| `quant_base` (branch 2 `?`) | `-> {min: 0, max: 1}` | Object `{min:0, max:1}` (zero-or-one) |
| `quant_base` (branch 3 `counted_quantifier`) | `-> $1` | Whatever `counted_quantifier` produced (typed `{min, max}`) |
| `quantifier` | `-> {type: "quantifier", min: $1.min, max: $1.max, greediness: $2}` | Object `{type, min, max, greediness}` |
| `quant_suffix` (branch 0) | `-> "lazy"` | String `"lazy"` |
| `quant_suffix` (branch 1) | `-> "possessive"` | String `"possessive"` |
| `digits` | `@transform: str::parse::<usize>().unwrap_or(0)` | Number (integer) |
| `posix_class` | `-> $1` | Whatever the matched element produced |

Rules NOT in this list produce non-`Json` content (`Sequence`, `Quantified`, `Terminal`, `Alternative`) — they inherit the legacy recursive-envelope shape pending future annotation slices.

## How a consumer should treat it

### Pattern matching

```rust
use serde_json::Value;

fn walk(node: &ParseNode) {
    match &node.content {
        ParseContent::Json(value) => walk_json(value),
        ParseContent::Sequence(nodes) => nodes.iter().for_each(walk),
        ParseContent::Alternative(boxed) => walk(boxed),
        ParseContent::Quantified(nodes, _marker) => nodes.iter().for_each(walk),
        ParseContent::Terminal(s) => leaf_terminal(s),
        ParseContent::TransformedTerminal(s) => leaf_terminal(s),
    }
}

fn walk_json(value: &Value) {
    match value {
        Value::Object(map) => {
            // Most-common shape — inspect "type" discriminator.
            match map.get("type").and_then(|v| v.as_str()) {
                Some("regex") => /* {pattern: ...} */,
                Some("piece") => /* {atom, quantifier} */,
                Some(other) => /* unknown — log */,
                None => /* untagged object — see per-rule shapes */,
            }
        }
        Value::Array(items) => /* iterate */,
        Value::String(s) => /* leaf string, e.g. "lazy" */,
        Value::Number(n) => /* integer or float */,
        Value::Bool(b) => /* true/false */,
        Value::Null => /* explicit absence — e.g. unbounded max */,
    }
}
```

### Discriminator convention

Most object-shaped `Json` values carry a `"type"` field as their discriminator. The current discriminators in regex output:

- `"regex"` — emitted by the `regex` rule.
- `"piece"` — emitted by all piece-emitting rules.

Counted-quantifier bodies (the typed `{min, max}` shape) and quant_suffix outputs do NOT have `"type"` discriminators because their shape is unambiguous from context (they always appear inside a `quantifier` slot).

When walking, treat `"type"` as the canonical discriminator when present. Don't rely on field-presence as a proxy for type — that will silently break when a future shape adds optional fields.

### Mixed Json + Sequence in the same level

The `concatenation` rule's `[$1**]` flatten-spread produces a `Sequence` of `ParseNode`s, where each child node may have its own `Json(piece_obj)` content. So when walking pattern's contents, you'll see:

```text
ParseNode { rule_name: "concatenation", content: Sequence([
  ParseNode { rule_name: "...", content: Json({piece}) },
  ParseNode { rule_name: "...", content: Json({piece}) },
  ParseNode { rule_name: "...", content: Json({piece}) },
])}
```

(The synthetic rule_names of children aren't important — they're codegen artifacts.) When you `to_json_value()` this, you get a flat array of piece-objects, which is what the regex JSON dump shows.

For consumers walking the typed shape via `to_json_value()`, the Sequence wrapper is transparent — you just see the array. For consumers walking the ParseNode tree, you do have to descend through the Sequence to get to each piece's Json content.

## Why typed-Json was chosen over typed Rust enums

Two reasons:

1. **Annotation flexibility.** The annotation language can produce arbitrary JSON shapes. Typed Rust enums would have required generating one Rust type per shape, which is intractable for grammars with rich return annotations.
2. **Consumer ergonomics.** Most downstream consumers either serialize to JSON anyway or want to inspect by `as_str()` / `as_object()` etc. — `serde_json::Value` is the natural ergonomic choice.

The downside is that consumers don't get compile-time type checking on the shape — they have to validate at runtime. For the regex grammar, the per-rule shape reference chapters serve as the de-facto schema documentation.

## Why the Json variant exists at all (vs just emitting the inner Value)

The `ParseContent` enum has to carry several shapes (raw `Sequence`, recursive `Alternative`, etc.). `Json` is the variant that says "this content is a fully-typed value, not a recursive AST shape." Wrapping it in a `Json` variant lets the same `ParseContent` enum carry both shapes. Consumers that want to flatten everything to JSON call `to_json_value()`.

## Stability

The set of `serde_json::Value` shapes a given rule emits is documented per-rule and per-release. The `Json` variant itself is stable across PGEN 1.1.x — once a rule is annotated, that rule's annotation is part of the contract. Removing or substantially changing the shape requires a contract version bump.
