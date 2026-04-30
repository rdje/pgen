# piece and the Quoted-Run Family

The `piece` rule is the workhorse of regex — every quantified or unquantified atom in a pattern is a piece. This chapter covers `piece` itself plus its siblings introduced for the PGEN-RGX-0074 fix.

## `piece`

```ebnf
piece = piece_quoted_run_quantified -> $1
      | atom quantifier?
-> {type: "piece", atom: $1, quantifier: $2}
```

Two branches:

1. **Branch 0**: `piece_quoted_run_quantified -> $1`. Tried FIRST. Matches `\Q...\E quantifier` — multi-char quoted runs followed by a quantifier — and emits a Sequence of pieces (one per char, with the trailing piece carrying the quantifier).
2. **Branch 1**: `atom quantifier? -> {type: "piece", atom: $1, quantifier: $2}`. The standard piece shape: a single atom with an optional quantifier.

PEG-ordered alternation: branch 0 is attempted first; if it doesn't match, branch 1 takes over. For `\Qa\E{3}` (single-char quoted run), branch 0 fails (it requires the inner-piece-list to be non-empty before the trailing char), so branch 1 matches via the `quoted_literal` atom alternative.

### Shape — branch 1 (the common case)

```json
{
  "type": "piece",
  "atom": <atom-content>,
  "quantifier": <quantifier-content>
}
```

- `atom`: the atom rule's output. For currently-unannotated atom alternatives (most of them), this is the raw envelope shape (Terminal / Sequence / etc.). See [Atom Subtree](rules-atom.md).
- `quantifier`: the `quantifier?` slot — an array of 0 or 1 elements. See [Quantifier Subtree](rules-quantifier.md). When no quantifier was present, this is `[]`.

### Shape — branch 0 (quoted-run-with-quantifier)

```json
[
  { "type": "piece", "atom": <prefix-char>,  "quantifier": [] },
  { "type": "piece", "atom": <prefix-char>,  "quantifier": [] },
  ...
  { "type": "piece", "atom": <last-char>,    "quantifier": [<the-quantifier>] }
]
```

Returned as a Sequence (array) of N piece objects. Lifted by `concatenation`'s `[$1**]` flatten-spread into a flat list under concatenation, so consumers walking concatenation see one piece per char with the trailing char carrying the quantifier — exactly as PCRE2 semantics dictate.

### Examples

#### Plain literal `a` (branch 1, no quantifier)

```json
{ "atom": "a", "quantifier": [], "type": "piece" }
```

#### Quantified `a*` (branch 1, with quantifier)

```json
{
  "atom": "a",
  "quantifier": [
    "*",
    []
  ],
  "type": "piece"
}
```

The `quantifier` array's element 0 is the `quant_base` (raw `"*"` terminal — the rule isn't annotated yet). Element 1 is the `quant_suffix?` slot, empty here because there's no `?` or `+` modifier.

#### Lazy `a*?` (branch 1, with lazy quantifier)

```json
{
  "atom": "a",
  "quantifier": [
    "*",
    "lazy"
  ],
  "type": "piece"
}
```

Element 1 is now the typed `"lazy"` string from `quant_suffix`'s annotation.

#### `\Qab*\E{2,}` (branch 0, multi-char quoted run with quantifier)

The `piece_quoted_run_quantified` output produces 3 pieces (the quoted-run's chars, with the last carrying the quantifier). After concatenation's flatten-spread, those pieces appear inline:

```json
"pattern": [
  [[
    { "atom": "a", "quantifier": [],                          "type": "piece" },
    { "atom": "b", "quantifier": [],                          "type": "piece" },
    { "atom": "*", "quantifier": [{"min":2,"max":null}, []],  "type": "piece" }
  ]],
  []
]
```

This is the **PGEN-RGX-0074** fix. Pre-fix, this input emitted ONE piece with the whole `\Qab*\E` block as a single atom; post-fix, it correctly emits 3 pieces with the quantifier bound only to the last char.

## `piece_quoted_run_quantified`

```ebnf
piece_quoted_run_quantified
   = "\\Q" quoted_run_inner_piece* quoted_literal_char "\\E" quantifier
-> [$2**, {type: "piece", atom: $3, quantifier: $5}]
```

Matches `\Q` + (zero or more prefix chars) + (one trailing char) + `\E` + a required quantifier. The annotation flat-emits an array of pieces:

- `$2**` — the prefix chars wrapped as pieces by `quoted_run_inner_piece`'s annotation, flatten-spread.
- `{type: "piece", atom: $3, quantifier: $5}` — the trailing char paired with the quantifier.

### Shape

```json
[
  { "type": "piece", "atom": <prefix-char-1>, "quantifier": [] },
  { "type": "piece", "atom": <prefix-char-2>, "quantifier": [] },
  ...
  { "type": "piece", "atom": <last-char>,     "quantifier": <the-quantifier> }
]
```

Returned as `Json(Array(...))`.

### When this rule fires

This rule fires only when `\Q` + at least one char + at least one char (or just one char) + `\E` + a quantifier are all present. Specifically:

- `\Qab*\E{2,}` — fires (3 pieces emitted: a, b, *{2,}).
- `\Qabc\E?` — fires (3 pieces: a, b, c?).
- `\Qa\E{3}` — does NOT fire; the rule requires at least one inner piece BEFORE the trailing char. Falls through to `piece`'s branch 1, which matches via `atom = quoted_literal` consuming `\Qa\E`, then `quantifier? = {3}`.
- `\Q\E{2}` — does NOT fire; empty `\Q\E`. Falls through to branch 1 (`atom = quoted_literal` matches `\Q\E`, quantifier matches `{2}`).
- `\Qab\E` (no trailing quantifier) — does NOT fire; the rule REQUIRES a trailing quantifier. Falls through to branch 1.

The fall-through cases produce a single piece with the whole `\Q...\E` as atom — which is correct for those cases (no quantifier-attachment ambiguity exists).

## `quoted_run_inner_piece`

```ebnf
quoted_run_inner_piece = quoted_literal_char !"\\E"
-> {type: "piece", atom: $1, quantifier: []}
```

The per-char wrapper used inside `piece_quoted_run_quantified`. Each call matches a single `quoted_literal_char` and emits a piece object. The negative lookahead `!"\\E"` prevents the `*` quantifier in the parent rule from greedily consuming the LAST char of the run — that char must be left for the parent's explicit `quoted_literal_char` slot.

### Shape

```json
{ "type": "piece", "atom": <char>, "quantifier": [] }
```

### Why empty quantifier

Prefix chars in a quoted run never carry a quantifier by definition (per PCRE2 — the quantifier always binds to the last char). The literal `[]` (empty array) matches the byte-shape that an unmatched `quantifier?` Quantified produces in any other piece, so consumers see uniform piece shape across all sources.

## How the pieces flow upward

```text
quoted_run_inner_piece*  →  Quantified(prefix-piece-objects)
                                  ↓
piece_quoted_run_quantified  →  Sequence([prefix-pieces..., {last-piece-with-quantifier}])
                                  ↓
piece (branch 0, -> $1)  →  Sequence([prefix-pieces..., {last-piece-with-quantifier}])
                                  ↓
concatenation = piece+ -> [$1**]  →  Sequence([all-pieces-flat])
                                  ↓
parent context (alternative, alternation, regex)  →  pieces visible at pattern[0][0]
```

The `**` flatten-spread on `concatenation` is what unwraps the Sequence-shape carried by branch-0 pieces and merges them with branch-1 pieces (which carry Json content). Without `**`, consumers would see nested arrays for quoted-run-quantified inputs.

## What `atom` looks like inside a piece

The `atom` field's shape depends on which `atom` alternative matched. Currently `atom` is un-annotated, so the atom alternative's raw envelope appears directly. Common shapes:

- For a `literal`: a string like `"a"`.
- For a `dot`: `"."`.
- For a `quoted_literal` (e.g. inside a single-char `\Qx\E`): `["\\Q", [<char>], "\\E"]`.
- For an `escape`: a 2-element `["\\", <escape_unit>]` shape — see [Escape Subtree](rules-escape.md).
- For a `char_class`: nested structure — see [Character Class Subtree](rules-char-class.md).
- For a `group`: nested `(...)` content — see [Group Family](rules-groups.md).

The [Atom Subtree](rules-atom.md) chapter documents each alternative.
