# piece and the Quoted-Run Family

The `piece` rule is the workhorse of regex — every quantified or unquantified atom in a pattern is a piece. This chapter covers `piece` itself plus its siblings introduced for the PGEN-RGX-0074 fix.

## `piece`

```ebnf
piece = piece_quoted_run_quantified -> $1
      | anchor !quantifier -> {type: "piece", atom: $1, quantifier: []}
      | zero_width !quantifier -> {type: "piece", atom: $1, quantifier: []}
      | unterminated_quoted_literal -> {type: "piece", atom: $1, quantifier: []}
      | directive_verb_nonquant !quantifier -> {type: "piece", atom: $1, quantifier: []}
      | atom quantifier? -> {type: "piece", atom: $1, quantifier: $2}
      | atom zero_width+ quantifier
-> {type: "piece", atom: $1, quantifier: $3}
```

Seven branches (the anchor branch landed in release `1.1.82`, `REGEX-PCRE2-FIDELITY.3.13`; the two zero-width branches in release `1.1.86`, `REGEX-PCRE2-FIDELITY.3.19`; the non-quantifiable-directive branch in release `1.1.87`, `REGEX-PCRE2-FIDELITY.3.20`; the unterminated-quote branch in release `1.1.89`, `REGEX-PCRE2-FIDELITY.3.23`). Because `|` is a **longest-match tournament** (ties → the earlier branch), branch ORDER matters only for ties:

1. **Quoted-run branch**: `piece_quoted_run_quantified -> $1`. Tried FIRST. Matches `\Q...\E quantifier` — multi-char quoted runs followed by a quantifier — and emits a Sequence of pieces (one per char, with the trailing piece carrying the quantifier).
2. **Anchor branch**: `anchor !quantifier -> {type: "piece", atom: $1, quantifier: []}`. Anchors (`^ $ \A \Z \z \b \B \G \K`) are **non-quantifiable** in PCRE2 (err 109) — and, crucially, OPAQUE: a quantifier after an anchor errors *even when a repeatable atom precedes* (`a^*` REJECTS). So an anchor forms a piece with NO quantifier slot; the `!quantifier` lookahead makes `^*`/`$*`/`\b*`/`\A{2}`-style patterns REJECT while non-quantifier braces (`${`, `\A{a}`, `\A{}`) still parse as anchor + literal-brace pieces. `anchor` is no longer an `atom` alternative. See ledger `REGEX-0088`.
3. **Zero-width STANDALONE branch** (`.3.19`/`.3.23`): `zero_width !quantifier -> {type: "piece", atom: $1, quantifier: []}`. A stray `\E` **or an empty `\Q\E`** (`zero_width` — see below) is PCRE2 zero-width but, *unlike* an anchor, TRANSPARENT. On its own it forms a non-quantifiable piece; a bare `\E*` / `\Q\E*` REJECTS (err 109) because the standalone branch's `!quantifier` fails and no other branch matches. Byte-identical to the pre-`1.1.86`/pre-`1.1.89` shapes.
4. **Unterminated-quote branch** (`.3.23`): `unterminated_quoted_literal -> {type: "piece", atom: $1, quantifier: []}`. A `\Q…` with no closing `\E` quotes to END-OF-PATTERN as literal (see [`unterminated_quoted_literal`](rules-atom.md#unterminated_quoted_literal)), so `\Q)` / `\Q(` / `\Q^` / `\Q(?:` are ONE literal run, not live regex. It is a NON-atom greedy-to-end piece — deliberately, so the ABSORPTION branch can never pick a bare `\Q` as its atom (which would let the lookahead-blind generator emit `\Q\E<quantifier>` — a duality break). See ledger `REGEX-0099`.
5. **Non-quantifiable-directive branch** (`.3.20`): `directive_verb_nonquant !quantifier -> {type: "piece", atom: $1, quantifier: []}`. In PCRE2 only `(*ACCEPT)` may take a quantifier; every other `(*...)` directive (the 6 non-ACCEPT verbs, MARK, and the `(*:x)` shorthand) quantified is err 109. Those directives are matched by `directive_verb_nonquant` (see the [directives chapter](rules-misc.md#directive_verb)) and form a non-quantifiable piece. Because `directive_verb` — the quantifiable directive rule kept in `atom` — now matches only `(*ACCEPT...)` and the relaxed unknown-name catch-all, a quantified non-ACCEPT directive fails this `!quantifier` lookahead AND finds no matching atom, so the whole parse REJECTS (`(*PRUNE)+`, `(*:x)+` all err 109). See ledger `REGEX-0096`. **`.4.8`:** the bare **start options** (`(*UTF)`/`(*CRLF)`/`(*LIMIT_HEAP=…)`-class) are NO LONGER in this branch — they were peeled into a dedicated `start_option_piece` (`= start_option_verb !quantifier -> {type:"piece", atom:$1, quantifier:[]}`; same non-quantifiable piece shape, so `(*UTF)+`/`(*LIMIT_HEAP=5)+` still err 109) referenced ONLY from [`entry_concatenation`](rules-top-level.md#the-entry-chain--entry_alternation--entry_alternative--entry_concatenation), which is what enforces the start-option position rule structurally.
6. **Standard atom branch**: `atom quantifier? -> {type: "piece", atom: $1, quantifier: $2}`. A single atom with an optional quantifier — the common case. `(*ACCEPT)+` accepts here (the ACCEPT directive is a quantifiable `atom`).
7. **Zero-width ABSORPTION branch** (`.3.19`/`.3.23`): `atom zero_width+ quantifier -> {type: "piece", atom: $1, quantifier: $3}`. Placed LAST. A quantifier reaches THROUGH ≥1 transparent `zero_width` (a stray `\E` or an empty `\Q\E`) to bind the preceding atom (`a\E*` = `a*`, `a\Q\E*` = `a*`), with the zero-widths **elided** (PCRE2's delete model). The `+` confines this branch to exactly the `<atom> <zero_width>+ <quantifier>` case, so a plain `x*` is untouched. Since `1.1.89` (`.3.23`), `\Q\E*` has NO leading atom (empty `\Q\E` is itself a `zero_width`, and `\Q` is no longer an escape atom), so it REJECTS via branch 3 — the pre-`1.1.89` "deferred accepts-invalid" note is resolved.

For `\Qa\E{3}` (single-char quoted run), the quoted-run branch fails (it requires the inner-piece-list to be non-empty before the trailing char), so the standard-atom branch matches via the `quoted_literal` atom alternative.

### `zero_width` — the transparent stray `\E` and empty `\Q\E`

```ebnf
zero_width = stray_end_quote_escape | empty_quoted_literal
stray_end_quote_escape = "\\E" -> {type: "escape", kind: "shorthand", char: "E"}
empty_quoted_literal = "\\Q" "\\E" -> {type: "atom", kind: "quoted_literal", body: []}
```

Two PCRE2 zero-width, transparent constructs:

- **stray `\E`** — an end-of-quote with no matching `\Q` — is dropped from `simple_escape_letter_strict` (a POSITIVE exclusion, generation-faithful) and re-homed here, so a bare quantifier can no longer bind directly to it. Bare `\E`, `a\E`, `\Ea` are byte-identical to pre-`1.1.86`.
- **empty `\Q\E`** (release `1.1.89`, `REGEX-PCRE2-FIDELITY.3.23`) — an empty quoted literal is PCRE2 zero-width, exactly like a stray `\E`. It joined `zero_width` so `\Q\E*` REJECTS (err 109) and `a\Q\E*` = `a*` (absorption). This is sound only because `Q` left `simple_escape` this release (otherwise the tournament would re-accept `\Q\E*` as `\Q`-escape + `\E` + `*`). Its emit preserves the exact `{type:"atom", kind:"quoted_literal", body:[]}` byte-shape a bare `\Q\E` had as a quantifiable atom, so a standalone `\Q\E` is unchanged.

See the escape chapter and the [transparent-binding examples](examples-anchors.md#stray-e-transparent-quantifier-binding-release-1186-regex-pcre2-fidelity319).

### Shape — the standard atom branch (the common case)

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
    { "atom": "*", "quantifier": {"type":"quantifier","min":2,"max":null,"greediness":[]}, "type": "piece" }
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
