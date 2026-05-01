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

Despite the syntax resembling a character class, these are atomic anchors handled by the `posix_word_boundary_alias` rule. As of slice 9 (post-1.1.37) the rule is annotated and emits the same typed `{type:"anchor", kind:<name>}` shape as the regular `anchor` rule:

```json
"atom": {"type": "anchor", "kind": "posix_word_start"}
```

Or:

```json
"atom": {"type": "anchor", "kind": "posix_word_end"}
```

Consumers walking the typed shape can dispatch uniformly on `obj.type == "anchor"` regardless of whether the source used `\b` (regular word boundary) or `[[:<:]]`/`[[:>:]]` (POSIX-style aliases) — the dispatch shape is identical, only the `kind` value differs.

These are NOT character classes — consumers should NOT recursively descend looking for class items. They're anchors. The typed kind names `posix_word_start` / `posix_word_end` distinguish them from PCRE2's `\b` (which is `kind:"word_boundary"` and is bidirectional, matching at either edge).

## Consumer extraction pattern

As of slice 9 (post-1.1.37) all 11 anchor variants — the 9 from the `anchor` rule plus the 2 POSIX-style aliases from `posix_word_boundary_alias` — emit the same typed `{type:"anchor", kind:<name>}` shape. Consumer dispatch is uniform:

```rust
fn classify_anchor(atom: &Value) -> Option<AnchorKind> {
    let obj = atom.as_object()?;
    if obj.get("type")?.as_str()? != "anchor" {
        return None;
    }
    match obj.get("kind")?.as_str()? {
        "start_of_line" => Some(AnchorKind::StartOfLine),
        "end_of_line" => Some(AnchorKind::EndOfLine),
        "start_of_input" => Some(AnchorKind::StartOfInput),
        "end_of_input_or_before_last_newline" => Some(AnchorKind::EndOfInputOrBeforeLastNewline),
        "end_of_input" => Some(AnchorKind::EndOfInput),
        "word_boundary" => Some(AnchorKind::WordBoundary),
        "non_word_boundary" => Some(AnchorKind::NonWordBoundary),
        "match_start" => Some(AnchorKind::MatchStart),
        "keep_out" => Some(AnchorKind::KeepOut),
        "posix_word_start" => Some(AnchorKind::PosixWordStart),
        "posix_word_end" => Some(AnchorKind::PosixWordEnd),
        _ => None,
    }
}
```

The discriminator is `obj.type == "anchor"` plus `obj.kind` for the variant. No string-match fallback paths needed — the anchor family is fully typed.

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
