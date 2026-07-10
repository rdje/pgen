# Examples: Anchors and Boundaries

Concrete probe outputs for PCRE2 anchors and word-boundary constructs. As of slice 7 (post-1.1.35) the `anchor` rule emits a typed `{type: "anchor", kind: "<name>"}` object — consumers read `.kind` directly instead of dispatching by string match on the raw escape text.

## Quantified anchors reject (release 1.1.82, REGEX-PCRE2-FIDELITY.3.13)

PCRE2 rejects a quantifier applied directly to ANY of the 9 anchor forms (err 109, "quantifier
does not follow a repeatable item"), and since release `1.1.82` PGEN does too — at the grammar
layer (anchors are their own non-quantifiable `piece` branch; the check no longer lives in the
host validator):

- **REJECT** (`E_PARSE_FAILURE`): `^*` `$*` `$?` `^+` `${2}` `\A*` `\A{2}` `\A{2,3}` `\A{,2}`
  `\b*` `\B?` `\G+` `\z*` `\Z*` `\K*` — the escape-anchor forms were wrongly ACCEPTED before
  `1.1.82` (bug ledger `REGEX-0088`).
- **ACCEPT** (shapes unchanged): bare anchors; anchors in concatenation (`a^b$c`); grouped
  anchors — `(?:^)*` quantifies the GROUP, PCRE2-parity; class members `[$]*` / `[\b]` (a `$`
  in a class is a literal; `[\b]` is backspace); the POSIX word-boundary aliases
  `[[:<:]]*` / `[[:>:]]+` (PCRE2 compiles them to quantifiable sub-groups); and non-quantifier
  braces `${` `\A{a}` `\A{2` `\A{}` (not a valid quantifier shape ⇒ literal braces, PCRE2-parity).

## Stray `\E` transparent quantifier binding (release 1.1.86, REGEX-PCRE2-FIDELITY.3.19)

A stray `\E` (an end-of-quote with no matching `\Q`) is PCRE2 zero-width — but, *unlike* an anchor,
it is **TRANSPARENT**: a quantifier binds THROUGH it to the preceding repeatable atom instead of
erroring on it. Since release `1.1.86` PGEN encodes this at the grammar layer (stray `\E` is a
non-quantifiable `zero_width` piece; the `piece` ABSORPTION branch reaches through it). The
discriminating oracle fact (`pcre2test` 10.47): `a^*` REJECTS (an anchor is opaque) while `a\E*`
ACCEPTS (a stray `\E` is transparent).

- **REJECT** (`E_PARSE_FAILURE`, err 109 — no repeatable predecessor reachable through elision):
  `\E*` `\E+` `\E?` `\E{2}` `\E{2,}` `\E{2,3}` `\E{,2}` (bare stray `\E` + quantifier); `\E\E*`
  (two stray `\E`, still nothing repeatable); `^\E*` `\A\E*` `a^\E*` (an anchor before the `\E`
  is non-repeatable and blocks the bind); `(\E*)` `|\E*` `a|\E*` `(a|\E*)` (a group-open or
  alternation edge resets the predecessor). All were wrongly ACCEPTED before `1.1.86` (bug ledger
  `REGEX-0095`).
- **ACCEPT** (a quantifier binds through the transparent `\E` to the preceding atom): `a\E*` (=
  `a*`), `ab\E*` (binds `b`), `a\E\E*`, `\Qa\E\E*`, `()\E*`, `(a)\E*`, `(a|b)\E*`; `\Ea*`
  `\E\Ea*` `^\Ea*` `\E|a*` (a leading `\E`, then a quantified real atom).
- **ACCEPT** (unchanged): bare stray `\E`, `a\E`, `\Ea`; non-quantifier braces `\E{a}` `\E{`;
  grouped `(\E)*` (quantifies the group).

For the eight `<atom>\E<quant>` accept cells the AST now binds the quantifier to the repeatable
**atom** with the stray `\E`(s) elided (`a\E*` → `[piece(a, *)]`, not `[piece(a), piece(\E, *)]`);
all other accepted patterns are byte-identical. **Deferred:** empty `\Q\E`-quantified (`\Q\E*`,
`\Q\E{2}`, …) is still accepts-invalid and byte-UNCHANGED — it entangles with the
`\Q`-as-`simple_escape` / unterminated-`\Q...\E` model (`REGEX-PCRE2-FIDELITY.3.23`).

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
       | keep_out

# \K is extracted into its own rule so it can be gated (see below).
@predicate: { name: not_in_scope_kind, args: [lookaround], phase: pre }
keep_out = "\\K" -> {type: "anchor", kind: "keep_out"}
```

The `\K` AST is unchanged — `{"type":"anchor","kind":"keep_out"}` — so the consumer table above and the dispatch below are unaffected.

### `\K` is rejected inside a lookaround (release `1.1.101`, `REGEX-PCRE2-FIDELITY.4.10`)

PCRE2 forbids `\K` inside any lookaround body — `pcre2test` 10.47 rejects `(?=a\Kb)`, `(?!…)`, `(?<=…)`, `(?<!…)`, `(?*…)`, `(?<*…)`, and every alpha form (`(*pla:a\Kb)`…) with error 199 *"\K is not allowed in lookarounds"*, **including** when the `\K` sits inside a group nested in the lookaround (`(?=a(b\Kc))`, `((?=x\Ky))`). Every such pattern REJECTs with `E_PARSE_FAILURE`.

`\K` is still accepted **everywhere else** — bare (`a\Kb`, `\Kword`), inside an atomic group (`(?>a\Kb)`), a capturing group (`(a\Kb)`), a non-capturing group (`(?:a\Kb)`), and **after** a lookaround has closed (`(?=ab)\K`).

**This release is behavior-neutral for downstream consumers.** The released parser already rejected `\K`-in-lookaround before `1.1.101` — but via an out-of-band host validator (`find_invalid_keep_out_escape_in_lookaround`), while the *grammar alone* accepted it (a single-source-of-truth hole). `1.1.101` migrates the reject **into `grammars/regex.ebnf`** and deletes the standalone validator check, so the EBNF is now the single source of truth. The accept/reject verdict of the released parser is unchanged; only the layer that owns the rule moved.

The gate is declarative, with no engine change: each lookaround opens a `lookaround` semantic scope at its opener and closes it after its body, and `keep_out` carries `@predicate not_in_scope_kind(lookaround)` — the parser-agnostic scope-ancestry gate (`SCOPE-CONTEXT-PREDICATE.1`) that is true only when no currently-open scope (the innermost frame *or any ancestor*) is a lookaround. The whole-ancestor walk is what makes the nested `(?=a(b\Kc))` case reject, and the scope closing at the lookaround's end is what keeps the trailing `(?=ab)\K` accepted.

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
