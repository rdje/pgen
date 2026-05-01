# Examples: Anchors and Boundaries

Concrete probe outputs for PCRE2 anchors and word-boundary constructs. As of slice 7 (post-1.1.35) the `anchor` rule emits a typed `{type: "anchor", kind: "<name>"}` object — consumers read `.kind` directly instead of dispatching by string match on the raw escape text.

## `^` start anchor

```json
"pattern": [
  [[
    { "atom": {"type": "anchor", "kind": "start_of_line"}, "quantifier": [], "type": "piece" }
  ]],
  []
]
```

## `$` end anchor

```json
"pattern": [
  [[
    { "atom": {"type": "anchor", "kind": "end_of_line"}, "quantifier": [], "type": "piece" }
  ]],
  []
]
```

## `^foo$` — surrounded by anchors

```json
"pattern": [
  [[
    { "atom": {"type": "anchor", "kind": "start_of_line"}, "quantifier": [], "type": "piece" },
    { "atom": "f", "quantifier": [], "type": "piece" },
    { "atom": "o", "quantifier": [], "type": "piece" },
    { "atom": "o", "quantifier": [], "type": "piece" },
    { "atom": {"type": "anchor", "kind": "end_of_line"}, "quantifier": [], "type": "piece" }
  ]],
  []
]
```

Five pieces — anchors are pieces too.

## `\b` word boundary

```json
"pattern": [
  [[
    { "atom": {"type": "anchor", "kind": "word_boundary"}, "quantifier": [], "type": "piece" }
  ]],
  []
]
```

## `\B` non-word boundary

```json
"atom": {"type": "anchor", "kind": "non_word_boundary"}
```

## `\bword\b` — pattern surrounded by word boundaries

```json
"pattern": [
  [[
    { "atom": {"type": "anchor", "kind": "word_boundary"}, "quantifier": [], "type": "piece" },
    { "atom": "w", "quantifier": [], "type": "piece" },
    { "atom": "o", "quantifier": [], "type": "piece" },
    { "atom": "r", "quantifier": [], "type": "piece" },
    { "atom": "d", "quantifier": [], "type": "piece" },
    { "atom": {"type": "anchor", "kind": "word_boundary"}, "quantifier": [], "type": "piece" }
  ]],
  []
]
```

Six pieces.

## All 9 anchor kinds

| Source | `atom` |
|---|---|
| `^` | `{"type":"anchor","kind":"start_of_line"}` |
| `$` | `{"type":"anchor","kind":"end_of_line"}` |
| `\A` | `{"type":"anchor","kind":"start_of_input"}` |
| `\Z` | `{"type":"anchor","kind":"end_of_input_or_before_last_newline"}` |
| `\z` | `{"type":"anchor","kind":"end_of_input"}` |
| `\b` | `{"type":"anchor","kind":"word_boundary"}` |
| `\B` | `{"type":"anchor","kind":"non_word_boundary"}` |
| `\G` | `{"type":"anchor","kind":"match_start"}` |
| `\K` | `{"type":"anchor","kind":"keep_out"}` |

The full grammar:

```ebnf
anchor = "^"   -> {type: "anchor", kind: "start_of_line"}
       | "$"   -> {type: "anchor", kind: "end_of_line"}
       | "\\A" -> {type: "anchor", kind: "start_of_input"}
       | "\\Z" -> {type: "anchor", kind: "end_of_input_or_before_last_newline"}
       | "\\z" -> {type: "anchor", kind: "end_of_input"}
       | "\\b" -> {type: "anchor", kind: "word_boundary"}
       | "\\B" -> {type: "anchor", kind: "non_word_boundary"}
       | "\\G" -> {type: "anchor", kind: "match_start"}
       | "\\K" -> {type: "anchor", kind: "keep_out"}
```

## POSIX word-boundary aliases — `[[:<:]]` and `[[:>:]]`

Despite the syntax resembling a character class, these are atomic anchors handled by the `posix_word_boundary_alias` rule. **At this release that rule is NOT yet annotated** — it still emits a single Terminal of the entire 7-char sequence:

```json
"atom": "[[:<:]]"
```

Or:

```json
"atom": "[[:>:]]"
```

These are NOT character classes — consumers should NOT recursively descend looking for class items. They're anchors. A future slice will annotate `posix_word_boundary_alias` with a typed `{type:"anchor", kind:"posix_word_start"}` / `kind:"posix_word_end"` shape, joining them into the `anchor` typed family.

## Consumer extraction pattern

```rust
fn classify_anchor(atom: &Value) -> Option<AnchorKind> {
    // Typed anchor (slice 7 onward)
    if let Some(obj) = atom.as_object() {
        if obj.get("type").and_then(|v| v.as_str()) == Some("anchor") {
            return obj.get("kind").and_then(|v| v.as_str()).map(|kind| match kind {
                "start_of_line" => AnchorKind::StartOfLine,
                "end_of_line" => AnchorKind::EndOfLine,
                "start_of_input" => AnchorKind::StartOfInput,
                "end_of_input_or_before_last_newline" => AnchorKind::EndOfInputOrBeforeLastNewline,
                "end_of_input" => AnchorKind::EndOfInput,
                "word_boundary" => AnchorKind::WordBoundary,
                "non_word_boundary" => AnchorKind::NonWordBoundary,
                "match_start" => AnchorKind::MatchStart,
                "keep_out" => AnchorKind::KeepOut,
                _ => return None,
            });
        }
    }
    // Legacy: posix_word_boundary_alias still emits raw string until its slice lands.
    match atom.as_str()? {
        "[[:<:]]" => Some(AnchorKind::PosixWordStart),
        "[[:>:]]" => Some(AnchorKind::PosixWordEnd),
        _ => None,
    }
}
```

The discriminator is `obj.type == "anchor"` plus `obj.kind` for the variant. The POSIX aliases will join the typed family in their own slice.

## Migration from pre-1.1.35 (slice 7)

Before slice 7, the `anchor` rule emitted `Terminal(<text>)` and consumers dispatched on the raw escape text:

```rust
match atom.as_str()? {
    "^" => StartOfLine,
    "\\A" => StartOfInput,
    "\\b" => WordBoundary,
    // ...
}
```

Post-slice-7, dispatch is on `obj.kind`:

```rust
match atom.get("kind").and_then(|v| v.as_str())? {
    "start_of_line" => StartOfLine,
    "start_of_input" => StartOfInput,
    "word_boundary" => WordBoundary,
    // ...
}
```

The kind names are stable identifiers — they do not depend on the source escape text and won't change if PCRE2 syntax evolves.
