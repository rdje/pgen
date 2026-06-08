# Grammar and Scope (a simplified subset)

The full grammar is `grammars/json.ebnf` (nine rules). It models the **shape** of JSON — objects, arrays,
strings, numbers, the three keywords — but uses coarse regex terminals for the lexical layer, so it is a
**subset/superset** of standard JSON rather than a faithful implementation.

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
| `string` | `/\s*"[^"]*"\s*/` | **any run of non-`"` bytes** between quotes |
| `number` | `/\s*-?[0-9]+(\.[0-9]+)?\s*/` | optional sign, integer, optional fraction |

`true` / `false` / `null` are matched as whitespace-padded keywords and lowered to
`{type:"boolean", value:true/false}` / `{type:"null"}`.

## What it does NOT do (the gaps vs RFC 8259 / ECMA-404)

These are **deliberate simplifications**, each confirmed by the external corpus
([characterization](external-corpus-characterization.md)):

1. **No number exponent.** `1e10`, `0e1`, `2.5E-3` are **rejected** (the standard requires
   `([eE][+-]?[0-9]+)?`).
2. **No string escapes.** `string` is `"[^"]*"`, so:
   - valid escaped strings like `"a\"b"` or `"\n"` are **rejected** (the `[^"]*` stops at, or cannot
     express, the escape);
   - invalid strings — raw control characters, bad `\`-escapes, malformed `\uXXXX` surrogates — are
     **wrongly accepted** (any non-`"` byte passes). The standard allows only
     `\" \\ \/ \b \f \n \r \t \uXXXX` and forbids raw control characters.
3. **Leading zeros allowed.** The integer part is `[0-9]+`, so `01` / `-01` are **wrongly accepted**
   (the standard requires `0 | [1-9][0-9]*`).
4. **Loose trailing/whitespace.** The regex `\s` class includes form-feed (which JSON whitespace does
   not), and trailing-content handling is looser than a strict full-input requirement, so a few
   trailing-garbage / trailing-comment documents are **wrongly accepted**.
5. **No recursion/stack guard.** Deeply nested input (hundreds–thousands of `[`) **crashes** the
   recursive-descent parser instead of being rejected. (The regex family solved this class with a
   dedicated worker stack; the json parser has no equivalent yet.)

The planned, evidence-gated **`EXTERNAL-CORPUS.2a`** upgrade closes #1–#4 (and **`.2b`** addresses #5),
using the `json_corpus_bundle/` characterization as the acceptance metric.
