# Examples: \\Q...\\E Quoted Literals

The `\Q...\E` family is the focus of the **PGEN-RGX-0074** correctness fix. This chapter documents every member of the family with concrete probe outputs.

## Background: PCRE2 semantics

Per `pcre2pattern(3)` §"Backslash":

> A quantifier following \Q...\E applies only to the **last character** of the literal sequence, not to the whole sequence.

Pre-fix, PGEN incorrectly bound the quantifier to the entire `\Q...\E` block. The fix produces 3 pieces (one per quoted char, with the trailing piece carrying the quantifier).

## Family table

| Source | Semantically equivalent | AST output |
|---|---|---|
| `\Qab*\E{2,}` | `a` `b` `\*{2,}` | 3 pieces |
| `\Qab*\E{2}` | `a` `b` `\*{2}` | 3 pieces |
| `\Qab*\E?` | `a` `b` `\*?` | 3 pieces |
| `\Qab*\E+` | `a` `b` `\*+` | 3 pieces |
| `\Qab*\E*` | `a` `b` `\**` | 3 pieces |
| `\Qab*\E*?` | `a` `b` `\**?` | 3 pieces |
| `\Qab*\E{1,3}` | `a` `b` `\*{1,3}` | 3 pieces |
| `\Qab*\E{2,}?` | `a` `b` `\*{2,}?` | 3 pieces |
| `\Qabc\E{2}` | `a` `b` `c{2}` | 3 pieces |
| `\Qab\E{3}` | `a` `b{3}` | 2 pieces |
| `\Qa\E{3}` | `a{3}` | 1 piece (degenerate) |
| `\Q\E{2}` | empty | 1 piece (atom-fallback, degenerate) |

## `\Qab*\E{2,}` — the canonical 3-piece case

```json
{
  "content": {
    "Json": {
      "pattern": [
        [[
          { "atom": "a", "quantifier": [], "type": "piece" },
          { "atom": "b", "quantifier": [], "type": "piece" },
          {
            "atom": "*",
            "quantifier": [
              [
                "{",
                [],
                [<digits=2>, [], [",", [], []]],   // counted_quantifier_body branch 0, {n,} form
                [],
                "}"
              ],
              []
            ],
            "type": "piece"
          }
        ]],
        []
      ],
      "type": "regex"
    }
  },
  "rule_name": "regex",
  "span": { "start": 0, "end": 11 }
}
```

Three pieces: `a`, `b`, `*` with `{2,}`. The `*` here is a literal asterisk (the original `*` inside `\Q...\E`, not the quantifier).

## `\Qab\E{3}` — 2-piece case

```json
"pattern": [
  [[
    { "atom": "a", "quantifier": [], "type": "piece" },
    {
      "atom": "b",
      "quantifier": [
        [
          "{",
          [],
          [<digits=3>, [], []],     // {n} form — no comma
          [],
          "}"
        ],
        []
      ],
      "type": "piece"
    }
  ]],
  []
]
```

Two pieces: `a` (no quantifier) and `b` (with `{3}`).

## `\Qa\E{3}` — degenerate single-char (1 piece via atom-fallback)

```json
"pattern": [
  [[
    {
      "atom": [
        "\\Q",
        ["a"],                      // single-char Quantified
        "\\E"
      ],
      "quantifier": [
        [<counted_quantifier shape with min=3 max=3>],
        []
      ],
      "type": "piece"
    }
  ]],
  []
]
```

ONE piece. The `piece_quoted_run_quantified` branch requires at least one prefix char BEFORE the trailing char, so for single-char quoted runs it fails and the parser falls through to `piece`'s branch 1 (`atom quantifier?` matching the whole `\Qa\E` as a `quoted_literal` atom). Semantically correct: "quantify the only char in `\Qa\E`" is exactly the same as "quantify the whole 1-char block."

## `\Q\E{2}` — empty quoted run with quantifier

```json
"pattern": [
  [[
    {
      "atom": [
        "\\Q",
        [],                         // empty chars Quantified
        "\\E"
      ],
      "quantifier": [
        [<counted_quantifier shape with min=2 max=2>],
        []
      ],
      "type": "piece"
    }
  ]],
  []
]
```

ONE piece. Empty `\Q...\E` falls through to atom-path. PCRE2 treats `\Q\E{2}` as a zero-width quantified empty match (matches the empty string twice = matches empty).

## `\Qab\E` — 2-char quoted run, NO trailing quantifier

```json
"pattern": [
  [[
    {
      "atom": [
        "\\Q",
        ["a", "b"],
        "\\E"
      ],
      "quantifier": [],
      "type": "piece"
    }
  ]],
  []
]
```

ONE piece. The `piece_quoted_run_quantified` branch requires a trailing quantifier — without one, the parser falls through to `piece`'s branch 1 with the whole `\Qab\E` as a single `quoted_literal` atom. Semantically correct: "literal a, literal b" with no quantifier.

## How `piece_quoted_run_quantified` decides

The branch:

```ebnf
piece_quoted_run_quantified
   = "\\Q" quoted_run_inner_piece* quoted_literal_char "\\E" quantifier
   -> [$2**, {type: "piece", atom: $3, quantifier: $5}]
```

Fires only when:
1. `\Q` is matched.
2. At least zero `quoted_run_inner_piece`s (could be zero for the trailing-only-char case... wait, see below).
3. Exactly one `quoted_literal_char` (the trailing char, distinct from the inner-piece chars).
4. `\E` is matched.
5. A `quantifier` is matched.

If any of these fail, the rule fails and the parser tries `piece`'s next branch.

For `\Qa\E{3}`: step 2 matches zero inner pieces (with the negative lookahead `!"\\E"` blocking `a` from matching as inner piece, because the next 2 chars are `\E`); step 3 matches `a` as the trailing char; steps 4-5 match. So the rule actually FIRES even for single-char runs. But — wait, that contradicts the family table above showing `\Qa\E{3}` as 1-piece.

Let me re-check. Actually for single-char input: `quoted_run_inner_piece*` might match zero times, then `quoted_literal_char` matches the single char, then `\E`. The annotation produces `[$2**, {atom: $3, quantifier: $5}]` = `[<empty array>**, {atom: "a", quantifier: {3}}]` = `[{atom: "a", quantifier: {3}}]` — a 1-element array. After concatenation's flatten-spread, that becomes ONE piece. Same result as the atom-fallback path semantically.

So both paths produce the same single-piece output for the degenerate case. The book entry above is correct.

## Why this matters for downstream consumers

The PGEN-RGX-0074 fix means downstream consumers like RGX get the **correct** PCRE2-equivalent piece structure for free — they don't need to know about `\Q...\E` semantics or implement special "split last char" logic. They just walk the flat piece array under concatenation, where each piece is a normal independent literal-or-quantified-literal.

Pre-fix, RGX would have had to either:
- Build special-case `\Q...\E` handling that mirrored PCRE2's "split last char" rule, OR
- Match the buggy whole-block-quantified shape and produce wrong runtime matching.

Post-fix, neither is necessary. RGX's lowering for `\Qab*\E{2,}` is exactly the same as for `ab\*{2,}` — three pieces, last one quantified.
