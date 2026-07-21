# Examples: Literals

Concrete probe outputs for plain literal regexes.

## Single literal — `a`

```json
{
  "content": {
    "Json": {
      "pattern": [
        [[
          { "atom": "a", "quantifier": [], "type": "piece" }
        ]],
        []
      ],
      "type": "regex"
    }
  },
  "rule_name": "regex",
  "span": { "start": 0, "end": 1 }
}
```

Single piece. Atom is the bare string `"a"`. No quantifier (empty array).

## Multi-char literal — `abc`

```json
{
  "content": {
    "Json": {
      "pattern": [
        [[
          { "atom": "a", "quantifier": [], "type": "piece" },
          { "atom": "b", "quantifier": [], "type": "piece" },
          { "atom": "c", "quantifier": [], "type": "piece" }
        ]],
        []
      ],
      "type": "regex"
    }
  },
  "rule_name": "regex",
  "span": { "start": 0, "end": 3 }
}
```

Three pieces, one per char. `concatenation`'s `[$1**]` flatten-spread produced the flat array.

## Mixed literals and special chars — `a.b`

The `.` is the dot atom, not a literal:

```json
{
  "content": {
    "Json": {
      "pattern": [
        [[
          { "atom": "a", "quantifier": [], "type": "piece" },
          { "atom": ".", "quantifier": [], "type": "piece" },
          { "atom": "b", "quantifier": [], "type": "piece" }
        ]],
        []
      ],
      "type": "regex"
    }
  },
  "rule_name": "regex",
  "span": { "start": 0, "end": 3 }
}
```

Three pieces. The `.` piece's atom is `"."` — the `dot` rule's `Terminal(".")` output. A consumer distinguishes "dot" from a literal `.` by structural signature: both are bare strings, but for `dot`, the string is exactly `"."`. (Compare to literal escaping: `\.` would be an `escape` atom shape, not `dot`.)

## Empty regex — `` (zero-length input)

```json
{
  "content": {
    "Json": {
      "pattern": [],
      "type": "regex"
    }
  },
  "rule_name": "regex",
  "span": { "start": 0, "end": 0 }
}
```

Empty `pattern?` slot. The `Quantified-?` carrier with no match becomes `[]` after the Json conversion.

## A literal with escaped char — `a\.`

```json
{
  "content": {
    "Json": {
      "pattern": [
        [[
          { "atom": "a", "quantifier": [], "type": "piece" },
          {
            "atom": { "char": ".", "kind": "shorthand", "type": "escape" },
            "quantifier": [],
            "type": "piece"
          }
        ]],
        []
      ],
      "type": "regex"
    }
  },
  "rule_name": "regex",
  "span": { "start": 0, "end": 3 }
}
```

The `\.` produces a typed escape atom — the flat `{type:"escape", kind:"shorthand", char:"."}` object (the escape subtree is fully typed; see [Escape Subtree](rules-escape.md) and [Examples: Escapes](examples-escapes.md)). The escaped metacharacter is just `char: "."` — semantic interpretation is downstream.

## Single non-ASCII literal — `é`

```json
{
  "content": {
    "Json": {
      "pattern": [
        [[
          { "atom": "é", "quantifier": [], "type": "piece" }
        ]],
        []
      ],
      "type": "regex"
    }
  },
  "rule_name": "regex",
  "span": { "start": 0, "end": 2 }
}
```

The `é` (U+00E9, encoded as 2 UTF-8 bytes) is matched by `literal_char`'s `[^\x00-\x7F]` alternative. Span is byte-positions: `0..2`. The atom string IS the 2-byte char.

## Consumer extraction pattern

For the literal-only family:

```rust
fn extract_literal_pieces(regex: &Value) -> Vec<&str> {
    let pattern = regex.get("pattern").and_then(|p| p.as_array()).unwrap_or(&[]);
    let alt0 = pattern.get(0).and_then(|a| a.as_array()).unwrap_or(&[]);
    let concat = alt0.get(0).and_then(|c| c.as_array()).unwrap_or(&[]);
    concat.iter()
        .filter_map(|piece| {
            let atom = piece.get("atom")?;
            atom.as_str()
        })
        .collect()
}
```

For input `abc`: returns `vec!["a", "b", "c"]`.

For inputs with non-string atoms (escape, char_class, etc.), this function would skip — the consumer's actual lowering needs per-atom-kind handling per [Atom Subtree](rules-atom.md).
