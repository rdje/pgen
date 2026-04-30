# Examples: Anchors and Boundaries

Concrete probe outputs for PCRE2 anchors and word-boundary constructs. The `anchor` rule emits a `Terminal` of the matched anchor text; consumers identify by string match.

## `^` start anchor

```json
"pattern": [
  [[
    { "atom": "^", "quantifier": [], "type": "piece" }
  ]],
  []
]
```

The `^` anchor produces an atom of just `"^"`.

## `$` end anchor

```json
"pattern": [
  [[
    { "atom": "$", "quantifier": [], "type": "piece" }
  ]],
  []
]
```

## `^foo$` — surrounded by anchors

```json
"pattern": [
  [[
    { "atom": "^", "quantifier": [], "type": "piece" },
    { "atom": "f", "quantifier": [], "type": "piece" },
    { "atom": "o", "quantifier": [], "type": "piece" },
    { "atom": "o", "quantifier": [], "type": "piece" },
    { "atom": "$", "quantifier": [], "type": "piece" }
  ]],
  []
]
```

Five pieces — anchors are pieces too.

## `\b` word boundary

```json
"pattern": [
  [[
    { "atom": "\\b", "quantifier": [], "type": "piece" }
  ]],
  []
]
```

The atom is `"\\b"` — a 2-character string (backslash + b).

## `\B` non-word boundary

```json
"atom": "\\B"
```

## `\bword\b` — pattern surrounded by word boundaries

```json
"pattern": [
  [[
    { "atom": "\\b", "quantifier": [], "type": "piece" },
    { "atom": "w",   "quantifier": [], "type": "piece" },
    { "atom": "o",   "quantifier": [], "type": "piece" },
    { "atom": "r",   "quantifier": [], "type": "piece" },
    { "atom": "d",   "quantifier": [], "type": "piece" },
    { "atom": "\\b", "quantifier": [], "type": "piece" }
  ]],
  []
]
```

Six pieces.

## `\A` absolute start, `\Z` near end, `\z` absolute end

| Input | Atom |
|---|---|
| `\A` | `"\\A"` |
| `\Z` | `"\\Z"` |
| `\z` | `"\\z"` |

## `\G` match-start anchor

```json
"atom": "\\G"
```

## `\K` keep-out (PCRE2-specific reset)

```json
"atom": "\\K"
```

The full set of `anchor` Or alternatives:

```ebnf
anchor = "^" | "$" | "\\A" | "\\Z" | "\\z" | "\\b" | "\\B" | "\\G" | "\\K"
```

Each emits a `Terminal` of the matched text.

## POSIX word-boundary aliases — `[[:<:]]` and `[[:>:]]`

Despite the syntax resembling a character class, these are atomic anchors handled by the `posix_word_boundary_alias` rule. They produce a single Terminal of the entire 7-char sequence:

```json
"atom": "[[:<:]]"
```

Or:

```json
"atom": "[[:>:]]"
```

These are NOT character classes — consumers should NOT recursively descend looking for class items. They're anchors.

## Consumer extraction pattern

For anchor identification:

```rust
fn classify_anchor(atom: &Value) -> Option<AnchorKind> {
    match atom.as_str()? {
        "^" => Some(AnchorKind::StartOfLine),
        "$" => Some(AnchorKind::EndOfLine),
        "\\A" => Some(AnchorKind::StartOfInput),
        "\\Z" => Some(AnchorKind::EndOfInputOrBeforeLastNewline),
        "\\z" => Some(AnchorKind::EndOfInput),
        "\\b" => Some(AnchorKind::WordBoundary),
        "\\B" => Some(AnchorKind::NonWordBoundary),
        "\\G" => Some(AnchorKind::MatchStart),
        "\\K" => Some(AnchorKind::KeepOut),
        "[[:<:]]" => Some(AnchorKind::PosixWordStart),
        "[[:>:]]" => Some(AnchorKind::PosixWordEnd),
        _ => None,
    }
}
```

The discriminator is exact-string match on the atom value.
