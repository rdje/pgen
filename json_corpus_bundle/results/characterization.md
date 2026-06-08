# PGEN `json` parser — external-corpus characterization

- Corpus: **JSONTestSuite** (`test_parsing/`, pinned commit `1ef36fa0`, MIT) — 318 files.
- Parser: PGEN generated `json` parser from `grammars/json.ebnf` (`make -C rust focus_json`).
- Runner: `json_corpus_bundle/scripts/run_json_corpus.sh` (deterministic; the corpus's own
  `y_/n_/i_` prefixes are the fix-independent oracle).
- Measured: 2026-06-08 (`PGEN-EXTERNAL-CORPUS-0002`, leaf `EXTERNAL-CORPUS.2`).

## Headline result — does `json.ebnf` match the official JSON standard? **No.**

`grammars/json.ebnf` is a deliberately **simplified subset** of JSON (RFC 8259 / ECMA-404), and the
recognized corpus proves it precisely:

| Class | Files | Parser result | Conformance |
| --- | --- | --- | --- |
| `y_` (MUST accept) | 95 | **81 accept**, 14 reject | **81 / 95** |
| `n_` (MUST reject) | 188 | 158 reject, **28 wrongly accept**, 2 crash | **158 / 188** |
| `i_` (impl-defined) | 35 | 13 accept, 21 reject, 1 crash | n/a (either valid) |
| **crashes (any class)** | — | **3** abort (stack overflow) | robustness defect |

So the simplified grammar is **right on the common JSON core** (81/95 valid documents accepted,
158/188 invalid documents rejected) but **diverges on the standard's stricter number/string lexing and on
robustness**. Every divergence below is a **grammar-scope limit, not a parser engine bug** — the PEG
engine faithfully implements `json.ebnf`; `json.ebnf` simply is not the full standard.

## Root-caused divergences (all traceable to the simplified grammar)

### 1. No number **exponent** — `number := /\s*-?[0-9]+(\.[0-9]+)?\s*/`
Rejects all 11 exponent forms the standard requires + 2 extreme-number cases (13 of the 14 `y_` rejects):
`y_number_0e1`, `y_number_0e+1`, `y_number_int_with_exp`, `y_number_real_exponent`,
`y_number_real_capital_e[_pos_exp/_neg_exp]`, `y_number_real_[pos/neg]_exp`,
`y_number_real_fraction_exponent`, `y_number.json`, `y_object_extreme_numbers`. RFC 8259 number =
`-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?`.

### 2. No string **escapes** — `string := /\s*"[^"]*"\s*/`
`[^"]*` accepts any byte except `"` and understands no `\`-escape, so it is BOTH too narrow and too wide:
- too narrow → rejects valid escaped strings: `y_string_allowed_escapes`, `y_string_backslash_doublequotes`.
- too wide → wrongly accepts 20 invalid `n_string_*`: unescaped control chars (`n_string_unescaped_tab`,
  `_newline`, `_ctrl_char`, `n_string_escaped_ctrl_char_tab`), bad/incomplete escapes
  (`n_string_escape_x`, `_invalid_backslash_esc`, `_incomplete_escape`, `_escaped_backslash_bad`,
  `_backslash_00`, `_escaped_emoji`), and incomplete/invalid surrogate `\u` pairs
  (`n_string_1_surrogate_then_escape*`, `n_string_incomplete_surrogate*`,
  `n_string_invalid_unicode_escape`, `n_string_unicode_CapitalU`). RFC 8259 string allows only
  `\" \\ \/ \b \f \n \r \t \uXXXX` and forbids raw control characters.

### 3. **Leading zeros** allowed — integer part is `[0-9]+`, not `0|[1-9][0-9]*`
Wrongly accepts `n_number_with_leading_zero`, `n_number_-01`, `n_number_neg_int_starting_with_zero`.

### 4. **Trailing garbage / comments** tolerated
Wrongly accepts `n_object_trailing_comment`, `n_object_trailing_comment_slash_open`,
`n_object_with_trailing_garbage`, `n_structure_object_with_comment`, `n_structure_trailing_#`,
`n_structure_whitespace_formfeed` (form-feed is in the regex `\s` class; JSON whitespace is only
space/tab/newline/CR). The grammar's `/\s*…\s*/`-padded terminals + value-level trailing handling are
looser than the standard's exact whitespace set.

### 5. **No recursion/stack guard** — 3 deep-nesting files **abort** (stack overflow)
`i_structure_500_nested_arrays`, `n_structure_100000_opening_arrays`, `n_structure_open_array_object`
crash the recursive-descent parser. The regex family already solved this class with a dedicated worker
stack (RGX-0085); the json parser has none. This is a real **robustness** defect (a parser must reject —
not crash on — adversarial input), and is exactly the kind of finding an external corpus surfaces and the
internal stimuli generator, by construction, never would.

## Why this is the value of the external corpus

The stimuli generator can only manufacture strings the grammar already describes, so it can never reveal
gaps #1–#5 — they are gaps between the *grammar* and the *real language*. JSONTestSuite, authored
independently of `json.ebnf`, is the oracle that exposes them. This is the entire point of
[[project_external_corpus_doctrine]]: internal (generator) + external (corpus), both required.

## Recommended follow-ups (evidence-gated, each its own task-tree leaf)

- **Upgrade `json.ebnf` toward RFC 8259** (own slice): exponent in `number`; the `0|[1-9][0-9]*` integer
  rule; a proper escaped-`string` production (`\" \\ \/ \b \f \n \r \t \uXXXX`, forbid raw controls); exact
  JSON whitespace; strict full-input/no-trailing. Re-run this characterization as the acceptance metric
  (target: `y_` 95/95, `n_` non-crash 186/186, the 2 deep-nesting `n_` rejected not crashed).
- **Recursion/stack guard for the json parser** (robustness) — the 3 aborts (cf. RGX-0085's dedicated
  worker stack / a depth bound).

Until then the live tracker must describe `json` honestly as a *simplified-subset* parser characterized
against the recognized corpus — not a conforming JSON parser.
