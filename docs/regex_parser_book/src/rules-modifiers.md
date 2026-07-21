# Modifier and Inline-Modifier Subtree

PCRE2 inline modifiers like `(?i)`, `(?-mx)`, `(?^x:...)` etc. **Both atom-level rules
are annotated** (the atom-subtree typed-shape campaign): consumers receive typed
`{type: "atom", kind: ...}` objects with a structured `spec` — no raw sequence walking
is needed anywhere in this subtree.

## `inline_modifiers`

```ebnf
inline_modifiers = "(?" modifier_spec? ")"
```

The setting-only form — applies from this point in the enclosing group. Typed shape:

```json
{ "type": "atom", "kind": "inline_modifiers", "spec": <spec> }
```

Live examples (exact probe output):

| Pattern | Emitted atom |
|---|---|
| `(?i)` | `{"type":"atom","kind":"inline_modifiers","spec":{"reset":false,"seq":{"set":["i"],"unset":[]}}}` |
| `(?imsx)` | `{"type":"atom","kind":"inline_modifiers","spec":{"reset":false,"seq":{"set":["i","m","s","x"],"unset":[]}}}` |
| `(?-i)` | `{"type":"atom","kind":"inline_modifiers","spec":{"reset":false,"seq":{"set":[],"unset":["i"]}}}` |
| `(?^)` | `{"type":"atom","kind":"inline_modifiers","spec":{"reset":true,"seq":[]}}` |
| `(?^i)` | `{"type":"atom","kind":"inline_modifiers","spec":{"reset":true,"seq":{"set":["i"],"unset":[]}}}` |
| `(?)` | `{"type":"atom","kind":"inline_modifiers","spec":[]}` (absent optional spec — consumer maps `[]` to "no-op") |

## `scoped_inline_modifiers`

```ebnf
scoped_inline_modifiers = "(?" modifier_spec ":" pattern? ")"
```

The scoped form — the modifiers apply only to the embedded pattern. Typed shape:

```json
{ "type": "atom", "kind": "scoped_inline_modifiers", "spec": <spec>, "body": <pattern> }
```

For `(?i-mx:foo)`:

```json
{
  "type": "atom",
  "kind": "scoped_inline_modifiers",
  "spec": { "reset": false, "seq": { "set": ["i"], "unset": ["m", "x"] } },
  "body": [[[
    { "atom": "f", "quantifier": [], "type": "piece" },
    { "atom": "o", "quantifier": [], "type": "piece" },
    { "atom": "o", "quantifier": [], "type": "piece" }
  ]], []]
}
```

`body` is the standard inner `pattern` carrier (walk it exactly like the top-level
pattern — see [Walking the AST](walking-the-ast.md)).

## The `spec` object

Produced by `modifier_spec` and folded into the parent atom — consumers never see the
`modifier_spec`/`modifier_seq`/`modifier_group`/`modifier_item` sub-rules raw:

```ebnf
modifier_spec = "^" modifier_seq?
              | modifier_seq
modifier_seq  = modifier_group ("-" modifier_group)?
              | "-" modifier_group
modifier_group = modifier_item+
modifier_item  = "a" ascii_restrict_modifier?
               | "x" "x"?
               | modifier_char
ascii_restrict_modifier = "D" | "S" | "W" | "P" | "T"
modifier_char = "i" | "m" | "s" | "U" | "J" | "n" | "r"
```

| Field | Value | Meaning |
|---|---|---|
| `reset` | `true` / `false` | `true` iff the `^` reset form was used (`(?^...)` — PCRE2 "unset everything, then set") |
| `seq` | `{set, unset}` object, or `[]` | the modifier items; `[]` when the reset form has no trailing seq (`(?^)`) |
| `seq.set` | array of items | items before the `-` |
| `seq.unset` | array of items | items after the `-` (empty when there is no `-` part) |

Each item in `set` / `unset` is one of:

- a **plain string** — a single modifier letter (`"i"`, `"m"`, `"s"`, `"U"`, `"J"`,
  `"n"`, `"r"`, `"x"`) or the doubled `"xx"` (PCRE2 extended-more mode);
- an **ASCII-restrict object** `{"char": "a", "restrict": <letter>}` for the
  `a`-with-restriction forms (`(?aD)` → `{"char":"a","restrict":"D"}`; restrict ∈
  `D S W P T`). A bare `a` (no restriction letter) emits with `"restrict": []`.

Mixed example — `(?aDx-imr)`:

```json
{ "reset": false,
  "seq": { "set": [ {"char": "a", "restrict": "D"}, "x" ],
           "unset": [ "i", "m", "r" ] } }
```

Unknown letters reject at parse time, so a consumer can trust every item in
`set`/`unset` to be a recognized modifier form.

Consumer extraction:

```rust
fn extract_modifiers(atom: &Value) -> Option<(bool, Vec<Value>, Vec<Value>)> {
    let obj = atom.as_object()?;
    let kind = obj.get("kind")?.as_str()?;
    if kind != "inline_modifiers" && kind != "scoped_inline_modifiers" {
        return None;
    }
    let spec = obj.get("spec")?;
    // absent spec (`(?)`) serializes as []
    let Some(spec_obj) = spec.as_object() else { return Some((false, vec![], vec![])) };
    let reset = spec_obj.get("reset").and_then(|v| v.as_bool()).unwrap_or(false);
    let (mut set, mut unset) = (vec![], vec![]);
    if let Some(seq) = spec_obj.get("seq").and_then(|v| v.as_object()) {
        if let Some(arr) = seq.get("set").and_then(|v| v.as_array()) { set = arr.clone(); }
        if let Some(arr) = seq.get("unset").and_then(|v| v.as_array()) { unset = arr.clone(); }
    }
    Some((reset, set, unset))
}
```

## Historical note

Before the atom-subtree campaign typed this family, both rules emitted raw sequences
(`["(?", <modifier_spec?>, ")"]` and the 5-element scoped form) and consumers walked
the nested `modifier_seq`/`modifier_group` arrays by position. If you maintain code
from that era, migrate to the `kind`/`spec` field reads above — the raw shapes no
longer exist in any released artifact. (See the [Changelog Index](changelog-index.md)
for the release-by-release record.)
