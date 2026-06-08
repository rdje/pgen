# Grammar and Scope

The full grammar is `grammars/json.ebnf` (nine rules). As of the `EXTERNAL-CORPUS.2a` upgrade its lexical
terminals track RFC 8259 / ECMA-404, validated against the recognized JSONTestSuite corpus (every
must-accept file passes: **95/95**).

## What it accepts (the rules)

| Rule | Production (abridged) | Notes |
| --- | --- | --- |
| `json` | `value` | top entry; wraps in `{type:"json", value}` |
| `value` | `object \| array \| string \| number \| true \| false \| null` | the seven JSON value forms |
| `object` | `{}`  or  `{ members }` | |
| `members` | `pair ( , pair )*` | comma-separated pairs |
| `pair` | `string : value` | |
| `array` | `[]`  or  `[ elements ]` | |
| `elements` | `value ( , value )*` | comma-separated values |
| `string` | `/[ \t\n\r]*"(\\(["\\\/bfnrt]\|u[0-9a-fA-F]{4})\|[^"\\\x00-\x1f])*"[ \t\n\r]*/` | the RFC escape set; forbids raw control chars / lone `\` |
| `number` | `/[ \t\n\r]*-?(0\|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?[ \t\n\r]*/` | sign, `0\|[1-9]…`, fraction, exponent |

`true` / `false` / `null` are matched as whitespace-padded keywords (whitespace = the exact JSON set
`[ \t\n\r]`) and lowered to `{type:"boolean", value:true/false}` / `{type:"null"}`.

## RFC-8259 lexical conformance (closed by `EXTERNAL-CORPUS.2a`)

The original simplified grammar diverged on number/string/whitespace lexing; the upgrade closed those:

- **Number exponents** — `1e10`, `0e1`, `2.5E-3` now parse (`([eE][+-]?[0-9]+)?`).
- **String escapes** — only `\" \\ \/ \b \f \n \r \t \uXXXX` are valid escapes; **raw control characters
  (`\x00-\x1f`) and malformed/lone escapes are now rejected**.
- **Leading zeros** — `01` / `-01` are rejected (`0 | [1-9][0-9]*`).
- **Whitespace** — exactly `[ \t\n\r]` (form-feed / vertical-tab no longer slip through).

## Known residuals (each its own follow-up leaf)

1. **Trailing content after a complete value is not yet rejected** (`EXTERNAL-CORPUS.2c`). `{"a":"b"}//`,
   `{"a":"b"}#`, `{"a":/*comment*/"b"}` etc. are still accepted — the value parses and the trailing bytes
   are left unconsumed without the parse failing. The fix is strict end-of-input enforcement at `json`.
2. **No recursion/stack guard** (`EXTERNAL-CORPUS.2b`). Adversarially deep nesting (hundreds–thousands of
   `[`) **crashes** the recursive-descent parser instead of being rejected. (The regex family solved this
   class with a dedicated worker stack; the json parser needs an equivalent.)

These two are the only remaining gaps versus a fully hardened standards parser; the
[External-Corpus Characterization](external-corpus-characterization.md) measures them precisely.
