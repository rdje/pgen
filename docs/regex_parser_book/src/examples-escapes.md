# Examples: Escapes

Concrete probe outputs for PCRE2 escape sequences. As of slice 17 (post-1.1.47), the `escape` rule is a transparent wrapper (`-> $2`) and **all 7** `escape_unit` sub-rules emit typed `{type:"escape", kind:<form>, ...}` objects directly: simple, single_byte, control (slice 14), hex, unicode (slice 15), octal (slice 16), property (slice 17). The escape subtree is now fully typed.

## Shorthand escapes — `\d`, `\w`, `\s`, `\.`, `\\`, etc.

These all flow through `simple_escape` (the catch-all branch of `escape_unit`):

```ebnf
simple_escape  = !"o{" !"x{" !"p{" !"P{" any_char
                  -> {type: "escape", kind: "shorthand", char: $5}
```

The `!"o{"` / `!"x{"` / `!"p{"` / `!"P{"` negative-lookahead guards (PGEN-RGX-0079 fix) prevent `simple_escape` from absorbing the prefix of an invalid braced escape (e.g. `\o{1239}` with a non-octal digit, `\x{12g}`, `\p{!}`). Without the guards `\o{1239}` would have parsed as `simple_escape` consuming only the `o` and the rest as separate atoms — a silent misparse. Now invalid braced forms reject as a whole.

For `\d`:

```json
{
  "atom": {"type": "escape", "kind": "shorthand", "char": "d"},
  ...
}
```

For `\w`: same shape, `char:"w"`. For `\s`: `char:"s"`. For `\.`: `char:"."`. For `\\`: `char:"\\"` (the actual backslash char). Any letter/symbol after `\` produces a `kind:"shorthand"` typed object with the char in the `char` field.

(Pre-slice-14 the same input emitted `["\\", [[[[[ "d" ]]]]]]` — a 5-level un-annotated chain wrapping the matched char Terminal. Consumers walking that chain via `to_json_value()` saw deeply-nested arrays. Post-slice the typed shape is a single field read.)

## Escaped metacharacters — `\.`, `\\`, `\(`, `\)`, etc.

Same `kind:"shorthand"` shape as shorthand classes — `simple_escape` matches any char after `\`:

For `\.`:

```json
{
  "atom": {"type": "escape", "kind": "shorthand", "char": "."},
  ],
  ...
}
```

For `\\` (escaped backslash): `char:"\\"` (the literal backslash char).

## Hex escape (1-2 digit form) — `\xFF`

```json
{
  "atom": {"type": "escape", "kind": "hex", "digits": "FF"},
  ...
}
```

For `\xF` (single digit): `digits:"F"`. The `hex_escape_short_payload` regex literal `/([0-9A-Fa-f]{1,2})/` accepts 1 or 2 hex digits.

## Hex escape (braced form) — `\x{1F}`

```json
{
  "atom": {"type": "escape", "kind": "hex", "digits": "1F"},
  ...
}
```

The braced form accepts any number of hex digits via `hex_digits = /([0-9A-Fa-f]+)/`, with optional whitespace inside the braces (PCRE2's `brace_ws` allowance).

## Hex escape (long codepoint) — `\x{1F600}` (😀)

```json
{
  "atom": {"type": "escape", "kind": "hex", "digits": "1F600"},
  ...
}
```

Consumers parse the codepoint with `usize::from_str_radix(obj.digits, 16)`.

## `digits` is a string, not an int

The hex/unicode `digits` field carries the raw hex string. Decode to a numeric codepoint yourself:

```rust
let cp = usize::from_str_radix(obj["digits"].as_str().unwrap(), 16).unwrap();
```

PGEN's `@transform` machinery is currently hard-coded to `str::parse::<TYPE>().unwrap_or(DEFAULT)`-style and can't express `from_str_radix(s, 16)`. Extending it is a separate codegen-feature slice.

## Unicode escape — `\u{1F600}`

```json
{
  "atom": {"type": "escape", "kind": "unicode", "digits": "1F600"},
  ...
}
```

**Validator note:** PGEN's host-side compile validator currently rejects `\u{...}` escapes ("unsupported regex escape `\u`"). The annotation IS in place and correct when the validator allows the escape through; for inputs the validator rejects, no AST is produced. That validator behavior is pre-existing and tracked separately from the slice 15 shape work.

## Octal escape (braced) — `\o{777}`

```json
{
  "atom": {"type": "escape", "kind": "octal", "digits": "777"},
  ...
}
```

For `\o{7777}` (longer): `digits:"7777"`. For `\o{1}`: `digits:"1"`.

## Octal escape (bare 1-3 digit) — `\377`

**At atom-level**, bare `\N…` is disambiguated **at parse time** per
PCRE2's compile-time rule (`pcre2pattern(3)` BACKREFERENCES), which is
keyed on the numeric **value** `N` (longest digit run, no leading
zero — `\0…` is always octal via `octal_escape`):

- **`N` < 10 (a single digit `\1`…`\9`): ALWAYS a numeric back
  reference.** The "groups opened so far" rule never applies to a
  single digit (the report states this verbatim: *"Here `\1` is a
  single digit, so the 'up to that point' rule never applies"*). This
  is unchanged pre-existing behavior.
- **`N` ≥ 10 (two+ digits `\10`, `\377`, …): a back reference iff
  there are ≥ N capturing groups opened *up to that source
  position*; otherwise an octal escape** (re-split through `escape` →
  `octal_escape`/`simple_escape`).

This fixes downstream report `PGEN-RGX-0084`: bare two+-digit `\NN…`
was previously an *unconditional* numeric backreference regardless of
group count.

> **Superseded note (was true before this fix):** earlier editions of
> this book stated atom-level bare `\NNN` is always
> `{"type":"backreference","kind":"numeric","index":NNN}` and that
> "PCRE2 disambiguation … PEG cannot express directly … left to
> consumers via post-parse semantic analysis." That is **no longer
> true and no longer accurate**: PGEN expresses the disambiguation
> directly in the grammar via its always-on semantic-annotation
> mechanism (a parse-time decision, **not** a post-parse consumer
> concern). Single-digit `\1`…`\9` behavior is unchanged.

So `\377` (two+ digits, N=377) with fewer than 377 capturing groups
opened so far is an octal escape, **not** backref index 377:

```json
{
  "atom": {"type": "escape", "kind": "octal", "digits": "377"},
  ...
}
```

…while a single-digit `\1`, or a two+-digit `\10` with ≥ 10 groups
opened, is a back reference (shape unchanged):

```json
// (a)\1   →   the \1 atom (single digit → always backref):
{"type": "backreference", "kind": "numeric", "index": 1}
```

Worked family (verified end-to-end):

| pattern | `\N…` atom | why |
| --- | --- | --- |
| `\1` (0 groups) | `{backreference, numeric, index:1}` | single digit → always backref (N<10) |
| `\8` (0 groups) | `{backreference, numeric, index:8}` | single digit → always backref (N<10) |
| `(a)\1` | `{backreference, numeric, index:1}` | single digit (N<10) |
| `(a)\2` | `{backreference, numeric, index:2}` | single digit → always backref even with 1 group |
| `(?:abc)\1` | `{backreference, numeric, index:1}` | single digit (N<10) — non-capturing irrelevant |
| `(\1)` | `{backreference, numeric, index:1}` | single digit (N<10) |
| `(?<n>x)\1` | `{backreference, numeric, index:1}` | single digit (N<10) |
| `\377` (0 groups) | `{escape, octal, digits:"377"}` | N≥10, 0 < 377 |
| `(a)\12` | `{escape, octal, digits:"12"}` | N≥10, only 1 group < 12 |
| `(a)(b)\10` | `{escape, octal, digits:"10"}` | N≥10, only 2 groups < 10 |
| `(a)…(j)\10` (10 groups) | `{backreference, numeric, index:10}` | N≥10, 10 groups ≥ 10 |
| `(?<a>…)…(?<j>…)\10` (10 named) | `{backreference, numeric, index:10}` | PCRE2 numbers named groups too |
| `()()()()()()()()()(?:(?(10)\10a\|b)(X\|Y))+` | `\10` → `{escape, octal, digits:"10"}` | N≥10, only 9 groups precede `\10`; group 10 opens *after* (the reported reproducer) |

**Mechanism (grammar-level, parser-agnostic):** the single-vs-multi
digit split *is* the N<10 vs N≥10 boundary. `numeric_backreference_single`
(`\` + one `[1-9]`) is **ungated** — always a numeric backref.
`numeric_backreference` (`\` + `[1-9][0-9]+`, value always ≥ 10) is
**gated**: every capturing-group **open** marker (`capture_open` for
`(…)`, plus the named / python-named open markers) carries
`@emit_fact: {kind: regex_capture_group, …}`, recording one generic
fact the instant the group's `(` is consumed (emit-at-OPEN, so a
self/forward ref like `(\1)` sees its own group). `numeric_backreference`
carries `@predicate: {name: fact_count_at_least, args:
[regex_capture_group, $index], phase: post}` — a *generic* engine
predicate (a strict generalization of `has_fact`) true iff the
running `regex_capture_group` fact count ≥ the matched digit value.
`$index` reads `numeric_backreference`'s own `->` output field (the
`SEMREF-SHAPED` shaped-structure semantic-ref resolution). When false
the rule backtracks and `atom` falls through to `escape` →
`octal_escape`/`simple_escape` — PCRE2's octal/literal fallback,
automatic via PEG ordered choice (`numeric_backreference` is tried
before `numeric_backreference_single` so the longest digit run wins).
Non-capturing / lookaround / atomic / conditional / verb groups
deliberately do **not** emit, so they are correctly not counted. The
"capture group" knowledge lives only in `regex.ebnf`; the engine
predicate is grammar-agnostic.

**Inside character classes**, the `class_range_escape_unit` path reaches the bare-octal branch and emits the typed shape:

```json
{
  "atom": [
    "[",
    [],
    [],
    [
      {"type": "escape", "kind": "octal", "digits": "377"}
    ],
    "]"
  ],
  ...
}
```

That is `[\377]` parses with the bare-octal escape typed inside the class body.

## PGEN-RGX-0087: `[89]`-leading multi-digit escape hard-rejects (PCRE2-faithful)

This is a **family-linked residual** of the `PGEN-RGX-0084` fix above
(`REGEX-0083` stays closed and correct — this is a separate sub-family
it did not cover, **not a reopen**). The disambiguation above degrades
an `N≥10` run that is not a valid full back-reference to an
octal/literal escape. That degrade is only correct when the digit run
can form an octal escape — i.e. its **first digit is `0`–`7`**. The
digits **`8` and `9` are not octal digits**, so an `[89]`-leading
multi-digit run (`\8N`/`\9N`, `N≥10`) that is *not* a valid full
back-reference is **neither** a valid back-reference **nor** a valid
octal escape. PCRE2 (authoritative oracle: `pcre2test` 10.47, the
version family RGX vendors) therefore **rejects such a pattern at
compile** (error 115, *reference to non-existent subpattern*) — it
does **not** silently re-split it into a shorter escape plus literals.

Before this fix PGEN re-split it: for `((((((((x))))))))\81` (8
groups) the gated multi-digit `numeric_backreference` predicate failed
(`8 < 81`), the PEG backtracked, and the **ungated**
`numeric_backreference_single` matched the lone leading `8` as a
*valid* single back-reference (group 8 exists) leaving `1` literal —
so PGEN accepted a pattern PCRE2 rejects. (Even with the single-digit
rule guarded, the `simple_escape` *catch-all* `\<any-char>` would
still consume `\8` as a `{kind:"shorthand"}` escape — so the fix needs
**two** guards.)

The fix adds two negative-lookahead guards (the proven
`PGEN-RGX-0079` `!"string"` idiom) in `grammars/regex.ebnf`:
`numeric_backreference_single` only matches a **complete** one-digit
run, and `simple_escape` **never** matches `\<digit>` (a backslash
followed by a digit is *always* a back-reference / octal escape /
compile error in PCRE2 — never a shorthand literal). `[1-7]`-leading
runs still degrade to `octal_escape` (tried before `simple_escape`),
so the `REGEX-0083` behaviour is **byte-identical**; `[89]`-leading
runs can't octal, can't single-degrade, can't shorthand-degrade ⇒
`atom` exhausts ⇒ a clean hard parse **REJECT** (`E_PARSE_FAILURE`),
exactly matching PCRE2.

Worked family (verified end-to-end against the PCRE2 10.47 oracle —
expecteds derived from the spec, not the fix):

| pattern | groups | result | why |
| --- | --- | --- | --- |
| `((((((((x))))))))\81` | 8 | **REJECT** | `[89]`-led, `N=81 > 8` groups, `8` not octal ⇒ PCRE2 error 115 (was: `\8` backref + `1` lit) |
| `((((((((x))))))))\82` | 8 | **REJECT** | same family |
| `((((((((x))))))))\91` | 8 | **REJECT** | `\9` family; group 9 missing |
| `\89` | 0 | **REJECT** | `[89]`-led, `N=89 > 0`; PCRE2 10.47 = error 115 (**not** literal "89") |
| `\80` (0) / `(x)\81` (1) | 0 / 1 | REJECT | unchanged |
| `\199` | 0 | `{escape, octal, digits:"1"}` + `9` `9` | `[1-7]`-led: `\1` = octal `0o1`, then literal "99" (PCRE2-faithful; was wrongly `{backreference,index:1}`) |
| `(((((((((x)))))))))\10` | 9 | `{escape, octal, digits:"10"}` | `[1-7]`-led octal — **byte-identical** to `REGEX-0083` |
| `((((((((x))))))))\8` | 8 | `{backreference, numeric, index:8}` | single digit `N<10` — Non-Goal, **unchanged** |
| `(x)\1` | 1 | `{backreference, numeric, index:1}` | single digit — **unchanged** |
| `\012` / `\07` | 0 | `{escape, octal, …}` | `\0`-led octal — **unchanged** |

Accept-set tightening + one corrected classification (`\199`@0-groups);
no new AST `kind`/shape ⇒ **AST-dump schema stays `1`** (release-only
bump, same category as `REGEX-0083`/`REGEX-0084`). Bug ledger:
`REGEX-0086` (downstream `PGEN-RGX-0087`).

> **RGX maintainer note.** The `#[ignore]`d RGX unit test
> `parser_multi_digit_non_octal_backref_becomes_literal` asserts
> `\89`@0-groups compiles as the literal `"89"`. That is **wrong vs
> PCRE2 10.47** (which errors 115). Re-enable the test expecting a
> clean parse **REJECT**, not "literal 89". Its second assertion
> (`\199` → `\x01` + "99") is spec-correct and now matches PGEN's
> `{escape, octal, digits:"1"}` + "99".

### FIX2 (release 1.1.79): the hard-reject is scoped to NON-character-class context

The `[89]`-leading multi-digit hard-reject above is a **pattern-body
(atom)** concern: it is about `\NN` being neither a valid
back-reference nor a valid octal escape. **A character class has no
back-references** — inside `[...]`, `\8`/`\9`/`\<digit>` are
octal/literal characters and PCRE2 (oracle: `pcre2test` 10.47)
**ACCEPTs** them. Release 1.1.78 implemented the hard-reject by
guarding `simple_escape`, but a single class-member escape (`[\8]`,
`[A\8B\9C]`) is reached via `class_escape = escape`, which reused
that same digit-guarded `simple_escape` — so 1.1.78 wrongly rejected
class-context `\8`/`\9`. Release 1.1.79 gives `class_escape` its own
`class_escape_unit` with an **unguarded** `class_simple_escape` (the
pre-1.1.78 form), mirroring the existing `class_range_escape_unit`
precedent. Class context is now byte-identical to pre-1.1.78 (same
`{type:"escape", kind:"shorthand", char:…}` shape; **schema stays
`1`**); the non-class hard-reject is unchanged.

| pattern | result | why |
| --- | --- | --- |
| `[\8]` `[\9]` `^[A\8B\9C]+$` `[\88]` `[\89]` | ACCEPT | inside `[...]`: octal/literal, no back-references (PCRE2 10.47) |
| `[\8-\9]` `[\377]` | ACCEPT | class-range / octal, unchanged (now consistent with `[\8]`) |
| `((((((((x))))))))\81` `\82` `\91` ; `\89`@0g ; `\80`@0g ; `(x)\81`@1g | REJECT | **non-class** `[89]`-leading hard-reject — unchanged from 1.1.78 |
| `\199`@0g → `{escape,octal,digits:"1"}`+"99" ; `\10`@9g → octal | ACCEPT | `[1-7]`-led octal-degrade — unchanged from 1.1.78 |

### FIX2.3 (release 1.1.80): octal `>\377` overflow now rejects — `PGEN-RGX-0087` CLOSED

PCRE2 (oracle: `pcre2test` 10.47) limits the bare `\ddd` octal escape
to value **≤ 0o377 (255)** in 8-bit non-UTF mode; a `>0o377` run is a
**hard compile error** (error 151) in **both** pattern-body **and**
`[...]` class context, and PCRE2 does **not** truncate the offending
3-octal-digit run to a shorter one. PGEN's
`octal_escape_short_payload` `/([0-7]{1,3})/` had no range check, so
`\6666666666` (testinput9:287 `(?i:A{1,}\6666666666)`), `\400`,
`\666`, `\777`, `\7777`, and the class forms `[\666]`/`[\400]` were
wrongly ACCEPTed. Release 1.1.80 fixes it grammar-only:
`octal_escape_short_payload` is split so a 3-octal-digit run is valid
only if its value ≤ 0o377 (first digit `[0-3]`) and a 1-2-digit run
only if it is octal-**complete** (the proven `!"0"…!"7"`
negative-lookahead idiom); an overflow triple matches neither ⇒
`octal_escape` fails ⇒ hard reject. FIX2.1's `class_simple_escape`
gained `!"0"…!"7"` octal-digit guards (not `8`/`9`) so an
octal-overflow `\<octal-digit>` is not class-shorthand-rescued, while
`\8`/`\9` stay (FIX2.1 preserved).

| pattern | result | why |
| --- | --- | --- |
| `\400` `\666` `\777` `\7777` `(?i:A{1,}\6666666666)` | REJECT | octal `>0o377` — PCRE2 10.47 err 151 |
| `[\666]` `[\400]` | REJECT | octal `>0o377` rejects **inside `[...]`** too |
| `\377` `\3777`(=`\377`+lit`7`) `\10` `\012` `\07` `\0` `\77` `\199`@0g | accept | octal ≤ 0o377 — **byte-identical** (RGX-0084/RGX-0087) |
| `[\377]` `[\012]` `[\0]` `[\8]` `[\9]` `[\88]` | accept | class octal/literal — **byte-identical** (RGX-0084/FIX2.1) |

Empirical `--parse-dump-ast-pretty` proof confirmed the entire
RGX-0084 octal family / RGX-0087 / FIX2.1 set is **byte-identical**
pre vs post (only the previously-wrongly-accepted overflow set now
rejects); no new shape ⇒ **schema stays `1`**. With this,
**`PGEN-RGX-0087` is fully resolved & closed** (all FIX2 sub-leaves
done; RGX PCRE2 differential ratchet reaches the report's full target
**12,807/3**). The braced `\o{...}` overflow (PCRE2 err 134, a
distinct production never reported) is out of scope — `RGX-0079`
owns `\o{...}`.

## `digits` is a string for octal too

Same convention as hex/unicode — consumers parse with `usize::from_str_radix(obj.digits, 8)`.

## Control escape — `\cA`

```json
{
  "atom": {"type": "escape", "kind": "control", "char": "A"},
  ...
}
```

Typed `{type:"escape", kind:"control", char:<C>}` — the `c` prefix is dropped, the matched control letter is in the `char` field. For `\cZ`: `char:"Z"`. For `\cz`: `char:"z"` (case-sensitive).

## Property escape (braced) — `\p{Lu}`

```json
{
  "atom": {"type": "escape", "kind": "property", "name": "Lu", "negated": false},
  ...
}
```

For `\p{Letter}` (long name): `name:"Letter"`. For `\P{Nd}` (negated): `name:"Nd", negated:true`. The `prop_name` regex literal admits the full PCRE2 property-identifier alphabet (letters, digits, whitespace, `_`, `:`, `-`, `=`, `&`, `^`).

## Short property escape — `\pL`, `\PN`

```json
{
  "atom": {"type": "escape", "kind": "property", "name": "L", "negated": false},
  ...
}
```

For `\PN` (negated, short): `name:"N", negated:true`. For `\pZ`: `name:"Z", negated:false`. The `short_prop_letter` regex literal admits `[CLMNPSZclmnpsz]` — the standard PCRE2 single-letter property shorthands.

## `negated` is a real boolean

Both braced and short property forms emit `negated` as a literal `true`/`false` boolean. No need for the consumer to inspect the leading-token shape (`"p"` vs `"P"`) or the rule branch index — read `obj.negated` directly.

## Single-byte escape — `\C`

```json
{
  "atom": {"type": "escape", "kind": "single_byte"},
  ...
}
```

PCRE2's `\C` matches one code unit. Typed `{type:"escape", kind:"single_byte"}` — no `char` field since the char is fixed (always uppercase `C`).

## Identifying escape kind from the AST shape

The `escape_unit` is an Or with 7 branches. As of slice 17, **all 7** branches emit typed `{type:"escape", kind:<form>, ...}` objects directly. Consumers dispatch on `kind` — no structural fallback needed.

```rust
fn classify_escape(escape_atom: &Value) -> Option<EscapeKind> {
    let obj = escape_atom.as_object()?;
    if obj.get("type").and_then(|v| v.as_str()) != Some("escape") {
        return None;
    }
    match obj.get("kind").and_then(|v| v.as_str())? {
        "shorthand"   => Some(EscapeKind::Simple),
        "single_byte" => Some(EscapeKind::SingleByte),
        "control"     => Some(EscapeKind::Control),
        "hex"         => Some(EscapeKind::Hex),
        "unicode"     => Some(EscapeKind::Unicode),
        "octal"       => Some(EscapeKind::Octal),
        "property"    => Some(EscapeKind::Property),
        _             => None,
    }
}
```

For property escapes, the consumer reads `obj.name` (string) and `obj.negated` (bool) directly. For hex/unicode/octal, `obj.digits` (string) is parsed via `usize::from_str_radix(digits, <base>)`. For control / simple, `obj.char` (string) carries the matched character.

## Future direction

Escape subtree campaign **closed** — all 7 `escape_unit` branches typed. The next atom-subtree slice picks one of the still-untyped atom alternatives (literal, whitespace_literal, dot, quoted_literal, char_class outer, group/conditional/lookaround/etc.).
