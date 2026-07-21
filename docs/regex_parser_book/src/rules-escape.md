# Escape Subtree

PCRE2 escape sequences (`\d`, `\w`, `\xFF`, `\u{1F}`, `\o{777}`, `\cA`, `\p{Lu}`, etc.). **The whole subtree is annotated** (slices 14–17): every escape form emits a flat typed `{type: "escape", kind: ..., ...}` object — one field read per payload, no chain walking.

## `escape`

```ebnf
@type: "context_sensitive_escape_sequence"
escape = "\\" escape_unit
```

**Transparent** (`-> $2`): the matched `escape_unit` branch's typed object IS the output — the wrapper adds nothing. (The `@type`/`@description`/`@effect`/etc. are semantic annotations, not return annotations.)

## `escape_unit`

```ebnf
escape_unit = single_byte_escape
            | hex_escape
            | unicode_escape
            | octal_escape
            | control_escape
            | property_escape
            | simple_escape
```

7-way Or; the Or itself carries no annotation — each branch emits its own typed object.

### Branches and shapes (all live-verified)

| Branch | Form | Example | Emitted object |
|---|---|---|---|
| 0 (`single_byte_escape`) | `\C` | `\C` | `{"type":"escape","kind":"single_byte"}` |
| 1 (`hex_escape`) | `\xFF`, `\x{FFFF}` | `\xFF` | `{"type":"escape","kind":"hex","digits":"FF"}` |
| 2 (`unicode_escape`, `relaxed` only) | `\u{FFFF}` | `\u{1F}` | `{"type":"escape","kind":"unicode","digits":"1F"}` |
| 3 (`octal_escape`) | `\o{777}`, `\377` | `\o{101}` | `{"type":"escape","kind":"octal","digits":"101"}` |
| 4 (`control_escape`) | `\cA` | `\cA` | `{"type":"escape","kind":"control","char":"A"}` |
| 5 (`property_escape`) | `\p{Lu}`, `\P{Lu}`, `\pL`, `\PL` | `\pL` | `{"type":"escape","kind":"property","name":"L","negated":false}` |
| 6 (`simple_escape`) | `\<char>` (catch-all) | `\d`, `\.`, `\\` | `{"type":"escape","kind":"shorthand","char":"d"}` |

The branches are tried in order; `simple_escape` is the catch-all that matches any unrecognized `\<char>` escape. **Profile note (REGEX-PCRE2-FIDELITY.3.1):** branch 2 (`unicode_escape`, `\u{…}`) is `@profiles: ["relaxed"]` — active only under the `relaxed` profile; branch 6 (`simple_escape`)'s letter component carries the strict/relaxed split. In the default (`pcre2`) profile the six PCRE2-unsupported escape letters `\i \F \l \L \u \U` are rejected. See [Profiles — strict default vs `relaxed`](#profiles--strict-default-vs-relaxed). **Anchor note (REGEX-PCRE2-FIDELITY.3.13, release 1.1.82):** the 7 escape-anchor spellings `\A \Z \z \b \B \G \K` are NOT reachable through `simple_escape` at the pattern level in either profile — they are anchors, owned by the `anchor` rule (their own non-quantifiable `piece` branch), which is how `\b*`-style quantified escape-anchors reject PCRE2-faithfully (err 109).

## `single_byte_escape`

```ebnf
single_byte_escape = "C"
```

PCRE2's `\C` — match one code unit. `Terminal("C")`.

> **Pattern-body only (REGEX-PCRE2-FIDELITY.4.4, release 1.1.92).** `single_byte_escape` is reachable
> from `escape_unit` (the pattern body) but **not** from `class_escape_unit` — PCRE2 forbids `\C`
> inside a character class, so `[\C]` hard-REJECTS at the grammar layer (together with the
> escape-in-class guards on `class_simple_escape_*` and the `A`/`G`/`z` drop from the class-range
> escape letters; see the Character Class chapter → *Escape validity inside a class*). This migrated
> the escape-in-class rejects (`\A \B \C \G \K \N`-unbraced `\R \X \Z \z`) out of the out-of-band
> validator into the EBNF — behavior-neutral, `E_PARSE_FAILURE` unchanged.

## `simple_escape`

```ebnf
simple_escape  = !"o{" !"x{" !"p" !"P" !"k" simple_escape_tail
simple_escape_tail = simple_escape_letter | whitespace | special_char | unicode_char
simple_escape_letter = simple_escape_letter_strict | simple_escape_letter_relaxed
```

The catch-all single-char escape. Emits the typed shorthand object `{type: "escape", kind: "shorthand", char: <char>}` (positional ref `char: $6`) — the character that follows the backslash.

> **`\k` note (REGEX-PCRE2-FIDELITY.4.2, release 1.1.93):** lowercase `k` is excluded from the
> catch-all (the `!"k"` guard + its drop from `simple_escape_letter_strict`) — `\k` is ALWAYS a
> named-backreference introducer (owned by `backreference`'s `\k<…>`/`\k'…'`/`\k{…}` branches,
> see [Named backreferences](rules-misc.md)), never a bare shorthand. A malformed `\k`
> (`\k`/`\kabc` err 169, empty `\k''`/`\k<>`/`\k{}` err 162 in PCRE2) now hard-REJECTs at the
> grammar layer — the exact `\p`/`\P` precedent from `.4.1`.

For `\d`: the inner shape is `{type:"escape",kind:"shorthand",char:"d"}` — the standard PCRE2 metacharacter is just text from the parser's perspective; semantic interpretation (`\d` = digit-class) is downstream.

**Since release 1.1.82 (REGEX-PCRE2-FIDELITY.3.13)** the letter component is a **positive enumeration** instead of negative-lookahead guards over `any_char` (the stimuli generator is lookahead-blind, so guard-based exclusions were invisible to generation; a positive set is generation-faithful). Same accepted set per profile as before, minus the 7 escape-anchor letters (see below):

- `simple_escape_letter_strict` (always active): 35 letters — every ASCII letter EXCEPT the 7 escape-anchor spellings `A B G K Z b z` (those are anchors, never shorthand escapes — REGEX-PCRE2-FIDELITY.3.13), the six PCRE2-unsupported letters `F L U i l u` (REGEX-PCRE2-FIDELITY.3.1), the uppercase `E` (a stray `\E` is a PCRE2 zero-width end-of-quote, re-homed to `stray_end_quote_escape` / the `zero_width` piece — REGEX-PCRE2-FIDELITY.3.19, release 1.1.86), the uppercase `Q` (uppercase `\Q` is a QUOTE-OPENER, never a bare shorthand — PCRE2 quotes `\Q…\E` or `\Q…` to end-of-pattern as literal; re-homed to `quoted_literal` / `unterminated_quoted_literal` / `empty_quoted_literal` — REGEX-PCRE2-FIDELITY.3.23, release 1.1.89), and the two Unicode-property introducers `P p` (`\p`/`\P` are ALWAYS property escapes owned by `property_escape`, never bare shorthands — a bare non-category / at-EOF form like `\pA`/`\P_`/`\p` now REJECTS at the grammar layer; REGEX-PCRE2-FIDELITY.4.1, release 1.1.90). The LOWERCASE `\e` (ESC) and `\q` stay shorthands. See the [piece chapter's `zero_width`](rules-piece.md#zero_width--the-transparent-stray-e-and-empty-qe) section, the [`quoted_literal`](rules-atom.md#quoted_literal) atom, and the [`property_escape`](#property_escape) rule below.
- `simple_escape_letter_relaxed` (`@profiles: ["relaxed"]`): re-admits `\i \F \l \L \u \U`. The anchor letters, `E`/`Q`, and the property introducers `p`/`P` stay excluded in BOTH profiles.
- Digits are excluded positively (no `digit` alternative — the PGEN-RGX-0087 rule: `\<digit>` is always backref/octal/error, never shorthand); the `\o{`/`\x{` brace-form leads remain **2-char** negative lookaheads (`o`/`x` ARE shorthand letters, so only the `{`-braced octal/hex forms need excluding — a "not followed by `{`" condition has no positive spelling), while `\p`/`\P` are **whole-letter** negative lookaheads (`!"p"`/`!"P"`, REGEX-PCRE2-FIDELITY.4.1 — never shorthand letters at all).

## Profiles — strict default vs `relaxed`

**REGEX-PCRE2-FIDELITY.3.1.** PGEN's regex parser is **PCRE2-faithful by default**. The six escape letters PCRE2 does not support — `\i \F \l \L \u \U` (and the braced `\u{…}` form) — are **rejected in the default (`pcre2`) profile** and re-admitted only under the opt-out **`relaxed`** profile. This rule lives entirely in the grammar (the EBNF is the single source of truth); it was previously enforced by an out-of-band host validator.

Three escape catch-alls carry the strict/relaxed split so the six letters are rejected in **every** context (PCRE2 rejects them in all three — verified against `pcre2test` 10.47):

| Context | Rule | Default (`pcre2`) | `relaxed` |
|---|---|---|---|
| atom (`\u`) | `simple_escape` | REJECT | accept (`{kind:"shorthand",char}`) |
| char class (`[\u]`) | `class_simple_escape` | REJECT | accept |
| class range bound (`[\u-\x7f]`) | `class_range_literal_escape_letter` | REJECT | accept |
| braced (`\u{41}`) | `unicode_escape` | REJECT | accept (`{kind:"unicode",digits}`) |

Selecting the profile:

```bash
# default — strict PCRE2: \u rejected
ast_pipeline grammars/regex.ebnf --generate-stimuli ...
parseability_probe --parse regex pattern.re

# relaxed opt-out: \u / \u{41} / \i / \F / \l / \L / \U accepted
parseability_probe --parse regex pattern.re --profile relaxed
```

> **Where the default is declared (DEFAULT-PROFILE.2, 2026-07-07).** The "unspecified profile =
> strict `pcre2`" resolution is declared **in the grammar** — `grammars/regex.ebnf` carries the
> grammar-level `@default_profile: pcre2` directive, and the generated parser embeds it (a
> `DEFAULT_GRAMMAR_PROFILE` constant; the constructor starts on it, and `set_grammar_profile(None)`
> restores it). Observable behavior is unchanged from the prior engine-side default; it is now
> grammar-declared rather than engine-hard-coded, per the EBNF-single-source-of-truth doctrine.

> **Downstream note (RGX):** the stable `pgen::embedding_api` host surface currently exposes only the strict `regex_default` profile, so embedded consumers get PCRE2-faithful default behavior. A rejected `\u` (etc.) surfaces as diagnostic code `E_PARSE_FAILURE` with a machine-localizable location — match on the **code**, not the message text. Selecting `relaxed` through the embedding API is a planned follow-on.

> **Scope.** This covers exactly the six letters above. PCRE2 also rejects other unrecognized `\<letter>` escapes (e.g. `\I`, `\J`) that the default profile still accepts — the full recognized-escape whitelist is tracked separately (REGEX-PCRE2-FIDELITY.3.11).

## `hex_escape`

```ebnf
hex_escape = "x" hex_digit hex_digit?
           | "x{" brace_ws? hex_digits brace_ws? "}"
```

2 branches (the two PCRE2 syntactic forms) — both emit the SAME typed object with the
digit string joined:

```json
{ "type": "escape", "kind": "hex", "digits": <hex-digit-string> }
```

Live-verified: `\xF` → `digits:"F"`; `\xFF` → `digits:"FF"`; `\x{1F}` →
`digits:"1F"` (the braced form's whitespace and delimiters are folded away).
Consumers parse `digits` as a hex integer; the source spelling (short vs braced) is
not preserved.

## `unicode_escape`

```ebnf
@profiles: ["relaxed"]
unicode_escape = "u{" hex_digits "}"
```

Typed object `{type: "escape", kind: "unicode", digits: <hex_digits>}`.

**Profile-gated (REGEX-PCRE2-FIDELITY.3.1):** `\u{…}` is PCRE2-unsupported in the default (`pcre2`) profile (`pcre2test` 10.47 → error 137; only `PCRE2_ALT_BSUX` gives it meaning), so `unicode_escape` is tagged `@profiles: ["relaxed"]` and is **only active under the `relaxed` profile**. In the default profile it backtracks, and `\u{41}` is rejected (the strict `simple_escape` excludes `\u`, so the whole pattern fails to parse). Under `relaxed`, `\u{1F600}` yields `{type:"escape",kind:"unicode",digits:"1F600"}`.

## `octal_escape`

```ebnf
octal_escape = "o{" brace_ws? octal_digits brace_ws? "}"
             | octal_digit octal_digit? octal_digit?
```

2 branches (braced and bare 1-3-digit) — both emit the SAME typed object with the
digit string joined and leading zeros preserved:

```json
{ "type": "escape", "kind": "octal", "digits": <octal-digit-string> }
```

Live-verified: `\o{101}` → `digits:"101"`; `\0` → `digits:"0"`. (The `octal_digits`
helper folds into the joined `digits` string — no per-char walking.)

## `control_escape`

```ebnf
control_escape = "c" any_char
```

Typed object:

```json
{ "type": "escape", "kind": "control", "char": <char> }
```

Live-verified: `\cA` → `{"type":"escape","kind":"control","char":"A"}`.

## `property_escape`

```ebnf
property_escape = "p{" prop_name "}"
                | "P{" prop_name "}"
                | "p" short_prop_letter
                | "P" short_prop_letter
```

4 branches (braced vs short-form × lower vs upper `p`) — all four emit the SAME typed
object; the upper-case `P` surfaces as `negated: true`:

```json
{ "type": "escape", "kind": "property", "name": <string>, "negated": <bool> }
```

Live-verified: `\p{Lu}` → `{"kind":"property","name":"Lu","negated":false}`; `\pL` →
`name:"L"`; `\PN` → `{"name":"N","negated":true}`. `name` is the clean joined
property name (braced) or the single category letter (short form).

**Bare short-form validity (REGEX-PCRE2-FIDELITY.4.1, release 1.1.90).** In PCRE2 a bare (un-braced) `\p`/`\P` MUST be one of the 14 one-letter general categories in [`short_prop_letter`](#short_prop_letter). `\p`/`\P` are ALWAYS property introducers — they are excluded from `simple_escape` (dropped from `simple_escape_letter_strict`; the `!"p"`/`!"P"` whole-letter lookaheads on `simple_escape` / `class_simple_escape`), so a bad-letter (`\pA`, `\P_`), at-EOF (`\p`, `\P`), or class-context (`[\pA]`) form has **no fully-consuming parse and hard-REJECTS** rather than decomposing into `\p` + a literal. This is grammar-owned (the acceptance rule migrated out of the out-of-band `regex_compile_validation.rs::find_invalid_property_escape` — the EBNF is now the single source of truth). Behavior-neutral for downstream: these forms were rejected by the validator before and by the grammar now.

## `short_prop_letter`

```ebnf
short_prop_letter = 'C' | 'L' | 'M' | 'N' | 'P' | 'S' | 'Z'
                  | 'c' | 'l' | 'm' | 'n' | 'p' | 's' | 'z'
```

14-way Or. `Terminal(<letter>)`.

## `prop_name`

```ebnf
prop_name = prop_name_chars+
```

Quantified-`+` of `prop_name_chars`. Concatenate to recover the name string.

## `prop_name_chars`

```ebnf
prop_name_chars = letter | digit | whitespace | '_' | ':' | '-' | '=' | '&' | '^'
```

9-way Or, each emitting a Terminal of the matched char.

## Walking a `\d+` example

Exact probe output for `\d+`:

```json
{
  "atom": { "type": "escape", "kind": "shorthand", "char": "d" },
  "quantifier": { "type": "quantifier", "min": 1, "max": null, "greediness": [] },
  "type": "piece"
}
```

Extraction is a single field read:

```rust
fn extract_shorthand_escape_char(atom: &Value) -> Option<&str> {
    let obj = atom.as_object()?;
    if obj.get("type")?.as_str()? != "escape" { return None; }
    if obj.get("kind")?.as_str()? != "shorthand" { return None; }
    obj.get("char")?.as_str()
}
```

## Walking a `\x{1F}` example

Exact probe output:

```json
{
  "atom": { "type": "escape", "kind": "hex", "digits": "1F" },
  "quantifier": [],
  "type": "piece"
}
```

`digits` is the clean joined hex string — `u32::from_str_radix(digits, 16)` recovers
the code point. The same one-object read applies to every escape kind (`octal`
digits, `control` char, `property` name/negated, `unicode` digits under `relaxed`).

## Historical note

In the pre-slice-14 era the escape subtree emitted raw shapes — the 2-element
`["\\", <escape_unit chain>]` with deeply-nested un-annotated wrappers that consumers
descended level by level. Those shapes no longer exist in any released artifact; every
escape form now emits the flat typed `{type: "escape", kind, ...}` objects documented
above (see the [Changelog Index](changelog-index.md) for the slice-by-slice record).
